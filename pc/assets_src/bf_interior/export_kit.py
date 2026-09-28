"""Export the BF interior kit for the runtime model API (gd.model_load / gd.model_spawn): one
shared atlas, one GXMS v2 mesh per part, and a collision sidecar per part.

    blender --factory-startup --background --python pc/assets_src/bf_interior/export_kit.py --         --output pc/scripts/examples/bf_interior_room/models         --lua pc/scripts/examples/bf_interior_room/scripts/main.lua

bf_interior_playset.py (a copy of the stage kit's generator, E:/Projects/Blender-Stage-Kit)
builds the parts; this bakes them together, so every part samples the same atlas:
    bf_kit.gxtex / bf_kit.glow.gxtex (+ .png previews)   the shared colour and glow atlases
    <model>.gxmesh                                      one per part, in the part's own frame
    <model>.coll.json                                   collision sidecar v1 (docs/scripting.md):
        {"version":1,"atlas":"bf_kit","lines":[[kind,x0,y0,x1,y1,flags]]}
Every part, visual-only ones included, gets a sidecar: it names the shared atlas, which the
runtime loads once for all parts.

Coordinates: Blender X -> game X, Blender Z -> game Y, Blender -Y (the open front) -> game +Z
(towards the camera), in game units (UNIT per metre), origin at the part's origin. Glass is left
out (the model draw is opaque) and so are faces that point away from the camera.
"""

import argparse
import json
import math
from pathlib import Path
import os
import runpy
import struct
import sys

import bpy
import bmesh
from mathutils import Matrix

HERE = Path(__file__).resolve().parent
BF_EXPORT = runpy.run_path(str(HERE.parent / "bf_platform" / "export.py"), run_name="bf_export")
HEADER = struct.Struct(">4sIIIfffII")
KIT_SCALE = 1.3     # the kit's size in game: 1.0 = BF_Platform's 5 units per metre. GD's knob;
                    # the doorway (1.6 x 2.6 m) should read as a door a fighter walks through
UNIT = 5.0 * KIT_SCALE  # game units per Blender metre: meshes, sidecars and the room grid (--lua)
SPACING = 16.0      # metres between parts in the bake scene (no AO crosstalk)
GLOW_GAIN = 0.8
EDGE_MAX = 0.6


def line(x0, z0, x1, z1, passthrough=True, ledges=True, kind="floor"):
    return dict(kind=kind, x0=x0, z0=z0, x1=x1, z1=z1, passthrough=passthrough, ledges=ledges)


# Collision in part metres. Floors are pass-through by default;
# an assembly makes its main floor solid. Slopes have no ledges.
COLLISION = {
    "Floor_4m": [line(-2, 0, 2, 0)],
    "Floor_2m": [line(-1, 0, 1, 0)],
    "Ramp_4m_Rise2m": [line(-2, 0, 2, 2, ledges=False)],
    # through the steps' inner corners: feet sit at most one step (0.25 m) into a tread
    "Stairs_4m_Rise2m": [line(-2, 0, 2, 2, ledges=False)],
    "Balcony_4m": [line(-2, 0, 2, 0)],
    # no ledges at the gap: walking in drops you to the floor below instead of catching the lip
    "Floor_Opening_4m": [line(-2, 0, -1, 0, ledges=False), line(1, 0, 2, 0, ledges=False)],
}


def options():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--output", type=Path, required=True, help="models/ directory")
    p.add_argument("--lua", type=Path, help="room script whose 'local U = ...' line is set to UNIT")
    p.add_argument("--texture-size", type=int, default=1024)
    p.add_argument("--samples", type=int, default=48)
    a = p.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else [])
    if a.texture_size < 256 or a.texture_size > 4096 or a.texture_size & (a.texture_size - 1):
        p.error("texture-size must be a power of two, from 256 to 4096")
    return a


def model_name(part):
    return "bf_" + part.lower()


# ------------------------------------------------------------------ bake scene
def build_parts():
    os.environ["BF_NO_RENDER"] = "1"
    os.environ["BF_NO_SAVE"] = "1"
    runpy.run_path(str(HERE / "bf_interior_playset.py"))
    return [(c.name, c) for c in bpy.data.collections if "module_size" in c]


def combined(parts):
    scene = bpy.context.scene
    for ob in list(scene.collection.all_objects):
        ob.hide_render = True
    objs = []
    for i, (name, col) in enumerate(parts):
        for ob in col.objects:
            if ob.type != "MESH":
                continue
            me = ob.data.copy()
            bm = bmesh.new()
            bm.from_mesh(me)
            kill = [f for f in bm.faces
                    if "transparent" in me.materials[f.material_index].name or f.normal.y > 0.7]
            bmesh.ops.delete(bm, geom=kill, context="FACES")
            bm.to_mesh(me)
            bm.free()
            if not me.polygons:
                continue
            pid = me.attributes.new("part_id", "INT", "FACE")
            for d in pid.data:
                d.value = i
            me.transform(Matrix.Translation((i * SPACING, 0, 0)))  # bf_pos keeps part coordinates
            o = bpy.data.objects.new(name + " bake", me)
            scene.collection.objects.link(o)
            objs.append(o)
    bpy.ops.object.select_all(action="DESELECT")
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.join()
    ob = bpy.context.view_layer.objects.active
    ob.name = "BF_Kit_Bake"
    ob.hide_render = False
    return ob


def unwrap(ob, size):
    me = ob.data
    uv = me.uv_layers.new(name="UVBake")
    me.uv_layers.active = uv
    uv.active_render = True
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=1.15, island_margin=6 / size,
                             margin_method="FRACTION", correct_aspect=True)
    bpy.ops.object.mode_set(mode="OBJECT")


# ------------------------------------------------------------------ bake
def route_material(mat, image, glow):
    """BF_Platform's bake routing (export.py), minus its single-object assumptions: no radial
    falloff on BF_Core and no hull height gradient, which would be measured across the bake scene."""
    nodes, links = mat.node_tree.nodes, mat.node_tree.links
    bsdf = next(n for n in nodes if n.type == "BSDF_PRINCIPLED")
    output = next(n for n in nodes if n.type == "OUTPUT_MATERIAL")
    target = nodes.new("ShaderNodeTexImage")
    target.image = image
    for n in nodes:
        n.select = False
    target.select = True
    nodes.active = target

    def math_(op, a, b=0):
        n = nodes.new("ShaderNodeMath")
        n.operation = op
        for socket, value in zip(n.inputs, (a, b)):
            if isinstance(value, (int, float)):
                socket.default_value = value
            else:
                links.new(value, socket)
        return n.outputs[0]

    def value(socket):
        if socket.is_linked:
            return socket.links[0].from_socket
        n = nodes.new("ShaderNodeRGB")
        n.outputs[0].default_value = socket.default_value
        return n.outputs[0]

    def scale(colour, factor):
        n = nodes.new("ShaderNodeVectorMath")
        n.operation = "SCALE"
        links.new(colour, n.inputs[0])
        if isinstance(factor, (int, float)):
            n.inputs[3].default_value = factor
        else:
            links.new(factor, n.inputs[3])
        return n.outputs[0]

    geom = nodes.new("ShaderNodeNewGeometry")
    if glow:
        strength = bsdf.inputs["Emission Strength"]
        energy = strength.links[0].from_socket if strength.is_linked else strength.default_value
        colour = scale(scale(value(bsdf.inputs["Emission Color"]), energy), GLOW_GAIN)
    else:
        colour = value(bsdf.inputs["Base Color"])
        ao = nodes.new("ShaderNodeAmbientOcclusion")
        ao.inputs["Distance"].default_value = 0.28
        ao.samples = 32
        colour = scale(colour, math_("ADD", 0.38, math_("MULTIPLY", ao.outputs["AO"], 0.62)))
        xyz = nodes.new("ShaderNodeSeparateXYZ")
        links.new(geom.outputs["Normal"], xyz.inputs[0])
        colour = scale(colour, math_("ADD", 0.77, math_("MULTIPLY", xyz.outputs[2], 0.23)))
        if mat.name == "BF_Trim":
            noise = next(n for n in nodes if n.type == "TEX_NOISE")
            colour = scale(colour, math_("ADD", 0.7, math_("MULTIPLY", noise.outputs["Fac"], 0.6)))
        bevel = nodes.new("ShaderNodeBevel")
        bevel.inputs["Radius"].default_value = 0.022
        bevel.samples = 8
        dot = nodes.new("ShaderNodeVectorMath")
        dot.operation = "DOT_PRODUCT"
        links.new(bevel.outputs["Normal"], dot.inputs[0])
        links.new(geom.outputs["Normal"], dot.inputs[1])
        edge = math_("MINIMUM", math_("MULTIPLY", math_("SUBTRACT", 1, dot.outputs["Value"]), 10), EDGE_MAX)
        mix = nodes.new("ShaderNodeMix")
        mix.data_type = "RGBA"
        links.new(edge, mix.inputs["Factor"])
        links.new(colour, mix.inputs[6])
        mix.inputs[7].default_value = (0.86, 0.88, 0.95, 1.0)
        colour = mix.outputs[2]
    emission = nodes.new("ShaderNodeEmission")
    links.new(colour, emission.inputs["Color"])
    links.new(emission.outputs[0], output.inputs["Surface"])


def bake(ob, size, glow, samples):
    from array import array
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    sc.cycles.device = "CPU"
    sc.cycles.samples = samples
    sc.cycles.seed = 19
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    image = bpy.data.images.new("BF_KitGlow" if glow else "BF_KitColor",
                                width=size, height=size, alpha=True, float_buffer=True)
    for mat in ob.data.materials:
        route_material(mat, image, glow)
    bpy.ops.object.bake(type="EMIT", margin=6, use_clear=True)
    pixels = array("f", [0.0]) * (size * size * 4)
    image.pixels.foreach_get(pixels)
    return pixels


# ------------------------------------------------------------------ per-part output
def write_part(path, me, part_index, offset_x, size_m):
    uv = me.uv_layers["UVBake"].data
    pid = me.attributes["part_id"].data
    ids, verts, idx = {}, [], []
    for tri in me.loop_triangles:
        if pid[tri.polygon_index].value != part_index:
            continue
        for loop_id in tri.loops:
            co = me.vertices[me.loops[loop_id].vertex_index].co
            n = me.corner_normals[loop_id].vector
            t = uv[loop_id].uv
            key = ((co.x - offset_x) * UNIT, co.z * UNIT, -co.y * UNIT, t.x, 1.0 - t.y, n.x, n.z, -n.y)
            if key not in ids:
                ids[key] = len(verts)
                verts.append(key)
            idx.append(ids[key])
    if not verts or len(verts) > 65535 or len(idx) > 65535:
        raise RuntimeError(f"{path.name}: {len(verts)} vertices, {len(idx)} indices (limit 65535)")
    width = size_m[0] * UNIT
    voff = HEADER.size
    ioff = voff + len(verts) * 32
    with path.open("wb") as f:
        f.write(HEADER.pack(b"GXMS", 2, len(verts), len(idx), width,
                            max(size_m[1], .05) * UNIT, max(size_m[2], .05) * UNIT, voff, ioff))
        for v in verts:
            f.write(struct.pack(">8f", *v))
        for i in idx:
            f.write(struct.pack(">H", i))
    return len(verts), len(idx) // 3


def sidecar(lines):
    """Collision sidecar v1: [kind, x0, y0, x1, y1, flags] in part-relative game units.
    Floor flags: 1 pass-through, 2 ledges (a spawn can override them with floor_flags)."""
    out = []
    for l in lines:
        flags = (1 if l["passthrough"] else 0) | (2 if l["ledges"] else 0) if l["kind"] == "floor" else 0
        out.append([l["kind"], l["x0"] * UNIT, l["z0"] * UNIT, l["x1"] * UNIT, l["z1"] * UNIT, flags])
    return {"version": 1, "atlas": "bf_kit", "lines": out}


def main():
    a = options()
    a.output.mkdir(parents=True, exist_ok=True)
    parts = build_parts()
    ob = combined(parts)
    unwrap(ob, a.texture_size)
    me = ob.data
    me.calc_loop_triangles()
    X = BF_EXPORT
    base = bake(ob, a.texture_size, False, a.samples)
    glow = X["soften_glow"](bake(ob, a.texture_size, True, a.samples), a.texture_size)
    colour_rgba = X["atlas_bytes"](base, glow, a.texture_size, False)
    glow_rgba = X["atlas_bytes"](base, glow, a.texture_size, True)
    stem = a.output / "bf_kit"
    X["write_png"](stem.with_suffix(".png"), colour_rgba, a.texture_size)
    X["write_png"](Path(str(stem) + ".glow.png"), glow_rgba, a.texture_size)
    X["write_gxtex"](stem.with_suffix(".gxtex"), colour_rgba, a.texture_size)
    X["write_gxtex"](Path(str(stem) + ".glow.gxtex"), glow_rgba, a.texture_size)
    me.calc_loop_triangles()
    for i, (part, col) in enumerate(parts):
        size_m = tuple(col["module_size"])
        lines = COLLISION.get(part, [])
        name = model_name(part)
        if not any(pid.value == i for pid in me.attributes["part_id"].data):
            print(f"  {name}: skipped (all glass; the model draw is opaque)")
            continue
        nv, nt = write_part(a.output / (name + ".gxmesh"), me, i, i * SPACING, size_m)
        (a.output / (name + ".coll.json")).write_text(
            json.dumps(sidecar(lines), separators=(",", ":")) + "\n", encoding="utf-8")
        print(f"  {name}: {nv} vertices, {nt} triangles, {len(lines)} line(s)")
    if a.lua:
        import re
        src = a.lua.read_text(encoding="utf-8")
        src, n = re.subn(r"^local U = [^\n]*", "local U = %g  -- game units per kit metre (export_kit.py KIT_SCALE %g; generated)"
                         % (UNIT, KIT_SCALE), src, count=1, flags=re.M)
        if n != 1:
            raise RuntimeError(f"{a.lua}: no 'local U = ' line to set")
        a.lua.write_text(src, encoding="utf-8")
    print(f"BF kit export: {len(parts)} parts, {a.texture_size}px shared atlas -> {a.output}")


if __name__ == "__main__":
    main()
