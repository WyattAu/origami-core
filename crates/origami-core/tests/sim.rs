//! Gates for the bar-and-hinge simulator (roadmap Slice 2).
//!
//! Small meshes only: these run under miri too. The full-pattern check
//! lives in origami-cli's integration tests.

use core::f64::consts::PI;
use origami_core::fold::{CreasePattern, EdgeAssignment as Assignment};
use origami_core::sim::{simulate_fold, SimParams};

/// Quad sheet, 2 triangles, one diagonal crease between vertices 0-2.
fn quad_cp(assignment: Assignment) -> CreasePattern {
    let built = CreasePattern::new(
        vec![[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [0.0, 1.0]],
        vec![[0, 1], [1, 2], [2, 3], [3, 0], [0, 2]],
        vec![
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            Assignment::Boundary,
            assignment,
        ],
        vec![vec![0, 1, 2], vec![0, 2, 3]],
    );
    assert!(built.is_ok(), "fixture is structural: {built:?}");
    match built {
        Ok(cp) => cp,
        Err(_) => unreachable!("fixture validity asserted"),
    }
}

fn max_abs_z(v: &[[f64; 3]]) -> f64 {
    v.iter().map(|p| p[2].abs()).fold(0.0_f64, f64::max)
}

#[test]
fn flat_percent_stays_flat_and_converges() {
    let cp = quad_cp(Assignment::Mountain);
    let report = simulate_fold(
        &cp,
        &SimParams {
            fold_percent: 0.0,
            max_iterations: 50,
            ..SimParams::default()
        },
    );
    assert!(report.converged, "flat sheet is already at the target");
    assert!(
        max_abs_z(&report.vertices) < 1.0e-9,
        "flat fold must not leave the plane"
    );
}

#[test]
fn single_crease_folds_out_of_plane() {
    let cp = quad_cp(Assignment::Mountain);
    let report = simulate_fold(
        &cp,
        &SimParams {
            fold_percent: 0.5,
            max_iterations: 1500,
            ..SimParams::default()
        },
    );
    assert!(
        report.max_axial_strain < 0.05,
        "bars must stay near rest length: {}",
        report.max_axial_strain
    );
    assert!(
        report.max_angle_error_rad < 0.2,
        "crease must reach its target: {}",
        report.max_angle_error_rad
    );
    assert!(
        max_abs_z(&report.vertices) > 0.2,
        "half-fold must leave the plane: {}",
        max_abs_z(&report.vertices)
    );
    // Target angle is half of pi by construction.
    let half_pi = 0.5 * PI;
    assert!(
        (report.mean_angle_error_rad.abs()) < 0.2,
        "mean angle error {} vs target {half_pi}",
        report.mean_angle_error_rad
    );
}

#[test]
fn simulation_is_deterministic() {
    let cp = quad_cp(Assignment::Valley);
    let a = simulate_fold(
        &cp,
        &SimParams {
            fold_percent: 0.4,
            max_iterations: 400,
            ..SimParams::default()
        },
    );
    let b = simulate_fold(
        &cp,
        &SimParams {
            fold_percent: 0.4,
            max_iterations: 400,
            ..SimParams::default()
        },
    );
    assert_eq!(a.vertices, b.vertices, "no randomness allowed");
    assert_eq!(a.iterations, b.iterations);
}

#[test]
fn opposite_percent_folds_opposite_way() {
    let cp = quad_cp(Assignment::Mountain);
    let up = simulate_fold(
        &cp,
        &SimParams {
            fold_percent: 0.4,
            max_iterations: 1200,
            ..SimParams::default()
        },
    );
    let down = simulate_fold(
        &cp,
        &SimParams {
            fold_percent: -0.4,
            max_iterations: 1200,
            ..SimParams::default()
        },
    );
    let (Some(&za), Some(&zb)) = (
        up.vertices.get(1).map(|p| &p[2]),
        down.vertices.get(1).map(|p| &p[2]),
    ) else {
        unreachable!("4 vertices in fixture")
    };
    assert!(
        za * zb < 0.0,
        "opposite crease signs must fold to opposite sides: {za} vs {zb}"
    );
}

#[test]
fn unassigned_crease_is_free() {
    // U: no torsion spring -> sheet has no driver and stays flat, and
    // the run must not produce NaNs.
    let cp = quad_cp(Assignment::Unassigned);
    let report = simulate_fold(
        &cp,
        &SimParams {
            fold_percent: 0.5,
            max_iterations: 200,
            ..SimParams::default()
        },
    );
    assert!(report
        .vertices
        .iter()
        .all(|p| p[0].is_finite() && p[1].is_finite() && p[2].is_finite()));
    assert!(max_abs_z(&report.vertices) < 1.0e-6);
}

#[test]
fn degenerate_edge_is_rejected_before_simulation() {
    // The constructor catches it; simulate_fold would just see zero
    // rest length. Guard the fixture path anyway.
    let bad = CreasePattern::new(
        vec![[0.0, 0.0], [1.0, 0.0]],
        vec![[0, 0]],
        vec![Assignment::Mountain],
        Vec::new(),
    );
    assert!(matches!(
        bad,
        Err(origami_core::fold::StructureError::DegenerateEdge { .. })
    ));
}
