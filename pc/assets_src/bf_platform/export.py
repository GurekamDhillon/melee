"""Bake BF_Platform for gd.stage_add_platform's optional model.

    blender --background --python pc/assets_src/bf_platform/export.py -- \
        --width 8 --depth 2.4 --thickness 0.4 \
        --output pc/scripts/examples/bf_platform/models/bf_platform

Writes <output>.gxmesh, <output>.gxtex, <output>.glow.gxtex and PNG previews.
The mesh is original geometry, with Blender X/Z/Y mapped to game X/Y/Z.  One Blender
metre defaults to five game world units; the renderer scales only X to the collision
line's width. Cycles bakes procedural colour, AO and edge detail separately from
emission. GXMS v2 carries corner normals; GXTX includes mips down to 64 pixels.
"""

import argparse
from array import array
from pathlib import Path
import runpy
import struct
import sys
import zlib

import numpy as np

import bpy


HERE = Path(__file__).resolve().parent
HEADER = struct.Struct(">4sIIIfffII")
GLOW_GAIN = 0.8   # emissive strength baked into the glow atlas (was 0.42: lines too dim at play distance)
RIM_GAIN = 1.35   # the camera-facing rim band
EDGE_MAX = 0.6    # edge-wear highlight, as a mix towards bright metal  # GXMS, version, vertex/index counts, dimensions, offsets


def options():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--width", type=float, default=8.0, help="Blender metres along X")
    p.add_argument("--depth", type=float, default=2.4, help="Blender metres along Y")
    p.add_argument("--thickness", type=float, default=0.4, help="Blender metres below the top")
    p.add_argument("--units-per-meter", type=float, default=5.0)
    p.add_argument("--texture-size", type=int, default=512)
    p.add_argument("--output", type=Path, required=True, help="path stem, without extension")
    a = p.parse_args(sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else [])
    if not (0.5 <= a.width <= 200 and 0.3 <= a.depth <= 100 and
            0.05 <= a.thickness <= 20 and 0 < a.units_per_meter < 100000):
        p.error("dimensions must stay within BF_Platform's slider ranges and units-per-meter must be positive")
    if a.texture_size < 64 or a.texture_size > 4096 or a.texture_size & (a.texture_size - 1):
        p.error("texture-size must be a power of two, from 64 to 4096")
    return a


def set_dimensions(ob, a):
    mod = ob.modifiers[0]
    ids = {it.name: it.identifier for it in mod.node_group.interface.items_tree
           if it.item_type == "SOCKET" and it.in_out == "INPUT"}
    for name, value in (("Width", a.width), ("Depth", a.depth), ("Thickness", a.thickness)):
        getattr(mod.properties.inputs, ids[name]).value = value
    ob.location = (0, 0, 0)
    ob.rotation_euler = (0, 0, 0)
    ob.scale = (1, 1, 1)
    ob.update_tag()
    bpy.context.view_layer.update()


def evaluated_mesh(ob):
    depsgraph = bpy.context.evaluated_depsgraph_get()
    mesh = bpy.data.meshes.new_from_object(ob.evaluated_get(depsgraph),
                                           preserve_all_data_layers=True, depsgraph=depsgraph)
    uv = mesh.uv_layers.get("UVBake")
    if uv is None or len(mesh.materials) != 6:
        raise RuntimeError("BF_Platform must evaluate to six materials and a UVBake layer")
    mesh.uv_layers.active = uv
    uv.active_render = True
    mesh.calc_loop_triangles()
    return mesh


def write_mesh(path, mesh, a):
    uv = mesh.uv_layers["UVBake"].data
    vertex_ids = {}
    vertices = []
    indices = []
    unit = a.units_per_meter
    for tri in mesh.loop_triangles:
        for loop_id in tri.loops:
            loop = mesh.loops[loop_id]
            co = mesh.vertices[loop.vertex_index].co
            tex = uv[loop_id].uv
            normal = mesh.corner_normals[loop_id].vector
            # GX's image row zero is at the top; Blender UV V=0 is at the bottom.
            key = (float(co.x * unit), float(co.z * unit), float(co.y * unit),
                   float(tex.x), float(1.0 - tex.y),
                   float(normal.x), float(normal.z), float(normal.y))
            if key not in vertex_ids:
                vertex_ids[key] = len(vertices)
                vertices.append(key)
            indices.append(vertex_ids[key])
    if not vertices or len(vertices) > 65535 or len(indices) > 65535:
        raise RuntimeError("mesh must have 1..65535 vertices and 1..65535 triangle indices")
    voff = HEADER.size
    ioff = voff + len(vertices) * 32
    with path.open("wb") as f:
        f.write(HEADER.pack(b"GXMS", 2, len(vertices), len(indices),
                            a.width * unit, a.depth * unit, a.thickness * unit, voff, ioff))
        for v in vertices:
            f.write(struct.pack(">8f", *v))
        for idx in indices:
            f.write(struct.pack(">H", idx))
    return len(vertices), len(indices) // 3


def bake_object(mesh):
    ob = bpy.data.objects.new("BF_Platform_Bake", mesh)
    bpy.context.scene.collection.objects.link(ob)
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.context.view_layer.objects.active = ob
    bpy.context.scene.render.engine = "CYCLES"
    bpy.context.scene.cycles.device = "CPU"
    bpy.context.scene.cycles.samples = 48
    bpy.context.scene.cycles.seed = 19
    return ob


def repack_uv(mesh, size):
    """Reserve the upper 42% for the deck; pack other faces into the lower band.

    Only the evaluated bake mesh is edited. GD's generator remains authoritative.
    Eight-pixel gutters protect the first few mip levels from neighbouring islands.
    """
    uv = mesh.uv_layers['UVBake'].data
    deck = [p for p in mesh.polygons if mesh.materials[p.material_index].name == 'BF_Deck']
    for p in mesh.polygons:
        p.select = p not in deck
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.uv.smart_project(angle_limit=1.15, island_margin=8 / size,
                             margin_method='FRACTION', correct_aspect=True)
    bpy.ops.object.mode_set(mode='OBJECT')
    uv = mesh.uv_layers['UVBake'].data
    deck = [p for p in mesh.polygons if mesh.materials[p.material_index].name == 'BF_Deck']
    for p in mesh.polygons:
        if p not in deck:
            for i in p.loop_indices:
                uv[i].uv.y = 0.015 + uv[i].uv.y * 0.53
    points = [mesh.vertices[mesh.loops[i].vertex_index].co for p in deck for i in p.loop_indices]
    xmin, xmax = min(p.x for p in points), max(p.x for p in points)
    ymin, ymax = min(p.y for p in points), max(p.y for p in points)
    for p in deck:
        for i in p.loop_indices:
            co = mesh.vertices[mesh.loops[i].vertex_index].co
            uv[i].uv = (0.025 + 0.95 * (co.x-xmin)/(xmax-xmin),
                        0.585 + 0.39 * (co.y-ymin)/(ymax-ymin))
    mesh.calc_loop_triangles()


def route_material(mat, image, glow):
    nodes, links = mat.node_tree.nodes, mat.node_tree.links
    bsdf = next(n for n in nodes if n.type == 'BSDF_PRINCIPLED')
    output = next(n for n in nodes if n.type == 'OUTPUT_MATERIAL')
    target = nodes.new('ShaderNodeTexImage')
    target.image = image
    for n in nodes:
        n.select = False
    target.select = True
    nodes.active = target

    def math(op, a, b=0):
        n = nodes.new('ShaderNodeMath')
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
        n = nodes.new('ShaderNodeRGB')
        n.outputs[0].default_value = socket.default_value
        return n.outputs[0]

    def scale(colour, factor):
        n = nodes.new('ShaderNodeVectorMath')
        n.operation = 'SCALE'
        links.new(colour, n.inputs[0])
        if isinstance(factor, (int, float)):
            n.inputs[3].default_value = factor
        else:
            links.new(factor, n.inputs[3])
        return n.outputs[0]

    geom = nodes.new('ShaderNodeNewGeometry')
    if glow:
        strength = bsdf.inputs['Emission Strength']
        energy = strength.links[0].from_socket if strength.is_linked else strength.default_value
        colour = scale(value(bsdf.inputs['Emission Color']), energy)
        if mat.name == 'BF_Core':
            # Radial falloff on the core, in original object metres (not atlas space).
            xyz = nodes.new('ShaderNodeSeparateXYZ')
            links.new(geom.outputs['Position'], xyz.inputs[0])
            radius = math('SQRT', math('ADD', math('MULTIPLY', xyz.outputs[0], xyz.outputs[0]),
                                     math('MULTIPLY', xyz.outputs[1], xyz.outputs[1])))
            falloff = math('MAXIMUM', math('SUBTRACT', 1, math('MULTIPLY', radius, 1.65)), 0)
            colour = scale(colour, math('ADD', 0.12, math('MULTIPLY', falloff, 0.88)))
        colour = scale(colour, GLOW_GAIN)
    else:
        colour = value(bsdf.inputs['Base Color'])
        ao = nodes.new('ShaderNodeAmbientOcclusion')
        ao.inputs['Distance'].default_value = 0.28
        ao.samples = 32
        colour = scale(colour, math('ADD', 0.38, math('MULTIPLY', ao.outputs['AO'], 0.62)))
        # A broad directional fill keeps the form legible even under weak stage lights.
        xyz = nodes.new('ShaderNodeSeparateXYZ')
        links.new(geom.outputs['Normal'], xyz.inputs[0])
        colour = scale(colour, math('ADD', 0.77, math('MULTIPLY', xyz.outputs[2], 0.23)))
        if mat.name == 'BF_Hull':
            pos = nodes.new('ShaderNodeSeparateXYZ')
            links.new(geom.outputs['Position'], pos.inputs[0])
            bounds = bpy.context.object.dimensions.z
            height = math('MINIMUM', math('MAXIMUM', math('ADD', 1, math('DIVIDE', pos.outputs[2], bounds)), 0), 1)
            colour = scale(colour, math('ADD', 0.72, math('MULTIPLY', height, 0.28)))
        if mat.name == 'BF_Trim':
            noise = next(n for n in nodes if n.type == 'TEX_NOISE')
            colour = scale(colour, math('ADD', 0.7, math('MULTIPLY', noise.outputs['Fac'], 0.6)))
        if mat.name == 'BF_Rim':
            # the band facing the camera: brighter, so the slab's silhouette reads at play distance
            colour = scale(colour, RIM_GAIN)
        # Edge wear on every opaque material: where a small bevel normal leaves the face normal, the
        # surface sits on a chamfer or frame edge - lift it towards a bright worn-metal highlight.
        bevel = nodes.new('ShaderNodeBevel')
        bevel.inputs['Radius'].default_value = 0.022
        bevel.samples = 8
        dot = nodes.new('ShaderNodeVectorMath')
        dot.operation = 'DOT_PRODUCT'
        links.new(bevel.outputs['Normal'], dot.inputs[0])
        links.new(geom.outputs['Normal'], dot.inputs[1])
        edge = math('MINIMUM', math('MULTIPLY', math('SUBTRACT', 1, dot.outputs['Value']), 10), EDGE_MAX)
        mix = nodes.new('ShaderNodeMix')
        mix.data_type = 'RGBA'
        mix.blend_type = 'MIX'
        links.new(edge, mix.inputs['Factor'])
        links.new(colour, mix.inputs[6])
        mix.inputs[7].default_value = (0.86, 0.88, 0.95, 1.0)
        colour = mix.outputs[2]
    emission = nodes.new('ShaderNodeEmission')
    links.new(colour, emission.inputs['Color'])
    links.new(emission.outputs[0], output.inputs['Surface'])


def bake(mesh, size, glow):
    image = bpy.data.images.new("BF_GlowBake" if glow else "BF_ColorBake",
                                width=size, height=size, alpha=True, float_buffer=True)
    for mat in mesh.materials:
        route_material(mat, image, glow)
    bpy.ops.object.bake(type="EMIT", margin=6, use_clear=True)
    pixels = array("f", [0.0]) * (size * size * 4)
    image.pixels.foreach_get(pixels)
    return pixels


def srgb_byte(linear):
    x = min(1.0, max(0.0, linear))
    s = 12.92 * x if x <= 0.0031308 else 1.055 * x ** (1.0 / 2.4) - 0.055
    return max(0, min(255, round(s * 255)))


def atlas_bytes(base, glow, size, separate):
    out = bytearray(size * size * 4)
    for i in range(0, len(out), 4):
        for c in range(3):
            out[i + c] = srgb_byte(glow[i + c] / (1.0 + glow[i + c]) if separate else base[i + c])
        out[i + 3] = 255
    return out


def write_png(path, rgba, size):
    def chunk(kind, payload):
        return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", zlib.crc32(kind + payload))
    stride = size * 4
    # The byte array follows Blender's bottom-up image.pixels order; PNG rows are top-down.
    rows = b"".join(b"\0" + rgba[y * stride:(y + 1) * stride]
                    for y in range(size - 1, -1, -1))
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0))
                     + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b""))


def soften_glow(pixels, size):
    # Small separable Gaussian shoulders preserve line centres at play distance.
    # The bake has black gutters; no wraparound at image edges.
    rgb = np.asarray(pixels, dtype=np.float32).reshape(size, size, 4).copy()
    blur = rgb.copy()
    for axis in (0, 1):
        pad = [(0, 0)] * 3
        pad[axis] = (2, 2)
        padded = np.pad(blur, pad, mode='constant')
        blur = sum(weight * np.take(padded, range(i, i+size), axis=axis)
                   for i, weight in enumerate((1/16, 4/16, 6/16, 4/16, 1/16)))
    rgb[:, :, :3] = 0.72 * rgb[:, :, :3] + 0.28 * blur[:, :, :3]
    return rgb.ravel()


def write_gxtex(path, rgba, size):
    # GXTX v1 stores byte length, so contiguous GX mip levels need no new header.
    # Stop at 64: smaller atlas mips would mix unrelated surfaces across gutters.
    image = bytearray()
    level = np.frombuffer(rgba, dtype=np.uint8).reshape(size, size, 4)[::-1].copy()
    while size >= 64:
        for ty in range(0, size, 4):
            for tx in range(0, size, 4):
                tile = level[ty:ty+4, tx:tx+4].reshape(16, 4)
                image.extend(tile[:, [3, 0]].tobytes())
                image.extend(tile[:, [1, 2]].tobytes())
        # Average in linear space, then encode to the same sRGB-byte GX convention.
        rgb = level[:, :, :3].astype(np.float32) / 255
        rgb = np.where(rgb <= 0.04045, rgb/12.92, ((rgb+0.055)/1.055)**2.4)
        size //= 2
        rgb = rgb.reshape(size, 2, size, 2, 3).mean(axis=(1, 3))
        rgb = np.where(rgb <= 0.0031308, rgb*12.92, 1.055*rgb**(1/2.4)-0.055)
        level = np.full((size, size, 4), 255, dtype=np.uint8)
        level[:, :, :3] = np.rint(rgb*255).astype(np.uint8)
    header = struct.pack('>4sIIIIIIIIII', b'GXTX', 1, 6, rgba_size := int((len(rgba)/4)**0.5),
                         rgba_size, 0xFFFFFFFF, 0, len(image), 0, 64, 64+len(image))
    path.write_bytes(header + bytes(64-len(header)) + image)


def main():
    a = options()
    a.output.parent.mkdir(parents=True, exist_ok=True)
    runpy.run_path(str(HERE / "bf_platform_build.py"))
    source = bpy.data.objects["BF_Platform"]
    set_dimensions(source, a)
    mesh = evaluated_mesh(source)
    source.hide_render = True
    bake_object(mesh)
    repack_uv(mesh, a.texture_size)
    nv, nt = write_mesh(a.output.with_suffix(".gxmesh"), mesh, a)
    base = bake(mesh, a.texture_size, False)
    glow = soften_glow(bake(mesh, a.texture_size, True), a.texture_size)
    color_rgba = atlas_bytes(base, glow, a.texture_size, False)
    glow_rgba = atlas_bytes(base, glow, a.texture_size, True)
    write_png(a.output.with_suffix(".png"), color_rgba, a.texture_size)
    write_png(a.output.with_suffix(".glow.png"), glow_rgba, a.texture_size)
    write_gxtex(a.output.with_suffix(".gxtex"), color_rgba, a.texture_size)
    write_gxtex(a.output.with_suffix(".glow.gxtex"), glow_rgba, a.texture_size)
    print(f"BF platform export: {nv} vertices, {nt} triangles, {a.texture_size}px atlas -> {a.output}")


if __name__ == "__main__":
    main()
