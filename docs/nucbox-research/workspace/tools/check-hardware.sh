#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
export RUSTUP_HOME="$root/.toolchain/rustup"
export CARGO_HOME="$root/.toolchain/cargo"
export PATH="$root/.toolchain/nasm/bin:$CARGO_HOME/bin:$root/.venv/bin:$PATH"
export C_INCLUDE_PATH="$root/.toolchain/vulkan-headers/include${C_INCLUDE_PATH:+:$C_INCLUDE_PATH}"
export CPLUS_INCLUDE_PATH="$root/.toolchain/vulkan-headers/include${CPLUS_INCLUDE_PATH:+:$CPLUS_INCLUDE_PATH}"
export PKG_CONFIG_PATH="$root/.toolchain/wayland/share/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
export CARGO_BUILD_JOBS=2
cd "$root/vendor/OpenNOW"
cargo test --manifest-path native/opennow-streamer/Cargo.toml \
  --target-dir "$root/build/opennow-qt/streamer-rust-target" --release -j 2 \
  -p opennow-streamer-platform-linux --features ffmpeg-bundled,vaapi \
  --lib video::ffmpeg::tests::shared_vulkan_hevc_p010_gpu_only \
  -- --ignored --exact --nocapture --test-threads=1
