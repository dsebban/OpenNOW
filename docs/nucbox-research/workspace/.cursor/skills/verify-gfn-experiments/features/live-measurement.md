# Measure real playback

## Sub-features

- `measure-calibration`: confirm fresh telemetry and full numeric output over one short window.
- `measure-full`: record 20 seconds of warmup and three 60-second scored windows.
- `measure-rejection`: retain a low-quality, invalid, or rejected result with its reason.
- `measure-compare`: compare a candidate against controls on the same cloud session.

## How to get to it (user POV)

Launch a measured OpenNOW variant, resume Ori, and return to the animated waterfall. The terminal measurement command then watches the existing stream. A full measurement takes about three minutes and twenty seconds after healthy playback begins, plus startup and resume time.

## Driving it with the live trial CLI

Run from the repository root after `doctor.py --require-stream` passes at the explicitly requested display rate. Method 3 accepts `--display-hz 60` or `--display-hz 120`, defaults to 60, and checks every Qt readback within 1 Hz of that value. Source video remains 60 FPS. Choose unused safe labels. The commands below establish a new block named `verify-seat1`; increment that suffix when a new cloud session is allocated. Reusing a suffix from an earlier cloud session would make the evidence misleading.

Before launching each measured client, verify its predecessor's Qt/core instances exited. Then archive accumulated logs into an unused directory with an existing private parent:

```sh
python3 tools/archive-trial-logs.py --output "$GFN_PROOF_DIR/prelaunch-logs"
```

The helper refuses workspace Qt/core processes, symlink inputs, and existing output directories. It moves only `native-streamer.log`, `native-streamer.log.previous`, `native-streamer.previous.log`, and the Qt presentation JSONL, recording size, original inode, SHA-256, and destination in a private manifest. It never prints log contents. Preserve the archive; use a new directory for each attempt. A copy without removing the stopped active generation leaves the accumulated rotation risk in place. Never archive or truncate a log while the client is alive.

After healthy playback starts, check native-log size and recent growth before scoring. The 2 MiB cap applies during the run, so a fresh file alone cannot guarantee sufficient space if diagnostics become unusually noisy. Preserve any rotation-invalidated trial. Do not relax the collector's continuity check.

Start new network and socket observers for every attempt, including retries. Use 360 seconds per observer immediately before measurement, covering the runner's 300-second deadline and samples on either side of the scored windows. Verify actual coverage afterward; a finished observer does not imply full overlap. Analyze copied or archived telemetry outside scoring, and retain the original trial status beside any ancillary census result.

```sh
python3 tools/live-trial.py measure verify_calibration --profile germany-h264-1080p60-ori-waterfall-verify-seat1 --variant baseline --warmup 0 --seconds 20 --windows 1
python3 tools/live-trial.py measure verify_a1 --profile germany-h264-1080p60-ori-waterfall-verify-seat1 --variant baseline
```

A calibration can check parsing and continuity, but cannot pass full acceptance. For a candidate, close/relaunch through the GUI using the intended `run-live-trial.sh` variant, resume the same cloud session, and restore the identical scene/UI state. For example, after launching `continuous` and resuming the scene:

```sh
python3 tools/live-trial.py measure verify_continuous --profile germany-h264-1080p60-ori-waterfall-verify-seat1 --variant continuous
```

The readiness-wait experiment is preserved on `nucbox-readywake-trial`; it is not a supported active launcher variant. Retain its measurements and rejection/crash evidence. The candidate scored worse; the later Qt XCB crash does not establish that readiness waiting caused it. For any future explicitly selected trial branch, build before the comparison block and keep that binary fixed. Retain the owning operator's allowlisted flag readback for candidate and controls. Doctor verifies stream readiness, not experiment flags or their implementation.

After restoring and resuming baseline:

```sh
python3 tools/live-trial.py measure verify_a2 --profile germany-h264-1080p60-ori-waterfall-verify-seat1 --variant baseline
python3 tools/live-trial.py compare .runtime/hillclimb/verify_a1.json .runtime/hillclimb/verify_continuous.json --control .runtime/hillclimb/verify_a2.json
```

Read all three trial files and the comparison. Verify the controls have the same variant as each other, the same source/build/script fingerprints, profile, cloud session, and method version. Candidate variant metadata must match the actual launch. Profile labels are operator assertions, so GUI proof must support them. Do not compare different cloud sessions or conceal a changed UI theme behind an unchanged label.

For an explicitly planned display experiment, use `--display-hz 60` on the baseline and restored control, and `--display-hz 120` with a distinct candidate variant. Confirm the hardware mode and Qt readback; the flag itself changes no settings. The comparator requires equal requested display rates for both controls and permits the candidate's explicit different request. At a 120 Hz display with 60 FPS source video, window callbacks without a fresh source are expected. Their fraction cannot be interpreted as dropped-video percentage.

Method 3 changes only requested-display validation and metadata. The method-2 implementation and old reports are preserved; the method-3 comparator rejects old or mixed-method reports. Start new controls before candidate scoring. The primary score, 25 ms threshold, all FPS floors, and all acceptance/latency/throughput gates are unchanged.

The frozen score uses the median window fraction of fresh source intervals strictly greater than 25 ms. Acceptance needs at least a 20% relative reduction, an absolute reduction exceeding pooled control spread, and all latency/playback guards. Every candidate window must present at least 57 FPS. Positive source playback below 57 FPS remains measurable for baselines and controls and records a quality failure. Incoming assembled frames must average at least 57 FPS to establish the intended workload. Decoder-admission FPS is a separate outcome.

The inherited method-2 noise gate deliberately uses a conservative maximum-minus-minimum control range. That range can exceed the baseline median, making acceptance impossible even for a zero-late candidate. Such a rejection is inconclusive about whether the change can help. Use this method for first triage, then diagnose changing workload or scheduling phases. A later method may use repeated independently matched blocks and a different uncertainty estimate, but version and freeze it before new candidates, rerun controls, and retain the old verdicts. Require enough repeats to clear measured noise; never select a lucky minimum or quietly relax gates after seeing a candidate.

The runner rejects continuity failures such as counter rollback, changed epochs, PTS discontinuities, missing telemetry, inactive presentation, or recovery. Comparison guards include median/p95 ping, decoder residence, submit delay, source interval p99, worst interval, histogram overflow, relative media-lag growth, packet loss, and source/native/decoded throughput. Preserve every window's numbers and stated reasons.

Record the decision before another candidate. Freeze the script, binaries, settings, workload, and acceptance rules for the block. If scoring instrumentation must change, retain the old artifacts, give the method a new version, and rerun controls before candidates. Additive diagnostics have a separate schema version; they still require fresh controls on the new build and must not alter the frozen acceptance decision. The current stop rule requires an accepted 50% reduction after at least four evaluated candidates, or documented exhausted hypotheses with a repeated plateau.

For builds with the window/notification census, run this after scoring:

```sh
python3 tools/presentation-census.py --trial .runtime/hillclimb/verify_a1.json --telemetry .runtime/hillclimb/qt-presentation.jsonl --output .runtime/hillclimb/verify_a1-census.json
```

The schema-2 census aligns its counters to the scored windows and does not change the frozen score. It reports all-window swaps separately from fresh-source swaps, notification enqueue/drain/emission rates, runtime acquisition/recording outcomes, and render prepare/import/draw/fresh-submit counts. Oldest/latest drain delays concern pending notifications, not exact GPU-frame age. Window swaps without a fresh source are not automatically repeated-video scanouts. Its `complete` status describes analysis of retained windows, including windows from an invalid partial trial; it does not make that trial valid. Keep the matching Qt JSONL; the reader rejects missing fields on old builds and changed epochs. Run its bounded file scan outside scored windows.

## Gotchas

- Existing labels overwrite the runner's result path. Choose a new label for every attempt; never erase the failed trial to make a retry look like the original.
- Native diagnostic logs rotate to `.previous` at 2 MiB. If rotation invalidates a run, preserve its result and retained windows, then repeat under a fresh label. Do not delete or truncate logs during playback or relax continuity checks to recover a valid verdict.
- Do not discard a slow baseline as invalid merely because it exposes the problem. Workload validity and playback quality have separate rules.
- Lower late-frame counts do not establish improvement when source throughput falls or frames accumulate delay.
- Qt frame callbacks do not measure physical scanout. Queue/mailbox counters cannot be summed into exact visible frame losses.
- Optional process CPU samples can be distorted when Qt lines arrive in batches. They are excluded from acceptance and CPU performance claims.
- Full traces are private. Publish aggregate numbers and the exact rejection reasons, including preflight failures with no measured windows.
