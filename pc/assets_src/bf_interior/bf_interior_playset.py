"""Reusable 2 m grid interior modules, plus an open-front assembly example (v6).

Blender X is travel, Z is height, Y is depth. The front is negative Y.
Meshes are new structural assets; BF source supplies shared trim/light/hull materials.
The scene is an art assembly, not an installed Melee stage or collision export.

v6 (2026-09-27, art pass): chamfered boxes everywhere, panelled walls with pilasters,
trim bands and baseboard; framed windows with a lit vista card; doorways with
reveals, a lit architrave and a glowing corridor; floors and ramp share a faceted
BF-style fascia (steel lip, dark rim band with cyan dashes, magenta underside line);
the ramp is a full-thickness sloped floor with a rear rail. Connection dimensions and
origins are unchanged from v5.

Env: BF_NO_RENDER=1 skips the preview renders; BF_RENDER_ONLY=room,shelf,close picks them;
BF_NO_SAVE=1 leaves the .blend alone (the game exporter runs this script that way).
"""
import math
import os
import runpy
from pathlib import Path
import bpy
import bmesh
from mathutils import Vector

HERE = Path(__file__).resolve().parent
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
BF_SRC = next(p for p in (HERE.parent / 'platform' / 'bf_platform_build.py',       # stage kit
                           HERE.parent / 'bf_platform' / 'bf_platform_build.py')    # game repo
              if p.exists())
BF = runpy.run_path(str(BF_SRC))
bpy.data.objects.remove(bpy.data.objects['BF_Platform'], do_unlink=True)
scene = bpy.context.scene
scene.name = '01 | Open-front playset example'
trim = bpy.data.materials['BF_Trim']
cyan = bpy.data.materials['BF_Glow']
magenta = bpy.data.materials['BF_Core']
hull = bpy.data.materials['BF_Hull']
attr_vec, cmix, seam_lines, fmax, band = (BF[k] for k in
    ('attr_vec', 'cmix', 'seam_lines', 'fmax', 'band'))
VIOLET = BF['VIOLET']


# ------------------------------------------------------------------ materials
def material(name, color, metal=.0, rough=.4, alpha=1, emit=None, strength=0):
    m = bpy.data.materials.new(name); m.use_nodes = True
    m.diffuse_color = (*color, alpha)
    p = m.node_tree.nodes.get('Principled BSDF')
    p.inputs['Base Color'].default_value = (*color, 1)
    p.inputs['Metallic'].default_value = metal
    p.inputs['Roughness'].default_value = rough
    p.inputs['Alpha'].default_value = alpha
    if emit:
        p.inputs['Emission Color'].default_value = (*emit, 1)
        p.inputs['Emission Strength'].default_value = strength
    if alpha < 1:
        m.surface_render_method = 'DITHERED'; m.use_transparency_overlap = False
    return m


def gradient_emitter(name, z0, z1, bottom, top, strength):
    """Emissive card whose colour runs bottom->top over local Z (bf_pos)."""
    m, t, bsdf = BF['new_material'](name)
    px, py, pz = attr_vec(t, 'bf_pos')
    k = ((pz - z0) / (z1 - z0)).max(0.0).min(1.0)
    col = cmix(t, k, (*bottom, 1), (*top, 1), 'Bottom to top')
    bsdf.inputs['Base Color'].default_value = (0, 0, 0, 1)
    bsdf.inputs['Roughness'].default_value = .8
    t.link(col, bsdf.inputs['Emission Color'])
    bsdf.inputs['Emission Strength'].default_value = strength
    BF['layout'](t)
    return m


def walking_surface():
    """Playable top: lit violet front to dark rear, 1 m seams, a cyan guide line near the front."""
    m, t, bsdf = BF['new_material']('Interior | walking surface')
    px, py, pz = attr_vec(t, 'bf_pos')
    guide = band(py, -1.0, .014).named('Front guide line')
    seams = fmax(t, seam_lines(px, 1.0, .007), band(py, .95, .006)) * (1.0 - guide)
    k = ((py + 1.2) / 2.4).max(0.0).min(1.0)
    base = cmix(t, k, (.11, .07, .30, 1), (.035, .028, .10, 1), 'Front to rear')
    base = cmix(t, seams, base, (.006, .005, .016, 1), 'Seams')
    t.link(base, bsdf.inputs['Base Color'])
    bsdf.inputs['Metallic'].default_value = .8
    t.put(bsdf.inputs['Roughness'], .26 + seams * .4)
    bsdf.inputs['Emission Color'].default_value = BF['CYAN']
    t.put(bsdf.inputs['Emission Strength'], guide * 2.2)
    BF['layout'](t)
    return m


wall = material('Interior | wall shell', (.022, .02, .048), .3, .62)
panel = material('Interior | wall panel', (.042, .036, .088), .55, .45)
floor = walking_surface()
glass = material('Interior | blue transparent insert', (.12, .38, .7), .1, .05, .1)
dark = material('Interior | deep frame', (.012, .012, .03), .6, .45)
rimband = material('Interior | rim band', (.02, .02, .05), .9, .4)
accent = material('Interior | wall accent line', (0, 0, 0), 0, .5, emit=VIOLET[:3], strength=1.6)
portal = gradient_emitter('Interior | portal glow', 0, 2.6,
                          (.30, .07, .78), (.01, .008, .035), .9)
vista = gradient_emitter('Interior | window vista', .7, 3.3,
                         (.55, .10, .60), (.008, .01, .05), 1.3)

COL = None
modules = {}


def module(name, usage, size):
    global COL
    COL = bpy.data.collections.new(name)
    COL.use_fake_user = True
    COL.asset_mark()
    COL.asset_data.description = usage + ' Grid: 2 m; front: -Y; travel: X; up: Z.'
    COL['module_size'] = size
    COL['purpose'] = usage
    COL['grid_m'] = 2.0
    modules[name] = COL


# ------------------------------------------------------------------ mesh helpers
def finish(name, bm, mat, collection=None):
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    me = bpy.data.meshes.new(name); bm.to_mesh(me); bm.free()
    ob = bpy.data.objects.new(name, me); (collection or COL).objects.link(ob)
    me.materials.append(mat)
    a = me.attributes.new(name='bf_pos', type='FLOAT_VECTOR', domain='POINT')
    for v, d in zip(me.vertices, a.data): d.vector = v.co
    uv = me.uv_layers.new(name='UVMap')
    for p in me.polygons:
        axis = max(range(3), key=lambda i: abs(p.normal[i]))
        axes = [i for i in range(3) if i != axis]
        for li in p.loop_indices:
            co = me.vertices[me.loops[li].vertex_index].co
            uv.data[li].uv = (co[axes[0]], co[axes[1]])
    return ob


def warp(bm, shear=0.0, lift=0.0):
    """Ramp shear: z += shear*x + lift, applied after bevels so they keep their size."""
    if shear or lift:
        for v in bm.verts: v.co.z += shear * v.co.x + lift


def box(name, p, s, mat, bevel=0.0, shear=0.0, lift=0.0, collection=None):
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        v.co = Vector((p[0] + v.co.x * s[0], p[1] + v.co.y * s[1], p[2] + v.co.z * s[2]))
    b = min(bevel, .45 * min(s))
    if b > .001:
        bmesh.ops.bevel(bm, geom=list(bm.edges), offset=b, segments=1, affect='EDGES',
                        profile=.5, clamp_overlap=True)
    warp(bm, shear, lift)
    return finish(name, bm, mat, collection)


def prism(name, x0, x1, profile, mat, shear=0.0, lift=0.0, collection=None):
    """Extrude a YZ polygon along X (flush-tiling cross-sections)."""
    bm = bmesh.new()
    a = [bm.verts.new((x0, y, z)) for y, z in profile]
    b = [bm.verts.new((x1, y, z)) for y, z in profile]
    n = len(profile)
    bm.faces.new(a); bm.faces.new(b)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((a[i], a[j], b[j], b[i]))
    warp(bm, shear, lift)
    return finish(name, bm, mat, collection)


def quad(name, pts, mat, shear=0.0, lift=0.0, collection=None):
    bm = bmesh.new(); bm.faces.new([bm.verts.new(p) for p in pts])
    warp(bm, shear, lift)
    return finish(name, bm, mat, collection)


def dashes(label, width, y, z, mat, spacing=.5, size=(.2, .008, .04), shear=0.0, lift=0.0):
    """Evenly spaced lights; spacing stays near `spacing` whatever the width."""
    n = max(1, round(width / spacing))
    for i in range(n):
        x = (i - (n - 1) / 2) * width / n
        box(label, (x, y, z), size, mat, shear=shear, lift=lift)


# ------------------------------------------------------------------ floors and ramp
# Hull cross-section (Y, Z): a flat walkable top, a vertical front band, then two
# facets stepping back underneath, like BF_Platform's faceted underside.
HULL = [(-1.18, -.05), (1.2, -.05), (1.2, -.4), (-.70, -.4), (-1.0, -.335), (-1.18, -.20)]
FACET = ((-1.0, -.335), (-.70, -.4))  # the lower facet carries the magenta line


def fascia(width, shear=0.0, lift=0.0, dy=0.0):
    """Shared floor/ramp build: deck, steel lip, rim band with cyan dashes, faceted hull."""
    kw = dict(shear=shear, lift=lift)
    prism('Floor / faceted hull', -width / 2, width / 2,
          [(y + dy, z) for y, z in HULL], hull, **kw)
    box('Floor / walkable deck', (0, .02 + dy / 2, -.025), (width, 2.36 - dy, .05), floor, .008, **kw)
    box('Floor / front steel lip', (0, -1.18 + dy, -.035), (width, .09, .075), trim, .018, **kw)
    box('Floor / dark rim band', (0, -1.195 + dy, -.13), (width, .04, .1), rimband, .006, **kw)
    box('Floor / rear socket edge', (0, 1.17, .01), (width, .06, .03), trim, .008, **kw)
    dashes('Floor / rim light', width, -1.217 + dy, -.13, cyan, **kw)
    (y0, z0), (y1, z1) = FACET
    ny, nz = z1 - z0, -(y1 - y0)  # outward normal of the facet (down and forward)
    L = math.hypot(ny, nz); ny, nz = -ny / L * .004, -nz / L * .004
    ym, zm = (y0 + y1) / 2 + dy, (z0 + z1) / 2
    ty, tz = (y1 - y0) * .06, (z1 - z0) * .06
    quad('Floor / underside energy line',
         [(-width / 2, ym - ty + ny, zm - tz + nz), (width / 2, ym - ty + ny, zm - tz + nz),
          (width / 2, ym + ty + ny, zm + tz + nz), (-width / 2, ym + ty + ny, zm + tz + nz)],
         magenta, **kw)


module('Floor_4m', 'Straight load-bearing floor; top surface is Z=0.', (4, 2.4, .4))
fascia(4)
module('Floor_2m', 'Short landing or infill floor; top surface is Z=0.', (2, 2.4, .4))
fascia(2)

module('Ramp_4m_Rise2m', 'Full-thickness traversable ramp with a rear rail. '
       'Start X=-2 Z=0; end X=2 Z=2.', (4, 2.4, 2))
# The floor fascia, sheared to the slope and set 3 cm back so where it dips into a
# floor below it stays behind that floor's own fascia.
RAMP = dict(shear=.5, lift=1.0)
fascia(4, dy=.03, **RAMP)
for x in (-1.5, -.5, .5, 1.5):
    box('Ramp / rail post', (x, 1.1, .55), (.07, .07, 1.1), trim, .015, **RAMP)
box('Ramp / rail top', (0, 1.1, 1.1), (4, .1, .06), trim, .02, **RAMP)
box('Ramp / rail light', (0, 1.048, 1.055), (4, .006, .018), cyan, **RAMP)
box('Ramp / rail glass', (0, 1.1, .56), (4, .02, .9), glass, **RAMP)


# ------------------------------------------------------------------ walls
WALL_Y, FRONT = 1.4, 1.31  # wall centre and its front face


def wall_bands(width=4, base_gaps=(), dado_gaps=()):
    """Baseboard, dado rail, crown, pilasters and a quiet accent line. Gaps are (x0, x1) spans."""
    def runs(gaps):
        edges = [-width / 2] + [v for g in gaps for v in g] + [width / 2]
        return [(edges[i], edges[i + 1]) for i in range(0, len(edges), 2)]
    for a, b in runs(base_gaps):
        box('Wall / baseboard', ((a + b) / 2, 1.28, .11), (b - a, .07, .22), trim, .012)
        box('Wall / baseboard light', ((a + b) / 2, 1.243, .05), (b - a, .006, .015), accent)
    for a, b in runs(dado_gaps):
        box('Wall / dado rail', ((a + b) / 2, 1.29, 1.12), (b - a, .05, .06), trim, .012)
    box('Wall / crown band', (0, 1.295, 3.66), (width, .04, .24), panel, .01)
    box('Wall / crown accent', (0, 1.273, 3.66), (width, .006, .022), accent)
    box('Wall / crown', (0, 1.275, 3.9), (width, .09, .2), trim, .02)
    for x in (-width / 2 + .09, width / 2 - .09):
        # A half pilaster at each bay edge; two neighbouring bays read as one pilaster.
        box('Wall / pilaster', (x, 1.28, 1.9), (.18, .06, 3.36), panel, .02)


def panels(x0, x1, rows, cols=None, y=1.29, name='Wall / raised panel'):
    """Raised plates with 6 cm reveals. Columns stay ~1.8 m wide so details never stretch."""
    span = x1 - x0
    n = cols or max(1, round(span / 1.8))
    w = (span - .06 * (n - 1)) / n
    for i in range(n):
        cx = x0 + w / 2 + i * (w + .06)
        for z0, z1 in rows:
            box(name, (cx, y, (z0 + z1) / 2), (w, .04, z1 - z0), panel, .022)


module('Wall_Solid_4m', 'Back wall bay, placed behind the fighter travel plane.', (4, .18, 4))
box('Wall / shell', (0, WALL_Y, 2), (4, .18, 4), wall)
wall_bands()
panels(-1.79, 1.79, ((.3, 1.04), (1.2, 3.46)))

module('Wall_Doorway_4m', 'Back-wall portal with a real 1.6 m by 2.6 m opening.', (4, .18, 4))
for x in (-1.4, 1.4): box('Door / shell side', (x, WALL_Y, 2), (1.2, .18, 4), wall)
box('Door / shell above', (0, WALL_Y, 3.3), (1.6, .18, 1.4), wall)
wall_bands(base_gaps=((-1.0, 1.0),), dado_gaps=((-1.0, 1.0),))
for s in (-1, 1):
    panels(*sorted((s * 1.79, s * 1.06)), ((.3, 1.04), (1.2, 3.46)))
panels(-.9, .9, ((2.98, 3.46),))
# Architrave: steel frame proud of the wall with a cyan line on its inner edge.
for s in (-1, 1):
    box('Door / architrave jamb', (s * .89, 1.265, 1.39), (.18, .1, 2.78), trim, .025)
    box('Door / frame light', (s * .812, 1.212, 1.3), (.022, .01, 2.5), cyan)
box('Door / architrave head', (0, 1.265, 2.69), (1.96, .1, .18), trim, .025)
box('Door / frame light', (0, 1.212, 2.612), (1.6, .01, .022), cyan)
box('Door / status light', (0, 1.25, 2.87), (.5, .03, .06), magenta, .008)
box('Door / threshold', (0, 1.33, .018), (1.6, .2, .036), trim, .01)
# Reveal: the opening has depth, a short lit corridor and a glowing end wall.
DEPTH = .95
cy = FRONT + DEPTH / 2
for s in (-1, 1):
    box('Door / reveal lining', (s * .785, cy, 1.3), (.03, DEPTH, 2.6), panel)
    box('Door / reveal light', (s * .765, cy + .15, .03), (.01, DEPTH - .3, .02), cyan)
box('Door / reveal soffit', (0, cy, 2.585), (1.6, DEPTH, .03), panel)
box('Door / corridor floor', (0, cy, -.02), (1.6, DEPTH, .04), floor)
box('Door / corridor ceiling light', (0, cy + .1, 2.56), (.9, .08, .02), cyan)
box('Door / corridor end glow', (0, FRONT + DEPTH, 1.3), (1.6, .03, 2.6), portal)
for s in (-1, 1):
    box('Door / corridor end rib', (s * .45, FRONT + DEPTH - .03, 1.3), (.06, .04, 2.6), dark)

module('Wall_Window_4m', 'Back-wall viewing opening onto a lit vista card. '
       'Glass is a separate optional insert.', (4, .18, 4))
for x in (-1.75, 1.75): box('Window / shell side', (x, WALL_Y, 2), (.5, .18, 4), wall)
for z in (.5, 3.5): box('Window / shell above and below', (0, WALL_Y, z), (3, .18, 1), wall)
wall_bands(dado_gaps=((-1.62, 1.62),))
panels(-1.79, 1.79, ((.3, .78),))
panels(-1.79, 1.79, ((3.2, 3.46),), cols=1)
for s in (-1, 1):
    box('Window / frame jamb', (s * 1.56, 1.265, 2), (.12, .11, 2.24), trim, .025)
    box('Window / reveal lining', (s * 1.485, 1.41, 2), (.03, .2, 2), panel)
box('Window / frame head', (0, 1.265, 3.07), (3.24, .11, .1), trim, .025)
box('Window / sill', (0, 1.245, .93), (3.34, .17, .1), trim, .025)
for z in (1.015, 2.985): box('Window / reveal lining', (0, 1.41, z), (3, .2, .03), panel)
box('Window / mullion', (0, 1.42, 2), (.07, .07, 2), trim, .015)
box('Window / transom', (0, 1.42, 2.45), (3, .05, .05), trim, .012)
dashes('Window / sill light', 3.0, 1.158, .93, cyan, spacing=1.0, size=(.24, .008, .03))
# Vista card and a few dark silhouettes: a dim backdrop instead of an opaque board.
box('Window / vista card', (0, 2.45, 2), (3.6, .02, 2.6), vista)
for x, w, h in ((-1.15, .5, 1.25), (-.55, .35, .8), (.35, .6, 1.6), (1.05, .32, .95)):
    box('Window / vista silhouette', (x, 2.3, .7 + h / 2), (w, .1, h), dark)
    for k in range(int(h / .3)):
        box('Window / vista window light', (x - w * .2 + (k % 2) * w * .4, 2.245, .85 + k * .3),
            (.05, .006, .06), cyan)

module('Window_Glass_Insert', 'Optional translucent pane for the 3 m window opening.', (2.9, .025, 1.9))
box('Window / removable glass', (0, 1.36, 2), (2.96, .02, 1.96), glass)
for x0 in (-1.2, .3):
    quad('Window / glass glint', [(x0, 1.348, 1.1), (x0 + .12, 1.348, 1.1),
                                  (x0 + .62, 1.348, 2.9), (x0 + .5, 1.348, 2.9)], glass)

module('Wall_Side_Return', 'End wall across the room depth. Mirror or rotate for the opposite end.',
       (.18, 2.4, 4))
box('Return / shell', (0, 0, 2), (.18, 2.4, 4), wall)
box('Return / exposed cut edge', (0, -1.2, 2), (.26, .1, 4), trim, .025)
for i in range(6):
    box('Return / cut edge light', (0, -1.252, .5 + i * .6), (.04, .006, .2), cyan)
for s in (-1, 1):
    for z, h in ((.11, .22), (3.9, .2)):
        box('Return / molding', (s * .1, 0, z), (.04, 2.4, h), trim, .01)
    for z0, z1 in ((.3, 1.04), (1.2, 3.46)):
        box('Return / raised panel', (s * .1, .05, (z0 + z1) / 2), (.04, 2.1, z1 - z0), panel, .022)


# ------------------------------------------------------------------ rear structure
module('Beam_4m', 'Rear lintel or ceiling beam, kept behind the fighter plane.', (4, .3, .3))
box('Beam / body', (0, 1.12, -.16), (4, .34, .3), hull, .025)
box('Beam / lower lip', (0, 1.1, -.315), (4, .38, .035), trim, .01)
box('Beam / upper lip', (0, 1.1, -.015), (4, .38, .03), trim, .01)
box('Beam / energy line', (0, .947, -.16), (4, .006, .02), magenta)

module('Rear_Post_4m', 'Structural post placed at the rear of a landing or wall seam.', (.22, .3, 4))
box('Post / web', (0, 1.1, 2), (.2, .2, 4), trim, .03)
box('Post / groove', (0, .998, 2), (.07, .012, 3.3), dark)
box('Post / light', (0, .99, 2), (.022, .006, 3.2), cyan)
for z in (.12, 3.88): box('Post / socket', (0, 1.1, z), (.34, .34, .24), hull, .04)

module('Rear_Glass_Rail_4m', 'Optional rear balcony rail; does not obscure the open front.', (4, .14, 1.1))
box('Rail / glass', (0, 1.07, .56), (3.76, .02, .9), glass)
for x in (-1.92, 1.92): box('Rail / post', (x, 1.07, .55), (.08, .14, 1.1), trim, .02)
box('Rail / top', (0, 1.07, 1.08), (4, .12, .06), trim, .02)
box('Rail / light', (0, 1.007, 1.03), (4, .006, .018), cyan)

module('Door_Leaf', 'Optional separate door leaf; doorway remains usable without it.', (1.5, .08, 2.5))
for s in (-1, 1):
    box('Door leaf / half', (s * .39, 1.4, 1.29), (.77, .06, 2.56), panel, .015)
    box('Door leaf / inset', (s * .39, 1.366, 1.45), (.55, .01, 1.9), dark, .004)
    box('Door leaf / edge light', (s * .035, 1.366, 1.29), (.012, .006, 2.3), cyan)
box('Door leaf / latch', (0, 1.36, 1.15), (.2, .03, .12), magenta, .01)

module('Floor_End_Trim', 'Exposed end trim for a floor boundary; attach at its X end.', (.08, 2.4, .4))
prism('End / cap', -.04, .04, [(y - .01, z) for y, z in HULL], hull)
box('End / upper silver edge', (0, 0, -.02), (.1, 2.42, .04), trim, .012)
box('End / light', (0, -1.205, -.13), (.1, .01, .04), cyan)


# ------------------------------------------------------------------ backlog parts (v6)
def moved(dx, build):
    """Run a builder and shift what it made along X (bf_pos follows, so seams stay put)."""
    before = set(COL.objects)
    build()
    for ob in set(COL.objects) - before:
        for v in ob.data.vertices: v.co.x += dx
        for d in ob.data.attributes['bf_pos'].data: d.vector.x += dx


module('Floor_Opening_4m', 'Floor bay with a 2 m drop-through gap in the middle; '
       'same ends and top as Floor_4m.', (4, 2.4, .4))
for s in (-1, 1):
    moved(s * 1.5, lambda: fascia(1))
    prism('Opening / cut face cap', s * 1.0 - .03, s * 1.0 + .03, [(y - .01, z) for y, z in HULL], hull)
    box('Opening / edge lip', (s * 1.0, 0, -.02), (.08, 2.42, .04), trim, .012)
    box('Opening / hazard light', (s * .955, -.05, .002), (.02, 2.2, .004), magenta)

module('Stairs_4m_Rise2m', 'Eight 0.25 m steps on the ramp footprint. Start X=-2 Z=0; end X=2 Z=2.',
       (4, 2.4, 2))
prism('Stairs / faceted stringer', -2, 2, [(y + .03, z) for y, z in HULL], hull, **RAMP)
(y0, z0), (y1, z1) = FACET
quad('Stairs / underside energy line',
     [(-2, (y0 + y1) / 2 + .03, (z0 + z1) / 2 - .005), (2, (y0 + y1) / 2 + .03, (z0 + z1) / 2 - .005),
      (2, (y0 + y1) / 2 + .045, (z0 + z1) / 2 - .01), (-2, (y0 + y1) / 2 + .045, (z0 + z1) / 2 - .01)],
     magenta, **RAMP)
for i in range(8):
    x0, top = -2 + i * .5, (i + 1) * .25
    box('Stairs / tread', (x0 + .25, .02, top - .15), (.5, 2.36, .3), floor, .01)
    box('Stairs / nosing', (x0 + .25, -1.15, top - .03), (.5, .1, .06), trim, .015)
    box('Stairs / riser light', (x0 + .25, -1.203, top - .16), (.22, .008, .035), cyan)
for x in (-1.5, -.5, .5, 1.5):
    box('Stairs / rail post', (x, 1.1, .55), (.07, .07, 1.1), trim, .015, **RAMP)
box('Stairs / rail top', (0, 1.1, 1.1), (4, .1, .06), trim, .02, **RAMP)
box('Stairs / rail light', (0, 1.048, 1.055), (4, .006, .018), cyan, **RAMP)

module('Balcony_4m', 'Upper ledge on corbels, hung from the back wall; top surface is Z=0.',
       (4, 2.4, 1.2))
fascia(4)
for x in (-1.3, 1.3):
    prism('Balcony / corbel', x - .09, x + .09, [(.55, -.4), (1.3, -.4), (1.3, -1.35)], hull)
    box('Balcony / corbel light', (x, .9, -.62), (.03, .01, .26), cyan)
box('Balcony / rear rail', (0, 1.07, .56), (3.76, .02, .9), glass)
box('Balcony / rail top', (0, 1.07, 1.08), (4, .12, .06), trim, .02)

module('Corner_Inside_4m', 'Rear inside corner where the back wall meets a side return. '
       'Built for the left end (room interior +X); mirror X for the right end.', (.4, .4, 4))
box('Corner / column', (.2, 1.2, 2), (.22, .2, 4), panel, .03)
box('Corner / steel edge', (.31, 1.1, 2), (.04, .04, 3.6), trim, .012)
box('Corner / light', (.3, 1.085, 2), (.012, .006, 3.2), cyan)
for z, h in ((.11, .22), (3.9, .2)):
    box('Corner / cap', (.2, 1.2, z), (.28, .26, h), trim, .015)

module('Corner_Outside_4m', 'Convex corner cap for an exposed wall end that turns toward the camera.',
       (.34, .34, 4))
box('Corner / cut-corner body', (0, 0, 2), (.34, .34, 4), hull, .1)
box('Corner / steel face', (0, -.172, 2), (.12, .02, 3.7), trim, .008)
for i in range(6):
    box('Corner / light', (0, -.185, .5 + i * .6), (.03, .006, .2), cyan)
for z, h in ((.12, .24), (3.88, .24)):
    box('Corner / socket', (0, 0, z), (.42, .42, h), trim, .05)


# ------------------------------------------------------------------ assembly
assembly = bpy.data.collections.new('ASSEMBLY | instance the reusable parts')
scene.collection.children.link(assembly)


def instance(name, at, label=None, collection=assembly):
    ob = bpy.data.objects.new(label or name, None); collection.objects.link(ob)
    ob.instance_type = 'COLLECTION'; ob.instance_collection = modules[name]; ob.location = at
    return ob


# Clear side-on route: ground hall -> two continuous ramps -> upper exit landing.
for x in (-6, -2, 2, 6): instance('Floor_4m', (x, 0, 0))
for x, walltype in ((-6, 'Wall_Doorway_4m'), (-2, 'Wall_Solid_4m'), (2, 'Wall_Window_4m'), (6, 'Wall_Solid_4m')):
    instance(walltype, (x, 0, 0))
instance('Window_Glass_Insert', (2, 0, 0))
for x in (-6, -2, 2, 6):
    instance('Wall_Doorway_4m' if x == 6 else 'Wall_Solid_4m', (x, 0, 4))
instance('Floor_4m', (6, 0, 4), 'Upper exit landing')
instance('Ramp_4m_Rise2m', (-2, 0, 0), 'Ramp lower half')
instance('Ramp_4m_Rise2m', (2, 0, 2), 'Ramp upper half')
for x in (-8, 8):
    for z in (0, 4): instance('Wall_Side_Return', (x, 0, z))
for x in (4, 8): instance('Rear_Post_4m', (x, 0, 0))
for x in (-6, -2, 2, 6): instance('Beam_4m', (x, 0, 8))
for x in (-8, 8): instance('Floor_End_Trim', (x, 0, 0))
instance('Floor_End_Trim', (4, 0, 4), 'Landing exposed end')
instance('Rear_Glass_Rail_4m', (6, 0, 4), 'Landing rear rail')
instance('Balcony_4m', (-6, 0, 4), 'Upper left balcony')
instance('Floor_End_Trim', (-4, 0, 4), 'Balcony exposed end')
for z in (0, 4): instance('Corner_Inside_4m', (-8, 0, z), 'Rear inside corner')
for z in (0, 4): instance('Corner_Inside_4m', (8, 0, z), 'Rear inside corner, mirrored').scale.x = -1

# The second scene is a spaced parts shelf with one instance of every asset.
library = bpy.data.scenes.new('02 | Modular parts shelf')
shelf = bpy.data.collections.new('PARTS SHELF'); library.collection.children.link(shelf)
SHELF_COLS, SHELF_DX, SHELF_DZ = 6, 5.4, 6.4
for i, name in enumerate(modules):
    ob = instance(name, ((i % SHELF_COLS) * SHELF_DX, 0, -(i // SHELF_COLS) * SHELF_DZ), collection=shelf)
    ob.name = name + ' | library sample'


# ------------------------------------------------------------------ presentation
label_mat = material('Preview | label', (0, 0, 0), 0, .5, emit=(.55, .6, .8), strength=1.2)
backdrop_mat = gradient_emitter('Preview | backdrop', -10, 16, (.03, .018, .07), (.003, .003, .01), 1.0)
world = bpy.data.worlds.new('Studio dusk'); world.use_nodes = True
world.node_tree.nodes['Background'].inputs[0].default_value = (.02, .018, .045, 1)
world.node_tree.nodes['Background'].inputs[1].default_value = .35


def stage_view(sc, tag, eye, target, lens, backdrop_at, lights):
    pres = bpy.data.collections.new('PRESENTATION | ' + tag)
    sc.collection.children.link(pres)
    cam_data = bpy.data.cameras.new(tag); cam_data.lens = lens
    cam = bpy.data.objects.new(tag, cam_data); pres.objects.link(cam)
    cam.location = eye
    cam.rotation_euler = (Vector(target) - Vector(eye)).to_track_quat('-Z', 'Y').to_euler()
    sc.camera = cam
    box('Preview / gradient backdrop', backdrop_at, (90, .1, 50), backdrop_mat, collection=pres)
    for name, loc, aim, energy, size, color in lights:
        ld = bpy.data.lights.new(name, 'AREA'); ld.energy = energy; ld.color = color
        ld.shape = 'RECTANGLE'; ld.size, ld.size_y = size
        lo = bpy.data.objects.new(name, ld); pres.objects.link(lo); lo.location = loc
        lo.rotation_euler = (Vector(aim) - Vector(loc)).to_track_quat('-Z', 'Y').to_euler()
    sc.world = world
    r = sc.render
    r.engine = 'CYCLES'; sc.cycles.samples = 96; sc.cycles.use_denoising = True
    r.resolution_x = 1600; r.resolution_y = 1000; r.resolution_percentage = 100
    sc.view_settings.view_transform = 'AgX'; sc.view_settings.look = 'AgX - Medium High Contrast'
    return cam, pres


WARM, COOL = (1, .93, .9), (.75, .82, 1)
ROOM_LIGHTS = (
    ('Key, high front left', (-9, -16, 15), (-1, 0, 3), 5200, (8, 8), WARM),
    ('Fill, low front right', (12, -14, 3), (2, 0, 3), 1400, (8, 5), COOL),
    ('Graze, above the wall', (0, -2.5, 11), (0, 1.3, 3), 1800, (18, 1.2), COOL),
)
cam, room_pres = stage_view(scene, 'Cutaway overview', (0, -44, 12), (0, 0, 3.7), 74, (0, 7, 3), ROOM_LIGHTS)

sx = (SHELF_COLS - 1) * SHELF_DX / 2
rows = (len(modules) + SHELF_COLS - 1) // SHELF_COLS
sz = -(rows - 1) * SHELF_DZ / 2 + 1.6
for i, name in enumerate(modules):
    tb = bpy.data.curves.new('label ' + name, 'FONT'); tb.body = name.replace('_', ' ')
    tb.size = .32; tb.align_x = 'CENTER'
    to = bpy.data.objects.new('Label | ' + name, tb); tb.materials.append(label_mat)
    library.collection.objects.link(to)
    to.location = ((i % SHELF_COLS) * SHELF_DX, -1.6, -(i // SHELF_COLS) * SHELF_DZ - 1.2)
    to.rotation_euler = (math.pi / 2, 0, 0)
SHELF_LIGHTS = (
    ('Key, high front left', (sx - 12, -20, sz + 16), (sx, 0, sz), 9000, (12, 12), WARM),
    ('Fill, low front right', (sx + 16, -18, sz - 4), (sx, 0, sz), 2500, (10, 8), COOL),
)
stage_view(library, 'Shelf overview', (sx, -66, sz + 20), (sx, 0, sz), 58, (sx, 7, sz), SHELF_LIGHTS)

# A close-up camera in the room scene: doorway, ramp foot and floor fascia.
cd = bpy.data.cameras.new('Close-up'); cd.lens = 40
close = bpy.data.objects.new('Close-up | doorway and ramp foot', cd)
room_pres.objects.link(close)
close.location = (-4.2, -11.5, 2.6)
close.rotation_euler = (Vector((-4.6, 0, 1.5)) - close.location).to_track_quat('-Z', 'Y').to_euler()

bpy.context.preferences.filepaths.save_version = 0
bpy.context.preferences.filepaths.file_preview_type = 'NONE'
for screen in bpy.data.screens:
    for area in screen.areas:
        if area.type == 'VIEW_3D':
            area.spaces.active.region_3d.view_perspective = 'CAMERA'
            area.spaces.active.shading.type = 'MATERIAL'
if not os.environ.get('BF_NO_SAVE'):
    bpy.ops.wm.save_as_mainfile(filepath=str(HERE / 'bf_interior_playset.blend'))

if not os.environ.get('BF_NO_RENDER'):
    only = os.environ.get('BF_RENDER_ONLY', 'room,shelf,close').split(',')
    shots = (('room', scene, cam, 'bf_interior_playset_preview.png'),
             ('shelf', library, library.camera, 'bf_interior_playset_shelf.png'),
             ('close', scene, close, 'bf_interior_playset_closeup.png'))
    for key, sc, c, fname in shots:
        if key not in only: continue
        sc.camera = c; sc.render.filepath = str(HERE / fname)
        bpy.ops.render.render(write_still=True, scene=sc.name)
    scene.camera = cam
print('INTERIOR KIT:', len(modules), 'asset collections; open-front demo plus parts shelf')
