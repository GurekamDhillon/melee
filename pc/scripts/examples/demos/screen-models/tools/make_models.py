#!/usr/bin/env python3
"""Writes this demo's twelve original test models (GXMS v2) and their shared atlas (GXTX v1 RGBA8).

Plain geometry written from scratch for the demo: nothing here is derived from a game or from any
other project's assets (the Envoy drive models are Sonic Adventure 2 data and are NOT used). Python
standard library only.   usage: python make_models.py [../models]
"""
import math
import struct
import sys
from pathlib import Path

ATLAS = 256
COLS, ROWS = 4, 3
CELL_W, CELL_H = ATLAS // COLS, ATLAS // ROWS  # 64 x 85


def sub(a, b): return (a[0] - b[0], a[1] - b[1], a[2] - b[2])
def cross(a, b): return (a[1]*b[2] - a[2]*b[1], a[2]*b[0] - a[0]*b[2], a[0]*b[1] - a[1]*b[0])
def dot(a, b): return a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
def norm(a):
    n = math.sqrt(dot(a, a)) or 1.0
    return (a[0]/n, a[1]/n, a[2]/n)


def ring(n, r, y, phase=0.0):
    return [(r*math.cos(phase + 2*math.pi*i/n), y, r*math.sin(phase + 2*math.pi*i/n)) for i in range(n)]


def quad(a, b, c, d): return [(a, b, c), (a, c, d)]


def octahedron():
    t, b = (0, 1.0, 0), (0, -1.0, 0)
    r = ring(4, 0.8, 0)
    return [(t, r[i], r[(i+1) % 4]) for i in range(4)] + [(b, r[(i+1) % 4], r[i]) for i in range(4)]


def cube():
    s = 0.62
    v = [(x*s, y*s, z*s) for x in (-1, 1) for y in (-1, 1) for z in (-1, 1)]
    f = [(0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)]
    return [t for q in f for t in quad(*[v[i] for i in q])]


def prism(n, r, h, caps=True):
    top, bot = ring(n, r, h/2), ring(n, r, -h/2)
    out = []
    for i in range(n):
        j = (i+1) % n
        out += quad(bot[i], bot[j], top[j], top[i])
    if caps:
        for i in range(1, n-1):
            out.append((top[0], top[i], top[i+1]))
            out.append((bot[0], bot[i+1], bot[i]))
    return out


def bipyramid(n, r, h, belt=0.0):
    t, b = (0, h, 0), (0, -h, 0)
    m = ring(n, r, belt)
    return [(t, m[i], m[(i+1) % n]) for i in range(n)] + [(b, m[(i+1) % n], m[i]) for i in range(n)]


def pyramid():
    t = (0, 0.95, 0)
    base = ring(4, 0.85, -0.7, math.pi/4)
    out = [(t, base[i], base[(i+1) % 4]) for i in range(4)]
    out += [(base[0], base[2], base[1]), (base[0], base[3], base[2])]
    return out


def tetrahedron():
    v = [(0.9, 0.9, 0.9), (-0.9, -0.9, 0.9), (-0.9, 0.9, -0.9), (0.9, -0.9, -0.9)]
    v = [(x*0.85, y*0.85, z*0.85) for x, y, z in v]
    return [(v[0], v[1], v[2]), (v[0], v[3], v[1]), (v[0], v[2], v[3]), (v[1], v[3], v[2])]


def icosahedron():
    p = (1 + math.sqrt(5)) / 2
    v = [(-1, p, 0), (1, p, 0), (-1, -p, 0), (1, -p, 0), (0, -1, p), (0, 1, p), (0, -1, -p), (0, 1, -p),
         (p, 0, -1), (p, 0, 1), (-p, 0, -1), (-p, 0, 1)]
    v = [tuple(c * 0.5 for c in x) for x in v]
    f = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11), (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6),
         (7, 1, 8), (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9), (4, 9, 5), (2, 4, 11), (6, 2, 10),
         (8, 6, 7), (9, 8, 1)]
    return [(v[a], v[b], v[c]) for a, b, c in f]


def cone():
    t = (0, 1.0, 0)
    base = ring(10, 0.75, -0.8)
    out = [(t, base[i], base[(i+1) % 10]) for i in range(10)]
    out += [((0, -0.8, 0), base[(i+1) % 10], base[i]) for i in range(10)]
    return out


def torus():
    """Smooth-normal torus: returns (triangle, per-corner normals) pairs."""
    R, r, nu, nv = 0.62, 0.3, 14, 8
    def pt(i, j):
        u, v = 2*math.pi*i/nu, 2*math.pi*j/nv
        c = (R*math.cos(u), 0, R*math.sin(u))
        n = (math.cos(v)*math.cos(u), math.sin(v), math.cos(v)*math.sin(u))
        return (c[0] + r*n[0], c[1] + r*n[1], c[2] + r*n[2]), n
    out = []
    for i in range(nu):
        for j in range(nv):
            a, b = pt(i, j), pt(i+1, j)
            c, d = pt(i+1, j+1), pt(i, j+1)
            out.append(((a[0], b[0], c[0]), (a[1], b[1], c[1])))
            out.append(((a[0], c[0], d[0]), (a[1], c[1], d[1])))
    return out


SHAPES = [
    ("gem_octa", octahedron, False), ("gem_cube", cube, False), ("gem_tprism", lambda: prism(3, 0.8, 1.5), False),
    ("gem_hexbi", lambda: bipyramid(6, 0.75, 1.0, 0.0), False), ("gem_icosa", icosahedron, False),
    ("gem_hexprism", lambda: prism(6, 0.7, 1.5), False), ("gem_pyramid", pyramid, False),
    ("gem_tetra", tetrahedron, False), ("gem_coin", lambda: prism(12, 0.95, 0.45), False),
    ("gem_wide", lambda: bipyramid(8, 1.0, 0.6, 0.0), False), ("gem_torus", torus, True),
    ("gem_cone", cone, False),
]


def build(index, tris, smooth):
    """Vertices unshared per triangle: planar UV into this model's atlas cell on the face's dominant axis."""
    cu, cv = (index % COLS) * CELL_W, (index // COLS) * CELL_H
    verts, idx = [], []
    for item in tris:
        tri, normals = (item if smooth else (item, None))
        n = norm(cross(sub(tri[1], tri[0]), sub(tri[2], tri[0])))
        cen = tuple(sum(p[k] for p in tri) / 3 for k in range(3))
        if smooth:
            want = tuple(sum(m[k] for m in normals) for k in range(3))
            flip = dot(n, want) < 0
        else:
            flip = dot(n, cen) < 0  # outward: every shape here is star-shaped about the origin
        if flip:
            tri = (tri[0], tri[2], tri[1])
            normals = (normals[0], normals[2], normals[1]) if smooth else None
            n = (-n[0], -n[1], -n[2])
        ax = max(range(3), key=lambda k: abs(n[k]))
        a, b = [k for k in range(3) if k != ax]
        for k, p in enumerate(tri):
            u = (cu + 2 + (p[a] * 0.5 / 1.05 + 0.5) * (CELL_W - 4)) / ATLAS
            v = (cv + 2 + (0.5 - p[b] * 0.5 / 1.05) * (CELL_H - 4)) / ATLAS
            nn = normals[k] if smooth else n
            idx.append(len(verts))
            verts.append((p[0], p[1], p[2], u, v, nn[0], nn[1], nn[2]))
    return verts, idx


def write_mesh(path, verts, idx):
    xs, ys, zs = ([v[k] for v in verts] for k in range(3))
    ext = [max(c) - min(c) or 0.1 for c in (xs, ys, zs)]
    voff = 36
    blob = struct.pack(">4sIIIfffII", b"GXMS", 2, len(verts), len(idx), ext[0], ext[1], ext[2], voff, voff + len(verts)*32)
    blob += b"".join(struct.pack(">8f", *v) for v in verts) + b"".join(struct.pack(">H", i) for i in idx)
    path.write_bytes(blob)


def hsv(h, s, v):
    h6 = (h % 1.0) * 6
    i, f = int(h6), h6 - int(h6)
    p, q, t = v*(1-s), v*(1-f*s), v*(1-(1-f)*s)
    r, g, b = [(v, t, p), (q, v, p), (p, v, t), (p, q, v), (t, p, v), (v, p, q)][i % 6]
    return int(r*255), int(g*255), int(b*255)


def atlas_pixels():
    """Base colour: one hue per cell, a soft gradient and faint stripes. Glow: a bright diagonal line
    pattern, so the emissive path (added to the colour before the light) is exercised."""
    base = bytearray(ATLAS*ATLAS*4)
    glow = bytearray(ATLAS*ATLAS*4)
    for y in range(ATLAS):
        for x in range(ATLAS):
            ci = min(x // CELL_W, COLS-1) + COLS * min(y // CELL_H, ROWS-1)
            lx, ly = x % CELL_W, y % CELL_H
            hue = (ci / 12.0 + 0.03) % 1.0
            shade = 0.55 + 0.45 * (1 - ly / CELL_H)
            stripe = 0.88 if ((lx + ly) // 6) % 2 else 1.0
            r, g, b = hsv(hue, 0.78, shade * stripe)
            o = (y*ATLAS + x) * 4
            base[o:o+4] = bytes((r, g, b, 255))
            line = 150 if abs(((lx - ly) % 21) - 10) < 2 else 0
            ring = 90 if min(lx, ly, CELL_W-1-lx, CELL_H-1-ly) < 2 else 0
            gl = max(line, ring)
            glow[o:o+4] = bytes((gl, int(gl*0.9), int(gl*0.7), 255))
    return bytes(base), bytes(glow)


def write_gxtex(path, rgba):
    """GXTX v1, RGBA8, one level: 4x4 tiles, each 16 AR pairs then 16 GB pairs; row 0 on top."""
    image = bytearray()
    for ty in range(0, ATLAS, 4):
        for tx in range(0, ATLAS, 4):
            ar, gb = bytearray(), bytearray()
            for y in range(4):
                for x in range(4):
                    o = ((ty+y)*ATLAS + tx + x) * 4
                    ar += bytes((rgba[o+3], rgba[o])); gb += bytes((rgba[o+1], rgba[o+2]))
            image += ar + gb
    head = struct.pack(">4sIIIIIIIIII", b"GXTX", 1, 6, ATLAS, ATLAS, 0xFFFFFFFF, 0, len(image), 0, 64, 64 + len(image))
    path.write_bytes(head + bytes(64 - len(head)) + bytes(image))


def main():
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent / "models"
    out.mkdir(parents=True, exist_ok=True)
    base, glow = atlas_pixels()
    write_gxtex(out / "shapes.gxtex", base)
    write_gxtex(out / "shapes.glow.gxtex", glow)
    for i, (name, fn, smooth) in enumerate(SHAPES):
        verts, idx = build(i, fn(), smooth)
        write_mesh(out / f"{name}.gxmesh", verts, idx)
        (out / f"{name}.coll.json").write_text('{"version":1,"atlas":"shapes","lines":[]}\n')
        print(f"{name}: {len(idx)//3} triangles, {len(verts)} vertices")


if __name__ == "__main__":
    main()
