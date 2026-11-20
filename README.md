# origami-core

FOLD-native origami design core: crease-pattern data model, flat-
foldability gates, a validated civilian design library, and the start of
a provenance-tracked materials database. Part of the
[WyattAu Omni family](https://github.com/WyattAu?tab=repositories&q=omni-)
(Estate tier-a gates, Apache-2.0).

## Design placement rule (read first)

**Defense-related or finance-related designs never enter this repo.**
They go to the private Forgejo repo `origami-controlled`
(`git@forgejo:wyatt/origami-controlled.git`). When in doubt, treat the
design as controlled. Details: `designs/README.md`, SPEC section 6.

## What is inside

```
crates/origami-core      L0 library: FOLD model, Maekawa + Kawasaki gates (no_std-capable)
crates/origami-cli       fold-check: validates .fold files and the whole design library
crates/origami-service   template axum demo (future FEA job backend)
designs/                 civilian design library (CC0-1.0 by default) + tools
data/materials/          materials DB entries (provenance-required TOML)
docs/plans/              plan ledger (crash-proof task state)
docs/adr/                decision log (start at 0000)
xtask/ fuzz/ supply-chain/ scripts/ + Makefile   estate gates
```

## Quick start

Pick a door; all three resolve to identical toolchains:

| Door | Command |
|---|---|
| nix + direnv (host) | `direnv allow` |
| Devcontainer (image) | VS Code: Reopen in Container |
| Devcontainer (nix)  | palette: Rebuild in Container, pick `.devcontainer/nix/` |

No nix, no docker? `./scripts/bootstrap.sh` prints the manual path.

Then: `make ci` must be green before your first push. CI runs exactly this.

## Make targets

| Target | Gate |
|---|---|
| `make build` | `cargo build --workspace --all-features --locked` |
| `make test` | `cargo nextest run --workspace --all-features --locked` |
| `make lint` | clippy, tier-a posture (`-D warnings`, pedantic, panic family denied) |
| `make fmt` / `fmt-check` | rustfmt |
| `make coverage` | llvm-cov, >=90% lines (process surfaces excluded: ADR-0005) |
| `make vet` | cargo-vet supply-chain check |
| `make semver` | semver-checks vs latest tag |
| `make mutants` | cargo-mutants (weekly policy, MUTATION.md) |
| `make contract` | Omni Core Contract structural checks |
| `make ci` | contract + fmt-check + lint + test + vet |

## The design library

Every design is a directory under `designs/` with `*.fold` (FOLD JSON)
plus `meta.toml`. The repo's own test suite is the gate: each committed
`.fold` must load and pass Maekawa + Kawasaki
(`crates/origami-cli/tests/designs.rs`). Add a design by hand or generate
the seed:

```
python3 designs/tools/make_miura.py
cargo run -p origami-cli -- fold-check --all designs
```

## Materials database

`data/materials/*.toml`, one file per material batch. Every entry
declares `status = "measured"` (with a rig run id under `data/rig/`) or
`status = "estimate"`. Estimates exist to be replaced: P0 benchtop
fold-test rig spec is `docs/P0-RIG.md`. Crease fatigue and dynamic
minimum bend radius are the two properties that matter most for real
designs; until the rig produces numbers, every value in this repo is an
estimate.

## CI gates (tier a)

| Gate | Threshold |
|---|---|
| build/test `--locked` (all-features + no-default-features) | green |
| clippy `-D warnings` + pedantic + unwrap/indexing/panic denied | green |
| llvm-cov | >= 90% lines (process surfaces excluded, ADR-0005) |
| loom model checking (`--cfg loom`) | green |
| miri (nightly) | green |
| cargo-deny (advisories/licenses/bans) | green |
| cargo-vet | green (weekly auto-refresh) |
| MSRV | 1.85 (dedicated CI leg) |
| design library | every `designs/**/*.fold` passes foldability gates |

## Release flow

release-plz PRs version bumps + CHANGELOG on push to main; merging
publishes to crates.io. Secrets needed: `RELEASE_PLZ_TOKEN`,
`CARGO_REGISTRY_TOKEN` (ADR-0004). Semver-checks skips until the first
tag exists.

## Performance budgets and determinism

`make bench` measures against the committed `bench/baseline.tsv`
(threshold: `ORIGAMI_BENCH_THRESHOLD_PCT`, default 10%). `make repro`
builds twice from a clean state and compares artifact hashes
(ADR-0007, ADR-0008).

## License

Apache-2.0; commercial use expressly permitted. Design-library files
default to CC0-1.0 (see `designs/README.md`).
