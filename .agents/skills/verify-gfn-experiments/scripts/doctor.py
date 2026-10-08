#!/usr/bin/env python3
"""Read-only, allowlisted OpenNOW experiment preflight. No account requests."""
import argparse
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import subprocess
import sys
import time

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]


def command(args, timeout=15):
    try:
        result = subprocess.run(args, capture_output=True, timeout=timeout, check=False)
        return result.stdout.decode('utf-8', 'replace') if result.returncode == 0 else None
    except (OSError, subprocess.TimeoutExpired):
        return None


def numeric(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def integer_file(path):
    try:
        return int(Path(path).read_text().strip())
    except (OSError, ValueError):
        return None


def build():
    result = {}
    for name in ('opennow-qt', 'opennow-core', 'libopennow_streamer_ffi.so'):
        path = ROOT / 'build/opennow-qt' / name
        try:
            with path.open('rb') as stream:
                digest = hashlib.file_digest(stream, 'sha256').hexdigest()
            result[name] = {'present': True, 'sha256': digest}
        except OSError:
            result[name] = {'present': False}
    revision = command(['git', '-C', str(ROOT / 'vendor/OpenNOW'), 'rev-parse', 'HEAD'])
    result['source_revision'] = revision.strip() if revision and re.fullmatch(r'[0-9a-f]{40}\s*', revision) else None
    return result


def processes():
    result = []
    ticks = os.sysconf('SC_CLK_TCK')
    boot_wall = time.time() - float(Path('/proc/uptime').read_text().split()[0])
    for path in Path('/proc').iterdir():
        if not path.name.isdigit():
            continue
        try:
            executable = (path / 'exe').readlink()
            name = executable.name.removesuffix(' (deleted)')
            if name not in ('opennow-qt', 'opennow-core'):
                continue
            fields = (path / 'stat').read_text().rsplit(')', 1)[1].split()
            result.append({'pid': int(path.name), 'program': name, 'parent_pid': int(fields[1]),
                           'start_ticks': int(fields[19]),
                           'started_wall_ms': (boot_wall + int(fields[19]) / ticks) * 1000,
                           'expected_build': executable == ROOT / 'build/opennow-qt' / name})
        except (OSError, ValueError, IndexError):
            continue
    return sorted(result, key=lambda item: item['pid'])


def wifi():
    raw = command(['/home/gamer/.local/bin/gfn-route', 'wifi', '--json'])
    try:
        data = json.loads(raw) if raw else {}
        if not isinstance(data, dict):
            return {'available': False}
    except ValueError:
        return {'available': False}
    result = {'available': bool(data), 'connected': data.get('connected') is True,
              'power_save': data.get('power_save') if data.get('power_save') in ('on', 'off') else 'unknown',
              'bssid_lock_matches': bool(data.get('bssid')) and data.get('bssid') == data.get('profile_bssid'),
              'driver_aspm_disabled': data.get('driver_aspm_disabled') is True}
    for key in ('signal_dbm', 'frequency_mhz'):
        result[key] = data.get(key) if numeric(data.get(key)) else None
    return result


def capabilities():
    raw = command([sys.executable, str(ROOT / 'tools/probe_vaapi.py')])
    try:
        data = json.loads(raw) if raw else {}
        profiles = []
        for item in data.get('profiles', []):
            name = item.get('profile', '')
            if not isinstance(name, str) or not re.fullmatch(r'VAProfile[A-Za-z0-9_]+', name):
                continue
            entries = [entry for entry in item.get('entrypoints', [])
                       if isinstance(entry, str) and re.fullmatch(r'VAEntrypoint[A-Za-z0-9_]+', entry)]
            profiles.append({'profile': name, 'entrypoints': entries})
        return {'available': bool(profiles), 'profiles': profiles,
                'meaning': 'Advertised VA-API support; live decoder and GPU import remain unverified.'}
    except (ValueError, TypeError, AttributeError):
        return {'available': False}


def observer(client, display_hz=60):
    spec = importlib.util.spec_from_file_location('gfn_trial_doctor', ROOT / 'tools/live-trial.py')
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    latest = {}
    try:
        with (ROOT / '.runtime/hillclimb/qt-presentation.jsonl').open('rb') as stream:
            stream.seek(0, 2)
            offset = max(0, stream.tell() - 1024 * 1024)
            stream.seek(offset)
            if offset:
                stream.readline()
            for line in stream:
                try:
                    sample = module.parse_qt(line)
                    latest[sample['observerId']] = sample
                except module.InvalidTrial:
                    continue
    except OSError:
        return {'available': False, 'ready': False, 'reason': 'telemetry_missing'}
    now = time.time() * 1000
    fresh = [item for item in latest.values() if numeric(item.get('wallTimeMs'))
             and 0 <= now - item['wallTimeMs'] <= 5000
             and client and item['wallTimeMs'] >= client['started_wall_ms']]
    healthy = [item for item in fresh if module.healthy(item, display_hz) is None]
    result = {'available': bool(latest), 'fresh_observers': len(fresh),
              'healthy_observers': len(healthy), 'ready': len(healthy) == 1}
    if not fresh:
        result['reason'] = 'no_fresh_telemetry_for_current_client'
    elif len(healthy) > 1:
        result['reason'] = 'multiple_healthy_observers'
    chosen = healthy[0] if len(healthy) == 1 else max(fresh, key=lambda item: item['wallTimeMs'], default=None)
    if chosen:
        result['sample_age_ms'] = round(now - chosen['wallTimeMs'], 1)
        for key in ('visible', 'windowActive', 'fullscreen', 'runtimeRunning', 'presentationAllowed',
                    'gated', 'frameGeneration', 'displayHz', 'videoWidth', 'videoHeight',
                    'sinceLastSourceSwapMs', 'sourceSwapsTotal', 'ptsDiscontinuitiesTotal'):
            if key in chosen:
                result[key] = chosen[key]
        if not result['ready']:
            result.setdefault('reason', module.healthy(chosen, display_hz))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--require-stream', action='store_true', help='Fail unless current Qt telemetry is healthy')
    parser.add_argument('--capabilities', action='store_true', help='Also query advertised VA-API profiles before scoring')
    parser.add_argument('--display-hz', type=int, choices=(60, 120), default=60,
                        help='Expected display refresh; does not change the 60 FPS source profile')
    args = parser.parse_args()
    result = {'schema': 1, 'checked_wall_ms': round(time.time() * 1000), 'requested_display_hz': args.display_hz,
              'auth': 'Not probed. Confirm saved-account readiness through the normal GUI.',
              'build': build(), 'processes': processes(), 'wifi': wifi(),
              'rmem_max': integer_file('/proc/sys/net/core/rmem_max'),
              'rmem_default': integer_file('/proc/sys/net/core/rmem_default')}
    clients = [item for item in result['processes'] if item['program'] == 'opennow-qt']
    client = clients[0] if len(clients) == 1 and clients[0]['expected_build'] else None
    result['observer'] = observer(client, args.display_hz)
    if args.capabilities:
        result['capabilities'] = capabilities()
    issues = []
    if not all(result['build'][name]['present'] for name in ('opennow-qt', 'opennow-core', 'libopennow_streamer_ffi.so')):
        issues.append('required_build_missing')
    if len(clients) > 1 or any(not item['expected_build'] for item in result['processes']):
        issues.append('multiple_or_unexpected_clients')
    if any(item['program'] == 'opennow-core' and item['parent_pid'] not in [p['pid'] for p in clients]
           for item in result['processes']):
        issues.append('standalone_core_present_do_not_start_another_vault_owner')
    if not result['wifi'].get('connected'):
        issues.append('wifi_unavailable_or_disconnected')
    if args.capabilities and not result['capabilities']['available']:
        issues.append('capability_query_failed')
    if args.require_stream and not result['observer']['ready']:
        issues.append('stream_preflight_failed')
    result['issues'] = issues
    result['status'] = 'needs_attention' if issues else 'ready_for_measurement_preflight' if args.require_stream else 'ready_for_gui'
    print(json.dumps(result, indent=2, allow_nan=False))
    return 1 if issues else 0


if __name__ == '__main__':
    raise SystemExit(main())
