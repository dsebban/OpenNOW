# Flint 2 custom firmware and GFN

Reviewed 2026-10-12. Operational settings and authorized exceptions belong in
[/home/gamer/.codex/references/machine-operations.md](/home/gamer/.codex/references/machine-operations.md).
Keep firmware research read-only under the existing connectivity rule. The
user's permanent SQM-off instruction does not authorize firmware flashing,
Wi-Fi changes, or changes to routing, DNS, VPN, or unrelated firewall policy.

## Provenance and build

The user identifies [pesa1234/MT6000_cust_build](https://github.com/pesa1234/MT6000_cust_build)
as the Flint 2 firmware reference. The installed router matches the latest
published artifact set visible at review:

- Build directory: `2026-08-29_r36948-611fea61ac_next-r4.9.2.rss.mtk`.
- Source branch: `next-r4.9.2.rss.mtk` in
  [pesa1234/openwrt](https://github.com/pesa1234/openwrt/tree/next-r4.9.2.rss.mtk).
- Source commit: `611fea61ac079f127189f73261b0a0179e1b015d`.
- Target: `mediatek/filogic`, `glinet_gl-mt6000`, `aarch64_cortex-a53`.
- Kernel package: `6.18.44~e681dd05ce0396defcfaedcdb36af8cc-r1`.
- mt76/mt7915e packages: `6.18.44.2026.07.01~59676919-r1`.
- LuCI advanced app: `1.1.0-r2`; SQM scripts: `1.8.0-r2`.

Read-only router package/version output matched these published values. This
establishes a strong build match, not a byte-for-byte image attestation.
`OpenWrt SNAPSHOT` in `ubus` identifies the upstream base; it does not mean the
router runs an unmodified vanilla image or stock GL.iNet firmware.

The [published target directory](https://github.com/pesa1234/MT6000_cust_build/tree/main/2026-08-29_r36948-611fea61ac_next-r4.9.2.rss.mtk/targets/mediatek/filogic)
contains `config.buildinfo`, `feeds.buildinfo`, `version.buildinfo`, the package
manifest, image profiles and checksums. The feeds pin the author's packages
and LuCI forks plus upstream routing, telephony and video revisions. The build
uses Cortex-A53 `-O2` optimization and OpenSSL/zlib speed optimizations.

To reproduce the release, use its source branch/commit, feed pins and build
configuration. The fork's `main` branch and the examples in the discussion
can refer to different versions. Generic snapshot kernel modules may have a
different ABI; consult the release's packages and
[maintainer's package-compatibility discussion](https://github.com/pesa1234/MT6000_cust_build/issues/1).
Do not substitute an image, package feed or bootloader based on a model-name match.

## Features relevant to stability

| Feature | Verified source/build evidence | GFN relevance and limits |
| --- | --- | --- |
| Ethernet RSS and IRQ/RPS distribution | RSS patches and an MT6000-specific CPU/IRQ map in `advanced_setup` | Can distribute receive work under load. A missing standard `network.globals.packet_steering` value does not prove steering is off; the custom script has its own defaults. |
| MediaTek Wi-Fi patches | mt76/mt7915e vendor patches, including EDCCA and queue handling | Potential stability/performance changes. Their presence is not evidence of lower GFN packet loss on this installation. |
| WED control | Advanced module-option synchronization and a WDMA TX-hang fix | Runtime WED was `N`, and the saved setting was `0`. Mode changes require a reboot according to the source. The WED-specific fix is already included and does not justify enabling WED blindly. |
| Airtime fairness | ATF/WATF controls and runtime `vow_atf` support | Saved ATF was `1`, with both observed PHY runtime values `1`. Fairness can change scheduling between stations; no GFN improvement was established. |
| CAKE/SQM and optional CAKE-MQ control | CAKE, SQM and LuCI packages; an optional debugfs synchronization control | Ordinary CAKE was observed before the user's SQM-off instruction. A CAKE-MQ control in the script does not establish that CAKE-MQ was active. |
| Additional LuCI Wi-Fi controls | The author's feature overview describes iBF, EDCCA, TWT and background-radar/chanlist controls | These change radio behavior. Inspect the exact release and current state before considering a separately authorized test. Avoid applying old discussion settings as a template. |

The pinned source includes the
[WED v2 WDMA TX-hang fix](https://github.com/pesa1234/openwrt/commit/ec602512ab07b1a5194dd226cc0466ff16e4e7e5),
dated 2026-08-28. The
[author's build and feature discussion](https://forum.openwrt.org/t/mt6000-custom-build-with-luci-and-some-optimization-kernel-6-6-x/185241)
is useful background, but spans several kernel generations. Use current
release files and source for exact behavior.

## Assessment for GFN

This firmware is usable for GFN and is already installed at the reviewed
published version. No newer published artifact was found in this repository
at review. Reflashing that same version would not introduce the documented
features that are already present, and is not an established freeze fix.

RSS/IRQ distribution and driver fixes are relevant when receive processing
or router scheduling is the bottleneck. Radio scheduling features are relevant
when local-link delay/retries correlate with a stall. Neither case is proved by
a good signal reading or a feature being available in LuCI.

Use `tools/fast-stall-feedback.py` beside real playback for quick incident
feedback. Keep source resolution/FPS, bitrate, codec, client, cloud allocation
and workload recorded. An incident-free short run is not sustained acceptance.
The SQM-off observation started after the prior stream had stopped, so a fresh
allocation is a confound; do not attribute a changed ping to SQM alone.

Prioritize measured incident correlation and one-variable comparisons over
enabling several custom optimizations. Preserve the current user-directed
SQM state unless the user asks to change it. No GFN-specific guarantee of zero
dropped frames or under-62-ms RTT was found in the firmware's documentation.
