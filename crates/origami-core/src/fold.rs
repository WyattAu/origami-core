//! FOLD-native crease-pattern data model: pure data, no I/O.
//!
//! Mirrors the subset of the FOLD spec (Demaine et al.) that this crate
//! consumes: `vertices_coords`, `edges_vertices`, `edges_assignments`,
//! `faces_vertices`. JSON decoding lives in `origami-cli`; this module
//! stays `no_std`-capable per the estate `no_std` policy (ADR-0003).

use alloc::vec::Vec;

/// Classification of one edge, following FOLD `edges_assignments` tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeAssignment {
    /// Sheet border: not a crease.
    Boundary,
    /// Mountain fold.
    Mountain,
    /// Valley fold.
    Valley,
    /// Crease present but not yet assigned.
    Unassigned,
}

impl EdgeAssignment {
    /// Parses a FOLD assignment token (`"B"`, `"M"`, `"V"`, `"U"`).
    #[must_use]
    pub fn parse(token: &str) -> Option<Self> {
        match token {
            "B" => Some(Self::Boundary),
            "M" => Some(Self::Mountain),
            "V" => Some(Self::Valley),
            "U" => Some(Self::Unassigned),
            _ => None,
        }
    }
}

/// Structural invariant violations detected at construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructureError {
    /// `edges_assignments` length differs from the edge count.
    AssignmentCount {
        /// Number of assignment tokens supplied.
        assignments: usize,
        /// Number of edges supplied.
        edges: usize,
    },
    /// An edge endpoint indexes past the end of the vertex list.
    BadVertexIndex {
        /// Index of the offending edge.
        edge: usize,
        /// The out-of-range vertex index.
        vertex: usize,
        /// Total vertex count.
        vertex_count: usize,
    },
    /// Both endpoints of an edge are the same vertex.
    DegenerateEdge {
        /// Index of the offending edge.
        edge: usize,
    },
    /// A face vertex index refers to a nonexistent vertex.
    BadFaceIndex {
        /// Index of the offending face.
        face: usize,
        /// The out-of-range vertex index.
        vertex: usize,
        /// Total vertex count.
        vertex_count: usize,
    },
}

/// A flat crease pattern: the FOLD data-model subset this crate models.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CreasePattern {
    /// Vertex positions in the flat (unfolded) state.
    pub vertices: Vec<[f64; 2]>,
    /// Edges as pairs of vertex indices.
    pub edges: Vec<[usize; 2]>,
    /// One assignment per edge, same order as `edges`.
    pub assignments: Vec<EdgeAssignment>,
    /// Faces as vertex-index cycles; may be empty when unknown.
    pub faces: Vec<Vec<usize>>,
}

impl CreasePattern {
    /// Builds a crease pattern and checks its structural invariants.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError`] when assignment counts mismatch, an edge
    /// endpoint is out of range, or an edge is degenerate.
    pub fn new(
        vertices: Vec<[f64; 2]>,
        edges: Vec<[usize; 2]>,
        assignments: Vec<EdgeAssignment>,
        faces: Vec<Vec<usize>>,
    ) -> Result<Self, StructureError> {
        let pattern = Self {
            vertices,
            edges,
            assignments,
            faces,
        };
        pattern.validate()?;
        Ok(pattern)
    }

    /// Checks structural invariants without consuming the pattern.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError`] when an invariant fails.
    pub fn validate(&self) -> Result<(), StructureError> {
        if self.assignments.len() != self.edges.len() {
            return Err(StructureError::AssignmentCount {
                assignments: self.assignments.len(),
                edges: self.edges.len(),
            });
        }
        let vertex_count = self.vertices.len();
        for (edge_id, pair) in self.edges.iter().enumerate() {
            let [a, b] = *pair;
            if a == b {
                return Err(StructureError::DegenerateEdge { edge: edge_id });
            }
            if a >= vertex_count || b >= vertex_count {
                let vertex = if a >= vertex_count { a } else { b };
                return Err(StructureError::BadVertexIndex {
                    edge: edge_id,
                    vertex,
                    vertex_count,
                });
            }
        }
        for (face_id, face) in self.faces.iter().enumerate() {
            for vertex in face {
                if *vertex >= vertex_count {
                    return Err(StructureError::BadFaceIndex {
                        face: face_id,
                        vertex: *vertex,
                        vertex_count,
                    });
                }
            }
        }
        Ok(())
    }

    /// Incident edge ids per vertex. Edges referencing out-of-range
    /// vertices are ignored; run `validate` first for a strict check.
    #[must_use]
    pub fn adjacency(&self) -> Vec<Vec<usize>> {
        let mut adj: Vec<Vec<usize>> = Vec::with_capacity(self.vertices.len());
        for _ in 0..self.vertices.len() {
            adj.push(Vec::new());
        }
        for (edge_id, pair) in self.edges.iter().enumerate() {
            let [a, b] = *pair;
            if let Some(slot) = adj.get_mut(a) {
                slot.push(edge_id);
            }
            if let Some(slot) = adj.get_mut(b) {
                slot.push(edge_id);
            }
        }
        adj
    }
}
