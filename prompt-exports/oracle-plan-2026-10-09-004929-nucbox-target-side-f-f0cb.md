# Oracle Plan

## Oracle group
- Group ID: `6B92B7C7-9BD4-4185-AF65-99D6EEFB93D6`
- Status: `completed`
- Oracle count: 2

**Reconciling these Oracle lanes**
2 independent answers to the same request follow. Lane order is not a ranking; the first lane supplies the top-level continuation handle, and a successful follow-up through any lane's chat ID re-runs every lane.
- Read every lane through the end-of-group marker (`End of Oracle group: 2 lanes above.`). If the marker or a lane is missing, page the export or try the read-only `oracle_chat_log` with that lane's chat ID. Logs may be scoped or clipped; report any remaining gap. Do not start a follow-up just to retrieve prior text.
- Reconcile by evidence, not lane order, answer length, or model identity: check material single-lane and conflicting claims against the code, and report unresolved disagreements. A failed or partial lane is incomplete evidence.
- Before synthesizing, inventory every material claim from every lane, including single-lane claims. Merge only exact duplicates, retaining all source lanes. Begin your answer with `**Oracle reconciliation**`, state how many lanes completed and name any that did not. For every inventory item, give its source lanes, checked evidence, and exactly one disposition: `accepted`, `rejected`, or `unresolved`. Never silently omit an item.

Lanes (2):
- Oracle — `claude-opus-5-5-xhigh` — completed — chat ID `nucbox-target-side-freez-0C3867`
- Oracle 2 — `gpt-6-astra-pro` — completed — chat ID `nucbox-target-side-freez-66449A`

## Oracle results

### Oracle (Primary)
- Lane index: 0
- Role: `primary`
- Chat ID: `nucbox-target-side-freez-0C3867`
- Provider: _Not specified._
- Model: `custom_provider_claude-opus-5-5-xhigh`
- Status: `completed`
- Execution provider: `Custom`
- Execution model: `claude-opus-5-5-xhigh`

#### Response

# H2: reuse the loop's existing clock reads, measured with a symmetric admission bracket

## 1. Summary

**Decision:** continue with exactly one more hardening candidate, H2. If H2 fails its predeclared gate, stop measurement hardening; the stop reason is in §3.6.

**What H2 changes in the receive loop.** It removes three of the five diagnostic clock reads paid on every VIDEO datagram in the raw Mjolnir loop. It does this by stamping from instants the loop already reads: the loop-top `now` and the post-match `now`. Every datagram still gets all of the following:

- Receive, Preprocess, Process, Forward, Service and Between wall spans;
- the ≥1 ms tail check;
- raw, authenticated and assembled counters;
- per-bin CPU and SO_MEMINFO snapshots.

**What H2 changes in the harness.** The admission harness gets one correction: ordinary and instrumented passes are timed over the same bracket. This is required for two reasons:

- The current harness cannot represent "reuse a read that both builds already perform."
- The current harness charges instrumented passes about two clock reads that no production build executes.

**Gate and verdicts.**

- The threshold stays at ≤5%, from one three-pair Linux run with no retry.
- The p99 condition is operationalized before measurement as a median paired change of ≤ +5%.
- The H0 and H1 rows stay FAIL_COST.
- If H2 passes, the already-planned diagnostic build, identity binding and one 180-s capture follow (§3.7).
- Nothing here measures or claims physical input-to-photon latency or a live gain.

---

## 2. Current-state analysis

### 2.1 Where H1 pays per datagram

| # | H1 diagnostic read | Location | Existing protocol read next to it |
|---|---|---|---|
| 1 | `checkpoint` fast path `origin.elapsed()` | `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:417-435`, called at `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7491-7492` | Loop-top `let now = Instant::now()` at `nvst.rs:7439`. Only the ping-interval comparison sits between them; it sends rarely, then sets `last_ping = now` (`nvst.rs:7440-7489`). |
| 2 | `before_receive` → `stamp()` | `receive_diagnostics.rs:287-293`, called at `nvst.rs:7493-7494` | Same loop-top `now`. |
| 3 | Receive end `d.stamp()` | `nvst.rs:7496-7513` | None. `recv_from` returns at `nvst.rs:7495`. |
| 4 | Process end `d.stamp()` | `nvst.rs:7581-7594` | None. The process start already reuses `received_at` (`nvst.rs:7579-7580`). |
| 5 | Forward end `d.stamp()`, or `nvst.rs:7659` on non-processed iterations | `nvst.rs:7631-7640` | Post-match `let now = Instant::now()` at `nvst.rs:7687`. Only the diagnostic end block `nvst.rs:7657-7686` sits between them. |

- [obs] The read sites and their adjacency are exactly as in the table.
- [inf, from the rows above] Reads 1 and 2 duplicate `nvst.rs:7439`, separated by one comparison. Read 5 duplicates `nvst.rs:7687`, separated only by diagnostic bookkeeping. Reads 3 and 4 have no existing equivalent.
- [obs] The command drain runs before the loop-top `now` (`nvst.rs:7416-7437`). So the loop-top read does not precede any protocol work that Between must cover.

### 2.2 The harness bracket is asymmetric

- [obs] The ordinary service time is `finished - start`. The instrumented service time is `outer_start → outer_start.elapsed()`. See `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs:275`, `:288`, `:314`, `:333-339`.
- [obs] The gate total sums `service_ns` (`receive_replay_tests.rs:223-225`).
- [inf, from `receive_replay_tests.rs:275-339`] The instrumented interval fully contains the harness's own `start` and `finished` reads, which are endpoints in the ordinary interval. It also adds the half-costs of `outer_start` and `elapsed()`. The net effect is about two clock-read-equivalents per packet charged only to the instrumented side, and no production build executes them.
- [obs] The harness re-implements the hook sequence by hand (`receive_replay_tests.rs:276-332`) rather than calling a shared entry point. The admitted code and the deployed code can therefore drift apart.
- [inf] Because the ordinary pass has no analogue of the loop-top or post-match reads, any candidate that reuses those reads would be charged for them anyway. The bracket must be corrected to represent H2 at all.

### 2.3 What the numbers suggest (inference, not attribution)

**Observed inputs:**

- [obs] H1 Linux p50 increments were 0.231, 0.210 and 0.240 µs. Ordinary totals were about 292 ms over 104,390 measured packets, a mean of about 2.80 µs (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:830-838`). So 5% is about 0.14 µs per packet.
- [obs] The same H1 harness measured 1.95% on the Mac, where the Linux checkpoint hook is not compiled (`optimize-nucbox-latency-freezes-runs.md:808`; `receive_replay_tests.rs:276-279`).
- [obs] Linux measured 8.46% with clocksource `tsc` (`optimize-nucbox-latency-freezes-runs.md:819`, `:836`).

**Inference from those observations:**

- On the NucBox, the increment is dominated by vDSO clock reads, not accumulator bookkeeping.
- A rough model is: increment ≈ (reads charged × r) + B, where r is one clock read and B is the bookkeeping.
- Fitting H1 (about 7 charged reads) gives r ≈ 20–28 ns and B ≈ 40–90 ns.

| Case | Charged reads | Estimated increase |
|---|---|---|
| H2, symmetric bracket | 2 | ≈ 3.4–4.6% |
| H2, legacy bracket structure | 4 | ≈ 5.4–6.1% |
| H1, symmetric bracket | 5 | ≈ 6–7%, still FAIL |

The last row is a model estimate only. It is not run and not a re-score; it does show that the bracket correction alone would not admit H1.

The p99 behaviour is not explained by this model. H1 p99 moved +17.7%, −1.0% and +15.2% (`optimize-nucbox-latency-freezes-runs.md:832-834`), so H2 can still fail on p99.

### 2.4 Is the gate mismatched to live packet rates?

The gate is not fundamentally mismatched. It is the right kind of gate, and it is conservative:

- **Right kind.** [obs] Replay arrivals only drive the protocol `now`; service runs back to back (`receive_replay_tests.rs:271-272`). [inf] The fixture therefore measures how much the instrumentation slows the worker's drain rate. That is the quantity that matters when the socket queue is full, which is the +168 VIDEO-drop condition.
- **Conservative.** [obs] The fixture denominator excludes `recv_from`, the source/STUN/feedback work and the post-service housekeeping (`nvst.rs:7495`, `:7537-7578`, `:7687` onward). [inf] The real per-datagram worker time is larger, so the same absolute cost is a smaller live fraction. The target's `recv_from` cost is unmeasured, so this is not used to pass anything.
- **One genuine instrument defect** (§2.2). It is corrected going forward only. The H0 rows (`optimize-nucbox-latency-freezes-runs.md:775-781`) and H1 rows (`:830-836`) remain FAIL_COST.

---

## 3. Design

### 3.1 Choice and rejected alternatives

| Option | Reads removed | Coverage lost | Verdict |
|---|---|---|---|
| **H2: reuse existing clock boundaries + symmetric bracket** | 3 of 5 in production | Only a few-ns boundary shift (§3.5) | **Selected** |
| Sparse Process/Forward split | 1, on unsampled datagrams only | Process vs Forward attribution for unsampled datagrams, which includes most rare ≥1 ms tails | Rejected. The §2.3 model estimates ~7.5% (still FAIL), and it blinds exactly the events the selection rule needs. |
| Sampling all hooks per datagram | Most | The every-datagram Service stall check and counters | Rejected. It violates a fixed requirement. |
| Correct the harness only | 0 | None | Rejected. It would amount to moving the instrument, not reducing cost, and the model says H1 fails anyway. |
| Stop now | — | — | Premature. An untested change with no coverage loss is available. |

### 3.2 Accumulator API — `receive_diagnostics.rs`

The production loop and the harness call the same entry points; only their clock sources differ. Both call sites get identical code generation through `#[inline]`. The signatures below are illustrative.

```rust
#[inline] pub(super) fn receive_start(&mut self, loop_now: Instant, sampled: Option<Stamp>, pinged: bool) -> Stamp;
#[inline] pub(super) fn receive_returned(&mut self, start: Stamp, bytes: Option<usize>, idle: bool) -> Stamp; // 1 clock read
#[inline] pub(super) fn process_returned(&mut self, receive_end: Stamp, received_at: Instant) -> Stamp;      // 1 clock read
#[inline] pub(super) fn complete_iteration(&mut self, at: Instant, receive_end: Stamp, received: bool,
                                           process_end: Option<Stamp>, counts: [u64; 6]);
#[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
#[inline] pub(super) fn checkpoint(owner: &mut Option<Self>, socket: &UdpSocket, loop_now: Instant) -> Option<Stamp>;
#[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
#[cold] #[inline(never)] fn sample_bin(owner: &mut Option<Self>, socket: &UdpSocket, before: u64) -> Option<Stamp>;
```

**`checkpoint`**

- If there is no owner, return `None`.
- Set `before = stamp_at(loop_now).wall`. The existing `complete` and `index` logic now uses this value instead of `origin.elapsed()` (`receive_diagnostics.rs:425-432`).
- If the bin is not due, return `None` with no clock read.
- If due, call `sample_bin`. It holds the existing sampling body (`receive_diagnostics.rs:436-476`) unchanged.
  - It returns `Some(Stamp { wall: after })`, reusing the existing post-`getsockopt` read at `receive_diagnostics.rs:450-455`, when it records a bin.
  - It returns `None` when it exported at completion, because the owner was taken.

**`receive_start`**

- Set `s` to the first of these that applies:
  1. `sampled`, if a sample was taken;
  2. a fresh `stamp()`, if `pinged`;
  3. `stamp_at(loop_now)` otherwise.
- If `last_service.take()` returns a previous end, record a Between span from that end to `s`. This replaces `before_receive` (`receive_diagnostics.rs:287-293`), which is removed.

**`receive_returned`**

- Set `end = stamp()`.
- Record the Receive span from `start` to `end`.
- Call `result(end, …)` as before (`nvst.rs:7498-7511`).

**`process_returned`**

- Set `end = stamp()` and `start = stamp_at(received_at)`.
- Record Preprocess from `receive_end` to `start`, and Process from `start` to `end`.
- This preserves the H1 semantics at `nvst.rs:7582-7593`.

**`complete_iteration`**

- Set `end = stamp_at(at)`.
- If `received`:
  - if `process_end` is present, record Forward from `process_end` to `end`;
  - otherwise record Preprocess from `receive_end` to `end`;
  - in both cases record Service from `receive_end` to `end`.
- Always call `counters(end, counts)` and `service_end(end)`.
- This folds `nvst.rs:7631-7640` and `:7657-7686` into one entry point.

**Other changes in this file**

- Add `#[inline]` to `Stats::add`, `stamp`, `stamp_at`, `bin`, `span`, `result`, `counters` and `service_end`.
- Bump `schema` from 1 to 2 in the start log (`receive_diagnostics.rs:397-404`) and in the export JSON.
- Add an export field `phase_boundaries`. Its text: "Receive start = worker loop-top instant except on ping or sample iterations (fresh stamp). Forward, Service and Between boundaries = post-iteration instant."
- Schema 2 ties any live capture to the H2 boundary semantics. No schema-1 captures exist [obs: `optimize-nucbox-latency-freezes-runs.md:844`].

**Threading:** unchanged. All calls happen on the single owning worker thread, and the exporter is unchanged.

### 3.3 Production loop rewiring — `nvst.rs`

The ordinary build compiles identically, because only `#[cfg]` diagnostic statements move. Illustrative order:

```rust
let now = Instant::now();                       // existing, 7439
/* ping block unchanged; sets last_ping = now when it runs */
#[cfg(diag)] let receive_start = {
    let sampled = receive_diagnostics::Capture::checkpoint(&mut diagnostic, &socket, now);
    let pinged = last_ping == now;              // true iff the ping branch ran this iteration
    diagnostic.as_mut().map(|d| d.receive_start(now, sampled, pinged))
};
let receive_result = socket.recv_from(&mut datagram);   // unchanged
#[cfg(diag)] let receive_end = diagnostic.as_mut().map(|d| d.receive_returned(receive_start.unwrap(), len, idle));
#[cfg(diag)] let mut process_end = None;
#[cfg(diag)] let received_datagram = receive_result.is_ok();
match receive_result {
    Ok(..) => 'datagram: {
        /* unchanged through process_datagram(…, received_at) */
        #[cfg(diag)] { process_end = diagnostic.as_mut().map(|d| d.process_returned(receive_end.unwrap(), received_at)); }
        /* forward loop; on failure: */
        //   #[cfg(diag)] d.complete_iteration(Instant::now(), receive_end.unwrap(), true, process_end, counts);
        //   stop; return;
    }
    /* idle / error arms unchanged */
}
let now = Instant::now();                       // existing, 7687: the diag block now goes AFTER this line
#[cfg(diag)] if let Some(d) = diagnostic.as_mut() {
    d.complete_iteration(now, receive_end.unwrap(), received_datagram, process_end, counts);
}
/* housekeeping unchanged */
```

**Removals:** `did_process`, `forward_finished` and the post-forward stamp (`nvst.rs:7514-7517`, `:7631-7640`).

**Counts:** the `counts` array is the existing six-element array at `nvst.rs:7676-7683`.

**Protocol timestamps:**

- `received_at` and the loop-top `now` are untouched.
- The post-match `now` is now read before the diagnostic bookkeeping instead of after it. This puts it at the same position as in the ordinary build, which is closer to ordinary behaviour than H1.
- `recv_from`, queues, recovery and stop paths are unchanged.

**Unwrap safety:**

- [inf, from `nvst.rs:7491-7519` and `receive_diagnostics.rs:466-471`] `diagnostic` can only become `None` inside `checkpoint`. That happens before `receive_start` is computed, so `receive_start` and `receive_end` are `Some` whenever `diagnostic` is `Some`.
- The `last_ping == now` check is exact. `last_ping` starts in the past (`nvst.rs:7401`) and equals `now` only after `nvst.rs:7488` runs.

### 3.4 Admission harness — `receive_replay_tests.rs`

**`PacketCost` gains a field** `bracketed_ns: u64`. `traversal` sets it equal to its `service_ns`; it is unused there.

**`replay` timing, both modes.** Illustrative:

```rust
let loop_top = Instant::now();           // models nvst.rs:7439 (exists in both builds)
// Linux+feature: sampled = checkpoint(&mut diagnostic, socket, loop_top), else None
let receive_end = diagnostic.as_mut().map(|d| {
    let s = d.receive_start(loop_top, sampled, false);
    d.receive_returned(s, Some(len), false)
});
let start = Instant::now();              // models received_at, nvst.rs:7579
/* process_datagram */
let processed = Instant::now();          // harness-only, both modes (unchanged)
let process_end = diagnostic.as_mut().map(|d| d.process_returned(receive_end.unwrap(), start));
/* forward loop unchanged */
let finished = Instant::now();           // models post-match now, nvst.rs:7687
if let Some(d) = diagnostic.as_mut() {
    d.complete_iteration(finished, receive_end.unwrap(), true, process_end, counts);
}
let bracket_end = Instant::now();        // harness-only, both modes
// service_ns   = finished - start      (R1 definition, both modes)
// bracketed_ns = bracket_end - loop_top (admission metric, both modes)
```

- `outer_start` and `outer_elapsed` are removed.
- The replay semantics and asserts are unchanged, as is the post-loop `assert_fixture` (`receive_replay_tests.rs:381-388`).
- [inf] The bracketed difference is now exactly H2's production delta:
  - two clock reads;
  - accumulator bookkeeping;
  - the checkpoint fast path;
  - about four sampled bins per pass on Linux.

**`receive_diagnostic_hook_cost_report`.** Keep the fixture, the warmup, the `[false, true, false]` order and the release assert (`receive_replay_tests.rs:207-240`). Each pair row prints:

- `symmetric_increase_percent`: 100 × (Σ instrumented `bracketed_ns` ÷ Σ ordinary `bracketed_ns` − 1). **Gate.**
- `symmetric_p99_change_percent`: computed from the `distribution` p99 of `bracketed_ns` (`receive_replay_tests.rs:573-584`). **Gate.**
- `asymmetric_legacy_increase_percent`: instrumented `bracketed_ns` versus ordinary `service_ns`. This has the same structure as the H0 and H1 comparison. It is **informational only** and is the number directly comparable to the existing FAIL rows.
- The bracketed distributions for both modes, plus updated `bracket` and `limits` text.

After the three pairs it prints one `RECEIVE_DIAGNOSTIC_ADMISSION` line containing:

- the two gate medians;
- the median of the legacy number;
- a computed verdict string.

None of these values are asserted on. Validity failures stay test failures; performance verdicts do not.

**Default-suite coverage.** In `authenticated_frames_survive_rollover_reorder_and_real_fec_repair` (`receive_replay_tests.rs:586-596`), also run `replay(&repair, 0, true)` with the same three assertions. This puts the instrumented path, the new methods and `assert_fixture` into every `--lib` run:

- portable on the Mac;
- with real socket and CPU samples under Linux plus the feature.

### 3.5 Exact coverage: retained, shifted, lost

**Retained for every datagram:**

- all six phase aggregates, including count, wall and max;
- the ≥1 ms non-Receive tail pool, with its omission and cutoff counts;
- raw, idle and error counts and bytes;
- the six cumulative counters;
- media-coverage endpoints;
- per-bin CPU and SO_MEMINFO samples, and final samples;
- partial export on worker exit.

**Shifted, disclosed under schema 2:**

1. On iterations with no ping sent and no bin sampled, Receive starts at the loop-top `now`, before the ping comparison and the checkpoint fast-path check, rather than after them. Those few nanoseconds move from Between to Receive. Receive spans are aggregate-only and never become tails (`receive_diagnostics.rs:273-279`), so no tail is lost.
2. Forward, Service and Between boundaries move from H1's in-arm stamp to the post-match `now`. Nothing executes between the two positions. Between still includes the diagnostic end-block bookkeeping, as it did in H1.

**Gained:** the forward-failure terminal path now also records final counters through `complete_iteration`.

**Lost:** nothing beyond item 1 above.

### 3.6 Predeclared H2 admission gate and stop rule

Append this text to the ledger and the H2 source receipt **before** any Linux measurement.

**Validity**

- The Linux default and feature `--lib` suites pass.
- The admission run emits three pair rows.
- `assert_fixture` passes for all three instrumented passes: samples > 0, and zero sampling errors, omissions and cutoff.

**Cost:** the median of the three `symmetric_increase_percent` values is ≤ 5.000%.

**p99:** the median of the three `symmetric_p99_change_percent` values is ≤ +5.000%. This is the operational meaning of "no unexplained p99 regression."

**Run discipline**

- Run once.
- A run that aborts before emitting rows (compile error or assertion) is a defect. Fix it and rerun once, with disclosure.
- Once rows are emitted, the verdict is final. No reruns and no favourable pair selection.

**Reporting:** always record the informational legacy median next to the gate medians.

**Stop rule if H2 fails either gate.** Stop measurement hardening and record this reason:

> Two wall reads per datagram is the minimum for full per-datagram phase localization in this loop. Any further reduction trades required coverage — sparse Process/Forward attribution, or merging Receive with inline work — so it is a user decision, not a measurement correction.

No live capture is earned in that case.

### 3.7 After PASS: build, deploy and one capture

All prior gates are unchanged.

1. **Fresh private preflight.** Confirm the exact source HEAD and that the tree is clean. Confirm that the CUA operator owning stdio 91787 has freshly confirmed ownership.
2. **Build.** Run `tools/build-native.sh` with `OPENNOW_DEVELOPER_RECEIVE_DIAGNOSTICS=ON`, using the exact invocation from the delegated preflight receipt. Add no other flags.
3. **Bind identity.**
   - Record `streamer-ffi-features.txt`.
   - Confirm `grep -a -q -F OPENNOW_NVST_RECEIVE_DIAGNOSTICS_FILE` succeeds on the deployed FFI library.
   - After launch, confirm that `/proc/<pid>/maps` shows that same library path, and record its `sha256sum`.
4. **Launch.**
   - Use a fresh Qt process.
   - Set `OPENNOW_NVST_RECEIVE_DIAGNOSTICS_FILE` to a new absolute path inside a 0700 directory.
   - Require `schema=2 status=started role=opennow-nvst-mjolnir` in the native log.
5. **Capture.**
   - The operator keeps an authorized VIDEO stream running across the 180-s window.
   - Bins before `first_assembled_ns` plus settling are labelled as startup.
   - The capture is valid only if both `status=exported` is logged and the JSON parses.
   - A window with zero drop delta is *inconclusive*, not negative.
6. **Store.** Raw data stays private. The ledger gets only sanitized aggregates, hashes and paths.
7. **Controls.** Any later ordinary or control build exports `OPENNOW_DEVELOPER_RECEIVE_DIAGNOSTICS=OFF` explicitly, and the marker grep must fail on it.

**Do not:** install tracing tools, or change rmem, network, profile or display settings.

### 3.8 Post-capture selection rule (applied once)

**Definition.** A *drop interval* runs between two consecutive bin socket samples where SO_MEMINFO `drops` (index 8) increased.

**Select exactly one optimization** only if all three conditions hold:

- **(a) Repetition.** At least two drop intervals each overlap at least one retained non-Receive tail.
- **(b) Dominance.** A single phase (Preprocess, Process, Forward or Between) owns more than 50% of the tail wall time across those intervals.
- **(c) On-CPU consistency.** For each of those intervals, the worker CPU delta between its bracketing samples is at least that phase's tail wall time. This is consistent with on-CPU work, though it does not prove it.

When selected, the candidate is confined to that phase's code. Any gain claim requires a matched control and candidate within the same cloud session.

**Otherwise, select no optimization** and report the exact missing observable:

| Outcome | Missing observable to report |
|---|---|
| Drops with only Receive-dominated wall | Kernel arrival/softirq timing and worker runqueue wait during the interval, e.g. external `/proc/<pid>/task/<tid>/schedstat` sampling |
| Condition (c) fails | Blocking vs preemption attribution |
| No drops in the window | The incident was not captured |

**Terminology.** Keep these measures separate:

- VIDEO socket drops;
- component wall durations;
- presentation intervals;
- terminal stall.

None of them is physical input-to-photon latency.

---

## 4. File-by-file impact

All paths are relative to the repository root. Rows 1–3 must land in one commit, because the API changes atomically. Commit exactly these three files, matching the H1 allowlist practice (`optimize-nucbox-latency-freezes-runs.md:812-817`).

| File | Changes | Depends on |
|---|---|---|
| `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs` | **Added:** `receive_start`, `receive_returned`, `process_returned`, `complete_iteration`; cold `sample_bin`. **Removed:** `before_receive`. **Changed:** `checkpoint(owner, socket, loop_now) -> Option<Stamp>`; `#[inline]` on the hot helpers; schema 2 plus `phase_boundaries`. **Tests:** see the two bullets below. | — |
| `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs` | Hook sites rewired per §3.3. The diagnostic end block moves below the post-match `now`. `did_process` and `forward_finished` removed. The failure path uses `complete_iteration`. No non-`cfg` line changes. | Row 1 |
| `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs` | `bracketed_ns`; `loop_top` and `bracket_end` reads in both modes; the shared entry points; the report's gate, informational fields and admission line; the instrumented replay in the default semantic test; `traversal` field fill. | Row 1 |
| `prompt-exports/optimize-nucbox-latency-freezes-runs.md` | Append-only:<br>1. H2 design and gate, before measurement.<br>2. H2 rows, verdict and legacy comparison.<br>3. Capture aggregates, if earned.<br>Not committed. | Before step 6 (§6) |

**Tests in `receive_diagnostics.rs`:**

- **New:** `existing_clock_boundaries_are_reused_without_changing_phase_accounting`. It covers:
  - `receive_start` returns exactly `stamp_at(loop_now)` with no sample and no ping;
  - it returns the sample stamp when one is given;
  - it returns a fresh stamp ≥ `loop_now` when `pinged` is set;
  - Between equals the previous `at` through the next start;
  - the Forward and Service walls end exactly at `stamp_at(at)`;
  - a datagram with no `process_end` gets Preprocess only, with no Forward;
  - an idle iteration gets no Service, but counters and `service_end` are set.
- **Updated:** `linux_socket_cpu_and_partial_export_smoke`.
  - The first `checkpoint` returns `Some`.
  - An immediate second call returns `None` and takes no sample.
  - After the 100-ms sleep it returns `Some`.
  - The existing serialization asserts are kept, with `schema == 2`.

**Validation before editing:** grep for `before_receive(` and `checkpoint(` to confirm there are no callers beyond the three listed files. The module is private to `nvst` [obs: `nvst.rs:7366-7368`].

---

## 5. Risks

- **The bracket change could be read as moving the goalposts.** Mitigations:
  - the threshold is unchanged and predeclared;
  - the legacy-structure number is reported from the same run;
  - the H0 and H1 rows are untouched;
  - the §2.3 estimate says H1 fails under the corrected bracket too.
- **p99 noise.** The ordinary p99 alone varied by 0.26–0.29 µs between passes [obs: `optimize-nucbox-latency-freezes-runs.md:777-779`, `:832-834`]. H2 can fail on p99 alone. If so, the stop rule applies, with no rerun.
- **Estimate uncertainty.** The read-versus-bookkeeping split is a one-point model fit, not a measurement. That uncertainty is why the gate exists.
- **First Linux compile happens on the target.** The Linux-only code is compiled first on the target, as with H1 (`optimize-nucbox-latency-freezes-runs.md:826-827`). A compile failure is a fix-forward commit and does not count as an admission run.

---

## 6. Implementation order

1. **Predeclare.** Append the §3.6 gate, the H2 composition and the stop rule to the ledger. Write the source receipt.
2. **Implement (atomic).** Make the `receive_diagnostics.rs` API and test changes, then the `nvst.rs` rewiring, then the `receive_replay_tests.rs` changes.
3. **Run local checks** on the Mac, using the H1 command forms (`optimize-nucbox-latency-freezes-runs.md:806-810`):
   - `cargo test --locked --release --manifest-path native/opennow-streamer/Cargo.toml -p opennow-streamer-transport --lib`. This includes the instrumented default replay.
   - The same command with `--features receive-diagnostics --lib receive_diagnostics -- --nocapture`, run under `env -u OPENNOW_NVST_RECEIVE_DIAGNOSTICS_FILE`.
   - The ordinary `cargo build --locked --release … -p opennow-streamer-transport`.
   - `rustfmt --edition 2024 --check` on the three files, then `git diff --check`.
4. **Publish.** Commit exactly the three files and verify the staged set against the allowlist. Push to `nucbox-wayland-hdr`. Then run `cd ~/dev/gfn-client-research/vendor/OpenNOW && git pull --ff-only`, and verify the exact HEAD and a clean tree (`optimize-nucbox-latency-freezes-runs.md:819`).
5. **Run the Linux suites.** Use the same `RUSTUP_HOME`, `CARGO_HOME`, `CARGO_BUILD_JOBS=2` and `PATH` (`optimize-nucbox-latency-freezes-runs.md:825`). Run the default and `--features receive-diagnostics` `--lib` suites with the runtime file unset (`:826-827`).
6. **Run the Linux admission once.** Use the exact ignored test command (`optimize-nucbox-latency-freezes-runs.md:828`): `--exact --ignored --nocapture --test-threads=1`. Record the clocksource.
7. **Record the outcome.** Append the three rows, both gate medians, the legacy median and the verdict to the ledger. Raw logs stay private.
8. **Branch on the verdict.**
   - **PASS:** run the §3.7 build, identity and capture sequence, then apply §3.8 once.
   - **FAIL:** stop at the §3.6 reason and release the operator. No build, deploy or live run.

### Oracle 2
- Lane index: 1
- Role: `additional`
- Chat ID: `nucbox-target-side-freez-66449A`
- Provider: _Not specified._
- Model: `custom_provider_gpt-6-astra-pro`
- Status: `completed`
- Execution provider: `Custom`
- Execution model: `gpt-6-astra-pro`

#### Response



## 1. Summary

**Continue with one H2 measurement candidate: retain exhaustive outer Receive/Service observation, sample the inner Preprocess/Process/Forward split on one in sixteen successful receives, and share the receive-entry timestamp with the checkpoint due check.** This removes one redundant wall-clock read on ordinary iterations and most inner-phase bookkeeping while preserving the observations needed to detect a receive-owner service stall. Keep the existing Linux admission gate, run its single three-pair report once, and proceed to the optimized Qt diagnostic build and one worker-timed 180-second capture only if admission passes. H0 and H1 remain failed admissions; H2 is a bounded reduction hypothesis, not a demonstrated improvement. **Direct observation:** H1 remains unpublished as an application artifact because its Linux admission failed, and the ledger identifies sparse phase splitting as the next unimplemented measurement candidate. `prompt-exports/optimize-nucbox-latency-freezes-runs.md:830-848`

All references below are relative to `/Users/danielsivan/dev/OpenNOW`. **Direct observation** identifies supplied source or ledger evidence, not a new execution. **Inference** identifies a conclusion from that evidence. **Decision** specifies the implementation contract.

## 2. Current-state analysis

### The current checkpoint is source `2ed0fb1d`, not the earlier preparation state

**Direct observation:** H1 was committed as `2ed0fb1d5c8e5710190d9f7da0a93cf4eb37705f`, pushed normally, and pulled cleanly on the target. Actual Linux release tests passed: 309 default-feature tests and 310 enabled-feature tests, including the private serialized CPU/socket smoke. Its three overhead pairs nevertheless failed admission: median added service was **8.464895%**, with median paired p99 **+15.226040%**. Qt was not built or deployed; the client remained closed and disk artifacts retained their prior c10 attribution. `prompt-exports/optimize-nucbox-latency-freezes-runs.md:817-844`

**Decision:** build H2 on that reconciled source lineage. Preserve the completed b3 review corrections, pending `AGENTS.md` work, ledger history, and private evidence. There is no second review round and no return to the earlier `6472cae7` deployment-branch proposal.

### The remaining instrumentation cost has two visible reduction points

**Direct observation:** `run_nvst_udp_receiver` selects the raw VIDEO path after the RTC delegation. On each active diagnostic iteration, it calls `Capture::checkpoint`, then `before_receive`, then the unchanged `recv_from`. Successful datagrams subsequently record Preprocess, Process, Forward, Service, receiver-counter deltas, and the next Between interval. `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7366-7745`

**Direct observation:** `checkpoint` reads elapsed wall time even when that bin already has a socket sample. `before_receive` immediately obtains another wall timestamp. H1 also retains all three inner phase updates on every processed datagram. The diagnostic owns bounded observation state; it does not own protocol or recovery state. `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:100-176` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:245-308` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:418-477`

**Inference:** sharing the first boundary and sparsifying only the inner split is the strongest remaining small candidate visible in this source. It removes specific work without reducing Service-stall detection. The evidence does **not** establish how much of H1’s overhead those operations cause.

For scale, H1’s median-cost pair would need approximately **97 ns less timed work per measured packet** to reach 5%. That follows from its ordinary/instrumented totals and 104,390 measured packets; it is a break-even calculation, not a predicted H2 saving. `prompt-exports/optimize-nucbox-latency-freezes-runs.md:830-838`

### Reuse the existing implementation and validation

**Direct observation:** the existing accumulator already separates raw receipts, accepted packets, assembly, repairs, and counter discontinuities. It preserves non-Receive tails, explicitly excludes ordinary receive waits from that tail pool, and retains metadata-only bins during export. The replay now observes and validates the capture outside packet timing. `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:178-244` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:268-342` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:466-505` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs:260-415`

**Decision:** make a targeted change in the same three Rust files. Keep the existing feature, runtime opt-in, exporter, build wiring, fixtures, and operator workflow. No additional observer framework, tracing dependency, collector program, or playback optimization is needed.

## 3. Design

### A. H2 observation contract

**Decision:** fix the sampling stride at **16** for this candidate. It is a compile-time diagnostic policy, with no runtime tuning option.

Use the existing worker `inbound_datagrams` ordinal after its successful-receive increment. Select ordinals **1, 17, 33, …**. The replay uses the same ordinal over its complete packet sequence, including warmup; it must not restart sampling at the measured-frame boundary.

| Observation | H2 coverage |
|---|---|
| Receive count, wall sum, maximum, raw bytes, idle/error outcomes | Retained on every currently observed receive iteration |
| Service count, wall sum, maximum, and ≥1 ms tail detection | Retained for every currently observed completed datagram service |
| Between count, wall sum, maximum, and tail detection | Retained |
| Raw/authenticated/assembled/repaired/STUN/source counters | Retained |
| Bracketed thread-CPU and `SO_MEMINFO` samples | Same first-worker-opportunity policy per 100 ms bin, plus existing final sampling |
| Preprocess/Process/Forward timing | Recorded only for selected successful datagrams |
| Individual Receive tail records | Still excluded, as in H1 |
| Existing early-return, cutoff, delayed-export, and missing-sample limitations | Preserved and disclosed |

“Retained” preserves H1’s coverage; it does not turn its documented partial-exit and logical-cutoff gaps into complete coverage. **Direct observation:** those limitations are explicit in the current module and exported record. `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:1-18` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:479-505`

For selected datagrams, preserve existing phase boundaries. For unselected datagrams:

- Keep the actual receive-return stamp.
- Keep the actual Service-end stamp.
- Skip the Process-end diagnostic clock read.
- Skip Preprocess, Process, and Forward accumulator updates.
- Still retain a long Service span whenever it meets the existing threshold.

Do not force extra inner-phase sampling after observing a stall. The Process/Forward split cannot be recovered retrospectively, and stall-triggered sampling would introduce another policy.

**Exact lost coverage:** an unsampled Service stall remains visible with its full outer endpoints, but its duration cannot be assigned to Preprocess, Process, or Forward. Sampled phase totals and distributions describe the selected cohort only. Never multiply them by sixteen or present their maxima or percentiles as full-stream phase statistics. Fixed-stride selection is deterministic and may align with packet/frame structure; it is not an unbiased statistical sample.

### B. Consolidate the receive-entry and checkpoint boundary

**Decision:** replace the production `checkpoint` → `before_receive` pair with one Linux helper. Keep it inside `Capture`; no new owner, thread, or synchronization primitive is introduced.

Partial interface shapes:

```text
prepare_receive(owner, socket) -> Option<Stamp>
before_receive_at(stamp) -> Stamp
split_selected(successful_receive_ordinal) -> bool
record_service(start, end, process_called, split_selected)
```

`prepare_receive` is synchronous and executes on the VIDEO owner:

1. Return `None` immediately when the capture is inactive.
2. Read one wall stamp.
3. Use it to evaluate the existing logical cutoff and bin-sample due condition.
4. If the bin already has its sample, reuse that stamp as the receive-entry boundary.
5. If CPU/socket sampling is due, perform the existing bracketed samples, then obtain the receive-entry stamp **after** that work.
6. Record Between using the resulting entry boundary and return it.
7. If cutoff finalizes and removes the capture, return `None`; callers must not retain a stale stamp.

This saves the redundant elapsed-time read on iterations that do not sample a bin. On sampling iterations, refreshing the entry stamp keeps CPU/socket-query time in Between instead of newly charging it to Receive.

**Direct observation supporting this change:** the existing checkpoint already samples before `before_receive`; its fast path returns after a wall-time read when the current bin is populated. `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:287-308` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:418-477`

Keep all production `Instant::now()` calls used by protocol processing, feedback, pings, reports, and timeout logic in their existing production positions. Continue reusing `received_at` for a selected datagram’s Process start. A previous-loop or process-start timestamp must not substitute for receive return.

### C. Integrate sampling without changing transport behavior

**Decision:** update only the diagnostic branches in `run_nvst_udp_receiver`.

- Choose the split after incrementing the existing successful-receive ordinal.
- Set `did_process` independently of whether the datagram was selected. Currently it is assigned inside the Process-timing hook; leaving it there would misclassify unselected processed datagrams.
- For selected processed datagrams, retain Preprocess, Process, and Forward spans.
- For selected STUN-only paths, retain the existing Preprocess-only interpretation.
- For unselected paths, record the outer Service span and counters without manufacturing inner spans.
- On the existing `forward_receive_event == false` path, retain Service timing; record Forward only when selected.
- Keep the existing handling of other partial exits. Do not add recovery, retries, queue operations, or new event delivery.

`record_service` should update the existing Service statistics and the sampling-coverage counters through the same bin selection, rather than adding another clock read or independent bin lookup.

**Direct observation:** the source already separates receive success, STUN handling, processing, forwarding, and Service finalization, including the forwarding-closure path. These are the existing seams to modify. `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7480-7745`

### D. Make sparse coverage unambiguous in the private output

**Decision:** use **diagnostic schema 2**. Changing phase-count meaning while continuing to emit an indistinguishable schema-1 record would make old assumptions unsafe.

Retain existing field names and phase order. Add:

| Location | Field | Definition |
|---|---|---|
| Top level | `phase_sampling` | Policy object declaring stride 16, ordinal selection rule, and sampled phases |
| Each bin | `split_sampled_services` | Selected datagrams whose Service span completed in this bin |
| Each bin | `processed_services` | Datagrams that called `process_datagram` and whose Service span completed in this bin |
| Long-span record | `inner_split_selected` | Boolean for Service records; `null` for other phase records |

Use the existing Service count as the completed-service denominator. The sampled processed-service numerator is the existing Forward count: Forward and Service share their end boundary on those paths.

This gives two exact coverage ratios:

- Selected completed services / all completed services.
- Selected processed services / all completed processed services.

Do not use a bin’s raw-receive count as the denominator for its completed services: receive and completion may occupy different bins. Similarly, inner phase records remain assigned to their own existing completion boundaries.

For a retained Service tail, `inner_split_selected=false` explicitly means **no inner localization was collected**. A `true` value identifies selection; completeness still depends on cutoff and partial-exit status.

Keep `cpu_missing` tied to the number of **recorded phase spans** without per-span CPU. It must not become a substitute counter for unsampled phases.

**Direct observation:** existing bins are completion bins, phase CPU fields intentionally express missing per-span CPU, and the current schema exports only count/sum/maximum plus bounded tails. `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:43-98` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:479-505`

The new counters remain worker-owned scalars in the preallocated bins. Keep 1,800 bins and 1,024 tail slots. Preserve `create_new`, mode `0600`, private output, one-shot ownership, and export after collection. Record the actual new allocation size through `preallocated_bytes`.

### E. Verification and the unchanged admission decision

**Decision:** update the existing assertions and add only the coverage regression required by H2.

1. **Exhaustive envelope and counters.**  
   Receive and Service counts still match all fixture packets; raw/authenticated/frame/repair totals remain unchanged.

2. **Exact selected coverage.**  
   Preprocess/Process/Forward counts match the deterministic selected ordinal set. For the retained 109,609-packet complete fixture, that is **6,851 selected datagrams**, including warmup. Validate measured-period coverage separately from complete-fixture coverage.

3. **Unsampled stall retention.**  
   Add a focused accumulator case where an unselected datagram has a ≥1 ms Service span. Assert that the Service statistics and tail survive, its selection flag is false, and no Process/Forward spans are fabricated.

4. **Checkpoint boundary behavior.**  
   Exercise both the already-sampled and sample-due branches. Verify that a metadata sample remains bracketed before the returned receive-entry boundary and that metadata-only bins still survive export.

5. **Existing validity checks remain mandatory.**  
   Keep bounded-tail, cutoff, counter-discontinuity, private serialization, CPU/socket error, and semantic assertions. An instrumented replay must still own a capture when final validation runs; disappearance must fail validation.

6. **Keep the capture observable outside timing.**  
   Retain `black_box` and the completed-capture assertions. Production and replay must use the same sampling rule and receive-preparation helper.

**Direct observation:** the current fixture assertions, Linux smoke, and retained-capture validation provide these extension points. `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:178-244` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:550-650` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs:207-415`

Run the existing overhead report once on actual Linux:

- Same 100-Mbps Repair fixture.
- Same 30 warmup and 600 measured frames.
- Same three pair orders.
- Same timing boundaries and denominator.
- No separate Mac timing campaign.
- No 18-case matrix.
- No favorable-pair selection or automatic retry.

The report’s process exit is a validity result, not admission. Keep **median paired added service ≤5%** and **no unexplained p99 regression**. This fixture does not itself explain scheduling-related p99 regressions; a remaining positive median paired p99 change therefore remains a stop for this attempt, not something to clear because it is small in absolute units.

**Direct observation:** these are the retained admission criteria, and H1’s successful test exit did not satisfy them. `prompt-exports/optimize-nucbox-latency-freezes-runs.md:823-838`

### F. Why the fixture gate stays useful, with limits

**Inference:** the unpaced replay is a conservative hook-admission test, not a model of target CPU utilization at a measured live packet rate. Its nominal 100-Mbps arrival schedule is supplied to protocol processing while the host loop runs without network pacing; its timing excludes the receive syscall and exporter. A short run also exercises fewer 100 ms sampling opportunities than a real 180-second stream. `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs:60-205` `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs:207-415` `prompt-exports/optimize-nucbox-latency-freezes-runs.md:838-840`

That limits the gate’s interpretation but does not invalidate it or retroactively pass H0/H1. **Decision:** passing H2 earns the bounded target capture only. If H2 fails, stop this attempt; do not dilute the denominator with receive waiting or silently replace the gate with a host-wide CPU percentage.

## 4. File-by-file impact

| File | Changes | Dependencies |
|---|---|---|
| `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs` | Add fixed split policy, shared receive-preparation boundary, Service coverage accounting, schema-2 metadata and tail selection field. Update fixture assertions and Linux serialization smoke; add the unsampled-Service-tail regression. | Defines the interfaces used by both production and replay. |
| `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs` | Replace checkpoint/before-receive pairing, select inner splits using `inbound_datagrams`, move `did_process` assignment outside sampled timing, preserve exhaustive Service/counters and existing failure behavior. | Must land with the diagnostic API changes. |
| `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs` | Use the same helper and selection rule; validate actual selected coverage and retained capture. Preserve fixture, pair ordering, timing scope, semantic checks, and report entry point. | Must land with the diagnostic API and coverage changes. |
| `prompt-exports/optimize-nucbox-latency-freezes-runs.md` | Append H2’s predeclared contract, source receipt, validation/admission rows, and eventual capture or stop disposition. Preserve the previous prefix and private hashes. | Evidence only; excluded from the source commit. |

The three Rust files are one atomic source change. Cargo manifests, CMake, core/FFI runtime, privacy repairs, baseline tooling, and operator scripts need no modification for H2.

## 5. Risks and migration

Schema 2 applies only to new private diagnostic output. Preserve historical schema-1 files unchanged. Analysis of H2 must explicitly recognize the sampling metadata; historical phase counts must not be reinterpreted as sampled counts.

The material risks are insufficient cost reduction, sparse inner-phase coverage, and delayed worker-owned observations. No application settings, protocol ABI, media queue, or persisted application data changes. Ordinary builds must continue to compile the diagnostic hooks out.

## 6. Implementation order

1. **Freeze H2 and obtain a fresh read-only preflight.**  
   Record the source parent, branch, expected local pending work, and clean target state. Resolve the current supported `tools/build-native.sh` recipe by inspecting the actual script and retained preflight receipt. Record working directory, exact arguments, feature environment, outputs, runtime-copy destination, and restoration path.

   The script body is not included in this snapshot, so this plan cannot honestly supply its positional arguments. The implementer must extract and record them before publication; unsupported flags are not an acceptable fallback.

2. **Implement the three-file H2 change atomically.**  
   Use an isolated worktree based on the verified `2ed0` lineage. Preserve unrelated pending work. Keep the existing feature and runtime activation boundaries.

3. **Run focused local correctness and ordinary release compilation.**  
   Run the transport release suite, enabled-feature correctness tests, ordinary optimized transport build, formatting checks, and `git diff --check`. Do not rerun unrelated core/control suites whose source is unchanged, and do not run another Mac overhead campaign.

4. **Publish only the reviewed source scope.**  
   Inspect the full diff and staged file list against the three-file allowlist. Commit and normally push the authorized source branch. Record the full H2 OID and file hashes. Existing authorization covers this action.

5. **Pull and validate on the closed target.**  
   Use the freshly verified target checkout, historically `~/dev/gfn-client-research/vendor/OpenNOW`, and `git pull --ff-only`. Verify exact H2 OID and cleanliness. Use the already established isolated toolchain; keep `OPENNOW_NVST_RECEIVE_DIAGNOSTICS_FILE` unset throughout tests. Confirm no Qt/core/NVST worker is running.

   Run the default and enabled Linux release transport suites and an ordinary release transport build. Then run exactly:

   ```sh
   env -u OPENNOW_NVST_RECEIVE_DIAGNOSTICS_FILE \
     cargo test --locked --release \
     --manifest-path native/opennow-streamer/Cargo.toml \
     -p opennow-streamer-transport \
     --features receive-diagnostics --lib \
     nvst::tests::receive_replay_tests::receive_diagnostic_hook_cost_report \
     -- --exact --ignored --nocapture --test-threads=1
   ```

   The target toolchain and equivalent command are recorded in the supplied receipt. `prompt-exports/optimize-nucbox-latency-freezes-runs.md:825-828`

6. **Apply admission before any Qt diagnostic build.**  
   Preserve all three pairs and validity summaries. If semantics, sampling, cost, or p99 fails, append the failure and stop with the client closed and prior app artifacts intact. Do not claim an irreducible instrumentation floor: the result establishes only that this candidate remains inadmissible.

7. **After admission, build and bind the actual artifact.**  
   Have the same operator freshly confirm ownership; the historical controller is not currently owned. Archive existing c10-attributed artifacts, then invoke the inspected normal wrapper with `OPENNOW_DEVELOPER_RECEIVE_DIAGNOSTICS=ON`.

   Require a receipt proving:

   - H2 source OID and clean build inputs.
   - Optimized profile and enabled receive-diagnostics feature.
   - Correct feature-stamp transition and actual rebuild.
   - Hash equality between produced FFI and its deployed runtime copy.
   - Qt executable and mapped FFI identity after launch.

   If source, feature, copied output, or mapped runtime cannot be bound, stop before streaming. **Direct observation:** these application-build and mapped-runtime gates remain unearned in the latest checkpoint. `prompt-exports/optimize-nucbox-latency-freezes-runs.md:842-844`

8. **Capture once, starting observers before the stream.**  
   Launch through the established operator with an absolute, nonexistent `OPENNOW_NVST_RECEIVE_DIAGNOSTICS_FILE` inside a new private directory. Start the existing bounded observers while Qt is idle, then start the authorized stream.

   This capture has **no arm stage**: its 180-second timer starts at worker setup. Record startup, first authenticated/assembled evidence, scene-ready time, and all operator input intervals. Do not describe the result as 180 seconds of stable scene coverage. Verify the actual dedicated VIDEO worker, PID/TID, socket mapping, and overlap with authenticated media.

   Once the scene is established, keep input and settings unchanged. No baseline collector, builds, tests, hashing activity, or new tracing tools run during the incident window.

9. **Finalize at the fixed checkpoint and classify the evidence.**  
   At timer completion, require a valid schema-2 export. If the worker exits or export does not complete within the predeclared five-second collection grace, retain partial/missing evidence and end the attempt through the existing operator procedure. Do not extend or restart the capture.

   Analyze only joins supported by the records:

   - Positive VIDEO drop deltas between valid `SO_MEMINFO` snapshots.
   - Full Service/Between tails overlapping those brackets.
   - Sampled inner splits where present.
   - Worker-wide CPU deltas over their actual bracketed intervals.
   - Counter and authenticated-media coverage.

   Counter decreases, unavailable socket samples, clock uncertainty, tail omissions, and cutoff gaps remain explicit. Worker wall-minus-CPU is non-CPU elapsed time; it does not identify blocking, preemption, or run-queue delay. The live accumulator does not provide full duration percentiles.

10. **Restore an ordinary deployment and append the outcome.**  
    Close the exact client instance. Unset the runtime file opt-in, use the supported wrapper with the developer diagnostic gate explicitly `OFF`, and verify rebuild/runtime-copy behavior and ordinary artifact identity. Leave the client closed. If ordinary restoration fails, restore the archived c10 bytes and retain their original attribution.

    Append sanitized aggregates, source/artifact receipts, private evidence paths/hashes, actual media overlap, and the disposition. A complete no-drop window remains a valid diagnostic attempt without incident reproduction; it does not authorize another window.

### Subsequent optimization-selection rule

**Select at most one playback optimization only if at least two distinct positive VIDEO-drop brackets contain adequately covered, repeated Service-pressure evidence identifying the same concrete operation, and source inspection supports one minimally invasive change to that operation.**

A sampled Process span alone does not distinguish SRTP from FEC. An unsampled Service stall does not identify Process versus Forward. Worker non-CPU elapsed time does not identify a lock or scheduling cause. If those distinctions remain unresolved—or loss occurs without correlated worker/service evidence—select **no optimization** and name the exact missing observable.

Any later gain claim requires a separately authorized matched control/candidate comparison within the same cloud session. This H2 attempt measures diagnostic admissibility and, if earned, target receive-owner behavior. It does not establish a live gain or physical input-to-photon latency.

End of Oracle group: 2 lanes above.

## Coordinator reconciliation / executable H2
Both lanes completed; O1=Opus, O2=Astra. All material claims inventoried. No lane priority by order.

| Claim | Sources | Checked evidence | Disposition |
|---|---|---|---|
| H0/H1 failed, Linux validity passed,2ed0 source clean/published but no diagnostic app deployed | O1,O2 | Parent read ledger T2/T3 and exact numeric target admission JSON; source log | accepted |
| Continue one bounded hardening candidate, no playback fix before evidence/no retroactive FAIL→PASS | O1,O2 | Two material failures but unused existing timestamps observed in actual loop | accepted |
| Reuse loop-top clock for checkpoint/start except ping/sample iterations; post-match clock for end | O1 | Parent read nvst7435-7504 and7630-7701: duplicate diagnostic reads adjacent existing now; preserve ping/sample fresh boundary | accepted |
| Harness asymmetry charges instrumented outer reads only; use identical brackets and common helper APIs | O1 | Parent previously read replay207-383, outer instrumented vs ordinary finished-start confirmed | accepted |
| Clock reads dominate/20–28ns model predicts3.4–4.6% H2 | O1 | Source read counts and H1 deltas support hypothesis, not isolation or generated machine code | unresolved |
| Shared checkpoint boundary removes redundant read and retains sample-before-receive semantics | O2 | Parent helper plus adjacent checkpoint/before_receive read; covered by selected full-reuse design | accepted |
| Fixed1/16 inner splits should be strongest next candidate | O2 | It loses rare-stall localization and untried full-coverage reuse exists | rejected |
| Sparse split accounting/schema/ordinal/denominator/unsampled-tail flags/cutoff caveats needed IF sampling used | O2 | Logical coverage constraints checked against completion-bin model; sampling NOT selected now | accepted |
| Exhaustive service/receive/counters, CPU/socket and tail bounds retained; intentionalCPUmissing not errors | O1,O2 | Existing helper/API/fixture reviewed; explicit H2 invariant | accepted |
| Schema2 phase boundary disclosure, meaningful helper/idle/STUN/closure/export tests and instrumented fixture observable | O1 | Actual private schema1 + changed helper boundaries warrant version and tests | accepted |
| Symmetric metric new prospectively, report old-structure informational numbers, keep history;5% cost and5% p99 gate declared BEFORE next run | O1 | Actual bracket defect checked; no re-score older rows. Medianp99 threshold is explicit future operationalization, not waived after results | accepted |
| Gate conservative drain-rate screen, not paced liveCPU/recv/export cost; don't dilute denominator by waits | O1,O2 | Unpaced replay and actual hook vs receive-loop separation observed | accepted |
| One3pair Linux run/no cherrypick, validity failure repair disclosed; no18case/Mac timing campaign | O1,O2 | Current source fixtures/test entry point and prior receipts | accepted |
| H2 failure proves irreducible two-clock/full-localization floor and requires user scope decision | O1 | No exhaustive alternatives/cost isolation; failure proves only candidate inadmissible | rejected |
| Stop this hardening attempt if H2 fails; report precise gap, don't endlessly rerun | O1,O2 | User outcome and bounded failed candidate trail; no speculative playback changes | accepted |
| Historical stdio91787 owner confirmation usable | O1 | Preflight/ledger proves expired handle and owned controller exited | rejected |
| Fresh supported isolatedCua/operator, source+FFImarker/hash/mapped identity/wrapperON needed; network/display/private-data boundaries unchanged | O1,O2 | Current wrapper/privacy/ownership receipts and actual expired IDs; recreate original observable supported transport | accepted |
| Capture180s worker origin with startup/media/input labels, observers/no builds duringwindow; no-drop inconclusive | O1,O2 | Explicit runtime design and prior experiment limitations | accepted |
| Arbitrary five-second grace should terminate advancing exporter/worker | O2 | Long-job instructions preserve healthy originals; use observable progress and bounded export interpretation, not premature cancel | rejected |
| Restoring c10 bytes automatically if OFF build fails | O2 | Known oldc10 privacy risk and source/artifact attribution; report failure preserve evidence, don't silently reinstateunsafeartifact | rejected |
| ExplicitOFF ordinary restore/marker absence after diagnostic trial | O1,O2 | CMake cachedgate requiresOFF; keep normalartifactidentity and defaultOFF | accepted |
| Correlated repeated drop brackets + concrete operation needed; perbinCPU consistency not causal proof, SRTP vsFEC needs operation evidence | O1,O2 | Socket/CPU/bracket limits checked; minimum evidence principles accepted, no invented precision | accepted |
| Always require exactly>50%tail dominance or twoincidents to select anyfix | O1,O2 | Useful screens but no cause established by arbitrary thresholds alone; use concrete reproducible mechanism and matchedcontrols | rejected |
| Fresh isolated writable worktree mandatory for this sole same-agent threefile edit | O2 | Only this agent editing these files; pending unrelated work excluded and inspected | rejected |

Executable decision: O1 full-coverage timestamp reuse plus truly symmetric prospective admission bracket, common production/replay helper sequence. Exactly3 Rustfiles; no sparse sampling yet. Same5% cost threshold; explicitly predeclare median paired symmetricp99<=5% for H2, report historical-structure informational delta. No retroactive rescore, no livegain claims. No second review. If admitted finish actualwrapper/deployedidentity/Cua180s evidence, then explicitOFF normalrestore. If inadmissible stop this attempt with exact gap, not irreversible floor claim.

### Resumed H3 decision after user authorization
H2 completed at f5b13c5b: symmetric median cost4.71% PASS, medianp99+9.56% FAIL; no diagnosticQt deployed/capture. User explicitly resumes and permits code edits onNucBox, overriding prior local-only authoring boundary for this task. Preserve clean source attribution/normalGitpublication and unrelated work/private data.

O2 sparse inner-phase candidate is now selected because full-coverage clock reuse was tried. Exactly3Rustfiles; keepH2symmetricmetric/commonAPI/boundaryreuse. Schema3 explicitfixed1/16successordinal sampling, actual completion coverage denominators and unsampledService-tail flag. EveryReceive/Service/Between stall and raw/auth/assembly counters still observed; detailedPreprocess/Process/Forward onlyselectedcohort, no extrapolation/retrospectivephaseattribution. Same prospective5%cost+5%p99 gate andone3pairLinuxreport. Currentmeaningfultests anddefaultOFF exclusion mandatory. NoMac timing campaign/newOracle scaffolding. Ifadmitted build/prove real optimizedQt/FFI, freshownedCua one180s capture with honest startup/media/scene coverage, thenOFFrestore. No networking/display/profile changes; currentprofile2560x1080/60H26510bit420/fullscreen3440x1440@100Hz preserved despite datedskill defaults. Source may be authoredonbox but synchronize exactnormalcommit/localbranch and preserveAGENTS.