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

- Gateway ICMP, public ICMP, endpoint TCP preflight, native stream ping, and router CAKE delay measure different paths or stages. Label each source and time interval.
- Observation duration must cover warmup, scoring and retries. If an observer expires during a retry, mark its overlap as partial and start a new observer for later windows. Persistent host samples can supplement their own metrics, but cannot supply missing router or UDP-counter evidence. Never treat an expired observer as full coverage.
- Zero native packet loss does not exclude kernel UDP socket drops. Socket drops alone do not identify every dropped datagram as video RTP.
- Router station retry/failure counters need validated driver semantics. Aggregate router drops are not automatically GFN losses.
- A larger receive buffer may reduce socket overflow while increasing delay. It still has to pass the unchanged playback and latency gates.
- Metadata from encrypted traffic does not reveal decrypted session content. Keep raw captures private and publish only justified header/timing observations.
- Preserve Steam's Wi-Fi scan guard, recovery services, and memory protection. Read the current custom-firmware state; older SQM/radio defaults are not a restoration target.
