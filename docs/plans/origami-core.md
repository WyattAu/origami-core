# origami-core

status: active
updated: 2026-11-20  rev: 3

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

## Next action
P0: order the fold-test-rig hardware (load cell + linear actuator); task 12 unblocks on delivery.

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
