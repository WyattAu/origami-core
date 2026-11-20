#!/usr/bin/env bash
# llvm-cov gate — tier a threshold (>=90% lines). Override: ./scripts/coverage.sh 80
# Flag note: cargo-llvm-cov >=0.7 removed `--fail-under`; the equivalent
# line-coverage gate is `--fail-under-lines` (ADR-0010: same threshold,
# renamed flag after upstream bitrot).
# Process surfaces (bins, xtask) are excluded per ADR-0005.
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo llvm-cov --workspace --all-features --summary-only \
  --fail-under-lines "${1:-90}" \
  --ignore-filename-regex 'src/main\.rs$|xtask/src'
