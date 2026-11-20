//! Minimal FOLD (JSON) reader for the subset that `origami-core` models.
//!
//! Reads `vertices_coords`, `edges_vertices`, `edges_assignments`, and
//! `faces_vertices`. Untrusted-input note: this parser runs on files from
//! the design library and from users; see THREAT-MODEL.md. Keep input
//! handling defensive: no panics on malformed JSON, errors only.

use std::fs;
use std::path::Path;

use origami_core::fold::{CreasePattern, EdgeAssignment, StructureError};
use serde_json::Value;

/// Failures while loading a FOLD file.
#[derive(Debug)]
pub enum FoldLoadError {
    /// The file could not be read.
    Io(std::io::Error),
    /// The content is not valid JSON.
    Json(serde_json::Error),
    /// A required FOLD key is missing or has the wrong shape.
    MissingOrBad(&'static str),
    /// An assignment token outside the FOLD set (B, M, V, U, and friends).
    BadAssignment(String),
    /// Structural invariants failed.
    Structure(StructureError),
}

/// Loads one FOLD file into a validated [`CreasePattern`].
///
/// # Errors
///
/// Returns [`FoldLoadError`] on I/O, JSON, shape, token, or structural
/// problems.
pub fn load_fold(path: &Path) -> Result<CreasePattern, FoldLoadError> {
    let text = fs::read_to_string(path).map_err(FoldLoadError::Io)?;
    let value: Value = serde_json::from_str(&text).map_err(FoldLoadError::Json)?;
    build_pattern(&value)
}

/// Assembles the pattern from decoded FOLD keys.
fn build_pattern(value: &Value) -> Result<CreasePattern, FoldLoadError> {
    let vertices = coord_pairs(value, "vertices_coords")?;
    let edges = index_pairs(value, "edges_vertices")?;
    let faces = face_cycles(value)?;
    let assignments = match value.get("edges_assignments") {
        Some(Value::Array(tokens)) => {
            let mut parsed = Vec::with_capacity(tokens.len());
            for token in tokens {
                let text = token
                    .as_str()
                    .ok_or(FoldLoadError::MissingOrBad("edges_assignments"))?;
                let Some(assignment) = EdgeAssignment::parse(text) else {
                    return Err(FoldLoadError::BadAssignment(text.to_owned()));
                };
                parsed.push(assignment);
            }
            parsed
        }
        _ => vec![EdgeAssignment::Unassigned; edges.len()],
    };
    CreasePattern::new(vertices, edges, assignments, faces).map_err(FoldLoadError::Structure)
}

/// Reads `key` as an array of `[x, y]` number pairs.
fn coord_pairs(value: &Value, key: &'static str) -> Result<Vec<[f64; 2]>, FoldLoadError> {
    let rows = value
        .get(key)
        .and_then(Value::as_array)
        .ok_or(FoldLoadError::MissingOrBad(key))?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let cells = row.as_array().ok_or(FoldLoadError::MissingOrBad(key))?;
        let [x, y] = cells.as_slice() else {
            return Err(FoldLoadError::MissingOrBad(key));
        };
        let px = x.as_f64().ok_or(FoldLoadError::MissingOrBad(key))?;
        let py = y.as_f64().ok_or(FoldLoadError::MissingOrBad(key))?;
        out.push([px, py]);
    }
    Ok(out)
}

/// Reads `key` as an array of `[a, b]` index pairs.
fn index_pairs(value: &Value, key: &'static str) -> Result<Vec<[usize; 2]>, FoldLoadError> {
    let rows = value
        .get(key)
        .and_then(Value::as_array)
        .ok_or(FoldLoadError::MissingOrBad(key))?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let cells = row.as_array().ok_or(FoldLoadError::MissingOrBad(key))?;
        let [a, b] = cells.as_slice() else {
            return Err(FoldLoadError::MissingOrBad(key));
        };
        let va = usize_value(a).ok_or(FoldLoadError::MissingOrBad(key))?;
        let vb = usize_value(b).ok_or(FoldLoadError::MissingOrBad(key))?;
        out.push([va, vb]);
    }
    Ok(out)
}

/// Reads `key` as an array of vertex-index cycles.
fn face_cycles(value: &Value) -> Result<Vec<Vec<usize>>, FoldLoadError> {
    let Some(Value::Array(faces)) = value.get("faces_vertices") else {
        return Ok(Vec::new());
    };
    let mut out = Vec::with_capacity(faces.len());
    for face in faces {
        let cycle = face
            .as_array()
            .ok_or(FoldLoadError::MissingOrBad("faces_vertices"))?;
        let mut vertices = Vec::with_capacity(cycle.len());
        for cell in cycle {
            let idx = usize_value(cell).ok_or(FoldLoadError::MissingOrBad("faces_vertices"))?;
            vertices.push(idx);
        }
        out.push(vertices);
    }
    Ok(out)
}

/// JSON numbers holding integral values become `usize`.
fn usize_value(cell: &Value) -> Option<usize> {
    let n = cell.as_u64()?;
    usize::try_from(n).ok()
}

#[cfg(test)]
mod tests {
    use super::{build_pattern, FoldLoadError};
    use origami_core::fold::StructureError;
    use serde_json::json;

    #[test]
    fn missing_required_keys_are_named_errors() {
        let no_vertices = json!({"edges_vertices": [[0, 1]]});
        assert!(matches!(
            build_pattern(&no_vertices),
            Err(FoldLoadError::MissingOrBad("vertices_coords"))
        ));
        let no_edges = json!({"vertices_coords": [[0.0, 0.0], [1.0, 0.0]]});
        assert!(matches!(
            build_pattern(&no_edges),
            Err(FoldLoadError::MissingOrBad("edges_vertices"))
        ));
    }

    #[test]
    fn wrong_row_shapes_are_named_errors() {
        let base = |verts: serde_json::Value, edges: serde_json::Value| json!({"vertices_coords": verts, "edges_vertices": edges});
        let short_vertex_row = base(json!([[0.0, 0.0], [1.0]]), json!([[0, 1]]));
        assert!(matches!(
            build_pattern(&short_vertex_row),
            Err(FoldLoadError::MissingOrBad("vertices_coords"))
        ));
        let short_edge_row = base(json!([[0.0, 0.0], [1.0, 1.0]]), json!([[0]]));
        assert!(matches!(
            build_pattern(&short_edge_row),
            Err(FoldLoadError::MissingOrBad("edges_vertices"))
        ));
        let non_number = base(json!([[0.0, 0.0], ["x", 1.0]]), json!([[0, 1]]));
        assert!(matches!(
            build_pattern(&non_number),
            Err(FoldLoadError::MissingOrBad("vertices_coords"))
        ));
    }

    #[test]
    fn unknown_assignment_token_is_named_error() {
        let v = json!({
            "vertices_coords": [[0.0, 0.0], [1.0, 0.0]],
            "edges_vertices": [[0, 1]],
            "edges_assignments": ["X"]
        });
        assert!(matches!(
            build_pattern(&v),
            Err(FoldLoadError::BadAssignment(word)) if word == "X"
        ));
    }

    #[test]
    fn assignment_count_mismatch_is_structural() {
        let v = json!({
            "vertices_coords": [[0.0, 0.0], [1.0, 0.0]],
            "edges_vertices": [[0, 1]],
            "edges_assignments": ["M", "V"]
        });
        assert!(matches!(
            build_pattern(&v),
            Err(FoldLoadError::Structure(
                StructureError::AssignmentCount { .. }
            ))
        ));
    }

    #[test]
    fn minimal_valid_pattern_loads_with_faces() {
        let v = json!({
            "vertices_coords": [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            "edges_vertices": [[0, 1], [1, 2], [2, 3], [3, 0]],
            "edges_assignments": ["B", "B", "B", "B"],
            "faces_vertices": [[0, 1, 2, 3]]
        });
        let built = build_pattern(&v);
        assert!(built.is_ok(), "minimal pattern must load: {built:?}");
        let Ok(pattern) = built else {
            unreachable!("minimal pattern load asserted")
        };
        assert_eq!(pattern.faces.len(), 1);
        assert_eq!(pattern.assignments.len(), 4);
    }

    #[test]
    fn malformed_face_rows_are_named_errors() {
        // Non-numeric face cell: schema-level error.
        let v = json!({
            "vertices_coords": [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            "edges_vertices": [[0, 1], [1, 2], [2, 0]],
            "faces_vertices": [["x"]]
        });
        assert!(matches!(
            build_pattern(&v),
            Err(FoldLoadError::MissingOrBad("faces_vertices"))
        ));
        // Out-of-range face index: structural error.
        let v2 = json!({
            "vertices_coords": [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            "edges_vertices": [[0, 1], [1, 2], [2, 0]],
            "faces_vertices": [[0, 1, 9]]
        });
        assert!(matches!(
            build_pattern(&v2),
            Err(FoldLoadError::Structure(StructureError::BadFaceIndex {
                face: 0,
                vertex: 9,
                vertex_count: 3
            }))
        ));
    }
}
