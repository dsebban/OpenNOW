---
name: start-gfn-hdr
description: Start or resume the native OpenNOW GeForce NOW client on the ChimeraOS NucBox with Wayland HDR, saved credentials and HEVC 10-bit video. Use for HDR startup, restoring this machine's HDR playback profile, or checking whether its stream actually contains HDR.
---

# Start GFN with HDR

Workspace: `/home/gamer/dev/gfn-client-research`.
Read the current [global runbook](/home/gamer/.codex/AGENTS.md) and
[$operate-gfn-nucbox](/home/gamer/.agents/skills/operate-gfn-nucbox/SKILL.md).
For GUI actions use [$cua-driver](/home/gamer/.agents/skills/cua-driver/SKILL.md)
and its [Linux guide](/home/gamer/.agents/skills/cua-driver/LINUX.md).
Use [$verify-gfn-experiments](/home/gamer/.codex/skills/verify-gfn-experiments/SKILL.md)
for measurements; its H.264 baseline is a separate experiment profile. An HDR
startup request preserves the user's current profile and clears experimental
timing flags. The table below records a known fallback profile.

## Inspect the desktop and existing client

- Discover the current session and monitor with `loginctl` and
  `gdctl show --properties`. Expected configuration is GNOME Wayland,
  MSI MD342CQP on DP-1, 3440x1440, scale 1, color mode `bt2100`.
  The user changed refresh to 100 Hz on 2026-10-08; preserve that choice.
  Confirm current state instead of assuming the connector or mode is unchanged.
- Persistent HDR is recorded in `~/.config/monitors.xml`. X11 cannot provide
  this client's Linux HDR output path. A session migration restarts the desktop;
  warn before it and use the runbook's LightDM backup/rollback procedure.
  Ordinary HDR startup does not need a desktop restart.
- Inspect existing OpenNOW windows through CUA. Reuse a working native Wayland
  client. If replacement is needed, warn before interrupting playback, use the
  normal Quit confirmation, and verify the exact workspace Qt process and its
  core child have exited. Quit can first return to the library. Never kill by
  name or launch a second authenticated core alongside the GUI.
- The tested build is vendor branch `nucbox-wayland-hdr`, commit `819ce3f`, or a
  successor containing its Mutter compatibility fix. The decisive live check is
  Settings → Stream showing **HDR output ready**. Do not silently change source
  branches or rebuild during playback.

If local HDR is off, preserve the discovered resolution and refresh when
enabling it. This historical 60 Hz command illustrates the syntax; substitute
the current mode (presently `3440x1440@100.000`) rather than resetting refresh:

```sh
gdctl set --persistent --logical-monitor --primary --scale 1 \
  --monitor DP-1 --mode 3440x1440@59.999 --color-mode bt2100
```

Accept GNOME's **Keep Changes** dialog within its 20-second deadline through a
fresh CUA desktop snapshot. Re-read `gdctl` and `monitors.xml`. Let the change
revert if the display becomes unusable. Do not add `--layout-mode physical`;
this compositor rejects configuring that property. `--verify` alone does not
apply a display change. Discover equivalent values for a different display.

## Launch with the current Wayland environment

The installed **OpenNOW Custom** GNOME favorite uses
`~/.local/share/applications/io.github.opencloudgaming.OpenNOW.desktop` and this
maintained launcher with `--activate-existing`. It focuses an existing workspace
Qt instance through native single-instance activation; otherwise it launches
normally. The switch is consumed by the helper, never forwarded to gameplay.
The default command below still refuses an active Qt/core instance. The desktop
filename matches the client's fixed Wayland app ID; its name and blue C icon
distinguish the custom build. It preserves saved preferences and display mode.
Steam uses `--foreground --activate-existing`: the helper replaces itself with
the normal native launcher, which then replaces itself with Qt, so a new launch
remains tracked until OpenNOW exits. Activating an older instance still exits
after the native handoff; it does not transfer that process's lifetime to Steam.

Once the old workspace Qt/core processes have exited:

```sh
python3 /home/gamer/.codex/skills/start-gfn-hdr/scripts/launch-wayland.py
```

The helper reuses `tools/run-native.sh`, imports the allowlisted desktop
environment from the user service manager, requires Wayland and sets
`QT_QPA_PLATFORM=wayland`. It clears the known experiment and live-telemetry
variables and refuses a second workspace Qt/core process. It does not edit
preferences, switch sessions, enable display HDR, unlock the keyring, or prove
stream readiness. Use its returned PID to discover the current CUA window.

CUA needs its native Wayland backend, the current user-session environment and
the already installed `winrects@cua` extension. Follow its Linux guide if capture
fails. Do not substitute X11 input tools or reuse old window IDs/coordinates.

Restore the saved account through the normal GUI. A locked **Default keyring**
needs its vault password, which can differ from sudo. Have the user unlock the
prompt or use credentials explicitly supplied for it in the current session.
Never embed passwords in the skill/helper, reset the vault, or export tokens.

## Set the HDR startup profile

In Settings → Stream, first confirm **HDR output ready**. Preserve the saved
profile for ordinary launch. When an explicit profile restore is requested,
these settings are the known October 7 fallback:

| Setting | Value |
|---|---|
| Codec | H.265 / HEVC |
| Color quality | 10-bit, YUV 4:2:0 |
| HDR | On |
| Resolution / frame rate | 1920x1080 / 60 FPS |
| Maximum bitrate | 50 Mbps |
| Decode / native video backend | Auto, expected embedded Vulkan |
| Upscaling, frame generation, replay buffer | Off |
| Save bandwidth | Off |
| Fullscreen when ready | On |

These are the successful 2026-10-07 startup settings, not a measured performance
optimum. Respect a later explicit user profile. This iGPU has no advertised AV1
hardware decode. A synthetic HEVC capability result alone does not prove live
playback. Acknowledge the app's first-use 10-bit notice if it appears.

Settings apply to the next stream. Use normal Resume or Play for the requested
game and check what the resumed session actually negotiated. If a new cloud
session is required, explain the interruption before ending the existing one.
Application Quit and in-stream End session have different effects.

## Verify output and incoming content separately

1. Confirm the signed-in library and then visible game frames. Use one harmless
   keyboard input to confirm control. In Ori, keyboard navigation works when
   captured mouse clicks do not. Resume the existing save, never create or
   overwrite one merely to verify playback.
2. Use **Ctrl+N** for stream statistics. Check HEVC, actual 10-bit color quality,
   1080p60 and hardware decode. Record bounded FPS/ping/loss readings without
   presenting them as a controlled comparison. Restore the prior overlay state.
3. Check the game's HDR option and current-session decoded color evidence.
   The HUD's HDR badge reflects profile flags and is not proof of HDR content.
   `bt2100` and **HDR output ready** prove the local output configuration only.
4. Capture private evidence under a new workspace `.runtime/` directory with
   `umask 077`. Record UTC time, process identity, profile and screenshots.
   Before launch, note the size/inode of `.runtime/logs/launch.log`; inspect only
   newly appended data. Rotation or old records cannot prove this attempt.
   Extract only this allowlisted message from that data, never dump raw logs:

```text
Video composition: sourceColorSpace=PQ2020 textureBits=10 outputBits=16 dither=none
```

The values above illustrate the message format; use actual observed values.
It is emitted after GPU texture import. `SDR709` means that frame is classified
as SDR even when it is 10-bit. `PQ2020` supports an HDR content finding, although
unspecified bitstream metadata can inherit negotiated defaults. `outputBits=16`
can describe an intermediate target; it is not proof of physical scanout depth.
Match output readiness, decoded metadata and the in-game setting before claiming
HDR playback. Report any missing evidence explicitly.

The last checked Ori session requested HDR and decoded HEVC 10-bit successfully,
but imported SDR709 frames and showed **HDR N/A** in both borderless and exclusive
fullscreen. Borderless was restored. Do not repeat mode cycling as a routine
startup step. Keep usable playback and report this unresolved limitation rather
than claiming the toggle fixed it. Performance membership itself is not a proven
blocker; see [the investigation](/home/gamer/dev/gfn-client-research/research/wayland-hdr.md).

Leave the requested game running after a startup task. Preserve routes, Wi-Fi,
VPN, router/SQM policy and display refresh unless the user asks to change them.
