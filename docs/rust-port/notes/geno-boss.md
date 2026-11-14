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
  ChainStart (@0x3edc) and ChainThrow (@0x3e1c) they run his slot-5 approach kinetic (kin 23 / 24) by the target's side
  and range (R10026 / R10032: one handler for R10024..R10063, target vs enemy position through a per-id axis mask; the
  compare tail is [I]). The catch box is only 060A / 060D. Lowered (`approach_gate`, from the per-frame phrases) as lock,
  aim, `If(TargetDx > catch radius, Kinetic(slot speed), StopMove)`; new `Cond::TargetDx(Cmp, k)`. The host lets the
  actor's movement drive the carrier on the ground.
- **Bucculus's chew** (`bucculus_define`, `MELEE_VS_BOSS=bucculus`): Bites1 catch boxes (060A f14 / 20 / 28, r 1 / 2 / 7)
  -> Absorption (bite on its hitbox frame) -> Bites2 -> R21 every param 4003 (33) frames -> R23 at param 4002 (240) frames
  since the catch -> RollingJump, spat out with the spit-out box. New `Op::GrabHit(attack)` (`EV_GRAB` 4): the host
  applies the box's damage (`Fighter_TakeDamage_8006CC7C`) and Melee's CaptureDamage reaction. The hold chain 10 <-> 12
  comes from the enemy-style exits (`if status != N: CommonCatchProcExit`, now read by `hold_mask_from`); the catch ->
  Absorption hand-over is native in Brawl (unread [L]) and is `Holding` in Bites1 here. The passive grab-timer decay is
  given back in CaptureDamage too (a bite let the timer run out).
- `GrabState` gained `hit` (end of the block; hashed).

Still not done: the R10026 compare tail (thresholds 4003 -30 / 4004 65 not used; the catch radius stands in), Tabuu's
warp to x +-128 before ChainStart, Bucculus's lunge steering / burrowing / search box (|dx| <= 15 stands in), the 061A
base knockback word [I]. `port/geno-vm` had not landed: nothing to hook yet.

### Proof (2026-11-14, release exe of this branch)

| check | result |
|---|---|
| unit tests | `geno_boss` 7/7 incl. `geno_boss_grab` (catch window, hold, throw request, drop on leaving the hold, the local Tabuu IR lowers to Chain r13.5 f89-106 hold 36\|37 and Dive r14 f119 hold 19\|20\|21, throws ChainAttack 20+2.5L @361 kbg 68 bkb 90-1.5L and Dive 30+2.8L @22 kbg 75) |
| `geno_grab_check.sh hold` | Chain catch at ChainStart f89 -> ChainThrow -> thrown at f135 (26.2 at L 2.5); Dive catch at f119 -> DiveTeleport -> DiveSlam -> thrown (37.0); no escape without a mash |
| `mash` | caught at f89, mashed out (CaptureCut) before the throw |
| synctest (hold schedule, 3000 frames) | depth 4: 11988 checks, 0 mismatches; depth 7: 20958 checks, 0 mismatches; grabs and throws inside both |
| NHV1 (`golden`, hold schedule, 3000 frames) | identical over two runs; last row frame 2999 hash `ED7C78E4` |
| frame times (real time, `play`, Falcon vs CPU Fox carrier, profiler, 2999 frames) | 0 over 16.7 ms (+1 startup frame, the match load) |

Second round (release exe of this branch, `geno_grab_check.sh` all rows OK):

| check | result |
|---|---|
| unit tests | 8/8 (adds `bucculus_chew_from_ir`: caught, 4 bites, spat out) |
| `chase` (Falcon idle) | Tabuu walks to him in ChainStart (slot-5 kinetic) and catches |
| `chew` | Bites1 catch -> 4 bites of 10 (10 + 0.7 L, L 0) 78 frames apart -> spat out 281 frames after the catch (R23 is live in Bites2 only) |
| `chewmash` | caught, mashed out, Bucculus back to WaitBuried |
| synctest 3000 frames | Tabuu depth 4 / 7 and Bucculus depth 4 / 7: 0 mismatches each, grabs and releases inside |
| NHV1 | identical over two runs: Tabuu last `F4B351BC`, Bucculus last `88E9D1E8` |
| frame times (real time, 2999 frames each) | Tabuu and Bucculus: 0 over 16.7 ms (+1 startup frame) |
