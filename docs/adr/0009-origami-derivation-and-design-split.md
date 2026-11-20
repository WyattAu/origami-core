# ADR-0009: derive origami-core from OmniRust-template; design split

Status: accepted
Date: 2026-11-20

## Context

origami-core is a new repo: FOLD-native origami design core plus a
civilian design library. It has a hard data-governance constraint:
defense-related and finance-related designs are controlled technical
data and must never enter a public repository (ITAR/EAR and
equivalents). The estate contract (OMNI-CORE, ADR-0011) says new repos
derive from the Omni template family.

## Decision

1. Derivation by clone from OmniRust-template, then rebrand. Upstream
   copier content is not templated yet (copier.yml phase 1 reproduces
   the template byte-for-byte, it does not render names), so copier
   machinery (`copier.yml`, `.copier-answers.yml.jinja`,
   `COPIER-E2E-VERIFIED.md`) is removed here. Updates re-apply through
   `apply-standards.sh` until upstream templates content.
2. Crate layout: `origami-core` (L0 library, no_std-capable per
   ADR-0003), `origami-cli` (FOLD reader + `fold-check`), and the
   template service crate renamed `origami-service`, retained as demo
   code reserved for the future FEA-export job backend.
3. Domain modules in the L0 crate: `fold` (FOLD data model, pure) and
   `foldability` (Maekawa everywhere; Kawasaki behind the `std`
   feature, since sector angles need `atan2`). JSON decoding is
   std-only and lives in the CLI.
4. Design split: this repo carries `designs/` (civilian only, each
   `meta.toml` asserting `civilian = true`) and `data/materials/`
   (provenance-required TOML). Controlled designs live in the private
   Forgejo repo `origami-controlled`. The rule is stated in AGENTS.md
   (contract item 6), designs/README.md, and SPEC section 6.
5. CI/validation oracle for the library: a Rust integration test walks
   `designs/**/*.fold` and runs the same gates the CLI exposes, so the
   design library cannot regress without failing `make test`.

## Consequences

- Template example code (`text.rs` parser, parse bench, fuzz harness)
  stays until each is replaced by its domain counterpart; coverage
  counts them meanwhile.
- Kawasaki is skipped on no_std builds: documented in SPEC section 3.
- The controlled repo must exist before any design work that touches
  the pneumatic-origami program starts (P0 gate).
