#!/usr/bin/env python3
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('census', Path(__file__).with_name('presentation-census.py'))
census = importlib.util.module_from_spec(spec)
spec.loader.exec_module(census)


def sample(second):
    row = {key: 0 for key in census.COUNTERS}
    row.update({key: 1 for key in census.EPOCHS})
    row.update({'event': 'qt-presentation', 'wallTimeMs': 1000 + second * 1000,
                'monotonicNs': 1000000000 + second * 1000000000, 'sinceLastSourceSwapMs': 4})
    row.update({key: True for key in census.STATES})
    row.update({'gated': False, 'frameGeneration': False})
    row.update({'sourceSwapsTotal': second * 50, 'windowSwapsTotal': second * 60,
                'windowSwapsWithoutFreshSourceTotal': second * 10,
                'windowIntervalSamplesTotal': second * 60, 'notificationEnqueuedTotal': second * 80,
                'notificationCoalescedTotal': second * 20, 'notificationDeliveredTotal': second * 60,
                'notificationDrainSamplesTotal': second * 60})
    row.update({'runtimeAcquireCallsTotal': second * 60, 'runtimeAcquireSuccessTotal': second * 55,
                'runtimeAcquireEmptyTotal': second * 5, 'runtimeRecordCallsTotal': second * 55,
                'runtimeRecordSuccessTotal': second * 50, 'runtimeRecordNoFrameTotal': second * 5,
                'renderPrepareTotal': second * 60, 'renderTextureImportsTotal': second * 50,
                'renderCallTotal': second * 60, 'renderDrawsIssuedTotal': second * 60,
                'renderFreshSubmitsTotal': second * 50})
    row.update({key: [0] * 512 for key in census.HISTOGRAMS})
    row['windowIntervalHistogramMs'][16] = second * 60
    row['notificationOldestHistogramMs'][2] = second * 60
    row['notificationLatestHistogramMs'][0] = second * 60
    return row


def trial():
    return {'status': 'valid', 'method_version': 3, 'spec': {'windows': 1},
            'windows': [{'start_ms': 1000, 'end_ms': 3000, 'timing_epoch': 1, 'presentation_generation': 1}]}


class CensusTests(unittest.TestCase):
    def test_numeric_metrics_and_no_secret_output(self):
        rows = [sample(second) for second in range(3)]
        rows[1]['credentials'] = {'token': 'sentinel-secret'}
        output = census.analyze(trial(), rows)
        window = output['windows'][0]
        self.assertEqual(window['windowFps'], 60)
        self.assertEqual(window['freshSourceFps'], 50)
        self.assertAlmostEqual(window['withoutFreshSourceFraction'], 1 / 6)
        self.assertEqual(window['notificationEnqueuePerSecond'], 80)
        self.assertEqual(window['notificationCoalescedPerSecond'], 20)
        self.assertEqual(window['notificationDrainedPerSecond'], 60)
        self.assertEqual(window['oldestDrainP95UpperMs'], 3)
        self.assertEqual(window['latestDrainP99UpperMs'], 1)
        self.assertEqual(output['schema'], 2)
        self.assertEqual(output['trialStatus'], 'valid')
        self.assertEqual(window['stageRatesPerSecond']['runtimeAcquireEmptyTotal'], 5)
        self.assertEqual(window['stageRatesPerSecond']['runtimeRecordNoFrameTotal'], 5)
        self.assertEqual(window['stageRatesPerSecond']['renderTextureImportsTotal'], 50)
        self.assertNotIn('sentinel-secret', json.dumps(output))

    def test_old_build_reports_missing_fields(self):
        rows = [sample(second) for second in range(3)]
        del rows[1]['notificationTimingEpoch']
        with self.assertRaisesRegex(census.CensusError, 'missing_census_fields'):
            census.analyze(trial(), rows)
        rows = [sample(second) for second in range(3)]
        del rows[1]['runtimeRecordNoFrameTotal']
        with self.assertRaisesRegex(census.CensusError, 'missing_census_fields'):
            census.analyze(trial(), rows)

    def test_stage_outcome_equations_are_required_per_transaction(self):
        for key, reason in (
                ('runtimeAcquireEmptyTotal', 'runtime_acquire_outcome_mismatch'),
                ('runtimeRecordNoFrameTotal', 'runtime_record_outcome_mismatch'),
                ('renderDrawsIssuedTotal', 'render_call_outcome_mismatch')):
            row = sample(1)
            row[key] += 1
            with self.subTest(key=key), self.assertRaisesRegex(census.CensusError, reason):
                census.parse_sample(row)
        row = sample(1)
        row['runtimeRecordCallsTotal'] -= 1
        row['runtimeRecordSuccessTotal'] -= 1
        with self.assertRaisesRegex(census.CensusError, 'runtime_acquire_record_transaction_mismatch'):
            census.parse_sample(row)

    def test_runtime_generation_and_stage_rollbacks_are_rejected(self):
        rows = [sample(second) for second in range(3)]
        rows[1]['runtimeRecordStageGeneration'] = 2
        with self.assertRaisesRegex(census.CensusError, 'runtime_record_generation_mismatch'):
            census.analyze(trial(), rows)
        rows[1]['presentationGeneration'] = 2
        with self.assertRaisesRegex(census.CensusError, 'epoch_or_observer_changed'):
            census.analyze(trial(), rows)
        for group in (census.RUNTIME_COUNTERS, census.RENDER_COUNTERS):
            rows = [sample(second) for second in range(3)]
            rows[0].update({key: sample(2)[key] for key in group})
            with self.subTest(group=group), self.assertRaisesRegex(census.CensusError, 'counter_rollback'):
                census.analyze(trial(), rows)

    def test_complete_census_preserves_invalid_partial_trial_status(self):
        source = trial()
        source.update(status='invalid', method_version=3, spec={'windows': 3},
                      invalid_reasons=['sentinel-secret'])
        output = census.analyze(source, [sample(second) for second in range(3)])
        self.assertEqual(output['censusStatus'], 'complete')
        self.assertEqual(output['trialStatus'], 'invalid')
        self.assertEqual(output['trialMethodVersion'], 3)
        self.assertEqual(output['trialWindowCount'], 1)
        self.assertEqual(output['trialRequestedWindowCount'], 3)
        self.assertNotIn('sentinel-secret', json.dumps(output))
        del source['status']
        with self.assertRaisesRegex(census.CensusError, 'missing_or_invalid_trial_status'):
            census.analyze(source, [sample(second) for second in range(3)])

    def test_resets_gates_gaps_and_rollback_are_rejected(self):
        for key, value in [('epoch', 2), ('presentationGeneration', 2), ('gateEpoch', 2), ('notificationTimingEpoch', 2), ('observerId', 2),
                           ('notificationEnqueuedTotal', 0), ('gated', True), ('presentationAllowed', False),
                           ('monotonicNs', 1000000000), ('ptsDiscontinuitiesTotal', 1)]:
            rows = [sample(second) for second in range(3)]
            rows[1][key] = value
            if key == 'notificationEnqueuedTotal':
                rows[0][key] = 1
            with self.subTest(key=key), self.assertRaises(census.CensusError):
                census.analyze(trial(), rows)

    def test_histogram_validation_and_overflow_quantiles(self):
        rows = [sample(second) for second in range(3)]
        rows[1]['notificationOldestHistogramMs'][2] = 999
        with self.assertRaisesRegex(census.CensusError, 'rollback|mismatch'):
            census.analyze(trial(), rows)
        row = sample(1)
        row['notificationLatestHistogramMs'] = [0] * 513
        with self.assertRaisesRegex(census.CensusError, 'invalid_histogram'):
            census.parse_sample(row)
        self.assertIsNone(census.quantile([0] * 512, 100, .95))

    def test_one_frame_snapshot_skew_is_tolerated(self):
        rows = [sample(second) for second in range(3)]
        rows[-1]['windowSwapsTotal'] -= 1
        result = census.analyze(trial(), rows)
        self.assertEqual(result['windows'][0]['freshWindowBoundarySkewFrames'], -1)
        rows[-1]['windowSwapsTotal'] -= 2
        with self.assertRaisesRegex(census.CensusError, 'fresh_window_counter_mismatch'):
            census.analyze(trial(), rows)

    def test_strict_numbers_and_boundary_alignment(self):
        row = sample(1)
        row['notificationEnqueuedTotal'] = True
        with self.assertRaisesRegex(census.CensusError, 'invalid_numeric'):
            census.parse_sample(row)
        rows = [sample(second) for second in range(3)]
        rows[0]['wallTimeMs'] += 100
        with self.assertRaisesRegex(census.CensusError, 'window_boundary_missing'):
            census.analyze(trial(), rows)

    def test_reader_discards_unknown_fields(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'sample.jsonl'
            row = sample(0)
            row['secret'] = 'sentinel-secret'
            path.write_text(json.dumps(row) + '\n')
            self.assertNotIn('secret', census.read_rows(path)[0])


if __name__ == '__main__':
    unittest.main()
