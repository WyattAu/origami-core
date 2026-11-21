//! Library wrapper: keeps the binary a thin shell so the logic is
//! unit-testable (and coverage counts it).
//!
//! Provides the `fold-check` pipeline: load FOLD files, run the
//! flat-foldability gates from `origami-core`, and report violations.

use std::fs;
use std::path::{Path, PathBuf};

use origami_core::foldability::FoldabilityViolation;

pub mod fold_json;
pub mod materials_toml;
pub mod rig;

pub use materials_toml::{MaterialEntry, MaterialLoadError};

pub use fold_json::FoldLoadError;

/// Outcome of checking one FOLD file.
#[derive(Debug)]
pub enum CheckOutcome {
    /// Loaded and passed every gate.
    Pass,
    /// Failed to load.
    Load(FoldLoadError),
    /// Loaded but violated at least one gate.
    Violations(Vec<FoldabilityViolation>),
}

/// One file's check result.
#[derive(Debug)]
pub struct FileReport {
    /// Path that was checked.
    pub path: PathBuf,
    /// What happened.
    pub outcome: CheckOutcome,
}

impl FileReport {
    /// True when the file loaded and passed every gate.
    #[must_use]
    pub fn is_pass(&self) -> bool {
        matches!(self.outcome, CheckOutcome::Pass)
    }
}

/// Loads and checks one FOLD file.
#[must_use]
pub fn check_file(path: &Path) -> FileReport {
    let outcome = match fold_json::load_fold(path) {
        Ok(pattern) => {
            let violations = origami_core::foldability::check_flat_foldability(&pattern);
            if violations.is_empty() {
                CheckOutcome::Pass
            } else {
                CheckOutcome::Violations(violations)
            }
        }
        Err(err) => CheckOutcome::Load(err),
    };
    FileReport {
        path: path.to_path_buf(),
        outcome,
    }
}

/// Collects `*.fold` files under `dir`, recursively.
///
/// # Errors
///
/// Returns the I/O error when the root cannot be read.
pub fn collect_fold_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current)?;
        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("fold") {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

/// Runs `fold-check` semantics over a file or directory.
///
/// # Errors
///
/// Returns a message when the path does not exist or directory walking
/// fails.
pub fn run_fold_check(path: &Path, all: bool) -> Result<Vec<FileReport>, String> {
    if all || path.is_dir() {
        let files =
            collect_fold_files(path).map_err(|e| format!("cannot walk {}: {e}", path.display()))?;
        if files.is_empty() {
            return Err(format!("no .fold files under {}", path.display()));
        }
        Ok(files.iter().map(|p| check_file(p)).collect())
    } else {
        if !path.exists() {
            return Err(format!("no such file: {}", path.display()));
        }
        Ok(vec![check_file(path)])
    }
}

/// Renders one violation as a human-readable line.
#[must_use]
pub fn describe_violation(violation: &FoldabilityViolation) -> String {
    match violation {
        FoldabilityViolation::Maekawa {
            vertex,
            mountains,
            valleys,
        } => format!(
            "vertex {vertex}: Maekawa violated (mountains {mountains}, valleys {valleys}, need |m-v| = 2)"
        ),
        FoldabilityViolation::Kawasaki { vertex, error_deg } => format!(
            "vertex {vertex}: Kawasaki violated (alternating-angle residual {error_deg:.3} deg)"
        ),
    }
}

/// True when every report passed.
#[must_use]
pub fn all_pass(reports: &[FileReport]) -> bool {
    reports.iter().all(FileReport::is_pass)
}

/// Runs `rig-check`: validates the rig provenance chain. Returns
/// violation lines (empty = clean).
///
/// # Errors
///
/// Returns a message when inputs cannot be loaded.
pub fn run_rig_check(materials_dir: &Path, index_path: &Path) -> Result<Vec<String>, String> {
    rig::validate_provenance(materials_dir, index_path)
}

/// Outcome payload for `fold-sim`.
#[derive(Debug)]
pub struct SimOutput {
    /// Human-readable report lines, in print order.
    pub lines: Vec<String>,
    /// True when the relaxation met both tolerances.
    pub converged: bool,
}

/// Loads a FOLD file, runs the bar-and-hinge relaxation, and renders
/// the report (text lines, or a single JSON payload).
///
/// # Errors
///
/// Returns a message when the file cannot be loaded.
pub fn run_fold_sim(
    path: &Path,
    fold_percent: f64,
    max_iterations: u32,
    relax: f64,
    step_cap: f64,
    json: bool,
) -> Result<SimOutput, String> {
    let pattern =
        fold_json::load_fold(path).map_err(|e| format!("cannot load {}: {e:?}", path.display()))?;
    let report = origami_core::sim::simulate_fold(
        &pattern,
        &origami_core::sim::SimParams {
            fold_percent,
            max_iterations,
            relax,
            step_cap,
            ..origami_core::sim::SimParams::default()
        },
    );
    let max_z = report
        .vertices
        .iter()
        .map(|p| p[2].abs())
        .fold(0.0_f64, f64::max);
    let lines = if json {
        let payload = serde_json::json!({
            "iterations": report.iterations,
            "converged": report.converged,
            "max_axial_strain": report.max_axial_strain,
            "mean_angle_error_rad": report.mean_angle_error_rad,
            "max_angle_error_rad": report.max_angle_error_rad,
            "total_energy": report.total_energy,
            "crease_angles": report.crease_angles,
            "max_abs_z": max_z,
            "vertices": report.vertices,
        });
        vec![payload.to_string()]
    } else {
        vec![
            format!("iterations: {}", report.iterations),
            format!("converged: {}", report.converged),
            format!("max axial strain: {:.5}", report.max_axial_strain),
            format!(
                "angle error (mean/max): {:.4}/{:.4} rad",
                report.mean_angle_error_rad, report.max_angle_error_rad
            ),
            format!("total energy: {:.6}", report.total_energy),
            format!("crease_angles: {}", serde_json::json!(report.crease_angles)),
            format!("max |z|: {max_z:.4}"),
        ]
    };
    Ok(SimOutput {
        lines,
        converged: report.converged,
    })
}

/// Outcome payload for `material-check`.#[derive(Debug)]
pub struct MaterialReport {
    /// Human-readable report lines, in print order.
    pub lines: Vec<String>,
    /// True when gate mode accepts this entry (or gate mode is off).
    pub gate_ok: bool,
}

/// Loads a materials entry and renders the estimate-grade mechanics
/// report (bending rigidity, crease stiffness, facet buckling, radius
/// verdict). In `gate` mode the report refuses estimate-grade entries
/// (REQ-O-005/007).
///
/// # Errors
///
/// Returns a message when the entry cannot be loaded.
pub fn run_material_check(
    path: &Path,
    crease_length_m: f64,
    panel_width_m: f64,
    k_factor: f64,
    radius_ratio: Option<f64>,
    gate: bool,
) -> Result<MaterialReport, String> {
    let entry = materials_toml::load_entry(path)
        .map_err(|e| format!("cannot load {}: {e:?}", path.display()))?;
    let model = entry.model;
    let mut lines = Vec::new();
    lines.push(format!("material: {} ({})", entry.id, entry.name));
    lines.push(format!(
        "provenance: {:?} ({})",
        model.provenance, entry.provenance
    ));
    let d = model.bending_rigidity();
    let k = model.crease_torsional_stiffness(crease_length_m, 1.0);
    let sigma = model.plate_buckling_stress(panel_width_m, k_factor);
    lines.push(format!("bending rigidity D = {d:.3e} N.m (estimate)"));
    lines.push(format!(
        "crease stiffness k = {k:.3e} N.m/rad at L = {crease_length_m} m (C = 1.0, estimate)"
    ));
    lines.push(format!(
        "facet buckling sigma_cr = {sigma:.3e} Pa at b = {panel_width_m} m, k_factor = {k_factor}"
    ));
    match radius_ratio {
        Some(r) => {
            let verdict = model.radius_check(r);
            lines.push(format!("radius verdict at r/t = {r}: {verdict:?}"));
        }
        None => lines.push("radius verdict: skipped (no --radius-rt given)".to_owned()),
    }
    let gate_ok = !gate || model.is_gate_eligible();
    if gate && gate_ok {
        lines.push("GATE: OK (measured)".to_owned());
    }
    if gate && !gate_ok {
        lines.push("GATE: REFUSED (estimate entries are not gate-eligible: REQ-O-005)".to_owned());
    }
    Ok(MaterialReport { lines, gate_ok })
}

/// Example smoke check kept from the template pattern: assignment parsing.
#[cfg(test)]
mod tests {
    use super::{all_pass, check_file, describe_violation, run_fold_check, run_material_check};
    use origami_core::foldability::FoldabilityViolation;
    use std::path::{Path, PathBuf};

    fn kapton_entry() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/materials/kapton-hn-125um.toml")
    }

    #[test]
    fn describe_violation_renders_both_arms() {
        let maekawa = FoldabilityViolation::Maekawa {
            vertex: 3,
            mountains: 2,
            valleys: 2,
        };
        let kawasaki = FoldabilityViolation::Kawasaki {
            vertex: 1,
            error_deg: -20.0,
        };
        assert!(describe_violation(&maekawa).contains("Maekawa violated"));
        assert!(describe_violation(&maekawa).contains("vertex 3"));
        assert!(describe_violation(&kawasaki).contains("Kawasaki violated"));
        assert!(describe_violation(&kawasaki).contains("-20.000"));
    }

    #[test]
    #[cfg(not(miri))]
    fn missing_file_fails_cleanly() {
        // Miri isolation forbids `open`: this filesystem-touching test is
        // skipped under miri (the loader's error paths stay covered by
        // design tests elsewhere).
        let report = check_file(Path::new("does/not/exist.fold"));
        assert!(!all_pass(std::slice::from_ref(&report)));
    }

    #[test]
    #[cfg(not(miri))]
    fn gate_report_refuses_estimate_and_omits_gate_line_without_gate() {
        let path = kapton_entry();
        let gated = run_material_check(&path, 0.05, 0.05, 4.0, Some(20.0), true);
        let Ok(r) = gated else {
            unreachable!("kapton entry must load")
        };
        assert!(!r.gate_ok, "estimate entry must refuse gate");
        let joined = r.lines.join("\n");
        assert!(joined.contains("GATE: REFUSED"), "lines: {joined}");
        assert!(joined.contains("radius verdict at r/t = 20: Fail"));
        assert!(joined.contains("bending rigidity"));

        let ungated = run_material_check(&path, 0.05, 0.05, 4.0, None, false);
        let Ok(r2) = ungated else {
            unreachable!("kapton entry must load")
        };
        assert!(r2.gate_ok, "no gate mode: always ok");
        let joined2 = r2.lines.join("\n");
        assert!(!joined2.contains("GATE:"), "no gate line without --gate");
        assert!(joined2.contains("radius verdict: skipped"));
    }

    #[test]
    #[cfg(not(miri))]
    fn run_fold_check_errors_on_missing_path() {
        let err = run_fold_check(Path::new("no/such/dir"), true);
        assert!(err.is_err(), "missing path must error");
        let Err(msg) = err else {
            unreachable!("error asserted")
        };
        assert!(msg.contains("cannot walk"), "{msg}");
    }

    #[test]
    #[cfg(not(miri))]
    fn empty_directory_yields_named_error() {
        let dir = std::env::temp_dir().join(format!(
            "origami-empty-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        ));
        let created = std::fs::create_dir_all(&dir);
        assert!(created.is_ok(), "temp dir create failed: {created:?}");
        let err = run_fold_check(&dir, true);
        assert!(err.is_err(), "empty dir must error");
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    #[cfg(not(miri))]
    fn load_error_message_cites_path() {
        let err = run_material_check(
            Path::new("no/such/entry.toml"),
            0.05,
            0.05,
            4.0,
            None,
            false,
        );
        assert!(err.is_err(), "missing entry must error");
        let Err(msg) = err else {
            unreachable!("error asserted")
        };
        assert!(msg.contains("cannot load"), "{msg}");
    }
}
