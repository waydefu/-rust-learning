#!/usr/bin/env bash
# Frozen migration oracle: only historical JS code is extracted from the pin.
# Current Rust source, course and static assets are always taken from HEAD.
set -euo pipefail
cd "$(dirname "$0")/../.."
root="$PWD"
pin="b2c5abe32610f6f098e46ec2501c9514fed2066d"
scratch="$(mktemp -d)"
reference="$scratch/reference"
finish() {
  status=$?
  trap - EXIT
  set +e
  mkdir -p "$root/.migration-evidence"
  if [ -d "$reference/.migration-evidence" ]; then
    cp -a "$reference/.migration-evidence/." "$root/.migration-evidence/"
  fi
  rm -rf -- "$scratch"
  exit "$status"
}
trap finish EXIT
if ! git cat-file -e "$pin^{commit}" 2>/dev/null; then
  git fetch --no-tags --depth=1 origin "$pin"
fi
git cat-file -e "$pin^{commit}"
mkdir "$reference"
git archive "$pin" src scripts tests package.json package-lock.json playwright.config.mjs | tar -x -C "$reference"
# Replace historical assets, tools and content with the current committed sources.
rm -rf -- "$reference/src/assets"
git archive HEAD tools content examples src/assets rust-toolchain.toml | tar -x -C "$reference"
(
  cd "$reference"
  npm ci
  npm audit --audit-level=high
  export VERIFY_SITE_WRITE=1
  bash scripts/migration/replay-site-write.sh
  BASE_PATH=/ node scripts/migration/compare-full-app.mjs
  BASE_PATH=/ node scripts/migration/compare-site-check.mjs
  node scripts/migration/compare-examples.mjs
  BASE_PATH=/-rust-learning/ node scripts/migration/compare-full-app.mjs
  BASE_PATH=/-rust-learning/ node scripts/migration/compare-site-check.mjs
  # Preserve the historical comparator and Unicode tests as migration evidence.
  npx playwright test tests/metadata.spec.mjs tests/native-render.spec.mjs
)
printf '%s\n' "$pin" > "$reference/.migration-evidence/reference-commit.txt"
