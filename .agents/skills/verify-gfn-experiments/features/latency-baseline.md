# Descriptive latency baseline (schema 1)

## User path and prerequisites

Use the normal saved-account GUI to start/resume Ori and reach a stationary
animated scene. Preserve the user's current stream profile, desktop refresh,
window/fullscreen and overlay state; do not restore the old H.264 waterfall
experiment profile. Confirm dimensions, codec, region and source FPS in the
GUI before scoring. Capture the fixed scene before and after scoring.

Reuse a running client when it has fresh Qt telemetry. If it was launched
without `OPENNOW_LIVE_TELEMETRY`, obtain permission before closing/relaunching
with that flag; never silently restart, modify live logs, or start another
account/core owner. GUI actions use Cua Driver, background delivery first.
Foreground input needs explicit authorization. Build/hashing/capability work,
input checks and screenshots belong outside scored intervals.

## Agent entry point

One entry point, reusing the descriptive observer and additive census:

```sh
python3 tools/latency-baseline.py --pid CURRENT_QT_PID \
  --width VERIFIED_WIDTH --height VERIFIED_HEIGHT --display-hz 100 \
  --scene ori-inkwater-marsh-spirit-well --label YYYYMMDD-preflight1 --dry-run
python3 tools/latency-baseline.py --pid CURRENT_QT_PID \
  --width VERIFIED_WIDTH --height VERIFIED_HEIGHT --display-hz 100 \
  --scene ori-inkwater-marsh-spirit-well --label YYYYMMDD-a1 \
  --warmup 20 --seconds 60 --windows 3
```

Use a new safe label for every attempt. Explicit 60/100/120 Hz selects a
validation expectation, not a display setting. The program performs Doctor,
records process start ticks, source/build/tool hashes and allowlisted flags
before scoring, checks profile/state freshness, then measures N contiguous
windows using `observe-stream.py`. It preserves invalid/partial evidence.
It never launches, changes settings, authenticates or sends input.

Pair scoring with the existing socket/network observers when available:

```sh
python3 tools/live-socket-observer.py --pid CURRENT_QT_PID --duration 240 \
  --output NEW_PRIVATE_PROOF_DIRECTORY/sockets.json
python3 tools/live-network-observer.py --duration 240 \
  --output NEW_PRIVATE_PROOF_DIRECTORY/network.json
```

Those durations cover the default three windows; adjust them for N windows
and verify actual overlap afterward. Omit `--flint` unless router evidence is
specifically needed. Observers are descriptive metadata, not packet-age data.

## Observable result and evidence

Each new `.runtime/latency-baseline-LABEL/` contains `summary.json`,
`summary.md`, the full descriptive `observation.json`, and allowlisted numeric
samples for reproducible post-scoring summaries. `complete` requires every
requested window plus continuous presentation/native telemetry and valid
additive census. A zero-window preflight failure is **not** a latency baseline.
Publish only sanitized aggregates and limitations in `research/`.

- Network p50/p95/p99/max describe sampled native `pingMs`/`jitterMs`.
  They are not an all-packet RTT/jitter distribution.
- Qt fresh interval, submit-to-swap and oldest/latest GUI notification drain
  histograms give 1-ms bucket upper quantiles/max. Overflow tails are null,
  not invented exact maxima. Late intervals are strictly greater than 25 ms.
- Native decode call/residence expose overlapping rolling last-256-sample
  p50/p95/max. The report labels median rolling p50/p95 and greatest reported
  rolling max; true scored-window p99 is null. Do not add stage quantiles.
- Assembly-to-admission and ACK-queue timings exist in core telemetry, but
  this commit's timestamped native diagnostic summary omits their quantiles;
  they remain null in this entry point. Untimestamped stderr values are not
  substituted as window-aligned measurements.
- Qt callbacks are not physical presentation or input-to-photon. Mailbox/drop
  counts and GUI notification ages do not measure exact decoded-frame age.

## Coverage and cleanup

Record `latency-preflight`, `latency-windows`, `latency-summary` as pass/fail
with artifact paths. Dry-run alone does not pass `latency-windows`. Preserve
the reused client and all failed evidence. Wait for only the observers created
by this run; stop no unrelated processes. No settings need restoration because
the collector changes none. Keep raw evidence excluded from Git.
