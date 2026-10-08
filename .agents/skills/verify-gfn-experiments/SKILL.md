---
name: verify-gfn-experiments
description: Verify the NucBox OpenNOW Qt client through its real GUI, saved-account playback, repeatable frame measurements, and matched network experiments. Use when validating GFN client changes or maintaining the live experiment loop in gfn-client-research.
---

# Verify GFN experiments

Work on the NucBox (`ssh chimeraos`) in `/home/gamer/dev/gfn-client-research`; this copy is mirrored from the NucBox, so re-sync both ways when it changes. The user-facing client is native Qt/QML with an embedded Rust streamer. Use the normal saved-account GUI for real streams. The CLI observes playback and compares evidence.

Read the current [global runbook](/home/gamer/.codex/AGENTS.md) and [operate-gfn-nucbox](/home/gamer/.agents/skills/operate-gfn-nucbox/SKILL.md) before host work. For latency or networking also read [tune-gfn-vpn-routing](/home/gamer/.agents/skills/tune-gfn-vpn-routing/SKILL.md). For GUI work read [cua-driver](/home/gamer/.agents/skills/cua-driver/SKILL.md) and its [Linux guide](/home/gamer/.agents/skills/cua-driver/LINUX.md). Resolve dated configuration conflicts using the latest runbook observations and current state. Do not restore an old radio, VPN, or SQM template automatically.

One operator owns GUI input, account RPC, and the current cloud session. Isolated XDG directories still use the shared OS credential vault. Close the GUI before using `tools/probe_regions.py` or `tools/set-trial-profile.py`; never run an authenticated standalone core beside it. Read-only measurement and network observers can run beside the GUI. No credential export is needed.

## Launch

First run Doctor and inspect existing windows. Reuse the existing client when the requested check allows it. Record its PID and `/proc` start ticks; distinguish a reused user process from one this run creates.

Before replacing a client, complete its normal GUI close flow and verify that the recorded Qt PID/start-time instance and its recorded core child have exited. A Quit confirmation can return to the library while both processes remain alive. If that happens, take a fresh snapshot and close the remaining application window, then check process identity again. Do not launch another wrapper until the old instances are gone. A second launch can time out during single-instance handoff and leave the old client running without the requested experiment environment. Retain that failed attempt as recovery evidence and verify the newly launched process before continuing.

Between that verified shutdown and the next launch, use `tools/archive-trial-logs.py --output` with a new private directory. The [measurement procedure](features/live-measurement.md) gives the exact invocation. It preserves prior log generations and starts the next run with space below the native log's rotation limit.

For a new measured client, start a CUA session and launch the repository wrapper:

```sh
cd /home/gamer/dev/gfn-client-research
cua-driver start_session '{"session":"gfn-verify"}'
cua-driver launch_app '{"launch_path":"/home/gamer/dev/gfn-client-research/tools/run-live-trial.sh","additional_arguments":["baseline"]}'
cua-driver list_windows '{}'
```

The wrapper enables private Qt telemetry and resets the known experiment variables. Supported variants are `baseline`, `poll1`, `continuous`, `vaapi`, `basic`, and `idle0`. The `idle0` variant sets `QT_QPA_UPDATE_IDLE_TIME=0`. The readiness-wait candidate is preserved on `nucbox-readywake-trial` and is not supported by the active launcher; baseline still clears `OPENNOW_EMBEDDED_READY_WAKE`. It scored worse, and a later Qt XCB crash was observed with causality unproven. Variants are experimental; none of the first five candidates passed the completed acceptance block.

`tools/run-native.sh` preserves caller-supplied experiment variables so the measured wrapper can pass them through. For ordinary playback, launch it with those variables and telemetry explicitly removed:

```sh
env -u OPENNOW_LIVE_TELEMETRY -u OPENNOW_EMBEDDED_PUBLISH_POLL_MS \
  -u OPENNOW_CONTINUOUS_VIDEO_UPDATE -u OPENNOW_NATIVE_VIDEO_BACKEND \
  -u OPENNOW_EMBEDDED_READY_WAKE -u QSG_RENDER_LOOP -u QT_QPA_UPDATE_IDLE_TIME \
  tools/run-native.sh
```

Target the returned OpenNOW PID/window with `get_window_state`. Ready means the normal signed-in library or resume view is visible. Choose the visible Ori entry and normal `Resume` or `Play` action. Resume the same cloud session for a comparison block. If Play allocates a new session, start a new block and controls. Never infer successful auth from a running process.

At stream readiness, confirm Germany, H.264 8-bit, 1920×1080, 60 source FPS, frame generation off, and the intended decoder in the normal settings/statistics view. Return to the animated waterfall. Keep UI theme, window/fullscreen state, audio configuration, and scene identical across controls and candidates. Display refresh normally matches; an explicitly planned method-3 display experiment may request 60 or 120 Hz while source FPS remains 60. Both baseline controls must use the same requested refresh. Restarting or changing region interrupts playback; warn before that action. Teardown is in Cleanup.

## Doctor

Run this before driving the app, after unexpected behavior, and before scoring. Run build hashing and capability queries outside scored windows.

```sh
python3 .cursor/skills/verify-gfn-experiments/scripts/doctor.py --capabilities
python3 .cursor/skills/verify-gfn-experiments/scripts/doctor.py --require-stream
```

For an explicitly planned 120 Hz display candidate, add `--display-hz 120` to Doctor and measurement. The default is 60 Hz; actual Qt readback must stay within 1 Hz of the request. This option validates intent and does not change display hardware or stream settings.

The helper reports build hashes, exact Qt/core process identities, sanitized Wi-Fi state, receive-buffer limits, and fresh Qt observer state. It reads no credential files, process arguments, process environments, or raw native logs. It makes no account requests or configuration changes. VA-API capability enumeration describes advertised support; it does not prove live decoding or GPU-frame import. Saved-account readiness must be confirmed in the GUI.

Source revision and binary hashes are separate facts. Confirm the build record before attributing a running binary to a new source change. A successful Doctor does not rebuild the client or establish that correspondence. Doctor also does not verify experiment environment variables. Before scoring, the owning operator must separately retain an allowlisted readback of only the named experiment flags for the exact Qt PID/start-time instance; never dump its full environment. Prove the intended flag values in the candidate and their recorded control values, with the same compiled implementation throughout the comparison block.

`ready_for_gui` permits inspection or launch. `ready_for_measurement_preflight` means one current observer satisfies the runner's present-state checks. The full measurement still has to prove continuity and workload validity. Ordinary playback launched without telemetry can be healthy while `--require-stream` fails. Retain that distinction instead of restarting a user's stream for an unrelated check.

## Drive

Read [the feature map](features/README.md), then the relevant feature file. Use fresh CUA snapshots before each action and verify the visible result afterward. Prefer returned element tokens; use coordinates only from the current screenshot when Qt exposes no useful accessibility tree. Repeat `session: gfn-verify` on calls that accept it. Foreground input follows the existing CUA authorization rules; the user's authorization for this workflow persists.

Read-only stream measurement is an exact CLI operation. Start it after normal GUI playback reaches the fixed scene. Do not build, run tests, hash large files, change the workload, or send gameplay input inside scored windows. A single small input check belongs before warmup. Continue observations through the full interval; accept the runner's invalid or rejected result rather than retrying until a lucky number appears.

## Evidence

Create a new private proof directory before launch or reuse:

```sh
umask 077
mkdir -p /home/gamer/dev/gfn-client-research/.runtime
GFN_PROOF_DIR=$(mktemp -d /home/gamer/dev/gfn-client-research/.runtime/verification-XXXXXXXX)
python3 .cursor/skills/verify-gfn-experiments/scripts/doctor.py --capabilities > "$GFN_PROOF_DIR/doctor-before.json"
```

Save action/result screenshots there with `get_window_state`'s `screenshot_out_file`. Record the feature IDs covered, client ownership, UTC start/end, safe workload/variant labels, and the exact measurement artifact paths in `proof.md`. Capture game or statistics views; keep account identifiers and authentication URLs out of published evidence. Screenshot success alone proves no performance improvement.

The runner writes private trial/comparison JSON in `.runtime/hillclimb/`. Retain invalid trials, rejected candidates, preflight failures, and restoration evidence. Publish numeric aggregates and limitations in `research/`, with source revision, method version, and comparison decisions. Never publish tokens, credentials, session identifiers, raw log payloads, or packet contents. Packet-header metadata can show timing and transport structure; encrypted captures do not establish decoded protocol semantics.

Qt `frameSwapped` timing is a presentation callback measure. It does not prove physical scanout or input-to-photon latency. GPU mailbox replacements and queue drops are distinct events; adding them does not yield exact visible loss. CPU distributions from the current runner can be distorted by batched Qt telemetry, so do not use them for acceptance or CPU improvement claims.

## Cleanup

Preserve a reused user client unless the task calls for restarting or closing it. For a client this run created, use its exact CUA PID/window and normal close confirmation, then verify that recorded Qt PID/start-time instance and its recorded core child exited. Returning to the library does not complete cleanup. Never kill by process name. If normal close fails, terminate only the recorded process after checking its start ticks still match. Wait for the observer processes this run created to finish, or terminate those exact recorded instances.

Restore every temporary experiment setting to its recorded pre-run value and verify the restored value. For kernel buffer work, verify both `rmem_max` and `rmem_default`, restart the client to establish new socket allocations, and confirm actual native UDP socket buffers. Cancel only this experiment's rollback timer after restoration succeeds. The completed October 7 buffer block restored both caps to 212992 and observed socket values of 425984 bytes; inspect live state before a future run rather than assuming those values forever.

Retain proof directories, trial JSON, and decision history. Confirm their files still exist after cleanup. If the task requires leaving Ori running, restore the baseline variant and resume through the normal GUI. Report which process remains and which checks are incomplete.

## Helpers

- `scripts/doctor.py` is the executable read-only helper above. Exit 0 means its requested preflight passed; exit 1 lists sanitized `issues` to inspect.
- `tools/live-trial.py` measures and compares existing streams. See [live measurement](features/live-measurement.md).
- `tools/archive-trial-logs.py --output NEW_DIRECTORY` preserves the four known log files after all workspace Qt/core processes exit. It refuses existing output directories and prints metadata only.
- `tools/live-network-observer.py` and `tools/live-socket-observer.py` gather bounded numeric evidence. See [network comparison](features/network-comparison.md).

Maintain the feature map when an entry point, measurement rule, or cleanup contract changes. `/maintain-verification-skill` can guide that update. After changing this skill, execute Launch, Doctor, one mapped feature, Evidence, and Cleanup against the real app; retain the proof. Frontmatter validation alone does not complete that check.
