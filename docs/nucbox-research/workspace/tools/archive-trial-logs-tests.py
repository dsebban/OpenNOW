#!/usr/bin/env python3
"""Focused archival lifecycle checks using only temporary fixtures."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('archive_trial_logs', Path(__file__).with_name('archive-trial-logs.py'))
archive = importlib.util.module_from_spec(spec)
spec.loader.exec_module(archive)


class ArchiveTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name) / 'workspace'
        self.root.mkdir()
        self.proc = Path(self.temp.name) / 'proc'
        self.proc.mkdir()
        self.output = self.root / 'evidence'

    def tearDown(self):
        self.temp.cleanup()

    def log(self, relative=None, data=b'private synthetic payload\n'):
        path = self.root / (relative or archive.FILES[0])
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def process(self, executable):
        directory = self.proc / '123'
        directory.mkdir()
        (directory / 'exe').symlink_to(executable)

    def test_moves_exact_files_preserves_bytes_and_records_identity(self):
        original = {}
        for relative in archive.FILES:
            path = self.log(relative)
            original[relative] = (path.stat().st_ino, path.read_bytes())
        unrelated = self.log('.runtime/config/OpenNOW/diagnostics/keep.log')
        result = archive.archive(self.root, self.output, self.proc)
        self.assertEqual(result['status'], 'complete')
        self.assertEqual(len(result['files']), 4)
        self.assertTrue(unrelated.exists())
        for item in result['files']:
            inode, data = original[item['source']]
            destination = Path(item['destination'])
            self.assertFalse((self.root / item['source']).exists())
            self.assertEqual(destination.read_bytes(), data)
            self.assertEqual(destination.stat().st_ino, inode)
            self.assertEqual(item['sha256'], hashlib.sha256(data).hexdigest())
            self.assertEqual(destination.stat().st_mode & 0o777, 0o600)
        self.assertEqual(self.output.stat().st_mode & 0o777, 0o700)
        saved = json.loads((self.output / 'manifest.json').read_text())
        self.assertEqual(saved, result)
        self.assertNotIn('private synthetic payload', json.dumps(saved))

    def test_running_workspace_qt_or_core_refuses_before_changes(self):
        for name in ('opennow-qt', 'opennow-core'):
            with self.subTest(name=name):
                source = self.log()
                self.process(self.root / 'build/opennow-qt' / name)
                with self.assertRaisesRegex(archive.ArchiveError, 'process_running'):
                    archive.archive(self.root, self.output, self.proc)
                self.assertTrue(source.exists())
                self.assertFalse(self.output.exists())
                (self.proc / '123/exe').unlink()
                (self.proc / '123').rmdir()

    def test_deleted_workspace_executable_still_refuses(self):
        self.log()
        self.process(str(self.root / 'build/opennow-qt/opennow-qt') + ' (deleted)')
        with self.assertRaisesRegex(archive.ArchiveError, 'process_running'):
            archive.archive(self.root, self.output, self.proc)
        self.assertFalse(self.output.exists())

    def test_other_workspace_process_does_not_block(self):
        self.log()
        self.process(Path(self.temp.name) / 'other/opennow-qt')
        self.assertEqual(archive.archive(self.root, self.output, self.proc)['status'], 'complete')

    def test_existing_output_never_overwritten(self):
        source = self.log()
        self.output.mkdir()
        marker = self.output / 'marker'
        marker.write_bytes(b'keep')
        with self.assertRaisesRegex(archive.ArchiveError, 'unused_directory'):
            archive.archive(self.root, self.output, self.proc)
        self.assertTrue(source.exists())
        self.assertEqual(marker.read_bytes(), b'keep')

    def test_source_symlink_refuses_without_archiving(self):
        target = self.log('.runtime/other.log')
        source = self.root / archive.FILES[0]
        source.parent.mkdir(parents=True, exist_ok=True)
        source.symlink_to(target)
        with self.assertRaisesRegex(archive.ArchiveError, 'opened_safely'):
            archive.archive(self.root, self.output, self.proc)
        self.assertTrue(target.exists())
        self.assertFalse(self.output.exists())

    def test_missing_sources_are_recorded(self):
        self.log()
        result = archive.archive(self.root, self.output, self.proc)
        self.assertEqual(len(result['files']), 1)
        self.assertEqual(result['missing_sources'], list(archive.FILES[1:]))

    def test_move_never_replaces_existing_file(self):
        source = self.log()
        destination = self.root / 'occupied'
        destination.write_bytes(b'keep')
        with self.assertRaisesRegex(archive.ArchiveError, 'move_failed'):
            archive.move_without_replace(source, destination)
        self.assertTrue(source.exists())
        self.assertEqual(destination.read_bytes(), b'keep')


if __name__ == '__main__':
    unittest.main()
