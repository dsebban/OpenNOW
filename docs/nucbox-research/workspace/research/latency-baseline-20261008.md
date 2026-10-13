# OpenNOW latency baseline — 2026-10-08

**COMPLETE: 3/3 fresh descriptive windows. This is a component/pacing baseline, not a measured end-to-end input-to-photon number.**
Scored 09:55:20.687–09:58:21.687 UTC. No optimization or source instrumentation was implemented.

## Conditions and provenance

- Branch `nucbox-wayland-hdr`, source `819ce3f37ff5315d0940f02df7272856d0d0f362`. Remote vendor and Mac source trees unchanged.
- User-approved normal GUI quit (including confirmation) ended original Qt PID `1497234` / start ticks `15357620` and core `1497311` / `15357727`. Exit checks passed; existing native/Qt logs were archived before relaunch.
- Relaunched through the measured repo wrapper using Cua Driver. Qt PID `1611294` / start ticks `16362358`; core `1611498` / `16362453`. Identity and source/binary fingerprints remained unchanged through scoring.
- Resumed existing Ori save at the stationary animated Ink water Marsh spirit-well scene. Before/after screenshots show the same position; no gameplay input or screenshots during scoring. Ctrl+N stats were checked and then closed before warmup.
- Saved profile unchanged: GUI reports Germany, H.265, 2560×1080, 10-bit 4:2:0, HDR and Linux decoder/embedded Vulkan, ~60 source FPS, 100 Mbps allocated. Native diagnostics also report H265 / 10bit_420 / hdr=true. This does not independently measure panel luminance or prove photon-level HDR output.
- Native Wayland, fullscreen; desktop 3440×1440 at 100 Hz, frame generation off. `OPENNOW_LIVE_TELEMETRY` enabled at construction; other measured experiment overrides absent. No VPN/network/router/Wi-Fi/SQM/rmem/profile changes; no builds/tests/hashing during scored intervals.
- One small entry point `tools/latency-baseline.py` reuses observe-stream, live-trial and presentation-census; supports N bounded windows, explicit profile expectations, private numeric evidence, retained failures and dry-run.
- Doctor normal/capabilities exits 0, `ready_for_gui`. Its frozen stream observer reports `display_refresh_mismatch` at this 100-Hz desktop (and its frozen profile is 1920×1080/60 or 120 Hz). That legacy acceptance gate is **not passed or weakened**; the descriptive collector validates explicit 2560×1080/100-Hz expectations separately.

Built SHA-256: Qt `3b72d8411d8a613fbd79aee01964a790f34e6da1fa577a1c5946e653571cb964`; core `87baad1986e90dc5d00e2357c7892f2ab6b7ecd5af43f92996ed68ad9058fc0a`; FFI `2349750aeb4ca2955f1c111d70cdecd5bb8f1f27ca1f6a33aa86e558d850c11e`. Hashes are artifact identity, not an independently reproducible build attestation.

## Exact commands

Remote working directory: `/home/gamer/dev/gfn-client-research`. SSH used `ssh -o HostName=192.168.8.224 chimeraos` after intermittent mDNS failure. The launch below was issued via Cua `launch_app`, not an additional shell/account operator. Background raw-input delivery refused; the explicitly approved foreground fallback was used.

```sh
env QT_QPA_PLATFORM=wayland /home/gamer/dev/gfn-client-research/tools/run-live-trial.sh baseline
python3 .cursor/skills/verify-gfn-experiments/scripts/doctor.py --capabilities
python3 tools/latency-baseline.py --pid 1611294 --width 2560 --height 1080 \
  --display-hz 100 --scene ori-inkwater-marsh-spirit-well \
  --label 20261008-preflight3 --dry-run
# Socket and host observers ran alongside scoring (no --flint / router access):
python3 tools/live-socket-observer.py --pid 1611294 --duration 240 \
  --output .runtime/latency-baseline-20261008-live-KwogFwiG/sockets.json
python3 tools/live-network-observer.py --duration 240 \
  --output .runtime/latency-baseline-20261008-live-KwogFwiG/network.json
python3 tools/latency-baseline.py --pid 1611294 --width 2560 --height 1080 \
  --display-hz 100 --scene ori-inkwater-marsh-spirit-well \
  --label 20261008-a1 --warmup 20 --seconds 60 --windows 3
```

## Results

All slash-separated timing entries are **p50/p95/p99/max, milliseconds**. Network entries describe only six sampled native summaries per window; p95/p99 therefore equal the sampled max. Decode p50/p95 are medians of reported last-256 rolling summaries, max is greatest rolling max, and true window p99 is unavailable. Qt entries are 1-ms histogram bucket upper bounds, not exact elapsed-time quantiles.

| Window | Duration (s) | Fresh / decoded FPS | Late >25 ms | RTT | RTP jitter estimator |
| --- | ---: | --- | --- | --- | --- |
| 1 | 60.996 | 59.95 / 60.04 | 28 (0.77%) | 61.38/62.085/62.085/62.085 | 0.067/0.089/0.089/0.089 |
| 2 | 60.002 | 59.96 / 60.05 | 30 (0.83%) | 61.441/61.979/61.979/61.979 | 0.111/0.122/0.122/0.122 |
| 3 | 60.001 | 59.93 / 60.06 | 30 (0.83%) | 61.059/63.118/63.118/63.118 | 0.061/0.1/0.1/0.1 |

| Stage | Window 1 | Window 2 | Window 3 |
| --- | --- | --- | --- |
| Decode call (rolling) | 7.964/10.319/—/14.824 | 7.98/10.305/—/14.73 | 7.941/10.247/—/12.663 |
| Decoder residence (rolling) | 7.965/10.32/—/14.826 | 7.982/10.306/—/14.731 | 7.942/10.247/—/12.664 |
| Fresh-source interval | 17/21/25/36 | 17/21/25/35 | 17/21/25/41 |
| Render submit → Qt swap | 1/1/1/12 | 1/1/1/12 | 1/1/1/13 |
| Oldest notification → GUI drain | 1/1/2/6 | 1/1/1/7 | 1/1/1/6 |
| Latest notification → GUI drain | 1/1/2/6 | 1/1/1/7 | 1/1/1/6 |

- Fresh swaps: 3657 / 3598 / 3596. Histogram overflow, PTS discontinuity, acquire/record stale/error counters: all zero. Presentation/notification/record epochs remained stable; no boundary skew.
- Window callbacks ~62.61 / 62.60 / 62.60 FPS, with 4.24 / 4.21 / 4.26% having no fresh source. This is not 100-Hz physical output measurement.
- Video-mailbox replacement/drop events: 2 / 3 / 4. Native reported packet-loss samples: all 18 zero; this does not prove every packet survived.
- Assembly→admission, admission→ACK-control queue and assembly→ACK-control queue percentiles: **unavailable/null** in timestamped native output, not zero.
- Paired observers completed (240 samples each), fully covering all windows. Owned UDP ports 49005/49006: observed kernel drop deltas zero; sampled max receive queue 0/0, 0/0, 0/2304 bytes. One-second snapshots can miss short queues.
- Gateway RTT p50/p95/p99/max by window: 0.601/1.35/5.49/5.49; 0.647/1.55/1.87/1.87; 0.676/1.65/3.06/3.06 ms, all 181 in-window probes replied. Gateway RTT is not GFN RTT.
- Host UDP InErrors/RcvbufErrors/MemErrors deltas zero. Namespace NoPorts deltas 9/4/1 are not attributable stream loss. Memory-pressure totals increased by 1622 µs in window 2, zero in windows 1/3; do not infer causality. Boundary-bounded counter samples exclude subsecond edges.

## Verification and artifacts

- `latency-preflight`, `latency-windows`, `latency-summary`: **PASS for this descriptive baseline**, not a frozen acceptance or end-to-end latency pass.
- Three >=60-second windows, continuous telemetry, additive census, unchanged process/start ticks/source/artifact identity. All reported windows were recomputed from retained 18 native numeric and 182 Qt census samples and exactly matched summary JSON.
- Compile/import, empty/overflow tails, historical three-window parser integration and duplicate-label preservation checks passed outside scoring.
- A post-run Markdown-only fix put all window rows before stage tables. Original Markdown retained; scored JSON unchanged (SHA-256 `d48b3cda6d4b74508d07eb40c4732b257ac0a1a779b5c19193cfadf35a53e8c1`). Run-time and post-format script hashes are recorded separately.
- Main evidence: `.runtime/latency-baseline-20261008-a1/` (summary JSON/Markdown, observation, numeric samples); GUI/Doctor/archive/socket/network/verification proof: `.runtime/latency-baseline-20261008-live-KwogFwiG/`.
- Earlier failed preflights `20261008-preflight1/2` retained (old launch lacked Qt telemetry and was not streaming); `preflight3` passed after approved relaunch. Preparation/audit retained in `.runtime/latency-baseline-20261008-snseO0nu/`.
- Raw logs/screenshots/account material remain private under ignored `.runtime/`. Nothing committed/published. Observers exited normally; the telemetry-enabled client remains running in the measured scene. No settings require restoration.

## Measurement coverage and validity

| Component | Existing evidence and limit |
| --- | --- |
| Network RTT/jitter | Sampled `pingMs` (fresh ICE preferred, video/bundle fallback), RTP interarrival-jitter estimator. Sample p50/p95/p99/max, not every probe/packet or one-way delay. |
| Receive/assembly | Owned-socket queue bytes/drop deltas and assembled counters; no packet residence, read/processing time or first-packet-to-assembly latency. |
| Assembly → admission/ACK queue | Rolling last-256 quantiles exist internally but timestamped native diagnostic allowlist omits them; report null. Untimestamped stderr is not scored-window proof. |
| Decode call/residence | Last-256 rolling p50/p95/max. Collector reports median rolling p50/p95, greatest reported max; true window p99 unavailable. Call time is not complete GPU duration. |
| Decoded queue/mailbox | Replacement/drop counters; **no per-frame residence/age** to Qt. Counts cannot be summed into exact visible losses. |
| Qt drain/submit/pacing | Cumulative notification-drain and submit-to-swap/fresh-interval histograms allow 1-ms upper-bound p50/p95/p99/max, with overflow tails null. Notification age is not GPU-frame age. Enabled and continuous in the scored launch. |
| Physical output/input-to-photon | **Not observable**. Qt `frameSwapped` is neither compositor presentation nor physical scanout/panel response. |

Rolling summaries overlap and may include warmup. Do not add stage percentiles, RTT/2 or callback proxies to infer total end-to-end latency. The same-source earlier 60-Hz experiment in `research/latency-20261008.md` is a different session/profile/time and is not a matched optimization comparison. Three contiguous windows establish short-run repeatability, not across-session/day reliability. Instrumentation and observer overhead were not measured against an otherwise identical telemetry-off control.

## Ranked instrumentation proposals (not implemented)

1. Export existing native stage metrics and **cumulative bounded timing
   histograms** with sample/overflow/epoch counters and monotonic clock mapping;
   obtain real window deltas/p99 instead of overlapping rolling summaries.
2. Carry sender frame ID/PTS and monotonic stamps from decode through queue,
   mailbox, Qt acquisition/render/swap; include replacement age, outcomes and
   surface generations. Update all ABI producers/consumers/versioning together.
3. At the native socket owner, measure kernel packet age, read wait, processing
   to next read, first/last packet to assembly and overflow; align kernel and
   userspace clocks explicitly. Do not change buffer policy yet.
4. Correlate Qt swaps with compositor/present feedback where supported; retain
   physical panel-response uncertainty.
5. Separate input-response proxy workload: stamp accepted input and transport
   send, detect a deterministic *causal* cloud-game menu change in a bounded
   diagnostic ROI, with no-input false-positive controls. No normal-path CPU
   copying or per-frame raw logging.

Input timestamp → **next arbitrary frame** measures only callback wait; an
unrelated animated frame can arrive before the server reacts. Camera-free
causal response detection can estimate input-to-client-response/presentation,
**not true input-to-photon**. No input experiment or optimization was made.
