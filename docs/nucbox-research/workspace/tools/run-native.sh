#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
qt="$root/.toolchain/qt/6.9.1/gcc_64"
umask 077
mkdir -p "$root/.runtime/config" "$root/.runtime/data" "$root/.runtime/cache" "$root/.runtime/logs"
chmod 700 "$root/.runtime"
export XDG_CONFIG_HOME="$root/.runtime/config"
export XDG_DATA_HOME="$root/.runtime/data"
export XDG_CACHE_HOME="$root/.runtime/cache"
export LD_LIBRARY_PATH="$qt/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export QT_PLUGIN_PATH="$qt/plugins"
export QML_IMPORT_PATH="$qt/qml"
cd "$root/build/opennow-qt"
exec ./opennow-qt "$@" >> "$root/.runtime/logs/launch.log" 2>&1
