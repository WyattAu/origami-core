//! Gates for the rig provenance chain (REQ-O-005) and the materials
//! TOML schema. Skipped under miri (filesystem).

use std::path::Path;

#[test]
#[cfg(not(miri))]
fn current_repo_state_validates() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let violations = origami_cli::run_rig_check(
        &root.join("data/materials"),
        &root.join("data/rig/runs/index.toml"),
    );
    assert!(violations.is_ok(), "rig-check must run: {violations:?}");
    let Ok(v) = violations else {
        unreachable!("run asserted")
    };
    assert!(
        v.is_empty(),
        "seed materials DB must be provenance-clean: {v:?}"
    );
}

#[test]
#[cfg(not(miri))]
fn measured_entry_without_run_id_is_a_violation() {
    let dir = std::env::temp_dir().join(format!(
        "origami-rig-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    let made = std::fs::create_dir_all(&dir);
    assert!(made.is_ok(), "temp dir: {made:?}");
    let mats = dir.join("materials");
    let made2 = std::fs::create_dir_all(&mats);
    assert!(made2.is_ok(), "materials dir: {made2:?}");
    let body = "id = \"fake\"\nname = \"Fake\"\nthickness_um = 100\n\
                youngs_modulus_gpa = 2.0\nstatus = \"measured\"\nprovenance = \"rig\"\n";
    let written = std::fs::write(mats.join("fake.toml"), body);
    assert!(written.is_ok(), "fixture: {written:?}");
    let index = dir.join("index.toml");
    let written2 = std::fs::write(&index, "");
    assert!(written2.is_ok(), "index: {written2:?}");

    let violations = origami_cli::run_rig_check(&mats, &index);
    let Ok(v) = violations else {
        unreachable!("rig-check asserted ok")
    };
    assert!(
        v.iter().any(|x| x.contains("cites no rig runs")),
        "violations: {v:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[cfg(not(miri))]
fn estimate_entries_never_trigger_the_gate() {
    // The committed DB is all-estimate: rig-check must be clean even
    // with zero runs in the index.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let violations = origami_cli::run_rig_check(
        &root.join("data/materials"),
        &root.join("data/rig/runs/index.toml"),
    );
    let Ok(v) = violations else {
        unreachable!("rig-check asserted ok")
    };
    assert!(v.is_empty(), "clean DB must validate: {v:?}");
}
