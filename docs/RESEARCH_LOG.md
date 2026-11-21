# RESEARCH LOG: origami-core simulation

status: active
updated: 2026-11-21

## R-001: Miura rigid-fold sign pattern (2026-11-21)

Finding: the Miura CP's rigid-foldable dihedral sign pattern is
**uniform across all creases** (all mountains-as-labeled or all
flipped), not the checkerboard M/V labeling. Derivation: the rigid
vertex relation `tan(theta_H / 2) * tan(theta_Z / 2) = tan^2(alpha / 2)`
(the Schenk-Guest 4-valent fold-angle relation for the Miura sector
pattern) forces theta_H and theta_Z to share a sign. The M/V labels
drawn on a Miura CP encode flat-foldability and include a per-crease
facet-parity factor that differs from the dihedral sign.

Measured evidence: `fold-sim` on the seed design with the raw M/V
labels stalls at 3.6% strain (percent 0.02, 8000 sweeps) with ~9
creases resting at exactly the negative of their target; relaxing the
labels (all-crease same sign) reaches ~1.5708 rad (percent 0.5) on most
creases within 5000 sweeps with the projection solver.

Status: solver target derivation from labels vs kinematics is an open
design question. Options on the table:
1. derive targets from a Tachi-style linear fold-rate solve (needs the
   full per-vertex sector-angle equations, not just opposite-equality);
2. accept `target_mode` parameter with a documented uniform mode;
3. keep label mode and document that flat-foldability-labeled CPs fold
   at reduced percent only.

## R-002: global Gauss-Newton step diverges (2026-11-21)

Attempted: global Gauss-Newton constraint projection (assemble J over
bars + torsions, solve `(J J^T + lambda I) y = -e`, apply `J^T y`) on
the Miura seed. Diverged (strain growing past 100%) for lambda both
fixed 1e-6 and scaled to the mean diagonal; bar/torsion row scales
differ by orders of magnitude and the cap clamp interacts badly with
the global solve.

Status: rejected for v1. The position-based Gauss-Seidel projection
(torsion creep + exact bar projection, bars last) is stable and
measured at <1.5% strain on the seed within 60k sweeps
(`step_cap` 0.002). Next research step: damped LM with line search on
the total energy, or the linear-system solve restricted to torsion
constraints with bars as exact projections.

## R-003: solver parameter surface (2026-11-21)

`step_cap` is the dominant stability knob: strain decreases
monotonically with smaller caps (0.05: ~10%, 0.01: ~4%, 0.002: ~1.4%
on the seed at fixed sweep budgets). `relax` 0.9 fine at small caps.
Bars-last ordering matters for the reported residual (end state
satisfies lengths).
