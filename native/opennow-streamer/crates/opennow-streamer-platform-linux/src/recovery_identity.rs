//! Bounded, experiment-only keyframe identity evidence. Never changes decoder input.
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

const TARGET_CAP: usize = 16;
const EVENT_CAP: u64 = 4096;
const FALLBACK_CAP: u64 = 64;
static FALLBACKS: AtomicU64 = AtomicU64::new(0);
static WORKERS: AtomicU64 = AtomicU64::new(0);

pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("OPENNOW_FEC_DEADLINE_MODE_FILE")
            .is_some_and(|path| std::path::Path::new(&path).is_absolute())
    })
}

fn wall_us() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_micros())
}

/// A missing/negative AVFrame PTS invalidates exact identity, even if the fallback matches.
#[cfg(feature = "ffmpeg")]
pub(crate) fn pts_fallback(pts: Option<i64>, fallback_us: u64) {
    if !enabled() || exact_pts(pts) {
        return;
    }
    let ordinal = FALLBACKS.fetch_add(1, Ordering::Relaxed) + 1;
    if ordinal <= FALLBACK_CAP {
        eprintln!(
            "recovery-identity schema=1 event=pts_fallback ordinal={ordinal} fallback_pts_ns={} wall_us={} correlation_only=1",
            fallback_us.saturating_mul(1_000),
            wall_us()
        );
    } else if ordinal == FALLBACK_CAP + 1 {
        eprintln!("recovery-identity schema=1 status=fallback-cap-reached cap={FALLBACK_CAP}");
    }
}

#[cfg(any(feature = "ffmpeg", test))]
fn exact_pts(pts: Option<i64>) -> bool {
    pts.is_some_and(|pts| pts >= 0)
}

pub(crate) struct Probe {
    worker_id: u64,
    epoch: u64,
    targets: [Option<u64>; TARGET_CAP],
    events: u64,
    targets_omitted: u64,
}

impl Probe {
    pub(crate) fn from_env() -> Option<Self> {
        enabled().then(Self::new)
    }

    fn new() -> Self {
        Self {
            worker_id: WORKERS.fetch_add(1, Ordering::Relaxed) + 1,
            epoch: 0,
            targets: [None; TARGET_CAP],
            events: 0,
            targets_omitted: 0,
        }
    }

    fn event(&mut self, event: &str, pts_us: u64, reason: &str) {
        self.events += 1;
        if self.events <= EVENT_CAP {
            eprintln!(
                "recovery-identity schema=1 event={event} worker_id={} ordinal={} decoder_epoch={} pts_ns={} reason={reason} wall_us={} correlation_only=1",
                self.worker_id,
                self.events,
                self.epoch,
                pts_us.saturating_mul(1_000),
                wall_us()
            );
        } else if self.events == EVENT_CAP + 1 {
            eprintln!("recovery-identity schema=1 status=event-cap-reached cap={EVENT_CAP}");
        }
    }

    pub(crate) fn opened(&mut self, reason: &str) {
        self.epoch += 1;
        self.targets = [None; TARGET_CAP];
        self.event("decoder_epoch", 0, reason);
    }

    pub(crate) fn submitted(&mut self, pts_us: u64, keyframe: bool) {
        if !keyframe {
            return;
        }
        if self.targets.contains(&Some(pts_us)) {
            self.event("keyframe_pts_ambiguous", pts_us, "duplicate_target");
            return;
        }
        if let Some(slot) = self.targets.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(pts_us);
            self.event("keyframe_submitted", pts_us, "exact_pts_target");
        } else {
            self.targets_omitted += 1;
        }
    }

    pub(crate) fn decoded(&mut self, pts_us: u64) -> bool {
        let Some(slot) = self.targets.iter_mut().find(|slot| **slot == Some(pts_us)) else {
            return false;
        };
        *slot = None;
        self.event("keyframe_decoded", pts_us, "matched_target");
        true
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        let fallbacks = FALLBACKS.load(Ordering::Relaxed);
        eprintln!(
            "recovery-identity schema=1 status=finished worker_id={} decoder_epoch={} emitted={} omitted={} targets_omitted={} fallback_total={fallbacks} fallback_omitted={}",
            self.worker_id,
            self.epoch,
            self.events.min(EVENT_CAP),
            self.events.saturating_sub(EVENT_CAP),
            self.targets_omitted,
            fallbacks.saturating_sub(FALLBACK_CAP)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_or_negative_pts_cannot_claim_exact_identity() {
        assert!(!exact_pts(None));
        assert!(!exact_pts(Some(-1)));
        assert!(exact_pts(Some(0)));
        assert!(exact_pts(Some(42)));
    }

    #[test]
    fn exact_target_matches_once_and_cannot_cross_decoder_replacement() {
        let mut probe = Probe::new();
        probe.opened("initial");
        probe.submitted(42, false);
        assert!(!probe.decoded(42));
        probe.submitted(42, true);
        assert!(!probe.decoded(43));
        assert!(probe.decoded(42));
        assert!(!probe.decoded(42));
        probe.submitted(51, true);
        for reason in ["format", "queue", "reference_loss", "fallback"] {
            let epoch = probe.epoch;
            probe.opened(reason);
            assert_eq!(probe.epoch, epoch + 1);
            assert!(!probe.decoded(51));
            probe.submitted(51, true);
        }
    }

    #[test]
    fn target_overflow_is_observable_and_storage_stays_bounded() {
        let mut probe = Probe::new();
        for pts in 0..20 {
            probe.submitted(pts, true);
        }
        assert_eq!(probe.targets.iter().flatten().count(), TARGET_CAP);
        assert_eq!(probe.targets_omitted, 4);
        assert!(!probe.decoded(19));
    }
}
