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
