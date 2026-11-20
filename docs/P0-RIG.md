# P0: crease fold-test rig (benchtop)

status: spec, hardware not ordered
Date: 2026-11-20

Purpose: replace every `status = "estimate"` in `data/materials/` with
measured crease stiffness and fatigue data. This rig is the seed asset
of the paid materials DB: the data is the moat, not the code.

## Apparatus

| Part | Requirement |
|---|---|
| Linear actuator | stepper + leadscrew, 0 to 200 mm travel, speed 1 to 100 mm/s |
| Load cell | 5 to 50 kgf range, 0.1% FS; mounts in-line with actuator |
| Mandrel set | radii 0.2/0.5/1/2/4 mm (wraps the specimen for a fixed r/t) |
| Cycle counter | actuator encoder count; failure = force drop past 20% |
| Data | force-displacement at >=100 Hz to CSV per run |

## Method

1. Specimen: 20 mm x 100 mm strip, creased once at mid-length over a
   mandrel, then cycled 0 to 180 degrees in the rig.
2. Static run: torque/force vs angle sweep (stiffness curve per crease).
3. Fatigue run: cycles-to-failure N at each mandrel radius.
4. Matrix: 5 materials x 3 radii (r/t = 5, 20, 40) x n = 5 specimens.

## Materials queue

| Run | Material | Entry |
|---|---|---|
| R-001 | copy paper 80 gsm | `data/materials/paper-copy-80gsm.toml` |
| R-002/R-003 | Kapton HN 125 um | `data/materials/kapton-hn-125um.toml` |
| R-004 | 301 SS full hard 200 um | `data/materials/ss301-fullhard-200um.toml` |
| R-005 | PEEK film 125 um | `data/materials/peek-film-125um.toml` |

## Outputs

- `data/rig/runs/<run-id>.csv`: raw force-displacement.
- Materials TOML files flip `status` to `measured` and cite the run id.
- Foldability gate semantics unchanged; stiffness/fatigue feed the
  bar-and-hinge calibration (future module).

## Acceptance

- 3 materials fully measured (paper, Kapton, 301 SS) with published
  stiffness curves and N(r/t) fatigue medians.
- Every claimed number traceable to a CSV under `data/rig/runs/`.
