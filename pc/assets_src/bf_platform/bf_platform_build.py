# Builds "BF_Platform": a parametric platform-fighter platform.
# Thin Battlefield-style slab, Final Destination-inspired angular hull, emblem and glow lines.
#
# Every feature (bevels, rim, corner cuts, border, core, ribs) has a fixed real-world size.
# Changing Width / Depth, or scaling the object, only lengthens the straight runs.
#
# Re-runnable: rebuilds its own node group, materials and object from scratch each time.

import bpy
import math
import operator

OBJ_NAME = "BF_Platform"
GROUP_NAME = "BF_Platform_Gen"
SQ2 = math.sqrt(2.0)
MITER = 2.0 - SQ2  # how much a 45 degree corner cut shrinks per unit of parallel inset
GAP = 0.1          # spacing between UV islands, in metres


# --------------------------------------------------------------------------------------
# Small expression layer so formulas read like formulas
# --------------------------------------------------------------------------------------
class Tree:
    def __init__(self, nt):
        self.nt = nt
        self.section = "misc"
        self.sections = {}
        self.order = []

    def sec(self, name):
        self.section = name
        if name not in self.sections:
            self.sections[name] = []
            self.order.append(name)

    def node(self, idname, label=None, **props):
        n = self.nt.nodes.new(idname)
        for k, v in props.items():
            setattr(n, k, v)
        if label:
            n.label = label
        if self.section not in self.sections:
            self.sec(self.section)
        self.sections[self.section].append(n)
        return n

    def link(self, a, b):
        return self.nt.links.new(a, b)

    def put(self, sock, val):
        if isinstance(val, F):
            val = val.v
        if isinstance(val, bpy.types.NodeSocket):
            self.link(val, sock)
            return
        if sock.bl_idname == 'NodeSocketBool':
            val = bool(val)
        elif sock.bl_idname.startswith('NodeSocketInt'):
            val = int(round(val))
        sock.default_value = val

    def f(self, v):
        return v if isinstance(v, F) else F(self, v)


class F:
    """A float that is either a constant or a node socket."""
    __slots__ = ("t", "v")

    def __init__(self, t, v):
        self.t = t
        self.v = v.v if isinstance(v, F) else v

    @property
    def const(self):
        return not isinstance(self.v, bpy.types.NodeSocket)

    def _m(self, op, *args, pyfn=None, label=None):
        fs = [self.t.f(a) for a in args]
        if pyfn is not None and all(a.const for a in fs):
            return F(self.t, float(pyfn(*[a.v for a in fs])))
        if len(fs) == 2:
            # skip nodes that would do nothing (x + 0, x * 1, ...)
            a, b = fs
            if op == 'ADD':
                if a.const and a.v == 0.0:
                    return b
                if b.const and b.v == 0.0:
                    return a
            elif op == 'SUBTRACT':
                if b.const and b.v == 0.0:
                    return a
            elif op == 'MULTIPLY':
                for x, y in ((a, b), (b, a)):
                    if x.const and x.v == 0.0:
                        return F(self.t, 0.0)
                    if x.const and x.v == 1.0:
                        return y
            elif op == 'DIVIDE':
                if b.const and b.v == 1.0:
                    return a
                if a.const and a.v == 0.0:
                    return F(self.t, 0.0)
        n = self.t.node('ShaderNodeMath', label=label, operation=op)
        for i, a in enumerate(fs):
            self.t.put(n.inputs[i], a.v)
        return F(self.t, n.outputs[0])

    def __add__(s, o): return s._m('ADD', s, o, pyfn=operator.add)
    def __radd__(s, o): return s._m('ADD', o, s, pyfn=operator.add)
    def __sub__(s, o): return s._m('SUBTRACT', s, o, pyfn=operator.sub)
    def __rsub__(s, o): return s._m('SUBTRACT', o, s, pyfn=operator.sub)
    def __mul__(s, o): return s._m('MULTIPLY', s, o, pyfn=operator.mul)
    def __rmul__(s, o): return s._m('MULTIPLY', o, s, pyfn=operator.mul)
    def __truediv__(s, o): return s._m('DIVIDE', s, o, pyfn=operator.truediv)
    def __rtruediv__(s, o): return s._m('DIVIDE', o, s, pyfn=operator.truediv)
    def __neg__(s): return s._m('MULTIPLY', s, -1.0, pyfn=operator.mul)

    def min(s, o): return s._m('MINIMUM', s, o, pyfn=min)
    def max(s, o): return s._m('MAXIMUM', s, o, pyfn=max)
    def floor(s): return s._m('FLOOR', s, pyfn=math.floor)
    def round(s): return s._m('ROUND', s, pyfn=lambda a: math.floor(a + 0.5))
    def abs(s): return s._m('ABSOLUTE', s, pyfn=abs)
    def sqrt(s): return s._m('SQRT', s, pyfn=math.sqrt)
    def fract(s): return s._m('FRACT', s, pyfn=lambda a: a - math.floor(a))
    def mod(s, o): return s._m('FLOORED_MODULO', s, o, pyfn=lambda a, b: a % b)
    def lt(s, o): return s._m('LESS_THAN', s, o, pyfn=lambda a, b: 1.0 if a < b else 0.0)
    def gt(s, o): return s._m('GREATER_THAN', s, o, pyfn=lambda a, b: 1.0 if a > b else 0.0)
    def atan2(s, o): return s._m('ARCTAN2', s, o, pyfn=math.atan2)

    def named(s, label):
        if not s.const:
            s.v.node.label = label
        return s


def fmax(t, *xs):
    r = t.f(xs[0])
    for x in xs[1:]:
        r = r.max(x)
    return r


def hyp(a, b):
    return (a * a + b * b).sqrt()


def band(x, centre, half_width):
    return (x - centre).abs().lt(half_width)


def between(x, lo, hi):
    return x.gt(lo) * x.lt(hi)


def mix(c, a, b):
    """a when c == 0, b when c == 1."""
    return a + c * (b - a)


def vec(t, x, y, z, label=None):
    n = t.node('ShaderNodeCombineXYZ', label=label)
    t.put(n.inputs[0], x)
    t.put(n.inputs[1], y)
    t.put(n.inputs[2], z)
    return n.outputs[0]


def sep(t, sock):
    n = t.node('ShaderNodeSeparateXYZ')
    t.link(sock, n.inputs[0])
    return F(t, n.outputs[0]), F(t, n.outputs[1]), F(t, n.outputs[2])


def enabled(sockets, name=None):
    return [s for s in sockets if s.enabled and (name is None or s.name == name)][0]


def iswitch(t, index, items, label=None):
    n = t.node('GeometryNodeIndexSwitch', label=label, data_type='FLOAT')
    while len(n.index_switch_items) < len(items):
        n.index_switch_items.new()
    t.put(n.inputs[0], index)
    for i, it in enumerate(items):
        t.put(n.inputs[1 + i], t.f(it))
    return F(t, n.outputs[0])


def switch(t, kind, cond, false, true, label=None):
    n = t.node('GeometryNodeSwitch', label=label, input_type=kind)
    t.put(n.inputs['Switch'], cond)
    if false is not None:
        t.put(n.inputs['False'], false)
    if true is not None:
        t.put(n.inputs['True'], true)
    return n.outputs['Output']


def make_group(name, in_names, out_names, fn):
    """Wrap a repeated formula in its own small node group, so the main tree shows one node per use."""
    old = bpy.data.node_groups.get(name)
    if old:
        bpy.data.node_groups.remove(old)
    g = bpy.data.node_groups.new(name, 'GeometryNodeTree')
    for nm in in_names:
        g.interface.new_socket(nm, in_out='INPUT', socket_type='NodeSocketFloat')
    for nm in out_names:
        g.interface.new_socket(nm, in_out='OUTPUT', socket_type='NodeSocketFloat')
    tt = Tree(g)
    tt.sec(name)
    gi = tt.node('NodeGroupInput')
    go = tt.node('NodeGroupOutput')
    outs = fn(*[F(tt, gi.outputs[nm]) for nm in in_names])
    for nm, o in zip(out_names, outs):
        tt.put(go.inputs[nm], o)
    layout(tt)
    return g


def call(t, group, args, label=None):
    n = t.node('GeometryNodeGroup', label=label)
    n.node_tree = group
    for sock, a in zip(n.inputs, args):
        t.put(sock, a)
    return [F(t, o) for o in n.outputs]


def layout(t, col_w=230, gap=140):
    """Columns by dependency depth, one framed block per section."""
    nt = t.nt
    incoming = {n: [] for n in nt.nodes}
    for l in nt.links:
        incoming[l.to_node].append(l.from_node)
    depth = {}

    def d(n):
        if n in depth:
            return depth[n]
        depth[n] = 0
        ins = incoming[n]
        depth[n] = 0 if not ins else 1 + max(d(m) for m in ins)
        return depth[n]

    import sys
    sys.setrecursionlimit(max(sys.getrecursionlimit(), 5000))
    for n in nt.nodes:
        d(n)
    y_cursor = 0.0
    for name in t.order:
        nodes = t.sections[name]
        if not nodes:
            continue
        frame = nt.nodes.new('NodeFrame')
        frame.label = name
        frame.label_size = 28
        lo = min(depth[n] for n in nodes)
        cols = {}
        for n in nodes:
            cols.setdefault(depth[n] - lo, []).append(n)
        tallest = 0.0
        for ci, col in cols.items():
            y = 0.0
            for n in col:
                vis = sum(1 for s in list(n.inputs) + list(n.outputs) if s.enabled and not s.hide)
                h = 70 + 22 * vis
                n.parent = frame
                n.location = (ci * col_w, y_cursor - y)
                y += h + 30
            tallest = max(tallest, y)
        y_cursor -= tallest + gap


# --------------------------------------------------------------------------------------
# Materials (all driven by attributes written by the node group, so they follow the sliders)
# --------------------------------------------------------------------------------------
CYAN = (0.04, 0.55, 1.0, 1)
VIOLET = (0.42, 0.10, 1.0, 1)
MAGENTA = (1.0, 0.05, 0.50, 1)


def new_material(name):
    m = bpy.data.materials.get(name)
    if m:
        bpy.data.materials.remove(m)
    m = bpy.data.materials.new(name)
    try:
        m.use_nodes = True
    except Exception:
        pass
    nt = m.node_tree
    nt.nodes.clear()
    t = Tree(nt)
    t.sec("Output")
    out = t.node('ShaderNodeOutputMaterial')
    bsdf = t.node('ShaderNodeBsdfPrincipled')
    t.link(bsdf.outputs[0], out.inputs[0])
    return m, t, bsdf


def attr_vec(t, name):
    n = t.node('ShaderNodeAttribute', label=name, attribute_type='GEOMETRY', attribute_name=name)
    return sep(t, n.outputs['Vector'])


def attr_f(t, name):
    n = t.node('ShaderNodeAttribute', label=name, attribute_type='GEOMETRY', attribute_name=name)
    return F(t, n.outputs['Fac'])


def cmix(t, fac, a, b, label=None):
    n = t.node('ShaderNodeMix', label=label, data_type='RGBA')
    t.put(n.inputs[0], fac)
    t.put(n.inputs[6], a)
    t.put(n.inputs[7], b)
    return n.outputs[2]


def seam_lines(v, spacing, half_width):
    fr = (v / spacing).fract()
    return ((0.5 - (fr - 0.5).abs()) * spacing).lt(half_width)


def mat_deck():
    m, t, bsdf = new_material("BF_Deck")
    t.sec("Coordinates in real units")
    px, py, pz = attr_vec(t, "bf_pos")
    hxp, hyp_, fs = attr_vec(t, "bf_dim")
    R = (hyp_ * 0.80).max(0.05).named("Emblem radius")
    ex = px / R
    ey = py / R
    r = (ex * ex + ey * ey).sqrt().named("Radius (emblem units)")
    ang = ey.atan2(ex)

    t.sec("Centre emblem (always round, never stretched)")
    ring1 = band(r, 0.93, 0.035).named("Outer ring")
    ring2 = band(r, 0.62, 0.018).named("Inner ring")
    tk = (ang * (16.0 / (2.0 * math.pi)) + 0.5).fract()
    ticks = (between(r, 0.70, 0.85) * (tk - 0.5).abs().lt(0.10)).named("Radial ticks")
    dia = band(ex.abs() + ey.abs(), 0.45, 0.025).named("Diamond")
    dot = r.lt(0.11).named("Centre dot")
    emblem = fmax(t, ring1, ring2, ticks, dia, dot)

    t.sec("Rails running to each end")
    yr = R * 0.36
    w = 0.016
    xe = hxp - 0.28
    room = xe.gt(R * 1.15)
    rail = band(py.abs(), yr, w) * r.gt(0.965) * px.abs().lt(xe + w) * room
    endbar = band(px.abs(), xe, w) * py.abs().lt(yr + w) * room
    glow = fmax(t, emblem, rail, endbar).named("All glowing lines")

    t.sec("Panel seams every 0.5 m")
    seams = fmax(t, seam_lines(px, 0.5, 0.006), seam_lines(py, 0.5, 0.006)) * r.gt(1.04) * (1.0 - glow)

    t.sec("Shading")
    grad = py.abs() / hyp_.max(0.01)
    base = cmix(t, grad, (0.070, 0.045, 0.200, 1), (0.014, 0.014, 0.060, 1), "Depth gradient")
    base = cmix(t, r.lt(0.93), base, (0.060, 0.020, 0.130, 1), "Emblem disc tint")
    base = cmix(t, seams, base, (0.004, 0.004, 0.012, 1), "Seams")
    em = cmix(t, r.min(1.0), VIOLET, CYAN, "Violet centre to cyan")
    t.link(base, bsdf.inputs['Base Color'])
    bsdf.inputs['Metallic'].default_value = 0.85
    t.put(bsdf.inputs['Roughness'], 0.26 + seams * 0.4)
    t.link(em, bsdf.inputs['Emission Color'])
    t.put(bsdf.inputs['Emission Strength'], glow * 2.2)
    layout(t)
    return m


def mat_trim():
    m, t, bsdf = new_material("BF_Trim")
    t.sec("Brushed steel")
    a = t.node('ShaderNodeAttribute', label="bf_pos", attribute_type='GEOMETRY', attribute_name="bf_pos")
    vm = t.node('ShaderNodeVectorMath', operation='MULTIPLY')
    t.link(a.outputs['Vector'], vm.inputs[0])
    vm.inputs[1].default_value = (0.6, 14.0, 14.0)
    nz = t.node('ShaderNodeTexNoise')
    t.link(vm.outputs[0], nz.inputs['Vector'])
    nz.inputs['Scale'].default_value = 3.0
    nz.inputs['Detail'].default_value = 4.0
    bsdf.inputs['Base Color'].default_value = (0.34, 0.33, 0.52, 1)
    bsdf.inputs['Metallic'].default_value = 1.0
    t.put(bsdf.inputs['Roughness'], 0.28 + F(t, nz.outputs[0]) * 0.14)
    layout(t)
    return m


def mat_rim():
    m, t, bsdf = new_material("BF_Rim")
    t.sec("Rim lights, evenly spaced and centred on every edge")
    u, L, count = attr_vec(t, "bf_edge")
    lvl = attr_f(t, "bf_level")
    # the count is decided once per edge by the node group; deciding it per pixel flickers at rounding boundaries
    n = count.round().max(1.0).named("Lights on this edge")
    spc = (L / n).named("Actual spacing")
    ph = ((u + L * 0.5) / spc).fract()
    dash = ((ph - 0.5).abs() * spc).lt(0.10) * (lvl.fract() - 0.5).abs().lt(0.17) * L.gt(0.3)
    bsdf.inputs['Base Color'].default_value = (0.03, 0.03, 0.07, 1)
    bsdf.inputs['Metallic'].default_value = 0.9
    bsdf.inputs['Roughness'].default_value = 0.4
    bsdf.inputs['Emission Color'].default_value = CYAN
    t.put(bsdf.inputs['Emission Strength'], dash * 2.6)
    layout(t)
    return m


def mat_hull():
    m, t, bsdf = new_material("BF_Hull")
    t.sec("Hull plating")
    px, py, pz = attr_vec(t, "bf_pos")
    lvl = attr_f(t, "bf_level")
    fl = lvl.floor()
    on_slope = fmax(t, band(fl, 6.0, 0.5), band(fl, 12.0, 0.5))
    line = (band(lvl.fract(), 0.5, 0.04) * on_slope).named("Energy line")
    seams = seam_lines(px, 0.8, 0.006) * (1.0 - line)
    base = cmix(t, seams, (0.055, 0.035, 0.105, 1), (0.008, 0.006, 0.016, 1), "Plate seams")
    t.link(base, bsdf.inputs['Base Color'])
    bsdf.inputs['Metallic'].default_value = 0.9
    t.put(bsdf.inputs['Roughness'], 0.42 + seams * 0.3)
    bsdf.inputs['Emission Color'].default_value = MAGENTA
    t.put(bsdf.inputs['Emission Strength'], line * 2.2)
    layout(t)
    return m


def mat_emissive(name, colour, strength):
    m, t, bsdf = new_material(name)
    bsdf.inputs['Base Color'].default_value = (0.0, 0.0, 0.0, 1)
    bsdf.inputs['Roughness'].default_value = 0.5
    bsdf.inputs['Emission Color'].default_value = colour
    bsdf.inputs['Emission Strength'].default_value = strength
    layout(t)
    return m


def build_materials():
    return {
        "deck": mat_deck(),
        "trim": mat_trim(),
        "rim": mat_rim(),
        "hull": mat_hull(),
        "glow": mat_emissive("BF_Glow", CYAN, 2.6),
        "core": mat_emissive("BF_Core", MAGENTA, 3.0),
    }


# --------------------------------------------------------------------------------------
# Geometry node group
# --------------------------------------------------------------------------------------
def build_group(mats):
    ng = bpy.data.node_groups.get(GROUP_NAME)
    if ng:
        bpy.data.node_groups.remove(ng)
    ng = bpy.data.node_groups.new(GROUP_NAME, 'GeometryNodeTree')
    ng.is_modifier = True
    I = ng.interface
    I.new_socket("Geometry", in_out='OUTPUT', socket_type='NodeSocketGeometry')

    def fin(panel, name, default, lo, hi, desc):
        s = I.new_socket(name, in_out='INPUT', socket_type='NodeSocketFloat', parent=panel, description=desc)
        s.subtype = 'DISTANCE'
        s.default_value = default
        s.min_value = lo
        s.max_value = hi

    p = I.new_panel("Size")
    fin(p, "Width", 8.0, 0.5, 200.0, "Overall length along X. Only the straight middle grows; corners and edges keep their size")
    fin(p, "Depth", 2.4, 0.3, 100.0, "Overall depth along Y")
    fin(p, "Thickness", 0.4, 0.05, 20.0, "Top surface down to the bottom plate")
    p = I.new_panel("Profile (fixed real-world sizes)")
    fin(p, "Corner Cut", 0.45, 0.02, 50.0, "Length cut off each corner, seen from above")
    fin(p, "Rim Height", 0.12, 0.01, 10.0, "Height of the vertical edge band that carries the lights")
    fin(p, "Bevel", 0.03, 0.002, 1.0, "Chamfer above and below the rim")
    fin(p, "Border Width", 0.16, 0.01, 10.0, "Width of the raised metal frame around the deck")
    fin(p, "Panel Recess", 0.02, 0.002, 1.0, "How far the deck panel sits below the frame. Its sloped wall is the glowing inlay")
    fin(p, "Underside Ledge", 0.10, 0.005, 10.0, "Flat strip under the rim before the hull slopes away")
    fin(p, "Hull Inset", 0.45, 0.01, 50.0, "How far the bottom plate is set in from the edge. Fixed distance, so the slope angle never changes with size")
    p = I.new_panel("Details")
    fin(p, "Core Size", 0.6, 0.0, 20.0, "Radius of the glowing core under the centre. Limited to suit the depth. 0 removes it")
    fin(p, "Rib Spacing", 0.9, 0.0, 50.0, "Target gap between underside ribs. Longer platforms get more ribs. 0 removes ribs and spine")
    p = I.new_panel("Behaviour")
    s = I.new_socket("Follow Object Scale", in_out='INPUT', socket_type='NodeSocketBool', parent=p,
                     description="Scaling the object resizes the platform without stretching its details")
    s.default_value = True

    # small reusable formulas, each wrapped once and reused
    g_hyp = make_group("BF Slope Length", ["Across", "Down"], ["Length"], lambda a, b: [hyp(a, b)])
    g_corner = make_group(
        "BF Corner Cut At Inset", ["Corner Cut", "Inset", "Smaller Half Size"], ["Corner Cut"],
        lambda cc, inset, mm: [(cc - inset * MITER).max(cc * 0.3).min((mm - inset) * 0.9)])

    def step_fn(da, za, ca, db, zb, cb, pa, pda):
        dd = db - da
        dz = zb - za
        run = (dd * 2.0 + (cb - ca)) / SQ2
        return [pa + hyp(dd, dz), pda + hyp(run, dz)]

    g_step = make_group(
        "BF Profile Step",
        ["Inset A", "Height A", "Corner Cut A", "Inset B", "Height B", "Corner Cut B", "Distance A", "Corner Distance A"],
        ["Distance B", "Corner Distance B"], step_fn)

    t = Tree(ng)
    gi_cache = {}

    def inp(name):
        n = gi_cache.get(t.section)
        if n is None:
            n = t.node('NodeGroupInput', label="Sliders")
            gi_cache[t.section] = n
        return F(t, n.outputs[name])

    def store(geo, name, dtype, domain, value):
        n = t.node('GeometryNodeStoreNamedAttribute', label="Write " + name, data_type=dtype, domain=domain)
        t.link(geo, n.inputs['Geometry'])
        n.inputs['Name'].default_value = name
        t.put(enabled(n.inputs, 'Value'), value)
        return n.outputs[0]

    def named(name, dtype='FLOAT'):
        n = t.node('GeometryNodeInputNamedAttribute', label="Read " + name, data_type=dtype)
        n.inputs['Name'].default_value = name
        return enabled(n.outputs, 'Attribute')

    def on_faces(value, dtype='FLOAT', label="Average over each face"):
        n = t.node('GeometryNodeFieldOnDomain', label=label, data_type=dtype, domain='FACE')
        t.put(enabled(n.inputs), value)
        return enabled(n.outputs)

    def position():
        return sep(t, t.node('GeometryNodeInputPosition').outputs[0])

    def blank_to(geo, mat, label):
        """Primitives arrive with one empty material slot; swap it rather than appending after it."""
        n = t.node('GeometryNodeReplaceMaterial', label=label)
        t.link(geo, n.inputs['Geometry'])
        n.inputs['New'].default_value = mat
        return n.outputs[0]

    def setmat(geo, sel, mat, label):
        n = t.node('GeometryNodeSetMaterial', label=label)
        t.link(geo, n.inputs['Geometry'])
        t.put(n.inputs['Selection'], sel)
        n.inputs['Material'].default_value = mat
        return n.outputs[0]

    def set_pos(mesh, x, y, z, label):
        n = t.node('GeometryNodeSetPosition', label=label)
        t.link(mesh, n.inputs['Geometry'])
        t.link(vec(t, x, y, z), n.inputs['Position'])
        return n.outputs[0]

    def box_uv(x_face, side_face, z_face):
        """Unfold a small part. Ends face X, sides lean between Y and Z, top and bottom face Z.
        Each callback takes a 0/1 'positive side' flag and returns (u, v)."""
        nx, ny, nz = sep(t, on_faces(t.node('GeometryNodeInputNormal').outputs[0], 'FLOAT_VECTOR', "Face normal"))
        isx = nx.abs().gt(0.5).named("Faces X")
        isy = ((1.0 - isx) * ny.abs().gt(0.01)).named("Leaning side")
        isz = ((1.0 - isx) * (1.0 - isy)).named("Faces Z")
        ux, vx = x_face(nx.gt(0.0))
        uy, vy = side_face(ny.gt(0.0))
        uz, vz = z_face(nz.gt(0.0))
        return isx * ux + isy * uy + isz * uz, isx * vx + isy * vy + isz * vz

    # ---------------------------------------------------------------- 1
    t.sec("1  Size, object scale and safety clamps")
    so = t.node('GeometryNodeSelfObject')
    oi = t.node('GeometryNodeObjectInfo', transform_space='ORIGINAL')
    t.link(so.outputs[0], oi.inputs['Object'])
    osx, osy, osz = sep(t, oi.outputs['Scale'])

    def eff_scale(raw, label):
        return F(t, switch(t, 'FLOAT', inp("Follow Object Scale"), 1.0, raw.abs().max(0.001), label))

    sx = eff_scale(osx, "Scale X in use")
    sy = eff_scale(osy, "Scale Y in use")
    sz = eff_scale(osz, "Scale Z in use")
    hx = (inp("Width") * sx * 0.5).named("Half width")
    hy = (inp("Depth") * sy * 0.5).named("Half depth")
    Te = (inp("Thickness") * sz).max(0.01).named("Thickness")
    m = hx.min(hy).named("Smaller half size")

    b0 = inp("Bevel")
    bw0 = inp("Border Width")
    pr0 = inp("Panel Recess")
    rh0 = inp("Rim Height")
    lg0 = inp("Underside Ledge")
    hi0 = inp("Hull Inset")
    dmax0 = (b0 + bw0 + pr0).max(b0 + lg0 + hi0)
    fs = ((m * 0.8) / dmax0).min(1.0).min((Te * 0.85) / (rh0 + b0 * 2.0)).named("Feature scale (1 unless the platform is tiny)")
    b = (b0 * fs).named("Bevel")
    bw = bw0 * fs
    pr = (pr0 * fs).named("Recess")
    rh = rh0 * fs
    lg = lg0 * fs
    hi = hi0 * fs
    gap = fs * GAP
    c = inp("Corner Cut").max(0.02).min(m * 0.9).named("Corner cut")

    d1 = (b + bw).named("Inset: frame inner edge")
    d0 = (d1 + pr).named("Inset: deck panel")
    d6 = (b + lg).named("Inset: ledge inner edge")
    d7 = (d6 + hi).named("Inset: bottom plate")
    zt = (rh + b * 2.0).named("Depth of rim plus bevels")
    zero = F(t, 0.0)
    d = [d0, d1, b, zero, zero, b, d6, d7]
    z = [-pr, zero, zero, -b, -(b + rh), -zt, -zt, -Te]

    corner_cache = {}

    def corner_at(inset, ring):
        if inset.const and inset.v == 0.0:
            return c                      # at the outer edge the corner cut is simply the slider value
        key = inset.v.as_pointer()
        if key not in corner_cache:
            corner_cache[key] = call(t, g_corner, [c, inset, m], "Corner cut, ring %d" % ring)[0]
        return corner_cache[key]

    cl = [corner_at(dv, i) for i, dv in enumerate(d)]

    # distance travelled down the profile: P on straight edges, Pd on the corner cuts
    P = [zero]
    Pd = [zero]
    for i in range(7):
        pb, pdb = call(t, g_step, [d[i], z[i], cl[i], d[i + 1], z[i + 1], cl[i + 1], P[-1], Pd[-1]],
                       "Profile step, ring %d to %d" % (i, i + 1))
        P.append(pb)
        Pd.append(pdb)

    def slope(a, b, label):
        return call(t, g_hyp, [a, b], label)[0]

    # ---------------------------------------------------------------- 1b
    t.sec("1b  Sizes of the core, ribs and spine")
    core_on = inp("Core Size").gt(0.001)
    r1 = (inp("Core Size") * fs).min((hy - d6) * 0.6).min((hx - d7) * 0.6).max(0.01).named("Core radius (kept in proportion on small platforms)")
    t1 = r1 * 0.10
    hc = r1 * 0.50
    rs = inp("Rib Spacing")
    e = fs * 0.05
    th = (fs * 0.09).named("Rib thickness")
    ytop = (hy - d6 + e).named("Rib half width at the ledge")
    ybot = (hy - d7 + e).named("Rib half width at the bottom plate")
    ztop = -(zt - 0.01 * fs)
    zbot = -(Te + e * 0.6)
    H = (ztop - zbot).named("Rib height")
    rib_slant = slope(ytop - ybot, H, "Rib slope length")
    x0 = (core_on * r1 + 0.25).max(rs * 0.5).named("First rib, clear of the core")
    x1 = (hx - d7 - cl[7] - 0.12).named("Last rib, clear of the corner cut")
    span = x1 - x0
    n = (((span / rs.max(0.05)) + 0.5).floor() + 1.0).max(1.0).named("Ribs per side")
    step = (span / (n - 1.0).max(1.0)).named("Actual rib spacing")
    ribs_on = rs.gt(0.01) * span.gt(0.0)
    wt = (fs * 0.18).named("Spine width at the top")
    wb = (fs * 0.08).named("Spine width at the bottom")
    hs_top = -(Te - 0.01 * fs)
    hs_bot = -(Te + fs * 0.06)
    hs = hs_top - hs_bot
    sp_slant = slope((wt - wb) * 0.5, hs, "Spine side length")
    xs = x1.max(0.05).named("Spine half length")
    spine_on = rs.gt(0.01) * x1.gt(0.3)

    # ---------------------------------------------------------------- 1c
    t.sec("1c  UV layout: rows stacked upward, in metres")
    Ld0 = c * SQ2
    Lx0 = (hx - c) * 2.0
    Ly0 = (hy - c) * 2.0
    half_outline = (Ld0 * 2.0 + Lx0 + Ly0).named("Half the outline length")
    Pm = P[7].max(Pd[7]).named("Height of one side strip")
    row_strip_b = (Pm + gap).named("Row 2: second side strip")
    row_deck = (Pm * 2.0 + gap * 2.0).named("Row 3: deck")
    hy7 = hy - d7
    row_plate = (row_deck + hy * 2.0 + gap).named("Row 4: bottom plate")
    row_parts = (row_plate + hy7 * 2.0 + gap).named("Row 5: core and rib")
    cs = (r1 * 4.0).named("Core block size")
    rib_w = ytop * 2.0 + gap * 2.0 + th * 2.0
    rib_h = (H * 2.0 + gap * 3.0 + th * 2.0).max(rib_slant)
    row_spine = (row_parts + cs.max(rib_h) + gap).named("Row 6: spine")
    spine_h = sp_slant * 2.0 + gap * 3.0 + wt * 2.0
    lay_h = row_spine + spine_h
    lay_w = half_outline.max(hx * 2.0).max(cs + gap + rib_w).max(xs * 2.0 + gap + wt)
    lay = lay_w.max(lay_h).named("Layout size")

    # ---------------------------------------------------------------- 2
    t.sec("2  Body: eight rings of eight points, lofted top to bottom")
    cyl = t.node('GeometryNodeMeshCylinder', label="Ring cage (8 x 8)", fill_type='NGON')
    cyl.inputs['Vertices'].default_value = 8
    cyl.inputs['Side Segments'].default_value = 7
    cyl.inputs['Fill Segments'].default_value = 1
    idx = F(t, t.node('GeometryNodeInputIndex').outputs[0])
    j = (idx / 8.0).floor().named("Ring number (0 = deck)")
    k = (idx - j * 8.0).named("Point around the ring")
    dj = iswitch(t, j, d, "Inset of this ring")
    zj = iswitch(t, j, z, "Height of this ring")
    cj = iswitch(t, j, cl, "Corner cut of this ring")
    onx = (k + 1.0).mod(4.0).lt(2.0).named("On an end edge")
    sxk = ((k + 2.0).mod(8.0).lt(4.0) * 2.0 - 1.0).named("Left or right")
    syk = (k.lt(4.0) * 2.0 - 1.0).named("Front or back")
    X = sxk * (hx - dj - (1.0 - onx) * cj)
    Y = syk * (hy - dj - onx * cj)
    g = set_pos(cyl.outputs['Mesh'], X, Y, zj, "Place every point")
    g = store(g, "bf_level", 'FLOAT', 'POINT', j)

    # ---------------------------------------------------------------- 3
    t.sec("3  Body UVs: every face unfolds true to shape")
    uvu, uvv, _ = sep(t, cyl.outputs['UV Map'])
    kc = (uvu * 8.0).round().named("Corner number 0..8")
    lvl = F(t, named("bf_level"))
    d2 = iswitch(t, lvl, d, "Inset of this ring")
    c2 = iswitch(t, lvl, cl, "Corner cut of this ring")
    Ld = (c2 * SQ2).named("Length of a corner cut")
    Lx = ((hx - d2 - c2) * 2.0).named("Length of a long edge")
    Ly = ((hy - d2 - c2) * 2.0).named("Length of an end edge")
    seg = F(t, on_faces(uvu * 8.0)).floor().named("Which edge this face sits on")
    is_diag = (1.0 - seg.mod(2.0)).named("Is a corner cut")
    sm4 = seg.mod(4.0)
    is_lx = band(sm4, 1.0, 0.5).named("Is a long edge")
    is_ly = band(sm4, 3.0, 0.5).named("Is an end edge")
    Lseg = (is_diag * Ld + is_lx * Lx + is_ly * Ly).named("Length of this edge")
    uc = ((kc - seg - 0.5) * Lseg).named("Distance from the edge centre")

    def ge(a, v):
        return a.gt(v - 0.5)

    second = ge(seg, 4).named("Second half of the outline")
    start = ((seg + 1.0) * 0.5).floor() * Ld0 + (ge(seg, 2) + ge(seg, 6)) * Lx0 + second * Ly0
    centre = (start + (is_diag * Ld0 + is_lx * Lx0 + is_ly * Ly0) * 0.5 - second * half_outline).named("Where this edge sits in its strip")
    down = mix(is_diag, iswitch(t, lvl, P, "Distance down the profile"), iswitch(t, lvl, Pd, "Distance down the profile (corner)"))
    U = centre + uc
    V = (Pm - down) + (1.0 - second) * row_strip_b
    px, py, pz = position()
    top_uv = vec(t, px + hx, py + hy + row_deck, 0.0, "Deck: flat")
    bot_uv = vec(t, hx - px, py + hy7 + row_plate, 0.0, "Bottom plate: flat, mirrored so it reads correctly from below")
    cap_uv = switch(t, 'VECTOR', cyl.outputs['Top'], bot_uv, top_uv, "Deck or bottom plate")
    uv = switch(t, 'VECTOR', cyl.outputs['Side'], cap_uv, vec(t, U, V, 0.0, "Sides: along the edge, down the profile"), "Side or cap")
    g = store(g, "UVMap", 'FLOAT2', 'CORNER', uv)
    lights = ((Lseg / 0.6) + 0.5).floor().max(1.0).named("Rim lights on this edge")
    edge = switch(t, 'VECTOR', cyl.outputs['Side'], (0.0, 0.0, 1.0), vec(t, uc, Lseg, lights), "Sides only")
    g = store(g, "bf_edge", 'FLOAT_VECTOR', 'CORNER', edge)

    # ---------------------------------------------------------------- 4
    t.sec("4  Materials by band")
    fv = F(t, on_faces(named("bf_level"))).named("Band number")
    g = blank_to(g, mats['hull'], "Hull everywhere first")
    g = setmat(g, fv.lt(0.25), mats['deck'], "Deck panel")
    g = setmat(g, between(fv, 0.25, 1.0), mats['glow'], "Glowing inlay")
    g = setmat(g, fmax(t, between(fv, 1.0, 3.0), between(fv, 4.0, 5.0)), mats['trim'], "Frame and bevels")
    body = setmat(g, between(fv, 3.0, 4.0), mats['rim'], "Rim")

    # ---------------------------------------------------------------- 5
    t.sec("5  Energy core (fixed size, never stretched)")
    cyc = t.node('GeometryNodeMeshCylinder', label="Core cage (8 x 4)", fill_type='NGON')
    cyc.inputs['Vertices'].default_value = 8
    cyc.inputs['Side Segments'].default_value = 3
    idc = F(t, t.node('GeometryNodeInputIndex').outputs[0])
    jc = (idc / 8.0).floor().named("Core ring number")
    rad = iswitch(t, jc, [r1, r1, r1 * 0.78, r1 * 0.40], "Radius of this ring")
    zc = iswitch(t, jc, [ztop, -(Te + t1), -(Te + t1), -(Te + t1 + hc)], "Height of this ring")
    cx, cy, cz = position()
    gc = set_pos(cyc.outputs['Mesh'], cx * rad, cy * rad, zc, "Shape the core")
    gc = store(gc, "bf_level", 'FLOAT', 'POINT', jc + 10.0)
    cu, cv, _ = sep(t, cyc.outputs['UV Map'])
    gc = store(gc, "UVMap", 'FLOAT2', 'CORNER', vec(t, cu * cs, cv * cs + row_parts, 0.0, "Core block"))
    fc = F(t, on_faces(named("bf_level"))).named("Core band number")
    gc = blank_to(gc, mats['hull'], "Core sides")
    gc = setmat(gc, between(fc, 10.25, 11.0), mats['trim'], "Collar")
    gc = setmat(gc, fc.gt(12.75), mats['core'], "Glowing face")
    turn = t.node('GeometryNodeTransform', label="Turn flats to face the axes")
    t.link(gc, turn.inputs['Geometry'])
    turn.inputs['Rotation'].default_value = (0.0, 0.0, math.radians(22.5))
    core = switch(t, 'GEOMETRY', core_on, None, turn.outputs[0], "Core on or off")

    # ---------------------------------------------------------------- 6
    t.sec("6  Ribs (longer platform = more ribs, spacing stays even)")
    cube = t.node('GeometryNodeMeshCube', label="One rib")
    rx, ry, rz = position()
    top = rz.gt(0.0)
    gr = set_pos(cube.outputs['Mesh'], rx * th, ry * 2.0 * mix(top, ybot, ytop), mix(top, zbot, ztop),
                 "Wrap the rib around the hull")
    gr = store(gr, "bf_level", 'FLOAT', 'POINT', 20.0)
    qx, qy, qz = position()
    ru, rv = box_uv(
        lambda pos: (qy + ytop, (qz - zbot) + pos * (H + gap)),
        lambda pos: (ytop * 2.0 + gap + (qx + th * 0.5) + pos * (th + gap), slope(qy.abs() - ybot, qz - zbot, "Distance up the slope")),
        lambda pos: (qy + ytop, H * 2.0 + gap * 2.0 + (qx + th * 0.5) + pos * (th + gap)),
    )
    gr = store(gr, "UVMap", 'FLOAT2', 'CORNER', vec(t, ru + cs + gap, rv + row_parts, 0.0, "Rib block, right of the core"))
    gr = blank_to(gr, mats['trim'], "Rib metal")
    lines = t.node('GeometryNodeJoinGeometry', label="Both sides")
    for sign, lab in ((1.0, "Right side"), (-1.0, "Left side")):
        ln = t.node('GeometryNodeMeshLine', label=lab)
        t.put(ln.inputs['Count'], n)
        t.link(vec(t, x0 * sign, 0.0, 0.0), ln.inputs['Start Location'])
        t.link(vec(t, step * sign, 0.0, 0.0), ln.inputs['Offset'])
        t.link(ln.outputs[0], lines.inputs[0])
    inst = t.node('GeometryNodeInstanceOnPoints', label="A rib at every point")
    t.link(lines.outputs[0], inst.inputs['Points'])
    t.link(gr, inst.inputs['Instance'])
    real = t.node('GeometryNodeRealizeInstances')
    t.link(inst.outputs[0], real.inputs[0])
    ribs = switch(t, 'GEOMETRY', ribs_on, None, real.outputs[0], "Ribs on or off")

    # ---------------------------------------------------------------- 6b
    t.sec("6b  Spine: one plain bar, so lengthening it stretches nothing")
    bar = t.node('GeometryNodeMeshCube', label="Spine")
    bx, by, bz = position()
    btop = bz.gt(0.0)
    gs = set_pos(bar.outputs['Mesh'], bx * 2.0 * xs, by * mix(btop, wb, wt), mix(btop, hs_bot, hs_top),
                 "Run the spine end to end")
    gs = store(gs, "bf_level", 'FLOAT', 'POINT', 21.0)
    wx, wy, wz = position()
    su, sv = box_uv(
        lambda pos: (xs * 2.0 + gap + (wy + wt * 0.5), (wz - hs_bot) + pos * (hs + gap)),
        lambda pos: (wx + xs, slope(wy.abs() - wb * 0.5, wz - hs_bot, "Distance up the side") + pos * (sp_slant + gap)),
        lambda pos: (wx + xs, (sp_slant + gap) * 2.0 + (wy + wt * 0.5) + pos * (wt + gap)),
    )
    gs = store(gs, "UVMap", 'FLOAT2', 'CORNER', vec(t, su, sv + row_spine, 0.0, "Spine block"))
    gs = blank_to(gs, mats['trim'], "Spine metal")
    spine = switch(t, 'GEOMETRY', spine_on, None, gs, "Spine on or off")

    # ---------------------------------------------------------------- 7
    t.sec("7  Join, bake UVs, tags for the shaders, undo the object scale")
    join = t.node('GeometryNodeJoinGeometry')
    for part in (spine, ribs, core, body):
        t.link(part, join.inputs[0])
    g = store(join.outputs[0], "bf_pos", 'FLOAT_VECTOR', 'POINT', t.node('GeometryNodeInputPosition').outputs[0])
    g = store(g, "bf_dim", 'FLOAT_VECTOR', 'POINT', vec(t, hx - d0, hy - d0, fs, "Deck half size, feature scale"))
    bu, bv, _ = sep(t, named("UVMap", 'FLOAT_VECTOR'))
    g = store(g, "UVBake", 'FLOAT2', 'CORNER', vec(t, bu / lay, bv / lay, 0.0, "Same layout, fitted into 0..1"))
    flat = t.node('GeometryNodeSetShadeSmooth', label="Crisp flat facets")
    t.link(g, flat.inputs[0])
    flat.inputs['Shade Smooth'].default_value = False
    undo = t.node('GeometryNodeTransform', label="Undo object scale so details stay true size")
    t.link(flat.outputs[0], undo.inputs['Geometry'])
    t.link(vec(t, 1.0 / sx, 1.0 / sy, 1.0 / sz), undo.inputs['Scale'])
    out = t.node('NodeGroupOutput')
    t.link(undo.outputs[0], out.inputs[0])

    for n_ in gi_cache.values():
        for o in n_.outputs:
            o.hide = not o.is_linked
    layout(t)
    return ng


# --------------------------------------------------------------------------------------
# Scene
# --------------------------------------------------------------------------------------
def archive_old():
    old = bpy.data.objects.get("Platform")
    if not old:
        return None
    name = "Archive_old_platform_v1"
    col = bpy.data.collections.get(name)
    if not col:
        col = bpy.data.collections.new(name)
        bpy.context.scene.collection.children.link(col)
    for c_ in list(old.users_collection):
        c_.objects.unlink(old)
    col.objects.link(old)
    old.name = "Platform_v1_old"
    col.hide_render = True
    lc = bpy.context.view_layer.layer_collection.children.get(name)
    if lc:
        lc.exclude = True
    return old.name


def build_object(ng):
    keep = None
    ob = bpy.data.objects.get(OBJ_NAME)
    if ob:
        keep = (ob.location.copy(), ob.rotation_euler.copy(), ob.scale.copy())
        me_old = ob.data
        bpy.data.objects.remove(ob)
        if me_old and me_old.users == 0:
            bpy.data.meshes.remove(me_old)
    me = bpy.data.meshes.new(OBJ_NAME)
    ob = bpy.data.objects.new(OBJ_NAME, me)
    target = bpy.data.collections.get("Collection") or bpy.context.scene.collection
    target.objects.link(ob)
    ob.location = (0.0, 0.0, 3.0)
    if keep:
        ob.location, ob.rotation_euler, ob.scale = keep
    md = ob.modifiers.new("BF Platform", 'NODES')
    md.node_group = ng
    for o in bpy.context.view_layer.objects:
        o.select_set(False)
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    return ob


def main():
    archived = archive_old()
    mats = build_materials()
    ng = build_group(mats)
    ob = build_object(ng)
    bpy.context.view_layer.update()
    ev = ob.evaluated_get(bpy.context.evaluated_depsgraph_get()).data
    return {
        "archived_old_object_as": archived,
        "object": ob.name,
        "group_nodes": len(ng.nodes),
        "verts": len(ev.vertices),
        "faces": len(ev.polygons),
        "materials": [mm.name if mm else None for mm in ev.materials],
        "uv_layers": [u.name for u in ev.uv_layers],
        "warnings": [(w.type, w.message) for w in ob.modifiers[0].node_warnings],
    }


result = main()
