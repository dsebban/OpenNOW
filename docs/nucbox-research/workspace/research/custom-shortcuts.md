# Custom OpenNOW launch entries

Installed on October 8, 2026 for the NucBox's native Wayland build.

## Desktop

`~/.local/share/applications/io.github.opencloudgaming.OpenNOW.desktop` is named
**OpenNOW Custom**, uses a blue C icon and is appended to GNOME's favorites.
Existing favorites retain their order. Its desktop ID matches the client's
fixed Wayland app ID, so the native window groups with its launcher.

The entry runs the maintained HDR skill's `scripts/launch-wayland.py` with
`--activate-existing`. Existing workspace Qt windows receive native
single-instance activation. A core-only orphan still causes refusal rather
than another account client. The helper consumes its own switches, imports
the current session's allowlisted display environment and clears experiments.
It preserves the saved client profile, user display mode and OS credential vault.

## Steam and Big Picture

The durable ChimeraOS definition is
`~/.local/share/chimera/shortcuts/opennow-custom.yaml`. It points to the same
native helper with `--foreground --activate-existing`, with no Proton override.
Foreground mode uses `exec` through `tools/run-native.sh` into Qt, keeping a
new client launch attached for its full lifetime. Activating an older desktop
instance cannot transfer that process's lifetime to Steam.

Steam's missing client files were downloaded through its normal bootstrap,
and the user signed in through Steam. Account files and shortcut backups stay
private. Steam's overlay is disabled for this shortcut; other entries and
settings are preserved. The real library entry is installed in that signed-in
profile in addition to the durable ChimeraOS definition.
ChimeraOS's installed generator hardcodes overlay-on during future Gamescope
shortcut regeneration, so recheck this entry's overlay preference afterward.
Its stable command/name-derived ID matches the definition and avoids duplicates.

Live verification showed exactly one tile in Big Picture's Non-Steam tab.
After normal close of the earlier workspace Qt/core instances, Steam's Play
button launched a new workspace Qt process with `QT_QPA_PLATFORM=wayland` and
no experiment or telemetry flags. The saved GFN account and library were
visible. Process ancestry through Steam's reaper confirmed foreground lifetime
tracking. This check did not launch a cloud game or measure streaming quality.

## Source publication

The custom build's source branch was pushed to
<https://github.com/dsebban/OpenNOW/tree/nucbox-wayland-hdr>.
Remote and local heads both resolved to
`819ce3f37ff5315d0940f02df7272856d0d0f362`. The working source tree was clean.
The outgoing HDR commit contains four source/test files; runtime data,
credentials and packet captures were not uploaded.

These launch entries establish convenient access, not a measured latency or
frame-pacing improvement. Direct internet is required by the user's current
instruction, and the user-selected 100 Hz HDR desktop is preserved.
