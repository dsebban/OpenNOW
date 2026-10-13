#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p research
exa-cli search 'GeForce NOW open source native Linux client reverse engineered protocol GitHub' --num-results 5 > research/clients.json
exa-cli search 'GeForce NOW protocol RTSP RTP WebRTC reverse engineering open source' --num-results 5 > research/protocol.json
exa-cli search 'GeForce NOW OpenNow native client hardware decoding Linux Moonlight' --num-results 5 > research/native.json
exa-cli contents https://github.com/OpenCloudGaming/OpenNOW https://opennow.zortos.me/reference/webrtc/ --max-characters 4500 > research/opennow-sources.json
