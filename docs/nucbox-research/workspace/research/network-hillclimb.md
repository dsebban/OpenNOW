# Network context for the Linux presentation experiments

Four completed observer blocks collected 4,838 host samples, 960 router samples and 4,838 gateway replies across ten scored trials. The final block stopped after 1,079 host samples when the final trial ended. Each block used one persistent read-only Flint SSH connection, numeric counters every five seconds, and host metadata/gateway probes near one Hz. No route, radio or QoS configuration changed.

Each row covers three scored 60-second windows from the corresponding real Qt trial. CPU percentages cover all four router cores. Counter changes use the nearest samples outside the scored interval, so host boundaries extend by about one second and router boundaries by about five seconds per side.

| Trial | Valid gateway probes / replies | Gateway RTT median / p95 / max, ms | Host UDP receive-buffer errors | Router CPU median / p95, % | WAN queue drops | Download IFB queue drops |
| --- | --- | --- | --- | --- | --- | --- |
| Baseline A1, corrected measurement | 181 / 181 | 2.71 / 5.56 / 6.23 | +431 | 6.94 / 8.18 | +486 | 0 |
| Continuous Qt video update | 180 / 180 | 0.80 / 5.18 / 91.90 | +221 | 7.05 / 8.44 | +905 | 0 |
| Publisher polling at 1 ms | 180 / 180 | 3.16 / 6.79 / 13.20 | 0 | 6.86 / 9.17 | +1164 | +3 |
| Basic Qt render loop | 180 / 180 | 0.62 / 2.40 / 43.50 | +566 | 7.38 / 10.13 | +2601 | +1 |
| Baseline A2 | 182 / 182 | 0.58 / 2.41 / 22.80 | +396 | 6.36 / 10.69 | +2541 | +2 |
| Continuous Qt update B2, partial network coverage | 118 / 118 | 1.59 / 6.57 / 8.84 | Unmeasured whole interval | 6.93 / 9.25 | Unmeasured whole interval | Unmeasured whole interval |
| Baseline A3 | 182 / 182 | 0.81 / 4.99 / 12.50 | 0 | 7.64 / 11.37 | +1842 | +5 |
| Baseline A4, third seat | 181 / 181 | 2.61 / 5.70 / 29.60 | 0 | 7.34 / 9.79 | +735 | +3 |
| 8 MiB receive allowance B2, third seat | 181 / 181 | 0.57 / 2.96 / 47.20 | 0 | 7.30 / 9.59 | +1134 | 0 |
| Restored-cap baseline A5, third seat | 181 / 181 | 0.73 / 3.58 / 22.40 | +26 | 7.43 / 9.45 | +723 | +1 |

All observed gateway probes received replies, and measured memory-pressure `some.avg10` remained zero. Continuous B2's first scored window and first seconds of its second window fell between network-observer blocks; its gateway/router statistics describe the observed 118 samples, not full-run conditions. Its separate PID socket observer covered all three windows. Earlier and later trials have full network coverage.

Host UDP checksum, send-buffer and memory-error counters did not increase in the fully covered trials. `InErrors` matched receive-buffer errors for the first five rows and the third-seat A4/buffer-B2/A5 sequence. Baseline A3 had one additional `InErrors` increment with no receive-buffer increment, so that error remains separately unexplained. `NoPorts` increased by 4, 8, 2, 11, 10, 3, 11, 14 and 7 in the nine fully covered trials. These are host-wide counters.

The first observer block's changing receive-buffer errors are a confound worth retaining. They belong to every UDP socket on the host, and that block had no process/socket attribution. Its baseline A1 increase of 431 and continuous-update B1 increase of 221 occurred during each trial's first scored minute; its 1 ms polling trial had no increase. The corrected baseline's first minute began roughly ten minutes after stream startup, so this timing does not establish a startup/recovery cause. Later attributed drops occurred in other scored windows, as shown below. OpenNOW's reported zero stream loss does not prove every host UDP socket was healthy. Continuous-update B1 also had a 91.9 ms local gateway RTT sample, although its p95 remained below 6 ms. A repeated candidate comparison should account for that tail rather than treating the whole network path as identical.

Router load remained modest in these samples. WAN and IFB queue counters include all household traffic; overlimits are shaper activity, not an additional packet-loss count. WAN queue drops increased across all trials, while download drops were almost absent. These aggregates cannot establish coincidence with individual late video swaps or a client-code cause. Gateway ICMP measures local reachability and RTT, not GFN in-stream UDP latency.

The VA-API candidate failed capability validation before playback. This is not evidence that the AMD hardware lacks H.264 decode. The embedded H.264 SDR backend uses native cros-codecs and requires a usable decode context plus an NV12 surface. Supplemental FFmpeg VA-API color probing explicitly excludes H.264. The core then replaces the original capability failure with a generic unsupported-color message when `colorQualities` omits `8bit_420`.

Source references at the reviewed fork are `native/opennow-core/src/streamer.rs:494`, `native/opennow-streamer/crates/opennow-streamer-platform/src/lib.rs:174`, `opennow-streamer-platform-linux/src/video/vaapi_probe.rs:164`, `opennow-streamer-platform-linux/src/video/vaapi.rs:229`, and `opennow-streamer-platform-linux/src/session.rs:1253`, with the last three paths under `native/opennow-streamer/crates/`. Preserve the original failure reason and validate context/surface export before repairing or selecting an alternate H.264 VA-API path. Do not bypass the capability gate.

Private timestamps, per-window numeric values and bracketing intervals are retained in `.runtime/hillclimb/network-comparison.json`. This report contains aggregate measurements only. No payloads, account/session identifiers, credentials or connection contexts are included.

## Later socket attribution during the basic Qt trial

A separate observer matched the exact Qt process's file-descriptor socket inodes against UDP namespace tables. Its video socket on local UDP 49005 accumulated 566 receive drops during the second scored minute: increments of 166 and 400 were observed at 13:12:46.456 and 13:12:58.456 UTC. The first and third scored minutes had no new observed drops, and the bundle socket on UDP 49006 had none. Native log samples for all three minutes still reported zero packet-loss percent.

These are verified drops on that client's UDP video socket. One-Hz sampling places each increment between the preceding and current sample; it cannot identify exact dropped datagrams or prove coincidence with a particular video-swap interval. UDP 49005 also carries NATT/STUN, so the counter alone cannot classify every discarded datagram as authenticated video RTP. This result does not retrospectively attribute the earlier dead processes' host-wide errors.

The later PID socket observers establish the following per-window video-socket increments. All seven completed observers covered their scored windows without a drop interval crossing a window boundary. Most ended when their target processes exited; A5 was stopped after scoring. Those end statuses do not invalidate the completed windows.

| Trial | Window 1 video drops | Window 2 video drops | Window 3 video drops | Bundle drops across scored windows |
| --- | --- | --- | --- | --- |
| Basic Qt render loop | 0 | 566 | 0 | 0 |
| Baseline A2 | 0 | 0 | 396 | 0 |
| Continuous Qt update B2 | 0 | 0 | 0 | 0 |
| Baseline A3 | 0 | 0 | 0 | 0 |
| Baseline A4, third seat | 0 | 0 | 0 | 0 |
| 8 MiB receive allowance B2, third seat | 0 | 0 | 0 | 0 |
| Restored-cap baseline A5, third seat | 0 | 26 | 0 | 0 |

The zero-drop A3 control shows why continuous-update B2's zero drops alone cannot establish an improvement from continuous updates. The earlier host-only errors remain unattributed. These observations require repeated matched cadence/latency comparisons and do not replace the frozen presentation score.

A subsequent read-only check found `net.core.rmem_max` and `rmem_default` both at 212,992 bytes. The then-running baseline client requested an 8 MiB receive buffer in `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:5358`, but actual socket-memory readback showed a 425,984-byte receive allocation on both UDP sockets. Its fresh counters were zero at that instant. The socket limit prevents the requested buffering; a successful setting call alone does not establish the requested size.

The receive worker calls `recv_from` once, then authenticates, repairs FEC, assembles frames and forwards events before reading the next datagram, `nvst.rs:7485` and `:7545`. Its 10 ms read timeout is an idle wait, not a fixed delay for every arriving packet. Decode is elsewhere, but repair/assembly and synchronous diagnostics can delay drainage. No kernel drop ancillary data is collected by `recv_from`. Native network metrics use received authenticated RTP sequence counters, `nvst.rs:569` and `:1118`; retransmission can close a deficit, and sparse logged snapshots can miss short deficits. Zero logged loss is therefore not a measurement of zero kernel socket drops.

The first receive-buffer attempt was not run. Noninteractive sudo refused the rollback-timer command because authentication was required. That attempt ended before mutation: no rollback timer was created, and both caps stayed at 212,992 bytes.

A later administrator-authorized setup temporarily raised only `rmem_max` to 8,388,608 bytes at 13:40:52 UTC, leaving `rmem_default` at 212,992 and arming a system-owned rollback for 13:55:52 UTC. The previous cloud seat expired before scoring. Root restored the cap and cancelled the timer around 13:47 UTC; this setup produced no measured buffer result.

A new third cloud seat is used for baseline A4, buffer B2 and closing baseline A5, with a fourth network-observer block covering that sequence. A3 belongs to the earlier seat and is not a matched buffer control. A4 completed with zero video/bundle socket drops in all three scored windows and actual receive allocation of 425,984 bytes. Despite that, its fresh source rates were 55.23, 50.13 and 56.21 FPS, with 286, 601 and 227 late intervals. The presentation problem can therefore occur without kernel socket drops in this control.

The subsequent buffer B2 startup readback confirmed 16,777,216-byte receive allocation on both sockets, `rmem_max=8388608` and unchanged `rmem_default=212992`. Buffer B2 completed with no observed socket drops in any scored window. Fresh source rates were 56.47, 59.30 and 58.36 FPS, with 210, 42 and 103 late intervals. Both A4 and buffer B2 are structurally valid trials but failed the presented-FPS quality floor in at least one window. A4 already had no receive drops, so fewer kernel drops cannot explain the apparent cadence difference.

Root restored the cap and stopped the rollback timer around 14:00 UTC before reopening control A5 on the same third seat. A5 readback verified both caps at 212,992 and both socket allocations at 425,984 bytes. Its fresh source rates were 59.85, 56.20 and 53.48 FPS, with 11, 220 and 392 late intervals. It accumulated 26 video-socket drops in window 2; the bundle remained at zero.

The completed same-seat A4/B2/A5 comparison rejected the larger-buffer candidate. Its median late-interval rate was 2.8933%, versus 7.6800% for the pooled original-cap controls, a raw 62.33% reduction. However, control spread was 19.347 percentage points, the candidate failed the FPS floor, and native median/p95 stream-ping guards worsened. The effect did not exceed those acceptance limits. Local gateway RTT and native stream ping are separate measurements; B2's lower gateway median does not override the stream-ping gate. No larger receive-buffer default or performance win is adopted.

For a later authorized trial, stage a system-owned 15-minute restore of `net.core.rmem_max=212992`, verify that timer before changing only `rmem_max` to 8,388,608, restart the local client, and verify actual receive allocation. Restore the original cap and restart before scoring the control. Leave `rmem_default`, routes, radios, QoS and media queues unchanged. The private command plan is retained with the experiment artifacts. Larger buffering may trade drops for stale backlog; it must preserve cadence and latency to count as a win. A receive/assembly split would require a new bounded queue and overflow policy and should follow evidence that larger socket buffering alone is insufficient.
