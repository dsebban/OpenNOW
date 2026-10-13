# Ori long observation, October 11

The 12-minute diagnostic window reproduced transient stalls and a terminal
session failure. Every qualifying publish gap had its first observed long gap
at enqueue, before submit/decode/publish. This favors investigating availability
of assembled video frames before changing Qt pacing or GPU-copy scheduling.
It does not establish whether packets arrived late or were processed late.

The terminal incident also supplies a separate control-channel lead: an RTSPS
protocol read failure followed by exhausted recovery. No mitigation was applied
and no performance improvement is claimed.
Because this was an idle scene with no keepalive input, normal server inactivity
termination remains a confound; this incident alone does not establish a client
keepalive bug.

## Setup and validity

Observation ran from **22:52:20 to 23:04:20 Israel time**, October 11, 2026.
The saved waterfall scene warmed up before the start. The initial GUI launch
failed while checking an older cloud session; normal Try again recovered it.
That failure and all menu/loading activity were outside the observation window.

- Display remained 3440x1440 at 100 Hz; desktop mode and HDR settings were
  retained. Stream settings remained Germany, HEVC, 2560x1080 at 60 source FPS,
  10-bit 4:2:0, frame generation off. Doctor passed for this explicit profile.
- Same deployed build as the prior continuation, source revision
  `4e5ea00790c387a514a370608e826889db9a3cdd` plus existing uncommitted work.
  No rebuild was performed. Build hashes are retained privately.
- `OPENNOW_FRAME_STAGE_LOG=1` was the only enabled checked experiment flag.
  Both pacers and deferred snapshot wait were unset.
- Direct routing and `dani_5` association were preserved. No active scan,
  association, router, VPN, firewall, DNS, buffer, or display change was made.
- No gameplay input or keepalive pulses were sent during the observation.
  Two diagnostic screenshots inspected scene/failure state during the window;
  this was not a scored acceptance trial.
- All 7200 scheduling samples retained the same Qt PID/start-time identity.
  Wall-time versus monotonic span differed by approximately 7 microseconds.

**Actual video production ended after 7 minutes 32 seconds.** The session
reported stopped after 7 minutes 48 seconds. The remaining part of the
12-minute window is observation after failure, not healthy playback or a
zero-stall control. Initial and midpoint screenshots show the same game
location; the later screenshot shows the library after automatic teardown.

## Timing and local-link observations

| Metric | Result |
| --- | --- |
| Recorded frame-stage lines | 106097 |
| Publish records | 26521 |
| Publish gaps over 40 ms | 56 |
| Publish gaps at or above 67.112 ms | 31 |
| Qualifying-gap clusters, merging separations of at most 1 s | 11 |
| Largest completed publish gap | 5351.142 ms, followed by recovery |
| First observed stalled stage in qualifying gaps | enqueue, 31/31 |
| Qualifying gaps overlapping estimated gateway RTT over 20 ms | 18/31 |
| Gateway replies | 7045/7045 over the full observation |
| Gateway median / p99 during frame production | 1.74 / 31.1 ms |
| Gateway maximum over the full observation | 265 ms |
| Observer heartbeat maximum lateness | 0.778 ms |
| Maximum sampled thread run-queue wait delta in stall-overlap buckets | 0.991 ms |
| Observed process-owned UDP receive-drop delta | 0 |
| Maximum sampled UDP receive queue | 55296 bytes |
| Host softnet drop / time-squeeze deltas | 0 / 0 |

The longest completed pause was followed by further gaps of roughly 906,
539, 429 and 309 ms during a recovery burst. Several keyframe-request log
records cluster in that period. They are log records, not independently
verified wire requests.

The available station counters increased by 118687 transmit retries and four
transmit failures over the full mixed playback/after-failure interval. Beacon
loss and receive-drop-misc did not increment. Counters cover the host's whole
association, including other traffic, and do not measure GFN packet loss.
Only five of the 31 qualifying gaps overlapped a station retry-per-packet
bucket above the same-run playback p95. Thus the current evidence does not
justify treating every stall as an RF retry event.

The passive Wi-Fi listener covered only the later 300 seconds. It recorded
100 signal-threshold events and no scan, disconnect, roam or channel events.
Its missing early coverage cannot rule out an earlier event. No relevant
kernel records were found in the collected journal slice.

## Terminal incident

| Time after observation start | Recorded event |
| --- | --- |
| 452.379 s | Last publish record; no later frames in the capture |
| 454.715 s | RTSPS `control_exit reason=read-failed error_category=Protocol io_kind=None` |
| 460.395 s | Native status reports `streaming` during recovery, without resumed frames |
| 468.400 s | `nvst-recovery-exhausted`, followed by status `stopped` |

At control-worker exit, uptime was 884872 ms, with 425 pings sent, zero matched
pongs, 424 missed pongs and no inbound control messages since that worker
started. These counters begin after RTSP setup. They do not count video/audio
UDP traffic and do not prove that the server requires or supports the chosen
keepalive behavior.

The worker sends WebSocket Ping frames and counts only matching Pong payloads
in [the current implementation](../vendor/OpenNOW/native/opennow-streamer/crates/opennow-streamer-core/src/nvst_rtsp.rs).
Its diagnostic groups all tungstenite protocol errors into `Protocol`, so the
exact read-error subtype is unknown. Media stopped before the logged control
exit; temporal proximity alone does not establish which failure caused which.
The terminal stop was about 11 minutes after the Continue-menu screenshot and
9 minutes 43 seconds after the scene-ready screenshot. The final menu input
occurred between them. Reproduction with verified periodic input is needed to
separate normal server inactivity handling from an abnormal terminal failure.

## Decision and next investigation

There is enough evidence to move investigation upstream of decode and Qt
presentation. There is insufficient evidence to select a radio change or to
claim that increasing receive buffers would help.

The next two focused questions are:

1. During transient stalls, did video datagrams reach the socket while frame
   assembly stopped advancing, or did the datagrams themselves stop arriving?
   Tie receive/assembly timing and keyframe recovery to frame identity.
2. For terminal failures, is the negotiated RTSPS keepalive contract satisfied,
   and which payload-free protocol-error subtype ends the control worker?
   First compare idle observation against verified periodic input. Reproduce
   recovery independently of the transient-stall metric. Do not
   assume missing Pong counters alone explain termination.

Limitations remain: ping send time is estimated from a userspace reply stamp
and RTT, not a kernel arrival stamp. Schedstat samples are cumulative buckets,
not scheduler wakeup/switch traces, and cannot exclude blocked threads or
kernel delays. Socket polling can miss a final increment just before socket
closure. The threshold identifies logged frame-availability gaps, not physical
scanout or input-to-photon latency. No new performance candidate passed an
acceptance comparison in this run.

## Artifacts and observer changes

Private evidence is in `.runtime/ori-long-20261011/`: `link-stalls.json`,
`sockets.json`, `host-context.json`, `frame-stages.log`, numeric native logs,
control-exit metadata, screenshots, build/identity/flag readbacks, checkpoints
and analysis JSON. The private analysis helper reads the retained stage-only
file, so subsequent client logs do not replace this run's evidence.

`tools/observe-link-stalls.py` now admits bounded runs up to 900 seconds and
optional numeric progress output once per minute. Its 720-second real-client
run completed with private output and unchanged process identity. The socket
observer completed 725 samples with two tracked sockets. The passive listener
ended by its expected timeout. No build or test suite was run.

After evidence collection, the measured Qt/core instances exited through the
normal Quit confirmation and their process identities were verified gone.
Native logs and Qt telemetry were archived privately. Ordinary Ori playback
was restored with frame-stage logging, Qt telemetry and all checked experiment
flags unset. The restored client is intentionally left running for the user;
its identity and restoration screenshots are retained in the evidence folder.
Routing, association and display settings remained unchanged.
