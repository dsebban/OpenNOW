#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
export RUSTUP_HOME="$root/.toolchain/rustup"
export CARGO_HOME="$root/.toolchain/cargo"
export PATH="$root/.toolchain/nasm/bin:$CARGO_HOME/bin:$root/.venv/bin:$PATH"
export CARGO_BUILD_JOBS=2
export PKG_CONFIG_PATH="$root/.toolchain/wayland/share/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
vulkan_include="$root/.toolchain/vulkan-headers/include"
export C_INCLUDE_PATH="$vulkan_include${C_INCLUDE_PATH:+:$C_INCLUDE_PATH}"
export CPLUS_INCLUDE_PATH="$vulkan_include${CPLUS_INCLUDE_PATH:+:$CPLUS_INCLUDE_PATH}"
qt_prefix="$root/.toolchain/qt/6.9.1/gcc_64"
if [ -d "$qt_prefix/lib/cmake/Qt6" ]; then
  export CMAKE_PREFIX_PATH="$qt_prefix${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
  export PATH="$qt_prefix/bin:$PATH"
  export LD_LIBRARY_PATH="$qt_prefix/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi
configure_args=()
if [ "${1:-}" = "--fresh" ]; then configure_args+=(--fresh); shift; fi
if [ "$#" -ne 0 ]; then echo 'Usage: build-native.sh [--fresh]' >&2; exit 2; fi
cmake "${configure_args[@]}" -S "$root/vendor/OpenNOW/opennow-qt" -B "$root/build/opennow-qt" \
  -G Ninja -DCMAKE_BUILD_TYPE=Release -DVulkan_INCLUDE_DIR="$vulkan_include"
cmake --build "$root/build/opennow-qt" --target opennow-qt --parallel 2
