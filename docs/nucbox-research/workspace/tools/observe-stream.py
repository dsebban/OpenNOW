#!/usr/bin/env python3
"""Describe an existing stream; no candidate acceptance or account access."""
import argparse
import hashlib
import importlib.util
import os
import pathlib
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
loader = importlib.util.spec_from_file_location('live_trial', ROOT/'tools/live-trial.py')
trial = importlib.util.module_from_spec(loader)
sys.modules[loader.name] = trial
loader.loader.exec_module(trial)


def health(sample, args):
    if (sample['videoWidth'], sample['videoHeight']) != (args.width, args.height):
        return 'video_dimensions_mismatch'
    # Reuse state gates; explicit dimensions were checked above and stay in output.
    return trial.healthy(dict(sample, videoWidth=1920, videoHeight=1080), args.display_hz)


def observe(args):
    spec = trial.TrialSpec(args.label, 'descriptive-explicit-dimensions', args.telemetry,
                           args.warmup, args.seconds, args.windows, 0, args.deadline, args.display_hz)
    script_sha256 = hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()
    result = {'schema': 2, 'script_sha256': script_sha256,
              'mode': 'descriptive-only', 'status': 'invalid', 'label': args.label,
              'pid': args.pid, 'expected_width': args.width, 'expected_height': args.height,
              'expected_display_hz': args.display_hz, 'warmup_seconds': args.warmup,
              'requested_windows': args.windows, 'window_seconds': args.seconds,
              'windows': [], 'invalid_reasons': [], 'observed_dimensions': [],
              'limits': ['Qt fresh-source callbacks are not physical scanout.',
                         'No acceptance decision or matched-profile comparison.',
                         'Native p95 summaries are rolling, not all-frame percentiles.',
                         'Codec/backend/game scene require operator verification.'],
              'started_ms': int(time.time()*1000)}
    qt_tail, native_tail = trial.Tail(args.telemetry), trial.Tail(args.native_log)
    observer = previous = first = native_previous = warm_start = None
    samples, native_rows = [], []
    started = last_received = time.monotonic()
    start_ms = None
    measuring = False
    try:
        identity = trial.process_cpu(args.pid)
        if identity is None or pathlib.Path(os.readlink(f'/proc/{args.pid}/exe')).resolve() != (ROOT/'build/opennow-qt/opennow-qt').resolve():
            raise trial.InvalidTrial('wrong_or_missing_qt_process')
        print('Waiting for fresh telemetry; descriptive collection only.', flush=True)
        while time.monotonic()-started < args.deadline:
            process = trial.process_cpu(args.pid)
            if process is None or process[1] != identity[1]:
                raise trial.InvalidTrial('qt_process_exited_or_restarted')
            for line in native_tail.lines():
                row = trial.parse_native(line)
                if not row:
                    continue
                if row.get('invalid'):
                    raise trial.InvalidTrial(row['invalid'])
                if observer is not None and row['kind'] == 'telemetry':
                    if native_previous:
                        if row.get('decodeTimings.epoch') != native_previous.get('decodeTimings.epoch'):
                            raise trial.InvalidTrial('changed_decode_epoch')
                        for key in ('decodeTimings.outputsTotal', 'decodeTimings.submissionsTotal', 'frameStageTimings.assembledFramesTotal'):
                            if key in row and key in native_previous and row[key] < native_previous[key]:
                                raise trial.InvalidTrial('native_counter_rollback')
                    native_previous = row
                native_rows.append(row)
            for line in qt_tail.lines():
                current = trial.parse_qt(line)
                dimensions = [current['videoWidth'], current['videoHeight']]
                if dimensions not in result['observed_dimensions'] and len(result['observed_dimensions']) < 16:
                    result['observed_dimensions'].append(dimensions)
                if observer is None:
                    reason = health(current, args)
                    if reason:
                        result['last_wait_reason'] = reason
                        continue
                    observer = current['observerId']
                    result['observer_id'] = observer
                if current['observerId'] != observer:
                    continue
                last_received = time.monotonic()
                wall_ms = current.get('wallTimeMs')
                if wall_ms is None or abs(wall_ms-time.time()*1000) > 3000:
                    raise trial.InvalidTrial('stale_or_missing_qt_wall_time')
                reason = health(current, args) or (trial.continuity(previous, current) if previous else None)
                if reason:
                    raise trial.InvalidTrial(reason)
                previous = current
                if warm_start is None:
                    warm_start = current['monotonicNs']
                    print('Healthy stream; warmup started.', flush=True)
                if not measuring:
                    if (current['monotonicNs']-warm_start)/1e9 < args.warmup:
                        continue
                    measuring, first, start_ms, samples = True, current, wall_ms, [current]
                    continue
                samples.append(current)
                if (current['monotonicNs']-first['monotonicNs'])/1e9 < args.seconds:
                    continue
                window = trial.window_result(first, current, samples, native_rows, start_ms, wall_ms, spec)
                window.pop('quality_failures')
                window.pop('validity_failures')  # Frozen FPS floors do not govern this observation.
                window['video_width'], window['video_height'] = dimensions
                result['windows'].append(window)
                print(f"Window {len(result['windows'])}: {window['source_fps']:.2f} fresh FPS, {window['late_interval_rate']:.4%} late.", flush=True)
                if len(result['windows']) == args.windows:
                    result['status'] = 'complete'
                    return result
                first, start_ms, samples = current, wall_ms, [current]
                native_rows = [r for r in native_rows if r['time_ms'] > start_ms]
            if observer is not None and time.monotonic()-last_received > 3:
                raise trial.InvalidTrial('qt_telemetry_stopped')
            if len(native_rows) > 100000:
                raise trial.InvalidTrial('native_evidence_limit')
            time.sleep(.1)
        raise trial.InvalidTrial('observation_deadline')
    except trial.InvalidTrial as error:
        result['invalid_reasons'] = [str(error)]
    except KeyboardInterrupt:
        result['status'], result['invalid_reasons'] = 'interrupted', ['operator_interrupted']
    except (OSError, ValueError, ZeroDivisionError):
        result['invalid_reasons'] = ['telemetry_or_process_unavailable']
    finally:
        result['partial_window'] = {'qt_samples': len(samples), 'first': first, 'last': previous}
        result['finished_ms'] = int(time.time()*1000)
        qt_tail.close()
        native_tail.close()
        trial.write_private(args.output, result)
        print(f"{result['status']}: {args.output}", flush=True)
    return result


def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('width', 'height', 'pid'):
        parser.add_argument('--'+name, type=int, required=True)
    parser.add_argument('--label', required=True)
    parser.add_argument('--display-hz', type=int, choices=(60, 100, 120), default=60)
    parser.add_argument('--telemetry', type=pathlib.Path, default=ROOT/'.runtime/hillclimb/qt-presentation.jsonl')
    parser.add_argument('--native-log', type=pathlib.Path, default=ROOT/'.runtime/config/OpenNOW/diagnostics/native-streamer.log')
    parser.add_argument('--warmup', type=float, default=20)
    parser.add_argument('--seconds', '--duration', type=float, default=60)
    parser.add_argument('--windows', type=int, default=3)
    parser.add_argument('--deadline', type=float, default=300)
    args = parser.parse_args()
    if not trial.SAFE_LABEL.fullmatch(args.label) or not (1 <= args.width <= 16384 and 1 <= args.height <= 16384 and args.pid > 0 and 0 <= args.warmup <= 300 and 1 <= args.seconds <= 300 and 1 <= args.windows <= 10 and 1 <= args.deadline <= 3600):
        parser.error('invalid bounded observation parameters')
    args.output = ROOT/'.runtime/latency-20261008'/(args.label+'.json')
    if args.output.exists():
        parser.error('label already exists; retain prior evidence and choose a new label')
    return 0 if observe(args)['status'] == 'complete' else 2


if __name__ == '__main__':
    sys.exit(main())
