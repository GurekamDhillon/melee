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
