# Oracle Plan

## Oracle group
- Group ID: `EED14EDD-630B-4D76-9D91-F943C64CDB6C`
- Status: `completed`
- Oracle count: 2

**Reconciling these Oracle lanes**
2 independent answers to the same request follow. Lane order is not a ranking; the first lane supplies the top-level continuation handle, and a successful follow-up through any lane's chat ID re-runs every lane.
- Read every lane through the end-of-group marker (`End of Oracle group: 2 lanes above.`). If the marker or a lane is missing, page the export or try the read-only `oracle_chat_log` with that lane's chat ID. Logs may be scoped or clipped; report any remaining gap. Do not start a follow-up just to retrieve prior text.
- Reconcile by evidence, not lane order, answer length, or model identity: check material single-lane and conflicting claims against the code, and report unresolved disagreements. A failed or partial lane is incomplete evidence.
- Before synthesizing, inventory every material claim from every lane, including single-lane claims. Merge only exact duplicates, retaining all source lanes. Begin your answer with `**Oracle reconciliation**`, state how many lanes completed and name any that did not. For every inventory item, give its source lanes, checked evidence, and exactly one disposition: `accepted`, `rejected`, or `unresolved`. Never silently omit an item.

Lanes (2):
- Oracle — `claude-opus-5-5-xhigh` — completed — chat ID `nucbox-latency-and-freez-F7454F`
- Oracle 2 — `gpt-6-astra-pro` — completed — chat ID `nucbox-latency-and-freez-4A7D70`

## Oracle results

### Oracle (Primary)
- Lane index: 0
- Role: `primary`
- Chat ID: `nucbox-latency-and-freez-F7454F`
- Provider: _Not specified._
- Model: `custom_provider_claude-opus-5-5-xhigh`
- Status: `completed`
- Execution provider: `Custom`
- Execution model: `claude-opus-5-5-xhigh`

#### Response



# NucBox loop: STOP_BLOCKED on the Cua Linux input backend

## 1. Summary

**Decision: `STOP_BLOCKED`.** It is not `PLATEAU` and not `TARGET_MET`.

- **Why there is no next performance move.** Every measurement path depends on starting or resuming Ori through the normal GUI using Cua. The installed Driver's Linux input backend (RemoteDesktop/EIS via libei) is not ready.
- **Why the semantic route cannot substitute.** The AT-SPI route works, but the current tree exposes no Start control.
- **Why it cannot be fixed inside this scope.** No documented, non-retry recovery exists within the authorized tool boundaries.
- **What is required to resume.** A separately scoped Driver/EIS input-readiness repair is the explicit external prerequisite. Performance iteration stops here.
- **The only action this step authorizes** is one append-only ledger entry. There are no host, GUI, Cua, Doctor, code, build or test actions.

## 2. Evidence

Labels: **[obs]** = direct observation; **[inf]** = inference from the cited observations.

| Claim | Label | Citation |
|---|---|---|
| Scoring requires a normal saved-account GUI start/resume. The collector "never launches… or sends input". GUI actions must use Cua Driver. | obs | `.agents/skills/verify-gfn-experiments/features/latency-baseline.md:5-9`, `:14-15`, `:32-37` |
| B1: foreground Start returned `foreground_timeout`/effect none. A delayed screenshot showed no change and there was no replay. Result: 0 windows, INCOMPLETE. | obs | `prompt-exports/optimize-nucbox-latency-freezes-runs.md:178`, `:184-186`, `:189` |
| H1: the 32-node AT-SPI tree has no Start/Play/Resume. Background Return was refused. Foreground Return failed with `libei input backend did not become ready within 20s`. | obs | `…runs.md:200` |
| `health_report` does not exercise RemoteDesktop→EIS. No documented libei reset/reattach or input-grant tool exists. Whether the cause is missing consent or a wedged EIS is unresolved. | obs | `…runs.md:201-202` |
| H1's stated prerequisite: a completed RemoteDesktop grant/EIS handshake, or a truly exposed semantic Start. | obs | `…runs.md:202` |
| App-side rejection is unproven because no persistent core activation/request sink exists. | obs | `…runs.md:199` |
| Foreground Cua input succeeded earlier the same day (pixel action at S1; `press_key` before A1). AT-SPI is proven only for an exposed button (Quit). | obs | `…runs.md:47`, `:58`, `:50`, `:202` |
| Foreground input readiness regressed between the A block (~12:27) and B1 (~13:37). The cause is unresolved and is not shown to be an app defect. | inf | from `:47`, `:58`, `:62`, `:179`, `:200-201` |
| No candidate has been selected or measured, and repeatability is INCOMPLETE. | obs | `…runs.md:83`, `:124`, `:189` |
| Therefore `PLATEAU` (needs a measured, attributable non-improving change) and `TARGET_MET` (needs a valid matched baseline plus a gain) are both unearned. | inf | from the row above |
| Deploying a product change requires local review, tests and publishing approval. That approval has not been granted. | obs | `…runs.md:97` |
| Final state: the client was preserved idle and ownership was released at 13:57:19.923705Z (`release-final.json`). No observers or collectors are outstanding. | obs | `…runs.md:203` |
| The Herdr managed-pane prerequisite is unmet and spoofing it is prohibited. | user-supplied | — |

Measurement status, unchanged:
- A1 descriptive Qt fresh-interval upper bounds were 21/24/56 ms, with 88 of 10,915 intervals late (`…runs.md:79`).
- A2 is INVALID and retained. D1 is STAGE_ONLY (`…runs.md:119`, `:124`).
- These are component values, not input-to-photon latency.

## 3. Design

### 3.1 Candidate next moves screened

| Candidate | Verdict | Reason |
|---|---|---|
| Retry Start/Return, or revive the Cua session | Rejected | Already failed with an explicit backend error (`:200-201`). Excluded by the user. |
| Background AT-SPI activation of Start | Rejected | No Start node exists (`:200`). |
| Core/account RPC session start | Rejected | This is an input bypass (B1 action exclusions, `:193`). |
| Driver upgrade, daemon restart or portal reset | Rejected | Undocumented (`:201`) and risks the shared session. |
| Read-only portal/journal diagnosis | Out of loop | Not a documented skill route. It only informs the repair, so it belongs to the repair task. |
| Local control-close diagnostic, or exposing Start accessibly | Out of loop | Cannot be validated live, and deployment is unapproved (`:97`). An accessible-Start change would be an alternative repair path only with separate product approval. |
| Herdr route | Rejected | Prerequisite unmet; spoofing prohibited. |

### 3.2 External prerequisite and resume gate (owned outside this loop)

The performance loop may resume only when **all** of the following hold:

1. **R1, input path repaired.** Either:
   - (a) a repair-owned receipt shows the installed Driver completed the RemoteDesktop grant and EIS handshake, proven by one foreground Cua action with a visible effect on a fresh snapshot and no `tool_invocation_failed`/`foreground_timeout`; or
   - (b) a separately approved and deployed change exposes Start semantically, and it is activated through the AT-SPI route.
2. **R2, fresh identity.** Doctor reports `ready_for_gui`, with a fresh readback of client identity and hashes. Do not infer readiness from the idle Qt1788545/core1788748 sleep state.
3. **R3, ownership.** A fresh sole GUI/account handoff to the performance operator.

The first post-gate step is a fresh label **B2** under B1's reconciled scope (`…runs.md:175`):
- at most 3 same-session attempts, each warmup 20 s, 60-s windows, 3 windows;
- 300-s observers and no idle tail;
- one timestamped pause/resume outside scoring per attempt.

The control-close diagnostic remains a later, separately approved proposal.

### 3.3 Scoreboard entry (append once)

```md
### E1 — STOP_BLOCKED: Cua Linux input backend not ready — <UTC append time>
- End condition: STOP_BLOCKED (not PLATEAU/TARGET_MET). No candidate selected, measured or landed.
- Basis: B1 foreground Start foreground_timeout/effect none, 0 windows; H1 foreground Return tool_invocation_failed (libei not ready within 20 s), background refused, 32-node AT-SPI tree lacks Start/Play/Resume; health_report does not exercise RemoteDesktop→EIS; no documented reset; consent vs wedged EIS unresolved; app rejection unproven. Earlier same-day foreground input succeeded (S1/S4); regression cause unresolved.
- Measurement (unchanged): A1 component upper bounds 21/24/56 ms, 88/10,915 late, not input-to-photon; A2 INVALID; D1 STAGE_ONLY; A repeatability INCOMPLETE; B1 0 windows; H1 BLOCKED_CUA_LIBEI_NOT_READY.
- External prerequisite: separately scoped Driver/EIS input-readiness repair (R1a) or approved semantic Start exposure (R1b). Resume gate R1–R3; first step fresh B2 per B1 reconciled scope.
- State: last verified Qt1788545/start17752599, core1788748/start17752687 idle; ownership with coordinator since 13:57:19.923705Z; no fresh readiness asserted.
- Receipts: A remote .runtime/nucbox-baseline-20261008-Sth9bYSs/ + local /private/tmp/nucbox-baseline-Sth9bYSs/; build .runtime/verification-deploy-tgLBRD4n/build-receipt.json; D1 private D1-freeze-attribution.md (SHA256 18a18ed5…915fa); B1 .runtime/verification-b1-qyUq9sDz/ + /private/tmp/nucbox-b1-qyUq9sDz/; H1 .runtime/verification-h1-PuZJo7g6/ (release-final.json) + /private/tmp/nucbox-h1-PuZJo7g6/.
- Actions: ledger append only; no host/GUI/Cua/Doctor, code, build/tests, network/display/profile, commit/push/publish.
- Append verification: preceding <n> bytes / SHA256 <h> unchanged.
```

Receipt pointers come from `…runs.md:87`, `:102`, `:113-114`, `:118`, `:171`, `:190`, `:203-204`.

## 4. File-by-file impact

| Path | Change |
|---|---|
| `prompt-exports/optimize-nucbox-latency-freezes-runs.md` | Append the E1 entry only. No other file changes. |

## 5. Implementation order

1. Record the ledger's byte length (`wc -c`) and SHA256 (`shasum -a 256`).
2. Append E1 with `cat >>`.
3. Confirm the prefix is unchanged: `head -c <n>` of the ledger must reproduce the hash from step 1. Fill both values into E1's verification line.
4. Hand the R1 prerequisite to the coordinator or user, then end the performance loop.

### Oracle 2
- Lane index: 1
- Role: `additional`
- Chat ID: `nucbox-latency-and-freez-4A7D70`
- Provider: _Not specified._
- Model: `custom_provider_gpt-6-astra-pro`
- Status: `completed`
- Execution provider: `Custom`
- Execution model: `gpt-6-astra-pro`

#### Response

## Decision — **STOP_BLOCKED**

**Inference:** Stop the performance iteration with reason **`CUA_LIBEI_NOT_READY`**. Neither **plateau** nor **target met** is supported: B1 produced no scored windows, and H1 demonstrated failure to initialize the authorized input backend. **No new, supported, bounded host action is established by the supplied evidence.** Lifting the iteration cap does not satisfy this missing prerequisite. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:178-189`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:200-202`)

## Evidence

These are **direct observations recorded in the frozen ledger**, not fresh host checks:

- **B1 never reached gameplay.** The timed-out foreground Start action had no subsequently visible effect. Allocation and negotiated workload were unverified; T1–T3 remained `NOT_STARTED`, with **zero windows**. There is no new baseline to assess. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:178-189`)
- **H1 identified a concrete input prerequisite failure:** `libei input backend did not become ready within20s`. The complete accessibility tree exposed no Start/Play/Resume control. Driver health did not exercise the RemoteDesktop/EIS handshake; no consent dialog or documented reset, reattach, or prompting-grant tool was available. Consent versus wedged EIS remains unresolved. The original B1 activation/core-request outcome also remains unproven. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:199-202`)
- **Historical performance evidence retains its existing status.** A1 completed three descriptive windows, with pooled fresh-interval histogram upper estimates **p95/p99/max = 21/24/56 ms**. A2 remains `INVALID` with retained freeze evidence; D1 remains `STAGE_ONLY`. These component measurements establish no physical input-to-photon result. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:61-79`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:119-124`)

## One external prerequisite; no current executable recovery

**Recommendation, inferred from H1:** A **separately scoped Driver/RemoteDesktop/EIS setup repair** is required before performance work resumes. Its recovery command cannot honestly be specified from the supplied installed guidance: H1 found no supported recovery interface. **Current host probe/command: none justified.** (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:201-202`)

The resumption gate is a repair receipt demonstrating usable input readiness, followed by fresh sole-owner handoff and a supported foreground Cua action that visibly reaches loading/gameplay. That verification belongs **after an established repair**, not another unchanged Start/key/session-revival attempt. `health_report` or `ready_for_gui` alone cannot pass the gate. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:198-202`)

No performance code or control-close instrumentation should be landed in this iteration. **Inference:** control-stop diagnostics could improve a later streaming investigation, but cannot resolve the demonstrated pre-gameplay input failure.

## Closure, receipts, and scoreboard

**Direct observation:** H1 recorded the original Qt/core instances preserved idle and ownership released at **2026-10-08T13:57:19.923705Z**, with no outstanding observers or collectors. That receipt establishes the handoff state; it is not present readiness proof. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:198-204`)

Existing receipt pointers:

- H1 final handoff: `/home/gamer/dev/gfn-client-research/.runtime/verification-h1-PuZJo7g6/release-final.json`
- H1 sanitized evidence: `/private/tmp/nucbox-h1-PuZJo7g6/`
- B1 execution proof: `/home/gamer/dev/gfn-client-research/.runtime/verification-b1-qyUq9sDz/proof.md`

Their locations are recorded at `prompt-exports/optimize-nucbox-latency-freezes-runs.md:190-191` and `prompt-exports/optimize-nucbox-latency-freezes-runs.md:203-204`.

**Only planned write:** one sanitized append to the existing ledger; no new retrospective report, source changes, or tests. Suggested entry, **not executed here**:

> **Performance continuation checkpoint — STOP_BLOCKED / CUA_LIBEI_NOT_READY.** B1 incomplete: zero scored windows. H1 established input-backend initialization failure; consent versus wedged EIS unresolved. A1 descriptive evidence retained; A2 INVALID and D1 STAGE_ONLY unchanged. Performance iteration stopped pending separately scoped, supported Driver/EIS repair and demonstrated Cua input readiness. Existing B1/H1 receipts retained. No retry, measurement, instrumentation, deployment, or performance improvement claimed.

End of Oracle group: 2 lanes above.