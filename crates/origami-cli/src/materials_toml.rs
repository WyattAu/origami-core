//! Materials TOML loader for `data/materials/` entries.
//!
//! Untrusted-input note: TOML comes from the repo and from users; keep
//! handling defensive (typed errors, no panics). See THREAT-MODEL.md.

use std::fs;
use std::path::Path;

use origami_core::materials::{MaterialModel, Provenance};
use toml::Value;

/// One loaded materials entry plus its metadata.
#[derive(Debug, Clone)]
pub struct MaterialEntry {
    /// Entry id (the TOML `id` key).
    pub id: String,
    /// Display name.
    pub name: String,
    /// Normalized SI model for the mechanics functions.
    pub model: MaterialModel,
    /// Free-text provenance note (required by REQ-O-005).
    pub provenance: String,
    /// Rig run ids backing a `measured` entry (REQ-O-005).
    pub rig_runs: Vec<String>,
}

/// Failures while loading a materials entry.
#[derive(Debug)]
pub enum MaterialLoadError {
    /// The file could not be read.
    Io(std::io::Error),
    /// The content is not valid TOML.
    Toml(toml::de::Error),
    /// A required key is missing or has the wrong type.
    MissingOrBad(&'static str),
    /// `status` outside the vocabulary {measured, estimate}.
    BadStatus(String),
}

/// Loads one materials entry.
///
/// # Errors
///
/// Returns [`MaterialLoadError`] on I/O, TOML, or schema problems.
pub fn load_entry(path: &Path) -> Result<MaterialEntry, MaterialLoadError> {
    let text = fs::read_to_string(path).map_err(MaterialLoadError::Io)?;
    let value = text.parse::<Value>().map_err(MaterialLoadError::Toml)?;
    build_entry(&value)
}

/// Assembles the entry from decoded keys.
fn build_entry(value: &Value) -> Result<MaterialEntry, MaterialLoadError> {
    let id = str_field(value, "id")?;
    let name = str_field(value, "name")?;
    let provenance = str_field(value, "provenance")?;
    let status = str_field(value, "status")?;
    let provenance_class = match status.as_str() {
        "measured" => Provenance::Measured,
        "estimate" => Provenance::Estimate,
        other => return Err(MaterialLoadError::BadStatus(other.to_owned())),
    };
    let thickness_um = float_field(value, "thickness_um")?;
    let modulus_gpa = float_field(value, "youngs_modulus_gpa")?;
    let poisson = value
        .get("poisson_ratio")
        .and_then(num_value)
        .unwrap_or(0.3);
    let min_radius = value
        .get("min_fold_radius_over_thickness")
        .and_then(num_value);
    let rig_runs = match value.get("rig_runs") {
        Some(Value::Array(tokens)) => {
            let mut parsed = Vec::with_capacity(tokens.len());
            for token in tokens {
                let Some(text) = token.as_str() else {
                    return Err(MaterialLoadError::MissingOrBad("rig_runs"));
                };
                parsed.push(text.to_owned());
            }
            parsed
        }
        _ => Vec::new(),
    };
    let model = MaterialModel {
        youngs_modulus_pa: modulus_gpa * 1.0e9,
        poisson_ratio: poisson,
        thickness_m: thickness_um * 1.0e-6,
        min_radius_ratio: min_radius,
        provenance: provenance_class,
    };
    Ok(MaterialEntry {
        id,
        name,
        model,
        provenance,
        rig_runs,
    })
}

/// Reads a required string field.
fn str_field(value: &Value, key: &'static str) -> Result<String, MaterialLoadError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or(MaterialLoadError::MissingOrBad(key))
}

/// Reads a numeric field that may be encoded as float or integer.
#[allow(clippy::cast_precision_loss)] // TOML dimensioned integers fit f64 mantissa here
fn num_value(cell: &Value) -> Option<f64> {
    cell.as_float()
        .or_else(|| cell.as_integer().map(|i| i as f64))
}

/// Reads a required numeric field.
fn float_field(value: &Value, key: &'static str) -> Result<f64, MaterialLoadError> {
    value
        .get(key)
        .and_then(num_value)
        .ok_or(MaterialLoadError::MissingOrBad(key))
}
