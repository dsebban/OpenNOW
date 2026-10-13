#!/usr/bin/env python3
"""Read additive Qt census data after scoring; never changes the trial score."""
import argparse
import json
import math
import os
from pathlib import Path

RUNTIME_COUNTERS = (
    'runtimeAcquireCallsTotal', 'runtimeAcquireSuccessTotal', 'runtimeAcquireEmptyTotal',
    'runtimeAcquireStaleTotal', 'runtimeAcquireErrorTotal', 'runtimeRecordCallsTotal',
    'runtimeRecordSuccessTotal', 'runtimeRecordNoFrameTotal', 'runtimeRecordStaleTotal',
    'runtimeRecordErrorTotal',
)
RENDER_COUNTERS = (
    'renderPrepareTotal', 'renderTextureImportsTotal', 'renderCallTotal',
    'renderDrawsIssuedTotal', 'renderFreshSubmitsTotal',
)
COUNTERS = (
    'sourceSwapsTotal', 'windowSwapsTotal', 'windowSwapsWithoutFreshSourceTotal',
    'windowIntervalSamplesTotal', 'windowIntervalOverflowTotal',
    'notificationEnqueuedTotal', 'notificationCoalescedTotal', 'notificationDeliveredTotal',
    'notificationDrainSamplesTotal', 'notificationOldestOverflowTotal', 'notificationLatestOverflowTotal',
    'ptsDiscontinuitiesTotal',
) + RUNTIME_COUNTERS + RENDER_COUNTERS
EPOCHS = ('observerId', 'epoch', 'gateEpoch', 'presentationGeneration', 'notificationTimingEpoch',
          'runtimeRecordStageGeneration')
HISTOGRAMS = ('windowIntervalHistogramMs', 'notificationOldestHistogramMs', 'notificationLatestHistogramMs')
STATES = ('gated', 'visible', 'windowActive', 'runtimeRunning', 'presentationAllowed', 'frameGeneration', 'notificationStatsAvailable')
MAX_BYTES = 64 * 1024 * 1024
MAX_LINE = 128 * 1024
MAX_ROWS = 10000


class CensusError(Exception):
    pass


def integer(value):
    return (isinstance(value, int) and not isinstance(value, bool) and 0 <= value <= 2**64 - 1
            or isinstance(value, float) and math.isfinite(value) and value.is_integer() and 0 <= value < 2**53)


def parse_sample(raw):
    if not isinstance(raw, dict) or raw.get('event') != 'qt-presentation':
        raise CensusError('unexpected_sample')
    required = COUNTERS + EPOCHS + HISTOGRAMS + STATES + ('monotonicNs', 'wallTimeMs', 'sinceLastSourceSwapMs')
    if any(key not in raw for key in required):
        raise CensusError('missing_census_fields_old_build_or_incomplete_sample')
    result = {}
    for key in COUNTERS + EPOCHS + ('monotonicNs', 'wallTimeMs'):
        if not integer(raw[key]):
            raise CensusError('invalid_numeric_' + key)
        result[key] = int(raw[key])
    for key in HISTOGRAMS:
        values = raw[key]
        if not isinstance(values, list) or len(values) != 512 or not all(integer(value) for value in values):
            raise CensusError('invalid_histogram_' + key)
        result[key] = [int(value) for value in values]
    for key in STATES:
        if not isinstance(raw[key], bool):
            raise CensusError('invalid_state_' + key)
        result[key] = raw[key]
    age = raw['sinceLastSourceSwapMs']
    if not isinstance(age, (int, float)) or isinstance(age, bool) or not math.isfinite(age) or age < 0:
        raise CensusError('invalid_fresh_source_age')
    result['sinceLastSourceSwapMs'] = age
    if result['runtimeRecordStageGeneration'] != result['presentationGeneration']:
        raise CensusError('runtime_record_generation_mismatch')
    if result['runtimeAcquireCallsTotal'] != sum(result[key] for key in (
            'runtimeAcquireSuccessTotal', 'runtimeAcquireEmptyTotal', 'runtimeAcquireStaleTotal', 'runtimeAcquireErrorTotal')):
        raise CensusError('runtime_acquire_outcome_mismatch')
    if result['runtimeRecordCallsTotal'] != sum(result[key] for key in (
            'runtimeRecordSuccessTotal', 'runtimeRecordNoFrameTotal', 'runtimeRecordStaleTotal', 'runtimeRecordErrorTotal')):
        raise CensusError('runtime_record_outcome_mismatch')
    if result['runtimeRecordCallsTotal'] != result['runtimeAcquireSuccessTotal']:
        raise CensusError('runtime_acquire_record_transaction_mismatch')
    if (result['renderDrawsIssuedTotal'] > result['renderCallTotal']
            or result['renderFreshSubmitsTotal'] > result['renderCallTotal']):
        raise CensusError('render_call_outcome_mismatch')
    return result


def quantile(histogram, overflow, fraction):
    target = math.ceil((sum(histogram) + overflow) * fraction)
    if target == 0:
        return None
    count = 0
    for index, value in enumerate(histogram):
        count += value
        if count >= target:
            return index + 1
    return None


def summarize_window(rows, window):
    expected_epoch = window['timing_epoch']
    expected_generation = window['presentation_generation']
    start, end = window['start_ms'], window['end_ms']
    if not all(integer(value) for value in (expected_epoch, expected_generation, start, end)) or end <= start:
        raise CensusError('invalid_trial_window')
    eligible = [row for row in rows if integer(row.get('wallTimeMs')) and start - 25 <= row['wallTimeMs'] <= end + 25]
    observers = set()
    for row in eligible:
        if not integer(row.get('observerId')):
            raise CensusError('invalid_numeric_observerId')
        if (row.get('epoch') == expected_epoch and row.get('presentationGeneration') == expected_generation
                and row.get('visible') is True and row.get('gated') is False):
            observers.add(row['observerId'])
    if len(observers) != 1:
        raise CensusError('ambiguous_or_missing_observer')
    selected_observer = next(iter(observers))
    samples = sorted((parse_sample(row) for row in eligible if row.get('observerId') == selected_observer), key=lambda row: row['wallTimeMs'])
    if len(samples) < 2 or abs(samples[0]['wallTimeMs'] - start) > 25 or abs(samples[-1]['wallTimeMs'] - end) > 25:
        raise CensusError('window_boundary_missing')
    first, last = samples[0], samples[-1]
    if first['epoch'] != expected_epoch or first['presentationGeneration'] != expected_generation:
        raise CensusError('trial_epoch_mismatch')
    for index, sample in enumerate(samples):
        if sample['gated'] or not sample['visible'] or not sample['windowActive'] or not sample['runtimeRunning'] or not sample['presentationAllowed']:
            raise CensusError('inactive_presentation')
        if sample['frameGeneration'] or not sample['notificationStatsAvailable']:
            raise CensusError('unsupported_or_unavailable_census_state')
        if sample['sinceLastSourceSwapMs'] > 1000:
            raise CensusError('fresh_source_stalled')
        if any(sample[key] != first[key] for key in EPOCHS):
            raise CensusError('epoch_or_observer_changed')
        if sample['ptsDiscontinuitiesTotal'] != first['ptsDiscontinuitiesTotal']:
            raise CensusError('source_pts_discontinuity')
        if index:
            prior = samples[index - 1]
            interval = (sample['monotonicNs'] - prior['monotonicNs']) / 1e9
            wall_interval = (sample['wallTimeMs'] - prior['wallTimeMs']) / 1000
            if not 0 < interval <= 3 or not 0 < wall_interval <= 3 or abs(interval - wall_interval) > .1:
                raise CensusError('sample_gap_or_clock_change')
            if any(sample[key] < prior[key] for key in COUNTERS):
                raise CensusError('counter_rollback')
            if any(any(b < a for a, b in zip(prior[key], sample[key])) for key in HISTOGRAMS):
                raise CensusError('histogram_rollback')
            if sample['sourceSwapsTotal'] == prior['sourceSwapsTotal']:
                raise CensusError('fresh_source_no_progress')
    duration = (last['monotonicNs'] - first['monotonicNs']) / 1e9
    changes = {key: last[key] - first[key] for key in COUNTERS}
    histograms = {key: [b - a for a, b in zip(first[key], last[key])] for key in HISTOGRAMS}
    for histogram, overflow, total in (
        ('windowIntervalHistogramMs', 'windowIntervalOverflowTotal', 'windowIntervalSamplesTotal'),
        ('notificationOldestHistogramMs', 'notificationOldestOverflowTotal', 'notificationDrainSamplesTotal'),
        ('notificationLatestHistogramMs', 'notificationLatestOverflowTotal', 'notificationDrainSamplesTotal'),
    ):
        if sum(histograms[histogram]) + changes[overflow] != changes[total]:
            raise CensusError('histogram_counter_mismatch')
    skew = changes['windowSwapsTotal'] - changes['windowSwapsWithoutFreshSourceTotal'] - changes['sourceSwapsTotal']
    if abs(skew) > 2:
        raise CensusError('fresh_window_counter_mismatch')
    if not changes['windowSwapsTotal'] or changes['windowSwapsWithoutFreshSourceTotal'] > changes['windowSwapsTotal']:
        raise CensusError('invalid_window_census')
    if changes['notificationDeliveredTotal'] > changes['notificationDrainSamplesTotal'] + 1:
        raise CensusError('invalid_notification_census')
    output = {
        'startMs': first['wallTimeMs'], 'endMs': last['wallTimeMs'], 'seconds': duration,
        'samples': len(samples), 'observerId': first['observerId'], 'timingEpoch': first['epoch'],
        'presentationGeneration': first['presentationGeneration'], 'notificationTimingEpoch': first['notificationTimingEpoch'],
        'runtimeRecordStageGeneration': first['runtimeRecordStageGeneration'],
        'windowFps': changes['windowSwapsTotal'] / duration, 'freshSourceFps': changes['sourceSwapsTotal'] / duration,
        'withoutFreshSourceFraction': changes['windowSwapsWithoutFreshSourceTotal'] / changes['windowSwapsTotal'],
        'notificationEnqueuePerSecond': changes['notificationEnqueuedTotal'] / duration,
        'notificationCoalescedPerSecond': changes['notificationCoalescedTotal'] / duration,
        'notificationDrainedPerSecond': changes['notificationDrainSamplesTotal'] / duration,
        'notificationDeliveredPerSecond': changes['notificationDeliveredTotal'] / duration,
        'freshWindowBoundarySkewFrames': skew, 'counterDeltas': changes,
        'stageRatesPerSecond': {key: changes[key] / duration for key in RUNTIME_COUNTERS + RENDER_COUNTERS},
    }
    for name, key, overflow in (
        ('oldestDrain', 'notificationOldestHistogramMs', 'notificationOldestOverflowTotal'),
        ('latestDrain', 'notificationLatestHistogramMs', 'notificationLatestOverflowTotal'),
    ):
        output[name + 'P95UpperMs'] = quantile(histograms[key], changes[overflow], .95)
        output[name + 'P99UpperMs'] = quantile(histograms[key], changes[overflow], .99)
    return output


def trial_provenance(trial):
    if not isinstance(trial, dict) or trial.get('status') not in ('valid', 'invalid', 'interrupted', 'calibration'):
        raise CensusError('missing_or_invalid_trial_status')
    method = trial.get('method_version')
    windows = trial.get('windows')
    if not integer(method) or not isinstance(windows, list) or len(windows) > 16:
        raise CensusError('missing_or_invalid_trial_provenance')
    requested = trial.get('spec', {}).get('windows') if isinstance(trial.get('spec'), dict) else None
    return {'trialStatus': trial['status'], 'trialMethodVersion': int(method),
            'trialWindowCount': len(windows),
            'trialRequestedWindowCount': int(requested) if integer(requested) and 1 <= requested <= 16 else None}


def analyze(trial, rows):
    provenance = trial_provenance(trial)
    windows = trial.get('windows')
    if not isinstance(windows, list) or not 1 <= len(windows) <= 16:
        raise CensusError('missing_or_unbounded_trial_windows')
    results = []
    observer = None
    for index, window in enumerate(windows, 1):
        if not isinstance(window, dict) or any(key not in window for key in ('start_ms', 'end_ms', 'timing_epoch', 'presentation_generation')):
            raise CensusError('invalid_trial_window')
        result = summarize_window(rows, window)
        if observer is not None and result['observerId'] != observer:
            raise CensusError('observer_changed_between_windows')
        observer = result['observerId']
        results.append({'window': index, **result})
    return {'schema': 2, 'status': 'complete', 'censusStatus': 'complete', **provenance, 'windows': results, 'limits': [
        'Ancillary numeric diagnostics only. The existing live-trial score is unchanged.',
        'Census completion describes only the retained diagnostic windows. trialStatus and trialWindowCount preserve the original scoring status and completeness.',
        'Runtime acquire/record counts cover all observers and commit complete attempts atomically. Render counts belong to the selected observer and reset with its timing epoch.',
        'Render stage counters count actual calls even while the timing gate is set. DrawsIssued counts QRhi draw commands, not visible pixels or scanout.',
        'FreshSubmits counts the existing fresh-mark requests even if no draw command was issued. It does not change the primary metric.',
        'Imports, draws, fresh marks and swaps are separate timing boundaries; exact cross-layer equality is not required.',
        'Window callbacks without fresh sources are not necessarily repeated-video scanout.',
        'Delivery/drainage concern GUI notifications, not decoded frame delivery or presentation. Drain delay is measured when GUI consumes the pending batch.',
        'Oldest/latest pending notification delays are not exact newest GPU-frame ages.',
        'Fresh and all-window updates use separate locks; each snapshot may differ by one frame, so interval deltas allow two-frame boundary skew.',
        'Quantiles are 1 ms bucket upper bounds. Samples >=512 ms are overflow; quantiles in overflow return null.',
    ]}


def read_rows(path):
    if path.stat().st_size > MAX_BYTES:
        raise CensusError('telemetry_file_too_large')
    result = []
    with path.open() as file:
        while line := file.readline(MAX_LINE + 1):
            if len(line) > MAX_LINE or len(result) >= MAX_ROWS:
                raise CensusError('telemetry_capacity_exceeded')
            try:
                row = json.loads(line)
            except (ValueError, RecursionError):
                raise CensusError('invalid_qt_json') from None
            if not isinstance(row, dict) or row.get('event') != 'qt-presentation' or not integer(row.get('wallTimeMs')):
                raise CensusError('invalid_qt_record')
            allowed = COUNTERS + EPOCHS + HISTOGRAMS + STATES + ('event', 'monotonicNs', 'wallTimeMs', 'sinceLastSourceSwapMs')
            result.append({key: row[key] for key in allowed if key in row})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--trial', required=True, type=Path)
    parser.add_argument('--telemetry', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    os.umask(0o077)
    provenance = {}
    try:
        if args.trial.stat().st_size > 10 * 1024 * 1024:
            raise CensusError('trial_file_too_large')
        trial = json.loads(args.trial.read_text())
        provenance = trial_provenance(trial)
        result = analyze(trial, read_rows(args.telemetry))
    except (CensusError, OSError, ValueError, RecursionError) as error:
        result = {'schema': 2, 'status': 'unavailable', 'censusStatus': 'unavailable', **provenance,
                  'reason': str(error) if isinstance(error, CensusError) else 'input_unavailable_or_invalid', 'windows': []}
    args.output.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, 'O_NOFOLLOW', 0)
    with os.fdopen(os.open(args.output, flags, 0o600), 'w') as file:
        json.dump(result, file, separators=(',', ':'), allow_nan=False)
        file.write('\n')
    print(json.dumps({'censusStatus': result['status'], 'trialStatus': result.get('trialStatus'),
                      'trialWindowCount': result.get('trialWindowCount'), 'windows': len(result['windows']), 'reason': result.get('reason')}))
    return 0 if result['status'] == 'complete' else 1


if __name__ == '__main__':
    raise SystemExit(main())
