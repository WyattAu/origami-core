# AGENTS.md: instructions for AI coding agents

Project: **origami-core**: FOLD-native origami design core (data model,
foldability gates, materials engine) plus the civilian design library.

## Non-negotiable contract

1. **Scripts are canonical.** `scripts/*` do the work; `make` wraps them;
   CI runs the same scripts. NEVER add CI-only steps.
2. **One source of truth per concern.** Version pins, lint policy, and
   formatting each live in exactly one file. Never add a parallel config.
3. **Never weaken the gates.** Coverage thresholds, lints, and policy
   checks are lowered only via an ADR (`docs/adr/`) with justification.
4. **Never commit secrets.** Placeholders in `.env.example` style only.
5. **Decisions get ADRs.** Layout, gate, or tooling changes require
   `docs/adr/NNNN-*.md` (Status/Context/Decision/Consequences).
6. **Design placement rule (hard).** Any origami model, crease pattern,
   geometry, or derived data that is defense-related or finance-related
   goes to the Forgejo repo `origami-controlled`
   (`git@forgejo:wyatt/origami-controlled.git`), NEVER into this public
   repo. When in doubt, treat the design as controlled and place it
   there. See `designs/README.md` and SPEC section 6.

## Stack

- Rust workspace, edition 2021, MSRV 1.85:
  - `crates/origami-core`: L0 library; FOLD data model, foldability
    gates; no_std-capable (ADR-0003); zero workspace deps.
  - `crates/origami-cli`: `fold-check` binary; FOLD JSON reader lives here
    (std-only), composing on origami-core.
  - `crates/origami-service`: template axum demo; reserved for the future
    FEA-export job backend. Demo code until then.
- `designs/`: civilian design library; every `*.fold` is validated by
  `crates/origami-cli/tests/designs.rs` (REQ-O-004).
- `data/materials/`: materials DB entries; TOML, provenance-required.
- nix flake devShell owns system deps; `Cargo.lock` owns language deps.

## Commands

- `make ci`: contract + fmt-check + lint + test + vet (must be green
  before any push; CI runs exactly this).
- `make build` / `make test` / `make lint` / `make fmt` / `make fmt-check`
- `make coverage` / `make vet` / `make semver` / `make mutants`
- `make bench` (perf budget vs committed baseline) / `make bench-update`
- `make repro` (build twice, compare artifact hashes)
- `make contract` (Omni Core Contract structural checks)
- Design check by hand:
  `cargo run -p origami-cli -- fold-check --all designs`
- Mechanics report by hand:
  `cargo run -p origami-cli -- material-check data/materials/kapton-hn-125um.toml --gate`
- Regenerate the seed design: `python3 designs/tools/make_miura.py`

## Controlled-designs policy

This repo is public. Designs, simulations, load cases, or documentation
that would assist a weapons application (including anything derived from
the pneumatic-origami program) or a finance application are controlled:
place them in `origami-controlled` on Forgejo and reference them only
abstractly here. Export-control law (ITAR/EAR and equivalents) applies to
technical data, not just code: a crease pattern with arms intent is
controlled technical data. Public repo = civilian uses only.

## Conventions

- Commits: conventional prefixes (`feat:`, `fix:`, `test:`, `docs:`,
  `chore:`).
- Requirements are REQ-tagged and traced in `REQUIREMENTS.md`.
- Materials entries: `status` is either `measured` (with rig run id) or
  `estimate`; estimates never feed gate decisions.
- Estate-wide standards:
  [engineering-standards](https://github.com/WyattAu/engineering-standards)
  and
  [OMNI-CORE.md](https://github.com/WyattAu/engineering-standards/blob/main/OMNI-CORE.md).
- When unsure, read `ARCHITECTURE.md`: it explains *why*; this file
  explains *what*.
