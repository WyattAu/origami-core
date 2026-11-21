//! Rig-run provenance validation (REQ-O-005): measured materials
//! entries must cite rig runs that exist in `data/rig/runs/index.toml`,
//! and every indexed run must reference a CSV that exists.

use std::fs;
use std::path::Path;

use toml::Value;

/// One indexed rig run.
#[derive(Debug, Clone)]
pub struct RunEntry {
    /// Run id (`R-NNN`), cited by materials entries.
    pub id: String,
    /// Materials entry id this run measured.
    pub material: String,
    /// CSV file name, relative to the index directory.
    pub csv: String,
}

/// Failures while loading the rig index.
#[derive(Debug)]
pub enum RigLoadError {
    /// The index file could not be read.
    Io(std::io::Error),
    /// The index is not valid TOML.
    Toml(toml::de::Error),
    /// A run entry is missing a required field.
    MissingOrBad(&'static str),
}

/// Loads `data/rig/runs/index.toml`.
///
/// # Errors
///
/// Returns [`RigLoadError`] on I/O, TOML, or schema problems.
pub fn load_index(path: &Path) -> Result<Vec<RunEntry>, RigLoadError> {
    let text = fs::read_to_string(path).map_err(RigLoadError::Io)?;
    let value = text.parse::<Value>().map_err(RigLoadError::Toml)?;
    let mut runs = Vec::new();
    let Some(entries) = value.get("run").and_then(Value::as_array) else {
        return Ok(runs);
    };
    for entry in entries {
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or(RigLoadError::MissingOrBad("id"))?
            .to_owned();
        let material = entry
            .get("material")
            .and_then(Value::as_str)
            .ok_or(RigLoadError::MissingOrBad("material"))?
            .to_owned();
        let csv = entry
            .get("csv")
            .and_then(Value::as_str)
            .ok_or(RigLoadError::MissingOrBad("csv"))?
            .to_owned();
        runs.push(RunEntry { id, material, csv });
    }
    Ok(runs)
}

/// Validates the rig provenance chain: indexed CSVs exist, and every
/// `measured` materials entry cites run ids present in the index.
/// Returns violation strings; empty means clean.
///
/// # Errors
///
/// Returns a message when inputs cannot be loaded.
pub fn validate_provenance(materials_dir: &Path, index_path: &Path) -> Result<Vec<String>, String> {
    let mut violations = Vec::new();
    let runs =
        load_index(index_path).map_err(|e| format!("index {}: {e:?}", index_path.display()))?;
    let index_dir = index_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    for run in &runs {
        let csv = index_dir.join(&run.csv);
        if !csv.exists() {
            violations.push(format!(
                "run {} cites missing csv {}",
                run.id,
                csv.display()
            ));
        }
    }
    let ids: Vec<&str> = runs.iter().map(|r| r.id.as_str()).collect();
    let Some(entries) = fs::read_dir(materials_dir).ok() else {
        return Err(format!(
            "cannot read materials dir {}",
            materials_dir.display()
        ));
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let loaded = super::materials_toml::load_entry(&path);
        let Ok(entry) = loaded else {
            violations.push(format!("materials entry unreadable: {}", path.display()));
            continue;
        };
        if entry.model.provenance == origami_core::materials::Provenance::Measured
            && entry.rig_runs.is_empty()
        {
            violations.push(format!("measured entry {} cites no rig runs", entry.id));
        }
        for run_id in &entry.rig_runs {
            if !ids.contains(&run_id.as_str()) {
                violations.push(format!("entry {} cites unknown rig run {run_id}", entry.id));
            }
        }
    }
    Ok(violations)
}
