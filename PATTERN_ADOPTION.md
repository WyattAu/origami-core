# Pattern adoption

| Pattern | Status | Evidence |
|---|---|---|
| T1 | adopted | crates/origami-core/src/fold.rs:26 (EdgeAssignment::parse from tokens), crates/origami-core/src/fold.rs:104 (validate returns a typed StructureError) |
| T13 | adopted | crates/origami-core/src/fold.rs:65 (no_std crease-pattern core), crates/origami-cli/src/main.rs:1 (fold-check CLI as the shell) |
| P64 | adopted | crates/origami-core/tests/parse.rs:7 (proptest suites over parsing) |
