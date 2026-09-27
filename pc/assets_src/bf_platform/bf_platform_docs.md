# BF_Platform — technical reference

A parametric platform-fighter stage platform, built in Blender with Geometry Nodes.
Source: `bf_platform_build.py` (rebuilds it from scratch). `bf_platform_verify.py` is
the automated check that produced the numbers below. `bf_platform_backup.blend` is a
saved snapshot of the object at its default size.

## What it is

- One mesh object, `BF_Platform`, driven entirely by a Geometry Nodes modifier called
  `BF Platform` (node group `BF_Platform_Gen`).
- Thin slab, cut corners, a raised metal frame around a glowing deck panel, a rim with
  evenly spaced lights, and a faceted underside with ribs, a spine, and a glowing core.
- Six materials: `BF_Deck`, `BF_Trim`, `BF_Rim`, `BF_Hull`, `BF_Glow`, `BF_Core`.
- Two UV maps: `UVMap` (real-world metres, for tiling textures) and `UVBake` (the same
  layout fitted into 0–1, non-overlapping, for baking to a single texture).

## Regenerating it

Run `bf_platform_build.py` inside Blender (Scripting tab, or via the MCP
`execute_blender_code` tool). It is idempotent: rerunning it deletes and rebuilds
`BF_Platform_Gen`, the six materials, and the `BF_Platform` object, keeping the
object's existing transform if one is already in the scene. Any earlier object
literally named `Platform` is moved into a hidden collection
(`Archive_old_platform_v1`), not deleted.

Run `bf_platform_verify.py` after any change to the build script. It stress-tests the
generator at ten different sizes/scales and asserts: no empty material slots, no
degenerate geometry, correct UV layer names, UV stretch ≤ 1.001×, and the bake UV
layout staying inside 0–1 with no overlap. It restores the object's original slider
values and scale when it finishes, whether it errored or not.

## Modifier inputs

Set these in the `BF Platform` modifier's panel, or via
`modifier.properties.inputs.<Socket_N>.value` (map slider name → socket id through
`node_group.interface.items_tree`).

**Size**
| Input | Default | Meaning |
|---|---|---|
| Width | 8.0 m | Overall length along X |
| Depth | 2.4 m | Overall depth along Y |
| Thickness | 0.4 m | Top surface down to the bottom plate |

**Profile** (each of these is a fixed real-world size — it does not scale with Width/Depth)
| Input | Default | Meaning |
|---|---|---|
| Corner Cut | 0.45 m | Length cut off each corner, seen from above |
| Rim Height | 0.12 m | Height of the vertical edge band carrying the lights |
| Bevel | 0.03 m | Chamfer above and below the rim |
| Border Width | 0.16 m | Width of the raised frame around the deck |
| Panel Recess | 0.02 m | How far the deck sits below the frame (its sloped wall is the glow inlay) |
| Underside Ledge | 0.10 m | Flat strip under the rim before the hull slopes away |
| Hull Inset | 0.45 m | How far the bottom plate is set in from the edge |

**Details**
| Input | Default | Meaning |
|---|---|---|
| Core Size | 0.6 m | Radius of the glowing underside core. 0 removes it |
| Rib Spacing | 0.9 m | Target gap between underside ribs. Longer platforms get more ribs, not stretched ones. 0 removes ribs and the spine |

**Behaviour**
| Input | Default | Meaning |
|---|---|---|
| Follow Object Scale | On | When on, scaling the object (S key) resizes the platform the same way the Width/Depth/Thickness sliders do, without stretching any detail |

## Resizing rules (verified)

- **Width/Depth changes only lengthen the flat middle.** Corner cuts, the rim, bevels,
  border, and hull slope keep their real-world size at every width tested (8 m, 20 m,
  40 m). Rib count scales with length (6 → 20 → 42 ribs); rib spacing stays within
  ~0.01 m of the target.
- **Object scale (S key) behaves identically to the sliders**, as long as *Follow
  Object Scale* is on. Do not apply the scale afterward — that bakes it in and the
  platform snaps back to its slider-defined size on the next modifier evaluation.
- **Thickness is the one slider that changes the shape**, not just the size: it
  changes the hull's slope angle (26° at the default 0.4 m, 71° at 1.5 m).
- **Small platforms degrade gracefully.** A "feature scale" factor automatically
  shrinks the bevel/border/rim/etc. in proportion when the platform is too small or
  thin for them at full size (tested down to 1 m × 0.5 m × 0.1 m), rather than
  producing overlapping or inverted geometry.
- At every size tested: 0 non-manifold edges, 0 zero-length edges, 0 zero-area faces,
  0 inside-out islands, all faces planar, all UVs within 0–1 stretch tolerance.

## Materials

All six are procedural (node-based), not image textures, and read from the same
per-vertex attributes the geometry writes (`bf_pos`, `bf_dim`, `bf_level`, `bf_edge`):

- `BF_Deck` — panel seams, a centred circular emblem, rails to each end, all in cyan
  glow lines on a dark base; scales with the deck's own footprint, not the whole platform.
- `BF_Trim` — brushed steel with subtle noise-driven roughness variation.
- `BF_Rim` — evenly spaced cyan lights along every edge, count recalculated per edge length.
- `BF_Hull` — panelled underside with a magenta energy line.
- `BF_Glow` / `BF_Core` — flat emissive materials (inlay strip, underside core).

Because they're procedural, they need no UVs to look correct in Blender, but an
external engine that only reads baked textures will need the `UVBake` layout (see
below).

## UV maps

- **`UVMap`** — every face unfolds true to its real-world shape, in metres. Good for
  tiling a physical texture (e.g. a metal panel material) so it never stretches
  regardless of platform size.
- **`UVBake`** — the same layout, uniformly scaled to fit inside 0–1 with no
  overlapping faces (verified per test case). This is the one to bake AO/curvature/ID
  maps onto, or to use as the export UV if a target engine expects a single 0–1 map.

Note: `UVBake`'s U/V *range* changes with the platform's proportions (e.g. it may fill
only 0–0.16 vertically on a very long platform) — it is not repacked to fill the full
square at every size. If a fixed-resolution bake is needed regardless of platform
size, repack `UVBake` in Blender's UV editor before baking.

## Known limits

- Six separate materials means six draw calls per platform until textures are baked
  down to one material.
- The core's UV unwrap is approximate (not verified to the same stretch tolerance as
  the rest of the body).
- Procedural materials do not survive FBX/glTF export as-is; export only carries
  vertex colors/UVs/mesh data unless the materials are baked to images first.
- The node tree is large (about 550 nodes across 10 labelled sections plus 3 small
  reusable helper groups: `BF Slope Length`, `BF Corner Cut At Inset`, `BF Profile
  Step|), organised by dependency depth for readability, not modding — treat
  `bf_platform_build.py` as the source of truth and re-run it rather than hand-editing
  the node tree.
