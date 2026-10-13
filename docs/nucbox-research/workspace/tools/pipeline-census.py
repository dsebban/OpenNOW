#!/usr/bin/env python3
"""Summarize measured pipeline stages from existing live-trial reports."""
import argparse
import json
import math
from pathlib import Path


def finite(value):
    return value if isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value) else None


def census(report):
    rows = []
    for index, window in enumerate(report.get('windows', []), 1):
        native = window.get('native_metrics', {})
        incoming = finite(window.get('incoming_assembled_fps'))
        decoded = finite(window.get('decoded_fps'))
        presented = finite(window.get('source_fps'))
        duration = finite(window.get('duration_seconds'))
        drops = window.get('queue_drops_by_source', {})
        rows.append({
            'window': index,
            'duration_seconds': duration,
            'incoming_assembled_fps': incoming,
            'decoder_admission_fps': finite(native.get('framesPerSecond', {}).get('median')),
            'decoded_output_fps': decoded,
            'fresh_presented_fps': presented,
            'decoded_minus_presented_fps': decoded - presented if decoded is not None and presented is not None else None,
            'late_interval_rate': finite(window.get('late_interval_rate')),
            'decoder_residence_interval_p95_median_ms': finite(native.get('decodeTimings.residence.p95', {}).get('median')),
            'submit_to_swap_p95_upper_ms': finite(window.get('submit_p95_upper_ms')),
            'queue_drop_counts': {key: finite(value) for key, value in drops.items()
                                  if key in ('video-mailbox', 'decoded-video', 'video')},
        })
    status = report.get('status')
    return {'method_version': finite(report.get('method_version')),
            'status': status if status in ('valid', 'invalid') else None, 'windows': rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reports', type=Path, nargs='+')
    args = parser.parse_args()
    output = []
    for path in args.reports:
        try:
            report = json.loads(path.read_text())
            if not isinstance(report, dict):
                raise ValueError('report must be an object')
            output.append({'file': path.name, **census(report)})
        except (OSError, ValueError, TypeError, AttributeError):
            parser.error('Could not read a valid live-trial report')
    print(json.dumps({'reports': output, 'limits': [
        'Stage rates use existing telemetry boundaries and are not exact per-frame attribution.',
        'Queue counters are separate sources; their sum is not a count of visible frame loss.',
        'Qt presentation is a callback measurement, not physical display scanout.',
    ]}, indent=2, allow_nan=False))


if __name__ == '__main__':
    main()
