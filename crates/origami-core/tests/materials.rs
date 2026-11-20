//! Gates for the materials mechanics (REQ-O-005, REQ-O-007).

use core::f64::consts::PI;
use origami_core::materials::{MaterialModel, Provenance, RadiusVerdict};

// 301 SS-like steel: E 193 GPa, Poisson 0.3, t 200 um.
fn steel() -> MaterialModel {
    MaterialModel {
        youngs_modulus_pa: 193.0e9,
        poisson_ratio: 0.3,
        thickness_m: 2.0e-4,
        min_radius_ratio: Some(5.0),
        provenance: Provenance::Estimate,
    }
}

#[test]
fn bending_rigidity_matches_hand_computation() {
    // D = 193e9 * (2e-4)^3 / (12 * 0.91) = 1.544 / 10.92 = 0.14139 N.m
    let d = steel().bending_rigidity();
    let expected = 193.0e9 * 8.0e-12 / (12.0 * 0.91);
    assert!((d - expected).abs() < 1.0e-12, "D = {d}");
    assert!((d - 0.141_39).abs() < 5.0e-4, "hand value sanity: {d}");
}

#[test]
fn stiffness_scales_with_thickness_cubed() {
    let thin = steel();
    let thick = MaterialModel {
        thickness_m: 4.0e-4,
        ..thin
    };
    let ratio = thick.bending_rigidity() / thin.bending_rigidity();
    assert!(
        (ratio - 8.0).abs() < 1.0e-9,
        "t doubling must give 8x, got {ratio}"
    );
}

#[test]
fn buckling_governs_before_yield_for_thin_facets() {
    // 50 mm facet of 200 um steel: sigma_cr ~ 8 MPa << 1400 MPa yield.
    let sigma = steel().plate_buckling_stress(5.0e-2, 4.0);
    let expected = 4.0 * PI * PI * steel().bending_rigidity() / (2.5e-3 * 2.0e-4);
    assert!((sigma - expected).abs() < 1.0, "sigma = {sigma}");
    assert!(
        sigma < 2.0e7,
        "buckling stress should be O(10 MPa), got {sigma}"
    );
}

#[test]
fn radius_gate_verdicts() {
    assert_eq!(steel().radius_check(5.0), RadiusVerdict::Pass);
    assert_eq!(steel().radius_check(10.0), RadiusVerdict::Pass);
    assert_eq!(steel().radius_check(4.9), RadiusVerdict::Fail);
    let no_data = MaterialModel {
        min_radius_ratio: None,
        ..steel()
    };
    assert_eq!(no_data.radius_check(100.0), RadiusVerdict::NoData);
}

#[test]
fn estimates_are_not_gate_eligible() {
    assert!(!steel().is_gate_eligible());
    let measured = MaterialModel {
        provenance: Provenance::Measured,
        ..steel()
    };
    assert!(measured.is_gate_eligible());
}

#[test]
fn crease_stiffness_is_linear_in_length_and_calibration() {
    let k1 = steel().crease_torsional_stiffness(0.05, 1.0);
    let k2 = steel().crease_torsional_stiffness(0.10, 1.0);
    assert!((k2 / k1 - 2.0).abs() < 1.0e-9);
    let k3 = steel().crease_torsional_stiffness(0.05, 0.5);
    assert!((k3 / k1 - 0.5).abs() < 1.0e-9);
}
