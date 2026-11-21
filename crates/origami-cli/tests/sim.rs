//! Gates for `fold-sim` and the full-pattern relaxation
//! (roadmap Slice 2). Skipped under miri: loads files and runs
//! iteration-heavy solvers.

use std::path::Path;

#[test]
#[cfg(not(miri))]
fn miura_folds_out_of_plane_without_tearing() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../designs/examples/miura-ori/miura.fold");
    let pattern = origami_cli::fold_json::load_fold(&path);
    let Ok(pattern) = pattern else {
        unreachable!("seed design must load")
    };
    let params = origami_core::sim::SimParams {
        fold_percent: 0.5,
        max_iterations: 2000,
        ..origami_core::sim::SimParams::default()
    };
    let report = origami_core::sim::simulate_fold(&pattern, &params);
    assert!(
        report.max_axial_strain < 0.05,
        "Miura must fold without stretching: {}",
        report.max_axial_strain
    );
    let max_z = report
        .vertices
        .iter()
        .map(|p| p[2].abs())
        .fold(0.0_f64, f64::max);
    assert!(
        max_z > 0.05,
        "half-folded Miura must leave the plane: {max_z}"
    );
    let report2 = origami_core::sim::simulate_fold(&pattern, &params);
    assert_eq!(report.vertices, report2.vertices, "determinism");
}

#[test]
#[cfg(not(miri))]
fn fold_sim_cli_converges_on_a_small_pattern() {
    use std::process::Command;

    let dir = std::env::temp_dir().join(format!(
        "origami-fold-sim-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    let created = std::fs::create_dir_all(&dir);
    assert!(created.is_ok(), "temp dir: {created:?}");
    let fold = serde_json::json!({
        "file_spec": "fold",
        "file_version": 1.0,
        "vertices_coords": [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [0.0, 1.0]],
        "edges_vertices": [[0, 1], [1, 2], [2, 3], [3, 0], [0, 2]],
        "edges_assignments": ["B", "B", "B", "B", "M"],
        "faces_vertices": [[0, 1, 2], [0, 2, 3]]
    });
    let path = dir.join("quad.fold");
    let written = std::fs::write(&path, serde_json::to_string(&fold).unwrap_or_default());
    assert!(written.is_ok(), "fixture write: {written:?}");

    let out = Command::new(env!("CARGO_BIN_EXE_origami-cli"))
        .args([
            "fold-sim",
            path.to_string_lossy().as_ref(),
            "--fold-percent",
            "0.5",
            "--max-iterations",
            "1200",
        ])
        .output();
    assert!(out.is_ok(), "binary runs: {out:?}");
    let Ok(out) = out else {
        unreachable!("binary spawn asserted")
    };
    assert!(
        out.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("converged: true"), "stdout: {stdout}");

    let json_out = Command::new(env!("CARGO_BIN_EXE_origami-cli"))
        .args([
            "fold-sim",
            path.to_string_lossy().as_ref(),
            "--fold-percent",
            "0.5",
            "--max-iterations",
            "1200",
            "--json",
        ])
        .output();
    assert!(json_out.is_ok(), "binary runs: {json_out:?}");
    let Ok(jout) = json_out else {
        unreachable!("binary spawn asserted")
    };
    let parsed = serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&jout.stdout));
    assert!(parsed.is_ok(), "--json must emit valid JSON");
    let _ = std::fs::remove_dir_all(&dir);
}
