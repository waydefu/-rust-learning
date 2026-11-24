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
trap 'rm -rf "$scratch"' EXIT
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
cp tools/site-tools/target/debug/site-write "$scratch/recording/site-write"
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
# Evidence stays outside dist and is retained as a CI artifact.
mkdir -p "$root/.migration-evidence"
cp "$scratch/record.json" "$scratch/original.json" "$scratch/rust.json" "$scratch/inputs.jsonl" "$root/.migration-evidence/"
cp -r "$scratch/recording/rust/app-cases" "$root/.migration-evidence/app-cases"
