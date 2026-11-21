//! Bar-and-hinge folded-state simulation (estimate-grade).
//!
//! Method: fan-triangulate the crease pattern, treat every mesh edge as
//! an axial bar with rest length from the flat CP, and every interior
//! crease (plus every triangulation diagonal) as a torsional spring.
//! Mountain/valley creases get targets driven by `fold_percent`
//! (target = assignment sign times `fold_percent` times `pi`), following
//! the FOLD / Origami Simulator convention where facet normals come from
//! face winding. The solver is position-based Gauss-Seidel constraint
//! projection: each sweep creeps torsion springs toward their (ramped)
//! targets by rotating the two opposite tips, then projects bars to
//! rest length exactly. Deterministic (no randomness).
//!
//! Known limits (v1, measured): on the 4x5 Miura seed the solver reaches
//! <1.5% max axial strain within 60k sweeps at `step_cap` 0.002; larger
//! caps fold faster but leave residual strain. A global Gauss-Newton
//! step (solve `J J^T y = -e`) was tried and diverges on this problem;
//! a damped/line-searched variant is the next research step (see
//! `docs/RESEARCH_LOG.md`). The Miura rigid fold needs uniform dihedral
//! signs, not the flat-foldability M/V labeling: percent 0.5 on the seed
//! reaches its targets when the labels agree with the fold direction
//! (see `docs/RESEARCH_LOG.md`).

// Geometry code conventionally uses short component names (a, b, c, d,
// u, v); the pedantic min-ident lint is disabled for this module only.
#![allow(clippy::many_single_char_names)]

use crate::fold::{CreasePattern, EdgeAssignment};
use alloc::vec::Vec;
use core::f64::consts::PI;

/// How crease target angles are derived.
///
/// Research context (`docs/RESEARCH_LOG.md` R-001): M/V labels encode
/// flat-foldability and are NOT rigid-fold targets for patterns like
/// Miura; the kinematic rate solve is the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetMode {
    /// Derive target signs from the rigid-folding fold-rate solve
    /// (Tachi-style opposite-equality). Falls back to label targets
    /// when the rate system has no 1-D null space (e.g. patterns
    /// without interior 4-valent vertices).
    #[default]
    Rates,
    /// Raw M/V labels as target signs (legacy; folds only patterns
    /// whose labels happen to match the dihedral convention).
    Labels,
}

/// Solver parameters for [`simulate_fold`].
#[derive(Debug, Clone)]
pub struct SimParams {
    /// Fold depth in [-1, 1]: crease target angle = sign * percent * `pi`.
    pub fold_percent: f64,
    /// Hard cap on projection sweeps.
    pub max_iterations: u32,
    /// Per-projection relaxation factor (0 to 1].
    pub relax: f64,
    /// Per-vertex displacement cap per projection, in model units.
    /// Small caps (0.002 to 0.005) converge smoothly; large caps fold
    /// fast but leave residual strain.
    pub step_cap: f64,
    /// Convergence tolerance on per-crease angle error, radians.
    pub angle_tol_rad: f64,
    /// Convergence tolerance on max axial strain (dimensionless).
    pub strain_tol: f64,
    /// Fraction of the sweep budget over which crease targets ramp from
    /// 0 to full (load-stepping; 0 disables).
    pub ramp_fraction: f64,
    /// Target derivation mode.
    pub target_mode: TargetMode,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            fold_percent: 0.5,
            max_iterations: 30000,
            relax: 0.9,
            step_cap: 0.002,
            angle_tol_rad: 0.03,
            strain_tol: 5.0e-3,
            ramp_fraction: 0.3,
            target_mode: TargetMode::Rates,
        }
    }
}

/// One bar + optional torsion spring.
#[derive(Debug, Clone, Copy)]
struct Spring {
    a: usize,
    b: usize,
    /// Opposite facet vertices (the two triangle tips across the edge).
    r: Option<usize>,
    s: Option<usize>,
    rest_len: f64,
    /// Torsion target angle, radians (facet springs: 0).
    target_angle: f64,
    /// Relative torsion stiffness (axial stiffness is 1 for all bars).
    torsion_k: f64,
    /// M/V label sign for driven creases; None for facet diagonals and
    /// boundary edges.
    label_sign: Option<f64>,
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn len3(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// Fan-triangulates one face cycle.
fn fan_triangles(face: &[usize]) -> Vec<[usize; 3]> {
    let mut tris = Vec::new();
    for k in 1..face.len().saturating_sub(1) {
        let (Some(&a), Some(&b), Some(&c)) = (face.first(), face.get(k), face.get(k + 1)) else {
            continue;
        };
        tris.push([a, b, c]);
    }
    tris
}

/// Normalized, undirected edge key.
fn edge_key(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Tips (third vertices) of the triangles incident to edge (u, v).
fn third_tips(tris: &[[usize; 3]], u: usize, v: usize) -> Vec<usize> {
    tris.iter()
        .copied()
        .filter(|t| {
            let [a, b, c] = *t;
            let k = edge_key(a, b);
            let k2 = edge_key(b, c);
            let k3 = edge_key(c, a);
            k == edge_key(u, v) || k2 == edge_key(u, v) || k3 == edge_key(u, v)
        })
        .filter_map(|t| {
            let [a, b, c] = t;
            if a != u && a != v {
                Some(a)
            } else if b != u && b != v {
                Some(b)
            } else if c != u && c != v {
                Some(c)
            } else {
                None
            }
        })
        .collect()
}

/// Builds the spring network from a flat crease pattern.
fn build_springs(cp: &CreasePattern) -> Vec<Spring> {
    let mut tris: Vec<[usize; 3]> = Vec::new();
    for face in &cp.faces {
        tris.extend(fan_triangles(face));
    }

    let mut edges: Vec<(usize, usize)> = Vec::new();
    for t in &tris {
        let [a, b, c] = *t;
        for (u, v) in [(a, b), (b, c), (c, a)] {
            let key = edge_key(u, v);
            if !edges.contains(&key) {
                edges.push(key);
            }
        }
    }
    edges.sort_unstable();

    let cp_edges: Vec<((usize, usize), EdgeAssignment)> = cp
        .edges
        .iter()
        .zip(cp.assignments.iter())
        .map(|(pair, a)| (edge_key(pair[0], pair[1]), *a))
        .collect();

    let mut springs = Vec::with_capacity(edges.len());
    for (a, b) in edges {
        let (Some(pa), Some(pb)) = (cp.vertices.get(a), cp.vertices.get(b)) else {
            continue;
        };
        let dx = pa[0] - pb[0];
        let dy = pa[1] - pb[1];
        let rest_len = (dx * dx + dy * dy).sqrt();
        let assignment = cp_edges
            .iter()
            .find(|(k, _)| *k == edge_key(a, b))
            .map(|(_, assign)| *assign);
        let opposite = third_tips(&tris, a, b);
        let r = opposite.first().copied();
        let s = opposite.get(1).copied();
        let label_sign = match assignment {
            Some(EdgeAssignment::Mountain) => Some(1.0),
            Some(EdgeAssignment::Valley) => Some(-1.0),
            _ => None,
        };
        // Facet diagonals: stiff zero-angle springs keep facets flat.
        // Everything else starts neutral; targets land via
        // `apply_targets` once the fold rates are known.
        let torsion_k = if label_sign.is_none() && r.is_some() && s.is_some() {
            5.0
        } else {
            0.0
        };
        springs.push(Spring {
            a,
            b,
            r,
            s,
            rest_len,
            target_angle: 0.0,
            torsion_k,
            label_sign,
        });
    }
    springs
}

/// Signed dihedral angle across spring (a, b) at positions `x`. Sign
/// convention: facet normals come from triangle winding (FOLD/Origami
/// Simulator convention), so M/V assignments map directly to signs.
fn dihedral_angle(x: &[[f64; 3]], sp: &Spring) -> f64 {
    let (Some(r), Some(s)) = (sp.r, sp.s) else {
        return 0.0;
    };
    let (Some(&pa), Some(&pb)) = (x.get(sp.a), x.get(sp.b)) else {
        return 0.0;
    };
    let (Some(&pr), Some(&ps)) = (x.get(r), x.get(s)) else {
        return 0.0;
    };
    let dir = sub3(pa, pb);
    let dir_len = len3(dir);
    if dir_len < 1.0e-12 {
        return 0.0;
    }
    let d = [dir[0] / dir_len, dir[1] / dir_len, dir[2] / dir_len];
    let n1 = cross3(sub3(pa, pr), sub3(pb, pr));
    let n2 = cross3(sub3(pb, ps), sub3(pa, ps));
    let l1 = len3(n1);
    let l2 = len3(n2);
    if l1 < 1.0e-12 || l2 < 1.0e-12 {
        return 0.0;
    }
    let (n1u, n2u) = (
        [n1[0] / l1, n1[1] / l1, n1[2] / l1],
        [n2[0] / l2, n2[1] / l2, n2[2] / l2],
    );
    let sin_part = dot3(cross3(n1u, n2u), d);
    let cos_part = dot3(n1u, n2u);
    sin_part.atan2(cos_part)
}

/// Total bar + torsion energy at positions `x`; `ramp` scales crease
/// targets (facet targets are 0 and unaffected).
fn energy(x: &[[f64; 3]], springs: &[Spring], ramp: f64) -> f64 {
    let mut e = 0.0;
    for sp in springs {
        let (Some(&pa), Some(&pb)) = (x.get(sp.a), x.get(sp.b)) else {
            continue;
        };
        let d = sub3(pa, pb);
        let l = len3(d);
        e += 0.5 * (l - sp.rest_len) * (l - sp.rest_len);
        if sp.torsion_k > 0.0 {
            let theta = dihedral_angle(x, sp);
            let err = theta - sp.target_angle * ramp;
            e += 0.5 * sp.torsion_k * err * err;
        }
    }
    e
}

/// Per-spring residual metrics at positions `x`.
struct Residuals {
    max_axial_strain: f64,
    mean_angle_error: f64,
    max_angle_error: f64,
}

fn residuals(x: &[[f64; 3]], springs: &[Spring], ramp: f64) -> Residuals {
    let mut out = Residuals {
        max_axial_strain: 0.0,
        mean_angle_error: 0.0,
        max_angle_error: 0.0,
    };
    let mut angle_count = 0u32;
    for sp in springs {
        let (Some(&pa), Some(&pb)) = (x.get(sp.a), x.get(sp.b)) else {
            continue;
        };
        let l = len3(sub3(pa, pb));
        if sp.rest_len > 1.0e-12 {
            let strain = (l - sp.rest_len).abs() / sp.rest_len;
            if strain > out.max_axial_strain {
                out.max_axial_strain = strain;
            }
        }
        if sp.torsion_k > 0.0 {
            let err = (dihedral_angle(x, sp) - sp.target_angle * ramp).abs();
            out.mean_angle_error += err;
            angle_count += 1;
            if err > out.max_angle_error {
                out.max_angle_error = err;
            }
        }
    }
    if angle_count > 0 {
        out.mean_angle_error /= f64::from(angle_count);
    }
    out
}

/// One projection sweep: creep torsion springs toward their (ramped)
/// targets by rotating the two opposite tips (displacement-capped),
/// re-center on the centroid, then project bars to rest length exactly.
/// Bars last so the sweep's end state satisfies lengths.
fn project_sweeps(x: &mut [[f64; 3]], springs: &[Spring], ramp: f64, relax: f64, cap: f64) {
    // Re-center: kills rigid-body translation drift.
    if x.len() > 1 {
        let mut mean = [0.0f64; 3];
        for p in x.iter() {
            mean[0] += p[0];
            mean[1] += p[1];
            mean[2] += p[2];
        }
        #[allow(clippy::cast_precision_loss)] // vertex count, exact below 2^53
        let inv = 1.0 / x.len() as f64;
        mean = [mean[0] * inv, mean[1] * inv, mean[2] * inv];
        for p in x.iter_mut() {
            *p = [p[0] - mean[0], p[1] - mean[1], p[2] - mean[2]];
        }
    }

    // Torsion: rotate the two opposite tips about the crease axis by
    // half the angle error each, in opposite senses. Rotating r by
    // delta and s by -delta moves the dihedral by -2 delta under
    // dihedral_angle's convention, so delta carries (theta - target).
    for sp in springs {
        if sp.torsion_k <= 0.0 {
            continue;
        }
        let theta = dihedral_angle(x, sp);
        let delta = (0.5 * (theta - sp.target_angle * ramp) * relax).clamp(-cap, cap);
        let (Some(&pa), Some(&pb)) = (x.get(sp.a), x.get(sp.b)) else {
            continue;
        };
        let d = sub3(pa, pb);
        let l = len3(d);
        if l < 1.0e-12 {
            continue;
        }
        let u = [d[0] / l, d[1] / l, d[2] / l];
        let origin = pa;
        if let Some(r) = sp.r {
            if let Some(p) = x.get(r).copied() {
                let rotated = rotate_about_axis(p, origin, u, delta);
                if let Some(slot) = x.get_mut(r) {
                    *slot = rotated;
                }
            }
        }
        if let Some(s) = sp.s {
            if let Some(p) = x.get(s).copied() {
                let rotated = rotate_about_axis(p, origin, u, -delta);
                if let Some(slot) = x.get_mut(s) {
                    *slot = rotated;
                }
            }
        }
    }

    // Bars: exact projection (a pulls toward b, b pulls toward a).
    for sp in springs {
        let (Some(&pa), Some(&pb)) = (x.get(sp.a), x.get(sp.b)) else {
            continue;
        };
        let d = sub3(pa, pb);
        let l = len3(d);
        if l < 1.0e-12 || sp.rest_len < 1.0e-12 {
            continue;
        }
        let err = (l - sp.rest_len) * 0.5;
        let u = [d[0] / l, d[1] / l, d[2] / l];
        if let Some(fa) = x.get_mut(sp.a) {
            *fa = [pa[0] - u[0] * err, pa[1] - u[1] * err, pa[2] - u[2] * err];
        }
        if let Some(fb) = x.get_mut(sp.b) {
            *fb = [pb[0] + u[0] * err, pb[1] + u[1] * err, pb[2] + u[2] * err];
        }
    }
}

/// Rotates point `p` about the axis through `origin` with unit
/// direction `u` by `angle` (Rodrigues).
fn rotate_about_axis(p: [f64; 3], origin: [f64; 3], u: [f64; 3], angle: f64) -> [f64; 3] {
    let rel = sub3(p, origin);
    let c = angle.cos();
    let s = angle.sin();
    let cross_u_rel = cross3(u, rel);
    let along = dot3(u, rel) * (1.0 - c);
    [
        rel[0] * c + cross_u_rel[0] * s + u[0] * along + origin[0],
        rel[1] * c + cross_u_rel[1] * s + u[1] * along + origin[1],
        rel[2] * c + cross_u_rel[2] * s + u[2] * along + origin[2],
    ]
}

/// Solves the dense linear system `a * y = b` by Gaussian elimination
/// with partial pivoting. Returns None when singular.
fn solve_linear(a: &mut [Vec<f64>], b: &mut [f64]) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let mut pivot = col;
        let mut best = 0.0f64;
        for ri in col..n {
            let v = a
                .get(ri)
                .and_then(|row| row.get(col))
                .copied()
                .unwrap_or(0.0)
                .abs();
            if v > best {
                best = v;
                pivot = ri;
            }
        }
        if best < 1.0e-14 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        let pv = a
            .get(col)
            .and_then(|row| row.get(col))
            .copied()
            .unwrap_or(0.0);
        if let Some(row) = a.get_mut(col) {
            for cell in row.iter_mut() {
                *cell /= pv;
            }
        }
        if let Some(bcell) = b.get_mut(col) {
            *bcell /= pv;
        }
        for ri in 0..n {
            if ri == col {
                continue;
            }
            let factor = a
                .get(ri)
                .and_then(|row| row.get(col))
                .copied()
                .unwrap_or(0.0);
            if factor.abs() < 1.0e-15 {
                continue;
            }
            let pivot_row = a.get(col).cloned().unwrap_or_default();
            if let Some(row) = a.get_mut(ri) {
                for (cell, prow) in row.iter_mut().zip(pivot_row.iter()) {
                    *cell -= factor * prow;
                }
            }
            let pv_b = b.get(col).copied().unwrap_or(0.0);
            if let Some(bcell) = b.get_mut(ri) {
                *bcell -= factor * pv_b;
            }
        }
    }
    Some(b.to_vec())
}

/// Kinematic fold rates for the pattern's driven creases (Tachi-style
/// opposite-equality, Schenk & Guest: at a rigid-foldable 4-valent
/// vertex, opposite creases carry equal fold rates). Returns
/// `Some(rates)` (normalized to max magnitude 1, in driven-spring
/// order) when the constraint system has a null-space direction
/// closest to uniform folding; `None` when the pattern is
/// unconstrained (no interior 4-valent vertices) or the label pattern
/// has no compatible fold direction.
#[must_use]
pub fn fold_rates(cp: &CreasePattern) -> Option<Vec<f64>> {
    let springs = build_springs(cp);
    let driven: Vec<usize> = springs
        .iter()
        .enumerate()
        .filter(|(_, sp)| sp.label_sign.is_some())
        .map(|(i, _)| i)
        .collect();
    if driven.is_empty() {
        return None;
    }
    let crease_of: Vec<(usize, usize)> = driven
        .iter()
        .filter_map(|&i| {
            let sp = springs.get(i)?;
            Some(edge_key(sp.a, sp.b))
        })
        .collect();
    let rows = rate_constraint_rows(cp, &crease_of, nc_of(&springs, &driven))?;
    let rates = null_space_uniform_direction(rows, nc_of(&springs, &driven))?;
    // Permute rates back to driven-spring order is unnecessary: rows
    // already reference driven indices directly.
    Some(rates)
}

fn nc_of(springs: &[Spring], driven: &[usize]) -> usize {
    let upper = driven.last().map_or(0, |&i| i + 1);
    upper.min(springs.len()).max(driven.len())
}

/// Builds the opposite-equality constraint rows over driven creases.
fn rate_constraint_rows(
    cp: &CreasePattern,
    crease_of: &[(usize, usize)],
    nc: usize,
) -> Option<Vec<Vec<f64>>> {
    let mut adj: Vec<Vec<usize>> = alloc::vec![Vec::new(); cp.vertices.len()];
    for (k, (u, v)) in crease_of.iter().enumerate() {
        if let Some(slot) = adj.get_mut(*u) {
            slot.push(k);
        }
        if let Some(slot) = adj.get_mut(*v) {
            slot.push(k);
        }
    }

    let mut rows: Vec<Vec<f64>> = Vec::new();
    for (vi, inc) in adj.iter().enumerate() {
        if inc.len() != 4 {
            continue;
        }
        let base = cp.vertices.get(vi)?;
        let mut with_angle: Vec<(f64, usize)> = Vec::new();
        for &k in inc {
            let (u, v) = crease_of.get(k)?;
            let other = if *u == vi { *v } else { *u };
            let p = cp.vertices.get(other)?;
            with_angle.push(((p[1] - base[1]).atan2(p[0] - base[0]), k));
        }
        if with_angle.len() != 4 {
            continue;
        }
        with_angle.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut r = alloc::vec![0.0; nc];
        for (pos, (_, k)) in with_angle.iter().enumerate() {
            let sign = if pos % 2 == 0 { 1.0 } else { -1.0 };
            if let Some(cell) = r.get_mut(*k) {
                *cell = sign;
            }
        }
        rows.push(r);
    }
    if rows.is_empty() {
        None
    } else {
        Some(rows)
    }
}

/// Picks the null-space direction closest to uniform folding (all-ones
/// projection, normalized). Vanishing projection: None.
/// Gauss-Jordan-eliminates the constraint rows; returns the reduced
/// rows and the pivot column list.
fn reduce_rows(mut rows: Vec<Vec<f64>>, nc: usize) -> (Vec<Vec<f64>>, Vec<usize>) {
    let nrows = rows.len();
    let mut pivot_cols: Vec<usize> = Vec::new();
    let mut r = 0usize;
    for col in 0..nc {
        let mut pivot = None;
        let mut best = 0.0f64;
        for ri in r..nrows {
            let val = rows
                .get(ri)
                .and_then(|row| row.get(col))
                .copied()
                .unwrap_or(0.0)
                .abs();
            if val > best {
                best = val;
                pivot = Some(ri);
            }
        }
        let Some(pi) = pivot else {
            continue;
        };
        rows.swap(r, pi);
        let pv = rows
            .get(r)
            .and_then(|row| row.get(col))
            .copied()
            .unwrap_or(0.0);
        if pv.abs() < 1.0e-12 {
            continue;
        }
        if let Some(row) = rows.get_mut(r) {
            for cell in row.iter_mut() {
                *cell /= pv;
            }
        }
        for ri in 0..nrows {
            if ri == r {
                continue;
            }
            let factor = rows
                .get(ri)
                .and_then(|row| row.get(col))
                .copied()
                .unwrap_or(0.0);
            if factor.abs() < 1.0e-15 {
                continue;
            }
            let pivot_row = rows.get(r).cloned().unwrap_or_default();
            if let Some(row) = rows.get_mut(ri) {
                for (cell, pr) in row.iter_mut().zip(pivot_row.iter()) {
                    *cell -= factor * pr;
                }
            }
        }
        pivot_cols.push(col);
        r += 1;
        if r == nrows {
            break;
        }
    }
    (rows, pivot_cols)
}

fn null_space_uniform_direction(rows: Vec<Vec<f64>>, nc: usize) -> Option<Vec<f64>> {
    let (rows, pivot_cols) = reduce_rows(rows, nc);
    let free: Vec<usize> = (0..nc).filter(|c| !pivot_cols.contains(c)).collect();
    if free.is_empty() {
        return None;
    }

    // Null-space basis: one unit vector per free column, pivot entries
    // filled from the reduced rows.
    let mut basis: Vec<Vec<f64>> = Vec::new();
    for &f in &free {
        let mut v = alloc::vec![0.0; nc];
        if let Some(slot) = v.get_mut(f) {
            *slot = 1.0;
        }
        for (ri, &p) in pivot_cols.iter().enumerate() {
            let coef = rows
                .get(ri)
                .and_then(|row| row.get(f))
                .copied()
                .unwrap_or(0.0);
            if let Some(slot) = v.get_mut(p) {
                *slot = -coef;
            }
        }
        basis.push(v);
    }

    // Fold mode = null-space projection of the all-ones vector.
    let kdim = basis.len();
    let mut gram: Vec<Vec<f64>> = alloc::vec![alloc::vec![0.0; kdim]; kdim];
    let mut rhs: Vec<f64> = alloc::vec![0.0; kdim];
    for bi in 0..kdim {
        for bj in 0..kdim {
            let mut acc = 0.0;
            for c in 0..nc {
                let pair = basis
                    .get(bi)
                    .and_then(|row| row.get(c))
                    .zip(basis.get(bj).and_then(|row| row.get(c)));
                if let Some((&a, &b)) = pair {
                    acc += a * b;
                }
            }
            if let Some(cell) = gram.get_mut(bi).and_then(|row| row.get_mut(bj)) {
                *cell = acc;
            }
        }
        let mut acc = 0.0;
        for c in 0..nc {
            if let Some(&a) = basis.get(bi).and_then(|row| row.get(c)) {
                acc += a;
            }
        }
        if let Some(cell) = rhs.get_mut(bi) {
            *cell = acc;
        }
    }
    let coeffs = solve_linear(&mut gram, &mut rhs)?;
    let mut rates = alloc::vec![0.0; nc];
    for (bi, &coeff) in coeffs.iter().enumerate() {
        for (c, rate) in rates.iter_mut().enumerate() {
            if let Some(&bv) = basis.get(bi).and_then(|row| row.get(c)) {
                *rate += coeff * bv;
            }
        }
    }
    let norm = rates.iter().fold(0.0f64, |acc, v| acc.max(v.abs()));
    if norm < 1.0e-9 {
        return None;
    }
    for rate in &mut rates {
        *rate /= norm;
    }
    Some(rates)
}

/// Applies crease targets per `mode`. Rates mode: sign of the solved
/// rate times `fold_percent` times `pi`; zero-rate creases are passive
/// (free hinges). Labels mode or rates fallback: label sign times
/// `fold_percent` times `pi`.
fn apply_targets(
    springs: &mut [Spring],
    mode: TargetMode,
    rates: Option<&[f64]>,
    fold_percent: f64,
) {
    for (i, sp) in springs.iter_mut().enumerate() {
        let Some(sign) = sp.label_sign else {
            continue;
        };
        let rate = if mode == TargetMode::Rates {
            rates.and_then(|r| r.get(i).copied())
        } else {
            None
        };
        match rate {
            Some(rate) if rate.abs() > 1.0e-9 => {
                sp.target_angle = rate.signum() * fold_percent * PI;
                sp.torsion_k = 0.2;
            }
            Some(_) => sp.torsion_k = 0.0,
            None => {
                sp.target_angle = sign * fold_percent * PI;
                sp.torsion_k = 0.2;
            }
        }
    }
}

/// Runs the folded-state relaxation: repeated constraint-projection
/// sweeps until both tolerances hold (or the sweep cap hits).
#[must_use]
pub fn simulate_fold(cp: &CreasePattern, params: &SimParams) -> SimReport {
    let mut springs = build_springs(cp);
    let rates = match params.target_mode {
        TargetMode::Rates => fold_rates(cp),
        TargetMode::Labels => None,
    };
    apply_targets(
        &mut springs,
        params.target_mode,
        rates.as_deref(),
        params.fold_percent,
    );
    let mut x: Vec<[f64; 3]> = cp.vertices.iter().map(|p| [p[0], p[1], 0.0]).collect();

    let mut res = residuals(&x, &springs, 1.0);
    let mut iterations = 0u32;
    let mut converged = false;

    while iterations < params.max_iterations {
        if res.max_angle_error < params.angle_tol_rad && res.max_axial_strain < params.strain_tol {
            converged = true;
            break;
        }
        iterations += 1;
        let ramp = if params.ramp_fraction <= 0.0 {
            1.0
        } else {
            let budget = params.ramp_fraction * f64::from(params.max_iterations);
            (f64::from(iterations) / budget).min(1.0)
        };
        project_sweeps(&mut x, &springs, ramp, params.relax, params.step_cap);
        res = residuals(&x, &springs, 1.0);
    }

    let total_energy = energy(&x, &springs, 1.0);
    let crease_angles: Vec<(f64, f64)> = springs
        .iter()
        .filter(|sp| sp.torsion_k > 0.0)
        .map(|sp| (dihedral_angle(&x, sp), sp.target_angle))
        .collect();
    SimReport {
        iterations,
        converged,
        max_axial_strain: res.max_axial_strain,
        mean_angle_error_rad: res.mean_angle_error,
        max_angle_error_rad: res.max_angle_error,
        total_energy,
        vertices: x,
        crease_angles,
    }
}

/// Outcome of a folded-state simulation.
#[derive(Debug, Clone)]
pub struct SimReport {
    /// Sweeps actually taken.
    pub iterations: u32,
    /// True when both tolerances were met before the cap.
    pub converged: bool,
    /// Largest bar stretch |L - L0| / L0 at the end state.
    pub max_axial_strain: f64,
    /// Mean torsion-spring angle error, radians.
    pub mean_angle_error_rad: f64,
    /// Largest torsion-spring angle error, radians.
    pub max_angle_error_rad: f64,
    /// Total bar + torsion energy at the end state (arb. units).
    pub total_energy: f64,
    /// Final vertex positions.
    pub vertices: Vec<[f64; 3]>,
    /// Per torsion spring: (achieved angle, target angle) in radians,
    /// in spring build order.
    pub crease_angles: Vec<(f64, f64)>,
}
