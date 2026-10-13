# Network context for the Qt idle-time trial

These are ancillary observations for the idle-time A1/B1/A2/A3 block. They do not change the frozen presentation score. A2 was invalidated by native diagnostic-log rotation in its third window; its two retained windows remain useful evidence, not a complete closing control.

| Trial | Scored windows retained | Dedicated gateway replies / probes | Gateway RTT median / p95 / max, ms | Video socket drops by window | Router CPU median / p95, % | WAN / IFB drop increments |
| --- | --- | --- | --- | --- | --- | --- |
| Baseline A1 | 3 | 181 / 181 | 1.01 / 4.46 / 16.50 | 0 / 0 / 72 | 7.68 / 9.50 | 1094 / 1 |
| Zero-idle B1 | 3 | 180 / 180 | 1.05 / 9.05 / 28.90 | 0 / 466 / 0 | 8.85 / 11.01 | 1662 / 2 |
| Baseline A2, invalid partial run | 2 | 121 / 121 | 0.67 / 5.10 / 37.30 | 393 / 433 | 7.12 / 9.71 | 912 / 0 |
| Replacement baseline A3 | 3 | 47 / 47, incomplete | 0.66 / 73.50 / 147.00, partial only | 0 / 0 / 0 | 7.48 / 12.10, partial only | Unmeasured whole interval |

All video/bundle socket observers covered their scored windows with stable process-owned socket identity; bundle socket drops were zero. Whole-host receive-buffer error increments matched video-socket drops for fully covered A1/B1/A2 intervals. Counter boundaries extend to nearby host/router samples and include other household traffic. Router queue drops are not automatically GFN packet losses.

A3's dedicated network observer expired 47 seconds into window 1. It therefore cannot establish full A3 gateway, UDP-counter or router coverage. Persistent host observations independently supply all 182 gateway replies across the full A3 windows:

| A3 window | Replies / samples | Gateway RTT median / p95 / max, ms |
| --- | --- | --- |
| 1 | 61 / 61 | 2.02 / 21.60 / 213.00 |
| 2 | 60 / 60 | 0.93 / 3.42 / 6.10 |
| 3 | 61 / 61 | 1.80 / 6.73 / 30.50 |

The 213 ms gateway sample at 15:14:02.372 UTC occurred near a native 359.607 ms stream-ping / 0.6308% packet-loss sample at 15:14:03.068 UTC. Additional host gateway samples reached 37.5 ms one second after the 213 ms sample and 114 ms at 15:14:15.276. This is a local-delay confound worth retaining. ICMP and native stream timing measure different traffic, and sampled temporal proximity does not prove that the same delay caused individual video misses.

The bounded NetworkManager/iwd journal query for the surrounding minute returned no records. That gives no logged scan, disconnect or reconnect evidence; it does not prove a scan could not occur. Twelve persistent Wi-Fi metadata samples in that minute recorded two additional TX failures and 2,891 retries, with no beacon-loss or RX-drop increase. Those counters are local metadata, not decoded-frame loss.

The fresh Flint ring contained one SSH disconnect at 15:14:12 UTC from the ending observer connection. It was a Dropbear event, not a Wi-Fi station disconnect. No matching WAN/PPPoE, memory/watchdog or queue-fault event appeared. Persistent forwarded router syslog was not used as fresh evidence.

Activity during the final control has not been independently confirmed. The visible scene differed after ordinary playback resumed, adding workload uncertainty without establishing user intervention. The network observation changed no route, radio, QoS or power policy and sent no GUI input.

Private evidence is retained in `.runtime/round2/network-comparison.json`, `a3-host-supplement.json`, `a3-host-journal.json`, `a3-router-incident.json` and `a3-local-incident.json`. The host supplement fills RTT coverage only; it does not fill missing router or host UDP counters. Observer durations must cover retries as well as the initial planned trial.
