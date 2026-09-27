# Measures the evaluated BF_Platform against what the sliders promise.
# Temporarily changes slider values / object scale, then restores them.

import bpy
import bmesh
import math
import time
from collections import Counter
from mathutils import Vector

OBJ = "BF_Platform"


def input_ids(ob):
    md = ob.modifiers[0]
    return {it.name: it.identifier for it in md.node_group.interface.items_tree
            if it.item_type == 'SOCKET' and it.in_out == 'INPUT'}


def get_inputs(ob):
    md = ob.modifiers[0]
    return {n: getattr(md.properties.inputs, i).value for n, i in input_ids(ob).items()}


def set_inputs(ob, **kw):
    md = ob.modifiers[0]
    ids = input_ids(ob)
    for name, val in kw.items():
        getattr(md.properties.inputs, ids[name.replace("_", " ")]).value = val
    ob.update_tag()


def r(v, n=4):
    return round(float(v), n)


def tri_overlap_pixels(faces_uv, res=384):
    """Rasterise UV triangles; return (covered texels, texels covered by more than one face)."""
    grid = {}
    for fi, poly in enumerate(faces_uv):
        p0 = poly[0]
        for a, b in zip(poly[1:-1], poly[2:]):
            tri = (p0, a, b)
            minx = max(0, int(math.floor(min(p[0] for p in tri) * res)))
            maxx = min(res - 1, int(math.floor(max(p[0] for p in tri) * res)))
            miny = max(0, int(math.floor(min(p[1] for p in tri) * res)))
            maxy = min(res - 1, int(math.floor(max(p[1] for p in tri) * res)))
            (x1, y1), (x2, y2), (x3, y3) = tri
            den = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3)
            if abs(den) < 1e-14:
                continue
            for iy in range(miny, maxy + 1):
                py = (iy + 0.5) / res
                for ix in range(minx, maxx + 1):
                    px = (ix + 0.5) / res
                    l1 = ((y2 - y3) * (px - x3) + (x3 - x2) * (py - y3)) / den
                    l2 = ((y3 - y1) * (px - x3) + (x1 - x3) * (py - y3)) / den
                    l3 = 1.0 - l1 - l2
                    eps = 1e-4   # strictly inside, so faces that merely share an edge do not count
                    if l1 > eps and l2 > eps and l3 > eps:
                        grid.setdefault((ix, iy), set()).add(fi)
    return len(grid), sum(1 for s in grid.values() if len(s) > 1)


def measure(ob, deep=False):
    t0 = time.perf_counter()
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    ev_ob = ob.evaluated_get(dg)
    me = ev_ob.data
    eval_s = time.perf_counter() - t0
    mw = ob.matrix_world.copy()
    origin = mw.translation.copy()
    out = {"update_seconds": r(eval_s, 4)}
    try:
        out["modifier_ms"] = r(ob.modifiers[0].execution_time * 1000.0, 3)
    except Exception:
        pass
    out["slots"] = [m.name if m else None for m in me.materials]
    out["slots_used"] = sorted(set(p.material_index for p in me.polygons))
    out["uv_layers"] = [u.name for u in me.uv_layers]
    out["counts"] = {"v": len(me.vertices), "f": len(me.polygons),
                     "tris": sum(len(p.vertices) - 2 for p in me.polygons)}
    out["face_sides"] = dict(Counter(len(p.vertices) for p in me.polygons))
    lvl = [d.value for d in me.attributes["bf_level"].data]
    W = [mw @ v.co - origin for v in me.vertices]
    bfp = [Vector(d.vector) for d in me.attributes["bf_pos"].data]
    out["shader_coords_vs_world_max_error"] = r(max((a - b).length for a, b in zip(W, bfp)), 6)

    bm = bmesh.new()
    bm.from_mesh(me)
    bm.verts.ensure_lookup_table()
    bm.faces.ensure_lookup_table()
    out["non_manifold_edges"] = sum(1 for e in bm.edges if not e.is_manifold)
    out["loose_verts"] = sum(1 for v in bm.verts if not v.link_edges)
    el = [(W[e.verts[0].index] - W[e.verts[1].index]).length for e in bm.edges]
    out["edge_len_min"] = r(min(el), 5)
    out["zero_len_edges"] = sum(1 for l in el if l < 1e-5)

    def poly_area_normal(idxs):
        n = Vector((0, 0, 0))
        p0 = W[idxs[0]]
        for a, b in zip(idxs[1:-1], idxs[2:]):
            n += (W[a] - p0).cross(W[b] - p0)
        return n.length * 0.5, (n.normalized() if n.length > 1e-12 else n)

    areas = []
    max_dev = 0.0
    for f in bm.faces:
        idxs = [v.index for v in f.verts]
        a, n = poly_area_normal(idxs)
        areas.append(a)
        c = sum((W[i] for i in idxs), Vector((0, 0, 0))) / len(idxs)
        for i in idxs:
            max_dev = max(max_dev, abs((W[i] - c).dot(n)))
    out["zero_area_faces"] = sum(1 for a in areas if a < 1e-9)
    out["max_out_of_plane"] = r(max_dev, 7)

    seen = set()
    islands = []
    for v in bm.verts:
        if v.index in seen:
            continue
        stack = [v]
        seen.add(v.index)
        comp = set()
        while stack:
            x = stack.pop()
            comp.add(x.index)
            for e in x.link_edges:
                o = e.other_vert(x)
                if o.index not in seen:
                    seen.add(o.index)
                    stack.append(o)
        islands.append(comp)
    island_of = {}
    for ii, comp in enumerate(islands):
        for vi in comp:
            island_of[vi] = ii
    vols = [0.0] * len(islands)
    for f in bm.faces:
        idxs = [v.index for v in f.verts]
        p0 = W[idxs[0]]
        for a, b in zip(idxs[1:-1], idxs[2:]):
            vols[island_of[idxs[0]]] += p0.dot(W[a].cross(W[b])) / 6.0
    out["islands"] = len(islands)
    out["inside_out_islands"] = sum(1 for v in vols if v <= 0)

    rings = {}
    for i, l in enumerate(lvl):
        if l < 8:
            rings.setdefault(int(round(l)), []).append(W[i])
    body = [p for ps in rings.values() for p in ps]
    out["body_size"] = [r(max(p.x for p in body) - min(p.x for p in body)),
                        r(max(p.y for p in body) - min(p.y for p in body)),
                        r(max(p.z for p in body) - min(p.z for p in body))]
    out["top_z"] = r(max(p.z for p in body))

    def ring_info(j):
        ps = rings[j]
        hx = max(p.x for p in ps)
        hy = max(p.y for p in ps)
        far_y = [p for p in ps if abs(p.y - hy) < 1e-5 and p.x > 0]
        cut = hx - max(p.x for p in far_y) if far_y else None
        return hx, hy, ps[0].z, cut

    info = {j: ring_info(j) for j in sorted(rings)}
    out["features"] = {
        "rim_height": r(info[3][2] - info[4][2]),
        "bevel": r(info[3][0] - info[2][0]),
        "border_width": r(info[2][0] - info[1][0]),
        "panel_recess": r(info[1][2] - info[0][2]),
        "corner_cut": r(info[3][3]),
        "hull_inset": r(info[6][0] - info[7][0]),
        "hull_slope_deg": r(math.degrees(math.atan2(info[6][2] - info[7][2], info[6][0] - info[7][0])), 2),
    }

    core = [W[i] for i, l in enumerate(lvl) if 9.5 < l < 19.5]
    out["core"] = ({"radius": r(max(math.hypot(p.x, p.y) for p in core)),
                    "top_z": r(max(p.z for p in core)), "bottom_z": r(min(p.z for p in core))} if core else None)
    ribv = [W[i] for i, l in enumerate(lvl) if 19.5 < l < 20.5]
    if ribv:
        xs = sorted(set(round(p.x, 4) for p in ribv))
        centres = sorted(set(round((a + b) / 2, 4) for a, b in zip(xs[0::2], xs[1::2])))
        gaps = sorted(set(r(b - a, 3) for a, b in zip(centres, centres[1:])))
        out["ribs"] = {"count": len(centres), "thickness": r(xs[1] - xs[0]), "distinct_gaps": gaps,
                       "outermost": r(max(abs(c) for c in centres))}
    else:
        out["ribs"] = None
    spv = [W[i] for i, l in enumerate(lvl) if l > 20.5]
    out["spine"] = ({"length": r(max(p.x for p in spv) - min(p.x for p in spv)),
                     "width_top": r(max(p.y for p in spv) - min(p.y for p in spv)),
                     "depth": r(max(p.z for p in spv) - min(p.z for p in spv))} if spv else None)

    # ---- UVs: does every body face unfold true to shape?
    uv = me.uv_layers["UVMap"].data
    worst = 1.0
    worst_all = 1.0
    for p in me.polygons:
        li = list(p.loop_indices)
        vi = list(p.vertices)
        is_body = max(lvl[v] for v in vi) < 8
        is_core = 9.5 < lvl[vi[0]] < 19.5
        for a in range(len(li)):
            b = (a + 1) % len(li)
            wl = (W[vi[a]] - W[vi[b]]).length
            ul = (Vector(uv[li[a]].uv) - Vector(uv[li[b]].uv)).length
            if wl < 1e-6:
                continue
            ratio = (ul / wl) if ul > 1e-9 else float("inf")
            stretch = max(ratio, 1.0 / ratio) if ratio not in (0.0, float("inf")) else float("inf")
            if is_body:
                worst = max(worst, stretch)
            if not is_core:
                worst_all = max(worst_all, stretch)
    out["uv_stretch_body"] = r(worst, 4) if worst != float("inf") else "inf"
    out["uv_stretch_body_ribs_spine"] = r(worst_all, 4) if worst_all != float("inf") else "inf"

    bk = me.uv_layers["UVBake"].data
    us = [d.uv[0] for d in bk]
    vs = [d.uv[1] for d in bk]
    out["uvbake_range"] = {"u": [r(min(us)), r(max(us))], "v": [r(min(vs)), r(max(vs))]}
    if deep:
        first_rib = None
        faces_uv = []
        for p in me.polygons:
            vi = list(p.vertices)
            l0 = lvl[vi[0]]
            if 19.5 < l0 < 20.5:
                isl = island_of[vi[0]]
                if first_rib is None:
                    first_rib = isl
                if isl != first_rib:
                    continue   # ribs are identical copies and share one UV block by design
            faces_uv.append([tuple(bk[li].uv) for li in p.loop_indices])
        covered, overlapped = tri_overlap_pixels(faces_uv)
        out["uvbake_texels_covered"] = covered
        out["uvbake_texels_overlapping"] = overlapped
        out["uvbake_fill_percent"] = r(100.0 * covered / (384 * 384), 1)
    bm.free()
    return out


def run():
    ob = bpy.data.objects[OBJ]
    saved = get_inputs(ob)
    saved_scale = ob.scale.copy()
    res = {}
    try:
        res["A default"] = measure(ob, deep=True)
        set_inputs(ob, Width=20.0)
        res["B slider: width 20"] = measure(ob, deep=True)
        set_inputs(ob, Width=40.0, Depth=6.0)
        res["C slider: 40 x 6"] = measure(ob)
        set_inputs(ob, Width=8.0, Depth=2.4, Thickness=1.5)
        res["D slider: thickness 1.5"] = measure(ob, deep=True)
        set_inputs(ob, Width=3.0, Depth=1.2, Thickness=0.25)
        res["E slider: small 3 x 1.2 x 0.25"] = measure(ob, deep=True)
        set_inputs(ob, Width=1.0, Depth=0.5, Thickness=0.1)
        res["F slider: tiny 1 x 0.5 x 0.1"] = measure(ob, deep=True)
        set_inputs(ob, Width=8.0, Depth=2.4, Thickness=0.4)
        ob.scale = (2.5, 1.0, 1.0)
        res["G object scaled 2.5x in X (S key)"] = measure(ob)
        ob.scale = (1.0, 2.0, 3.0)
        res["H object scaled Y 2x, Z 3x"] = measure(ob)
        ob.scale = (1.0, 1.0, 1.0)
        set_inputs(ob, Corner_Cut=0.05)
        res["I nearly square corners (cut 0.05)"] = measure(ob, deep=True)
        set_inputs(ob, Corner_Cut=0.45, Core_Size=0.0, Rib_Spacing=0.0)
        res["J core and ribs switched off"] = measure(ob, deep=True)
    finally:
        for n, v in saved.items():
            set_inputs(ob, **{n: v})
        ob.scale = saved_scale
        bpy.context.view_layer.update()
    res["restored"] = {"inputs": {k: (r(v) if isinstance(v, float) else v) for k, v in get_inputs(ob).items()},
                       "scale": [r(s) for s in ob.scale]}
    return res


result = run()
