# Geno v5.2: enemy-engine actors (bosses and small enemies)

Status: v5.2 draft 1 (2026-11-14), implemented in `rust/melee_modes/src/tu/geno_boss.rs` (runtime) and
`geno_boss_ir.rs` (IR lowering). First proof: Petey Piranha (notes `docs/rust-port/notes/geno-boss.md`).

Geno v5 extends *fighters*. v5.2 adds a second kind of Geno object, the **actor**: a data-driven enemy run by a
generic rule/status engine, the shape of Brawl's enemy engine (statuses with enter and per-frame scripts, a
prioritised rule table, weighted selection tables, kinetic kinds, timelines, a level scalar L). The same actor
serves a boss (several parts, HP pools, phases, custom defeat) and a small enemy (one core part, a few rules).
Nothing in the engine names any enemy: an actor is a *define*, from a mod or lowered at load time from a local IR.

## 1. Define

| block | content |
|---|---|
| `parts` | 1-4 parts: role `core` / `weak_point` / `limb`, HP (`Num`, truncated at spawn L), per-part procs (§3), the statuses where the part is intangible, `int_level` (the part's attacks use trunc L) |
| `vars` | up to 24 named f32 variables with initial values (flags are 0/1) |
| `statuses` | index, name, `enter` ops, `frame` ops (`frame_once` = a one-shot sequence) |
| `rules` | index, priority, default enabled, condition, action ops, optional target status, `per_target` (draws and locks a target first) |
| `tables` | weighted entries (`Num` weights, a repeat factor on the stored previous pick); an entry runs ops and/or picks a motion |
| `kinetics` | `Decay(Num)` or `Accel { start, acc: Num, max: Num }` along the aim |
| `motions` | name, length, looping, timeline events `{frame, count, every, op}` |
| `messages` | message id -> handler ops (§3) |
| `level_rows` | L = rows[strength-1][difficulty] (0 out of range); strength starts at `strength0` and changes with `SetStrength` |
| target | four slot weights + repeat factor; lock / reset |
| routing | core damage -> part mask x factor, applied by `Forward` |
| defeat | the status that means "defeated by threshold" (cause 1) and "part broken" (cause 2) |
| arena / camera | spawn x, goal snap, camera distance / ease / tilt / fov / limits |

`Num` = `K(c)` | `Lin(base, per_level)` (= base + per_level * L) | `Var(v)`.

Conditions: `Always`, `Never`, `MotionEnd`, `TimerDone(var)`, `Var(v op k)`, `VarVar(a op b)`, `StatusIn(mask)`,
`PrevStatusIn(mask)`, `Hit(Any|Air|Ground|Behind)`, `MinWeakPctLe(x)`, `PartPctLe(part, x)`, `Target(unlocked)`,
`And`, `Not`. Ops: see `geno_boss::Op` (rule enable / reset / disable, target lock, kinetics, timers, vars, motions,
weighted picks, `If`, status changes incl. by target side and by sending part, goal x, aim, stage events, sounds,
shakes, intangibility, damage forwarding, strength, phase counters, windowed evaluation, hitboxes on/off, despawn).

## 2. Frame order

1. Input: the host reports player damage per part (`damage(part, n)`): core damage goes to the core pool and is held
   as this frame's `dmg_core` for `Forward`; a weak point takes its own hits directly. Intangible parts take none.
2. Rule evaluation (§4).
3. The status's frame ops.
4. Timelines of the current motion (events between the last fired frame and now; repeats `count` x `every`).
5. Part procs (§3); their messages run their handlers synchronously.
6. Movement: the kinetic, then the goal-x snap (`|goal - x| <= 0.1 + |vx|` -> x = goal).
7. Counters: status frame +1, motion frame +1 (a looping motion wraps; others hold their last frame).

## 3. Parts and messages

A weak point's procs: on a frame it lost HP it sends `on_damaged`; otherwise, while its counter equals a threshold's
counter and its percent <= the threshold, it sends that message (every frame while true); on reaching 0 it sends
`on_zero` once. Handlers are op lists; `EvalWindowOfSender` runs one evaluation restricted to a single priority
(per sending part) and ends the handler if a rule fired; `StatusOfSender` / `StageEventOfSender` pick per part.

## 4. Rule evaluation

The define lists its evaluation passes; the shared enemy engine's (enemy-engine IR) are: priority -100 first, then
the damage window -250..-150 on a frame the core was hit (a hit reaction), then the normal window -50..250. A pass
that changes the status ends the frame's evaluation. Within a pass: enabled rules whose condition holds, lowest
priority first, table order on ties. A rule with a target status runs its
action, enters the status and ends the evaluation. A rule without one (an action-only rule: damage forwarding, a phase
step, a notification) runs its action and the evaluation goes on; at most one rule fires per priority level. This
fall-through is the v5.2 reading of "one rule per evaluation": with strict one-rule semantics an always-true
notification rule (Petey's rule 30 after a cage break) blocks the status rules of a one-frame status forever.

## 5. Determinism, rollback, hash, cost

The define is immutable after load (`OnceLock`, lowered on a helper thread so its heap stays outside the game arena).
All mutable state is `geno_boss::State`: fixed size, plain data, one static, own LCG (`x * 1103515245 + 12345`,
`>> 8`), no wall clock. Engine memory snapshots cover it. `state_hash()` (FNV-1a over the state words) is mixed into
NHV1 by `RB_NeutralHash` through `melee_fighter::tu::fighter::BOSS_HASH`, only while an actor is live, so every
other NHV1 row is unchanged. Arithmetic is f32 with no fused multiply-add (v5.1 §1). A frame is a few dozen ops and
condition tests: no allocation, no search beyond the 64-rule table.

## 6. IR lowering (load time)

`geno_boss_ir::load(file, host_motions)` reads `<dir>/<file>` where dir = `MELEE_BOSS_IR_DIR`, else
`<MELEE_BRAWL_LOCAL>/boss-ir`, else the first `_build/boss-ir` above the working directory. The IR is the local
analyst file (`ports/ir/schema/boss.schema.json`); it is never committed. The lowering knows only the IR's
vocabulary: op names, the cited engine primitives (`CommonAICheckDamage*`, `CommonNoneFlyDamageProc`) and its
condition phrasing (`statusFrame >= T`, `phase == 0 and status in {2,3}`, `jumpCount >= jumpsWanted`,
`target.x < boss.x -> a else b`, `counter 0 and pct <= 50 -> send 0x1400`). Every number comes from the file. An
unknown phrase is a load error listing each one; the actor is then unavailable and the host falls back (Petey: the
hand-coded boss), so a boss never runs with a silently dropped rule. Presentation-only ops (`effect`) lower to no-ops.

## 7. Host contract

The host (a mode, e.g. Envoy's Petey) supplies the world each frame (target candidates' x, airborne), applies the
actor's events (`EV_SOUND`, `EV_SHAKE`, `EV_STAGE`, `EV_STATUS`), draws the parts, turns hurt/hit shapes into the
engine's collision, and ends the fight on `cause != 0`. Mods drive an actor through the same calls (`begin`,
`damage`, `tick`) with a define they build.

## 8. Planned (designed, not yet in the runtime)

From the Ridley / Galleom / enemy IR gap rows (geno-gaps R1-R8, G1-G5):

| need | design |
|---|---|
| area-out defeat (R1) | `Cond::OutOfArea` from a host world flag, used by a priority -300 rule whose status is the defeat status |
| HP-percent enrage (R2) | already expressible: `PartPctLe(core, 50)` rule -> `SetStrength`; L is live everywhere |
| fixed rule cycles (R3), interstitial turn (G1) | a counter var + `Var` conditions; `Hit(Behind)` needs a facing var |
| grab of a player (R4) | `Op::Grab { part, joint, release_frame }`: the host attaches the victim to the part's joint (Melee's catch state) |
| multi-slot flying movement, goal points, arrival (R5, G3) | kinetic *slots* (3-5) summed per frame, `Kinetic::Toward { goal, speed: Num }`, `Cond::AtGoal` |
| offscreen staging (R6) | an actor flag that suspends blast-zone checks and camera framing while set |
| spawned projectiles (R7, G2) | `Op::Spawn { record, joint }` -> a Geno article owned by the actor, per-define pool, L-scaled speed |
| multi-hit groups (R8), L-scaled pushers (G5) | `HitboxDef` already carries `Num` damage / kbg / bkb / size and a group; WKB becomes `Num` |
| defeat that waits for a safe status (G4) | defeat statuses gain a guard mask: the defeat rule fires only when the current status is in it |
| two stances with their own motion sets (Duon) | a `stance` var; `Op::Motion` resolves through a per-stance motion map (motion index -> stance motion) |
| weight tables by target distance / direction (Duon) | table entries gain an optional `Cond` (distance band, side) evaluated before the draw; zero weight when false |
| minions with their own state machines (Porky) | a minion is another actor define; `Op::SpawnActor { define, joint }` with a per-boss pool; messages flow both ways |
| homing hazards | an article kinetic `Homing { turn: Num, speed: Num }` toward the locked target |
| moving collision players stand on | a part may own a collision line set that follows a joint (the host moves the stage line each frame, as moving platforms do) |
| a second animated part | already: parts play their own motion index; `animation_sync` = lock-step with the core (Petey's cages) |
| walk-cycle continuation | `Op::Motion` with `keep_frame` (carry the motion frame across a status change) |
| multi-threshold HP phases | already: several `PartPctLe` rules (or thresholds on the core part) with `SetStrength` / phase vars |
