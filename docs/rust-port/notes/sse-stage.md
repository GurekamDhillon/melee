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

## Round 3, continued (Nov 14)

- The Jungle is GD stage 0x7002 (060002 + 060002a at x+3000), second GD icon; `--match ...,jungle`. Its doors carry
  neither the room-door nor the way-back flag, so `link_doors` pairs an unlinked side room with the first free main
  door (door 0 at (625,193) <-> the wheel room's door at (3000,-4)). Door 1 (1770,75) stays shut; no exit door.
- VS walks (real menus, 2 CPUs): Lake and Jungle on vanilla and ACE pick their slot and play; FD walk unchanged.
- `MELEE_EXPLORE_GOTO=door@frame,...` (checks only) sends the party through to a door at a room frame: used for the
  side-room screenshots (Lake rooms 1 and 2, Jungle wheel room).
- Frame time (hidden window, no screenshots, MELEE_PROFILE): Lake and Jungle each 1 frame over 16.7 ms in 4190, the
  match-setup load frame (92.7 / 95.5 ms; FD's own is 74.5 ms, the extra is the map's first draw and texture upload).
  Screenshot frames hitch on FD too (shot capture), so profiles with shots are not evidence.
- Still open: water-wheel platforms (sfyuka00-03 on st02asuisha), hiding broken bricks' models; the 060003 platform
  is skipped (GBLK bone index unresolved).

## Story table, chaining and the Envoy entry points (Nov 14)

- **Chaining.** `secs_of` lists every section of a GD stage in story order; side rooms follow their main section.
  The Lake (0x7001): 080103 (+a, b) -> 080104 (+a, b) -> 080105 (+a). The Jungle (0x7002): 060001 -> 060002 (+a,
  the wheel room) -> 060003 -> 060004 (+a, the climb). Each main section's way on is a door id in the table
  (`SecDef::exit`; 0 = its GMSJ door): it leads to the next main section's entry (`ENTRY` pseudo door at the
  section's first player spawn), and past the last one to the story (`STORY_NEXT`). Sections sit 8000 apart on x,
  side rooms 4000 below. Proof: `MELEE_EXPLORE_OPEN=door@frame` story walks (runs `st_lake_vs|vsace`,
  `st_jun_vs|vsace`): 080103 -> 080104 -> 080105 -> way on -> scene 4 -> Rayquaza (placeholder) -> scene 5; Jungle
  060001 -> 060002 -> 060003 -> 060004 -> way on -> scene 2 -> scene 3. Damage carries (same match).
- **Story table** `explore::STORY`: `Step::Map(id) | Boss(kind) | Scene(slot, kind)` with `Carry::Keep | Rest`.
  Scene slots found from the level data (Brawl's cutscenes are in `movie/`, never read): 1 Jungle opening, 2 Jungle
  end (060004's yellow door 06000402), 3 Lake opening, 4 before Rayquaza (080105's GMSJ exit), 5 after Rayquaza.
  Every slot runs a placeholder card (black, 2 s) for now; `SceneKind` (Card, EnvoyFight, Camp, Results) names what
  the owner wants there. Carry `Rest` is recorded, not yet applied.
- **What Envoy can use** (envoy-sse-design "what Envoy asks"):
  - a map as an expedition room: `explore::enter_section(gd_id)` -> carrier stage; every section loaded before the
    first frame (one loader per map at engine start); end event `explore::END == END_EXIT` when the party takes the
    level's way on. Doors and exits are room transitions inside it.
  - a boss: `explore::boss::start(kind, tier, hp_scale)`; the Geno boss lane sets `boss::BOSS_HOOK` and raises
    `END_BOSS` itself. With no hook: logged as blocked, placeholder card, `END_BOSS`.
  - a scene slot: `explore::scene::start(slot)`, `END_SCENE`.
  - enemies: `explore::ENEMY_HOOK(brawl_kind, x, y, facing, section)` at every map spawn point (the Geno enemies);
    the hand-made `subspace` kinds stay the stand-in; pool cap and clear-on-door unchanged.
  - slice 0 primitives: `explore::content_mask()` (bit per GD stage, bit 30 enemy looks; 0 without files) and
    `explore::digest(gd_id)` (FNV-1a of the map's collision as the game gets it). **Not yet wired** into the plan,
    the run record, the replay header, the hash or the netplay AND, and the stage pick is not yet in the replay
    header.
- **Rollback-safety fixes found on the way.** (1) The map state lived in static `OnceLock`s filled by loader
  threads and the loaders printed: a snapshot rewrite of the statics / std's stderr state hung the gate synctest one
  run in three to six. Now one leaked block (thread-local on the game thread), loader notes printed by the game
  thread; 8/8 synctests pass. (2) `melee_engine::logq`: the asset worker's m-ex decode logs (`mexdata: ...`) panicked
  in std's stderr ("RefCell already borrowed") after a rollback load and hung an ACE subspace_check; worker lines are
  now queued and printed by the game thread.
- **Moving platforms.** 060002a's sfyuka00-03 follow st02asuisha's bone path (600 frames, a circle ~60 across):
  `movers_tick` keeps prev_pos (`mpLib_80057424`), writes the points, `mpJointUpdateBounding`. Riding not yet
  checked with a fighter; the wheel model is not drawn (prop rigs carry no textures).
- **Bricks.** GBLK names its brick by model index, not bone (bytes +0x2C/+0x2D: 0x7c..0x8e / 0x65..0x67), and the
  main model has one bone, so the Jungle's bricks have no place: left out (they were invisible collision at the
  origin). Brick models draw until broken where a brick has a place (none on these maps yet).
- **The ~93 ms map load frame** (for the zero-lag lane): file read+parse 47 ms (GrNLa.dat etc., also on FD), draw
  15-17 ms and texture upload 12-15 ms (the map's first draw; FD's own load frame is 74.5 ms), fighter create 7 ms.

## Slice 0 wiring, section warm-up, wheel ride, camp rests (Nov 14, sse3)

- **Slice 0.** `explore::ALLOW` (the session's allowance, `u32::MAX` = this machine's own) gates `available()`, so
  the stage select row, a `--match` pick and Envoy's room all follow it. `content_token()` = `m<mask>.k:<digest32>`
  per stage; `header_token()` = `p<pick>.m<mask>.d<digest>`.
  - netplay: the guest sends its token in the HELLO info (`/x...`, fits the 64-byte field), the host ANDs it with
    its own (`melee_net::app::room::content_and`: a stage bit stays only when both digests match) and dictates
    `;sse=pick.mask` in the agreed scene; both apply it (`apply_netplay`) before the match. No extra = no SSE content.
    The host's pick comes from its `--match` (`jungle`, `lake`/`sse`).
  - replay header: meta `sse` (pick, mask, digest), applied on playback (`apply_header_token`).
  - hash: `coll_override` sets `melee_core::hash::CONTENT_WORD` (pick, mask, digest) on an SSE map, 0 otherwise;
    `melee_modhost::session_hash` mixes it in only when non-zero, so vanilla hashes and goldens are unchanged.
  - Envoy plan words 42/43 (`P_SSE_MASK`, `P_SSE_DIGEST`) stamped at `match_begin`; `run.bin` carries an 8-byte
    trailer (mask, content digest); a saved run made with other content plays its rooms without SSE content.
  - Test knob `MELEE_SSE_TEST_MASK=<hex>` ANDs this machine's mask.
  - Proof (play_pair, 30 ms lag, 3% loss, host `falcon,jungle`): both have it -> Jungle on both, 3037 frames, 0
    hash differences, replay 0 mismatches; guest without the Jungle (mask 40000001) -> FD on both, 2653 frames, 0
    differences; guest with no Brawl files -> mask 0, FD, 3035 frames, 0 differences. Synctest `--match` Jungle and
    Lake with a door at room frame 600: d4 1797 passes / 7188 checks and d7 1794 / 12558, 0 mismatches each.
- **Story slots.** Jungle opening Card, Jungle end Camp (Rest), Lake opening Card, before Rayquaza Camp (Rest),
  after Rayquaza Results. `scene::start` applies `Carry::Rest` (`rest_party`: every fighter's percent and HUD to 0);
  the camp itself is still the placeholder card.
- **Draw / zero lag.** The section models join the draw one every 2 room frames, nearest the party first (no frame
  converts and uploads the whole map: Lake 50 / Jungle 67 textures); the party's section is drawn whole, every other
  one as one triangle per material batch, off camera (`petey_model::SAMPLE`), which keeps its textures in the
  renderer's cache (120-frame keep) so a door uploads nothing. `MELEE_EXPLORE_WARM=all` = the old draw, for A/B.
  Merged `port/zerolag2` (bounded stage wait, carrier hover hint, texture conversion on threads, disc warm).
  Profiles (hidden, real time, real menu walk + every section by `MELEE_EXPLORE_GOTO`, gpu warm list seeded;
  machine NOT quiet, 6 other melee/cargo processes): door and first-draw frames: Lake 7 doors / Jungle 5 doors, none
  over 16 ms with the warm-up (seed run before the warm list: 36-69 ms at doors, from pipelines). Whole match:
  6 frames over 16 ms per map (16.1-18.0 ms, no textures/pipelines/files in them: draw + the busy machine) vs
  150-235 with `all`. **Not under budget: the match-setup tick** (Lake 41 ms work / Jungle 49 ms: two logic
  steps with on_enter 9.8, 3 pipelines that never stay in the warm list 9.5, first section's textures 4.8, frame
  capture 11). The 47 ms parse is gone (GrNLa.dat now an asset apply hit, ~3 ms). Next: a renderer pre-upload entry
  keyed like `tex_key` fed from the map's batches at the stage pick, and why those 3 pipelines are not kept.
- **Wheel ride.** `MELEE_EXPLORE_RIDE=block@frame` sets the first fighter just above a moving block and tracks one
  loop (600 frames). Jungle wheel room (`MELEE_EXPLORE_GOTO=9@200` since the ADSJ chain: arrival points are pseudo doors 15+ on the Lake, 8+ on the Jungle; ride at 300), blocks 0-3: grounded all 601
  frames, worst frame-to-frame step 0.007, place on the platform within 1.33 (one frame of platform travel: the
  fighter follows the platform's previous place). No fall-through, no jitter.

## Match-setup frame (Nov 14, sse3 round 2)

Quiet machine (no other melee/cargo/rustc/link, 3 build slots held), real menu walk, hidden, real time, warm list
persisted (`MELEE_GPUCACHE`). Setup frame, longest tick / game work (ms):

| | before this round | after |
|---|---|---|
| Lake | 41 work (shared machine) | tick 15.9-17.5, work 19.8-24.6 |
| Jungle | 49 work (shared machine) | tick 19.4-22.0, work 28.6-36.6 |
| Final Destination (same walk) | - | tick 12.1-13.3, work 15.8-16.9 |

Every other match frame, doors included, is under 16 ms on both maps.
- Textures: `melee_gx::render::preload` + `gxbridge::preload_tex`; `explore::sss_preload` queues both maps on the
  stage select (the pick is only ~14 frames before the match). Setup-frame texture upload 5.7 ms -> 0.3.
- Pipelines: they are kept. With a warm list that persists, the second run compiles 0 in the whole match. The "3 every
  run" came from a fresh exe folder (the warm list sits beside the exe). `pipe_seed.sh` now seeds `lake` and `jungle`.
- Draw: the party's section goes out as prebuilt display lists (`pm::build_lists`), ~1 ms -> ~0.9 ms incl. state.
- Spread: the enemies and crates of a section come in one a room frame (was all 17 on room frame 2, inside the setup
  tick).
- Still over: the Jungle setup tick, plus the setup frame's work on both maps; FD's own is at the edge (15.8-16.9). The
  setup tick runs a second, catch-up tick inside the same frame. A pacing fix is drafted, not built or measured:
  present after a scene's first tick and run the owed tick in the next frame (`present_now`;
  `scratch/sse3/scene_tick_wip.patch`; its first version stalled the loop because nothing cleared the flag).
  Further work: the VS setup's own on_enter (~9 ms, FD too), and the remaining `explore map draw` state setup.
- Synctest Jungle/Lake with a door at 600: d4 1797 passes / 7188 checks and d7 1794 / 12558, 0 mismatches each.

## Setup-frame pacing (Nov 15, sse4)

`RetailPlay::scene_tick` (`present_now`): the tick that enters a scene presents at once; a tick its time left owing
runs in the next frame instead of behind it in the same one (pacing only; `present` clears the flag, which the first
draft lacked and so stalled the loop). Same quiet-machine walk (`scratch/sse4/quiet.sh`), setup frame, tick / work ms:

| | before (sse3) | after |
|---|---|---|
| Lake | tick 15.9-17.5, work 19.8-24.6 | tick 16.8 (34.1 on the first, cold run), work 23.4 (37.8 cold) |
| Jungle | tick 19.4-22.0, work 28.6-36.6 | tick 17.1-20.3, work 25.2-27.0 |
| Final Destination | tick 12.1-13.3, work 15.8-16.9 | tick 16.1, work 23.3 |

The setup frame is now one tick (`ticks` 1), but that tick alone is still over: `scene on_enter` 8.8-10.6 ms (stage
asset apply 2.3-2.8 ms of a 596 KB plan, 39k events and ~1300 page faults; three fighter creates ~1 ms each), the scene's
first frame (`ENTRY_RUNS_FRAME`: gobj procs and the explore map draw run twice), and frame capture 2.3-4 ms (the
async decode of that frame). The present side adds ~6 ms (prepare 2, draw 1, unaccounted 3.5-4.8). Still open:
spreading on_enter is sim work (the arena apply and fighter creates write game memory), so the remaining lever on the
host side is the capture/decode and the present's unaccounted time. Synctest d4/d7 Jungle and Lake, door at 600:
1797/7188 and 1794/12558, 0 mismatches.

## Match-start texture preloads and EFB reshape (Nov 15, sse5)

Two host-side costs in a match's first frames (`scratch/sse5/quiet.sh`, `an.py`; quiet machine, hidden, real menu walk):

- The SSE maps' preloads (`melee_gx::render::preload`) are queued when the match starts (the walk picks within the
  stage select's first frames, before a map has loaded), and `preload_some` converted ~1 MB of texels on the presenting
  thread each frame: 2.5-4.7 ms of `texture_ms` in ticks 10-23 (Jungle tick 17 34 ms of work with a 30 ms tick). The
  conversion now runs on one thread of its own (`gx-tex-preload`); a frame only creates and uploads converted ones, at
  most `PRELOAD_MS` = 1 ms / 2 MB, and none in a frame that converts a burst of its own (`convert_ahead`).
- The 3.5-4.8 ms "unaccounted" of the setup frame's present was `Presenter::set_content_aspect`: the 4:3 -> 16:9 switch
  made new EFB targets (and freed the old) outside the timed parts. `Renderer::set_target` keeps the replaced targets as
  a spare, and the stage select prewarms the match's shape (`Presenter::prewarm_aspect`, `view::wide_aspect`).

Setup frame (tick / work ms) and the worst other frame, after:

| | setup tick | setup work | setup unaccounted | worst early frame (ticks 9-23) |
|---|---|---|---|---|
| Lake | 18.0 (31.3 cold first run) | 21.5 (34.4 cold) | 3.7-3.9 -> 0.07-0.08 | 15.3 (was 16.4-17.2) |
| Jungle | 19.4-20.9 | 23.5-24.8 | 3.7-4.1 -> 0.08 | under 15.5 (was 15.5-34) |
| Final Destination | 15.8 | 19.5 | 4.2 -> 0.8 | 11.2 |

Still over: the setup tick itself (scene on_enter 9-11 ms of sim work, the frame capture 2-4 ms), and isolated
steady frames at 16-19 ms of work whose tick is 13-15 ms (sim). Synctest d4/d7 Jungle and Lake, door at 600:
1797/7188 and 1794/12558, 0 mismatches.

## ADSJ chain proofs (Nov 15, ssechain)
- `MELEE_EXPLORE_GOTO` targets a door index; arrival points are now pseudo doors after the real ones (`chains_link
  --nocapture` prints them). Every section: Lake `17@300,16@450,20@600,15@750,23@900,27@1050,14@1200`
  (080103a, 080103b, 080104b, 080104, 080104a, 080105, 080105a); Jungle `8@300,9@450,10@600,12@750,13@900`.
- Lake story walk (`MELEE_EXPLORE_OPEN`): 080103 door 1 -> 080103b, door 5 (b1) -> 080104b, door 12 (b2) -> 080105,
  door 13 clear. Door 6 (08010400) leads back to 080103. Jungle: doors 0, 2, 4, 6 (clear).
- SSE 16 / 22 / 27 real-menu walks (`walk_vs_sse16|22|27_2cpu.txt`, 5th GD icon now shown): door 0, then the clear
  door (16: door 1; 22, 27: door 2) takes the way on. SyncTest d4 / d7 on SSE 22 through its door: 0 mismatches.
