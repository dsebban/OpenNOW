# Research completion evidence

The objective is protocol analysis and an evidence-backed design/prototype plan.
The following evidence verifies those deliverables. It does not certify a faster
client, complete production readiness, or untested live playback behavior.

| Requirement | Authoritative artifact/evidence | Scope established |
|---|---|---|
| Exa CLI research | `tools/research.sh`; saved `research/clients.json`, `protocol.json`, `native.json`, `opennow-sources.json`, `alternative-clients.json` | Three five-result searches and selected primary-source retrievals; queries rerunnable |
| Existing docs/examples/clients | README sources; current native checkout; `transport-review.md`, `gpu-review.md` | Source-pinned architecture, GS parser/NACK examples, Linux backends, and alternative-client comparison |
| Protocol reverse-engineering observations | Private checksum-verified PCAP; `tools/analyze_pcap.py`; `research/capture-summary.json` | 40,000 packet headers, native video GS metadata, FEC coordinates, TLS/STUN/DTLS flows and separated STUN timings |
| Fork and code reuse | `dsebban/OpenNOW`, `nucbox-linux-client`, commit `6817b093b312769dac45b5c5eecf67afcfacac51` | Public source-only reuse/measurement plan; native code based on upstream `bee18c118dbc89f42319436dcdb172d5b9e15e0c` |
| Native Linux foundation feasibility | Successful `gfn-client-native-build-ffmpeg-20261007.service`; built application/core/library; `research/build-artifacts.json` | Full native build, runtime dependency resolution, real X11 Qt window launch through CUA |
| Hardware acceleration evidence | `tools/probe_vaapi.py`, `research/vaapi-capabilities.json`; `tools/check-hardware.sh`; `research/hardware-result.txt` | Actual driver profiles plus real GPU-only Vulkan Video HEVC Main10 decode/lifetime test on AMD RADV RENOIR |
| Lower latency/drop design | `research/prototype-plan.md`, focused source reviews, fork `docs/nucbox-linux-client.md` | Ownership/queues/pacing/recovery/input analysis, prioritized hypotheses, concrete metrics, scenarios, acceptance gates and matched comparisons |
| Monitoring and requested capture | Live collectd/nlbwmon status, numeric RRD samples and accounting; private PCAP | Installed Statistics/accounting, RAM history, original SQM configuration unchanged; capture checksum verified and router RAM copy removed |

## Verified results

- Capture SHA-256:
  `cbea97d10d81aa1d7757008497d4f260632c78664f50d02578e34de8de88cd7d`.
  File mode 0600, 37,726,206 bytes, excluded from Git.
- Native build result `success`, about 2.9 GiB peak RAM within a 4 GiB/two-core
  scope. Application binary is 22,341,768 bytes. SHA-256:
  `39b1a6e4a8b73e2a2b5a33b2261b7f47c90740459099794d120c5ad8f96d2969`.
- Native app was actually launched, its Qt sign-in screen rendered, and its
  live window remained present on reinspection. This is startup evidence.
- Hardware test result `1 passed; 0 failed; 0 ignored; 196 filtered out`.
  Adapter AMD Radeon Graphics, RADV RENOIR, vendor 0x1002/device 0x164c, API
  1.4.311. H.265 NV12/P010 were supported; H.265 NV24/P410 were unsupported.
  The single test took 0.36 seconds. That duration is not a playback benchmark.
- Packet analysis was rerun on the actual trace. There were no truncated
  packets or IPv4 fragments; the video range had no unseen sequence slots.
  This is not an assertion of client rendering or SRTP authentication.

## Explicit limits and next implementation work

Passive TLS/SRTP contents were not decrypted. Native control/authentication and
recovery formats are source/documentation evidence, with selected unencrypted
headers independently corroborated by capture. Not every service message has
been observed on the wire.

The hardware test decoded one 256x144 P010 frame on Vulkan Video and checked GPU
metadata/lifetime. It did not import DMA-BUF, render through Qt, inspect final
pixels, or measure 1080p60 pacing. Those and normal authorized live sign-in/playback
are explicit future implementation acceptance gates, not claimed test coverage.

Current source already supplies many desired native components. Prioritized
changes require measurements: queue residence and mailbox skips, sender-frame
correlation, recovery deadlines relative to actual RTT, and bounded input/command
backlogs. No source runtime performance policy was changed or advertised as
improved. GPU FEC and predictive rendering remain lower-priority experiments.

Raw traces, runtime state, credentials, and private diagnostic exports remain
outside Git and were not pushed to the public fork. The prototype plan preserves
the requested CPU/GPU offload and latency/drop objectives with distinct evidence
and validation requirements for each stage.
