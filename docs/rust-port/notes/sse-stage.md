# SSE stage slot (port/sse-stage, port/sse-stage2)

The Brawl Subspace Emissary map (The Lake, step 3: `080103` + side rooms `080103a` (hut) and `080103b`, read at run
time from the owner's own Brawl files; `explore.rs`) is its own stage slot. Final Destination is Final Destination
again.

## Which level this is

Round 1 called it The Jungle because Brawl's own model names for it say so (`081mAdvJungle03`). The level is
**The Lake**: prefix `08` holds Rayquaza (`080001`, GEG1 kind 53), and Rayquaza is The Lake's boss; the boss-battle
files are in story order Petey (03) -> Rayquaza (08) -> Porky (09). Brawl's internal names keep "Jungle" for the
whole Diddy arc. The constant is `GD_STAGE_SSE_LAKE3`; the stage select names it "The Lake 3 (SSE)". `--match
...,lake` (also `sse`, `jungle`) picks it.

**Is `06` The Jungle?** Yes, by the same evidence (inferred, not read from a story table): it sits between Sea of
Clouds (05) and The Plain (07) in prefix order, which follows the story, and its enemy mix is The Jungle's (Goomba,
Koopa, Hammer Bro, Bullet Bill, Giant Goomba; GEG1 kinds 0, 25, 19, 13, 8) with the level's vertical climb
(`060004`) and no boss file of its own (Jungle ends in a cutscene, not a boss).

## GD stage ids

- `explore::GD_STAGE_FIRST..=GD_STAGE_LAST` (0x7000..0x70FF): external stage ids owned by the explore loader. Retail
  external ids stop below 0x21; m-ex external ids start at 288 (`SSS_EXT_MEX_FIRST`), so the range clashes with neither.
- `explore::GD_STAGES`: the GD stage table (id, name, archives). One row today: `GD_STAGE_SSE_LAKE3` (0x7001),
  "The Lake 3 (SSE)". `080103b` is optional: without it the main section still plays and its door stays shut.
- `explore::available(id)`: every archive of the row is in the Brawl folder (`MELEE_BRAWL_LOCAL`, else `_build/brawl`
  walking up). `MELEE_BRAWL_ABSENT=1` behaves as if they were missing.
- `explore::carrier(id)`: the retail stage id a match really loads. A GD id -> Final Destination (0x20) and `PICK` is
  set; any other id clears `PICK`. The rest of the game (preload cache, stage tables, replays, results) only ever sees
  a retail id. The stage load (`coll_override`) swaps in the room's collision when `PICK` (VS) or `WANT` (Envoy) is set.

## What the room reads from Brawl's data (round 2)

Offsets are entry-relative; only ids and counts here.

- **Collision:** every collision file of every section, object by object. A collision object names the bone it rides
  (+0x2A index, +0x4C name). World objects (no bone, 0xFFFF) are the static map and are welded across files (the
  arena's welding code). Bricks (`AdvGdtPunBrick*`) at their bone's rest place become their own collision joint each
  (breakable blocks). Item-box tops (`AdvGdtItemBox*`) are dropped: the crate is a Melee item. Everything else rides
  a moving part (GMOV) the port does not animate yet and is left out (logged). `080103`: 3 files, 3 world objects,
  one moving floor (left out), one item-box top.
- **Enemies (GEG1):** kind = u16 at +0x1C, mapped by `enemy_kind` to the nearest of our four Subspace kinds (table
  below); a boss kind is not spawned. Each section's points spawn when the party enters it (the previous section's
  enemies go), at most the pool's 8. They are `subspace.rs` sim entities, so a plain VS match on the slot gets them
  too (round 1 spawned only inside Envoy runs because the old script enemies needed Envoy's articles).
- **Doors (GDOR):** id u32 +0x30 (level, step, room letter, index); +0x34 = 1 on a side room's way back; +0x51 = 0x13
  on a main-section door into a side room; type byte +0x44 (2 = arrow). `link_doors`: the main section's room doors
  lead, in order, into rooms `a` then `b`; each room's way-back door returns to the door that led in. `080103` has
  its two room doors at one spot: the one into a room not yet seen opens first (`VISITED`), so the first visit is
  the hut, the next `b`. This pairing rule is inferred from the data shapes (`080103`, `080104` have the same pair),
  not from a Brawl table.
- **Level exit (GMSJ):** carries a copy of its exit door (a nested GDOR); that door id is the exit. A section set
  with no GMSJ (`080103`'s) uses its arrow door. The exit sends the party back to the start (VS) as before.
- **Crates (GIB2):** x, y at +0x30/+0x34, size at +0x38 (0 small, 1 crate, 2 big) -> Melee capsule, box, barrel
  (`Item_80268B18`), spawned once per room. Their contents follow the match's item switch.
- **Blocks (GBLK):** the brick collision joints above. Any live fighter hitbox touching one breaks it
  (`mpLib_80057BC0` switches its joint off). `080103` has none (the level's blocks are in `080101`/`080102`/`080105a`
  /`080201`); the path is checked by the loader test only.

Enemy kinds seen in level `08` and its neighbours, and what they become:

| Brawl kind (id) | ours | | Brawl kind (id) | ours |
|---|---|---|---|---|
| Goomba (0) | Goomba | | Towtow (18) | Giant Goomba |
| Poppant (1) | Goomba | | Hammer Bro (19) | Primid |
| Feyesh (2) | Goomba | | Primid (23) | Primid |
| Jyk (3) | Goomba | | Koopa (25) | Goomba |
| Auroros (4) | Bullet Bill | | Boomerang Primid (32) | Primid |
| Borboras (7) | Giant Goomba | | Sword Primid (35) | Primid |
| Giant Goomba (8) | Giant Goomba | | Glunder (45) | Goomba |
| Bucculus / Mite (10, 16) | Goomba | | Rayquaza (53, boss) | not spawned |
| Bullet Bill (13) | Bullet Bill | | Spaak (15), Ticken (17) | Primid, Goomba |

Any other id: Goomba, logged as "unmapped".

**Bullet Bill in a map room** (handle >= `subspace::ROOM_HANDLE_BASE`) starts at its cannon's own point (the FD
re-placement is skipped), explodes on the terrain it flies into (`mpCheckAll` along its path), on a player, at the
end of its life, and when beaten (it blows up where it comes down). Final Destination / Envoy test-spawned Bills keep
their old behaviour, so the subspace check is unchanged.

## Stage select

`mnstagesel.rs` `gd_icons_add`: after the retail grid (or the m-ex icons on ACE) one row per available GD stage is
appended past `mnStageSel_NumStages`, so random never picks it. Its icon is a placeholder (the screen's own random
plate, bottom-right below the grid); hovering it hides the retail name plate and shows the row's name as an SIS text.
Picking it writes the carrier id into the rules via `carrier`. Without the Brawl files no row is added and the screen
is the retail one.

## Envoy

`envoy::match_begin` asks for `GD_STAGE_SSE_LAKE3` for every non-boss fight when `MELEE_ENVOY_EXPLORE=1` and the
files are present (unchanged opt-in), and loads its carrier.

**Why the camp run lost fight 2 in round 1 (investigated in round 2).** The room's sections are closed boxes:
`080103` has a wall on the left and a ceiling line, and both side rooms are 160 x 80 boxes. A launched fighter hits
a wall or the ceiling long before the blast zone (120 past the extent), so nobody is ever KO'd. Measured in round 2
with the camp script (P1 a level-9 CPU): fight 1 ran past 24000 frames with the foe at 443% and no KO (the FD
baseline wins fight 1 in about 2000 frames and reaches fight 4). With nobody KO-able the run cannot be won, and a
two-foe fight (fight 2: two CPUs) grinds the party down. Two more things made it worse and are fixed:

- any fighter's up input opened a door, so a CPU (the foe, or the party's own CPU) dragged everyone into the hut
  (seen at room frames 7351 and 9395 in round 2's first run). Now only a human player of the party opens doors.
- round 2's map enemies first took all 8 Subspace pool slots and the Envoy sim's own room enemies were refused
  (`room enemy spawned ... -1`); the map now keeps half the pool for the sim inside a run and clears only its own
  enemies (`subspace::clear_room`) on a door.

The fix for the KOs is `explore::screen_kos`: a fighter in a damage flight whose knockback would carry it past
where the blast zone would be without the walls (Melee's knockback decays 0.051 a frame, so its reach is
v^2 / 0.102) is put just past the real blast zone on that side, and Melee's own KO takes it. Brawl KOs a fighter
sent off the screen in the same way.

## Known limits

- Replays and netplay record the carrier (Final Destination); the pick itself is not in the replay header yet.
- The icon is a placeholder; no preview picture.
- Not the full m-ex stage tables: only the SSS row and the id range. m-ex stage *loading* (`grfunction`) is separate.
- Moving collision (GMOV/GMPS), and the brick models (a broken brick's model is not hidden) are not done.
- A brick's place is its bone's rest translation found by name; several bricks sharing one bone name would land on
  one spot (none in this map).

## Round 3 (partial)

- Subspace enemy kinds 4-6: Koopa (rig EnmPatapata, Walker; shell slide stops 45 frames after it connects),
  Hammer Bro (EnmHammerbros, Brawler; throw box and jump swing), Jyk (EnmJyakeel, new `Arch::Floater`: drifts
  toward a player within 40 of home sideways/up and 60 down; on FD its home is clamped to y 10..35). Map spawns:
  enemy id 3 -> Jyk, 19 -> Hammer Bro, 25 -> Koopa. Hits on Missile/Floater kinds are now logged ("hit by slot").
- subspace_check d4/d7 vanilla: all 7 kinds ok, 0 mismatches. Gate OK.
- Not done this round: Jungle as its own GD slot, moving collision (GMOV/GMPS; 060002a sfyuka00-03 ride the
  st02asuisha wheel CHR0, 600 frames), brick models / hiding broken bricks, side-room screenshots, frame-time
  profiles on the maps.
