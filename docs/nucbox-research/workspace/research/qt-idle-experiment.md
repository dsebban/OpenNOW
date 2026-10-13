# Qt update delay on the NucBox

## Result

`QT_QPA_UPDATE_IDLE_TIME=0` did not pass the frozen experiment gates. The normal setting is restored. The pooled control median was **1.8135% late fresh-source intervals**, versus **7.0003%** for the candidate. Two candidate windows fell below 57 fresh FPS. This block establishes no performance improvement.

The controls also varied substantially: their late-interval range was 10.8938 percentage points. There was a local-network incident during the repeated closing control. These observations limit causal attribution; the candidate's worse aggregate is not a clean estimate of the setting's isolated effect.

## Setup and method

- Ryzen 7 5700U / AMD Lucienne, GNOME Shell/X11, 3440×1440 at 60.00 Hz. Graphics clocks reached 1900 MHz during the earlier read-only sample; manual GPU policy was not a fixed 200 MHz cap.
- Germany, H.264 8-bit 1920×1080 at 60 FPS, embedded Vulkan decoder, fullscreen Nocturn UI, stats overlay off, frame generation off. Ori's waterfall scene was visible before scoring. Each restart used normal saved-account GUI resume of the same cloud session.
- Source `3c85a3d9cb53bb8062d2487c8cc72a5d3144c3d9`, Qt executable SHA-256 `1230fa250f1b6f90c0c6b1a4a15d5d98879cb45b9256b885b2e0744baca319f1`. This build adds window-swap and GUI-notification diagnostics without changing the scheduler, protocol or decoder.
- Method 2 and its acceptance gates were unchanged. Additive census schema 1 is diagnostic only. Fresh controls used the new build; earlier builds are not mixed into this comparison. Each full trial used 20 seconds of warmup and three roughly 60-second windows. No agent gameplay input, builds or tests ran in scored windows.
- The candidate changed only the Qt update-idle environment variable. Readback confirmed it was `0` for the candidate and absent for the restored control. No route, radio, VPN, router, SQM, CPU/GPU power policy or socket limit was changed.

## Per-window results

| Trial | Fresh FPS, windows 1 / 2 / 3 | Late intervals, % | Outcome |
|---|---|---|---|
| A1 baseline | 57.80 / 59.68 / 58.81 | 3.8639 / 0.6319 / 1.6719 | Valid control |
| B1 zero idle | 52.40 / 55.95 / 59.03 | 14.5181 / 7.0003 / 1.5810 | Rejected |
| A2 restored | 58.58 / 59.37 / — | 2.0768 / 1.0771 / — | Invalid: diagnostic-log rotation |
| A3 restored repeat | 57.85 / 59.97 / 54.10 | 1.9552 / 0.1668 / 11.0606 | Valid control; network incident in window 1 |

A2 stopped when the native diagnostic log rotated at 2 MiB. Its two completed windows and the failed attempt remain in the evidence; they are not used as a valid three-window control. A3 repeated the complete control with a new label, unchanged client process/build and fresh socket observation. The comparison used A1/B1/A3.

Native window-median ping ranged from 61.7–63.2 ms in A1, 63.0–65.1 ms in B1, and 63.0–63.3 ms in A3. A3's first window contained one native telemetry sample at 359.6 ms and 0.6308% packet loss; neither a window median nor a TCP preflight describes that incident. Other scored native packet-loss samples were zero. See the [network analysis](qt-idle-network.md).

## What the new counters establish

A short calibration measured **59.65 all-window callbacks/s**, **50.80 fresh-source callbacks/s**, and **59.95 GUI frame-notification emissions/s**. Only two notifications coalesced in 20 seconds; pending-notification-to-GUI-drain p95 was below the 2 ms bucket boundary. A 50 FPS fresh-source reading therefore did not mean the window itself swapped only 50 times per second.

The strongest full-window example is A3 window 3: decoded output was approximately **60.05 FPS**, notifications **60.02/s**, GUI emissions **59.87/s**, window callbacks **59.75/s**, and fresh-source callbacks **54.10/s**. It had 357 mailbox replacements, no measured video-socket receive drops, and notification-drain p95 below 3 ms. These stage rates use slightly different native/Qt boundaries, so they do not assign every missing frame to one cause.

The next source investigation is the acquisition/recording boundary. The current wrapper maps both an empty acquisition and a post-acquisition GPU-recording failure to `NO_FRAME`; Qt then retains the previous texture. Add separate acquire-success/empty/stale, record-success/not-ready, prepare and fresh-submit counters before selecting a change. Shared Vulkan snapshots normally wait for their copy fence before publication, so readiness failure is a path to measure, not an established cause. Removing that fence without proving buffer lifetime would be incorrect.

More buffering, a GPU clock change, and a faster polling loop have not been justified by this block. The useful direction is reducing missed use of available decoded frames while preserving frame age and latency.

## Evidence, verification and limits

Private evidence is in `.runtime/round2/` and `.runtime/hillclimb/idle_*.json`. The frozen design, binary/script fingerprints, retained native logs, Qt JSONL, census reports, network/socket observations, invalid control, comparison and restoration readback are preserved. `tools/pipeline-census.py` and `tools/presentation-census.py` reproduce the numeric stage summaries.

Three focused Qt suites passed: 351 cases, with 10 offscreen skips. Seven ancillary-analyzer tests passed. The generated [validation skill](../.cursor/skills/verify-gfn-experiments/SKILL.md) also completed its real launch, preflight, calibration and exact-process cleanup proof. It now documents both process-exit verification and diagnostic-log rotation.

Qt callbacks are not physical scanout or input-to-photon measurements. Window callbacks without a fresh source are not automatically repeated-video scanouts. Notification delays are not exact GPU-frame ages. CPU distributions from the current collector remain descriptive. After ordinary playback resumed, the visible game location changed; activity during the final control has not been independently confirmed. This adds workload uncertainty to the already rejected comparison.

Ordinary `tools/run-native.sh` playback was restored with all experiment and live-telemetry variables absent. Both receive-buffer limits remain 212992. The instrumented source remains on the experimental branch; no candidate was enabled by default.
