#!/usr/bin/env bash
# Same-machine determinism check (engineering-standards REPRODUCIBILITY.md):
# two from-scratch sanitized release builds must hash identically.
set -euo pipefail
cd "$(dirname "$0")/.."
EPOCH="$(git log -1 --pretty=%ct)"
wipe_and_build() {
  cargo clean
  env -i PATH="$PATH" HOME="$HOME" TZ=UTC LC_ALL=C SOURCE_DATE_EPOCH="$EPOCH" \
    cargo build --release --locked -p origami-core
  find target/release -maxdepth 1 -name '*.rlib' -o -name '*.rmeta' | sort | xargs sha256sum
}
wipe_and_build > /tmp/origami-repro-a.txt
wipe_and_build > /tmp/origami-repro-b.txt
if diff -q /tmp/origami-repro-a.txt /tmp/origami-repro-b.txt; then
  echo "reproducible: OK"
else
  echo "reproducible: MISMATCH"; diff /tmp/origami-repro-a.txt /tmp/origami-repro-b.txt; exit 1
fi
