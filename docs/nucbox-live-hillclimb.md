# Real GeForce NOW playback experiments

This report accompanies the experimental `nucbox-live-hillclimb` branch. The
measurement scripts and private data live in the separate NucBox research
workspace named below; they are not bundled with this client checkout.

## Final outcome

This experiment block produced a repeatable measurement tool and ten completed three-window trials. No candidate met the frozen acceptance rules, so no production default was changed.

Continuous update remains worth investigating. On the second session it reduced the median late interval fraction by a raw 59.61% relative to the combined baseline controls. The result failed the noise and ping guards, so it is not an accepted performance improvement. The basic Qt render loop was worse, VA-API failed preflight, and a later authorized receive-buffer trial also failed the acceptance gates. Five candidate configurations were evaluated without an accepted win.

## First experiment block

No candidate is accepted. Both scheduling changes produced fewer late intervals in their first trials, but the baseline varied too much for the frozen noise gate. The first continuous-update trial also used the BlueStar UI configuration, which differed from the comparison setup. Its measurements remain diagnostic and cannot support an accepted improvement claim.

The table reports three measured windows in order. Percentages are fresh source intervals greater than 25 ms. FPS is actual source presentation observed by Qt.

| Trial | Late interval percentages | Presented FPS | Mailbox replacements | Result |
| --- | --- | --- | ---: | --- |
| Baseline | 0.084%, 4.872%, 13.203% | 59.76, 57.20, 53.02 | 584 | Measurable baseline with a low-FPS quality failure. |
| Continuous update | 0.703%, 0.167%, 0.139% | 59.30, 59.90, 59.91 | 42 | Promising observation; UI configuration confound excludes acceptance. |
| One-millisecond publication polling | 0.223%, 0.223%, 0.056% | 59.87, 59.66, 59.98 | 15 | Valid measurement; frozen baseline-noise gate fails. |
| VA-API | No measurement windows | No measurement windows | Not measured | Preflight rejected the embedded decoder profile. |

The baseline's median late interval fraction was 4.872%, but its window range spanned 13.120 percentage points. The reductions in candidate medians were smaller than that spread. The frozen rule therefore cannot accept either scheduling candidate, despite the lower observed rates. No repeated baseline control exists on that original seat.

The baseline's final window assembled about 59.99 FPS and decoded about 60.07 FPS while presenting 53.02 FPS. That window recorded 419 mailbox replacements. This is evidence that decoded frames were superseded before Qt consumed them. It does not prove which scheduler transition caused the loss. All native packet-loss samples in the three completed trials reported 0.0%.

Rare hitches also remained. The baseline's largest observed interval upper bound was 167 ms. Continuous update reached 217 ms, and one-millisecond polling reached 251 ms. Median window ping was 62.21 ms for baseline, 57.64 ms for continuous update, and 62.34 ms for polling. These are observations across different runs, with no claim that a local scheduling change reduced network latency.

After the VA-API preflight failure, the old remote session was unavailable. A normal Play action allocated a new game session. Its comparison profile is `germany-h264-1080p60-ori-waterfall-seat2`. Results from that session must be compared within its own block. They must not supply a control for the original seat.

## Second experiment block

The second block used the new cloud session and the same Nocturne UI configuration. The basic render-loop trial was followed by a baseline, continuous update, and a repeated baseline control. These results are compared within this session only.

| Trial | Late interval percentages | Presented FPS | Mailbox replacements | Result |
| --- | --- | --- | ---: | --- |
| Basic Qt render loop | 2.594%, 0.790%, 0.028% | 58.47, 59.06, 59.98 | 127 | Rejected for the primary metric and longest-interval guard. |
| Baseline A2 | 12.742%, 3.805%, 0.440% | 53.26, 57.82, 59.57 | 559 | Measurable control with a low-FPS quality failure. |
| Continuous update B2 | 1.838%, 0.274%, 0.251% | 58.85, 59.83, 59.85 | 91 | Rejected by the noise and median-ping guards. |
| Baseline A3 | 0.587%, 0.164%, 0.770% | 59.65, 59.92, 59.60 | 50 | Valid repeated control. |

The pooled six baseline windows had a median late fraction of 0.6785%. Continuous update measured 0.2740%, a raw relative decrease of 59.61%. The absolute decrease was only 0.4045 percentage points, while baseline spread was 12.5782 percentage points. The comparison also rejected its median stream ping, about 61.75 ms, against the baseline controls near 57.72 ms. The data does not establish that the client change caused this ping difference.

The basic render loop measured a 0.7901% median late fraction, 16.44% worse than the pooled baseline median. Its longest interval upper bound was 369 ms. The matched controls reached at most 165 ms. It failed both the improvement gate and the longest-interval guard.

All native packet-loss samples in these four completed trials reported 0.0%. The low-FPS baseline retained high incoming and decoded throughput. These observations continue to point toward local frame publication and presentation timing as a useful investigation, while preserving the limits of the callback measurements.

The first attempt at an eight-megabyte receive-buffer limit was blocked before mutation because its rollback setup required administrator authentication. That historical attempt remains recorded as `rmem8m_b1`. The user then authorized the privileged work. The old cloud session expired before scoring began, so the temporary setting was restored and its rollback timer cancelled. The actual experiment used a fresh third session with its own baseline controls.

## Receive-buffer experiment on the third session

The third block used `germany-h264-1080p60-ori-waterfall-seat3`. Baseline A4, the enlarged-buffer trial, and restored baseline A5 all used that session. No earlier session supplied a control.

The experiment changed the kernel receive-buffer maximum from 212992 to 8388608 bytes while keeping the default at 212992. Both native UDP sockets reported receive-buffer values of 425984 bytes during A4, 16777216 bytes during the candidate, and 425984 bytes after restoration during A5. These are the kernel-reported socket values, separate from the configured limits.

| Trial | Late interval percentages | Presented FPS | Mailbox replacements | Result |
| --- | --- | --- | ---: | --- |
| Baseline A4 | 8.630%, 19.653%, 6.730% | 55.23, 50.13, 56.21 | 1116 | Measurable baseline with low-FPS quality failures. |
| Enlarged receive buffers | 6.198%, 1.180%, 2.893% | 56.47, 59.30, 58.36 | 355 | Rejected by FPS, noise, median-ping, and p95-ping guards. |
| Restored baseline A5 | 0.306%, 6.418%, 12.216% | 59.85, 56.20, 53.48 | 616 | Measurable repeated control with low-FPS quality failures. |

The pooled control median was 7.6800% late intervals. The candidate median was 2.8933%, a raw reduction of 62.33%. The control spread was 19.3470 percentage points, larger than the absolute reduction. The candidate's first window presented only 56.47 FPS. Its median window ping was about 59.99 ms, compared with 57.25 ms for A4 and 57.06 ms for A5. Both median and p95 ping guards failed. These results do not establish a benefit from the larger receive buffers.

The candidate's third window reported a maximum packet-loss reading of 0.0164%. Its first two windows and all control windows reported maxima of 0.0%. Incoming and decoded rates remained near 60 FPS, including the baseline window presenting only 50.13 FPS. Large phase variation and mailbox replacement therefore remain important unresolved local observations.

The kernel receive-buffer maximum and default were restored to 212992 bytes. The repeated control verified both native UDP sockets at 425984 bytes. The runtime toggles remain opt-in experiments, and the client is returned to baseline Ori playback. No scheduling, decoder, render-loop, or receive-buffer candidate was accepted as a production default.

## Method

The objective is fewer dropped or late frames while retaining playback rate and latency. Each trial uses the real Ori stream on Germany, H.264 8-bit at 1920×1080 and 60 FPS. The repeated scene is the animated waterfall. The comparable workload label is `germany-h264-1080p60-ori-waterfall`. A separate variant label records the local decoder or presentation change.

The operator uses OpenNOW's normal GUI and saved credential vault to launch or resume the game. The measurement script observes the existing client. It makes no account requests and never reads credentials. Resume of the same cloud seat preserves the negotiated stream profile. A resumed session must show the expected codec, resolution, and frame rate before measurement.

Each full trial warms up for 20 seconds, then records three consecutive 60-second windows. The observer must see fresh video, an active visible window, the expected video dimensions, a display near 60 Hz, and disabled frame generation. The runner selects one healthy video observer and ignores unrelated video items. Builds and tests run outside scored windows.

The primary metric is the fraction of fresh source frame intervals strictly greater than 25 ms. The comparison uses the median of the three window fractions. The 25 ms boundary is counted directly, so the one-millisecond histogram does not round a frame across the threshold. Candidate acceptance requires source playback of at least 57 FPS in every window, or 95% of the negotiated rate. A baseline with positive playback below 57 FPS remains measurable and records a quality failure.

Native queue drops remain separate by source. GPU mailbox replacement, decoded queue overflow, and compressed video queue overflow describe different pipeline stages. Their sum is not an exact count of visible frame loss.

A candidate must reduce the primary metric by at least 20%, with an absolute reduction larger than the baseline window spread. Acceptance also requires a repeated baseline control using the same build and profile. This A/B/A sequence helps expose scene or network drift. A zero-late baseline is a measurement floor and cannot establish a reduction.

The comparison rejects regressions beyond baseline variation in median or p95 stream ping, decoder residence, submit-to-swap delay, reported packet loss, source FPS, native FPS, and decoded output rate. It also guards source interval p99, the worst window interval maximum, intervals beyond the histogram range, and growth in relative media lag. A lower late-frame fraction cannot pass by lowering playback rate or accumulating delay.

The runner invalidates windows containing presentation resets, counter rollback, changed epochs, PTS discontinuities, inactive video, missing telemetry, or recovery. Incoming assembled frames must average at least 57 FPS for the workload to qualify. Native accepted FPS is a separate decoder-admission outcome and is not treated as incoming FPS. Each report fingerprints the source revision and tracked diff, Qt executable, Rust streamer library, and benchmark script. The primary metric and acceptance rules are fixed before scored trials.

The stopping condition is an accepted reduction of at least 50% after at least four evaluated candidates, or documented exhaustion of the hypotheses with repeated trials showing a plateau. A failed or invalid candidate does not count as evidence of improvement.

### Method version 2

The first baseline exposed a flaw in the original validity rule. Its first window presented 58.95 FPS with 15 late intervals. The second fell to 52.9 FPS with 443 late intervals, and the runner rejected that window before saving its complete numerical summary. Low presented FPS is part of the problem under investigation, so excluding it would bias the experiment. The original invalid report is retained privately.

Before candidate experiments, method version 2 separates measurement validity from playback quality. Positive presented FPS up to 65 remains measurable. A window below 57 FPS records a quality failure, and a candidate with that failure cannot be accepted. Baseline and control windows retain their full measurements. The incoming workload check uses differences in `frameStageTimings.assembledFramesTotal`, while decode epoch and counter checks protect continuity. The source confirms that native `framesPerSecond` counts frames accepted by the decoder queue, so that value remains an outcome.

The primary metric and 25 ms threshold are unchanged. Baseline, candidates, and repeated control must all use method version 2 and the same frozen measurement script.

## Reproduction

Use the workspace at `/home/gamer/dev/gfn-client-research`. A full measurement takes about three minutes and twenty seconds once healthy playback begins, plus client startup and game-resume time. Use the same profile label and cloud session for each candidate and its controls. A new cloud session requires a new comparison block. Restarting the client interrupts playback. Close the existing client through its normal confirmation flow before selecting a variant. Launch the selected variant, then resume Ori through the normal GUI and return to the waterfall scene.

```sh
tools/run-live-trial.sh baseline
```

The launcher supports these variants:

| Variant | Local change under evaluation |
| --- | --- |
| `baseline` | Default settings for this instrumented build. |
| `poll1` | Request a one-millisecond embedded publication polling interval. |
| `continuous` | Enable continuous video update requests. |
| `vaapi` | Request the VA-API native video backend. |
| `basic` | Select Qt's basic render loop. |

These names describe requested configurations. A trial still needs confirmation that the selected backend starts and the real stream meets the validity checks.

Run a short calibration after the client displays healthy playback:

```sh
python3 tools/live-trial.py measure calibration \
  --profile germany-h264-1080p60-ori-waterfall \
  --variant baseline --warmup 0 --seconds 20 --windows 1
```

Calibration checks telemetry and lifecycle handling. It cannot satisfy the full experiment acceptance gate.

Run the baseline with the default durations:

```sh
python3 tools/live-trial.py measure baseline-a1 \
  --profile germany-h264-1080p60-ori-waterfall \
  --variant baseline
```

After launching and confirming a candidate, measure it with a distinct trial label. This example uses `poll1`:

```sh
python3 tools/live-trial.py measure candidate-poll1 \
  --profile germany-h264-1080p60-ori-waterfall \
  --variant poll1
```

Return to `baseline`, resume the same workload, and record `baseline-a2`. Compare the reports:

```sh
python3 tools/live-trial.py compare \
  .runtime/hillclimb/baseline-a1.json \
  .runtime/hillclimb/candidate-poll1.json \
  --control .runtime/hillclimb/baseline-a2.json
```

A comparison without the repeated control can only be provisional. Trial JSON, Qt telemetry, and native diagnostics stay under the private `.runtime` directory. Public results should copy aggregate numbers and methodological limits only.

## Next investigation

The current source points to a narrower next experiment around frame readiness and Qt update timing. `run_embedded_linux_monitor` in `native/opennow-streamer/crates/opennow-streamer-platform/src/media.rs` polls `try_recv_latest_frame`, publishes the latest decoded frame, then sleeps. The default interval is two milliseconds. The `poll1` experiment changes that interval to one millisecond.

`GraphicsFramePublisher::publish` in `native/opennow-streamer/crates/opennow-streamer-platform/src/graphics.rs` replaces the pending mailbox frame and invokes its frame-available callback. `NativeStreamRuntime::enqueueFrameAvailable` coalesces notifications before a queued GUI-thread drain. `StreamVideoItem` then handles `frameAvailable` through `requestFrame`, which calls `update`. The continuous variant adds another update request after each window swap while ordinary video is active.

These source paths and the measured mailbox replacements justify inspecting the time between decoder output, publication, Qt update request, and the next swap. A next candidate can wake the publication monitor when a decoded frame becomes ready, while keeping its bounded latest-frame storage and existing ownership. That is a proposed experiment. It has not been implemented or measured in this block.

The completed same-session comparison did not accept continuous update. A next block should first explain or control the large baseline phase variation, then compare a readiness notification with the polling path under the same metric and guardrails. It should also record the candidate's CPU cost. The current evidence does not justify a larger decoder or client rewrite.

## Limits

Qt's `frameSwapped` callback measures application presentation progress. It does not measure physical display scanout or input-to-photon latency. The source interval measurements count fresh decoded video outputs. They exclude window redraws that reuse the same video frame.

The primary metric applies to the selected 60 FPS workload. It does not measure packet loss directly. A mailbox replacement means that a newer decoded frame replaced an older pending frame. It does not by itself prove a visible skip.

Presentation histograms use one-millisecond bins through 511 ms. Durations of 512 ms or more increment a separate overflow counter. A percentile beyond that range has no finite upper estimate and cannot support acceptance. A window with overflow uses the timing epoch's maximum as a conservative upper bound for its longest interval.

Native decoder p95 values are rolling telemetry summaries. The report does not combine them into an all-frame percentile. Queue-drop reports flush at most once per second, so their attribution near a window boundary can shift by about one second. The observer does not independently count loss in the Qt callback queue. Optional process CPU samples can be distorted when several queued Qt telemetry lines arrive together, because they share a short reader interval. Those CPU values were not used for acceptance and cannot establish a CPU improvement.

The runner verifies video dimensions and display state. Codec, backend, cloud region, and game scene require confirmation in the normal client. The profile label records that confirmation. Relative media lag measures change against a local timing anchor and does not establish absolute network or input latency.

This loop evaluates local client changes on one live stream and workload. A successful result requires later confirmation across other scenes and sessions before a broad performance claim. HEVC startup negotiation remains a separate unresolved issue from the earlier investigation.
