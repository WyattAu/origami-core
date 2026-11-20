# Roadmap: origami-core software

status: active
updated: 2026-11-20
rule: hardware work is gated; see docs/P0-RIG.md. Software slices land in
order; each slice has an acceptance gate before the next starts.

## Slice 0: scaffold + gates (DONE)

- [x] OmniRust derivation, rebrand, tier-a gates green
- [x] FOLD data model + flat-foldability gates (Maekawa, Kawasaki)
- [x] `fold-check` CLI + design-library CI oracle
- [x] Miura seed design, generated + verified
- [x] Materials DB format with provenance requirement
- [x] Controlled-designs split (public vs forgejo `origami-controlled`)

## Slice 1: materials engine + mechanics estimates (CURRENT)

Ship estimate-grade mechanics so every DB entry can answer design
questions before rig data exists. Every output labeled; `--gate` mode
refuses estimates (REQ-O-005/REQ-O-007).

- [x] `origami-core::materials`: bending rigidity D = E·t³/(12(1-ν²)),
  crease torsional stiffness k = C·D·L (calibration constant C exposed,
  1.0 default), plate-strip buckling σ_cr = k·π²·D/(b²·t),
  dynamic radius verdicts (Pass/Fail/NoData)
- [ ] `material-check` CLI subcommand with `--gate` refusal
- [ ] SPEC + REQUIREMENTS updates (REQ-O-007)
- [ ] tests: t³ scaling, hand-computed D and σ_cr, radius verdicts,
  gate refusal; all DB entries parse
- Acceptance gate: clippy + tests green; `material-check --gate` exits
  non-zero on every current (estimate) entry

## Slice 2: bar-and-hinge folded-state simulation

The core value: fold a CP in 3D, get energy landscape + stiffness vs
fold angle, bistability detection. Research-grade references: Ghassaei
GPU solver (MIT), Schenk & Guest bar-and-hinge, zzhuyii compliant-crease
model (license check pending).

- [ ] Kinematic folding engine: rotate facets about creases for
  rigid-foldable patterns; fold-angle vector as the state
- [ ] Bar-and-hinge energy: axial bars + crease torsion springs
  (calibrated from slice 1 formulas; rig constants slot in later)
- [ ] Energy landscape scan over fold parameter; local minima = stable
  states; barrier height = snap-through threshold
- [ ] Self-intersection check on folded state (approximate first:
  facet-pair AABB then triangle test)
- [ ] CLI: `fold-sim <file.fold> [--fold-percent N]` outputs folded
  coords + energy + stability report
- Acceptance gate: Miura seed folds to a positive dihedral; energy
  monotone per vertex for single-vertex fixtures; no NaN
- no_std not required: simulation runs behind the `std` feature

## Slice 3: import/export surface

- [ ] SVG import (crease-pattern drawings; color/opacity conventions of
  Origami Simulator), fuzz target for both parsers before any web
  exposure (THREAT-MODEL commitment)
- [ ] Manufacturing export: DXF/SVG with kerf compensation, score
  lines, relief cuts at crease junctions; per-material laser params in
  the materials DB
- [ ] CalculiX export: shell mesh + crease hinge connectors from folded
  states (`fold-export-fea`); no in-house FEA (ADR policy)
- Acceptance gate: round-trip SVG import; DXF opens in a reference
  viewer; CalculiX runs the exported deck on a sample CP

## Slice 4: web app (editor + sim UI)

- [ ] Core compiled to WASM; TS UI; fold editor + 3D preview +
  materials selector + load-case summary
- [ ] Public deployment only after parser fuzzing + size caps
- Acceptance gate: browser folds the Miura seed in real time; all
  gates green in CI including wasm target

## Slice 5: design repo productization

- [ ] Design index site generated from `designs/` metadata
- [ ] Pro version: measured materials DB, stress reports, manufacturing
  exports (open-core boundary per business decision)
- [ ] Contribution flow: PR template checks civilian flag + gates
- Acceptance gate: external contribution passes end-to-end without
  maintainer intervention

## Explicitly deferred (gated)

- Hardware rig build + measurement: gated on operator approval of the
  completed plan in docs/P0-RIG.md
- Anything defense/finance-derived: forgejo `origami-controlled` only
