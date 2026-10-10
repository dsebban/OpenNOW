## Final Prompt
<taskname="NucBox Freeze Diagnosis Plan"/>
<task>Choose and specify exactly one next bounded step in the NucBox OpenNOW latency/freeze loop. A2 reproduced a terminal freeze and invalidated the requested three-attempt repeatability gate; do not optimize or claim a reliable latency comparison. The next step must use retained private evidence and non-mutating metadata observers, if sufficient, to distinguish freeze stage/cause and unlock one later attributable root-cause iteration. Do not implement now. Produce an executable plan with the metric, evidence gates, exact supported probes/commands (or explicitly say no new host probe is currently justified), writable scope, tests only if instrumentation proves indispensable, stop/continue decision, and append-only scoreboard entry. Preserve user constraints: ChimeraOS foreground Cua is authorized; no push/publish, network/display/profile changes, vendor source edits, unrelated or pending AGENTS.md work, and no raw captures/logs/credentials in public context. Report measured component values accurately; never call them physical input-to-photon latency.</task>

<architecture>OpenNOW streams through a Qt/QML client and embedded Rust NVST streamer. The receive worker in the transport crate tracks inbound datagrams and authenticates packets; `NvstVideoReceiver::poll_timeout` is based on the last authenticated packet and emits typed timeout/recovery events. The core event worker forwards these events and allows a single recovery attempt before terminal stop. Protocol logging summarizes/filters diagnostic values. Existing experiment tooling provides descriptive latency collection and read-only socket/network observations; those observers do not expose packet age or prove cloud-path health.</architecture>

<selected_context>
prompt-exports/optimize-nucbox-latency-freezes-runs.md: Authoritative append-only experiment ledger, full chronology, A1/A2 data, evidence pointers, constraints, one-time build/runtime identity, private evidence locations, and Cua ownership handoff. Do not rewrite history or expose raw/private evidence.
.agents/skills/verify-gfn-experiments/SKILL.md: Host workflow rules, one GUI/account owner, Doctor/launch/evidence behavior, observer capabilities, privacy and cleanup boundaries.
.agents/skills/verify-gfn-experiments/features/latency-baseline.md: Exact supported descriptive collector and observer invocations and interpretation limits.
scripts/nucbox-latency-baseline.py: Existing safe bounded descriptive baseline entry point (already in workspace context from initial selection); does not launch, drive, or alter the client.
native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs (selected slices): Authenticated receive timeout semantics and production video UDP receive loop, including inbound/source/auth processing, periodic counters, and timeout forwarding.
native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs (selected slices): Existing NVST event-worker behavior, event forwarding, one-attempt recovery, terminal stop.
native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs (selected slice): Diagnostic message summarization/filter seam relevant to sanitized retained-event analysis.</selected_context>

<relationships>
- NucBox UDP receive loop → authenticated NVST receiver → typed `RecoveryNeeded`/timeout event → core event forwarding → one recovery attempt → terminal stop/status.
- Existing `native-streamer.log` summaries are about ten seconds apart; inbound/authenticated/assembled counters and timestamps may help locate the stall but do not prove remote cloud state or packet-path cause.
- `latency-baseline.py` and the skill feature map describe the matched 100-Hz collector; the socket/network observers overlap a run but cannot establish remote UDP path health.
- A1 completed three descriptive windows (fresh interval aggregate p95/p99/max 21/24/56 ms; 88 late intervals of 10,915). A2 is INVALID with one completed window plus a partial freeze: fresh-gap completed-window p95/p99/max 20/24/300 ms; then roughly 16 s no fresh frames before automatic teardown. The 300-ms histogram does not bound the later open-ended stall.
- At A2 freeze, native receive/auth/assembly and media/decode output counters plateaued while Qt acquire-empty/swaps and outbound native pings/logging continued; last timing fields were stale, so this points upstream of Qt but does not identify cloud/session/path/kernel/receive-processing cause.
- A2 first had a recovered ~299.113-ms gap associated with a compressed-video-discontinuity-or-overflow event, keyframe recovery and 14 compressed queue drops. Initiating cause is unproven; keep this separate from terminal freeze.
- Existing private sanitized retained evidence is at `/private/tmp/nucbox-baseline-Sth9bYSs/`; full raw proof stays remote at the private path recorded in the ledger. No need to copy raw captures/logs into prompt or Git.
- GUI ownership was released to the coordinator at 12:48:31 UTC. Existing client identities are historical facts only; do not infer current GUI readiness from a sleeping process state.</relationships>

<ambiguities>
The existing evidence cannot distinguish remote cloud/session termination from UDP path loss or receive-loop processing failure: the receive loop has aggregate inbound/authenticated counters but no packet-age/capture or durable per-stage correlation at incident time. Sampled gateway and kernel socket health do not prove the remote UDP path was healthy. Resolve first by inspecting existing private sanitized endpoint/terminal/receive evidence and source-defined counter semantics; do not add telemetry or start another stream by default. If that analysis still cannot discriminate and an additional incident is necessary, require a fresh ownership/readiness gate and propose only the smallest debug-gated receive-owner observation; spell out its local tests and evidence scope before any product edit. Leave the scoreboard incomplete and stop if the private evidence is insufficient or no authorized live session exists.</ambiguities>

## Selection
- Files: 6 total (3 full, 3 slice)
- Total tokens: 19461 (Auto view)
- Token breakdown: full 11152, slice 8309
- Token accounting: fresh from active_tab_published

### Files
### Selected Files
├── .agents/
│   └── skills/
│       └── verify-gfn-experiments/
│           ├── features/
│           │   └── latency-baseline.md — 1,133 tokens (full)
│           └── SKILL.md — 3,203 tokens (full)
├── native/
│   └── opennow-streamer/
│       └── crates/
│           ├── opennow-streamer-core/
│           │   └── src/
│           │       └── lib.rs — 2,960 tokens (lines 1730-1810 (NVST event worker setup and existing liveness/telemetry flow around per-session receiver events.), 2060-2300 (Typed receive event forwarding, one-shot recovery limit, and terminal event emission semantics, used to interpret automatic teardown.))
│           ├── opennow-streamer-protocol/
│           │   └── src/
│           │       └── log.rs — 964 tokens (lines 160-260 (Diagnostic event allowlist and message sanitization/summary behavior, relevant to safe reuse of retained logs without leaking endpoint/session secrets.))
│           └── opennow-streamer-transport/
│               └── src/
│                   └── nvst.rs — 4,385 tokens (lines 4230-4290 (Authenticated receive idle timeout state and recovery gating; timeout semantics used to attribute terminal freeze.), 4360-4420 (NVST packet authentication/acceptance updates, needed to interpret last-authenticated timestamp.), 7380-7640 (Production video UDP receive loop: inbound counters, source/auth filtering, assembly processing, ten-second telemetry and timeout forwarding; primary existing observation seam.))
└── prompt-exports/
    └── optimize-nucbox-latency-freezes-runs.md — 6,816 tokens (full)


---

## Generated Plan
- Oracle group status: failed
- Oracle group: `EED14EDD-630B-4D76-9D91-F943C64CDB6C`

**Reconciling these Oracle lanes**
2 independent answers to the same request follow. Lane order is not a ranking; the first lane supplies the top-level continuation handle, and a successful follow-up through any lane's chat ID re-runs every lane.
- Read every lane through the end-of-group marker (`End of Oracle group: 2 lanes above.`). If the marker or a lane is missing, page the export or try the read-only `oracle_chat_log` with that lane's chat ID. Logs may be scoped or clipped; report any remaining gap. Do not start a follow-up just to retrieve prior text.
- Reconcile by evidence, not lane order, answer length, or model identity: check material single-lane and conflicting claims against the code, and report unresolved disagreements. A failed or partial lane is incomplete evidence.
- Before synthesizing, inventory every material claim from every lane, including single-lane claims. Merge only exact duplicates, retaining all source lanes. Begin your answer with `**Oracle reconciliation**`, state how many lanes completed and name any that did not. For every inventory item, give its source lanes, checked evidence, and exactly one disposition: `accepted`, `rejected`, or `unresolved`. Never silently omit an item.

Lanes (2):
- Oracle — `claude-opus-5-5-xhigh` — Failed — chat ID `nucbox-latency-and-freez-F7454F`
- Oracle 2 — `gpt-6-astra-pro` — Completed — chat ID `nucbox-latency-and-freez-4A7D70`

### Oracle
- Status: Failed
- Provider: `Custom`
- Model: `claude-opus-5-5-xhigh`
- Effective effort: Provider default / not specified
- Chat: `nucbox-latency-and-freez-F7454F`

Error [context_builder_inactivity_timeout]: Context Builder Oracle exceeded its 600s inactivity budget during streaming.

### Oracle 2
- Status: Completed
- Provider: `Custom`
- Model: `gpt-6-astra-pro`
- Effective effort: Provider default / not specified
- Chat: `nucbox-latency-and-freez-4A7D70`



## Summary

Choose exactly one next step: **D1 — a bounded review of the retained A2 evidence to identify the last demonstrated point of progress and determine whether any existing record distinguishes the remaining causes.** Reconcile the private receive, terminal, Qt, and observer records against the supplied timeout and recovery code. **No new host probe is currently justified**, and this step introduces no instrumentation, stream restart, optimization, or latency comparison. The output is one private adjudication report and one append-only ledger entry. A specific discriminator can support one later proposal; otherwise, record the missing evidence and stop. **Inference supporting this choice:** A2 establishes a real freeze upstream of Qt’s continued activity, but the supplied record does not distinguish cloud/session termination, UDP path loss, and local receive delivery or processing failure. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:72-86`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:98-113`)

**Evidence boundary:** I have not accessed the private artifacts. Below, **direct observation** means a fact present in the supplied ledger or source excerpts; **inference** means a conclusion drawn from those cited observations. D1’s result remains unexecuted.

## Current-state analysis

### What the measurements establish

**Direct observation — ledger:** The retained results are:

| Record | Reported measurement | Interpretation |
|---|---|---|
| A1, three completed windows combined | Fresh-interval histogram upper estimates: p95 **21 ms**, p99 **24 ms**, max **56 ms**; **88 / 10,915** intervals strictly over 25 ms, or **0.806230%** | Descriptive completed-gap baseline for this attempt. |
| A2, completed first window | Fresh-interval histogram upper estimates: p95 **20 ms**, p99 **24 ms**, max **300 ms**; **35 / 3,583** over 25 ms, or **0.976835%** | Retained completed window within an invalid attempt. |
| A2, partial second window | Approximately **16 seconds without a fresh source frame**, ending in automatic teardown | The unclosed freeze is not bounded by the earlier 300-ms histogram maximum. |
| Earlier recovered A2 gap | **299.112574 ms**, associated with a compressed-video-discontinuity-or-overflow diagnostic, keyframe recovery, and **14 compressed-video drops** | A separate incident. Overflow alone is not an established initiating cause. |

The first three rows come from `prompt-exports/optimize-nucbox-latency-freezes-runs.md:60-78`; the final correction distinguishing discontinuity from overflow is at `prompt-exports/optimize-nucbox-latency-freezes-runs.md:107-109`.

**Direct observation — measurement contract:** These are component observations. Qt callbacks do not establish physical presentation or input-to-photon latency. Native decode “p95” summaries represent sampled, overlapping rolling distributions: the ledger’s reported values are medians of sampled rolling-p95 summaries, not scored-window per-frame percentiles. (`.agents/skills/verify-gfn-experiments/features/latency-baseline.md:61-74`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:103-104`)

### Receive ownership and transformation boundaries

**Direct observation — source:** The production video UDP worker owns its local `NvstVideoReceiver`, datagram buffer, receive-loop counters, ping tracking, and command processing. Each loop handles commands, potentially sends a ping, calls `socket.recv_from`, processes the result, emits periodic statistics, and polls timeout state. `inbound_datagrams` increments immediately after a successful `recv_from`, before source classification, STUN handling, and media processing. `pings_sent` increments after a successful local `send_to`; it is not an acknowledgement from the remote peer. (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7380-7640`)

**Direct observation — source:** Successful raw receipt and accepted NVST packet progress are different boundaries:

1. The worker classifies the source and handles applicable STUN traffic.
2. `NvstVideoReceiver::process_datagram` invokes `srtp.unprotect`.
3. Unprotect failures, unexpected payload types, and unexpected SSRCs return before the accepted-packet timestamp update.
4. Only after those checks does the receiver update `last_authenticated_packet`, clear `initial_timeout_pending`, update frame-progress authentication state, and increment `authenticated_packets`.
5. Subsequent video-payload interpretation and frame processing remain separate from that accepted-packet update.

Therefore, an accepted-packet increment does not itself prove that a complete video frame was assembled. (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4360-4420`, `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7380-7640`)

**Direct observation — source:** `poll_timeout(now)` operates only while the receiver is `Running`. Its reference is `last_authenticated_packet`, falling back to `timeout_origin`; it selects `startup_timeout` or `timeout` according to `initial_timeout_pending`. On expiration it resets media state, enters `RecoveryRequired`, and returns one typed timeout event. `recover()` subsequently clears `last_authenticated_packet`, establishes a new `timeout_origin`, disables the startup timeout, and returns the receiver to `Running`. (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4230-4290`)

**Inference:** The second timeout must be interpreted within the recovery interval. Its reported idle duration cannot automatically be treated as time since the original pre-freeze packet. Likewise, a `Running` lifecycle event after recovery proves a control-state transition, not restored packet or frame progress. These conclusions follow from the reset behavior above and the core’s handling of `Running`. (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4230-4290`, `native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2060-2300`)

### Core recovery and termination

**Direct observation — source:** `forward_nvst_session_events` manages session event processing and media feedback under a lifecycle generation. `forward_nvst_event` rejects obsolete generations. Generic recovery events reach `attempt_nvst_recovery`, which allows the existing bounded attempt before calling `emit_nvst_terminal`. Terminal handling clears lifecycle context, sets `State::Idle`, stops resources, and emits both error and stopped-status events with `source: "nvst-transport"` and `resumable: null`. (`native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:1730-1810`, `native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2060-2300`)

**Direct observation — source:** `PacketGap` and `MediaConsumerBackpressured` have separate keyframe-request handling that does not spend the terminal transport-recovery budget in those branches. **Inference:** Do not count every keyframe request or use of “recovery” as the bounded timeout-recovery attempt, including the earlier recovered 299-ms incident. (`native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2060-2300`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:74-76`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:109-109`)

### Existing observation limits

**Direct observation — ledger:** During the terminal incident, reported inbound/authenticated/assembled and media/decode counters plateaued while Qt acquire-empty/swaps and native outbound pings/logging continued. The first timeout was reported at **12:36:15.487 UTC**; the second at **12:36:23.506 UTC**, followed by recovery exhaustion at **12:36:23.507 UTC**. The second timeout reported `idle_for: 8.008563384s`. Existing socket/network observations overlapped the incident but did not establish cloud-path health. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:75-77`)

**Inference:** Once identity, counter continuity, and diagnostic freshness are corroborated, flat raw-inbound counters alongside new worker activity can establish that the worker observed no additional successful raw receives over the corresponding sampled interval. That narrows the failure boundary, but does not locate missing datagrams between the remote sender and local socket delivery. (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7380-7640`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:76-77`)

**Direct observation — source:** `message_summary` deliberately retains selected discriminators and numeric fields while excluding arbitrary messages and other content. Its keyframe reason mapping preserves the combined code `compressed-video-discontinuity-or-overflow`. **Inference:** Absence of a remote termination explanation from a sanitized summary does not prove that no such explanation existed in the original record. Also, the existing `nvst-video` log calls explicitly include endpoint fields, so the summary allowlist must not be treated as a sanitizer for every native log line. (`native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs:160-260`, `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7380-7640`)

## Design

### 1. D1 scope, inputs, and bounds

D1 is an **evidence-analysis operation**, not a runtime component or refactor. It adds no Rust types, interfaces, protocol fields, Qt state, configuration, or application persistence.

**Inference and decision:** A targeted retained-evidence review is appropriate because the observed terminal failure and the missing causal discriminator are already identified; changing a subsystem now would introduce a candidate before establishing what it should fix. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:98-109`)

Use these existing local private inputs:

- `/private/tmp/nucbox-baseline-Sth9bYSs/stage-analysis.json`
- `/private/tmp/nucbox-baseline-Sth9bYSs/sanitized-native-events.json`
- `/private/tmp/nucbox-baseline-Sth9bYSs/build-receipt.json`
- `/private/tmp/nucbox-baseline-Sth9bYSs/checkpoint-identity.json`
- The associated retained A2 summary, observation, Qt census, native numeric, and observer numeric exports.

**Direct observation — ledger:** These exports, including source-file/line/time pointers and the bounded diagnostic-event export, are recorded as available. Their actual contents are not supplied here. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:85-86`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:101-105`)

Apply these fixed bounds:

| Boundary | Required scope |
|---|---|
| Attempt | A2 only; A1 receipts may corroborate shared build/process provenance. |
| Available incident context | **12:34:00–12:39:00 UTC on 2026-10-08**, corresponding to the retained A2 observer block. |
| Primary terminal analysis | **12:35:50–12:36:30 UTC**, plus the nearest predecessor/successor records already in the allowed exports when needed to bound progress. |
| Separate short-gap context | **12:34:24–12:34:31 UTC**; retain as a separate incident, without selecting a fix for it. |
| Additional retained-source inspection | Only original records referenced by the supplied private exports, read in place if necessary to resolve a discriminator lost in summarization. No broader log search or new capture. |
| Completion boundary | One report and one final ledger append, followed by handoff. |

The retained observer timing and short/terminal incident times supporting these bounds are direct observations in `prompt-exports/optimize-nucbox-latency-freezes-runs.md:74-77` and `prompt-exports/optimize-nucbox-latency-freezes-runs.md:95-95`.

**Unknown to validate:** The associated A2 export filenames and JSON layouts are not shown. Resolve them from the existing directory and provenance references; document the mapping in the private report. Do not invent filenames, field meanings, or units. A missing required mapping produces `BLOCKED_EVIDENCE`.

### 2. Primary metric and report contract

The primary metric is **the furthest processing boundary with demonstrated progress during the terminal incident**, accompanied by progress-time bounds and evidence coverage. It is not a latency percentile or a new performance score.

For every available stage, record:

| Report field | Required interpretation |
|---|---|
| `stage` | Raw video receive, source/STUN classification, accepted NVST packet, assembly, media admission, decode submission/output, or Qt fresh source. |
| `counter_owner` | The actual producing component and counter name; keep transport-local, shared-feedback, media, and Qt counters separate. |
| `epoch` | Applicable historical process identity, session/component generation, and any observed reset boundary. |
| `progress_delta` | Difference between comparable cumulative samples within that epoch; otherwise unknown. |
| `last_progress_bounds` | Source-supported interval for the final increment, or unknown. |
| `observation_time_basis` | Monotonic event time, timestamped diagnostic/sample time, or logging time with unresolved delay. |
| `coverage` | Fresh independent observations, duplicated/forwarded observations, missing records, or uncertain semantics. |
| `evidence_pointer` | Private source artifact and original line/event pointer. |

These are fields in the human-readable report, not a new application serialization schema.

**Inference and decision:** Do not subtract assembler, ACK, decode, notification, and Qt-source values from one another to calculate “lost frames.” They describe different boundaries, and the ledger explicitly lacks durable per-frame cross-stage correlation. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:75-76`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:85-85`)

### 3. Analysis procedure

Execute the following procedure once.

1. **Validate retained provenance.**  
   Confirm that the selected records belong to A2 and the build/process identities already recorded in the receipts. Use retained receipts; do not rebuild, rehash deployed artifacts, or infer current state from old identities. Preserve any contradiction as evidence.

2. **Build a private observation table.**  
   Preserve original timestamps, owner identities, counter names, source pointers, and reset/lifecycle events. Deduplicate repeated references to the same original record for analysis only; preserve every original file. Distinct records containing the same counter values remain distinct observations.

3. **Validate counter semantics before calculating deltas.**  
   Keep the worker’s `inbound_datagrams` separate from `feedback.record_socket_receive` metrics. The worker can call `record_socket_receive` both before and after processing when no authenticated packet was previously recorded; `authenticated_before` is a historical-state test, not the authentication result of the current datagram. **Direct source observation:** these calls and conditions appear in the receive loop. (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7380-7640`)  
   
   The supplied excerpts do not include the complete implementations of `stats_line`, `reset_media_state`, or `NvstSessionResources::recover`. Validate only the semantics needed for D1 against a locally available source snapshot matching the recorded revision. If unavailable, mark affected reset behavior or command-application timing unknown; do not fetch, checkout, build, or assume it.

4. **Calculate progress bounds within each valid epoch.**  
   For a cumulative counter’s terminal plateau, identify its first sample at the final plateau value and its nearest earlier sample with a lower value. The last increment is bounded between those observations only to the precision of their measurement timestamps. If either endpoint is missing, retain an open bound.  
   
   A counter decrease or generation change ends the comparable interval. A dropped observation is missing coverage, not a zero delta. Repeated forwarding of an old timing value does not create a fresh timing measurement.

5. **Reconstruct the timeout/recovery sequence separately from frame age.**  
   Identify the first timeout, core recovery request, any evidence of worker-side recovery application, subsequent accepted-packet progress, second timeout, and terminal events. Preserve unknown application times.  
   
   **Direct source observation:** the worker calls `poll_timeout` before formatting its timeout diagnostic, and `poll_timeout` resets media state. **Inference:** fields in that diagnostic may reflect post-transition state; they must not automatically be used as a pre-timeout snapshot. (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4230-4290`, `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7380-7640`)

6. **Correlate Qt and the existing observers without upgrading their meaning.**  
   Treat the approximately **12:36:07.494 UTC** last-source boundary as a Qt-age-derived estimate, not an observed network arrival. End the active-stream freeze accounting at automatic teardown/gating; later increasing source age does not extend streaming-freeze duration. These distinctions are direct observations and limitations recorded in the ledger. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:75-76`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:85-85`)  
   
   Preserve socket/gateway sampling gaps and counter resets. Do not turn sampled low receive-queue occupancy, zero sampled socket drops, or successful gateway probes into proof of remote UDP delivery. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:77-77`)

7. **Check for an independent causal discriminator.**  
   Examine existing session/terminal records and any necessary source-referenced original records for evidence whose origin and ordering are actually established. A remote event must correlate privately to the same allocation, have a understood event meaning, and precede the receive-loss boundary with sufficient ordering evidence to support a causal claim. An overlapping time bracket establishes association only.  
   
   **Inference:** The local `nvst-recovery-exhausted` event is evidence of the client’s timeout decision, not independent evidence that the cloud terminated the session; its emitting code explicitly labels the source `nvst-transport`. (`native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2060-2300`)

Use source-local monotonic/event ordering where available. Do not subtract unrelated process `Instant` values or resolve cross-component ordering more precisely than the retained clocks and logging behavior permit.

### 4. Attribution and stop/continue rules

The following are **inference rules for D1**, not claims that their conditions have already been verified. They follow the receive/acceptance/assembly boundaries in the source and the evidence limitations recorded for A2. (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4360-4420`, `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7380-7640`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:75-85`)

| Verified evidence pattern | Permitted conclusion |
|---|---|
| Raw inbound remains flat while fresh records show the same worker continuing activity | Receive delivery silence over the observed interval. Remote sender, path, and local delivery cause remain unresolved. |
| Raw inbound advances, but the available traffic is wrong-source or handled STUN | Datagrams reached the worker; qualifying media progress is unproven. This alone is not evidence of SRTP failure. |
| Qualifying media arrivals and specific rejection evidence advance while accepted-packet progress stops | Narrow the boundary to acceptance/rejection, using the actual observed reason. |
| Accepted-packet progress continues while assembly stops | Narrow the boundary to frame production after acceptance; do not attribute it to Qt. |
| Assembly/admission/decode progress continues while Qt fresh-source progress stops | A downstream boundary requires investigation; this would need reconciliation with the recorded terminal plateau before selecting a candidate. |
| An independently attributable session/remote event causally precedes receive cessation | A session/remote-cause proposal may be supported; local timeout events do not satisfy this condition. |

End D1 with exactly one report verdict:

- **`BLOCKED_EVIDENCE`** — Required identity, provenance, counter semantics, or coverage cannot be validated. Record the missing item and stop.
- **`STAGE_ONLY`** — The stage is supported, but no retained observation distinguishes the remaining causes. Record the competing explanations and exact missing discriminator; stop without choosing a product change.
- **`DISCRIMINATOR_FOUND`** — Positive, attributable evidence distinguishes one specific mechanism sufficiently to formulate one later investigation or correction. Record that single mechanism, its owning boundary, the predicted changed observable, and what could falsify it. Return the proposal to the coordinator; do not execute it in D1.

**Inference from the frozen record:** `STAGE_ONLY` is the presently supported expectation. D1 must earn a stronger conclusion through additional retained evidence, not through the confidence of the write-up. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:98-109`)

Every verdict leaves the three-attempt/nine-window repeatability gate **INCOMPLETE** and A2 **INVALID but retained**. Those statuses are direct ledger observations, not conditions D1 can repair. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:80-83`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:111-113`)

### 5. Host probes, ownership, and writable scope

**No new host probe is currently justified.** Reuse the completed socket/network observer outputs. Do not rerun Doctor, the descriptive collector, or observers against historical PIDs: present idle-client metadata cannot reconstruct incident-time packet arrivals.

**Direct observation supporting that decision:** The observers completed and the stream automatically stopped; GUI ownership was subsequently released to the coordinator. The handoff explicitly limits the old process-state check and does not assert fresh GUI readiness. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:83-83`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:111-113`)

Writable scope for the future execution of D1 is exactly:

1. **Create:** `/private/tmp/nucbox-baseline-Sth9bYSs/D1-freeze-attribution.md`, using exclusive creation.
2. **Append only:** `prompt-exports/optimize-nucbox-latency-freezes-runs.md`.

Preserve all existing private exports and original receipts. If the report already exists, stop on the collision rather than overwrite it. If required inputs are unavailable, record `BLOCKED_EVIDENCE`; do not manufacture replacement observations.

The private report may contain existing provenance pointers. The ledger append contains only sanitized times, numeric values, fixed diagnostic codes, verdicts, and limitations. Do not reproduce raw lines, endpoint values, credentials, session identifiers, or free-form server messages.

No source, skill, feature-map, collector, build, network, display, profile, or `AGENTS.md` changes belong to D1. No commit, push, or publication is included. Foreground Cua authorization remains available for a later authorized workflow; D1 performs no GUI or account operation.

### 6. Conditional instrumentation boundary

**No instrumentation or tests are authorized by this plan.** First complete the retained-evidence review.

If D1 finds a missing observable, its report must state which competing hypotheses would produce different readings from that observable. If all remaining hypotheses predict “no datagrams delivered while the loop continues,” additional receive-owner timestamps alone cannot distinguish remote sender silence from off-host path loss; instrumentation is not justified by that ambiguity.

Only if D1 demonstrates a distinguishable receive-owner gap should a later proposal consider a default-off observation at the existing receive worker: bounded aggregate loop progress, ages since raw/expected-source/accepted-packet progress, rejection categories, and timeout-origin transitions before reset. It must preserve timeout, recovery, queue, and event behavior.

Before any such later edit or deployment, require:

- Fresh coordinator ownership and GUI/account readiness; historical PIDs are insufficient.
- Local deterministic tests for raw/STUN/rejected versus accepted progress, timeout/reset origins, one-shot timeout behavior, disabled-mode equivalence, bounded emission, and redaction.
- A reviewed local change and an authorized deployment route.

**Direct observation supporting these boundaries:** The supplied workflow requires one GUI/account owner, the ledger restricts deployment and vendor edits, and the receiver code defines the state changes the conditional tests would need to preserve. (`.agents/skills/verify-gfn-experiments/SKILL.md:8-20`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:96-96`, `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4230-4290`)

## File-by-file impact

| File | Planned change | Reason and dependency |
|---|---|---|
| `/private/tmp/nucbox-baseline-Sth9bYSs/D1-freeze-attribution.md` | Create one private report containing provenance validation, normalized observations, progress bounds, recovery sequence, verdict, and missing discriminator or single successor proposal. | Primary D1 deliverable. Depends on the retained-input review. No runtime persistence schema is introduced. |
| `prompt-exports/optimize-nucbox-latency-freezes-runs.md` | Append one D1 result entry; preserve all preceding bytes and corrections. | Records the decision without rewriting A1/A2 history. Depends on completion or explicit blocking of the report. |

The supplied Rust files are **read-only interpretation dependencies**:

- `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs`
- `native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs`
- `native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs`

There are no changed signatures, production call sites, protocol contracts, actor/thread ownership rules, or application migrations.

### Append-only scoreboard entry

Append this entry **after execution**, replacing the evidence-dependent fields from the report. It is a template, not a completed result:

> ### D1 — retained A2 freeze adjudication — `<actual completion UTC>`
>
> - Scope: bounded retained-evidence analysis only; A2 context 12:34:00–12:39:00 UTC, terminal focus 12:35:50–12:36:30 UTC. Earlier recovered 299.112574-ms incident retained separately.
> - Execution: `<completed or blocked>`; private report `D1-freeze-attribution.md`.
> - Verdict: `<BLOCKED_EVIDENCE / STAGE_ONLY / DISCRIMINATOR_FOUND>`.
> - Demonstrated progress boundary: `<component, counter evidence, epoch, time bounds, and uncertainty>`.
> - Recovery sequence: `<supported first-timeout, recovery-application, second-timeout, and terminal ordering; unavailable boundaries explicitly unknown>`.
> - Independent discriminator: `<specific sanitized evidence, or NONE>`; unresolved alternatives: `<remaining explanations>`.
> - Evidence coverage: `<validated private artifact labels and any missing or conflicting records>`. No raw records or cloud identifiers included.
> - Measurement status: A1 descriptive results retained; A2 INVALID with completed and partial evidence retained; three-attempt/nine-window repeatability INCOMPLETE. No latency improvement or physical input-to-photon claim.
> - Actions: no new host probe, stream restart, input, instrumentation, build, tests, configuration change, vendor edit, commit, push, or publication.
> - Decision: `<STOP for missing evidence / STOP with stage-only attribution / return one named successor proposal to coordinator>`. No successor action executed.

## Implementation order

1. **Establish the D1 input boundary.** Resolve the named private exports and their existing source pointers. Confirm that the report destination does not already exist.
2. **Validate provenance and semantics.** Match A2 identity/build receipts; resolve only the counter and reset semantics needed for comparison. Record unavailable information explicitly.
3. **Perform the bounded correlation.** Construct per-owner progress bounds, reconstruct the two timeout intervals, and align Qt/observer evidence with their uncertainty preserved.
4. **Adjudicate once.** Apply the three verdicts and record either the exact missing discriminator or one supported successor proposal. Do not start another observation block.
5. **Write the private report, then append the ledger result.** These operations are ordered; do not append a successful report reference before the report exists. No original evidence is modified.
6. **Stop and hand back the result.** Leave the client, account, repository source, and host configuration untouched. Even `DISCRIMINATOR_FOUND` completes only D1; it does not start a fix or establish a reliable latency comparison.

End of Oracle group: 2 lanes above.



> 💡 The 2 Oracle lanes above are independent answers; reconcile them using the guidance at the top of the group. Lane order is not a ranking.

Optional later follow-up: Continue this plan conversation with ask_oracle(chat_id: "nucbox-latency-and-freez-F7454F", new_chat: false)