#!/usr/bin/env python3
"""Export Git-selected research files to the OpenNOW branch, without private evidence."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
DESTINATION = ROOT / 'vendor/OpenNOW/docs/nucbox-research/workspace'
FORBIDDEN = {'.runtime', 'vendor', 'captures', 'build', '.toolchain', '.venv', '.git', '__pycache__'}


def main():
    paths = subprocess.check_output(
        ['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'], cwd=ROOT
    ).decode().split('\0')
    manifest = []
    for name in sorted(set(paths) - {''}):
        relative = Path(name)
        if relative.is_absolute() or '..' in relative.parts or FORBIDDEN.intersection(relative.parts):
            raise SystemExit('Refusing excluded path: ' + name)
        source = ROOT / relative
        if source.is_symlink() or not source.is_file():
            raise SystemExit('Expected ordinary file: ' + name)
        target = DESTINATION / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        if hashlib.sha256(target.read_bytes()).hexdigest() != digest:
            raise SystemExit('Copy verification failed: ' + name)
        manifest.append({'path': name, 'sha256': digest})
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip()
    (DESTINATION.parent / 'manifest.json').write_text(json.dumps({
        'sourceCommit': revision, 'selection': 'Git tracked and nonignored untracked files',
        'files': manifest,
    }, indent=2) + '\n')
    print(json.dumps({'exportedFiles': len(manifest), 'destination': str(DESTINATION)}))


if __name__ == '__main__':
    main()
