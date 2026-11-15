# Geno enemy-engine actors, first proof: Petey (port/geno-boss)

Spec: `docs/specs/geno-v5.2-boss.md`. Code: `rust/melee_modes/src/tu/geno_boss.rs` (the engine: define types, state,
rule passes, ops, part procs, timelines, kinetics, hash), `geno_boss_ir.rs` (local IR -> define at load time, own tiny
JSON reader), `petey.rs` (the host: carrier, hit / hurt shapes, draw, camera, Envoy and VS entry).

## Running

| what | how |
|---|---|
| Envoy fight 5 | `MELEE_ENVOY_BOSS=petey` + the Envoy mods, as before (`petey-boss.md`) |
| VS | `MELEE_VS_BOSS=petey`: the first CPU slot of a VS match becomes Petey's carrier, on FD / his arena, no timer |
| the IR | `MELEE_BOSS_IR_DIR`, else `<MELEE_BRAWL_LOCAL>/boss-ir`, else `_build/boss-ir` above the working dir; file `petey.brawl.ir.json`. Never committed, never printed |
| hand-coded Petey | `MELEE_PETEY_IMPL=hand` (comparison only). Without the IR the hand-coded boss runs and the log says why (`petey: no Geno define (...)`) |
| logs | `MELEE_PETEY_LOG=1` (host lines `petey:`) / `MELEE_GENO_BOSS_LOG=1` (engine lines `geno_boss:`: status, rules, messages, stage events) |
| checks | `tools/rust-port/envoy/geno_boss_check.sh [idle death break bodybreak sync]` (EXE, MODS, DEPTH, FRAMES) |
| unit tests | `cargo test -p melee_modes --lib geno_boss` (lowers and simulates the local IR when present) |

## What is generic and what is the host

Generic (engine + IR): statuses, the 31 rules, the four weighted tables with L-scaled weights and the 0.3 repeat cut,
target draw / lock / side tests, L tables and strength changes, cage HP from the IR's formula at spawn, damage routing
body -> cages, cage procs (msg 1 / 0x1400 / 0x1401 / 0x13FF), message handlers incl. the -198/-197 one-priority death
window, kinetics k14/k15/k16 and the goal-x snap, motion picks, timelines (shakes, stage events), phases, both defeat
paths, per-part intangibility. All numbers come from the IR at load time.

Host (petey.rs, unchanged from the hand port): hit and hurt shapes on the animated bones and their capsule mapping
(the hitbox numbers still come from the hand table; the define carries the IR's attacks by name only), the carrier and
KO, draw, camera, knockback scale, sounds the IR lists without an id (looked up in the host's per-motion table).

## Cross-check rows (petey-crosscheck.md) on the Geno actor

| # | item | now |
|---|---|---|
| 1 | cage hit in Wait A/B | follows the code: msg 1 -> 13/14 (DamageLeft/Right), then rule 19 -> 4 / 20 -> 5 (forwarded body damage also counts as the cage being damaged). The IR keeps the observation conflict open |
| 2 | phase-1 roar | rule 28 fires at the first evaluation in status 2/3 after 0x1400 (check: 1 frame after the wait is entered) |
| 3 | rule 29 | agrees (priority -100, no status change) |
| 4 | death | status 15: stage event 1, the death sound, stage event 1000/1001 for the lower cage, no Down |
| 5 | Down end | Down's stage events 5/6/7/8 are emitted; the fight still ends at Down frame 180 (host, stage-side) |
| 6 | cage hit to 0 | the break path runs: 0x13FF -> stage 1001 -> cageBroken -> rule 10 -> Down; rules 6/7 stay off; a cage hit is not hitByPlayer |
| 7 | phase thresholds | per cage, own pct vs own counter (5106 sets both) |
| 8 | threshold on a damaged frame | msg 1 instead; 0x1400/0x1401 on the next undamaged frame |
| 9 | body contact in JumpAttack | unchanged (host hit table) |
| 10 | cage HP | the IR's L formula, truncated; scales with Envoy's difficulty |
| 11 | arena clamp | none (IR: boss x unclamped; goal snap only) |
| 12 | knockback scale | host tuning, unchanged |
| 13 | AI flag 0x12000012 | not modelled (nothing reads it) |

## Engine reading choices (documented in the spec)

- Evaluation passes: -100, damage window -250..-150 on a hit frame, normal -50..250; an action-only rule fires and the
  pass goes on (one per priority level), the first rule with a status ends it.
- `hitByPlayer` is the core's flag only.

## Proof (2026-11-14, exe from this branch)

| check | result |
|---|---|
| unit tests | `geno_boss` 4/4 (engine, JSON, lowering of the local IR: 16 statuses, 31 rules, 4 tables, 5 messages; a headless fight to the death path) |
| `geno_boss_check.sh idle death break bodybreak` | all ok: loop and wait lengths, roar 1 frame into the first wait after the phase-1 hit, phase 2 at its L, threshold-then-hit death (status 15, stage events 1 and 1000, no Down), cage break (0x13FF, stage 1001, Down, KO), body break (stage 1000, Down, KO) |
| synctest (death schedule, 3000 frames) | depth 4: 2997 passes, 11988 hash checks, 0 mismatches; depth 7: 2994 passes, 20958 checks, 0 mismatches |
| NHV1 (golden, `pad_falcon_fox_fd` + Envoy fight 5, 3000 frames) | identical over two runs; last row frame 2999 hash `1CB3982F`, seed `777F059B` (the boss state word is mixed in while the actor is live) |
| frame times (real time, profiler, 2199 frames) | 0 frames over 16.7 ms (one startup frame, the match load) |
| VS (`MELEE_VS_BOSS=petey`, no mods) | his arena, the Geno actor, death at the scripted hits, carrier KO |

## Projectiles: spawn records and the weapon pool (lane B)

Code: `rust/melee_modes/src/tu/geno_boss_spawn.rs` (records, pool, flight, explosions, homing, hits, hash, IR lowering);
`geno_vm_host.rs` (VM bosses); core touch points: `Op::Spawn { record, joint }`, `Define::spawns`, `State::spawn`
(`MAX_SPAWNS` slots), `EV_SPAWN` / `EV_SPAWN_HIT` / `EV_SPAWN_END`, `hash_into` in `state_hash`.

| piece | reading |
|---|---|
| launch | the enemy spawner: angle and speed drawn from the record's ranges at the owner's L; v = (cos a * speed * facing, sin a * speed); accel = (cos a, sin a) * the record's accel, facing on x, plus the record's y accel |
| per frame | pos += v, then v += accel (the weapon module's frame) |
| boxes | flying: the record's start script; explosion: the record's proc set (script and frames) |
| ends | per weapon proc: on the floor, on contact, at life end; then the explosion, then removal (owner-side) |
| L | a projectile keeps the owner's L at spawn |

Code addresses, record field positions and every per-weapon value: the local engine-ops.md and the owner's local IR
(`boss.weapon_records`, written by `_build/boss-ir/tools/gen_weapons.py`). Nothing of it is in the repo.

VM bosses (the generic host): the VM emits `EV_WEAPON` per executed 650E (`v` = the record's param id, joint in the
instruction). `geno_vm_host::frame` spawns after its event loop (`geno_boss_spawn::spawn_at`, the record looked up by
param id in `vm_records(<boss>)`, loaded from the local IR at the match's start; speed / angle draws from the VM
actor's own generator), then steps the pool against the players (contact, homing). `petey::vm_hits` puts live
projectile boxes into the carrier's hit capsules left after the boss's own boxes (one attack per projectile and
phase), so damage, knockback and hitlag are Melee's. `state_hash` mixes the pool on the VM path while one is live.

Placement: `geno_vm_host::weapon` puts a new projectile on the script's bone through `vm_boss_model::bone_world`
(the record's offset in that bone's space, posed at the spawn frame); with no model or no such bone it falls back to
the actor's point plus the offset. The check script's `weapons` row asserts "on its bone"; `WALK_X` defaults to 60.

Later proof (merged with the VM boss models): Galleom and Duon pass every row with bone placement (synctest d4/d7
0 mismatches, NHV1 identical, 0 slow frames); a 10 000-frame Galleom run shows many grenade volleys and shockwaves,
all removed, but still no missile spawn, so the missile record is unproven in play.

Limits: four capsules per frame in all (boss boxes first); the homing turn rate, the homing explosion and a box-less weapon's contact
radius are inferred, not read.

Proof (2026-11-14, exe from this branch, `tools/rust-port/envoy/geno_vm_check.sh weapons sync nhv1 perf`,
`BOSS=galleom WALK_X=-60` / `BOSS=duon`):

| check | Galleom | Duon |
|---|---|---|
| weapons | grenade spawned, explodes on the floor, Falcon hit (percent up by the explosion's damage), removed | bombs spawned, two reach Falcon, all removed |
| synctest d4 / d7 (3000 f, projectiles inside) | 0 / 0 mismatches | 0 / 0 mismatches |
| NHV1 twice | identical | identical |
| frames over 16.7 ms (real time) | 0 (+1 startup) | 0 (+1 startup) |
| unit tests | `cargo test -p melee_modes --lib geno_ -- --test-threads=1`: 16/16 | |

Seen in the runs, for the VM lane: Galleom stays in his charge status at the arena's edge after a few hundred frames,
so the 3000-frame fight shows one grenade volley and no missiles; Duon's frame with two 650E (no joint / with joint)
emitted one EV_WEAPON.

## Grabs (lane A, port/geno-boss-grab)

Code: `rust/melee_modes/src/tu/geno_boss_grab.rs` (grab state, host, the Tabuu grab test define), lowering arms in
`geno_boss_ir.rs` (`grab_timeline`, `grab_status_frame`, grab phrases in `Lx::op`), variants in `geno_boss.rs`.

| piece | what |
|---|---|
| `Op::Grab { part, joint, radius, frames, release }` | a catch box (060A) for a motion-frame window (060A .. next 060D - 1; one frame without a 060D); `release` = the hold-status mask |
| `Op::GrabRelease(Option<attack>)` | 6528: throw with the attack's first box (a 061A of the same frame: the grab's `throw` block; else the box the motion's attack opens then), or drop |
| `Cond::Holding` | a fighter is held; "if caught (flag 6) -> ... N" lowers to `If(Holding, ChangeStatus(N))` |
| `EV_GRAB` | 1 armed, 2 throw, 3 drop |
| hold mask | the statuses playing the catch motion plus, transitively, each "if the next status is not N: CommonCatchProcExit" exit's N (Tabuu: Chain 36\|37, Dive 19\|20\|21); `enter` drops the victim on leaving it |
| State | `State::grab` (`GrabState`, appended last), all of it in `state_hash` |

Host (`host_frame`, after `geno_boss::tick`): the catch box is a sphere on the carrier's Melee catch joint (`ft_data
x8.x11`; Brawl bone numbers mean nothing on a Melee body) tested against the other fighters' grabbable hurt capsules on
the motion frame whose timeline ran (`tl_mf`). A catch is Melee's own capture: `fn_800DAADC` + `fn_800DB6C8`
(CapturePulled -> CaptureWait) with the carrier as grabber (`victim_gobj`, `capturedamage.x18` set every frame), so
mashing, CaptureCut / CaptureJump and hits on the held fighter are the game's code. The host gives back CaptureWait's
passive per-frame timer decay: a boss hold ends by its release or by mashing (Brawl's bosses throw before the common
catch timer matters; Melee's 75-frame passive expiry dropped every Chain hold before ChainThrow). Throw = Melee's throw
release (`ftCo_800DE2A8` with the box in the carrier's `xDF4[0]`, then `ftCo_800DE7C0`); drop = `ftCo_CaptureCut_Enter`.
An escape is any frame the victim is no longer in a capture state of the carrier.

Test host: `MELEE_VS_BOSS=tabuu` (+ `MELEE_VS_BOSS_SLOT=n` for an all-human scenario): a minimal Tabuu define
(Wait 60 frames, Chain and Dive alternating; `MELEE_TABUU_GRAB=chain|dive`, `MELEE_TABUU_WAIT`) whose catch boxes, hold
masks and throws are lowered from `tabuu.brawl.ir.json`; motion lengths are the IR's timelines; the carrier is the
visible fighter (no Tabuu model, no teleports, no other attacks). Logs `MELEE_GRAB_LOG=1` (`geno_grab:` lines).
Check: `tools/rust-port/envoy/geno_grab_check.sh [hold mash sync nhv1 perf]`.

Also: the IR reader accepts `NaN` / `Infinity` (Tabuu's and Rayquaza's files have them; they failed to parse before).

Second round (same day):

- **Tabuu's "catch gates" are approach-movement gates** (traced; addresses in the local engine-ops.md, third pass):
  6604 n / 6605 n / 660E n are the enemy kinetic-unit module (command kind 0x66): unit n resume / pause / reverse. In
  ChainStart and ChainThrow they run his slot-5 approach kinetic (kin 23 / 24) by the target's side
  and range (R10026 / R10032: one handler for R10024..R10063, target vs enemy position through a per-id axis mask; the
  compare tail is [I]). The catch box is only 060A / 060D. Lowered (`approach_gate`, from the per-frame phrases) as lock,
  aim, `If(TargetDx > catch radius, Kinetic(slot speed), StopMove)`; new `Cond::TargetDx(Cmp, k)`. The host lets the
  actor's movement drive the carrier on the ground.
- **Bucculus's chew** (`bucculus_define`, `MELEE_VS_BOSS=bucculus`): Bites1 catch boxes (the timeline's 060A boxes)
  -> Absorption (bite on its hitbox frame) -> Bites2 -> R21 every param 4003 frames -> R23 at param 4002 frames
  since the catch -> RollingJump, spat out with the spit-out box. New `Op::GrabHit(attack)` (`EV_GRAB` 4): the host
  applies the box's damage (`Fighter_TakeDamage_8006CC7C`) and Melee's CaptureDamage reaction. The hold chain 10 <-> 12
  comes from the enemy-style exits (`if status != N: CommonCatchProcExit`, now read by `hold_mask_from`); the catch ->
  Absorption hand-over is native in Brawl (unread [L]) and is `Holding` in Bites1 here. The passive grab-timer decay is
  given back in CaptureDamage too (a bite let the timer run out).
- `GrabState` gained `hit` (end of the block; hashed).

Still not done: the R10026 compare tail (its two threshold parameters not used; the catch radius stands in), Tabuu's
warp to x +-128 before ChainStart, Bucculus's lunge steering / burrowing / search box (|dx| <= 15 stands in), the 061A
base knockback word [I]. `port/geno-vm` had not landed: nothing to hook yet.

### Proof (2026-11-14, release exe of this branch)

| check | result |
|---|---|
| unit tests | `geno_boss` 7/7 incl. `geno_boss_grab` (catch window, hold, throw request, drop on leaving the hold, the local Tabuu IR lowers to the Chain catch window with hold 36\|37 and the Dive's one-frame catch with hold 19\|20\|21, throws from ChainAttack's box and the Dive throw block) |
| `geno_grab_check.sh hold` | Chain catch at the window's first frame -> ChainThrow -> thrown at the release frame; Dive catch on its catch frame -> DiveTeleport -> DiveSlam -> thrown; no escape without a mash (values: the local IR) |
| `mash` | caught, mashed out (CaptureCut) before the throw |
| synctest (hold schedule, 3000 frames) | depth 4: 11988 checks, 0 mismatches; depth 7: 20958 checks, 0 mismatches; grabs and throws inside both |
| NHV1 (`golden`, hold schedule, 3000 frames) | identical over two runs; last row frame 2999 hash `ED7C78E4` |
| frame times (real time, `play`, Falcon vs CPU Fox carrier, profiler, 2999 frames) | 0 over 16.7 ms (+1 startup frame, the match load) |

Second round (release exe of this branch, `geno_grab_check.sh` all rows OK):

| check | result |
|---|---|
| unit tests | 8/8 (adds `bucculus_chew_from_ir`: caught, 4 bites, spat out) |
| `chase` (Falcon idle) | Tabuu walks to him in ChainStart (slot-5 kinetic) and catches |
| `chew` | Bites1 catch -> 4 bites (the Absorption box) at the Bites2 cadence -> spat out once the chew limit passed (R23 is live in Bites2 only) |
| `chewmash` | caught, mashed out, Bucculus back to WaitBuried |
| synctest 3000 frames | Tabuu depth 4 / 7 and Bucculus depth 4 / 7: 0 mismatches each, grabs and releases inside |
| NHV1 | identical over two runs: Tabuu last `F4B351BC`, Bucculus last `88E9D1E8` |
| frame times (real time, 2999 frames each) | Tabuu and Bucculus: 0 over 16.7 ms (+1 startup frame) |

## Lane C: boss-carried collision (port/geno-boss-coll)

Code: `rust/melee_modes/src/tu/geno_boss_coll.rs`; hooks `ground::BOSS_COLL` / `BOSS_COLL_INIT` (stage load, after
`EXPLORE_COLL`, as the TT platforms' `Ground_TTMod_BuildCollData` / `InitLines`) and `grlast::BOSS_COLL_FRAME`
(the harness's frame, Final Destination). Interface for the actor hosts:

| piece | what |
|---|---|
| `Part.coll: Option<PartColl>` | the IR's `boss.platforms[]` on that part (`point_joints`, `pass_through`), set by the lowering |
| `geno_boss_coll::want(n, pass)` | before the match: at the next stage load the stage's collision gets one extra joint of `n - 1` linked floor lines (pass-through) |
| `geno_boss_coll::set_points(&[[x, y]])` | every logic frame, world points left to right: `mpLib_80057424` (riders' previous place), vertices written, `mpJointUpdateBounding`: the explore.rs `movers_tick` path |
| `geno_boss_coll::build_surface(name)` | helper thread: the boss's bones posed for every frame of every clip, reduced to the walkable top (below) |
| State | `coll_n`, `coll_pts[16]`, `coll_clock`, `coll_x` at the end of `geno_boss::State`; the points are in `state_hash` |

Harness (no riding actor host yet): `MELEE_BOSS_PLATFORM=porky|duon|falconflyer` on FD: the boss's surface from the
owner's files, one clip looping (`MELEE_BOSS_PLATFORM_CLIP`, default `*Wait`), walking between x = +-(80 - half
width) at 0.35 / frame; `_Y` (lowest top point, default 12), `_SCALE` (default: fit 120 units wide),
`_LOG=<n>` logs the surface and each fighter's ON / off every n frames.

### The polyline reading, run: corrected

`grcoldataset.md` read the surface as the polyline through the Landform points *in index order*. Posing the bones
says otherwise: the points are an **outline**, not a left-to-right deck.

| boss | points in index order (side view) | walkable top |
|---|---|---|
| Porky | top / bottom pairs of a box with a raised middle | the four top points, left to right |
| Duon | along **z**: ground, side, top, side, ground | the four upper points |
| Falcon Flyer | along **z**: a hull; index 1 is the tail's underside, index 10 a step in the deck | 11 of the 13 points, tail to nose |

So: the side view uses the horizontal axis with the larger spread (Duon and the Flyer face along z), and the rider
stands on the **upper envelope** (`upper_envelope`: sorted by h, the highest of points within 1 unit, points more
than 12 below the line through their neighbours dropped, ends more than 24 below their neighbour dropped), chosen once
from the `*Wait` pose and kept for every clip. The lines Brawl draws between the points are presumably in the boss's
collision record (not in table 0); until that table is read, this envelope is the reading. The Flyer's out-of-order pair in the IR is the tail's underside point, which explains the break in its run.

### Proof (2026-11-14, exe from this branch, harness on FD)

| check | result |
|---|---|
| unit test `geno_boss_coll_surfaces` | all three surfaces build from the local files (Porky 23 clips, Duon 39, Flyer 9) |
| ride, Porky (fox vs idle falcon, pad: hops, ride, walk right) | lands frame 780, rides 220 frames carried 77 units (0.35 / frame, y following the bob and the raised middle), walks off the right end frame ~1010, lands on FD 1020 |
| ride, Duon / Flyer (one timed hop) | Duon: lands 600, rides 420 frames through the turnaround, walks off at ~1040. Flyer (scale 0.45): lands 690, rides through the turnaround, walks down the deck to the nose |
| ride with a CPU (level 1) | both fighters ride at once; a hit knocks the rider off as on any platform |
| synctest (`pad_falcon_fox_fd`, Porky surface at y 5 so both fighters keep landing on it: 360 ON samples at depth 7) | depth 4: 2997 passes, 11988 checks, 0 mismatches; depth 7: 2994 passes, 20958 checks, 0 mismatches (also 0 / 0 at y 12, nobody riding) |
| NHV1 (same, 3000 frames, the surface points in the boss word) | identical over two runs; last row frame 2999 `2BAB40AA`, seed `85955F6D` (45 riding samples in each) |
| frame times (real time, profiler, Porky ride, 1199 frames) | 0 over 16.7 ms (+1 startup frame, the match load) |

Not yet: an actor host for Porky / Duon / the Flyer (lane A/B) calls `want` + `set_points` with its own x, facing
and motion frame; the harness then goes. The landing reaction scripts (`grColDataSet` tables 1-2) are not played.

### Kinetic units: kind 0, the placement path, R60 (2026-11-14, read from the code)

- Kind 0 runs the directed-unit code (0x81A1D298, like kinds 8-13). Word 9's marker 35790 becomes the angle. 0x81A1EC18
  maps it to 270 degrees with no mirroring, so the unit moves straight down. Its limits hold the "none" marker, so
  there is no cap. A "keep" start speed is 0 when the slot's previous engine kind is unset or in another category
  (0x81A1B370, 0x81A1AE30; see the geno-vm4 fix below).
- Value 41000 / 6602's argument 2 is the placement's path object (create parameter [off] -> Enemy [off]), a
  grGimmickMotionPath. Kinds 21 / 22 (0x81A1D2D4, 0x81A1DFCC) follow it: each frame they go to the path point at
  frame [off] plus the offset values 3054 / 3055; the frame advances by the speed, wraps in loop mode (word 12 = 1),
  and the unit is marked finished at the end (0x81A1C110). The host passes the path as `Input::path`. The GEG1 field is traced
  (2026-11-15, below). Without a path, the unit heads straight for `home` [L].
- R60 = module accessor [off] vfunc 0x20. [off] is the catch (grabber) module (accessor constructor 0x81A6821C);
  soCatchModuleImpl 0x81A64F80 returns true while a victim is held.
- 661D slot kind [n] [flag] (0x81A19684 -> 0x81A16CFC), ported: the move vectors of every live unit are added up and
  the category-0 units ended; with a non-zero sum, a free slot and a directed record (kinds 0 / 8-13) whose speed word
  holds one of the three "from the vector" markers, the slot restarts with speed = the vector's length, direction = the
  sum reversed (flag 0) or with its larger axis reversed (flag 1), angle = the record's or (marker) the direction's
  angle for the facing (0x81A15A00). Bullet Bill runs it when reflected: a bounce.
- 661E joint (0x81A1AA1C -> 0x81A1F3B8), ported as an event: the joint turned about z to the summed movement's angle
  (negated facing left). The VM has no per-joint rotation, so the host gets `EV_AIM` (`ins` = the instruction, `v` =
  the angle's f32 bits). Hosts ignore it today.
- R75 (requirement switch case 0x4B -> 0x81A7AA6C) = accessor [off] vfunc 0x3C. The accessor build (0x81A80C24) puts
  soReflectModuleNull there (typeinfo name), so [off] is the reflect module; soReflectModuleImpl's 0x3C (0x81A64044)
  is "reflector id ([off]) set", stored by the hit callback 0x81A640AC on a reflector hit and reset by vfuncs
  0x20 / 0x24. New `Input::reflected`.
- The accessor's slots are decoded from the soModuleAccesser constructor's mangled name (0x81A68598) and, for the
  unnamed slots, the Null classes' typeinfo strings in the 0x81A80C.. build. The map is in the local boss-ir notes.
- Angle markers on a record start (0x81A1EF60): 35791 and 35789 keep the slot's current angle (was: taken as degrees;
  changes Tabuu and Jyk, not the six-boss set: their 3 x 3600-frame headless hashes are unchanged).
- Fixed (geno-vm4): the "keep" start speed compares the old and new units' engine kinds, not the kind numbers. The
  record start (0x81A1B0C0) copies the unit's words to the stack before it loads the record, so "previous" is the old
  engine kind and "new" is the record's word 0. 0x81A1B330 / 0x81A1B370 -> 0x81A1B39C, for a start speed whose integer
  part is 35791 / 35789 / 35788: 0 when the old engine kind is unset (0x81A1AE10: outside 0-24) or of another
  category (0x81A1AE30); 0 when a directed old kind (0 / 8-13) goes to a root-movement one (1 / 14-19 / 21-24); else
  35791 the old speed, but the length of the unit's vector (0x81A10830) for a root-movement old kind going to a
  directed one; 35789 / 35788 the vector's x / y. The VM uses its move vector for the unit's vector [M] (zero for
  root-movement kinds: not modelled [L]); 35790 (0x81A1B564: a random speed between the limits) is not modelled.
- Why Porky's hash changes: Porky's own records are kinds 14-34 with engine kinds 0 / 1. The kind-number check sent
  every kind >= 25 through 0x81A1AE10's range test as "unset", so 25-34 with a 35791 start (Porky's 31 -> 31 / 30 /
  33 -> 31, 30 -> 30, 27 -> 28) always restarted at 0. The engine kinds (0 / 1, both category 0) are in range, so
  those starts now keep the slot's speed. Three starts go the other way (23 -> 25, 23 -> 19, 21 -> 17: root-movement
  to directed, now the vector's length, 0 in the VM; before, the old speed). The code reads the copied
  engine kind ([off] from the stack copy), so the new behaviour is the right one.
- Bullet Bill: after the bounce (661D kind 20, then 6600 18) the script starts kind 20 on the slot again from kind 18;
  that start now keeps the speed (vm_bullet_bill asserts it through `vm::start_unit`). The same frame then starts 18,
  whose start speed is a number, so Bullet Bill's hash does not change.
- Gate reading: 0x81A1C9C4 is "a flag bit clear" (not running), separate from the finished flag [off]
  (0x81A1C110) that the VM's `done` stands for; 661D / 661E skip only off and paused units [M].

### The placement path source, the running bit (2026-11-15, read from the code)

- sora_adv_stage.rel's enemy creator copies its GEG1 record and, when the record's model-index byte is not 0xFF,
  builds `grGimmickMotionPath::create(index, "path_locator", "grGimmickMotionPath")` (0x81A124EC -> 0x81C6E13C) and
  starts it on the section archive: model data (archive file type 2) number `index`, a small BRRES with one MDL0
  (`enemy_moveNN`) holding a `path_locator` node and its CHR0. The creator hands the object to the enemy through the
  emCreate it builds at 0x81A12F14 (copied at 0x81A130A0; `emManager::createEnemy` 0x81A0C9D4 called at 0x81A131B4).
  getTRS (0x81C6EB6C) poses the model at the frame (vfunc 0x178, setFrameModelAnim) and reads the node's position.
- explore.rs: archive leaves carry their (type, index) path (`pac_leaves_tagged`); every GEG1 spawn gets its path
  (`Map::spawn_path`), sampled through a bone-only rig (`pm::locator_rig`; `mdl0` refused models without triangles);
  `enemy_vm_place(stage, spawn)` gives the VM `Input::path` and `Input::home` (the spawn point). The path's frame 0 is
  the spawn point. The Jungle has 24 placement paths, 13 on Bullet Bills (060003), 241 frames each.
- Proof (`jungle_bullet_bill_paths`, ignored test, owner's files): Bullet Bill status 4 rides its real path (speed 0.7,
  loop mode) with a fixed offset from the path point at its frame 1195 / 1200 frames; replays from frame 600 are
  identical in all 14 statuses for three Bills. The program's arena box is a boss arena's; the test widens it (a map host
  passes walls and floors).
- Running vs done: 0x81A1C9C4 tests the unit's running bit; the record start (0x81A1BE44) sets it and clears the
  finished flag, the kinetic stop points (0x81A1DCF0-0x81A1E298) clear it. The path movers' end (0x81A1DFCC-0x81A1E0F8)
  only calls 0x81A1C110 (finished): a looping rider is finished from the first wrap on and still running, as the VM
  already had it. Requirement switch (sora_enemy table, ids 10000-10059): 10000 -> 0x81A1794C (off or not running),
  10001 -> 0x81A179E0 (off or held: 0x81A1CA10, the pause byte), 10002 -> 0x81A17A74 -> 0x81A1CA50, 10003 ->
  0x81A17B08 -> 0x81A1CAE4, 10055 -> 0x81A15078, 10006 / 10007 -> 0x81A13C14. Ported: R10000 counts a done path rider
  as running; R10001 is "off or paused". No boss uses R10001, and Duon / Porky's R10000 never sees a path rider, so the
  boss set is unchanged (headless hashes identical).
- Open, read but not ported: 0x81A1CA50 is "speed >= a record word" (fcmpo speed, word; true when not less), the VM's
  10002 is `spd <= max`; 0x81A1CAE4 (10003) is "speed <= another word". Which VM fields those words are is not mapped
  [L]. R23 (sora_melee case 0x17 -> 0x81A78A70): model module vfunc 0x1F8 (a float argument, a buffer the resource
  module's vfunc 0x18 fills, a flag) >= 0; the vfunc is not identified yet.

### Round 5 (2026-11-15): batch-2 SSE enemies

- The VM's FILES list holds the 22 new regular enemies (all 40 SSE enemy classes now load); `class_of` maps them to
  their accessor classes, and feyesh / glunder were corrected (Pacci / Galfire, from their accessor symbols). The local
  accessor map's 44000-44004 entries of the if-tree accessors were re-read (44000 is the block base itself); that
  added the missing 44003 and fixed the others for those classes (local data only).
- New test `vm_enemy_statuses`: every batch-2 enemy (or `MELEE_VM_ENEMIES=a,b`) headless from every start status,
  1200 frames, replay from frame 600 identical. All 22 pass.
- Ported: R10055 (0x81A15078: emAreaModuleImpl contact with area n of kind 13 via 0x81A27688; host input `area`),
  6D00 (emCollisionHitModuleImpl, command kind 0x6D, 0x81A2BC5C -> 0x81A2BA44: the hit module's vfunc 0x44 on each
  hurtbox of param 43006, word 2 of the data root), 100D (soGenerateArticleManageModuleImpl case 13 -> vfunc 0x64
  0x810977A0: max less live articles of a kind; host input `article_room`), 6508 (0x81A06660, integer 6509), 651B
  (0x81A0872C, percentage with the module constants 0 / 100), 6521 (0x81A06C44, a node position [L: no skeleton]),
  R56 (link module vfunc 0x168 = emLinkModuleImpl 0x81A28530: partner id != -1; host input `linked`).
- Forwarded to the host: the link module's 0F00-0F02 (command kind 0x0F, soLinkModuleImpl), 6519 (copies a variable
  into the linked enemy through the enemy manager: 0x81A0B438 -> 0x81A0B1E8) and 6527 (emGroundModuleImpl
  0x81A2B630: ground-collision shape pairs).
- Read, not ported yet: R10052 (0x81A14D38 -> 0x81A0AEC8: a stage-wide manager (sora_melee 0x812331C4 count /
  0x81233208 entry) asked whether all, or entry b, of group a exist), R10053 (0x81A14E70 -> 0x81A0B34C).

### Round 6 (2026-11-15)

- Ported: R10052 (needs a free enemy-manager slot (0x81A0CEF0), then group a's count and its entries; host inputs
  `enemy_room`, `groups`), R10053 (the linked enemy is missing or its work flag 0 is set; host input `partner_alive`),
  21009 (a create-record word the constructor keeps, 0x81A003E8; `Program::place_word`, meaning not traced) and
  21010 (the strength byte the level getter 0x81A03678 reads). The VS host sets none of the new inputs: R10052 stays
  false as before for Meta Ridley; R10053 only matters for the Falcon Flyer, which is not a VM actor yet.
- Read, not ported: 6901 / 6902 share one case of the search-module command switch (0x81A26510, case 1 / 2: five
  arguments: slot, x, y, out variable, flag). 6901 calls 0x81A25C60 (target position from the search slots 0-7,
  0x81A25524 / 0x81A2619C / 0x81A262E8), 6902 calls 0x81A25DF0 (the same body on the player / slot-kind targets,
  0x81A23A0C / 0x81A23CD4 / 0x81A23E0C, slots 0-3 players, 4-5 other kinds). Both offset the point by (x, y) through
  0x81A24278 and hand the actor-to-point vector to 0x81A25BD0; the out variable is then written. The VM's 6901 (an
  angle by atan2) is a reading of that, not yet checked against 0x81A25BD0; port both together.

### Round 7 (2026-11-15)

- 6901 / 6902 ported together (search-module switch 0x81A26510, sub-ops 1 / 2 -> 0x81A25C60 / 0x81A25DF0). The point
  is the slot's target plus the offset (x * ax, y) * scale (0x81A24278; its rotation call gets angle 0), the angle is
  0x81A25BD0 -> 0x81A15A00 in degrees [0, 360) from the actor's facing: the f1 the callers leave is the posture
  module's lr (vfunc 0x2C on the posture module, the last call before 0x81A25BD0, whose float result is the argument).
  ax is the target's facing when the flag is set (0x81A23CD4 / 0x81A2619C), else 1; scale 0x81A23E0C / 0x81A262E8.
  6902 reads the target now (0x81A23A0C), 6901 the search snapshot the same helpers fill (0x81A25440 loop), i.e. the
  VM's `found`. Slots 0-3 players (absent: point 0, ax 0, scale 1); 4-7 point 0, factors 1 (kinds 4 / 5 linked
  objects and 6 a module point are not modelled [L]); others point 0 and, for 6901, factors -1. The out variable is
  written for any slot (the old 6901 skipped it). The actor point is the actor's position (the code's node is the
  enemy object's own matrix). New host inputs `target_lr`, `target_scale` (0 = 1).
- Kinetics: 0x81A1E658 raises the speed to the lower limit whatever it was before; the VM clamped only on a crossing,
  so Meta Ridley's status 26 drift (speed 0, decay, floor 0.4) never moved. Only the crossing still ends the record.
- R10059 (0x81A15284): no arguments; a flag byte of a global sora_melee object (created and cleared at 0x81C21B30, set
  at 0x81C51C04 when one of its timers matches) [L: meaning not traced]. Host input `global_flag`, clear in VS.
- Proof: Meta Ridley leaves status 26 at f930 in the in-game fight row (26 -> 2 -> 3 -> 22 -> 23 -> 26 -> 2 ...,
  14 status changes; was stuck from f169). Seven-boss set (merge with geno-boss3 1e4b3c905): fight rows ok for all,
  sync d4 / d7 0 mismatches, NHV1 identical twice, Ridley / Rayquaza root and Glunder crawl ok. vm_disc passes.
- Host (geno-boss3 lane): set `target_lr` / `target_scale` per player from the fighter's facing and model scale;
  `enemy_room` / `groups` (R10052) and `partner_alive` (R10053) are still unset in VS, where no boss needs them
  (R10052 false for Meta Ridley as in Brawl's VS, R10053 is the Falcon Flyer's).

### Round 8 (2026-11-15)

- R10054 (0x81A14EB0 -> 0x81A0B5EC): X = the enemy manager's live entries, Y = those whose enemy's id word is a (0 when
  the static object's vfunc 0x0C gates a); d = b - (X - Y) * f in single precision, true when d > 0 and d > Y. Host
  inputs `enemy_live`, `enemy_ids` (Bytan; no boss uses it).
- R10057 (0x81A15204 -> 0x81A06EB4 on the actor, the 0x803F0F44 call being a __dynamic_cast of the actor): work vfunc
  0x50 sets the actor's own work flag 17, the manager walk 0x81A0CC44 with callback 0x81A06DDC clears it (vfunc 0x54)
  when a live enemy linked to the actor is not dead, and the result is the flag (vfunc 0x4C). The VM writes flag 17 on
  every evaluation; host input `child_alive` (Gamyga head). R10053 re-read with the same cast: unchanged (the actor's
  own entry's link id, then that partner's dead flag).
- `cond` / `chain` / `skip_false` now take `&mut self` for that write. vm_disc passes; `disc` timeline_baskets /
  timeline_waits fail before and after this round (not caused here).

