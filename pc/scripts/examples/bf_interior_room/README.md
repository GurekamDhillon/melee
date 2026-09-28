# BF interior kit: open-front room

A gameplay mod that assembles a room from the BF interior kit's parts with the runtime model API
(`gd.model_load` / `gd.model_spawn`, docs/scripting.md). Each part is its own model with its own
collision; nothing is baked as a scene.

- `scripts/main.lua`: `ROOM` is the layout in kit metres (floors, stairs, balcony, ramp, a landing
  with a drop-through gap, back walls on two storeys, a door leaf, side returns, inside and outside
  corners, beams, posts, a rear rail and end trims: 41 parts from 17 models). `ORIGIN` sets where
  the room goes on FD and Battlefield; on any other stage it goes 20 units above P1's start.
  `room_build(x, y)` and `room_despawn()` are globals, for tests and the console.
- `models/<model>.gxmesh`: one GXMS v2 mesh per part, in the part's own frame. All of them sample
  one shared atlas (`bf_kit.gxtex`, `bf_kit.glow.gxtex`), loaded once.
- `models/<model>.coll.json`: the part's collision sidecar (v1), which also names the atlas.
  Floors carry pass-through and ledge flags; the room overrides them per placement with
  `floor_flags` (solid ground, no ledges at interior seams). Walls and scenery have no lines.

Rebuild `models/` and the room's `local U` with `pc/assets_src/bf_interior/export_kit.py` (its
docstring has the command). `KIT_SCALE` there is the kit's size in game (1.3: the 1.6 x 2.6 m
doorway reads as a door next to Fox); it scales meshes, sidecars and the room grid together.
The binaries are generated, not committed (about 13 MB).

Tested in the LAB and VS on FD and Battlefield, and in Classic and Adventure (ACE, Fox): landing
on the balcony, dropping through the gap, a despawn of every part, a savestate restore that
brings all 41 back, and LAB restarts that free and reload the assets each time.

## What the runtime still lacks

1. **Transparency.** The model draw is opaque, so the exporter leaves out glass (window panes,
   the glass insert, rail glass).
2. **Mirroring.** Scale is uniform; the right inside corner cannot be mirrored, so the room
   shifts an unmirrored copy into place.
3. **A stage-bounds query.** The origin is picked by hand for FD and Battlefield, and from P1's
   start elsewhere.
4. **Draw cost.** 41 instances add about 160 draw calls and about 1.1-1.4 ms of frame cost.
