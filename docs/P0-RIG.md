# P0: crease fold-test rig: COMPLETE PLAN (gated: DO NOT ORDER)

status: plan complete, awaiting operator approval
Date: 2026-11-20
Gate: **No hardware is ordered or built until the operator approves this
plan.** Every cost is an estimate; every part is a candidate pending a
supplier check at approval time.

Purpose: replace every `status = "estimate"` in `data/materials/` with
measured crease stiffness and fatigue data. The data is the paid
materials DB moat; the rig is commodity bench hardware.

## 1. Measurement targets

| Output | Definition | Feeds |
|---|---|---|
| Crease stiffness curve | torque vs fold angle, 0 to 180 deg | k-calibration constant C in `materials.rs` |
| Fatigue life N(r/t) | cycles-to-failure at fixed bend radius ratio | `crease_fatigue` field |
| Dynamic min radius | largest r/t with N >= 10,000 cycles | `min_fold_radius_over_thickness` |

## 2. Bill of materials (estimates, candidate specs)

| Item | Candidate spec | Est. cost (USD) |
|---|---|---|
| Linear stage | NEMA 17 stepper + T8 leadscrew 300 mm + couplers + limit switches | 80 to 160 |
| Stepper driver + PSU | TMC2209 driver board + 24 V 5 A supply | 30 to 60 |
| Load cell | S-type 20 kgf, 2 mV/V | 15 to 40 |
| Amplifier | HX711 24-bit (start) or INA125; upgrade path: 16-bit ADC | 8 to 25 |
| Rotary encoder | magnetic encoder on fold axis, 0.1 deg resolution | 10 to 30 |
| Frame | 2040 aluminum extrusion 4x + brackets + fasteners | 40 to 90 |
| Mandrel set | steel rods 1/2/4/8 mm + 0.2 mm shim stock | 20 to 50 |
| Controller | RP2040 or ESP32 dev board (fixed-rate loop, USB serial) | 6 to 15 |
| Misc | wires, mounts, adhesive, safety glasses | 20 to 40 |
| **Total** | | **229 to 510** |

Notes: quotes are ranges because prices drift; confirm at approval.
Accuracy start-point: HX711 gives ~20 g resolution on 20 kgf cell, enough
for crease torque at 100 mm crease length on stiff films; the 20 kgf cell
is oversized on purpose (launch-shock specimens later). If resolution
proves too coarse for paper, add a 2 kgf cell (+15 USD) rather than
restarting the design.

## 3. Apparatus

- Actuator drives a folding jaw pair through a four-bar; fold axis on
  the encoder; load cell in-line with the driven jaw.
- Mandrels mount between jaws to set r/t; specimen wraps the mandrel.
- Frame: two plates on extrusion, one fixed, one actuator-driven.

## 4. Method

1. Specimen: 20 mm x 100 mm strip, single crease at mid-length,
   pre-formed over the mandrel (defines r/t).
2. Static run: sweep 0 to 180 deg at fixed speed, log torque + angle
   (3 cycles, take the 2nd; 1st conditions the crease).
3. Fatigue run: cycle 0 to 180 deg until force drops 20% from the
   conditioned peak; record N.
4. Matrix: 5 materials x 3 radii (r/t = 5, 20, 40) x n = 5 specimens
   (med + min reported; outliers kept, never dropped silently).

## 5. Data contract

- `data/rig/runs/<run-id>.csv`: `t_ms, force_n, angle_deg, temp_c`
  at >= 100 Hz, one file per run, run-id = R-NNN.
- `data/rig/runs/index.toml`: run-id, material id, r/t, specimen n,
  speed, operator, date, notes.
- Materials TOML flips `status` to `measured`, cites run-ids, and the
  stiffness calibration constant C lands in the repo as a measured
  constant (still per-material).
- No entry reaches `measured` without a CSV + index row traceable to
  it (REQ-O-005; spec-audit checks the chain once the first run lands).

## 6. Procedure (per material batch)

1. Calibrate load cell with hanging weights spanning 10 to 90% FS;
   record zero-drift before each run.
2. Condition specimens at room temp; record temp + humidity per run.
3. Static sweep x3 per specimen.
4. Fatigue cycling per radius; stop at 10^5 cycles unclosed = "10k+
   survivor" (right-censored datum, recorded as such).
5. Export CSV + index row; flip materials entry only after acceptance.

## 7. Acceptance (this is what "done" means)

- 3 materials fully measured (paper, Kapton, 301 SS) with published
  stiffness curves and N(r/t) medians.
- Calibration constant C per material class documented with residuals.
- Every claimed number traceable to a CSV under `data/rig/runs/`.
- SPEC section 5 oracle (schema validator) ships with the first index
  file; `material-check --gate` passes on measured entries.

## 8. Risk register

| Risk | Mitigation |
|---|---|
| HX711 too coarse for paper | add 2 kgf cell; keep fixture unchanged |
| Crease placement variability | crease jig over mandrel; pre-form fixture |
| Cyclying overheats films | cap cycle rate; record temp; pause policy |
| Speed sensitivity | static sweep at 3 speeds first; pick flat region |

## 9. Approval block

- [x] Operator approves BOM + budget band (2026-11-21, "proceed all")
- [ ] Operator picks suppliers / sources parts (physical; pending)
- [ ] Build + calibration, then tasks land per section 7

The software-side deliverables of this plan (index schema, rig-check
validator, CSV contract) are implemented and gated in-repo; hardware
steps remain physical operator actions.
