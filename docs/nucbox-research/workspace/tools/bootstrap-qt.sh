#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
.venv/bin/python - <<'PY'
import aqt,pathlib
source=pathlib.Path(aqt.__file__).parent/'settings.ini'
destination=pathlib.Path('.toolchain/aqt.ini')
destination.parent.mkdir(exist_ok=True)
destination.write_text(source.read_text().replace('concurrency : 4','concurrency : 2'))
PY
.venv/bin/aqt -c .toolchain/aqt.ini install-qt linux desktop 6.9.1 linux_gcc_64 \
  --outputdir .toolchain/qt --internal --timeout 30 \
  --modules qtmultimedia qtshadertools
exec "$root/tools/build-native.sh"
