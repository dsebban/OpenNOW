#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root/.toolchain/nasm-src"
./autogen.sh
./configure --prefix="$root/.toolchain/nasm"
make --jobs=2
install -d "$root/.toolchain/nasm/bin"
install -m 755 nasm ndisasm "$root/.toolchain/nasm/bin/"
exec "$root/tools/build-native.sh"
