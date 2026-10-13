# GFN Linux client research

Research objective: understand the native GFN protocol and build a measured Linux
client foundation for reducing local decode, queue, and presentation delay.

The custom client runs Ori on Germany at 1080p60 H.264 through the embedded
Vulkan decoder. Eight candidate configurations have been evaluated across the
experiment program. None passed all the frozen acceptance rules, so no
performance default changed.

The latest [Exa research](research/exa-next-hypotheses.md) led to a
[120 Hz display comparison](research/refresh-experiment.md) and a
[decoder-ready wakeup experiment](research/ready-wake-experiment.md). This round
completed eight full trials, each containing three one-minute windows. The
display candidate had a raw 56% reduction in late intervals but failed the
frozen gates; the wakeup candidate worsened the matched median and was reverted.

See [live playback experiments](research/live-hillclimb.md) for the results,
validity checks, and commands. A full measurement takes about three minutes and
twenty seconds after healthy playback begins. It measures fresh video presentation
callbacks, not physical display scanout or input-to-photon latency.

Saved-session discovery also checked all 24 regions with five TCP preflight rounds
each. The earlier [live evaluation](research/live-evaluation.md) records those
results and the unresolved HEVC startup failure. Credentials remain in the normal
OS vault. Account and game operations use the existing Qt client.

The user-authorized fork is [dsebban/OpenNOW](https://github.com/dsebban/OpenNOW).
The checkout is `vendor/OpenNOW`, on branch
[`nucbox-live-hillclimb`](https://github.com/dsebban/OpenNOW/tree/nucbox-live-hillclimb),
with `origin` pointing to the fork and `upstream` to OpenCloudGaming/OpenNOW.
Commit `44c3171` retains the additional acquisition/render diagnostics and reverts
the rejected wakeup implementation. The latter remains on `nucbox-readywake-trial`
for investigation, including an observed Qt/XCB crash with unproven causality.
The earlier
`nucbox-linux-client` branch contains the initial diagnostics.

## Repeat a live measurement

Start the baseline client with `tools/run-live-trial.sh baseline`, then use its
normal GUI to resume Ori and return to the chosen waterfall scene. Close another
OpenNOW instance before launching, so account refresh and stream ownership remain
with one client. Restarting interrupts playback.

```sh
python3 tools/live-trial.py measure baseline-example \
  --profile germany-h264-1080p60-ori-waterfall \
  --variant baseline
```

The command observes the running stream without account access. Keep one cloud
session and profile for each candidate and its baseline controls. The detailed
report explains comparison commands and the remaining measurement limits.

## Completed first milestones

- Installed Flint LuCI Statistics, ping/SQM/netlink collectors, and nlbwmon.
  Verified running services, numeric RRD samples, CAKE graph definitions, and
  device accounting. RRD data is in `/tmp/rrd`, accounting in `/tmp/nlbwmon`,
  RRD flash backups disabled. Existing SQM configuration was preserved.
- Queried sources through Exa CLI and checked out OpenNOW at
  `bee18c118dbc89f42319436dcdb172d5b9e15e0c`.
- Captured 40,000 packets through Flint `phy1-ap0` over 62.02 seconds, including
  session startup. Tcpdump reported zero kernel capture drops. The private
  capture is `captures/gfn-startup-20261007.pcap`, mode 0600, excluded from Git.
  Its SHA-256 is
  `cbea97d10d81aa1d7757008497d4f260632c78664f50d02578e34de8de88cd7d`.
- Verified transfer checksum before removing the router's RAM copy.
- Queried actual VA-API driver capabilities. AMD Lucienne/Renoir with Mesa
  25.1.1 advertises H.264 and HEVC Main/Main10 decode. No AV1 decode profile
  was advertised. This does not prove decode performance or GPU frame export.

## Observed protocol map

| Component | Evidence in this capture | OpenNOW implementation |
|---|---|---|
| Session control | TCP 322, TLS record headers | RTSPS session setup and negotiation |
| Video | Server UDP 5004 to client 49005 | Dedicated Mjolnir SRTP receiver |
| Video metadata | RTP extension profile `0x4753`, 16-byte GS headers | Frame and stream-packet identifiers, flags, FEC coordinates |
| Audio/control bundle | UDP 5004 to/from client 49006; STUN and DTLS headers | ICE/DTLS/SCTP/audio/control transport |
| Path timing | 733 matched client-initiated STUN exchanges | Median 56.53 ms, p95 57.14 ms at the router capture point |

The video direction contained 24,364 GS-header packets representing 2,743 unique
frame IDs. 5,628 packets had FEC-parity coordinates. There were no duplicate or
unseen RTP sequence slots in that captured video range. These are packet-header
observations, not authenticated SRTP decode or rendered-frame measurements.

STUN timings can include retransmission time. The analyzer separates initiators:
the 733 client-initiated exchanges had a maximum of 147.06 ms. Two server-initiated
exchanges measure the local client leg from this capture point, with one taking
about 1.9 seconds during startup. That is not a steady path RTT measurement.
Router timestamps precede Wi-Fi delivery to the NucBox and cannot expose its
later decode or display delays.

## Proposed native foundation

OpenNOW's checked-out source has a Qt 6 shell, Rust application core, and native
NVST streaming engine. Its Exa-indexed README described an older Electron layout;
the current Git source and AGENTS.md identify Qt/native as the supported product.
Use the checked-out source as evidence for architecture and current behavior.

The current client reuses the account/session and native transport implementation.
The live experiments narrow the next work on the NucBox's local pipeline:

1. Keep the demonstrated shared Vulkan H.264 path for playback comparisons.
   The GPU-only HEVC Main10 test passed, but live HEVC negotiation is unresolved.
2. Investigate decoded-frame readiness, publication, and Qt update timing.
   VA-API startup failed the embedded profile check in this experiment block;
   advertised driver support alone does not establish a usable fallback.
3. Bounded encoded queues and a latest-frame GPU mailbox, with correct reference
   recovery rather than indiscriminately dropping encoded reference frames.
4. Separate receive, FEC/reassembly, decode, input/audio, and presentation owners.
5. Record per-stage timestamps, queue depth, drops, and input-to-submit timing.
6. Compare identical negotiated settings, game, rig, route, and display cadence.
   Verified desktop mode is 3440x1440 at 60 Hz; the GFN stream is 1080p60.

A custom client can change local pipeline delay and recovery behavior. It cannot
be credited with reducing path RTT until a separate network comparison measures
that. GPU FEC, predictive rendering, upscaling, and different jitter budgets are
experiments after the basic decode/presentation path has been measured. The trace
currently supplies no evidence of video packet gaps at the router.

One transport hypothesis deserves a controlled test: the dedicated NACK-v2
documentation describes a 52 ms tracking window, while this trace's client-side
STUN RTT is about 56.5 ms. A requested retransmission may return after that
window. Examine the negotiated official 52/68 ms policies and test adaptive
recovery against synthetic loss and realistic RTT before changing a live client.
Extra waiting can improve recovery while increasing latency; it is not a free
performance gain, and this capture had no observed video sequence gaps.

## Reproduce the observations

```sh
python tools/analyze_pcap.py captures/gfn-startup-20261007.pcap
python tools/probe_vaapi.py
```

The analyzer emits headers and aggregate numeric data, never payload bytes,
credentials, session IDs, or SRTP keys. Raw traces must remain private.

## Sources

Other Exa-discovered clients include [NEXTCLIENT](https://github.com/clarkarch/nextclient),
a MIT-licensed Flutter/WebRTC rewrite advertising Linux VA-API paths, and
[gfn-electron](https://github.com/hmlendea/geforcenow-electron), whose README says
it is archived. The indexed `giooplion/GFNClient` repository is an older OpenNOW
fork. These are comparison leads; this work has validated the wire headers only
against the current OpenNOW native source and our official-client trace.

- [OpenNOW current source](https://github.com/OpenCloudGaming/OpenNOW/tree/bee18c118dbc89f42319436dcdb172d5b9e15e0c), MIT licensed.
- [Video protocol comparison](https://github.com/OpenCloudGaming/OpenNOW/blob/bee18c118dbc89f42319436dcdb172d5b9e15e0c/docs/streamer-comparison/video.md).
- [GS header parser](https://github.com/OpenCloudGaming/OpenNOW/blob/bee18c118dbc89f42319436dcdb172d5b9e15e0c/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs).
- [Mjolnir NACK v2](https://github.com/OpenCloudGaming/OpenNOW/blob/bee18c118dbc89f42319436dcdb172d5b9e15e0c/docs/mjolnir-nack-v2.md).
- [Linux hardware video backends](https://github.com/OpenCloudGaming/OpenNOW/blob/bee18c118dbc89f42319436dcdb172d5b9e15e0c/docs/linux-video-backends.md).

## Verified native foundation and next implementation phase

The complete native application and streaming library built successfully under
4 GiB RAM/two-core limits. Peak build RAM was 2.9 GiB. The real Qt application
was launched through CUA on X11 and its sign-in UI rendered. It remains available
through `tools/run-native.sh`. Use one authenticated application instance.

The account-free `shared_vulkan_hevc_p010_gpu_only` test passed on AMD Radeon
Graphics, RADV RENOIR, vendor 0x1002/device 0x164c. It decoded a generated
256x144 ten-bit HEVC frame through Vulkan Video, checked GPU-only P010 metadata,
and retained the GPU frame after decoder destruction. The filtered run had one
pass, zero failures, and 196 other tests filtered out. It does not validate
DMA-BUF import, Qt composition, 1080p60 pacing, or live GFN playback.

Rust 1.99, CMake 3.31.10, Qt 6.9.1, Vulkan Headers 1.4.313, Wayland Protocols
1.39, and NASM 2.16.03 are local to this workspace. Certificate validation stayed
enabled when selecting a working Qt HTTPS mirror. These dependencies resolved
the initial missing Qt Quick, Vulkan headers, Wayland XMLs, and assembler errors.

The research and design deliverables are described in
`research/prototype-plan.md`, with current evidence in `research/evidence.md`.
Matched live trials now cover local scheduling, render-loop selection, and receive
buffers. Their measurements identify substantial presentation variability despite
incoming and decoded rates near 60 FPS. Next work should explain that phase
variation and distinguish video-socket arrival bursts from receive-thread work
using bounded socket-age and processing-time diagnostics. A readiness-wait
publisher did not establish improvement. HEVC composition/import,
recovery, and input experiments remain separate work. No optimized-client latency
or frame-drop improvement has been accepted.

Focused source reviews are in `research/transport-review.md` and
`research/gpu-review.md`. Reuse the native transport and existing P010 GPU path.
Measure recovery deadlines, queue age, and mailbox replacement events before
changing buffer capacities or negotiated recovery policies. The embedded Qt path
does not run the standalone Linux frame pacer, so changing that pacer would not
automatically change desktop presentation.
