# Network and socket comparison

## Sub-features

- `network-observe`: pair host/gateway and optional Flint numeric observations with playback windows.
- `socket-observe`: attribute queue/drop counters to the exact current client process.
- `network-compare`: change one authorized network variable and use matched controls.
- `network-restore`: restore the recorded original setting and verify its actual runtime effect.

## How to get to it (user POV)

Start the real stream and open its statistics overlay before scoring. The terminal observers collect numeric host/router/socket evidence while playback continues. Router graphs are available in LuCI Statistics and Services → Bandwidth Monitor; those graphs provide network context rather than the client frame score.

## Driving it with the network observer CLIs

Read the global operating and routing skills first. Preserve UTC incident time and inspect current observability before changing configuration. Router-forwarded syslog was stale in the October 7 block; verify freshness rather than assuming an active receiver has current data.

Run these bounded observers concurrently with a full trial, using new private output paths. Set `GFN_PID` to the exact current Qt PID returned by Doctor and confirmed by CUA; never use an old screenshot's PID. Start observers just before measurement and record any missing overlap.

```sh
python3 tools/live-network-observer.py --duration 360 --flint --output "$GFN_PROOF_DIR/network.json"
python3 tools/live-socket-observer.py --pid "$GFN_PID" --duration 360 --output "$GFN_PROOF_DIR/sockets.json"
```

Use separate managed command sessions for the two observers and the trial. Their 360-second limits cover the runner's 300-second deadline plus boundary samples. Start fresh observers with new output files for every retry; do not reuse the remainder of an older observer's lifetime. Verify actual overlap with every scored window afterward. Retain their session/PID ownership so cleanup can stop only these processes. The socket observer checks process start ticks, reads owned socket inodes, and emits local ports, queue sizes, and drop counters without peer addresses. The network observer's `--flint` mode uses one read-only SSH connection and five-second router sampling. Omit `--flint` when router access is unavailable and report the missing coverage.

For a configuration experiment, first record the original values and a concrete hypothesis. Warn before route, server, Wi-Fi, radio, or router-port changes because they can interrupt playback. Use existing task authorization; this skill adds no approval step. Stage the relevant timed rollback before a change that can remove access. Root-owned mutations belong to the single operator, outside scored windows.

Measure baseline, candidate, and restored baseline on the same cloud session and profile. If the session expires, establish a new block with new controls. Verify actual effects, including native `SO_RCVBUF` values when testing receive-buffer limits. Kernel limit changes affect new socket requests; an old process cannot prove the new allocation. Linux socket accounting can report twice the requested buffer size.

For restoration, read back the original sysctl/UCI/runtime values and affected sockets or queues. Cancel only the timer installed by this experiment after restoration is confirmed. Do not leave a rejected candidate enabled. Record the restoration artifact alongside the comparison decision.

## Gotchas

### Bounded native receive diagnostics

For receive/assembly localization, build with
`OPENNOW_DEVELOPER_RECEIVE_DIAGNOSTICS=ON tools/build-native.sh` while the client
is stopped. Set `OPENNOW_NVST_RECEIVE_DIAGNOSTICS_FILE` to a new absolute private
JSON path on the next client. Keep per-frame console logging and pacers off.
The probe observes the dedicated video receive owner for 180 seconds.

To capture the chosen scene, also set
`OPENNOW_NVST_RECEIVE_DIAGNOSTICS_ARM_FILE` to a new absolute marker path that
does not exist. Reach and verify the scene first, start matching external
observers, then create that marker. Verify `status=started` and
`timer_origin=operator_arm` in the private native log. A pending marker allocates
no capture bins and starts no timer; checks occur at most every 100 ms. Without
an arm path, the timer begins at worker setup and may cover loading instead.
The capture is one-shot per process; restart normally for another attempt.
`OPENNOW_NVST_RECEIVE_DIAGNOSTICS_DURATION_SECONDS` may select an integral
180–1800 second window. Default is 180 seconds; invalid values reject the
capture. Verify actual `duration_ms` and `bin_count` in the output rather than
assuming a requested duration was applied. Longer windows retain the same
bounded phase-tail and gap-episode capacities; inspect omission counts.

The same armed capture writes a sibling `*.fec-head.json` with bounded FEC
head-wait episodes. Companion schema 2 classifies incomplete heads and wholly
missing FEC blocks. For a wholly missing block, frame/block coordinates belong
to the first available successor; the expected base identifies the missing
range. It also records at most 128 RTP gap episodes and 32 effective incoming
FEC shard tuples, with omitted counts. Tuple counts are authenticated packets,
including stale or duplicate packets, rather than distinct blocks. Zero
incomplete-head episodes alone does not exclude a missing-block wait.
It retains at most 128 completed FEC episodes plus the pending
head, with omission counts. A ready-successor scan occurs at opening or timer
renewal, and at most every 16 ms otherwise using the receive owner's existing
clock. The probe changes no NACK, dequeue deadline, or recovery behavior.
Require same-head deadline restarts, ready successors, and observed head age
above 52 ms before attributing delay to deadline renewal. Preexisting heads have
unknown earlier history; observed ages are lower bounds. `complete` records the
capture timer boundary. Early worker exit attempts a partial export; require
the actual file before using that evidence. The
companion's wall-clock origin is approximate. Check both files and their
omission counts; an empty episode list alone does not establish continuous
assembly or playback.

Require the expected PID/thread role, clock anchor, actual scene overlap,
authenticated/assembled endpoints, and the probe's cutoff/omission/error counts
before attributing a stall. `complete` means the timer elapsed, not full scene
coverage. Receive timing is userspace socket return timing, not kernel arrival
or physical packet loss. Preserve partial and rejected captures.

With the diagnostic output configured, `input-proof` records contain queue
success counts and first/last/max-gap timing. They omit input contents. SCTP
queue acceptance is not remote delivery; verify an ordinary gameplay effect
through CUA too. Use verified periodic gameplay input for an active control and
retain its cadence. A no-input terminal remains confounded by server inactivity.

### Interpretation limits

- Gateway ICMP, public ICMP, endpoint TCP preflight, native stream ping, and router CAKE delay measure different paths or stages. Label each source and time interval.
- Observation duration must cover warmup, scoring and retries. If an observer expires during a retry, mark its overlap as partial and start a new observer for later windows. Persistent host samples can supplement their own metrics, but cannot supply missing router or UDP-counter evidence. Never treat an expired observer as full coverage.
- Zero native packet loss does not exclude kernel UDP socket drops. Socket drops alone do not identify every dropped datagram as video RTP.
- Router station retry/failure counters need validated driver semantics. Aggregate router drops are not automatically GFN losses.
- A larger receive buffer may reduce socket overflow while increasing delay. It still has to pass the unchanged playback and latency gates.
- Metadata from encrypted traffic does not reveal decrypted session content. Keep raw captures private and publish only justified header/timing observations.
- Preserve Steam's Wi-Fi scan guard, recovery services, and memory protection. Read the current custom-firmware state; older SQM/radio defaults are not a restoration target.

## FEC deadline experiment

`rtt_one_frame_v1` was rejected in the 12 October live trial: six eligible
heads expired, four used extra waiting, and none completed recovery in the
observed window. Keep it disabled. The partial comparison is retained as
negative episode evidence, not an aggregate performance acceptance.

This default-off candidate requires a client built with receive diagnostics.
Set `OPENNOW_FEC_DEADLINE_MODE_FILE` to an absolute, ordinary local file owned
by this experiment. Its exact contents are `fixed52` or `rtt_one_frame_v1`,
optionally followed by one newline. Both arms poll at most every 100 ms. A
mode applies only when the receive owner's RTP and FEC gap waits are empty.
Require an applied receipt with `mode`, `revision` and `no_active_gap=1`; a
staged receipt does not establish application. Missing or invalid content
keeps the current mode. No environment variable preserves default behavior.

The candidate extends only an incomplete FEC head after its first successful
NACK covering the entire current missing run, admitted before the initial
52 ms deadline with a measured RTT. It latches at most one deadline, capped
at 69 ms. A changed first-missing packet or legacy timer disables the
extension for that head and restores the current legacy timer. NACK wire
format, tracking/retries, whole-block/RTP deadlines, buffer limits and
reference recovery on failure stay unchanged. Completion is still checked
before timeout, as in baseline.

Use the same uninterrupted cloud session and profile for A–B–B–A windows.
Sparse `fec-deadline-timing` events cover all arms independently of the
one-shot receive capture, with a 4096-event cap and omission summaries.
Count `eligible_admission` in both modes; `extension_latched` is candidate
only. Positive recovery evidence needs `extension_used`, a matching
post-NACK application observation, and completion within the extended
window. Late completion alone does not prove that baseline would expire.
Arrival watches close at head completion/expiry, so no match after baseline
expiry does not prove that no retransmission response arrived. These are
application observation timestamps, not kernel arrivals or independent RTX
identification.

Require at least three eligible episodes per arm before accepting a
performance result. Reject a candidate that only delays the same expiries
or worsens reference recovery or latency. Keep incomplete comparisons and
restore `fixed52` before cleanup. Source checks do not establish live
application or performance; retain mode receipts and measured evidence.

## Fast triage feedback

For immediate feedback beside live telemetry-enabled playback, run
`python3 tools/fast-stall-feedback.py --help`, then supply the verified current
Qt PID and a new private output path. The observer reports frame drops,
presentation gaps and repeated over-target stream RTT with gateway ICMP,
observer wake delay and owned UDP counters. Its defaults match the current
zero-drop, under-62-ms target; change thresholds before an experiment.

Use this to reject a bad candidate early. `no_incident_in_this_window` is a
bounded observation, not evidence of sustained improvement or physical scanout.
Retain invalid runs. Log rotation, changed presentation identity and stale
telemetry stop observation. Gateway correlation does not establish RF causality.
Use the scored measurement and matched-control procedure for acceptance.
