//! Default-off, application-only FEC recovery experiments. No wire state is owned here.
//! The receive owner polls the mode file; feedback only reports existing NACK operations.
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const BASE_WAIT: Duration = Duration::from_millis(52);
const HARD_WAIT: Duration = Duration::from_millis(69);
const NACK_MARGIN: Duration = Duration::from_millis(4);
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const EVENT_CAP: u64 = 4096;
const ADMISSION_CAP: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Fixed52,
    RttOneFrameV1,
    PreemptiveIdrV1,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Self::Fixed52 => "fixed52",
            Self::RttOneFrameV1 => "rtt_one_frame_v1",
            Self::PreemptiveIdrV1 => "preemptive_idr_v1",
        }
    }

    fn parse(value: &[u8]) -> Option<Self> {
        let value = value
            .strip_suffix(b"\r\n")
            .or_else(|| value.strip_suffix(b"\n"))
            .unwrap_or(value);
        match value {
            b"fixed52" => Some(Self::Fixed52),
            b"rtt_one_frame_v1" => Some(Self::RttOneFrameV1),
            b"preemptive_idr_v1" => Some(Self::PreemptiveIdrV1),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Admission {
    first: u64,
    last: u64,
    at: Instant,
    seen: u64,
}

#[derive(Debug)]
struct Head {
    generation: u64,
    base: u64,
    first: u64,
    last: u64,
    start: Instant,
    current_start: Instant,
    mode: Mode,
    disabled: bool,
    first_admission_seen: bool,
    eligible_admission: bool,
    effective: Option<Instant>,
    extension_used: bool,
    admissions: Vec<Admission>,
    preemptive_decided: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PreemptiveObservation {
    pub(super) base: u64,
    pub(super) frame_index: u32,
    pub(super) block_index: u8,
    pub(super) last_block_index: u8,
    pub(super) gap_started: Instant,
    pub(super) missing_data: usize,
    pub(super) parity: usize,
    pub(super) ready_later: usize,
}

#[derive(Clone, Copy, Debug)]
struct Intent {
    generation: u64,
    base: u64,
    deadline: Instant,
    armed_at: Instant,
}

#[derive(Debug)]
struct QueuedRequest {
    intent: Intent,
    queued_at: Instant,
    speculative: bool,
}

#[derive(Debug)]
struct LateWatch {
    base: u64,
    generation: u64,
    until: Instant,
    admissions: Vec<Admission>,
}

#[derive(Debug)]
struct State {
    mode: Mode,
    staged: Option<Mode>,
    applied_once: bool,
    invalid_file: bool,
    revision: u64,
    next_poll: Instant,
    head: Option<Head>,
    events: u64,
    omitted: u64,
    admissions_omitted: u64,
    arrival_offsets_omitted: u64,
    head_generation: u64,
    intent: Option<Intent>,
    last_speculative_attempt: Option<Instant>,
    queued_requests: VecDeque<QueuedRequest>,
    late_watches: VecDeque<LateWatch>,
    watch_omitted: u64,
    last_expired_head: Option<(u64, u64, Instant)>,
}

#[derive(Debug)]
pub(super) struct Experiment {
    mode_file: PathBuf,
    origin: Instant,
    realtime_origin_ns: u64,
    active_head: AtomicBool,
    pending_intent: AtomicBool,
    queued_observation: AtomicBool,
    late_observation: AtomicBool,
    state: Mutex<State>,
}

fn ns(duration: Duration) -> u64 {
    duration.as_nanos().try_into().unwrap_or(u64::MAX)
}

impl Experiment {
    pub(super) fn from_env() -> Option<Arc<Self>> {
        let path = PathBuf::from(std::env::var_os("OPENNOW_FEC_DEADLINE_MODE_FILE")?);
        if !path.is_absolute() {
            opennow_streamer_protocol::log::log_async(
                "WARN",
                "fec-deadline-mode",
                "schema=1 status=non-absolute-mode-file",
            );
            return None;
        }
        Some(Arc::new(Self::new(path, Instant::now())))
    }

    fn new(mode_file: PathBuf, origin: Instant) -> Self {
        let realtime_origin_ns = ns(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default());
        Self {
            mode_file,
            origin,
            realtime_origin_ns,
            active_head: AtomicBool::new(false),
            pending_intent: AtomicBool::new(false),
            queued_observation: AtomicBool::new(false),
            late_observation: AtomicBool::new(false),
            state: Mutex::new(State {
                mode: Mode::Fixed52,
                staged: None,
                applied_once: false,
                invalid_file: false,
                revision: 0,
                next_poll: origin,
                head: None,
                events: 0,
                omitted: 0,
                admissions_omitted: 0,
                arrival_offsets_omitted: 0,
                head_generation: 0,
                intent: None,
                last_speculative_attempt: None,
                queued_requests: VecDeque::with_capacity(ADMISSION_CAP),
                late_watches: VecDeque::with_capacity(ADMISSION_CAP),
                watch_omitted: 0,
                last_expired_head: None,
            }),
        }
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn event(&self, state: &mut State, event: &str, now: Instant, fields: impl FnOnce() -> String) {
        if state.events >= EVENT_CAP {
            state.omitted += 1;
            if state.omitted == 1 {
                opennow_streamer_protocol::log::log_async(
                    "WARN",
                    "fec-deadline-timing",
                    "schema=1 status=event-cap-reached cap=4096",
                );
            }
            return;
        }
        state.events += 1;
        let t_ns = ns(now.saturating_duration_since(self.origin));
        opennow_streamer_protocol::log::log_async(
            "INFO",
            "fec-deadline-timing",
            &format!(
                "schema=1 event={event} ordinal={} t_ns={t_ns} utc_ns={} revision={} {}",
                state.events,
                self.realtime_origin_ns.saturating_add(t_ns),
                state.revision,
                fields(),
            ),
        );
    }

    /// Call with the existing receive-loop clock, before packet processing.
    pub(super) fn poll_mode(&self, now: Instant, no_active_gap: bool) {
        {
            let mut state = self.state();
            if now < state.next_poll {
                return;
            }
            state.next_poll = now + POLL_INTERVAL;
        }
        use std::io::Read;
        let value = std::fs::File::open(&self.mode_file)
            .and_then(|file| {
                let mut bytes = Vec::with_capacity(64);
                file.take(65).read_to_end(&mut bytes)?;
                Ok(bytes)
            })
            .ok()
            .and_then(|bytes| Mode::parse(&bytes));
        let mut state = self.state();
        let Some(mode) = value else {
            state.staged = None;
            if !state.invalid_file {
                opennow_streamer_protocol::log::log_async(
                    "WARN",
                    "fec-deadline-mode",
                    "schema=1 status=invalid-mode-file",
                );
            }
            state.invalid_file = true;
            return;
        };
        state.invalid_file = false;
        if !no_active_gap {
            if state.staged != Some(mode) {
                state.staged = Some(mode);
                opennow_streamer_protocol::log::log_async(
                    "INFO",
                    "fec-deadline-mode",
                    &format!(
                        "schema=1 status=staged mode={} no_active_gap=0",
                        mode.label()
                    ),
                );
            }
            return;
        }
        state.staged = None;
        if !state.applied_once || state.mode != mode {
            state.mode = mode;
            state.applied_once = true;
            state.revision += 1;
            let t_ns = ns(now.saturating_duration_since(self.origin));
            opennow_streamer_protocol::log::log_async(
                "INFO",
                "fec-deadline-mode",
                &format!(
                    "schema=1 status=applied mode={} revision={} no_active_gap=1 t_ns={t_ns} utc_ns={}",
                    mode.label(),
                    state.revision,
                    self.realtime_origin_ns.saturating_add(t_ns),
                ),
            );
        }
    }

    /// Only incomplete heads call this. Fixed52 preserves the legacy renewed timer.
    pub(super) fn deadline(
        &self,
        base: u64,
        first: u64,
        last: u64,
        start: Instant,
        now: Instant,
    ) -> Instant {
        let mut state = self.state();
        if state.head.as_ref().is_some_and(|head| head.base != base) {
            self.close_locked(&mut state, now, "head_changed");
        }
        if state.head.is_none() {
            let mode = state.mode;
            state.head_generation += 1;
            let generation = state.head_generation;
            state.head = Some(Head {
                generation,
                base,
                first,
                last,
                start,
                current_start: start,
                mode,
                disabled: false,
                first_admission_seen: false,
                eligible_admission: false,
                effective: None,
                extension_used: false,
                admissions: Vec::with_capacity(ADMISSION_CAP),
                preemptive_decided: false,
            });
            self.active_head.store(true, Ordering::Release);
            self.event(&mut state, "head_open", now, || format!(
                "mode={} head_base={base} first={first} last={last} gap_start_ns={} nominal_expiry_ns={}",
                mode.label(), ns(start.saturating_duration_since(self.origin)), ns((start + BASE_WAIT).saturating_duration_since(self.origin)),
            ));
        }
        let head = state.head.as_mut().expect("observed incomplete head");
        let progressed = head.first != first || head.start != start;
        let newly_disabled = progressed && !head.disabled;
        if progressed {
            head.disabled = true;
            head.effective = None;
        }
        head.first = first;
        head.last = last;
        head.current_start = start;
        // Keep the initial start for eligibility/cap. Once progress changes the
        // timer, this head permanently uses the caller's current legacy timer.
        let deadline = if head.mode == Mode::RttOneFrameV1 && !head.disabled {
            head.effective.unwrap_or(start + BASE_WAIT)
        } else {
            start + BASE_WAIT
        };
        if newly_disabled {
            self.event(&mut state, "extension_disabled", now, || format!(
                "head_base={base} first={first} last={last} reason=missing_run_or_timer_changed effective_expiry_ns={}",
                ns(deadline.saturating_duration_since(self.origin)),
            ));
        }
        deadline
    }

    pub(super) fn enqueued(&self, first: u64, last: u64, now: Instant) {
        let mut state = self.state();
        let base = state
            .head
            .as_ref()
            .filter(|head| first <= head.last && last >= head.first)
            .map_or(0, |head| head.base);
        self.event(&mut state, "nack_enqueued", now, || {
            format!("head_base={base} first={first} last={last} timestamp_kind=receive_owner_stamp")
        });
    }

    pub(super) fn taken(
        &self,
        first: u64,
        last: u64,
        attempt: u8,
        retry_rtt: Option<Duration>,
        now: Instant,
    ) {
        let mut state = self.state();
        let base = state
            .head
            .as_ref()
            .filter(|head| first <= head.last && last >= head.first)
            .map_or(0, |head| head.base);
        self.event(&mut state, "nack_taken", now, || format!(
            "head_base={base} first={first} last={last} attempt={attempt} retry_rtt_ns={} retry_rtt_present={} timestamp_kind=control_owner_stamp",
            retry_rtt.map_or(0, ns), u8::from(retry_rtt.is_some()),
        ));
    }

    pub(super) fn admitted(
        &self,
        first: u64,
        last: u64,
        success: bool,
        measured_rtt: Option<Duration>,
        now: Instant,
    ) {
        let mut state = self.state();
        let mut base = 0;
        let mut effective = None;
        let mut eligible = false;
        let mut latched = false;
        let mut admission_omitted = false;
        if let Some(head) = state.head.as_mut()
            && first <= head.last
            && last >= head.first
        {
            base = head.base;
            if success {
                if head.admissions.len() < ADMISSION_CAP {
                    head.admissions.push(Admission {
                        first,
                        last,
                        at: now,
                        seen: 0,
                    });
                } else {
                    admission_omitted = true;
                }
                // Only a fully covering success consumes this opportunity.
                // Missing RTT or late admission still prevents later retries
                // from gaining eligibility. Partial receipts remain evidence.
                let fully_covering = first <= head.first && last >= head.last;
                let first_admission = fully_covering && !head.first_admission_seen;
                head.first_admission_seen |= fully_covering;
                if first_admission
                    && first <= head.first
                    && last >= head.last
                    && !head.disabled
                    && now < head.start + BASE_WAIT
                    && let Some(rtt) = measured_rtt
                {
                    eligible = true;
                    head.eligible_admission = true;
                    if head.mode == Mode::RttOneFrameV1 {
                        let expiry = (head.start + HARD_WAIT)
                            .min((head.start + BASE_WAIT).max(now + rtt + NACK_MARGIN));
                        head.effective = Some(expiry);
                        effective = Some(expiry);
                        latched = true;
                    }
                }
            }
        }
        state.admissions_omitted += u64::from(admission_omitted);
        self.event(&mut state, "nack_admission", now, || format!(
            "head_base={base} first={first} last={last} admitted={} measured_rtt_ns={} measured_rtt_present={} eligible_admission={} extension_latched={} effective_expiry_ns={} timestamp_kind=post_admission_return",
            u8::from(success), measured_rtt.map_or(0, ns), u8::from(measured_rtt.is_some()),
            u8::from(eligible), u8::from(latched), effective.map_or(0, |at| ns(at.saturating_duration_since(self.origin))),
        ));
    }

    /// Called only after the existing buffer/window limits permit another wait.
    pub(super) fn wait_extended(
        &self,
        base: u64,
        now: Instant,
        buffered_blocks: usize,
        block_span: u64,
    ) {
        let mut state = self.state();
        let expiry = state.head.as_mut().and_then(|head| {
            let expiry = head.effective?;
            if head.base != base
                || head.disabled
                || head.extension_used
                || head.mode != Mode::RttOneFrameV1
                || now < head.current_start + BASE_WAIT
                || now >= expiry
            {
                return None;
            }
            head.extension_used = true;
            Some(expiry)
        });
        if let Some(expiry) = expiry {
            self.event(&mut state, "extension_used", now, || format!(
                "head_base={base} bounds_allow_wait=1 buffered_blocks={buffered_blocks} block_span={block_span} effective_expiry_ns={}",
                ns(expiry.saturating_duration_since(self.origin)),
            ));
        }
    }

    pub(super) fn needs_preemptive_observation(&self, base: u64) -> bool {
        let state = self.state();
        state.head.as_ref().is_some_and(|head| {
            head.base == base
                && !head.preemptive_decided
                && head
                    .admissions
                    .iter()
                    .any(|request| request.first <= head.first && request.last >= head.last)
        })
    }

    pub(super) fn observe_preemptive(
        &self,
        observation: PreemptiveObservation,
        now: Instant,
        ordinary_pending: bool,
    ) {
        let mut state = self.state();
        let Some(head) = state.head.as_mut() else {
            return;
        };
        if head.base != observation.base
            || head.preemptive_decided
            || observation.ready_later == 0
            || observation.missing_data <= observation.parity
            || now >= observation.gap_started + BASE_WAIT
        {
            return;
        }
        let Some(nack_at) = head
            .admissions
            .iter()
            .find(|request| request.first <= head.first && request.last >= head.last)
            .map(|request| request.at)
        else {
            return;
        };
        if now < nack_at {
            return;
        }
        head.preemptive_decided = true;
        let intent = Intent {
            generation: head.generation,
            base: head.base,
            deadline: observation.gap_started + BASE_WAIT,
            armed_at: now,
        };
        let mode = head.mode;
        self.event(&mut state, "preemptive_eligible", now, || format!(
            "mode={} generation={} head_base={} frame_index={} block={} last_block={} missing_data={} total_parity={} ready_later={} nack_admission_ns={} ordinary_pending={} deadline_ns={}",
            mode.label(), intent.generation, intent.base, observation.frame_index, observation.block_index, observation.last_block_index,
            observation.missing_data, observation.parity, observation.ready_later, ns(nack_at.saturating_duration_since(self.origin)),
            u8::from(ordinary_pending), ns(intent.deadline.saturating_duration_since(self.origin)),
        ));
        if mode != Mode::PreemptiveIdrV1 {
            return;
        }
        self.event(&mut state, "preemptive_armed", now, || {
            format!("generation={} head_base={}", intent.generation, intent.base)
        });
        if ordinary_pending {
            self.event(&mut state, "preemptive_coalesced", now, || {
                format!(
                    "generation={} head_base={} reason=existing_pending",
                    intent.generation, intent.base
                )
            });
        } else {
            state.intent = Some(intent);
            self.pending_intent.store(true, Ordering::Release);
        }
    }

    pub(super) fn control_poll(
        &self,
        upstream: bool,
        control_open: bool,
        channels_present: bool,
        ordinary_last_attempt: &mut Instant,
        pending: impl FnMut() -> bool,
        send: impl FnOnce() -> bool,
    ) {
        if !self.pending_intent.load(Ordering::Acquire) {
            return;
        }
        self.control_poll_at(
            upstream,
            control_open,
            channels_present,
            ordinary_last_attempt,
            pending,
            send,
            Instant::now,
        );
    }

    fn control_poll_at(
        &self,
        upstream: bool,
        control_open: bool,
        channels_present: bool,
        ordinary_last_attempt: &mut Instant,
        mut pending: impl FnMut() -> bool,
        send: impl FnOnce() -> bool,
        mut clock: impl FnMut() -> Instant,
    ) {
        let now = clock();
        let mut state = self.state();
        let Some(intent) = state.intent else {
            return;
        };
        let reason = if now >= intent.deadline {
            Some("expired")
        } else if pending() {
            Some("existing_pending")
        } else if now.saturating_duration_since(*ordinary_last_attempt)
            < super::KEYFRAME_REQUEST_COOLDOWN
        {
            Some("ordinary_cooldown")
        } else if state
            .last_speculative_attempt
            .is_some_and(|at| now.saturating_duration_since(at) < super::KEYFRAME_REQUEST_COOLDOWN)
        {
            Some("local_throttle")
        } else {
            None
        };
        if let Some(reason) = reason {
            state.intent = None;
            self.pending_intent.store(false, Ordering::Release);
            self.event(&mut state, "preemptive_cancelled", now, || {
                format!(
                    "generation={} head_base={} reason={reason}",
                    intent.generation, intent.base
                )
            });
            return;
        }
        if !upstream || !control_open || !channels_present {
            return;
        }
        state.intent = None;
        self.pending_intent.store(false, Ordering::Release);
        drop(state);
        // Do not hold experiment state while consulting the ordinary owner or
        // entering SCTP. A late result retains this claimed generation only.
        let before_send = clock();
        let ordinary_pending = pending();
        let mut state = self.state();
        if ordinary_pending
            || before_send >= intent.deadline
            || !state
                .head
                .as_ref()
                .is_some_and(|head| head.generation == intent.generation)
        {
            self.event(&mut state, "preemptive_cancelled", before_send, || {
                format!(
                    "generation={} head_base={} reason={}",
                    intent.generation,
                    intent.base,
                    if ordinary_pending {
                        "existing_pending_recheck"
                    } else {
                        "expired_or_head_released"
                    },
                )
            });
            return;
        }
        state.last_speculative_attempt = Some(before_send);
        self.event(&mut state, "preemptive_attempt", before_send, || {
            format!(
                "generation={} head_base={} armed_ns={} deadline_lead_ns={}",
                intent.generation,
                intent.base,
                ns(intent.armed_at.saturating_duration_since(self.origin)),
                ns(intent.deadline.saturating_duration_since(before_send)),
            )
        });
        drop(state);
        let queued = send();
        let returned = clock();
        if queued {
            *ordinary_last_attempt = returned;
        }
        let mut state = self.state();
        if queued {
            if state.queued_requests.len() == ADMISSION_CAP {
                state.queued_requests.pop_front();
                state.watch_omitted += 1;
            }
            state.queued_requests.push_back(QueuedRequest {
                intent,
                queued_at: returned,
                speculative: true,
            });
            self.queued_observation.store(true, Ordering::Release);
        }
        self.event(&mut state, "preemptive_admission", returned, || format!(
            "generation={} head_base={} queued={} ordinary_cooldown_updated={} speculative_retry=0 timestamp_kind=post_admission_return",
            intent.generation, intent.base, u8::from(queued), u8::from(queued),
        ));
    }

    pub(super) fn assembled_keyframe(
        &self,
        frame_index: u32,
        rtp_timestamp: u32,
        contiguous: bool,
    ) {
        if !self.queued_observation.load(Ordering::Acquire) {
            return;
        }
        let now = Instant::now();
        let mut state = self.state();
        while state
            .queued_requests
            .front()
            .is_some_and(|request| request.queued_at <= now)
        {
            let request = state
                .queued_requests
                .pop_front()
                .expect("observed queued request");
            self.event(&mut state, "idr_post_request_keyframe", now, || format!(
                "generation={} head_base={} speculative={} frame_index={frame_index} rtp_timestamp={rtp_timestamp} expected_native_pts_ns={} contiguous={} request_ns={} request_to_assembly_ns={} correlation_only=1 timestamp_kind=application_observation",
                request.intent.generation, request.intent.base, u8::from(request.speculative), (u64::from(rtp_timestamp) * 1_000_000 / 90_000) * 1000,
                u8::from(contiguous), ns(request.queued_at.saturating_duration_since(self.origin)), ns(now.saturating_duration_since(request.queued_at)),
            ));
        }
        self.queued_observation
            .store(!state.queued_requests.is_empty(), Ordering::Release);
    }

    pub(super) fn ordinary_control_admission(&self, queued: bool, now: Instant) {
        let mut state = self.state();
        let (base, generation) = state
            .last_expired_head
            .filter(|(_, _, at)| {
                now.saturating_duration_since(*at) <= super::KEYFRAME_REQUEST_COOLDOWN
            })
            .map_or((0, 0), |(base, generation, _)| (base, generation));
        self.event(&mut state, "ordinary_control_idr_admission", now, || format!(
            "generation={generation} head_base={base} speculative=0 queued={} head_correlation_only=1 timestamp_kind=post_admission_return", u8::from(queued),
        ));
        if queued {
            if state.queued_requests.len() == ADMISSION_CAP {
                state.queued_requests.pop_front();
                state.watch_omitted += 1;
            }
            state.queued_requests.push_back(QueuedRequest {
                intent: Intent {
                    generation,
                    base,
                    deadline: now,
                    armed_at: now,
                },
                queued_at: now,
                speculative: false,
            });
            self.queued_observation.store(true, Ordering::Release);
        }
    }

    pub(super) fn arrival(&self, index: u64, owner_stamp: Instant) {
        if !self.active_head.load(Ordering::Acquire)
            && !self.late_observation.load(Ordering::Acquire)
        {
            return;
        }
        self.arrival_at(index, owner_stamp, Instant::now());
    }

    fn arrival_at(&self, index: u64, owner_stamp: Instant, now: Instant) {
        let mut state = self.state();
        state.late_watches.retain(|watch| now < watch.until);
        self.late_observation
            .store(!state.late_watches.is_empty(), Ordering::Release);
        let mut late_matches = [None; ADMISSION_CAP];
        let mut late_count = 0;
        for watch in &mut state.late_watches {
            if let Some(request) = watch.admissions.iter_mut().find(|request| {
                request.first <= index && index <= request.last && now >= request.at
            }) {
                let offset = index - request.first;
                if offset < 64 && request.seen & (1_u64 << offset) == 0 {
                    request.seen |= 1_u64 << offset;
                    late_matches[late_count] = Some((watch.base, watch.generation, request.at));
                    late_count += 1;
                }
            }
        }
        for (base, generation, admission) in late_matches.into_iter().flatten() {
            self.event(&mut state, "late_matching_post_nack_arrival", now, || format!(
                "generation={generation} head_base={base} extended_seq={index} admission_ns={} receive_owner_t_ns={} diagnostic_only=1 timestamp_kind=application_observation",
                ns(admission.saturating_duration_since(self.origin)), ns(owner_stamp.saturating_duration_since(self.origin)),
            ));
        }
        let mut offset_omitted = false;
        let matched = state.head.as_mut().and_then(|head| {
            let request = head.admissions.iter_mut().find(|request| {
                request.first <= index && index <= request.last && now >= request.at
            })?;
            let bit = index.checked_sub(request.first)?;
            if bit >= 64 {
                offset_omitted = true;
                return None;
            }
            let bit = 1_u64 << bit;
            if request.seen & bit != 0 {
                return None;
            }
            request.seen |= bit;
            Some((head.base, request.at))
        });
        state.arrival_offsets_omitted += u64::from(offset_omitted);
        if let Some((base, admission)) = matched {
            self.event(&mut state, "matching_post_nack_arrival", now, || {
                format!(
                    "head_base={base} extended_seq={index} admission_ns={} since_admission_ns={} receive_owner_t_ns={} timestamp_kind=application_observation",
                    ns(admission.saturating_duration_since(self.origin)),
                    ns(now.saturating_duration_since(admission)),
                    ns(owner_stamp.saturating_duration_since(self.origin)),
                )
            });
        }
    }

    fn close_locked(&self, state: &mut State, now: Instant, outcome: &str) {
        if let Some(head) = state.head.take() {
            if outcome == "expired" {
                state.last_expired_head = Some((head.base, head.generation, now));
            }
            if state
                .intent
                .is_some_and(|intent| intent.generation == head.generation)
            {
                state.intent = None;
                self.pending_intent.store(false, Ordering::Release);
                self.event(state, "preemptive_cancelled", now, || {
                    format!(
                        "generation={} head_base={} reason=head_released",
                        head.generation, head.base
                    )
                });
            }
            self.active_head.store(false, Ordering::Release);
            self.event(state, "head_closed", now, || format!(
                "head_base={} mode={} first={} last={} outcome={outcome} disabled={} eligible_admission={} extension_used={} observed_age_ns={} nominal_expiry_ns={} effective_expiry_ns={}",
                head.base, head.mode.label(), head.first, head.last, u8::from(head.disabled), u8::from(head.eligible_admission), u8::from(head.extension_used),
                ns(now.saturating_duration_since(head.start)), ns((head.current_start + BASE_WAIT).saturating_duration_since(self.origin)),
                ns(head.effective.unwrap_or(head.current_start + BASE_WAIT).saturating_duration_since(self.origin)),
            ));
            if outcome == "expired" && !head.admissions.is_empty() {
                if state.late_watches.len() == ADMISSION_CAP {
                    state.late_watches.pop_front();
                    state.watch_omitted += 1;
                }
                state.late_watches.push_back(LateWatch {
                    base: head.base,
                    generation: head.generation,
                    until: now + BASE_WAIT,
                    admissions: head.admissions,
                });
                self.late_observation.store(true, Ordering::Release);
            }
        }
    }

    pub(super) fn finish_head(&self, base: u64, now: Instant, outcome: &str) {
        if !self.active_head.load(Ordering::Acquire) {
            return;
        }
        let mut state = self.state();
        if state.head.as_ref().is_some_and(|head| head.base == base) {
            self.close_locked(&mut state, now, outcome);
        }
    }

    pub(super) fn has_active_head(&self) -> bool {
        self.active_head.load(Ordering::Acquire)
    }

    pub(super) fn finish_head_with_context(
        &self,
        base: u64,
        now: Instant,
        outcome: &str,
        buffered_blocks: usize,
        ready_successors: usize,
    ) {
        let mut state = self.state();
        if state.head.as_ref().is_some_and(|head| head.base == base) {
            self.event(&mut state, "head_close_context", now, || format!(
                "head_base={base} outcome={outcome} buffered_blocks={buffered_blocks} ready_successors={ready_successors}",
            ));
            self.close_locked(&mut state, now, outcome);
        }
    }

    pub(super) fn reset(&self, now: Instant) {
        self.close_locked(&mut self.state(), now, "reset");
    }
}

impl Drop for Experiment {
    fn drop(&mut self) {
        let state = self
            .state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        opennow_streamer_protocol::log::log_async(
            "INFO",
            "fec-deadline-timing",
            &format!(
                "schema=1 status=finished emitted={} omitted={} admissions_omitted={} arrival_offsets_omitted={} watch_omitted={} cap={EVENT_CAP}",
                state.events,
                state.omitted,
                state.admissions_omitted,
                state.arrival_offsets_omitted,
                state.watch_omitted,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(start: Instant) -> Experiment {
        let experiment = Experiment::new(PathBuf::from("/unused/mode"), start);
        experiment.state().mode = Mode::RttOneFrameV1;
        experiment
    }

    #[test]
    fn strict_modes_reject_whitespace_unknown_and_extra_lines() {
        assert_eq!(
            Mode::parse(b"preemptive_idr_v1\n"),
            Some(Mode::PreemptiveIdrV1)
        );
        assert_eq!(Mode::parse(b"fixed52\n"), Some(Mode::Fixed52));
        assert_eq!(
            Mode::parse(b"rtt_one_frame_v1\r\n"),
            Some(Mode::RttOneFrameV1)
        );
        for value in [
            b" fixed52".as_slice(),
            b"fixed52 \n",
            b"fixed52\n\n",
            b"other",
        ] {
            assert_eq!(Mode::parse(value), None);
        }
    }

    #[test]
    fn admitted_measured_rtt_extends_once_and_is_capped() {
        let start = Instant::now();
        let experiment = candidate(start);
        assert_eq!(
            experiment.deadline(100, 101, 109, start, start),
            start + BASE_WAIT
        );
        experiment.admitted(
            101,
            109,
            true,
            Some(Duration::from_millis(56)),
            start + Duration::from_millis(4),
        );
        let expiry = start + Duration::from_millis(64);
        assert_eq!(experiment.deadline(100, 101, 109, start, start), expiry);
        experiment.admitted(
            101,
            109,
            true,
            Some(Duration::from_millis(80)),
            start + Duration::from_millis(8),
        );
        assert_eq!(experiment.deadline(100, 101, 109, start, start), expiry);
        experiment.finish_head(100, start, "completed");
        experiment.deadline(200, 201, 209, start, start);
        experiment.admitted(
            201,
            209,
            true,
            Some(Duration::from_millis(200)),
            start + Duration::from_millis(4),
        );
        assert_eq!(
            experiment.deadline(200, 201, 209, start, start),
            start + HARD_WAIT
        );
    }

    #[test]
    fn missing_rtt_late_or_failed_admission_cannot_extend() {
        let start = Instant::now();
        for (success, rtt, age) in [
            (true, None, Duration::from_millis(4)),
            (true, Some(Duration::from_millis(56)), BASE_WAIT),
            (
                false,
                Some(Duration::from_millis(56)),
                Duration::from_millis(4),
            ),
        ] {
            let experiment = candidate(start);
            experiment.deadline(100, 101, 109, start, start);
            experiment.admitted(101, 109, success, rtt, start + age);
            assert_eq!(
                experiment.deadline(100, 101, 109, start, start),
                start + BASE_WAIT
            );
        }
    }

    #[test]
    fn run_progress_preserves_legacy_renewal_and_never_relatches() {
        let start = Instant::now();
        let experiment = candidate(start);
        experiment.deadline(100, 101, 109, start, start);
        experiment.admitted(
            101,
            109,
            true,
            Some(Duration::from_millis(56)),
            start + Duration::from_millis(4),
        );
        let renewed = start + Duration::from_millis(40);
        assert_eq!(
            experiment.deadline(100, 102, 109, renewed, renewed),
            renewed + BASE_WAIT
        );
        experiment.admitted(
            102,
            109,
            true,
            Some(Duration::from_millis(56)),
            renewed + Duration::from_millis(4),
        );
        assert_eq!(
            experiment.deadline(100, 102, 109, renewed, renewed),
            renewed + BASE_WAIT
        );
        assert!(experiment.state().head.as_ref().unwrap().disabled);
    }

    #[test]
    fn partial_admission_is_diagnostic_only_and_first_full_success_may_qualify() {
        let start = Instant::now();
        let experiment = candidate(start);
        experiment.deadline(100, 101, 109, start, start);
        experiment.admitted(
            101,
            105,
            true,
            Some(Duration::from_millis(56)),
            start + Duration::from_millis(4),
        );
        assert_eq!(
            experiment.deadline(100, 101, 109, start, start),
            start + BASE_WAIT
        );
        experiment.admitted(
            101,
            109,
            true,
            Some(Duration::from_millis(56)),
            start + Duration::from_millis(8),
        );
        assert_eq!(
            experiment.deadline(100, 101, 109, start, start),
            start + Duration::from_millis(68)
        );
        let state = experiment.state();
        let head = state.head.as_ref().unwrap();
        assert_eq!(head.admissions.len(), 2);
        assert!(head.eligible_admission);
        assert!(head.effective.is_some());
    }

    #[test]
    fn fixed_arm_has_eligible_admissions_but_never_extends() {
        let start = Instant::now();
        let experiment = Experiment::new(PathBuf::from("/unused/mode"), start);
        experiment.deadline(100, 101, 109, start, start);
        experiment.admitted(
            101,
            109,
            true,
            Some(Duration::from_millis(56)),
            start + Duration::from_millis(4),
        );
        assert!(experiment.state().head.as_ref().unwrap().eligible_admission);
        assert_eq!(
            experiment.deadline(100, 101, 109, start, start),
            start + BASE_WAIT
        );
        experiment.wait_extended(100, start + Duration::from_millis(53), 2, 5);
        assert!(!experiment.state().head.as_ref().unwrap().extension_used);
    }

    #[test]
    fn extension_used_requires_a_wait_after_base_expiry_and_is_one_shot() {
        let start = Instant::now();
        let experiment = candidate(start);
        experiment.deadline(100, 101, 109, start, start);
        experiment.admitted(
            101,
            109,
            true,
            Some(Duration::from_millis(56)),
            start + Duration::from_millis(4),
        );
        experiment.wait_extended(100, start + Duration::from_millis(51), 2, 5);
        assert!(!experiment.state().head.as_ref().unwrap().extension_used);
        let before = experiment.state().events;
        experiment.wait_extended(100, start + Duration::from_millis(53), 2, 5);
        experiment.wait_extended(100, start + Duration::from_millis(54), 2, 5);
        assert_eq!(experiment.state().events, before + 1);
        assert!(experiment.state().head.as_ref().unwrap().extension_used);
    }

    #[test]
    fn matching_arrivals_are_distinct_and_out_of_mask_offsets_are_counted() {
        let start = Instant::now();
        let experiment = candidate(start);
        experiment.deadline(100, 101, 109, start, start);
        experiment.admitted(
            101,
            200,
            true,
            Some(Duration::from_millis(56)),
            start + Duration::from_millis(4),
        );
        let before = experiment.state().events;
        experiment.arrival_at(101, start, start + Duration::from_millis(56));
        experiment.arrival_at(101, start, start + Duration::from_millis(57));
        experiment.arrival_at(200, start, start + Duration::from_millis(58));
        assert_eq!(experiment.state().events, before + 1);
        assert_eq!(experiment.state().arrival_offsets_omitted, 1);
    }

    #[test]
    fn mode_file_polling_stages_during_gaps_and_applies_at_idle_boundary() {
        let start = Instant::now();
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("opennow-fec-mode-{}-{suffix}", std::process::id()));
        std::fs::write(&path, b"fixed52\n").unwrap();
        let experiment = Experiment::new(path.clone(), start);
        experiment.poll_mode(start, true);
        assert_eq!(experiment.state().revision, 1);
        std::fs::write(&path, b"rtt_one_frame_v1\n").unwrap();
        experiment.poll_mode(start + Duration::from_millis(50), true);
        assert_eq!(experiment.state().mode, Mode::Fixed52);
        experiment.poll_mode(start + POLL_INTERVAL, false);
        assert_eq!(experiment.state().staged, Some(Mode::RttOneFrameV1));
        assert_eq!(experiment.state().mode, Mode::Fixed52);
        experiment.poll_mode(start + POLL_INTERVAL * 2, true);
        assert_eq!(experiment.state().mode, Mode::RttOneFrameV1);
        assert_eq!(experiment.state().revision, 2);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn timing_events_are_strictly_bounded_with_omission_counts() {
        let start = Instant::now();
        let experiment = candidate(start);
        experiment.state().events = EVENT_CAP;
        experiment.enqueued(101, 109, start);
        experiment.taken(101, 109, 1, None, start);
        assert_eq!(experiment.state().events, EVENT_CAP);
        assert_eq!(experiment.state().omitted, 2);
    }

    fn preemptive(start: Instant, base: u64) -> Experiment {
        let experiment = Experiment::new(PathBuf::from("/unused/mode"), start);
        experiment.state().mode = Mode::PreemptiveIdrV1;
        experiment.deadline(base, base + 1, base + 9, start, start);
        experiment.admitted(base + 1, base + 9, true, None, start);
        experiment
    }

    fn observation(start: Instant, base: u64) -> PreemptiveObservation {
        PreemptiveObservation {
            base,
            frame_index: 7,
            block_index: 0,
            last_block_index: 1,
            gap_started: start,
            missing_data: 9,
            parity: 2,
            ready_later: 1,
        }
    }

    #[test]
    fn preemptive_guard_requires_strict_parity_ready_successor_admitted_nack_and_unexpired_wait() {
        let start = Instant::now();
        for kind in 0..4 {
            let experiment = preemptive(start, 100);
            let mut sample = observation(start, 100);
            let mut now = start;
            match kind {
                0 => sample.missing_data = sample.parity,
                1 => sample.ready_later = 0,
                2 => now += BASE_WAIT,
                _ => experiment.state().head.as_mut().unwrap().admissions.clear(),
            }
            experiment.observe_preemptive(sample, now, false);
            assert!(experiment.state().intent.is_none());
        }
        let experiment = preemptive(start, 100);
        experiment.observe_preemptive(observation(start, 100), start, false);
        assert!(experiment.state().intent.is_some());
        assert_eq!(
            experiment.deadline(100, 101, 109, start, start),
            start + BASE_WAIT
        );
        let generation = experiment.state().intent.unwrap().generation;
        experiment.observe_preemptive(observation(start, 100), start, false);
        assert_eq!(experiment.state().intent.unwrap().generation, generation);
    }

    #[test]
    fn preemptive_failed_attempt_is_one_shot_and_does_not_delay_ordinary_recovery() {
        let start = Instant::now();
        let experiment = preemptive(start, 100);
        experiment.observe_preemptive(observation(start, 100), start, false);
        let previous = start - super::super::KEYFRAME_REQUEST_COOLDOWN;
        let mut ordinary = previous;
        let calls = std::cell::Cell::new(0);
        experiment.control_poll_at(
            true,
            true,
            true,
            &mut ordinary,
            || false,
            || {
                calls.set(calls.get() + 1);
                false
            },
            || start + Duration::from_millis(4),
        );
        experiment.control_poll_at(
            true,
            true,
            true,
            &mut ordinary,
            || false,
            || {
                calls.set(calls.get() + 1);
                true
            },
            || start + Duration::from_millis(5),
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(ordinary, previous);
        experiment.finish_head(100, start + BASE_WAIT, "expired");
        let feedback = super::super::NvstFeedbackState::default();
        feedback.request_keyframe();
        assert_eq!(
            feedback.keyframe_request_routes(start + BASE_WAIT, ordinary, false, true),
            (false, true)
        );
        let new_start = start + Duration::from_millis(60);
        experiment.deadline(200, 201, 209, new_start, new_start);
        experiment.admitted(201, 209, true, None, new_start);
        experiment.observe_preemptive(observation(new_start, 200), new_start, false);
        experiment.control_poll_at(
            true,
            true,
            true,
            &mut ordinary,
            || false,
            || {
                calls.set(calls.get() + 1);
                true
            },
            || new_start,
        );
        assert_eq!(
            calls.get(),
            1,
            "failed speculation still respects local global throttle"
        );
        assert_eq!(ordinary, previous);
    }

    #[test]
    fn preemptive_success_updates_real_cooldown_and_pending_coalesces_without_mutation() {
        let start = Instant::now();
        let experiment = preemptive(start, 100);
        experiment.observe_preemptive(observation(start, 100), start, false);
        let returned = start + Duration::from_millis(5);
        let mut ordinary = start - super::super::KEYFRAME_REQUEST_COOLDOWN;
        experiment.control_poll_at(
            true,
            true,
            true,
            &mut ordinary,
            || false,
            || true,
            || returned,
        );
        assert_eq!(ordinary, returned);
        let feedback = super::super::NvstFeedbackState::default();
        feedback.request_keyframe();
        assert_eq!(
            feedback.keyframe_request_routes(start + BASE_WAIT, ordinary, false, true),
            (false, false)
        );
        let experiment = preemptive(start, 200);
        experiment.observe_preemptive(observation(start, 200), start, false);
        let check = std::cell::Cell::new(0);
        ordinary = start - super::super::KEYFRAME_REQUEST_COOLDOWN;
        let previous = ordinary;
        experiment.control_poll_at(
            true,
            true,
            true,
            &mut ordinary,
            || {
                check.set(check.get() + 1);
                check.get() == 2
            },
            || panic!("a pending ordinary request must coalesce at the final check"),
            || returned,
        );
        assert_eq!(ordinary, previous);
        assert_eq!(check.get(), 2);
        assert!(feedback.keyframe_request_pending());
        assert!(experiment.state().intent.is_none());
    }

    #[test]
    fn preemptive_recent_ordinary_success_cannot_be_bypassed() {
        let start = Instant::now();
        let experiment = preemptive(start, 100);
        experiment.observe_preemptive(observation(start, 100), start, false);
        let mut ordinary = start;
        experiment.control_poll_at(
            true,
            true,
            true,
            &mut ordinary,
            || false,
            || panic!("ordinary cooldown must block speculative send"),
            || start + Duration::from_millis(4),
        );
        assert_eq!(ordinary, start);
        assert!(experiment.state().last_speculative_attempt.is_none());
    }

    #[test]
    fn preemptive_off_unavailable_and_expired_intents_never_send() {
        let start = Instant::now();
        let experiment = preemptive(start, 100);
        experiment.state().head.as_mut().unwrap().mode = Mode::Fixed52;
        experiment.observe_preemptive(observation(start, 100), start, false);
        assert!(experiment.state().intent.is_none());
        let experiment = preemptive(start, 100);
        experiment.observe_preemptive(observation(start, 100), start, false);
        let mut ordinary = start - super::super::KEYFRAME_REQUEST_COOLDOWN;
        experiment.control_poll_at(
            false,
            true,
            true,
            &mut ordinary,
            || false,
            || panic!(),
            || start,
        );
        assert!(experiment.state().intent.is_some());
        experiment.control_poll_at(
            true,
            false,
            true,
            &mut ordinary,
            || false,
            || panic!(),
            || start,
        );
        experiment.control_poll_at(
            true,
            true,
            false,
            &mut ordinary,
            || false,
            || panic!(),
            || start,
        );
        experiment.control_poll_at(
            true,
            true,
            true,
            &mut ordinary,
            || false,
            || panic!(),
            || start + BASE_WAIT,
        );
        assert!(experiment.state().intent.is_none());
        assert!(experiment.state().last_speculative_attempt.is_none());
    }

    #[test]
    fn expired_sequence_watch_is_diagnostic_only_and_limited_to_52ms() {
        let start = Instant::now();
        let experiment = preemptive(start, 100);
        experiment.finish_head(100, start + BASE_WAIT, "expired");
        let before = experiment.state().events;
        experiment.arrival_at(101, start, start + BASE_WAIT + Duration::from_millis(51));
        assert_eq!(experiment.state().events, before + 1);
        experiment.arrival_at(102, start, start + BASE_WAIT * 2);
        assert_eq!(experiment.state().events, before + 1);
        assert!(experiment.state().head.is_none());
        assert!(experiment.state().late_watches.is_empty());
    }

    #[test]
    fn ordinary_keyframe_observation_is_balanced_bounded_and_never_precedes_request() {
        let start = Instant::now();
        let experiment = preemptive(start, 100);
        let future = start + Duration::from_secs(60);
        for _ in 0..ADMISSION_CAP + 2 {
            experiment.ordinary_control_admission(true, future);
        }
        assert_eq!(experiment.state().queued_requests.len(), ADMISSION_CAP);
        assert_eq!(experiment.state().watch_omitted, 2);
        assert!(
            experiment
                .state()
                .queued_requests
                .iter()
                .all(|request| !request.speculative)
        );
        experiment.assembled_keyframe(7, 90_000, false);
        assert_eq!(
            experiment.state().queued_requests.len(),
            ADMISSION_CAP,
            "an earlier callback cannot observe a request admitted while waiting for the observer lock"
        );
        for request in &mut experiment.state().queued_requests {
            request.queued_at = start - Duration::from_millis(1);
        }
        experiment.assembled_keyframe(8, 90_750, true);
        assert!(experiment.state().queued_requests.is_empty());
        assert!(!experiment.queued_observation.load(Ordering::Acquire));
    }

    #[test]
    fn late_send_result_keeps_claimed_generation_and_preserves_a_newer_intent() {
        let start = Instant::now();
        let experiment = preemptive(start, 100);
        experiment.observe_preemptive(observation(start, 100), start, false);
        let generation = experiment.state().intent.unwrap().generation;
        let mut ordinary = start - super::super::KEYFRAME_REQUEST_COOLDOWN;
        let mut clocks = VecDeque::from([
            start + Duration::from_millis(4),
            start + Duration::from_millis(4),
            start + Duration::from_millis(54),
        ]);
        experiment.control_poll_at(
            true,
            true,
            true,
            &mut ordinary,
            || false,
            || {
                experiment.finish_head(100, start + BASE_WAIT, "expired");
                let next = start + Duration::from_millis(53);
                experiment.deadline(200, 201, 209, next, next);
                experiment.admitted(201, 209, true, None, next);
                experiment.observe_preemptive(observation(next, 200), next, false);
                true
            },
            || clocks.pop_front().unwrap(),
        );
        let state = experiment.state();
        assert_eq!(
            state.queued_requests.front().unwrap().intent.generation,
            generation
        );
        assert_ne!(state.intent.unwrap().generation, generation);
        assert_eq!(state.head.as_ref().unwrap().base, 200);
    }

    #[test]
    fn source_guard_uses_exact_holes_and_arming_does_not_release_or_invalidate() {
        use super::super::{
            FecPacketLayout, FecReorderBuffer, NvstFeedbackState, RTP_FIXED_HEADER_LEN, RtpHeader,
            RtpPacket, RtpPacketOrigin,
        };
        fn packet(index: u64) -> RtpPacket {
            let mut plaintext = vec![0_u8; RTP_FIXED_HEADER_LEN];
            plaintext[0] = 0x80;
            plaintext[1] = 96;
            plaintext[2..4].copy_from_slice(&(index as u16).to_be_bytes());
            plaintext[8..12].copy_from_slice(&1_u32.to_be_bytes());
            RtpPacket {
                index,
                header: RtpHeader::parse(&plaintext).unwrap(),
                plaintext,
                origin: RtpPacketOrigin::Authenticated,
            }
        }
        fn layout(block: u8, shard: usize) -> FecPacketLayout {
            FecPacketLayout {
                frame_index: 7,
                block_index: block,
                last_block_index: 2,
                shard_index: shard,
                data_shards: if block == 0 { 4 } else { 2 },
                parity_shards: 1,
            }
        }
        for forced_bound in [false, true] {
            let start = Instant::now();
            let experiment = Arc::new(Experiment::new(PathBuf::from("/unused/mode"), start));
            experiment.state().mode = Mode::PreemptiveIdrV1;
            let feedback = Arc::new(NvstFeedbackState::default());
            let mut reorder = FecReorderBuffer::new(48);
            reorder.deadline_experiment = Some(Arc::clone(&experiment));
            reorder.experiment_feedback = Some(Arc::clone(&feedback));
            reorder.push(packet(100), layout(0, 0), start);
            reorder.push(packet(102), layout(0, 2), start);
            reorder.push(packet(105), layout(1, 0), start);
            reorder.push(packet(106), layout(1, 1), start);
            experiment.admitted(101, 101, true, None, start);
            let result = if forced_bound {
                reorder.push(packet(3000), layout(2, 0), start)
            } else {
                reorder.push(packet(107), layout(1, 2), start)
            };
            assert!(
                !feedback.keyframe_request_pending(),
                "speculation must not own the generic bit"
            );
            if forced_bound {
                assert!(result.recovery.is_some());
                assert!(experiment.state().intent.is_none());
            } else {
                assert!(result.recovery.is_none());
                assert!(result.dropped.is_none());
                assert!(result.ready.is_empty());
                assert_eq!(reorder.completed_through, None);
                assert_eq!(reorder.gap_wait.unwrap().0, 101);
                assert!(
                    experiment.state().intent.is_some(),
                    "two separated missing shards exceed parity1 even though the NACK run has length1"
                );
            }
        }
    }

    #[test]
    fn fec_owner_waits_only_in_candidate_arm_and_keeps_packet_gap_recovery() {
        use super::super::{
            FecPacketLayout, FecReorderBuffer, NvstRecovery, RTP_FIXED_HEADER_LEN, RtpHeader,
            RtpPacket, RtpPacketOrigin,
        };
        fn packet(index: u64) -> RtpPacket {
            let mut plaintext = vec![0_u8; RTP_FIXED_HEADER_LEN];
            plaintext[0] = 0x80;
            plaintext[1] = 96;
            plaintext[2..4].copy_from_slice(&(index as u16).to_be_bytes());
            plaintext[8..12].copy_from_slice(&1_u32.to_be_bytes());
            RtpPacket {
                index,
                header: RtpHeader::parse(&plaintext).unwrap(),
                plaintext,
                origin: RtpPacketOrigin::Authenticated,
            }
        }
        fn layout(block: u8, shard: usize) -> FecPacketLayout {
            FecPacketLayout {
                frame_index: 7,
                block_index: block,
                last_block_index: 2,
                shard_index: shard,
                data_shards: if block == 0 { 4 } else { 2 },
                parity_shards: 1,
            }
        }
        for mode in [Mode::Fixed52, Mode::RttOneFrameV1] {
            let start = Instant::now();
            let experiment = Arc::new(Experiment::new(PathBuf::from("/unused/mode"), start));
            experiment.state().mode = mode;
            let mut reorder = FecReorderBuffer::new(48);
            reorder.deadline_experiment = Some(Arc::clone(&experiment));
            reorder.push(packet(100), layout(0, 0), start);
            reorder.push(packet(105), layout(1, 0), start + Duration::from_millis(1));
            reorder.push(packet(106), layout(1, 1), start + Duration::from_millis(2));
            experiment.admitted(
                101,
                103,
                true,
                Some(Duration::from_millis(56)),
                start + Duration::from_millis(5),
            );
            let at_base =
                reorder.push(packet(107), layout(1, 2), start + Duration::from_millis(53));
            let expired = if mode == Mode::RttOneFrameV1 {
                assert!(at_base.recovery.is_none());
                assert!(at_base.ready.is_empty());
                assert!(experiment.state().head.as_ref().unwrap().extension_used);
                reorder.push(packet(108), layout(2, 0), start + Duration::from_millis(65))
            } else {
                at_base
            };
            assert_eq!(
                expired.recovery,
                Some(NvstRecovery::PacketGap {
                    first_missing_index: 101,
                    last_missing_index: 103
                })
            );
            assert_eq!(
                expired
                    .ready
                    .iter()
                    .map(|packet| packet.index)
                    .collect::<Vec<_>>(),
                [105, 106]
            );
            assert!(experiment.state().head.is_none());
        }
    }
}
