//! Library wrapper: keeps the binary a thin shell so the logic is
//! unit-testable (and coverage counts it).
//!
//! Provides the `fold-check` pipeline: load FOLD files, run the
//! flat-foldability gates from `origami-core`, and report violations.

use std::fs;
use std::path::{Path, PathBuf};

use origami_core::foldability::FoldabilityViolation;

pub mod fold_json;

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

/// Example smoke check kept from the template pattern: assignment parsing.
#[cfg(test)]
mod tests {
    use super::{all_pass, check_file};
    use std::path::Path;

    #[test]
    fn missing_file_fails_cleanly() {
        let report = check_file(Path::new("does/not/exist.fold"));
        assert!(!all_pass(std::slice::from_ref(&report)));
    }
}
