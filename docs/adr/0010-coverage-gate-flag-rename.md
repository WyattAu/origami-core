# ADR-0010: coverage gate flag rename after cargo-llvm-cov bitrot

Status: accepted
Date: 2026-11-20

## Context

CI `quality / coverage` failed: `scripts/coverage.sh` passes
`--fail-under` to `cargo llvm-cov`. cargo-llvm-cov 0.7 removed that flag
(0.9.1 here, latest on CI). This is upstream tool drift, not a policy
change; the estate bitrot posture (OMNI-CORE 7) exists to catch exactly
this.

## Decision

`scripts/coverage.sh` uses `--fail-under-lines` with the same 90
threshold. Line coverage is the metric the tier-a table (OMNI-CORE)
specifies for Rust (llvm-cov >=90% lines), so the gate keeps its
strength; only the flag spelling tracks the tool.

## Consequences

- No threshold change: >=90% lines stands.
- The script comment documents the mapping for the next flag rename.
- If a future llvm-cov renames again, the script fails loudly at CI and
  gets the same one-line fix.
