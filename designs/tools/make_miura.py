#!/usr/bin/env python3
"""Generates the seed Miura-ori crease pattern for the design library.

Emits designs/examples/miura-ori/miura.fold (FOLD JSON) plus meta.toml.
Self-verifying: before writing, every interior vertex is checked against
Maekawa (|M-V| = 2) and Kawasaki (alternating sector angles sum to 180
degrees per pair), using the same semantics as origami-core's gates.

Run from the repo root:  python3 designs/tools/make_miura.py
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

CELLS_X = 4
CELLS_Y = 5
OFFSET = 0.5
DESIGN_DIR = Path(__file__).resolve().parent.parent / "examples" / "miura-ori"


def vertex_pos(i: int, j: int) -> tuple[float, float]:
    """Lattice position: rows alternate a half-cell offset."""
    return (i + OFFSET * (j % 2), float(j))


def build_graph() -> tuple[dict, list, list, dict]:
    """Builds vertices, horizontal creases, vertical creases, edges."""
    verts: dict[tuple[int, int], int] = {}
    positions: list[tuple[float, float]] = []
    for j in range(CELLS_Y + 1):
        for i in range(CELLS_X + 1):
            verts[(i, j)] = len(positions)
            positions.append(vertex_pos(i, j))

    edges: list[tuple[int, int]] = []
    horiz: dict[tuple[int, int], int] = {}
    vert: dict[tuple[int, int], int] = {}
    for j in range(CELLS_Y + 1):
        for i in range(CELLS_X):
            horiz[(i, j)] = len(edges)
            edges.append((verts[(i, j)], verts[(i + 1, j)]))
    for j in range(CELLS_Y):
        for i in range(CELLS_X + 1):
            vert[(i, j)] = len(edges)
            edges.append((verts[(i, j)], verts[(i, j + 1)]))
    return verts, horiz, vert, edges


def interior_vertex(i: int, j: int) -> bool:
    return 1 <= i <= CELLS_X - 1 and 1 <= j <= CELLS_Y - 1


def assign_edges(verts, horiz, vert) -> list[str]:
    """Mountain/valley assignment; sheet-outline edges become B.

    Outline = bottom/top horizontal rows and left/right vertical columns.
    Horizontals alternate by row (collinear pairs share the assignment);
    verticals use a checkerboard, so every interior vertex sees a 3/1
    mountain/valley split (Maekawa).
    """
    assignments = ["B"] * (len(horiz) + len(vert))
    for (i, j), edge_id in horiz.items():
        if j == 0 or j == CELLS_Y:
            continue  # outline edge
        assignments[edge_id] = "M" if j % 2 == 0 else "V"
    for (i, j), edge_id in vert.items():
        if i == 0 or i == CELLS_X:
            continue  # outline edge
        assignments[edge_id] = "M" if (i + j) % 2 == 0 else "V"
    return assignments


def verify_foldability(positions, edges, assignments) -> None:
    """Numeric Maekawa + Kawasaki check, matching origami-core semantics."""
    adj: dict[int, list[int]] = {v: [] for v in range(len(positions))}
    for edge_id, (a, b) in enumerate(edges):
        adj[a].append(edge_id)
        adj[b].append(edge_id)
    checked = 0
    for v_id, incident in adj.items():
        if len(incident) != 4:
            continue
        tokens = [assignments[e] for e in incident]
        if any(t in ("B", "U") for t in tokens):
            continue
        mountains = tokens.count("M")
        valleys = tokens.count("V")
        assert abs(mountains - valleys) == 2, f"Maekawa violated at {v_id}"
        angles = []
        for e in incident:
            a, b = edges[e]
            other = b if a == v_id else a
            dx = positions[other][0] - positions[v_id][0]
            dy = positions[other][1] - positions[v_id][1]
            angles.append(math.atan2(dy, dx))
        angles.sort()
        sectors = []
        for k in range(4):
            span = angles[(k + 1) % 4] - angles[k]
            if span <= 0:
                span += 2 * math.pi
            sectors.append(span)
        residual = (sectors[0] + sectors[2]) - (sectors[1] + sectors[3])
        assert abs(residual) < 1e-9, f"Kawasaki violated at {v_id}: {residual}"
        checked += 1
    assert checked >= (CELLS_X - 1) * (CELLS_Y - 1), "too few interior vertices checked"


def faces_of(positions, edges) -> list[list[int]]:
    """Faces of the grid tiling: the parallelogram cell per lattice site.

    The generator knows the tiling it built, so faces are constructed
    analytically (one quadrilateral per cell) instead of traced. An Euler
    assert guards the result: V - E + F = 2 with one exterior face.
    """
    vidx: dict[tuple[int, int], int] = {}
    for idx, (j, i) in enumerate((j, i) for j in range(CELLS_Y + 1) for i in range(CELLS_X + 1)):
        vidx[(i, j)] = idx
    kept = []
    for j in range(CELLS_Y):
        for i in range(CELLS_X):
            kept.append([
                vidx[(i, j)],
                vidx[(i + 1, j)],
                vidx[(i + 1, j + 1)],
                vidx[(i, j + 1)],
            ])
    expected = CELLS_X * CELLS_Y
    v_count = len(positions)
    e_count = len(edges)
    f_total = len(kept) + 1  # + exterior face
    assert v_count - e_count + f_total == 2, (
        f"Euler check failed: V{v_count} E{e_count} F{f_total}"
    )
    assert len(kept) == expected, f"expected {expected} facets, got {len(kept)}"
    return kept


def main() -> int:
    verts, horiz, vert, edges = build_graph()
    positions = [vertex_pos(i, j) for j in range(CELLS_Y + 1) for i in range(CELLS_X + 1)]
    assignments = assign_edges(verts, horiz, vert)
    verify_foldability(positions, edges, assignments)
    faces = faces_of(positions, edges)

    fold = {
        "file_spec": "fold",
        "file_version": 1.0,
        "file_creator": "designs/tools/make_miura.py",
        "file_title": "Miura-ori seed pattern",
        "vertices_coords": [[x, y] for (x, y) in positions],
        "edges_vertices": [list(pair) for pair in edges],
        "edges_assignments": assignments,
        "faces_vertices": faces,
    }
    DESIGN_DIR.mkdir(parents=True, exist_ok=True)
    (DESIGN_DIR / "miura.fold").write_text(json.dumps(fold, indent=1) + "\n")
    meta = f"""# Seed design: generated Miura-ori crease pattern
name = "miura-ori-4x5"
description = "Flat-foldable Miura tessellation, {CELLS_X}x{CELLS_Y} cells, generated and gate-verified"
license = "CC0-1.0"
status = "verified-flat-foldable"
material = "unassigned"
generator = "designs/tools/make_miura.py"
created = "2026-11-20"
civilian = true
controlled = false
"""
    (DESIGN_DIR / "meta.toml").write_text(meta)
    interior = sum(
        1
        for j in range(1, CELLS_Y)
        for i in range(1, CELLS_X)
    )
    print(f"wrote {DESIGN_DIR}/miura.fold ({interior} interior vertices, {len(faces)} faces)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
