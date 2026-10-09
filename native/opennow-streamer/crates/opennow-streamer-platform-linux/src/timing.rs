use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const STAGE_SAMPLE_CAPACITY: usize = 256;

pub const IN_FLIGHT_CAPACITY: usize = 64;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DecodeStagePercentiles {
    pub p50_us: u64,
    pub p95_us: u64,
    pub max_us: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DecodeTimings {
    pub call: Option<DecodeStagePercentiles>,
    pub residence: Option<DecodeStagePercentiles>,
    pub call_window_samples: usize,
    pub residence_window_samples: usize,
    pub submissions_total: u64,
    pub outputs_total: u64,
    pub output_calls_total: u64,
    pub last_submission_at: Option<Instant>,
    pub last_output_at: Option<Instant>,
    pub in_flight: usize,
    pub oldest_in_flight_at: Option<Instant>,
    pub epoch: u64,
    pub epoch_started_at: Option<Instant>,
    pub unmatched_outputs: u64,
    pub unmatched_submissions: u64,
    pub duplicate_timestamps: u64,
}

impl DecodeTimings {
    pub fn is_empty(&self) -> bool {
        self.call.is_none() && self.residence.is_none()
    }

    pub fn has_progress(&self) -> bool {
        self.outputs_total > 0
    }

    pub fn has_observable_state(&self) -> bool {
        !self.is_empty() || self.submissions_total > 0 || self.outputs_total > 0
    }

    pub fn progress_reference(&self) -> Option<Instant> {
        let this_epoch_output = self
            .last_output_at
            .filter(|at| self.epoch_started_at.is_none_or(|floor| *at >= floor));
        let outstanding = self.oldest_in_flight_at;
        match (this_epoch_output, outstanding) {
            (Some(last), Some(oldest)) => Some(last.max(oldest)),
            (Some(last), None) => Some(last),
            (None, Some(oldest)) => Some(
                self.epoch_started_at
                    .map_or(oldest, |floor| oldest.max(floor)),
            ),
            (None, None) => None,
        }
    }
}

#[derive(Clone, Default)]
pub struct DecodeTimingProbe {
    inner: Arc<Mutex<DecodeTimingAccumulator>>,
}

impl DecodeTimingProbe {
    fn with<R>(&self, op: impl FnOnce(&mut DecodeTimingAccumulator) -> R) -> R {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        op(&mut inner)
    }

    pub fn record_submission(&self, timestamp_us: u64) {
        self.with(|inner| inner.record_submission(timestamp_us, Instant::now()));
    }

    pub fn record_output(&self, timestamp_us: u64) {
        self.with(|inner| inner.record_output(timestamp_us, Instant::now()));
    }

    pub fn record_call(&self, duration: Duration, produced_output: bool) {
        self.with(|inner| inner.record_call(duration, produced_output));
    }

    pub fn clear(&self) {
        self.with(|inner| inner.clear());
    }

    pub fn snapshot(&self) -> DecodeTimings {
        self.with(|inner| inner.snapshot())
    }
}

#[derive(Debug, Default)]
struct DecodeTimingAccumulator {
    in_flight: VecDeque<(u64, Instant)>,
    call_us: VecDeque<u64>,
    residence_us: VecDeque<u64>,
    submissions_total: u64,
    outputs_total: u64,
    output_calls_total: u64,
    last_submission_at: Option<Instant>,
    last_output_at: Option<Instant>,
    unmatched_outputs: u64,
    unmatched_submissions: u64,
    duplicate_timestamps: u64,
    epoch: u64,
    epoch_started_at: Option<Instant>,
}

impl DecodeTimingAccumulator {
    fn clear(&mut self) {
        let retired = self.in_flight.len() as u64;
        self.in_flight.clear();
        self.unmatched_submissions = self.unmatched_submissions.saturating_add(retired);
        self.epoch = self.epoch.saturating_add(1);
        self.epoch_started_at = Some(Instant::now());
    }

    fn record_submission(&mut self, timestamp_us: u64, at: Instant) {
        if self
            .in_flight
            .iter()
            .any(|(queued, _)| *queued == timestamp_us)
        {
            self.duplicate_timestamps = self.duplicate_timestamps.saturating_add(1);
        }
        while self.in_flight.len() >= IN_FLIGHT_CAPACITY {
            self.in_flight.pop_front();
            self.unmatched_submissions = self.unmatched_submissions.saturating_add(1);
        }
        self.in_flight.push_back((timestamp_us, at));
        self.submissions_total = self.submissions_total.saturating_add(1);
        self.last_submission_at = Some(at);
    }

    fn record_output(&mut self, timestamp_us: u64, at: Instant) {
        self.outputs_total = self.outputs_total.saturating_add(1);
        self.last_output_at = Some(at);
        let Some(position) = self
            .in_flight
            .iter()
            .position(|(queued, _)| *queued == timestamp_us)
        else {
            self.unmatched_outputs = self.unmatched_outputs.saturating_add(1);
            return;
        };
        let (_, submitted_at) = self.in_flight.remove(position).expect("position found");
        push_sample(
            &mut self.residence_us,
            at.saturating_duration_since(submitted_at),
        );
    }

    fn record_call(&mut self, duration: Duration, produced_output: bool) {
        if !produced_output {
            return;
        }
        self.output_calls_total = self.output_calls_total.saturating_add(1);
        push_sample(&mut self.call_us, duration);
    }

    fn snapshot(&self) -> DecodeTimings {
        DecodeTimings {
            call: summarize(&self.call_us),
            residence: summarize(&self.residence_us),
            call_window_samples: self.call_us.len(),
            residence_window_samples: self.residence_us.len(),
            submissions_total: self.submissions_total,
            outputs_total: self.outputs_total,
            output_calls_total: self.output_calls_total,
            last_submission_at: self.last_submission_at,
            last_output_at: self.last_output_at,
            in_flight: self.in_flight.len(),
            oldest_in_flight_at: self.in_flight.front().map(|(_, at)| *at),
            epoch: self.epoch,
            epoch_started_at: self.epoch_started_at,
            unmatched_outputs: self.unmatched_outputs,
            unmatched_submissions: self.unmatched_submissions,
            duplicate_timestamps: self.duplicate_timestamps,
        }
    }
}

fn push_sample(samples: &mut VecDeque<u64>, value: Duration) {
    while samples.len() >= STAGE_SAMPLE_CAPACITY {
        samples.pop_front();
    }
    samples.push_back(value.as_nanos().try_into().unwrap_or(u64::MAX));
}

fn summarize(samples: &VecDeque<u64>) -> Option<DecodeStagePercentiles> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted: Vec<u64> = samples.iter().copied().collect();
    sorted.sort_unstable();
    Some(DecodeStagePercentiles {
        p50_us: percentile_us(&sorted, 50),
        p95_us: percentile_us(&sorted, 95),
        max_us: *sorted.last().expect("non-empty") / 1_000,
    })
}

fn percentile_us(sorted: &[u64], percentile: u32) -> u64 {
    let rank = (sorted.len() * percentile as usize).div_ceil(100).max(1);
    sorted[rank.min(sorted.len()) - 1] / 1_000
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(base: Instant, millis: u64) -> Instant {
        base + Duration::from_millis(millis)
    }

    #[test]
    fn empty_probe_reports_no_measured_stage() {
        let timings = DecodeTimingProbe::default().snapshot();
        assert!(timings.is_empty());
        assert!(!timings.has_progress());
        assert_eq!(timings.call_window_samples, 0);
        assert_eq!(timings.residence_window_samples, 0);
        assert_eq!(timings.submissions_total, 0);
        assert_eq!(timings.outputs_total, 0);
        assert_eq!(timings.output_calls_total, 0);
        assert!(timings.last_submission_at.is_none());
        assert!(timings.last_output_at.is_none());
        assert_eq!(timings.in_flight, 0);
        assert_eq!(timings.unmatched_outputs, 0);
        assert_eq!(timings.unmatched_submissions, 0);
        assert_eq!(timings.duplicate_timestamps, 0);
    }

    #[test]
    fn matched_submission_and_output_measure_residence() {
        let base = Instant::now();
        let mut accumulator = DecodeTimingAccumulator::default();
        accumulator.record_submission(4_500, base);
        accumulator.record_output(4_500, at(base, 7));
        let timings = accumulator.snapshot();
        let residence = timings.residence.expect("residence sample");
        assert_eq!(residence.p50_us, 7_000);
        assert_eq!(residence.p95_us, 7_000);
        assert_eq!(residence.max_us, 7_000);
        assert_eq!(timings.in_flight, 0);
        assert_eq!(timings.unmatched_outputs, 0);
        assert_eq!(timings.submissions_total, 1);
        assert_eq!(timings.outputs_total, 1);
        assert_eq!(timings.last_submission_at, Some(base));
        assert_eq!(timings.last_output_at, Some(at(base, 7)));
        assert!(timings.has_progress());
    }

    #[test]
    fn submit_only_calls_are_not_reported_as_decode_time() {
        let mut accumulator = DecodeTimingAccumulator::default();
        accumulator.record_call(Duration::from_micros(300), false);
        accumulator.record_call(Duration::from_micros(2_500), true);
        let timings = accumulator.snapshot();
        let call = timings.call.expect("call sample");
        assert_eq!(call.p50_us, 2_500);
        assert_eq!(timings.call_window_samples, 1);
        assert_eq!(timings.output_calls_total, 1);
    }

    #[test]
    fn output_without_a_recorded_submission_is_counted_not_estimated() {
        let mut accumulator = DecodeTimingAccumulator::default();
        accumulator.record_output(9_000, Instant::now());
        let timings = accumulator.snapshot();
        assert!(timings.residence.is_none());
        assert_eq!(timings.unmatched_outputs, 1);
    }

    #[test]
    fn earlier_in_flight_submissions_survive_a_later_output() {
        let base = Instant::now();
        let mut accumulator = DecodeTimingAccumulator::default();
        accumulator.record_submission(1_000, base);
        accumulator.record_submission(2_000, at(base, 1));
        accumulator.record_output(2_000, at(base, 5));
        let timings = accumulator.snapshot();
        assert_eq!(
            timings.residence.expect("residence").p50_us,
            4_000,
            "the matched submission is the one for this output"
        );
        assert_eq!(
            timings.in_flight, 1,
            "an earlier submission is not retired by a later output"
        );
        assert_eq!(timings.unmatched_submissions, 0);
        accumulator.record_output(1_000, at(base, 30));
        let timings = accumulator.snapshot();
        assert_eq!(
            timings.residence.expect("residences").max_us,
            30_000,
            "the earlier submission still produces its own residence sample"
        );
        assert_eq!(timings.in_flight, 0);
        assert_eq!(timings.outputs_total, 2);
    }

    #[test]
    fn out_of_order_outputs_match_their_own_submissions() {
        let base = Instant::now();
        let mut accumulator = DecodeTimingAccumulator::default();
        for timestamp in [1_000, 2_000, 3_000] {
            accumulator.record_submission(timestamp, base);
        }
        accumulator.record_output(3_000, at(base, 10));
        accumulator.record_output(1_000, at(base, 20));
        accumulator.record_output(2_000, at(base, 30));
        let timings = accumulator.snapshot();
        assert_eq!(timings.in_flight, 0);
        assert_eq!(timings.unmatched_outputs, 0);
        assert_eq!(timings.outputs_total, 3);
        let residence = timings.residence.expect("residence samples");
        assert_eq!(residence.max_us, 30_000);
        assert_eq!(residence.p50_us, 20_000);
        assert_eq!(residence.p95_us, 30_000);
    }

    #[test]
    fn unmatched_outputs_still_count_as_decoder_progress() {
        let base = Instant::now();
        let mut accumulator = DecodeTimingAccumulator::default();
        accumulator.record_output(9_000, base);
        let timings = accumulator.snapshot();
        assert_eq!(timings.unmatched_outputs, 1);
        assert_eq!(timings.outputs_total, 1);
        assert_eq!(timings.last_output_at, Some(base));
        assert!(timings.residence.is_none());
        assert!(timings.has_progress());
    }

    #[test]
    fn duplicate_identifiers_are_reported_and_matched_oldest_first() {
        let base = Instant::now();
        let mut accumulator = DecodeTimingAccumulator::default();
        accumulator.record_submission(3_000, base);
        accumulator.record_submission(3_000, at(base, 10));
        accumulator.record_output(3_000, at(base, 20));
        let timings = accumulator.snapshot();
        assert_eq!(timings.duplicate_timestamps, 1);
        assert_eq!(
            timings.residence.expect("residence").p50_us,
            20_000,
            "the oldest in-flight submission owns the earlier frame"
        );
        assert_eq!(timings.in_flight, 1);
    }

    #[test]
    fn in_flight_queue_stays_bounded_and_reports_evictions() {
        let mut accumulator = DecodeTimingAccumulator::default();
        for timestamp in 0..(IN_FLIGHT_CAPACITY as u64 + 4) {
            accumulator.record_submission(timestamp, Instant::now());
        }
        let timings = accumulator.snapshot();
        assert_eq!(timings.in_flight, IN_FLIGHT_CAPACITY);
        assert_eq!(timings.unmatched_submissions, 4);
    }

    #[test]
    fn clear_starts_a_new_epoch_and_accounts_retired_submissions() {
        let base = Instant::now();
        let mut accumulator = DecodeTimingAccumulator::default();
        accumulator.record_submission(700, base);
        accumulator.record_output(700, at(base, 3));
        accumulator.record_call(Duration::from_millis(2), true);
        accumulator.record_submission(800, base);
        assert_eq!(accumulator.snapshot().epoch, 0);
        accumulator.clear();
        let timings = accumulator.snapshot();
        assert_eq!(timings.in_flight, 0);
        assert_eq!(timings.oldest_in_flight_at, None);
        assert_eq!(timings.epoch, 1);
        assert!(timings.epoch_started_at.is_some());
        assert_eq!(
            timings.unmatched_submissions, 1,
            "the dropped in-flight submission is accounted, not silently forgotten"
        );
        assert_eq!(timings.call_window_samples, 1);
        assert_eq!(timings.residence_window_samples, 1);
        assert_eq!(timings.outputs_total, 1);
        accumulator.record_output(800, at(base, 9));
        assert_eq!(accumulator.snapshot().unmatched_outputs, 1);
    }

    #[test]
    fn a_decoder_hung_on_its_first_submission_is_still_observable() {
        let base = Instant::now();
        let mut accumulator = DecodeTimingAccumulator::default();
        accumulator.record_submission(1_000, base);
        let timings = accumulator.snapshot();
        assert!(timings.is_empty());
        assert!(!timings.has_progress());
        assert!(
            timings.has_observable_state(),
            "input with no output yet is decoder work Task3 must be able to see"
        );
        assert_eq!(timings.submissions_total, 1);
        assert_eq!(timings.outputs_total, 0);
        assert_eq!(timings.in_flight, 1);
        assert_eq!(timings.progress_reference(), Some(base));
    }

    #[test]
    fn progress_reference_ignores_outputs_from_before_the_epoch() {
        let base = Instant::now();
        let mut accumulator = DecodeTimingAccumulator::default();
        accumulator.record_submission(1_000, base);
        accumulator.record_output(1_000, at(base, 50));
        accumulator.clear();
        let cleared = accumulator.snapshot();
        let epoch_started = cleared.epoch_started_at.expect("epoch boundary");
        accumulator.record_submission(2_000, at(base, 60));
        let timings = accumulator.snapshot();
        let reference = timings.progress_reference().expect("outstanding work");
        assert!(
            reference >= epoch_started,
            "a stale pre-epoch output must not make the decoder look like it just progressed"
        );
        assert_eq!(reference, at(base, 60));
    }

    #[test]
    fn progress_reference_is_absent_without_outputs_or_outstanding_work() {
        let accumulator = DecodeTimingAccumulator::default();
        assert_eq!(accumulator.snapshot().progress_reference(), None);
    }

    #[test]
    fn stage_rings_stay_bounded_under_continued_load() {
        let base = Instant::now();
        let mut accumulator = DecodeTimingAccumulator::default();
        for timestamp in 0..(STAGE_SAMPLE_CAPACITY as u64 * 2) {
            accumulator.record_submission(timestamp, base);
            accumulator.record_output(timestamp, at(base, 1));
            accumulator.record_call(Duration::from_micros(500), true);
        }
        let timings = accumulator.snapshot();
        assert_eq!(timings.call_window_samples, STAGE_SAMPLE_CAPACITY);
        assert_eq!(timings.residence_window_samples, STAGE_SAMPLE_CAPACITY);
        assert_eq!(timings.unmatched_outputs, 0);
        assert_eq!(timings.unmatched_submissions, 0);
        assert_eq!(
            timings.outputs_total,
            STAGE_SAMPLE_CAPACITY as u64 * 2,
            "cumulative output totals keep counting after the window saturates"
        );
        assert_eq!(timings.submissions_total, STAGE_SAMPLE_CAPACITY as u64 * 2);
    }

    #[test]
    fn percentiles_use_nearest_rank_measured_values() {
        let mut accumulator = DecodeTimingAccumulator::default();
        for millis in 1..=20 {
            accumulator.record_call(Duration::from_millis(millis), true);
        }
        let call = accumulator.snapshot().call.expect("call samples");
        assert_eq!(call.p50_us, 10_000);
        assert_eq!(call.p95_us, 19_000);
        assert_eq!(call.max_us, 20_000);
    }
}

// Decoder-local, opt-in host timings. These never enter the shared rolling probe.
#[cfg(all(feature = "ffmpeg", feature = "vulkan"))]
pub(crate) mod vulkan_frame {
    use super::{Duration, Instant, STAGE_SAMPLE_CAPACITY, percentile_us};
    use crate::{StreamFormat, VideoCodec};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // A copy attempt owns one observation, including a failed wait. Consumers
    // must take it after that attempt; neither validation failure nor retries
    // may reuse the preceding frame's wait.
    #[derive(Default)]
    pub(crate) struct FenceWaitObservation {
        enabled: bool,
        latest: Option<Duration>,
    }
    impl FenceWaitObservation {
        pub(crate) fn enable(&mut self) {
            self.enabled = true;
        }
        pub(crate) fn begin_attempt(&mut self) {
            self.latest = None;
        }
        pub(crate) fn measure<T>(&mut self, wait: impl FnOnce() -> T) -> T {
            let started = self.enabled.then(Instant::now);
            let result = wait();
            self.latest = started.map(|at| at.elapsed());
            result
        }
        pub(crate) fn take(&mut self) -> Option<Duration> {
            self.latest.take()
        }
    }

    pub(crate) struct Attempt {
        pub(crate) start: Instant,
        utc_start_ns: Option<u64>,
        prefix: bool,
    }
    #[derive(Debug)]
    pub(crate) struct Batch {
        pub(crate) decoder: u64,
        pub(crate) seq: u64,
        prefix_frames: usize,
        pub(crate) format: StreamFormat,
        pub(crate) first_frame: u64,
        pub(crate) last_frame: u64,
        pub(crate) utc_start_ns: Option<u64>,
        pub(crate) utc_end_ns: Option<u64>,
        pub(crate) monotonic_start_ns: u64,
        pub(crate) monotonic_end_ns: u64,
        pub(crate) sums_ns: [u64; 3],
        pub(crate) quantiles_us: [[u64; 3]; 3],
    }
    pub(crate) struct Probe {
        decoder: u64,
        codec: VideoCodec,
        origin: Instant,
        format: Option<StreamFormat>,
        pairs: [[u64; 3]; STAGE_SAMPLE_CAPACITY],
        len: usize,
        prefix_in_batch: usize,
        pub(crate) submissions: u64,
        pub(crate) submission_errors: u64,
        prefix_outputs: u64,
        extra_outputs: u64,
        prefix_paired: u64,
        pub(crate) no_output_submissions: u64,
        discarded_prefix: u64,
        first_frame: u64,
        utc_start_ns: Option<u64>,
        monotonic_start_ns: u64,
        pub(crate) outputs: u64,
        pub(crate) paired: u64,
        pub(crate) batches: u64,
        pub(crate) discarded: u64,
        pub(crate) format_resets: u64,
        pub(crate) receive_errors: u64,
        pub(crate) conversion_errors: u64,
        pub(crate) no_output: u64,
        pub(crate) missing_wait: u64,
        pub(crate) pair_errors: u64,
        pub(crate) clock_errors: u64,
        log_errors: u64,
    }
    fn utc_ns() -> Option<u64> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos()
            .try_into()
            .ok()
    }
    fn ns(d: Duration) -> u64 {
        d.as_nanos().try_into().unwrap_or(u64::MAX)
    }
    fn project_utc_start(utc: u64, anchor: Instant, start: Instant) -> Option<u64> {
        utc.checked_sub(ns(anchor.checked_duration_since(start)?))
    }
    impl Probe {
        pub(crate) fn new(enabled: bool, codec: VideoCodec) -> Option<Box<Self>> {
            if !enabled {
                return None;
            }
            static NEXT: AtomicU64 = AtomicU64::new(1);
            Some(Box::new(Self {
                decoder: NEXT.fetch_add(1, Ordering::Relaxed),
                codec,
                origin: Instant::now(),
                format: None,
                pairs: [[0; 3]; STAGE_SAMPLE_CAPACITY],
                len: 0,
                prefix_in_batch: 0,
                submissions: 0,
                submission_errors: 0,
                prefix_outputs: 0,
                extra_outputs: 0,
                prefix_paired: 0,
                no_output_submissions: 0,
                discarded_prefix: 0,
                first_frame: 0,
                utc_start_ns: None,
                monotonic_start_ns: 0,
                outputs: 0,
                paired: 0,
                batches: 0,
                discarded: 0,
                format_resets: 0,
                receive_errors: 0,
                conversion_errors: 0,
                no_output: 0,
                missing_wait: 0,
                pair_errors: 0,
                clock_errors: 0,
                log_errors: 0,
            }))
        }
        pub(crate) fn begin(&self) -> Attempt {
            self.begin_kind(false)
        }
        pub(crate) fn begin_prefix(&mut self) -> Attempt {
            self.submissions += 1;
            self.begin_kind(true)
        }
        fn begin_kind(&self, prefix: bool) -> Attempt {
            let utc_start_ns = (self.len == 0).then(utc_ns).flatten();
            Attempt {
                start: Instant::now(),
                utc_start_ns,
                prefix,
            }
        }
        pub(crate) fn next_attempt(&self, prefix: &mut Option<Attempt>) -> Attempt {
            // The submission prefix is consumed ONCE, even on EAGAIN/error.
            // Further outputs in the same drain are receive-only observations.
            prefix.take().unwrap_or_else(|| self.begin())
        }
        pub(crate) fn note_no_output(&mut self, attempt: Option<&Attempt>) {
            self.no_output += 1;
            if attempt.is_some_and(|attempt| attempt.prefix) {
                self.no_output_submissions += 1;
            }
        }
        pub(crate) fn record(
            &mut self,
            format: Option<StreamFormat>,
            attempt: Attempt,
            received: Instant,
            converted: Instant,
            wait: Option<Duration>,
        ) -> Option<Batch> {
            self.record_with_anchor(format, attempt, received, converted, wait, || {
                let utc = utc_ns()?;
                Some((utc, Instant::now()))
            })
        }
        fn record_with_anchor(
            &mut self,
            format: Option<StreamFormat>,
            attempt: Attempt,
            received: Instant,
            converted: Instant,
            wait: Option<Duration>,
            anchor: impl FnOnce() -> Option<(u64, Instant)>,
        ) -> Option<Batch> {
            let Some(format) = format else {
                self.conversion_errors += 1;
                if attempt.prefix {
                    self.no_output_submissions += 1;
                }
                return None;
            };
            self.outputs += 1;
            if attempt.prefix {
                self.prefix_outputs += 1;
            } else {
                self.extra_outputs += 1;
            }
            if self.format.is_some_and(|old| old != format) {
                self.discarded += self.len as u64;
                self.discarded_prefix += self.prefix_in_batch as u64;
                self.len = 0;
                self.prefix_in_batch = 0;
                self.format_resets += 1;
            }
            self.format = Some(format);
            let Some(wait) = wait else {
                self.missing_wait += 1;
                return None;
            };
            let receive = received.saturating_duration_since(attempt.start);
            let conversion = converted.saturating_duration_since(received);
            if wait > conversion || received < attempt.start || converted < received {
                self.pair_errors += 1;
                return None;
            }
            if self.len == 0 {
                self.first_frame = self.outputs;
                // Fresh UTC followed immediately by a monotonic anchor. Using
                // anchor-start (NOT converted-start) includes delayed recording
                // and conservatively bounds the first frame before a reset.
                self.utc_start_ns = attempt.utc_start_ns.or_else(|| {
                    let (utc, at) = anchor()?;
                    project_utc_start(utc, at, attempt.start)
                });
                self.monotonic_start_ns = ns(attempt.start.saturating_duration_since(self.origin));
            }
            self.pairs[self.len] = [ns(receive), ns(conversion), ns(wait)];
            self.len += 1;
            self.paired += 1;
            if attempt.prefix {
                self.prefix_paired += 1;
                self.prefix_in_batch += 1;
            }
            if self.len != STAGE_SAMPLE_CAPACITY {
                return None;
            }
            let utc_end_ns = utc_ns();
            if self.utc_start_ns.is_none() || utc_end_ns.is_none() || utc_end_ns < self.utc_start_ns
            {
                self.clock_errors += 1;
            }
            let mut sums_ns = [0u64; 3];
            let mut quantiles_us = [[0; 3]; 3];
            let mut sorted = [0; STAGE_SAMPLE_CAPACITY];
            for stage in 0..3 {
                for (i, pair) in self.pairs.iter().enumerate() {
                    sorted[i] = pair[stage];
                    sums_ns[stage] = sums_ns[stage].saturating_add(pair[stage]);
                }
                sorted.sort_unstable();
                quantiles_us[stage] = [
                    percentile_us(&sorted, 50),
                    percentile_us(&sorted, 95),
                    sorted[STAGE_SAMPLE_CAPACITY - 1] / 1_000,
                ];
            }
            self.batches += 1;
            let prefix_frames = self.prefix_in_batch;
            self.len = 0;
            self.prefix_in_batch = 0;
            Some(Batch {
                decoder: self.decoder,
                seq: self.batches,
                prefix_frames,
                format,
                first_frame: self.first_frame,
                last_frame: self.outputs,
                utc_start_ns: self.utc_start_ns,
                utc_end_ns,
                monotonic_start_ns: self.monotonic_start_ns,
                monotonic_end_ns: ns(converted.saturating_duration_since(self.origin)),
                sums_ns,
                quantiles_us,
            })
        }
        #[cfg(test)]
        pub(crate) fn prefix_counts(&self) -> (u64, u64, u64) {
            (self.prefix_outputs, self.extra_outputs, self.prefix_paired)
        }

        #[cfg(test)]
        pub(crate) fn bound_format(&self) -> Option<StreamFormat> {
            self.format
        }

        fn coverage(&self) -> String {
            format!(
                "outputs={} paired={} batches={} partial={} discarded={} formatResets={} receiveErrors={} conversionErrors={} noOutputReceives={} missingWait={} pairErrors={} clockErrors={} logErrors={} submissions={} submissionErrors={} prefixOutputs={} extraOutputs={} prefixPaired={} noOutputSubmissions={} discardedPrefix={}",
                self.outputs,
                self.paired,
                self.batches,
                self.len,
                self.discarded,
                self.format_resets,
                self.receive_errors,
                self.conversion_errors,
                self.no_output,
                self.missing_wait,
                self.pair_errors,
                self.clock_errors,
                self.log_errors,
                self.submissions,
                self.submission_errors,
                self.prefix_outputs,
                self.extra_outputs,
                self.prefix_paired,
                self.no_output_submissions,
                self.discarded_prefix
            )
        }
        pub(crate) fn emit(&mut self, batch: &Batch, writer: &mut impl std::io::Write) {
            // Best-effort existing native stderr seam, captured privately by
            // run-native.sh. Serialization/write are OUTSIDE measured stages,
            // but synchronous writing can still perturb this worker.
            let line = format!("{} {}\n", batch.line(self.codec), self.coverage());
            if writer.write_all(line.as_bytes()).is_err() {
                self.log_errors += 1;
            }
        }
        fn close(&mut self, writer: &mut impl std::io::Write) {
            let partial = self.len;
            self.discarded += partial as u64;
            self.discarded_prefix += self.prefix_in_batch as u64;
            self.len = 0;
            self.prefix_in_batch = 0;
            let binding = self.format.map_or_else(|| "bound=0".to_owned(), |f|
                format!("bound=1 mode=vulkan-shared device=adopted output=VULKAN width={} height={} format={:?}", f.width, f.height, f.pixel_format));
            let line = format!(
                "type=vulkan-frame-timing schema=2 status=closed pid={} decoder={} codec={} {} partialDiscarded={} {}\n",
                std::process::id(),
                self.decoder,
                self.codec.label(),
                binding,
                partial,
                self.coverage()
            );
            let _ = writer.write_all(line.as_bytes());
        }
    }
    impl Batch {
        fn line(&self, codec: VideoCodec) -> String {
            let mut line = format!(
                "type=vulkan-frame-timing schema=2 status=complete pid={} decoder={} seq={} mode=vulkan-shared device=adopted output=VULKAN codec={} width={} height={} format={:?} samples={} prefixFrames={} receiveOnlyFrames={} firstFrame={} lastFrame={} utcStartNs={} utcEndNs={} monotonicStartNs={} monotonicEndNs={}",
                std::process::id(),
                self.decoder,
                self.seq,
                codec.label(),
                self.format.width,
                self.format.height,
                self.format.pixel_format,
                STAGE_SAMPLE_CAPACITY,
                self.prefix_frames,
                STAGE_SAMPLE_CAPACITY - self.prefix_frames,
                self.first_frame,
                self.last_frame,
                self.utc_start_ns.unwrap_or(0),
                self.utc_end_ns.unwrap_or(0),
                self.monotonic_start_ns,
                self.monotonic_end_ns
            );
            for (i, label) in ["sendReceive", "conversion", "fenceWait"]
                .iter()
                .enumerate()
            {
                use std::fmt::Write;
                let q = self.quantiles_us[i];
                let _ = write!(
                    line,
                    " {label}SumNs={} {label}P50Us={} {label}P95Us={} {label}MaxUs={}",
                    self.sums_ns[i], q[0], q[1], q[2]
                );
            }
            line
        }
    }
    impl Drop for Probe {
        fn drop(&mut self) {
            self.close(&mut std::io::stderr().lock());
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        fn feed(
            p: &mut Probe,
            format: StreamFormat,
            index: u64,
            wait: Option<Duration>,
        ) -> Option<Batch> {
            let start = p.origin + Duration::from_millis(index);
            p.record(
                Some(format),
                Attempt {
                    start,
                    utc_start_ns: Some(1),
                    prefix: false,
                },
                start + Duration::from_micros(2),
                start + Duration::from_micros(12),
                wait,
            )
        }
        #[test]
        fn pairs_are_bounded_complete_nonoverlap_and_format_reset_discards_partial() {
            let f = StreamFormat::video_default(256, 144).unwrap();
            let mut p = Probe::new(true, VideoCodec::H265).unwrap();
            for i in 0..255 {
                assert!(feed(&mut p, f, i, Some(Duration::from_micros(7))).is_none());
            }
            let first = feed(&mut p, f, 255, Some(Duration::from_micros(7))).unwrap();
            assert_eq!(first.sums_ns, [512_000, 2_560_000, 1_792_000]);
            assert_eq!(first.quantiles_us, [[2, 2, 2], [10, 10, 10], [7, 7, 7]]);
            assert_eq!(
                (first.first_frame, first.last_frame, first.seq, p.len),
                (1, 256, 1, 0)
            );
            for i in 256..512 {
                let b = feed(&mut p, f, i, Some(Duration::from_micros(3)));
                if i == 511 {
                    let b = b.unwrap();
                    assert_eq!((b.first_frame, b.last_frame, b.seq), (257, 512, 2));
                    assert_eq!(b.sums_ns[2], 768_000);
                } else {
                    assert!(b.is_none());
                }
            }
            feed(&mut p, f, 512, Some(Duration::from_micros(7)));
            let changed = StreamFormat::video_default(512, 144).unwrap();
            feed(&mut p, changed, 513, Some(Duration::from_micros(7)));
            assert_eq!((p.discarded, p.format_resets, p.len), (1, 1, 1));
            for i in 514..769 {
                if let Some(b) = feed(&mut p, changed, i, Some(Duration::from_micros(7))) {
                    assert_eq!(
                        (b.format.width, b.first_frame, b.last_frame),
                        (512, 514, 769)
                    );
                    let line = b.line(VideoCodec::H265);
                    assert!(
                        line.contains("mode=vulkan-shared device=adopted output=VULKAN codec=h265")
                    );
                    assert!(line.contains(
                        "prefixFrames=0 receiveOnlyFrames=256 firstFrame=514 lastFrame=769"
                    ));
                    assert!(line.contains("fenceWaitSumNs=1792000"));
                }
            }
            assert_eq!((p.batches, p.paired, p.len), (3, 769, 0));
        }
        #[test]
        fn reset_without_attempt_utc_accounts_for_delayed_recording() {
            let mut p = Probe::new(true, VideoCodec::H265).unwrap();
            let old = StreamFormat::video_default(256, 144).unwrap();
            let new = StreamFormat::video_default(512, 144).unwrap();
            feed(&mut p, old, 0, Some(Duration::from_micros(7)));
            let start = p.origin + Duration::from_micros(100);
            let received = start + Duration::from_micros(2);
            let converted = start + Duration::from_micros(12);
            let anchor = p.origin + Duration::from_micros(1000); // 888 us after conversion
            assert!(
                p.record_with_anchor(
                    Some(new),
                    Attempt {
                        start,
                        utc_start_ns: None,
                        prefix: true
                    },
                    received,
                    converted,
                    Some(Duration::from_micros(7)),
                    || Some((10_000_000, anchor))
                )
                .is_none()
            );
            assert_eq!(
                (p.format_resets, p.discarded, p.len, p.utc_start_ns),
                (1, 1, 1, Some(9_100_000))
            );
            // The old projection would yield 9_988_000 and falsely include
            // this batch in a window starting at 9_500_000.
            assert!(p.utc_start_ns.unwrap() < 9_500_000);
            let mut batch = None;
            for i in 1..256 {
                batch = feed(&mut p, new, i, Some(Duration::from_micros(7)));
            }
            let batch = batch.unwrap();
            assert_eq!(
                (batch.utc_start_ns, batch.prefix_frames),
                (Some(9_100_000), 1)
            );
        }
        #[test]
        fn submission_prefix_is_charged_once_and_no_output_calls_remain_visible() {
            let mut p = Probe::new(true, VideoCodec::H265).unwrap();
            let f = StreamFormat::video_default(256, 144).unwrap();
            let mut prefix = Some(p.begin_prefix());
            let first = p.next_attempt(&mut prefix);
            assert!(first.prefix);
            assert!(prefix.is_none());
            let start = first.start;
            p.record(
                Some(f),
                first,
                start + Duration::from_micros(11),
                start + Duration::from_micros(21),
                Some(Duration::from_micros(7)),
            );
            for _ in 0..2 {
                let next = p.next_attempt(&mut prefix);
                assert!(!next.prefix);
                let start = next.start;
                p.record(
                    Some(f),
                    next,
                    start + Duration::from_micros(2),
                    start + Duration::from_micros(12),
                    Some(Duration::from_micros(7)),
                );
            }
            assert_eq!(
                (
                    p.submissions,
                    p.prefix_outputs,
                    p.extra_outputs,
                    p.prefix_in_batch
                ),
                (1, 1, 2, 1)
            );
            assert_eq!(p.pairs[..3].iter().map(|pair| pair[0]).sum::<u64>(), 15_000);
            let mut empty = Some(p.begin_prefix());
            let attempt = p.next_attempt(&mut empty);
            p.note_no_output(Some(&attempt));
            let next = p.next_attempt(&mut empty);
            p.note_no_output(Some(&next));
            assert_eq!(
                (
                    p.submissions,
                    p.no_output_submissions,
                    p.no_output,
                    p.paired
                ),
                (2, 1, 2, 3)
            );
            let mut batch = None;
            for i in 3..256 {
                batch = feed(&mut p, f, i, Some(Duration::from_micros(7)));
            }
            let batch = batch.unwrap();
            assert_eq!(batch.prefix_frames, 1);
            assert!(
                batch
                    .line(VideoCodec::H265)
                    .contains("samples=256 prefixFrames=1 receiveOnlyFrames=255")
            );
        }
        #[test]
        fn logging_failure_and_destruction_keep_explicit_coverage_without_changing_pairs() {
            struct Broken;
            impl std::io::Write for Broken {
                fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                    Err(std::io::ErrorKind::BrokenPipe.into())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }
            let mut p = Probe::new(true, VideoCodec::H265).unwrap();
            let f = StreamFormat::video_default(256, 144).unwrap();
            let mut batch = None;
            for i in 0..256 {
                batch = feed(&mut p, f, i, Some(Duration::from_micros(7)));
            }
            let batch = batch.unwrap();
            p.emit(&batch, &mut Broken);
            assert_eq!(
                (p.log_errors, p.outputs, p.paired, p.batches, p.len),
                (1, 256, 256, 1, 0)
            );
            let mut bytes = Vec::new();
            p.emit(&batch, &mut bytes);
            let line = String::from_utf8(bytes).unwrap();
            assert!(
                line.contains("prefixFrames=0 receiveOnlyFrames=256 firstFrame=1 lastFrame=256")
            );
            assert!(line.contains("sendReceiveSumNs=512000"));
            assert!(line.contains("logErrors=1"));
            feed(&mut p, f, 256, Some(Duration::from_micros(7)));
            let mut bytes = Vec::new();
            p.close(&mut bytes);
            let closed = String::from_utf8(bytes).unwrap();
            assert!(closed.contains(
                "bound=1 mode=vulkan-shared device=adopted output=VULKAN width=256 height=144"
            ));
            assert!(closed.contains("partialDiscarded=1"));
            assert!(closed.contains("partial=0 discarded=1"));
            assert_eq!((p.outputs, p.paired, p.discarded), (257, 257, 1));
            println!(
                "fixed_probe_bytes={} pair_storage_bytes={}",
                std::mem::size_of::<Probe>(),
                std::mem::size_of::<[[u64; 3]; STAGE_SAMPLE_CAPACITY]>()
            );
        }
        #[test]
        fn disabled_and_failed_attempts_cannot_reuse_or_pollute_observations() {
            assert!(Probe::new(false, VideoCodec::H265).is_none());
            let mut wait = FenceWaitObservation::default();
            assert_eq!(wait.measure(|| Err::<(), _>(17)), Err(17));
            assert!(wait.take().is_none());
            wait.enable();
            assert_eq!(wait.measure(|| Err::<(), _>(23)), Err(23));
            assert!(wait.take().is_some());
            assert!(wait.take().is_none());
            wait.measure(|| ());
            wait.begin_attempt();
            assert!(wait.take().is_none());
            let mut p = Probe::new(true, VideoCodec::H265).unwrap();
            let f = StreamFormat::video_default(256, 144).unwrap();
            let start = p.origin;
            assert!(
                p.record(
                    None,
                    Attempt {
                        start,
                        utc_start_ns: Some(1),
                        prefix: false,
                    },
                    start + Duration::from_micros(2),
                    start + Duration::from_micros(12),
                    Some(Duration::from_micros(7))
                )
                .is_none()
            );
            assert_eq!(
                (p.outputs, p.paired, p.len, p.conversion_errors),
                (0, 0, 0, 1)
            );
            feed(&mut p, f, 0, None);
            feed(&mut p, f, 1, Some(Duration::from_micros(11)));
            for i in 2..258 {
                feed(&mut p, f, i, Some(Duration::from_micros(7)));
            }
            assert_eq!(
                (
                    p.outputs,
                    p.paired,
                    p.batches,
                    p.missing_wait,
                    p.pair_errors
                ),
                (258, 256, 1, 1, 1)
            );
            assert!(p.coverage().contains("conversionErrors=1"));
        }
    }
}
