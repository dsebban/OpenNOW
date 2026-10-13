# NucBox Wayland HDR investigation — 2026-10-07

## Result

The native OpenNOW Qt client now recognizes the local HDR output after a narrow Mutter compatibility fix. This establishes output readiness. The observed Ori stream remains SDR: successful HEVC 10-bit playback produced `sourceColorSpace=SDR709 textureBits=10 outputBits=16`, and Ori's Display menu showed HDR as unavailable. End-to-end HDR and playback performance improvement have not been demonstrated.

## Environment and diagnosis

The NucBox uses an AMD Ryzen 7 5700U integrated GPU, Mesa 25.1.1, GNOME 48.2 / Mutter 48.3.1, and custom Qt 6.9.1. The desktop was migrated to native Wayland. GNOME retained DP-1 at 3440×1440, approximately 60 Hz, with bt2100 enabled; DRM reported BT2020_RGB. Vulkan exposed HDR10_ST2084_EXT and EXTENDED_SRGB_LINEAR_EXT surface formats. OpenNOW was already built with `OPENNOW_WAYLAND_HDR`, generated color-management protocol bindings, and the required Wayland libraries.

A separate metadata-only Wayland probe queried the compositor without creating a surface or accessing the account. It reported one output and a ready color manager, with:

| Metadata | Observed value |
|---|---|
| Transfer function | ST2084 PQ |
| Primaries | BT.2020, D65 |
| Raw luminances | minimum 50, maximum 10000, reference 203 |
| Converted luminances | minimum 0.005 nits, maximum 10000 nits, reference white 203 nits |
| Description | Ready and complete |
| Target primaries / target luminance | Not emitted |

**10000 is the primary color volume's encoding maximum, not a measured panel peak.** The missing target-luminance event caused OpenNOW's observer to reject the output before its Vulkan swapchain checks.

Exact [Mutter 48.3.1 implementation](https://github.com/GNOME/mutter/blob/48.3.1/src/wayland/meta-wayland-color-management.c#L380-L490) emits primary chromaticities, transfer function and luminances, followed by completion, but no target metadata. The [Wayland color-management protocol](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/staging/color-management/color-management-v1.xml) requires target information events for parametric descriptions. Its parametric defaults define unspecified mastering luminance using the primary color volume; it also states that target luminance is theoretical and need not match actual display emission. The corresponding pinned OpenNOW XML is `opennow-qt/protocols/color-management-v1.xml`, particularly the `set_mastering_luminance` and `wp_image_description_info_v1` descriptions.

## Fix

Vendor branch **`nucbox-wayland-hdr`**, commit **`819ce3f`** (`Support Mutter HDR descriptions without target luminance`):

- `WaylandHdrOutput::stateForDescription` uses the primary range when target luminance is omitted. Existing completeness, PQ, color and finite/range checks remain enforced; explicit target values remain authoritative.
- `State.targetLuminanceProvided` preserves the distinction and participates in publication change detection.
- `HdrOutput` withholds `nativeHdrDisplay` when target luminance was omitted, so the fallback encoding maximum is never advertised to the core as panel brightness.
- HDR output support still requires an active, renderable HDR swapchain. Missing display metadata retains the core's existing content requests: maximum 1000, minimum 0 and maximum frame average 400.

Relevant source owners are `opennow-qt/src/streaming/rendering/WaylandHdrOutput.cpp`, `HdrOutput.cpp`, and `native/opennow-core/src/cloudmatch.rs::monitor_display_data`. Focused regressions are in `opennow-qt/tests/tst_waylandhdroutput.cpp`.

## Validation and limits

The bounded build succeeded. Focused Wayland observer and HDR color/shader tests passed. The initial shader test attempt lacked `DISPLAY`; it was rerun successfully with the corrected Xwayland environment. These tests cover metadata handling and shader behavior, not physical HDR scanout.

The real native Wayland GUI subsequently reported **HDR output ready**. A live Ori session successfully played HEVC 10-bit video at approximately 60 FPS. Existing Qt composition logging, emitted after successful GPU texture import, reported SDR709 / 10-bit source texture / 16-bit composition output. The HUD's HDR label comes from profile flags and therefore does not prove HDR content. Outgoing ANNOUNCE requested HDR and 10-bit 4:2:0; existing logs did not provide an accepted server HDR mode. The cause of the SDR stream remains unresolved.

[NVIDIA HDR documentation, answer 5390](https://nvidia.custhelp.com/app/answers/detail/a_id/5390), checked on 2026-10-07, says both **Performance and Ultimate** memberships support HDR10 and 10-bit color precision. It distinguishes HDR content from SDR content encoded at 10-bit precision. Membership alone cannot explain this result. Its supported-platform table does not certify this custom Linux client.

Private evidence is retained under `.runtime/wayland-hdr-20261007/`: numeric probe metadata and source, build/test logs, output-readiness screenshots and the live stream/game observations. Credentials, raw session messages and identifiers are excluded from this public report. No end-to-end HDR or performance claim is made.
