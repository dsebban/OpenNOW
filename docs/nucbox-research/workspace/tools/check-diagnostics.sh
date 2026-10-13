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
workspace="$root/vendor/OpenNOW/native/opennow-streamer/Cargo.toml"
target="$root/build/opennow-qt/streamer-rust-target"
cargo test --manifest-path "$workspace" --target-dir "$target" --release -j 2 \
 -p opennow-streamer-protocol --lib log::tests
cargo test --manifest-path "$workspace" --target-dir "$target" --release -j 2 \
 -p opennow-streamer-core \
 --features opennow-streamer-platform/linux-ffmpeg-bundled,opennow-streamer-platform/linux-vaapi \
 --lib queue_drops::tests
cargo test --manifest-path "$workspace" --target-dir "$target" --release -j 2 \
 -p opennow-streamer-platform --features linux-ffmpeg-bundled,linux-vaapi \
 --lib media::tests::embedded_
