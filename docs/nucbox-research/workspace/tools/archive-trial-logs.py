#!/usr/bin/env python3
"""Archive stopped OpenNOW telemetry without reading account state."""
import argparse
import ctypes
import datetime
import hashlib
import json
import os
from pathlib import Path
import stat
import sys

ROOT = Path(__file__).resolve().parents[1]
PROGRAMS = {'opennow-qt', 'opennow-core'}
FILES = (
    '.runtime/config/OpenNOW/diagnostics/native-streamer.log',
    '.runtime/config/OpenNOW/diagnostics/native-streamer.log.previous',
    '.runtime/config/OpenNOW/diagnostics/native-streamer.previous.log',
    '.runtime/hillclimb/qt-presentation.jsonl',
)


class ArchiveError(Exception):
    pass


def require_stopped(root, proc_root=Path('/proc')):
    """Inspect executable links only; never arguments, environment, or credentials."""
    for entry in proc_root.iterdir():
        if not entry.name.isdigit():
            continue
        try:
            target = Path(str((entry / 'exe').readlink()).removesuffix(' (deleted)'))
        except FileNotFoundError:
            continue
        except PermissionError:
            try:
                name = (entry / 'comm').read_text().strip()
            except FileNotFoundError:
                continue
            except OSError:
                raise ArchiveError('process_identity_unavailable') from None
            if name in PROGRAMS:
                raise ArchiveError('opennow_process_identity_unavailable')
            continue
        except OSError:
            raise ArchiveError('process_scan_failed') from None
        if target.name in PROGRAMS and target.is_relative_to(root):
            raise ArchiveError('workspace_opennow_process_running')


def fingerprint(path):
    flags = os.O_RDONLY | getattr(os, 'O_NOFOLLOW', 0) | getattr(os, 'O_NONBLOCK', 0)
    try:
        descriptor = os.open(path, flags)
    except FileNotFoundError:
        return None
    except OSError:
        raise ArchiveError('source_cannot_be_opened_safely') from None
    with os.fdopen(descriptor, 'rb') as stream:
        before = os.fstat(stream.fileno())
        if not stat.S_ISREG(before.st_mode):
            raise ArchiveError('source_is_not_a_regular_file')
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
        after = os.fstat(stream.fileno())
        if (before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (after.st_size, after.st_mtime_ns, after.st_ctime_ns):
            raise ArchiveError('source_changed_while_hashing')
    return {'bytes': before.st_size, 'device': before.st_dev, 'inode': before.st_ino,
            'mtime_ns': before.st_mtime_ns, 'sha256': digest}


def move_without_replace(source, destination):
    """Linux rename preserves the file and fails if the destination exists."""
    library = ctypes.CDLL(None, use_errno=True)
    rename = getattr(library, 'renameat2', None)
    if rename is None:
        raise ArchiveError('rename_without_replace_unavailable')
    rename.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
    rename.restype = ctypes.c_int
    if rename(-100, os.fsencode(source), -100, os.fsencode(destination), 1):
        raise ArchiveError('archive_move_failed_errno_' + str(ctypes.get_errno()))


def archive(root, output, proc_root=Path('/proc')):
    root = root.resolve()
    output = output.absolute()
    require_stopped(root, proc_root)
    if output.exists() or output.is_symlink():
        raise ArchiveError('output_must_be_an_unused_directory')
    if not output.parent.is_dir():
        raise ArchiveError('output_parent_missing')
    plan, missing = [], []
    for relative in FILES:
        source = root / relative
        info = fingerprint(source)
        if info is None:
            missing.append(relative)
        else:
            if info['device'] != output.parent.stat().st_dev:
                raise ArchiveError('archive_requires_same_filesystem')
            plan.append({'source': relative, 'destination': str(output / source.name), **info})
    require_stopped(root, proc_root)
    output.mkdir(mode=0o700)
    manifest = {'schema': 1, 'created_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                'status': 'incomplete', 'files': [], 'missing_sources': missing}
    try:
        for item in plan:
            require_stopped(root, proc_root)
            source = root / item['source']
            current = source.lstat()
            if (current.st_dev, current.st_ino, current.st_size, current.st_mtime_ns) != (
                    item['device'], item['inode'], item['bytes'], item['mtime_ns']):
                raise ArchiveError('source_changed_before_move')
            destination = Path(item['destination'])
            move_without_replace(source, destination)
            # A rename preserves source mode; retained private evidence must be private.
            destination.chmod(0o600)
            manifest['files'].append(item)
            if fingerprint(destination) != {key: value for key, value in item.items() if key not in ('source', 'destination')}:
                raise ArchiveError('archive_verification_failed')
        manifest['status'] = 'complete'
    except (ArchiveError, OSError):
        manifest['status'] = 'incomplete'
        raise
    finally:
        flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, 'O_NOFOLLOW', 0)
        with os.fdopen(os.open(output / 'manifest.json', flags, 0o600), 'w') as file:
            json.dump(manifest, file, indent=2)
            file.write('\n')
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True, help='New private directory with an existing parent, on the workspace filesystem')
    args = parser.parse_args()
    os.umask(0o077)
    try:
        result = archive(ROOT, args.output)
    except ArchiveError as error:
        print(json.dumps({'status': 'refused_or_incomplete', 'reason': str(error)}))
        return 1
    except OSError:
        print(json.dumps({'status': 'refused_or_incomplete', 'reason': 'filesystem_unavailable'}))
        return 1
    print(json.dumps({'status': result['status'], 'archived_files': len(result['files']),
                      'manifest': str(args.output.absolute() / 'manifest.json')}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
