# vk_concepts — a second stage/room kit, as a concept

A concept kit sitting alongside `bf_platform/` and `bf_interior/`. It shares **no code**
with either; it was written after reading them, to answer a different question: what does
the *system* look like if you keep the BF kit's rules but change the structure and the
colour pipeline?

**This is not integrated.** Nothing here is exported to GXMS v2, nothing references
`gd.model_load` / `gd.model_spawn`, no Lua, and nothing has been built or run in the port.
The only binaries are PNGs. If it graduates, it would need its own `export_*.py`.

## Files

| file | what |
|---|---|
| `vk_platform.py` | parametric platform-fighter slab; generates the greymasks and the 6 platform materials |
| `vk_kit.py` | 13 module parts on the 2 m grid; imports `vk_platform.py` by `runpy`, the way `bf_interior_playset.py` imports the stage kit |
| `vk_textures/` | 4 tileable **greymask** PNGs (luminance, Non-Color) + the palettes |
| `vk_kit_out/` | collision sidecars (v1 shape), module manifest, stage placement table |
| `reference/` | Cycles renders made while developing |

## Run

```sh
blender --factory-startup --background --python pc/assets_src/vk_concepts/vk_platform.py
blender --factory-startup --background --python pc/assets_src/vk_concepts/vk_kit.py
```

Both resolve `vk_textures/` and `vk_kit_out/` relative to their own directory, so they can
be run from anywhere. Idempotent: each run deletes and rebuilds its own masks, materials,
node-free geometry and module collections.

## What was kept from the BF kit

- every feature has a **fixed real-world size**; resizing only lengthens the straight runs
- a `feature_scale` factor degrades detail on small platforms instead of inverting geometry
- two UV layers: `UVMap` (true metres) and `UVBake` (fitted 0-1)
- per-vertex attributes driving procedural materials (`vk_pos`, `vk_dim`, `vk_level`, `vk_edge`, `vk_brace`)
- 2 m grid; Blender X = travel, Y = depth, front = -Y, Z = height
- one collection per part, asset-marked, carrying `module_size` / `purpose` / `grid_m`
- collision as lines, sidecars in the same v1 shape, `flags = 1 passthrough + 2 ledges`
- a placement table equivalent to the Lua `ROOM` table

## What was changed

**Structure.** `bf_platform` is a slab over a solid faceted hull. This is a slab over an
**open space-frame truss**: perimeter chords, posts, diagonal bracing and a suspended
glowing core, so the underside reads as architecture. The profile uses a true parallel
inset (`offset_poly`), not a scale factor, so a border really is border-width at any size.

**Colour.** The BF kit bakes one shared smart-projected atlas per kit. This one ships 4
standalone tileable greymasks and takes hue from a palette dict:

```
vk_textures/vk_palette.json      6 platform slots
vk_textures/vk_kit_palette.json  the above plus wall/panel/accent/glass/vista
```

Every material is `mask × palette colour`, sampled through `UVMap` in world metres so
texel density is size-independent. Recolour by editing a JSON and re-running, or by
poking one colour node in the shader — nothing re-bakes.

> Gotcha worth keeping: because masks project through world-metre UVs, a **spatially
> banded** mask lands small emissive props on random parts of the band and they render
> dark. Greymasks that are UV-projected must be flat-bright; shape belongs in geometry.

**Palette.** bone deck / oxidised copper / patina teal / slate hull / amber glow,
against the BF kit's indigo-violet with cyan and magenta.

## Verified

`verify()` in `vk_platform.py` stress-tests the generator at four sizes and asserts no
zero-length edges, no zero-area faces, no non-manifold edges, no loose verts, no empty or
unused material slots, both UV layers present, all five attributes present, and `UVBake`
inside 0-1.

| width | feature_scale | rim lights | truss bays | z_span | issues |
|---|---|---|---|---|---|
| 8 m | 1.000 | 26 | 14 | 1.5185 | none |
| 20 m | 1.000 | 66 | 40 | 1.5185 | none |
| 40 m | 1.000 | 130 | 82 | 1.5185 | none |
| 1 x 0.5 x 0.1 m | 0.294 | 8 | 4 | 0.4290 | none |

`z_span` is identical at 8/20/40 m: feature sizes do not move, only counts scale.

Platform: 1228 tris, `issues: []`. Kit: 13 modules, 8342 tris over 36 instances.
6 non-manifold edges across the modules, all on intentional single-quad decals (the
underside energy line, the glass glints) — boundary edges on flat planes, not defects.

Four real bugs were caught by the verifier, worth noting because they are the kind that
render fine and only fail an audit: a duplicate overlapping rim lathe, `materials.clear()`
silently resetting every face's material index, one bow-tie quad face in the box builder
(156 non-manifold edges -> 0), and rim lights centred on the run *start* rather than its
midpoint, which floated them off the platform.

## Known gaps

- **No GXMS v2 export.** Sidecars and the placement table match the v1 shapes; the mesh
  binary, the Lua and the runtime instancing are not written.
- **No atlas bake.** Four tileable masks instead of one per-kit sheet: no 16 m-spaced bake
  scene and no AO crosstalk, but also no single-texture draw call.
- **`UVBake` islands overlap.** It is a per-face planar projection scaled and centred into
  0-1. `export_kit.py`'s `smart_project` does not overlap; this does.
- Masks are 8-bit PNG, so subtle ramps may band.
- The verifier covers the platform only; the 13 modules were checked once by hand, not
  swept at multiple sizes.
