# Subspace Emissary audit (Brawl `stage/adventure`, read-only)

Scope: the 271 SSE map files (`LLSSNN[a].pac`) plus `adventure_common.arc` on the owner's disc, compared with our
Rust port (`melee_modes/src/tu/explore.rs`). Only names, counts and ids here. Nothing from the disc is copied.

Provenance tags used below:
* **[file]**: counted by a throwaway python reader (same ARC / LZ walk as `petey_model::pac_leaves`, same collision
  test and BLOC sub-block walk as `explore.rs`). Enemy kind = u16 at GEG1 entry +0x1C, door type = byte at GDOR
  entry +0x44.
* **[pub]**: public knowledge. Enemy id -> name table and the BLOC sub-block names/categories come from BrawlCrate's
  `BrawlLib/SSBB/ResourceNodes/Subspace/*` (enum `EnemyList` in `Hazards/GEG1Node.cs`). Story names and order are
  common knowledge of the game.
* **[inferred]**: level-prefix -> story-name mapping, reasoned from bosses and enemy mix; not read from any table.

## 0. Headline findings

* 271 map files in 36 level prefixes; 1018 collision files in total (avg ~3.8 per file) [file].
* 2392 GEG1 enemy generators [file]; **46 regular enemy kinds** (Gamyga's 4 base parts and Petey..Tabuu
  excluded) plus **11 boss kinds** (Petey Piranha, Rayquaza, Porky, Galleom, Ridley, Duon, Meta Ridley, Falcon
  Flyer (Meta Ridley's set piece), Tabuu, Master Hand, Crazy Hand) [file + pub]. We have 4 kinds.
* **71 distinct BLOC sub-block tags** in use [file]; excluding GEG1/GFG1/EPSP/EPBM (spawning) about **67 gimmick /
  trigger / camera / sound kinds**. We read 2 (GDOR, GEG1 positions).
* **Naming check, please verify:** prefix `08` holds Rayquaza (`080001`), and the boss-battle files `92xx01` are in
  boss story order Petey(03) -> Rayquaza(08) -> Porky(09) -> Galleom -> Galleom -> Ridley -> Duon -> Meta Ridley ->
  Tabuu. Rayquaza is the boss of **The Lake**, so `080103` is most likely **The Lake, step 3**, and **The Jungle is
  prefix `06`** (Goomba/Koopa/Hammer Bro/Bullet Bill/Giant Goomba, vertical shaft `060004`) [inferred]. This doc
  keeps calling `080103` "our map" and gives both readings where it matters.
* Door type values seen: 0 normal (most), 1 (one per level, the level-end / yellow door), 2 arrow (08 only),
  3 and 5 (Great Maze hubs) [file; meaning of 1/3/5 from community notes, unverified].
* Bullet Bill sits in 9 files with ~79 generators; Goomba/Koopa-family enemies cluster in 06, 08, 20, 27.

## 1. Levels in story order

Mapping [inferred] (boss anchors are solid, the rest from enemy mix; `Lake`/`Jungle` per the check above):

| story level [pub] | prefix [inferred] | files [file] | bosses [file] | notes |
|---|---|---|---|---|
| Midair Stadium | 01 (opening fight), 03 | 1 + 2 | Petey Piranha (030101) | 010001 fighter fight (GFG1); 030001 Primid arena (EPSP waves) |
| Skyworld | 04 | 4 | - | Greap, Primids, Glunder, Jyk; GLOK/GFRT triggers |
| Sea of Clouds | 05 | 4 | - | Feyesh, Poppant, Mite, Towtow; GBLK, GLAD, GFRK |
| The Jungle | 06 | 6 | - | Goomba 39, Koopa 22, Bullet Bill 17, Hammer Bro 7, Giant Goomba 1, Jyk 10; GMPS 12 moving platforms, GBC1/2 |
| The Plain | 07 | 3 | - | Borboras, Feyesh, Ticken, Fire Primid 11; GBLK 13 |
| The Lake | 08 | 13 | Rayquaza (080001) | our map lives here (080103/080103a) |
| The Ruined Zoo | 09 | 6 | Porky (090101) | Bytan 11, Floow, Ticken; GWAT 8 water, GSPR |
| The Battlefield Fortress | 10 | 9 | (Galleom file not here, see 24) | Sword/Scope Primids, Roader, Autolance, Armight; GTRR/GTRC 16, GSTG |
| The Forest | 12 | 4 | - | Puppit, Trowlon, Shellpod; GMPS 12, GSBK/GMCA switch kit |
| The Research Facility I | 14 | 8 | - | R.O.B. Sentry/Blaster/Distance/Launcher; GDBF in every file; 140104 has 27 coll files / 26 GMOV |
| The Lake Shore | 16 | 10 | - | 4 fighter-fight files (GFG1); Bombed 18, Shaydas; GTIK 30 |
| The Path to the Ruins | 18 | 5 | - | Gamyga (+4 base parts), Glire, Nagagog; GFPR 21 |
| The Cave | 20 | 5 | - | Goomba/Koopa/Hammer Bro, Glire 9; GHPE 6 |
| The Ruins | 22 | 6 | - | Borboras, Floow, Mite, Bytan, Shaydas; GSHT 12 |
| The Wilds I / II | 24 (+25) | 6 + 1 | Galleom (240101, 250001) | Gamyga, Trowlon, Roader; GBC2 14 |
| The Ruined Hall / Swamp | 26 | 3 | - | Feyesh, Shellpod, Puppit, Trowlon; GWND 3 |
| The Swamp / Glacier side | 27 | 8 | - | Bullet Bill 37, Koopa 37, Hammer Bro 11; GSPR 35, GWAT 22, GMVA 13 |
| The Research Facility II | 28 | 10 | Ridley (280301) | R.O.B.s, Glice, Cymul 12; GWAP 12, GMRL 8 |
| The Glacier Peak / The Canyon | 29, 30, 31, 32 | 3, 1, 6, 1 | - | 30001/32xx are EPSP wave arenas; Glice 11 (310003) |
| Battleship Halberd Exterior/Bridge/Interior | 33, 34, 35 | 9, 5, 1 | Duon (350001) | GDBF everywhere; 330201 fighter fight |
| The Subspace Bomb Factory I / II | 36, 37 | 5, 10 | Meta Ridley + Falcon Flyer (370301) | GBLT 8, GMIN 10, R.O.B. Launchers |
| Entrance to Subspace | 39 | 1 | - | Big Primid 5, Armank 2 |
| Subspace I / II | 40, 41 | 10, 3 | - | Bombed, Glice, Mite, Gamyga; GTIK 41, GMOV 28 (400004) |
| The Great Maze | 42 | 88 | Petey..Tabuu boss rematches inside | 416 GEG1 over 58 kinds, 160 doors, 465 GFG1, 9 GSAV save points |
| Tabuu | 92 0801 | 1 | Tabuu | |
| Boss Battles / extra | 90, 91, 92 | 3, 1, 9 | Master Hand, Crazy Hand; boss rematches | 910101 = 90 GNDV, likely a menu/hub map |

Per-file detail (every file except the 87 small Great Maze rooms; 420001a shown as their typical shape) is the table
in the appendix: collision files, MDL0 count, door types, enemy kinds with counts, bosses, and every other BLOC
sub-block with its count [file].

### Gimmick kinds (BLOC sub-blocks) [file counts; category from BrawlCrate [pub], meaning where public]

* Navigation: GDOR doors, GMSJ (level jump/end), GSCN (scene change), GWAP (warp), ADSJ.
* Spawning: GEG1 enemy generators, GFG1 fighter (CPU / ally) spawns, EPSP/EPBM enemy spawn points / boss marks.
* Objects: GBLK breakable blocks, GIB2 item boxes / crates, GITM items, GITR item roll, GLAD ladders, GMPS/GPS2 moving
  platforms, GMWA/GMWO walls / switch-gated walls, GSBK switch blocks, GCAT, GSTG, GTRR/GTRC trains/tracks,
  GBC1/GBC2 cannons (barrel / Bullet Bill style), GBLT conveyor belts, GSHT shutters, GSPR springs, GFIM, GELA/GELE
  elevators, GMTA, GMRL, GBRF, GSAV save point, GDBF/GDBS (Halberd / factory doors), GDTP.
* Hazards: GWAT water, GWND/GWD2 wind, GFRK/GFPR falling rocks / fire pillars, GHPE, GKNG, GMCA, GMEA, GENC, GFG1.
* Animation: GMOV moving map parts, GMV4/GMVA, GMOT model motion, GLOK/GLK2 lock / camera lock, GQUK quake.
* Triggers: GET1/GET2 area triggers, GEPT enemy-pattern trigger, GFT2/GFRT, GASD, GMOA.
* Camera: GCAM, GCMP, GFSR (fixed / scroll range). Sound: GSND, BGMG. Graphics: GEFF, GNDV. Other: GDMD, GFIN,
  GMIN, GTIK.

Cutscene triggers: Brawl's SSE cutscenes are movie files played between steps [pub]; inside maps they are driven by
the step tables and the trigger blocks (GET1/GET2/GMSJ/GSCN), not a dedicated tag [inferred].

## 2. `080103` ("our map") in detail, Brawl vs port

Level `08` files [file]:

| file | coll | MDL0 | doors | enemies | other |
|---|---|---|---|---|---|
| 080001 | 1 | 11 | - | Rayquaza (boss) | GWD2 |
| 080101 | 3 | 23 | arrow 4, normal 1 | Auroros 8, Goomba 7, Koopa 4, Towtow 3, Hammer Bro 2 | GBLK, GMOV 2, GIB2, GET1 5, GET2 13 |
| 080102 | 6 | 44 | arrow 3 | Goomba 6, Hammer Bro 4, Auroros 3, Koopa 2, Giant Goomba 2, Borboras 1 | GSBK 9, GMCA 9, GQUK, GLAD, GMWO, GSCN |
| **080103** | **3** | **15** | **normal 2, arrow 1** | **Bucculus 4, Mite 4** | **GMWA 1, GMOV 2, GIB2 1, GET1 4, GET2 4, EPSP 1** |
| **080103a** | **1** | **8** | **normal 1** | - | **GDMD, GMOT, GASD** |
| 080103b | 1 | 4 | normal 1, arrow 1 | - | - |
| 080104 | 2 | 17 | normal 2, arrow 1 | Goomba 3, Giant Goomba 1 | GQUK 5, GFT2 6, GMWO, GLOK, GEFF |
| 080104a / b | 1 / 1 | 8 / 7 | 1 / 3 | - / Towtow 1 | GDMD,GMOT / GFT2,GEPT |
| 080105 | 4 | 36 | normal 1 | Bullet Bill 12, Hammer Bro 3, Koopa 3, Goomba 2, Giant Goomba 2, Jyk 1 | GWAT 1, GMPS 1, GMSJ (end) |
| 080105a | 4 | 15 | normal 1 | - | GMPS 2, GBLK 3, GIB2 2 |
| 080201 | 3 | 21 | - | Koopa 8, Goomba 3, Hammer Bro 2 | GBC1, GCMP, GBLK |
| 080301 | 1 | 2 | - | - | GFG1 15 (fighter fight) |

| aspect | Brawl (`080103` + rooms) | our port |
|---|---|---|
| geometry | 15 MDL0 in `080103`, 8 in `a`, 4 in `b`; 38 CHR0 animations [file] | main map model only, bind pose; no CHR0/SRT0 playback; `080103b` not loaded |
| collision | 3 files in `080103` (+1 per room); GMOV parts carry their own collision [file/inferred] | largest file only, welded static; no moving collision |
| doors | 3 + 1 + 2 GDOR (normal 4, arrow 2) | GDOR read, hand-linked (`LINKS`), up tilt + fade; Brawl's step/jump tables not read; `b` room missing |
| enemies | 8 GEG1: Bucculus x4, Mite x4 (Mite = swarm generator) | first 4 GEG1 of section 0 only, one stub kind (Goomba); kinds at +0x1C ignored |
| items / crates | 1 GIB2 (item box/crate gen) in `080103` | none |
| moving parts | GMOV 2, GMWA 1 (switch wall), GMOT in hut | none |
| triggers | GET1 4, GET2 4, EPSP 1, GASD in hut | none |
| camera | no GCAM here; Brawl SSE camera follows player inside scroll limits (GFSR elsewhere) | section extent |
| music / sound | level BGM + map sfx via sound bank (not in BLOC except GSND in some maps) [pub] | none; Brawl BRSTM/RWSD decoding exists in `melee_audio/src/brawl` but unwired |

If `08` is in fact The Lake (see section 0), The Jungle proper is `060001 060002 060002a 060003 060004 060004a`:
6 files, 22 collision files, Goomba 39 / Koopa 22 / Bullet Bill 17 / Hammer Bro 7 / Jyk 10 / Giant Goomba 1, 12 GMPS
moving platforms, 9 cannon blocks (GBC1/2), 8 GBLK, 9 GIB2, 3 GITM [file].

## 3. What is missing to play "very similarly" to Brawl

1. Enemy roster: 42 of 46 regular kinds missing, and every boss except our Petey arena; per-GEG1 kind, count, respawn timers and
   trigger links (EPSP/GET2) rather than "first 4 points".
2. Bullet Bill explosion; Mite/Bucculus-style swarm and water enemies needed right in our own map.
3. All collision files per section and moving collision (GMOV/GMPS/GELA/GLAD), plus water (GWAT), wind, ladders.
4. Animated scenery (CHR0/SRT0/CLR0 on map models), GMOT/GMOV motion.
5. Items: GIB2 crates/boxes, GITM pickups, trophies/stickers drops.
6. Triggers and progression: GET1/GET2 areas, switches (GSBK/GMWA/GMWO), level end (GMSJ), Brawl step/door tables
   instead of hand links; cutscene movie hand-off between steps.
7. Camera: Brawl's follow camera with scroll limits / fixed zones (GFSR/GCMP/GCAM).
8. Audio: level BGM and map/enemy sfx through the existing Brawl decoder.
9. Missing rooms (e.g. `080103b`), fighter fights (GFG1), save points (GSAV) and the team/stock flow.

## 4. Prioritized plan

Next level: **The Plain** in story order. Under the mapping above our `08` map is The Lake, so the true order is
Jungle(06) -> Plain(07) -> Lake(08) -> Ruined Zoo(09). Recommended (cheapest playable sequence):

1. Read GEG1 kind ids (u16 +0x1C) and spawn every point, mapping unsupported kinds to the nearest of our 4 (log the
   miss). Zero new art; makes every map populated. Add Bullet Bill's explosion.
2. Load all collision files per section (the `whole` path in `load_secs` already welds them) and read GMSJ as the
   level exit; read Brawl's door targets instead of hand `LINKS`.
3. GIB2 crates with Melee items (a crate is a crate); GBLK breakable blocks.
4. Make `06` (The Jungle) playable end to end: 6 files, kinds Goomba/Koopa/Hammer Bro/Bullet Bill/Giant Goomba/Jyk,
   so only Koopa, Hammer Bro, Jyk are new; GMPS moving platforms are the one gimmick to build.
5. `07` The Plain: 3 files; new kinds Borboras, Feyesh, Ticken, Fire Primid, Boomerang Primid (Primid variants are
   cheap: same body, one attack each); GBLK, GMOV, GFRT.
6. `08` The Lake around our current map: add Bucculus, Mite, Auroros, Towtow; arrow doors done; GWAT water; then
   Rayquaza (`080001`) as the second boss after Petey.
7. Then CHR0 map animation, follow camera, BGM. Then Ruined Zoo (09: Bytan, Floow, water, Porky).

Cheapest path in one line: generic GEG1-kind spawning + all collision + crates + GMSJ exit gets 06 -> 07 -> 08
playable with 3 + 5 + 4 new enemy kinds and two gimmicks (moving platforms, water).

## 5. Quantities

| what | Brawl SSE | port |
|---|---|---|
| map files | 271 (36 prefixes) | 3 (`080103`, `080103a`, `030101`) |
| collision files | 1018 | 1 per section |
| GEG1 enemy generators | 2392 | 4 used |
| regular enemy kinds | 46 (+ 4 Gamyga base parts) | 4 (Goomba, Primid, Bullet Bill, Giant Goomba) |
| bosses | 11 kinds (9 story encounters + Master/Crazy Hand) | 1 (Petey arena) |
| BLOC sub-block kinds | 71 (~67 gimmick/trigger/camera/sound) | 2 |
| door types seen | 5 values (0, 1, 2, 3, 5) | GDOR read, link by hand |

## Appendix: per-file counts [file]

Door types: normal 0, yellow 1, arrow 2, t3/t5 (maze hubs). The 87 other Great Maze rooms (`4200xx`, `42004x-42008x`,
`420101`) are omitted; level 42 totals are in section 1.

| file | coll files | MDL0 | doors (type:n) | enemies (GEG1 kind x n) | bosses | other BLOC sub-blocks (tag x n) |
|---|---|---|---|---|---|---|
| 010001 | 2 | 5 |  |  |  | GEFF 2, GFG1 15, GMOT 1, GSND 1 |
| 030001 | 2 | 6 |  | Primid 15, Spaak 6, BoomerangPrimid 5, SwordPrimid 4 |  | EPSP 10, GEPT 2, GMOT 1, GSND 1 |
| 030101 | 2 | 12 |  |  | PeteyPiranha | GET1 1, GMOT 1, GSND 1 |
| 040001 | 2 | 12 | normal:1 |  |  | GET1 7, GIB2 3, GLK2 1, GMOV 1, GNDV 6 |
| 040101 | 7 | 27 | normal:1 | Primid 8, Spaak 4, Greap 1, Cymul 1, BoomerangPrimid 1 |  | EPSP 8, GASD 2, GEPT 1, GET1 3, GET2 6, GFT2 2, GIB2 2, GLOK 2, GMOT 4, GMOV 1 |
| 040201 | 6 | 48 | normal:1, yellow:1 | Glunder 12, Jyk 9, BoomerangPrimid 6, Spaak 6, Primid 5, Roturret 1 |  | EPSP 6, GASD 1, GCAT 1, GEPT 1, GET1 6, GET2 29, GFRT 3, GFSR 1, GLOK 1, GMOT 1, GMOV 6, GNDV 4 |
| 040201a | 3 | 14 | normal:1 |  |  | GASD 1, GET1 1, GFSR 1, GIB2 2, GLK2 1, GMOT 1, GMOV 2 |
| 050001 | 1 | 8 |  |  |  | GASD 2, GET1 1, GMOV 1, GMSJ 1, GWD2 1 |
| 050102 | 3 | 29 | normal:2 | Feyesh 8, Poppant 5, Primid 4, Glunder 4, BoomerangPrimid 3, Towtow 1 |  | EPSP 3, GBLK 6, GET1 7, GET2 19, GIB2 1, GITM 1, GMOV 1 |
| 050102a | 2 | 6 | normal:1 |  |  | GIB2 2 |
| 050103 | 8 | 59 | yellow:1 | Mite 14, Glunder 7, Spaak 5, Feyesh 4, Greap 2, BoomerangPrimid 2, Poppant 2 |  | EPSP 7, GASD 4, GBLK 1, GENC 2, GEPT 1, GET1 8, GET2 20, GFRK 4, GFRT 6, GFT2 11, GIB2 2, GLAD 2, GLOK 3, GMOT 10, GMOV 6, GMVA 3 |
| 060001 | 4 | 49 | normal:1 | Goomba 14, Koopa 11 |  | EPSP 2, GASD 1, GBC1 8, GBC2 8, GBLK 1, GET2 25, GIB2 3, GMOV 1, GMWA 1 |
| 060002 | 7 | 57 | normal:2 | Goomba 17, Koopa 10, HammerBro 3, BulletBill 2 |  | EPBM 1, EPSP 5, GBC1 1, GBLK 6, GEPT 1, GET1 4, GET2 13, GFRT 1, GFT2 1, GIB2 2, GITM 2, GLAD 2, GLOK 1, GMOT 1, GMOV 3, GMPS 12, GMWA 1, GPS2 1 |
| 060002a | 3 | 12 | normal:1 | Goomba 2, Koopa 1 |  | GET1 3, GIB2 2, GMOT 1, GSND 1 |
| 060003 | 2 | 28 | normal:1 | BulletBill 15, HammerBro 4, Goomba 3 |  | EPBM 1, EPSP 4, GET1 1, GET2 11, GFSR 1, GITM 1, GMOV 1 |
| 060004 | 1 | 24 | normal:1, yellow:1 | Jyk 10, Goomba 3, GiantGoomba 1 |  | EPSP 1, GASD 1, GCAT 1, GEFF 1, GET1 1, GET2 10, GLOK 1, GMOV 2, GSND 3 |
| 060004a | 3 | 8 | normal:1 |  |  | GIB2 2, GSND 1 |
| 070001 | 8 | 50 | normal:2 | Feyesh 7, Primid 7, Borboras 6, BoomerangPrimid 5, Ticken 4, Jyk 3, Poppant 2 |  | EPSP 7, GBLK 9, GET1 9, GET2 20, GFRT 4, GIB2 2, GMOT 2, GMOV 7 |
| 070001a | 3 | 11 | normal:1 |  |  | GBLK 4, GET1 1, GIB2 1, GITM 2, GSND 1 |
| 070002 | 2 | 31 | yellow:1 | FirePrimid 11, Primid 3, Spaak 3, BoomerangPrimid 2, Borboras 2, Glunder 2 |  | EPSP 9, GEPT 2, GET1 3, GFSR 4, GFT2 1, GITM 1, GLOK 4, GMOV 1, GMSJ 1, GMV4 1 |
| 080001 | 1 | 11 |  |  | Rayquaza | GET1 1, GWD2 1 |
| 080101 | 3 | 23 | normal:1, arrow:4 | Auroros 8, Goomba 7, Koopa 4, Towtow 3, HammerBro 2 |  | EPSP 3, GBLK 1, GET1 5, GET2 13, GIB2 1, GMOV 2 |
| 080102 | 6 | 44 | arrow:3 | Goomba 6, HammerBro 4, Auroros 3, Koopa 2, GiantGoomba 2, Borboras 1 |  | EPSP 3, GBLK 1, GET1 1, GET2 16, GFT2 2, GIB2 1, GLAD 1, GMCA 9, GMOV 2, GMWO 1, GQUK 1, GSBK 9, GSCN 1 |
| 080103 | 3 | 15 | normal:2, arrow:1 | Bucculus 4, Mite 4 |  | EPSP 1, GET1 4, GET2 4, GIB2 1, GMOV 2, GMWA 1 |
| 080103a | 1 | 8 | normal:1 |  |  | GASD 1, GDMD 1, GMOT 1 |
| 080103b | 1 | 4 | normal:1, arrow:1 |  |  |  |
| 080104 | 2 | 17 | normal:2, arrow:1 | Goomba 3, GiantGoomba 1 |  | EPSP 1, GASD 2, GEFF 1, GET1 1, GFT2 6, GLOK 1, GMOV 2, GMWO 1, GQUK 5 |
| 080104a | 1 | 8 | normal:1 |  |  | GASD 1, GDMD 1, GMOT 1 |
| 080104b | 1 | 7 | normal:2, arrow:1 | Towtow 1 |  | GASD 1, GEPT 1, GFT2 1 |
| 080105 | 4 | 36 | normal:1 | BulletBill 12, Koopa 3, HammerBro 3, Goomba 2, GiantGoomba 2, Jyk 1 |  | EPSP 3, GASD 1, GET1 4, GET2 22, GFSR 2, GIB2 2, GMOV 2, GMPS 1, GMSJ 1, GWAT 1 |
| 080105a | 4 | 15 | normal:1 |  |  | GBLK 3, GET1 1, GIB2 2, GMPS 2 |
| 080201 | 3 | 21 |  | Koopa 8, Goomba 3, HammerBro 2 |  | EPBM 1, EPSP 1, GASD 1, GBC1 1, GBLK 1, GCMP 1, GET1 2, GET2 12, GFSR 1, GFT2 1, GIB2 1, GMOV 1, GMSJ 1 |
| 080301 | 1 | 2 |  |  |  | GFG1 15 |
| 090001 | 4 | 37 |  | Primid 8, Buckot 3, BoomerangPrimid 2, Towtow 1 |  | EPSP 1, GBLK 11, GET1 1, GET2 10, GFSR 1, GKNG 1, GMOV 3, GMSJ 1, GMWA 3, GWAT 1 |
| 090101 | 2 | 6 |  |  | Porky | GET1 1 |
| 090201 | 12 | 37 | normal:1 | Primid 10, Auroros 9, Floow 9, BoomerangPrimid 5 |  | EPSP 7, GBLK 3, GDMD 1, GEPT 2, GET1 5, GET2 14, GFT2 2, GLAD 2, GLOK 1, GMOT 5, GMOV 5, GMTA 5, GWAT 3 |
| 090202 | 8 | 30 | normal:1 | Bytan 11, Glunder 2, Jyk 1, Borboras 1 |  | EPSP 1, GELA 3, GENC 1, GEPT 1, GET1 1, GET2 6, GFT2 2, GIB2 1, GLAD 1, GLOK 1, GMOT 2, GMOV 1, GMWA 1, GWAT 2 |
| 090203 | 5 | 23 | normal:1, yellow:1 | Ticken 8, FirePrimid 4, Floow 4, BoomerangPrimid 3 |  | EPSP 5, GDMD 1, GEPT 1, GET1 7, GET2 8, GFT2 3, GIB2 2, GLAD 1, GLOK 1, GMOT 2, GMSJ 1, GMTA 4, GSPR 7, GWAT 2 |
| 090203a | 6 | 16 | normal:1 |  |  | GBRF 2, GDMD 1, GIB2 2, GMOT 1 |
| 100001 | 6 | 29 | normal:2 | SwordPrimid 11, Primid 6, Armight 5, Jyk 3, Borboras 1 |  | EPBM 2, EPSP 5, GBRF 1, GET1 10, GET2 13, GIB2 1, GITM 1, GITR 1, GLAD 1, GMOA 1, GMOV 3, GMWA 1 |
| 100001a | 3 | 15 | normal:1 | Jyk 2 |  | EPSP 1, GIB2 2 |
| 100002 | 2 | 13 | normal:1 | Roader 8, Spaak 3, SwordPrimid 2, Primid 1, Borboras 1 |  | EPSP 2, GET2 11, GMOV 1, GMWA 2 |
| 100101 | 1 | 13 |  | SwordPrimid 11, ScopePrimid 8, Primid 5, Buckot 5, Greap 1 |  | EPSP 9, GEPT 7, GSND 1, GWND 1 |
| 100201 | 3 | 39 | normal:1 | Primid 10, SwordPrimid 7, Autolance 6, Feyesh 6, ScopePrimid 3, Armight 2, Poppant 1, Greap 1 |  | EPBM 1, EPSP 9, GEPT 5, GET1 6, GET2 7, GFSR 1, GIB2 1, GITM 3, GLOK 5, GMOT 1, GMOV 2 |
| 100202 | 10 | 45 | normal:3 | SwordPrimid 9, Buckot 5, Primid 4, FirePrimid 2, Towtow 2, ScopePrimid 1 |  | EPSP 3, GASD 1, GDMD 1, GEPT 1, GET1 1, GET2 17, GFT2 2, GIB2 1, GLOK 1, GMOT 2, GMOV 11, GMTA 1, GSTG 3, GTRC 6, GTRR 10 |
| 100202a | 4 | 15 | normal:1 |  |  | GBLK 1, GDMD 1, GIB2 2, GTRC 1 |
| 100203 | 1 | 2 |  |  |  | GET1 1, GMSJ 1 |
| 100205 | 2 | 20 | yellow:1 | SwordPrimid 8, Buckot 4, Primid 3, Feyesh 3, Nagagog 3, Roader 1 |  | EPSP 4, GEPT 4, GET1 3, GET2 6, GFT2 1, GIB2 1, GLOK 1, GMOV 1, GMSJ 1 |
| 120001 | 10 | 28 | normal:3 | Puppit 9, BoomerangPrimid 5, Primid 4, Bucculus 3, SwordPrimid 1 |  | EPSP 7, GEPT 3, GET1 6, GET2 10, GFT2 4, GIB2 2, GLOK 2, GMOT 5, GMOV 3 |
| 120001a | 4 | 17 | normal:1 | Jyk 1 |  | EPSP 1, GET2 1, GIB2 2 |
| 120002 | 10 | 40 | normal:1 | Trowlon 6, Puppit 5, Primid 4, Ticken 3, Auroros 3 |  | EPSP 4, GASD 1, GET1 3, GET2 16, GFT2 4, GIB2 4, GLOK 1, GMCA 4, GMOT 2, GMOV 1, GMWO 3, GQUK 3, GSBK 4, GSPR 6, GWAT 1 |
| 120003 | 10 | 48 | yellow:1 | Feyesh 9, FirePrimid 6, Shellpod 4, Mite 4 |  | EPBM 1, EPSP 7, GBLK 4, GEPT 1, GET1 8, GET2 11, GFT2 3, GIB2 2, GLAD 1, GLOK 1, GMOT 1, GMOV 5, GMPS 12, GMSJ 1, GMWA 1, GSPR 3 |
| 140001 | 2 | 7 |  |  |  | GBLK 1, GDBF 1 |
| 140005 | 4 | 16 |  | ROBSentry 8, ROBBlaster 6 |  | EPSP 1, GBLK 3, GDBF 2, GELE 1, GET1 2, GET2 9, GIB2 1 |
| 140101 | 1 | 8 |  | ROBSentry 4, ROBBlaster 2, ROBDistance 1 |  | EPSP 1, GDBF 1, GET1 1, GET2 6 |
| 140102 | 4 | 27 |  | ROBSentry 5, ROBBlaster 1 |  | EPSP 2, GASD 1, GBLK 3, GDBF 4, GET2 6, GIB2 1, GLAD 3, GMOV 7 |
| 140103 | 6 | 40 |  | ROBSentry 9, Roturret 4, ROBBlaster 1, Buckot 1, ROBDistance 1 |  | EPSP 4, GCAT 3, GDBF 1, GET1 7, GET2 5, GFT2 5, GIB2 1, GITM 2, GITR 2, GLOK 1, GMOA 2, GMOV 9, GMWA 1 |
| 140104 | 27 | 75 |  | Jyk 6, Glunder 1 |  | GASD 1, GCMP 3, GDBF 1, GEPT 3, GET1 1, GET2 5, GFSR 1, GFT2 6, GLOK 1, GMOV 26, GMWA 1, GNDV 1, GQUK 5 |
| 140105 | 3 | 17 |  | Bytan 8, ROBLauncher 2 |  | GASD 2, GDBF 2, GET1 2, GET2 6, GIB2 1, GLAD 2, GMOV 1 |
| 140106 | 6 | 32 |  | MetalPrimid 4, Bytan 3, ROBLauncher 3 |  | EPBM 1, EPSP 3, GDBF 2, GELA 3, GET1 4, GET2 3, GFT2 2, GIB2 1, GLAD 1, GLOK 1, GMIN 7, GMOV 3, GNDV 1 |
| 160001 | 1 | 3 |  |  |  | GFG1 15 |
| 160002 | 1 | 3 |  |  |  | GFG1 15 |
| 160101 | 1 | 3 |  |  |  | GFG1 30 |
| 160102 | 1 | 3 |  |  |  | GFG1 30 |
| 160201 | 3 | 42 | normal:1 | Bombed 18, BulletBill 6, Ticken 2, Spaak 2, Mite 2, Greap 1 |  | EPBM 1, EPSP 2, GASD 1, GEPT 4, GET1 5, GET2 14, GFSR 2, GIB2 1, GLOK 1, GMOT 1, GMOV 3, GNDV 2 |
| 160202 | 7 | 36 | normal:1 | Jyk 5, Bucculus 5, Mite 3 |  | EPBM 1, EPSP 2, GCAT 1, GET1 2, GET2 14, GFRT 4, GFSR 1, GIB2 1, GMOV 8, GTIK 16 |
| 160301 | 16 | 70 | normal:3 | Mite 8, Shaydas 7, Feyesh 6, Ticken 3 |  | EPBM 1, EPSP 4, GASD 4, GBLK 1, GDMD 1, GEPT 2, GET1 7, GET2 10, GFPR 1, GFRT 5, GFT2 2, GIB2 1, GITM 3, GITR 3, GLOK 2, GMCA 4, GMOA 3, GMOT 2, GMOV 16, GMTA 5, GMWO 3, GQUK 3, GSBK 4, GSTG 1, GTRC 1, GWAT 1 |
| 160301a | 3 | 9 | normal:1 |  |  | GASD 2, GIB2 2, GWAT 3 |
| 160301b | 5 | 13 | normal:1 |  |  | GASD 1, GBLK 1, GIB2 2, GTIK 14, GWAT 1 |
| 160301c | 3 | 9 | yellow:1 |  |  | GASD 1, GIB2 2 |
| 180001 | 13 | 44 | normal:1 | Puppit 8, Jyk 8, Primid 3, Shellpod 3, BoomerangPrimid 2, Gamyga 2, Borboras 1 |  | EPSP 4, GBLK 1, GDMD 1, GELE 1, GEPT 3, GET1 8, GET2 11, GIB2 2, GLOK 1, GMOT 2, GMSJ 1, GTIK 18 |
| 180001a | 2 | 11 | normal:1 |  |  | GBLK 6, GET1 1, GITM 4 |
| 180002 | 10 | 61 | normal:1 | Glire 6, FirePrimid 6, Armight 5, Nagagog 3, Primid 2, Puppit 2, ScopePrimid 1 |  | EPSP 4, GDMD 3, GELE 1, GEPT 1, GET1 10, GET2 5, GFPR 21, GFRT 6, GFSR 1, GFT2 2, GIB2 3, GITM 1, GLOK 4, GMEA 6, GMOT 3, GMOV 9, GMPS 2, GMV4 3, GMWA 4 |
| 180003 | 1 | 4 |  |  |  | GET1 1, GMSJ 1 |
| 180101 | 1 | 2 |  |  |  | GFG1 15 |
| 200001 | 13 | 51 | normal:2 | Goomba 13, Koopa 5, HammerBro 4 |  | EPBM 1, EPSP 3, GET1 12, GET2 11, GFRT 3, GFT2 2, GIB2 2, GLAD 3, GLOK 1, GMOT 8, GMOV 9, GMVA 1 |
| 200001a | 5 | 13 | normal:1 |  |  | GIB2 2, GMOT 1, GMOV 2 |
| 200002 | 3 | 10 | normal:1 |  |  | GFPR 2, GFRK 1, GITM 1, GITR 1, GMOA 1, GMOV 1 |
| 200003 | 12 | 68 | normal:1, yellow:1 | Koopa 9, Glire 9, BulletBill 7, GiantGoomba 3, Bucculus 2, HammerBro 2, Shellpod 1 |  | EPSP 1, GBRF 4, GELA 4, GET1 9, GET2 14, GFT2 3, GHPE 6, GLAD 1, GLOK 2, GMOT 5, GMOV 10, GMVA 1, GMWA 6, GTIK 5 |
| 200003a | 5 | 11 | normal:1 |  |  | GBRF 1, GIB2 2, GTIK 10 |
| 220001 | 16 | 62 | normal:1 | Borboras 8, Floow 8, Mite 8, Glire 2, Jyk 1, Roturret 1 |  | EPSP 3, GASD 2, GDMD 1, GELE 1, GET1 7, GET2 24, GFRK 6, GFT2 6, GIB2 2, GITM 1, GMOT 8, GMOV 6, GMTA 2, GMV4 2, GMWA 5, GNDV 3, GTRC 1, GWAT 1 |
| 220002 | 4 | 63 | normal:2 | Mite 9, Glire 4, Bytan 4, MetalPrimid 3, Jyk 2, FirePrimid 2 |  | EPBM 2, EPSP 4, GBLK 3, GDMD 1, GEFF 2, GEPT 2, GET1 9, GET2 29, GFPR 10, GFSR 1, GFT2 4, GIB2 1, GMOV 2, GMSJ 1, GMTA 2, GMVA 13, GTRC 1 |
| 220002a | 4 | 14 | normal:1 |  |  | GASD 1, GIB2 2, GMOV 6 |
| 220003 | 6 | 62 | normal:2 | Bytan 6, Armight 5, Shaydas 4, Borboras 3, Towtow 2, Autolance 1, Roturret 1 |  | EPBM 1, EPSP 1, GDMD 1, GET1 4, GET2 9, GFPR 4, GFSR 4, GFT2 2, GIB2 2, GLOK 3, GMOV 5, GMV4 1, GMWA 13, GSHT 10 |
| 220003a | 5 | 12 | normal:1 |  |  | GDMD 1, GIB2 2, GITM 1, GMWA 2, GSHT 2 |
| 220101 | 1 | 3 |  |  |  | GFG1 15 |
| 240001 | 14 | 51 | normal:2 | ScopePrimid 6, Trowlon 6, Buckot 4, Primid 3, Roader 3, Armank 1, Gamyga 1, Bytan 1 |  | EPSP 3, GASD 1, GBLK 2, GDMD 1, GET1 7, GET2 17, GFSR 1, GFT2 2, GIB2 2, GLOK 1, GMIN 2, GMOV 8, GTIK 13, GWND 1 |
| 240001a | 4 | 11 | normal:1 |  |  | GIB2 2, GMOV 2 |
| 240002 | 11 | 72 | normal:2, yellow:1 | Mite 9, Auroros 7, Bombed 4, Towtow 3, Primid 2, BigPrimid 1, BoomerangPrimid 1 |  | EPBM 1, EPSP 3, GBC1 3, GBC2 14, GET1 18, GET2 13, GFRK 2, GFSR 1, GFT2 4, GIB2 1, GLOK 1, GMOV 12, GSHT 1 |
| 240002a | 5 | 21 | normal:1 | Jyk 3 |  | GBLK 3, GDMD 1, GET1 1, GET2 3, GIB2 3, GMOV 1 |
| 240002b | 2 | 6 | normal:1 |  |  | GIB2 1 |
| 240101 | 1 | 10 |  |  | Galleom | GET1 1 |
| 250001 | 1 | 10 |  |  | Galleom | EPBM 1, EPSP 1, GET1 1 |
| 260001 | 10 | 67 | normal:2 | Feyesh 8, Shellpod 4, Puppit 4, Mite 3, Ticken 2, Armight 1, Bucculus 1, Borboras 1 |  | EPBM 1, EPSP 3, GBLK 4, GELE 3, GET1 5, GET2 11, GFSR 1, GFT2 2, GITM 1, GLAD 1, GLOK 2, GMCA 1, GMOT 3, GMOV 7, GMPS 4, GMWO 1, GPS2 2, GQUK 1, GSBK 2 |
| 260001a | 4 | 14 | normal:1 |  |  | GBLK 1, GIB2 2, GMOV 1, GMPS 1 |
| 260002 | 12 | 52 | yellow:1 | ScopePrimid 3, Primid 3, Trowlon 3, Spaak 3, Bytan 3, Roader 3, Nagagog 2, SwordPrimid 1, Greap 1 |  | EPSP 5, GASD 3, GBLK 1, GDMD 2, GENC 1, GEPT 1, GET1 14, GET2 7, GFRK 3, GFSR 1, GFT2 4, GIB2 1, GLAD 1, GLOK 1, GMOT 3, GMOV 5, GMSJ 1, GMTA 4, GWND 3 |
| 270001 | 7 | 49 | normal:1 | Goomba 7, Koopa 7, Puppit 4, HammerBro 2 |  | EPBM 1, EPSP 2, GASD 4, GELA 3, GET1 5, GET2 14, GIB2 1, GLAD 2, GMCA 4, GMOV 5, GMWO 1, GQUK 1, GSBK 4, GWND 2 |
| 270002 | 9 | 92 | normal:2 | BulletBill 24, Koopa 8, HammerBro 5, Floow 2, Poppant 2, Puppit 1, Goomba 1 |  | EPBM 1, EPSP 3, GASD 4, GCAT 1, GDMD 1, GEPT 1, GET1 15, GET2 17, GFSR 2, GFT2 2, GIB2 1, GITM 1, GLAD 13, GLOK 1, GMCA 6, GMOT 4, GMOV 5, GMWO 3, GQUK 3, GSBK 6, GWND 4 |
| 270002a | 3 | 9 | normal:1 |  |  | GASD 1, GIB2 2, GWND 1 |
| 270101 | 1 | 2 |  |  |  | GFG1 15, GSND 1 |
| 270201 | 5 | 81 | normal:2 | Jyk 7, Koopa 6, HammerBro 4 |  | EPSP 4, GASD 2, GBC1 4, GBC2 12, GCAT 9, GEPT 2, GET1 2, GET2 12, GFT2 4, GIB2 1, GLOK 1, GMCA 8, GMOT 1, GMOV 1, GMWO 2, GQUK 2, GSBK 8, GSTG 1, GTRC 1, GWAT 10 |
| 270202 | 8 | 62 | normal:2 | Koopa 12, BulletBill 3, Feyesh 3, Borboras 2, BoomerangPrimid 1 |  | EPBM 1, EPSP 3, GASD 2, GBLK 1, GDMD 1, GET1 5, GET2 13, GIB2 2, GITM 1, GMOV 7, GMTA 19, GSPR 35, GTIK 3, GWAT 9 |
| 270202a | 5 | 24 | normal:1 |  |  | GASD 1, GIB2 1, GITM 2, GMCA 6, GMOV 1, GMWO 1, GNDV 1, GQUK 1, GSBK 6, GSTG 1, GWAT 1 |
| 270203 | 5 | 54 | yellow:1 | BulletBill 10, Goomba 6, Koopa 4, GiantGoomba 1 |  | EPSP 1, GASD 2, GBLK 1, GEPT 11, GET1 2, GET2 17, GFSR 2, GFT2 6, GLOK 1, GMCA 4, GMOV 4, GMVA 13, GMWO 6, GNDV 1, GQUK 6, GSBK 4, GWAT 2 |
| 280002 | 8 | 44 |  | Glice 6, ROBBlaster 4, ROBSentry 3, BigPrimid 2, SwordPrimid 2, Roader 1 |  | EPSP 3, GBLK 1, GDBF 2, GET1 5, GET2 13, GFT2 1, GLAD 1, GMOV 6, GMWA 6, GSCN 1, GSHT 15 |
| 280002a | 3 | 9 |  |  |  | GDBF 1, GET1 1, GIB2 2 |
| 280003 | 1 | 14 |  | ROBSentry 2, ROBBlaster 2, Buckot 2 |  | EPSP 1, GDBF 2, GET1 1, GET2 3 |
| 280101 | 1 | 3 |  |  |  | GFG1 30 |
| 280201 | 4 | 37 |  | ROBSentry 8, ROBBlaster 5, ROBLauncher 2, ROBDistance 1 |  | EPSP 3, GBLK 5, GDBF 1, GDMD 1, GET1 1, GET2 15, GFRT 5, GITM 1, GMOV 5, GMTA 3, GMWA 10 |
| 280202 | 6 | 54 |  | Cymul 11, ROBSentry 5, ROBLauncher 3, ROBDistance 2, Autolance 1, ROBBlaster 1 |  | EPBM 1, EPSP 4, GBLK 1, GDBF 3, GEPT 1, GET1 3, GET2 12, GITM 1, GLAD 3, GLOK 1, GMCA 5, GMOV 2, GMRL 8, GMWA 3, GMWO 2, GQUK 2, GSBK 5 |
| 280202a | 4 | 22 |  |  |  | GDBF 1, GDMD 1, GIB2 2, GMOV 2, GMTA 4, GMWA 4 |
| 280203 | 8 | 38 |  | Glice 3, ROBDistance 2, ScopePrimid 2, Roturret 1, Cymul 1, Primid 1 |  | EPSP 2, GASD 1, GDBF 2, GET1 3, GET2 5, GFT2 1, GIB2 1, GLOK 1, GMOV 8, GMV4 2, GMWA 1, GSPR 8, GWAP 12 |
| 280204 | 2 | 7 |  |  |  | GDBF 2, GIB2 1 |
| 280301 | 1 | 5 |  |  | Ridley | GET1 1, GFT2 1, GITM 1 |
| 290001 | 8 | 32 | normal:3 | ROBSentry 6, Auroros 6, ROBLauncher 4, Roader 4, ROBDistance 4, Gamyga 2, Shellpod 2 |  | EPSP 6, GBLK 1, GET1 5, GET2 13, GIB2 1, GLAD 1, GLOK 1, GMOV 2 |
| 290001a | 1 | 4 | normal:1 |  |  | GITM 1 |
| 290001b | 3 | 9 | normal:1 |  |  | GET1 1, GIB2 2, GITM 1, GTIK 10 |
| 300001 | 2 | 12 |  | Trowlon 8, Primid 7, SwordPrimid 7, ScopePrimid 4, Bombed 4, Spaak 4, Shaydas 3, MetalPrimid 3 |  | EPSP 15, GEPT 5, GSND 3, GWD2 1 |
| 310001 | 5 | 56 | normal:1 | Primid 9, Floow 9, BoomerangPrimid 5, Ticken 3, Shellpod 2, Poppant 1, Bucculus 1 |  | EPSP 4, GBC2 7, GDMD 1, GET1 11, GET2 9, GIB2 1, GLAD 3, GMOV 10, GMTA 2 |
| 310002 | 7 | 44 | normal:1 | SwordPrimid 6, Floow 6, MetalPrimid 4, Primid 2 |  | EPSP 3, GDMD 1, GEFF 17, GET2 20, GFSR 1, GIB2 2, GMOT 9, GMOV 5, GMTA 13 |
| 310003 | 5 | 47 | normal:3 | Glice 11, Spaak 8, Shaydas 5, Towtow 2, Jyk 2 |  | EPSP 4, GASD 1, GBC1 1, GEPT 6, GET1 6, GET2 11, GFT2 4, GLAD 13, GLOK 1, GMOT 1, GMOV 3, GMWO 4, GWND 1 |
| 310003a | 3 | 9 | normal:1 | Primid 2, BoomerangPrimid 1 |  | GIB2 2, GMWA 1, GSCN 1 |
| 310003b | 5 | 16 | normal:1 |  |  | GDMD 1, GIB2 2, GITM 2, GLAD 2, GMCA 1, GMWO 1, GQUK 1, GSBK 1, GWAT 1 |
| 310101 | 1 | 3 |  |  |  | GFG1 15, GSND 1 |
| 320001 | 1 | 14 |  | Glire 8, Primid 6, BigPrimid 6, FirePrimid 6, BoomerangPrimid 5, Puppit 4, Armight 3, Feyesh 2, Roturret 1 |  | EPSP 5, GEPT 13, GSND 1 |
| 330001 | 1 | 5 |  |  |  | GASD 1, GDBF 1 |
| 330002 | 2 | 13 |  | Primid 6, ScopePrimid 4, SwordPrimid 2 |  | EPSP 3, GASD 2, GDBF 2, GET2 12, GLAD 1, GMWA 1, GTIK 23 |
| 330002a | 5 | 23 |  | Jyk 1 |  | EPSP 1, GASD 1, GBLK 2, GDBF 1, GDMD 1, GET2 1, GIB2 2, GTIK 8 |
| 330101 | 6 | 33 |  | Primid 11, ScopePrimid 4, Bytan 2, Mite 2, Floow 1, Cymul 1, Armight 1 |  | EPBM 1, EPSP 3, GASD 7, GBLK 1, GDBF 2, GEFF 3, GENC 3, GET1 6, GET2 5, GFT2 3, GIB2 2, GITM 1, GLAD 3, GMOV 4, GMWA 3, GSPR 6 |
| 330101a | 3 | 13 |  | Glire 2, FirePrimid 1 |  | GASD 1, GDBF 1, GENC 1, GET2 1, GIB2 2 |
| 330102 | 1 | 13 |  | SwordPrimid 5, Shaydas 2, Autolance 1 |  | EPSP 3, GASD 4, GDBF 1, GEPT 1, GET1 2, GLOK 1, GSCN 1 |
| 330103 | 2 | 18 |  | Roader 3, ScopePrimid 2, Buckot 2, FirePrimid 1 |  | GASD 1, GDBF 1, GDMD 1, GET1 1, GET2 7, GIB2 1, GMOV 1, GMTA 1, GMWA 3, GSCN 1 |
| 330104 | 5 | 34 |  | Bombed 1, Primid 1, MetalPrimid 1, Nagagog 1, Towtow 1, Cymul 1 |  | GASD 7, GDBF 1, GDMD 1, GEFF 6, GET1 2, GET2 4, GIB2 2, GLAD 4, GMOV 7, GMTA 1, GSPR 2 |
| 330201 | 1 | 3 |  |  |  | GFG1 30 |
| 340001 | 1 | 11 |  |  |  | GASD 2, GDBF 1, GMOV 2, GWND 1 |
| 340002 | 7 | 45 |  | Mite 10, ScopePrimid 4, Poppant 1 |  | EPSP 2, GASD 1, GDBF 1, GET1 2, GET2 8, GFPR 6, GIB2 6, GMIN 14, GMOT 2, GMOV 1, GMWA 3, GMWO 1, GQUK 1, GSHT 1, GTIK 6 |
| 340003 | 1 | 16 |  |  |  | GASD 1, GDBF 1, GET1 1, GMOA 10 |
| 340004 | 7 | 42 |  | Mite 14, Buckot 4, Feyesh 3, Greap 2, Primid 2, ScopePrimid 2, Puppit 2 |  | EPSP 7, GASD 1, GDBF 1, GDMD 1, GEPT 2, GET1 2, GET2 16, GFPR 14, GIB2 3, GLOK 1, GMIN 1, GMOT 1, GMOV 5 |
| 340005 | 2 | 10 |  |  |  | GASD 2, GET1 1, GIB2 1, GMSJ 1, GWND 1 |
| 350001 | 2 | 22 |  |  | Duon | GET1 1, GSND 2, GWD2 1 |
| 360001 | 6 | 85 |  | ROBSentry 6, Jyk 4, Glunder 4, ROBBlaster 4, Bombed 3, Borboras 3, Shaydas 2, Roturret 2, Nagagog 2, BigPrimid 1 |  | EPSP 1, GASD 2, GBLK 3, GDBF 4, GELE 2, GEPT 1, GET1 16, GET2 6, GFT2 1, GIB2 3, GLK2 9, GMOV 9, GMV4 9, GMWA 4 |
| 360001a | 4 | 16 |  | SwordPrimid 3, ROBLauncher 2 |  | EPSP 2, GASD 1, GDBF 1, GET1 1, GIB2 2, GLOK 1, GMV4 1 |
| 360001b | 3 | 15 |  | Armank 1 |  | GDBF 1, GET1 1, GLAD 1, GLOK 1, GMOV 1 |
| 360001c | 3 | 9 |  |  |  | GDBF 1, GIB2 2 |
| 360002 | 3 | 15 |  | ROBBlaster 2, ROBSentry 2, ROBLauncher 2, ROBDistance 1 |  | EPSP 2, GDBF 2, GEPT 1, GET1 3, GET2 2, GFT2 3, GLOK 1, GMOT 1, GMOV 1, GMSJ 1 |
| 370001 | 8 | 50 |  | ROBLauncher 8, Cymul 4, Autolance 3, ROBSentry 2, Jyk 2, MetalPrimid 2, ROBDistance 1 |  | EPBM 1, EPSP 4, GBLK 3, GDBF 3, GDMD 1, GEPT 3, GET1 3, GET2 12, GFSR 2, GFT2 2, GLAD 1, GLOK 1, GMCA 2, GMOT 3, GMOV 3, GMWO 1, GQUK 1, GSBK 2 |
| 370001a | 5 | 12 |  |  |  | GDBF 1, GET1 1, GIB2 3, GMOT 1, GSPR 5 |
| 370002 | 5 | 68 |  | Mite 10, Buckot 6, ROBSentry 3, Jyk 3, Floow 2, Armight 1, Poppant 1 |  | EPBM 1, EPSP 3, GASD 5, GBLK 3, GBLT 8, GDBF 2, GDMD 1, GDTP 1, GEFF 3, GEPT 2, GET1 15, GET2 5, GITM 1, GLAD 1, GLOK 1, GMOT 3, GMOV 4, GMTA 4, GMV4 2, GMWA 3 |
| 370002a | 5 | 25 |  |  |  | GDBF 1, GIB2 2, GLAD 1, GMCA 5, GMOV 1, GMWO 1, GQUK 1, GSBK 5 |
| 370003 | 13 | 65 |  | Bytan 6, ROBLauncher 6, ROBDistance 2, ROBSentry 2, FirePrimid 2, Primid 1 |  | EPBM 1, EPSP 2, GBLK 3, GCAT 1, GDBF 1, GEPT 1, GET1 5, GET2 8, GFPR 12, GFSR 1, GIB2 2, GITM 1, GITR 1, GLAD 5, GLOK 1, GMOA 1, GMOT 2, GMOV 13, GMSJ 1, GMTA 5, GMWA 5 |
| 370101 | 2 | 22 |  | Auroros 20, SwordPrimid 9, Primid 6, Ticken 4, ScopePrimid 3, Floow 3, Nagagog 3, Armank 2 |  | EPSP 6, GEPT 7 |
| 370201 | 9 | 47 |  | Cymul 5, Primid 5, SwordPrimid 3, ScopePrimid 1 |  | EPSP 3, GBLK 1, GDBF 1, GDMD 2, GET1 1, GET2 18, GFSR 1, GLAD 2, GMOV 22, GMTA 2 |
| 370202 | 3 | 33 |  | Mite 4, FirePrimid 3, Towtow 1 |  | EPBM 1, EPSP 1, GDBF 1, GENC 1, GET1 8, GET2 5, GFSR 1, GITM 2, GLAD 1, GMIN 10, GMOV 3 |
| 370203 | 2 | 7 |  |  |  | GET1 1, GIB2 1, GMOV 1, GMSJ 1 |
| 370301 | 2 | 10 |  |  | MetaRidley, FalconFlyer | GET1 1, GFT2 1, GITM 1, GWD2 1 |
| 390001 | 7 | 40 | normal:1 | BigPrimid 5, BoomerangPrimid 3, Primid 3, Floow 2, Armank 2, Borboras 2, MetalPrimid 2, Spaak 1, FirePrimid 1, Greap 1 |  | EPSP 9, GENC 6, GEPT 1, GET1 4, GET2 3, GFSR 1, GFT2 1, GITM 1, GLOK 5, GMOT 1, GMOV 5 |
| 400001 | 1 | 15 | normal:1 | Glice 5, Bombed 4, Feyesh 2 |  | GASD 1, GEFF 1, GET1 1, GET2 11, GITM 1, GMOV 1 |
| 400002 | 3 | 26 | normal:3 | Poppant 2, Trowlon 2, Towtow 1 |  | GASD 1, GEFF 1, GET1 3, GET2 3, GITM 1, GMOV 6 |
| 400003 | 3 | 12 | normal:1 |  |  | GASD 2, GEFF 2, GET1 1, GIB2 1, GITM 2, GTIK 5 |
| 400004 | 8 | 54 | normal:4 | Feyesh 5, Bytan 4, Mite 4, BigPrimid 2, Roader 2 |  | EPBM 1, EPSP 3, GASD 3, GEFF 3, GENC 3, GET1 10, GET2 3, GFT2 9, GITM 3, GMCA 4, GMOT 1, GMOV 28, GMWO 1, GQUK 1, GSBK 4, GSPR 2 |
| 400005 | 3 | 15 | normal:1 |  |  | GASD 2, GEFF 2, GET1 1, GITM 2, GMCA 1, GMWO 1, GQUK 1, GSBK 2 |
| 400006 | 1 | 10 | normal:1 |  |  | GASD 2, GEFF 2, GET1 1, GITM 2, GMIN 1 |
| 400007 | 5 | 25 | normal:2 | Puppit 6 |  | EPSP 1, GASD 1, GBLK 1, GEFF 1, GET1 7, GITM 1, GMCA 2, GMOV 1, GMWO 1, GQUK 1, GSBK 2, GTIK 36 |
| 400008 | 1 | 23 | normal:2 | Mite 6, Jyk 2 |  | EPBM 1, EPSP 2, GASD 2, GEFF 2, GENC 1, GET1 3, GET2 2, GFSR 2, GITM 2 |
| 400009 | 5 | 20 | normal:2 |  |  | GASD 1, GEFF 1, GET1 1, GFT2 3, GITM 1, GMOV 11 |
| 400101 | 1 | 5 |  |  |  | GFG1 15 |
| 410001 | 3 | 27 | normal:1 | Bombed 7, Feyesh 6, ScopePrimid 2, SwordPrimid 2 |  | EPSP 2, GASD 3, GEFF 3, GENC 1, GET1 4, GET2 8, GIB2 2, GITM 3, GMIN 5, GMOV 1 |
| 410002 | 6 | 57 | normal:1 | Spaak 7, Bucculus 3, FirePrimid 2, ScopePrimid 2, Primid 2, Armight 1, Autolance 1 |  | EPBM 1, EPSP 4, GASD 4, GBLK 3, GEFF 4, GET1 2, GET2 8, GFSR 1, GFT2 4, GITM 5, GLOK 2, GMCA 6, GMOV 3, GMWO 6, GQUK 6, GSBK 6 |
| 410003 | 13 | 68 | yellow:1 | Shellpod 4, Ticken 4, Auroros 3, Glire 3, Bytan 2, Roturret 1, Gamyga 1 |  | GASD 3, GBLK 3, GCAT 3, GEFF 3, GENC 2, GET1 4, GET2 10, GFT2 5, GIB2 1, GITM 3, GLK2 1, GLOK 1, GMOT 1, GMOV 14, GMPS 2 |
| 420001a | 1 | 9 | normal:1, t3:1, t5:3 |  |  | GET1 1, GLK2 1, GSAV 1 |
| 900001 | 1 | 13 |  |  | MasterHand, CrazyHand | EPBM 6, EPSP 6, GCAM 2, GET1 1, GFIN 1 |
| 900101 | 1 | 6 |  |  | MasterHand | EPBM 3, EPSP 3, GET1 1, GFIN 1 |
| 900201 | 1 | 6 |  |  | CrazyHand | EPBM 3, EPSP 3, GET1 1, GFIN 1 |
| 910101 | 1 | 6 |  |  |  | GEPT 10, GET1 3, GFIM 10, GITM 8, GLOK 1, GMSJ 1, GNDV 90 |
| 920001 | 2 | 13 |  |  | PeteyPiranha | EPSP 3, GET1 1, GMOT 1, GSND 1 |
| 920101 | 1 | 11 |  |  | Rayquaza | EPBM 3, EPSP 3, GET1 1, GWD2 1 |
| 920201 | 2 | 6 |  |  | Porky | EPBM 3, EPSP 3, GET1 1 |
| 920301 | 1 | 10 |  |  | Galleom | EPBM 3, EPSP 3, GET1 1 |
| 920401 | 1 | 10 |  |  | Galleom | EPBM 3, EPSP 3, GET1 1 |
| 920501 | 1 | 5 |  |  | Ridley | EPBM 3, EPSP 3, GET1 1 |
| 920601 | 2 | 22 |  |  | Duon | EPBM 3, EPSP 3, GET1 1, GSND 2, GWD2 1 |
| 920701 | 2 | 10 |  |  | MetaRidley, FalconFlyer | EPBM 3, EPSP 3, GET1 1, GWD2 1 |
| 920801 | 1 | 4 |  |  | Tabuu | EPBM 1, EPSP 1, GET1 1 |
