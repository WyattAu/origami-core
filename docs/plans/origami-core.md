# origami-core

status: active
updated: 2026-11-20  rev: 2

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
| 8 | Publish public repo (GitHub) | active | local commits | gh repo create + push |
| 9 | Controlled repo (forgejo) scaffold | done | pushed origin/main 2152d46 | ; |
| 10 | estate.yml registration | done | engineering-standards 274cba1 pushed | ; |
| 11 | P0: fold-test rig spec | done | docs/P0-RIG.md | order hardware |
| 12 | P0: measure 3 materials | blocked | bench hardware | order load cell + actuator |
| 13 | Domain code: fold model + gates + CLI | done | crates/origami-core/src/{fold,foldability}.rs; fold-check | ; |
| 14 | docs gate (doc-lint tier k) | active | em-dashes fixed; rerun pending | rerun after publish |

## Next action
Publish to GitHub (gh repo create WyattAu/origami-core --public --push).

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
