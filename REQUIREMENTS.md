# Requirements: origami-core

Numbered, testable requirements. Every requirement maps to at least one named
test/script check; every security-relevant check cites at least one
requirement. Doc comments on the implementing item carry `REQ-NNN` tags.

## Template (toolchain) requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-T-001 | `make ci` runs the same scripts CI runs; no CI-only steps | MUST |
| REQ-T-002 | Fresh clone → green `make ci` inside the devcontainer with zero manual steps | MUST |
| REQ-T-003 | Both devcontainer flavors (image, nix) build and reach green CI | MUST |
| REQ-T-004 | Monorepo layout demonstrates one internal dependency between packages | MUST |

## Security

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-T-100 | No secrets in the scaffold; `.env.example` pattern only | MUST |
| REQ-T-101 | CI workflows run with least-privilege `permissions:` blocks | MUST |

## Traceability Matrix

| Requirement | Check (script, test) | Property class |
|-------------|----------------------|----------------|
| REQ-T-001 | `make ci` == `.github/workflows/ci.yml` step list | structural |
| REQ-T-002 | devcontainer postCreate + `make ci` in CI devcontainer job | integration |

## Domain requirements (origami)

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-O-001 | Structurally invalid FOLD files fail to load with a named error | MUST |
| REQ-O-002 | Maekawa gate: \|M - V\| = 2 at every judged interior vertex | MUST |
| REQ-O-003 | Kawasaki gate: alternating sector-angle identity within 1e-6 deg (std builds) | MUST |
| REQ-O-004 | Every committed designs/**/*.fold passes REQ-O-001..003 on CI | MUST |
| REQ-O-005 | Materials entries carry provenance; measured requires a rig run id | MUST |
| REQ-O-006 | Every design meta.toml asserts civilian = true, controlled = false | MUST |
| REQ-O-007 | material-check --gate refuses estimate entries, accepts measured | MUST |
| REQ-O-008 | Mechanics: t^3 stiffness scaling; missing radius data = NoData, never pass | MUST |
| REQ-O-009 | Rates target mode: kinematic sign derivation, deterministic, strain-bounded on the seed | MUST |

## Domain traceability

| Requirement | Check (script, test) | Property class |
|-------------|----------------------|----------------|
| REQ-O-001 | crates/origami-core/tests/fold.rs::structural_errors_surface_from_constructor | unit |
| REQ-O-002 | crates/origami-core/tests/fold.rs::maekawa_only_failure_is_reported | unit |
| REQ-O-003 | crates/origami-core/tests/fold.rs::kawasaki_only_failure_is_reported | unit |
| REQ-O-004 | crates/origami-cli/tests/designs.rs + fold-check --all designs | integration |
| REQ-O-005 | review gate; TOML schema validator lands with P0 rig data | review |
| REQ-O-006 | review gate; agent contract AGENTS.md item 6 | review |
| REQ-O-007 | crates/origami-cli/tests/materials.rs (gate refusal + DB parse) | unit |
| REQ-O-008 | crates/origami-core/tests/materials.rs (scaling + verdicts) | unit |
| REQ-O-009 | crates/origami-cli/tests/sim.rs (Miura rates characterization) | integration |
