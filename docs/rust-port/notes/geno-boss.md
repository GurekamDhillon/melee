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
target draw / lock / side tests, L tables and strength changes, cage HP `trunc(120 + 15 L)` at spawn, damage routing
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
| 4 | death | status 15: stage event 1, sound 8837, stage event 1000/1001 for the lower cage, no Down |
| 5 | Down end | Down's stage events 5/6/7/8 are emitted; the fight still ends at Down frame 180 (host, stage-side) |
| 6 | cage hit to 0 | the break path runs: 0x13FF -> stage 1001 -> cageBroken -> rule 10 -> Down; rules 6/7 stay off; a cage hit is not hitByPlayer |
| 7 | phase thresholds | per cage, own pct vs own counter (5106 sets both) |
| 8 | threshold on a damaged frame | msg 1 instead; 0x1400/0x1401 on the next undamaged frame |
| 9 | body contact in JumpAttack | unchanged (host hit table) |
| 10 | cage HP | `trunc(120 + 15 L)` from the IR: 157 at Normal, scales with Envoy's difficulty |
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
| `geno_boss_check.sh idle death break bodybreak` | all ok: loop and wait lengths, roar 1 frame into the first wait after 50%, phase 2 at L 4.1, threshold-then-hit death (status 15, stage events 1 and 1000, no Down), cage break (0x13FF, stage 1001, Down, KO), body break (stage 1000, Down, KO) |
| synctest (death schedule, 3000 frames) | depth 4: 2997 passes, 11988 hash checks, 0 mismatches; depth 7: 2994 passes, 20958 checks, 0 mismatches |
| NHV1 (golden, `pad_falcon_fox_fd` + Envoy fight 5, 3000 frames) | identical over two runs; last row frame 2999 hash `1CB3982F`, seed `777F059B` (the boss state word is mixed in while the actor is live) |
| frame times (real time, profiler, 2199 frames) | 0 frames over 16.7 ms (one startup frame, the match load) |
| VS (`MELEE_VS_BOSS=petey`, no mods) | his arena, the Geno actor, death at the scripted hits, carrier KO |

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
