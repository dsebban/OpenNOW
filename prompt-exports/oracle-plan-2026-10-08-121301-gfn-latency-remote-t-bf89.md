## Final Prompt
<taskname="GFN Latency Plan"/>

<task>
Produce a deep, prioritized engineering plan to reduce end-to-end latency (input-to-photon and network+decode+present) for GeForce NOW in OpenNOW Qt/native streamer on Linux (AMD Lucienne iGPU, Wayland, VA-API H.264/HEVC, Vulkan import, Qt scene graph). Do not implement. Establish required instrumentation first, then give an experiment order. For every proposed experiment state hypothesis, expected gain (with uncertainty), risk, owning file/crate, measurement, and acceptance criteria. Address decoder low-latency/no-reorder/B-frame behavior; bounded ingress/decode/output queues and age-based dropping; mailbox/publication wake; Qt render loop/vsync/tearing/Wayland presentation-time/frame pacing; Mjolnir NACK/FEC/jitter; input SCTP/coalescing/polling/backlog; audio; CloudMatch region and QoS/DSCP/socket options; bitrate/FPS/resolution; thread priorities/affinity. Treat absent measurements as gaps; baseline measurements are being collected by another agent and remote harness is outside this repository.
</task>

<architecture>
- Rust `opennow-streamer-transport` owns NVST UDP/SRTP video receive, FEC/reorder/assembly/NACK and input/SCTP; `frame_stage_timing.rs` carries current transport stages.
- `opennow-streamer-core` owns session lifecycle, event telemetry and transport-to-media wiring; FFI creates the embedded runtime used in Qt.
- `opennow-streamer-platform` owns bounded media queues, decoder command flow, embedded monitor/publisher, graphics mailbox, captured input queue and audio output.
- `opennow-streamer-platform-linux` owns Linux FFmpeg/VA-API decoder, decoded-frame queue/session, Vulkan frame producer/import and standalone presentation support.
- Qt's `StreamVideoItem` / `NativeStreamRenderCallback` consume native frames and expose app-side swap/presentation timing. Wayland callback timestamps do not establish physical scanout.
- `native/opennow-core/cloudmatch.rs` allocates CloudMatch sessions; `SettingsState.qml` and `ShellStore.qml` resolve/apply stream profile settings.
</architecture>

<selected_context>
docs/nucbox-linux-client.md: NucBox Linux architecture, current queue capacities, key limitations, measurement needs, and earlier endpoint preflight context.
docs/nucbox-live-hillclimb.md: full trial method, strict acceptance philosophy, experiment outcomes and limitations. Continuous update, 1 ms polling, basic render loop, VA-API preflight and rmem experiment did not pass acceptance; previous publication-ready wake was proposed but not tested successfully. Preserve these as failures/inconclusive evidence, not wins.
docs/mjolnir-nack-v2.md: private NACK-v2 route, retry budget, bounds and known diagnostic semantics.
docs/linux-video-backends.md and docs/linux-vulkan-acceptance.md: Linux backend/zero-copy acceptance constraints.
native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs (codemap): current receive pipeline, packet reorder/FEC/NACK limits and frame-stage hooks; inspect concrete symbols from this map.
native/opennow-streamer/crates/opennow-streamer-transport/src/frame_stage_timing.rs: measured transport stage summaries.
native/opennow-streamer/crates/opennow-streamer-transport/src/nvst_input.rs (codemap): input command transmission, SCTP and queues.
native/opennow-streamer/crates/opennow-streamer-transport/src/nvst_network.rs: network diagnostics surface.
native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs (codemap): telemetry assembly and session wiring.
native/opennow-streamer/crates/opennow-streamer-platform/src/media.rs (codemap): embedded monitor, media/decode command flow, queue bounds, timing and telemetry owners.
native/opennow-streamer/crates/opennow-streamer-platform/src/graphics.rs: GPU frame publication/mailbox and frame-available callback.
native/opennow-streamer/crates/opennow-streamer-platform/src/video_queue.rs and queue.rs: generic frame/queue mechanics.
native/opennow-streamer/crates/opennow-streamer-platform/src/embedded_input.rs: embedded capture queue contract.
native/opennow-streamer/crates/opennow-streamer-platform/src/audio_playout.rs: audio playout interface.
native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/ffmpeg.rs and vaapi.rs: Linux decoder setup and options.
native/opennow-streamer/crates/opennow-streamer-platform-linux/src/session.rs and queue.rs: Linux decoded-frame session and queue behavior.
native/opennow-streamer/crates/opennow-streamer-platform-linux/src/frame_producer.rs (codemap): DMA-BUF/Vulkan production path.
native/opennow-streamer/crates/opennow-streamer-platform-linux/src/presentation.rs (codemap): standalone presentation path; embedded Qt does not necessarily use it.
native/opennow-streamer/crates/opennow-streamer-ffi/src/lib.rs (codemap): embedded callbacks and FFI publication boundary.
native/opennow-core/src/cloudmatch.rs (codemap): session allocation / CloudMatch ownership; do not infer region selection policy beyond symbols.
native/opennow-streamer/README.md: crate responsibilities, embedded boundary and existing diagnostics.
opennow-qt/src/streaming/StreamVideoItem.cpp/.h and StreamVideoItemInput.cpp: Qt video consumer and input path.
opennow-qt/src/streaming/rendering/NativeStreamRenderCallback.cpp/.h: Qt scene graph callback/swap tracking.
opennow-qt/src/streaming/rendering/StreamPresentTimings.h and StreamSwapStallWatchdog.h: app-side timing definitions/watchdog.
opennow-qt/src/input/platform/WaylandPointerCapture.cpp/.h: Wayland pointer capture path.
opennow-qt/qml/state/settings/SettingsState.qml: stream settings policy and user-facing controls.
</selected_context>

<relationships>
- UDP receive → FEC/reorder/NACK → access-unit assembly → bounded decoder queue → Linux FFmpeg/VA-API decode → decoded-frame queue → embedded monitor → `GraphicsFramePublisher` mailbox → FFI frame-available callback → Qt `StreamVideoItem` update/render callback → swap; each edge needs sender-authored frame identity and timestamps.
- Input capture → `CapturedInputQueue` → core command routing → SCTP data channel → remote input; measure capture-to-send age and preserve key/button/release/text/shutdown ordering if coalescing/bounding is considered.
- Audio network/decode/playout is a parallel latency path and can affect A/V sync and underruns.
- CloudMatch/profile controls region and negotiated codec/resolution/FPS/bitrate; distinguish preflight TCP endpoint results from in-stream UDP RTT and gameplay latency.
</relationships>

<ambiguities>
- User-provided context says direct stream ping is about 57 ms and presentation late intervals are 1–12% with 0% packet loss and decoder p50 6–12 ms; baseline measurements are actively being collected elsewhere, so treat these only as context and request the fresh baseline report rather than treating them as final numbers.
- Prior live trials found baseline phase variation large enough to invalidate apparent improvements. Require same session/profile, repeated A/B/A or stronger randomized blocks, fixed acceptance gates, and report distributions/tails/freezes.
- No sender-authored frame ID correlation across all client stages is currently established. Define this instrumentation before tuning; Qt frameSwapped is not physical scanout and absolute input-to-photon requires an external optical/latency measurement method.
- Decide how to rank network RTT improvements versus local queue age: present independent stage-specific gains and avoid claiming end-to-end improvement without combined measurement.
- The previous wake experiment is mentioned as failed in the user's task brief; distinguish it from the documented proposed next investigation and ask for its exact trial evidence if the discrepancy matters.
</ambiguities>

<instructions>
Use the existing acceptance method and its limits from the selected docs. Prioritize instrumentation and one-variable experiments. Make a ranked plan with owners, expected gain range/qualitative confidence, risks, measurement, and explicit pass/fail thresholds for every idea. Separate plausible mechanisms from claims, include negative controls and rollback, and call out external harness requirements. Do not invent baseline measurements or assert unsupported performance gains. Respect AGENTS.md: smallest bounded changes, experiment-backed decisions, preserve unrelated work, and no implementation in this handoff task.
</instructions>

## Selection
- Files: 41 total (27 full, 14 codemap)
- Total tokens: 156954 (Auto view)
- Token breakdown: full 137508, codemap 19446
- Token accounting: incomplete from active_tab_published; refresh pending; incomplete: codemap_presentation

### Files
### Selected Files
├── docs/
│   ├── linux-video-backends.md — 1,077 tokens (full)
│   ├── linux-vulkan-acceptance.md — 1,643 tokens (full)
│   ├── mjolnir-nack-v2.md — 1,389 tokens (full)
│   ├── nucbox-linux-client.md — 1,446 tokens (full)
│   └── nucbox-live-hillclimb.md — 5,005 tokens (full)
├── native/
│   └── opennow-streamer/
│       ├── crates/
│       │   ├── opennow-streamer-platform/
│       │   │   └── src/
│       │   │       ├── audio_playout.rs — 1,456 tokens (full)
│       │   │       ├── embedded_input.rs — 4,238 tokens (full)
│       │   │       ├── graphics.rs — 8,860 tokens (full)
│       │   │       ├── queue.rs — 743 tokens (full)
│       │   │       └── video_queue.rs — 3,464 tokens (full)
│       │   ├── opennow-streamer-platform-linux/
│       │   │   └── src/
│       │   │       ├── video/
│       │   │       │   ├── ffmpeg.rs — 22,277 tokens (full)
│       │   │       │   └── vaapi.rs — 5,004 tokens (full)
│       │   │       ├── queue.rs — 2,125 tokens (full)
│       │   │       └── session.rs — 22,076 tokens (full)
│       │   └── opennow-streamer-transport/
│       │       └── src/
│       │           ├── frame_stage_timing.rs — 4,883 tokens (full)
│       │           └── nvst_network.rs — 852 tokens (full)
│       └── README.md — 5,005 tokens (full)
└── opennow-qt/
    ├── qml/
    │   └── state/
    │       └── settings/
    │           └── SettingsState.qml — 10,242 tokens (full)
    └── src/
        ├── input/
        │   └── platform/
        │       ├── WaylandPointerCapture.cpp — 2,827 tokens (full)
        │       └── WaylandPointerCapture.h — 213 tokens (full)
        └── streaming/
            ├── rendering/
            │   ├── NativeStreamRenderCallback.cpp — 8,475 tokens (full)
            │   ├── NativeStreamRenderCallback.h — 57 tokens (full)
            │   ├── StreamPresentTimings.h — 3,105 tokens (full)
            │   └── StreamSwapStallWatchdog.h — 1,219 tokens (full)
            ├── StreamVideoItem.cpp — 5,307 tokens (full)
            ├── StreamVideoItem.h — 2,810 tokens (full)
            └── StreamVideoItemInput.cpp — 11,710 tokens (full)

### Codemaps
native/
├── opennow-core/
│   └── src/
│       ├── push/
│       │   └── transport.rs — 375 tokens (auto)
│       └── updater.rs — 1,157 tokens (auto)
└── opennow-streamer/
    └── crates/
        ├── opennow-streamer-core/
        │   └── src/
        │       └── lib.rs — 2,995 tokens (auto)
        ├── opennow-streamer-platform/
        │   └── src/
        │       ├── media.rs — 4,095 tokens (auto)
        │       └── output.rs — 3,627 tokens (auto)
        ├── opennow-streamer-platform-linux/
        │   └── src/
        │       ├── video/
        │       │   └── vaapi_probe.rs — 468 tokens (auto)
        │       ├── audio.rs — 1,076 tokens (auto)
        │       ├── error.rs — 93 tokens (auto)
        │       ├── format.rs — 958 tokens (auto)
        │       └── timing.rs — 629 tokens (auto)
        ├── opennow-streamer-platform-macos/
        │   └── src/
        │       └── queue.rs — 232 tokens (auto)
        └── opennow-streamer-platform-windows/
            └── src/
                ├── windows/
                │   └── decoder.rs — 2,028 tokens (auto)
                ├── format.rs — 568 tokens (auto)
                └── lib.rs — 1,145 tokens (auto)


---

## Generated Plan
- Oracle group status: failed
- Oracle group: `D41EAD64-DCE0-4340-B66C-C2CA65A13747`

**Reconciling these Oracle lanes**
2 independent answers to the same request follow. Lane order is not a ranking; the first lane supplies the top-level continuation handle, and a successful follow-up through any lane's chat ID re-runs every lane.
- Read every lane through the end-of-group marker (`End of Oracle group: 2 lanes above.`). If the marker or a lane is missing, page the export or try the read-only `oracle_chat_log` with that lane's chat ID. Logs may be scoped or clipped; report any remaining gap. Do not start a follow-up just to retrieve prior text.
- Reconcile by evidence, not lane order, answer length, or model identity: check material single-lane and conflicting claims against the code, and report unresolved disagreements. A failed or partial lane is incomplete evidence.
- Before synthesizing, inventory every material claim from every lane, including single-lane claims. Merge only exact duplicates, retaining all source lanes. Begin your answer with `**Oracle reconciliation**`, state how many lanes completed and name any that did not. For every inventory item, give its source lanes, checked evidence, and exactly one disposition: `accepted`, `rejected`, or `unresolved`. Never silently omit an item.

Lanes (2):
- Oracle — `claude-opus-5-5-xhigh` — Failed — chat ID `gfn-latency-remote-test--652397`
- Oracle 2 — `gpt-6-astra-pro` — Completed — chat ID `gfn-latency-remote-test--E461B9`

### Oracle
- Status: Failed
- Provider: `Custom`
- Model: `claude-opus-5-5-xhigh`
- Effective effort: Provider default / not specified
- Chat: `gfn-latency-remote-test--652397`

Error [context_builder_inactivity_timeout]: Context Builder Oracle exceeded its 600s inactivity budget during streaming.

### Oracle 2
- Status: Completed
- Provider: `Custom`
- Model: `gpt-6-astra-pro`
- Effective effort: Provider default / not specified
- Chat: `gfn-latency-remote-test--E461B9`



## 1. Summary

Use a **targeted instrumentation extension followed by bounded, independently reversible experiments**. Preserve the existing transport, decoder, GPU ownership, and Qt integration. First establish frame identity across receive, decode, publication, and drawing; distinguish actual drawing from update requests and window swaps; and measure input queue age through transport admission. Then investigate publication wakeups, Qt scheduling, queue residence, and input backlog before changing decoder or recovery policy. Evaluate region selection separately because it changes the remote session and network path. Report client-stage improvements, smoothness improvements, and optical input-to-photon improvements as separate results. The supplied experiment report supports investigating publication and presentation timing, but records no accepted production improvement from continuous update, faster polling, the basic Qt render loop, VA-API preflight, or enlarged receive buffers. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/nucbox-live-hillclimb.md:7-32`, `/Users/danielsivan/dev/OpenNOW/docs/nucbox-live-hillclimb.md:156-178`]**

## 2. Current-state analysis

### Evidence boundary

This plan uses the supplied bodies and code maps. Several files named in `selected_context` do **not** actually have bodies or code maps in the pasted evidence: `nvst.rs`, `nvst_input.rs`, `frame_producer.rs`, `presentation.rs`, the FFI implementation, and `cloudmatch.rs`. `NativeStreamRuntime` and `ShellStore.qml` are also incomplete or absent.

Their architectural roles are supplied, but their complete signatures, internal queue declarations, and call sites are not. The implementer must resolve those mechanical integration details from the checkout before the affected phase. I do not assign fabricated line references to them.

Two external inputs are required **before scoring changes**, without blocking instrumentation implementation:

- The fresh baseline report, including effective backend, copy path, negotiated profile, timing definitions, binaries, environment, and cloud-session comparison block.
- The exact alleged failed publication-wake trial: patch, effective configuration, build fingerprints, and result. The supplied report describes readiness wake as proposed and unmeasured; it does not establish a successful or failed implemented wake candidate. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/nucbox-live-hillclimb.md:156-164`]**

### O1 — Transport admission timing is already partially instrumented

`FrameStageTimingsAccumulator` matches assembly and admission using a sender frame index, retains at most 512 pending stamps, and stores bounded stage samples. Its measured endpoints are assembly, admission, and ACK queueing. An ACK queued for transport is not evidence of decoder completion, network transmission, or presentation.

The internal vectors named `*_us` actually receive nanoseconds through `Duration::as_nanos()` and are converted to milliseconds when summarized. Preserve the numerical behavior; an internal rename must not introduce a unit conversion.

**[Direct observation: `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/frame_stage_timing.rs:4-34`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/frame_stage_timing.rs:95-174`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/frame_stage_timing.rs:201-224`]**

Core integration points are `consume_encoded_media`, `forward_nvst_media_feedback`, `frame_stage_timings_event`, `decode_timings_event`, and `flush_nvst_telemetry`. Their locations are established by the supplied code map; their complete implementations were not supplied.

### O2 — Compressed queues already enforce reference continuity

`VideoQueue` starts by waiting for a keyframe. Discontinuity and overflow can invalidate queued descendants, increment a generation, and coalesce keyframe requests. `pop()` deliberately converts a local reset into `frame.contiguous = false`. Its atomic downstream-admission guard, `submit_if_current`, is currently compiled for Windows/tests.

Linux adds another queue of `VideoCommand::{Decode, Reconfigure}`. Admission is serialized by `video_submit`; overflow rejects dependent frames and retains control commands when admitting a recovery keyframe. Defaults are 16 encoded commands and three decoded frames.

These are two reference-sensitive admission boundaries. Replacing either with generic drop-oldest behavior would violate existing recovery semantics.

**[Direct observation: `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/video_queue.rs:59-167`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/session.rs:76-125`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/session.rs:468-544`]**

The documented 15-frame ingress capacity at 60 FPS and these downstream capacities establish storage limits, not measured latency. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/nucbox-linux-client.md:53-67`]**

### O3 — “Decode time” includes different operations on different paths

The native H.264 VA-API decoder accepts SDR NV12, synchronizes a ready VA surface, exports it, and returns DMA-BUF backing. The FFmpeg decoder configures one thread for hardware modes; software mode already requests bounded slice threading and `AV_CODEC_FLAG_LOW_DELAY`.

`run_video_worker` measures the complete decoder call. FFmpeg’s call can include receive, conversion, GPU snapshot work, or CPU download. Therefore that duration is not automatically isolated hardware decode execution time.

**[Direct observation: `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/vaapi.rs:244-334`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/ffmpeg.rs:132-188`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/ffmpeg.rs:334-421`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/session.rs:699-724`]**

FFmpeg VA-API permits CPU transfer for eligible SDR output when export fails. The native H.264 VA-API implementation instead propagates its export error. Instrument the implementation and transfer path, not just the label “VAAPI.”

**[Direct observation: `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/ffmpeg.rs:441-550`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/vaapi.rs:281-334`]**

### O4 — Publication and rendering have distinct owners

The documented embedded monitor polls decoded output, publishes the latest frame, and sleeps for a default two milliseconds. `GraphicsFramePublisher::publish` replaces one pending frame under the scene mutex and invokes the availability callback after releasing that mutex.

`RenderThreadGraphics` binds initialization, acquisition, recording, and retirement to the render thread. Leases and tokens are epoch checked. Recording consumes a token’s recording opportunity even when the backend subsequently returns `NotReady`.

**[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/nucbox-live-hillclimb.md:158-162`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/graphics.rs:379-417`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/graphics.rs:452-547`]**

The Qt route borrows QRhi resources and records native work into the host command stream. It does not use the standalone `LinuxFramePacer` as its presentation scheduler. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/README.md:39-49`, `/Users/danielsivan/dev/OpenNOW/docs/nucbox-linux-client.md:60-62`]**

### O5 — Presentation measurement needs correction and a cheaper observation path

`NativeStreamRenderCallback::recordFrame` currently marks a fresh submission whenever `m_outputDirty` is true, even if `m_textures.render()` returned false. A later swap can therefore consume a submission marker that was not accompanied by a video draw.

Separately, `observeSwapProgress()` calls the full timing snapshot during rendering. That snapshot copies histograms and allocates/sorts the percentile sample window while holding its mutex.

**[Direct observation: `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/streaming/rendering/NativeStreamRenderCallback.cpp:496-508`, `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/streaming/rendering/NativeStreamRenderCallback.cpp:550-571`, `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/streaming/rendering/StreamPresentTimings.h:147-190`]**

These justify a measurement correction and an overhead experiment. They do **not** establish that either caused the historical hitches. The watchdog also executes from rendering; it cannot independently observe a render thread that never returns to this code. **[Inference from O5 and the cited call sites.]**

### O6 — Input ownership must survive any queue change

`EmbeddedInputCapture` serializes capture transitions with controller submission, preserves neutral controller state, and cancels text on capture loss. XInput capture requires an explicit X11 window.

Wayland motion arrives through its protocol callback. The 100 ms `refresh` timer maintains capture state; it is not the motion sampling interval.

**[Direct observation: `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/embedded_input.rs:35-141`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/embedded_input.rs:186-194`, `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/input/platform/WaylandPointerCapture.cpp:106-114`, `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/input/platform/WaylandPointerCapture.cpp:145-171`]**

The supplied foundation document identifies an unbounded upstream transport command backlog despite downstream SCTP limits. Its exact queue implementation remains a source-verification prerequisite. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/nucbox-linux-client.md:69-75`]**

### O7 — Recovery, audio, and session selection are separate latency domains

Mjolnir uses private NACK-v2, a 52 ms tracking budget, RTT-aware retry intervals, and bounded batches. The partial control channel is documented as unordered with a 300 ms lifetime. Its local send log records admission, not server acknowledgement.

**[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/mjolnir-nack-v2.md:25-61`]**

`AudioPlayoutBuffer` implements bounded stereo SDL playout and temporary catch-up. Linux also has its own `LinuxSession` audio worker and ALSA/PipeWire sinks. Do not assume tuning the SDL buffer changes the active embedded Linux audio path.

**[Direct observation: `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/audio_playout.rs:1-105`; code-map observation: `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/media.rs:3120-3120`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/audio.rs:373-441`]**

Qt Auto’s documented policy excludes automatic software decoding. Preserve that application policy even though lower-level decoder preferences expose broader fallback orders. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/linux-video-backends.md:7-18`]**

## 3. Design

### 3.1 Establish a common trace identity and timing contract

#### Components and ownership

Add a small shared instrumentation module, `client_timing.rs`, to `opennow-streamer-protocol`. It contains plain data types and a nonblocking recording interface; it owns no codec, graphics device, socket, or Qt object.

Add `ClientTraceRecorder` in `opennow-streamer-core`. `Engine` owns its lifecycle; lower layers receive an optional shared recording handle. It owns:

- A runtime-local monotonic clock origin.
- A bounded queue of 8,192 fixed-size numeric events.
- Counters for rejected events and incomplete correlations.
- The background diagnostic writer and its stop state.
- A per-run identifier and the effective experiment configuration.

Event payloads must contain no media bytes, input text, credentials, endpoint tokens, or GPU pointers. Queue saturation drops diagnostic events and increments a counter; it never blocks media admission. A scored trace with dropped events is incomplete.

Use these identity fields:

| Field | Meaning |
|---|---|
| `runtime_id: u64` | Unique within the diagnostic run; never a pointer. |
| `session_generation: u64` | Existing engine lifecycle generation. |
| `stream_epoch: u64` | Changes on transport source/reset boundaries that invalidate identity. |
| `local_frame_id: u64` | Monotonic local identifier for an assembled access unit. |
| `sender_frame_index: Option<u32>` | Original sender frame index, unchanged. |
| `sender_ssrc: Option<u32>` | Original authenticated source identity when available. |
| `rtp_timestamp: u64`, `clock_rate_hz: u32` | Original media timestamp and its clock rate. |

Decoder epoch, scene-graph epoch, local GPU sequence, and decoded-output ordinal are **additional correlation fields**, not replacements for sender identity.

Add optional `client_timing` metadata to the platform `EncodedFrame` and Linux encoded/decoded frame structures. Existing constructor signatures remain unchanged and initialize it to `None`. Internal transformations explicitly preserve it.

Decoded metadata also records `DecodedTimestampOrigin` with exactly three cases:

- `Decoder`: obtained from the decoder’s output timestamp.
- `InputFallback`: substituted from the most recent input timestamp.
- `Unknown`: provenance is unavailable.

This distinction matters because FFmpeg currently supplies both packet PTS and DTS from the input timestamp, and several output paths substitute `last_timestamp_us` when output PTS is missing. A substituted timestamp must not produce a supposedly exact frame match. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/ffmpeg.rs:613-623`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/ffmpeg.rs:1179-1192`]**

#### Correlation algorithm

1. Assign identity after authenticated access-unit assembly. Retain first and last authenticated packet arrival times in the assembler’s existing bounded frame state.
2. Carry identity through core conversion, `MediaSink`, `VideoQueue`, Linux submission, and `VideoCommand::Decode`.
3. Extend `DecodeTimingProbe` with a bounded identity association alongside its existing timing state. Match output using decoder epoch and actual output timestamp.
4. Accept only a unique match. Missing PTS, ambiguous duplicate timestamps, an expired association, or an old epoch produces an unmatched output counter.
5. Preserve the match through `LinuxGpuFrameProducer`, publication, acquisition, recording, and Qt drawing.
6. On decoder replacement, invalidate pending decoder associations. On session replacement, retire all session associations. A scene-graph recreation changes graphics correlation without inventing a new sender frame.
7. Count each discard at its actual stage. Never sum stage discards into “unique lost frames.”

Keep at most 512 pending frame associations per relevant owner. Bounded linear matching is sufficient at this size and follows the existing timing approach; a new general-purpose tracing database is unnecessary. **[Design grounded in O1–O4.]**

#### Required stage records

Use explicit stage names:

| Domain | Events |
|---|---|
| Transport | `receive_first`, `receive_last`, `assembled`, `transport_delivery` |
| Compressed admission | `ingress_enqueue`, `ingress_dequeue`, `decode_enqueue`, `decode_dequeue` |
| Decoder | `decode_call_begin`, `decode_call_end`, `decoded_output`, `gpu_ready` |
| Publication | `published`, `mailbox_replaced`, `frame_notification` |
| Qt | `notification_drain`, `update_requested`, `acquired`, `record_begin`, `record_end`, `draw_recorded`, `window_swapped` |
| Optional compositor evidence | `surface_commit`, `compositor_presented`, `compositor_discarded` |
| Discards | `frame_discarded`, with a bounded stage/reason enum |

`gpu_ready` is emitted only where completion is actually known. A CPU submission return does not manufacture that event.

Record queue depth and residence at enqueue/dequeue, and separate decoder call duration from output residence. Compute complete client-path distributions from matched frames. **Do not add independently calculated p95 values.**

#### Clock boundaries and physical latency

Use the recorder’s Rust monotonic clock for native events. Retain Qt’s existing steady-clock timestamps with an explicit clock-domain identifier.

Add an optional trace-clock FFI query. At startup and every ten seconds, collect 16 Qt-before/native/Qt-after triples. Use the narrowest bracket for mapping; retain half its width as uncertainty. Cross-domain spans are ineligible if uncertainty exceeds 100 μs or calibration shows a discontinuity. These are new measurement thresholds, frozen before trials.

Expose three distinctly named metrics:

- `receiveFirstToQtSwapMs`: client processing and application presentation.
- `receiveFirstToCompositorPresentMs`: available only with correctly associated compositor evidence.
- Optical `inputToPhotonMs`: measured externally.

Neither of the first two includes an established remote encode-to-client one-way network duration. RTP timestamps and network RTT do not supply that missing measurement.

For optical tests, require at least 200 input/visible-response observations per variant, distributed across three comparison blocks. Use a physical input trigger with an observable timing reference and a repeatable remote response. Instrument resolution must be at most 1 ms for a claimed 2 ms improvement; otherwise raise the minimum detectable improvement to twice the instrument resolution.

### 3.2 Correct Qt measurement semantics before collecting the new baseline

Modify `recordFrame` so that:

- A fresh submission is recorded only after `drawIssued == true`.
- Failed drawing preserves pending dirty state for a later attempt.
- A replacement before successful drawing is recorded as superseded work.
- Generated frames, repeated source draws, and first draws of a new decoded output have separate classifications.
- Source identity comes from the successfully recorded output’s trace metadata.
- Missing PTS is represented explicitly; a valid zero timestamp is not automatically “missing.”

Keep existing histogram resolution and the strict `interval > 25 ms` test.

Add `StreamPresentTimings::ProgressSnapshot` containing only the fields used by the watchdog: timing epoch, gate state/epoch, pending-submit state, last-source-swap state/time, and source-swap count. Add synchronous `progressSnapshot() const`.

Retain full `snapshot()` for periodic statistics. Copy its bounded sample storage under the mutex, then calculate percentiles after releasing the mutex. Evaluate switching the render watchdog to the lightweight accessor as a separate performance experiment.

Introduce `presentationMeasurementVersion: 3` and `traceSchemaVersion: 1` in diagnostics. Existing JSON fields remain available, but traces collected before the fresh-draw correction must not be pooled with corrected traces. Historical trial conclusions remain unchanged. **[Design grounded in O5.]**

#### FFI and Qt interfaces

Preserve existing graphics context, record-command, frame-info, and recorded-frame layouts. Add an optional versioned trace extension providing:

- A runtime trace-clock query.
- A read-only trace-identity query on an owned frame token.
- Submission of a fixed-size trace event from Qt.

`NativeStreamRuntime` resolves these optional functions once. An older library produces “trace unavailable,” while normal rendering continues. No per-frame JSON request is added.

Add a default `GraphicsFrame::client_timing()` accessor returning `None`; the Linux implementation exposes its carried metadata. `GraphicsFrameToken` forwards the accessor without recording or extending GPU use.

Frame queries must occur while the token is owned. Diagnostic records retain numeric copies only.

### 3.3 Define readiness and queue-policy changes precisely

#### Readiness receiver

Add `DecodedFrameReceiver`, a cloneable receive handle holding only the decoded queue’s `Arc`. Obtain it from `LinuxSession` before entering a blocking wait.

Add:

- `BoundedQueue::wait_pop_latest(timeout) → QueuePop<(T, skipped_count)>`
- `LinuxSession::decoded_frame_receiver() → DecodedFrameReceiver`

`wait_pop_latest` must check nonempty/closed predicates under the existing queue mutex, wait on its condition variable, take the newest item, and discard older items atomically. Notifications are hints; persistent queue state is authoritative.

The ready-mode monitor:

1. Processes pending control/backend events.
2. Waits for decoded output with a **2 ms maximum timeout**.
3. Takes the latest output and records discarded older outputs.
4. Checks session and graphics availability.
5. Publishes outside the decoded-queue mutex.
6. Immediately repeats; it does not sleep after successful publication.

The timeout preserves existing maintenance responsiveness without requiring backend events to masquerade as frames. Never wait while holding `SharedPipeline`’s session lock or a lock needed by submission/stop.

A closed receiver ends that monitor instance. A later session creates a new receiver. This reuses the existing mutex/condition-variable design instead of adding a frame-ready event queue. **[Design grounded in O4 and `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/queue.rs:123-170`]**

#### Age policies

Define the negotiated frame period as `T = ceil(1 second / fps)`.

Test these independently:

| Boundary | Candidate limit | Required action |
|---|---:|---|
| Decoded output | `2T` since decoded output became available | Discard stale decoded output; keep the last displayed image until a fresh frame arrives. No keyframe request. |
| Compressed ingress | `2T` of local queue residence | Use reference invalidation and coalesced keyframe recovery. |
| Linux decode commands | `2T` of local queue residence | Remove stale decode work, preserve `Reconfigure`, and recover from a keyframe. |

Compressed expiry algorithm:

1. Examine age before decoder admission.
2. If expired, retain a suffix only when it begins with an unexpired complete keyframe and all retained descendants are contiguous.
3. Otherwise discard pending decode work, increment the existing generation, and wait for a keyframe.
4. Request one keyframe per waiting episode.
5. Preserve control commands in FIFO order.
6. Reject work popped under an obsolete generation.
7. Reopen/reset through the existing decoder recovery path; never decode a dependent frame after deleting its reference.

Extend `submit_if_current` to Linux for the ingress-to-decoder admission boundary. Its closure may enqueue compressed work only. Keep the lock order **ingress admission → Linux submission → Linux command queue**; issue feedback outside those locks.

Pause/reconfigure and worker output admission must share the generation discipline. Already submitted GPU work may complete and retains its resources; generation changes prevent subsequent admission, not retroactive cancellation.

Keep capacities unchanged during age experiments. Later capacity trials, if warranted, test one boundary at a time using `ceil(fps × 0.05)` entries, clamped to 2–8. At 60 FPS that is three entries. Capacity and age limits are separate variables. **[Design grounded in O2.]**

### 3.4 Instrument and bound input without losing transitions

Preserve the existing `CapturedInputSample.captured_at`. Add optional input trace metadata carrying session/capture generation and a local input ID. Qt-originated samples additionally carry callback time and clock provenance through additive metadata-aware FFI entry points; existing entry points use the same admission implementation without metadata.

Measure separately:

- Native capture to core dequeue.
- Core dequeue to transport command dequeue.
- Transport dequeue to SCTP admission.
- SCTP admission to observable UDP emission, only where transmission can be associated.
- Qt event/callback delay where source-clock mapping is established.

SCTP admission is not remote receipt or remote game execution.

For the transport backlog experiment, use one bounded input queue with **256 ordinary entries**, plus bounded neutralization storage covering supported keys, five mouse buttons, and four controllers. Shutdown remains independently signalable and must wake the transport owner.

First test bounded admission without coalescing. Then test adjacent-motion coalescing:

- Merge only adjacent relative-motion events from the same capture generation.
- Accumulate in a wider integer and split into valid wire-sized deltas.
- Replace adjacent absolute positions only when coordinate dimensions and capture generation match.
- Never cross a key, button, wheel, text, or control boundary.
- Preserve the absolute position associated with a click.
- Do not coalesce gamepad button transitions or text.
- Expire relative motion older than **20 ms** before transport admission. Do not age-drop keys, buttons, wheel events, or required release/neutral events.

If ordinary capacity is exhausted, discard supersedable motion first. If a lossless transition still cannot be admitted, terminate the **input capture epoch**: cancel pending text, neutralize held state through reserved storage, reject further pressed actions, and report capture overload through the existing input-reset/error route. Video and audio continue; stale input is not replayed after reacquisition.

Do not change shared SCTP reliability or its lifetime as part of this experiment. That would also affect other traffic on the channel. **[Design grounded in O6 and O7.]**

### 3.5 Freeze the acceptance protocol

The existing 60 FPS smoothness campaign retains its rules:

- 20-second warm-up and three consecutive 60-second windows.
- Same cloud session/profile and repeated baseline control.
- Fresh-source intervals strictly greater than 25 ms.
- At least 57 presented FPS in every candidate window.
- At least 57 assembled FPS for workload qualification.
- At least 20% primary-metric reduction, with absolute reduction exceeding the baseline window spread.
- Existing ping, decoder residence, presentation delay, packet loss, throughput, interval-tail, overflow, lag-growth, epoch, and recovery guards.

These are documented requirements, not reconstructed claims about the unseen comparator implementation. Run that frozen comparator unchanged. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/nucbox-live-hillclimb.md:71-97`]**

Add the following separately registered gates:

| Gate | Definition |
|---|---|
| **V — Valid measurement** | Correct effective configuration; no trace overflow; no unexplained correlation ambiguity; no clock failure, unexpected epoch change, recovery, or missing required measurement. |
| **S — Smoothness improvement** | Pass the existing frozen 60 FPS comparator. |
| **L — Client-stage improvement** | Target-stage p95 decreases by at least `max(0.25 ms, 20% of control p95)` and exceeds control variation; a block-level confidence interval excludes zero. Existing regression guards also pass. |
| **P — Input-to-photon improvement** | Optical median decreases by at least `max(2 ms, twice instrument resolution, control-median spread)`; optical p95 does not regress beyond control variation. |
| **C — Cost guard** | Process CPU use increases by no more than 0.02 CPU-core equivalents over matched controls, unless a different budget was explicitly frozen for a profile experiment. No new thermal throttling or resource growth. |

Use A/B/A initially. A promising candidate requires three complete same-session blocks with randomized block ordering. Analyze at block level rather than treating individual frames as independent trials.

A result passing L alone is a measured client-stage improvement. It is not a claim of S or P. A zero-late baseline cannot establish an S improvement.

Measure CPU from process CPU-time deltas over independent monotonic intervals. Do not reuse the old telemetry-reader timing approach that the report identifies as distorted by queued deliveries. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/nucbox-live-hillclimb.md:166-178`]**

### 3.6 Ranked experiments

Expected gains below are **hypotheses or conditional bounds**, never measured promises. Each native candidate is opt-in, reads its configuration once at owner creation, records the effective value, and defaults to existing behavior. Change one variable per trial.

#### E0 — Instrumentation and negative controls — required first

- **Hypothesis:** Accurate frame association and stage timing can identify where delay accumulates.
- **Expected gain:** None; this establishes evidence.
- **Owners:** Shared timing types, core recorder, transport timing hooks, Linux timing/frame metadata, Qt timing and FFI trace extension.
- **Measurements:** Trace-enabled versus disabled overhead; no-op variant comparisons; exact count reconciliation; synthetic timestamp collisions, resets, and dropped records.
- **Accept:** V; recorder admission p99 below 50 μs; CPU overhead within C; no media/presentation regression outside control variation.
- **Risk/rollback:** Observer overhead and incorrect joins. Disable tracing; preserve playback. Do not tune from incomplete traces.

#### E1 — Remove full timing snapshots from the render watchdog

- **Hypothesis:** Full percentile snapshots contribute avoidable render-thread work.
- **Expected gain:** The measured cost of that work; potentially negligible. Any additional frame-period benefit requires evidence that it crossed a scheduling deadline.
- **Owners:** `StreamPresentTimings.h`, `NativeStreamRenderCallback.cpp`.
- **Measurement:** Watchdog observation CPU duration, mutex wait, render preparation tails, and source intervals.
- **Accept:** Identical watchdog outcomes for identical observations; at least 20% reduction in observation p95 cost; C and no presentation regression. Claim latency only if L or P also passes.
- **Risk/rollback:** Divergent snapshot semantics. Select the original accessor.
- **Basis:** Inference from O5.

#### E2 — Wake publication on decoded readiness

- **Hypothesis:** Replacing polling sleep with a predicate-based wait reduces output-to-publication delay and phase sensitivity.
- **Expected gain:** Under timely scheduling, approximately 0–2 ms at this boundary. A one-frame presentation-phase benefit is possible but unproven.
- **Owners:** Linux `queue.rs`, `session.rs`; platform `media.rs`.
- **Measurement:** Decoded-output-to-publish p50/p95/p99, wake-to-run delay, mailbox replacements, Qt acquire delay, CPU.
- **Accept:** V + L + C; unchanged three-frame decoded capacity; no lost wake in race tests; S required for a smoothness claim.
- **Risk/rollback:** Waiting under a session lock, delayed control handling, or lost close notification. Return to the existing two-millisecond polling mode.
- **Basis:** Inference from O4.

#### E3 — Notification rearm and continuous redraw

Run these as separate candidates.

**Notification rearm:** Inspect `NativeStreamRuntime::enqueueFrameAvailable` only after traces show a missed or unnecessarily delayed GUI notification. Use a generation-scoped sequence counter plus one pending-drain flag. After emission, clear the flag and recheck the sequence; racing publication must either schedule a drain or be covered by that recheck. Old-generation drains must not clear a new generation’s pending state.

**Continuous redraw:** Retest the existing option only after baseline phase variation is explained or controlled.

- **Expected gain:** Zero when notification/update scheduling is already timely; otherwise the measured delay, possibly approximately one display period.
- **Owners:** `NativeStreamRuntime`; `StreamVideoItem::requestFrame` and `connectFrameSwaps`.
- **Measurement:** Callback-to-GUI-drain, drain-to-update, update-to-acquire, source draws versus old-frame window redraws.
- **Accept:** V + L + C; S for smoothness promotion; no phantom fresh draws. Notification races must pass deterministic generation/teardown tests.
- **Risk/rollback:** Excess redraws, GPU cost, or stale queued callbacks. Restore the original notification path or disable continuous update.
- **Basis:** Inference from O4; previous continuous-update results remain rejected/inconclusive. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/nucbox-live-hillclimb.md:34-49`]**

#### E4 — Decoded-age limit, then compressed-age limits

Run decoded output, ingress, and Linux command aging in separate trials using §3.3.

- **Hypothesis:** A bounded queue can still retain old work; residence-based recovery may reduce persistent lag.
- **Expected gain:** Only observed excess residence. Queue capacity multiplied by frame period is not a measured saving.
- **Owners:** `video_queue.rs`, platform `media.rs`, Linux `session.rs`/`queue.rs`, graphics publication boundary.
- **Measurement:** Queue-age distributions, age discards, recovery/keyframe rate, throughput, source freezes, relative media-lag growth.
- **Accept:** V + L; no increase in recovery episodes, corruption, or interval maxima beyond controls; S for smoothness promotion. The candidate must still meet FPS requirements.
- **Risk/rollback:** Decoder-reset storms or apparent latency improvement achieved by freezing/dropping too much. Disable the tested age threshold.
- **Basis:** Inference from O2 and O4.

Only after age traces show sustained occupancy should the separate capacity candidate be evaluated. It uses the same gates and the 50 ms capacity formula defined above.

#### E5 — Input backlog, adjacent coalescing, then polling

- **Hypothesis:** Old motion or a transport command backlog can dominate local input latency despite healthy video.
- **Expected gain:** The measured queue age removed. Reducing a poll interval can save at most the removed scheduled wait under normal scheduling.
- **Owners:** `CapturedInputQueue`, `EmbeddedInputCapture`, core captured-input forwarding, `nvst_input.rs`, Qt input wrappers.
- **Measurement:** Capture-to-dequeue, transport queue age, SCTP admission, observable transmission, queue depth, coalesced/expired counts.
- **Accept:** V + L and preferably P; zero lost required transitions; no held input after focus loss, queue saturation, text cancellation, or reconnect.
- **Polling candidate:** Test 1 ms only if the actual existing interval is longer. If it is already 1 ms or shorter, mark that candidate inapplicable. Do not change the Wayland capture refresh timer.
- **Risk/rollback:** Changed movement integration, reordered clicks, lost releases, or CPU wakeups. Restore the previous admission/poll policy.
- **Basis:** Inference from O6.

#### E6 — Decoder low-delay and reorder behavior

Instrument first:

- Effective decoder implementation and codec.
- Actual input/output timestamp order.
- Reported decoder delay/reorder state, available thread settings, output picture type.
- H.264/H.265 reorder limits where the existing parser exposes them.
- Time held in synchronization, export, conversion, and output queues.

Missing reorder metadata remains unknown. A B-picture label alone is insufficient to assign a buffering duration.

Test FFmpeg hardware `LOW_DELAY` as one opt-in change only for a stream whose reorder requirements have been established. Native H.264 VA-API is a different implementation and will not acquire FFmpeg behavior merely because a FFmpeg flag was changed.

- **Expected gain:** Zero if already operating without avoidable buffering; otherwise the measured removable decoder residence. One buffered output corresponds to `T`, but no such buffering has yet been established.
- **Owners:** Linux `video/ffmpeg.rs`, `video/vaapi.rs`, `timing.rs`, `session.rs`.
- **Accept:** V + L, identical output identity/count/order in fixtures, no new reference recovery, unchanged color/depth, and P before an end-to-end claim.
- **Risk/rollback:** Incorrect output ordering or driver regressions. Remove the hint.
- **Restrictions:** Never force a reported B-frame count to zero, delete reference pictures, invent a CloudMatch no-B-frame setting, or apply a demuxer “nobuffer” option as though it controlled this access-unit decoder path.
- **Basis:** Inference from O3.

A native-versus-FFmpeg H.264 VA-API comparison is a separate implementation-selection experiment. Both arms must remain explicitly VA-API, use the same profile, and report actual CPU/GPU transfer behavior.

#### E7 — Export/import and surface-retention costs

- **Hypothesis:** CPU download, repeated imports, or retained decoder surfaces account for measured conversion/residence tails.
- **Expected gain:** The measured download/import/wait component; currently unknown.
- **Owners:** Linux `ffmpeg.rs`, `vaapi.rs`, `frame_producer.rs`; Qt native callback.
- **Measurement:** Export path, CPU-transfer count/bytes, import duration, GPU completion, live surface count, and release timing.
- **Candidate:** Reuse existing import/conversion caches. If a missing cache is demonstrated, bound it to active presentation slots and key by device epoch, backing ownership, format, modifier, planes, and dimensions—not raw FD value alone.
- **Accept:** V + L + C; zero unexpected CPU transfers for a claimed GPU-only path; correct gradients/range/depth; no resource growth across ten session/recreation cycles.
- **Risk/rollback:** Premature surface reuse, stale imports, or reference-surface starvation. Restore original resource retention.
- **Restriction:** Do not remove VA synchronization or GPU fences to improve a CPU timer.
- **Basis:** Inference from O3 and O4.

#### E8 — Display cadence, vsync, VRR, and tearing

Keep the existing render loop and synchronization policy as the initial control. The previous basic-loop result is a negative control, not a recommended default.

Evaluate separately:

1. A higher display refresh rate with stream settings held fixed.
2. Compositor-supported VRR.
3. An explicitly supported tearing mode, only when tearing is acceptable for that trial.

- **Expected gain:** Zero to the measured presentation-phase wait; a 60 Hz period is approximately 16.67 ms. No guarantee follows from requesting a mode.
- **Owners:** Qt presentation setup and callback; compositor/display configuration belongs to the external harness.
- **Measurement:** Effective refresh/present behavior, source-frame age, compositor feedback where valid, and optical latency.
- **Accept:** V + P; no source-throughput regression. Record tearing/artifact policy before the trial.
- **Risk/rollback:** Tearing, uneven cadence, ignored requests, or higher GPU use. Restore display/compositor configuration and recreate the normal Qt window.
- **Restrictions:** No second swapchain, direct surface commits, or transplanting the standalone pacer into Qt.
- **Basis:** Inference from O4 and historical basic-loop observations.

For Wayland presentation-time evidence, associate feedback with the **actual Qt-owned surface commit**. Do not attach an arbitrary feedback request and assume it identifies the video frame. Because the commit hook is absent from the evidence, compositor correlation is an external/source-validation dependency. Without it, retain Qt-swap and optical measurements and mark compositor timing unavailable.

#### E9 — Bitrate, FPS, resolution, and codec

Use existing settings and normal CloudMatch negotiation. Record requested and negotiated values. Test:

- Bitrate: 20% below the current negotiated baseline.
- Resolution: one existing lower preset at the same frame rate.
- FPS: the next supported higher rate.
- Codec: H.264 versus HEVC at matched resolution, rate, and color quality.

Each is a separate trial; codec comparisons require independently confirmed decoder paths.

- **Hypothesis:** Workload changes alter network burst size, encode/decode cost, or frame cadence.
- **Expected gain:** Unknown. FPS can reduce a cadence wait; bitrate/resolution can remove measured congestion or processing delay.
- **Owners:** Existing `SettingsState.qml`, settings/session resolution, CloudMatch allocation; no new persistence needed.
- **Measurement:** P, received/admitted/decoded/presented rates, actual bitrate, stage tails, and predefined visual-quality checks.
- **Accept:** V + P with agreed quality limits. A lower-resolution result is a quality/latency tradeoff, not an equivalent-quality improvement.
- **Risk/rollback:** Worse image quality, decoder overload, or increased traffic. Restore saved settings.

Different profiles require a separately frozen comparison protocol. For that protocol, decoded/assembled throughput must reach 95% of negotiated FPS; fresh presentation must reach 95% of the lesser of stream and effective display rate. Do not describe such a result as passing the historical 60 FPS/25 ms campaign.

#### E10 — Mjolnir retention and FEC/reorder recovery

Perform deterministic fault tests before live testing. Test RTTs of 30, 57, and 80 ms, reordering delays of 4/8/16 ms, and isolated bursts of 1/3/8 missing packets. Include no-loss and FEC-repaired controls.

Compare separately:

- Existing 52 ms retention.
- Fixed 68 ms retention.
- RTT-informed retention: `clamp(ceil_ms(fresh RTT + 8 ms), 52 ms, 68 ms)`; use 52 ms without a fresh RTT.

Freeze each loss episode’s deadline when detected. Tracking and packet dequeue use that same deadline. Retry intervals remain RTT + 4 ms; total attempts remain bounded to three, using the selected budget consistently. Admission failure restores the attempt.

- **Hypothesis:** Some requested packets return after current retention expires.
- **Expected gain:** Approximately zero on the healthy path; fewer recovery hitches if the hypothesis is confirmed. Fixed 68 ms can add up to 16 ms of exceptional waiting.
- **Owners:** Transport receive/NACK owners in `nvst.rs`; existing NACK tests and timing instrumentation.
- **Measurement:** Detection, request admission, authenticated requested-packet return, FEC completion, expiry, usable frame completion, and keyframe recovery.
- **Accept:** V; at least 20% fewer expired/recovery episodes beyond control variation in the affected fault case; no healthy-path residence regression; live S/P evidence before promotion.
- **Risk/rollback:** Waiting longer for unrecoverable data. Restore the 52 ms policy.
- **Restriction:** A packet received after a request is not automatically proven to be a retransmission; name that observation accurately.
- **Basis:** Inference from O7.

Do not prioritize GPU FEC or a larger jitter buffer without traces showing FEC computation or recovery eligibility as the bottleneck.

#### E11 — Region, QoS/DSCP, and socket behavior

These are three independent experiment families.

**Region selection can run early as a separate campaign.**

- Use actual in-stream RTT, not TCP endpoint preflight.
- Require at least three fresh sessions per region with matched profile/workload and randomized region ordering.
- **Accept:** At least 5 ms lower RTT median beyond between-session variation, no p95 regression, and P for an input-to-photon claim.
- **Expected gain:** Conditional on a real path improvement; currently unknown.
- **Risk/rollback:** Different rigs/encoders and route drift. Return to the original region.
- **Owner:** Existing region settings and CloudMatch session allocation.

**DSCP/QoS is conditional on congestion evidence.**

- Test DSCP CS4/value 32 on the outbound control/audio bundle as a single opt-in change, preserving ECN bits.
- Verify the actual marking at a permitted capture point.
- Recognize the bundle-wide scope; marking client egress does not establish priority for server video downlink.
- **Accept:** At least 20% lower congestion-induced input/RTT p95, no idle regression, and P for gameplay claims.
- **Expected gain:** Near zero without relevant queuing; otherwise bounded by measured queue inflation.
- **Risk/rollback:** Ignored/reclassified markings or adverse queue treatment. Restore original socket settings.
- **Owner:** Actual UDP socket creation in transport; router shaping is external and requires its own trial.

**Socket changes require kernel evidence.**

Instrument actual `SO_RCVBUF`, receive-overflow counters where available, and kernel-receive-to-dequeue delay. Timestamp clock domains must be recorded and calibrated.

Then test either an application receive-buffer request of 512 KiB **or** bounded receive batching of at most eight datagrams. Do not combine them.

- **Accept:** At least 20% fewer demonstrated socket-overflow drops, or L for kernel queue residence; no client-age/tail regression.
- **Expected gain:** Zero if there is no socket bottleneck.
- **Risk/rollback:** Larger backlog or burstier downstream delivery. Restore original socket options/batching.
- **Restriction:** If the kernel does not honor the request, the candidate is unsupported, not successful.

The previous 8 MiB receive-buffer trial failed its guards and was restored. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/docs/nucbox-live-hillclimb.md:53-69`]**

#### E12 — Audio output latency

- **Hypothesis:** The active ALSA/PipeWire output route retains avoidable audio backlog.
- **Candidate:** Request a 20 ms output-buffer target only if the measured current buffer is larger. Keep packet recovery and decoder policy fixed. Record the actual accepted device configuration.
- **Expected gain:** The measured audio backlog difference; no assumed video improvement.
- **Owners:** Linux `audio.rs`/`session.rs`; shared SDL playout only if confirmed active.
- **Measurement:** Packet-to-PCM time, write blocking, available device-delay measurements, underruns, and external A/V offset.
- **Accept:** At least 5 ms lower measured audio delay beyond control variation; no additional underruns/output failures; A/V offset does not worsen by more than 5 ms.
- **Risk/rollback:** Crackling or repeated output recovery. Restore the original buffer request.
- **Restrictions:** Keep mute draining, fixed-device routing, bounded PLC, and audio-only failure isolation. Do not add a second audio clock that delays video.
- **Basis:** Inference from O7.

#### E13 — Thread priority and affinity — last

- **Hypothesis:** Runnable-to-running delay, rather than pipeline work, dominates a remaining stage.
- **Candidates:** First a modest priority change, such as nice −5, for one demonstrated bottleneck thread. Separately test affinity for that thread using measured CPU topology/load; exclude a core already dominated by IRQ or compositor work.
- **Expected gain:** At most the measured scheduling delay removed; potentially negative.
- **Owners:** Existing Linux worker creation and Qt render-thread initialization; scheduling traces/topology are external harness inputs.
- **Measurement:** Wake-to-run p95/p99, CPU utilization, thermal state, compositor/audio/input progress.
- **Accept:** At least 20% and 0.25 ms lower target scheduling p95; C; no regression in other latency domains; L/P as applicable.
- **Risk/rollback:** Starving the compositor, audio, or network processing. Restore scheduler attributes and affinity.
- **Restrictions:** No real-time FIFO policy, busy polling, or pinning all media threads to one core as a default.

## 4. File-by-file impact

The phases below identify intended owners. Files whose bodies were absent require source verification before modification; their roles are not claimed as inspected implementations.

### Shared/native instrumentation

| File | Changes, rationale, dependencies |
|---|---|
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-protocol/src/client_timing.rs` **new** | Shared frame/input identities, timestamp provenance, stage/reason enums, fixed numeric records, and nonblocking sink contract. Establishes common semantics before other instrumentation. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-protocol/src/lib.rs` | Export the new internal instrumentation module; preserve existing command versions. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-core/src/client_trace.rs` **new** | Implement `ClientTraceRecorder`, bounded event admission, clock, health counters, and background export. Depends on shared types. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs` | Own trace lifecycle; preserve metadata in `consume_encoded_media`; extend captured-input forwarding and optional telemetry serialization. No interpretation of admission as presentation. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/frame_stage_timing.rs` | Extend bounded correlation and stage counts; distinguish sample counts for stages with different availability; rename misleading internal nanosecond vector names without changing wire units. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs` | Add authenticated receive/assembly/recovery/socket hooks. Later, isolated NACK retention and socket candidates. Exact receive and command owners require verification. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst_input.rs` | Carry input timestamps/identity to transport admission; instrument backlog; later implement the bounded/coalesced policy. Exact command queue and send signatures require verification. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst_network.rs` | Add bounded network diagnostic reporting only where appropriate; do not turn VPN-name detection into routing or QoS policy. Actual socket options remain with socket owners. |

### Platform and Linux pipeline

| File | Changes, rationale, dependencies |
|---|---|
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/media.rs` | Add/preserve optional encoded/input timing metadata; instrument queue transformations; modify `run_embedded_linux_monitor` for opt-in readiness; preserve recording/replay input and backend policy. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/video_queue.rs` | Record enqueue residence; extend Linux generation-checked admission; implement opt-in ingress age/capacity policy using existing invalidation and request coalescing. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/embedded_input.rs` | Preserve trace metadata and capture-generation boundaries; extend closure/neutralization tests for backlog handling. Existing text/controller ownership remains authoritative. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform/src/graphics.rs` | Add default trace accessor and token forwarding; trace publish/replacement/acquire/record outcomes. Preserve epoch, one-record, and retirement contracts. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/format.rs` | Add optional frame timing metadata and timestamp-origin representation. Update constructors/validation without changing GPU backing ownership. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/timing.rs` | Extend `DecodeTimingProbe` with bounded unique identity matching and explicit unmatched reasons; retain existing timing outputs. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/queue.rs` | Add atomic `wait_pop_latest`; test wake, timeout, closure, and skipped-count behavior. Later add selective decode-work discard needed by age policy. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/session.rs` | Add `DecodedFrameReceiver`; carry metadata through commands and output; serialize generation-sensitive mutations; instrument decode/recovery. Later add command aging and audio target configuration. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/ffmpeg.rs` | Record actual timestamp provenance, decoder configuration, and transfer path; distinguish conversion/snapshot timing; add isolated low-delay and implementation-selection candidates. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/vaapi.rs` | Trace surface synchronization/export and retain sender correlation through output. Keep synchronization and export ownership intact. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/frame_producer.rs` | Preserve frame identity and expose import/conversion/completion/retention timing. Any cache change is conditional on E7; implementation body requires verification. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/audio.rs` | Add optional measured device-delay reporting where available; distinguish unsupported measurement from zero; later implement the explicit buffer-target experiment. |

New dependencies on shared timing types must be registered in the affected crate manifests if not already present. Constructor-literal updates must land with metadata changes, including Linux fixtures and any unprovided backend constructors discovered by compilation.

### FFI and Qt

| File | Changes, rationale, dependencies |
|---|---|
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-ffi/src/lib.rs` | Add versioned optional trace queries/event submission and metadata-aware input entry points. Reuse existing input admission; preserve current graphics layouts and untraced APIs. Body requires verification. |
| `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-ffi/include/opennow_streamer_trace.h` **new** | Declare the optional trace extension using existing opaque runtime/frame types. Include the actual existing ABI header; its filename was not supplied. |
| `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/streaming/NativeStreamRuntime.h` and `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/streaming/NativeStreamRuntime.cpp` | Resolve optional trace functions; record notification/input/record boundaries; expose immutable copied trace identity. Modify notification rearm only for E3 after inspecting current behavior. |
| `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/streaming/StreamVideoItem.h` and `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/streaming/StreamVideoItem.cpp` | Record update requests and measurement version/health; preserve GUI-thread update dispatch and current experimental continuous-update option. |
| `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/streaming/StreamVideoItemInput.cpp` | Stamp actual input callbacks and carry trace metadata; preserve click position, shortcuts, releases, and capture transitions. |
| `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/input/platform/WaylandPointerCapture.h` and `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/input/platform/WaylandPointerCapture.cpp` | Add timestamp/provenance delivery for relative samples; retain fractional deltas and capture lifecycle. Do not repurpose the refresh timer. |
| `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/streaming/rendering/StreamPresentTimings.h` | Correctly classify source measurements; add `ProgressSnapshot`; separate mutex-protected copying from percentile calculation; version changed semantics. |
| `/Users/danielsivan/dev/OpenNOW/opennow-qt/src/streaming/rendering/NativeStreamRenderCallback.cpp` | Gate submission markers on actual draws; trace acquired/recorded/imported/drawn identity; use lightweight observation in E1; preserve external-command and teardown sequencing. |
| `/Users/danielsivan/dev/OpenNOW/opennow-qt/tests/latency/` **new test group** | Test false draws, repeated/generated frames, generation changes, trace unavailability, notification races, and timing snapshot equivalence. Register with the existing Qt test build owner, whose contents were not supplied. |

### Documentation and external handoff

Update these existing documents:

- `/Users/danielsivan/dev/OpenNOW/docs/nucbox-linux-client.md`: trace identity, stage definitions, and active-path verification.
- `/Users/danielsivan/dev/OpenNOW/docs/nucbox-live-hillclimb.md`: append the new measurement version and future results; preserve previous failures and comparison blocks.
- `/Users/danielsivan/dev/OpenNOW/docs/linux-video-backends.md`: clarify implementation-specific transfer diagnostics.
- `/Users/danielsivan/dev/OpenNOW/docs/linux-vulkan-acceptance.md`: add correlated ownership/completion checks and optical/compositor evidence limits.
- `/Users/danielsivan/dev/OpenNOW/docs/mjolnir-nack-v2.md`: document optional experimental retention only when E10 is implemented.
- `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/README.md`: describe optional tracing and its compatibility/lifecycle.

The remote harness remains external. Its required work is clock calibration, raw-trace aggregation, optical/compositor capture, effective-configuration verification, randomized comparison blocks, and the separately frozen latency/profile gates.

Region/profile experiments use the existing settings and allocation interfaces. This plan does not introduce speculative CloudMatch fields or require changes to `SettingsState.qml`/`ShellStore.qml` merely to run those experiments.

## 5. Risks and migration

**Rust source compatibility.** Adding fields to public runtime frame/sample structs affects struct literals even when constructors retain their signatures. Land declarations, all in-workspace construction sites, and consumers atomically. External Rust consumers, if any, require matching updates. These fields are diagnostic runtime state, not a settings or recording schema migration.

**FFI compatibility.** Existing graphics layouts and entry points remain unchanged. New Qt tracing functions are optional and version/size checked. Older runtimes keep rendering but cannot qualify for trace-dependent experiments.

**Measurement compatibility.** Corrected fresh-draw semantics require a new baseline. Preserve historical reports; do not reinterpret them as if they used measurement version 3.

**Cancellation limits.** Readiness waits must be interruptible or bounded and must not obstruct submission/stop locks. This does not make driver calls cancellable. The current startup timeout is followed by `join()`, and native VA output includes a synchronous surface wait; neither establishes a hard end-to-end shutdown deadline. **[Direct observation: `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/session.rs:238-256`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/session.rs:428-454`, `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/vaapi.rs:281-284`]**

**Rollback.** Every candidate defaults off. Restore its option, perform an orderly session/client restart when required by that option, verify the effective baseline configuration, and collect the repeated control. Returning a setting to its old value is not itself evidence of restored behavior.

## 6. Implementation order

1. **Resolve the evidence and integration inventory.** Inspect the omitted transport, FFI, runtime, frame-producer, and build owners. Obtain the fresh baseline and exact wake-trial evidence. Confirm the real backend and transfer path.
2. **Define shared trace types and bounded recorder.** Compile and test event saturation, clock provenance, and session isolation independently.
3. **Land frame/sample metadata propagation atomically.** Update constructors and all transformations together. Verify duplicate timestamps, reordered outputs, missing timestamps, resets, and unmatched behavior.
4. **Land optional FFI/Qt tracing atomically.** Verify older-runtime trace unavailability and ensure no token is queried after release.
5. **Correct fresh-draw measurement and version it.** Test failed draws, repeated draws, generated frames, gating, and recreation. Preserve historical measurement records.
6. **Qualify the observer.** Run E0, optical/compositor calibration where available, and repeated baseline controls. Explain large phase variation before scoring speculative changes.
7. **Run E1, then E2.** Measure render-observation overhead and decoded-readiness publication independently.
8. **Run E3 only where traces justify it.** Validate notification rearm deterministically; evaluate continuous redraw under controlled phase conditions.
9. **Run queue and input experiments separately.** Decoded age first; then ingress age, command age, bounded input, adjacent coalescing, and polling. Keep each variable isolated.
10. **Run decoder and resource-path experiments.** Establish reorder behavior before low-delay hints; verify real export/import/completion before resource optimization.
11. **Evaluate display and stream-profile tradeoffs.** Use optical evidence and a separately frozen protocol for changed profiles.
12. **Run region comparisons as an independent session campaign.** It may begin after measurement qualification without supplying controls for local same-seat experiments.
13. **Run loss/QoS/socket experiments only when their triggering measurements exist.** Complete deterministic recovery tests before live retention changes.
14. **Evaluate audio and scheduler changes last.** Require evidence from the active output route or runnable-delay measurements.
15. **Promote only the demonstrated result.** State which of V, S, L, P, and C passed, retain negative controls, and confirm the candidate on another scene and cloud session before making it a production default.

End of Oracle group: 2 lanes above.



> 💡 The 2 Oracle lanes above are independent answers; reconcile them using the guidance at the top of the group. Lane order is not a ranking.

Optional later follow-up: Continue this plan conversation with ask_oracle(chat_id: "gfn-latency-remote-test--652397", new_chat: false)