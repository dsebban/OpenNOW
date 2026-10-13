#!/usr/bin/env python3
"""Observe gateway RTT and local scheduling without changing connectivity.

Run beside normal playback, outside scored trials. The heartbeat and pressure
samples help identify host stalls; they do not measure NIC arrival or prove an
RF cause. Output contains numeric metadata only, with no account or peer data.
Ping timestamps are userspace reply timestamps. Schedstat counters are
cumulative bucket measurements, not scheduler wakeup/switch traces.
"""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import threading
import time


def process_start_ticks(pid):
    return int(Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()[19])


def thread_counters(pid):
    counters = {}
    for task in Path(f'/proc/{pid}/task').iterdir():
        try:
            values = [int(v) for v in (task / 'schedstat').read_text().split()]
            counters[task.name] = {'runtimeNs': values[0], 'runqueueWaitNs': values[1],
                                   'timeslices': values[2]}
        except (OSError, ValueError, IndexError):
            continue  # A thread may exit between directory enumeration and read.
    return counters


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seconds', type=int, default=60)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--pid', type=int, help='Read schedstat for this exact client instance')
    parser.add_argument('--progress', action='store_true', help='Print numeric progress every minute')
    args = parser.parse_args()
    if not 1 <= args.seconds <= 900:
        parser.error('--seconds must be between 1 and 900')
    if args.pid is not None and args.pid <= 0:
        parser.error('--pid must be positive')
    identity = process_start_ticks(args.pid) if args.pid else None
    os.umask(0o077)
    args.output.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    # Reserve output before observing; never overwrite another attempt.
    with args.output.open('x') as output:
        result = {'schema': 1, 'requestedSeconds': args.seconds,
                  'gateway': '192.168.8.1', 'pings': [], 'heartbeat': [],
                  'pressure': [], 'interrupted': False}
        result['process'] = {'pid': args.pid, 'startTicks': identity}
        result['threadSamples'] = []
        result['processChanged'] = False
        try:
            result['schedSchedstats'] = int(Path('/proc/sys/kernel/sched_schedstats').read_text())
        except OSError:
            result['schedSchedstats'] = None
        start = time.monotonic()
        result['startWallUs'] = time.time_ns() // 1000
        ping = subprocess.Popen(
            ['ping', '-n', '-D', '-O', '-i', '0.1', '-w', str(args.seconds),
             '192.168.8.1'], stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, env={**os.environ, 'LC_ALL': 'C'})

        def read_pings():
            for line in ping.stdout:
                counts = re.search(r'(\d+) packets transmitted, (\d+) received', line)
                if counts:
                    result['pingTransmitted'] = int(counts[1])
                    result['pingReceived'] = int(counts[2])
                match = re.search(
                    r'^\[([\d.]+)\].*icmp_seq=(\d+).*time([=<])([\d.]+) ms', line)
                if match:
                    result['pings'].append({
                        'pingWallUs': round(float(match[1]) * 1_000_000),
                        'readerWallUs': time.time_ns() // 1000,
                        'sequence': int(match[2]), 'rttMs': float(match[4]),
                        'rttUpperBound': match[3] == '<'})

        reader = threading.Thread(target=read_pings, daemon=True)
        reader.start()
        try:
            for tick in range(args.seconds * 10):
                due = start + tick / 10
                time.sleep(max(0, due - time.monotonic()))
                now = time.monotonic()
                result['heartbeat'].append({
                    'wallUs': time.time_ns() // 1000,
                    'elapsedMs': (now - start) * 1000,
                    'lateMs': max(0, now - due) * 1000})
                if args.pid:
                    try:
                        if process_start_ticks(args.pid) != identity:
                            result['processChanged'] = True
                            break
                        sample = {'startWallUs': time.time_ns() // 1000,
                                  'counters': thread_counters(args.pid)}
                        sample['endWallUs'] = time.time_ns() // 1000
                        result['threadSamples'].append(sample)
                    except (OSError, ValueError):
                        result['processChanged'] = True
                        break
                if tick % 10 == 0:
                    sample = {'wallUs': time.time_ns() // 1000}
                    for kind in ('cpu', 'io', 'memory'):
                        try:
                            sample[kind] = Path('/proc/pressure', kind).read_text().strip()
                        except OSError:
                            sample[kind] = None
                    result['pressure'].append(sample)
                if args.progress and tick and tick % 600 == 0:
                    replies = result['pings'][:]
                    print(json.dumps({'elapsedSeconds': round(now - start),
                                      'replies': len(replies),
                                      'maxRttMs': max((p['rttMs'] for p in replies), default=None),
                                      'maxHeartbeatLateMs': max(h['lateMs'] for h in result['heartbeat'])}),
                          flush=True)
        except KeyboardInterrupt:
            result['interrupted'] = True
        finally:
            if (result['interrupted'] or result['processChanged']) and ping.poll() is None:
                ping.terminate()
            try:
                ping.wait(timeout=2)
            except subprocess.TimeoutExpired:
                ping.kill()
                ping.wait(timeout=2)
            reader.join(timeout=2)
        result['endWallUs'] = time.time_ns() // 1000
        result['elapsedSeconds'] = time.monotonic() - start
        result['wallMonotonicSpanDifferenceUs'] = (
            result['endWallUs'] - result['startWallUs']
            - result['elapsedSeconds'] * 1_000_000)
        result['pingExitCode'] = ping.returncode
        result['pingReaderFinished'] = not reader.is_alive()
        result['pingError'] = ping.stderr.read(4096).strip()
        result['summary'] = {
            'transmitted': result.get('pingTransmitted'),
            'received': result.get('pingReceived'),
            'replies': len(result['pings']),
            'rttOver20Ms': sum(p['rttMs'] > 20 for p in result['pings']),
            'heartbeatOver20Ms': sum(h['lateMs'] > 20 for h in result['heartbeat']),
            'maxRttMs': max((p['rttMs'] for p in result['pings']), default=None),
            'maxHeartbeatLateMs': max((h['lateMs'] for h in result['heartbeat']), default=None)}
        json.dump(result, output, allow_nan=False)
        output.write('\n')
    print(json.dumps(result['summary']))
    return 0 if (result['pings'] and not result['pingError']
                 and result['pingReaderFinished'] and not result['processChanged']) else 1


if __name__ == '__main__':
    raise SystemExit(main())
