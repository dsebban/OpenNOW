## Final Prompt
<taskname="NucBox Rollout"/>

<task>Complete the documented rollout of already-pushed OpenNOW commit `6472cae764a270864cd20446f6c1beafa1bfce8b` from `nucbox-wayland-hdr` to the NucBox via `ssh chimeraos`, then verify saved-account real Qt playback end to end. The prior handoff says the commit has not yet been pulled or built on the host, and the NucBox research-workspace skill remains unchanged. Do not make product changes, run speculative performance experiments, push changes, or rewrite shared history.</task>

<architecture>The repo-local `verify-gfn-experiments` skill defines host prerequisites, single-operator GUI/account ownership, normal Qt launch and saved-account playback, CUA evidence, process identity checks, private proof storage, and cleanup. Its feature map indexes playback evidence. `doctor.py` is a read-only preflight helper. The requested native build entry point and research workspace live on the NucBox at `~/dev/gfn-client-research/tools/build-native.sh`; the local repo does not contain that host build script.</architecture>

<selected_context>
.agents/skills/verify-gfn-experiments/SKILL.md: Full operational contract for NucBox access, current host runbooks, CUA, GUI/account ownership, launch, Doctor, evidence, and cleanup.
.agents/skills/verify-gfn-experiments/features/README.md: Feature map and evidence/pass-status requirements.
.agents/skills/verify-gfn-experiments/features/playback.md: Real saved-account GUI path, CUA action/result requirements, visible input/statistics checks, and cleanup pass criteria.
.agents/skills/verify-gfn-experiments/scripts/doctor.py: Exact preflight implementation and safety/read-only behavior.
</selected_context>

<relationships>
- Host runbooks and the `operate-gfn-nucbox`, `cua-driver`, and Linux CUA guides are required prerequisites before host/GUI work; read their current remote versions because configuration and procedures may have changed.
- `doctor.py` preflight → normal Qt/QML client launch → visible signed-in library/resume → Ori resume (or normal Play if no resumable session) → visible gameplay/input and statistics confirmation → private evidence + normal close/process verification.
- The local skill mirror must be synchronized with the research-workspace copy carefully: inspect both versions and establish whether differences are pre-existing or host-only before changing anything. Do not blindly overwrite remote differing content. Preserve evidence of the comparison and retain unrelated host material.
</relationships>

<plan>
1. **Sync and build the requested revision.** On the NucBox, inspect the repository branch/status and remote skill/runbooks first; compare the skill mirror with the version from this commit, preserving any differing host content rather than replacing it blindly. Pull only with `git pull --ff-only` and keep the clone source changes limited to that operation. Synchronize the mirrored skill safely, then run `~/dev/gfn-client-research/tools/build-native.sh`. Done when the host source is at the full requested commit, the mirror sync outcome is recorded without loss of divergent content, and the build completes with its revision/artifact identity recorded. Dependency: current host runbooks and skill comparison are reviewed before mutations/build.
2. **Verify real playback and restore state.** Follow the skill’s Doctor and CUA workflow, using background delivery. Reuse an existing client when suitable; otherwise make a normal saved-account launch, resume Ori through visible GUI controls (or use normal Play if no resumable session remains), verify visible gameplay and one visible input plus the requested stream/profile/statistics facts where available, capture action/result screenshots and private proof, then close only a run-owned client and verify the recorded Qt/core process instances exited. Preserve prior account/session state and retain the private evidence directory. Done when the playback feature subchecks are marked pass/fail/skipped with artifact paths or concrete reasons, the final process/state cleanup is verified, and all unavailable or incomplete checks are reported plainly. Dependency: successful requested build and clean launch boundary; do not run measurement trials or change network/display settings.
</plan>

<ambiguities>Remote skill content may differ from the local mirror; the correct sync must be based on an explicit comparison and preserve host-only changes. Playback may resume an existing Ori cloud session or require normal Play and a new session; record which path occurred. The supplied context authorizes NucBox access, in-scope client restart/playback, and background CUA actions. Root `AGENTS.md` could not be resolved into the selected workspace; its relevant instructions were supplied directly in the conversation and should be honored.</ambiguities>

## Selection
- Files: 4 total (4 full)
- Total tokens: 6935 (Auto view)
- Token breakdown: full 6935
- Token accounting: fresh from active_tab_published

### Files
### Selected Files
.agents/
└── skills/
    └── verify-gfn-experiments/
        ├── features/
        │   ├── README.md — 346 tokens (full)
        │   └── playback.md — 1,045 tokens (full)
        ├── scripts/
        │   └── doctor.py — 2,341 tokens (full)
        └── SKILL.md — 3,203 tokens (full)


---

## Generated Plan
- Oracle group status: completed
- Oracle group: `AAAFB6D1-6231-44E8-83D7-AFF70F9FC2BB`

**Reconciling these Oracle lanes**
2 independent answers to the same request follow. Lane order is not a ranking; the first lane supplies the top-level continuation handle, and a successful follow-up through any lane's chat ID re-runs every lane.
- Read every lane through the end-of-group marker (`End of Oracle group: 2 lanes above.`). If the marker or a lane is missing, page the export or try the read-only `oracle_chat_log` with that lane's chat ID. Logs may be scoped or clipped; report any remaining gap. Do not start a follow-up just to retrieve prior text.
- Reconcile by evidence, not lane order, answer length, or model identity: check material single-lane and conflicting claims against the code, and report unresolved disagreements. A failed or partial lane is incomplete evidence.
- Before synthesizing, inventory every material claim from every lane, including single-lane claims. Merge only exact duplicates, retaining all source lanes. Begin your answer with `**Oracle reconciliation**`, state how many lanes completed and name any that did not. For every inventory item, give its source lanes, checked evidence, and exactly one disposition: `accepted`, `rejected`, or `unresolved`. Never silently omit an item.

Lanes (2):
- Oracle — `claude-opus-5-5-xhigh` — Completed — chat ID `orchestrate-nucbox-rollo-938433`
- Oracle 2 — `gpt-6-astra-pro` — Completed — chat ID `orchestrate-nucbox-rollo-E1A414`

### Oracle
- Status: Completed
- Provider: `Custom`
- Model: `claude-opus-5-5-xhigh`
- Effective effort: Provider default / not specified
- Chat: `orchestrate-nucbox-rollo-938433`



# NucBox rollout of `6472cae7…` and saved-account playback verification

All citation paths are relative to the local repo root `/Users/danielsivan/dev/OpenNOW`. **Obs** marks a direct observation in the cited text. **Inf** marks an inference, and the observations it rests on are cited next to it. Root `AGENTS.md` instructions were supplied in conversation and cannot be cited here; they still apply on top of this plan.

## 1. Summary

This is an operational rollout with no product-code changes. On `ssh chimeraos`, the work has two parts.

**Source, mirror, and build:**
- Fast-forward the host OpenNOW clone to exactly `6472cae764a270864cd20446f6c1beafa1bfce8b`.
- Reconcile the research-workspace copy of `verify-gfn-experiments` against that commit. Use a byte-exact, history-aware comparison: stale host files are updated and divergent host content is preserved.
- Rebuild with `tools/build-native.sh` and record a build identity.

**Playback verification:**
- Verify one real saved-account Qt playback through the documented CUA path, using background delivery.
- Record pass, fail, or skipped for each `playback-*` sub-feature, with evidence in a private proof directory.
- Close only the run-owned client and prove its exit by PID plus start ticks.

The targeted approach fits because every mechanism already exists: Doctor, the ordinary launch command, the playback feature file, proof conventions, and cleanup rules. The plan sequences them, adds explicit gates, and resolves the decisions the skill leaves open.

## 2. Current-state analysis

### Host layout (all derived from the skill; validate in step 1)

- **Workspace and mirror direction.**
  - Obs: the workspace is `/home/gamer/dev/gfn-client-research`. The skill copy "is mirrored from the NucBox, so re-sync both ways" (`.agents/skills/verify-gfn-experiments/SKILL.md:8-8`).
- **Host skill directory is `<ws>/.cursor/skills/verify-gfn-experiments/`.**
  - Inf. Supporting observations:
    - Every documented Doctor call is `python3 .cursor/skills/verify-gfn-experiments/scripts/doctor.py`, run from the workspace (`.agents/skills/verify-gfn-experiments/SKILL.md:50-53`, `.agents/skills/verify-gfn-experiments/SKILL.md:74-77`).
    - Doctor sets `ROOT = Path(__file__).resolve().parents[4]` (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:16-16`).
- **Mirror content is written in host-relative terms, so a verbatim byte copy is the correct sync with no path rewriting.**
  - Inf, from the same `.cursor/skills/...` command paths that appear inside the repo copy (`.agents/skills/verify-gfn-experiments/SKILL.md:50-53`).
- **Host OpenNOW clone is `<ws>/vendor/OpenNOW`.**
  - Inf. Doctor reports `source_revision` from `git -C ROOT/vendor/OpenNOW rev-parse HEAD` (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:48-49`).
- **Build artifacts.**
  - Obs: Doctor hashes `<ws>/build/opennow-qt/{opennow-qt, opennow-core, libopennow_streamer_ffi.so}` (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:38-47`).
- **Never use the local repo's `doctor.py` for host evidence.**
  - Inf. In the OpenNOW repo it would resolve `ROOT` to the repo root, which has no `build/opennow-qt` or `vendor/OpenNOW` (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:16-16`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:41-41`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:48-48`).

### Ownership and launch-boundary rules

- **Single operator.**
  - Obs: one operator owns GUI input, account RPC, and the cloud session.
  - Obs: close the GUI before `probe_regions.py` or `set-trial-profile.py`.
  - Obs: never run an authenticated standalone core beside the GUI (`.agents/skills/verify-gfn-experiments/SKILL.md:12-12`).
- **Reuse and identity.**
  - Obs: run Doctor first and reuse a client when the check allows it.
  - Obs: record PID and `/proc` start ticks, and distinguish a reused client from a run-created one (`.agents/skills/verify-gfn-experiments/SKILL.md:16-16`).
- **Close pitfalls.**
  - Obs: a Quit confirmation can return to the library with the Qt/core pair still alive.
  - Obs: a second launch can then time out in single-instance handoff (`.agents/skills/verify-gfn-experiments/SKILL.md:18-18`, `.agents/skills/verify-gfn-experiments/features/playback.md:29-29`).
- **Log archiving.**
  - Obs: between a verified shutdown and the next launch, run `tools/archive-trial-logs.py --output NEW_DIR` (`.agents/skills/verify-gfn-experiments/SKILL.md:20-20`).
  - Obs: it requires all workspace Qt/core processes to have exited and refuses existing directories (`.agents/skills/verify-gfn-experiments/SKILL.md:98-98`).
  - Obs: the exact invocation lives in `features/live-measurement.md` (`.agents/skills/verify-gfn-experiments/SKILL.md:20-20`). That file is not in the provided context.

### Doctor semantics relevant to gating

- **Process identity source.**
  - Obs: Doctor reports `pid`, `parent_pid`, `start_ticks`, `started_wall_ms`, and `expected_build` for each `opennow-qt`/`opennow-core` process (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:53-72`).
- **Running a stale or replaced binary is flagged.**
  - Obs: `expected_build` compares the raw `/proc/<pid>/exe` link against the build path, and any mismatch raises `multiple_or_unexpected_clients` (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:61-69`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:175-176`).
  - Inf: a client still running when the build replaces its binary shows a `(deleted)` exe link and is flagged. Such a client also runs pre-pull code. So a client started before the build is never suitable for reuse in this task.
- **Issue set.**
  - Obs: `required_build_missing`, `multiple_or_unexpected_clients`, `standalone_core_present_do_not_start_another_vault_owner`, `wifi_unavailable_or_disconnected`, `capability_query_failed`, `stream_preflight_failed`. Any issue gives exit 1 (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:172-189`).
- **Wi-Fi is read only.**
  - Obs: Wi-Fi state comes from `/home/gamer/.local/bin/gfn-route wifi --json` (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:75-89`).
- **Telemetry dependence.**
  - Obs: `--require-stream` depends on Qt telemetry in `.runtime/hillclimb/qt-presentation.jsonl` (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:116-139`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:184-185`).
  - Obs: ordinary playback launched without telemetry can be healthy while `--require-stream` fails (`.agents/skills/verify-gfn-experiments/SKILL.md:61-61`).
- **Possible crash.**
  - Obs: `observer()` always loads `tools/live-trial.py` (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:110-114`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:169-169`).
  - Inf: if that file is missing, Doctor fails with a traceback instead of an `issues` list.
- **What Doctor does not prove.**
  - Obs: it does not prove that the running binary corresponds to the source, and it does not verify environment variables (`.agents/skills/verify-gfn-experiments/SKILL.md:57-59`).

### Launch variants

- **Measured wrapper.**
  - Obs: `run-live-trial.sh` enables telemetry and is for measured clients (`.agents/skills/verify-gfn-experiments/SKILL.md:22-31`).
- **Ordinary playback.**
  - Obs: `env -u <experiment vars> tools/run-native.sh` (`.agents/skills/verify-gfn-experiments/SKILL.md:33-40`).

### Playback flow and pass criteria

- **Sub-features.**
  - Obs: `playback-launch`, `-resume`, `-play`, `-input`, `-stats`, `-close` (`.agents/skills/verify-gfn-experiments/features/playback.md:5-10`).
- **Steps.**
  - Obs: the driving steps (`.agents/skills/verify-gfn-experiments/features/playback.md:18-22`).
  - Obs: the pass condition (`.agents/skills/verify-gfn-experiments/features/playback.md:24-24`).
  - Obs: ready means a visible signed-in library; never infer auth from a running process (`.agents/skills/verify-gfn-experiments/SKILL.md:42-42`).
- **Profile facts.**
  - Obs: Germany, H.264 8-bit, 1920×1080, 60 FPS, frame generation off, intended decoder.
  - Obs: warn before a restart interrupts playback (`.agents/skills/verify-gfn-experiments/SKILL.md:44-44`).
- **Statistics shortcut.**
  - Obs: Ctrl+N is the default; F3 is not (`.agents/skills/verify-gfn-experiments/features/playback.md:14-14`, `.agents/skills/verify-gfn-experiments/features/playback.md:21-21`).
- **Status recording.**
  - Obs: mark each sub-feature pass, fail, or skipped with an artifact or reason (`.agents/skills/verify-gfn-experiments/features/README.md:10-10`).

### Evidence and cleanup

- **Evidence.**
  - Obs: the proof directory recipe and `proof.md` contents (`.agents/skills/verify-gfn-experiments/SKILL.md:71-80`).
- **Cleanup.**
  - Obs: close the exact window, never kill by name, and terminate only a recorded PID after a start-tick match (`.agents/skills/verify-gfn-experiments/SKILL.md:88-88`).
  - Obs: retain proof and confirm it still exists (`.agents/skills/verify-gfn-experiments/SKILL.md:92-92`).
- **Skill-change rule.**
  - Obs: after changing this skill, run Launch, Doctor, one feature, Evidence, and Cleanup against the real app (`.agents/skills/verify-gfn-experiments/SKILL.md:101-101`).
  - Inf: this rollout's playback run satisfies that requirement for the synced skill.

### Not in context (inspect on host before use)

- `tools/build-native.sh`
- `tools/run-native.sh`
- `tools/archive-trial-logs.py`
- `features/live-measurement.md`
- The host runbooks and CUA guides (`.agents/skills/verify-gfn-experiments/SKILL.md:10-10`)
- The contents of commit `6472cae…`

The local files shown may not match the commit's blobs. The commit objects in the host clone are authoritative.

## 3. Design

### A. Preconditions and ownership gate

- **Read first (read-only):**
  - `/home/gamer/.codex/AGENTS.md`
  - `/home/gamer/.agents/skills/operate-gfn-nucbox/SKILL.md`
  - `/home/gamer/.agents/skills/cua-driver/SKILL.md`
  - `/home/gamer/.agents/skills/cua-driver/LINUX.md`
  - Skip `tune-gfn-vpn-routing`, because no latency or network work is in scope (`.agents/skills/verify-gfn-experiments/SKILL.md:10-10`). Record that reason.
  - Where a runbook prescribes a more specific procedure (build arguments, CUA background-delivery syntax, session handling), it wins over this plan's defaults.
- **Create the proof directory** with the exact `umask 077` / `mktemp -d` recipe (`.agents/skills/verify-gfn-experiments/SKILL.md:73-77`). Then run `doctor.py --capabilities` into `doctor-before.json`, using the host `.cursor/skills/...` copy before any sync.
- **Gating rules for every Doctor run:**

| Doctor result | Action |
|---|---|
| traceback / no JSON | Record stderr. Diagnose read-only (likely missing `tools/live-trial.py`). Do not edit `doctor.py`. Blocker for launch. |
| `standalone_core_present_…` | **Blocker.** It is not run-owned, so do not terminate it. Report. |
| `multiple_or_unexpected_clients` before the build | Handle under the pre-existing client policy below. |
| `multiple_or_unexpected_clients` after launch | Fail `playback-launch`. Do not proceed. |
| `required_build_missing` before the build | Informational. After the build it is a blocker. |
| `wifi_unavailable_or_disconnected` / `capability_query_failed` | Record only. Never change network settings. |
| `source_revision: null` | The `vendor/OpenNOW` assumption is wrong. Locate the clone through `operate-gfn-nucbox`. If it cannot be resolved, stop. |

- **Pre-existing client policy.** A client that predates the build cannot verify the new revision (Inf above). Handle it as follows:
  1. Record its Qt and core `(pid, start_ticks)` as "pre-existing, user-owned".
  2. Post a warning that closing it interrupts any active stream (`.agents/skills/verify-gfn-experiments/SKILL.md:44-44`).
  3. Close it through its normal GUI flow immediately before the build (step 6). Restart is authorized by the task.
  4. Do not relaunch it afterwards. The final state is "no client running".
  5. If its close confirmation offers "leave cloud session running" versus "end session", choose to leave it running, which preserves the prior session state.
- **CUA session:** `start_session {"session":"gfn-verify"}`.
  - If the Linux guide indicates another live session is driving an OpenNOW window, stop: single-operator ownership cannot be established.
  - Use background delivery for all inputs, following the guide's syntax. Pass `session`, `pid`, and `window_id` on every call that accepts them (`.agents/skills/verify-gfn-experiments/SKILL.md:65-65`).

### B. Source update in `<ws>/vendor/OpenNOW` (exact-commit fast-forward)

**Read-only inspection, saved to `git-before.txt`:**
- current branch and upstream
- `HEAD` (call it `OLD`)
- `status --porcelain`
- `git submodule status`
- `git remote` names only. Do not record remote URLs; they may embed credentials.

**Gate the pull.** First run `git fetch origin` (worktree untouched). With `T = 6472cae764a270864cd20446f6c1beafa1bfce8b`, apply these rules in order:

| Condition | Action |
|---|---|
| Branch ≠ `nucbox-wayland-hdr` or HEAD is detached | Blocker. Do not check out. |
| `git cat-file -e T^{commit}` fails | Blocker (commit not reachable from the configured remote). |
| `T` is not an ancestor of `origin/nucbox-wayland-hdr` | Blocker. |
| `OLD == T` | Pull is a no-op. Record that the handoff was stale and continue to the build. |
| `OLD` is not an ancestor of `T` | Blocker. A fast-forward is impossible, and history must not be rewritten. |
| Upstream tip `== T` | Run `git pull --ff-only`. |
| Upstream tip is a descendant of `T` | Run `git pull --ff-only origin T`. This avoids overshooting past the requested commit. If the remote refuses a SHA fetch, it is a blocker. Do not fall back to `merge` or `reset`. |
| Dirty worktree and the pull refuses | Blocker. Never stash, reset, or clean. If the pull succeeds with unrelated local edits, record them unchanged. |

**After the pull**, save `git-after-pull.txt` with:
- `HEAD` (must equal the full `T`)
- `status --porcelain` (compare to before)
- `git diff --stat OLD..T`
- `git submodule status`

If submodule pointers changed, do not run `submodule update` unless `build-native.sh` or the runbook does it.

**Use the `--stat` output to set build expectations:**
- If the range touches `opennow-qt/` or `native/`, at least one artifact hash should change.
- If it touches only `.agents/` or docs, identical hashes are acceptable.
- If the range touches Qt platform, Wayland, or HDR code (plausible given the branch name), expect CUA behavior differences and note it.

### C. Skill mirror reconciliation

- **Copies.**
  - Host: `H = <ws>/.cursor/skills/verify-gfn-experiments/`. First check `readlink -f`: if `H` resolves into `vendor/OpenNOW`, the pull already synced it. Record "no-op (symlink)".
  - Also list any other non-vendor `skills/verify-gfn-experiments` copies in the workspace and treat each the same way.
  - Commit: `P = .agents/skills/verify-gfn-experiments/` at `T` in the host clone.
  - The local Mac working tree is not used as the source.
- **Snapshot before any change:** `tar` `H` into `skill-host-before.tar`. If the workspace is a git repo, record its `git status --porcelain -- .cursor/skills`.
- **Classification algorithm.** Comparison is byte-exact with no whitespace or line-ending normalization, so no host edit can be silently dropped.

```
files = relpaths(H) ∪ git ls-tree -r --name-only T -- P
for f in files:
  host = git hash-object H/f            | ABSENT
  tgt  = git rev-parse T:P/f            | ABSENT
  hist = blobs of f at every ancestor commit of T that touched it
         (git log --follow --name-only on P/f; include renamed paths)
  host == tgt                          -> same
  host ABSENT, f in no ancestor <T     -> new
  host ABSENT, f in an ancestor <T     -> divergent (host deliberately removed)
  tgt ABSENT, host ∈ hist              -> removed-upstream
  tgt ABSENT, host ∉ hist              -> host-only
  host ∈ hist (≠ tgt)                  -> stale
  else                                 -> divergent
```

- **Actions per class:**
  - `same`, `host-only`: no action. Host-only material is retained.
  - `new`, `stale`: write the `T` blob with the `T` file mode from `git ls-tree`, so `doctor.py` keeps 100755 if the commit has it. Verify the result with `git hash-object` equal to `tgt`.
  - `removed-upstream`: delete. This is safe because the host bytes equal a committed version and the tarball preserves them.
  - `divergent`: **do not modify.** Save `diff -u` of host vs `T`, and of host vs the nearest historical blob, under `skill-diffs/`. Report each such file as a pending reverse sync into the repo. That follow-up is out of scope: no repo commit and no push.
- **Output:** `skill-sync.tsv` with columns `path | class | host_blob_before | tgt_blob | action | host_blob_after`.
- **Do not commit in the workspace.** Leave changes in place and record its resulting `git status` if it is a repo.
- **After the sync, re-read the synced `SKILL.md` and `features/playback.md`.** If their procedure differs from §E–§G of this plan, the synced text wins. Note the difference in `proof.md`.

### D. Build and artifact identity

1. Read `tools/build-native.sh` without executing it, to learn its outputs, required arguments, and whether it writes a build record or touches git.
2. Invoke it with no arguments unless the runbook prescribes otherwise. Record the exact invocation.
3. Run it detached, so an ssh drop does not kill it. Tee the log to `build.log` and write the exit code to `build.exit`. Illustrative only:

```
setsid sh -c 'tools/build-native.sh >"$P/build.log" 2>&1; echo $? >"$P/build.exit"' &
```

- **Preconditions:** the pre-existing client's close is verified (Doctor shows no `opennow-qt`/`opennow-core`). This avoids replacing mapped binaries under a live process and avoids the `(deleted)` exe mismatch (Inf, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:61-69`).
- **`build-record.txt` contents:**
  - start and end UTC
  - exit code
  - `HEAD` before and after the build (must remain `T`)
  - `status --porcelain` after the build (record any tracked-file changes the build made)
  - `doctor-after-build.json` with `build.*.sha256` and `source_revision == T`
  - artifact mtimes later than the build start
  - any record the script itself writes
- **Attribution rule.** This composite is the "build record" that ties binaries to source (`.agents/skills/verify-gfn-experiments/SKILL.md:59-59`). If hashes are unchanged while the diff stat touches native or Qt code, flag the build as possibly a no-op and do not claim correspondence.
- **On failure** (non-zero exit or `required_build_missing`):
  - Stop. Do not launch.
  - Mark all `playback-*` as `skipped: build failed (build.log)`.
  - Leave the source at `T` and report it.

### E. Launch boundary and launch

1. **Boundary:** Doctor shows zero `opennow-qt`/`opennow-core` processes.
2. **Archive logs:** run `tools/archive-trial-logs.py --output "$GFN_PROOF_DIR/logs-pre-launch"`. The path must not exist yet. Use the exact invocation from host `features/live-measurement.md` if it adds arguments. Save its stdout, which is metadata only, as `archive-logs.txt`.
3. **Launch path (decided): ordinary playback, not the measured wrapper.**
   - Measurement is out of scope.
   - The ordinary command is the documented non-measured launch (`.agents/skills/verify-gfn-experiments/SKILL.md:33-40`).
   - Consequence: skip `--require-stream` and record "N/A: no telemetry by design" (`.agents/skills/verify-gfn-experiments/SKILL.md:61-61`).
   - Skip the allowlisted environment readback, since it is only required before scoring (`.agents/skills/verify-gfn-experiments/SKILL.md:59-59`).
4. **Launch through CUA** with the same `env -u` list. Illustrative shape:

```json
{"launch_path":"/bin/sh","additional_arguments":["-c",
 "cd /home/gamer/dev/gfn-client-research && exec env -u OPENNOW_LIVE_TELEMETRY … -u QT_QPA_UPDATE_IDLE_TIME tools/run-native.sh"]}
```

   - The `cd` makes the documented relative invocation independent of the daemon's working directory.
   - Confirm the argument schema with `cua-driver describe launch_app`.
5. **Identity:**
   - The returned PID is the wrapper. Run `list_windows` and Doctor (`doctor-launched.json`).
   - The run-owned Qt is the single `opennow-qt` with `expected_build: true` and `started_wall_ms` after launch.
   - The run-owned core is the `opennow-core` whose `parent_pid` equals that Qt PID.
   - Record both `(pid, start_ticks)` pairs, and target that Qt window from now on.
6. **First-screen handling:**

| Screen | Action |
|---|---|
| Signed-in library or resume view | `playback-launch: pass` |
| Login / auth prompt | `playback-launch: fail`, concrete blocker. Never enter or retrieve credentials (`.agents/skills/verify-gfn-experiments/features/playback.md:24-24`). |
| Update prompt | Choose the dismiss/later option. Never install, because that would replace the verified build. |
| One-time onboarding / what's-new | Choose skip/keep-current. If no such option exists, stop and report. |

### F. Playback verification

Take a fresh snapshot before and after every action. Prefer element tokens, and use coordinates only from the latest screenshot (`.agents/skills/verify-gfn-experiments/SKILL.md:65-65`).

1. **Requested profile (pre-stream, read-only):**
   - Open the normal settings view from the library. Screenshot the region, codec/bit depth, resolution, FPS, frame generation, and decoder values.
   - Navigate back without toggling anything.
   - If an "unsaved changes" prompt appears, discard.
2. **Session path:**
   - **Resume visible:** select it, then `playback-resume` is pass or fail on the outcome, and `playback-play` is skipped ("resumable session existed").
   - **No Resume, or Resume fails because the session expired:**
     - Record `playback-resume` as skipped or fail with the reason.
     - Use normal Play, then `playback-play` is pass or fail.
     - Note "new session allocated" (`.agents/skills/verify-gfn-experiments/features/playback.md:31-31`).
   - **Queue or loading:** snapshot about every 60 s. Treat 15 minutes with no visible progress, or an error dialog, as a fail with the screenshot.
   - **In-game menus:** after Play, allow only Continue/load-existing-save navigation. Never choose New Game, overwrite, or settings.
   - **Store or account-link prompts inside the cloud:** blocker. Enter no credentials.
3. **Input (`playback-input`):**
   - Before any other gameplay, take a pre-snapshot.
   - Send one background `press_key` jump: Space, unless the visible game prompt shows otherwise. Check the schema with `cua-driver describe press_key`.
   - Take a post-snapshot immediately.
   - **Pass only if Ori's position or pose changes.** The waterfall animates by itself, so a changed frame alone is not proof.
   - If there is no response, the result is fail ("background delivery did not reach the game"). Foreground delivery is not attempted, per the task.
4. **Statistics (`playback-stats`):**
   - Record the initial overlay state.
   - Send Ctrl+N using the schema's modifier syntax. If no overlay appears, read the configured shortcut in settings (read-only) and use that instead.
   - Screenshot the overlay and transcribe the negotiated values.
   - Toggle again and verify the overlay matches its initial state.
   - Pass when the overlay was shown, read, and restored.
5. **Profile comparison** (`proof.md` table, one row per fact):
   - Values: `matched` / `mismatched` / `unverified (not exposed)`.
   - Never change settings to fix a mismatch.
   - Any mismatch means the overall pass condition ("intended profile", `.agents/skills/verify-gfn-experiments/features/playback.md:24-24`) is not met, and must be reported.
   - "Intended decoder" is the client default for ordinary playback. Record the observed decoder.

### G. Evidence layout

```
$GFN_PROOF_DIR/
  proof.md                        # feature statuses, ownership, UTC start/end, labels, paths
  doctor-before.json              # --capabilities, pre-mutation, pre-sync doctor
  doctor-after-preexisting-close.json   # only if a client pre-existed
  doctor-after-build.json  doctor-launched.json  doctor-after-close.json
  git-before.txt  git-after-pull.txt  build.log  build.exit  build-record.txt
  skill-host-before.tar  skill-sync.tsv  skill-diffs/
  archive-logs.txt  logs-pre-launch/
  screenshots/NN-<step>-{action,result}.png
```

- **`proof.md` labels:**
  - variant: `ordinary-playback (run-native.sh, telemetry unset)`
  - workload: `Ori`
  - session path: `resume|play`
- **Ownership table:** pre-existing and run-owned Qt/core PIDs and start ticks.
- **Status table:** each `playback-*` ID with an artifact path or reason (`.agents/skills/verify-gfn-experiments/features/README.md:10-10`).
- **Privacy:** no tokens, session IDs, remote URLs, or account identifiers in text files. Screenshots stay private (`.agents/skills/verify-gfn-experiments/SKILL.md:80-82`).

### H. Cleanup

1. Close the run-owned window through CUA and accept the normal confirmation.
   - If the confirmation offers a choice: with the Resume path, leave the cloud session running; with the Play path, end the run-created session.
   - If the client returns to the library, take a fresh snapshot and close the remaining window (`.agents/skills/verify-gfn-experiments/features/playback.md:22-22`).
2. Run `doctor-after-close.json` and check `/proc/<pid>/stat` start ticks. Pass when neither recorded `(pid, start_ticks)` is present.
3. If the normal close fails, terminate only the recorded PID after confirming its start ticks still match. Apply the same rule to an orphaned recorded core. Never kill by name (`.agents/skills/verify-gfn-experiments/SKILL.md:88-88`).
4. No settings were changed, so there is nothing else to restore. The statistics overlay was already restored in §F.
5. End the CUA session according to the guide.
6. List `$GFN_PROOF_DIR` to confirm the files exist (`.agents/skills/verify-gfn-experiments/SKILL.md:92-92`).
7. Final report:
   - commit and build record
   - skill-sync outcome, including divergent files as follow-ups
   - per-feature statuses
   - session path
   - remaining processes (expected: none)
   - every unverified or incomplete item

## 4. File-by-file impact (all on the NucBox; the local repo is unchanged)

| Path | Change | Driver | Ordering |
|---|---|---|---|
| `<ws>/vendor/OpenNOW/` (worktree, `HEAD`) | Fast-forward to `T` only | §B | After the ownership gate and fetch checks |
| `<ws>/.cursor/skills/verify-gfn-experiments/**` | Per-file `stale`/`new` writes and `removed-upstream` deletes. Divergent and host-only files untouched. | §C | After `T` objects are fetched (classification); applied after the pull |
| `<ws>/build/opennow-qt/{opennow-qt,opennow-core,libopennow_streamer_ffi.so}` | Rebuilt by `build-native.sh` | §D | After the pull and after the pre-existing client exits |
| Four known native/Qt log files (paths per `archive-trial-logs.py`) | Archived into the proof directory | §E | After all processes exit, before launch |
| `<ws>/.runtime/verification-XXXXXXXX/` | New private proof directory | §G | First mutation |
| `/Users/danielsivan/dev/OpenNOW/**` | None | Task forbids product changes and pushes | — |

## 5. Risks and rollback

- **No source rollback.** The pull is not reverted on build failure; the report states the host source is at `T` with a failed build. Reverting would be an unrequested mutation.
- **Session continuity.** The build can outlast cloud-session retention. Falling back to Play is expected and is recorded as a new session.
- **Platform change in `T`.** If `T` changes the Qt windowing platform (Wayland/HDR branch), CUA screenshots or background input may not work. Treat this as recorded failures, not a reason to change display settings.
- **Input delivery.** Background delivery to the streamed game may not register, which yields `playback-input: fail` with reason.
- **Divergent skill content** stays on the host unsynced. The reverse sync to the repo is a reported follow-up.

## 6. Implementation order

1. Read the host runbooks and CUA guides (§A). Create the proof directory and write `doctor-before.json`.
2. Record ownership: Doctor processes and `list_windows`. Apply the gating table. Start the CUA session.
3. Inspect the clone and fetch. Evaluate the pull gate (§B). **Stop on any blocker before touching the user's client.**
4. Classify the skill files against `T` and snapshot `H` (§C, read-only part).
5. Pull (`--ff-only`, exact `T`). Record `git-after-pull.txt`.
6. Apply the skill-sync actions. Verify blob hashes. Re-read the synced playback procedure.
7. If a client pre-existed: warn, close it via the GUI, and verify exit (`doctor-after-preexisting-close.json`).
8. Read `build-native.sh`, run the build detached, and assemble `build-record.txt` and `doctor-after-build.json`. **Gate:** exit 0, `source_revision == T`, all artifacts present.
9. Launch boundary: archive logs, then the ordinary CUA launch. Record identity in `doctor-launched.json`. Mark `playback-launch`.
10. Read the requested profile in settings. Take the Resume or Play path. Run the input check, then the statistics check and restore. Build the profile table.
11. Close the run-owned client and verify both identities exited (`doctor-after-close.json`). End the CUA session.
12. Finalize `proof.md`, confirm the proof files exist, and deliver the final report.

Steps 5–6 and step 8 each mutate the host. Steps 7–9 must run back to back, with no client launched between the close and the build.

### Oracle 2
- Status: Completed
- Provider: `Custom`
- Model: `gpt-6-astra-pro`
- Effective effort: Provider default / not specified
- Chat: `orchestrate-nucbox-rollo-E1A414`



## 1. Summary

Use a **targeted operational rollout**, with no new product implementation or refactor: deploy the exact requested commit, synchronize the verification skill through a comparison that preserves host changes, build with the existing NucBox entry point, and verify the resulting client through normal saved-account Qt playback. Keep source identity, build identity, running-process identity, and visible playback evidence as separate acceptance gates. **Inference:** this approach fits the supplied architecture because the existing skill already defines launch, verification, evidence, and cleanup, while Doctor explicitly does not establish source-to-binary correspondence or saved-account readiness. (`.agents/skills/verify-gfn-experiments/SKILL.md:14-20`, `.agents/skills/verify-gfn-experiments/SKILL.md:57-61`)

This response is a plan only. No host access, pull, build, synchronization, or playback verification has been performed.

**Notation and evidence:** `T` means the user-requested commit `6472cae764a270864cd20446f6c1beafa1bfce8b`; `R` means `/home/gamer/dev/gfn-client-research`. The branch `nucbox-wayland-hdr`, SSH alias `chimeraos`, and `R/tools/build-native.sh` are supplied task requirements, not independently verified host facts. References below use repository-relative `path:start-end` ranges counted from the supplied file bodies. **Direct observation** means an observation of that frozen text or code; **inference / decision** identifies the resulting implementation choice.

## 2. Current-state analysis

### Responsibilities and ownership

| Component | Existing responsibility and contract | Evidence |
|---|---|---|
| Verification skill | Defines the NucBox research workspace, native Qt/QML client with embedded Rust streamer, required host documentation, and single-operator account/session ownership. | **Direct observation:** `.agents/skills/verify-gfn-experiments/SKILL.md:8-12` |
| `doctor.py` build inspection | Hashes `opennow-qt`, `opennow-core`, and `libopennow_streamer_ffi.so` under `R/build/opennow-qt`; separately reads Git HEAD from `R/vendor/OpenNOW`. It does not build them or connect their hashes to that HEAD. | **Direct observation:** `.agents/skills/verify-gfn-experiments/scripts/doctor.py:38-50` |
| `doctor.py` process inspection | Discovers Qt/core executable names through `/proc`, records `pid`, `parent_pid`, `start_ticks`, estimated `started_wall_ms`, and whether the executable path matches the expected build path. | **Direct observation:** `.agents/skills/verify-gfn-experiments/scripts/doctor.py:53-72` |
| `doctor.py` observer inspection | Imports the host’s `tools/live-trial.py`, uses its `parse_qt` and `healthy` functions, reads the last approximately 1 MiB of presentation telemetry, and checks recent samples against the selected client’s estimated start time. | **Direct observation:** `.agents/skills/verify-gfn-experiments/scripts/doctor.py:110-152` |
| CUA playback workflow | Establishes saved-account readiness through visible GUI state, selects Resume/Play, checks game input and statistics, and closes the exact client instance. | **Direct observation:** `.agents/skills/verify-gfn-experiments/features/playback.md:18-24` |
| Evidence and acceptance | Requires per-feature `pass`, `fail`, or `skipped` results with artifact paths or concrete reasons; a successful launch alone does not prove the remaining features. | **Direct observation:** `.agents/skills/verify-gfn-experiments/features/README.md:10-12` |

### Existing control and data flow

**Direct observation:** `main()` collects build, process, Wi-Fi, and receive-buffer information; selects a client only when exactly one expected Qt process exists; obtains observer state; optionally enumerates advertised VA-API capabilities; and emits a schema-1 JSON object plus exit status. Its classifications are `needs_attention`, `ready_for_gui`, and `ready_for_measurement_preflight`. (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:155-189`)

**Direct observation:** `ready_for_gui` is an inspection/launch preflight, not proof of authentication. Ordinary playback without telemetry can be healthy while `--require-stream` fails. The helper’s `expected_build` field compares executable paths, not running executable contents. (`.agents/skills/verify-gfn-experiments/SKILL.md:57-61`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:61-69`)

**Inference / decision:** the rollout must establish this chain explicitly:

1. Requested commit → checked-out source.
2. Checked-out source → completed build invocation and artifact hashes.
3. Completed artifacts → the exact Qt/core instances used for verification.
4. Those instances → visible saved-account GUI flow.
5. GUI actions → observed gameplay, input response, statistics, and cleanup.

A result at one boundary must not substitute for evidence at another. This follows directly from the separate build/process inspections and the GUI-only authentication contract. (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:38-72`, `.agents/skills/verify-gfn-experiments/features/playback.md:18-24`)

### Important layout constraint

**Direct observation:** Doctor calculates `ROOT` using `Path(__file__).resolve().parents[4]`, while the documented host invocations use `.cursor/skills/verify-gfn-experiments/scripts/doctor.py`. The supplied repository copy lives under `.agents/skills/...`. (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:15-16`, `.agents/skills/verify-gfn-experiments/SKILL.md:50-53`)

**Inference / decision:** inspect the host’s `.cursor`/`.agents` layout before synchronization. The installed helper’s resolved root must be `R`. A symlink directly into `R/vendor/OpenNOW/.agents/...` would resolve Doctor’s root to the vendor checkout and direct its build and tool lookups to the wrong location. Preserve a working host alias arrangement; do not replace it with that shortcut. (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:15-16`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:40-48`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:110-117`)

### What remains unknown

The frozen context does not contain the current host runbooks, active host mirror, build/launch wrapper implementations, or the target commit’s complete diff. These are **execution-time validation gates**, not facts to infer from the handoff. In particular, inspect the existing builder to establish its source directory, output locations, and completion behavior before invoking it.

No new application types, Qt/Rust interfaces, Doctor arguments, or JSON schema changes are needed. Preserve Doctor’s existing CLI and schema-1 output. **Inference:** the missing work is deployment and verification, using the interfaces already present. (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:155-189`)

## 3. Design

### 3.1 Operator, prerequisites, and proof ownership

**Direct observation:** host work requires the current global runbook and `operate-gfn-nucbox` skill; GUI work additionally requires the CUA skill and Linux guide. The workflow permits only one operator to own GUI input, account RPC, and the cloud session. Separate XDG directories do not isolate the credential vault. (`.agents/skills/verify-gfn-experiments/SKILL.md:10-12`)

**Decision:** the implementer must first read these current remote files:

- `/home/gamer/.codex/AGENTS.md`
- `/home/gamer/.agents/skills/operate-gfn-nucbox/SKILL.md`
- `/home/gamer/.agents/skills/cua-driver/SKILL.md`
- `/home/gamer/.agents/skills/cua-driver/LINUX.md`

Resolve operational details from those files before mutations or GUI actions. The supplied task already authorizes NucBox access, the required client restart, ordinary Play/Resume, and background CUA delivery; those routine actions do not need another confirmation. An unreadable required runbook is a prerequisite failure to report, not a reason to invent its contents.

**Direct observation:** evidence must use a new private directory under `R/.runtime`, created before launch or reuse, with screenshots and `proof.md` recording ownership, UTC boundaries, covered features, and artifact paths. (`.agents/skills/verify-gfn-experiments/SKILL.md:69-80`)

**Decision:** create one proof directory for the rollout and retain:

- `proof.md`: stage outcomes, source/build identity, mirror decisions, client ownership, playback results, and cleanup.
- `doctor-before.json`, `doctor-after-build.json`, and `doctor-final.json`.
- `source-state.txt`: initial/final commit, branch, tracking information, cleanliness, and pull result; omit credential-bearing remote URLs.
- `build.log` and a build receipt containing exit status, UTC start/end, resolved source/output paths, and the three artifact hashes.
- `mirror/`: comparison manifest, original host files, target files, merged candidates where applicable, and activation outcome.
- GUI action/result screenshots named by playback feature.
- A separately named log archive created at the verified shutdown boundary.

Record initial account/session **state**, not account identifiers or session IDs: signed-in library, resume offered, active playback, authentication required, or unknown. Also record window/fullscreen and statistics-overlay state so temporary GUI inspection can be reversed. **Inference:** this supplies the evidence and restoration data required by the existing contract without adding a new persistence service or application schema. (`.agents/skills/verify-gfn-experiments/SKILL.md:80-92`, `.agents/skills/verify-gfn-experiments/features/playback.md:21-24`)

### 3.2 Exact revision selection and safe skill synchronization

#### Establish the inputs before changing the host checkout

The source update method and exact target are **task requirements**. Record the host checkout’s initial commit as `H`; inspect its status, active branch or detached state, configured remotes, and any unfinished Git operation.

Apply these fixed rules:

- Preserve the current branch/tracking configuration.
- Treat tracked modifications, non-ignored untracked content, an unfinished Git operation, or concurrent checkout mutation as a blocker.
- Do not stash, reset, switch branches, clean files, or repair history.
- Obtain desired skill files from the **committed tree at `T`**, not from an arbitrary working-tree copy.
- Prepare and review the skill comparison before pulling or building.

Use the initiating repository’s existing `T` object to export the desired skill subtree into private staging. Read the corresponding base files from the host’s pre-pull commit `H`. If the exact target material cannot be obtained, stop before deployment; the rendered prompt is not a byte-accurate substitute for Git blobs.

#### Use a three-way comparison

For each path represented in the old or target committed skill subtree, compare:

- **Base:** committed file at `H`.
- **Host:** active research-workspace mirror.
- **Target:** committed file at `T`.

The instruction to preserve divergent mirror content is a **task requirement**. The existing skill additionally says current host observations matter when dated configuration instructions conflict. (`.agents/skills/verify-gfn-experiments/SKILL.md:8-10`)

Use this complete decision table, treating absence as a value and comparing file kind and relevant mode bits as well as content:

| Comparison | Required result |
|---|---|
| Host equals Target | Keep it; record `identical`. |
| Host equals Base | Apply Target, including an intentional target deletion; record `updated_from_target`. |
| Target equals Base, but Host differs | Keep Host; record `host_change_preserved`. |
| Both changed and a text three-way merge is conflict-free | Stage the merge; review it for contradictory operational instructions; record `merged` only after that review. |
| Overlapping unresolved changes, edit/delete conflicts, incompatible file types, or divergent binary files | Record `blocked_conflict`; leave the active mirror unchanged. |

Start the candidate from a complete copy of the active mirror. Preserve host-only files outside the old/target tracked-path union unchanged. A backup does not justify discarding their active contents.

For the known four files, inspect the merged result specifically for consistency among the launch procedure, playback instructions, feature-map coverage, and Doctor’s root/import behavior. **Inference:** these are the dependency boundaries that can make a superficially successful copy unusable. (`.agents/skills/verify-gfn-experiments/SKILL.md:33-65`, `.agents/skills/verify-gfn-experiments/features/README.md:5-12`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:15-16`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:110-114`)

The exact additional skill paths, if any, must be enumerated from the committed subtree before mutation. Apply the same rules to them; do not invent filenames from the directory-only tree.

#### Pull the immutable target

Once the comparison is approved and the required shutdown boundary has been completed, use **`git pull --ff-only` with the verified source remote and the explicit full target SHA**. Do not rely on an unqualified pull selecting the right branch or on the remote branch tip remaining unchanged.

If HEAD already equals `T`, record the source stage as already satisfied. Otherwise:

- A rejected fast-forward or refusal to fetch the explicit target is a source-stage failure.
- After success, require full HEAD equality with `T` and a clean source tree.
- Any other resulting HEAD blocks the build.
- Keep the source changes limited to that pull operation, as requested.

#### Activate the mirror as one controlled change

Publish the reviewed candidate while no verification helper is running. Preserve the old directory and its metadata, retain any established alias arrangement, and install the candidate at the existing research-workspace location. If directory replacement fails halfway, restore the prior active directory before proceeding.

Then verify that:

- Doctor’s resolved `ROOT` is `R`.
- Its expected host dependencies, including `tools/live-trial.py`, exist at that root.
- The installed files match the reviewed candidate manifest.
- Retained host-only material is still present.

**Inference:** activation must be treated as a complete-directory operation because publishing a new Doctor with old companion instructions, or changing its resolved root, would break the documented workflow even though individual files copied successfully. (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:15-16`, `.agents/skills/verify-gfn-experiments/scripts/doctor.py:110-114`, `.agents/skills/verify-gfn-experiments/SKILL.md:101-101`)

### 3.3 Client lifecycle and build attribution

#### Decide reuse from evidence

**Direct observation:** the skill permits reuse, requires PID/start-tick identity, and distinguishes reused user processes from processes created by the verification run. It also warns that a replacement launch can hand off to an older client and leave the intended launch environment unapplied. (`.agents/skills/verify-gfn-experiments/SKILL.md:16-18`)

**Decision:** reuse only when an existing receipt already establishes all of the following:

1. The completed build used source `T`.
2. The current three artifact hashes match that build receipt.
3. The exact running Qt/core instances are attributable to that build.
4. The client is using the ordinary playback launch configuration.
5. No artifact replacement is needed.

Doctor’s path match alone is insufficient. If any attribution is missing, perform the already-authorized normal restart. The handoff’s “not yet pulled or built” statement is not a live observation; inspect before deciding.

#### Establish a clean replacement boundary

Before replacing a client:

1. Capture its current GUI state.
2. Record the Qt PID/start ticks and the existing core child’s PID/start ticks.
3. Notify the user before interrupting active playback.
4. Complete the normal GUI close flow.
5. If confirmation returns to the library, close the remaining application window.
6. Verify the recorded instances have exited.
7. Confirm no other workspace Qt/core instance remains before archiving logs or replacing build artifacts.

**Direct observation:** returning to the library is not sufficient; verified process exit is the required boundary, and log archiving belongs after all workspace Qt/core processes exit. (`.agents/skills/verify-gfn-experiments/SKILL.md:18-20`, `.agents/skills/verify-gfn-experiments/SKILL.md:98-98`)

Do not issue a second launch after a timeout until a fresh snapshot and process inspection establish what happened to the first. Preserve unsuccessful handoff attempts as evidence.

#### Build and bind the outputs

Run the user-specified `R/tools/build-native.sh` with its documented normal invocation and no invented build flags. First inspect the script to verify that it consumes the intended checkout and produces the outputs Doctor will inspect. If it targets another source tree or requires additional source mutations, record that mismatch rather than attributing its output to `T`.

The build receipt must contain:

- Source HEAD and cleanliness immediately before and after the build.
- The actual source/output paths selected by the wrapper.
- Start/end times and exit status.
- SHA-256 hashes for:
  - `R/build/opennow-qt/opennow-qt`
  - `R/build/opennow-qt/opennow-core`
  - `R/build/opennow-qt/libopennow_streamer_ffi.so`

**Inference:** these fields close the attribution gap between Doctor’s independent source-revision and file-hash observations. (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:38-50`, `.agents/skills/verify-gfn-experiments/SKILL.md:59-59`)

A failed or interrupted build leaves the stage **incomplete**, even if all three filenames exist from an earlier build. Do not launch those files as evidence of the requested revision.

For a fresh launch, record the newly observed Qt/core identities and verify expected executable paths and parentage. Verify the launch wrapper’s library selection against the recorded streamer artifact; if necessary, inspect only the loaded mapping for `libopennow_streamer_ffi.so`. If that attribution cannot be established, report the library identity as unverified.

Maintain a single build/launch owner. If SSH delivery is interrupted, establish whether the recorded build or launch is still running before retrying; an unknown completion state is not permission to start another instance.

### 3.4 Doctor and ordinary saved-account playback

#### Interpret Doctor by stage

**Direct observation:** Doctor reports missing artifacts, unexpected/multiple clients, standalone core ownership, Wi-Fi availability, optional capability-query failure, and optional stream-preflight failure as separate issues. (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:172-185`)

**Decision:** retain those distinctions:

| Finding | Rollout handling |
|---|---|
| Required build missing before deployment | Record it; the requested build addresses this stage. |
| Required build missing after a successful build claim | Fail build acceptance. |
| Multiple/unexpected clients or standalone core | Resolve ownership before launch or replacement; do not terminate an unrelated process. |
| Network unavailable | Record the playback blocker; do not change networking. |
| VA-API capability enumeration failed | Report advertised capability as unavailable; do not infer the live decoder from it. |
| Telemetry absent during ordinary playback | Report measurement preflight as unavailable; do not treat that alone as failed playback. |

The last two distinctions are explicit in the skill. (`.agents/skills/verify-gfn-experiments/SKILL.md:57-61`)

Run Doctor before GUI work, after unexpected behavior, and after the build. Use the existing capability query outside gameplay interaction. `--require-stream` is not an acceptance prerequisite for this ordinary-playback rollout.

#### Launch with the ordinary environment

**Direct observation:** `tools/run-native.sh` preserves caller-supplied experiment variables. The skill supplies an ordinary-playback invocation that removes seven named variables. (`.agents/skills/verify-gfn-experiments/SKILL.md:33-40`)

**Decision:** use that documented invocation, removing:

- `OPENNOW_LIVE_TELEMETRY`
- `OPENNOW_EMBEDDED_PUBLISH_POLL_MS`
- `OPENNOW_CONTINUOUS_VIDEO_UPDATE`
- `OPENNOW_NATIVE_VIDEO_BACKEND`
- `OPENNOW_EMBEDDED_READY_WAKE`
- `QSG_RENDER_LOOP`
- `QT_QPA_UPDATE_IDLE_TIME`

Launch in the desktop-user context prescribed by the current host guide, then attach CUA session `gfn-verify` to the actual returned/discovered Qt PID and window. Use the current CUA guide’s **background delivery** mechanism for GUI actions; its exact schema is not supplied here and must be read rather than guessed.

This requires no new launcher, global environment change, or product setting.

#### Drive one observed GUI transition at a time

**Direct observation:** playback requires fresh snapshots, actual returned element tokens or coordinates from the current screenshot, and visible action results. Authentication must be established through the normal GUI. (`.agents/skills/verify-gfn-experiments/SKILL.md:42-42`, `.agents/skills/verify-gfn-experiments/SKILL.md:65-65`, `.agents/skills/verify-gfn-experiments/features/playback.md:18-24`)

**Decision:** follow this sequence:

1. **Saved-account readiness:** capture the signed-in library or resume view. If authentication is expired, mark the account-path check failed and dependent playback checks skipped. Do not retrieve credentials or substitute an account RPC.
2. **Session selection:** choose visible Ori **Resume** when offered. If no resumable session remains, choose normal **Play**. Record which occurred; Play starts a new session/comparison block even when game and region match.
3. **Gameplay:** wait for visible real gameplay, retaining loading/error evidence. A process, window, loading screen, or static menu is insufficient.
4. **Input:** send one press-and-release using an established movement/jump binding supported by the visible game state. Confirm a player response, not merely a changed waterfall or background animation. If the binding cannot be established, mark input unverified rather than guessing.
5. **Statistics:** use the configured shortcut, normally Ctrl+N, with modifier syntax taken from the current CUA schema. Capture the statistics view and any necessary normal settings view.
6. **Restore inspection state:** restore the previous overlay state and any window state changed by the verification.

The Resume/Play distinction, input procedure, shortcut, and unavailable-field handling are supplied explicitly. (`.agents/skills/verify-gfn-experiments/features/playback.md:19-24`, `.agents/skills/verify-gfn-experiments/features/playback.md:31-33`)

Treat delayed or missing action responses as uncertain outcomes: take a fresh snapshot before deciding the next action. Do not resend Play, Resume, launch, or close against stale state.

#### Check the profile without changing it

**Direct observation:** the documented profile is Germany, H.264 8-bit, 1920×1080, 60 source FPS, frame generation off, and the intended decoder; the skill distinguishes source FPS from display refresh. (`.agents/skills/verify-gfn-experiments/SKILL.md:44-44`, `.agents/skills/verify-gfn-experiments/SKILL.md:55-55`)

**Decision:** record a separate result for each fact:

| Fact | Verification rule |
|---|---|
| Region | Visible Germany indication. |
| Codec and bit depth | H.264 and 8-bit where the GUI exposes them; “H.264” alone does not prove bit depth. |
| Resolution | Visible 1920×1080 negotiation. |
| Source FPS | Visible 60 FPS source profile. |
| Frame generation | Visible disabled state. |
| Decoder | Match the ordinary baseline decoder specified by the current host runbook; if none is specified, record the observed value without asserting a match. |

A visible mismatch fails that fact. An absent field is `skipped` with an explicit “not exposed” reason. Neither case authorizes changing region, decoder, network, display refresh, or profile to obtain a pass.

### 3.5 Coverage, cleanup, and final result

**Direct observation:** every playback sub-feature has a distinct entry, and the feature map requires explicit outcomes with evidence. (`.agents/skills/verify-gfn-experiments/features/playback.md:3-10`, `.agents/skills/verify-gfn-experiments/features/README.md:10-12`)

Use this acceptance table in `proof.md`:

| Feature | Pass condition | Legitimate skipped case |
|---|---|---|
| `playback-launch` | Fresh requested-build launch reaches the normal signed-in view. | Verified target client reused; fresh account restoration was not retested. |
| `playback-resume` | Visible Resume reaches gameplay. | No resumable session; normal Play used. |
| `playback-play` | Visible Play reaches gameplay. | Existing session resumed. |
| `playback-input` | One controlled input produces a visible player response. | Gameplay blocked or binding could not be established. |
| `playback-stats` | Statistics inspected, individual profile facts recorded, overlay restored. | View unavailable or playback blocked; identify missing facts individually. |
| `playback-close` | Run-owned client and recorded core child exit after normal close. | Reused user client intentionally preserved. |

Do not turn legitimate skipped alternatives into passes. A successful reused-client check must say startup was not retested.

**Direct observation:** cleanup preserves reused user clients, closes run-owned instances through normal GUI flow, checks exact PID/start-tick identities, and retains evidence. Normal termination fallback must target the recorded instance, never a process name. (`.agents/skills/verify-gfn-experiments/SKILL.md:88-92`)

**Decision:** after the final normal close action, allow 30 seconds for exit. If normal close fails, revalidate the identity and terminate only the exact instance whose closure this task owns; allow another 15 seconds. Apply the same identity check to its recorded core child if it remains. A surviving instance leaves cleanup incomplete and blocks another launch. A reused PID with different start ticks is not the original process and must not be signaled.

Preserve saved-account configuration and credentials. Prefer the normal close/disconnect path that preserves resumability when the GUI offers it; do not claim that the cloud session will remain alive indefinitely.

Finally, verify that all referenced evidence files still exist. Report three independent outcomes:

- **Rollout:** exact source, mirror synchronization, build identity.
- **Playback:** exercised path, gameplay/input results, profile facts and unavailable checks.
- **Cleanup:** exited instances or intentionally retained user client, restored GUI state, unresolved processes.

These are functional results. Screenshots and Doctor output do not support a performance-improvement claim. (`.agents/skills/verify-gfn-experiments/SKILL.md:80-84`)

## 4. File-by-file impact

The following names are established by the supplied evidence or explicit task. The target Git diff and full mirror inventory are not supplied; enumerate any additional committed deployment paths during preflight rather than inventing them.

| File or path | Change and reason | Ordering dependency |
|---|---|---|
| `R/vendor/OpenNOW` checkout | Advance to `T` only through the authorized fast-forward pull. No authored source edits or branch/tracking changes. | Cleanliness/ownership inspection and reviewed mirror candidate. |
| `R/.cursor/skills/verify-gfn-experiments/SKILL.md` | Install or merge the committed skill instructions while retaining unrelated host changes. Preserve the actual resolved host layout. | Three-way comparison; controlled mirror activation. |
| `R/.cursor/skills/verify-gfn-experiments/features/README.md` | Synchronize the feature map and evidence contract from the target candidate. | Same activation as companion skill files. |
| `R/.cursor/skills/verify-gfn-experiments/features/playback.md` | Synchronize playback procedures and sub-feature criteria. | Same activation; current CUA documentation reviewed before use. |
| `R/.cursor/skills/verify-gfn-experiments/scripts/doctor.py` | Synchronize the reviewed target/helper content. No additional implementation or interface changes. | Correct resolved `ROOT`; host dependencies available. |
| Additional paths in the committed skill manifest | Apply the same fixed three-way rules; preserve unrelated mirror entries. Record every affected path before activation. | Complete inventory and conflict review. |
| `R/build/opennow-qt/opennow-qt` | Produced/replaced by the existing builder and hashed for attribution. | Source `T`; verified shutdown boundary. |
| `R/build/opennow-qt/opennow-core` | Produced/replaced by the existing builder and hashed; later matched to its process identity. | Same build gate. |
| `R/build/opennow-qt/libopennow_streamer_ffi.so` | Produced/replaced by the builder and included in build/runtime attribution. | Same build gate. |
| New private `R/.runtime/verification-…/` directory | Stores receipts, comparisons, backups, screenshots, and outcomes. | Created before launch/reuse; retained after cleanup. |

The documented logical `.cursor` paths above may resolve through an existing valid `.agents` alias; preserve that arrangement after verifying Doctor still resolves `R`.

**Direct observation supporting these impacts:** the supplied helper names the source and artifact locations, while the skill names the host helper entry point and evidence location. (`.agents/skills/verify-gfn-experiments/scripts/doctor.py:38-50`, `.agents/skills/verify-gfn-experiments/SKILL.md:50-53`, `.agents/skills/verify-gfn-experiments/SKILL.md:73-80`)

The existing host `build-native.sh`, `run-native.sh`, `archive-trial-logs.py`, `live-trial.py`, and `probe_vaapi.py` are inspected or invoked through their established roles. This plan does not modify them or add Qt, Rust, QML, or test source files.

## 5. Risks and migration

No new application-data schema migration is proposed. The material deployment risks are:

- **Stale runtime attribution:** an older running client can survive a handoff even when newer files are on disk. The verified shutdown and fresh process receipt address this. **Direct observation:** `.agents/skills/verify-gfn-experiments/SKILL.md:18-18`; **inference / mitigation:** the build and runtime gates above.
- **Divergent host skill content:** a legitimate merged mirror may differ from the target blobs. Record the actual deployed hashes and preservation decisions; do not describe it as byte-identical to `T`. This follows from the task’s explicit preservation requirement.
- **Partial deployment:** mirror activation can restore its preserved predecessor if activation fails. A failed build must remain an incomplete rollout; the task’s source-change restriction does not authorize an automatic reset to an earlier commit.
- **Session continuity:** normal Play can allocate a new session after Resume expires. Record that transition rather than claiming the original session was preserved. **Direct observation:** `.agents/skills/verify-gfn-experiments/features/playback.md:31-31`.

## 6. Implementation order

1. **Read current host prerequisites and establish ownership.** Create the private proof directory, inspect windows/processes, and record the initial source, GUI, and session state.
2. **Inventory and stage synchronization.** Obtain exact target blobs, preserve the active host mirror, complete the three-way comparison, and resolve the helper’s actual root. Stop on unresolved conflicts.
3. **Choose reuse or replacement.** Reuse only with existing target-build/runtime evidence. Otherwise warn before interrupting playback, close the recorded client normally, verify Qt/core exit, and archive prior logs.
4. **Advance source to the exact target.** Perform only the authorized pinned fast-forward pull when needed. Verify full HEAD equality and source cleanliness.
5. **Activate the reviewed mirror.** Publish the complete candidate as one controlled operation, verify hashes and root resolution, and preserve the predecessor.
6. **Build and record attribution.** Run the existing host builder, retain its completion result, verify source state, and hash all three outputs. Do not proceed on a partial build.
7. **Run post-build Doctor and real GUI verification.** Launch with the documented ordinary environment when necessary; perform saved-account readiness, Resume or Play, visible gameplay, one input, and statistics inspection using background CUA.
8. **Restore and close according to ownership.** Restore overlay/window state, close run-owned instances, verify exact process exits, retain a reused user client when appropriate, and complete the feature/result tables.

**Atomic boundaries:** activate the mirror as one coherent change; treat successful source selection plus completed build attribution as the gate before any new client launch. Neither an intermediate checkout nor a partially written build is eligible for playback acceptance.

End of Oracle group: 2 lanes above.



> 💡 The 2 Oracle lanes above are independent answers; reconcile them using the guidance at the top of the group. Lane order is not a ranking.

## Orchestrator decisions and live checklist

**Oracle reconciliation:** Both lanes completed. Inventory of material claim groups follows; remote observations remain execution gates.

| Claim | Lanes | Checked evidence | Disposition |
| --- | --- | --- | --- |
| Operational rollout, exact commit, no product changes/push/tuning | Both | User scope; clean local Git status | accepted |
| Host layout/runbooks, single operator and private evidence | Both | SKILL.md | accepted |
| Separate source/artifact/runtime/GUI identity gates | Both | doctor.py 38–72; SKILL.md | accepted |
| Direct vendor skill symlink is an acceptable sync no-op | 1 | doctor.py ROOT resolves parents[4], confirming lane 2 root warning | rejected |
| Preserve host-only/divergent skill files and compare before sync | Both | User preservation rules, mirror note | accepted |
| Historical classification versus full directory three-way activation | 1/2 | Host baseline/layout not inspected yet | unresolved |
| Mandatory newer mtimes/hash changes after incremental build | 1 | Untouched targets may remain unchanged; receipt is needed | rejected |
| Exact pinned ff-only pull and preservation of unrelated work | Both | User deployment contract | accepted |
| Observable persistent build; no duplicate invocation/retry | Both | Long-running contract | accepted |
| Ordinary wrapper/env exclusions, no scoring | Both | SKILL.md launch | accepted |
| Saved-account GUI, Resume/Play alternative, real input, configured stats shortcut and per-field profile evidence | Both | SKILL.md and playback feature map | accepted |
| Exact Qt/core cleanup; restore inspection state; retain proof | Both | SKILL.md cleanup | accepted |
| Arbitrary queue timeouts, guessed Space binding, automatic cloud-session termination after Play | 1/2 | Current host/game state unknown; preserve existing state | rejected |
| Dirty checkout always blocks versus preserving unrelated edits | 2/1 | Actual dirt/conflict not inspected | unresolved |
| Missing runbook, unexpected owner, network/capability checks are distinct gates | Both | SKILL.md prerequisites and Doctor | accepted |
| Check mapped streamer library identity when needed | 2 | Doctor path identity insufficient for mappings | accepted |

Use the smallest safe sync after byte comparison; no speculative sync tooling. Preserve divergence and report it. Do not require mtime/hash changes for unchanged build targets. No foreground GUI delivery without explicit approval. Remote unresolved decisions belong to the execution agent, based on observed state.

- [ ] Item 1 — **PARTIAL/BLOCKED**: independently verified remote clean HEAD 6472cae764a270864cd20446f6c1beafa1bfce8b; six mirror files identical, host intro preserved, correct Doctor root. Background Quit unavailable; termination denied for foreign process. Complete snapshot exposes no semantic Quit. Original Qt/core identities remain alive. Build never invoked. Private receipt: /home/gamer/dev/gfn-client-research/.runtime/verification-deploy-tgLBRD4n/item1-shutdown-investigation-receipt.json. Deploy/sync/build. Done: exact source commit; safe mirror reconciliation and Doctor root; successful build receipt; private evidence; clean launch boundary. Scope: research skill mirror/build/private proof; vendor source only ff-only pull. Dependency: remote prerequisites. Size: bounded operational.
- [ ] Item 2 — **NOT STARTED**: successful build dependency unsatisfied; no second GUI operator dispatched. Real playback/cleanup. Done: per-feature outcomes and action/result evidence; runtime identity; observed profile facts; exact Qt/core cleanup; retained proof. Scope: background CUA GUI/private proof only. Dependency: Item 1 build PASS. Size: bounded operational.