# Saved-account playback

## Sub-features

- `playback-launch`: launch the expected native Qt build and restore the normal saved account.
- `playback-resume`: resume an existing Ori cloud session through the visible GUI.
- `playback-play`: launch Ori through normal Play when no resumable session remains, and record a new comparison block.
- `playback-input`: verify one keyboard input reaches the game before scoring.
- `playback-stats`: inspect the live profile and statistics, then restore the previous overlay state.
- `playback-close`: close only the client instance owned by the verification run.

## How to get to it (user POV)

Open OpenNOW, use its normal signed-in library or resume view, select the visible Ori game, and choose Resume or Play. Once gameplay appears, use the game's keyboard controls. Ctrl+N is the default statistics shortcut; inspect the configured shortcut if it has been customized. The window close action may present a confirmation before exiting.

## Driving it with CUA Driver

1. Follow Launch and Doctor in `../SKILL.md`. Capture the initial state with `get_window_state`, supplying the exact returned `pid`, `window_id`, session `gfn-verify`, and an absolute `screenshot_out_file` inside the proof directory.
2. Select the visible Ori/Resume/Play control using the latest `element_token`. If the accessibility tree is empty, click the control's location from that same screenshot. Take a fresh snapshot after each action and observe loading through to gameplay. Do not fabricate a session ID or use an internal setter to bypass the real flow.
3. Before warmup, send one appropriate movement or jump input supported by the visible game state. For `press_key`, supply the actual `key`, exact `pid`/`window_id`, and `session`. Inspect `cua-driver describe press_key` if the tool schema differs. Verify a changed game frame, then return to the fixed scene.
4. Toggle statistics using the configured shortcut, normally Ctrl+N, through `press_key`. Inspect the current tool schema for modifier syntax. Capture the statistics view and confirm negotiated resolution/FPS/codec plus the intended decoder when exposed. Restore the overlay state with the same verified shortcut. Some fields require the normal settings view; absent information stays unverified. F3 is not the current default.
5. For a run-owned client, close that exact window through CUA. Inspect and accept the normal confirmation if present. If it returns to the library, take a fresh snapshot and close the remaining application window. Verify that both the recorded Qt PID/start-time instance and its recorded core child ended before launching another variant. Preserve all screenshots and the proof file.

The pass condition is visible real gameplay after the normal account/session path, verified input, the intended profile, and correct process cleanup or an explicitly requested running baseline. Auth expiration is a concrete blocker; do not retrieve secret values to work around it.

## Gotchas

- Separate `.runtime` directories share the OS credential vault. Standalone authenticated core probes must not overlap the GUI.
- A Quit confirmation can leave the Qt/core pair running at the library. A replacement launch then risks a single-instance handoff timeout and an unchanged experiment environment. Process exit, verified by PID and start ticks, is the required launch boundary.
- Native GPU capability enumeration is insufficient. HEVC negotiation and embedded VA-API preflight have failed on this host even when lower-level hardware queries advertised support.
- Resume can fail because the cloud session expired. A new Play allocation changes the comparison block, even if the region and game match.
- A scene loaded at a menu or static frame does not meet the waterfall workload. Backgrounded or hidden video also fails measurement preflight.
- Leave gameplay input and screenshots outside scored windows. Keep UI theme and overlay state identical across a comparison.
