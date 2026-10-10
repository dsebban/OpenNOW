## Final Prompt
<taskname="NucBox Receive Diagnostic"/>
<task>Plan exactly one bounded target-side receive-pressure diagnostic deployment for OpenNOW Qt/native on ChimeraOS. The user now explicitly authorizes ordinary local commits/push and target pull/build. Reconcile the private target source lineage with local source, inspect diffs, add the minimum diagnostic-only instrumentation needed to observe VIDEO receive-owner pressure during a real event, validate it, publish only the authorized source commit(s), then build/deploy and capture one bounded target incident window under the existing CUA operator. Do not repeat the descriptive baseline or ship speculative throughput/recovery/keepalive fixes. Deliver a concrete, executable plan with metric definition, evidence gates, supported commands/build path (inspect actual build script and preflight receipts before invocation; do not invent unsupported flags), writable scope, indispensable tests only, deployment/capture/checkpoint sequence, stop/continue criteria, and one explicit subsequent optimization selection rule. Do not claim a live gain or call any measure physical input-to-photon latency.

<architecture>Qt/QML client embeds the Rust streamer via FFI. The transport crate owns the `opennow-nvst-video` worker: serial `recv_from` followed inline by source/STUN handling, `NvstVideoReceiver::process_datagram` (authentication/reorder/FEC/frame assembly), then `forward_receive_event` to bounded media/event queues. Transport feeds typed NVST events to core; core forwards them, allows one recovery attempt and can stop on timeout/exhaustion. Protocol log helpers are bounded and payload-free by construction; FFI configures the embedded native log file. The NucBox `tools/build-native.sh` normal release flow is the artifact builder; its exact supported invocation and outputs must be established from the delegated fresh preflight and script inspection, not inferred from this context.</architecture>

<selected_context>
prompt-exports/optimize-nucbox-latency-freezes-runs.md: Authoritative append-only evidence ledger, exact B1R3 live results, terminal incident, R1/R1b replay coverage and limitations, capability receipts, identities, private evidence pointers, ownership handoff. Preserve all earlier records; no raw evidence/credentials in public context.
.agents/skills/verify-gfn-experiments/SKILL.md: Live host workflow, CUA single-operator ownership, Doctor/launch/evidence/cleanup rules and privacy constraints.
.agents/skills/verify-gfn-experiments/features/latency-baseline.md: Supported descriptive collector and observer commands/limits; this next step is diagnostic, not another baseline.
scripts/nucbox-latency-baseline.py: Existing local collector entry point; context only, do not rerun for this task.
native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs (full): Existing test-only authenticated replay harness, semantic invariants, corrected queue-volume model, measured local results and limitations. Reuse its existing tests/fixtures where they can meaningfully validate instrumentation helpers; it cannot establish target scheduling/socket causality.
native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs (slices): Production receive worker, `recv_from`→`process_datagram`→forward path and event handoff; receiver semantics; socket setup; current cfg(test) replay module declaration and adjacent handoff tests.
native/opennow-streamer/crates/opennow-streamer-transport/Cargo.toml: Existing feature set; no existing diagnostic feature is defined. If a production diagnostic gate is needed, explicitly add the smallest supported debug/support gate and keep instrumentation absent from ordinary optimized release.
native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs (slices): NVST event worker and current recovery/terminal-stop semantics; maintain event order and behavior.
native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs (slice): Existing bounded payload-free log channel and sanitization; consider bounded summaries only, no per-packet output or log I/O in receive hot path.
native/opennow-streamer/crates/opennow-streamer-ffi/src/lib.rs (slice): Native log file setup entry point.
native/opennow-streamer/crates/opennow-streamer-platform-linux/README.md and native/opennow-streamer/README.md: Linux build/runtime architecture and crate responsibilities.
</selected_context>

<relationships>
- UDP VIDEO socket → receive-owner worker → `process_datagram` → `forward_receive_event`/bounded consumer → core event worker → recovery/terminal state.
- B1R T3 observed +168 VIDEO socket drops and +168 host `InErrors`/`RcvbufErrors`; effective receive buffer was 425,984 bytes. DTLS bundle had zero drops. This local boundary evidence does not establish exact packet arrival schedule, skb memory capacity, or the initiating cause.
- B1R’s nine short descriptive windows: pooled fresh-interval upper p95/p99 22/26 ms, 349/32,613 late intervals (1.070125%), one >512 ms overflow with null finite maximum. A later silence/teardown occurred ~601 seconds after last input, outside observers; cause/initiator remains unresolved. Do not rerun or relabel those measurements.
- R1 corrected authenticated replay matrix: 1,409,340 packets, 10,800 exact frames, 60 FEC repairs; all 10,800 frame-boundary waiting samples were zero packets/bytes, no growth; local negative does not exclude NucBox scheduling, syscalls or target bursts. R1b one actual >1 ms sample was 1.943 ms (189.666 µs processing + 1,753.084 µs forwarding) on ordinary EOF data; independent SRTP tails had different identities. No blocking operation/candidate localized.
- Missing target observable: time-correlated active VIDEO worker arrivals/drop intervals, recv-return→end-of-service durations, and runnable/execution vs off-CPU/wait state during the same drop interval. A raw receive gap alone may mean idle. Existing sparse socket/network observers and `/proc` aggregates do not provide this correlation; perf/strace/trace-cmd/BPF tools are absent/restricted, and host policy must not be changed or tools installed by default.
- The private target checkout is at `~/dev/gfn-client-research`, clean at `b66a0ea9`, two unpublished diagnostic commits `c10d37d0` and `b66a0ea9` ahead of local/origin `6472cae7`; original binaries are attributed to c10, b66 unbuilt. Local `6472cae7` has separate pending `AGENTS.md` additions and test-only receive replay hook/module. Existing target/local transport source trees reportedly match; privately reconcile commit ancestry and inspect exact diffs before publishing. Preserve all unrelated/pending AGENTS work and do not copy vendor diagnostics/log payloads into public context.
- Sole GUI/account operator remains the original pair with retained agent-scoped CUA stdio 91787; ownership is not available to this planner and must be freshly confirmed by that operator. External Claude has an unsent draft only. Delegated target preflight is concurrent; wait for its actual command/capability/build-script findings and never assume readiness from historical state.
</relationships>

<ambiguities>
The local transport Cargo manifest has no receive-diagnostic feature today. Determine whether an existing cfg/build gate is suitable from the actual FFI/build graph and target script; if none exists, propose the narrowest explicit opt-in support/diagnostic gate that leaves normal optimized builds instrumentation-free. Choose telemetry sufficient to correlate wall service duration with thread CPU/runnable state and drops/queue pressure without per-packet logs, allocations, file I/O, protocol identity leakage, or changed receive timestamps/order/queues/recovery semantics. Quantify timing-hook overhead with the existing local replay harness before target capture; do not assume it is negligible. An optimized instrumented diagnostic artifact is distinct from the normal production artifact and its one target run is diagnostic evidence, not an A/B gain claim.

Required acceptance: private fresh preflight confirms exact source/branch/cleanliness/ownership and build script behavior; source review proves only authorized files changed and diagnostic code is opt-in; meaningful focused tests plus transport tests and optimized ordinary release compile pass; diagnostic artifact is identified by source revision, enabled gate and binary hashes/receipt; one valid target capture overlaps the authorized running VIDEO stream and yields a time-aligned service/drop/counter record with private raw data, sanitized aggregate only in ledger. Stop before push/build/live use if source ancestry/diff is ambiguous, operator is unavailable, build/runtime identity cannot be bound, instrumentation overhead or semantic changes are material, or safe target proof cannot be collected. Do not install tracing tools or change host/network/rmem/profile/display policy.

After capture, select exactly one optimization only if repeated/correlated evidence identifies a specific dominant mechanism and a minimally invasive candidate; require a matched control/candidate within the same cloud session for any gain claim. If evidence shows loss without a target-side worker/service correlation, choose no optimization and report the exact remaining missing observable. Keep scoreboard append-only, preserve private evidence paths/hashes and invalid outcomes, and distinguish VIDEO socket drops, observed component durations, presentation intervals and terminal stall. Do not label any of these physical input-to-photon latency.
</ambiguities>

## Selection
- Files: 13 total (9 full, 4 slice)
- Total tokens: 83659 (Auto view)
- Token breakdown: full 57864, slice 25795
- Token accounting: fresh from active_tab_published

### Files
### Selected Files
├── .agents/
│   └── skills/
│       └── verify-gfn-experiments/
│           ├── features/
│           │   ├── README.md — 346 tokens (full)
│           │   └── latency-baseline.md — 1,133 tokens (full)
│           └── SKILL.md — 3,203 tokens (full)
├── native/
│   └── opennow-streamer/
│       ├── crates/
│       │   ├── opennow-streamer-core/
│       │   │   └── src/
│       │   │       └── lib.rs — 5,647 tokens (lines 1-180 (Core module imports and NVST event types), 1600-2000 (NVST receive event worker and timeout/recovery/terminal stop handling))
│       │   ├── opennow-streamer-ffi/
│       │   │   └── src/
│       │   │       └── lib.rs — 543 tokens (lines 780-830 (FFI native file logging entry point and log setup behavior, relevant to diagnostic output privacy/bounds))
│       │   ├── opennow-streamer-platform-linux/
│       │   │   └── README.md — 2,491 tokens (full)
│       │   ├── opennow-streamer-protocol/
│       │   │   └── src/
│       │   │       └── log.rs — 2,340 tokens (lines 1-260 (Protocol diagnostic log sanitization/summarization behavior, bounded and payload-safe seam))
│       │   └── opennow-streamer-transport/
│       │       ├── src/
│       │       │   ├── nvst/
│       │       │   │   └── receive_replay_tests.rs — 9,385 tokens (full)
│       │       │   └── nvst.rs — 17,265 tokens (lines 1-170 (Imports/types and transport module context used by receive implementation), 4140-4485 (NvstVideoReceiver state and process_datagram authenticated processing path used by the active UDP receive worker), 5290-5390 (UDP socket creation/binding and platform-specific behavior), 7050-7785 (Production video UDP worker: recv_from, processing, counters/timeouts, event forwarding, and adjacent test-module declaration), 8400-8620 (Existing receive handoff/backpressure semantic tests relevant to safe instrumentation invariants))
│       │       └── Cargo.toml — 390 tokens (full)
│       └── README.md — 5,005 tokens (full)
├── prompt-exports/
│   └── optimize-nucbox-latency-freezes-runs.md — 32,360 tokens (full)
└── scripts/
    └── nucbox-latency-baseline.py — 3,551 tokens (full)


---

## Generated Plan
- Oracle group status: failed
- Oracle group: `6B92B7C7-9BD4-4185-AF65-99D6EEFB93D6`

**Reconciling these Oracle lanes**
2 independent answers to the same request follow. Lane order is not a ranking; the first lane supplies the top-level continuation handle, and a successful follow-up through any lane's chat ID re-runs every lane.
- Read every lane through the end-of-group marker (`End of Oracle group: 2 lanes above.`). If the marker or a lane is missing, page the export or try the read-only `oracle_chat_log` with that lane's chat ID. Logs may be scoped or clipped; report any remaining gap. Do not start a follow-up just to retrieve prior text.
- Reconcile by evidence, not lane order, answer length, or model identity: check material single-lane and conflicting claims against the code, and report unresolved disagreements. A failed or partial lane is incomplete evidence.
- Before synthesizing, inventory every material claim from every lane, including single-lane claims. Merge only exact duplicates, retaining all source lanes. Begin your answer with `**Oracle reconciliation**`, state how many lanes completed and name any that did not. For every inventory item, give its source lanes, checked evidence, and exactly one disposition: `accepted`, `rejected`, or `unresolved`. Never silently omit an item.

Lanes (2):
- Oracle — `claude-opus-5-5-xhigh` — Failed — chat ID `nucbox-target-side-freez-0C3867`
- Oracle 2 — `gpt-6-astra-pro` — Completed — chat ID `nucbox-target-side-freez-66449A`

### Oracle
- Status: Failed
- Provider: `Custom`
- Model: `claude-opus-5-5-xhigh`
- Effective effort: Provider default / not specified
- Chat: `nucbox-target-side-freez-0C3867`

Error [context_builder_inactivity_timeout]: Context Builder Oracle exceeded its 600s inactivity budget during streaming.

### Oracle 2
- Status: Completed
- Provider: `Custom`
- Model: `gpt-6-astra-pro`
- Effective effort: Provider default / not specified
- Chat: `nucbox-target-side-freez-66449A`

# 1. Summary

Plan one **optimized, explicitly enabled Linux receive diagnostic build**, followed by **one 180-second target capture** on the actual VIDEO receive owner. The diagnostic will correlate received-datagram timing, VIDEO socket pressure and drops, receive-service spans, and independently recorded thread scheduling activity. It will preserve transport timestamps, packet order, media queues, and recovery behavior. Use a clean deployment branch rooted at the verified common ancestor; preserve the private `c10d37d0`/`b66a0ea9` history separately and publish only reviewed, allowlisted changes. **Publication and deployment remain gated on the missing fresh preflight, actual FFI feature graph, supported `tools/build-native.sh` invocation, and a demonstrated permission-compatible scheduling source.** Those inputs are not included here, so this plan does not invent their results or a build-script flag. The result will be diagnostic evidence, with no automatic rerun, baseline repetition, or gain claim.

All references below are relative to `/Users/danielsivan/dev/OpenNOW`. **Observation** means a statement directly supported by the supplied source or ledger; it does not mean I performed a new verification. **Inference** identifies a conclusion drawn from those observations. **Decision** identifies a proposed implementation contract.

# 2. Current-state analysis

## Receive ownership and execution

**Observation — the dedicated VIDEO worker is a serial service path.** `run_nvst_udp_receiver` delegates immediately to `run_nvst_webrtc_bundle` when `setup.rtc.is_some()`. Otherwise, one worker drains commands, sends due pings, receives one datagram, performs source/STUN handling, calls `NvstVideoReceiver::process_datagram`, and forwards the resulting events. Receiver reports, periodic diagnostics, packet timeout, and frame-progress checks then run before the next iteration.  
`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7050-7785`

The relevant flow is:

1. The reserved UDP socket receives a datagram.
2. The worker counts the raw receive and performs source/STUN handling.
3. `NvstVideoReceiver` checks lifecycle, authenticates SRTP, validates payload type and SSRC, updates feedback, reorders/repairs packets, and assembles access units.
4. `forward_receive_event` converts completed frames to `EncodedMediaFrame` and attempts bounded media delivery.
5. Core consumes typed transport events and media feedback, manages recovery, and forwards lifecycle/telemetry to the embedded client.

**Observation — raw receipt, accepted authentication, assembly, and delivery are different boundaries.** `inbound_datagrams` increments after a successful socket receive, including datagrams that subsequently fail admission. `authenticated_packets` increments only after source, lifecycle, SRTP, payload-type, and SSRC checks. Existing `record_socket_receive` calls also occur before processing and, under a first-authentication condition, after processing; they are not an independent per-packet authentication counter.  
`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4140-4485`  
`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7050-7785`

**Decision:** measure these boundaries separately. Do not “repair” existing feedback accounting as part of this diagnostic.

## Semantics that instrumentation must preserve

**Observation — transient consumer pressure deliberately preserves receiver liveness and reference-chain discontinuity.** `forward_receive_event` publishes assembly before attempting delivery, retires an undelivered frame, retains `delivery_gap` after backpressure, and marks the next successfully delivered frame discontinuous. Existing tests cover delivery/admission racing, backpressure, frame identity, and subsequent continuity restoration.  
`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7050-7785`  
`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:8400-8620`

**Observation — recovery owns state transitions and timeout epochs.** `resume` and `recover` clear accepted-packet timing and reset the timeout origin; cumulative receive counters are separate state. Core defines `NVST_RECOVERY_ATTEMPT_LIMIT = 1` and consumes the existing transport events in its session worker.  
`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4140-4485`  
`native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:1-180`  
`native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:1600-2000`

**Decision:** retain every existing call, event order, return value, timestamp argument, queue operation, and recovery condition. Record lifecycle transitions observationally; never use diagnostic state to request a keyframe, recover, pause, or stop media.

## What the retained evidence establishes

**Observation — the live evidence identifies a local loss boundary, not its cause.** B1R retained nine descriptive windows, 349 late intervals out of 32,613, and one histogram overflow. Its T3 VIDEO socket accumulated 168 drops; the retained mapping attributes zero drops to the bundle and reports a 425,984-byte VIDEO receive-buffer value. The later terminal incident occurred outside the bounded observers.  
`prompt-exports/optimize-nucbox-latency-freezes-runs.md:250-317`

**Observation — corrected R1 and R1b did not establish a production bottleneck.** Corrected R1 reported no frame-boundary waiting volume across the fixed matrix. R1b localized one elapsed tail to forwarding, but did not identify a blocking operation or a repeated shared cause. The ledger explicitly retains missing target scheduling, socket, arrival-distribution, and contention coverage.  
`prompt-exports/optimize-nucbox-latency-freezes-runs.md:408-491`  
`prompt-exports/optimize-nucbox-latency-freezes-runs.md:493-566`

**Inference — another uninstrumented baseline or unchanged replay cannot resolve the remaining question.** The next useful observation must join actual target socket pressure with the actual worker’s service and scheduling intervals. Neither the local negative nor the historical buffer size predicts target packet capacity or excludes target pressure. The supporting observations are the live VIDEO-drop attribution and the replay coverage limits cited above.

## Existing extension points and limitations

**Observation — no receive-diagnostic Cargo feature currently exists in the supplied transport manifest.** Its only declared feature is `sony-peer-probe`.  
`native/opennow-streamer/crates/opennow-streamer-transport/Cargo.toml:8-46`

**Observation — the embedded application already has a native file-log destination, but its logger is unsuitable as the primary trace store.** FFI configures that sink; `write_line` serializes file access, limits messages to 2,048 characters, rotates at 2 MiB, and assigns the wall timestamp while writing. These properties do not provide receive-event timestamps or enough structured incident detail.  
`native/opennow-streamer/crates/opennow-streamer-ffi/src/lib.rs:780-830`  
`native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs:1-260`

**Decision:** reuse existing native and Qt logs as corroborating evidence. Store the new bounded numeric capture privately through a diagnostic-only exporter, with no receive-thread file logging.

**Observation — live ownership and deployment facts are historical.** The ledger preserves private diagnostic ancestry, unbuilt `b66`, qualified c10 artifact attribution, and the retained operator transport. They do not establish current ownership or readiness. The verification workflow requires one GUI/account operator and exact process-instance checks before replacement.  
`prompt-exports/optimize-nucbox-latency-freezes-runs.md:233-257`  
`.agents/skills/verify-gfn-experiments/SKILL.md:8-20`

# 3. Design

## A. Source reconciliation and publication boundary

### Required private preflight

Before creating a publishable commit, obtain the delegated preflight’s actual receipt. It must establish:

| Required fact | Validation and consequence |
|---|---|
| Local and target repository roots, branch names, HEADs, and worktree status | Record them privately. Any unexpected modifications stop reconciliation. |
| Ancestry of `6472cae7`, `c10d37d0`, and `b66a0ea9` | Verify commit objects and parent relationships, rather than relying on abbreviated names. |
| Exact changes in both private commits | Inspect complete diffs privately, including logging payloads and non-transport changes. |
| Transport equivalence | Compare the committed transport tree at the common base against both private commits. Do not include the local uncommitted test hook in that comparison. |
| Current GUI/account ownership | The existing operator explicitly confirms ownership and access to the retained CUA transport. |
| Actual build entry point | Inspect `tools/build-native.sh`, its invoked CMake/Cargo commands, supported feature injection, working directory, build directories, and installed outputs. |
| Artifact attribution | Record the existing Qt, core, and FFI files, their hashes, the running process identities, and the mapped FFI identity. |
| Diagnostic prerequisites | Confirm Linux socket metadata support and the scheduling capability described below without changing host policy. |

**Decision:** the publishable branch starts from the verified common base, expected to be `6472cae7`. It contains the new diagnostic and the reviewed test support needed to validate it.

- Preserve `c10` and `b66` through their existing target branch and explicit private preservation refs.
- Do not merge or cherry-pick those commits wholesale.
- If inspection confirms that their differences are unnecessary private diagnostics, omit them from the deployment branch.
- If either contains a functional dependency required for this task, **stop for reconciliation**. Do not quietly deploy a different behavioral baseline.
- Preserve the original local worktree, including pending `AGENTS.md`, ledger changes, and unrelated files. Prepare the publishable changes in an isolated local worktree.
- Inspect the complete commit diff and its file list before pushing. Never use broad staging or push all refs.

**Inference — this separation makes publication reviewable without publishing private logging history.** The ledger says the private changes and deployed artifact attribution are distinct from transport-source equivalence; equivalence of the transport tree does not establish equivalence of the whole application.  
`prompt-exports/optimize-nucbox-latency-freezes-runs.md:233-257`  
`prompt-exports/optimize-nucbox-latency-freezes-runs.md:309-317`

### Explicit compilation gate

Add the feature **`receive-diagnostics`**, disabled by default:

- Transport: enables the optional Linux diagnostic dependency and implementation.
- Core: forwards to `opennow-streamer-transport/receive-diagnostics`.
- FFI: forwards to `opennow-streamer-core/receive-diagnostics`.

Production hooks compile only under:

```text
feature = "receive-diagnostics" AND target_os = "linux"
```

Portable accumulator tests may additionally compile under `cfg(test)`.

Use an optional Linux `libc` dependency resolved to the version already present in the actual lockfile. Do not upgrade unrelated packages or guess a dependency version from this prompt. The lockfile change, if required, must be limited to the diagnostic dependency edge.

**Decision:** normal optimized builds contain no diagnostic socket wrapper, observer thread, environment lookup, timing hook, accumulator, or exporter. `debug_assertions` is not the gate: the diagnostic artifact must use the ordinary optimized release profile.

The core and FFI manifests were not supplied. Validate their actual dependency edges before adding the forwarding features. If they differ from this path, stop and revise the wiring before publication.

### Build invocation gate

The target preflight must return a concrete build recipe containing:

```text
working_directory
argv
allowlisted_environment
existing_required_features
diagnostic_feature_addition
output_artifact_paths
build_log_path
```

Run **that inspected recipe** through `tools/build-native.sh`.

Do not substitute a standalone Cargo FFI build for the application builder, guess a `--features` flag accepted by the shell script, or replace its existing feature set. If the builder has no supported way to enable the FFI feature, this plan stops before publication/build; an unreviewed build-script modification is outside the writable scope below.

## B. Diagnostic ownership and runtime activation

Add a private `ReceiveDiagnosticOwner` in the transport crate. It is created only after the `setup.rtc` delegation branch, for the dedicated raw VIDEO worker.

Its ownership is:

| Component | Owned state and lifetime |
|---|---|
| `ReceiveDiagnosticOwner` | Worker-local capture state, current phase, counters, timing accumulators, lifecycle epoch, and completion guard. |
| `DiagnosticControl` | Small shared atomic control state: readiness, arm request, capture start/deadline, cancellation, producer completion. |
| `WorkerCapture` | Preallocated numeric bins, arrival counts, bounded long-span records, validity counters. Mutated only by the VIDEO worker; transferred once to the exporter. |
| `ReceiveDiagnosticObserver` | Scheduling reader, metadata-only duplicate of the VIDEO socket, clock anchors, socket snapshots, private output handles, and its own bounded buffers. |
| `DiagnosticCaptureResult` | Immutable completed or partial capture plus typed validity and coverage results. |

No new state belongs in `NvstVideoReceiver`, core lifecycle, media queues, or FFI protocol objects.

Partial interface shapes:

```text
prepare(socket, worker_identity)
    -> Result<Option<ReceiveDiagnosticOwner>, DiagnosticUnavailable>

record_receive_boundary(...)
record_service_boundaries(...)
record_lifecycle_transition(...)

finish(reason)
    -> one ownership transfer of WorkerCapture
```

`None` means runtime opt-in was absent. Diagnostic failure must not become `TransportError`.

### Runtime opt-in

A diagnostic-enabled artifact additionally requires:

```text
OPENNOW_NVST_RECEIVE_DIAGNOSTICS_DIR=<new private capture directory>
```

Read it once during diagnostic setup. Require an absolute, newly created, private directory belonging to the current user. The ordinary artifact ignores it because the implementation is absent.

The observer writes a private `ready.json` containing only:

- schema and gate identifiers;
- PID, OS TID, worker role, and socket inode;
- capability results;
- a process-local capture ordinal;
- readiness status.

The external collector supplies the independently verified process start ticks and artifact hashes.

### State machine

Use exactly these capture states:

| State | Transition |
|---|---|
| `WaitingForArm` | Prepared successfully; no measurement yet. |
| `Recording` | One valid arm request accepted for the recorded process/TID/socket. |
| `Complete` | The 180-second interval ended with complete required evidence. |
| `Partial` | Worker/session ended, or a required end boundary was not observed. |
| `Invalid` | Capability, clock, identity, capacity, or serialization validation failed. |

A process-local one-shot claim prevents a second worker or later session from creating another capture. A recovery within the same worker increments the diagnostic lifecycle epoch but does not restart the timer or clear lifetime counters.

Expire an unarmed diagnostic after **300 seconds**. No automatic rearming is permitted.

## C. Receive timing and socket metadata

### One processing path

Keep the existing source/STUN → `process_datagram` → `forward_receive_event` implementation shared.

Only the receive primitive differs while diagnostics are active:

- Ordinary build or inactive runtime: existing `UdpSocket::recv_from`.
- Active Linux diagnostic: a small `recvmsg` adapter that returns the same length/source result and additionally captures ancillary metadata.

Preserve the existing datagram buffer, blocking/read-timeout behavior, receive flags, error classification, and address handling. Do not introduce batching, draining loops, nonblocking mode, altered socket buffers, or a second packet reader.

**Decision:** an ancillary-data failure invalidates diagnostic coverage but does not discard an otherwise successfully received datagram. A real socket receive error continues through the existing error path.

The Linux adapter must be validated against target UAPI definitions and loopback behavior before live use. The supplied code establishes the current `recv_from` seam and error handling; it does not establish support for the proposed ancillary options.  
`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:5290-5390`  
`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7050-7785`

### Timing boundaries

Record diagnostic monotonic timestamps at:

| Boundary | Definition |
|---|---|
| `recv_enter_ns` | Immediately before the socket receive call. |
| `recv_return_ns` | Immediately after it returns, before first-inbound logging or other handling. |
| `process_enter_ns` | Immediately before `process_datagram`. |
| `process_exit_ns` | Immediately after `process_datagram`. |
| `service_exit_ns` | After all resulting events have been forwarded, or immediately before an existing early return. |
| `next_recv_enter_ns` | The following receive call’s entry. |

Derive:

- **Receive-call duration:** `recv_return − recv_enter`; includes waiting and wakeup effects.
- **Pre-processing duration:** `process_enter − recv_return`; includes source/STUN handling for packets reaching processing.
- **Processing duration:** `process_exit − process_enter`.
- **Forwarding duration:** `service_exit − process_exit`.
- **Receive-service duration:** `service_exit − recv_return`.
- **Between-receive work:** `next_recv_enter − previous service_exit`; includes commands, pings, reports, diagnostics, and watchdog work.

STUN-only and rejected datagrams have their own service span and no fabricated processing/forwarding sample.

**Decision:** retain the existing production `received_at = Instant::now()` calls at their present locations. Diagnostic timestamps never replace the `now` passed to protocol processing, feedback, timeout logic, or frame forwarding.

### Socket evidence

Require the diagnostic adapter/observer to establish these proposed Linux contracts:

1. **Software kernel receive timestamp**, obtained through a supported timestamp ancillary API.
2. **Per-socket overflow metadata**, obtained through `SO_RXQ_OVFL`.
3. **Current socket memory/drop snapshot**, obtained through a supported `SO_MEMINFO` query.

The target capability probe must verify returned layouts, lengths, units, and counter widths. Use the installed UAPI definitions; do not hand-assume cross-architecture ancillary layouts.

Snapshot socket metadata every **10 ms** from the observer, with monotonic timestamps immediately before and after the query. Record:

```text
sample_before_ns
sample_after_ns
rmem_alloc_bytes
rcvbuf_bytes
socket_drops
query_status
```

Interpretations are deliberately limited:

- `rmem_alloc_bytes` is socket memory accounting, not queued UDP payload bytes.
- `rcvbuf_bytes` is the current reported socket bound, not a packet-capacity estimate.
- A positive drop delta belongs to the interval between snapshots; it is not an exact drop timestamp.
- Overflow ancillary values may describe an earlier enqueue state and can arrive late.
- Kernel timestamps exist only for delivered datagrams. They do not reveal the arrival times of dropped packets.
- Counter decreases or incompatible snapshots mark a discontinuity; do not silently turn them into negative loss or assume a wrap.

Record actual buffer values at setup and completion. Do not carry the historical 425,984-byte value into a fresh capture as an assumption.

## D. Scheduling and CPU evidence: mandatory capability gate

**Inference — CPU clocks alone do not supply the missing scheduling discriminator.** R1/R1b already contain elapsed service measurements that can include descheduling, while the capability receipt says the available `/proc` aggregates do not provide the required correlated trace. A new wall-time measurement without an independent scheduling source would leave that limitation unresolved.  
`prompt-exports/optimize-nucbox-latency-freezes-runs.md:493-566`

### Chosen source

Use **self-thread kernel context-switch recording through `perf_event_open`**, inside the diagnostic implementation, only if a fresh probe demonstrates that the current target policy permits the required recording.

This is a proposed capability, not an assertion that the target currently permits it.

The probe and runtime must use:

- the current thread, not system-wide or another-process attachment;
- a software event suitable for context-switch records;
- monotonic event timestamps;
- kernel/hypervisor exclusions required by current policy;
- context-switch records and sample identity sufficient to join the recorded VIDEO TID;
- explicit lost-record accounting.

No capability elevation, sysctl change, tracing-tool installation, or alternative privileged mode is allowed.

The probe must demonstrate readable switch-out/switch-in records around a known voluntary wait and a runnable interval, and verify the actual timestamp domain. A successful descriptor open alone is insufficient.

**If this fails, stop before publication/build/live deployment.** Do not replace it with zero `schedstat` values, a high-frequency `/proc` poll, or a wall-minus-CPU field labelled “run-queue time.”

### CPU cross-check and interpretation

Sample `CLOCK_THREAD_CPUTIME_ID` at worker bin boundaries, with bracketing monotonic timestamps. This avoids two CPU-clock syscalls on every datagram.

Scheduling records provide interval-level on/off-CPU coverage. CPU-clock deltas independently check aggregate execution accounting.

Report separately:

- recorded on-CPU overlap;
- off-CPU overlap following a switch explicitly marked preempted;
- other off-CPU overlap;
- uncovered scheduling time;
- CPU-clock delta over its actual bracketing interval.

Do not label every non-preempted switch as blocking: voluntary yielding and wake-to-run delay remain possible. Do not call all off-CPU time run-queue delay. A capture may therefore identify preemption or CPU service pressure while leaving the blocking-versus-wakeup distinction unresolved.

Lost scheduling records, unmatched boundaries, or inadequate coverage invalidate causal attribution for the affected interval.

## E. Bounded storage, clocks, privacy, and lifecycle

### Fixed resource budgets

These are **proposed limits**, not measured target capabilities:

- Capture duration: **180 seconds**.
- Worker aggregate bins: **18,000 × 10 ms**.
- Kernel receive-time counts: **180,000 × 1 ms**, in the kernel timestamp’s own clock domain.
- Long-span records: maximum **4,096**, retaining service or between-receive spans of at least **1 ms**.
- Scheduling records: maximum **262,144**.
- Total preallocated capture memory: maximum **64 MiB**.
- Serialized private output: maximum **64 MiB**.

Preallocate before arming. During recording, the VIDEO worker performs timestamp reads, fixed-array updates, and bounded checks only. It performs no per-packet allocation, formatting, channel send, file access, or log output.

All datagrams contribute aggregate counts and duration histograms. Long-span records contain only local timestamps, a numeric ordinal, phase labels, and outcome counters—never RTP identities, payloads, addresses, or server frame identifiers.

If any record capacity is exhausted, increment an explicit omission counter and mark coverage invalid. Never overwrite early evidence silently.

### Aggregate definitions

Worker bins include:

- raw receive successes, bytes, idle returns, and errors;
- expected/unexpected source counts;
- handled/invalid STUN counts;
- accepted-authenticated packet deltas;
- repaired-packet and assembled-frame deltas;
- frame-forward attempts;
- forwarding terminal returns;
- receiver state and lifecycle epoch;
- per-phase count, total duration, maximum, and bounded histogram;
- metadata-missing and diagnostic-error counts.

“Frame-forward attempt” is not renamed “admitted frame.” Admission remains a separate existing feedback boundary.

Duration histograms use documented upper bounds, with an overflow bucket and an independently retained exact observed maximum. Counts are assigned by completion bin. Cross-bin long spans retain their full endpoints; analysis must use interval overlap, not treat completion-bin totals as CPU utilization within that bin.

### Clock joining

Use distinct fields for:

1. diagnostic `CLOCK_MONOTONIC`;
2. thread CPU time;
3. kernel receive timestamp clock;
4. existing native/Qt log timestamps.

Record realtime/monotonic calibration anchors at capture start, once per second, and at completion. Each anchor includes a bracketing uncertainty interval.

A kernel-to-monotonic join is usable only where its conservative uncertainty is at most **1 ms**. Wider uncertainty or a discontinuous wall-clock mapping marks the affected segment unjoinable.

Never use native async log write time as the timestamp of the underlying packet or recovery operation. The supplied logger assigns its timestamp during writing.  
`native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs:1-260`

### Export and cleanup

The observer collects its socket/scheduling data into preallocated memory during the window. Export occurs after recording ends.

The worker transfers its completed buffer once through a capacity-one channel. The observer owns serialization. It must close its metadata socket duplicate and scheduling resources before doing export I/O, so file output cannot retain the media socket.

The worker’s completion guard covers every existing exit path:

- `Stop`;
- command-channel disconnect;
- fatal receive/send failure;
- media-consumer closure;
- panic/unwind where cleanup is possible.

It submits partial evidence without waiting for filesystem output. The observer must not hold a reference to the receiver, media consumer, feedback mutex, or core lifecycle.

On exporter failure, preserve any valid prefix and write a typed failure receipt when possible. Streaming behavior remains governed by the existing worker. The collector reports failure rather than waiting indefinitely or triggering another capture.

### Private schema

Use a new `schema: 1`, independent of the descriptive-baseline schema:

- `ready.json`
- `arm.json`
- `started.json`
- `metadata.json`
- `worker-bins.json`
- `kernel-arrival-counts.json`
- `long-spans.json`
- `socket-samples.json`
- `scheduler-records.json`
- `clock-anchors.json`
- `summary.json`

Every file is exclusive-create, private, and bounded. Unknown schema versions fail analysis explicitly.

No changes are required to persisted application settings, command DTOs, callback telemetry, FFI ABI, or `PROTOCOL_VERSION`.

## F. Collector and evidence acceptance

Add `scripts/nucbox-receive-diagnostic.py` as the sole diagnostic collection entry point. It performs no GUI, authentication, account RPC, launch, restart, or setting mutation.

Its interface should require:

```text
--pid
--start-ticks
--proof-dir
--label
--arm
```

Duration is fixed at 180 seconds in schema 1; there is no user-adjustable repetition option.

The collector:

1. Validates the private directory and safe label.
2. Verifies the exact process instance.
3. Reads and validates `ready.json`.
4. Checks the operator’s build/artifact receipt and the actual VIDEO role/TID/socket binding.
5. Exclusively creates one arm request bound to those identities.
6. Waits for the actual start acknowledgement.
7. Waits for completion, with a deadline of capture end plus five seconds for the completion acknowledgement.
8. Preserves partial output on timeout.
9. Validates schema, coverage, counters, clocks, and identity continuity.
10. Produces a numeric summary and a typed disposition.

Use these dispositions:

| Disposition | Meaning |
|---|---|
| `VALID_DROP_CAPTURE` | Positive VIDEO socket-drop delta with complete required evidence over its bracket. |
| `VALID_NO_DROP_CAPTURE` | Complete diagnostic window, but no positive VIDEO socket-drop delta. |
| `PARTIAL_SESSION_ENDED` | Session or receive owner ended before completion. |
| `INVALID_DIAGNOSTIC_COVERAGE` | Missing clocks, metadata, scheduling records, identities, or bounded storage coverage. |
| `BLOCKED_PREFLIGHT` | Required readiness or capability was not established before arming. |

`VALID_NO_DROP_CAPTURE` is not incident reproduction and does not authorize another run.

## G. Indispensable validation and overhead limits

### Semantic and adapter tests

Reuse existing SRTP/RTP/FEC fixtures and handoff tests. Do not duplicate cryptography or rerun the ignored R1 matrix.

Add only these focused tests:

1. **Receive-adapter equivalence:** Linux loopback delivery through ordinary receive and diagnostic receive preserves datagram bytes, source address, order, idle/error classification, and the resulting authenticated frame/handoff behavior. Include malformed or missing ancillary metadata and IPv4/IPv6 address decoding.
2. **Metric accounting:** raw, accepted-authenticated, STUN, frame-attempt, and drop counters remain distinct; lifecycle epochs do not reset lifetime totals; missing samples and counter discontinuities remain explicit.
3. **Clock/scheduler parsing:** bounds checks, timestamp joins, cross-bin spans, missing switch records, lost-record events, and clock discontinuities produce the specified validity result.
4. **Bounded lifecycle:** one arm only; capacity exhaustion never overwrites evidence or backpressures media; early worker exit creates partial output; diagnostic resources are released.

Retain the existing backpressure, closure, rollover, reorder, payload, and FEC assertions. Their relevance follows from the receive handoff’s existing contracts.  
`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:8400-8620`  
`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs:1-320`

### One overhead check, not another mechanism matrix

Add an ignored `receive_diagnostic_hook_cost_report` test to the existing replay module.

Use one unchanged **100-Mbps Repair fixture**, 30 warmup and 600 measured frames, with three predeclared ordinary/instrumented pairs ordered:

```text
ordinary → instrumented
instrumented → ordinary
ordinary → instrumented
```

Use fresh receiver state for each pass. Preserve all semantic assertions. Do not run independent SRTP passes, backlog-disposition analysis, or the 18-replay matrix.

Measure the outer service interval so diagnostic hook work is included. Report paired total service, p99 service, maxima, and worker/helper CPU cost. Do not subtract individual unrelated packet samples.

**Predeclared overhead gates:**

- median paired increase in total service no greater than **5%**;
- no pair’s total-service increase greater than **10%**;
- median paired p99-service increase no greater than **10%**;
- combined worker/helper CPU increase no greater than **5%** over the same complete replay interval;
- no semantic failure or diagnostic omission.

These are conservative diagnostic admission limits, not claims that the overhead will meet them.

On macOS, portable-hook results must be labelled as such. They cannot validate Linux `recvmsg`, socket queries, or scheduling collection. After the single target build, run one bounded Linux loopback parity/cost check with the actual enabled implementation **before GUI streaming**. Failure stops live use and triggers restoration; it does not earn a repeated timing trial.

### Required commands

The following repository commands are supported by the supplied manifest and prior test layout; run them in the isolated diagnostic worktree:

```sh
cargo test --locked --manifest-path native/opennow-streamer/Cargo.toml \
  -p opennow-streamer-transport --lib

cargo build --locked --release --manifest-path native/opennow-streamer/Cargo.toml \
  -p opennow-streamer-transport

cargo test --locked --release --manifest-path native/opennow-streamer/Cargo.toml \
  -p opennow-streamer-transport --features receive-diagnostics --lib \
  nvst::tests::receive_replay_tests::receive_diagnostic_hook_cost_report \
  -- --exact --ignored --nocapture --test-threads=1
```

The final command is the newly specified test entry point. Its implementation must preserve that exact name.

Also run the focused enabled-feature tests on Linux, formatting checks for touched Rust files, and `git diff --check`. Record whether Cargo compiled or reused artifacts. Do not force recompilation by touching unrelated source files.

# 4. File-by-file impact

| File | Changes and purpose | Dependencies |
|---|---|---|
| `native/opennow-streamer/crates/opennow-streamer-transport/Cargo.toml` | Add default-off `receive-diagnostics`; add optional Linux `libc` dependency using the inspected locked version. | Lockfile inspection. |
| `native/opennow-streamer/crates/opennow-streamer-core/Cargo.toml` | Forward the diagnostic feature to transport. No core runtime changes. | Validate actual dependency graph. |
| `native/opennow-streamer/crates/opennow-streamer-ffi/Cargo.toml` | Expose and forward the opt-in feature used by the application build. | Core forwarding and supported builder recipe. |
| `native/opennow-streamer/Cargo.lock` | Only the required diagnostic dependency edge, if resolution changes it. | Manifest edits; no unrelated upgrades. |
| `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs` | Add gated module declarations and hooks in the raw VIDEO branch; use the diagnostic receive adapter only when active; record unchanged lifecycle boundaries; finalize on every exit. | Diagnostic owner and Linux adapter. |
| `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs` **new** | Capture state, bounded accumulators, metric/schema definitions, one-shot ownership transfer, validity handling, and portable tests. | None beyond the gated dependency design. |
| `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics_linux.rs` **new** | Ancillary receive adapter, socket snapshots, scheduling capability/probe and reader, clock anchors, observer lifecycle, private export. | Target UAPI/capability validation. |
| `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs` | Reuse fixtures for the one overhead report; preserve existing replay semantics and historical report entry point. | Accumulator/timing hooks. |
| `scripts/nucbox-receive-diagnostic.py` **new** | Identity-bound one-shot arming, bounded wait, numeric validation, summary, and typed outcomes. | Final schema. |
| `.agents/skills/verify-gfn-experiments/features/receive-diagnostic.md` **new** | Document this diagnostic’s exact entry point, ownership, bounds, evidence gates, and restoration procedure. | Final collector/build contract. |
| `.agents/skills/verify-gfn-experiments/features/README.md` | Add one feature-map row pointing to the diagnostic procedure. | New procedure. |
| `prompt-exports/optimize-nucbox-latency-freezes-runs.md` | Append preflight, source/build receipts, overhead outcome, capture disposition, limitations, and restoration. Preserve the entire previous prefix. | Append at each settled checkpoint. |

**Explicitly unchanged:** `AGENTS.md`, `scripts/nucbox-latency-baseline.py`, core/FFI/protocol runtime source, `log.rs`, decoder/platform code, Qt rendering, network/rmem/display/profile settings, and `tools/build-native.sh`.

The new feature-map entry is validated by the same single diagnostic workflow; it does not create a second live smoke run.

# 5. Risks and migration

There is no application-data migration or protocol/ABI change. The material concerns are diagnostic perturbation, publication provenance, incomplete scheduling capability, and rollback of a temporary artifact.

- **Perturbation:** timing calls, ancillary metadata, and the observer can alter service cost. The paired overhead gates and target loopback check are mandatory.
- **Scheduling ambiguity:** switch records do not necessarily separate every voluntary wait from wake-to-run delay. Preserve that distinction in the result.
- **Capture loss:** bounded storage or missing metadata produces invalid coverage, never a quiet approximation.
- **Lineage:** the diagnostic artifact derives from the reviewed publication branch. It must not be described as an unchanged c10 artifact merely because transport sources previously matched.
- **Rollback:** archive the exact pre-deployment Qt/core/FFI artifacts and launch/build receipts after verified GUI shutdown. Restore those bytes and the preserved target checkout after the diagnostic, with hashes checked. Restoring historically c10-attributed binaries alongside unbuilt b66 must retain that qualification.

Do not replace files mapped by a running Qt process. Diagnostic output schema 1 is separate from the baseline schema, so existing baseline readers and historical reports remain unchanged.

# 6. Implementation order

1. **Settle the delegated fresh preflight.**  
   Obtain the actual ownership, source, build-script, feature-injection, artifact, and scheduling-capability results. No historical PID, stdio handle, or readiness label substitutes for this receipt. Stop if the operator is unavailable, the source relationship is ambiguous, the builder cannot enable the gate, or scheduling coverage cannot be obtained under existing policy.

2. **Reconcile source privately and prepare the isolated local worktree.**  
   Preserve the original worktree and private target refs. Inspect both private diffs. Confirm the common base and transport equivalence. Copy only the reviewed relevant test support into the isolated branch.

3. **Land the feature wiring and diagnostic implementation as one locally reviewable unit.**  
   The three Cargo feature edges, gated module declarations, and worker hooks must be atomic. The ordinary configuration must compile at the end of this step.

4. **Implement and run the focused validation.**  
   Complete adapter, accounting, clock/scheduler, bounded-lifecycle, and existing transport tests. Run the one local overhead report and ordinary optimized transport build. Preserve failures and all paired samples. Any material overhead or semantic change stops publication.

5. **Review the exact publishable commit.**  
   Compare its complete diff against the file allowlist. Confirm that private c10/b66 commits are not ancestors of the new branch unless explicitly resolved by the earlier gate; confirm no pending `AGENTS.md`, unrelated ledger history, private trace, raw logging text, or credentials were staged.

6. **Commit and push only the authorized diagnostic branch.**  
   Use a normal non-force push of the reviewed branch. Record the full resulting source OID. The user has authorized this action; no additional permission request is needed once the stated evidence gates pass.

7. **Have the sole operator prepare target replacement.**  
   Through the normal GUI, close the exact client instance and verify its Qt PID/start ticks and core child have exited. Archive prior logs and artifacts using the inspected existing procedure. Preserve the private target branch, fetch only the authorized branch, switch the clean checkout, and fast-forward to the exact reviewed OID.  
   **Observation supporting this ordering:** the verification workflow explicitly requires verified shutdown before replacement and log archival between shutdown and launch.  
   `.agents/skills/verify-gfn-experiments/SKILL.md:14-29`

8. **Invoke the inspected application builder once.**  
   Use the preflight’s exact `tools/build-native.sh` recipe and additive diagnostic feature configuration. Record original invocation, exit status, logs, source OID, enabled feature, outputs, and hashes. Do not reuse a consumed historical build-once guard or retry with guessed flags.

9. **Complete target pre-live validation.**  
   Run the one bounded Linux adapter/cost check against the enabled implementation. Confirm ordinary and diagnostic artifact identities are distinct and correctly attributed. Failure restores the prior deployment without a live capture.

10. **Launch the diagnostic client through the existing operator.**  
    Use the inspected normal wrapper with the single diagnostic-directory opt-in and existing allowed telemetry. Verify the new process instances and mapped FFI. Reach the normal saved Ori scene through CUA, verify the unchanged profile, and perform any input/scene checks before arming. Do not run the descriptive collector.

11. **Start observers, then arm exactly one capture.**  
    Use the already supported observer commands with fresh output paths:

    ```sh
    python3 tools/live-socket-observer.py --pid CURRENT_QT_PID \
      --duration 300 --output NEW_PRIVATE_PROOF_DIRECTORY/sockets.json

    python3 tools/live-network-observer.py --duration 300 \
      --output NEW_PRIVATE_PROOF_DIRECTORY/network.json
    ```

    Start them only after the scene and diagnostic readiness are established. Arm within 15 seconds of their start; otherwise retain a pre-arm failure and stop. During the 180-second capture, perform no gameplay input, screenshots, hashing, builds, tests, or configuration changes. The native observer supplies the fine-grained evidence; the existing observers provide independent coarse corroboration.

    **Observation:** the ledger already records successful use of the 300-second observer form; the feature procedure requires actual overlap to be checked afterward.  
    `prompt-exports/optimize-nucbox-latency-freezes-runs.md:289-301`  
    `.agents/skills/verify-gfn-experiments/features/latency-baseline.md:30-44`

12. **End at the first fixed checkpoint.**  
    Stop recording at 180 seconds, or finalize partial evidence if the worker/session ends sooner. Preserve the actual outcome. Allow the original bounded observers to finish. Do not extend into an idle tail, restart the stream, allocate another session, or run another window because no drops occurred.

13. **Validate, restore, and append the result.**  
    Verify source/artifact/process identity, diagnostic coverage, clock joins, actual observer overlap, socket-drop deltas, and resource closure. After normal GUI shutdown, restore the archived ordinary artifacts and preserved checkout. If a client was alive at handoff, the operator may restore its ordinary idle UI without starting another stream. Append sanitized numeric findings and private evidence hashes/paths to the ledger; preserve all previous bytes and invalid outcomes.

## The single subsequent optimization-selection rule

**Select at most one candidate only when this capture contains at least two distinct positive VIDEO-drop episodes, separated by at least one second, with complete clock/scheduling/socket coverage, and both identify the same specific mechanism in the same source-owned operation.** The operation must explain the dominant measured loss of receive-service opportunity, and code inspection must support one minimally invasive change without leaving equally plausible competing operations underneath the measured span.

Consequences of that one rule:

- Repeated CPU-heavy `process_datagram` spans alone do not identify SRTP versus FEC versus another internal operation.
- Repeated off-CPU spans without adequate state discrimination do not identify a lock or justify a scheduling change.
- Concentrated delivered arrivals do not reconstruct dropped-packet arrivals or justify a buffer change.
- A drop interval without worker/service correlation yields **no optimization**.
- One correlated episode, no drops, or incomplete coverage yields **no optimization** and an exact statement of the missing observable.

Any selected candidate is only a proposal after this checkpoint. A gain claim requires a separately defined matched control/candidate comparison within the same cloud session. VIDEO socket drops, component durations, Qt presentation intervals, and terminal stalls remain separate measurements; none is physical input-to-photon latency.

End of Oracle group: 2 lanes above.

## Coordinator reconciliation: compact diagnostic, actual Mjolnir owner, existing lineage

**Oracle reconciliation:** Astra completed; Opus failed streaming inactivity timeout, incomplete evidence, no duplicate call. Fresh delegated preflight was obtained afterward. Material claims/proposals:

| Claim/proposal | Source | Checked evidence | Disposition |
| --- | --- | --- | --- |
| Authorized normal source publication/target deployment can proceed after verification | Astra | New explicit user push/pull authorization | accepted |
| Nine-window evidence/socket drops real; cause/targetservice unproved | Astra | Ledger/private receipts previously checked | accepted |
| Local replay negative cannot transfer to NucBox or establish terminalcause | Astra | M3/synthetic/observer coverage limits | accepted |
| Preserve productiontimestamps/order/auth/feedback/reference/recovery semantics | Astra | Ownership contract and existing tests | accepted |
| Raw receive path alone is actual targetVIDEOowner | Astra placement assumption | Fresh retainedpreflight says Mjolnir branch; rtc delegates | rejected |
| Instrument actual identified VIDEO owner, distinguish raw/accepted/frameattempts | Astra general principle | Must bind actual Mjolnir socketreader/process owner first | accepted |
| Buildgate defaultOFF, optimizeddiagnostic notdebugslowdown, runtimeoptin, boundednumericdata | Astra | Cargo lacksdiagnosticfeature; verification discipline | accepted |
| SeparateSupportowner/helperfile; no hotpathperpacket IO/allocation/format | Astra | Performance/privacy requirements | accepted |
| recvmsg/timestamp/overflowadapter and10ms metadata would provide finerarrival evidence | Astra | UAPI/platform implementation not validated; changes receiveprimitive | unresolved |
| recvmsg replacement mandatory for firsttargetcheckpoint | Astra | Can obtain narrower usefulCPU/service/drop evidence preserving recv_from | rejected |
| Self perf context-switch capability is necessary before ANY target evidence | Astra | Installedtools absent; capabilityunproved; scope narrower than complete schedulingcausality | rejected |
| CPU wall distinction doesn't equal runqueue/preemption/blockingcause | Astra | Clock semantics; no kernel schedulingtrace | accepted |
|12files/64MiB/one-shotarm/schema11files/privateschedulerprobe required | Astra | Disproportionate to first bounded service observation | rejected |
|10ms bins/rarelongspans/counters/clockdomain/omission/capacity explicit | Astra | Needed interpretable numericbounded evidence, exactbudgets implementer | accepted |
| Newpublicationbranch omittingc10/b66 and rollback wholecheckout required | Astra | User authorizes current push/pull; reviewedlineage preserves fastforward | rejected |
| Inspect ancestry/full privatecommitdiff, preserve pendingwork/no broadstaging/no secretlogs | Astra | Remote cleanb66ahead2, localownedpendingwork | accepted |
| Exact current buildscript/features/outputreceipt must be verified; envonly maynotrebuild | Astra | Freshscript Release/Linuxfeatures/CMake-deploytrap verified bydelegate | accepted |
| Unsupportedbuildflag/noexistinghook means stop without narrowbuildowner fix | Astra | Requestedoutcome permits focused localNativeRuntime gate ifrequired | rejected |
| Ordinaryrelease stripsdiagnostics; optionalLinuxlibc lockededge only ifneeded | Astra | Actualmanifest/buildbinding implementer toverify | accepted |
| Source privacy c10 peertext/log sink unsuitablehotpath, rawprivate | Astra | Existingloggingreview andprivate-only evidence | accepted |
| Focused semantic/accounting/lifecycle/clock checks, overheadquantification | Astra | Required for bounded diagnostic; reusemeaningfulfixtures | accepted |
| Arbitrary5/10percent overhead gates prove targetnonperturbation | Astra | Local vsLinux gap; quantifyandreport, targetcheck beforelive | unresolved |
|180sec diagnostic no repeatedbaseline/idle/antiidle, soleoperatorCua | Astra | User/skillauthorization, existing observers; actualactivecapturedeadline only | accepted |
| Arbitrary15sec arm gate/5sec timeout cancels advancingjobs | Astra possible operational interpretation | User healthyoriginalprocess preservation | rejected |
| Two positiveepisodes separated1sec mandatory beforeanycandidate | Astra | Fixed arbitrarycount can prevent evidence-grounded rootcause; strength judged bymechanism | rejected |
| Candidate requires concrete source-owned mechanism andmatchedsamecloudsession proof forgain | Astra | Performance attribution contract | accepted |
| ExistingoriginalCua stdio/operatorhistoricallyready | Neither currentclaim | Freshpreflight runtimeexited/Unknown91787; preservedconfig canrestore later | rejected |
| HerdrcurrentClaudeDONE canbeassumed | Neither currentclaim | Genuinepane prerequisite unavailable; stateunverified | rejected |

Execute compact LOCAL preparation first: reconcile/import reviewedcode-only c10/b66 lineage withoutforce/losingownedwork; implement minimum opt-in support timing in ACTUAL Mjolnir VIDEO owner, source-based buildfeaturegate ifneeded, meaningfultests/ordinaryoptimizedbuild/limitedoverheadcheck. NO network/rmem/profile/policy changes, no bigarmingframework or perf prerequisite. Wall/CPU/service and bracketed socketdrops are narrowerdiagnostic, never complete schedulercausality. Stopbeforepublication for requiredreview. Then samepair may commit/push scopedcode afterreview, remoteffpull/build with actualdeploymentverification, restore supported Cua config asneeded (oldprocessabsent, no replayoflivecontroller), freshsoleoperator/profilescenecheck, oneboundedtargetcapture. Capture neednotproveallcauses; it must move actualservice/CPU/nonCPU attribution and explicitlyretain gaps. No genuineHerdrcontext spoof orunrelatedfocusedinput.

Owner binding resolved by implementation agent: core starts ICE/DTLSbundle separately; spawn_nvst_mjolnir_receiver explicitly passes rtc:None and namesopennow-nvst-mjolnir, reaching run_nvst_udp_receiver/recv_from/process_datagram/forward_receive_event. Thus hooks in the sharedrawloop cover the actualdedicatedMjolnir VIDEOowner; onlybundle delegates. The earlier raw-only objection was an unresolvedplacement assumption, notproof ofwrongloop. Accepted subject to parent source spot-check beforedeployment.



> 💡 The 2 Oracle lanes above are independent answers; reconcile them using the guidance at the top of the group. Lane order is not a ranking.

Optional later follow-up: Continue this plan conversation with ask_oracle(chat_id: "nucbox-target-side-freez-0C3867", new_chat: false)