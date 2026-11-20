//! Gates for the flat-foldability conditions (REQ-O-002, REQ-O-003).
//!
//! Note on style: the workspace denies `unwrap`/`expect`/`panic!`/slicing
//! even in tests, so assertions are phrased with `assert!`/`matches!`.

use core::f64::consts::PI;
use origami_core::fold::{CreasePattern, EdgeAssignment as Assignment, StructureError};
use origami_core::foldability::{check_flat_foldability, FoldabilityViolation};

/// Builds a single interior vertex plus degree-1 arm tips at the given
/// directions (degrees from the +x axis).
fn star_cp(directions_deg: &[f64], assignments: &[Assignment]) -> CreasePattern {
    let mut vertices: Vec<[f64; 2]> = vec![[0.0, 0.0]];
    let mut edges: Vec<[usize; 2]> = Vec::new();
    for (k, deg) in directions_deg.iter().enumerate() {
        let rad = deg * PI / 180.0;
        vertices.push([rad.cos(), rad.sin()]);
        edges.push([0, k + 1]);
    }
    let built = CreasePattern::new(vertices, edges, assignments.to_vec(), Vec::new());
    assert!(built.is_ok(), "test fixture must be structurally valid");
    match built {
        Ok(cp) => cp,
        Err(_) => unreachable!("fixture validity already asserted"),
    }
}

#[test]
fn miura_style_vertex_passes_both_gates() {
    // Collinear horizontal pair (0/180) plus diagonals at 45/315.
    // Maekawa: 3 mountains, 1 valley. Kawasaki: sectors 45/135/135/45.
    let cp = star_cp(
        &[0.0, 45.0, 180.0, 315.0],
        &[
            Assignment::Mountain,
            Assignment::Mountain,
            Assignment::Mountain,
            Assignment::Valley,
        ],
    );
    let violations = check_flat_foldability(&cp);
    assert!(
        violations.is_empty(),
        "expected no violations, got {violations:?}"
    );
}

#[test]
fn kawasaki_only_failure_is_reported() {
    // Non-collinear star at 0/70/200/300: sectors 70/130/100/60.
    // Alternating sums are 170 vs 190, so Kawasaki fails while the
    // 3-mountain/1-valley split keeps Maekawa green.
    let cp = star_cp(
        &[0.0, 70.0, 200.0, 300.0],
        &[
            Assignment::Mountain,
            Assignment::Mountain,
            Assignment::Mountain,
            Assignment::Valley,
        ],
    );
    let violations = check_flat_foldability(&cp);
    assert_eq!(violations.len(), 1, "got {violations:?}");
    assert!(
        matches!(
            violations.first(),
            Some(FoldabilityViolation::Kawasaki { vertex: 0, .. })
        ),
        "expected Kawasaki at vertex 0, got {violations:?}"
    );
    if let Some(FoldabilityViolation::Kawasaki { error_deg, .. }) = violations.first() {
        assert!(
            (*error_deg + 20.0).abs() < 1.0e-6,
            "residual should be -20 deg, got {error_deg}"
        );
    }
}

#[test]
fn maekawa_only_failure_is_reported() {
    // Flat cross with 2 mountains and 2 valleys: Kawasaki holds (90x4),
    // Maekawa needs a 3/1 split.
    let cp = star_cp(
        &[0.0, 90.0, 180.0, 270.0],
        &[
            Assignment::Mountain,
            Assignment::Valley,
            Assignment::Mountain,
            Assignment::Valley,
        ],
    );
    let violations = check_flat_foldability(&cp);
    assert_eq!(violations.len(), 1, "got {violations:?}");
    assert!(matches!(
        violations.first(),
        Some(FoldabilityViolation::Maekawa {
            mountains: 2,
            valleys: 2,
            ..
        })
    ));
}

#[test]
fn unassigned_or_boundary_vertices_are_skipped() {
    // Unassigned creases: cannot judge, so no violation is raised.
    let cp = star_cp(
        &[0.0, 90.0, 180.0, 270.0],
        &[
            Assignment::Mountain,
            Assignment::Valley,
            Assignment::Unassigned,
            Assignment::Unassigned,
        ],
    );
    assert_eq!(check_flat_foldability(&cp), Vec::new());

    // A boundary edge makes the vertex unjudgeable too.
    let cp2 = star_cp(
        &[0.0, 45.0, 180.0, 315.0],
        &[
            Assignment::Boundary,
            Assignment::Mountain,
            Assignment::Mountain,
            Assignment::Valley,
        ],
    );
    assert_eq!(check_flat_foldability(&cp2), Vec::new());
}

#[test]
fn structural_errors_surface_from_constructor() {
    let vertices = vec![[0.0, 0.0], [1.0, 0.0]];
    // Degenerate edge: both endpoints identical.
    let degenerate = CreasePattern::new(
        vertices.clone(),
        vec![[0, 0]],
        vec![Assignment::Mountain],
        Vec::new(),
    );
    assert!(matches!(
        degenerate,
        Err(StructureError::DegenerateEdge { edge: 0 })
    ));
    // Out-of-range endpoint.
    let out_of_range = CreasePattern::new(
        vertices.clone(),
        vec![[0, 9]],
        vec![Assignment::Mountain],
        Vec::new(),
    );
    assert!(matches!(
        out_of_range,
        Err(StructureError::BadVertexIndex { .. })
    ));
    // Assignment count mismatch.
    let mismatch = CreasePattern::new(vertices, vec![[0, 1]], vec![], Vec::new());
    assert!(matches!(
        mismatch,
        Err(StructureError::AssignmentCount {
            assignments: 0,
            edges: 1
        })
    ));
}
