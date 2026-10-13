#!/usr/bin/env python3
"""Check validity boundaries and public output reduction for the trial runner."""
import copy
import importlib.util
import json
import pathlib
import sys
import contextlib
import io
import tempfile
import types
from unittest.mock import patch
import unittest

spec = importlib.util.spec_from_file_location('live_trial', pathlib.Path(__file__).with_name('live-trial.py'))
trial = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = trial
spec.loader.exec_module(trial)


def sample():
    value = {'event': 'qt-presentation', 'monotonicNs': 1000000000,
             'observerId': 1, 'epoch': 1, 'gateEpoch': 0, 'presentationGeneration': 1,
             'gated': False, 'frameGeneration': False, 'visible': True,
             'windowActive': True, 'fullscreen': True, 'runtimeRunning': True,
             'presentationAllowed': True, 'displayHz': 60, 'sinceLastSourceSwapMs': 5,
             'sourceIntervalMaxMs': 17, 'submitMaxMs': 7, 'videoWidth': 1920, 'videoHeight': 1080}
    value.update({key: 0 for key in trial.COUNTERS})
    value.update({key: [0]*512 for key in trial.HISTOGRAMS})
    return value


class TrialBoundaries(unittest.TestCase):
    def test_display_readback_matches_explicit_request(self):
        raw = sample()
        self.assertIsNone(trial.healthy(raw))
        self.assertEqual(trial.healthy(raw, 120), 'display_refresh_mismatch')
        for expected in (60, 120):
            for readback in (expected-1, expected, expected+1):
                raw['displayHz'] = readback
                self.assertIsNone(trial.healthy(raw, expected))
            for readback in (expected-1.01, expected+1.01):
                raw['displayHz'] = readback
                self.assertEqual(trial.healthy(raw, expected), 'display_refresh_mismatch')
        raw['displayHz'] = 120
        self.assertEqual(trial.healthy(raw), 'display_refresh_mismatch')
        self.assertEqual(trial.TrialSpec('a', 'profile', pathlib.Path('/tmp/unused')).display_hz, 60)

    def test_started_stream_requires_actual_frame_and_correct_dimensions(self):
        raw = sample()
        raw['sinceLastSourceSwapMs'] = None
        self.assertEqual(trial.healthy(trial.parse_qt(json.dumps(raw))), 'video_stalled')
        raw['sinceLastSourceSwapMs'] = 5
        raw['videoWidth'] = 2560
        self.assertEqual(trial.healthy(trial.parse_qt(json.dumps(raw))), 'video_dimensions_mismatch')

    def test_no_arbitrary_fields_escape_qt_parser(self):
        raw = sample()
        raw['credentials'] = {'token': 'sentinel-secret'}
        parsed = trial.parse_qt(json.dumps(raw))
        self.assertNotIn('sentinel-secret', json.dumps(parsed))
        raw['epoch'] = True
        with self.assertRaisesRegex(trial.InvalidTrial, 'invalid_qt_counter_epoch'):
            trial.parse_qt(json.dumps(raw))

    def test_qt_integral_json_numbers_are_exact_and_safe(self):
        raw = sample()
        raw['monotonicNs'] = 1.25e12
        raw['sourceSwapsTotal'] = 3600.0
        raw['sourceIntervalHistogramMs'][16] = 30.0
        parsed = trial.parse_qt(json.dumps(raw))
        self.assertEqual(parsed['monotonicNs'], 1250000000000)
        self.assertIsInstance(parsed['sourceSwapsTotal'], int)
        self.assertIsInstance(parsed['sourceIntervalHistogramMs'][16], int)
        for invalid in (3.5, float(2**53), float('inf'), -1.0):
            raw['monotonicNs'] = invalid
            with self.assertRaises(trial.InvalidTrial):
                trial.parse_qt(json.dumps(raw))

    def test_reset_or_missing_samples_cannot_be_a_win(self):
        before = sample()
        after = copy.deepcopy(before)
        after['monotonicNs'] += 1000000000
        after['epoch'] += 1
        self.assertEqual(trial.continuity(before, after), 'changed_epoch')
        after['epoch'] = before['epoch']
        after['ptsDiscontinuitiesTotal'] = 1
        self.assertEqual(trial.continuity(before, after), 'source_pts_discontinuity')
        after['ptsDiscontinuitiesTotal'] = 0
        after['monotonicNs'] += 3000000000
        self.assertEqual(trial.continuity(before, after), 'qt_sample_gap_or_clock_reset')
        after['monotonicNs'] = before['monotonicNs']+1000000000
        before['sourceSwapsTotal'] = 3
        self.assertEqual(trial.continuity(before, after), 'rollback_sourceSwapsTotal')

    def test_native_parser_retains_only_bounded_numeric_fields(self):
        parsed = trial.parse_native('1000 INFO type=telemetry pingMs=57 token=sentinel-secret url=https://private framesPerSecond=60')
        self.assertEqual(parsed, {'time_ms': 1000, 'kind': 'telemetry', 'pingMs': 57.0, 'framesPerSecond': 60.0})
        self.assertEqual(trial.parse_native('1000 INFO event=queue-dropped unit=frames media=video-mailbox count=3'),
                         {'time_ms': 1000, 'kind': 'drop', 'media': 'video-mailbox', 'count': 3})

    def test_window_reports_exact_late_count_and_rejects_epoch_change(self):
        first = sample()
        last = copy.deepcopy(first)
        last['monotonicNs'] += 60000000000
        last.update(sourceSwapsTotal=3600, sourceIntervalSamplesTotal=3600,
                    lateSourceIntervalsTotal=10, submitSamplesTotal=3600)
        last['sourceIntervalHistogramMs'][16] = 3590
        last['sourceIntervalHistogramMs'][30] = 10
        last['submitHistogramMs'][5] = 3600
        rows = [{'time_ms': t, 'kind': 'telemetry', 'pingMs': 57, 'framesPerSecond': 60,
                 'packetLossPercent': 0, 'decodeTimings.residence.p95': 7,
                 'decodeTimings.epoch': 1, 'decodeTimings.outputsTotal': t,
                 'decodeTimings.submissionsTotal': t, 'frameStageTimings.assembledFramesTotal': t*.06} for t in range(10000, 60001, 10000)]
        config = trial.TrialSpec('a', 'profile', pathlib.Path('/tmp/unused'))
        result = trial.window_result(first, last, [first, last], rows, 0, 60000, config)
        self.assertEqual(result['late_intervals'], 10)
        self.assertAlmostEqual(result['late_interval_rate'], 10/3600)
        self.assertEqual(result['source_fps'], 60)
        self.assertEqual(result['submit_p95_upper_ms'], 6)
        rows[-1]['decodeTimings.epoch'] = 2
        with self.assertRaisesRegex(trial.InvalidTrial, 'changed_or_missing_decode_epoch'):
            trial.window_result(first, last, [first, last], rows, 0, 60000, config)
        rows[-1]['decodeTimings.epoch'] = 1
        last['sourceSwapsTotal'] = 3000
        slow = trial.window_result(first, last, [first, last], rows, 0, 60000, config)
        self.assertEqual(slow['source_fps'], 50)
        self.assertEqual(slow['quality_failures'], ['presented_fps_below_57'])
        self.assertEqual(slow['validity_failures'], [])
        self.assertEqual(slow['incoming_assembled_fps'], 60)
        rows[-1]['frameStageTimings.assembledFramesTotal'] -= 500
        workload = trial.window_result(first, last, [first, last], rows, 0, 60000, config)
        self.assertEqual(workload['validity_failures'], ['incoming_assembled_fps_below_57'])

    def test_comparison_needs_control_and_rejects_lower_fps(self):
        def report(label, rate, fps=60):
            metric = {'samples': 6, 'median': 57, 'p95': 58, 'max': 58}
            window = {'late_interval_rate': rate, 'source_fps': fps, 'submit_p95_upper_ms': 8,
                      'decoded_fps': 60, 'source_interval_p99_upper_ms': 18,
                      'source_interval_max_upper_ms': 34, 'source_interval_overflow_per_second': 0,
                      'relative_media_lag_growth_ms': 0,
                      'native_metrics': {'pingMs': metric, 'decodeTimings.residence.p95': metric,
                                         'framesPerSecond': {'samples': 6, 'median': 60, 'p95': 60, 'max': 60},
                                         'packetLossPercent': {'samples': 6, 'median': 0, 'p95': 0, 'max': 0}}}
            return {'method_version': 3, 'label': label, 'variant': 'poll1' if label == 'candidate' else 'baseline',
                    'status': 'valid', 'profile': 'same', 'build': {'binary': 'base'},
                    'spec': {'warmup': 20, 'seconds': 60, 'display_hz': 60}, 'windows': [copy.deepcopy(window) for _ in range(3)]}
        with tempfile.TemporaryDirectory() as directory:
            base = pathlib.Path(directory)/'base.json'
            candidate = pathlib.Path(directory)/'candidate.json'
            control = pathlib.Path(directory)/'control.json'
            base.write_text(json.dumps(report('base', .02)))
            candidate.write_text(json.dumps(report('candidate', .008)))
            control.write_text(json.dumps(report('control', .02)))
            args = types.SimpleNamespace(baseline=base, candidate=candidate, control=None)
            with patch.object(trial, 'write_private'), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(trial.compare(args)['verdict'], 'provisional')
                args.control = control
                self.assertEqual(trial.compare(args)['verdict'], 'accept')
                faster_display = report('candidate', .008)
                faster_display['spec']['display_hz'] = 120
                candidate.write_text(json.dumps(faster_display))
                accepted = trial.compare(args)
                self.assertEqual(accepted['verdict'], 'accept')
                self.assertEqual(accepted['candidate_display_hz'], 120)
                self.assertEqual(accepted['baseline_display_hz'], 60)
                wrong_refresh = report('control', .02)
                wrong_refresh['spec']['display_hz'] = 120
                control.write_text(json.dumps(wrong_refresh))
                with self.assertRaisesRegex(trial.InvalidTrial, 'control_display_refresh_mismatch'):
                    trial.compare(args)
                control.write_text(json.dumps(report('control', .02)))
                for missing_or_invalid in (None, 59, 90, 120.0, True):
                    invalid_display = report('candidate', .008)
                    if missing_or_invalid is None:
                        del invalid_display['spec']['display_hz']
                    else:
                        invalid_display['spec']['display_hz'] = missing_or_invalid
                    candidate.write_text(json.dumps(invalid_display))
                    with self.assertRaisesRegex(trial.InvalidTrial, 'missing_or_invalid_requested_display'):
                        trial.compare(args)
                for changed in ((base,), (candidate,), (control,), (base, candidate, control)):
                    base.write_text(json.dumps(report('base', .02)))
                    candidate.write_text(json.dumps(report('candidate', .008)))
                    control.write_text(json.dumps(report('control', .02)))
                    for path in changed:
                        legacy = json.loads(path.read_text())
                        legacy['method_version'] = 2
                        path.write_text(json.dumps(legacy))
                    with self.assertRaisesRegex(trial.InvalidTrial, 'comparison_method_version_mismatch'):
                        trial.compare(args)
                base.write_text(json.dumps(report('base', .02)))
                candidate.write_text(json.dumps(report('candidate', .008)))
                control.write_text(json.dumps(report('control', .02)))
                for variant in ('continuous', None):
                    wrong_control = report('control', .02)
                    wrong_control['variant'] = variant
                    control.write_text(json.dumps(wrong_control))
                    with self.assertRaisesRegex(trial.InvalidTrial, 'control_variant_mismatch'):
                        trial.compare(args)
                control.write_text(json.dumps(report('control', .02)))
                candidate.write_text(json.dumps(report('candidate', .008, fps=59)))
                rejected = trial.compare(args)
                self.assertEqual(rejected['verdict'], 'reject')
                self.assertIn('regressed_source_fps', rejected['reasons'])
                candidate.write_text(json.dumps(report('candidate', .008, fps=56)))
                self.assertIn('candidate_presented_fps_below_57', trial.compare(args)['reasons'])
                delayed = report('candidate', .008)
                for window in delayed['windows']:
                    window['relative_media_lag_growth_ms'] = 50
                candidate.write_text(json.dumps(delayed))
                self.assertIn('regressed_relative_media_lag_growth_ms', trial.compare(args)['reasons'])
                hitch = report('candidate', .008)
                hitch['windows'][0]['source_interval_max_upper_ms'] = 450
                candidate.write_text(json.dumps(hitch))
                self.assertIn('regressed_source_interval_max_upper_ms', trial.compare(args)['reasons'])
                slower = report('candidate', .008)
                for window in slower['windows']:
                    window['decoded_fps'] = 58
                    window['native_metrics']['framesPerSecond']['median'] = 58
                candidate.write_text(json.dumps(slower))
                self.assertIn('regressed_decoded_fps', trial.compare(args)['reasons'])
                self.assertIn('regressed_native_fps', trial.compare(args)['reasons'])

    def test_overflow_bucket_does_not_claim_a_finite_percentile(self):
        histogram = [0]*512
        self.assertIsNone(trial.percentile_bucket(histogram, .95, overflow=1))
        histogram[-1] = 1
        self.assertEqual(trial.percentile_bucket(histogram, .95), 512)


if __name__ == '__main__':
    unittest.main()
