#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
root="$PWD"
# Pin the upstream release and the SHA-256 supplied by the GitHub release API.
# No setup/MCP registration, installation into the host, credentials or deploy.
version=v4.0.0
asset="rust-anything-$version-x86_64-unknown-linux-gnu"
digest=6ca6e017f7a1dc1f870236c3200c6163ee9dfb331eebe541e9b28ba60a98d4d2
scratch="$(mktemp -d)"
# Preserve synthetic test evidence even when recording or replay fails.
finish() {
  status=$?
  trap - EXIT
  set +e
  mkdir -p "$root/.migration-evidence"
  for file in record.json original.json rust.json inputs.jsonl full-record.json full-original.json full-rust.json full-inputs.jsonl; do
    if [ -f "$scratch/$file" ]; then cp "$scratch/$file" "$root/.migration-evidence/"; fi
  done
  if [ -d "$scratch/recording/rust/app-cases" ]; then
    cp -r "$scratch/recording/rust/app-cases" "$root/.migration-evidence/app-cases"
  fi
  if [ -d "$scratch/full-app/rust/app-cases" ]; then
    cp -r "$scratch/full-app/rust/app-cases" "$root/.migration-evidence/full-app-cases"
  fi
  rm -rf "$scratch"
  exit "$status"
}
trap finish EXIT
mkdir -p "$scratch/tool" "$scratch/recording"
curl --fail --silent --show-error --location --retry 3 --max-time 90 \
  "https://github.com/erkamyaman/rust-anything/releases/download/$version/$asset.tar.gz" \
  -o "$scratch/tool/$asset.tar.gz"
printf '%s  %s\n' "$digest" "$scratch/tool/$asset.tar.gz" | sha256sum --check -
tar -xzf "$scratch/tool/$asset.tar.gz" -C "$scratch/tool" "$asset/rust-anything"
tool="$scratch/tool/$asset/rust-anything"
"$tool" --version
cargo build --locked --manifest-path tools/site-tools/Cargo.toml --bin site-write
cp scripts/migration/legacy-site-write.mjs "$scratch/recording/original.mjs"
cp tools/target/debug/site-write "$scratch/recording/site-write"
node scripts/migration/write-cases.mjs > "$scratch/inputs.jsonl"
# The command runs inside a fresh sandbox per case. The input corpus includes
# invalid packets; there is no arbitrary shell command or production checkout.
"$tool" app record "$scratch/recording" --name site-write --inputs "$scratch/inputs.jsonl" \
  --no-traps --json -- node original.mjs > "$scratch/record.json"
cat "$scratch/record.json"
expected="$(wc -l < "$scratch/inputs.jsonl")"
jq -e --argjson n "$expected" '.cases == $n and (.noisy | length) == 0 and (.unstable | length) == 0 and (.timed_out | length) == 0' "$scratch/record.json"
"$tool" app replay "$scratch/recording" --json > "$scratch/original.json"
"$tool" app replay "$scratch/recording" --build ./site-write --json > "$scratch/rust.json"
for result in original rust; do
  cat "$scratch/$result.json"
  jq -e --argjson n "$expected" '.total == $n and .passed == $n and .skipped == 0 and (.failures | length) == 0 and ((.fallback.cases // 0) == 0)' "$scratch/$result.json"
done
cat "$scratch/recording/rust/app-cases/replay-report.md"

# Full CLI black-box observation in a separate recording directory. Each
# observed run starts from a fresh copy of the actual source tree.
mkdir -p "$scratch/full-app"
cp scripts/migration/observe-full-cli.mjs "$scratch/full-app/observe.mjs"
node scripts/migration/full-cli-cases.mjs > "$scratch/full-inputs.jsonl"
export RUST_LEARNING_REPLAY_ROOT="$root"
export RUST_LEARNING_REPLAY_TARGET="$scratch/full-target"
"$tool" app record "$scratch/full-app" --name rust-learning-cli --inputs "$scratch/full-inputs.jsonl" \
  --no-traps --timeout 240 --json -- node observe.mjs legacy > "$scratch/full-record.json"
cat "$scratch/full-record.json"
expected="$(wc -l < "$scratch/full-inputs.jsonl")"
jq -e --argjson n "$expected" '.cases == $n and (.noisy | length) == 0 and (.unstable | length) == 0 and (.timed_out | length) == 0' "$scratch/full-record.json"
"$tool" app replay "$scratch/full-app" --json > "$scratch/full-original.json"
"$tool" app replay "$scratch/full-app" --build "node observe.mjs native" --json > "$scratch/full-rust.json"
for result in full-original full-rust; do
  cat "$scratch/$result.json"
  jq -e --argjson n "$expected" '.total == $n and .passed == $n and .skipped == 0 and (.failures | length) == 0 and ((.fallback.cases // 0) == 0)' "$scratch/$result.json"
done
cat "$scratch/full-app/rust/app-cases/replay-report.md"
