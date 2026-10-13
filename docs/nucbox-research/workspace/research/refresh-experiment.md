# Display refresh on the NucBox

## Result

**120 Hz did not pass the frozen acceptance rules.** In the completed 60 / 120 / 60 Hz block, the pooled controls had a median of **0.8920% late fresh-source intervals**, versus **0.3904%** for the candidate: a raw decrease of **56.23%**. The change failed the control-variation gate and the native-FPS guard. No display change is accepted as a new default; 60 Hz was restored.

The controls varied substantially, and the completed block contains local video-socket receive drops in both controls and the candidate. These observations limit causal attribution. An earlier 60 / 120 Hz block ended before its closing control and remains separately marked incomplete. Its trials, failed preflight and transport shutdown evidence are retained.

## Setup and frozen method

- Germany, H.264 8-bit 1920×1080 at 60 source FPS, embedded Vulkan decoding, Ori's waterfall scene. The display changed from 60 to 120 Hz; the source stream remained 60 FPS. The operator verified the scene and refresh readback before scoring.
- Instrumented source `78d9bc03a1b586c4b1bfba9f461b74a2f31e27eb`, with the same executable and native streamer library across the refresh trials. Build and harness SHA-256 fingerprints are retained in each private result.
- Method 3 was frozen before scoring. It adds explicit requested display refresh, validates readback within ±1 Hz, and requires matching refresh in the opening and closing controls. Its score and acceptance math are unchanged from method 2. Results from those method versions cannot be mixed.
- Each full trial uses 20 seconds of warmup and three roughly 60-second windows. The primary score is the fraction of **fresh source intervals longer than 25 ms**, still based on the 60 FPS source. Fresh FPS, source interval tails, decoder and submit delays, media-lag growth, native throughput, ping and packet loss remain guardrails.
- A candidate must maintain at least 57 fresh FPS in every window. A slower baseline remains measurable and is flagged as a quality failure; discarding it would hide the problem being investigated. Acceptance also needs at least a 20% relative improvement and an absolute improvement greater than the measured control range.

## Completed block: C1 / D1 / C2

This block used a newly allocated cloud session and its own controls. All three trials were valid, used identical build/harness fingerprints, and retained three complete windows. C1 scored from **16:39:14–16:42:15 UTC**, D1 from **16:43:51–16:46:53**, and C2 from **16:48:21–16:51:23** on 2026-10-07.

| Trial | Display | Fresh FPS, windows 1 / 2 / 3 | Late intervals, % | Mailbox replacements |
|---|---:|---|---|---:|
| C1 opening control | 60 Hz | 59.35 / 59.69 / 55.77 | 1.0390 / 0.2197 / 7.6188 | 293 |
| D1 candidate | 120 Hz | 59.54 / 59.77 / 59.51 | 0.2203 / 0.3904 / 0.4959 | 4 |
| C2 closing control | 60 Hz | 56.83 / 59.41 / 59.47 | 5.5718 / 0.7450 / 0.1930 | 231 |

The exact frozen verdict is `reject`, for:

- `improvement_below_20_percent_or_control_spread`: the relative gain exceeds 20%, but the absolute decrease of **0.5016 percentage points** is below the pooled control range of **7.4258 points**. The combined reason label covers either failure; this block failed the spread part.
- `regressed_native_fps`: the median of the controls' six native `framesPerSecond` window medians was **59.9932487021 FPS**. Their range was **0.0025657467 FPS**, making the guard's floor **59.9906829554 FPS**. D1's median was **59.9895847569 FPS**, **0.0010981985 FPS** below that floor.

The second rejection is a very small numerical difference, not evidence of perceptible throughput loss. The native value counts accepted video-frame feedback over its own telemetry interval; it is separate from Qt fresh-source FPS and the cumulative assembled-frame workload check. D1's assembled rate was **59.8880 / 59.9268 / 59.8880 FPS** and its fresh-source rate stayed above 57 FPS throughout. The frozen guard and rejection remain unchanged; rounded values of “60 FPS” would conceal why it fired.

The candidate still contained long hitches: worst source intervals were **253 / 207 / 269 ms**, versus **103 / 161 / 35 ms** in C1 and **62 / 160 / 262 ms** in C2. These passed the existing worst-interval tolerance because the controls also varied. All nine submit-delay p95 bounds were below 1 ms; no interval histogram overflow occurred. Passing those guards does not mean the stream was hitch-free.

### Completed-block pipeline evidence

All nine stage censuses completed. Again, every successful frame acquisition was recorded and imported, with zero post-acquisition `NO_FRAME`, stale or error counts. C1 window 3 had **60.01 all-window callbacks/s**, **59.97 GUI emissions/s**, **55.77 fresh FPS**, **254 empty acquisitions**, and **249 mailbox replacements**. It had no video-socket receive-drop increment. C2 window 1 similarly presented **56.83 fresh FPS** despite **60.00 window callbacks/s**, with **190 empty acquisitions**, **199 mailbox replacements**, and no socket-drop increment.

At 120 Hz, D1 produced **61.66–62.92 all-window callbacks/s** and **59.51–59.77 fresh FPS**. Its notification-drain p95 upper bound was 1 ms in each window. Extra empty acquisition attempts are expected when the window runs faster than source delivery; their count alone cannot diagnose lost video frames.

### Completed-block network evidence and coverage

| Observation | C1, 60 Hz | D1, 120 Hz | C2, 60 Hz |
|---|---:|---:|---:|
| Gateway replies | 181 / 181 | 182 / 182 | 182 / 182 |
| Gateway RTT median / p95 / max, ms | 0.60 / 2.06 / 8.74 | 0.61 / 3.16 / 5.84 | 0.62 / 1.82 / 4.01 |
| Video-socket receive drops, windows 1 / 2 / 3 | 0 / 369 / 0 | 422 / 285 / 388 | 0 / 420 / 744 |
| Bundle-socket receive drops | 0 | 0 | 0 |
| Host UDP receive-buffer errors | 369 | 1095 | 1164 |
| Router CPU median / p95, % | 6.58 / 7.63 | 6.64 / 7.65 | 6.56 / 8.45 |
| Router WAN / download queue drop increments | 379 / 0 | 375 / 0 | 708 / 0 |

Every scored window had samples on both sides of its boundaries. Host gaps were at most 1.00 s, socket gaps 1.001 s, and router gaps 5.064 s. All scored socket samples were valid, with no ownership-read errors and stable socket identity at each window boundary. The longer observers were deliberately stopped after scoring; their `interrupted` status does not leave a gap in the scored intervals.

The video socket's drops sum to the host receive-buffer error increase in each trial. This is direct evidence of local socket receive loss during the block, despite low gateway RTT and no gateway loss. It does not establish whether burst arrival, delayed receive-thread service or another mechanism exhausted the queue. Drops also occurred in the closing 60 Hz control, so they cannot be assigned to 120 Hz alone. Router queue drops cover household traffic and remain unattributed.

D1 window 1 contained a native packet-loss sample of **5.4713%** and ping of **81.02 ms**. The other two candidate windows had zero reported native packet loss, as did all control windows. The frozen packet-loss guard compares the median of per-window maxima, so this single affected window did not trigger that guard. The incident remains in the report and evidence; it is not removed or relabeled to change the verdict. Native telemetry samples and socket counters describe different stages and sampling intervals.

## First-block observations

All times below are UTC on 2026-10-07. A1 scored from 16:18:10 to 16:21:12; B1 from 16:25:55 to 16:28:57.

| Trial | Display | Fresh FPS, windows 1 / 2 / 3 | Late intervals, % | Mailbox replacements |
|---|---:|---|---|---:|
| A1 opening control | 60 Hz | 59.20 / 59.87 / 56.90 | 1.3570 / 0.1369 / 5.4482 | 241 |
| B1 candidate | 120 Hz | 60.00 / 60.00 / 60.00 | 0.3552 / 0.4918 / 0.1944 | 0 |
| A2 closing control | 60 Hz restored | Not collected | Not collected | — |

A1's median was **1.3570%**, versus **0.3552%** for B1: a raw relative decrease of **73.82%**. The absolute difference was **1.0018 percentage points**, below A1's **5.3112-point** within-trial range. This is an exploratory observation, not an accepted effect estimate. A1's final window also fell below the 57 FPS quality target.

Native incoming video remained approximately 59.99–60.01 FPS and decoded output 60.05–60.07 FPS across the six windows. Submit-delay p95 was below the 1 ms bucket boundary throughout. Source-interval p99 upper bounds were 34 / 19 / 34 ms for A1 and 24 / 24 / 23 ms for B1. The largest source intervals were 108 ms and 83 ms respectively. Every scored native packet-loss sample was zero.

## First-block stage observations

The new counters separate empty frame acquisition from failure while recording an acquired GPU frame. In all six windows, acquired frames were successfully recorded and imported; post-acquisition `NO_FRAME`, stale and error counts were zero. These windows do not support GPU recording readiness failure as the explanation for the fresh-frame deficit.

A1 window 3 is the clearest example: decoded output was about **60.05 FPS**, GUI frame notifications **59.98/s**, and all-window callbacks **60.00/s**, but fresh source callbacks were **56.90/s**. There were **186 empty acquisitions**, **183 mailbox replacements**, and no observed video-socket receive-drop increment. This points to the timing of frame production and acquisition as a useful next investigation. The counters alone do not identify the exact cause or match each replaced frame to a later empty acquisition.

At 120 Hz, the window produced only **63.10–63.18 callbacks/s**, with approximately 60 fresh source frames/s. Extra acquisition attempts found an empty queue about 3.1 times/s while fresh delivery remained intact. The roughly 5% of callbacks without a new source frame is **not a dropped-frame percentage**. Display refresh, window callbacks, and fresh source delivery have different denominators.

The next source experiment replaces the embedded monitor's short polling sleep with a bounded readiness wait. It needs a separate matched block on a single compiled build. The completed block also motivates aligning socket-drop times with receive-thread service and frame arrival before attributing long hitches to presentation. The earlier receive-buffer experiment did not establish an accepted improvement; these observations do not justify silently adopting a larger buffer or removing GPU synchronization.

## First-block network observations

| Scored observation | A1, 60 Hz | B1, 120 Hz |
|---|---:|---:|
| Gateway replies | 182 / 182 | 182 / 182 |
| Gateway RTT median / p95 / max | 0.69 / 2.13 / 5.02 ms | 0.94 / 3.08 / 5.61 ms |
| Native window-median ping range | 57.68–65.20 ms | 57.55–58.22 ms |
| Router CPU median / p95 | 6.44 / 8.02% | 6.57 / 8.50% |
| Video and bundle socket drop increments | 0 in every window | 0 in every window |
| Router WAN / download queue drop increments | 652 / 0 | 837 / 0 |

Fresh host, router and socket observers covered both complete scored intervals. They were stopped after scoring, so their files record an intentional interruption of the longer requested observer duration. Router queue deltas use samples bracketing each trial and include other household traffic; they do not attribute drops to GFN. Gateway ICMP is independent of native stream ping.

These dedicated observers ended before the later transport shutdown. Their healthy scored-window observations cannot establish the network state at that shutdown.

## Why the first closing control is missing

The client recorded a video transport timeout at **16:35:05 UTC**. At **16:35:13**, another timeout exhausted its single recovery attempt; shutdown followed. The closing-control doctor then failed because fresh stream telemetry was unavailable, and the GUI had returned to the library. The display had already been restored to 60 Hz more than four minutes earlier.

This identifies the client's shutdown trigger, not why video stopped arriving. A transport idle timeout is not evidence of account inactivity expiration, and the retained evidence does not establish that the refresh change caused the shutdown.

The comparator enforces equal workload/profile labels but deliberately does not record cloud session identifiers. Session continuity therefore also requires operator verification through normal GUI resume. A newly allocated cloud session requires a new profile label and a complete new A/B/A block; reusing the previous label would not prove that the remote workload remained matched.

## Reproduction and evidence

Use the [validation skill](../.cursor/skills/verify-gfn-experiments/SKILL.md) for launch, saved-account playback, preflight, observers and cleanup. After confirming the matching scene and display readback, the measurement command is:

```bash
python3 tools/live-trial.py measure TRIAL_LABEL \
  --profile OPERATOR_VERIFIED_PROFILE --variant baseline --display-hz 60
```

For the candidate, use a new label, `--variant display120 --display-hz 120`, after the operator has changed and verified the display mode. Restore 60 Hz and collect the closing control with the original variant. Each run takes about 201 seconds; short calibration does not qualify for acceptance. Compare only a completed matched block:

```bash
python3 tools/live-trial.py compare OPENING.json CANDIDATE.json --control CLOSING.json
```

Private evidence is retained under `.runtime/round3/` and `.runtime/hillclimb/refresh_*.json`, including numeric results, build fingerprints, independent stage censuses, observer files, display readbacks, preflight failure, both aggregate reports and the rejected comparison. The original Qt/native logs were archived under `.runtime/round3/pre-readywake-logs/` after verified Qt/core exit. The earlier 21-second calibration was measurable but slow (53.90 fresh FPS, 11.3074% late intervals); it remains a calibration, not an acceptance result. Credentials remain in the normal saved-account vault and are used only through the owning GUI process.

Qt `frameSwapped` indicates that a frame was queued for presentation; these measurements are not physical scanout or input-to-photon latency. Native and Qt counters have slightly different sampling boundaries. CPU distributions from batched Qt telemetry remain descriptive and are not acceptance metrics. The conservative control-range gate can make acceptance impossible in a nonstationary block; rejection by that gate is inconclusive about whether a mechanism could help. Any different statistical method must be explicitly versioned and frozen before new candidates are scored.
