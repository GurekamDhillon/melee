#!/usr/bin/env python3
"""Writes the original Envoy drive models (GXMS v2) and their shared atlas (GXTX v1 RGBA8, colour + glow).

Every shape, colour and texel here is computed by this script from plain geometry and arithmetic. Nothing is
derived from a game, a model file or any other project's assets. Python standard library only.
usage: python make_drives.py [../models]
"""
import math
import random
import struct
import sys
from pathlib import Path

W = H = 128                  # atlas size
RH = 5                       # rows are RH texels high; a row is a colour ramp along u
HALF = 2.2                   # world units: every model fits a +-HALF cube (the Envoy lifts a drive by 2.6)


# ------------------------------------------------------------------------------------------ vector helpers
def sub(a, b): return (a[0]-b[0], a[1]-b[1], a[2]-b[2])
def add(a, b): return (a[0]+b[0], a[1]+b[1], a[2]+b[2])
def mul(a, s): return (a[0]*s, a[1]*s, a[2]*s)
def cross(a, b): return (a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0])
def dot(a, b): return a[0]*b[0]+a[1]*b[1]+a[2]*b[2]
def norm(a):
    n = math.sqrt(dot(a, a)) or 1.0
    return (a[0]/n, a[1]/n, a[2]/n)
def mid(pts): return tuple(sum(p[k] for p in pts)/len(pts) for k in range(3))


# ------------------------------------------------------------------------------------------ atlas rows
# Each row is a ramp along u from t=0 (dark body) to t=1 (bright, emissive). (dark, main, glow, glow power).
ROWS = {}
ROW_LIST = []
def row(name, dark, main, glow, gpow=2.0):
    ROWS[name] = len(ROW_LIST)
    ROW_LIST.append((dark, main, glow, gpow))

row("red",    (120, 20, 28),    (244, 62, 54),   (255, 64, 40))
row("red_hot", (214, 36, 40), (255, 190, 150), (255, 150, 90), 1.0)
row("green",  (8, 78, 40),    (34, 210, 108),  (60, 255, 140))
row("green_hot", (30, 200, 100), (200, 255, 215), (140, 255, 180), 1.0)
row("blue",   (20, 44, 120),    (60, 124, 255),  (80, 150, 255))
row("blue_hot", (60, 124, 255), (150, 205, 255), (140, 190, 255), 1.0)
row("yellow", (104, 76, 8),    (255, 204, 30),  (255, 214, 50))
row("yellow_hot", (250, 196, 24), (255, 245, 190), (255, 235, 120), 1.0)
row("purple", (64, 22, 104),    (182, 74, 236),  (210, 100, 255))
row("purple_hot", (182, 74, 236), (232, 170, 255), (225, 150, 255), 1.0)
row("white",  (104, 108, 124),   (232, 236, 246), (230, 235, 255))
row("white_hot", (226, 230, 240), (255, 255, 255), (255, 255, 255), 1.0)
PRISM = [(255, 80, 90), (255, 170, 60), (240, 230, 70), (70, 220, 120), (70, 190, 255), (170, 110, 255)]
for i, c in enumerate(PRISM):
    row("prism%d" % i, (40, 42, 52), c, tuple(min(255, int(x*1.05)) for x in c), 1.5)
row("magic",  (10, 40, 50),   (90, 220, 240),  (120, 240, 255), 1.0)
row("rare",   (30, 14, 56),   (170, 130, 255), (190, 150, 255), 1.0)
row("unique", (50, 34, 6),    (250, 200, 70),  (255, 215, 90), 1.0)
row("glass",  (200, 210, 225), (235, 242, 255), (0, 0, 0), 1.0)
assert len(ROW_LIST) * RH <= H


def lerp(a, b, t): return tuple(a[i] + (b[i]-a[i])*t for i in range(3))


def atlas_pixels():
    base = bytearray(W*H*4)
    glow = bytearray(W*H*4)
    for y in range(H):
        ri = y // RH
        if ri < len(ROW_LIST):
            dark, main, gl, gpow = ROW_LIST[ri]
        else:
            dark = main = gl = (0, 0, 0); gpow = 1.0
        for x in range(W):
            t = x / (W-1)
            c = lerp(dark, main, min(1.0, (t*1.15) ** 0.8))
            g = tuple(v * (t ** gpow) for v in gl)
            o = (y*W + x)*4
            base[o:o+4] = bytes((int(c[0]), int(c[1]), int(c[2]), 255))
            glow[o:o+4] = bytes((int(g[0]), int(g[1]), int(g[2]), 255))
    return bytes(base), bytes(glow)


def write_gxtex(path, rgba):
    """GXTX v1, RGBA8, one level: 4x4 tiles, each 16 AR pairs then 16 GB pairs; row 0 on top."""
    image = bytearray()
    for ty in range(0, H, 4):
        for tx in range(0, W, 4):
            ar, gb = bytearray(), bytearray()
            for y in range(4):
                for x in range(4):
                    o = ((ty+y)*W + tx + x)*4
                    ar += bytes((rgba[o+3], rgba[o])); gb += bytes((rgba[o+1], rgba[o+2]))
            image += ar + gb
    head = struct.pack(">4sIIIIIIIIII", b"GXTX", 1, 6, W, H, 0xFFFFFFFF, 0, len(image), 0, 64, 64 + len(image))
    path.write_bytes(head + bytes(64 - len(head)) + bytes(image))


# ------------------------------------------------------------------------------------------ mesh builder
class Mesh:
    def __init__(self):
        self.tris = []   # (p0, p1, p2, (t0, t1, t2), rowname)

    def tri(self, pts, ts, rw, centre):
        """One flat-shaded triangle, wound so that its normal points away from `centre`."""
        a, b, c = pts
        n = cross(sub(b, a), sub(c, a))
        if dot(n, sub(mid(pts), centre)) < 0:
            pts = (a, c, b); ts = (ts[0], ts[2], ts[1])
        self.tris.append((pts[0], pts[1], pts[2], ts, rw))

    def poly(self, pts, ts, rw, centre):
        """Fan-triangulate a convex polygon."""
        for i in range(1, len(pts)-1):
            self.tri((pts[0], pts[i], pts[i+1]), (ts[0], ts[i], ts[i+1]), rw, centre)

    def pyramid(self, base, apex, tb, ta, rw, centre=None):
        c = centre or mid(list(base) + [apex])
        for i in range(len(base)):
            self.tri((base[i], base[(i+1) % len(base)], apex), (tb, tb, ta), rw, c)

    def prism(self, poly2d, z0, z1, front, rows, ts):
        """Convex 2D polygon extruded from z0 (back) to z1 (front); the front cap is shrunk to `front`
        about the centroid, so the side walls are a bevel. rows=(side, cap), ts=(back, bevel rim, cap)."""
        cx = sum(p[0] for p in poly2d)/len(poly2d); cy = sum(p[1] for p in poly2d)/len(poly2d)
        bk = [(p[0], p[1], z0) for p in poly2d]
        ft = [(cx + (p[0]-cx)*front, cy + (p[1]-cy)*front, z1) for p in poly2d]
        ctr = (cx, cy, (z0+z1)/2)
        side_row, cap_row = rows
        t_back, t_rim, t_cap = ts
        n = len(poly2d)
        for i in range(n):
            j = (i+1) % n
            self.tri((bk[i], bk[j], ft[j]), (t_back, t_back, t_rim), side_row, ctr)
            self.tri((bk[i], ft[j], ft[i]), (t_back, t_rim, t_rim), side_row, ctr)
        self.poly(ft, [t_cap]*n, cap_row, ctr)
        self.poly(bk, [t_back]*n, side_row, ctr)

    def transformed(self, fn):
        m = Mesh()
        for p0, p1, p2, ts, rw in self.tris:
            m.tris.append((fn(p0), fn(p1), fn(p2), ts, rw))
        return m

    def extend(self, other):
        self.tris += other.tris


def pad_bounds(m):
    """Zero-area triangles at +-HALF so a mesh shares the bodies' bounds (the screen model auto-fits on bounds)."""
    m.tris.append(((HALF, HALF, 0), (HALF, HALF, 0), (HALF, HALF, 0), (0, 0, 0), "glass"))
    m.tris.append(((-HALF, -HALF, 0), (-HALF, -HALF, 0), (-HALF, -HALF, 0), (0, 0, 0), "glass"))
    return m


def normalise(mesh):
    """Centre on the bounding box, scale so the larger of (radius about Y, half height) is HALF."""
    pts = [p for t in mesh.tris for p in t[:3]]
    lo = [min(p[k] for p in pts) for k in range(3)]; hi = [max(p[k] for p in pts) for k in range(3)]
    c = [(lo[k]+hi[k])/2 for k in range(3)]
    rad = max(math.hypot(p[0]-c[0], p[2]-c[2]) for p in pts)
    hy = (hi[1]-lo[1])/2
    s = HALF / max(rad, hy)
    return mesh.transformed(lambda p: ((p[0]-c[0])*s, (p[1]-c[1])*s, (p[2]-c[2])*s))


# ------------------------------------------------------------------------------------------ the six families
def red_jack():
    """DAMAGE: a six-spoked spike jack (a caltrop): sharp points on every axis."""
    m = Mesh()
    b = 0.34
    axes = [((1, 0, 0), (0, 1, 0), (0, 0, 1)), ((-1, 0, 0), (0, 1, 0), (0, 0, 1)),
            ((0, 1, 0), (1, 0, 0), (0, 0, 1)), ((0, -1, 0), (1, 0, 0), (0, 0, 1)),
            ((0, 0, 1), (1, 0, 0), (0, 1, 0)), ((0, 0, -1), (1, 0, 0), (0, 1, 0))]
    for d, u, v in axes:
        cen = mul(d, b)
        base = [add(cen, add(mul(u, sx*b), mul(v, sy*b))) for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
        m.pyramid(base, mul(d, 1.55), 0.95, 0.05, "red", centre=(0, 0, 0))
    return m


def green_chevrons():
    """SPEED: two forward chevrons, a streamlined fast-forward arrow, glowing bevels."""
    m = Mesh()
    k, hh = 0.52, 1.0
    for xr, xt in ((-1.05, -0.02), (0.08, 1.11)):
        A, B, C, D, E, F = ((xr, hh), (xr+k, hh), (xt, 0), (xr+k, -hh), (xr, -hh), (xt-k, 0))
        for arm in ([A, B, C, F], [F, C, D, E]):
            m.prism(arm, -0.4, 0.4, 0.62, ("green", "green_hot"), (0.0, 1.0, 0.25))
    return m


def blue_shield():
    """DEFENCE: a faceted heater shield with a raised boss: layered, glowing rim."""
    m = Mesh()
    rim = [(-0.92, 0.82), (0.0, 1.0), (0.92, 0.82), (0.92, 0.05), (0.5, -0.62), (0.0, -1.0), (-0.5, -0.62), (-0.92, 0.05)]
    ctr = (0, 0, 0.1)
    mid_ring = [(x*0.72, y*0.72 - 0.02) for x, y in rim]
    inner = [(x*0.38, y*0.38 - 0.02) for x, y in rim]
    n = len(rim)
    R = [(x, y, 0.0) for x, y in rim]
    M2 = [(x, y, 0.28) for x, y in mid_ring]
    I = [(x, y, 0.62) for x, y in inner]
    back = [(x, y, -0.22) for x, y in rim]
    for i in range(n):
        j = (i+1) % n
        m.tri((back[i], back[j], R[j]), (0, 0, 0.9), "blue", ctr); m.tri((back[i], R[j], R[i]), (0, 0.9, 0.9), "blue", ctr)
        m.tri((R[i], R[j], M2[j]), (0.9, 0.9, 0.2), "blue", ctr); m.tri((R[i], M2[j], M2[i]), (0.9, 0.2, 0.2), "blue", ctr)
        m.tri((M2[i], M2[j], I[j]), (0.2, 0.2, 0.8), "blue_hot", ctr); m.tri((M2[i], I[j], I[i]), (0.2, 0.8, 0.8), "blue_hot", ctr)
    m.poly(I, [0.5]*n, "blue_hot", ctr)
    m.poly(back, [0.0]*n, "blue", ctr)
    return m


def yellow_wings():
    """AIR / JUMP: two swept wings of three feathers climbing from a small core, tips up and out."""
    m = Mesh()
    root = (0.0, -0.95)
    for side in (1, -1):
        for ang, L, w in ((22, 1.0, 0.23), (50, 1.15, 0.23), (76, 1.3, 0.22)):
            a = math.radians(ang)
            d = (side*math.cos(a), math.sin(a)); p = (-d[1], d[0])
            P = [root, (root[0]+d[0]*0.38*L+p[0]*w, root[1]+d[1]*0.38*L+p[1]*w),
                 (root[0]+d[0]*L, root[1]+d[1]*L),
                 (root[0]+d[0]*0.38*L-p[0]*w, root[1]+d[1]*0.38*L-p[1]*w)]
            m.prism(P, -0.12, 0.12, 0.55, ("yellow", "yellow_hot"), (0.0, 1.0, 0.3))
    c = (0, -0.95, 0); r = 0.36
    top, bot = (c[0], c[1]+r*1.3, c[2]), (c[0], c[1]-r*1.3, c[2])
    eq = [(c[0]+r, c[1], c[2]), (c[0], c[1], c[2]+r), (c[0]-r, c[1], c[2]), (c[0], c[1], c[2]-r)]
    for i in range(4):
        m.tri((top, eq[i], eq[(i+1) % 4]), (0.7, 1.0, 1.0), "yellow_hot", c)
        m.tri((bot, eq[i], eq[(i+1) % 4]), (0.5, 1.0, 1.0), "yellow_hot", c)
    return m


def icosa_pts():
    p = (1 + math.sqrt(5))/2
    v = [(-1, p, 0), (1, p, 0), (-1, -p, 0), (1, -p, 0), (0, -1, p), (0, 1, p), (0, -1, -p), (0, 1, -p),
         (p, 0, -1), (p, 0, 1), (-p, 0, -1), (-p, 0, 1)]
    v = [norm(x) for x in v]
    f = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11), (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6),
         (7, 1, 8), (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9), (4, 9, 5), (2, 4, 11), (6, 2, 10),
         (8, 6, 7), (9, 8, 1)]
    return v, f


def purple_spores():
    """STATUS: a spore cluster: a faceted core orb ringed by bulbs of different sizes; round, bubbly."""
    m = Mesh()
    v, f = icosa_pts()
    cc = (0, 0.05, 0)
    for a, b, c in f:
        m.tri(tuple(add(cc, mul(v[i], 0.82)) for i in (a, b, c)), (0.5, 0.5, 0.5), "purple", cc)
    bulbs = [((0.95, 0.55, 0.2), 0.34), ((-0.9, 0.62, -0.15), 0.3), ((0.15, -0.98, 0.3), 0.36),
             ((-0.75, -0.62, 0.1), 0.27), ((0.88, -0.5, -0.2), 0.26), ((0.0, 1.12, -0.1), 0.28)]
    faces = [(0, 2, 4), (4, 2, 1), (1, 2, 5), (5, 2, 0), (0, 4, 3), (4, 1, 3), (1, 5, 3), (5, 0, 3)]
    for pos, r in bulbs:
        octa = [add(pos, o) for o in ((r, 0, 0), (-r, 0, 0), (0, r, 0), (0, -r, 0), (0, 0, r), (0, 0, -r))]
        for a, b, c in faces:
            m.tri((octa[a], octa[b], octa[c]), (0.6, 0.6, 0.6), "purple_hot", pos)
    return m


def white_prism(seed=7):
    """WILD: one lopsided, jittered gem whose every face is a different spectral colour."""
    rnd = random.Random(seed)
    v, f = icosa_pts()
    vv = []
    for p in v:
        r = 0.7 + 0.55*rnd.random()
        vv.append((p[0]*r, p[1]*r, p[2]*r*0.8))
    m = Mesh()
    for k, (a, b, c) in enumerate(f):
        m.tri((vv[a], vv[b], vv[c]), (0.25, 0.95, 0.6), "prism%d" % (k % 6), (0, 0, 0))
    for d, h in (((0.75, 0.55, 0.25), 0.55), ((-0.6, -0.8, 0.0), 0.5)):   # two shards break the symmetry
        n = norm(d)
        base_c = mul(n, 0.85)
        u = norm(cross(n, (0, 0, 1))); w = cross(n, u)
        base = [add(base_c, add(mul(u, 0.2*sx), mul(w, 0.2*sy))) for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
        m.pyramid(base, add(base_c, mul(n, h + 0.45)), 0.9, 0.2, "white_hot", centre=(0, 0, 0))
    return m


def glass_shell():
    """The translucent shell the Envoy draws over every floor drive (alpha, tint per colour)."""
    m = Mesh()
    v, f = icosa_pts()
    for a, b, c in f:
        m.tri(tuple(mul(v[i], HALF*1.12) for i in (a, b, c)), (0.5, 0.5, 0.5), "glass", (0, 0, 0))
    return m


# ------------------------------------------------------------------------------------------ rarity overlays
def ring_overlay(radius, thick, segs, rw, tilt=0.0, y=0.0):
    """A thin triangular-section ring about the vertical axis; `tilt` rolls it about Z."""
    m = Mesh()
    sec = [(0.0, thick), (thick*0.9, -thick*0.5), (-thick*0.9, -thick*0.5)]   # (dr, dy) of the cross-section
    for i in range(segs):
        a0, a1 = 2*math.pi*i/segs, 2*math.pi*(i+1)/segs
        def P(a, s):
            r = radius + sec[s][0]
            return (r*math.cos(a), sec[s][1], r*math.sin(a))
        ctr = ((radius*(math.cos(a0)+math.cos(a1))/2), 0, (radius*(math.sin(a0)+math.sin(a1))/2))
        for s in range(3):
            t = (s+1) % 3
            m.tri((P(a0, s), P(a1, s), P(a1, t)), (0.6, 0.6, 0.6), rw, ctr)
            m.tri((P(a0, s), P(a1, t), P(a0, t)), (0.6, 0.6, 0.6), rw, ctr)
    def xf(p):
        q = rot_z(p, tilt) if tilt else p
        return (q[0], q[1] + y, q[2])
    return m.transformed(xf)


def rot_z(p, a):
    c, s = math.cos(a), math.sin(a)
    return (p[0]*c - p[1]*s, p[0]*s + p[1]*c, p[2])


def crown(radius, height, n, y):
    m = Mesh()
    for i in range(n):
        a = 2*math.pi*(i+0.5)/n
        c = (radius*math.cos(a), y, radius*math.sin(a))
        t = (-math.sin(a), 0, math.cos(a)); rr = (math.cos(a), 0, math.sin(a))
        w = 0.17
        base = [add(c, add(mul(t, sx*w), mul(rr, sy*w))) for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
        m.pyramid(base, (c[0], y + height, c[2]), 0.6, 0.85, "unique", centre=(c[0], y + height/3, c[2]))
    return m


def overlays():
    magic = ring_overlay(0.93, 0.1, 12, "magic")
    rare = Mesh()
    rare.extend(ring_overlay(0.9, 0.1, 12, "rare", y=0.55))
    rare.extend(ring_overlay(0.9, 0.1, 12, "rare", y=-0.55))
    uniq = Mesh()
    uniq.extend(ring_overlay(0.93, 0.11, 12, "unique"))
    uniq.extend(crown(0.93, 0.55, 8, 0.0))
    return {"drive_ring_magic": magic, "drive_ring_rare": rare, "drive_ring_unique": uniq}


# ------------------------------------------------------------------------------------------ output
def vertices(mesh):
    verts, idx = [], []
    for p0, p1, p2, ts, rw in mesh.tris:
        cr = cross(sub(p1, p0), sub(p2, p0))
        n = norm(cr) if dot(cr, cr) > 1e-12 else (0, 1, 0)
        for p, t in zip((p0, p1, p2), ts):
            u = (3 + t*(W-6)) / W
            v = (ROWS[rw]*RH + RH/2) / H
            idx.append(len(verts))
            verts.append((p[0], p[1], p[2], u, v, n[0], n[1], n[2]))
    return verts, idx


def write_mesh(path, verts, idx):
    xs, ys, zs = ([v[k] for v in verts] for k in range(3))
    ext = [max(c) - min(c) or 0.1 for c in (xs, ys, zs)]
    voff = 36
    blob = struct.pack(">4sIIIfffII", b"GXMS", 2, len(verts), len(idx), ext[0], ext[1], ext[2], voff, voff + len(verts)*32)
    blob += b"".join(struct.pack(">8f", *v) for v in verts) + b"".join(struct.pack(">H", i) for i in idx)
    path.write_bytes(blob)
    return ext


def build_all():
    bodies = {"drive_red": red_jack(), "drive_green": green_chevrons(), "drive_blue": blue_shield(),
              "drive_yellow": yellow_wings(), "drive_purple": purple_spores(), "drive_white": white_prism()}
    models = {k: (normalise(m), 0) for k, m in bodies.items()}
    models["drive_glass"] = (glass_shell(), 1)
    for k, m in overlays().items():
        models[k] = (pad_bounds(m.transformed(lambda p: mul(p, HALF))), 0)
    return models


def main():
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent / "models"
    out.mkdir(parents=True, exist_ok=True)
    base, glow = atlas_pixels()
    write_gxtex(out / "drive_atlas.gxtex", base)
    write_gxtex(out / "drive_atlas.glow.gxtex", glow)
    for name, (mesh, alpha) in build_all().items():
        verts, idx = vertices(mesh)
        ext = write_mesh(out / (name + ".gxmesh"), verts, idx)
        (out / (name + ".coll.json")).write_text('{"version":1,"atlas":"drive_atlas","alpha":%d,"lines":[]}\n' % alpha)
        mat = ('{"builtin": "glass", "opacity": 0.28, "roughness": 0.4, "tint": [1, 1, 1, 1]}\n' if name == "drive_glass"
               else '{"builtin": "lit", "emission": 1.0}\n')
        (out / (name + ".material.json")).write_text(mat)
        print("%-18s %3d triangles %4d vertices  extent %.2f x %.2f x %.2f" % (name, len(idx)//3, len(verts), *ext))
    write_party(out)


# ------------------------------------------------------------------------------------------ the Party pack set (step 9)
# One mesh per drive (46) and per Unique (16) from the shared part lists in drive_parts.py (the same lists draw the
# sticker glyphs), the Rare / Epic tier rings and the ascended halo + star overlay (DRIVE-MODELS.md 2, 3).
# Atlas decision: ONE taller atlas (128 x 256, rows 4 texels high) for the whole new set, not a separate Uniques atlas:
# 12 family + 9 treatment + 32 Unique rows = 53 rows fit in one image, so the drawer keeps a single colour lookup for
# the set (no per-model atlas switch) and the legacy 128 x 128 atlas stays byte-identical for the old six bodies.
# No hull(): the ink outline is an effects-layer pass (port/fx: MAT_OUTLINE 0x40 on a drive body), not geometry.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import drive_parts as DP  # noqa: E402

PW, PH, PRH = 128, 256, 4
PARTY_ROWS = list(DP.ROWS)          # row order = atlas order
assert len(PARTY_ROWS) * PRH <= PH, len(PARTY_ROWS)
OVERLAYS = ["ring_rare2", "ring_epic", "ascended"]
# body triangles per model, counted in DRIVE-MODELS.md 5 and 6 (the self-test holds the meshes to these)
SPEC_TRIS = {'ember': 72, 'bolt': 16, 'nova': 76, 'wildfire': 108, 'spire': 20, 'kindle': 48, 'cinder_heart': 100,
    'solar_flare': 72, 'frost': 80, 'shatter': 40, 'hex': 56, 'static': 72, 'affliction': 80, 'contagion': 116,
    'steam': 88, 'feather_storm': 56, 'gatekeeper': 92, 'comet': 52, 'updraft': 48, 'skyhook': 76, 'meteor': 44,
    'thunderhead': 112, 'rush': 120, 'tailwind': 104, 'clean_landing': 108, 'perfect_form': 140, 'slipstream': 80,
    'slick': 108, 'jet_trail': 118, 'quicksilver': 140, 'riposte': 44, 'insulate': 108, 'bulwark': 48, 'thorns': 56,
    'leech': 36, 'breaker': 32, 'last_stand': 72, 'colossus': 36, 'echo': 36, 'metronome': 36, 'relay': 168,
    'surge': 24, 'crescendo': 48, 'snowball': 40, 'magpie': 88, 'mirror': 44, 'puppeteer': 120, 'ashbank': 196,
    'changing_of_the_guard': 64, 'turnabout': 80, 'mockingbird': 136, 'hundred_year_seed': 56, 'lifeline_coin': 128,
    'kiln': 80, 'absolution': 136, 'clockwork_heel': 184, 'hair_trigger': 72, 'shared_flame': 204, 'pied_piper': 92,
    'fools_gold': 72, 'albatross': 72, 'ouroboros': 122,
    'ring_rare2': 84, 'ring_epic': 112, 'ascended': 92}   # epic: the doc says 84 + a 16-triangle spark; the sheet's spark prism is 28


def party_atlas():
    """Base: the sheet's ramp (dark -> main, plus the glow tint t^2 * .18). Glow (what the drawer adds times its glow
    parameter): hot rows t^1.2, treatment rows t^1.5, body rows a faint t^3 (the bevel rim)."""
    base = bytearray(PW*PH*4); glow = bytearray(PW*PH*4)
    for y in range(PH):
        ri = y // PRH
        name = PARTY_ROWS[ri] if ri < len(PARTY_ROWS) else None
        for x in range(PW):
            t = x / (PW-1)
            o = (y*PW + x)*4
            if name is None:
                base[o:o+4] = bytes((0, 0, 0, 255)); glow[o:o+4] = bytes((0, 0, 0, 255)); continue
            c = DP.ramp(name, t)
            g = DP.ROWS[name][2]
            if name.endswith("_hot"):
                k = (t ** 1.2) * .8
            elif name in ("white", "sun") or name.startswith("foil"):
                k = (t ** 1.5) * .6
            else:
                k = (t ** 3) * .35
            base[o:o+4] = bytes((int(c[0]), int(c[1]), int(c[2]), 255))
            glow[o:o+4] = bytes((int(g[0]*k), int(g[1]*k), int(g[2]*k), 255))
    return bytes(base), bytes(glow)


def write_gxtex_wh(path, rgba, w, h):
    image = bytearray()
    for ty in range(0, h, 4):
        for tx in range(0, w, 4):
            ar, gb = bytearray(), bytearray()
            for y in range(4):
                for x in range(4):
                    o = ((ty+y)*w + tx + x)*4
                    ar += bytes((rgba[o+3], rgba[o])); gb += bytes((rgba[o+1], rgba[o+2]))
            image += ar + gb
    head = struct.pack(">4sIIIIIIIIII", b"GXTX", 1, 6, w, h, 0xFFFFFFFF, 0, len(image), 0, 64, 64 + len(image))
    path.write_bytes(head + bytes(64 - len(head)) + bytes(image))


def overlay_mesh(name):
    if name == "ring_rare2":
        return DP.tier_mesh(DP.Mesh(), 1)
    if name == "ring_epic":
        return DP.tier_mesh(DP.Mesh(), 2)
    # ascended: the 12-segment flat gold ring and the two-sided ten-point star over the form (asc_mesh minus the body)
    m = DP.Mesh()
    m.ring_y(.52, .075, 12, "sun", 1.0, y=1.34)
    st = [(0, 1), (.3, .32), (1.02, .3), (.46, -.16), (.64, -.88), (0, -.48), (-.64, -.88), (-.46, -.16), (-1.02, .3), (-.3, .32)]
    s = DP.Mesh()
    for i in range(10):
        for zz in (.1, -.1):
            s.tri(((0, 0, zz), (st[i][0]*.38, st[i][1]*.38, 0), (st[(i+1) % 10][0]*.38, st[(i+1) % 10][1]*.38, 0)), 1.0, "sun", (0, 0, 0))
    return m.extend(s.scaled(1, (0, 1.46, 0)))


def party_models():
    """key -> drive_parts Mesh in the parts box (-1..1). Written scaled by HALF with NO recentring, so every body and
    every overlay share one frame (the tier ring lies under the form at y -1.14, the halo over it at 1.34)."""
    out = {d["key"]: DP.dmesh(d) for d in DP.DRIVES}
    out.update({u["key"]: DP.umesh(u) for u in DP.UNIQUES})
    out.update({k: overlay_mesh(k) for k in OVERLAYS})
    return out


def party_vertices(mesh):
    verts, idx = [], []
    for p0, p1, p2, t, rw in mesh.tris:
        p0, p1, p2 = mul(p0, HALF), mul(p1, HALF), mul(p2, HALF)
        cr = cross(sub(p1, p0), sub(p2, p0))
        n = norm(cr) if dot(cr, cr) > 1e-12 else (0, 1, 0)
        u = (3 + t*(PW-6)) / PW
        v = (PARTY_ROWS.index(rw)*PRH + PRH/2) / PH
        for p in (p0, p1, p2):
            idx.append(len(verts))
            verts.append((p[0], p[1], p[2], u, v, n[0], n[1], n[2]))
    return verts, idx


def party_order():
    """Model order after the ten legacy models: the three overlays, the 46 drives, the 16 Uniques."""
    return OVERLAYS + [d["key"] for d in DP.DRIVES] + [u["key"] for u in DP.UNIQUES]


def self_test(models):
    """Triangle counts against the spec, bounds, ramp range, atlas rows, outward winding; exits on any miss."""
    bad = []
    if set(SPEC_TRIS) != set(models):
        bad.append("model set differs from the spec: %s" % sorted(set(SPEC_TRIS) ^ set(models)))
    for k, m in models.items():
        if SPEC_TRIS.get(k) != len(m.tris):
            bad.append("%s: %d triangles, spec %s" % (k, len(m.tris), SPEC_TRIS.get(k)))
        for p0, p1, p2, t, rw in m.tris:
            if rw not in DP.ROWS:
                bad.append("%s: row %s" % (k, rw)); break
            if not all(abs(c) <= 1.9 for p in (p0, p1, p2) for c in p):
                bad.append("%s: outside the 1.9 box" % k); break
            if not 0 <= t <= 1.0001:
                bad.append("%s: ramp %.2f" % (k, t)); break
        if k != "ascended":   # the star is two-sided flat, zero volume
            vol = sum(dot(p0, cross(p1, p2)) for p0, p1, p2, _, _ in m.tris) / 6
            if vol <= 0:
                bad.append("%s: inward winding (volume %.3f)" % (k, vol))
    if bad:
        raise SystemExit("self-test FAILED:\n  " + "\n  ".join(bad))
    return sum(len(m.tris) for m in models.values())


def write_party(out):
    base, glow = party_atlas()
    write_gxtex_wh(out / "drive_party_atlas.gxtex", base, PW, PH)
    write_gxtex_wh(out / "drive_party_atlas.glow.gxtex", glow, PW, PH)
    models = party_models()
    total = self_test(models)
    order = party_order()
    for key in order:
        verts, idx = party_vertices(models[key])
        write_mesh(out / ("drive_%s.gxmesh" % key), verts, idx)
        (out / ("drive_%s.coll.json" % key)).write_text('{"version":1,"atlas":"drive_party_atlas","alpha":0,"lines":[]}\n')
        (out / ("drive_%s.material.json" % key)).write_text('{"builtin": "lit", "emission": 1.0}\n')
    # the manifest the game is held to (drive_draw.rs's include list and envoy_ui's key lookup): one key per line,
    # model id = 10 + line number
    (out / "drive_party.txt").write_text("".join(k + "\n" for k in order))
    print("party set: %d models (%d drives, %d Uniques, %d overlays), %d triangles, atlas %dx%d, %d rows; self-test ok"
          % (len(order), len(DP.DRIVES), len(DP.UNIQUES), len(OVERLAYS), total, PW, PH, len(PARTY_ROWS)))


if __name__ == "__main__":
    main()
