#!/usr/bin/env python3
"""Summarize retained Qt/native logs without account access or raw log output.

Historical triage only: no matched workload, physical scanout or causal claim.
Inputs are bounded to their initial file size so live appends cannot extend a run.
"""
import argparse
from collections import Counter
from datetime import datetime
import importlib.util
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('playback_trial', ROOT / 'tools/live-trial.py')
trial = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = trial
spec.loader.exec_module(trial)


def timestamp(value):
    date = datetime.fromisoformat(value)
    if date.tzinfo is None:
        raise argparse.ArgumentTypeError('timezone offset required')
    return date.timestamp() * 1000


def snapshot_lines(path):
    with path.open('rb') as source:
        size = os.fstat(source.fileno()).st_size
        data = source.read(size)
    lines = data.splitlines()
    partial = bool(data and not data.endswith(b'\n'))
    if partial:
        lines.pop()
    return [line.decode('utf8', errors='replace') for line in lines], {
        'path': str(path), 'snapshotBytes': size, 'partialLastLineExcluded': partial}


def summarize_pairs(pairs, native):
    if not pairs:
        return None
    histogram = [0] * 512
    counts = Counter()
    seconds = 0
    large_events = []
    for first, last in pairs:
        seconds += (last['monotonicNs'] - first['monotonicNs']) / 1e9
        for key in trial.COUNTERS:
            counts[key] += last[key] - first[key]
        delta = [b - a for a, b in zip(first['sourceIntervalHistogramMs'], last['sourceIntervalHistogramMs'])]
        histogram = [a + b for a, b in zip(histogram, delta)]
        # Buckets floor(ms), so bucket 100 starts at 100 ms; overflow >=512 ms.
        large = sum(delta[100:]) + last['sourceIntervalOverflowTotal'] - first['sourceIntervalOverflowTotal']
        if large:
            large_events.append({'startMs': first['wallTimeMs'], 'endMs': last['wallTimeMs'],
                                 'count': large, 'cumulativeMaxMs': last['sourceIntervalMaxMs']})
    start, end = pairs[0][0]['wallTimeMs'], pairs[-1][1]['wallTimeMs']
    # Native sample attribution uses only intervals actually included in this view.
    included = []
    cursor = 0
    for row in native:
        while cursor < len(pairs) and pairs[cursor][1]['wallTimeMs'] < row['time_ms']:
            cursor += 1
        if cursor < len(pairs) and pairs[cursor][0]['wallTimeMs'] < row['time_ms'] <= pairs[cursor][1]['wallTimeMs']:
            included.append(row)
    telemetry = [r for r in included if r['kind'] == 'telemetry']
    drops = Counter()
    for row in included:
        if row['kind'] == 'drop':
            drops[row['media']] += row['count']
    video_drop_times = [r['time_ms'] for r in included if r['kind'] == 'drop' and r['media'] == 'video']
    correlated_gaps = sum(g['count'] for g in large_events
                          if any(g['startMs'] - 1000 <= t <= g['endMs'] + 1000 for t in video_drop_times))
    overflow = counts['sourceIntervalOverflowTotal']
    expected = counts['sourceIntervalSamplesTotal']
    counter_rates = {}
    for key in ('frameStageTimings.assembledFramesTotal', 'decodeTimings.outputsTotal', 'decodeTimings.submissionsTotal'):
        delta, span, excluded = 0, 0, 0
        for first, last in zip(telemetry, telemetry[1:]):
            if (key not in first or key not in last
                    or first.get('decodeTimings.epoch') is None
                    or first.get('decodeTimings.epoch') != last.get('decodeTimings.epoch')
                    or last[key] < first[key]):
                excluded += 1
                continue
            # Do not bridge a window-focus exclusion or missing telemetry interval.
            elapsed = (last['time_ms'] - first['time_ms']) / 1000
            if not 0 < elapsed <= 15:
                excluded += 1
                continue
            delta += last[key] - first[key]
            span += elapsed
        if span:
            counter_rates[key] = {'rate': delta / span, 'coveredSeconds': span,
                                  'excludedIntervals': excluded}
    return {
        'startMs': start, 'endMs': end, 'includedSeconds': seconds,
        'intervals': len(pairs), 'freshPresentations': counts['sourceSwapsTotal'],
        'sourceFps': counts['sourceSwapsTotal'] / seconds,
        'histogramCountMatches': sum(histogram) + overflow == expected,
        'sourceIntervalP95UpperMs': trial.percentile_bucket(histogram, .95, overflow),
        'sourceIntervalP99UpperMs': trial.percentile_bucket(histogram, .99, overflow),
        'sourceIntervalMaxUpperMs': None if overflow else max((i + 1 for i, n in enumerate(histogram) if n), default=None),
        'sourceIntervalsAtLeast100Ms': sum(histogram[100:]) + overflow,
        'sourceIntervalsAtLeast512Ms': overflow,
        'lateIntervalsAboveThreshold': counts['lateSourceIntervalsTotal'],
        'lateThresholdMs': pairs[0][0]['lateThresholdMs'],
        'intervalHistogram': histogram,
        'largeGapIntervals': large_events,
        'cumulativeMaxMsAtEnd': pairs[-1][1]['sourceIntervalMaxMs'],
        'queueDropsBySource': dict(drops),
        'largeGapsWithin1SecondOfVideoQueueDrop': correlated_gaps,
        'keyframeRequests': sum(r['kind'] == 'keyframe-request' for r in included),
        'nativeRecoveryOrStopEvents': sum('invalid' in r for r in included),
        'nativeTelemetrySamples': len(telemetry),
        'nativeTelemetryMaxGapMs': max((b['time_ms'] - a['time_ms'] for a, b in zip(telemetry, telemetry[1:])), default=None),
        'nativeCounterRatesPerSecond': counter_rates,
        'nativeMetrics': {key: trial.summary([r[key] for r in telemetry if key in r]) for key in trial.NATIVE_KEYS},
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--qt', type=Path, default=ROOT / '.runtime/hillclimb/qt-presentation.jsonl')
    parser.add_argument('--native', type=Path, default=ROOT / '.runtime/config/OpenNOW/diagnostics/native-streamer.log')
    parser.add_argument('--since', type=timestamp, required=True)
    parser.add_argument('--until', type=timestamp, required=True)
    parser.add_argument('--warmup-seconds', type=float, default=120)
    parser.add_argument('--minimum-seconds', type=float, default=30)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.until <= args.since or args.warmup_seconds < 0 or args.minimum_seconds < 0:
        parser.error('ordered time bounds and nonnegative duration limits required')
    lines, qt_input = snapshot_lines(args.qt)
    native_lines, native_input = snapshot_lines(args.native)
    native = [r for line in native_lines if (r := trial.parse_native(line)) is not None
              and args.since <= r['time_ms'] <= args.until]
    native.sort(key=lambda r: r['time_ms'])
    segments, current = [], []
    exclusions = Counter()
    for line in lines:
        try:
            raw = json.loads(line)
        except ValueError:
            exclusions['malformed_json'] += 1
            if current:
                segments.append(current)
                current = []
            continue
        if not args.since <= raw.get('wallTimeMs', 0) <= args.until:
            continue
        try:
            row = trial.parse_qt(line)
        except trial.InvalidTrial as error:
            exclusions[str(error)] += 1
            if current:
                segments.append(current)
                current = []
            continue
        row['lateThresholdMs'] = raw.get('lateThresholdMs')
        if row['gated'] or not row['runtimeRunning'] or not row['presentationAllowed']:
            exclusions['not_playing'] += 1
            if current:
                segments.append(current)
                current = []
            continue
        if current:
            previous = current[-1]
            broken = trial.continuity(previous, row)
            if previous['observerId'] != row['observerId']:
                broken = 'changed_observer'
            if (previous['videoWidth'], previous['videoHeight'], previous['displayHz']) != (row['videoWidth'], row['videoHeight'], row['displayHz']):
                broken = 'changed_video_profile'
            if abs((row['wallTimeMs'] - previous['wallTimeMs']) - (row['monotonicNs'] - previous['monotonicNs']) / 1e6) > 50:
                broken = 'wall_clock_discontinuity'
            if broken:
                exclusions[broken] += 1
                segments.append(current)
                current = []
        current.append(row)
    if current:
        segments.append(current)
    report = {'schema': 1, 'method': 'Historical triage; contiguous counter deltas; 1-ms Qt histogram upper bounds.',
              'limitations': ['Not matched gameplay or physical scanout evidence.',
                              'Mailbox replacements are not exact visible frame losses.',
                              'Current preferences do not prove historical negotiation.'],
              'inputs': [qt_input, native_input], 'warmupSeconds': args.warmup_seconds,
              'excludedSamplesOrBoundaries': dict(exclusions), 'segments': [],
              'nativeLifecycleEvents': [{'timeMs': r['time_ms'], 'kind': r['kind'], 'invalid': r.get('invalid')}
                                        for r in native if r['kind'] in ('lifecycle', 'recovery-required')]}
    for rows in segments:
        pairs = list(zip(rows, rows[1:]))
        if not pairs or (rows[-1]['monotonicNs'] - rows[0]['monotonicNs']) / 1e9 < args.minimum_seconds:
            continue
        after = rows[0]['wallTimeMs'] + args.warmup_seconds * 1000
        active = [(a, b) for a, b in pairs if a['visible'] and b['visible'] and a['windowActive'] and b['windowActive']]
        steady = [(a, b) for a, b in active if a['wallTimeMs'] >= after]
        report['segments'].append({'identity': {k: rows[0][k] for k in ('observerId', 'epoch', 'presentationGeneration', 'gateEpoch')},
                                  'video': [rows[0]['videoWidth'], rows[0]['videoHeight'], rows[0]['displayHz']],
                                  'allPlaying': summarize_pairs(pairs, native),
                                  'activeWindow': summarize_pairs(active, native),
                                  'activeAfterWarmup': summarize_pairs(steady, native)})
    os.umask(0o077)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open('x') as target:
        json.dump(report, target, indent=2, allow_nan=False)
        target.write('\n')
    print(json.dumps({'output': str(args.output), 'segments': len(report['segments']),
                      'excludedSamplesOrBoundaries': dict(exclusions)}))


if __name__ == '__main__':
    main()
