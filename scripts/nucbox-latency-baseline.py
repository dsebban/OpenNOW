#!/usr/bin/env python3
"""Fixed-scene descriptive baseline. Never launches, drives, or changes a client."""
import argparse
import datetime
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import statistics
import subprocess
import sys
import time

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]

def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module

observe = load('baseline_observe', ROOT/'tools/observe-stream.py')
trial = observe.trial
census = load('baseline_census', ROOT/'tools/presentation-census.py')
STAGES = ('decodeTimings.call', 'decodeTimings.residence',
          'frameStageTimings.deliveryToAdmissionMs',
          'frameStageTimings.admissionToControlQueueMs',
          'frameStageTimings.assembledToControlQueueMs')
FLAGS = ('QT_QPA_PLATFORM', 'OPENNOW_EMBEDDED_PUBLISH_POLL_MS',
         'OPENNOW_CONTINUOUS_VIDEO_UPDATE', 'OPENNOW_NATIVE_VIDEO_BACKEND',
         'OPENNOW_EMBEDDED_READY_WAKE', 'QSG_RENDER_LOOP', 'QT_QPA_UPDATE_IDLE_TIME')
LIMITS = [
    'Descriptive baseline only; no optimization or acceptance decision.',
    'Scene is an operator assertion requiring before/after GUI evidence; no gameplay input during scoring.',
    'Network quantiles describe sampled telemetry values, not every ping or packet.',
    'Native stage p50/p95 are medians of overlapping last-256-sample summaries; max is maximum reported rolling max; p99 unavailable. These are NOT all-frame scored-window percentiles and may include warmup.',
    'Qt histogram quantiles/max are 1-ms bucket upper bounds; values >=512 ms overflow and max is then unavailable.',
    'Notification drain delay is not GPU mailbox age. Submit-to-swap is not decode-to-present latency.',
    'Qt frameSwapped is not compositor presentation, physical scanout, or input-to-photon.',
    'Stage percentiles must not be added to infer total latency.',
]

def distribution(values):
    values = sorted(v for v in values if trial.number(v))
    if not values:
        return {'samples': 0, 'p50': None, 'p95': None, 'p99': None, 'max': None}
    return {'samples': len(values), 'p50': statistics.median(values),
            'p95': values[math.ceil(.95*len(values))-1],
            'p99': values[math.ceil(.99*len(values))-1], 'max': values[-1]}

def histogram_summary(histogram, overflow):
    occupied = [i+1 for i, n in enumerate(histogram) if n]
    return {'samples': sum(histogram)+overflow, 'overflow': overflow,
            **{key: trial.percentile_bucket(histogram, fraction, overflow)
               for key, fraction in (('p50', .5), ('p95', .95), ('p99', .99))},
            'max': None if overflow or not occupied else max(occupied),
            'semantics': '1-ms bucket upper bounds'}

def native_rows(path):
    if path.stat().st_size > 3*1024*1024:
        raise trial.InvalidTrial('native_file_too_large')
    rows = []
    for line in path.read_text(errors='replace').splitlines():
        stamp = re.match(r'^(\d+) ', line)
        fields = dict(re.findall(r'([A-Za-z][A-Za-z0-9_.]*)=([^ ]+)', line))
        if not stamp or fields.get('type') != 'telemetry':
            continue
        row = {'time_ms': int(stamp[1])}
        for key in ('pingMs', 'jitterMs', 'packetLossPercent') + tuple(
                stage+'.'+q for stage in STAGES for q in ('p50', 'p95', 'max')):
            try:
                value = float(fields.get(key, 'nan'))
            except ValueError:
                continue
            if math.isfinite(value):
                row[key] = value
        rows.append(row)
    return rows

def summarize(window, rows, qt_rows):
    selected = [r for r in rows if window['start_ms'] < r['time_ms'] <= window['end_ms']]
    stages = {}
    for stage in STAGES:
        observed = {q: [r[stage+'.'+q] for r in selected if stage+'.'+q in r]
                    for q in ('p50', 'p95', 'max')}
        stages[stage] = {'telemetry_samples': len(observed['p50']),
                        'p50': statistics.median(observed['p50']) if observed['p50'] else None,
                        'p95': statistics.median(observed['p95']) if observed['p95'] else None,
                        'p99': None, 'max': max(observed['max']) if observed['max'] else None,
                        'semantics': 'rolling-summary medians; maximum reported rolling max'}
    qt = {name: histogram_summary(window['histograms'][key], window['counters'][overflow])
          for name, key, overflow in (
              ('fresh_source_interval', 'sourceIntervalHistogramMs', 'sourceIntervalOverflowTotal'),
              ('render_submit_to_swap', 'submitHistogramMs', 'submitOverflowTotal'))}
    detail = census.summarize_window(qt_rows, window)
    eligible = [census.parse_sample(row) for row in qt_rows
                if row['observerId'] == detail['observerId']
                and window['start_ms'] <= row['wallTimeMs'] <= window['end_ms']]
    for name, key, overflow in (
            ('oldest_notification_to_drain', 'notificationOldestHistogramMs', 'notificationOldestOverflowTotal'),
            ('latest_notification_to_drain', 'notificationLatestHistogramMs', 'notificationLatestOverflowTotal')):
        qt[name] = histogram_summary([b-a for a,b in zip(eligible[0][key], eligible[-1][key])],
                                     eligible[-1][overflow]-eligible[0][overflow])
    return {key: window[key] for key in ('duration_seconds', 'start_ms', 'end_ms', 'source_fps',
            'decoded_fps', 'incoming_assembled_fps', 'late_interval_rate', 'late_intervals',
            'queue_drops_by_source')} | {
            'network_ms': {key: distribution([r.get(key) for r in selected]) for key in ('pingMs','jitterMs')},
            'packet_loss_percent': distribution([r.get('packetLossPercent') for r in selected]),
            'native_stages_ms': stages, 'qt_stages_ms': qt, 'presentation_census': detail}

def main():
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ('pid', 'width', 'height'):
        parser.add_argument('--'+key, required=True, type=int)
    parser.add_argument('--display-hz', required=True, type=int, choices=(60,100,120))
    parser.add_argument('--scene', required=True, help='Safe fixed-scene label verified through the GUI')
    parser.add_argument('--label', required=True)
    parser.add_argument('--windows', default=3, type=int)
    parser.add_argument('--seconds', default=60, type=float)
    parser.add_argument('--warmup', default=20, type=float)
    parser.add_argument('--dry-run', action='store_true', help='Preflight only; saves no scored windows')
    args = parser.parse_args()
    if not all(trial.SAFE_LABEL.fullmatch(value) for value in (args.label,args.scene)) or not (
            args.pid > 0 and 1 <= args.width <= 16384 and 1 <= args.height <= 16384 and
            1 <= args.windows <= 10 and 1 <= args.seconds <= 300 and 0 <= args.warmup <= 300):
        parser.error('invalid bounded parameters or unsafe label')
    output = ROOT/'.runtime'/('latency-baseline-'+args.label)
    try:
        output.mkdir(mode=0o700)  # Never overwrite failed evidence.
    except FileExistsError:
        parser.error('label already exists; retain evidence and choose a new label')
    args.output = output/'observation.json'
    args.telemetry = ROOT/'.runtime/hillclimb/qt-presentation.jsonl'
    args.native_log = ROOT/'.runtime/config/OpenNOW/diagnostics/native-streamer.log'
    args.deadline = args.warmup + args.windows*(args.seconds+3) + 40
    result = {'schema':1, 'status':'invalid', 'scene_assertion':args.scene,
              'expected_dimensions':[args.width,args.height], 'expected_display_hz':args.display_hz,
              'requested_windows':args.windows, 'window_seconds':args.seconds, 'warmup_seconds':args.warmup,
              'windows':[], 'invalid_reasons':[], 'limits':LIMITS,
              'command':sys.argv, 'created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'script_hashes':{path.name:hashlib.sha256(path.read_bytes()).hexdigest()
                    for path in (Path(__file__),ROOT/'tools/observe-stream.py',ROOT/'tools/live-trial.py',ROOT/'tools/presentation-census.py')}}
    try:
        result['build'] = trial.build_identity()  # Outside scored windows.
        doctor = subprocess.run([sys.executable, str(ROOT/'.cursor/skills/verify-gfn-experiments/scripts/doctor.py')],
                                 capture_output=True, timeout=30)
        result['doctor'] = json.loads(doctor.stdout)
        processes = result['doctor']['processes']
        client = [p for p in processes if p['program']=='opennow-qt' and p['pid']==args.pid]
        if doctor.returncode or len(client)!=1 or not client[0]['expected_build']:
            raise trial.InvalidTrial('doctor_or_process_preflight_failed')
        result['processes'] = processes
        identity = trial.process_cpu(args.pid)[1]
        result['start_ticks'] = identity
        environment = dict(item.split('=',1) for item in Path(f'/proc/{args.pid}/environ').read_bytes().decode().split('\0') if '=' in item)
        result['experiment_flags'] = {key:environment.get(key) for key in FLAGS}
        result['qt_telemetry_enabled'] = environment.get('OPENNOW_LIVE_TELEMETRY') == str(args.telemetry)
        if not result['qt_telemetry_enabled']:
            raise trial.InvalidTrial('qt_telemetry_not_enabled_for_current_process')
        qt_rows = census.read_rows(args.telemetry)
        if not qt_rows or not 0 <= time.time()*1000-qt_rows[-1]['wallTimeMs'] <= 3000:
            raise trial.InvalidTrial('no_fresh_qt_telemetry')
        # Confirm intended dimensions/refresh before spending a scored interval.
        with args.telemetry.open() as stream:
            stream.seek(max(0, args.telemetry.stat().st_size-128*1024))
            if stream.tell():
                stream.readline()
            latest = json.loads(list(stream)[-1])
        reason = observe.health(trial.parse_qt(json.dumps(latest)), args)
        if reason:
            raise trial.InvalidTrial(reason)
        native = native_rows(args.native_log)
        if not native or not 0 <= time.time()*1000-native[-1]['time_ms'] <= 25000:
            raise trial.InvalidTrial('no_fresh_native_telemetry')
        if args.dry_run:
            result['status'] = 'preflight-only'
        else:
            observed = observe.observe(args)
            result['observation_status'] = observed['status']
            result['invalid_reasons'] = observed['invalid_reasons']
            if trial.process_cpu(args.pid)[1] != identity:
                raise trial.InvalidTrial('process_identity_changed')
            native = native_rows(args.native_log)
            qt_rows = census.read_rows(args.telemetry)
            trial.write_private(output/'native-numeric-samples.json', [r for r in native
                if any(w['start_ms'] < r['time_ms'] <= w['end_ms'] for w in observed['windows'])])
            trial.write_private(output/'qt-census-samples.json', [r for r in qt_rows
                if any(w['start_ms'] <= r['wallTimeMs'] <= w['end_ms'] for w in observed['windows'])])
            for window in observed['windows']:
                result['windows'].append(summarize(window, native, qt_rows))
            result['status'] = observed['status']
            if trial.build_identity() != result['build']:
                raise trial.InvalidTrial('source_or_binary_changed_during_baseline')
    except (trial.InvalidTrial, census.CensusError) as error:
        result['status'] = 'invalid'
        result['invalid_reasons'].append(str(error))
    except (OSError, ValueError, IndexError, subprocess.TimeoutExpired):
        result['status'] = 'invalid'
        result['invalid_reasons'].append('preflight_or_evidence_unavailable')
    trial.write_private(output/'summary.json', result)
    lines = ['# Latency baseline: '+args.label, '', 'Status: **'+result['status']+'**',
             'Scene assertion: `'+args.scene+'`', 'Reasons: '+', '.join(result['invalid_reasons']), '',
             '| Window | Fresh FPS | Decoded FPS | Late >25ms | RTT p50/p95/p99/max (ms) |',
             '| --- | ---: | ---: | ---: | --- |']
    for index, window in enumerate(result['windows'],1):
        ping = window['network_ms']['pingMs']
        lines.append(f"| {index} | {window['source_fps']:.2f} | {window['decoded_fps']:.2f} | {100*window['late_interval_rate']:.2f}% | "+'/'.join(str(ping[k]) for k in ('p50','p95','p99','max'))+' |')
    for index, window in enumerate(result['windows'],1):
        lines.extend(['',f'## Window {index}: timing stages (ms)', '', '| Stage | Semantics | p50 | p95 | p99 | max |', '| --- | --- | ---: | ---: | ---: | ---: |'])
        for key, stage in (window['native_stages_ms'] | window['qt_stages_ms']).items():
            lines.append('| '+key+' | '+stage['semantics']+' | '+' | '.join(str(stage[k]) for k in ('p50','p95','p99','max'))+' |')
    lines.extend(['','## Limits','']+['- '+line for line in LIMITS])
    (output/'summary.md').write_text('\n'.join(lines)+'\n')
    print(json.dumps({'status':result['status'], 'windows':len(result['windows']),
                      'reasons':result['invalid_reasons'], 'output':str(output)}), flush=True)
    return 0 if result['status'] in ('complete','preflight-only') else 2

if __name__ == '__main__':
    sys.exit(main())
