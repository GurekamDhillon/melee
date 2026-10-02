# BF interior kit: open-front room

A gameplay mod that assembles a room from the BF interior kit's parts with the runtime model API
(`gd.model_load` / `gd.model_spawn`, docs/scripting.md). Each part is its own model with its own
collision; nothing is baked as a scene.

- `scripts/main.lua`: `ROOM` is the layout in kit metres (floors, stairs, balcony, ramp, a landing
  with a drop-through gap, back walls on two storeys, a door leaf, side returns, inside and outside
  corners, beams, posts, a rear rail and end trims: 43 instances from 19 models, including
  two glass meshes). `gd.stage_bounds()` centres the room over the main floor and places it
  20 units above the highest base-stage floor/platform. Right-hand pieces use `scale_x=-1`.
  `room_build(x, y)` and `room_despawn()` are globals, for tests and the console.
- `models/<model>.gxmesh`: one GXMS v2 mesh per part, in the part's own frame. All of them sample
  the shared opaque atlas (`bf_kit.gxtex`, `bf_kit.glow.gxtex`), loaded once. Glass parts
  use the separate shared `bf_kit_glass` atlas and `<model>_glass.gxmesh` names.
- `models/<model>.coll.json`: the part's collision sidecar (v1), which also names the atlas.
  Glass sidecars set `alpha: 1` and have no collision. Opaque frame parts still write depth.
  Floors carry pass-through and ledge flags; the room overrides them per placement with
  `floor_flags` (solid ground, no ledges at interior seams). Walls and scenery have no lines.

Rebuild `models/` and the room's `local U` with `pc/assets_src/bf_interior/export_kit.py` (its
docstring has the command). `KIT_SCALE` there is the kit's size in game (1.3: the 1.6 x 2.6 m
doorway reads as a door next to Fox); it scales meshes, sidecars and the room grid together.
The binaries are generated, not committed (about 13 MB).

## Placement contract

The authoring generator stores `grid_m=2`, `module_size`, `purpose` and an asset
description on each Blender collection. See
[`module()` and the doorway geometry](../../../assets_src/bf_interior/bf_interior_playset.py)
and the coordinate conversion in
[`export_kit.py`](../../../assets_src/bf_interior/export_kit.py).
The exported mesh and collision sidecar do **not** carry or enforce all of this
placement metadata; the layout builder must enforce it.

| Rule | Kit metres | Current game units (`UNIT=6.5`) |
| --- | --- | --- |
| Structural snapping grid | 2 | 13 |
| Floor/wall bay width | 4 | 26 |
| Wall storey height | 4 | 26 |
| Doorway clear opening | 1.6 wide × 2.6 high | 10.4 wide × 16.9 high |

A doorway is a complete back-wall bay. Replace a solid wall bay with
`bf_wall_doorway_4m`; do not place both at the same bay or shrink the doorway
into a freestanding sign. Its origin is centred horizontally at the floor level.
Place adjacent bay centres 26 units apart, and align the doorway origin with the
floor bay beneath it. Keep the original scale of wall and door modules so their
baseboards, trim and ceilings meet. A global kit-size change belongs in
`KIT_SCALE`, which regenerates meshes, collision and layout units together.

Walls, doors, beams and rear posts already contain their rear depth offsets in
their local geometry. Give them the same placement depth as the matching floor
(`z=0` in the example); do not add another arbitrary rear offset to just the door.
Blender X/Z/-Y export to game X/Y/Z. Place posts at bay seams and beams at storey
tops. The optional `bf_door_leaf` uses the doorway's exact origin and transform.

Doorway scenery has no collision or transition trigger. The gameplay layout must
derive the interaction anchor from the same doorway origin, and separately
implement room gating. A visible leaf alone does not physically block passage.
These rules describe the structural shell; smaller decorative details and
explicitly authored gameplay platforms can use other dimensions.

The previous 41-instance lane was reported tested in LAB/VS on FD and Battlefield,
Classic/Adventure, despawn/restore and LAB restart. Those results describe the old
opaque, uniform-scale renderer. The revised 43-instance glass/mirrored room has
not been built or run in this code-only change.

Regenerate both atlases and all sidecars before using the revised room. Existing
exports do not contain the two new glass meshes. Objects and material slots are
merged per part/pass; the runtime additionally batches compatible shared-atlas
instances. See [model-gaps-report.md](../../../../model-gaps-report.md) for the
F4 before/after capture procedure, historical ~160-draw report, and validation
status. No after-run draw count is claimed yet.
