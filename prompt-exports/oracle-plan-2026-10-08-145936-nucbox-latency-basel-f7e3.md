## Final Prompt
<taskname="NucBox Baseline Plan"/>

<task>
Plan and execute only the real NucBox Qt/native OpenNOW latency and freeze baseline through the agreed checkpoint, then stop for coordinator review. Produce an actionable single-operator shutdown, build, relaunch, and measurement sequence; reliable evidence gates; and an append-only scoreboard scaffold at `prompt-exports/optimize-nucbox-latency-freezes-runs.md`. After scoring, analyze existing telemetry for stage liveness/stalls and rank 2–3 smallest candidate hypotheses for later review. Do not optimize or add production instrumentation during this phase. Preserve every invalid, partial, or outlier sample and state exclusions explicitly; never exclude a sample merely because it is slow. Use existing collector/hooks; propose instrumentation only if the baseline demonstrates a specific measurement gap, and defer it beyond this checkpoint.
</task>

<architecture>
- `.agents/skills/verify-gfn-experiments/SKILL.md` and `features/latency-baseline.md` define the remote operator workflow, gates, collector use, and evidence preservation.
- `scripts/nucbox-latency-baseline.py` is the local mirror of the bounded collector. On the NucBox the command is `tools/latency-baseline.py`; it reuses observer/live-trial/presentation-census and records identity outside measured windows.
- `opennow-qt/src/streaming/StreamVideoItem.cpp`, `StreamPresentTimings.h`, `NativeStreamRenderCallback.cpp`, and `NativeStreamRuntime.{h,cpp}` define existing Qt source-swap, submit, histogram, epoch, liveness/watchdog telemetry.
- Native liveness path: NVST transport receive/assembly/recovery → core admitted/ACK/decode/output → Linux decoder session → embedded publisher → Qt frame consumption/presentation. `log.rs` exposes existing assembly/admission/decode counters; Linux `session.rs` exposes decoder-stage events; `media.rs` owns publisher polling; `vulkan_copy.rs` implements synchronous Vulkan snapshot copying and fence waits.
- `docs/nucbox-latency-baseline-20261008.md` is the historical baseline reference, not a current-build or matched control.
</architecture>

<selected_context>
- `.agents/skills/verify-gfn-experiments/SKILL.md`: NucBox operation constraints and live measurement requirements.
- `.agents/skills/verify-gfn-experiments/features/latency-baseline.md`: exact collector workflow, dry-run, multiple windows, and evidence rules.
- `docs/nucbox-latency-baseline-20261008.md`: prior conditions, results, commands, known validity caveats.
- `scripts/nucbox-latency-baseline.py`: collector logic and evidence output structure.
- `opennow-qt/src/streaming/StreamPresentTimings.h`, `StreamVideoItem.cpp`, `NativeStreamRenderCallback.cpp`, `NativeStreamRuntime.h/.cpp`: presentation measurements, statistics emission, watchdog semantics.
- `native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs`: existing exported assembly/admission/decode counters.
- `native/opennow-streamer/crates/opennow-streamer-platform-linux/src/session.rs`: existing decoder stage telemetry/events.
- `native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/vulkan_copy.rs`: snapshot-copy and fence-wait path relevant to a possible decoder stall.
- Targeted slices from `native/opennow-streamer/crates/opennow-streamer-platform/src/media.rs`, `...transport/src/nvst.rs`, and `...core/src/lib.rs` show publisher poll/shutdown, upstream receive/assembly, and core admission/output liveness.
</selected_context>

<relationships>
- NVST receive/auth and packet assembly/recovery → core assembled/admitted/ACK liveness → decoder submit/output → Linux backend frame publication → Qt source swap and render submit.
- Collector → live-trial identity + `observe-stream.py` windows + presentation census; bounded socket/network observers supply concurrent context when available. Align observer intervals explicitly with scoring.
- Qt telemetry distinguishes source-swap progress and render submissions, but histogram sampling/capacity and watchdog trigger conditions limit claims about precise freeze duration.
- Vulkan path: FFmpeg shared Vulkan decode → `VulkanCopyPool::copy()` snapshot → fence wait → published frame. A wait timeout is a candidate mechanism, not an established cause.
</relationships>

<known_state>
- Target local/remote branch `nucbox-wayland-hdr`, requested HEAD `6472cae764a270864cd20446f6c1beafa1bfce8b`. Remote checkout was clean at requested HEAD. Local status has only the already user-authorized AGENTS.md modification and unrelated untracked `prompt-exports/oracle-plan-2026-10-08-134034-orchestrate-nucbox-r-eafd.md`; preserve both and do not edit/replace AGENTS.md.
- At `/home/gamer/dev/gfn-client-research/.runtime/verification-deploy-tgLBRD4n`, build was NOT RUN; no build markers were found. A read-only probe saw old Qt PID 1611294/start ticks 16362358 and child core PID 1611498/start ticks 16362453 alive. Re-read remote guides and re-inspect identities before acting; these identities can go stale.
- User authorized normal GUI shutdown including foreground Cua fallback, but no kill or bypass. Verify the Qt/core processes exited through GUI shutdown before building. Use Cua background delivery so it does not steal focus; foreground only as authorized fallback.
- Build-once at that checkpoint enforces source/clean/no-Qt-core-alive preconditions, atomically marks invocation, runs `tools/build-native.sh` once, and keeps original logs/receipt/hashes. If progress/heartbeats continue, keep the original invocation alive; never duplicate, retry, cancel, or replace it.
- Sole GUI/account operator. Preserve session/save reuse. No router/VPN/WiFi/SQM/rmem/display/profile/region tuning; no flint/router/tuning actions. No publication, commits, pushes, history rewrites, or vendor edits.
</known_state>

<baseline_protocol>
- Prior setup: H.265 10-bit 4:2:0 HDR, 2560×1080 source at about 60 FPS; Wayland fullscreen on 3440×1440 @100 Hz desktop; frame generation off; saved Germany profile; stationary animated Ori Ink Water, Marsh, Spirit Well scene; stats overlay closed. Record observed state fresh before scoring; do not silently normalize a changed condition.
- Historical run (HEAD `819ce3f`, not current target build/control) had three contiguous windows: p95/p99/max 21/25/36 ms, 21/25/35 ms, 21/25/41 ms; >25 ms counts 28/30/30, rates 0.77/0.83/0.83%. It did not establish a long freeze. Three contiguous windows alone do not establish repeatability or across-attempt variance.
- Establish baseline repeatability with 3 fresh scored invocations, each 3 windows, same session/profile/scene/window/fullscreen/overlay, unless a documented practical/evidence gate prevents completion. Keep each attempt’s identity, receipts, raw outputs, observer data, and status.
- From remote cwd `/home/gamer/dev/gfn-client-research`, run the collector dry-run first with an unused preflight label, then measured invocation. Exact shape (replace placeholders only with freshly verified values and unique safe labels):

```sh
python3 tools/latency-baseline.py --pid VERIFIED_PID --width VERIFIED_WIDTH --height VERIFIED_HEIGHT --display-hz VERIFIED_HZ --scene ori-inkwater-marsh-spirit-well --label NEW_SAFE_LABEL --dry-run
python3 tools/latency-baseline.py --pid VERIFIED_PID --width VERIFIED_WIDTH --height VERIFIED_HEIGHT --display-hz VERIFIED_HZ --scene ori-inkwater-marsh-spirit-well --label NEW_SAFE_LABEL --warmup 20 --seconds 60 --windows 3
```

- The script accepts 60/100/120 Hz. Its value sets a validation expectation, not the desktop setting. The described frozen Doctor `--require-stream` check accepts 60/120; do not claim that gate passes at 100 Hz. The 100 Hz descriptive collector runs general Doctor and its own explicit profile/telemetry checks; report actual outputs and this caveat accurately.
- Start bounded socket/network observers for each attempt using a private per-attempt directory and unique output paths; feature-map examples use 240 s for three windows, but verify actual start/end overlap and extend only to the scoring interval when needed. No `--flint`. Observers are context, not packet-age data. No screenshots, input, build, tests, or hashing within scored windows. Close stats/overlays before the attempt.
- Collector writes private `.runtime/latency-baseline-LABEL/{summary.json,summary.md,observation.json,native-numeric-samples.json,qt-census-samples.json}`. It requires fresh Qt/native telemetry, process identity continuity, valid windows, census, and stable build identity. Dry-run is preflight only and never counts as a scored baseline. Preserve invalid and partial artifacts.
- Do not claim physical input-to-photon latency. Report whether receive/assembly, decode/output, publisher, and Qt source-swap/render stages remain live or stall based on existing raw observations; where logs only bound a stall or lack samples, say so. Decode rolling last-256 p50/p95/max overlaps warmup and is not a true per-window p99; Qt 1 ms histogram buckets have finite capacity and overflow loses exact maximum. These data can bound sampled stalls, not necessarily measure exact individual freeze duration.
</baseline_protocol>

<scoreboard>
Create `prompt-exports/optimize-nucbox-latency-freezes-runs.md` as an append-only run ledger scaffold before scoring. Include frozen target identity/configuration, operator and checkpoint, fields for attempt/window IDs, timestamps, build/source/tool hashes and receipt paths, observed session/profile/scene/display state, collector/network observer paths, latency and stage-liveness metrics, outcome/validity, exclusions with reasons, and freeform anomalies. Seed it with the 2026-10-08 historical run explicitly marked historical/unmatched, and three blank attempt sections. Append results as evidence arrives; never overwrite prior rows or delete failed/invalid/outlier evidence. Slowness alone is never an exclusion reason.
</scoreboard>

<post_scoring_analysis>
After all feasible attempts, use existing raw observe-stream/log/Qt telemetry to align stage liveness and identify where progress stops or resumes. Preserve all samples, label incomplete attempts, report variability across fresh invocations and windows, list explicit technical exclusions and their predeclared evidence-based reasons, and distinguish observed fact from inference. If no long freeze is seen, say NOT REPRODUCED in this baseline. Do not modify production code or add instrumentation during this checkpoint.

Rank these unproven small hypotheses for coordinator review, updating rank only from measured evidence:
1. Linux FFmpeg Vulkan conversion synchronously calls `VulkanCopyPool::copy()` and waits on a fence (1 s timeout) on the decode path; this could cause decode-tail stalls. Validate first against existing decode/publisher timing and fence/error logs; preserve GPU completion and ownership correctness in any future proposal.
2. Linux embedded publisher polls at a 2 ms default interval (`media.rs`); phase delay could contribute to ready-frame publication latency. Any later wake-driven alternative needs careful lost-wakeup/shutdown reasoning.
3. Qt watchdog is driven by `recordFrame`; it may require pending submit plus advancing decode and may miss quiescent publisher/render stalls. It does not by itself explain prior 35–41 ms tails absent correlated evidence.

Do not implement or instrument any candidate now. If evidence exposes a concrete observability gap, describe it and defer instrumentation proposal to the coordinator after this checkpoint.
</post_scoring_analysis>

<ambiguities>
- Existing telemetry does not prove physical input-to-photon latency or guarantee exact freeze duration; use stage-liveness language and qualify inference.
- The historic baseline differs in build identity and has no matched candidate/control. It is context only.
- The 100 Hz desktop conflicts with Doctor’s frozen 60/120 Hz acceptance profile. Keep 100 Hz as observed descriptive state; do not represent it as acceptance.
- Whether all 3×3 windows are practically achievable is unknown until the live operator follows the evidence gates. If blocked, report exactly which gate blocked completion; keep all prior evidence.
</ambiguities>

<checkpoint_output>
Stop after setup/build/relaunch/baseline evidence, scoreboard append, and post-scoring existing-data analysis. Return exact commands and receipts/private paths, build and run evidence, whether repeatability was achieved, every validity/exclusion decision, stage-liveness findings and limitations, ranked smallest next candidate, and unresolved gaps. No optimization, production instrumentation, or continuation beyond coordinator review.
</checkpoint_output>

## Selection
- Files: 15 total (7 full, 8 slice)
- Total tokens: 53075 (Auto view)
- Token breakdown: full 28879, slice 24196
- Token accounting: stale from active_tab_published; refresh pending

### Files
### Selected Files
├── .agents/
│   └── skills/
│       └── verify-gfn-experiments/
│           ├── features/
│           │   └── latency-baseline.md — 1,133 tokens (full)
│           └── SKILL.md — 3,203 tokens (full)
├── docs/
│   └── nucbox-latency-baseline-20261008.md — 3,003 tokens (full)
├── native/
│   └── opennow-streamer/
│       └── crates/
│           ├── opennow-streamer-core/
│           │   └── src/
│           │       └── lib.rs — 2,777 tokens (lines 2280-2340 (Core assembled/admitted/ACK counters and liveness updates downstream from transport.), 2410-2460 (Core decode/output/recovery progress observations used to locate a pipeline stall.))
│           ├── opennow-streamer-platform/
│           │   └── src/
│           │       └── media.rs — 3,159 tokens (lines 3190-3410 (Linux embedded frame publisher poll interval and monitor loop, decode-to-publish lifecycle, and shutdown behavior; relevant candidate hypothesis.))
│           ├── opennow-streamer-platform-linux/
│           │   └── src/
│           │       ├── video/
│           │       │   └── vulkan_copy.rs — 1,193 tokens (lines 1-45 (Vulkan snapshot pool ownership, capacity, and timeout constants used by the synchronous decode copy path.), 190-445 (VulkanCopyPool::copy, GPU submission, synchronous fence wait, output ownership, and timeout handling; relevant unproven decode stall hypothesis.))
│           │       └── session.rs — 3,444 tokens (lines 650-750 (Linux session decoder receive/decode/output stages and their existing liveness events; needed to interpret stage stalls.))
│           ├── opennow-streamer-protocol/
│           │   └── src/
│           │       └── log.rs — 839 tokens (lines 230-330 (Existing native log telemetry fields for packet assembly/admission/decode progress used in post-score stage liveness analysis.))
│           └── opennow-streamer-transport/
│               └── src/
│                   └── nvst.rs — 2,822 tokens (lines 4560-4610 (NVST packet assembly and transport counter/liveness updates used to distinguish receive and assembly stalls.), 7470-7690 (NVST authenticated receive, packet processing and recovery/retransmit path; supports upstream liveness interpretation.))
├── opennow-qt/
│   └── src/
│       └── streaming/
│           ├── rendering/
│           │   ├── NativeStreamRenderCallback.cpp — 6,481 tokens (lines 350-610 (Qt telemetry snapshot emission, recordFrame, watchdog/progress behavior and source submit; needed to establish what Qt stage liveness means.))
│           │   └── StreamPresentTimings.h — 3,105 tokens (full)
│           ├── NativeStreamRuntime.cpp — 12,275 tokens (full)
│           ├── NativeStreamRuntime.h — 2,609 tokens (full)
│           └── StreamVideoItem.cpp — 3,481 tokens (lines 260-365 (Periodic Qt presentation stats serialization including source swaps, epochs, histogram and since-last-swap telemetry.))
└── scripts/
    └── nucbox-latency-baseline.py — 3,551 tokens (full)


---

## Generated Plan
- Oracle group status: failed
- Oracle group: `37FC70D2-9CD0-4D59-90FD-A26140D44628`

**Reconciling these Oracle lanes**
2 independent answers to the same request follow. Lane order is not a ranking; the first lane supplies the top-level continuation handle, and a successful follow-up through any lane's chat ID re-runs every lane.
- Read every lane through the end-of-group marker (`End of Oracle group: 2 lanes above.`). If the marker or a lane is missing, page the export or try the read-only `oracle_chat_log` with that lane's chat ID. Logs may be scoped or clipped; report any remaining gap. Do not start a follow-up just to retrieve prior text.
- Reconcile by evidence, not lane order, answer length, or model identity: check material single-lane and conflicting claims against the code, and report unresolved disagreements. A failed or partial lane is incomplete evidence.
- Before synthesizing, inventory every material claim from every lane, including single-lane claims. Merge only exact duplicates, retaining all source lanes. Begin your answer with `**Oracle reconciliation**`, state how many lanes completed and name any that did not. For every inventory item, give its source lanes, checked evidence, and exactly one disposition: `accepted`, `rejected`, or `unresolved`. Never silently omit an item.

Lanes (2):
- Oracle — `claude-opus-5-5-xhigh` — Failed — chat ID `nucbox-latency-baseline--228BC4`
- Oracle 2 — `gpt-6-astra-pro` — Completed — chat ID `nucbox-latency-baseline--1C9B2F`

### Oracle
- Status: Failed
- Provider: `Custom`
- Model: `claude-opus-5-5-xhigh`
- Effective effort: Provider default / not specified
- Chat: `nucbox-latency-baseline--228BC4`

Error [context_builder_inactivity_timeout]: Context Builder Oracle exceeded its 600s inactivity budget during streaming.

### Oracle 2
- Status: Completed
- Provider: `Custom`
- Model: `gpt-6-astra-pro`
- Effective effort: Provider default / not specified
- Chat: `nucbox-latency-baseline--1C9B2F`



## 1. Summary

Use the existing NucBox workflow and collector for one controlled checkpoint: verify ownership and source identity, close the existing Qt/core instances through the authorized GUI flow, complete the guarded build once, relaunch the measured baseline, and collect **three fresh invocations of three windows each** under the same session and conditions. The repository change is limited to an append-only ledger at `prompt-exports/optimize-nucbox-latency-freezes-runs.md`; existing production code, collectors, and verification rules provide the measurement path. Preserve every attempt, distinguish measurement validity from performance outcomes, analyze existing telemetry after scoring, and return the evidence and provisional candidate ranking for coordinator review. **This response supplies the plan and ledger scaffold only: no host access, shutdown, build, measurement, or file creation has occurred. Current-build results, repeatability, and freeze reproduction remain unassessed.**

**Evidence labels used below:**

- **Direct observation** means an observation of the supplied source or historical document, not a new live observation.
- **Inference** identifies a conclusion supported by those observations.
- **Plan decision** specifies the implementation or operating rule.
- **Checkpoint input** identifies state or authorization supplied in `<known_state>` and `<baseline_protocol>`. Those inputs have no supplied file-line provenance; they must be verified live rather than assigned fabricated citations.

## 2. Current-state analysis

### Existing pipeline and ownership

| Boundary | Existing responsibilities and observable state | Evidence |
| --- | --- | --- |
| UDP receive → authenticated media → assembled frames | The NVST receive loop reads datagrams, distinguishes source/STUN handling, processes authenticated media, forwards receive events, and checks packet and produced-frame progress. Its periodic diagnostic includes receive/authentication/assembly counters and an elapsed time. | **Direct observation:** `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4560-4610`; `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7470-7690`. |
| Assembly/delivery → admission → ACK control queue | Core serializes `FrameStageTimings`, including assembled, admitted, queued-ACK, undelivered, pending, and unmatched counts. ACK timing ends at the control queue; it does not establish wire transmission or acknowledgment by the peer. | **Direct observation:** `native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2280-2340`. |
| Encoded submission → decoder output | The Linux decoder worker handles generation changes, clears decoded state and timing state during recovery, records submissions, measures decoder calls, records outputs, and enqueues returned frames. | **Direct observation:** `native/opennow-streamer/crates/opennow-streamer-platform-linux/src/session.rs:650-750`. |
| Vulkan decoder frame → presentation snapshot | `VulkanCopyPool` retains the FFmpeg source, submits a GPU copy, waits synchronously on a fence, and publishes an owned snapshot after completion. The snapshot pool is bounded at 12; the fence timeout is one second. A failed wait marks the pool failed and invalidates its shared device owner. | **Direct observation:** `native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/vulkan_copy.rs:1-45`; `native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/vulkan_copy.rs:190-445`. |
| Decoded queue → embedded GPU publisher | `run_embedded_linux_monitor` takes the latest decoded frame, reports skipped decoded frames, converts through `LinuxGpuFrameProducer`, and publishes under a graphics-context lease. It also samples and forwards decode timing reports. The default polling sleep is 2 ms; only the recognized `"1"` override selects 1 ms. | **Direct observation:** `native/opennow-streamer/crates/opennow-streamer-platform/src/media.rs:3190-3410`. |
| Native notifications → GUI/render consumption | `NativeStreamRuntime` owns callback state, notification histograms, presentation generation, record-stage counters, and the FFI handle. Ordinary calls share the handle lifetime lease; graphics/lifetime transitions use exclusive ownership. `recordLatestFrame` records acquisition/record outcomes and releases failed acquisitions’ frame tokens. | **Direct observation:** `opennow-qt/src/streaming/NativeStreamRuntime.cpp:157-239`; `opennow-qt/src/streaming/NativeStreamRuntime.cpp:638-718`; `opennow-qt/src/streaming/NativeStreamRuntime.h:100-132`. |
| Render recording → Qt swap callback | `recordFrame` updates render/submit bookkeeping and invokes `observeSwapProgress`. `frameSwapped` consumes submission state and updates swap statistics. `StreamPresentTimings` stores rolling samples, cumulative histograms, epochs, and a strict `>25 ms` late-interval count. These callbacks do not establish physical presentation. | **Direct observation:** `opennow-qt/src/streaming/rendering/NativeStreamRenderCallback.cpp:350-610`; `opennow-qt/src/streaming/rendering/StreamPresentTimings.h:13-109`; `docs/nucbox-latency-baseline-20261008.md:76-88`. |

Three consequences determine the evidence plan:

1. **Inference — a flat reported decode counter does not uniquely identify a decoder stall.** Decode timing reports pass through the embedded publisher monitor before core emits them. If that monitor stops advancing, core can retain an older report even while the decoder’s independent state differs. Localization requires corroborating counter changes or existing events. Supporting observations: `native/opennow-streamer/crates/opennow-streamer-platform/src/media.rs:3190-3410`; `native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2410-2460`.

2. **Inference — watchdog silence cannot exclude a quiescent render or publisher stall.** The visible watchdog observation runs from `recordFrame`, after its presentation/generation guard. The watchdog implementation itself is not supplied, so its exact trigger thresholds cannot be asserted. Supporting observation: `opennow-qt/src/streaming/rendering/NativeStreamRenderCallback.cpp:350-610`.

3. **Inference — notification counts, acquisition counts, render bookkeeping, and swaps must remain separate.** They describe different boundaries. In particular, `recordFrame` records fresh-submit bookkeeping independently of the `drawIssued` result, so a growing fresh-submit count alone does not prove a draw occurred. Supporting observations: `opennow-qt/src/streaming/rendering/NativeStreamRenderCallback.cpp:350-610`; `opennow-qt/src/streaming/rendering/StreamPresentTimings.h:111-145`.

### Collector reuse and evidence limits

**Direct observation:** The collector already loads `observe-stream.py` and `presentation-census.py`, delegates build/process checks to `live-trial`, rejects reused labels, checks fresh Qt/native telemetry, and compares build identity before and after observation. It writes summaries and preserves numeric samples after successful observation returns. Its native parser retains network values and stage quantiles, **not the liveness counters or NVST receive-stat lines** needed for the requested stall analysis.  
References: `scripts/nucbox-latency-baseline.py:20-45`; `scripts/nucbox-latency-baseline.py:63-113`; `scripts/nucbox-latency-baseline.py:115-204`.

**Plan decision:** Reuse the collector unchanged. Preserve private snapshots of the existing diagnostic logs and Qt telemetry alongside its outputs; do not extend its parser during this checkpoint.

**Direct observation:** Native diagnostic serialization exports decode rolling quantiles and selected decode/frame-stage counters, but omits the three frame-stage quantile groups already available in core. Therefore those collector fields remain unavailable with this supplied implementation. The diagnostic worker is bounded and can drop traces under storage pressure.  
References: `native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs:230-330`; `native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2280-2340`.

**Direct observation:** The historical run used source `819ce3f37ff5315d0940f02df7272856d0d0f362`, with three approximately one-minute windows and fresh-source histogram maxima of 36, 35, and 41 ms. The document explicitly limits its claims to that descriptive baseline. It is not evidence for the requested current build.  
Reference: `docs/nucbox-latency-baseline-20261008.md:8-17`; `docs/nucbox-latency-baseline-20261008.md:41-64`; `docs/nucbox-latency-baseline-20261008.md:88-88`.

### Change scope

**Plan decision:** This is an operational and evidence-recording change. No refactor, ABI change, new timing type, telemetry field, queue policy, watchdog modification, or persistence migration is needed. The supplied feature already defines the descriptive collection path and its cleanup contract.  
Reference: `.agents/skills/verify-gfn-experiments/features/latency-baseline.md:18-37`; `.agents/skills/verify-gfn-experiments/features/latency-baseline.md:76-82`.

## 3. Design

### A. Freeze ownership, identities, and the operating block

**Checkpoint inputs:**

- Requested branch: `nucbox-wayland-hdr`.
- Requested HEAD: `6472cae764a270864cd20446f6c1beafa1bfce8b`.
- Existing deployment checkpoint:  
  `/home/gamer/dev/gfn-client-research/.runtime/verification-deploy-tgLBRD4n`.
- Last supplied observation: build not run; Qt `1611294 / 16362358` and core `1611498 / 16362453` were alive. These PID/start-tick pairs are lookup hints only.
- Authorized interruption: normal GUI shutdown, including the authorized foreground Cua fallback. **No kill, forced termination, or shutdown bypass.**
- Preserve the authorized local `AGENTS.md` modification and unrelated untracked Oracle-plan file.

**Plan decision — one operator:** The implementing engineer is the sole GUI/account/session operator. Socket and host observers may run concurrently because they are read-only. No authenticated standalone core, region probe, or trial-profile tool runs beside the GUI. This follows the supplied ownership rule.  
Reference: `.agents/skills/verify-gfn-experiments/SKILL.md:8-12`.

Before host actions, the operator reads the current remote guides named by the supplied skill:

- `/home/gamer/.codex/AGENTS.md`
- `/home/gamer/.agents/skills/operate-gfn-nucbox/SKILL.md`
- `/home/gamer/.agents/skills/tune-gfn-vpn-routing/SKILL.md`
- `/home/gamer/.agents/skills/cua-driver/SKILL.md`
- `/home/gamer/.agents/skills/cua-driver/LINUX.md`

Reading the latency/network guide does not authorize configuration changes. The checkpoint’s prohibition on tuning remains controlling.  
Reference: `.agents/skills/verify-gfn-experiments/SKILL.md:10-12`.

Choose one unused safe block label, such as `20261008-baseline-b01`, and record it before execution. Reserve labels in the ledger; **do not pre-create collector output directories**, because the collector creates them exclusively and rejects existing directories.  
Reference: `scripts/nucbox-latency-baseline.py:128-136`.

Use these destinations:

| Destination | Ownership and purpose |
| --- | --- |
| Local `prompt-exports/optimize-nucbox-latency-freezes-runs.md` | Authoritative append-only ledger. Create locally before scoring; do not mirror it into the remote checkout during the build/measurement block. |
| Remote `.runtime/nucbox-baseline-BLOCK/` | New private operator proof directory, mode `0700`, with private files created under `umask 077`. |
| Existing `.runtime/verification-deploy-tgLBRD4n/` | Original guarded-build state, invocation, logs, and receipts. Preserve in place. |
| Remote `.runtime/latency-baseline-LABEL/` | Collector-owned directory for each unique preflight or measured invocation. |

Keeping the ledger local prevents its creation or appends from disturbing the remote clean-build predicate or the collector’s remote identity comparisons.

### B. Shutdown, archive, and guarded build

#### Gate B1 — establish live state

Execute the following read-only checks from the verified remote checkout, outside any scored interval:

```sh
cd /home/gamer/dev/gfn-client-research
git branch --show-current
git rev-parse HEAD
git status --porcelain=v1 --untracked-files=all
python3 .cursor/skills/verify-gfn-experiments/scripts/doctor.py --capabilities
```

Record the actual host, UTC time, branch, HEAD, status, Qt/core process relationships, PID/start ticks, and current Cua window identity. Do not substitute historical process identities for this check.

If remote HEAD or cleanliness violates the existing build guard, stop at that gate. Do not reset, clean, broadly synchronize, or edit the checkout to manufacture the precondition.

**Direct observation:** Doctor reports identities and build fingerprints, but does not rebuild or establish that a binary corresponds to a newly changed source tree.  
Reference: `.agents/skills/verify-gfn-experiments/SKILL.md:57-61`.

#### Gate B2 — complete normal GUI shutdown

1. Announce the already-authorized playback interruption.
2. Take a fresh Cua snapshot and target the verified OpenNOW instance.
3. Use background delivery first; use foreground only as the already-authorized fallback.
4. Complete the normal close/confirmation flow.
5. Recheck the recorded Qt PID/start-tick instance and its recorded core child.
6. If confirmation returned to the library and the client remains, take another snapshot and close that remaining application window normally.
7. Confirm both recorded instances exited and no replacement Qt/core owner appeared.

A vanished window or returned library view is insufficient. The supplied workflow explicitly identifies that failure mode.  
**Direct observation:** `.agents/skills/verify-gfn-experiments/SKILL.md:16-20`.

**Plan decision:** If normal GUI closure cannot complete, record `gui_shutdown_incomplete` and stop before building. The checkpoint’s narrower no-kill instruction overrides the skill’s general last-resort termination provision.

After verified exit, run the existing archive helper with a new destination:

```sh
python3 tools/archive-trial-logs.py --output PROOF/prelaunch-logs
```

`PROOF/prelaunch-logs` must not exist beforehand. Retain the helper’s metadata and exit status. Do not use this helper while the client is alive.  
Reference: `.agents/skills/verify-gfn-experiments/SKILL.md:20-20`; `.agents/skills/verify-gfn-experiments/SKILL.md:94-99`.

#### Gate B3 — preserve build-once semantics

The following state table is mandatory:

| Verified checkpoint state | Action |
| --- | --- |
| Never invoked; requested source, clean checkout, and no-client preconditions satisfied | Dispatch the existing guarded entry point once. Its build payload is `tools/build-native.sh`. |
| Existing invocation running or producing heartbeats/progress | Continue observing that invocation. Do not launch another, retry, cancel, replace, or clear its markers. |
| Existing invocation completed successfully | Verify its original receipt and artifact hashes; reuse that successful invocation. |
| Existing invocation failed | Preserve the original failure and stop for coordinator review. |
| Invocation state or ownership ambiguous | Record `build_once_state_unresolved`; do not dispatch another build. |

**Unresolved operational input:** The supplied context gives the guard’s behavior and checkpoint directory, but **does not name its executable entry point or exact CLI**. Resolve that from the existing checkpoint records and remote guides before dispatch. If it cannot be established, stop with `build_once_entrypoint_unavailable`. Running `tools/build-native.sh` directly would bypass the required guard and is not an approved substitute.

The successful build receipt must connect:

- Requested source HEAD and pre-build status.
- The single invocation and its process identity.
- Original stdout/stderr, heartbeat/progress records, and terminal exit status.
- Produced Qt, core, and FFI artifact paths and full SHA-256 values.

Hash equality is artifact identity; the invocation receipt supplies the source-to-build relationship. A Doctor result or file timestamp alone is insufficient.  
Supporting direct observation: `.agents/skills/verify-gfn-experiments/SKILL.md:59-61`.

### C. Relaunch and verify the measured client

Launch once through the existing Cua workflow. The intended application argument vector is:

```sh
env QT_QPA_PLATFORM=wayland /home/gamer/dev/gfn-client-research/tools/run-live-trial.sh baseline
```

Submit that launch through Cua’s existing `launch_app` facility; do not also run it from a separate SSH shell. The historical record documents this launch shape through Cua, and the supplied skill documents the baseline wrapper.  
**Direct observation:** `docs/nucbox-latency-baseline-20261008.md:21-37`; `.agents/skills/verify-gfn-experiments/SKILL.md:22-33`.

After launch:

1. Identify the actual Qt process and its core child; do not assume the returned wrapper PID is Qt.
2. Match the launched artifacts to the guarded-build receipt.
3. Record fresh PID/start ticks and launch receipts.
4. Confirm the normal signed-in library/resume view in the GUI.
5. Resume Ori through the normal GUI path.
6. Establish the stationary animated Spirit Well scene; close statistics and overlays before warmup.

**Plan decision — condition freeze:** Record the current profile and display state before A1. The supplied H.265/HDR, 2560×1080, approximately 60-source-FPS, Germany, Wayland/fullscreen, 100-Hz state is the expected prior condition, not a command to restore settings. Any difference must be declared before scoring and recorded as a distinct configuration. Use freshly verified dimensions and supported refresh in collector arguments. A condition rejected by the existing descriptive health checks remains blocked; do not change the checks or tune the client to force a pass.

Once A1 begins, the same session/profile/scene/display/window/overlay configuration is required for the remaining attempts. An intervening session allocation or material configuration change ends that repeatability block; earlier attempts remain retained and unmatched to the new state.

This preserves the feature’s current-state rule.  
Reference: `.agents/skills/verify-gfn-experiments/features/latency-baseline.md:5-16`.

**The 100-Hz distinction must appear in the ledger:** general Doctor plus the collector’s explicit descriptive checks can be evaluated at 100 Hz. The legacy `--require-stream` acceptance gate must not be reported as passed at that refresh or modified during this phase.  
**Direct observation:** `docs/nucbox-latency-baseline-20261008.md:15-15`; `scripts/nucbox-latency-baseline.py:120-120`; `scripts/nucbox-latency-baseline.py:149-179`.

### D. Run exactly three fresh scored invocations

Use attempt IDs `A1`, `A2`, and `A3`, each with windows `W1–W3`. Each attempt gets a distinct preflight label, measured label, proof directory, and observer output paths.

#### Per-attempt sequence

**1. Capture pre-attempt evidence.**

Record the GUI conditions and a before-scoring scene screenshot. Preserve current log metadata and any existing generations needed to detect rotation. Complete capability queries, source/artifact/tool hashing, and any small authorized input check before warmup.

**2. Run a unique dry-run.**

```sh
python3 tools/latency-baseline.py --pid VERIFIED_PID \
  --width VERIFIED_WIDTH --height VERIFIED_HEIGHT \
  --display-hz VERIFIED_HZ \
  --scene ori-inkwater-marsh-spirit-well \
  --label UNIQUE_PREFLIGHT_LABEL --dry-run
```

Require its recorded process identity, intended telemetry path, and explicit health checks to match the frozen attempt conditions.

The current implementation requires Qt telemetry within 3 seconds and native telemetry within 25 seconds during preflight. A dry-run produces `preflight-only`; it supplies no scored windows.  
**Direct observation:** `scripts/nucbox-latency-baseline.py:158-181`.

**3. Start bounded context observers.**

Use one socket observer and one network observer, with no router option:

```sh
python3 tools/live-socket-observer.py --pid VERIFIED_PID --duration 300 \
  --output PROOF/A1/sockets.json
```

```sh
python3 tools/live-network-observer.py --duration 300 \
  --output PROOF/A1/network.json
```

Substitute the appropriate attempt directory for A2/A3. Record each observer’s process identity, start/end times, exit status, and output path.

**Plan decision:** Use the same 300-second duration for all three attempts, subject to validating the existing helpers’ supported argument bounds before execution. The collector computes a 249-second observation deadline for the requested parameters, and performs work before observation; the feature’s 240-second example is therefore not a coverage guarantee. Actual timestamp overlap remains mandatory. If a helper does not support the planned duration, record the context gate and use no modified helper.  
Supporting observations: `scripts/nucbox-latency-baseline.py:140-150`; `.agents/skills/verify-gfn-experiments/features/latency-baseline.md:39-50`.

If an observer expires while scoring continues, any continuation must have a new bounded output path and cover only the remaining interval. Record gaps explicitly; do not claim seamless coverage from two fragments unless their timestamps establish it.

**4. Run the measured invocation.**

```sh
python3 tools/latency-baseline.py --pid VERIFIED_PID \
  --width VERIFIED_WIDTH --height VERIFIED_HEIGHT \
  --display-hz VERIFIED_HZ \
  --scene ori-inkwater-marsh-spirit-well \
  --label UNIQUE_MEASURED_LABEL \
  --warmup 20 --seconds 60 --windows 3
```

No screenshots, input, builds, tests, hashing, settings changes, or extra account operations occur within scored windows. Read-only observers continue through the interval.  
Reference: `.agents/skills/verify-gfn-experiments/SKILL.md:65-84`.

**5. Preserve results before the next attempt.**

Wait for the collector to exit, including its post-observation identity check. Then:

- Retain stdout/stderr and exit status.
- Retain every artifact actually produced.
- Capture the after-scoring scene screenshot.
- Take private read-only copies of the existing native diagnostic and Qt telemetry files, including required rotated generations.
- Record capture bounds and any rotation/truncation uncertainty.
- Wait for the attempt’s observers to finish; avoid overlapping a previous observer with the next attempt’s equivalent observer.
- Append the attempt disposition and available windows to the ledger.

Do not call the shutdown-only archive helper between live attempts. Do not truncate or rotate the live files yourself.

**Direct observation:** The collector rereads the native log after observation, rejects files larger than 3 MiB, and saves only selected native numeric fields. This makes retained source logs necessary for liveness analysis and rotation diagnosis.  
Reference: `scripts/nucbox-latency-baseline.py:63-82`; `scripts/nucbox-latency-baseline.py:183-204`.

**Plan decision:** A slow or failed A1 is not replaced by another A1. Continue to A2/A3 only when their existing preflight and same-condition gates can be satisfied. No fourth scored invocation is added to obtain a better result.

### E. Validity and exclusions

Keep these assessments separate:

| Assessment | Recorded values and rule |
| --- | --- |
| Collector status | Preserve the literal returned status and reasons. Never rewrite a collector result. |
| Attempt disposition | `PLANNED`, `PREFLIGHT_ONLY`, `BLOCKED`, `COMPLETE`, `PARTIAL`, `INVALID`, or `INTERRUPTED`. |
| Metric eligibility | Identify exactly which window/metric can be summarized and which cannot. |
| Context coverage | `FULL`, `PARTIAL`, or `UNAVAILABLE`, with measured overlap and missing intervals. |
| Performance finding | Record observed pacing/tails/stalls independently of validity. Slow performance is not an exclusion reason. |

Predeclared technical reasons for withholding a metric or same-condition comparison are:

- Process/source/artifact identity changed.
- Session, profile, scene, presentation generation, gating, or required workload state changed.
- Telemetry is missing, stale, malformed, discontinuous, or cannot be associated with the relevant observer.
- Counter reset/regression or histogram/census inconsistency prevents valid subtraction.
- Log rotation or missing boundaries prevents the requested interval from being reconstructed.
- The invocation was interrupted or did not produce the required duration/windows.

Apply each reason at the narrowest justified scope. For example, missing socket coverage with otherwise valid collector evidence limits network-context claims; it does not automatically invalidate Qt pacing.

**Plan decision:** If a freeze itself causes a freshness failure or early observer termination, preserve both facts: the quantitative window may be incomplete, and the failure remains relevant freeze evidence. Do not discard that attempt as unrepresentative.

A preflight failure may lack `observation.json` or numeric sample files; an interruption may lack `summary.json`. Record missing artifacts explicitly rather than manufacturing collector outputs.  
Supporting direct observation: `scripts/nucbox-latency-baseline.py:180-224`.

### F. Post-scoring liveness and variability analysis

#### Alignment and reconstruction

Use the retained collector outputs together with the existing raw diagnostic/Qt observations:

1. Preserve collector window boundaries and observer identity.
2. Match process/session block and all relevant epochs before comparing counters.
3. Compute deltas only from ordered, compatible samples. Do not bridge resets.
4. Use cumulative histogram differences for eligible Qt intervals.
5. Preserve native sampled/rolling semantics.
6. Record timestamp uncertainty and observer coverage beside each localization claim.

**Direct observation:** Native numeric summaries use `(start_ms, end_ms]`; the additional Qt census selection uses inclusive boundary samples. Preserve those definitions rather than silently shifting windows.  
Reference: `scripts/nucbox-latency-baseline.py:84-113`.

**Inference:** Native diagnostic wall timestamps and Qt telemetry timestamps support approximate alignment, but do not establish a common per-frame timeline. NVST’s elapsed counter helps distinguish receive progress from delayed log flushing; bounded asynchronous logging can still lose observations.  
Supporting observations: `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4560-4610`; `native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs:230-330`.

Do not subtract unrelated Rust `Instant` values and Qt monotonic values without an established clock mapping.

#### Stage-localization rules

| Observed relationship within compatible samples | Permitted inference |
| --- | --- |
| Authenticated receive advances while assembled-frame counts remain flat | Investigate assembly/recovery progress. Raw inbound datagrams alone are insufficient because the loop also handles non-media traffic. |
| Assembly advances while admission remains flat | Investigate delivery/admission or recovery gating; inspect undelivered, pending, and unmatched counts. |
| Decoder submissions advance while outputs remain flat in demonstrably updating reports | Decoder/output-path stall candidate; inspect recovery/backend/fence events. |
| Both submissions and outputs remain flat in repeated core telemetry | Decoder stall and stale monitor-forwarded telemetry remain distinguishable possibilities, not a resolved location. |
| Outputs advance while downstream frame notifications do not | Decoded-queue/publication/notification boundary candidate. Exact queue residence remains unavailable. |
| Notification enqueue advances while delivery does not | GUI drain/coalescing boundary candidate, subject to telemetry freshness. |
| Notification delivery advances while acquisition/record progress stops | Rendering/scheduling/generation boundary candidate. |
| Record success and draws advance while source swaps stop | Submit/swap callback boundary candidate; inspect gating, generation changes, and watchdog outcomes. |
| Required samples disappear | Location is unresolved; report the observation gap. |

These are **inferences**, grounded in the boundaries documented in Section 2. They are not claims that any such relationship has occurred in a fresh run.

#### Freeze reporting convention

**Plan decision:** Retain the existing strict `>25 ms` late count. Additionally report **source-callback gaps of at least 250 ms** as a descriptive long-stall category. This is an analysis threshold only; it changes no runtime timeout or acceptance gate.

Use:

- Completed fresh-source interval histogram buckets at or above 250 ms, including overflow counts.
- Increasing `sinceLastSourceSwapMs` with flat source-swap counts when an ongoing stall has not yet ended.
- Existing error/recovery events for supporting timing bounds.

A terminal stall need not produce a completed interval histogram entry, because `markSwap` records an interval only when another qualifying swap arrives.  
**Direct observation:** `opennow-qt/src/streaming/rendering/StreamPresentTimings.h:71-109`; `opennow-qt/src/streaming/rendering/NativeStreamRenderCallback.cpp:350-610`; `opennow-qt/src/streaming/StreamVideoItem.cpp:260-365`.

Report **“Long freeze: NOT REPRODUCED in the covered baseline intervals”** only when adequate coverage supports that statement. If coverage fails, use **“Freeze reproduction unresolved”** and identify the gap. A callback stall is not independently measured panel freeze duration.

#### Required performance summaries

For every eligible window, report:

- Actual duration, source/decoded/assembled FPS.
- Late count and rate.
- Qt interval and submit-to-swap p50/p95/p99/max, sample counts, and overflow.
- Notification-drain distributions.
- Native sampled network values and their sample counts.
- Native decode rolling-summary statistics, clearly labeled.
- Drop/replacement counts, epochs, and liveness findings.

For each attempt, merge compatible **histogram deltas**, not percentile values. Calculate aggregate late rate from total late intervals divided by total eligible interval samples. For all three attempts, show the individual results and their spread; do not pool away a poor attempt.

Native rolling p99 remains null. Assembly/admission/ACK quantiles remain null when absent from timestamped output. Qt overflow prevents an exact tail maximum.  
**Direct observation:** `.agents/skills/verify-gfn-experiments/features/latency-baseline.md:61-74`; `scripts/nucbox-latency-baseline.py:55-61`; `scripts/nucbox-latency-baseline.py:84-113`.

The repetition requirement is met only by three complete, compatible fresh invocations. Report whether that requirement was met and whether measured variability was small or substantial; do not invent a performance acceptance threshold.

### G. Candidate ranking for coordinator review

The following ranking is **provisional and unproven** until fresh measurements exist:

| Rank | Hypothesis | Evidence needed to strengthen it | Deferred scope |
| --- | --- | --- | --- |
| 1 | Synchronous Vulkan snapshot work contributes to decoder tails or stalls. | Decode/output stalls or unusually long calls aligned with existing snapshot/fence/backend errors. Decode call timing alone does not separate FFmpeg work, copy submission, locks, and fence waiting. | Review the smallest copy-path change only after localization. GPU completion, retained source ownership, snapshot reuse, and device-loss safety remain mandatory. |
| 2 | Default 2-ms publisher polling adds publication delay. | Decoder output progresses while downstream notification/publication progress lags, without evidence of a larger decoder or GUI stall. Current data may not resolve a 2-ms contribution. | Consider the existing polling experiment or a wake mechanism only in a later phase; do not activate either now. |
| 3 | Watchdog observation misses quiescent rendering/publication stalls. | A sustained downstream stall with upstream progress and absent `recordFrame`/watchdog observations. | Review observation scheduling and lifecycle behavior later. This is initially a detection/recovery hypothesis, not an explanation for ordinary pacing tails. |

**Supporting direct observations:**  
`native/opennow-streamer/crates/opennow-streamer-platform-linux/src/video/vulkan_copy.rs:190-445`; `native/opennow-streamer/crates/opennow-streamer-platform-linux/src/session.rs:650-750`; `native/opennow-streamer/crates/opennow-streamer-platform/src/media.rs:3190-3410`; `opennow-qt/src/streaming/rendering/NativeStreamRenderCallback.cpp:350-610`.

Reorder only when recorded evidence distinguishes candidates. If it does not, retain the provisional order and state that none was established.

An instrumentation request is justified only by a specific unresolved finding—for example, an observed decode-to-notification stall that existing reports cannot divide between decoder and publisher progress. Record the missing distinction and the smallest proposed measurement for later review; implement nothing at this checkpoint.

### H. Append-only scoreboard scaffold

The following is the inline content for the requested ledger. `PENDING` entries are initial records: subsequent results are appended at EOF and reference their attempt/window IDs. They are not filled in by overwriting the original rows.

```markdown
# NucBox latency and freeze run ledger

## Ledger contract

Append only. Preserve initial records, failures, partial attempts, outliers,
and corrections. Append corrections with the superseded record ID.
Slowness alone is never an exclusion reason.

This ledger contains sanitized metadata and private artifact paths.
No credentials, account identifiers, cloud-session identifiers, or raw payloads.

## Checkpoint input — desired build and operating scope

- Requested branch: nucbox-wayland-hdr
- Requested HEAD: 6472cae764a270864cd20446f6c1beafa1bfce8b
- Build checkpoint:
  /home/gamer/dev/gfn-client-research/.runtime/verification-deploy-tgLBRD4n
- Operator: PENDING
- Block label: PENDING
- Checkpoint start/end UTC: PENDING
- Verified host and checkout: PENDING
- Existing build-once entry point and receipt: PENDING
- Build invocation identity, terminal result, original logs: PENDING
- Qt/core/FFI artifact paths and SHA-256: PENDING
- Source-status and tool-hash receipts: PENDING
- Old/new Qt and core PID/start-tick identities: PENDING
- GUI shutdown, archive, launch, and cleanup receipts: PENDING

Expected prior condition, requiring fresh observation:
H.265, 10-bit 4:2:0 HDR, 2560x1080, approximately 60 source FPS;
Germany; Wayland fullscreen; 3440x1440 desktop at 100 Hz;
frame generation off; stationary animated Ori Spirit Well scene;
statistics and overlays closed.

Frozen observed configuration and deviations: PENDING
Session continuity uses safe labels only, such as "same-as-A1".
100-Hz descriptive validation is not a legacy acceptance-gate PASS.

## H0 — Historical, unmatched reference

Direct observation of the supplied historical report, not a fresh verification.

Source:
819ce3f37ff5315d0940f02df7272856d0d0f362

Scored interval:
2026-10-08 09:55:20.687–09:58:21.687 UTC

Evidence:
docs/nucbox-latency-baseline-20261008.md:8-17
docs/nucbox-latency-baseline-20261008.md:41-74

Historical private paths:
.runtime/latency-baseline-20261008-a1/
.runtime/latency-baseline-20261008-live-KwogFwiG/

| Window | Duration s | Fresh FPS | Decoded FPS | Interval p95/p99/max ms | Late >25 ms |
| --- | ---: | ---: | ---: | --- | --- |
| H0-W1 | 60.996 | 59.95 | 60.04 | 21/25/36 | 28; 0.77% |
| H0-W2 | 60.002 | 59.96 | 60.05 | 21/25/35 | 30; 0.83% |
| H0-W3 | 60.001 | 59.93 | 60.06 | 21/25/41 | 30; 0.83% |

Historical outcome: report states complete, 3/3 descriptive windows.
Current-build comparison eligibility: NONE; historical/unmatched.
No fresh repeatability or freeze conclusion follows from H0.

## A1 — Initial planned record

Preflight label/path: PENDING
Measured label/path: PENDING
Proof and observer paths: PENDING
Initial disposition: PLANNED

| Window | UTC start/end | Duration | Validity | Metrics/liveness record |
| --- | --- | --- | --- | --- |
| A1-W1 | PENDING | PENDING | PENDING | PENDING |
| A1-W2 | PENDING | PENDING | PENDING | PENDING |
| A1-W3 | PENDING | PENDING | PENDING | PENDING |

## A2 — Initial planned record

Preflight label/path: PENDING
Measured label/path: PENDING
Proof and observer paths: PENDING
Initial disposition: PLANNED

| Window | UTC start/end | Duration | Validity | Metrics/liveness record |
| --- | --- | --- | --- | --- |
| A2-W1 | PENDING | PENDING | PENDING | PENDING |
| A2-W2 | PENDING | PENDING | PENDING | PENDING |
| A2-W3 | PENDING | PENDING | PENDING | PENDING |

## A3 — Initial planned record

Preflight label/path: PENDING
Measured label/path: PENDING
Proof and observer paths: PENDING
Initial disposition: PLANNED

| Window | UTC start/end | Duration | Validity | Metrics/liveness record |
| --- | --- | --- | --- | --- |
| A3-W1 | PENDING | PENDING | PENDING | PENDING |
| A3-W2 | PENDING | PENDING | PENDING | PENDING |
| A3-W3 | PENDING | PENDING | PENDING | PENDING |

## Required fields in subsequent append records

- Record ID, UTC, operator, attempt/window reference.
- Exact command, exit status, literal collector status and reasons.
- Source/build/tool hashes and original receipt paths.
- Qt/core identities and observer identities.
- Observed profile, scene, display, overlay, and session continuity.
- Collector artifact paths and explicitly missing artifacts.
- Socket/network paths, actual overlap, gaps, and exit status.
- Raw diagnostic/Qt snapshot paths and capture/rotation bounds.
- Duration; source/decoded/assembled FPS; late count and denominator.
- Qt p50/p95/p99/max, samples, overflow, epochs, and gating.
- Native sampled network and rolling decode summaries with semantics.
- Drop/replacement counts and stage-liveness counter deltas.
- Observed stall bounds; inference; supporting source/sample references.
- Metric eligibility and every exclusion with reason.
- Freeform anomalies, including slow or failed observations.

## Checkpoint adjudication — initial record

Fresh completed attempts/windows: PENDING
Same-condition repetition requirement met: PENDING
Across-attempt and across-window variability: PENDING
Freeze reproduction and coverage: NOT ASSESSED
Stage localization: NOT ASSESSED
Technical exclusions and missing coverage: NOT EVALUATED
Ranked candidates and supporting evidence: PENDING
Deferred observability requests: PENDING
Cleanup and remaining owned processes: PENDING
Coordinator decision: PENDING

## Chronological append records

Append all subsequent events below this heading.
```

### I. Cleanup and checkpoint handoff

After the last feasible attempt, capture final GUI evidence outside scoring. The measured client was created by this workflow, so complete its normal GUI cleanup and verify the exact Qt/core instances exited. The supplied skill makes normal closure the default for a workflow-created client. The checkpoint still forbids forced termination.  
Reference: `.agents/skills/verify-gfn-experiments/SKILL.md:86-92`.

After verified exit, archive the final log generations to a new destination:

```sh
python3 tools/archive-trial-logs.py --output PROOF/final-logs
```

Retain all observer outputs, failures, build receipts, and collector directories. If normal cleanup fails, record the remaining exact process identities and the unresolved cleanup gate; do not kill them.

The coordinator handoff must include:

- Exact executed commands and receipt/private artifact paths.
- Build-once disposition and source/artifact correspondence.
- Every attempted preflight and scored invocation.
- Valid, partial, invalid, interrupted, and missing windows.
- Observer coverage and every metric exclusion.
- Repetition completion and measured variability.
- Freeze reproduction status with coverage limits.
- Stage observations separated from inferences.
- Evidence-based candidate ranking and deferred measurement gaps.
- Cleanup outcome and any remaining owned processes.

Stop there. A candidate change or additional scored block requires the subsequent coordinator decision.

## 4. File-by-file impact

| File or artifact | Change | Dependency and ordering |
| --- | --- | --- |
| `prompt-exports/optimize-nucbox-latency-freezes-runs.md` | Create the scaffold; append operating events, results, exclusions, analysis, corrections, and checkpoint disposition. | Create locally before scoring. Never replace an existing ledger or edit earlier records. |
| Existing remote `.runtime/verification-deploy-tgLBRD4n/` | Let the existing build-once mechanism retain its own invocation state, logs, and receipts. Add ledger references to their actual names. | Requires verified source and completed GUI shutdown. Do not invent receipt filenames or reset existing state. |
| New remote `.runtime/nucbox-baseline-BLOCK/` | Store operator proof, Cua screenshots, observer files, exact command/exit receipts, and private diagnostic snapshots. | Create privately before live actions; capture heavy evidence outside scoring. |
| Collector-created `.runtime/latency-baseline-LABEL/` directories | Existing collector creates summaries, observation data, and numeric samples when those stages are reached. | One unused label per invocation; preserve incomplete directories and missing-artifact information. |

The supplied `.rs`, `.cpp`, `.h`, collector, skill, and historical-report files are evidence inputs for this checkpoint. No edits to them are part of the plan. In particular, the existing `AGENTS.md` modification and unrelated Oracle-plan file must remain preserved.

The unprovided build-guard entry point, remote guide contents, and observer argument bounds are **verification gates**, not invitations to implement replacement helpers.

## 6. Implementation order

1. **Create the local ledger scaffold and assign the sole operator.** Record requested identity, block label, planned attempts, and evidence destinations.
2. **Read the current remote guides and verify the checkpoint.** Resolve the existing guarded-build entry point before dispatch; record any blocking mismatch.
3. **Complete GUI shutdown and archive prior logs.** Process identity verification and archive eligibility must be satisfied together before building.
4. **Run or continue the existing build-once invocation.** Its source/precondition checks, exclusive invocation marking, and dispatch form one guarded operation; never split or bypass them.
5. **Verify build receipts and relaunch once through Cua.** Establish fresh process/artifact correspondence, saved-account readiness, scene, and conditions.
6. **Execute A1, A2, and A3 sequentially where feasible.** Each attempt contains a unique preflight, bounded observers, three scored windows, evidence preservation, and an append-only disposition.
7. **Perform normal cleanup and final archival.** Preserve any unresolved process or evidence state explicitly.
8. **Analyze retained data and append adjudication.** Report variability, stage-liveness findings, freeze status, limitations, and the smallest supported next candidate.
9. **Stop for coordinator review.** At this response’s current state, all live execution gates and fresh results remain pending.

End of Oracle group: 2 lanes above.



> 💡 The 2 Oracle lanes above are independent answers; reconcile them using the guidance at the top of the group. Lane order is not a ranking.

Optional later follow-up: Continue this plan conversation with ask_oracle(chat_id: "nucbox-latency-baseline--228BC4", new_chat: false)