# Oracle Review

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



# Review: receive-diagnostics preparation (snapshot 2026-10-08/2349)

## Summary

This 10-file change adds a default-off `receive-diagnostics` Cargo feature. It is forwarded from FFI to core to transport, and on Linux only it compiles a one-shot, runtime-opt-in capture into the raw Mjolnir VIDEO loop. The capture records:

- Wall-clock phase spans in 100 ms completion bins.
- One bracketed thread-CPU snapshot and one read-only `SO_MEMINFO` snapshot per bin.
- A 1,024-entry pool of ≥1 ms non-receive tail spans.
- A private `create_new`/0600 JSON export, written off-thread at 180 s or on worker exit.

The diff also:

- Replaces c10's peer-text control-exit logging with typed, payload-free categories, and adds a sentinel-based logger test.
- Adds an ignored hook-overhead replay report.
- Moves FFI runtime deployment into `opennow-streamer-ffi-build`.

**Verified properties.** I checked these against the diff and found them sound:

- **Default build is untouched.** The module and every hook are gated on `cfg(all(feature, linux))`, so a default build has no module and no env lookup. Its `recv_from`/`match` semantics are unchanged (observed: `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7366-7368`, `:7413-7414`, `:7491-7520`).
- **The capture reaches the intended loop.** Mjolnir passes `rtc: None`, which routes into this loop rather than the RTC bundle (observed: `nvst.rs:5699-5710`, `:7378-7381`).
- **No CPU-clock syscalls per datagram.** `stamp()` is wall-only; CPU is read only in `sample_cpu` from `checkpoint`/`export` (observed: `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:184-196`, `:346-354`, `:374`).
- **Normal receive waits no longer fill the tail pool** (observed: `receive_diagnostics.rs:211-217`).
- **The hook `unwrap()`s cannot panic.** Every hook Option derives from the same `diagnostic.as_mut()` state, which only `checkpoint` (`nvst.rs:7492`) can clear, before the receive stamps (inferred from `nvst.rs:7494-7513`, `:7580-7601`, `:7620-7647`, `:7664-7693`).
- **Receive timestamp shift is negligible.** `received_at` now follows one extra stamp, a shift of a few clock reads (inferred from `nvst.rs:7582-7590`).
- **Control logs carry no peer text.** They emit only numeric status/close codes, fixed category strings, and `ErrorKind` debug output (observed: `native/opennow-streamer/crates/opennow-streamer-core/src/nvst_rtsp.rs:934-981`).

I found no P0 issue.

---

## P1 — Should fix

### 1. The privacy subprocess test can pass without running anything
**Gate:** before push.

- **Location:** `native/opennow-streamer/crates/opennow-streamer-core/src/nvst_rtsp_control_ping_tests.rs:340-355`, in `live_control_exit_records_server_close_code_and_reason`.
- **Problem:**
  - The parent re-executes the test binary with `--exact nvst_rtsp::control_ping_tests::…`, asserts only `status.success()`, then returns.
  - If that filter matches zero tests, libtest exits 0 with "0 passed". That happens if the module path differs, the test is renamed or moved, or the name gets a different prefix.
  - The sentinel and evidence assertions only run in the child (`:356` onward). The repository's only real-persistent-logger privacy check would then silently become a no-op.
- **Evidence:**
  - Early return after the success check (observed: `:342-355`).
  - libtest's exit-0 behavior on an empty filter is general harness behavior (inferred).
- **Fix:**
  - After the success assert, add `assert!(String::from_utf8_lossy(&output.stdout).contains("privacy_log_fixture="))`. The child prints that marker last.
  - Also check the already-captured local run output for that marker and for `1 passed`. Until then, do not count the "18 control tests" as privacy evidence.

### 2. The CMake gate is read only at configure time and sticks in the cache, so artifact identity is not self-proving
**Gate:** before build/deploy. No code change is required for push.

- **Location:** `opennow-qt/cmake/NativeRuntime.cmake:223-244`.
- **Problem:** There are three separate risks.
  - **(a) Ignored ON.** The env var is consulted only when CMake configures. If `tools/build-native.sh` runs `cmake --build` on an already-configured directory, `ON` is silently ignored and you get an ordinary artifact.
  - **(b) Sticky ON.** `ON` is written to the cache with `FORCE`, and `option()` never overrides an existing cache value. Every later configure in that build directory without the env var stays ON. That silently instruments future "ordinary" and matched-control builds, which matters for the post-capture selection rule.
  - **(c) Only the FFI library changes.** The standalone `opennow-streamer` binary is still built with `OPENNOW_STREAMER_CARGO_FEATURE_ARGS`. Hashing the bin or the Qt executable therefore does not identify the diagnostic build.
- **Evidence:**
  - Cache/env logic (observed: `NativeRuntime.cmake:225-234`).
  - The standalone bin's command uses the ordinary feature args (observed: the `opennow-streamer` `add_custom_command` in the same file).
  - The literal `OPENNOW_NVST_RECEIVE_DIAGNOSTICS_FILE` exists only inside the gated module (observed: `receive_diagnostics.rs:286-287`), so it works as a binary marker (inferred).
- **Fix:** add a mandatory receipt gate, which needs no tool installs:
  1. Record `cat <build>/streamer-ffi-features.txt`.
  2. Run `grep -a -q -F OPENNOW_NVST_RECEIVE_DIAGNOSTICS_FILE <deployed libopennow_streamer_ffi.so>`. It must succeed for the diagnostic artifact and fail for any ordinary or control artifact.
  3. After launch, find the actually loaded `libopennow_streamer_ffi.so` path in `/proc/<pid>/maps` and `sha256sum` that exact file.
  4. For any ordinary build in the same directory afterward, export `OPENNOW_DEVELOPER_RECEIVE_DIAGNOSTICS=OFF` explicitly.
  
  Optionally, make the gate non-sticky: use a plain variable that overrides the option for that configure only. The marker check is the part that actually binds identity.

### 3. No compiler has seen the instrumented path yet
**Gate:** required before deploy; recommended before push.

- **Location:**
  - Hooks in `nvst.rs:7413-7414`, `:7491-7519`, `:7579-7601`, `:7620-7647`, `:7664-7693`.
  - `receive_diagnostics.rs:281-478`, covering `start`, `checkpoint`, `export`, `Drop` and the Linux smoke test.
- **Problem:**
  - Everything that runs on the target is `cfg(all(feature = "receive-diagnostics", target_os = "linux"))`.
  - The macOS optimized feature check and the 309 macOS transport tests compiled none of it. They only prove the feature is inert off-Linux.
  - The first compile of the following will therefore happen on Linux:
    - the large `json!` export;
    - the `libc` types (`SYS_gettid` into `i64`, `SO_MEMINFO`);
    - the `#[cfg]` statement closures in the loop.
- **Evidence:** the cfg attributes cited above (observed).
- **Fix:**
  - If the Linux std target is already installed locally, run this before push:
    ```
    cargo check --manifest-path native/opennow-streamer/Cargo.toml \
      -p opennow-streamer-transport --features receive-diagnostics \
      --target x86_64-unknown-linux-gnu --all-targets
    ```
  - Otherwise, make the first NucBox step `cargo test --release -p opennow-streamer-transport --features receive-diagnostics`, plus the ordinary `cargo test -p opennow-streamer-transport`. Accept that any failure becomes a fix-forward commit.
  - Do not record the macOS feature check as a diagnostic PASS.

---

## P2 — Consider

### 4. The exporter writes unbuffered and runs on a detached thread
**Gate:** one-line fix, recommended before push.

- **Location:** `receive_diagnostics.rs:428-443`, the `serde_json::to_writer(&mut output, &value)` call in `Capture::export`.
- **Problem:**
  - `to_writer` on a raw `File` issues roughly one `write(2)` per JSON token. That is on the order of 10⁵ syscalls for 1,800 bins.
  - Those syscalls run on a thread concurrent with the live stream: at 180 s while streaming continues, or at worker exit during teardown.
  - This lengthens the window in which process exit leaves the pre-created file empty or truncated.
- **Evidence:**
  - Unbuffered writer (observed).
  - Syscall count estimated from the per-bin schema (inferred).
- **Fix:**
  - Wrap the output in `std::io::BufWriter::new(&mut output)` and flush, or use `to_vec` followed by `write_all`.
  - Evidence rule: a capture is valid only if the file parses and `receive-diag status=exported` appears in the native log.

### 5. Receive wall time cannot separate idle waiting from delayed wakeup
**Gate:** interpretation rule, no code.

- **Location:** `nvst.rs:7494-7513`; `receive_diagnostics.rs:184-196`, `:356-414`.
- **Problem:**
  - A long Receive span covers both "queue empty, blocked" and "data arrived, worker not yet scheduled". The second case is the one that produces drops.
  - Per-bin CPU deltas show both cases as non-CPU wall time.
  - `SO_MEMINFO` is sampled only at the first loop opportunity in each bin.
- **Evidence:** the code paths above (observed). The module doc already disclaims runqueue attribution (observed: `receive_diagnostics.rs:1-7`).
- **Fix:**
  - In the ledger, do not classify "drops + long Receive + low CPU" as either idle or runqueue delay. Classify a drop interval only from:
    1. non-receive phase wall time, maxima and tails;
    2. the bracketed CPU/wall ratio;
    3. post-wait bursts (`raw` count, `rmem_alloc`).
  - If the runnable-vs-sleeping split is needed, sample `/proc/<pid>/task/<tid>/schedstat` externally at ≤100 ms. Its second field is runqueue wait in ns, and the tid is already logged at `receive_diagnostics.rs:335-342`. Verify the file exists on the target kernel first.
  - Do not add worker code for this now.

### 6. Window and claim semantics need operator gates
**Gate:** operational, before live use.

- **Location:** `receive_diagnostics.rs:10-13`, `:218-222`, `:282-344`.
- **Problem:**
  - **Fixed window.** It covers the first 180 s from worker setup, including NAT and startup. It cannot be positioned around an incident, and a window with zero drop delta is inconclusive, not negative.
  - **One capture per process.** The first worker in the process claims the capture.
  - **Claim set before open.** `CLAIMED` is set before `open`, so an existing file at the path consumes that process's only capture.
  - **First-come tail pool.** Each ≥1 ms event can take two or three slots (phase, service and between).
- **Evidence:** the cited code (observed).
- **Fix:**
  - Use a fresh Qt process and a new absolute path in a 0700 directory for each capture.
  - Require `status=started role=opennow-nvst-mjolnir` in the native log. The thread name comes from `nvst.rs:5700`.
  - Label bins before `first_assembled_ns` plus a settling period as startup, not stable media.
  - Report `long_omitted`, `cutoff_spans` and `complete`.

### 7. The overhead report lacks guards and its local numbers don't transfer
**Gate:** before live use.

- **Location:** `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs:207-236`.
- **Problem:**
  - Unlike `serial_receive_cost_report`, it has no `assert!(!cfg!(debug_assertions))`.
  - No materiality threshold is declared.
  - The hooks add about six monotonic clock reads per media datagram. That is cheap only when the target clocksource supports vDSO reads, so macOS numbers say nothing about the NucBox.
- **Evidence:** the test body (observed). The clock-read count is inferred from the hook sites in `nvst.rs`.
- **Fix:**
  - Add the release assertion.
  - Declare the stop threshold before running.
  - Run the report on the NucBox with `--release --features receive-diagnostics -- --ignored --nocapture --test-threads=1`.
  - Record `cat /sys/devices/system/clocksource/clocksource0/current_clocksource`. If it is not `tsc`, stop and re-evaluate.

### 8. Minor hygiene items

- **Temp-file collisions:**
  - Two test fixtures are PID-named, opened with `create_new` plus `unwrap()`, and never removed (`receive_diagnostics.rs:491-501`, `nvst_rtsp_control_ping_tests.rs:356-362`).
  - PID reuse will eventually cause spurious test failures. Add a nanosecond suffix or remove the file after the assertions.
- **Dead code:**
  - Non-feature test builds leave `PHASES`, `SocketSample`, `Bin.socket`, `socket_errors`, `cutoff_spans` and `LongSpan.start`/`.end` unused.
  - `Stamp.cpu`, `Stats.cpu` and `cpu_covered_wall` are always `None`/0 in production.
  - If CI uses `-D warnings`, add a module-level `cfg_attr(..., allow(dead_code))`.
  - Confirm no packaging workflow builds with `--all-features`.
- **Exporter panic surface:**
  - `realtime_ns: u128` goes through the `json!` path, which unwraps `to_value` (`receive_diagnostics.rs:139`, `:437`). It fits in u64 today; storing it as u64 removes the panic path entirely.
  - `Drop` allocates a second full ~1 MB `Capture` just to swap (`:453-464`). That is acceptable at worker exit.

### 9. Publication scope: b66 and existing target logs

- **Problem:**
  - This diff fixes c10's peer-text logging. The removed formats were `server-close … reason={:?}`, `control_unsolicited head=…` and `unparseable-server-message head=…` (observed in the `nvst_rtsp.rs` diff).
  - b66 ("Record session.remote.list errors…") is HEAD and is not in this snapshot, but it will be published with this push.
- **Fix:**
  - Before push, confirm b66 persists only codes and categories.
  - Treat native logs produced by the c10-built binaries on the target as private: they may contain peer close reasons and RTSP heads.

---

Verdict: Approve with fixes — the hot path, default-off gating and privacy categories are sound; before push, make the privacy subprocess test fail when the child runs nothing (and preferably buffer the export), then treat Linux compilation/tests, on-target overhead with clocksource, and binary-marker/loaded-library identity as hard gates before deployment.

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

## Summary

The snapshot adds an opt-in Linux receive diagnostic, connects its Cargo feature to the Qt FFI build, and moves FFI copying into the target-specific build dependency. It also replaces control-exit text with payload-free error categories and numeric fields. **Direct observations:** `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7366-7414`; `/Users/danielsivan/dev/OpenNOW/opennow-qt/cmake/NativeRuntime.cmake:223-287`; `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-core/src/nvst_rtsp.rs:742-828`. I reviewed this against the **reconciled compact scope**. Two focused fixes are needed before publication: retain metadata-only bins during export, and make the optimized overhead test observe and validate the diagnostic state it measures. This is a static review of the supplied material; I did not execute tests, builds, or target checks.

## P1 — Fix before publication

### 1. Export can discard socket and CPU snapshots that were successfully collected

- **Location:** `Capture::bin`, `Capture::checkpoint`, and `Capture::export` in `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:197-205` and `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:355-435`.

- **Problem — inference from the code path:** When a checkpoint stores socket/CPU metadata in a newer bin, but no subsequent phase or counter update advances `used` into that bin, export silently excludes the collected samples. This loses an available drop/pressure bracket and CPU observation. The condition matters for partial finalization and when execution crosses the logical cutoff after a sample but before the next recorded boundary.

- **Evidence — direct observation:** Only `bin(end)` advances `self.used`. The non-complete checkpoint branch writes `bins[index].socket` and `bins[index].cpu_sample` without advancing it, while export serializes only `self.bins[..self.used]`.

  The supplied Linux smoke test already constructs this sequence: it records a processing span, sleeps for 100 ms, collects another checkpoint in a later bin, and drops the owner. Its assertions inspect the newer CPU sample **in memory** and the separate final CPU field, but do not verify that the newer socket/CPU bin survives serialization. This is a source-level observation, not a claim that I ran the test.  
  `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:502-551`

- **Fix:** In the checkpoint branch that stores a bin sample, also advance the export extent with `capture.used = capture.used.max(index + 1)`. Extend the existing smoke test to assert that the serialized output retains the latest sampled bin and its socket/CPU values. This preserves collected evidence without adding phase activity or changing the capture’s completion meaning.

### 2. The optimized overhead test leaves its diagnostic results unobserved

- **Location:** `receive_diagnostic_hook_cost_report` and `replay` in `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs:207-236` and `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_replay_tests.rs:253-374`.

- **Problem — inference:** The mandatory optimized overhead measurement does not reliably retain all the accumulator work it is intended to measure. The instrumented replay constructs a capture, updates it, and then returns only packet timings; it never consumes the collected phase, counter, or tail results. An optimizer can therefore eliminate unobserved diagnostic bookkeeping. Separately, the report cannot establish that its instrumented pass completed without diagnostic sampling errors or omissions because it never checks those results.

- **Evidence — direct observation:** `replay` creates the diagnostic with `Capture::new`, and its final assertions cover receiver, frame, and feedback semantics before returning `costs`. They do not inspect the diagnostic capture. `Capture::new` sets `output: None`, and its `Drop` implementation exports only when an output file exists, so dropping this benchmark capture does not make its collected results observable.  
  `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:142-181`  
  `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:453-464`

  **Inference qualification:** I have not inspected generated machine code and am not claiming that a particular compiler has already removed these writes. The defect is that the benchmark provides no observation preventing that optimization; surrounding the work with clock reads does not provide one.

- **Fix:** Make the completed capture observable outside the timed region—for example, pass the retained capture through `std::hint::black_box(diagnostic.as_ref())` before returning. Add a small test-only summary/assertion path that checks the expected raw/authenticated/frame counts, actual socket/CPU **sampling errors**, and unintended tail/cutoff omissions before emitting an acceptable overhead report. Keep the intentional absence of per-phase CPU samples distinct from sampling failures. These checks can remain outside packet timing and require no additional per-packet CPU calls.

## Non-blocking limits and remaining gates

The worker-setup timer, wall-only phase stamps, opportunistic 100 ms CPU/socket snapshots, excluded receive-wait tails, and logical cutoff are explicitly disclosed by the implementation. Its exported `complete` field means the timer was reached; the schema expressly limits scheduling and media-coverage conclusions. **Direct observation:** `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:1-13` and `/Users/danielsivan/dev/OpenNOW/native/opennow-streamer/crates/opennow-streamer-transport/src/nvst/receive_diagnostics.rs:437-440`. Those are accepted limitations of the revised task, not requests to restore the earlier broad tracing design.

After the two corrections, continue with the already-required actual Linux tests and optimized overhead check, then the real Qt wrapper and deployed-artifact identity verification before the single bounded capture. Their pending status is a deployment gate, not an additional code finding; this review does not require another review round.

Verdict: Request changes — Preserve already-collected diagnostic samples and make the optimized overhead measurement observe and validate its capture before publication and deployment.

End of Oracle group: 2 lanes above.

## Coordinator reconciliation (2026-10-09)
Both lanes completed. O1=Opus, O2=Astra. Each material claim has one disposition below.

| Claim | Sources | Checked evidence | Disposition |
|---|---|---|---|
| Default-off/Linux-only hooks preserve recv_from/protocol semantics and cover dedicated Mjolnir rtc=None | O1,O2 | Parent inspected manifests/CMake/helper/hooks and actual Mjolnir caller | accepted |
| Wall-only stamps, CPU queries per bin, normal waits excluded from tails, stable hook Option lifetimes | O1 | Parent full helper and hook diff; checkpoint precedes stamps | accepted |
| Control diagnostics contain safe typed categories/ErrorKind and numeric data, not peer text | O1,O2 | Parent privacy diff and exhaustive error mapping | accepted |
| Privacy child can exit0 with zero tests; require final fixture marker | O1 | Parent patch early return/status-only parent | accepted |
| Configure-time cached gate needs explicitOFF for controls; standalone bin does not identify diagnostic FFI | O1 | Parent CMake/previous wrapper read; require feature stamp+binary marker+mapped hash receipts | accepted |
| Mac tests did not compile Linux path; actual Linux tests/optimized overhead mandatory | O1,O2 | cfg and local receipts; target checks pending | accepted |
| Unbuffered JSON exporter expensive; validity needs parsed file plus exported log | O1 | Parent read File to_writer and detached export | accepted |
| Receive wall/lowCPU/drop cannot distinguish idle from runqueue delay; optional external schedstat only if needed/available | O1 | Parent sampling and explicit limitations | accepted |
| Timer starts at setup, first claim wins including failed open, fresh process/path/role required; disclose startup/tails/cutoff/media coverage | O1,O2 | Parent full helper/start/export; accepted compact scope | accepted |
| Overhead accumulator unobserved: black_box and validate state/counters/errors/omissions outside timing | O2 | Parent replay207-383: outputNone and no diagnostic consumption | accepted |
| Metadata-only checkpoint bins can disappear from export: advance used and assert serialized sample retention | O2 | Parent checkpoint/export/smoke340-557 | accepted |
| Missing release assertion/declared materiality threshold; local overhead not transferable | O1 | Parent ignored overhead test and runtime hooks | accepted |
| Non-tsc clocksource automatically requires stop | O1 | Clocksource name alone not measured cost; use actual target optimized overhead and limitations to judge admission | rejected |
| PID-only persistent fixture paths may collide; unique suffix justified | O1 | Parent test paths/privacy patch | accepted |
| Non-feature dead-code warnings/all-features packaging require checking, not assumptions | O1 | Runtime-only helper fields observed; packaging check delegated | accepted |
| Checked u64 realtime removes u128 JSON panic surface; exit-only replacement allocation acceptable | O1 | Parent helper/export inspection | accepted |
| b66 needs safe-field verification; prior c10 logs remain private | O1 | CORRECTION: parent reread main.rs274-319 confirms inherited rpc-error still persists runtime_failure_reason(message); prior categories-only assessment was wrong. Narrow main.rs/privacy regression correction authorized before push; raw evidence remains private | accepted |
| Wrapper gate, deployed marker/hash and actually mapped FFI identity, real bounded capture still required; no second review | O1,O2 | Target receipts pending, not PASS; same pair owns next unit | accepted |

No performance optimization or live latency/freezes gain established. Apply bounded corrections, publish explicit source allowlist, then target gates and one capture.