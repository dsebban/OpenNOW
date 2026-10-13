# Saved-session region evaluation

Recorded: 2026-10-07T11:06:07.884355+00:00

OpenNOW restored the existing account through its normal saved-session API. No credential was copied into the report. All 24 discovered NVIDIA regions returned usable results in each of five rounds. Each round is a rounded mean of three TCP connects after a warmup; the table reports the median of those means. These are endpoint preflights, not in-stream UDP latency or loss tests.

| Region | Median ms | Round range ms | Failed rounds |
|---|---:|---:|---:|
| Germany | 60 | 59–62 | 0 |
| Netherlands South | 64 | 64–65 | 0 |
| Netherlands North | 65 | 65–66 | 0 |
| France 1 | 69 | 68–73 | 0 |
| France 2 | 70 | 69–70 | 0 |
| United Kingdom 1 | 72 | 70–73 | 0 |
| United Kingdom 2 | 72 | 71–75 | 0 |
| Poland | 81 | 80–84 | 0 |
| Bulgaria | 85 | 83–87 | 0 |
| Sweden | 88 | 85–92 | 0 |
| Virginia (USA) | 144 | 143–144 | 0 |
| New Jersey (USA) | 153 | 151–155 | 0 |
| Georgia (USA) | 156 | 154–159 | 0 |
| Illinois (USA) | 159 | 158–160 | 0 |
| Ontario (Canada) | 160 | 150–179 | 0 |
| Quebec (Canada) | 162 | 156–175 | 0 |
| Florida (USA) | 170 | 169–172 | 0 |
| India | 171 | 169–174 | 0 |
| Texas (USA) | 178 | 175–179 | 0 |
| Arizona (USA) | 202 | 199–203 | 0 |
| Oregon (USA) | 202 | 202–205 | 0 |
| Northern California (USA) | 203 | 202–205 | 0 |
| Southern California (USA) | 204 | 202–204 | 0 |
| Japan | 296 | 282–300 | 0 |

## Prior live observation

The unmodified native client was observed on a requested Bulgaria session at 1440p60, H.264 8-bit 4:2:0. The HUD showed about 67 ms ping, 40.3 Mbps video, 7.4 ms decode/residence and 17 selected local queue drops across that session. This is not matched to the official 1080p60 HEVC 10-bit sample. Bulgaria’s later endpoint preflight measured 85 ms, illustrating why it cannot replace actual session latency.

The existing HUD omits decoded-video overflow and replaced GPU mailbox frames. The diagnostic branch corrects those units and reports mailbox replacement separately, and writes a bounded numeric telemetry summary without free-form payloads. It does not alter transport deadlines, decoder selection or presentation scheduling, and does not establish an improvement.

## Changes worth evaluating

1. Use the existing shared Vulkan HEVC decoder and GPU conversion path; a real P010 GPU-only hardware test already passed. Verify full Qt playback next. CPU AV1 and GPU FEC are not justified on this hardware by current evidence.
2. Measure render backlog, decoded queue drops and mailbox replacements before changing frame pacing. Preserve encoded reference frames and sender identities.
3. Instrument the 52 ms FEC/NACK retention policy on this roughly 57–60 ms route. Compare a bounded 68 ms or RTT-informed recovery candidate only with returned-packet and freeze evidence; preserve healthy-path latency.
4. Bound the unbounded commands ahead of SCTP, coalescing motion while preserving key releases and shutdown priority. Measure age and depth first.

The original profile is backed up privately. Region and codec settings are changed through authenticated core requests with readback, while the GUI is closed to avoid token-refresh and profile-lock races.

## Diagnostic branch verification

Fork commit `2cbd906` passed 18 focused release tests: 13 protocol-log tests,
4 queue-report tests and 1 Linux mailbox ownership/replacement test. The mailbox
fixture uses CPU-backed frames to validate publisher accounting, not GPU output.
The earlier independent P010 Vulkan decode hardware test exercised RADV RENOIR.
These are distinct proofs.

## Custom branch live trial

Two Germany HEVC 10-bit startup attempts failed before video arrived: the DTLS/SCTP transport became ready but video inbound/authenticated counts stayed zero, followed by media timeout/recovery exhaustion. This does not establish a GPU decoder failure. An H.264 8-bit trial then displayed Ori through the embedded Vulkan decoder at 1080p60. No network/router policy was changed.

The saved trial lasted 348.3 seconds, including startup, menus and gameplay. It had 33 telemetry windows: median ping 57.01 ms, median stream FPS 59.99, maximum reported packet-loss percent 0.0. Median-of-window decoder p50 was 5.529 ms; gameplay HUD readings reached about 7.9 ms. These are window summaries, not all-frame percentiles.

Local drop sources: `{'video-mailbox': 319, 'video': 13}`; keyframe/recovery events: `{'keyframe-request': 1}`. Mailbox replacements discard superseded decoded frames and are not network loss or proof of visible scanout skips. The prior unmodified HUD did not count this source, so totals are not an A/B improvement comparison. The compressed-video source still combines ingress/decoder-command overflow.

A concurrent 60-packet gateway sample had 60 replies, average 1.677 ms and maximum 6.594 ms. A bounded Flint header capture had 287 inbound and 13 outbound video-port packets; it confirms flow direction, not end-to-end loss. Full GPU decode, Qt composition and visible gameplay are now observed for H.264. HEVC 10-bit live negotiation remains unresolved despite the separate P010 hardware test passing.

The custom client is left running Germany/H.264 1080p60. Original settings are retained privately for restoration. Netherlands and other regions were TCP-preflight checked, not played. Next experiments should first address frame presentation/decoder backlog, then HEVC negotiation, bounded command admission and measured recovery policy.
