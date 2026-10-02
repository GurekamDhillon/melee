"""VK_Platform — my own take on a parametric platform-fighter platform.

Written after reading the BF kit (bf_platform_build.py / bf_interior_playset.py /
export_kit.py); shares no code with it. Same *rules* learned from it, different
structure and a different palette.

  rules borrowed
    - every feature has a fixed real-world size; resizing only lengthens the straight runs
    - a feature_scale degrades detail gracefully instead of inverting geometry
    - two UV layers: UVMap (true metres) and UVBake (fitted 0-1)
    - a verifier that asserts the invariants at several sizes

  structure taken differently
    theirs: a solid faceted hull under a slab
    mine:   an OPEN SPACE-FRAME TRUSS under the slab, so the underside reads as
            architecture -- perimeter chords, posts, diagonal bracing, a suspended core

  colour taken differently — and this is the important one
    Every surface samples a tileable GREYSCALE mask (Non-Color) from vk_textures/.
    Hue and value come from a palette dict, so the whole kit recolours by editing
    one JSON (or one colour socket in the shader) without re-baking anything.
    Masks carry: plate/seam layout, rivets, brushed streaks, AO, glow falloff.

    textures/vk_palette.json   the swappable colours
    textures/vk_masks.json     what each mask is, tiling, strength

Run inside Blender:
    exec(open('/tmp/opencode/vk_platform.py').read())
"""

import bpy
import bmesh
import json
import math
import os
from mathutils import Vector

import numpy as np

try:
    HERE = os.path.dirname(os.path.abspath(__file__))   # runpy / blender --python
except NameError:                                        # exec(open(...).read()) has no __file__
    HERE = '/tmp/opencode'
TEXDIR = os.path.join(HERE, 'vk_textures')
OBJ_NAME = "VK_Platform"
UNIT = 6.5
MASK = 512

# ================================================================ palette (swappable)
PALETTE = {
    "deck":   {"colour": [0.660, 0.628, 0.552], "mask": "vk_panel", "uv": 0.34,
               "rough": [0.52, 0.86], "metal": 0.05, "grain": 0.10},
    "trim":   {"colour": [0.520, 0.268, 0.112], "mask": "vk_brushed", "uv": 1.10,
               "rough": [0.22, 0.48], "metal": 0.92, "grain": 0.06},
    "patina": {"colour": [0.128, 0.268, 0.252], "mask": "vk_hull", "uv": 0.55,
               "rough": [0.38, 0.72], "metal": 0.35, "grain": 0.12},
    "hull":   {"colour": [0.072, 0.086, 0.104], "mask": "vk_hull", "uv": 0.40,
               "rough": [0.55, 0.92], "metal": 0.55, "grain": 0.14},
    "glow":   {"colour": [1.000, 0.560, 0.150], "mask": "vk_glow", "uv": 1.60,
               "rough": [0.30, 0.30], "metal": 0.0, "grain": 0.0, "emit": 6.5},
    "core":   {"colour": [1.000, 0.760, 0.420], "mask": "vk_glow", "uv": 2.40,
               "rough": [0.30, 0.30], "metal": 0.0, "grain": 0.0, "emit": 13.0},
}

# ================================================================ fixed feature sizes (metres)
CORNER_CUT = 0.50
BEVEL      = 0.035
RIM_H      = 0.145
LEDGE      = 0.105
HULL_INSET = 0.58
FRAME_W    = 0.185
FRAME_H    = 0.052
DECK_T     = 0.055
LIGHT_SP   = 0.62
LIGHT_SIZE = (0.105, 0.030, 0.048)

TRUSS_H    = 0.58
TRUSS_IN   = 0.30
CHORD_R    = 0.058
BRACE_R    = 0.030
BRACE_SP   = 0.95
CORE_R     = 0.32


# ================================================================ 1. greyscale mask generation
def _vnoise(n, freq, seed, octaves=1, aniso=1.0):
    """Tileable value noise: integer spatial frequencies, so it wraps exactly."""
    rng = np.random.default_rng(seed)
    yy, xx = np.mgrid[0:n, 0:n] / float(n)
    acc = np.zeros((n, n)); amp = 1.0; tot = 0.0; f = freq
    for _ in range(octaves):
        ph = rng.random(2) * (2 * np.pi)
        w = 2 * np.pi * f
        acc += amp * np.sin(w * xx + ph[0]) * np.sin(w * yy * aniso + ph[1])
        tot += amp; amp *= 0.5; f *= 2
    return acc / tot * 0.5 + 0.5


def _plates(n, cells, levels, seed):
    """Crisp plate values with hard seam lines on an exact lattice (not noise-blobs)."""
    rng = np.random.default_rng(seed)
    step = max(4, n // cells)
    yy, xx = np.mgrid[0:n, 0:n]
    ix = np.minimum((xx // step), cells)
    iy = np.minimum((yy // step), cells)
    table = 0.50 + 0.40 * rng.random((cells + 2, cells + 2))
    val = table[ix, iy]
    dx = np.minimum(xx % step, step - (xx % step))
    dy = np.minimum(yy % step, step - (yy % step))
    d = np.minimum(dx, dy)
    seam = np.clip(d / 2.0, 0.0, 1.0)
    return np.clip(val, 0, 1), seam


def _rivets(n, cells, radius, every=2, seed=0):
    """Dots on a lattice; returns 0..1 coverage."""
    g = np.zeros((n, n))
    step = n // cells
    if step < 3:
        return g
    r = max(1, int(radius * step))
    for iy in range(0, cells, every):
        for ix in range(0, cells, every):
            cy = int((iy + 0.5) * step); cx = int((ix + 0.5) * step)
            for dy in range(-r, r + 1):
                for dx in range(-r, r + 1):
                    if dx * dx + dy * dy > r * r:
                        continue
                    g[(cy + dy) % n, (cx + dx) % n] = 1.0
    return g


def build_masks():
    os.makedirs(TEXDIR, exist_ok=True)
    n = MASK
    out = {}

    # deck: plate seams + rivets + fine grain
    pl, seam = _plates(n, 6, 5, 11)
    rv = _rivets(n, 6, 0.10, every=2, seed=3)
    grain = _vnoise(n, 64, 21, octaves=3)
    m = pl * 0.70 + 0.30 * grain
    m = np.where(seam < 0.5, m * 0.42, m)
    m = np.clip(m + rv * 0.30, 0, 1)
    out["vk_panel"] = m

    # brushed metal: streaks along X, tight in Y
    streak = _vnoise(n, 2, 31, octaves=5, aniso=64.0)
    fine = _vnoise(n, 128, 41, octaves=2)
    out["vk_brushed"] = np.clip(0.42 + 0.40 * streak + 0.18 * fine, 0, 1)

    # hull: large plates, rivets, broad AO blotches
    pl, seam = _plates(n, 3, 4, 51)
    rv = _rivets(n, 3, 0.055, every=1, seed=7)
    ao = _vnoise(n, 2, 61, octaves=3)
    m = 0.34 + 0.44 * pl + 0.22 * ao
    m = np.where(seam < 0.5, m * 0.50, m)
    out["vk_hull"] = np.clip(m + rv * 0.22, 0, 1)

    # glow: near-uniform brightness with flicker. Masks are sampled through UVMap (world
    # metres), so a *spatially banded* mask would land small emissive props on random
    # parts of the band and come out dark. Keep it flat-bright; put the shape in geometry.
    flick = 0.84 + 0.16 * _vnoise(n, 6, 71, octaves=4)
    drift = 0.94 + 0.06 * _vnoise(n, 2, 83, octaves=2)
    out["vk_glow"] = np.clip(flick * drift, 0, 1)

    written = {}
    for name, arr in out.items():
        path = os.path.join(TEXDIR, name + ".png")
        img = bpy.data.images.get(name)
        if img:
            bpy.data.images.remove(img)
        img = bpy.data.images.new(name, n, n, alpha=False, float_buffer=False)
        img.colorspace_settings.name = 'Non-Color'
        rgba = np.empty((n, n, 4), dtype=np.float32)
        rgba[..., 0] = arr; rgba[..., 1] = arr; rgba[..., 2] = arr; rgba[..., 3] = 1.0
        img.pixels.foreach_set(rgba[::-1].ravel())      # Blender rows are bottom-up
        img.filepath_raw = path
        img.file_format = 'PNG'
        img.save()
        written[name] = path
    return written


# ================================================================ 2. shader helpers
def nd(t, idname, x, y, **kw):
    n = t.nodes.new(idname)
    n.location = (x, y)
    for k, v in kw.items():
        setattr(n, k, v)
    return n


def put(t, node, key, val):
    if hasattr(val, 'node'):
        return t.links.new(val, node.inputs[key])
    node.inputs[key].default_value = val
    return None


def mathn(t, op, x, y, a=None, b=None):
    n = nd(t, 'ShaderNodeMath', x, y, operation=op)
    if a is not None: put(t, n, 0, a)
    if b is not None: n.inputs[1].default_value = b
    return n


def cmix(t, fac, a, b, x=0, y=0, label=""):
    """ShaderNodeMix RGBA: in 0=Factor, 6=A, 7=B; out 2=Result."""
    n = nd(t, 'ShaderNodeMix', x, y, data_type='RGBA', label=label, clamp_factor=True)
    put(t, n, 0, fac)
    for idx, val in ((6, a), (7, b)):
        if hasattr(val, 'node'):
            t.links.new(val, n.inputs[idx])
        else:
            v = tuple(val)
            n.inputs[idx].default_value = v if len(v) == 4 else v + (1.0,)
    return n.outputs[2]


def attr_vec(t, name, x=-1500, y=0):
    a = nd(t, 'ShaderNodeAttribute', x, y, attribute_type='GEOMETRY', attribute_name=name)
    sp = nd(t, 'ShaderNodeSeparateXYZ', x + 180, y)
    t.links.new(a.outputs['Vector'], sp.inputs['Vector'])
    return sp.outputs['X'], sp.outputs['Y'], sp.outputs['Z']


def seam(t, p, spacing, half_w, x=0, y=0):
    d = mathn(t, 'DIVIDE', x, y, b=spacing); put(t, d, 0, p)
    fr = mathn(t, 'FRACT', x + 160, y); t.links.new(d.outputs[0], fr.inputs[0])
    iv = mathn(t, 'SUBTRACT', x + 160, y - 140, a=1.0)
    t.links.new(fr.outputs[0], iv.inputs[1])
    mn = mathn(t, 'MINIMUM', x + 320, y)
    t.links.new(fr.outputs[0], mn.inputs[0]); t.links.new(iv.outputs[0], mn.inputs[1])
    sc = mathn(t, 'MULTIPLY', x + 480, y, b=spacing); t.links.new(mn.outputs[0], sc.inputs[0])
    lt = mathn(t, 'LESS_THAN', x + 640, y, b=half_w); t.links.new(sc.outputs[0], lt.inputs[0])
    return lt.outputs[0]


def band(t, p, centre, half_w, x=0, y=0):
    s = mathn(t, 'SUBTRACT', x, y, b=centre); put(t, s, 0, p)
    ab = mathn(t, 'ABSOLUTE', x + 160, y); t.links.new(s.outputs[0], ab.inputs[0])
    lt = mathn(t, 'LESS_THAN', x + 320, y, b=half_w); t.links.new(ab.outputs[0], lt.inputs[0])
    return lt.outputs[0]


def build_materials(palette):
    """Every material = a greyscale mask x a palette colour. Recolour = edit the dict."""
    mats = {}
    for slot, spec in palette.items():
        name = "VK_" + slot.capitalize()
        m = bpy.data.materials.get(name)
        if m is None:
            m = bpy.data.materials.new(name)
        m.use_nodes = True
        t = m.node_tree
        t.nodes.clear()
        out = nd(t, 'ShaderNodeOutputMaterial', 900, 0)
        bsdf = nd(t, 'ShaderNodeBsdfPrincipled', 640, 0)
        t.links.new(bsdf.outputs['BSDF'], out.inputs['Surface'])

        img = bpy.data.images.get(spec["mask"])
        if img is None:
            img = bpy.data.images.load(os.path.join(TEXDIR, spec["mask"] + ".png"))
            img.name = spec["mask"]
        tex = nd(t, 'ShaderNodeTexImage', -200, 200, interpolation='Linear')
        tex.image = img
        tex.label = "GREY MASK (%s) - recolour, never re-bake" % spec["mask"]

        # world-metre UV -> tiled mask, so texel density is real-world constant
        ux = nd(t, 'ShaderNodeUVMap', -1100, 200); ux.uv_map = 'UVMap'
        mp = nd(t, 'ShaderNodeMapping', -880, 200)
        mp.inputs['Scale'].default_value = (spec["uv"], spec["uv"], spec["uv"])
        t.links.new(ux.outputs['UV'], mp.inputs['Vector'])
        t.links.new(mp.outputs['Vector'], tex.inputs['Vector'])
        mask = tex.outputs['Color']

        col = spec['colour']
        base = cmix(t, mask,
                    [c * 0.42 for c in col], [min(1.0, c * 1.22) for c in col],
                    300, 260, "Mask x palette colour")
        t.links.new(base, bsdf.inputs['Base Color'])
        t.links.new(cmix(t, mask, (spec['rough'][0],) * 3, (spec['rough'][1],) * 3, 300, -60),
                    bsdf.inputs['Roughness'])
        bsdf.inputs['Metallic'].default_value = spec['metal']
        if spec.get('emit'):
            t.links.new(base, bsdf.inputs['Emission Color'])
            st = mathn(t, 'MULTIPLY', 300, -320, b=spec['emit'])
            t.links.new(mask, st.inputs[0])
            t.links.new(st.outputs[0], bsdf.inputs['Emission Strength'])
        m.diffuse_color = (*col, 1.0)
        mats[slot] = m
    return mats


# ================================================================ 3. geometry core
def offset_poly(poly, s):
    """True parallel inset of a convex CCW polygon by s metres."""
    n = len(poly)
    if abs(s) < 1e-9:
        return [(float(a), float(b)) for a, b in poly]
    lines = []
    for i in range(n):
        p = Vector(poly[i]); q = Vector(poly[(i + 1) % n])
        d = q - p
        if d.length < 1e-12:
            d = Vector((1.0, 0.0))
        d.normalize()
        lines.append((p, d, Vector((-d.y, d.x))))
    out = []
    for i in range(n):
        p0, d0, n0 = lines[(i - 1) % n]
        p1, d1, n1 = lines[i]
        bx = (p1.x - p0.x) + (n1.x - n0.x) * s
        by = (p1.y - p0.y) + (n1.y - n0.y) * s
        a11, a12, a21, a22 = d0.x, -d1.x, d0.y, -d1.y
        det = a11 * a22 - a12 * a21
        if abs(det) < 1e-9:
            out.append((p1.x + n1.x * s, p1.y + n1.y * s)); continue
        tt = (bx * a22 - a12 * by) / det
        v = p0 + d0 * tt + n0 * s
        out.append((v.x, v.y))
    return out


def cut_corner_rect(w, d, cc):
    hx, hy = w * 0.5, d * 0.5
    cc = min(cc, hx * 0.85, hy * 0.85)
    return [(-hx + cc, -hy), (hx - cc, -hy), (hx, -hy + cc), (hx, hy - cc),
            (hx - cc, hy), (-hx + cc, hy), (-hx, hy - cc), (-hx, -hy + cc)]


class Geo:
    def __init__(self, name):
        self.name = name
        self.bm = bmesh.new()
        self.pos = []
        self.brace = []
        self.mats = []

    def slot(self, n):
        if n not in self.mats:
            self.mats.append(n)
        return self.mats.index(n)

    def v(self, co, brace=0.0):
        v = self.bm.verts.new((float(co[0]), float(co[1]), float(co[2])))
        self.pos.append((float(co[0]), float(co[1]), float(co[2])))
        self.brace.append(float(brace))
        return v

    def f(self, verts, mat, brace=0.0):
        try:
            fc = self.bm.faces.new(verts)
        except ValueError:
            return None
        fc.material_index = self.slot(mat)
        return fc

    def _basis_box(self, c, ax, ay, az, h, mat, brace=0.0):
        vs, idx, ctr = {}, {}, 0
        for sz in (-1, 1):
            for sy in (-1, 1):
                for sx in (-1, 1):
                    vs[ctr] = self.v(Vector(c) + ax * (sx * h[0]) + ay * (sy * h[1]) + az * (sz * h[2]), brace)
                    idx[ctr] = (sx, sy, sz)
                    ctr += 1
        for q in ((0, 2, 3, 1), (4, 5, 7, 6), (0, 1, 5, 4), (1, 3, 7, 5), (2, 6, 7, 3), (0, 4, 6, 2)):
            self.f([vs[i] for i in q], mat, brace)

    def box(self, lo, hi, mat, brace=0.0):
        x0, y0, z0 = lo; x1, y1, z1 = hi
        self._basis_box(((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2),
                        Vector((1, 0, 0)), Vector((0, 1, 0)), Vector((0, 0, 1)),
                        ((x1 - x0) / 2, (y1 - y0) / 2, (z1 - z0) / 2), mat, brace)

    def lathe(self, outline, profile, mats, cap_top_mat=None, cap_bot_mat=None, closed=False):
        """Revolve a (parallel_inset, z) profile around a convex outline.

        mats: one material name, or one per band. With closed=True the profile
        is a closed section: the last ring joins back to the first and no caps are
        made, so the result is a manifold shell (a ring has no n-gon cap).
        """
        rings = []
        for (ins, z) in profile:
            rings.append([self.v((px, py, z)) for (px, py) in offset_poly(outline, ins)])
        n = len(outline)
        bmats = list(mats) if isinstance(mats, (list, tuple)) else [mats] * (len(profile) - 1)
        if len(bmats) < len(profile) - 1:
            bmats += [bmats[-1]] * (len(profile) - 1 - len(bmats))
        if not closed and cap_top_mat:
            self.f(list(rings[0]), cap_top_mat)
        pairs = ([(k, (k + 1) % len(rings)) for k in range(len(rings))] if closed
                 else [(k, k + 1) for k in range(len(rings) - 1)])
        for k, (ia, ib) in enumerate(pairs):
            a, b = rings[ia], rings[ib]
            m = bmats[k % len(bmats)]
            for i in range(n):
                j = (i + 1) % n
                self.f((a[i], b[i], b[j], a[j]), m)
        if not closed and cap_bot_mat:
            self.f(list(reversed(rings[-1])), cap_bot_mat)
        return rings

    def strut(self, p0, p1, r0, r1, sides, mat, brace=0.0):
        a = Vector(p0); b = Vector(p1)
        d = b - a
        if d.length < 1e-7:
            return
        z = d.normalized()
        up = Vector((0, 0, 1))
        if abs(z.dot(up)) > 0.95:
            up = Vector((1, 0, 0))
        x = z.cross(up).normalized(); y = z.cross(x)
        rings = []
        for (p, r) in ((a, r0), (b, r1)):
            rings.append([self.v(p + x * (math.cos(i / sides * math.tau) * r)
                                    + y * (math.sin(i / sides * math.tau) * r), brace)
                          for i in range(sides)])
        self.f(list(reversed(rings[0])), mat, brace)
        self.f(list(rings[1]), mat, brace)
        for i in range(sides):
            j = (i + 1) % sides
            self.f((rings[0][i], rings[1][i], rings[1][j], rings[0][j]), mat, brace)

    def dashes(self, p0, p1, spacing, size, mat, brace=0.0):
        """Evenly spaced boxes along a run; count follows length, spacing stays near target.
        `size` is the full box size; the run is filled from its midpoint outwards."""
        a = Vector(p0); b = Vector(p1)
        run = b - a; L = run.length
        if L < 1e-6:
            return 0
        n = max(1, int(round(L / spacing)))
        dirn = run / L
        side = dirn.cross(Vector((0, 0, 1)))
        if side.length < 1e-6:
            side = Vector((0, 1, 0))
        side.normalize()
        up = dirn.cross(side)
        mid = (a + b) * 0.5
        h = (size[0] * 0.5, size[1] * 0.5, size[2] * 0.5)
        for i in range(n):
            c = mid + dirn * ((i - (n - 1) / 2.0) * L / n)
            self._basis_box(c, dirn, side, up, h, mat, brace)
        return n


def feature_scale(w, d, th):
    """Shrink detail on platforms too small to carry it, instead of inverting geometry."""
    return max(0.16, min(1.0, min(w, d) / 1.7, th / 0.20))


# ================================================================ 4. the platform
def build_platform(name, w, d, th, mats):
    fs = feature_scale(w, d, th)
    s = lambda v: v * fs
    outline = cut_corner_rect(w, d, CORNER_CUT)
    g = Geo(name)
    info = {}

    # ---- slab: one closed shell, one material per band, deck panel as the top cap
    z_rim0 = -s(BEVEL)
    z_rim1 = z_rim0 - s(RIM_H)
    z_led = z_rim1 - s(LEDGE)
    z_hull = -th
    g.lathe(outline,
            [(0.0, 0.0),                        # 0 deck panel (cap)
             (0.0, z_rim0),                     # 1
             (0.0, z_rim1),                     # 2  lit rim band
             (s(BEVEL), z_led),                 # 3  chamfer + ledge
             (s(HULL_INSET), z_hull)],          # 4  hull slope
            ['trim', 'trim', 'patina', 'hull'],
            cap_top_mat='deck', cap_bot_mat='hull')
    info['slab_profile'] = [(0.0, 0.0), (0.0, z_rim0), (0.0, z_rim1),
                            (s(BEVEL), z_led), (s(HULL_INSET), z_hull)]

    # ---- raised frame around the deck: a CLOSED section, so the inlay slope (the
    #      glow), the outer wall and the hidden underside form one manifold ring.
    g.lathe(outline,
            [(s(FRAME_W), 0.0), (0.0, s(FRAME_H)), (0.0, 0.0)],
            ['glow', 'trim', 'trim'], closed=True)

    # ---- rim lights: count follows edge length, spacing stays near target.
    # The bar's inner face sits ON the rim plane, so the lit part is fully proud of it.
    cc = min(CORNER_CUT, w * 0.42, d * 0.42)
    hx, hy = w / 2, d / 2
    zl = (z_rim0 + z_rim1) / 2
    proud = s(LIGHT_SIZE[1]) * 0.5
    light_runs = [
        ((-hx + cc, -hy - proud, zl), (hx - cc, -hy - proud, zl)),
        ((-hx + cc, hy + proud, zl), (hx - cc, hy + proud, zl)),
        ((-hx - proud, -hy + cc, zl), (-hx - proud, hy - cc, zl)),
        ((hx + proud, -hy + cc, zl), (hx + proud, hy - cc, zl)),
    ]
    total_lights = 0
    for (p0, p1) in light_runs:
        total_lights += g.dashes(p0, p1, s(LIGHT_SP), (s(LIGHT_SIZE[0]), s(LIGHT_SIZE[1]), s(LIGHT_SIZE[2])), 'glow')
    info['rim_lights'] = total_lights

    # ---- truss: chords, posts, diagonals, suspended core
    tx0, tx1 = -hx + s(HULL_INSET), hx - s(HULL_INSET)
    ty0, ty1 = -hy + s(HULL_INSET), hy - s(HULL_INSET)
    bx0, bx1 = tx0 + s(TRUSS_IN), tx1 - s(TRUSS_IN)
    by0, by1 = ty0 + s(TRUSS_IN), ty1 - s(TRUSS_IN)
    zt = z_hull
    zb = z_hull - s(TRUSS_H)
    cr, br = s(CHORD_R), s(BRACE_R)
    have_truss = (tx1 - tx0) > 4 * cr and (ty1 - ty0) > 4 * cr and s(TRUSS_H) > 4 * br
    bays = 0
    if have_truss:
        top = [(tx0, ty0, zt), (tx1, ty0, zt), (tx1, ty1, zt), (tx0, ty1, zt)]
        bot = [(bx0, by0, zb), (bx1, by0, zb), (bx1, by1, zb), (bx0, by1, zb)]
        for i in range(4):
            g.strut(top[i], top[(i + 1) % 4], cr, cr, 6, 'trim', brace=1.0)
            g.strut(bot[i], bot[(i + 1) % 4], cr * 0.85, cr * 0.85, 6, 'trim', brace=1.0)
        # bracing bays on the two long runs
        for (y_t, y_b, xa, xb) in ((ty0, by0, tx0, tx1), (ty1, by1, tx0, tx1)):
            run = xb - xa
            nb = max(1, int(round(run / s(BRACE_SP))))
            step = run / nb
            for i in range(nb + 1):
                x = xa + i * step
                g.strut((x, y_t, zt), (x, y_b, zb), br, br, 5, 'hull', brace=1.0)
                if i < nb:
                    xn = xa + (i + 1) * step
                    g.strut((x, y_t, zt), (xn, y_b, zb), br, br, 5, 'hull', brace=1.0)
                    bays += 1
        # end frames
        for (xt, xb_) in ((tx0, bx0), (tx1, bx1)):
            for (yt, yb_) in ((ty0, by0), (ty1, by1)):
                g.strut((xt, yt, zt), (xb_, yb_, zb), br, br, 5, 'hull', brace=1.0)
        # suspended glowing core
        g.strut((0, 0, zt), (0, 0, zb - s(CORE_R) * 0.55), br * 0.9, br * 0.9, 5, 'trim', brace=1.0)
        for i in range(6):
            a = i / 6 * math.tau
            g.strut((math.cos(a) * s(CORE_R) * 0.75, math.sin(a) * s(CORE_R) * 0.75, zb - s(CORE_R) * 0.55),
                    (0, 0, zb - s(CORE_R) * 1.5), br * 0.7, br * 0.35, 4, 'core', brace=1.0)
    info['truss'] = have_truss
    info['bays'] = bays
    info['feature_scale'] = fs

    z_lo = zb - s(CORE_R) * 1.6 if have_truss else z_hull
    g.mats_order = list(g.mats)
    me = g.finish(g.mats_order, z_lo, 0.0, (w, d, th))
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob, info


def finish(self, mat_order, z_lo, z_hi, dims):
    bmesh.ops.recalc_face_normals(self.bm, faces=list(self.bm.faces))
    # capture each face's material NAME before to_mesh: clearing the slot list on the
    # datablock resets every polygon's index, so the names have to be reapplied after.
    face_names = [self.mats[f.material_index] if f.material_index < len(self.mats) else self.mats[0]
                  for f in self.bm.faces]
    me = bpy.data.meshes.new(self.name)
    self.bm.to_mesh(me)
    self.bm.free()
    me.materials.clear()
    slot_of = {}
    for i, slot in enumerate(mat_order):
        slot_of[slot] = i
        me.materials.append(bpy.data.materials["VK_" + slot.capitalize()])
    for p, nm in zip(me.polygons, face_names):
        p.material_index = slot_of.get(nm, 0)
    a_pos = me.attributes.new(name='vk_pos', type='FLOAT_VECTOR', domain='POINT')
    a_dim = me.attributes.new(name='vk_dim', type='FLOAT_VECTOR', domain='POINT')
    a_lvl = me.attributes.new(name='vk_level', type='FLOAT', domain='POINT')
    a_edg = me.attributes.new(name='vk_edge', type='FLOAT', domain='POINT')
    a_brc = me.attributes.new(name='vk_brace', type='FLOAT', domain='POINT')
    span = max(1e-6, z_hi - z_lo)
    hx, hy = dims[0] * 0.5, dims[1] * 0.5
    for i, v in enumerate(me.vertices):
        co = v.co
        a_pos.data[i].vector = (co.x, co.y, co.z)
        a_dim.data[i].vector = (dims[0], dims[1], dims[2])
        a_lvl.data[i].value = min(1.0, max(0.0, (co.z - z_lo) / span))
        e = min(1.0, (hx - abs(co.x)) / max(1e-6, hx), (hy - abs(co.y)) / max(1e-6, hy))
        a_edg.data[i].value = 1.0 - max(0.0, e)
        a_brc.data[i].value = self.brace[i] if i < len(self.brace) else 0.0
    uv = me.uv_layers.new(name='UVMap')
    bk = me.uv_layers.new(name='UVBake')
    pairs, us, vs = [], [], []
    for p in me.polygons:
        ax = max(range(3), key=lambda i: abs(p.normal[i]))
        rest = [i for i in range(3) if i != ax]
        for li in p.loop_indices:
            co = me.vertices[me.loops[li].vertex_index].co
            uv.data[li].uv = (co[rest[0]], co[rest[1]])
            pairs.append((li, co[rest[0]], co[rest[1]]))
            us.append(co[rest[0]]); vs.append(co[rest[1]])
    # UVBake: the same layout, uniformly scaled AND centred so it always fits 0-1
    mu, mv = min(us), min(vs)
    su = (max(us) - mu) or 1.0
    sv = (max(vs) - mv) or 1.0
    sc = min(1.0 / su, 1.0 / sv)
    ou = (1.0 - su * sc) * 0.5
    ov = (1.0 - sv * sc) * 0.5
    for (li, u, v) in pairs:
        bk.data[li].uv = ((u - mu) * sc + ou, (v - mv) * sc + ov)
    if hasattr(me, "shade_flat"):
        me.shade_flat()
    return me


Geo.finish = finish


# ================================================================ 5. verifier
def verify(name):
    ob = bpy.data.objects[name]
    me = ob.data
    bm = bmesh.new(); bm.from_mesh(me)
    issues = []
    small = [e for e in bm.edges if e.calc_length() < 1e-6]
    degen = [f for f in bm.faces if f.calc_area() < 1e-9]
    nonman = [e for e in bm.edges if not e.is_manifold]
    loose = [v for v in bm.verts if not v.link_faces]
    empty_slots = [i for i, m in enumerate(me.materials) if m is None]
    unused = sorted({p.material_index for p in me.polygons} -
                    {i for i in range(len(me.materials))})
    if small: issues.append(f"{len(small)} zero-length edges")
    if degen: issues.append(f"{len(degen)} zero-area faces")
    if nonman: issues.append(f"{len(nonman)} non-manifold edges")
    if loose: issues.append(f"{len(loose)} loose verts")
    if empty_slots: issues.append(f"empty material slots {empty_slots}")
    if unused: issues.append(f"unused material slots {unused}")
    if 'UVMap' not in me.uv_layers: issues.append("no UVMap")
    if 'UVBake' not in me.uv_layers: issues.append("no UVBake")
    for a in ('vk_pos', 'vk_dim', 'vk_level', 'vk_edge', 'vk_brace'):
        if a not in me.attributes: issues.append(f"missing attribute {a}")
    ov = [d.uv[0] for d in me.uv_layers['UVBake'].data] + [d.uv[1] for d in me.uv_layers['UVBake'].data]
    if ov and (min(ov) < -1e-4 or max(ov) > 1.0001):
        issues.append(f"UVBake outside 0-1 ({min(ov):.3f}..{max(ov):.3f})")
    bm.free()
    return {"issues": issues,
            "verts": len(me.vertices), "faces": len(me.polygons),
            "tris": sum(max(0, len(p.vertices) - 2) for p in me.polygons),
            "materials": [m.name for m in me.materials]}


# ================================================================ 6. run
def main():
    masks = build_masks()
    with open(os.path.join(TEXDIR, 'vk_masks.json'), 'w') as fh:
        json.dump({k: {"file": os.path.basename(v), "channels": "luminance",
                       "colorspace": "Non-Color", "tileable": True}
                   for k, v in masks.items()}, fh, indent=2)
    with open(os.path.join(TEXDIR, 'vk_palette.json'), 'w') as fh:
        json.dump(PALETTE, fh, indent=2)

    mats = build_materials(PALETTE)

    old = bpy.data.objects.get(OBJ_NAME)
    if old:
        d = old.data
        bpy.data.objects.remove(old, do_unlink=True)
        if d.users == 0:
            bpy.data.meshes.remove(d)

    ob, info = build_platform(OBJ_NAME, 8.0, 2.4, 0.40, mats)
    rep = verify(OBJ_NAME)

    # resize sweep: feature sizes must not move, bay count must scale
    sweep = []
    for (w, d, t) in ((8.0, 2.4, 0.40), (20.0, 2.4, 0.40), (40.0, 2.4, 0.40), (1.0, 0.5, 0.10)):
        tmp, nfo = build_platform("VK__sweep", w, d, t, mats)
        me = tmp.data
        zs = [v.co.z for v in me.vertices]
        sweep.append({"w": w, "d": d, "t": t,
                      "feature_scale": round(nfo['feature_scale'], 3),
                      "rim_lights": nfo['rim_lights'], "bays": nfo['bays'],
                      "z_span": round(max(zs) - min(zs), 4),
                      "faces": len(me.polygons),
                      "issues": verify("VK__sweep")["issues"]})
        dm = me
        bpy.data.objects.remove(tmp, do_unlink=True)
        bpy.data.meshes.remove(dm)

    print(json.dumps({"main": rep, "info": {k: v for k, v in info.items() if k != 'slab_profile'},
                      "sweep": sweep}, indent=2, default=str))
    return rep, sweep, masks


RESULT = main()