# User-accepted GFN working profile

Recorded 2026-10-12 on the ChimeraOS NucBox. The user reported:
“Smooth as butter … it works great.” Preserve this setup during ordinary
playback. Further tuning should address a reported regression or an explicitly
requested experiment, rather than replacing this accepted baseline speculatively.

## Verified configuration

| Component | Working setting |
| --- | --- |
| Client | Custom OpenNOW Qt/native desktop client, existing deployed build |
| Source video | 2560×1080 ultrawide, 60 FPS |
| Codec/color | H.264, 8-bit YUV 4:2:0, HDR off |
| Requested bitrate ceiling | 25 Mbps |
| Decoder/backend preference | Auto / auto |
| Frame generation | Off |
| Physical display | DP-1, 3440×1440, 120 Hz, SDR `default` color mode |
| Network | Direct internet; no VPN |
| Flint SQM | Permanently disabled by explicit user instruction |
| Router WAN queue | `noqueue`; SQM IFB helper absent |
| Wi-Fi radio | Existing channel 64 / HE40 retained |
| WED | Existing runtime `N` / saved disabled state retained |
| Firmware | Existing pesa1234 `next-r4.9.2.rss.mtk` build retained |
| Adaptive quality | Saved `saveBandwidth=false`, unchanged |
| L4S preference | Saved `enableL4S=true`, unchanged |

L4S-off and an official-client comparison were researched but were **not**
applied. Do not attribute the improvement to either. No CPU performance-mode
change succeeded. There was no router firmware flash or radio reconfiguration
in this improvement sequence.

## Changes that led here

1. Replaced HEVC 10-bit HDR streaming with H.264 8-bit SDR at a temporary
   16:9 1920×1080 source resolution.
2. Changed active monitor output from 100 Hz HDR to 120 Hz SDR.
3. Reduced the saved bitrate ceiling from 100 to 25 Mbps, with API readback.
4. Restored the requested 2560×1080 ultrawide source resolution. Streams were
   restarted/reallocated as required by the profile changes. A blank picture
   on one allocation recovered after reconnecting the client.
5. On the user's explicit instruction, disabled Flint SQM permanently:
   `sqm.eth1.enabled=0`, committed its configuration, stopped its WAN queues,
   and disabled SQM boot startup. No timed restoration is scheduled.
6. The user then confirmed smooth gameplay and accepted this configuration.

The latest substantial network change was SQM disablement, but several client
settings and cloud allocations also changed. This sequence supports a working
configuration and user-reported improvement; it does not isolate one cause.

## Persistence and recovery

The stream settings and SQM-off state are saved. The **display change is
runtime-only**: at documentation time, `~/.config/monitors.xml` still stored
3440×1440 at 100 Hz with `bt2100`. After a reboot or desktop restart, inspect
the actual display state before comparing playback. Do not silently interpret
the old saved HDR mode as the accepted GFN working profile.

The prior SQM 230/50 Mbps configuration is backed up on Flint at
`/root/gfn-sqm-backups/20261012T190010Z-32246`. The user instructed that SQM stay
off even if no effect was measured. That backup is historical recovery evidence,
not permission to re-enable SQM automatically.

The SQM authorization is scoped. Keep Wi-Fi association, router radios, routing,
DNS, VPN and unrelated firewall policy unchanged under the connectivity guard.
See [machine operations](/home/gamer/.codex/references/machine-operations.md) and
[the firmware reference](flint2-custom-firmware.md) for the underlying history.

## Evidence and limits

The fast observer's completed SQM-off run recorded 90.1 seconds, eight
`video-mailbox` replacements, no ≥100 ms presentation-gap alarms, about
59.81 fresh presentations per second and a final stream RTT of 61.76 ms.
It followed a fresh allocation and included startup/low-motion workload, so
it is not a matched causal comparison or sustained gameplay acceptance test.

At documentation time the live readback showed about 59.99 FPS, 62.07 ms
stream RTT, 0% reported packet loss, and active ungated 2560×1080 playback.
The user's smooth-gameplay report is the practical acceptance. Zero recorded
drops and RTT below 62 ms at all times have not been demonstrated. Qt timing
and mailbox counters do not establish physical scanout or exact visible loss.

Private evidence remains in `.runtime/`:

- `sqm-permanent-off-result-20261012.json`
- `sqm-permanently-off-live-feedback-20261012.jsonl`
- `quickwin-state-20261012.json`
- `exa-flint-gfn-findings-20261012.json`

Use `tools/fast-stall-feedback.py` for quick regression feedback, with a freshly
verified Qt PID and a new output file. Keep this profile and the cloud allocation
recorded. A normal playback launch may omit experiment telemetry; the last
observed client used the baseline live-trial launcher with telemetry enabled.

## HDR / HEVC follow-up

Later on 12 October, the user reported that HDR could no longer be enabled and
H.265 attempts interrupted streaming. The SDR monitor mode from the smoothness
changes made the HDR control unavailable. Restoring active DP-1 output to
3440×1440 at 120 Hz with `bt2100` restored **HDR output ready** and the enabled
HDR control. At the user's brightness request, `gdctl prefs --monitor DP-1
--luminance 60` reduced the HDR luminance preference from 112.5 to 60.

The saved bitrate was already 50 Mbps when this investigation began; retain
that user change. The latest saved profile is H.265, 10-bit 4:2:0, HDR enabled,
2560×1080 at 60 FPS, 50 Mbps, with auto backend/decoder. Desktop client mode was
restored. SQM and other connectivity settings were untouched.

Reconnecting the existing STAR WARS: Galactic Racer session still received
H.264 8-bit, despite the new preferences. Ending that cloud session and starting
a fresh allocation produced H.265 10-bit video. A transient session-discovery
failure preceded the successful fresh allocation. Prior failures showed RTSP
setup/recovery errors; their exact cause remains unconfirmed. No decoder crash
or source-code fix was established.

The fresh H.265 stream stayed connected through a 180.1-second observer run and
showed game video. That run recorded two presentation-gap alarms and 148 video
mailbox replacements, most during a brief inactive-window interval. It included
startup, shader preparation and the brightness change, so it is recovery evidence,
not a matched performance comparison or proof of zero stalls. Decoded frames
were classified `SDR709` with 10-bit textures; actual HDR game content is still
unproven. Keep the H.264 SDR configuration above as the accepted smooth fallback.

The 120-Hz monitor change remains runtime-only; `monitors.xml` still stores
100 Hz HDR. Private evidence is in `.runtime/hdr-hevc-20261012-feedback.jsonl`,
`.runtime/hdr-hevc-20261012-prior/` and `.runtime/hdr-hevc-before.json`.
