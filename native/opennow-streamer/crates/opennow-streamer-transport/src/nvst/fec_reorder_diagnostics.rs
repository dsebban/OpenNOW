//! Bounded, payload-free FEC head observations. This observer owns no protocol state.
//! The receive owner supplies its existing clock and enables this only for an armed capture.
use std::path::PathBuf;
use std::time::Instant;

const EPISODE_CAPACITY: usize = 128;
const FEC_TUPLE_CAPACITY: usize = 32;
const SUCCESSOR_SAMPLE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Outcome {
    Completed,
    Expired,
    Reset,
    HeadChanged,
    CaptureEnd,
    GapFilled,
}

impl Outcome {
    fn label(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Expired => "expired",
            Self::Reset => "reset",
            Self::HeadChanged => "head_changed",
            Self::CaptureEnd => "capture_end",
            Self::GapFilled => "gap_filled",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum GapOrigin {
    IncompleteHead,
    WholeMissingBlock,
}

impl GapOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::IncompleteHead => "incomplete_fec_head",
            Self::WholeMissingBlock => "whole_missing_fec_block",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct WaitSnapshot {
    pub(super) gap_origin: GapOrigin,
    pub(super) base: u64,
    pub(super) frame_index: u32,
    pub(super) block_index: u8,
    pub(super) last_block_index: u8,
    pub(super) first_missing: u64,
    pub(super) last_missing: u64,
    pub(super) gap_started: Instant,
    pub(super) buffered_blocks: usize,
    /// Some only when needs_successor_sample requested a scan. None is unobserved, not zero.
    pub(super) ready_successors: Option<usize>,
}

struct Episode {
    gap_origin: GapOrigin,
    base: u64,
    frame_index: u32,
    block_index: u8,
    last_block_index: u8,
    observed_start: Instant,
    start_ns: u64,
    end_ns: Option<u64>,
    initial_missing: u64,
    final_missing: u64,
    last_missing: u64,
    gap_started: Instant,
    first_missing_changes: u64,
    deadline_restarts: u64,
    restarts_with_ready_successors: u64,
    successor_samples: u64,
    last_successor_sample: Option<Instant>,
    max_ready_successors: usize,
    max_buffered_blocks: usize,
    max_head_age_ns: u64,
    max_gap_age_ns: u64,
    preexisting: bool,
    outcome: Option<Outcome>,
    released_packets: usize,
    repaired: usize,
}

struct FecTuple {
    repair_percent: u8,
    data_shards: usize,
    parity_shards: usize,
    packets: u64,
}

struct RtpGap {
    first_missing: u64,
    last_missing: u64,
    observed_start: Instant,
    start_ns: u64,
    end_ns: Option<u64>,
    max_age_ns: u64,
    max_buffered_packets: usize,
    outcome: Option<Outcome>,
}

impl RtpGap {
    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "gapOrigin": "rtp_sequence_gap",
            "firstMissing": self.first_missing,
            "lastMissing": self.last_missing,
            "startNs": self.start_ns,
            "endNs": self.end_ns,
            "maxObservedAgeNs": self.max_age_ns,
            "maxBufferedPackets": self.max_buffered_packets,
            "outcome": self.outcome.map(Outcome::label),
        })
    }
}

pub(super) struct Probe {
    origin: Instant,
    realtime_origin_ns_approx: u64,
    output: PathBuf,
    preexisting_base: Option<u64>,
    episodes: Vec<Episode>,
    current: Option<Episode>,
    omitted: u64,
    unmatched_finishes: u64,
    fec_tuples: Vec<FecTuple>,
    last_fec_tuple: Option<usize>,
    fec_tuple_packets_omitted: u64,
    rtp_gaps: Vec<RtpGap>,
    current_rtp_gap: Option<RtpGap>,
    rtp_gaps_omitted: u64,
}

fn ns(duration: std::time::Duration) -> u64 {
    duration.as_nanos().try_into().unwrap_or(u64::MAX)
}

impl Probe {
    pub(super) fn new(origin: Instant, output: PathBuf, preexisting_base: Option<u64>) -> Self {
        let realtime_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let realtime_origin_ns_approx = realtime_ns
            .saturating_sub(origin.elapsed().as_nanos())
            .try_into()
            .unwrap_or(u64::MAX);
        Self {
            origin,
            realtime_origin_ns_approx,
            output,
            preexisting_base,
            episodes: Vec::with_capacity(EPISODE_CAPACITY),
            current: None,
            omitted: 0,
            unmatched_finishes: 0,
            fec_tuples: Vec::with_capacity(FEC_TUPLE_CAPACITY),
            last_fec_tuple: None,
            fec_tuple_packets_omitted: 0,
            rtp_gaps: Vec::with_capacity(EPISODE_CAPACITY),
            current_rtp_gap: None,
            rtp_gaps_omitted: 0,
        }
    }

    /// O(1). Sample immediately on transitions and at bounded 16 ms opportunities.
    pub(super) fn needs_successor_sample(
        &self,
        base: u64,
        first_missing: u64,
        gap_started: Instant,
        now: Instant,
    ) -> bool {
        self.current.as_ref().is_none_or(|head| {
            head.base != base
                || head.final_missing != first_missing
                || head.gap_started != gap_started
                || head.last_successor_sample.is_none_or(|sampled| {
                    now.saturating_duration_since(sampled) >= SUCCESSOR_SAMPLE_INTERVAL
                })
        })
    }

    pub(super) fn current_base(&self) -> Option<u64> {
        self.current.as_ref().map(|head| head.base)
    }

    pub(super) fn record_fec_tuple(
        &mut self,
        repair_percent: u8,
        data_shards: usize,
        parity_shards: usize,
    ) {
        let matches = |tuple: &FecTuple| {
            tuple.repair_percent == repair_percent
                && tuple.data_shards == data_shards
                && tuple.parity_shards == parity_shards
        };
        let index = self
            .last_fec_tuple
            .filter(|&index| matches(&self.fec_tuples[index]))
            .or_else(|| self.fec_tuples.iter().position(matches));
        if let Some(index) = index {
            self.fec_tuples[index].packets += 1;
            self.last_fec_tuple = Some(index);
        } else if self.fec_tuples.len() < FEC_TUPLE_CAPACITY {
            self.fec_tuples.push(FecTuple {
                repair_percent,
                data_shards,
                parity_shards,
                packets: 1,
            });
            self.last_fec_tuple = Some(self.fec_tuples.len() - 1);
        } else {
            self.fec_tuple_packets_omitted += 1;
        }
    }

    pub(super) fn finish_whole_gap_before(&mut self, available_base: u64, now: Instant) {
        if let Some(head) = self.current.as_ref()
            && head.gap_origin == GapOrigin::WholeMissingBlock
            && available_base <= head.base
        {
            self.finish_head(head.base, now, Outcome::GapFilled, 0, 0);
        }
    }

    pub(super) fn observe_rtp_gap(
        &mut self,
        first: u64,
        last: u64,
        gap_started: Instant,
        buffered: usize,
        now: Instant,
    ) {
        if self
            .current_rtp_gap
            .as_ref()
            .is_some_and(|gap| gap.first_missing != first)
        {
            self.finish_rtp_gap(now, Outcome::HeadChanged);
        }
        let gap = self.current_rtp_gap.get_or_insert_with(|| RtpGap {
            first_missing: first,
            last_missing: last,
            observed_start: gap_started.max(self.origin),
            start_ns: ns(gap_started.saturating_duration_since(self.origin)),
            end_ns: None,
            max_age_ns: 0,
            max_buffered_packets: 0,
            outcome: None,
        });
        gap.last_missing = last;
        gap.max_age_ns = gap
            .max_age_ns
            .max(ns(now.saturating_duration_since(gap.observed_start)));
        gap.max_buffered_packets = gap.max_buffered_packets.max(buffered);
    }

    pub(super) fn finish_rtp_gap(&mut self, now: Instant, outcome: Outcome) {
        if let Some(mut gap) = self.current_rtp_gap.take() {
            gap.max_age_ns = gap
                .max_age_ns
                .max(ns(now.saturating_duration_since(gap.observed_start)));
            gap.end_ns = Some(ns(now.saturating_duration_since(self.origin)));
            gap.outcome = Some(outcome);
            if self.rtp_gaps.len() < EPISODE_CAPACITY {
                self.rtp_gaps.push(gap);
            } else {
                self.rtp_gaps_omitted += 1;
            }
        }
    }

    #[cfg(test)]
    pub(super) fn test_episodes(&self) -> Vec<serde_json::Value> {
        self.episodes.iter().map(Episode::json).collect()
    }

    pub(super) fn observe_wait(&mut self, snapshot: WaitSnapshot, now: Instant) {
        if let Some(base) = self.current_base()
            && base != snapshot.base
        {
            self.finish_head(base, now, Outcome::HeadChanged, 0, 0);
        }
        if self.current.is_none() {
            let preexisting = self.preexisting_base.take() == Some(snapshot.base);
            let observed_start = if preexisting { self.origin } else { now };
            self.current = Some(Episode {
                gap_origin: snapshot.gap_origin,
                base: snapshot.base,
                frame_index: snapshot.frame_index,
                block_index: snapshot.block_index,
                last_block_index: snapshot.last_block_index,
                observed_start,
                start_ns: ns(observed_start.saturating_duration_since(self.origin)),
                end_ns: None,
                initial_missing: snapshot.first_missing,
                final_missing: snapshot.first_missing,
                last_missing: snapshot.last_missing,
                gap_started: snapshot.gap_started,
                first_missing_changes: 0,
                deadline_restarts: 0,
                restarts_with_ready_successors: 0,
                successor_samples: 0,
                last_successor_sample: None,
                max_ready_successors: 0,
                max_buffered_blocks: 0,
                max_head_age_ns: 0,
                max_gap_age_ns: 0,
                preexisting,
                outcome: None,
                released_packets: 0,
                repaired: 0,
            });
        }
        let head = self.current.as_mut().expect("observed head initialized");
        // Preserve the old timer's observed age before a renewal replaces it.
        head.max_gap_age_ns = head
            .max_gap_age_ns
            .max(ns(now.saturating_duration_since(head.gap_started)));
        if head.final_missing != snapshot.first_missing {
            head.first_missing_changes += 1;
        }
        if snapshot.gap_started > head.gap_started {
            head.deadline_restarts += 1;
            if snapshot.ready_successors.is_some_and(|count| count > 0) {
                head.restarts_with_ready_successors += 1;
            }
        }
        head.final_missing = snapshot.first_missing;
        head.last_missing = snapshot.last_missing;
        head.gap_started = snapshot.gap_started;
        head.max_head_age_ns = head
            .max_head_age_ns
            .max(ns(now.saturating_duration_since(head.observed_start)));
        head.max_buffered_blocks = head.max_buffered_blocks.max(snapshot.buffered_blocks);
        if let Some(count) = snapshot.ready_successors {
            head.last_successor_sample = Some(now);
            head.successor_samples += 1;
            head.max_ready_successors = head.max_ready_successors.max(count);
        }
    }

    pub(super) fn finish_head(
        &mut self,
        base: u64,
        now: Instant,
        outcome: Outcome,
        released_packets: usize,
        repaired: usize,
    ) {
        if self.current_base() != Some(base) {
            // Most healthy heads complete without ever blocking a successor.
            // Only a conflicting active observation indicates a missing hook.
            if self.current.is_some() {
                self.unmatched_finishes += 1;
            }
            if self.preexisting_base == Some(base) {
                self.preexisting_base = None;
            }
            return;
        }
        let mut head = self.current.take().expect("matching observed head");
        head.max_head_age_ns = head
            .max_head_age_ns
            .max(ns(now.saturating_duration_since(head.observed_start)));
        head.max_gap_age_ns = head
            .max_gap_age_ns
            .max(ns(now.saturating_duration_since(head.gap_started)));
        head.end_ns = Some(ns(now.saturating_duration_since(self.origin)));
        head.outcome = Some(outcome);
        head.released_packets = released_packets;
        head.repaired = repaired;
        if self.episodes.len() < EPISODE_CAPACITY {
            self.episodes.push(head);
        } else {
            self.omitted += 1;
        }
    }

    pub(super) fn finish(mut self, now: Instant, complete: bool) {
        // Keep the terminal head separate, so a full completed-episode buffer cannot hide it.
        if let Some(head) = self.current.as_mut() {
            head.max_head_age_ns = head
                .max_head_age_ns
                .max(ns(now.saturating_duration_since(head.observed_start)));
            head.max_gap_age_ns = head
                .max_gap_age_ns
                .max(ns(now.saturating_duration_since(head.gap_started)));
            head.end_ns = Some(ns(now.saturating_duration_since(self.origin)));
            head.outcome = Some(Outcome::CaptureEnd);
        }
        if let Some(gap) = self.current_rtp_gap.as_mut() {
            gap.max_age_ns = gap
                .max_age_ns
                .max(ns(now.saturating_duration_since(gap.observed_start)));
            gap.end_ns = Some(ns(now.saturating_duration_since(self.origin)));
            gap.outcome = Some(Outcome::CaptureEnd);
        }
        let elapsed_ns = ns(now.saturating_duration_since(self.origin));
        let result = std::thread::Builder::new()
            .name("nvst-fec-head-export".into())
            .spawn(move || {
                use std::io::Write;
                use std::os::unix::fs::OpenOptionsExt;
                let exported = (|| -> std::io::Result<()> {
                    if !self.output.is_absolute() {
                        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
                    }
                    let file = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .mode(0o600)
                        .open(&self.output)?;
                    let mut writer = std::io::BufWriter::new(file);
                    let episodes: Vec<_> = self.episodes.iter().map(Episode::json).collect();
                    let value = serde_json::json!({
                        "schema": 2,
                        "type": "fec-head-diagnostics",
                        "complete": complete,
                        "elapsedNs": elapsed_ns,
                        "realtimeOriginNsApprox": self.realtime_origin_ns_approx,
                        "episodeCapacity": EPISODE_CAPACITY,
                        "successorSampleIntervalNs": ns(SUCCESSOR_SAMPLE_INTERVAL),
                        "episodesOmitted": self.omitted,
                        "unmatchedFinishes": self.unmatched_finishes,
                        "unobservedPreexistingBase": self.preexisting_base,
                        "episodes": episodes,
                        "current": self.current.as_ref().map(Episode::json),
                        "fecTupleCapacity": FEC_TUPLE_CAPACITY,
                        "fecTuplePacketsOmitted": self.fec_tuple_packets_omitted,
                        "fecTupleCountUnit": "authenticated packets with a valid FEC layout, including duplicates and stale packets",
                        "fecTuples": self.fec_tuples.iter().map(|tuple| serde_json::json!({
                            "repairPercent": tuple.repair_percent, "dataShards": tuple.data_shards,
                            "parityShards": tuple.parity_shards, "packets": tuple.packets,
                        })).collect::<Vec<_>>(),
                        "rtpGaps": self.rtp_gaps.iter().map(RtpGap::json).collect::<Vec<_>>(),
                        "currentRtpGap": self.current_rtp_gap.as_ref().map(RtpGap::json),
                        "rtpGapsOmitted": self.rtp_gaps_omitted,
                        "limits": "Observed head ages start at first blocking observation, or capture arm for a preexisting head. They are lower bounds, not original block arrival ages. Ready successors are sampled on opening, hole changes, timer changes and 16 ms packet-driven opportunities, not continuously. Complete means capture timer reached, not media coverage. Sender coordinates contain no payload or endpoint data."
                    });
                    serde_json::to_writer(&mut writer, &value)?;
                    writer.flush()
                })();
                opennow_streamer_protocol::log::log_async(
                    if exported.is_ok() { "INFO" } else { "WARN" },
                    "fec-head-diag",
                    if exported.is_ok() { "status=exported" } else { "status=export-failed" },
                );
            });
        if result.is_err() {
            opennow_streamer_protocol::log::log_async(
                "WARN",
                "fec-head-diag",
                "status=export-thread-failed",
            );
        }
    }
}

impl Episode {
    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "gapOrigin": self.gap_origin.label(),
            "coordinateOwner": if self.gap_origin == GapOrigin::WholeMissingBlock { "first_available_successor" } else { "blocked_head" },
            "base": self.base,
            "frame": self.frame_index,
            "block": self.block_index,
            "lastBlock": self.last_block_index,
            "startNs": self.start_ns,
            "endNs": self.end_ns,
            "initialMissing": self.initial_missing,
            "finalMissing": self.final_missing,
            "lastMissing": self.last_missing,
            "firstMissingChanges": self.first_missing_changes,
            "deadlineRestarts": self.deadline_restarts,
            "restartsWithReadySuccessors": self.restarts_with_ready_successors,
            "successorSamples": self.successor_samples,
            "maxReadySuccessors": self.max_ready_successors,
            "maxBufferedBlocks": self.max_buffered_blocks,
            "maxHeadAgeNs": self.max_head_age_ns,
            "maxGapAgeNs": self.max_gap_age_ns,
            "headAgeIsLowerBound": true,
            "preexisting": self.preexisting,
            "outcome": self.outcome.map(Outcome::label),
            "releasedPackets": self.released_packets,
            "repaired": self.repaired,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn snapshot(base: u64, first_missing: u64, gap_started: Instant) -> WaitSnapshot {
        WaitSnapshot {
            gap_origin: GapOrigin::IncompleteHead,
            base,
            frame_index: 7,
            block_index: 0,
            last_block_index: 1,
            first_missing,
            last_missing: first_missing,
            gap_started,
            buffered_blocks: 2,
            ready_successors: Some(1),
        }
    }

    fn probe(origin: Instant, preexisting: Option<u64>) -> Probe {
        Probe::new(origin, PathBuf::from("/unused/fec-head.json"), preexisting)
    }

    #[test]
    fn hole_changes_and_deadline_restarts_are_distinct() {
        let origin = Instant::now();
        let mut probe = probe(origin, None);
        assert!(probe.needs_successor_sample(100, 101, origin, origin));
        probe.observe_wait(snapshot(100, 101, origin), origin);
        assert!(!probe.needs_successor_sample(100, 101, origin, origin));
        let later = origin + Duration::from_millis(40);
        probe.observe_wait(snapshot(100, 102, origin), later);
        probe.observe_wait(snapshot(100, 102, later), later);
        probe.observe_wait(snapshot(100, 103, later), later + Duration::from_millis(40));
        let head = probe.current.as_ref().unwrap();
        assert_eq!(head.first_missing_changes, 2);
        assert_eq!(head.deadline_restarts, 1);
        assert_eq!(head.restarts_with_ready_successors, 1);
        assert_eq!(head.max_head_age_ns, 80_000_000);
        assert_eq!(head.max_gap_age_ns, 40_000_000);
        assert_eq!(head.max_ready_successors, 1);
    }

    #[test]
    fn periodic_samples_detect_ready_successors_without_a_hole_or_timer_change() {
        let origin = Instant::now();
        let mut probe = probe(origin, None);
        let mut wait = snapshot(100, 101, origin);
        wait.ready_successors = Some(0);
        probe.observe_wait(wait, origin);
        let early = origin + Duration::from_millis(15);
        assert!(!probe.needs_successor_sample(100, 101, origin, early));
        wait.ready_successors = None;
        probe.observe_wait(wait, early);
        let due = origin + SUCCESSOR_SAMPLE_INTERVAL;
        assert!(probe.needs_successor_sample(100, 101, origin, due));
        wait.ready_successors = Some(1);
        probe.observe_wait(wait, due);
        assert!(!probe.needs_successor_sample(100, 101, origin, due));
        assert!(probe.needs_successor_sample(100, 101, origin, due + SUCCESSOR_SAMPLE_INTERVAL));
        let head = probe.current.as_ref().unwrap();
        assert_eq!(head.successor_samples, 2);
        assert_eq!(head.max_ready_successors, 1);
        assert_eq!(head.first_missing_changes, 0);
        assert_eq!(head.deadline_restarts, 0);
        assert_eq!(head.last_successor_sample, Some(due));
    }

    #[test]
    fn completed_episodes_are_bounded_and_current_survives_overflow() {
        let origin = Instant::now();
        let mut probe = probe(origin, None);
        for base in 0..EPISODE_CAPACITY as u64 + 3 {
            probe.observe_wait(snapshot(base, base + 1, origin), origin);
            probe.finish_head(base, origin, Outcome::Completed, 2, 0);
        }
        assert_eq!(probe.episodes.len(), EPISODE_CAPACITY);
        assert_eq!(probe.omitted, 3);
        probe.observe_wait(snapshot(999, 1_000, origin), origin);
        assert_eq!(probe.current_base(), Some(999));
    }

    #[test]
    fn completion_head_change_and_reset_close_only_the_matching_head() {
        let origin = Instant::now();
        let mut probe = probe(origin, None);
        probe.observe_wait(snapshot(100, 101, origin), origin);
        probe.finish_head(999, origin, Outcome::Expired, 0, 0);
        assert_eq!(probe.current_base(), Some(100));
        probe.finish_head(100, origin, Outcome::Completed, 2, 1);
        assert_eq!(probe.episodes[0].released_packets, 2);
        assert_eq!(probe.episodes[0].repaired, 1);
        probe.observe_wait(snapshot(103, 104, origin), origin);
        probe.observe_wait(snapshot(106, 107, origin), origin);
        assert_eq!(probe.episodes[1].outcome, Some(Outcome::HeadChanged));
        probe.finish_head(106, origin, Outcome::Reset, 0, 0);
        assert_eq!(probe.episodes[2].outcome, Some(Outcome::Reset));
        assert!(probe.current.is_none());
        assert_eq!(probe.unmatched_finishes, 1);
    }

    #[test]
    fn preexisting_head_age_starts_at_capture_arm_and_is_marked() {
        let origin = Instant::now();
        let mut probe = probe(origin, Some(100));
        probe.observe_wait(
            snapshot(100, 101, origin - Duration::from_millis(10)),
            origin + Duration::from_millis(20),
        );
        let head = probe.current.as_ref().unwrap();
        assert!(head.preexisting);
        assert_eq!(head.start_ns, 0);
        assert_eq!(head.max_head_age_ns, 20_000_000);
        assert_eq!(head.max_gap_age_ns, 30_000_000);
        assert_eq!(head.json()["headAgeIsLowerBound"], true);
        probe.finish_head(100, origin, Outcome::Reset, 0, 0);
        probe.observe_wait(snapshot(103, 104, origin), origin);
        assert!(!probe.current.as_ref().unwrap().preexisting);
    }
}
