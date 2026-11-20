//! Estimate-grade plate and crease mechanics for materials entries.
//!
//! Formulas are classic plate theory (the Schenk-Guest bar-and-hinge
//! lineage): bending rigidity from plate theory, crease torsion as a
//! calibrated multiple of it, and plate-strip buckling for facets. Every
//! output here is estimate-grade until the rig in docs/P0-RIG.md
//! produces measured calibration constants; estimate-grade values never
//! feed gate decisions (SPEC section 5, REQ-O-005/007).

/// Provenance class of a materials entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Values traceable to a rig run id under `data/rig/runs/`.
    Measured,
    /// Datasheet-typical values; never gate-eligible.
    Estimate,
}

/// A normalized material model in SI units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialModel {
    /// Young's modulus in pascals.
    pub youngs_modulus_pa: f64,
    /// Poisson ratio (dimensionless).
    pub poisson_ratio: f64,
    /// Sheet thickness in meters.
    pub thickness_m: f64,
    /// Minimum dynamic fold radius as a ratio over thickness, if known.
    pub min_radius_ratio: Option<f64>,
    /// Provenance class of the source data.
    pub provenance: Provenance,
}

/// Verdict of a dynamic fold-radius check at a requested r/t.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadiusVerdict {
    /// Requested r/t is at or above the material minimum.
    Pass,
    /// Requested r/t is below the material minimum.
    Fail,
    /// Material has no radius data: cannot judge (never "pass").
    NoData,
}

impl MaterialModel {
    /// Plate bending rigidity `D = E t^3 / (12 (1 - nu^2))` in N·m
    /// (per unit width). Bending stiffness scales with the cube of
    /// thickness; this is why thin facets buckle long before they yield.
    #[must_use]
    pub fn bending_rigidity(&self) -> f64 {
        let nu2 = self.poisson_ratio * self.poisson_ratio;
        self.youngs_modulus_pa * self.thickness_m.powi(3) / (12.0 * (1.0 - nu2))
    }

    /// Estimate crease torsional stiffness for a crease of
    /// `crease_length_m`: `k = calibration * D * L` in N·m/rad.
    ///
    /// `calibration` is 1.0 by convention until the rig measures the
    /// crease-level factor per material class; pass the measured value
    /// once available. Estimate-grade while inputs are estimates.
    #[must_use]
    pub fn crease_torsional_stiffness(&self, crease_length_m: f64, calibration: f64) -> f64 {
        calibration * self.bending_rigidity() * crease_length_m
    }

    /// Estimate critical buckling stress (Pa) of a plate strip of width
    /// `panel_width_m` with buckling coefficient `k_factor` (4.0 for a
    /// clamped long strip): `sigma = k pi^2 D / (b^2 t)`.
    #[must_use]
    pub fn plate_buckling_stress(&self, panel_width_m: f64, k_factor: f64) -> f64 {
        let b2 = panel_width_m * panel_width_m;
        let pi2 = core::f64::consts::PI * core::f64::consts::PI;
        k_factor * pi2 * self.bending_rigidity() / (b2 * self.thickness_m)
    }

    /// True when the requested r/t is at or above the material's
    /// dynamic minimum. `None` when the material has no radius data:
    /// missing data is a `NoData` verdict, never a silent pass.
    #[must_use]
    pub fn radius_check(&self, requested_ratio: f64) -> RadiusVerdict {
        match self.min_radius_ratio {
            Some(min) if requested_ratio >= min => RadiusVerdict::Pass,
            Some(_) => RadiusVerdict::Fail,
            None => RadiusVerdict::NoData,
        }
    }

    /// True when gate decisions may consume this entry (REQ-O-005).
    #[must_use]
    pub fn is_gate_eligible(&self) -> bool {
        self.provenance == Provenance::Measured
    }
}
