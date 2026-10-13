# Last night playback analysis 13 October 2026

The user reported very smooth gameplay on the evening of 12 October. Retained
logs show H.265 10-bit playback at 2560×1080 with a 120-Hz display. Typical stream
RTT was about 57 ms. Occasional presentation gaps and packet-loss recovery remained;
the strict zero-drop target was not met. No configuration was changed for this analysis.

All times below are Asia/Jerusalem. The two substantial playback intervals were
23:49:34–00:08:27 and 00:09:06–00:31:46, crossing into 13 October. A preceding
39-second reconnect was H.264 and is excluded from the table. The first long
interval ended through a local stop request. The final interval ended when the
host suspended, rather than through a demonstrated H.265 decoder crash.

## Steady playback

The table excludes the first two minutes of each interval and pairs containing
an inactive or hidden window. Measurements use cumulative counter differences,
not averages of the HUD's rounded FPS. Histogram counts reconcile in every view.

| Measurement | First session | Second session |
| --- | ---: | ---: |
| Included active playback | 16.7 minutes | 20.7 minutes |
| Fresh source presentations per second | 59.81 | 57.74 |
| Median stream RTT | 56.92 ms | 57.79 ms |
| Stream RTT P95 | 64.98 ms | 64.81 ms |
| Maximum sampled stream RTT | 82.35 ms | 75.26 ms |
| Source interval P99 upper bound | 33 ms | 42 ms |
| Source intervals at least 100 ms | 10 | 50 |
| Maximum source interval upper bound | 255 ms | 361 ms |
| Encoded video queue drops | 42 | 441 |
| Video mailbox replacements | 87 | 136 |
| Keyframe requests | 5 | 45 |
| Maximum sampled packet-loss percentage | 1.37% | 9.17% |

The median reported packet-loss percentage was zero in both intervals. Nonzero
bursts still occurred, including a 9.17% sample at 00:22:37 in the second session.
Seven of ten large gaps in the first interval and 44 of 50 in the second fell
within one second of an encoded-video queue-drop record. This is temporal
correlation, not proof of a Wi-Fi, router, ISP or cloud cause. No continuous
gateway or socket observer covered the later gameplay.

During valid native counter intervals, incoming assembled video averaged about
59.94 FPS in the first session and 58.24 FPS in the second. Decoded output averaged
59.94 and 57.87 FPS respectively. The latter calculation excludes a decoder
timing-epoch change at 00:29:07. These rates support reduced incoming/decoded frame
delivery in the second session; they do not identify its cause. Queue drops and
mailbox replacements are separate events and must not be summed as visible losses.

The full intervals included startup and brief focus changes. Their maximum gaps
were approximately 618 ms and 1184 ms; those cumulative maxima do not describe
the steady windows above. Decoded composition remained `SDR709`, with 10-bit
textures and a 16-bit composition target. Actual HDR game content is unproven.

## Final disconnect

At 00:31:46, systemd-logind recorded “The system will suspend now!” NetworkManager
then recorded sleep-driven Wi-Fi deactivation. At 00:31:47 the client reported
`nvst-transport-stopped`, and the kernel entered deep suspend. The next Qt sample
arrived at 16:34:37. Its wall clock advanced 57,770 seconds while the monotonic
clock advanced only 2.28 seconds, independently supporting overnight suspension.
This analysis did not initiate any network or sleep action.

## Decision and reproduction

Preserve the working configuration. The user's acceptance and successful HEVC
playback are useful results, but these unmatched sessions do not isolate SQM
disablement as the cause of improvement. The earlier Ori controls used a different
game, display/profile history and cloud allocation. Further diagnosis should
capture the remaining burst-loss and reference-recovery events during actual
gameplay, without replacing a setup the user considers smooth.

Run the read-only analyzer against retained logs:

```sh
python3 tools/analyze-playback-logs.py \
  --since 2026-10-12T23:40:00+03:00 \
  --until 2026-10-13T02:00:00+03:00 \
  --output .runtime/NEW-private-analysis.json
```

The analyzer snapshots each input's initial length, checks Qt continuity and
counter conservation, separates active playback, and excludes changed native
timing epochs from counter rates. It emits numeric summaries without credentials
or raw session payloads. Private proof is
`.runtime/last-night-playback-analysis-20261013-proof.json`; inputs remain
`.runtime/hillclimb/qt-presentation.jsonl` and
`.runtime/config/OpenNOW/diagnostics/native-streamer.log`. The terminal event was
cross-checked against the local system journal. All results are historical triage,
not physical scanout measurements or a matched performance acceptance test.
