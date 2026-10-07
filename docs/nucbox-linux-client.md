# NucBox Linux client foundation

This fork targets a measured Linux client on an AMD Lucienne/Renoir iGPU. The
initial source review used upstream commit
`bee18c118dbc89f42319436dcdb172d5b9e15e0c`. The Qt/native stack is the foundation;
this document records reuse decisions and experiments, not measured improvements.

## Reuse

- The Rust account/session core for normal NVIDIA sign-in, catalog access,
  account entitlements, CloudMatch allocation, and lifecycle management.
- The native two-socket NVST transport, authenticated SRTP, GameStream RTP
  extension identities, Reed-Solomon FEC, private NACK-v2, and reference recovery.
- VA-API HEVC Main10 decode, DRM PRIME export, P010 DMA-BUF modifier/plane
  validation, and Vulkan color conversion.
- Qt's opaque native-frame integration and existing resource ownership, input
  release, and teardown contracts. Keep overlays on the same streaming surface.
- Existing diagnostics and per-stage timing owners before adding more telemetry.

Relevant owners are `native/opennow-core`, native transport `nvst.rs`, Linux
`video/ffmpeg.rs` and `frame_producer.rs`, and Qt
`streaming/rendering/NativeStreamRenderCallback.cpp`.

## Establish a baseline first

1. Build and run the current native client without transport policy changes.
2. Verify the actual selected decoder, negotiated bit depth/chroma, GPU frame
   export/import, color conversion, and displayed output on the target driver.
3. Measure receive, assembly, decode, publisher, Qt-submit, and swap times and
   queue residence. Qt swap callbacks do not measure physical scanout.
4. Compare repeat samples with identical service settings, rig, route, game,
   and display cadence. Report tails and freezes as well as median latency.

Hardware-profile enumeration establishes a supported configuration, not a
working zero-copy path or a performance result. A trace captured at the router
also excludes subsequent client Wi-Fi, decoding, and display delays.

## Recovery experiment

Current dedicated Mjolnir NACK-v2 tracking and packet dequeue use a 52 ms budget.
Retry admission is RTT-aware, but a path RTT above that budget can permit only
one attempt and may return retransmissions after retention expires. This is a
testable hypothesis, not proof of a current fault.

Instrument detection, admission, send, authenticated return, FEC repair,
expiry, and keyframe recovery. Compare current behavior, a bounded 68 ms policy,
and an RTT-informed candidate under deterministic loss/reordering and repeated
live trials. Tracking and dequeue must remain consistent. Preserve memory and
packet bounds; extra exceptional waiting must not enlarge the healthy-path queue.
Older video comparison notes describe a different NACK implementation, so use
the current `nvst.rs` and `docs/mjolnir-nack-v2.md` as the implementation evidence.

## Decode and presentation experiments

The compressed ingress and decoder-command queues can hold 15 and 16 frames at
60 FPS, while decoded output holds three. Capacities do not establish actual
latency. Measure age and residence before reducing them, and preserve reference
continuity and explicit keyframe recovery on overflow.

The embedded Qt path uses its native publisher/mailbox and scene-graph cadence.
It does not run the standalone `LinuxFramePacer` in `output.rs`. Changing the
standalone pacer therefore will not automatically change desktop presentation.

Measure mailbox replacement separately from network loss and decoded-queue skips.
Native GPU-frame sequence IDs are currently locally generated; preserve sender
frame identity if correlating assembly, decode, and presentation loss. Keep CPU
readback and software fallback explicit and measurable.

## Input and command experiment

The upstream transport command backlog is unbounded despite downstream SCTP
admission limits. Profile queue age under slow workers and input floods. Any
bounded/coalesced implementation must preserve key/button ordering, releases,
text reservations, shutdown priority, and recovery feedback. Avoid replaying
stale pointer motion after a hitch.

GPU FEC, predictive rendering, and more post-processing are later experiments
after profiling identifies a bottleneck. Network path RTT, transport recovery,
decode delay, and display delay require separate measurements.

Private packet captures, credentials, session identifiers, keys, and private
diagnostic exports are excluded from this public fork.

## Diagnostic implementation

This branch now counts decoded-video queue overflow in frame units, reports
replaced GPU-mailbox frames as a separate `video-mailbox` source, and retains
bounded numeric latency/bitrate/decode percentile fields in telemetry logs.
The publisher reports a replacement only after a successful `Replaced` result;
normal admission, consumed frames and publisher errors do not increment it.

These are measurement changes. They do not establish reduced latency or fewer
drops, and the HUD total still combines several local queue sources. Physical
scanout loss and network loss need separate measurements. No protocol deadline,
frame scheduler, credential handling, codec selection or FFI version changed.

Authenticated discovery and five repeated endpoint preflight rounds checked
24 NVIDIA regions on the NucBox. Germany was lowest at a 60 ms median, followed
by Netherlands South/North at 64/65 ms. These are TCP-connect round means, not
in-stream UDP or gameplay results. Private runtime data and account metadata
are excluded from the fork.
