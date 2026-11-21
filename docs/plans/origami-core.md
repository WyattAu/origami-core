# origami-core

status: active
updated: 2026-11-21  rev: 6

## State
| # | Task | Status | Evidence | Next |
|---|------|--------|----------|------|
| 1 | Derive repo from OmniRust-template | done | git log (rebrand commit) | ; |
| 2 | Rebrand crates omni-* to origami-* | done | crates/ layout; grep omni shows estate refs only | ; |
| 3 | SPEC.md (tier k) + truth table | done | docs/SPEC.md | ; |
| 4 | Design library skeleton (designs/) | done | designs/README.md + examples/miura-ori | ; |
| 5 | Materials DB draft entries | done | data/materials/*.toml (4x status=estimate) | ; |
| 6 | ADR 0009: derivation + design-split policy | done | docs/adr/0009-origami-derivation-and-design-split.md | ; |
| 7 | Build + tests green locally | done | cargo test 17 passed; clippy clean (this commit) | ; |
| 8 | Publish public repo (GitHub) | done | origin/main 776cd7b+; github.com/WyattAu/origami-core | ; |
| 9 | Controlled repo (forgejo) scaffold | done | pushed origin/main 2152d46 | ; |
| 10 | estate.yml registration | done | engineering-standards 274cba1 pushed | ; |
| 11 | P0: fold-test rig spec | done | docs/P0-RIG.md | order hardware |
| 12 | P0: measure 3 materials | blocked | bench hardware | order load cell + actuator |
| 13 | Domain code: fold model + gates + CLI | done | crates/origami-core/src/{fold,foldability}.rs; fold-check | ; |
| 14 | docs gate (doc-lint tier k) | done | doc-lint: PASS tier k (em-dash sweep) | CI confirms on push |
| 15 | Verify clippy + tests clean | done | clippy 0 issues; 17 passed / 0 failed | ; |
| 16 | Full software roadmap (P1-P4, gated) | done | docs/plans/roadmap.md | ; |
| 17 | Complete rig plan + BOM, DO-NOT-ORDER gate | done | docs/P0-RIG.md (BOM 229-510 USD est.) | BLOCKED ON: operator approves plan |
| 18 | Materials engine (core math + material-check CLI) | done | crates/origami-core/src/materials.rs; tests/materials.rs | ; |
| 19 | REQ-O-007/008: gate mode refuses estimates | done | SPEC 5/5a; REQUIREMENTS; cli tests | ; |
| 20 | Dogfood audit + fixes | done | .cargo fix 819e751; estate.yml 841231c; dependabot security on; contract OK; spec-audit PASS | ; |
| 21 | CI triage + fixes (miri/deny/coverage/docs) | done | tokio 1.54.1 (RUSTSEC-2026-0386); coverage.sh --fail-under-lines + ADR-0005 exclusions; miri cfg-gate; Pages enabled | ; |
| 22 | Coverage >= 90% gate | done | TOTAL 93.88% lines (local llvm-cov, rev 38d0b1f) | ; |
| 23 | Slice 2 v1: PBD folded-state solver | done | crates/origami-core/src/sim.rs; 46 tests green; committed 38d0b1f | fold-rate-based targets = next |
| 24 | Vet refresh for toml/tokio deps | done | supply-chain imports+exemptions; cargo vet clean | ; |
| 25 | Slice 2 v2: kinematic fold-rate targets (Tachi) | pending | RESEARCH_LOG R-001 | replace label targets with rate solve |
| 26 | Slice 2 GN alternative (damped LM + line search) | blocked | RESEARCH_LOG R-002 | BLOCKED ON: research design of damping schedule |
| 22 | Coverage >= 90% gate | done | TOTAL 93.72% lines (local llvm-cov) | ; |
| 12 | P0: measure 3 materials | blocked | docs/P0-RIG.md APPROVED by operator 2026-11-21 | BLOCKED ON: physical procurement + assembly (operator) |
| 27 | Rig plan approval recorded | done | docs/P0-RIG.md approval block | ; |
| 28 | Rig data tooling: rig-check + index schema | pending | crates/origami-cli/src/rig.rs | implement (REQ-O-005 oracle) |
| 25 | Slice 2 v2: kinematic fold-rate targets (Tachi) | active | crates/origami-core/src/sim.rs | implement fold_rates + TargetMode |

## Next action
Software: implement task 25 (fold-rate targets) + task 28 (rig-check oracle), verify, ship.

## Policy: controlled designs (hard rule)
Any origami model that is defense-related or finance-related goes to the
Forgejo repo `origami-controlled` (git@forgejo:wyatt/origami-controlled.git),
never into this public repo. This rule is restated in AGENTS.md and
designs/README.md. When in doubt, place the design in the controlled repo.

## Blocked
- task 12: bench hardware (load cell, linear actuator) not yet ordered.

## Log
- 2026-11-20 derived from OmniRust-template (clone-based, copier content
  not templated upstream yet); ledger created; rev 1.
- 2026-11-20 rev 2: domain code (fold/foldability/fold-check), Miura seed
  design generated + gate-verified, materials DB drafts, SPEC + ADR-0009 +
  P0 rig spec, AGENTS placement rule, controlled repo pushed to Forgejo,
  estate.yml registered.
- 2026-11-20 rev 3: clippy + tests green (17/0), doc-lint PASS tier k,
  published to github.com/WyattAu/origami-core (shallow-clone push fixed
  via fetch --unshallow).
- 2026-11-20 rev 4: materials engine slice (estimate-grade mechanics +
  material-check --gate), roadmap written, rig plan completed + gated on
  operator approval, dogfood fixes (.cargo absolute path, estate.yml
  status vocab, dependabot security updates on).

- 2026-11-21 rev 5: Slice 2 v1 (PBD solver, fold-sim CLI, sim tests); CI triage fixes; vet refresh; GN attempt rejected (R-002).
- 2026-11-21 rev 6: operator approved rig plan (proceed all); task 25 + 28 started.
