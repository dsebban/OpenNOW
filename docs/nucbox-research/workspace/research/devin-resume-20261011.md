# Devin session continuation, October 11

Resumed the latest session for this workspace, `difficult-pleasure`, from
Devin's local session database and transcript. Its final database record was
node 5017, following a Wi-Fi module reload. The newer `light-station` session
belongs to `surf-cli` and is unrelated to this workspace.

The original objective remains reducing OpenNOW latency and freezes. There is
still no accepted performance improvement. This continuation collected
diagnostics; it changed no network, driver, display, or streaming settings.
Existing uncommitted launcher, stage-analysis, pacer and Doctor work was
preserved.

## Correction to the previous diagnosis

The reported `iw` value of 3 dBm does not establish a real transmit-power cap
or a misapplied beacon constraint. An upstream MT7921 patch documents a stale
`phy->txpower_cur` field that commonly produces this value independently of
the actual power limits. See the [patch explanation and implementation](https://lists.infradead.org/pipermail/linux-mediatek/2026-March/105163.html).

The host runs `6.14.7-chos4-chimeraos-2`. The exact custom module's source was
not audited, so the reporting bug is a strong explanation for the reading,
not proof of actual RF power. No power adjustment, module reload, scan,
association change, or router change was attempted here.

The previous four-event Wi-Fi finding also overstates its confidence. One
listed timestamp differs by about 0.46 seconds, exceeding its stated 100 ms
correlation tolerance. Gateway RTT and enqueue timing can support a local-link
hypothesis, but do not alone distinguish RF delay, kernel handling, or a
blocked receive thread. Enqueue timestamps are userspace processing boundaries.

## Fresh observations

Read-only checks found direct routing through the Flint gateway, `dani_5`
association on channel 60 / HE40, power saving off, and an inactive ExpressVPN
service. Host observability was current; router syslog was stale. The display
remained 3440x1440 at 100 Hz.

The saved-account GUI launched Ori normally into a new cloud allocation. The
live profile was Germany, HEVC, 2560x1080 at 60 source FPS, HDR and 10-bit
settings, frame generation off, and desktop mode. Doctor passed with explicit
100 Hz and 2560x1080 expectations. A startup check using Doctor's default 1920
width correctly rejected the mismatch; invoking the vendor mirror also failed
because it resolves workspace helpers under the vendor root. The working
entry point is the workspace `.cursor` Doctor.

The `stagelog` launcher enabled only frame-stage diagnostics among the checked
experiment flags. Neither pacer nor deferred snapshot wait was enabled.

| Observation | Duration | Gateway results | Presentation evidence |
| --- | --- | --- | --- |
| Initial idle sample | 45 s | 45 replies, median 0.648 ms, max 16.3 ms | Client closed |
| Idle scheduling capture | 60 s | 580 timestamped replies, max 72.0 ms | Heartbeat max lateness 0.763 ms |
| Startup/menu capture | 120 s | 1177/1177 replies, max 66.5 ms | Four publish gaps of 69.2–125.7 ms |
| Stationary waterfall capture | 180 s | 1759/1759 replies, max 55.9 ms | No publish gap at or above 67.0 ms |

Three startup/menu publish gaps overlapped estimated gateway transactions with
elevated RTT. These observations contain loading and menu input and are not a
gameplay benchmark. The fourth gap occurred during the same broader enqueue
starvation period as another gap; four gaps are not four independent incidents.

The fixed-scene capture retained 10,802 publish records. Its maximum publish
gap was 58.008 ms, with 22 gaps over 40 ms. Maximum enqueue gap was 54.755 ms.
Five gateway probes exceeded 20 ms. The observer heartbeat remained within
1.034 ms of schedule. Before/after screenshots show the same location; no
gameplay input or screenshots were sent during this interval.

The partial station-counter interval added 4819 transmit retries, with no
increment in transmit failures, beacon loss, or receive-drop-misc counters.
This interval does not cover the entire capture and supplies no event-level
RF attribution. Retry counters do not establish GFN packet loss.

## Reusable observer and limits

`tools/observe-link-stalls.py` collects timestamped gateway replies, a 100 ms
local scheduling heartbeat, pressure samples, and optional per-thread
`schedstat` counters for an exact PID/start-time instance. It refuses existing
output files, uses private permissions, checks process continuity, and bounds
the ping process. It makes no configuration changes or account requests.

Example, after confirming the current client PID through Doctor:

```sh
python3 tools/observe-link-stalls.py --seconds 180 --pid CURRENT_PID \
  --output .runtime/NEW_PRIVATE_CAPTURE.json
```

This is an unscored diagnostic observer. Ping timestamps are userspace reply
timestamps; estimated send times subtract RTT and are not kernel arrival
timestamps. Thread counters are cumulative bucket measurements, not scheduler
wakeup/switch traces. Low sampled run-queue waits cannot exclude blocked
threads or kernel/NIC delays. The kernel's `sched_schedstats` switch was 0;
per-task counters were readable and advancing. Upstream [proc implementation](https://github.com/torvalds/linux/blob/v6.14/fs/proc/base.c)
and [scheduler declarations](https://github.com/torvalds/linux/blob/v6.14/include/linux/sched/stat.h)
show that these per-task fields depend on `CONFIG_SCHED_INFO`.

The observer was exercised on the real host and stream. The 10-second idle
confirmation recorded 98/98 replies; the 120- and 180-second process captures
retained 1200 and 1800 thread samples with unchanged process identity. Syntax
and `git diff --check` passed. No build or performance acceptance trial ran.
The final observer revision also completed a five-second process capture with
50 thread samples, unchanged identity, private output, and recorded wall versus
monotonic span difference. A probe still outstanding at the ping deadline must
not be interpreted as established packet loss.

## Decision and next evidence

Result: insufficient qualifying fixed-scene stalls to accept a mechanism.
There is no new mitigation candidate or improvement claim. The threshold was
the greater of 50 ms and four times median publish cadence, approximately
67 ms here. This does not mean smaller hitches were absent.

The next investigation needs a longer fixed-scene capture with event-level
socket/kernel receive and critical-thread scheduling evidence. Require
multiple independent stalls, genuine transaction overlap, and a consistent
first stalled pipeline boundary before selecting a mitigation. Do not revive
the rejected pacers or change connectivity based on the 3 dBm display.

Private captures, screenshots, build hashes, advisor response, allowlisted
flags and the rerunnable analysis helper are retained under
`.runtime/resume-devin-20261011/`. Raw session logs and credentials are excluded
from this report. The Surf advisor completed successfully and recommended
observation before another mitigation.

The run-owned Qt client and core exited through the normal GUI Quit
confirmation. Their recorded PID/start-time instances were verified gone.
All bounded observers completed; direct routing and Wi-Fi association remained
intact after cleanup.
