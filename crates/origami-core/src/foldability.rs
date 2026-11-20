//! Flat-foldability vertex gates: Maekawa and Kawasaki.
//!
//! Scope: interior vertices with degree 4 whose incident edges are all
//! creases (`M`/`V`). Boundary vertices and unassigned creases are skipped:
//! the local conditions cannot judge them. Maekawa is pure counting and
//! builds everywhere; Kawasaki needs sector angles from `atan2`, so it is
//! gated behind the `std` feature (ADR-0003 gating pattern).

use crate::fold::{CreasePattern, EdgeAssignment};
use alloc::vec::Vec;

/// One violated necessary condition at one vertex.
#[derive(Debug, Clone, PartialEq)]
pub enum FoldabilityViolation {
    /// Interior vertex fails Maekawa: |mountains - valleys| != 2.
    Maekawa {
        /// Vertex index in `vertices_coords` order.
        vertex: usize,
        /// Number of mountain creases at the vertex.
        mountains: usize,
        /// Number of valley creases at the vertex.
        valleys: usize,
    },
    /// Sector angles fail Kawasaki: the alternating-angle identity does
    /// not hold (std builds only).
    Kawasaki {
        /// Vertex index in `vertices_coords` order.
        vertex: usize,
        /// Signed residual `(s0 + s2) - (s1 + s3)` in degrees.
        error_deg: f64,
    },
}

/// Tolerance for the Kawasaki identity, in degrees.
const KAWASAKI_EPS_DEG: f64 = 1.0e-6;

/// Runs both flat-foldability gates over every judgeable interior vertex.
///
/// An empty result means every checked vertex satisfies Maekawa (all
/// builds) and Kawasaki (`std` builds). These are necessary, not
/// sufficient, conditions: passing says the pattern has no local
/// flat-foldability violation, not that the global fold exists.
#[must_use]
pub fn check_flat_foldability(cp: &CreasePattern) -> Vec<FoldabilityViolation> {
    let adj = cp.adjacency();
    let mut violations: Vec<FoldabilityViolation> = Vec::new();
    for (vertex, incident) in adj.iter().enumerate() {
        if incident.len() != 4 {
            continue;
        }
        let mut mountains: usize = 0;
        let mut valleys: usize = 0;
        let mut judgeable = true;
        for edge_id in incident {
            match cp.assignments.get(*edge_id) {
                Some(EdgeAssignment::Mountain) => mountains += 1,
                Some(EdgeAssignment::Valley) => valleys += 1,
                _ => judgeable = false,
            }
        }
        if !judgeable {
            continue;
        }
        if mountains.abs_diff(valleys) != 2 {
            violations.push(FoldabilityViolation::Maekawa {
                vertex,
                mountains,
                valleys,
            });
        }
        #[cfg(feature = "std")]
        if let Some(error_deg) = kawasaki_error(cp, vertex, &adj) {
            if error_deg.abs() > KAWASAKI_EPS_DEG {
                violations.push(FoldabilityViolation::Kawasaki { vertex, error_deg });
            }
        }
    }
    violations
}

/// Kawasaki residual at one vertex in degrees: `(s0 + s2) - (s1 + s3)`
/// over the four sector angles in cyclic order. Flat foldability requires
/// each alternating pair to sum to 180 degrees, so the residual must be 0.
#[cfg(feature = "std")]
fn kawasaki_error(cp: &CreasePattern, vertex: usize, adj: &[Vec<usize>]) -> Option<f64> {
    let center = *cp.vertices.get(vertex)?;
    let (cx, cy) = (center[0], center[1]);
    let mut directions: Vec<f64> = Vec::with_capacity(4);
    for edge_id in adj.get(vertex)? {
        let pair = *cp.edges.get(*edge_id)?;
        let other = if pair[0] == vertex { pair[1] } else { pair[0] };
        let point = *cp.vertices.get(other)?;
        let (dx, dy) = (point[0] - cx, point[1] - cy);
        directions.push(dy.atan2(dx));
    }
    directions.sort_by(f64::total_cmp);
    let (Some(&a0), Some(&a1), Some(&a2), Some(&a3)) = (
        directions.first(),
        directions.get(1),
        directions.get(2),
        directions.get(3),
    ) else {
        return None;
    };
    let two_pi = core::f64::consts::PI + core::f64::consts::PI;
    let s0 = span(a0, a1, two_pi);
    let s1 = span(a1, a2, two_pi);
    let s2 = span(a2, a3, two_pi);
    let s3 = span(a3, a0 + two_pi, two_pi);
    let diff = (s0 + s2) - (s1 + s3);
    Some(diff * 180.0 / core::f64::consts::PI)
}

/// Clockwise span from direction `from` to direction `to`, in radians.
#[cfg(feature = "std")]
fn span(from: f64, to: f64, two_pi: f64) -> f64 {
    let mut s = to - from;
    if s <= 0.0 {
        s += two_pi;
    }
    s
}
