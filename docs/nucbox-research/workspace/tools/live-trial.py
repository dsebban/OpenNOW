#!/usr/bin/env python3
"""Measure a real, already running OpenNOW stream without account access."""
import argparse
import collections
import dataclasses
import hashlib
import json
import math
import os
import pathlib
import re
import statistics
import subprocess
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
COUNTERS = (
    'sourceSwapsTotal', 'sourceIntervalSamplesTotal', 'sourceIntervalOverflowTotal',
    'lateSourceIntervalsTotal', 'submitSamplesTotal', 'submitOverflowTotal', 'ptsDiscontinuitiesTotal',
)
HISTOGRAMS = ('sourceIntervalHistogramMs', 'submitHistogramMs')
NATIVE_KEYS = (
    'framesPerSecond', 'pingMs', 'jitterMs', 'packetLossPercent', 'bitrateMbps',
    'decoderResidenceMs', 'decodeTimeMs', 'decodeTimings.call.p95',
    'decodeTimings.residence.p95', 'decodeTimings.epoch', 'decodeTimings.outputsTotal',
    'decodeTimings.submissionsTotal', 'frameStageTimings.assembledFramesTotal',
)
SAFE_LABEL = re.compile(r'[A-Za-z0-9_-]{1,80}\Z')


class InvalidTrial(Exception):
    pass


@dataclasses.dataclass(frozen=True)
class TrialSpec:
    label: str
    profile: str
    telemetry: pathlib.Path
    warmup: float = 20
    seconds: float = 60
    windows: int = 3
    minimum_fps: float = 57
    deadline: float = 300
    display_hz: int = 60
    video_width: int = 1920
    video_height: int = 1080


def number(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def integer(value):
    if isinstance(value, bool):
        return False
    if isinstance(value, int):
        return 0 <= value <= 2**64 - 1
    return isinstance(value, float) and math.isfinite(value) and value.is_integer() and 0 <= value < 2**53


def parse_qt(line):
    try:
        raw = json.loads(line)
    except (json.JSONDecodeError, UnicodeError):
        raise InvalidTrial('malformed_qt_sample') from None
    if not isinstance(raw, dict) or raw.get('event') != 'qt-presentation':
        raise InvalidTrial('unexpected_qt_sample')
    sample = {}
    for key in ('monotonicNs', 'observerId', 'epoch', 'gateEpoch', 'presentationGeneration') + COUNTERS:
        if not integer(raw.get(key)):
            raise InvalidTrial('invalid_qt_counter_' + key)
        sample[key] = int(raw[key])
    for key in HISTOGRAMS:
        value = raw.get(key)
        if not isinstance(value, list) or len(value) != 512 or not all(integer(v) for v in value):
            raise InvalidTrial('invalid_qt_histogram_' + key)
        sample[key] = [int(item) for item in value]
    for key in ('gated', 'frameGeneration', 'visible', 'windowActive', 'fullscreen',
                'runtimeRunning', 'presentationAllowed'):
        if not isinstance(raw.get(key), bool):
            raise InvalidTrial('invalid_qt_state_' + key)
        sample[key] = raw[key]
    for key in ('displayHz', 'sourceIntervalMaxMs', 'submitMaxMs'):
        if not number(raw.get(key)) or raw[key] < 0:
            raise InvalidTrial('invalid_qt_number_' + key)
        sample[key] = raw[key]
    for key in ('relativeMediaLagMs', 'wallTimeMs', 'sinceLastSourceSwapMs'):
        if number(raw.get(key)):
            sample[key] = raw[key]
    for key in ('videoWidth', 'videoHeight'):
        if not integer(raw.get(key)):
            raise InvalidTrial('invalid_qt_dimension_' + key)
        sample[key] = int(raw[key])
    return sample


def healthy(sample, display_hz=60, video_width=1920, video_height=1080):
    if sample['gated'] or not sample['presentationAllowed']:
        return 'presentation_gated'
    if not sample['runtimeRunning']:
        return 'runtime_stopped'
    if sample['frameGeneration']:
        return 'frame_generation_enabled'
    if not sample['visible'] or not sample['windowActive']:
        return 'window_not_active'
    if not display_hz - 1 <= sample['displayHz'] <= display_hz + 1:
        return 'display_refresh_mismatch'
    if sample['videoWidth'] != video_width or sample['videoHeight'] != video_height:
        return 'video_dimensions_mismatch'
    if sample.get('sinceLastSourceSwapMs', 1001) > 1000:
        return 'video_stalled'
    return None


def continuity(previous, current):
    for key in ('epoch', 'gateEpoch', 'presentationGeneration', 'fullscreen'):
        if previous[key] != current[key]:
            return 'changed_' + key
    elapsed = (current['monotonicNs'] - previous['monotonicNs']) / 1e9
    if elapsed <= 0 or elapsed > 3:
        return 'qt_sample_gap_or_clock_reset'
    if current['ptsDiscontinuitiesTotal'] != previous['ptsDiscontinuitiesTotal']:
        return 'source_pts_discontinuity'
    for key in COUNTERS:
        if current[key] < previous[key]:
            return 'rollback_' + key
    for key in HISTOGRAMS:
        if any(b < a for a, b in zip(previous[key], current[key])):
            return 'rollback_' + key
    return None


class Tail:
    def __init__(self, path):
        self.path = path
        self.file = None
        self.identity = None
        self.partial = ''
        if path.exists():
            self.open(at_end=True)

    def open(self, at_end):
        self.file = self.path.open(encoding='utf8', errors='replace')
        status = os.fstat(self.file.fileno())
        self.identity = (status.st_dev, status.st_ino)
        if at_end:
            self.file.seek(0, os.SEEK_END)

    def lines(self):
        if not self.path.exists():
            return []
        if self.file is None:
            self.open(at_end=False)
        status = self.path.stat()
        if (status.st_dev, status.st_ino) != self.identity or status.st_size < self.file.tell():
            raise InvalidTrial('telemetry_file_rotated_or_truncated')
        data = self.file.read(1024 * 1024)
        if not data:
            return []
        chunks = (self.partial + data).split('\n')
        self.partial = chunks.pop()
        if len(self.partial) > 128 * 1024 or any(len(line) > 128 * 1024 for line in chunks):
            raise InvalidTrial('oversized_telemetry_line')
        return [line for line in chunks if line]

    def close(self):
        if self.file:
            self.file.close()


def parse_native(line):
    match = re.match(r'^(\d+) ', line)
    if not match:
        return None
    fields = dict(re.findall(r'([A-Za-z][A-Za-z0-9_.]*)=([^ ]+)', line))
    row = {'time_ms': int(match[1])}
    if fields.get('type') == 'telemetry':
        row['kind'] = 'telemetry'
        for key in NATIVE_KEYS:
            value = fields.get(key, '')
            if re.fullmatch(r'-?\d+(?:\.\d+)?', value):
                parsed = float(value)
                if math.isfinite(parsed):
                    row[key] = parsed
        if fields.get('transportFrameProgressStalled') == 'true':
            row['invalid'] = 'transport_stalled'
        if fields.get('decodeProgressStage') == 'recovery-required':
            row['invalid'] = 'decoder_recovery'
        return row
    event = fields.get('event')
    if event == 'queue-dropped' and fields.get('unit') == 'frames':
        media = fields.get('media', '')
        if re.fullmatch(r'[a-z0-9-]{1,40}', media) and fields.get('count', '').isdigit():
            return {**row, 'kind': 'drop', 'media': media, 'count': int(fields['count'])}
    if event in ('keyframe-request', 'recovery-required'):
        return {**row, 'kind': event, **({'invalid': 'native_recovery'} if event == 'recovery-required' else {})}
    if fields.get('status') in ('stopped', 'error') or fields.get('type') == 'error':
        return {**row, 'kind': 'lifecycle', 'invalid': 'native_stopped_or_error'}
    if 'trace_queue_dropped=' in line:
        return {**row, 'kind': 'diagnostics', 'invalid': 'native_diagnostics_overflow'}
    return None


def summary(values):
    if not values:
        return None
    ordered = sorted(values)
    return {'samples': len(ordered), 'median': statistics.median(ordered),
            'p95': ordered[max(0, math.ceil(.95 * len(ordered)) - 1)], 'max': ordered[-1]}


def percentile_bucket(histogram, percentile, overflow=0):
    target = math.ceil((sum(histogram) + overflow) * percentile)
    if target <= 0:
        return None
    cumulative = 0
    for bucket, count in enumerate(histogram):
        cumulative += count
        if cumulative >= target:
            return bucket + 1
    return None


def window_result(first, last, qt_samples, native_rows, start_ms, end_ms, spec):
    duration = (last['monotonicNs'] - first['monotonicNs']) / 1e9
    counters = {key: last[key] - first[key] for key in COUNTERS}
    histograms = {key: [b-a for a, b in zip(first[key], last[key])] for key in HISTOGRAMS}
    if sum(histograms['sourceIntervalHistogramMs']) + counters['sourceIntervalOverflowTotal'] != counters['sourceIntervalSamplesTotal']:
        raise InvalidTrial('source_histogram_count_mismatch')
    if sum(histograms['submitHistogramMs']) + counters['submitOverflowTotal'] != counters['submitSamplesTotal']:
        raise InvalidTrial('submit_histogram_count_mismatch')
    if not counters['sourceIntervalSamplesTotal'] or not counters['submitSamplesTotal']:
        raise InvalidTrial('no_presented_video')
    fps = counters['sourceSwapsTotal'] / duration
    if not 0 < fps <= 65:
        raise InvalidTrial('source_fps_outside_measurement_bounds')
    quality_failures = ['presented_fps_below_57'] if fps < spec.minimum_fps else []
    rows = [row for row in native_rows if start_ms < row['time_ms'] <= end_ms]
    telemetry = [row for row in rows if row['kind'] == 'telemetry']
    if not telemetry or any(end_ms-row['time_ms'] > 25_000 for row in telemetry[-1:]):
        raise InvalidTrial('missing_native_telemetry')
    stamps = [start_ms] + [row['time_ms'] for row in telemetry] + [end_ms]
    if any(b-a > 25_000 for a, b in zip(stamps, stamps[1:])):
        raise InvalidTrial('native_sample_gap')
    samples = {key: summary([r[key] for r in telemetry if key in r]) for key in NATIVE_KEYS}
    for key in ('pingMs', 'framesPerSecond', 'packetLossPercent', 'decodeTimings.residence.p95'):
        if samples[key] is None:
            raise InvalidTrial('missing_native_metric_' + key)
    epochs = {row['decodeTimings.epoch'] for row in telemetry if 'decodeTimings.epoch' in row}
    if len(epochs) != 1:
        raise InvalidTrial('changed_or_missing_decode_epoch')
    for key in ('decodeTimings.outputsTotal', 'decodeTimings.submissionsTotal', 'frameStageTimings.assembledFramesTotal'):
        values = [r[key] for r in telemetry if key in r]
        if len(values) < 2 or any(b < a for a, b in zip(values, values[1:])):
            raise InvalidTrial('native_counter_reset_or_missing_' + key)
    invalid = next((r['invalid'] for r in rows if 'invalid' in r), None)
    if invalid:
        raise InvalidTrial(invalid)
    drops = collections.Counter()
    for row in rows:
        if row['kind'] == 'drop':
            drops[row['media']] += row['count']
    decoded_rows = [r for r in telemetry if 'decodeTimings.outputsTotal' in r]
    decoded_duration = (decoded_rows[-1]['time_ms']-decoded_rows[0]['time_ms'])/1000
    if decoded_duration <= 0:
        raise InvalidTrial('invalid_native_counter_duration')
    decoded_fps = (decoded_rows[-1]['decodeTimings.outputsTotal']-decoded_rows[0]['decodeTimings.outputsTotal'])/decoded_duration
    incoming_rows = [r for r in telemetry if 'frameStageTimings.assembledFramesTotal' in r]
    incoming_duration = (incoming_rows[-1]['time_ms']-incoming_rows[0]['time_ms'])/1000
    if incoming_duration <= 0:
        raise InvalidTrial('invalid_incoming_counter_duration')
    incoming_fps = (incoming_rows[-1]['frameStageTimings.assembledFramesTotal']-incoming_rows[0]['frameStageTimings.assembledFramesTotal'])/incoming_duration
    validity_failures = ['incoming_assembled_fps_below_57'] if incoming_fps < 57 else []
    lag_values = [s['relativeMediaLagMs'] for s in qt_samples if 'relativeMediaLagMs' in s]
    histogram_max = max(index+1 for index, count in enumerate(histograms['sourceIntervalHistogramMs']) if count) if any(histograms['sourceIntervalHistogramMs']) else None
    return {'duration_seconds': duration, 'qt_samples': len(qt_samples),
            'timing_epoch': first['epoch'], 'presentation_generation': first['presentationGeneration'],
            'start_ms': start_ms, 'end_ms': end_ms,
            'quality_failures': quality_failures, 'validity_failures': validity_failures,
            'source_fps': fps, 'decoded_fps': decoded_fps, 'incoming_assembled_fps': incoming_fps, 'late_interval_rate': counters['lateSourceIntervalsTotal'] / counters['sourceIntervalSamplesTotal'],
            'late_intervals': counters['lateSourceIntervalsTotal'], 'counters': counters,
            'histograms': histograms,
            'source_interval_p95_upper_ms': percentile_bucket(histograms['sourceIntervalHistogramMs'], .95, counters['sourceIntervalOverflowTotal']),
            'source_interval_p99_upper_ms': percentile_bucket(histograms['sourceIntervalHistogramMs'], .99, counters['sourceIntervalOverflowTotal']),
            'source_interval_max_upper_ms': last['sourceIntervalMaxMs'] if counters['sourceIntervalOverflowTotal'] else histogram_max,
            'source_interval_overflow_per_second': counters['sourceIntervalOverflowTotal']/duration,
            'relative_media_lag_growth_ms': lag_values[-1]-lag_values[0] if len(lag_values) >= 2 else None,
            'submit_p95_upper_ms': percentile_bucket(histograms['submitHistogramMs'], .95, counters['submitOverflowTotal']),
            'relative_media_lag_ms': summary([s['relativeMediaLagMs'] for s in qt_samples if 'relativeMediaLagMs' in s]),
            'qt_process_cpu_percent': summary([s['qtProcessCpuPercent'] for s in qt_samples if 'qtProcessCpuPercent' in s]),
            'queue_drops_by_source': dict(drops),
            'queue_drops_per_second': {k: v/duration for k, v in drops.items()},
            'keyframe_requests': sum(r['kind'] == 'keyframe-request' for r in rows),
            'native_metrics': samples}


def build_identity():
    source = ROOT / 'vendor/OpenNOW'
    def git(*args):
        result = subprocess.run(['git', '-C', str(source), *args], capture_output=True, check=True)
        return result.stdout
    def digest(path):
        result = hashlib.sha256()
        with path.open('rb') as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b''):
                result.update(block)
        return result.hexdigest()
    return {'git_sha': git('rev-parse', 'HEAD').decode().strip(),
            'tracked_diff_sha256': hashlib.sha256(git('diff', 'HEAD', '--')).hexdigest(),
            'binary_sha256': digest(ROOT / 'build/opennow-qt/opennow-qt'),
            'streamer_ffi_sha256': digest(ROOT / 'build/opennow-qt/libopennow_streamer_ffi.so'),
            'benchmark_script_sha256': digest(pathlib.Path(__file__).resolve())}


def write_private(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + '.tmp')
    with temporary.open('w') as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write('\n')
    temporary.chmod(0o600)
    temporary.replace(path)


def process_cpu(pid):
    if pid is None:
        return None
    fields = pathlib.Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
    return (int(fields[11]) + int(fields[12])) / os.sysconf('SC_CLK_TCK'), int(fields[19])


def measure(args):
    spec = TrialSpec(args.label, args.profile, args.telemetry.resolve(), args.warmup,
                     args.seconds, args.windows, args.minimum_fps, args.deadline, args.display_hz,
                     args.video_width, args.video_height)
    result = {'schema': 1, 'method_version': 4, 'label': spec.label, 'profile': spec.profile,
              'profile_verification': 'operator-confirmed; Qt validates display and presentation state',
              'variant': args.variant, 'status': 'invalid', 'invalid_reasons': [], 'quality_failures': [], 'windows': [], 'build': build_identity(),
              'spec': {k: v for k, v in dataclasses.asdict(spec).items() if k != 'telemetry'},
              'measurement': 'Qt fresh video submit/swap callback timing; not physical scanout or input-to-photon',
              'primary_metric': 'fraction of fresh source intervals strictly greater than 25 ms, at matched source FPS',
              'limits': ['Negotiated codec/backend/workload are operator-confirmed.', 'Qt callback overflow is not independently counted by this observer.', 'Histogram durations >=512 ms appear only in overflow counters; tail quantiles beyond that range are null.', 'A window with interval overflow uses the epoch maximum as a conservative maximum bound.', 'CPU distributions use collector-read samples. Buffered Qt lines can distort them. These distributions are descriptive and are not used to accept a candidate.'],
              'native_timing': 'Native event timestamp windows. Queue reports flush at most once per second; boundary attribution may shift by one second. Native p95 values are rolling telemetry summaries, not combined frame percentiles.',
              'created_ms': int(time.time()*1000)}
    qt_tail = Tail(spec.telemetry)
    native_tail = Tail(args.native_log.resolve())
    native_rows = []
    native_previous = None
    cpu_previous = None
    cpu_time = None
    previous = None
    selected_observer = None
    first = None
    window_samples = []
    state = 'waiting'
    started = time.monotonic()
    last_received = started
    warm_start = None
    start_ms = None
    try:
        print('Waiting for fresh Qt and native telemetry.', flush=True)
        while time.monotonic()-started <= spec.deadline:
            for line in native_tail.lines():
                row = parse_native(line)
                if row:
                    if state == 'measuring' and row['kind'] == 'telemetry':
                        if native_previous and row.get('decodeTimings.epoch') != native_previous.get('decodeTimings.epoch'):
                            raise InvalidTrial('changed_decode_epoch')
                        for key in ('decodeTimings.outputsTotal', 'decodeTimings.submissionsTotal'):
                            if native_previous and key in row and key in native_previous and row[key] < native_previous[key]:
                                raise InvalidTrial('native_counter_rollback')
                    if row['kind'] == 'telemetry':
                        native_previous = row
                    native_rows.append(row)
            for line in qt_tail.lines():
                current = parse_qt(line)
                if selected_observer is None:
                    if healthy(current, spec.display_hz, spec.video_width, spec.video_height):
                        continue
                    selected_observer = current['observerId']
                if current['observerId'] != selected_observer:
                    continue
                last_received = time.monotonic()
                cpu_now = process_cpu(args.pid)
                if cpu_now is not None and cpu_previous is not None:
                    if cpu_now[1] != cpu_previous[1]:
                        raise InvalidTrial('qt_process_restarted')
                    current['qtProcessCpuPercent'] = 100*(cpu_now[0]-cpu_previous[0])/(last_received-cpu_time)
                cpu_previous, cpu_time = cpu_now, last_received
                wall_ms = current.get('wallTimeMs', int(time.time()*1000))
                if abs(wall_ms-time.time()*1000) > 3000:
                    raise InvalidTrial('stale_qt_sample')
                reason = healthy(current, spec.display_hz, spec.video_width, spec.video_height) or (continuity(previous, current) if previous else None)
                previous = current
                if reason:
                    if state == 'measuring':
                        raise InvalidTrial(reason)
                    warm_start = None
                    state = 'waiting'
                    continue
                if warm_start is None:
                    warm_start = current['monotonicNs']
                    state = 'warming'
                    print('Healthy stream. Warmup started.', flush=True)
                if state == 'warming' and (current['monotonicNs']-warm_start)/1e9 >= spec.warmup:
                    state = 'measuring'
                    first = current
                    start_ms = wall_ms
                    window_samples = [current]
                    print(f'Measuring window 1/{spec.windows}.', flush=True)
                    continue
                if state != 'measuring':
                    continue
                window_samples.append(current)
                if (current['monotonicNs']-first['monotonicNs'])/1e9 < spec.seconds:
                    continue
                for line in native_tail.lines():
                    row = parse_native(line)
                    if row:
                        native_rows.append(row)
                window = window_result(first, current, window_samples, native_rows, start_ms, wall_ms, spec)
                result['windows'].append(window)
                result['quality_failures'] = sorted(set(result['quality_failures']) | set(window['quality_failures']))
                if window['validity_failures']:
                    result['invalid_reasons'] = window['validity_failures']
                    return result
                print(f"Window {len(result['windows'])}: {window['source_fps']:.2f} FPS, {window['late_intervals']} late intervals, {window['late_interval_rate']:.4%} late.", flush=True)
                if len(result['windows']) == spec.windows:
                    result['status'] = 'valid'
                    return result
                first = current
                start_ms = wall_ms
                window_samples = [current]
                native_rows = [row for row in native_rows if row['time_ms'] > start_ms]
                print(f"Measuring window {len(result['windows'])+1}/{spec.windows}.", flush=True)
            if state == 'measuring' and time.monotonic()-last_received > 3:
                raise InvalidTrial('qt_telemetry_stopped')
            time.sleep(.1)
        raise InvalidTrial('trial_deadline')
    except InvalidTrial as error:
        result['invalid_reasons'] = [str(error)]
        return result
    except (OSError, ValueError, ZeroDivisionError):
        result['invalid_reasons'] = ['telemetry_or_process_unavailable']
        return result
    except KeyboardInterrupt:
        result['status'] = 'interrupted'
        result['invalid_reasons'] = ['operator_interrupted']
        return result
    finally:
        qt_tail.close()
        native_tail.close()
        result['finished_ms'] = int(time.time()*1000)
        output = ROOT / '.runtime/hillclimb' / (spec.label + '.json')
        write_private(output, result)
        print(f"Trial {spec.label}: {result['status']}. Report {output}", flush=True)


def scalar_metrics(result):
    direct = ('late_interval_rate', 'source_fps', 'decoded_fps', 'submit_p95_upper_ms',
              'source_interval_p99_upper_ms', 'source_interval_max_upper_ms',
              'source_interval_overflow_per_second', 'relative_media_lag_growth_ms')
    metrics = {key: [] for key in direct}
    metrics.update({key: [] for key in ('native_fps', 'stream_ping_ms', 'stream_ping_p95_ms',
                                      'decoder_residence_p95_ms', 'packet_loss_percent')})
    for window in result['windows']:
        for key in direct:
            metrics[key].append(window.get(key))
        metrics['stream_ping_p95_ms'].append(window['native_metrics']['pingMs']['p95'])
        for target, source in (('stream_ping_ms', 'pingMs'), ('native_fps', 'framesPerSecond'),
                               ('decoder_residence_p95_ms', 'decodeTimings.residence.p95'),
                               ('packet_loss_percent', 'packetLossPercent')):
            metrics[target].append(window['native_metrics'][source]['max' if target == 'packet_loss_percent' else 'median'])
    return metrics


def compare(args):
    baseline = json.loads(args.baseline.read_text())
    candidate = json.loads(args.candidate.read_text())
    control = json.loads(args.control.read_text()) if args.control else None
    reports = [baseline, candidate] + ([control] if control else [])
    if any(r.get('method_version') != 4 for r in reports):
        raise InvalidTrial('comparison_method_version_mismatch')
    if any(type(r.get('spec', {}).get('display_hz')) is not int or r['spec']['display_hz'] not in (60, 100, 120) for r in reports):
        raise InvalidTrial('comparison_missing_or_invalid_requested_display')
    if any(r.get('status') != 'valid' or len(r.get('windows', [])) < 3 for r in reports):
        raise InvalidTrial('comparison_requires_three_valid_windows')
    if any(r['spec']['warmup'] < 20 or r['spec']['seconds'] < 60 for r in reports):
        raise InvalidTrial('comparison_requires_full_duration_trials')
    if control and baseline['build'] != control['build']:
        raise InvalidTrial('control_build_mismatch')
    if control and (not baseline.get('variant') or control.get('variant') != baseline['variant']):
        raise InvalidTrial('control_variant_mismatch')
    if control and control['spec']['display_hz'] != baseline['spec']['display_hz']:
        raise InvalidTrial('control_display_refresh_mismatch')
    if len({r.get('profile') for r in reports}) != 1:
        raise InvalidTrial('comparison_profile_mismatch')
    if len({(r['spec'].get('video_width'), r['spec'].get('video_height')) for r in reports}) != 1:
        raise InvalidTrial('comparison_video_size_mismatch')
    base = scalar_metrics(baseline)
    trial = scalar_metrics(candidate)
    if control:
        for key, values in scalar_metrics(control).items():
            base[key].extend(values)
    if any(not number(value) for metrics in (base, trial) for values in metrics.values() for value in values):
        raise InvalidTrial('comparison_missing_numeric_metric')
    base_late = statistics.median(base['late_interval_rate'])
    trial_late = statistics.median(trial['late_interval_rate'])
    noise = max(base['late_interval_rate']) - min(base['late_interval_rate'])
    gain = (base_late-trial_late)/base_late if base_late else 0
    failures = []
    if any(window['source_fps'] < 57 for window in candidate['windows']):
        failures.append('candidate_presented_fps_below_57')
    if base_late == 0:
        failures.append('baseline_at_floor')
    if gain < .2 or base_late-trial_late <= noise:
        failures.append('improvement_below_20_percent_or_control_spread')
    for key in ('stream_ping_ms', 'stream_ping_p95_ms', 'decoder_residence_p95_ms', 'submit_p95_upper_ms', 'packet_loss_percent',
                'source_interval_p99_upper_ms', 'relative_media_lag_growth_ms'):
        tolerance = max(base[key])-min(base[key])
        if statistics.median(trial[key]) > statistics.median(base[key])+tolerance:
            failures.append('regressed_' + key)
    for key in ('source_interval_max_upper_ms', 'source_interval_overflow_per_second'):
        tolerance = max(base[key])-min(base[key])
        if max(trial[key]) > max(base[key])+tolerance:
            failures.append('regressed_' + key)
    for key in ('source_fps', 'native_fps', 'decoded_fps'):
        tolerance = max(base[key])-min(base[key])
        if statistics.median(trial[key]) < statistics.median(base[key])-tolerance:
            failures.append('regressed_' + key)
    verdict = {'schema': 1, 'method_version': 4, 'baseline_label': baseline['label'], 'candidate_label': candidate['label'],
               'baseline_variant': baseline.get('variant', 'unspecified'), 'candidate_variant': candidate.get('variant', 'unspecified'),
               'baseline_display_hz': baseline['spec']['display_hz'], 'candidate_display_hz': candidate['spec']['display_hz'],
               'verdict': 'reject' if failures else ('accept' if control else 'provisional'),
               'reasons': failures or ([] if control else ['repeat_baseline_control_required']),
               'relative_improvement': gain, 'baseline_spread': noise,
               'baseline_late_interval_rate': base_late, 'candidate_late_interval_rate': trial_late,
               'target_50_percent_met': gain >= .5 and not failures and control is not None,
               'stop_rule': 'Stop after an accepted 50 percent improvement and at least four evaluated candidates, or when documented hypotheses are exhausted and repeated trials show a plateau. Rejected or invalid trials do not prove improvement.'}
    output = ROOT / '.runtime/hillclimb' / (candidate['label'] + '-comparison.json')
    write_private(output, verdict)
    print(json.dumps(verdict, indent=2))
    return verdict


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    run = commands.add_parser('measure')
    run.add_argument('label')
    run.add_argument('--variant', default='baseline', help='Safe label for the local algorithm or backend under test')
    run.add_argument('--profile', required=True, help='Safe label for the operator-verified negotiated profile and workload')
    run.add_argument('--telemetry', type=pathlib.Path, default=ROOT/'.runtime/hillclimb/qt-presentation.jsonl')
    run.add_argument('--native-log', type=pathlib.Path, default=ROOT/'.runtime/config/OpenNOW/diagnostics/native-streamer.log')
    run.add_argument('--warmup', type=float, default=20)
    run.add_argument('--seconds', type=float, default=60)
    run.add_argument('--windows', type=int, default=3)
    run.add_argument('--minimum-fps', type=float, default=57)
    run.add_argument('--deadline', type=float, default=300)
    run.add_argument('--display-hz', type=int, choices=(60, 100, 120), default=60,
                     help='Explicit expected display refresh; Qt readback must remain within 1 Hz. Stream FPS stays 60.')
    run.add_argument('--video-width', type=int, default=1920,
                     help='Expected Qt-reported source width.')
    run.add_argument('--video-height', type=int, default=1080,
                     help='Expected Qt-reported source height.')
    run.add_argument('--pid', type=int, help='Optional existing Qt process for numeric CPU sampling')
    diff = commands.add_parser('compare')
    diff.add_argument('baseline', type=pathlib.Path)
    diff.add_argument('candidate', type=pathlib.Path)
    diff.add_argument('--control', type=pathlib.Path)
    args = parser.parse_args()
    if args.command == 'measure':
        if not SAFE_LABEL.fullmatch(args.label) or not SAFE_LABEL.fullmatch(args.profile) or not SAFE_LABEL.fullmatch(args.variant):
            parser.error('label, profile, and variant must use 1..80 letters, numbers, underscore, or hyphen')
        if not (0 <= args.warmup <= 120 and 20 <= args.seconds <= 180 and 1 <= args.windows <= 5
                and 57 <= args.minimum_fps <= 60 and args.deadline >= args.warmup+args.seconds*args.windows):
            parser.error('invalid duration, window count, or FPS guardrail')
        if not (640 <= args.video_width <= 4096 and 480 <= args.video_height <= 2160):
            parser.error('invalid expected video dimensions')
    try:
        result = measure(args) if args.command == 'measure' else compare(args)
    except (InvalidTrial, OSError, ValueError, KeyError, subprocess.CalledProcessError):
        print('Trial failed. Input, telemetry, or required build metadata was unavailable or invalid.')
        return 2
    return 0 if result.get('status') == 'valid' or result.get('verdict') == 'accept' else 1


if __name__ == '__main__':
    raise SystemExit(main())
