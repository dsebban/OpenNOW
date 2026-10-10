# Oracle Review

## Oracle group
- Group ID: `D48D50FB-8B66-4637-B50A-3B9DB0208FDD`
- Status: `completed`
- Oracle count: 2

**Reconciling these Oracle lanes**
2 independent answers to the same request follow. Lane order is not a ranking; the first lane supplies the top-level continuation handle, and a successful follow-up through any lane's chat ID re-runs every lane.
- Read every lane through the end-of-group marker (`End of Oracle group: 2 lanes above.`). If the marker or a lane is missing, page the export or try the read-only `oracle_chat_log` with that lane's chat ID. Logs may be scoped or clipped; report any remaining gap. Do not start a follow-up just to retrieve prior text.
- Reconcile by evidence, not lane order, answer length, or model identity: check material single-lane and conflicting claims against the code, and report unresolved disagreements. A failed or partial lane is incomplete evidence.
- Before synthesizing, inventory every material claim from every lane, including single-lane claims. Merge only exact duplicates, retaining all source lanes. Begin your answer with `**Oracle reconciliation**`, state how many lanes completed and name any that did not. For every inventory item, give its source lanes, checked evidence, and exactly one disposition: `accepted`, `rejected`, or `unresolved`. Never silently omit an item.

Lanes (2):
- Oracle — `claude-opus-5-5-xhigh` — completed — chat ID `new-chat-771557`
- Oracle 2 — `gpt-6-astra-pro` — completed — chat ID `new-chat-oracle-2-2795AC`

## Oracle results

### Oracle (Primary)
- Lane index: 0
- Role: `primary`
- Chat ID: `new-chat-771557`
- Provider: _Not specified._
- Model: `custom_provider_claude-opus-5-5-xhigh`
- Status: `completed`
- Execution provider: `Custom`
- Execution model: `claude-opus-5-5-xhigh`

#### Response



# R1 review and continuation decision

## Summary

R1 adds a test-only harness: a 5-line declaration nested inside the existing `#[cfg(test)] mod tests`, plus a 689-line child module. The harness drives the real authenticated `process_datagram → forward_receive_event` path over fixed 50/100 Mbps paced, burst and repair fixtures, then feeds a no-loss queue estimator.

What holds up:
- **Estimator correction is right.** It now samples waiting bytes at frame boundaries, excludes the in-service packet, and orders equal-time events correctly. The lag-only regression test separates completion lag from queue growth.
- **Production exclusion is sound.** The module is nested under `cfg(test)`, and the release rlib was byte-identical after a forced recompile.
- **Semantic coverage is real.** Tests check exact payload, identity, keyframe and continuity, plus RTP rollover, reorder, real Cauchy repair, and 8-frame backpressure and closure.
- **The scoped negative is supported.** On M3 Max, steady per-packet service produces no frame-boundary backlog and no sustained growth.
  - All 10,800 boundary samples are 0.
  - p50 is stable within about 0.17 µs across 18 replays.
  - 100/Burst peaks of 65–71 packets match a hand estimate: 174 × (1 − 5.75/8.5) plus ~21 packets for the ~181 µs frame-completing packet ≈ 77.

What does not hold up is the ledger's treatment of the tail. The matrix-maximum backlog comes from millisecond `process_datagram` stalls that recur at the same case and repetition positions in both independent matrix runs. The harness cannot identify which packet or code path produced them. That is the one material gap, and it is also the most concrete lead for the rank-1 hypothesis.

## Findings

### P1

**Recurring millisecond stalls in 100/Repair are not localized and are attributed to "possible local descheduling"**

- **Location:** `receive_replay_tests.rs`, `replay()` and `serial_receive_cost_report`. These emit only distributions, with no identity for individual samples. Also the ledger's "R1 correction" outlier paragraph.
- **Problem:** The largest service samples recur by repetition position across two separate matrix runs, which is not what random descheduling looks like. They also produce the matrix's largest modeled backlog. The timed region contains only production calls (`process_datagram`, `forward_receive_event`, and the drop of the event Vec), so a recurring stall is either production code or the host environment. The harness cannot tell which.
- **Evidence** (from the ledger tables, invalid run → corrected run):

  | 100/Repair | Max service, invalid run | Max service, corrected run |
  |---|---|---|
  | rep 1 | 12.848 ms | 9.360 ms |
  | rep 2 | 0.321 ms | 0.328 ms |
  | rep 3 | 2.374 ms | 2.432 ms |

  - Rep 3 also repeats a frame-completing max of 1.303 → 1.364 ms, and a p99 of 18.8 → 18.4 µs, versus 11–14 µs in the other 100 Mbps rows.
  - The pattern is specific. 50/Repair stays at or below 0.215 ms in both runs, and 100/Burst outliers do not recur (rep 2: 0.660 → 0.220 ms).
  - Rep 1's total process time exceeds rep 2's by ~14 ms, which fits about one event per replay rather than a cost paid on each of the 10 repairs.
  - These samples produce the matrix maxima: a 148-packet peak in 100/Repair/3 (85 % of the 174-packet criterion) and a 10.546 ms wait in 100/Repair/1. The peak depends on where the stall falls relative to the 1 ms burst, so the margin under the criterion is partly luck.
  - **Inferred, not proven:**
    - Every repetition builds a fresh receiver, yet rep 2 is clean in both runs. That points to process-global or wall-clock-dependent state, such as a rate-limited or lazily initialized diagnostic, rather than a per-receiver cost.
    - The only packet class unique to 100/Repair is the surplus second parity shard arriving after reconstruction (172 data + 2 parity, versus 86 + 1 at 50 Mbps).
- **Fix** (test-only, about 30 lines):
  1. Add `shard: usize` and `parity: bool` to `ReplayPacket`.
  2. In `replay()`, keep the 5 slowest samples. For each, record packet index, frame, shard, parity flag, RTP sequence, wall-clock µs since replay start, the process/forward split, and a process-global replay ordinal.
  3. Emit these as `slowest_samples` in each report.
  4. Rerun the fixed matrix once.
  5. Change the ledger wording to "unexplained; recurs at 100/Repair reps 1 and 3 across both runs".

### P2

- **The verdict's scope is partly fixed by how the fixture is built.**
  - Every frame has the same size, and keyframes differ only by NAL type. With no stall and no mean overload, waiting can never exceed one frame (≤ 174 packets). If the B1R rxQueue quantum of 2304 B per queued datagram holds (inferred), the video socket's capacity is about 184 packets. Under this arrival model, overflow therefore needs either a stall spanning more than about a frame of arrivals, or frames larger than the socket's capacity. That makes the 174-packet criterion effectively a stall detector, and the growth criterion detects only mean overload.
  - The ledger's non-coverage list also omits three things:
    - Larger IDR/keyframe bursts.
    - Feedback-lock contention with the consumer and feedback threads (the harness is single-threaded).
    - The ~1.67 Gbps instantaneous burst rate, which exceeds GbE line rate and is a synthetic choice.
  - **Fix:** add these lines to the ledger. No code change.
- **`process_datagram` is not broken down.** It accounts for about 99.6 % of service at a median of 8.4–8.5 µs. The report cannot separate the SRTP/AES-GCM backend, which differs between aarch64 M3 and the x86_64 NucBox, from post-authentication work, so the result does not transfer to the target.
  - **Fix:** in the same rerun, time `second_receiver.srtp.unprotect(&datagram)` over the same sequence using a separate `NvstVideoReceiver::new(replay_config())`, and emit an `unprotect` distribution.
- **Minor code points:**
  - The inner `#[cfg(test)]` is redundant inside the `cfg(test)` `mod tests`.
  - The new `src/nvst/` path breaks the sibling `nvst_*_tests.rs` naming convention.
  - `Vec::with_capacity(frame_finishes.capacity())` always reserves 0.
  - The handoff-lag loop reimplements the `estimate_backlog` recurrence.
  - The hardcoded `authentication_failures/unexpected_drops/consumer_full: 0` values are guaranteed by asserts but look like measured fields; label them as assert-enforced.
  - `host_cpu` reports "unavailable" on Linux. Reading `/proc/cpuinfo` `model name` would let a future on-target run identify its host.

## Continuation decision: CONTINUE with checkpoint R1b

R1b localizes the recurring stall, then lands exactly one fix if the cause is a production call. Otherwise it stops.

Why this move:
- **The structural service line is closed.** R1 is a valid scoped negative.
- **Throughput-style changes are barred.** Decoupling drain from processing would move the drop point and timing, which counts as a forbidden policy change.
- **Per-packet micro-optimization has no target yet.** There is no identified hotspot.
- **This is the one reproducible lead.** The stall is the only reproducible receive-thread stall in hand, it matches the rank-1 mechanism (a serial loop plus a millisecond stall leads to kernel queue overflow), and it can be localized locally in minutes.

**Owner and files:** the local agent. Changes go to `src/nvst/receive_replay_tests.rs` (test-only) and an append to the ledger. If the cause is localized, add exactly one production file, whichever owns the path.

**Steps:**

1. **Zero-cost check.** Scan the retained `release-matrix-corrected.log` and `release-matrix.log` for any lines other than `RECEIVE_REPLAY_JSON`, `RECEIVE_REPLAY_DISPOSITION`, and libtest framing. Record only their counts and positions relative to the 100/Repair rep 1–3 reports. A missing line does not rule anything out, because `log_async` may write elsewhere.
2. **Harness additions.** Add the P1 identity fields and the P2 `unprotect` split, then rerun the identical fixed matrix once.
3. **Decision rule, fixed before the run:**
   - **LOCALIZED:** every sample over 1 ms, in at least 2 of 3 reps of a case, shares one packet class and code path. Then:
     - **Reachability check:** confirm the path is reachable in production, using retained B1R `nvst-video` counters (read-only) or a code proof.
     - **Allowed fixes:** if the path is a synchronous log or diagnostic write, lazy initialization, a state-proportional allocation, rehash or scan, or a syscall, implement one change that removes it from the per-packet path. Examples are reporting it as a counter on the existing 10 s `nvst-video` line, or pre-sizing the structure. Event, drop, recovery, NACK and reference outputs must stay identical.
     - **Recording:** log this as optimization iteration 1 (local), with live status NOT VERIFIED.
   - **NOT LOCALIZED:** the slow samples have different identities across reps, or no shared class. Record the stall as environmental on the dev host, then **STOP_UNRESOLVED_DEPLOYMENT_BLOCKED**. This is not a plateau. The missing observable is the per-iteration receive-thread service time and the largest gap between successive `recv_from` calls on the NucBox, during a window with nonzero video-socket drops. Two ways to get it:
     - Run this replay test binary on the NucBox. That requires an upload, so it needs approval.
     - Ship a counters build. That needs publishing approval.
   - **LOCALIZED in policy code** (recovery, drop, keepalive, or reference handling): report it and STOP. Do not fix.

**Done when:**
- The fixed matrix shows no sample over 1 ms of the localized class in any of the 3 reps of any case.
- 100/Repair max service is at or near the rep-2 level (~0.33 ms), with max modeled wait around 1 ms or less.
- All 306 existing tests still pass, plus one regression test that pins semantic output for that packet class.
- Formatting checks and `git diff --check` are clean.

**Commands** (cwd `/Users/danielsivan/dev/OpenNOW`):

```sh
M=native/opennow-streamer/Cargo.toml
cargo test --locked --manifest-path $M -p opennow-streamer-transport --lib
cargo test --locked --release --manifest-path $M -p opennow-streamer-transport --lib \
  nvst::tests::receive_replay_tests::serial_receive_cost_report -- --exact --ignored --nocapture --test-threads=1
cargo build --locked --release --manifest-path $M -p opennow-streamer-transport
# Only if a production file changed:
cargo build --locked --release --manifest-path $M -p opennow-streamer-ffi
```

**Expected metric vs risk:**
- **Local:** removes a 2–9 ms receive-path stall.
- **Live:** benefit is unknown and limited to transient overflow on the localized path. It does not address the T1-type gap, which had 0 socket drops, or the terminal freeze ~601 s after input, which remains a separate unresolved item.
- **Risk:** low for a diagnostic or allocation change. It is guarded by the exact-payload harness and the existing suite.

**Attribution:**
- Use one isolated commit on top of the R1 test commit. The local 6472 transport source is identical to c10's, per sibling evidence.
- Any live evaluation needs publishing approval, a new artifact, and a future matched block with interleaved control. It must never be pooled with B1R, and it stays NOT VERIFIED until then.

Verdict: Approve with fixes — the test-only harness is correct and the scoped negative holds, but the recurring, unlocalized 100/Repair stalls must be identified before R1 can rule out the receive path.

### Oracle 2
- Lane index: 1
- Role: `additional`
- Chat ID: `new-chat-oracle-2-2795AC`
- Provider: _Not specified._
- Model: `custom_provider_gpt-6-astra-pro`
- Status: `completed`
- Execution provider: `Custom`
- Execution model: `gpt-6-astra-pro`

#### Response

## Summary

The changes add a test-only replay of authenticated `process_datagram` → `forward_receive_event` processing, exercise assembly, rollover, reordering, FEC and consumer backpressure, and measure service demand against fixed arrival schedules. The corrected estimator separates **waiting queue volume** from **frame-completion lag**, addressing the original false positive. **The supplied implementation and corrected results support `NOT_REPRODUCED_ON_LOCAL_HOST` for the stated synthetic workloads and decision criteria.** This is a static review of the supplied diff, source and ledger; I did not execute commands or independently inspect the retained binaries and reports.

## Findings

**No findings requiring changes in the supplied diff.**

The important checks are:

- **The semantic workload exercises the intended processing path.** In `receive_replay_tests.rs::replay`, every supplied datagram must increment `authenticated_packets` by exactly one, unexpected receive events fail the replay, and delivered frames are checked for complete payload, identity, timestamp, codec, keyframe status and continuity. Reconstruction is checked against the intact payload and the repaired-packet counter. The separate full/closed consumer test exercises the actual handoff function and verifies both the preserved discontinuity after backpressure and delivery accounting. These are meaningful correctness checks around the measurement.

- **The corrected queue calculation matches its stated definition.** In `estimate_backlog`, the serial-service recurrence determines packet start times; the arrival and start pointers then count packets that have arrived but have not started. Prefix sums independently account for their datagram bytes. At frame boundaries, arrivals exactly on the boundary are excluded and already-started packets are excluded, including the packet in service. The lag-only regression and contrasting sustained-overload case directly test the distinction that invalidated the first report.

- **The negative disposition follows from the corrected results.** The ledger’s **“R1 correction — waiting-volume decision arithmetic”** reports zero waiting packets and bytes at all 10,800 actual frame boundaries, so the growth condition is false. Every extra peak is also below its load-specific threshold: the largest is 148 packets against the 174-packet threshold at 100 Mbps. Consequently, no case meets the required two-of-three repetition rule. The nonzero transient backlog remains visible; this result does not mean that queues never formed.

- **Timing and causal claims remain appropriately bounded.** The measured interval covers the real processing and forwarding calls. Validation, draining and fixture construction occur outside it. The model deliberately does not replay protocol decisions using its calculated delayed service times. Those boundaries make this a service-demand experiment, rather than a complete receive-worker simulation. The reported 9.360 ms noncompleting, non-reconstruction sample cannot identify FEC cost or distinguish CPU work from local descheduling. The ledger preserves that limitation.

- **Production exclusion and verification are sufficient for this change.** The only existing-source modification is the five-line module declaration inside `nvst::tests`; the entire child module is excluded from ordinary builds. The supplied ledger reports 306 passing transport tests, one ignored report test, successful execution of the corrected 18-replay matrix, and a fresh ordinary release compilation yielding the same library bytes. Together with the visible `#[cfg(test)]` boundary, that is adequate verification for this scope. An additional FFI or GUI build would not resolve a remaining risk introduced by this diff.

The original `INVALID_DECISION_REPORT` remains historical evidence, separately retained from the corrected run. Its obsolete lag-based decision is not a remaining defect in the selected implementation.

## Continuation decision

**STOP at the corrected R1 checkpoint — accept the implementation and retain the bounded local negative. The target receive-servicing mechanism remains unresolved; no plateau or optimization gain has been established.**

**The single next move is for the coordinator to close R1 and record the specific target observable required before selecting a receive optimization.** Another local matrix would not supply it.

That missing observable is:

> **A time-correlated trace for the NucBox VIDEO socket showing queued arrivals or drops, receive-service timing, and the identified receive worker’s execution/scheduling state across the same interval.**

It must distinguish time spent doing inline processing from time when the worker cannot run, and from an arrival burst exceeding available receive capacity. A long interval between successful receives alone is insufficient because the socket could have been waiting for incoming traffic. A sparse queue sample taken after the event is also insufficient.

The established **168 VIDEO-socket drops and 425,984-byte effective buffer** identify the affected boundary. They do not identify which of those mechanisms caused it, and the replay’s datagram-byte counts cannot be converted into an exact kernel-buffer capacity or drop prediction. Likewise, source equivalence between the local transport and c10 supports the logic comparison, while the M3 Max measurements remain host-specific. These limits are explicitly retained in the supplied R1 identity and corrected-result sections.

**Ownership and scope:** the coordinator owns this checkpoint closure; pending `AGENTS` changes and other exports remain outside this review. The original Cua operator remains the appropriate owner for a subsequent authorized target capture. The supplied record does not establish an existing tracing capability that provides the required observable, so there is no sufficiently specified target experiment or production change to approve here. Foreground Cua authorization remains available; the present blocker is mechanism evidence.

Once that evidence supports one concrete candidate, a live comparison will require a fresh matched control/candidate block and the required publication/deployment approval. The ended B1R allocation cannot serve as its current control. **Optimization iterations remain zero, live control/candidate remains `NOT_VERIFIED`, and the separate post-scoring terminal silence remains unresolved.**

Verdict: No findings — the corrected test-only implementation supports its stated local negative, while the evidence does not yet justify a target receive optimization.

End of Oracle group: 2 lanes above.

## Coordinator reconciliation: bounded R1b, no automatic production fix

**Oracle reconciliation:** Both reviewers completed. Parent inspected all689 support lines,5line hook,18 corrected numeric reports; searched original/corrected ledger tables to confirm recurring tail magnitudes. Material inventory:

| Claim/proposal | Lanes | Checked evidence | Disposition |
| --- | --- | --- | --- |
| Semantic/auth/payload/FEC/identity/backpressure checks meaningful | Both | Complete diff and authenticated/frame/repair counters | accepted |
| Corrected waiting-volume estimator/equality/in-service logic sound | Both | Full implementation and lag-only/overload regression | accepted |
| cfg(test) exclusion/release compile sufficient | Both | Visible hook and compile/hash receipts | accepted |
| Bounded NOT_REPRODUCED_ON_LOCAL_HOST follows fixed criteria; transient queues retained | Both |18 reports, all boundarywaiting0, max148<174 | accepted |
| No target receive mechanism or gain earned; source equivalence not CPU equivalence | Both | M3 host and endedB1R/live limits | accepted |
| Millisecond tails recur in100Repair rep1/3 across two runs and lack packet identity | Opus | Ledger12.848→9.360ms and2.374→2.432ms; reports aggregate only | accepted |
| Recurrence is not random descheduling and identifies production path/global state | Opus inference | No packet identity or scheduling observation | unresolved |
| Surplus parity uniquely causes stalls | Opus hypothesis |172data+2parity topology differs; no event identity | unresolved |
| Top5 sample identity/phase/ordinal and separate unprotect timing could localize bounded lead | Opus | Actual repeated local tail; existing support can expose metadata only | accepted |
| Larger keyframes/contention/arrival model/crypto backend omitted from local transfer | Opus; Astra general limit | Fixedsize singlethreaded synthetic fixtures;1ms174packetburst | accepted |
| Redundant cfg/path/reservation/recurrence/hostCPU niceties require cleanup | Opus | Nonmaterial; scope smallest requested change | rejected |
| Assert-enforced zeros should be labeled; absent log line not causal exclusion | Opus | Current report hardcoded after assertions, log_async uncertainty | accepted |
| A shared class for>1ms samples alone proves code cause and permits immediate fix | Opus | Correlation alone cannot distinguish descheduling/work | rejected |
| Varying identities or failure to localize proves environmental cause | Opus | Unobserved scheduler/allocation/locks remain alternatives | rejected |
| Stop R1 with specific target timing/drop/scheduler observable required | Astra | Local synthetic negative cannot explain NucBox incident | accepted |
| Another identical matrix with no added discriminator would help | Neither | No new information | rejected |
| Continue R1b versus stop now | Opus CONTINUE; Astra STOP | Repeated tails unlocalized; bounded identity discriminator available | unresolved |
| Target queued/dropped arrivals plus service/scheduler state needed for target attribution | Astra; Opus deferred |168video drops/buffer identify boundary only | accepted |
| New deployedartifact/matchedsame-sessioncontrols/approval needed forlivegain; zeroiterations | Both | Skill/endedallocation/source split | accepted |

Choose ONE bounded **test-only R1b tail localization** on the already reproducible lead. This is not an identical baseline: add top5 packet/frame/shard/parity/sequence/ordinal/process-forward metadata and independent real-unprotect timing, label assert-enforced zeros, retain model/fixtures. Read retained log counts/positions only. Run unchanged fixed matrix ONCE. Preserve original/corrected artifacts. Do not re-review, refactor shipping code or automatically fix based on shared class; return exact evidence and any source-backed mechanism to coordinator. Lack of localization yields STOP_UNRESOLVED_TARGET_EVIDENCE, not environmental proof or plateau. Source/policy/receive counters/deployment/network/profile untouched. Append explicit workload/target limitations. Existing R1 boundednegative remains valid. No extra work beyond localization checkpoint.