#!/usr/bin/env python3
"""Report live stalls within seconds. Read-only triage, not a scored trial.

Uses existing Qt/native telemetry, gateway ICMP and owned UDP counters. Stops
on log rotation or changed client/presentation identity rather than skipping
evidence. Gateway correlation cannot identify RF, ISP or cloud causality.
"""

import argparse
from collections import deque
import importlib.util
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import threading
import time

ROOT = Path(__file__).resolve().parents[1]


def helpers(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'tools' / filename)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pid', type=int, required=True)
    parser.add_argument('--seconds', type=int, default=90)
    parser.add_argument('--stop-after-incidents', type=int, default=1,
                        help='0 keeps observing until the time limit')
    parser.add_argument('--drop-threshold', type=int, default=1,
                        help='Fail-fast frame drop count; use 8 to focus on bursts')
    parser.add_argument('--max-ping-ms', type=float, default=62,
                        help='Reject after 3 consecutive over-target stream RTT samples; 0 disables')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--gateway', default='192.168.8.1')
    args = parser.parse_args()
    if (not 1 <= args.seconds <= 900 or args.pid <= 0
            or args.stop_after_incidents < 0 or args.drop_threshold < 1
            or not 0 <= args.max_ping_ms <= 1000):
        parser.error('positive PID, seconds 1..900, nonnegative incident limit required')
    # Keep the probe LAN-only; do not let a typo turn it into an external probe.
    if args.gateway != '192.168.8.1':
        parser.error('only the known local gateway is supported')
    trial = helpers('feedback_trial', 'live-trial.py')
    sockets = helpers('feedback_sockets', 'live-socket-observer.py')
    identity = sockets.process_start_ticks(Path(f'/proc/{args.pid}/stat').read_text())
    os.umask(0o077)
    args.output.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    fd = os.open(args.output, os.O_CREAT | os.O_EXCL | os.O_WRONLY |
                 getattr(os, 'O_NOFOLLOW', 0), 0o600)
    output = os.fdopen(fd, 'w')
    qt = trial.Tail(ROOT / '.runtime/hillclimb/qt-presentation.jsonl')
    native = trial.Tail(ROOT / '.runtime/config/OpenNOW/diagnostics/native-streamer.log')
    pings = deque(maxlen=400)
    heartbeat = deque(maxlen=400)
    ping = subprocess.Popen(['ping', '-n', '-D', '-O', '-i', '0.1', '-w',
                             str(args.seconds), args.gateway], stdout=subprocess.PIPE,
                            stderr=subprocess.DEVNULL, text=True,
                            env={**os.environ, 'LC_ALL': 'C'})

    def read_pings():
        for line in ping.stdout:
            match = re.search(r'^\[([\d.]+)\].*icmp_seq=(\d+).*time([=<])([\d.]+) ms', line)
            if match:
                pings.append((float(match[1]) * 1000, float(match[4]),
                              int(match[2]), match[3] == '<'))

    reader = threading.Thread(target=read_pings, daemon=True)
    reader.start()
    started = time.monotonic()
    bridge = time.time() - started
    previous = None
    latest_qt = None
    first_qt = None
    latest_native = None
    drop_totals = {}
    incidents = 0
    finish_at = None
    next_report = started + 10
    next_socket = started
    socket_previous = {}
    socket_sample = None
    status = 'complete'
    reason = None
    ongoing_stall = False
    latency_samples = deque(maxlen=3)
    latency_reported = False

    def emit(kind, **values):
        record = {'event': kind, 'wallTimeMs': round(time.time() * 1000), **values}
        text = json.dumps(record, allow_nan=False)
        output.write(text + '\n')
        output.flush()
        print(text, flush=True)

    def context(wall_ms):
        replies = [r for r in list(pings) if wall_ms - 2000 <= r[0] <= wall_ms]
        wakes = [h[1] for h in list(heartbeat) if wall_ms - 2000 <= h[0] <= wall_ms]
        return {'gatewayRepliesLast2s': len(replies),
                'gatewayMaxMsLast2s': max((r[1] for r in replies), default=None),
                'gatewayReplyAgeMs': wall_ms - pings[-1][0] if pings else None,
                'observerWakeMaxMsLast2s': max(wakes, default=None),
                'socketSample': socket_sample,
                'streamPingMs': latest_native.get('pingMs') if latest_native else None,
                'streamPingAgeMs': wall_ms - latest_native['time_ms'] if latest_native else None}

    def incident(kind, **values):
        nonlocal incidents, finish_at
        incidents += 1
        emit(kind, ordinal=incidents, **values, **context(time.time() * 1000))
        if (args.stop_after_incidents and incidents >= args.stop_after_incidents
                and finish_at is None):
            finish_at = time.monotonic() + 2

    emit('started', schema=1,
         scriptSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
         pid=args.pid, startTicks=identity, limitSeconds=args.seconds,
         limits='Triage only. Qt export is 1 Hz; native drops flush about 1 Hz; '
                'stream ping about 10 s. ICMP and userspace wakeups do not prove NIC arrival.')
    due = started
    try:
        while time.monotonic() - started < args.seconds:
            now = time.monotonic()
            if finish_at is not None and now >= finish_at:
                break
            wall = time.time() * 1000
            if abs(time.time() - now - bridge) > .05:
                raise trial.InvalidTrial('wall_clock_changed')
            heartbeat.append((wall, max(0, now - due) * 1000))
            if sockets.process_start_ticks(Path(f'/proc/{args.pid}/stat').read_text()) != identity:
                raise trial.InvalidTrial('client_identity_changed')
            if now >= next_socket:
                current, errors = sockets.sample_process(args.pid, identity)
                shared = set(current) & set(socket_previous)
                comparable = (bool(shared) and set(current) == set(socket_previous)
                              and not errors and all(current[i]['socketDrops'] >= socket_previous[i]
                                                     for i in shared))
                delta = (sum(current[i]['socketDrops'] - socket_previous[i] for i in shared)
                         if comparable else None)
                socket_sample = {'wallTimeMs': round(wall), 'ownedUdpSockets': len(current),
                                 'dropDelta': delta, 'ownershipReadErrors': errors,
                                 'counterContinuity': comparable,
                                 'rxQueueMaxBytes': max((r['rxQueueBytes'] for r in current.values()), default=0)}
                socket_previous = {inode: row['socketDrops'] for inode, row in current.items()}
                next_socket = now + 1
            for line in native.lines():
                row = trial.parse_native(line)
                if row is None:
                    continue
                if row['kind'] == 'telemetry':
                    latest_native = row
                    if 'pingMs' in row:
                        latency_samples.append(row['pingMs'])
                        over = (args.max_ping_ms > 0 and len(latency_samples) == 3
                                and all(v >= args.max_ping_ms for v in latency_samples))
                        if over and not latency_reported:
                            incident('latency-over-target', targetMs=args.max_ping_ms,
                                     samplesMs=list(latency_samples))
                        latency_reported = over
                elif row['kind'] == 'drop':
                    media, count = row['media'], row['count']
                    drop_totals[media] = drop_totals.get(media, 0) + count
                    if count >= args.drop_threshold:
                        incident('frame-drop', source=media, count=count,
                                 nativeEventWallMs=row['time_ms'],
                                 logDetectionDelayMs=round(time.time()*1000-row['time_ms'], 1))
                elif row.get('invalid'):
                    incident('native-recovery-or-stop', reason=row['invalid'],
                             nativeEventWallMs=row['time_ms'])
            for line in qt.lines():
                raw = json.loads(line)
                if (not raw.get('runtimeRunning') or raw.get('videoWidth', -1) <= 0
                        or raw.get('videoHeight', -1) <= 0):
                    raise trial.InvalidTrial('stream_not_playing')
                row = trial.parse_qt(line)
                if previous is not None:
                    changed = trial.continuity(previous, row)
                    if changed:
                        raise trial.InvalidTrial(changed)
                    histogram = [b-a for a, b in zip(previous['sourceIntervalHistogramMs'],
                                                    row['sourceIntervalHistogramMs'])]
                    gaps = sum(histogram[100:])
                    overflow = row['sourceIntervalOverflowTotal'] - previous['sourceIntervalOverflowTotal']
                    active = all(row[key] and previous[key] for key in
                                 ('visible', 'windowActive', 'runtimeRunning'))
                    active = active and not row['gated'] and not previous['gated']
                    if (gaps or overflow) and active:
                        bins = [i for i, count in enumerate(histogram) if count]
                        incident('presentation-gap', count=gaps + overflow,
                                 gapMaxUpperMs=max(bins)+1 if bins and not overflow else None,
                                 intervalStartWallMs=latest_qt['wallTimeMs'],
                                 intervalEndWallMs=raw['wallTimeMs'],
                                 exportDetectionDelayMs=round(time.time()*1000-raw['wallTimeMs'], 1))
                previous = row
                if first_qt is None:
                    first_qt = (row['sourceSwapsTotal'], row['monotonicNs'])
                latest_qt = {'wallTimeMs': raw['wallTimeMs'],
                             'sourceSwapsTotal': row['sourceSwapsTotal'],
                             'active': raw['visible'] and raw['windowActive'] and not raw['gated'],
                             'resolution': [raw['videoWidth'], raw['videoHeight']]}
                elapsed = (row['monotonicNs'] - first_qt[1]) / 1e9
                latest_qt['sourceFpsSinceStart'] = ((row['sourceSwapsTotal'] - first_qt[0]) / elapsed
                                                  if elapsed > 0 else None)
                gap = raw.get('sinceLastSourceSwapMs')
                stalled = latest_qt['active'] and isinstance(gap, (int, float)) and gap >= 100
                if stalled and not ongoing_stall:
                    incident('ongoing-presentation-stall', elapsedGapMs=gap)
                ongoing_stall = stalled
            if latest_qt and wall - latest_qt['wallTimeMs'] > 2500:
                raise trial.InvalidTrial('qt_telemetry_stale')
            if latest_qt is None and now - started > 2.5:
                raise trial.InvalidTrial('qt_telemetry_missing')
            if (latest_native and wall - latest_native['time_ms'] > 25000
                    or latest_native is None and now - started > 25):
                raise trial.InvalidTrial('native_telemetry_missing_or_stale')
            if now >= next_report:
                emit('summary', elapsedSeconds=round(now-started, 1), incidents=incidents,
                     dropsBySource=drop_totals, qt=latest_qt, **context(wall))
                next_report = now + 10
            due = time.monotonic() + .1
            time.sleep(max(0, due - time.monotonic()))
    except (trial.InvalidTrial, OSError, ValueError) as error:
        status, reason = 'invalid', str(error)
    except KeyboardInterrupt:
        status, reason = 'interrupted', 'operator_interrupt'
    finally:
        if ping.poll() is None:
            ping.terminate()
        try:
            ping.wait(timeout=2)
        except subprocess.TimeoutExpired:
            ping.kill()
            ping.wait(timeout=2)
        reader.join(timeout=2)
        emit('finished', status=status, reason=reason, incidents=incidents,
             verdict=('unqualified' if status != 'complete' else
                      'incident_observed' if incidents else 'no_incident_in_this_window'),
             elapsedSeconds=round(time.monotonic()-started, 1), dropsBySource=drop_totals,
             qt=latest_qt, **context(time.time()*1000))
        output.close()
    return 0 if status == 'complete' else 1


if __name__ == '__main__':
    raise SystemExit(main())
