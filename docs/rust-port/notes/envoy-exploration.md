# Envoy exploration rooms: a Brawl Subspace map, doors (2026-11-13, port/envoy-stage1)

An experiment, off by default (`MELEE_ENVOY_EXPLORE=1`). Code: `rust/melee_modes/src/tu/explore.rs`; the shared Brawl
readers are in `petey_model.rs` (`pac_leaves`, `static_models`, `draw_fade`). Nothing read from the owner's Brawl
files is committed; the archives are read at run time from `_build/brawl` (or `MELEE_BRAWL_LOCAL`).

## The map, and why

The Jungle, step 3: `stage/adventure/080103.pac` (corridor, 770 x 240 units, floor at y 0, a raised slab with a
ledge, a higher shelf, a sloped run to a wooden bridge) and `080103a.pac` (the small hut room, 160 x 80, one door).
Chosen by surveying all 271 adventure archives (collision size, extents, doors): it is short, mostly flat, enclosed
(nothing to fall off), has an ordinary door that leads into a separate archive's room, and the Brawl collision is
small (46 points, 36 planes + 6 / 6). The vertical shaft sections (e.g. 060004) were rejected as not flat.

## The Brawl formats as understood (BrawlLib/BrawlCrate source on this machine was the reference)

* `.pac` = ARC (`ARC\0`, 0x40 header, 0x20 per entry: type, index, size; entries 0x20-aligned), entries may be
  LZ77 (0x10/0x11) and nest. Entry types: 2 models (BRRES), 3 textures (BRRES), 4 animations, 5 scene, 1 misc
  (collision, BLOC param blocks). The model entry's file index is what door records name (`modelIndex`).
* Collision: header (s16 points, planes, objects; offsets to points (BVec2, 8 B), planes (0x10), objects (0x6C)).
  Plane: point1, point2, link prev, link next, -1, u16 type (low nibble 1 floor / 2 ceiling / 4 right wall /
  8 left wall; 0x10 characters, 0x20 items, 0x40 PT, 0x80 Bucculus, 0x100 crush), material flags (0x01 drop
  through, 0x04 rotating, 0x20/0x40 left/right ledge, 0x80 no walljump), material id. Objects group plane ranges
  and may bind to a model bone (moving parts); a section's main collision is the largest file.
  The orientation and the kind bits match Melee's own (floors left to right, solid to the right of each line), so
  planes become retail `MapLine`s one to one (kind -> `hi_flags`, ledge -> 0x200, drop through -> 0x100,
  material -> terrain), grouped floor / ceiling / right / left as `mpLibLoad` expects, links remapped.
* BLOC: tag, count, offsets to sub-blocks (tag, count, entry offsets at +8). `GDOR` doors (0x90): area offset and
  range at 0x20, level / sequence / segment / door index at 0x30, gimmick kind (ground / air / auto) at 0x34,
  model index 0x36, position 0x38, door type 0x44 (normal, yellow, arrow, shadow, save, warp, factory).
  `GEG1` enemy generators (0x84): enemy id 0x1C, spawn position 0x28, facing 0x67 (plus timers, counts).
  Where a door leads is not in GDOR (Brawl's step/jump tables elsewhere); this map's links are written by hand.
* Models: BRRES MDL0 v9 + TEX0/PLT0, read by Petey's importer (same code), posed once in bind pose.

## What is implemented

* Collision: built once off the game thread into a template; at each stage load (`ground::EXPLORE_COLL`, before
  the script layer and `mpLibLoad`) copied into the game heap as a retail `MapCollData` with one joint, so every
  collision routine (ledges, platforms, walls, ECB) is Melee's own and rollback snapshots cover it.
* Two sections share one world: the hut room sits at x + 1500. Camera and blast bounds are the party's section
  (camera = collision extent, blast = 120 outside it), set at load and at each door.
* Camera / co-op: Melee's own match camera, bounded to the section; it keeps every fighter on screen by zooming
  out. The sections are small enough that the trailing player is never lost; at a door the whole party moves
  together (side by side at the destination). Decision: no "pull along" rubber band.
* Doors: a fresh up tilt on the ground within 15 units of the door (and up to 55 below an air door) opens it;
  a 24-frame black fade, the party moves at its midpoint. The arrow door is the exit: outside a run it returns
  to the corridor's door; in an Envoy run the room continues as the fight it replaced. General primitive: a list
  of sections + door links (`SECS`, `LINKS`); a third section is one more line each.
* Enemies: the corridor's `GEG1` spawn points (first four) spawn Envoy's script enemy (Goomba); the Subspace
  enemy lane (`port/envoy-subspace1`) is not merged.
* Look: the section MDL0s and the door model drawn in the fighter pass (`ftdrawcommon::MAP_DRAW`), Final
  Destination's own model hidden (`grdisplay::HIDE_GROUND`). Background: FD's.
* Envoy: with the flag, every non-boss fight is played in the room (`envoy::match_begin` picks FD and sets
  `explore::WANT`); outside a run any FD match is the room. Without the flag nothing is hooked.

## Missing

* Doors are linked by hand (`LINKS`); Brawl's own step/jump tables are not read. 080103 has a linked and an
  unlinked door at the same spot on the slab; the linked one wins.
* Look: some corridor geometry draws as stretched streaks (left edge of the corridor); the hut floor draws
  untextured (flat beige); map geometry in front of the play plane can cover fighters, and right after a door the
  camera takes a moment to catch up (on the ACE run the first frames back in the corridor show a wall).
  The background is FD's. No moving map parts, no Brawl lighting/fog, no map animations.
* Tap jump also fires on the door tilt (the fighter hops as the door opens).
* Enemies are Envoy's Goomba, not Brawl's (the Subspace enemy lane is not merged); only the first four spawns.
* Performance (owner's rule: no game-caused frame over 16 ms): NOT met. Profile of the door run (MELEE_PROFILE=1,
  vanilla, 1815 frames): 9 frames over 16.7 ms, worst 64.1 ms at tick 8 (42.8 ms GPU pipeline creation for the
  map's materials), 31.7 ms at the first entry into the hut (16.7 ms GPU pipelines: the hut's materials compile on
  first draw), and 25-39 ms around the return through the third door. The machine was busy (other agents' runs,
  CPU over 85% in 2 samples), so the door-time numbers are pessimistic, but the pipeline frames are real. Fix:
  create every section's pipelines at map load (before the match), not on first draw.

## Results (2026-11-13, after rebase onto port/desync1 70a0841ff)

* Door chain, vanilla and ACE discs, same pad script: exit door on the floor -> slab door, slab door -> hut, hut
  door -> slab. All three transitions fire and both fighters arrive together.
* Gate: goldens, synctest (0 mismatches), audio, baselines, semantics lint (0), snapguard all ok; the one failure
  is `lint: new R2 rust/melee/src/netmatch.rs`, which comes from the base (70a0841ff); this branch does not touch
  that file.

## What a second map costs

1. Pick the archive(s); add one `SECS` line each (file, world offset, model name) and one `LINKS` line per door.
2. Check the collision imports (moving or bone-bound collision objects need the moving-part work that is missing).
3. Fix whatever material/texture the model reader does not handle yet for that map (the hut floor shows gaps).
4. Pre-create its pipelines (nothing once the load-time pipeline fix exists).
5. A pad-script door test and a profile run. About half a day for a static map; moving platforms, ladders, water
   or autoscroll sections need new code each.
