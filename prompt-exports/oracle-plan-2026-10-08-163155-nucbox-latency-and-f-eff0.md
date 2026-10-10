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



# NucBox next step: input-timestamped matched block and zero-input idle tail

## 1. Summary

**Decision: continue with exactly one bounded host step, called Block B.** Block B runs on a fresh session. It contains three descriptive attempts, B1 to B3. Each attempt has warmup 20 s and three contiguous 60-s windows. A single visible Escape pause/resume immediately precedes each attempt, with host-clock timestamps and screenshot proof, all outside scoring.

If the session is still streaming after the attempts, one **zero-input idle tail (T)** follows. It runs on the same existing collector and observers.

**Answer to the question.**
- **Yes, conditionally, for a matched baseline.** B can become a trustworthy *within-B* matched descriptive baseline because it removes a structural input-recency confound in the A schedule.
- **No for A2's cause.** B is a different session. A2 stays INVALID, the A-block repeatability stays INCOMPLETE, and A1 is never pooled with B.
- **The discriminator is built into the same session.**
  - Any freeze inside B's input budget falsifies "inactivity alone" for that incident.
  - Tail T tests whether A2's signature reappears at the officially documented idle offset.

**Scope limits.**
- No new host probe, product edit, build, test, or anti-idle behavior is introduced.
- The RTSPS control-stop diagnostic is pre-specified only as the contingent next iteration. It cannot be deployed without publishing approval.

## 2. Current-state analysis

Labels used below: **[obs]** = direct observation; **[inf]** = inference from the cited observations.

### 2.1 What is already established (not re-adjudicated)
- **A1 completed three windows.** [obs] Aggregate fresh-interval upper bounds were p95/p99/max 21/24/56 ms, with 88 of 10,915 intervals late (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:62-68`, `:79`).
- **A2 completed one window, then froze.** [obs] The completed window gave 20/24/300 ms. A ~16-s open stall followed and that histogram does not bound it (`…runs.md:75-76`, `:79`, `:124`).
- **D1 verdict STAGE_ONLY.** [obs] Raw receive/auth/assembled counters stayed flat across timeouts while same-worker pings rose. Raw counters were still rising after 12:36:02.550; the first flat snapshot was at 12:36:12.555. The second timeout's idle time measures from the recovery origin. No independent discriminator exists (`…runs.md:119-121`).
- **A2 terminated through the local timeout/recovery path.** [obs] There was one recovery attempt and then `nvst-recovery-exhausted` with `Timeout`, which is a core-local decision (`…runs.md:76-77`, `:120`).
  - Source: `native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2228-2267` (one attempt, then exhausted) and `:2269-2297` (termination `source: nvst-transport`, `resumable: null`).
- **The timeout clock is set by authenticated packets only.** [obs] It runs from the last authenticated SRTP packet, or from the origin after `recover()` (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4233-4243`, `:4254-4277`). Only fully authenticated, payload- and SSRC-matched packets advance it (`nvst.rs:4366-4403`).
- **Pings and inbound counting are independent.** [obs] The receive loop sends pings regardless of inbound traffic (`nvst.rs:7433-7483`). It counts every datagram, including STUN, in `inbound` before authentication (`nvst.rs:7485-7543`).
- **Logging is coarse.** [obs] Counters are logged roughly every 10 s (`nvst.rs:7596-7611`). A non-idle socket error would stop the loop with a `video-receive` error rather than through the timeout path (`nvst.rs:7569-7582`).

### 2.2 New discriminant 1: input recency confounds the A schedule
- **Gameplay input timing.** [obs] The only gameplay input proof (Escape pause/resume) was taken during setup, before scoring (`…runs.md:58`). It predates the A1 collector start at 12:27:30, and its exact time is absent (`…runs.md:62`, `:122`).
- **No input after that.** [obs] A2's collector started at 12:34:02 with no intervening gameplay input; only screenshots and Cua session revival occurred (`…runs.md:68`, `:71`, `:74`).
- **Idle at onset exceeded 8 min.** [inf from `:62`, `:119`, `:122`] Idle time at A2 raw-counter cessation was greater than 12:36:02.550 − 12:27:30 = **8 min 32 s**. The upper bound is unknown; it is at most about 19 min if the last input was near the ~12:17 worker-start proxy.
- **The A design guaranteed this.** [inf from the same citations] The second and later attempts were structurally scheduled past an 8-min idle point. Any repeat of that schedule would re-introduce the confound into the repeatability gate itself.
- **Policy source.** [user-supplied, not in the frozen evidence] NVIDIA a_id 4478 and 3442 state an 8-minute inactivity disconnect. They do not define whether transport pings count as activity. This supports a workload confound, not an A2 cause.
- **Session-age covariate.** [inf from `…runs.md:122`, `:119`] A2's onset was about 19 min after the local stream-start proxy. That proxy is not server allocation proof. B must record session age so a fixed session-age cutoff can be told apart from idle.

### 2.3 New discriminant 2: control-channel stops are silent
- **The worker sends WebSocket pings, not GET_PARAMETER.** [obs] The RTSPS worker sends WebSocket `Ping` (`native/opennow-streamer/crates/opennow-streamer-core/src/nvst_rtsp.rs:770-782`).
- **Every exit is silent.** [obs] Each exit path leaves without any log or core event: peer `Close` (`:801`), any non-timeout read error (`:808`), buffer limit (`:810-812`), response parse error (`:813-823`), and ping/pong send failures (`:773-779`, `:788-791`). Parsed responses are discarded (`:815`).
- **The worker has no outbound channel.** [obs] It holds only `control_ping` and a shutdown receiver (`:744-832`).
- **A loopback test pins this behavior.** [obs] It checks WebSocket pings with no RTSP requests, and that a peer close ends the worker (`native/opennow-streamer/crates/opennow-streamer-core/src/nvst_rtsp_control_ping_tests.rs:205-224`; the user cited 198–224).
- **README mismatch.** [obs] The README claims GET_PARAMETER keepalives replaced client WebSocket pings (`native/opennow-streamer/README.md:221-226`). This is a documentation mismatch only and does not authorize any keepalive change.
- **Consequence.** [inf from `nvst_rtsp.rs:753-826`] A server-initiated control close, or a path-loss read error, is invisible in every existing retained channel. No existing evidence can establish "control closed before media silence".

### 2.4 Event retention gap
- [obs] The core emits `input-unavailable` with a `reason` field (`lib.rs:2194-2197`).
- [obs] The log summary allowlist has no `reason` key, except the keyframe-request mapping (`native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs:172-232`).
- [inf, matches `…runs.md:121`] Any summarized `input-unavailable` record loses its reason.

### 2.5 Tooling constraints
- **Collector.** [obs] It runs Doctor and records identity/hash/flags, measures N contiguous windows, preserves partial evidence, and never sends input (`.agents/skills/verify-gfn-experiments/features/latency-baseline.md:22-37`).
- **Observers.** [obs] 240 s is the documented duration for three windows. Observers are descriptive metadata, not packet age (`latency-baseline.md:39-50`).
- **Metric limits.** [obs] Bucket upper bounds only; rolling decode medians; not input-to-photon (`latency-baseline.md:61-74`).
- **Doctor.** [obs] `--display-hz 100` is unsupported, exit 2 (`…runs.md:59`). The 120-Hz option is the only alternative to the 60-Hz default (`SKILL.md:55`). So `--require-stream` is not usable at 100 Hz.
- **Operating rules.** [obs] One operator owns GUI/account (`SKILL.md:12`). Exit must be verified before relaunch, with archiving in between (`SKILL.md:16-20`). Launch commands are at `SKILL.md:22-28`. A single small input check belongs before warmup, and invalid results are not retried (`SKILL.md:67`). Never kill by name (`SKILL.md:88`).
- **Ownership.** [obs] GUI/account ownership was released to the coordinator at 12:48:31. A sleeping process does not prove readiness (`…runs.md:113-114`).
- **Deployment boundary.** [obs] Product changes need local review/tests plus publishing approval (`…runs.md:97`).
- **Consequence.** [inf from `:97`] No product seam can influence a live run in this step.

## 3. Design

### 3.1 Decision rationale
- **Why Block B.**
  - It is the only authorized move that yields new attributable evidence with existing tools.
  - Every outcome is informative: a clean in-budget block plus a tail match, a tail non-match, or an in-budget freeze each lead to a distinct next step (§3.8).
- **Why not the seam first.** It is undeployable now (§2.5) and would add nothing to the next run.
- **Why not stop.** Stopping is only correct if the ownership or identity gates fail (§3.9).
- **No new host probe is justified.** Each candidate is rejected for a stated reason:

| Rejected probe | Reason |
|---|---|
| `ss`/`tcpdump` loops | New probes, with privacy exposure to endpoints and payloads |
| Router `--flint` | Router syslog is stale (`…runs.md:45`) |
| Doctor `--require-stream --display-hz 100` | Unsupported (`…runs.md:59`) |
| GET_PARAMETER switch | Explicitly not authorized |
| Extra recovery retries | Explicitly not authorized |

### 3.2 Phase 0: read-only discovery (local repo, plus remote file reads and `--help`; no GUI)

These results decide extraction scope. Only D0-d can remove the tail; no D0 result blocks B.

| ID | Question | Supported command shape | Effect on plan |
|---|---|---|---|
| D0-a | Is native telemetry `pingMs` sourced from `NvstControlPing::ping_ms` (WebSocket pong, expiry), from STUN `publish_ping` (`nvst.rs:7513-7522`), or both, and with what precedence? | `rg -n "ping_ms\(\|publish_ping\|pingMs" native/opennow-streamer/crates` | If control-pong sourced, record null/non-null `pingMs` in the first native telemetry sample after any onset as supporting-only control-liveness evidence. Otherwise ignore it. |
| D0-b | Do opennow-core or Qt emit and *log* session-status, inactivity-warning or termination-reason events? Is the `input-unavailable` reason logged unsummarized anywhere? | `rg -n -i "inactiv\|idle\|termination\|input-unavailable\|session.?status" native/opennow-core/src opennow-qt/src` | Defines the "existing independent channel" allowlist for extraction. If none exists, record "channel not retained by existing app". |
| D0-c | Does `tools/live-socket-observer.py` record TCP sockets and states for the PID (the RTSPS WebSocket)? | Remote: `sed -n '1,400p' tools/live-socket-observer.py` (read only) | If yes, the TCP state or disappearance time becomes a ~1 Hz control-close bracket. If no, record the gap; no new probe. |
| D0-d | Collector limits: `--windows 5` accepted, minimum `--warmup`, label charset, exit-code meanings (A2 used 2 for `video_stalled`), aggregate quantile rule | `python3 tools/latency-baseline.py --help`; read its summary/quantile code | If `--windows 5` is unsupported, the tail uses the largest supported N ≥ 4, or tail = NOT_RUN. |
| D0-e | The four archived log paths and their rotation naming | Read `tools/archive-trial-logs.py` | Source list for the private raw-copy step (§3.7). |

### 3.3 Phase 1: ownership, identity and relaunch gates (host)

Run in this order; any failure means STOP (§3.9).

1. **G1 ownership.** Obtain the coordinator's explicit GUI/account handoff to the B operator and record its UTC time. Without it, STOP and leave the scoreboard entry incomplete.
2. **G2 proof directory and Doctor:**
   ```sh
   ssh chimeraos; cd /home/gamer/dev/gfn-client-research; umask 077
   B_DIR=$(mktemp -d /home/gamer/dev/gfn-client-research/.runtime/nucbox-blockB-XXXXXXXX)
   python3 .cursor/skills/verify-gfn-experiments/scripts/doctor.py --capabilities > "$B_DIR/doctor-before.json"; echo $? > "$B_DIR/doctor-before.exit"
   cua-driver start_session '{"session":"gfn-verify"}'; cua-driver list_windows '{}'
   ```
   - Requires `ready_for_gui`.
   - Hashes must equal S3: Qt `3b72d841…`, core `87baad19…`, FFI `2349750a…` (`…runs.md:54`).
3. **G3 normal close of the existing client.** Take a fresh `get_window_state`, then use the normal Quit plus confirmation. Prefer the AT-SPI Quit path that worked in S2 (`…runs.md:50`).
   - Verify that Qt 1723569/start 17257151 and core 1723776/start 17257246 have exited: Doctor `live[]` empty plus a `/proc` absence check.
   - If one fresh snapshot and closing the remaining window per `SKILL.md:18` still leaves them alive, STOP. Send no signals.
   - Rationale for relaunching rather than reusing: the A stream ended automatically and is not resumable (`…runs.md:78`). Relaunching also gives log-rotation headroom (`SKILL.md:20`).
4. **G4 archive:** `python3 tools/archive-trial-logs.py --output "$B_DIR/prelaunch-logs"`.
5. **G5 launch**, exactly once, using the S3 form (`…runs.md:55`):
   ```sh
   cua-driver launch_app '{"launch_path":"env QT_QPA_PLATFORM=wayland /home/gamer/dev/gfn-client-research/tools/run-live-trial.sh","additional_arguments":["baseline"]}'
   ```
   Then run Doctor into `doctor-after-launch.json`. Requirements:
   - `ready_for_gui`.
   - The same S3 hashes.
   - Record the new `QT`/`CORE` PIDs and start ticks.
   - **No build.**

### 3.4 Phase 2: session and workload setup (outside scoring)
- Use the normal GUI Ori **Resume** if offered, otherwise **Play**; record which. Record `t_ready`, the host UTC time when stream stats first show streaming. This is the session-age origin.
- Check stats once (as S4 did) and close the overlay. Expected values:
  - H.265 10-bit 4:2:0, HDR profile, 2560×1080, 60 source FPS.
  - 100 Hz fullscreen, frame generation off.
  - Region/zone recorded only.

  Any mismatch means STOP; no profile changes are allowed.
- Load the saved slot and reach the stationary animated Spirit Well. Navigation input is allowed here.

### 3.5 Phase 3: per-attempt protocol (N = 1..3)

**Label:** `LBL=<UTCDATE>-<lowercased B_DIR suffix>`. Attempts use `$LBL-bN`; preflights use `$LBL-bN-preflight`.

**Input proof.** At most one Escape pause/resume pair per attempt, plus at most one timing redo (§3.9).
1. `date -u +%FT%T.%6NZ > BN/t-pause-before.txt`, then `cua-driver press_key …`, then the after-timestamp.
2. Screenshot showing the pause menu.
3. Before-timestamp, resume `press_key`, after-timestamp.
   - **`t_in` = the before-timestamp of the resume press.** It is a conservative earliest bound, so idle time computed from it is an upper bound.
   - Reuse the exact `press_key` request shape from the private S4 action JSON, changing only pid/window.
4. Screenshot `BN/scene-before.png`. It must show resumed gameplay, the same Spirit Well, and overlays closed.
5. If a press reports a timeout or its effect is not visible within 30 s, do **not** replay. Take one fresh snapshot after ≥10 s. If the state is still unproven, STOP the block. Rationale: delayed effects seen earlier (`…runs.md:58`) could land inside scoring.

**Dry-run, observers, collector.** Collector flags are identical to A1 (`…runs.md:62`). Observer durations follow `latency-baseline.md:39-50`.
```sh
python3 tools/latency-baseline.py --pid "$QT" --width 2560 --height 1080 --display-hz 100 \
  --scene ori-inkwater-marsh-spirit-well --label "$LBL-bN-preflight" --dry-run   # stdout/stderr/exit -> BN/
# previous attempt's observers must have exited before these start
( python3 tools/live-socket-observer.py --pid "$QT" --duration 240 --output "$B_DIR/BN/sockets.json"; echo $? > "$B_DIR/BN/sockets.exit" ) &
( python3 tools/live-network-observer.py --duration 240 --output "$B_DIR/BN/network.json"; echo $? > "$B_DIR/BN/network.exit" ) &
# GATE: now - t_in <= 120 s, else do not start (see 3.9)
python3 tools/latency-baseline.py --pid "$QT" --width 2560 --height 1080 --display-hz 100 \
  --scene ori-inkwater-marsh-spirit-well --label "$LBL-bN" --warmup 20 --seconds 60 --windows 3
```
- Capture start/end UTC and stdout/stderr/exit into `BN/`.
- After the collector exits, take `BN/scene-after.png`.
- The next attempt's pause/resume may begin before this attempt's observers finish. Its observers must not start until they have.

**Timing budget** (A1 measured the collector run as 12:27:30 → scored end 12:30:54.686, `…runs.md:62`):

| Interval | Bound | Type |
|---|---|---|
| `t_in` → collector start | ≤ 120 s | Hard pre-start gate |
| collector start → scored end | ≈ 205 s | Expected |
| `t_in` → scored end | ≤ 360 s | Post-hoc validity (≥ 2 min margin below 8 min) |
| `t_in(N)` → `t_in(N+1)` | Target ≤ 390 s | Record actual; if > 420 s, flag `INTER_ATTEMPT_IDLE_LONG` |

**Prohibitions inside every collector run:** no input, screenshots, builds, tests, or hashing.

### 3.6 Phase 4: idle tail T (only if B1–B3 ran without any `media-timeout`, and the stream is still streaming)
- **Zero input after `t_in(B3)`.**
- **Start gate:** the tail collector must start ≤ 450 s after `t_in(B3)`. Otherwise T = NOT_RUN.
- **Commands:**
  ```sh
  python3 tools/latency-baseline.py … --label "$LBL-t-preflight" --dry-run
  ( python3 tools/live-socket-observer.py --pid "$QT" --duration 360 --output "$B_DIR/T/sockets.json"; … ) &
  ( python3 tools/live-network-observer.py --duration 360 --output "$B_DIR/T/network.json"; … ) &
  python3 tools/latency-baseline.py … --label "$LBL-t" --warmup 20 --seconds 60 --windows 5
  ```
- **Coverage:** about `t_in`+7.0 min to `t_in`+12.9 min, which brackets the documented 8-min mark.
- **Labeling:** T windows are labeled tail/non-comparable and are never pooled with B.
- **No screenshots during T**, so it stays comparable with A2's observation conditions. Take one `T/scene-after.png` after the collector exits.

### 3.7 Evidence retention, sanitization and metrics
- **Raw logs.** After T, or immediately after any incident, `cp -p` the D0-e live logs, plus any rotated generation, into `$B_DIR/raw/`.
  - Remote, mode 0700/0600, read-only use of the source logs.
  - Never edit live logs, and never copy raw logs locally.
- **Sanitized events.** Produce `$B_DIR/sanitized-native-events.json` with the same row shape as C4 (`…runs.md:106`): `{raw_path, line_1based, utc, category, text}`. Window: from `t_ready` to the end of T.
  - **Categories:**
    - `nvst-video` stats counters (inbound, pings, stun_ok, wrong_source, and the authenticated/assembled values in `stats_line`).
    - `media-timeout`; `transport` terminal codes; `keyframe-request reasonCode`; `queue-dropped`.
    - `input-ready`, `input-unavailable`, `status`, `error code`.
    - The D0-b channels.
  - **Redact:** `local_port=`, `peer_port=`, `ssrc`, any `source=`/`peer=` and any IP/port, and all free-form server text.
- **Local private export** to `/private/tmp/nucbox-blockB-<suffix>/`, all JSON only (no PNGs, raw logs, or credentials):
  - Each collector's `summary.json`, `observation.json` and numeric samples.
  - Observer JSON.
  - Doctor JSON.
  - `sanitized-native-events.json`.
  - Action timestamps.
- **Per-window metrics**, from collector summaries, in the same columns as the A1 table (`…runs.md:63-67`):
  - Fresh-interval p95/p99/max as 1-ms bucket **upper bounds**.
  - >25 ms count/rate.
  - Mailbox and video-queue drops.
  - Decode **median of sampled rolling p95** and **greatest rolling max** (`…runs.md:105`).
  - Qt submit p95/max upper bounds.
- **Block aggregate.** Sum bucket counts over the nine windows and apply the collector's own quantile rule (D0-d). Report per-attempt ranges only: no means, no variance estimate, no stage sums, no input-to-photon wording.
- **Short-stall incidents.** Count completed fresh intervals ≥ 250 ms, each with any keyframe-request `reasonCode` within ±2 s. Keep this separate from freezes, mirroring the separation in `…runs.md:110`.

### 3.8 Validity and cause classification

**Attempt validity (COMPLETE_VALID requires all of these):**
- Collector exit 0, complete 3 windows, `reasons[]` empty.
- Pre-start gate and post-hoc `t_in` gates met.
- Before/after scenes match.
- Observer intervals contain the scored interval, and both observers exit 0.
- Same QT/CORE identity and S3 hashes.
- Allowlisted flags limited to Wayland and LIVE_TELEMETRY.
- No Cua action timestamp inside the scored interval.

Failures are retained as `INVALID_<reason>` and never excluded.

**Session validity:**
- One allocation across B1–B3: no Play/Resume, no `status stopped` or `streaming` re-entry, and stable Qt generation.
- Profile readback unchanged.

**Block status:** `MATCHED_BASELINE` only if B1–B3 are all COMPLETE_VALID; otherwise `INCOMPLETE`.

**Onset of a freeze:**
- Primary: the Qt last-fresh-source time from collector age rows (±1 s).
- Cross-check: the native raw-cessation bracket, from the last increasing to the first flat 10-s stats line. Native times are async write times (`…runs.md:119`).
- Express onset as seconds after `t_in`, and as session age after `t_ready`.

**A2-like signature (all four must hold):**
1. Qt fresh source stops while acquire-empty and swaps continue.
2. Native inbound and auth counters are flat across ≥ 2 consecutive stats lines while pings rise.
3. `media-timeout` is followed by recovery or by `nvst-recovery-exhausted Timeout`.
4. No `video-receive` error or `nvst-transport-stopped` precedes the timeout.

**Outcome → A2 implication → next iteration:**

| Outcome | Condition | A2 implication | Next iteration |
|---|---|---|---|
| `IN_BUDGET_FREEZE` | Any `media-timeout` or terminal freeze in B1–B3 with onset ≤ 360 s after `t_in` | Inactivity alone is insufficient for that incident; A2 cause stays open | Contingent control-stop seam (§3.11) |
| `TAIL_IDLE_CONSISTENT` | A2-like signature in T, native bracket within [450, 690] s after `t_in(B3)` | A2 reclassified as *idle-policy-consistent workload confound (inferred, unproven)*; stays INVALID | Input-recency protocol becomes mandatory for all later attempts. Next root-cause iteration targets the recovered short-stall class (compressed discontinuity-or-overflow) using B's data. |
| `TAIL_EXPLICIT` | T ends with a non-transport termination or an existing app idle/session event (D0-b) | The idle path is observable, but A2 lacked it, so A2 stays unresolved | Seam |
| `TAIL_EARLY` / `TAIL_LATE` | Onset < 450 s, or > 690 s but before tail end | Not explained by the documented threshold | Seam |
| `TAIL_NOT_REPRODUCED` | No termination by the end of T | Documented 8-min disconnect not observed within ~13 min idle; A2 unresolved | Seam |
| `TAIL_NOT_RUN` | Tail gate failed or no live stream | No change | Tail-only rerun in a fresh session before the seam |

**Supporting-only evidence (never decisive alone):**
- D0-a `pingMs` null transitions.
- D0-c TCP state changes.
- An `input-unavailable` event inside the onset bracket.
- Session age relative to A2's ~19 min.

### 3.9 Stop and continue rules
- **STOP before scoring** on any of:
  - G1 to G5 failure.
  - Hash, flag or profile mismatch.
  - Normal close failure.
  - Unproven press effect.
  - The pre-start timing gate missed twice. One redo is allowed: a new pause/resume plus a new preflight label, with the missed one retained.
- **After an incident:**
  - After any `media-timeout` or freeze: no further collectors, no tail, no restart, no manual recovery. Record the incident.
  - After a non-freeze invalid attempt: no further scored attempts. Proceed to T from the last `t_in` only if the stream is still streaming and the tail gate holds.
- **At the end:**
  - Leave the client alive and record its identity.
  - Wait only for this run's observers.
  - Release GUI/account ownership to the coordinator with a timestamp.
  - Write no product, skill, README, AGENTS.md, vendor or tools files.
  - Make no network/display/profile changes. Do not commit, push or publish.

### 3.10 Scoreboard entry (append once, at the end or at the STOP point; unfilled fields stay `PENDING`/`NOT RUN`)

```md
### B — input-timestamped matched block + idle tail — <UTC start>–<UTC end>
- Decision basis: A-schedule input-recency confound (A2 raw cessation >8m32s after last documented input, inferred); external NVIDIA 8-min statement user-supplied, transport-ping activity undefined. Not an A2 repair; A2 INVALID retained; A repeatability INCOMPLETE; A1 not pooled.
- Ownership/identity: handoff <UTC>; Doctor ready_for_gui; hashes == S3 <yes/no>; Qt <pid/start>, core <pid/start>; flags allowlist <list>; session Resume|Play, t_ready <UTC>; profile readback <match/mismatch>.
- D0: pingMs provenance <…>; app session/idle channel <present+retained|absent>; socket observer TCP <yes/no>; collector windows=5 <yes/no>.
| Attempt | Status | t_in→start s | t_in→scored end s | Session age at start min | W1/W2/W3 fresh p95/p99/max upper ms | >25ms count/rate | mailbox/videoqueue drops | decode median-rolling-p95 / greatest-max ms | Qt submit p95/max upper | ≥250ms stalls (reasonCode) | observers exit/overlap |
- Block aggregate (summed buckets): N, late count/rate, p95/p99/max upper; per-attempt ranges only. Status MATCHED_BASELINE | INCOMPLETE.
- Incident/tail: <IN_BUDGET_FREEZE|TAIL_*>; onset s after t_in, session age; signature 1–4 <✓/✗>; supporting events (sanitized types/times only).
- Limits: Qt callback/component values only; not physical scanout or input-to-photon; no cross-block improvement claim; no freeze-free/endurance claim; ~1 Hz observer sampling; native times are write times.
- Actions: inputs = <count> pause/resume pairs outside scoring; no anti-idle loop; no build/tests/product/vendor/network/display/profile edits; no commit/push. README:221–226 vs nvst_rtsp.rs:770–782 mismatch recorded, unchanged.
- Next: <per 3.8 table>. Evidence: remote $B_DIR, local /private/tmp/nucbox-blockB-<suffix>/ (sanitized only).
- Verification: preceding ledger bytes <n>/SHA256 <h> unchanged; append-only.
```

**Ledger write mechanics (local):**
1. Record `wc -c` and `shasum -a 256` of the ledger.
2. Append with `cat >>`.
3. Verify that `head -c <n> | shasum -a 256` still equals the prior hash.

### 3.11 Contingent next iteration (specification only; not executed; requires local review, tests and publishing approval)

Use this only if §3.8 routes to "Seam". The target is the smallest default-off control-stop diagnostic in `nvst_rtsp.rs` `ActiveNvstRtspSession::spawn` (`:744-832`).

- **Types.** A private enum `ControlStopClass { LocalShutdown, PeerClose, PingSendFailed, PongSendFailed, ReadFailed, BufferLimit, ResponseInvalid }`, and:
  ```rust
  struct ControlStop {
      class: ControlStopClass,
      close_code: Option<u16>,
      error_class: &'static str,          // fixed mapping of tungstenite::Error / io::ErrorKind
      since_spawn_ms: u64,
      since_last_inbound_ms: Option<u64>,
      since_last_pong_ms: Option<u64>,
      ping_outstanding: bool,
  }
  ```
- **Worker changes.**
  - The worker returns `ControlStop`, and `worker` becomes `Option<JoinHandle<ControlStop>>`. `shutdown` discards the value.
  - Each existing break/return site assigns its class. Control flow, `control_ping.clear()` ordering, the WebSocket keepalive and the absence of RTSP requests stay byte-for-byte unchanged in behavior.
  - Add `last_inbound` (any `Ok` read) and `last_pong` (matched pong) instants.
- **Emission.**
  - When enabled, the worker writes one line at exit: `log_line("INFO","nvst-control", format_control_stop(&stop))`.
  - The line contains fixed tokens and integers only. Never include the close reason text, endpoints, RTSP bodies or session headers.
  - Gate: a pure `control_stop_logging_enabled(Option<&str>) -> bool`, true only for `"1"`. It is evaluated once at spawn from `OPENNOW_NVST_CONTROL_STOP_LOG`.
  - Deployment also needs a research wrapper pass-through and flag-allowlist entry, outside this step.
- **Tests**, extending `nvst_rtsp_control_ping_tests.rs` by joining the handle and asserting the class:
  - Peer close (`:205-224`) → `PeerClose`, `close_code` None.
  - New: Close with code 4000 and reason `"secret-reason"` → the formatted line contains `code=4000` and not the reason.
  - Server drop (`:226-252`) → `ReadFailed` with `error_class` in the fixed set.
  - Overflow (`:315-330`) → `BufferLimit`.
  - Shutdown tests (`:271-313`) → `LocalShutdown`.
  - Invalid body with the client's CSeq (adapting `:174-183`; first verify that it is not `nvst-rtsp-sequence-mismatch`) → `ResponseInvalid`.
  - The gate function returns false for None, `""`, `"0"` and `"true"`.
- **Commands:**
  - `cargo test -p opennow-streamer-core nvst_rtsp`, after verifying the package name in the crate's `Cargo.toml`.
  - The crate's full suite.
  - Clippy, if a CI workflow uses it.
- **Not selected as alternatives:**
  - An allowlist entry for the `input-unavailable` reason: its value domain is unverified and possibly free-form.
  - Any keepalive switch.
  - Any added recovery attempt.

## 4. File-by-file impact (this step)

| Path | Change | Driver | Ordering |
|---|---|---|---|
| `prompt-exports/optimize-nucbox-latency-freezes-runs.md` | Append one `### B` entry (§3.10); no rewrites | Scoreboard requirement | Last, or at the STOP point |
| Remote `/home/gamer/dev/gfn-client-research/.runtime/nucbox-blockB-XXXXXXXX/` (new, 0700) | Doctor JSON, prelaunch-logs archive, per-attempt and tail action/timestamp/screenshot/observer/collector receipts, `raw/`, `sanitized-native-events.json` | §3.3–3.7 | Created at G2 |
| Remote `.runtime/latency-baseline-$LBL-{b1,b2,b3,t}[-preflight]/` | Collector-created outputs | §3.5–3.6 | Per attempt |
| Local `/private/tmp/nucbox-blockB-<suffix>/` (new, private) | Sanitized JSON exports only | §3.7 | After the host phase |
| Product, test, README, skill, AGENTS.md, vendor, `tools/` | **No change** | Scope | — |
| (Contingent, a future step) `nvst_rtsp.rs`, `nvst_rtsp_control_ping_tests.rs` | §3.11 | Only if §3.8 routes to the seam | Separate approval |

## 5. Implementation order

1. **Phase 0:** local `rg` discovery (D0-a, D0-b). Record findings privately; nothing goes in the ledger yet.
2. **G1:** obtain and record the coordinator ownership handoff. If absent, append an entry marked incomplete and STOP.
3. **Remote read-only D0-c, D0-d and D0-e**, then G2 (proof directory, Doctor, Cua session).
4. **G3 close and identity-exit proof**, then G4 archive, then G5 single launch, then Doctor after launch. These must be completed strictly in sequence.
5. **Phase 2:** session start, `t_ready`, single stats/profile check, scene setup.
6. **B1, B2, B3** per §3.5, each gated. Stop rules per §3.9.
7. **Tail T** per §3.6, if eligible.
8. **Raw log copy**, sanitized event extraction, local sanitized export, metric and aggregate computation, classification per §3.8.
9. **Ownership release** to the coordinator, with the client left alive and its identity recorded.
10. **Append the ledger entry** with byte/hash prefix verification.

If the outcome routes to the seam, the §3.11 iteration begins only as a new, separately approved step.

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



## Summary

Choose **B1 — one new, activity-timestamped baseline block, capped at three separately preflighted attempts of three contiguous 60-second windows, with an immediate block stop on a liveness or session failure.** This is warranted because D1 is complete and identifies an unresolved workload variable: the last successful gameplay action was not timed. **Inference:** documenting normal pause/resume immediately before each short attempt can establish a trustworthy **new, bounded descriptive baseline**, provided identity, scene, timing, telemetry, and evidence-retention gates pass. It cannot establish A2’s cause or long-session reliability. D1 remains `STAGE_ONLY`; A2 remains `INVALID` and retained; the original repeatability block remains incomplete. No instrumentation, keepalive change, additional recovery attempt, build, or deployment belongs to B1. The newly identified RTSP diagnostic gap is recorded as a coverage limit and becomes relevant to a later proposal only if new evidence warrants it. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:116-126`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:151-171`)

## Current-state analysis

**Evidence convention:** “Observation” below means directly stated in the supplied source or ledger, including previously recorded measurements. I have not independently inspected the private artifacts. “Inference” identifies conclusions drawn from those observations. The NVIDIA inactivity statement is **user-supplied scout evidence outside the frozen file excerpts**; I cannot independently verify it or assign it a fabricated repository citation.

### What is established, and what remains open

**Observation — completed D1:** raw receive and authenticated-packet counters remained at `1,888,440`, and local assembled-frame count remained at `68,657`, while the same worker’s ping counter advanced. The native timestamps are diagnostic-write times with unknown delay. Qt’s last demonstrated fresh-source increment has a separate observation bracket. D1 found no independent session-cause discriminator. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:118-123`)

**Observation — important correction:** repeated decode/shared-stage plateau through teardown was **not** demonstrated. The last comparable native telemetry snapshot was at `12:36:13.537`, with submissions and outputs both `68,642` and `inFlight=0`. Its stale timing fields do not demonstrate continuing decoder activity. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:151-167`)

**Observation — retained component measurements:**

| Record | Fresh-interval histogram upper estimates: p95 / p99 / max | Intervals strictly over 25 ms | Status |
|---|---:|---:|---|
| A1, three completed windows pooled by histogram counts | 21 / 24 / 56 ms | 88 / 10,915, approximately 0.806230% | Completed descriptive evidence |
| A2, completed first window | 20 / 24 / 300 ms | 35 / 3,583, approximately 0.976835% | Completed window within an invalid attempt |
| A2, subsequent terminal incident | Earlier completed-gap histogram cannot bound the open stall | Approximately 16 seconds without a fresh source frame before teardown | Freeze evidence retained |

These figures are recorded in the ledger; the approximately 299-ms recovered incident and 14 compressed-video drops remain separate from the terminal freeze. Its initiating cause was not proven to be overflow alone. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:73-79`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:108-110`)

**Observation — measurement contract:** fresh-interval, submit-to-swap, and notification-drain histograms provide bucket upper estimates. Decode summaries are overlapping rolling distributions; the reported decode “p95” is the median of sampled rolling-p95 summaries. These measurements do not establish physical presentation or input-to-photon latency. (`.agents/skills/verify-gfn-experiments/features/latency-baseline.md:61-74`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:104-105`)

### Relevant runtime ownership and event flow

**Observation — receive path:** the video UDP worker owns its socket-receive loop, counters, and `NvstVideoReceiver`. Successful `recv_from` increments `inbound_datagrams` before source/STUN/media processing. Accepted-packet progress occurs later, after SRTP unprotection and payload-type/SSRC checks. Assembly and downstream media delivery are subsequent boundaries. A successful local ping send does not establish a remote response. (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4360-4420`, `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7380-7640`)

**Observation — timeout and recovery:** `NvstVideoReceiver::poll_timeout` uses `last_authenticated_packet`, falling back to `timeout_origin`. It transitions to `RecoveryRequired` once. The receive worker applies a queued recovery command; `recover()` clears the accepted-packet timestamp and establishes a new origin. Core event forwarding uses lifecycle generations and permits the bounded recovery attempt before terminal stop. `PacketGap` and `MediaConsumerBackpressured` keyframe handling do not consume that terminal-recovery budget. (`native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:4230-4290`, `native/opennow-streamer/crates/opennow-streamer-transport/src/nvst.rs:7380-7640`, `native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2060-2300`)

**Inference:** the second timeout measures the recovery interval, and a post-recovery `Running` event demonstrates a control-state transition. Neither establishes restored media progress. B1 must retain these distinctions when interpreting any new failure. The supporting observations are the reset and event-forwarding behavior above.

### The newly identified control-path seam

**Observation — source:** `ActiveNvstRtspSession` owns a control sender and worker thread. Its active loop sends WebSocket `Ping`, matches `Pong`, handles server `Ping`, and consumes buffered RTSP responses without using their result. Peer `Close`, several I/O failures, and parsing/buffer failures end the worker without an emitted stop-class diagnostic in the supplied implementation. Shutdown and worker exit clear `NvstControlPing`. (`native/opennow-streamer/crates/opennow-streamer-core/src/nvst_rtsp.rs:730-845`)

**Observation — source/documentation mismatch:** the existing loopback test explicitly exercises WebSocket ping behavior without RTSP requests, while the README describes session-scoped `GET_PARAMETER` keepalives replacing client WebSocket pings. (`native/opennow-streamer/crates/opennow-streamer-core/src/nvst_rtsp_control_ping_tests.rs:198-224`, `native/opennow-streamer/README.md:211-241`)

**Inference:** this establishes a mismatch between the supplied implementation and documentation, plus a real loss of diagnostic information. It does **not** establish which protocol behavior caused A2. Missing control-ping samples also cannot uniquely identify peer closure because the implementation clears samples in multiple circumstances. These observations justify preserving the seam as an unresolved boundary, without changing keepalive behavior in B1.

**Observation — existing summary limit:** `message_summary` retains selected discriminators and numeric fields. It does not generally retain `reason`; D1’s pointed `input-unavailable` records lacked the independent explanation needed for attribution. (`native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs:169-235`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:121-123`)

### Why the next action is a workflow change

**Inference:** a timed, visibly successful gameplay preflight addresses one documented measurement uncertainty without changing the suspected control or receive paths. Existing collection already supports the requested three-window measurement and bounded metadata observers. This makes B1 a targeted experiment-protocol change; a production refactor or diagnostic edit is not yet indispensable. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:122-126`, `.agents/skills/verify-gfn-experiments/features/latency-baseline.md:22-50`)

## Design

### 1. B1 contract and validity boundaries

B1 contains **at most three attempts**, named `B1-T1`, `B1-T2`, and `B1-T3`. Each requests:

- A normal, visibly successful game pause/resume before measurement.
- `--warmup 20 --seconds 60 --windows 3`.
- The same verified allocation, binaries, flags, profile, fixed scene, and presentation state throughout B1.
- Independent, private retention of existing application/session events and the existing numeric observers.
- No gameplay input or screenshots during warmup or scored windows.

B1 is a new measurement block even if the application can reuse an allocation. Do not rename it A3 or pool its histograms with A1/A2.

**Inference supporting the separation:** the new protocol adds a controlled gameplay-action boundary that A1/A2 did not measure. A newly allocated session would introduce an additional matching boundary. Historical A2 cannot become valid through successful later attempts. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:74-83`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:118-124`)

Keep three independent judgments:

| Judgment | Allowed states | Meaning |
|---|---|---|
| Attempt validity | `NOT_STARTED`, `VALID_DESCRIPTIVE`, `INVALID_RETAINED` | Whether that attempt satisfies B1’s measurement gates |
| B1 matching | `COMPLETE_SHORT_BASELINE`, `INCOMPLETE` | Whether all three attempts completed under the same verified block conditions |
| Incident attribution | `NO_INCIDENT_OBSERVED`, `UNRESOLVED`, `SUPPORTED_MECHANISM` | What the retained evidence establishes about an incident |

A completed collector can still fail B1 validity if later inspection establishes an in-window session warning, state change, or identity discontinuity. Preserve its original output and record the additional validity judgment separately.

**A successful B1 establishes only the bounded baseline defined above.** It does not establish endurance, absence of intermittent freezes, or an improvement over A1/A2.

### 2. Ownership, readiness, and evidence-retention gates

The coordinator remains the sole GUI/account owner. Current foreground Cua authorization applies; no additional permission is needed for the explicitly authorized normal playback/preflight flow.

Before the first attempt:

1. Read the current host workflow instructions required by the supplied skill, without modifying them.
2. Run Doctor capabilities and inspect current windows.
3. Record fresh Qt PID/start ticks, core-child identity, current artifact hashes, and the existing allowlisted experiment-flag receipt.
4. Match the current binaries to the retained build receipt. Do not rerun the one-shot build guard.
5. Establish saved-account readiness through the GUI.
6. Reach Ori through normal Play/Resume, then establish the private allocation identity used to test continuity across B1.

**Observation supporting these gates:** the workflow requires one GUI/account owner, fresh GUI readiness, exact process identities, and separate build/flag evidence. D1’s historical identities and sleeping process state do not establish present readiness. (`.agents/skills/verify-gfn-experiments/SKILL.md:8-31`, `.agents/skills/verify-gfn-experiments/SKILL.md:48-61`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:112-114`)

Reuse a suitable client. If the authorized measured launch requires replacing it, complete the normal GUI close flow, verify both recorded processes have exited, archive logs using the existing helper, then launch the existing baseline wrapper. Do not launch a second core/account owner.

Require the current workload to match the recorded target: H.265 10-bit 4:2:0/HDR configuration, 2560×1080, 60 source FPS, Wayland fullscreen, 100-Hz presentation, frame generation off, fixed animated Spirit Well scene, overlays closed. A mismatch blocks B1; do not change network, display, or profile settings to repair it. The newer recorded profile controls over the skill’s older default profile. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:5-14`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:57-59`)

#### Required discovery gates

The supplied context does not expose every current command schema or logging destination. Resolve these through narrowly scoped, read-only inspection before starting:

| Unknown | Required resolution | Failure behavior |
|---|---|---|
| Foreground Cua action/screenshot schema | Read the installed Cua guidance/schema; identify supported window targeting, key delivery, result timestamps, and private screenshot output | Stop if delivery or verification cannot be established |
| Existing application session/warning events | Locate existing producers/consumers and their retained log destinations in the matching application source and launch configuration | Record known omissions; stop if available relevant evidence cannot be preserved safely |
| Current allocation correlation | Identify an existing private GUI/application record that establishes allocation continuity | Stop if continuity cannot be established |
| Allowlisted experiment-flag readback | Reuse the established restricted method for the current PID/start instance | Stop; never substitute a full environment dump |
| Collector deadline/cancellation | Verify that the existing execution channel can cancel the original collector invocation without terminating Qt/core | Stop if bounded execution cannot be assured |

For the event gate, distinguish **known non-emission** from **unknown coverage**. The RTSP close reason is known to be unavailable from the shown worker; this alone does not block a descriptive trial. B1 must nevertheless preserve relevant events the application already emits and document exactly which channels are available.

Do not invent a `--session-events` collector, an authenticated status probe, or a logging flag. Source bodies for the application’s GFN status/push consumers are not supplied; their exact logging paths and fields must be discovered before claiming coverage.

### 3. Activity proof and bounded timing

The gameplay preflight is a **single ordinary pause/resume sequence per attempted measurement**, performed by the owning operator.

For each attempt:

1. Complete Doctor, identity/flag checks, the unique dry-run, and other potentially expensive preparation.
2. Start the two 300-second observers.
3. Take a fresh game snapshot.
4. Use foreground Cua to open the game’s pause menu with Escape; verify the game responded.
5. Resume with Escape; verify the same fixed animated scene and unchanged position.
6. Close overlays and confirm readiness before starting the collector.

**Observation:** that game pause/resume path was successfully used during the prior setup. Cua also exhibited delayed effects after reported timeouts. Therefore, a timeout requires a fresh state check rather than automatic replay. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:45-58`)

Retain these fields in the attempt proof:

| Field | Contract |
|---|---|
| Action identity | Game pause and resume, targeted Qt/window identity, Cua result |
| Dispatch/effect evidence | Exact returned delivery timestamp if available; otherwise a bounded interval from dispatch through visible confirmation |
| Clock basis | UTC plus a same-host monotonic reading outside scoring |
| Scene evidence | Private pause/resume screenshots and unchanged scene/profile confirmation |
| Measurement age bound | Conservative upper bound from the clock reading before resume dispatch to the post-collector clock reading |

Do not label a Cua submission time as server receipt time. If an exact delivery timestamp is unavailable, preserve the interval explicitly. An ambiguous or unbounded action result blocks the attempt.

Use these fixed timing limits:

- Verify pause/resume within **30 seconds of the earlier observer start**.
- Invoke the collector within **30 seconds of verified resume**, and no later than **60 seconds after that observer start**.
- Finish or cancel the collector by **280 seconds after that observer start**.
- Let both observers finish their original **300-second** bounds.

These are **proposed B1 bounds**, chosen to accommodate the 20-second warmup and 180 requested scored seconds while leaving an observation tail. They also keep each measurement well below the user-supplied eight-minute hypothesis without assuming how NVIDIA defines activity.

If preparation exceeds a limit, stop the block. Do not send another input to refresh the attempt. If the collector reaches its deadline, cancel only its original invocation through the verified execution channel and retain all partial output.

There is no periodic input, warning dismissal, inactivity timer, or automatic “keep active” behavior. No input is sent during measurement or simply to preserve the allocation between attempts.

### 4. Supported commands

These commands are for future execution; none has been run here.

Use the documented host working directory and private proof creation pattern, with a B1-specific directory:

```sh
cd /home/gamer/dev/gfn-client-research
umask 077
mkdir -p /home/gamer/dev/gfn-client-research/.runtime
B1_PROOF_DIR=$(mktemp -d /home/gamer/dev/gfn-client-research/.runtime/verification-b1-XXXXXXXX)

python3 .cursor/skills/verify-gfn-experiments/scripts/doctor.py --capabilities \
  > "$B1_PROOF_DIR/doctor-before.json"
```

**Observation — supported interfaces:** the skill documents this Doctor entry point and private proof pattern. Doctor capabilities is appropriate for readiness discovery; the prior `--display-hz 100` Doctor invocation was unsupported. Use the descriptive collector’s documented 100-Hz validation instead. (`.agents/skills/verify-gfn-experiments/SKILL.md:48-61`, `.agents/skills/verify-gfn-experiments/SKILL.md:69-78`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:59-59`)

For each attempt, bind:

- `B1_QT_PID` to the freshly verified instance.
- `B1_PREFLIGHT_LABEL` and `B1_TRIAL_LABEL` to distinct, unused safe labels containing the actual date, B1 block suffix, and attempt number.
- `B1_ATTEMPT_DIR` to a new private `T1`, `T2`, or `T3` directory beneath `B1_PROOF_DIR`.

Run the preflight **before** the gameplay-action proof:

```sh
python3 tools/latency-baseline.py --pid "$B1_QT_PID" \
  --width 2560 --height 1080 --display-hz 100 \
  --scene ori-inkwater-marsh-spirit-well \
  --label "$B1_PREFLIGHT_LABEL" --dry-run
```

Start these as two separate bounded observation jobs:

```sh
python3 tools/live-socket-observer.py --pid "$B1_QT_PID" \
  --duration 300 --output "$B1_ATTEMPT_DIR/sockets.json"

python3 tools/live-network-observer.py \
  --duration 300 --output "$B1_ATTEMPT_DIR/network.json"
```

After successful pause/resume proof, run:

```sh
python3 tools/latency-baseline.py --pid "$B1_QT_PID" \
  --width 2560 --height 1080 --display-hz 100 \
  --scene ori-inkwater-marsh-spirit-well \
  --label "$B1_TRIAL_LABEL" \
  --warmup 20 --seconds 60 --windows 3
```

**Observation — supported arguments:** the feature specifies these collector arguments, and the ledger records successful 300-second socket/network observer invocations. Omit `--flint`. (`.agents/skills/verify-gfn-experiments/features/latency-baseline.md:22-50`, `prompt-exports/optimize-nucbox-latency-freezes-runs.md:89-96`)

A minimal clock read outside scoring can supply the proposed timestamp receipts:

```sh
python3 -c 'import datetime, time; print(datetime.datetime.now(datetime.timezone.utc).isoformat(), time.monotonic_ns())'
```

If a new measured client is necessary, the recorded Wayland launch is:

```sh
cua-driver launch_app '{"launch_path":"env QT_QPA_PLATFORM=wayland /home/gamer/dev/gfn-client-research/tools/run-live-trial.sh","additional_arguments":["baseline"]}'
```

Use it only after the shutdown/ownership gate. The supported archive command is `python3 tools/archive-trial-logs.py --output NEW_PRIVATE_DIRECTORY`, after the workspace Qt/core instances have exited. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:49-58`, `.agents/skills/verify-gfn-experiments/SKILL.md:18-31`)

No new packet capture, remote-path probe, account RPC, or observer implementation is justified for B1.

### 5. Metrics, evidence processing, and causal interpretation

The primary performance metric remains the **fresh-source interval histogram**, reported per completed window:

- Sample count.
- p95/p99/max bucket upper estimates.
- Count and rate strictly over 25 ms.
- Overflow status.
- Any separately observed unclosed no-fresh interval.

Secondary metrics retain their existing meanings: rolling decode summaries, Qt callback histograms, queue/mailbox counts, native progress counters, and sampled socket/network metadata. Do not sum stage quantiles or derive exact visible-frame loss from counters belonging to different owners.

**Observation supporting these rules:** the collector’s documented measurement contract explicitly distinguishes these distributions and boundaries. (`.agents/skills/verify-gfn-experiments/features/latency-baseline.md:54-74`)

The evidence flow is:

**Existing GUI/application events and numeric observations → newly created private attempt proof → per-attempt validity/cause assessment → sanitized ledger append.**

Preserve original timestamps, producer identity, allocation correlation, and source pointers. Treat duplicated forwarded records as one underlying event when provenance establishes duplication. Missing samples remain missing coverage. Counter resets and session/generation changes end comparable intervals.

The new action-time bound is a workload-control measurement. It is not a latency component or proof of the service’s inactivity-clock reset.

Apply these interpretation rules:

| New result | Permitted conclusion |
|---|---|
| Three valid, matched attempts complete | B1 supplies a bounded short-measurement baseline under its documented preflight |
| Freeze occurs soon after visibly successful gameplay input | The new incident occurred within the recorded action-age bound; a simple elapsed-inactivity explanation is weakened only to the extent that the service recognizes that action |
| Same-allocation application event explicitly identifies inactivity termination | Supports an inactivity mechanism for the new incident, subject to event meaning and ordering |
| Existing independent control event identifies peer close, I/O failure, or parsing failure | Narrows the new incident’s control boundary; causal precedence still requires adequate timestamp provenance |
| Only `input-unavailable`, local timeout/exhaustion, or missing ping is retained | Cause remains unresolved |
| All trials succeed | Does not retrospectively identify A2’s cause |

**Inference supporting the last two limitations:** the shown core emits local transport exhaustion itself, while the RTSP worker and protocol summarizer omit information needed to distinguish several upstream causes. (`native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2060-2300`, `native/opennow-streamer/crates/opennow-streamer-core/src/nvst_rtsp.rs:730-845`, `native/opennow-streamer/crates/opennow-streamer-protocol/src/log.rs:169-235`)

Native asynchronous write timestamps cannot establish precise cross-component causal order without additional provenance. Existing socket/gateway observations also cannot establish cloud UDP delivery. Preserve those limitations in every result. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:151-169`)

### 6. Continue and stop decisions

**Continue to the next attempt only when** the current attempt is `VALID_DESCRIPTIVE`, both observers finish, actual coverage is verified, allocation/identity/profile/scene continuity holds, and no new session/liveness failure is found.

Stop B1 on any of the following:

- Failed or ambiguous gameplay-action proof.
- Failed identity, allocation, flag, profile, timing, or evidence-retention gate.
- Collector `video_stalled`, terminal stop, unexpected transport recovery, or post-readiness input-channel loss.
- An application inactivity warning or independent session/control failure.
- Deadline expiration, missing required telemetry, or inability to preserve partial evidence.

A late interval, histogram outlier, or queue-drop count **alone** is not an exclusion. Expected packet-gap/backpressure keyframe handling is not automatically terminal recovery. Preserve completed data and assess the actual event semantics. (`prompt-exports/optimize-nucbox-latency-freezes-runs.md:3-3`, `native/opennow-streamer/crates/opennow-streamer-core/src/lib.rs:2060-2300`)

On failure, do not restart, allocate another session, replay inputs, or run another attempt within B1. Let already-created observers finish their bounds. Preserve the application’s actual state and retain automatic recovery/teardown evidence.

A new incident with a supported mechanism may justify **one later attributable proposal**. An unresolved new incident may justify a separate plan for the smallest default-off control-stop diagnostic. Neither action is executed automatically by B1.

### 7. Writable scope and tests

Agent-directed writes are limited to:

1. New private B1 proof files under the new remote proof directory.
2. The collector’s new, uniquely labelled `.runtime/latency-baseline-LABEL/` outputs.
3. One final append to `prompt-exports/optimize-nucbox-latency-freezes-runs.md`.

Raw application/native logs and screenshots remain private on the host. Preserve existing records in place; any necessary snapshots of already emitted logs stay within the new private proof scope. Never publish raw reasons, endpoints, allocation identifiers, credentials, or payloads.

**No tests are needed or run for B1.** No production behavior, observer implementation, signature, protocol schema, lifecycle rule, or persistence schema changes. The existing loopback tests are interpretation evidence; their presence does not establish cloud protocol correctness.

The RTSP worker, README, protocol logger, receive timeout, recovery limit, vendor trees, helper scripts, skills, and `AGENTS.md` remain outside the writable scope. No commit, push, publication, or deployment is included.

## File-by-file impact

| File or output family | Change | Dependency |
|---|---|---|
| `prompt-exports/optimize-nucbox-latency-freezes-runs.md` | Append one B1 entry containing every attempted, invalid, partial, and unrun status | Final B1 decision and preserved proof |
| `<B1_PROOF_DIR>/proof.md` | New contemporaneous protocol, ownership, command, timing, evidence-coverage, and decision record | Successful exclusive proof-directory creation |
| `<B1_PROOF_DIR>/doctor-before.json` and new identity/flag receipts | Record current readiness and matching facts | Read-only supported discovery |
| `<B1_PROOF_DIR>/T1/`, `T2/`, `T3/` | New action receipts, private screenshots, clock receipts, `sockets.json`, `network.json`, and mapped existing-event evidence | Each attempt’s gates; create only directories actually needed |
| `.runtime/latency-baseline-<B1-label>/summary.json`, `summary.md`, `observation.json`, and existing numeric exports | Existing collector-generated outputs for unique preflight/trial labels | Collector execution; preserve incomplete outputs |

Exact existing numeric-export names and event-log sources must be recorded from the entry point and logging discovery gates; they are not invented here.

No repository source file changes.

### Append-only scoreboard entry

Append after execution, substituting actual results. This is an unexecuted template:

> ### B1 — activity-timestamped short baseline — `<completion UTC>`
>
> - Protocol: maximum three attempts; each warmup20/seconds60/windows3, normal verified game pause/resume before measurement, no input/screenshots during measurement.
> - Ownership/build/profile: `<sanitized current matching result; private receipt references>`.
> - Session continuity: `<MATCHED / CHANGED / UNKNOWN>`; private allocation correlation retained.
> - T1: `<NOT_STARTED / VALID_DESCRIPTIVE / INVALID_RETAINED>`; action-time bounds, scored boundaries, completed/partial coverage, exact collector result, fresh-interval bucket upper estimates, late counts/rates.
> - T2: `<same fields, or NOT_STARTED with reason>`.
> - T3: `<same fields, or NOT_STARTED with reason>`.
> - Observer coverage: `<actual overlap, exits, gaps, resets, teardown coverage>`.
> - Independent event coverage: `<available application/session/control channels and known omissions>`. Current RTSP stop-reason diagnostic unavailable unless existing evidence proves otherwise.
> - Incident attribution: `<NO_INCIDENT_OBSERVED / UNRESOLVED / SUPPORTED_MECHANISM>`; `<sanitized evidence, ordering limitations, remaining alternatives>`.
> - B1 matching: `<COMPLETE_SHORT_BASELINE / INCOMPLETE>`. Completed windows and invalid/partial attempts all retained.
> - Historical status unchanged: D1 STAGE_ONLY; A2 INVALID retained; original repeatability block INCOMPLETE.
> - Interpretation: bounded component measurements only; no latency-gain, endurance, A2-cause, or physical input-to-photon claim.
> - Actions: existing GUI/preflight/collector/metadata observers only; no instrumentation, build/tests, configuration change, keepalive change, extra recovery, vendor edit, commit/push/publication.
> - Decision: `<B1 complete and stop / block stopped with reason>`; `<one later proposal if earned>`. No successor action executed.

## Implementation order

1. **Reserve B1 and resolve discovery gates.** Establish private destinations, current command schemas, evidence-retention coverage, and bounded collector cancellation.
2. **Establish current ownership and client identity.** Reuse the suitable client or perform the authorized normal close/archive/measured-launch sequence. Do not rebuild.
3. **Establish the B1 allocation and workload.** Use normal saved-account playback; verify the fixed scene, profile, and presentation state.
4. **Run T1’s unique dry-run.** Complete identity/flag/hash work before gameplay-action timing.
5. **Start observers, prove pause/resume, then collect once.** Enforce the fixed timing limits and preserve all receipts.
6. **Assess T1 before continuing.** Wait for its observers, inspect the new evidence, and apply the validity/stop rules. Perform T2 and T3 only if each preceding attempt passes; three is a ceiling.
7. **Finalize private proof, then append the scoreboard.** Verify the preceding ledger bytes remain unchanged. Do not modify D1 or historical artifacts.
8. **Stop with the actual application state recorded.** A complete B1 supplies the new short baseline; a failure supplies retained incident evidence. Neither outcome authorizes an automatic product change or another measurement block.

End of Oracle group: 2 lanes above.

## Coordinator reconciliation and executable scope

**Oracle reconciliation:** Both Opus and Astra completed. Material claim inventory, checked against scoreboard, source spot-check, scout evidence and user rules:

| Claim | Lanes | Checked evidence | Disposition |
| --- | --- | --- | --- |
| New activity-timestamped short baseline warranted, preserve A2INVALID/D1STAGE_ONLY | Both | Ledger, missing last-input time, official NVIDIA support research | accepted |
| Pause/resume proves service inactivity timer reset | Opus stronger inference | Docs do not define recognized activity; only visible action proven | rejected |
| Timing bounds, no input/snapshots inside scored interval, three attempts same allocation/profile | Both | Existing collector and skill, supported observer receipts | accepted |
| Force relaunch rather than reuse suitable verified client | Opus | Skill permits reuse; current client can be freshly inspected | rejected |
| Idle tail and cause-classification thresholds automatically included | Opus | Unnecessary extra experiment for next baseline gate; unknown activity semantics | rejected |
| Arbitrary deadline cancellation of healthy advancing collector | Astra | User long-running contract preserves healthy original processes | rejected |
| Current identity/hash/flag/allocation checks, existing event retention, private outputs | Both | User and skill boundaries | accepted |
| Supported3-window collector and300sec observers | Astra; Opus240sec variant | Prior exact receipts and feature map | accepted |
| No builds/new probes/product/network/display/profile/publish changes | Both | Current scope and single operator | accepted |
| Native receive/timeout/reset semantics, stale timing and histogram limits | Both | D1 source validation and ledger | accepted |
| Control worker sends WebSocket ping and loses Close/read/parse stop info; README mismatch | Both | nvst_rtsp.rs730–845 spot-check and focused tests scout | accepted |
| Proposed future diagnostic enum/gate/join-handle/tests/wrapper changes are next chosen iteration | Opus | Conditional unexecuted proposal, not needed for B1 | unresolved |
| Simple cause classification from an input-age threshold or tail match establishes A2cause | Opus | Independent cause/time still missing | rejected |
| Missing known control-stop reason alone blocks descriptive baseline | Neither; preserve distinction | Source documents known non-emission | rejected |
| Stop on actual liveness/session/identity/unproven action failure; retain partial | Both | Measurement reliability contract | accepted |

Execute Astra B1 short baseline only, with explicit fresh sole-operator handoff. Reuse suitable client; new cloud allocation becomes a new matching block. No idle tail, no keepalive change, no new product instrumentation. Resolve read-only event retention/actual command schema gates. Observer duration300seconds. Stage all expensive preparation before proof action. Prefer Astra's short pre-start timing gates; retain delayed-action evidence and never replay blindly. If collector remains healthy beyond proposed timing budget, keep the original alive, mark timing validity incomplete and don't start another attempt; no arbitrary cancellation. Actual failure ends the block with evidence and no restart. At most3attempts; no anti-idle/periodic input loop. B1 completion is a measurement-hardening checkpoint, not performance improvement or A2cause proof. Scoreboard append-only; then return ownership to coordinator for next Oracle decision.