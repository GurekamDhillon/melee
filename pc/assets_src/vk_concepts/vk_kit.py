"""VK_Kit — my own take on the BF interior module kit.

Same conventions I read out of bf_interior_playset.py / export_kit.py, different parts:
  - 2 m grid, Blender X = travel, Y = depth, front = -Y, Z = height
  - every part is its own collection, marked as an asset, carrying module_size /
    purpose / grid_m so it is self-describing
  - collision is a list of lines, exported as a sidecar in their v1 shape
    {"version":1,"atlas":"vk_kit","lines":[[kind,x0,z0,x1,z1,flags]]}
    flags = 1 passthrough + 2 ledges  (their encoding)
  - a placement table equivalent to their Lua ROOM table, emitted as JSON
  - colour comes from the same greymask pipeline as the platform: never baked

It imports the platform generator the same way their playset imports theirs
(runpy on the stage-kit script), so the kit and the stage share one language.

Run inside Blender:
    exec(open('/tmp/opencode/vk_kit.py').read())
"""

import bpy
import bmesh
import json
import math
import os
import runpy
from mathutils import Vector

try:
    HERE = os.path.dirname(os.path.abspath(__file__))
except NameError:
    HERE = '/tmp/opencode'
OUT = os.path.join(HERE, 'vk_kit_out')
os.makedirs(OUT, exist_ok=True)

# the stage kit supplies the geometry helpers, the greymask pipeline and the base palette
BF = runpy.run_path(os.path.join(HERE, 'vk_platform.py'))
Geo = BF['Geo']
offset_poly = BF['offset_poly']
cut_corner_rect = BF['cut_corner_rect']
build_materials = BF['build_materials']
BASE_PALETTE = dict(BF['PALETTE'])
UNIT = BF['UNIT']              # game units per kit metre
TAU = math.tau

# ---------------------------------------------------------------- the kit's palette
# same greymasks, extra slots. Everything stays recolourable from one JSON.
KIT_PALETTE = dict(BASE_PALETTE)
KIT_PALETTE.update({
    "wall":   {"colour": [0.105, 0.118, 0.140], "mask": "vk_hull", "uv": 0.45,
               "rough": [0.55, 0.92], "metal": 0.15},
    "panel":  {"colour": [0.175, 0.190, 0.215], "mask": "vk_panel", "uv": 0.75,
               "rough": [0.38, 0.66], "metal": 0.35},
    "accent": {"colour": [0.220, 0.780, 0.720], "mask": "vk_glow", "uv": 2.0,
               "rough": [0.30, 0.30], "metal": 0.0, "emit": 4.2},
    "glass":  {"colour": [0.240, 0.480, 0.560], "mask": "vk_glow", "uv": 0.9,
               "rough": [0.05, 0.12], "metal": 0.0},
    "vista":  {"colour": [0.480, 0.300, 0.180], "mask": "vk_glow", "uv": 0.5,
               "rough": [0.90, 0.90], "metal": 0.0, "emit": 1.5},
})
MATS = build_materials(KIT_PALETTE)


# ---------------------------------------------------------------- Geo additions
def _prism_x(self, x0, x1, yz, mat):
    """Extrude a (y,z) cross-section along X. yz must wind CCW seen from +X."""
    r0 = [self.v((x0, y, z)) for (y, z) in yz]
    r1 = [self.v((x1, y, z)) for (y, z) in yz]
    n = len(yz)
    self.f(list(reversed(r0)), mat)
    self.f(list(r1), mat)
    for i in range(n):
        j = (i + 1) % n
        self.f((r0[i], r0[j], r1[j], r1[i]), mat)
Geo.prism_x = _prism_x


def _wedge(self, name, x0, x1, z_at_x0, z_at_x1, y0, y1, mat, thick):
    """A solid ramp: top face slopes from z_at_x0 to z_at_x1, `thick` deep."""
    self.prism_x(x0, x1, [(y0, z_at_x0 - thick), (y1, z_at_x0 - thick),
                          (y1, z_at_x1), (y0, z_at_x1)], mat)
Geo.wedge = _wedge


def warp(self, shear=0.0, lift=0.0):
    """Ramp shear, applied after the detail so bevelled detail keeps its size."""
    if shear or lift:
        for v in self.bm.verts:
            v.co.z += shear * v.co.x + lift
Geo.warp = warp


def to_object(g, name, coll, size):
    g.mats_order = list(g.mats)
    me = g.finish(g.mats_order, min(v[2] for v in g.pos), max(v[2] for v in g.pos), size)
    ob = bpy.data.objects.new(name, me)
    coll.objects.link(ob)
    return ob


# ---------------------------------------------------------------- modules
COL = None
MODULES = {}
COLLISION = {}
PLACEMENT = []


def module(name, purpose, size, grid=2.0):
    """One part = one collection = one model with its own collision."""
    global COL
    old = bpy.data.collections.get(name)
    if old:
        for ob in list(old.objects):
            d = ob.data
            bpy.data.objects.remove(ob, do_unlink=True)
            if isinstance(d, bpy.types.Mesh) and d.users == 0:
                bpy.data.meshes.remove(d)
        bpy.data.collections.remove(old)
    COL = bpy.data.collections.new(name)
    COL.use_fake_user = True
    try:
        COL.asset_mark()
        COL.asset_data.description = purpose + '  Grid: 2 m; front: -Y; travel: X; up: Z.'
    except Exception:
        pass
    COL['module_size'] = list(size)
    COL['purpose'] = purpose
    COL['grid_m'] = grid
    MODULES[name] = COL
    return COL


def collide(*lines):
    COLLISION[COL.name] = list(lines)


def line(x0, z0, x1, z1, passthrough=True, ledges=True, kind="floor"):
    return (kind, x0, z0, x1, z1, (1 if passthrough else 0) + (2 if ledges else 0))


# ---------------------------------------------------------------- floors
# Cross-section shared by the floor and the ramp: a walkable deck, a copper lip,
# a patina rim band with amber dashes, then two hull facets stepping back underneath.
FASCIA = [(-1.18, -0.05), (1.20, -0.05), (1.20, -0.30), (-0.74, -0.30),
          (-1.02, -0.225), (-1.18, -0.10)]
LOWER_FACET = ((-1.02, -0.225), (-0.74, -0.30))


def fascia(g, width, shear=0.0, lift=0.0, dy=0.0):
    """The shared floor build. Chamfered box detail, sheared last."""
    kw = dict(shear=shear, lift=lift)
    g.prism_x(-width / 2, width / 2, [(y + dy, z) for (y, z) in FASCIA], 'hull')
    g.box((-width / 2, -1.16 + dy, -0.075), (width / 2, 1.18 + dy, -0.025), 'deck')      # walkable top
    g.box((-width / 2, -1.19 + dy, -0.085), (width / 2, -1.10 + dy, -0.010), 'trim')    # copper lip
    g.box((-width / 2, -1.20 + dy, -0.20), (width / 2, -1.155 + dy, -0.10), 'patina')   # rim band
    g.box((-width / 2, 1.14, -0.03), (width / 2, 1.20, 0.0), 'trim')                   # rear socket
    g.dashes((-width / 2 + 0.12, -1.212 + dy, -0.15), (width / 2 - 0.12, -1.212 + dy, -0.15),
             0.55, (0.11, 0.024, 0.042), 'glow')
    (y0, z0), (y1, z1) = LOWER_FACET
    ny, nz = z1 - z0, -(y1 - y0)
    L = math.hypot(ny, nz)
    ny, nz = -ny / L * 0.004, -nz / L * 0.004
    ym, zm = (y0 + y1) / 2 + dy, (z0 + z1) / 2
    ty, tz = (y1 - y0) * 0.06, (z1 - z0) * 0.06
    vs = [g.v(p) for p in [(-width / 2, ym - ty + ny, zm - tz + nz), (width / 2, ym - ty + ny, zm - tz + nz),
                           (width / 2, ym + ty + ny, zm + tz + nz), (-width / 2, ym + ty + ny, zm + tz + nz)]]
    g.f(vs, 'core')
    g.warp(**kw)


def build_floors():
    module('VK_Floor_4m', 'Straight load-bearing floor; top surface is Z=0.', (4, 2.4, 0.4))
    g = Geo("VK_Floor_4m"); fascia(g, 4.0); to_object(g, "VK_Floor_4m", COL, (4, 2.4, 0.4))
    collide(line(-2, 0, 2, 0))

    module('VK_Floor_2m', 'Short landing or infill floor; top surface is Z=0.', (2, 2.4, 0.4))
    g = Geo("VK_Floor_2m"); fascia(g, 2.0); to_object(g, "VK_Floor_2m", COL, (2, 2.4, 0.4))
    collide(line(-1, 0, 1, 0))

    # a landing with a drop-through gap: two 1 m halves, no lip to catch a runner
    module('VK_Floor_Opening_4m', 'Floor bay with a 2 m drop-through gap; same ends and top.', (4, 2.4, 0.4))
    g = Geo("VK_Floor_Opening_4m")
    fascia(g, 1.0)
    for p in g.pos:
        pass
    for v in g.bm.verts:
        v.co.x += 1.5
    fascia(g, 1.0)
    for v in g.bm.verts:
        v.co.x += 1.5
    # cut faces around the gap
    for sx in (-1, 1):
        g.prism_x(sx * 1.0 - 0.03, sx * 1.0 + 0.03, [(y - 0.012, z) for (y, z) in FASCIA], 'hull')
        g.box((sx * 1.0 - 0.04, -1.19, -0.085), (sx * 1.0 + 0.04, -1.10, -0.010), 'trim')
        g.box((sx * 0.96, -0.05, -0.028), (sx * 0.99, 1.15, -0.022), 'accent')
    to_object(g, "VK_Floor_Opening_4m", COL, (4, 2.4, 0.4))
    collide(line(-2, 0, -1, 0, ledges=False), line(1, 0, 2, 0, ledges=False))


def build_ramp_and_stairs():
    module('VK_Ramp_4m_Rise2m', 'Full-thickness traversable ramp with a rear rail. '
           'Start X=-2 Z=0; end X=2 Z=2.', (4, 2.4, 2.0))
    g = Geo("VK_Ramp_4m_Rise2m")
    fascia(g, 4.0, shear=0.5, lift=1.0, dy=0.03)   # set back so it tucks behind a floor's fascia
    for x in (-1.5, -0.5, 0.5, 1.5):
        g.box((x - 0.035, 1.06, 0.0), (x + 0.035, 1.14, 1.10), 'trim')
    g.box((-2.0, 1.04, 1.06), (2.0, 1.16, 1.12), 'trim')
    g.box((-2.0, 1.00, 1.005), (2.0, 1.02, 1.025), 'glow')
    g.box((-2.0, 1.05, 0.01), (2.0, 1.07, 0.91), 'glass')
    g.warp(shear=0.5, lift=1.0)
    to_object(g, "VK_Ramp_4m_Rise2m", COL, (4, 2.4, 2.0))
    collide(line(-2, 0, 2, 2, ledges=False))

    module('VK_Stairs_4m_Rise2m', 'Eight 0.5 m steps, 0.5 m tread, 0.25 m rise. '
           'Start X=-2 Z=0; end X=2 Z=2.', (4, 2.4, 2.0))
    g = Geo("VK_Stairs_4m_Rise2m")
    for i in range(8):
        x0 = -2.0 + i * 0.5
        top = (i + 1) * 0.25
        g.box((x0, -1.15, top - 0.25), (x0 + 0.5, 1.15, top), 'deck')            # tread block
        g.box((x0 + 0.44, -1.19, top - 0.24), (x0 + 0.50, -1.10, top), 'trim')   # nosing
        g.box((x0 + 0.455, -1.205, top - 0.20), (x0 + 0.485, -1.16, top - 0.09), 'glow')
    g.prism_x(-2.0, 2.0, [(-1.18, -0.30), (1.20, -0.30), (1.20, -0.05), (-1.18, -0.05)], 'hull')
    for x in (-1.9, 1.9):                                                          # side stringers
        g.box((x - 0.05, -1.20, -0.32), (x + 0.05, 1.20, 2.02), 'patina')
    to_object(g, "VK_Stairs_4m_Rise2m", COL, (4, 2.4, 2.0))
    collide(line(-2, 0, 2, 2, ledges=False))


def build_balcony():
    module('VK_Balcony_4m', 'Cantilevered deck: walkway, glass rail, and brackets.', (4, 2.4, 0.4))
    g = Geo("VK_Balcony_4m")
    fascia(g, 4.0)
    for x in (-1.6, 0.0, 1.6):
        g.strut((x, 1.15, -0.28), (x, 1.95, -1.05), 0.06, 0.045, 4, 'trim')      # bracket
    for x in (-1.92, 1.92):
        g.box((x - 0.04, 1.03, -0.02), (x + 0.04, 1.17, 1.08), 'trim')
    g.box((-2.0, 1.01, 1.04), (2.0, 1.19, 1.10), 'trim')
    g.box((-2.0, 0.995, 1.005), (2.0, 1.015, 1.025), 'glow')
    g.box((-1.90, 1.06, 0.02), (1.90, 1.08, 1.00), 'glass')
    to_object(g, "VK_Balcony_4m", COL, (4, 2.4, 0.4))
    collide(line(-2, 0, 2, 0))


# ---------------------------------------------------------------- walls
WALL_Y, FRONT = 1.40, 1.31


def wall_bands(g, width=4.0, base_gaps=(), dado_gaps=()):
    def runs(gaps):
        edges = [-width / 2] + [v for gp in gaps for v in gp] + [width / 2]
        return [(edges[i], edges[i + 1]) for i in range(0, len(edges), 2)]
    for a, b in runs(base_gaps):
        g.box((a, 1.27, 0.0), (b, 1.34, 0.22), 'trim')
        g.box((a, 1.24, 0.035), (b, 1.255, 0.055), 'accent')
    for a, b in runs(dado_gaps):
        g.box((a, 1.28, 1.09), (b, 1.33, 1.15), 'trim')
    g.box((-width / 2, 1.28, 3.54), (width / 2, 1.32, 3.78), 'panel')
    g.box((-width / 2, 1.265, 3.63), (width / 2, 1.288, 3.652), 'accent')
    g.box((-width / 2, 1.27, 3.78), (width / 2, 1.36, 3.98), 'trim')
    for x in (-width / 2 + 0.09, width / 2 - 0.09):     # half pilaster per bay edge
        g.box((x - 0.09, 1.27, 0.22), (x + 0.09, 1.33, 3.54), 'panel')
        g.box((x - 0.055, 1.255, 0.30), (x - 0.035, 1.272, 3.48), 'accent')


def panels(g, x0, x1, rows, cols=None, y=1.285, mat='panel'):
    span = x1 - x0
    n = cols or max(1, int(round(span / 1.8)))
    w = (span - 0.06 * (n - 1)) / n
    for i in range(n):
        cx = x0 + w / 2 + i * (w + 0.06)
        for (z0, z1) in rows:
            g.box((cx - w / 2, y, z0), (cx + w / 2, y + 0.045, z1), mat)


def build_walls():
    module('VK_Wall_Solid_4m', 'Back wall bay, behind the fighter travel plane.', (4, 0.18, 4))
    g = Geo("VK_Wall_Solid_4m")
    g.box((-2.0, WALL_Y - 0.09, 0.0), (2.0, WALL_Y + 0.09, 4.0), 'wall')
    wall_bands(g)
    panels(g, -1.79, 1.79, ((0.30, 1.04), (1.20, 3.46)))
    to_object(g, "VK_Wall_Solid_4m", COL, (4, 0.18, 4))
    collide()

    module('VK_Wall_Doorway_4m', 'Back-wall portal with a real 1.6 x 2.6 m opening.', (4, 0.18, 4))
    g = Geo("VK_Wall_Doorway_4m")
    for x in (-1.4, 1.4):
        g.box((x - 0.6, WALL_Y - 0.09, 0.0), (x + 0.6, WALL_Y + 0.09, 4.0), 'wall')
    g.box((-0.8, WALL_Y - 0.09, 2.6), (0.8, WALL_Y + 0.09, 4.0), 'wall')
    wall_bands(g, base_gaps=((-1.0, 1.0),), dado_gaps=((-1.0, 1.0),))
    for sx in (-1, 1):
        a, b = sorted((sx * 1.79, sx * 1.06))
        panels(g, a, b, ((0.30, 1.04), (1.20, 3.46)))
    panels(g, -0.9, 0.9, ((2.98, 3.46),))
    for sx in (-1, 1):                                   # architrave, proud of the wall
        g.box((sx * 0.89 - 0.09, 1.24, 0.0), (sx * 0.89 + 0.09, 1.34, 2.78), 'trim')
        g.box((sx * 0.812 - 0.011, 1.215, 0.05), (sx * 0.812 + 0.011, 1.245, 2.55), 'glow')
    g.box((-0.98, 1.24, 2.60), (0.98, 1.34, 2.78), 'trim')
    g.box((-0.80, 1.215, 2.622), (0.80, 1.245, 2.644), 'glow')
    g.box((-0.25, 1.22, 2.84), (0.25, 1.26, 2.90), 'core')
    g.box((-0.80, 1.30, 0.0), (0.80, 1.50, 0.036), 'trim')                 # threshold
    DEPTH = 0.95                                           # the reveal has depth
    cy = FRONT + DEPTH / 2
    for sx in (-1, 1):
        g.box((sx * 0.785 - 0.015, cy, 0.0), (sx * 0.785 + 0.015, FRONT + DEPTH, 2.6), 'panel')
        g.box((sx * 0.765 - 0.008, cy + 0.15, 0.02), (sx * 0.765 + 0.008, FRONT + DEPTH - 0.15, 0.04), 'glow')
    g.box((-0.8, cy, 2.585), (0.8, FRONT + DEPTH, 2.615), 'panel')
    g.box((-0.8, cy, -0.02), (0.8, FRONT + DEPTH, 0.02), 'deck')
    g.box((-0.45, cy + 0.10, 2.545), (0.45, cy + 0.18, 2.565), 'glow')
    g.box((-0.8, FRONT + DEPTH, 0.0), (0.8, FRONT + DEPTH + 0.03, 2.6), 'vista')   # lit corridor end
    for sx in (-1, 1):
        g.box((sx * 0.45 - 0.03, FRONT + DEPTH - 0.03, 0.0), (sx * 0.45 + 0.03, FRONT + DEPTH, 2.6), 'hull')
    to_object(g, "VK_Wall_Doorway_4m", COL, (4, 0.18, 4))
    collide()

    module('VK_Wall_Window_4m', 'Back-wall viewing opening onto a lit vista card.', (4, 0.18, 4))
    g = Geo("VK_Wall_Window_4m")
    for x in (-1.75, 1.75):
        g.box((x - 0.25, WALL_Y - 0.09, 0.0), (x + 0.25, WALL_Y + 0.09, 4.0), 'wall')
    for z in (0.5, 3.5):
        g.box((-1.5, WALL_Y - 0.09, z - 0.5), (1.5, WALL_Y + 0.09, z + 0.5), 'wall')
    wall_bands(g, dado_gaps=((-1.62, 1.62),))
    panels(g, -1.79, 1.79, ((0.30, 0.78),))
    panels(g, -1.79, 1.79, ((3.20, 3.46),), cols=1)
    for sx in (-1, 1):
        g.box((sx * 1.56 - 0.06, 1.24, 0.88), (sx * 1.56 + 0.06, 1.35, 3.12), 'trim')
        g.box((sx * 1.485 - 0.015, 1.41, 0.9), (sx * 1.485 + 0.015, 1.61, 3.10), 'panel')
    g.box((-1.62, 1.24, 3.02), (1.62, 1.35, 3.12), 'trim')
    g.box((-1.67, 1.22, 0.88), (1.67, 1.39, 0.98), 'trim')
    g.box((-1.5, 1.41, 1.0), (1.5, 1.61, 1.03), 'panel')
    g.box((-1.5, 1.41, 2.97), (1.5, 1.61, 3.0), 'panel')
    g.box((-0.035, 1.41, 1.0), (0.035, 1.48, 2.97), 'trim')
    g.box((-1.5, 1.41, 2.42), (1.5, 1.46, 2.47), 'trim')
    g.dashes((-1.5, 1.19, 0.93), (1.5, 1.19, 0.93), 1.0, (0.22, 0.02, 0.028), 'glow')
    g.box((-1.8, 2.44, 0.70), (1.8, 2.46, 3.30), 'vista')
    for (x, w, h) in ((-1.15, 0.50, 1.25), (-0.55, 0.35, 0.80), (0.35, 0.60, 1.60), (1.05, 0.32, 0.95)):
        g.box((x - w / 2, 2.28, 0.70), (x + w / 2, 2.42, 0.70 + h), 'hull')
        for k in range(int(h / 0.3)):
            g.box((x - w * 0.30, 2.255, 0.80 + k * 0.3), (x - w * 0.30 + 0.05, 2.285, 0.86 + k * 0.3), 'glow')
    to_object(g, "VK_Wall_Window_4m", COL, (4, 0.18, 4))
    collide()

    module('VK_Glass_Pane_4m', 'Optional translucent pane for the 3 m window opening.', (3.0, 0.02, 2.1))
    g = Geo("VK_Glass_Pane_4m")
    g.box((-1.48, 1.355, 0.96), (1.48, 1.375, 3.04), 'glass')
    for x0 in (-1.2, 0.3):
        g.f([g.v(p) for p in [(x0, 1.350, 1.05), (x0 + 0.10, 1.350, 1.05),
                              (x0 + 0.52, 1.350, 2.95), (x0 + 0.42, 1.350, 2.95)]], 'glass')
    to_object(g, "VK_Glass_Pane_4m", COL, (3.0, 0.02, 2.1))
    collide()


# ---------------------------------------------------------------- structure
def build_structure():
    module('VK_Pillar_4m', 'Structural post at the rear of a landing or a wall seam.', (0.26, 0.34, 4))
    g = Geo("VK_Pillar_4m")
    g.box((-0.10, 1.02, 0.0), (0.10, 1.22, 4.0), 'trim')
    g.box((-0.035, 0.985, 0.30), (0.035, 1.02, 3.70), 'hull')
    g.box((-0.014, 0.972, 0.34), (0.014, 0.99, 3.66), 'accent')
    for z in (0.0, 3.78):
        g.box((-0.16, 0.96, z), (0.16, 1.28, z + 0.22), 'hull')
    to_object(g, "VK_Pillar_4m", COL, (0.26, 0.34, 4))
    collide()

    module('VK_Beam_4m', 'Rear lintel or ceiling beam, kept behind the fighter plane.', (4, 0.3, 0.3))
    g = Geo("VK_Beam_4m")
    g.box((-2.0, 0.95, -0.31), (2.0, 1.29, -0.01), 'hull')
    g.box((-2.0, 0.93, -0.345), (2.0, 1.31, -0.310), 'trim')
    g.box((-2.0, 0.93, -0.045), (2.0, 1.31, -0.010), 'trim')
    g.box((-2.0, 0.915, -0.20), (2.0, 0.935, -0.16), 'core')
    to_object(g, "VK_Beam_4m", COL, (4, 0.3, 0.3))
    collide()

    module('VK_End_Trim', 'Exposed end trim for a floor boundary; attach at its X end.', (0.08, 2.4, 0.4))
    g = Geo("VK_End_Trim")
    g.prism_x(-0.04, 0.04, [(y - 0.012, z) for (y, z) in FASCIA], 'hull')
    g.box((-0.05, -1.21, -0.06), (0.05, 1.21, -0.02), 'trim')
    g.box((-0.05, -1.222, -0.19), (0.05, -1.205, -0.11), 'glow')
    to_object(g, "VK_End_Trim", COL, (0.08, 2.4, 0.4))
    collide()


# ---------------------------------------------------------------- run
def build_all():
    for ob in list(bpy.context.scene.collection.all_objects):
        if ob.name.startswith(("VK_Platform", "Stage_")):
            d = ob.data
            bpy.data.objects.remove(ob, do_unlink=True)
            if isinstance(d, bpy.types.Mesh) and d.users == 0:
                bpy.data.meshes.remove(d)
    build_floors()
    build_ramp_and_stairs()
    build_balcony()
    build_walls()
    build_structure()

    # ---- collision sidecars, their v1 shape, in game units
    written = {}
    for part, lines in COLLISION.items():
        doc = {"version": 1, "atlas": "vk_kit",
               "lines": [[k, round(a * UNIT, 4), round(b * UNIT, 4),
                          round(c * UNIT, 4), round(d * UNIT, 4), f]
                         for (k, a, b, c, d, f) in lines]}
        p = os.path.join(OUT, part.lower() + ".coll.json")
        with open(p, 'w') as fh:
            json.dump(doc, fh)
        written[part] = p

    # ---- a stage placement table, the equivalent of their Lua ROOM table
    PLACEMENT.extend([
        # platforms: the stage kit's parametric slab at four sizes
        {"part": "VK_Platform_Main", "w": 20.0, "d": 2.4, "th": 0.45, "x": 0, "y": 0, "z": 0, "solid": True},
        {"part": "VK_Platform_SideL", "w": 8.0, "d": 2.0, "th": 0.35, "x": -13, "y": 0, "z": 5.0},
        {"part": "VK_Platform_SideR", "w": 8.0, "d": 2.0, "th": 0.35, "x": 13, "y": 0, "z": 5.0},
        {"part": "VK_Platform_Centre", "w": 5.0, "d": 1.8, "th": 0.30, "x": 0, "y": 0.6, "z": 8.0},
        # ground storey, 9 m behind the stage so the platform silhouette stays readable
        {"part": "VK_Wall_Solid_4m", "x": -6, "y": 9, "z": 0},
        {"part": "VK_Wall_Doorway_4m", "x": -2, "y": 9, "z": 0},
        {"part": "VK_Wall_Window_4m", "x": 2, "y": 9, "z": 0},
        {"part": "VK_Wall_Solid_4m", "x": 6, "y": 9, "z": 0},
        {"part": "VK_Glass_Pane_4m", "x": 2, "y": 9, "z": 0},
        {"part": "VK_Pillar_4m", "x": -8, "y": 9, "z": 0},
        {"part": "VK_Pillar_4m", "x": 8, "y": 9, "z": 0},
        {"part": "VK_Beam_4m", "x": -6, "y": 9, "z": 4.0},
        {"part": "VK_Beam_4m", "x": -2, "y": 9, "z": 4.0},
        {"part": "VK_Beam_4m", "x": 2, "y": 9, "z": 4.0},
        {"part": "VK_Beam_4m", "x": 6, "y": 9, "z": 4.0},
        # upper gallery at +2 m, reached by the stairs on the left and the ramp on the right
        {"part": "VK_Floor_4m", "x": -6, "y": 17, "z": 2.0},
        {"part": "VK_Floor_4m", "x": -2, "y": 17, "z": 2.0},
        {"part": "VK_Floor_Opening_4m", "x": 2, "y": 17, "z": 2.0},
        {"part": "VK_Floor_4m", "x": 6, "y": 17, "z": 2.0},
        {"part": "VK_Wall_Solid_4m", "x": -6, "y": 17, "z": 2.0},
        {"part": "VK_Wall_Solid_4m", "x": -2, "y": 17, "z": 2.0},
        {"part": "VK_Wall_Doorway_4m", "x": 2, "y": 17, "z": 2.0},
        {"part": "VK_Wall_Solid_4m", "x": 6, "y": 17, "z": 2.0},
        {"part": "VK_Pillar_4m", "x": -8, "y": 17, "z": 2.0},
        {"part": "VK_Pillar_4m", "x": 8, "y": 17, "z": 2.0},
        {"part": "VK_Beam_4m", "x": -6, "y": 17, "z": 6.0},
        {"part": "VK_Beam_4m", "x": -2, "y": 17, "z": 6.0},
        {"part": "VK_Beam_4m", "x": 2, "y": 17, "z": 6.0},
        {"part": "VK_Beam_4m", "x": 6, "y": 17, "z": 6.0},
        # the walkable route through the room, on the 2 m grid
        {"part": "VK_Floor_4m", "x": -20, "y": 13, "z": 0, "solid": True},
        {"part": "VK_Stairs_4m_Rise2m", "x": -16, "y": 13, "z": 0},
        {"part": "VK_Balcony_4m", "x": -12, "y": 13, "z": 2.0},
        {"part": "VK_Ramp_4m_Rise2m", "x": 12, "y": 13, "z": 0},
        {"part": "VK_Floor_2m", "x": 16, "y": 13, "z": 0, "solid": True},
        # exposed floor ends, attached where a floor run actually stops
        {"part": "VK_End_Trim", "x": -22, "y": 13, "z": 0},
        {"part": "VK_End_Trim", "x": 18, "y": 13, "z": 0},
        {"part": "VK_End_Trim", "x": 0, "y": 17, "z": 2.0},
    ])
    with open(os.path.join(OUT, 'vk_stage_placement.json'), 'w') as fh:
        json.dump({"unit": UNIT, "grid_m": 2.0, "front": "-Y",
                   "flag_encoding": "1 passthrough + 2 ledges",
                   "placement": PLACEMENT}, fh, indent=2)

    stats = {}
    for name, coll in MODULES.items():
        tris = 0
        for ob in coll.objects:
            if ob.type == 'MESH':
                me = ob.data
                me.calc_loop_triangles()
                tris += len(me.loop_triangles)
        stats[name] = {"objects": len(coll.objects), "tris": tris,
                       "size": list(coll['module_size']),
                       "collision_lines": len(COLLISION.get(name, []))}
    with open(os.path.join(OUT, 'vk_kit_manifest.json'), 'w') as fh:
        json.dump({"unit": UNIT, "atlas": "vk_masks", "modules": stats}, fh, indent=2)
    # the kit's full recolourable palette, a superset of the platform's six slots
    with open(os.path.join(BF['TEXDIR'], 'vk_kit_palette.json'), 'w') as fh:
        json.dump(KIT_PALETTE, fh, indent=2)
    return stats


STATS = build_all()
print(json.dumps(STATS, indent=2))