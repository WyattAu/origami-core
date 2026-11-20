//! Gates for the materials DB and the mechanics report
//! (REQ-O-005, REQ-O-007).

use std::fs;
use std::path::{Path, PathBuf};

use origami_cli::{run_material_check, MaterialLoadError};

/// Lists `*.toml` files directly under a directory.
fn toml_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

#[test]
fn every_db_entry_parses_with_provenance() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/materials");
    let files = toml_files(&dir);
    assert!(
        files.len() >= 4,
        "materials DB should have its seed entries, found {}",
        files.len()
    );
    for file in &files {
        let loaded = origami_cli::materials_toml::load_entry(file);
        assert!(
            loaded.is_ok(),
            "entry must parse: {} -> {loaded:?}",
            file.display()
        );
        let Ok(entry) = loaded else {
            unreachable!("parse success asserted")
        };
        assert!(
            !entry.provenance.is_empty(),
            "{} needs provenance",
            entry.id
        );
        assert!(
            entry.model.thickness_m > 0.0,
            "{} needs thickness",
            entry.id
        );
    }
}

#[test]
fn db_entries_are_currently_estimates_and_gate_refuses_them() {
    // Snapshot of the current truth: every seed entry is estimate-grade,
    // so gate mode must refuse all of them (REQ-O-007). When rig data
    // lands, this test's fixtures (below) still cover both branches.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/materials");
    for file in toml_files(&dir) {
        let report = run_material_check(&file, 0.05, 0.05, 4.0, None, true);
        assert!(report.is_ok(), "report must build for {}", file.display());
        let Ok(r) = report else {
            unreachable!("report build asserted")
        };
        assert!(
            !r.gate_ok,
            "estimate entry must refuse gate: {}",
            file.display()
        );
    }
}

/// Writes a minimal entry with a chosen status into a unique temp file.
fn write_entry(status: &str, tag: u128) -> PathBuf {
    let body = format!(
        "id = \"fixture-{tag}\"\nname = \"Fixture\"\nclass = \"test\"\nthickness_um = 125\n\
         youngs_modulus_gpa = 2.5\nstatus = \"{status}\"\nprovenance = \"test fixture\"\n"
    );
    let path = std::env::temp_dir().join(format!("origami-material-fixture-{tag}.toml"));
    let written = fs::write(&path, &body);
    assert!(written.is_ok(), "temp fixture write failed: {written:?}");
    path
}

#[test]
fn gate_mode_accepts_measured_and_refuses_estimate() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let estimate = write_entry("estimate", now);
    let measured = write_entry("measured", now + 1);
    let refused = run_material_check(&estimate, 0.05, 0.05, 4.0, None, true);
    let accepted = run_material_check(&measured, 0.05, 0.05, 4.0, None, true);
    let Ok(r) = refused else {
        unreachable!("estimate report must build")
    };
    let Ok(a) = accepted else {
        unreachable!("measured report must build")
    };
    assert!(!r.gate_ok, "estimate must refuse gate");
    assert!(a.gate_ok, "measured must pass gate");
    let _ = fs::remove_file(&estimate);
    let _ = fs::remove_file(&measured);
}

#[test]
fn bad_status_is_a_named_error() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let bad = write_entry("guesstimate", now + 2);
    let loaded = origami_cli::materials_toml::load_entry(&bad);
    assert!(
        matches!(loaded, Err(MaterialLoadError::BadStatus(_))),
        "expected BadStatus, got {loaded:?}"
    );
    let _ = fs::remove_file(&bad);
}
