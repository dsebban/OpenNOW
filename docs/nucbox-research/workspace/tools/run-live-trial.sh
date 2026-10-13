#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
umask 077
mkdir -p "$root/.runtime/hillclimb"
unset OPENNOW_EMBEDDED_PUBLISH_POLL_MS OPENNOW_CONTINUOUS_VIDEO_UPDATE OPENNOW_NATIVE_VIDEO_BACKEND OPENNOW_EMBEDDED_READY_WAKE OPENNOW_VULKAN_DEFER_SNAPSHOT_WAIT QSG_RENDER_LOOP QT_QPA_UPDATE_IDLE_TIME OPENNOW_VULKAN_FRAME_TIMING OPENNOW_FRAME_STAGE_LOG OPENNOW_EMBEDDED_PUBLISH_PACED OPENNOW_EMBEDDED_PUBLISH_CUSHION_US OPENNOW_EMBEDDED_PUBLISH_MINGAP_US OPENNOW_EMBEDDED_PUBLISH_GRID_US OPENNOW_SUBMIT_PACED OPENNOW_SUBMIT_CUSHION_US OPENNOW_SUBMIT_MAXHOLD_US
export OPENNOW_LIVE_TELEMETRY="$root/.runtime/hillclimb/qt-presentation.jsonl"
case "${1:-baseline}" in
  baseline) ;;
  poll1) export OPENNOW_EMBEDDED_PUBLISH_POLL_MS=1 ;;
  continuous) export OPENNOW_CONTINUOUS_VIDEO_UPDATE=1 ;;
  vaapi) export OPENNOW_NATIVE_VIDEO_BACKEND=vaapi ;;
  basic) export QSG_RENDER_LOOP=basic ;;
  idle0) export QT_QPA_UPDATE_IDLE_TIME=0 ;;
  deferwait) export OPENNOW_VULKAN_DEFER_SNAPSHOT_WAIT=1 ;;
  frametime) export OPENNOW_VULKAN_FRAME_TIMING=1 ;;
  stagelog) export OPENNOW_FRAME_STAGE_LOG=1 ;;
  paced) export OPENNOW_EMBEDDED_PUBLISH_PACED=1 ;;
  pacedlog) export OPENNOW_EMBEDDED_PUBLISH_PACED=1 OPENNOW_FRAME_STAGE_LOG=1 ;;
  subpace) export OPENNOW_SUBMIT_PACED=1 ;;
  subpacelog) export OPENNOW_SUBMIT_PACED=1 OPENNOW_FRAME_STAGE_LOG=1 ;;
  *) echo 'Usage: run-live-trial.sh baseline|poll1|continuous|vaapi|basic|idle0|deferwait|frametime|stagelog|paced|pacedlog|subpace|subpacelog' >&2; exit 2 ;;
esac
if [ "$#" -gt 0 ]; then shift; fi
exec "$root/tools/run-native.sh" "$@"
