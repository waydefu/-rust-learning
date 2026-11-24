#!/usr/bin/env bash
# Explicit opt-in execution boundary. No host mounts, Docker socket or tokens.
set -euo pipefail
cd "$(dirname "$0")/../.."
image="rust-learning-example-sandbox:local"
docker build --file tools/sandbox/Dockerfile --tag "$image" .
docker run --rm --init \
  --network none --read-only --cap-drop ALL \
  --security-opt no-new-privileges \
  --pids-limit 256 --memory 3g --memory-swap 3g --cpus 2 \
  --ulimit nofile=1024:1024 \
  --tmpfs /tmp:rw,exec,nosuid,nodev,size=2g,mode=1777 \
  "$image"
