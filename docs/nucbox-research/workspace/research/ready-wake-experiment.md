# Readiness wakeup on the NucBox

## Result and decision

**The readiness-wait candidate is rejected.** The pooled controls had a median of **0.3636% late fresh-source intervals**, compared with **0.7490%** for the candidate. That is a **105.97% increase**, or a signed improvement of −105.97%. Native stream ping also failed its frozen guard.

The candidate client later crashed in Qt's XCB platform library, **98.464 seconds after scoring ended**. The completed measurement remains valid numeric evidence. The unexpected crash is a separate stability failure; the available evidence does not establish that the readiness flag caused it.

No candidate from this experiment loop has earned a default change. Repeated scheduling, display and receive-buffer trials have reached a plateau under the current method. Stop this tuning loop rather than repeat candidates until a favorable sample appears. The candidate implementation and tests were reverted from the active source branch; the trial remains available on `nucbox-readywake-trial` at `3d553f6`. The restored source tree matches the counters-only `78d9bc0` tree. The restored source was rebuilt successfully and passed the three focused Qt suites (356 cases, eight offscreen skips). This restoration was not another scored experiment.

## Hypothesis and setup

The embedded monitor previously ended each iteration with a short polling sleep. The candidate replaces that sleep with a bounded condition-variable wait that wakes when decoded output becomes available. It preserves latest-frame selection and queue bounds. The intended benefit was earlier publication without adding buffered frames.

All three trials used the same compiled implementation and harness; only `OPENNOW_EMBEDDED_READY_WAKE` changed from absent to `1` and back to absent. The launcher clears other experiment variables. Source revision was `3d553f66a282efcb5ce353fad452a3217b4c6205`; the native streamer library SHA-256 was `dae01863eaae64676bcae90bbecbc6f58f41649252170ecd925d0db6d58208a3`. Full Qt and harness fingerprints are retained in each private trial result.

The workload was Germany, H.264 8-bit 1920×1080 at 60 source FPS, embedded Vulkan decoding, a 60 Hz display, frame generation off, and Ori's waterfall scene. The closing control resumed the same cloud session through its visible Active Session banner after the crash. It was not a new Play allocation. Saved credentials stayed in the normal vault, accessed by the single owning GUI process.

Method 3 and all acceptance rules were frozen. Each trial used 20 seconds of warmup and three approximately 60-second windows. The primary score remains the fraction of fresh-source intervals strictly longer than 25 ms. No build, test, gameplay input or configuration change ran during scored windows. Focused native checks recorded 16 unique passing tests before playback; those checks did not exercise a full real GPU/Qt stream.

## Per-window results

Times are UTC on 2026-10-07.

| Trial | Scored interval | Fresh FPS, windows 1 / 2 / 3 | Late intervals, % | Mailbox replacements |
|---|---|---|---|---:|
| A1 baseline | 17:07:18–17:10:19 | 59.68 / 59.55 / 59.92 | 0.1955 / 0.5318 / 0.1368 | 25 |
| B1 readiness wait | 17:14:40–17:17:41 | 58.41 / 59.11 / 59.47 | 2.7104 / 0.7490 / 0.2242 | 143 |
| A2 restored baseline | 17:23:30–17:26:32 | 58.25 / 59.32 / 59.95 | 3.0329 / 1.1885 / 0.1641 | 151 |

All nine windows were valid and remained above the 57 fresh-FPS floor. The controls' late-interval range was **2.8961 percentage points**; their substantially different phases limit attribution of the candidate's worse aggregate to the flag alone.

The exact comparator reasons are:

- `improvement_below_20_percent_or_control_spread`: the candidate was worse on the primary score and did not clear control variation.
- `regressed_stream_ping_ms`: pooled control window-median ping was **63.64365 ms**, with a **1.29685 ms** range. The permitted upper value was **64.94050 ms**; the candidate median was **65.34329 ms**, exceeding it by **0.40279 ms**.

Candidate assembled-frame rates were **59.99 / 59.87 / 60.01 FPS** and decoded rates **60.05 / 59.51 / 60.07 FPS**. Source-interval p99 upper bounds were **34 / 22 / 21 ms**; worst intervals were **50 / 246 / 253 ms**. Both controls also had long hitches, with maxima of 234 ms and 83 ms. All nine submit-delay p95 bounds were below 1 ms. The frozen verdict remains unchanged; these secondary values do not rescue the primary result.

## Acquisition and presentation evidence

All three schema-2 censuses completed. In every window, successful acquisitions were successfully recorded and imported. Post-acquisition `NO_FRAME`, stale and error counts were zero. This block again supplies no evidence that failure to record an already acquired GPU frame caused the fresh-frame deficit.

B1 window 1 had **60.00 all-window callbacks/s**, **60.01 GUI emissions/s**, and **58.41 fresh FPS**. It recorded **95 empty acquisitions** and **111 mailbox replacements**, with no observed video-socket receive drops. The closing control's first window was similarly weak: **60.00 window callbacks/s**, **59.98 GUI emissions/s**, **58.25 fresh FPS**, **105 empty acquisitions** and **107 mailbox replacements**, also with no socket drops.

The candidate's notification-drain p95 upper bounds were **8 / 10 / 2 ms**, against **9 / 10 / 10 ms** in A1 and **7 / 10 / 8 ms** in A2. The measurements do not show a consistent notification improvement. Queue replacements, empty acquisitions and fresh callbacks have different boundaries; their counts are not an exact frame-by-frame loss attribution.

## Network evidence and coverage

| Scored observation | A1 | B1 | A2 |
|---|---:|---:|---:|
| Gateway replies | 181 / 181 | 181 / 181 | 182 / 182 |
| Gateway RTT median / p95 / max, ms | 1.03 / 5.52 / 9.08 | 0.72 / 2.46 / 97.0 | 2.79 / 7.11 / 67.1 |
| Video-socket receive drops, windows 1 / 2 / 3 | 350 / 428 / 0 | 0 / 348 / 356 | 0 / 0 / 0 |
| Bundle-socket receive drops | 0 | 0 | 0 |
| Host UDP receive-buffer errors | 778 | 704 | 0 |
| Router CPU median / p95, % | 7.04 / 9.39 | 6.79 / 9.86 | 7.09 / 10.05 |
| Router WAN / download queue drops | 1753 / 0 | 2141 / 0 | 3576 / 0 |

All scored intervals had bracketing host, router and socket samples. Maximum gaps were **1.00 s**, **5.061 s**, and **1.002 s** respectively. Every scored socket sample was valid, with no ownership-read errors and stable socket identities at window boundaries. B1's socket observer ended with `process_read_unavailable` after the crash, well after the scored interval. The other observers were intentionally stopped after scoring. These end statuses do not remove the complete scored coverage.

Host receive-buffer errors match the observed video-socket drop totals. This establishes local socket receive loss in A1 and B1, but its absence in A2 did not prevent poor early fresh-frame pacing. Gateway spikes in B1 and A2 also mean the network cannot be described as uniformly stable. Gateway ICMP is separate from native stream ping, and router queue drops include other household traffic.

One native packet-loss sample reached **0.0155%** in A1 window 1; one reached **0.0547%** in B1 window 3. Other scored native loss samples were zero. The frozen median-of-window loss guard did not reject those isolated samples. They remain in the evidence rather than being hidden behind rounded zero-loss claims.

## Unexpected exit after B1

The system journal records an `opennow-qt` segmentation fault at **17:19:20.189 UTC**, in **`libQt6XcbQpa.so.6.9.1`**, with page-fault error 4. Coredump metadata confirms a retained dump. Scoring had ended at **17:17:41.725**, exactly **98.464 seconds earlier**. The last Qt observer at 17:19:19.729 still reported visible, active, ungated playback. The operator had delivered neither Quit nor Alt+F4 before the window disappeared.

The inspected 17:18:30–17:20:15 journal interval contained no OOM, memguard-kill or GPU-reset classification. Available memory stayed above **7.98 GB** during the observer. This supports a process crash rather than a recorded resource kill, but does not identify the underlying Qt/XCB bug. The child core process's exit mechanism is unproven. No raw core or debugger inspection was performed during the closing control.

The crash prevents treating the candidate as stable, regardless of performance. A fault in Qt's platform library does not by itself assign causality to the Rust readiness change.

## Restoration, next measurement and evidence

Production/test reverts `0da8547` and `44c3171` remove the candidate from the active branch while retaining the trial branch and all failures. The closing control itself used the original candidate-capable binary with the flag absent; it is distinct from the subsequent restoration build. The restored binary was launched through the ordinary wrapper with every named experiment flag and live telemetry absent, verified against its exact process. Both receive-buffer caps remained 212992 and the display rollback timer was inactive. Final GUI and process evidence is retained in `.runtime/round3/`.

The next useful investigation is **socket age versus receive-work timing**, as a separate instrumented cycle. Record bounded receive-batch duration, time between receive calls, queue occupancy/drop deltas, and time from a completed incoming frame to decoded publication and Qt acquisition. Align those numeric events with long source intervals. This can separate burst arrival or delayed receiver service from stale frames and presentation timing. First define the clock/boundary contract and observer cost; do not silently weaken this block's gates or add buffers from theory alone.

To reproduce this historical candidate, build the retained trial revision before starting a new matched block; the restored source does not implement the flag. Follow the [validation skill](../.cursor/skills/verify-gfn-experiments/SKILL.md) for GUI resume, flag readback, frozen profile, per-attempt observers and cleanup. The active launcher rejects `readywake`; selecting the trial source alone does not restore its historical launcher. For a deliberate reproduction, build that revision and use an isolated wrapper that clears all experiment flags, enables private telemetry, and sets `OPENNOW_EMBEDDED_READY_WAKE=1` only for the candidate. Retain the flag readback for each process, then use `tools/live-trial.py measure` with matching variant labels. Each full measurement takes about 201 seconds plus startup. Comparison is:

```bash
python3 tools/live-trial.py compare .runtime/hillclimb/wake_a1.json \
  .runtime/hillclimb/wake_b1.json --control .runtime/hillclimb/wake_a2.json
```

Private evidence includes `.runtime/round3/wake-*-logs/`, `wake-*-census.json`, network/socket observers, `wake-completed-block-aggregate.json`, sanitized exit reports, the frozen design and focused-test summary. Original trial and comparison JSON remain under `.runtime/hillclimb/`. Archives retain hashes and original file metadata. No credentials, session identifiers or raw log payloads are published here.

Qt callbacks are not physical scanout or input-to-photon measurements. CPU distributions from batched Qt telemetry are unsuitable for acceptance or CPU-saving claims. Nonstationarity, receive loss and the post-window crash limit causal conclusions; they do not change the retained rejection.
