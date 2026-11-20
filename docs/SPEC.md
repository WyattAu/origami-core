# SPEC: origami-core

status: draft
tier: k
updated: 2026-11-20

## 1. Scope

origami-core models flat crease patterns in the FOLD data format, checks
their structural and flat-foldability invariants, owns the civilian
design library under `designs/`, and defines the materials-database
entry format under `data/materials/`. Out of scope: 3D folded-state
simulation (planned bar-and-hinge module), SVG import, manufacturing
exports. Each out-of-scope item lands with its own REQ block.

## 2. FOLD data model (REQ-O-001)

The canonical subset: `vertices_coords` (f64 pairs), `edges_vertices`
(usize pairs), `edges_assignments` (tokens `B`, `M`, `V`, `U`),
`faces_vertices` (usize cycles, optional). JSON decoding lives in
`origami-cli::fold_json`; `origami-core::fold` holds the pure model and
rejects malformed structure at construction.

- REQ-O-001: a FOLD file with structurally invalid content (count
  mismatch, out-of-range index, degenerate edge) fails to load with a
  named error.
  Oracle: `cargo test -p origami-core structural_errors_surface_from_constructor`
  and `cargo test -p origami-cli`.

## 3. Flat-foldability gates (REQ-O-002, REQ-O-003)

Judged set: interior vertices with exactly four incident edges, all
assigned `M` or `V`. Everything else is unjudgeable and skipped.

- REQ-O-002: Maekawa gate: |mountains - valleys| = 2 at every judged
  vertex.
  Oracle: `cargo test -p origami-core` (`maekawa_only_failure_is_reported`).
- REQ-O-003: Kawasaki gate: sector angles a0..a3 in cyclic order satisfy
  |(a0 + a2) - (a1 + a3)| < 1e-6 degrees at every judged vertex.
  std builds only (`atan2`); no_std builds run Maekawa alone and say so.
  Oracle: `cargo test -p origami-core` (`kawasaki_only_failure_is_reported`).

Truth table (per judged vertex):

| Maekawa | Kawasaki | check_flat_foldability verdict |
|---|---|---|
| pass | pass | no violation (flat-foldable locally) |
| pass | fail | `Kawasaki { error_deg }` |
| fail | pass | `Maekawa { mountains, valleys }` |
| fail | fail | both violations listed |

No-oracle note: these are necessary conditions, not sufficient; global
foldability is out of scope until the kinematics module lands.

## 4. Design library (REQ-O-004)

Layout: one directory per design under `designs/`, containing at least
one `*.fold` and a `meta.toml` with `name`, `license` (default
`CC0-1.0`), `status`, `civilian = true`, `controlled = false`.

- REQ-O-004: every committed `designs/**/*.fold` loads and passes both
  gates of section 3 on CI.
  Oracle: `cargo test -p origami-cli --test designs` and
  `cargo run -p origami-cli -- fold-check --all designs`.

## 5. Materials database entries (REQ-O-005)

One TOML file per material batch under `data/materials/`. Required keys:
`id`, `name`, `status`, `provenance`, and physical properties with
explicit units in key names. `status` is `measured` (requires a rig run
id under `data/rig/`) or `estimate`.

- REQ-O-005: an entry with `status = "measured"` and no rig run id is
  invalid; `estimate` entries never feed gate decisions.
  No-oracle: schema validator lands with the first measured data
  (P0 rig); until then this requirement is enforced by review.

## 6. Placement policy (controlled designs)

Designs, load cases, simulations, or documentation that are
defense-related or finance-related are controlled technical data: they
belong in the Forgejo repo `origami-controlled`
(`git@forgejo:wyatt/origami-controlled.git`) and never in this public
repository. Every library design carries `civilian = true, controlled =
false` in `meta.toml`; absence of the flag means treat-as-controlled.

- REQ-O-006: every `meta.toml` in this repo asserts `civilian = true`.
  No-oracle: human policy gate (agent contract, AGENTS.md item 6);
  flag check joins the REQ-O-005 schema validator.
