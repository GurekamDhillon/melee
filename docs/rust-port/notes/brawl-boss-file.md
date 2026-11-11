# Brawl boss file: decoder and script runner (`rust/brawl_boss`), phase 1

2026-11-11, branch `port/bossfile`. Clean-room: written from the reader's prose specification only (no Brawl code,
no decompilation). Phase 1 adds a standalone crate and changes no gameplay; `petey.rs` is untouched.

## What the crate does

`brawl_boss` reads Petey Piranha's boss parameter file from the player's own Brawl files and runs its scripts. It
depends on no game crate, is `no_std` + `alloc`, has no `unsafe`, and keeps everything that will reach the
deterministic simulation in integers and fixed point.

| module | contents |
|---|---|
| `archive` | `ARC\0` containers and Nintendo LZ77 (0x10 / 0x11) as pure functions over slices; `find_petey_file(stage_archive)` walks the nested, compressed entries and returns the leaf whose first word equals its length and whose name table carries `wpnHitDataSet_ArmLeft`. `archive::build` makes synthetic ARC / LZ77 inputs for tests. |
| `file` | header, relocation list, roots (exports), the import chains, the 28-word boss record, the motion table, the 3 x 3 per-motion timeline tables (game / effect / sound, three threads each), the status enter / loop / exit tables (17 slots), the rule table and its 24-byte records, the init script, the general and tuning blocks, the hurtbox sets, the cages' small command arrays, and every reachable script, by offset. |
| `params` | the parameter block by param id (ranges, weighted lists with their condition scripts, movement kinds, cage spawn and proc records). |
| `script` | the command container and the typed command set (`Op`): the 63 command ids that occur, each with typed arguments. Every `Command` keeps its raw word and its argument list, so an unknown id or an unexpected argument shape decodes as `Op::Unknown` without losing anything. |
| `runner` | threads for timelines, status scripts and the cages' linked scripts; the control commands; the rule engine; the 16 imported common scripts as native built-ins. |
| `world` | the `World` trait (what the runner asks of and tells the game), the `Event` list, `Recorder` (a logging world for tests). |
| `fx`, `level`, `rng` | `Fx` = value x 60000 in an `i64` (the file's scalar unit, so script scalars are exact; f32 words convert once, by integer arithmetic); the level table; the `RandomSource` trait. |

Robustness: every read is bounds-checked and returns `Error`; table lengths, import chains, script lengths, archive
depth and decompressed sizes are bounded. The tests truncate the file at every length and corrupt random bytes
(thousands of cases, plus a run of the runner on every corrupted file that still parses) without a panic.

## API in brief

```rust
let file_bytes = brawl_boss::archive::find_petey_file(&stage_archive_bytes)?;
let bf = brawl_boss::BossFile::parse(&file_bytes)?;           // owns its data; decode once per process
let mut boss = brawl_boss::Runner::new(&bf, Config::default()); // plain data (Clone) apart from the &BossFile
boss.start(&mut world, &mut rng);                             // L, init script, opening status
// each frame, after movement:
boss.step(&mut world, &mut rng, &FrameInput { hp_now, hp_before, hit_by_player });
// cage reports (spec 6): Damaged, PhaseOne, PhaseTwo, Broken, Resync
boss.cage_event(&mut world, &mut rng, side, CageEvent::Damaged);
```

`World` answers motion lengths (the file does not carry them; the CHR0 counts), slot presence, a target's x
offset and angle, the cage HP percents (4001 / 4002) and the shared damage checks, and receives every `Event`:
motion changes, resolved hitboxes (level slopes applied, integer fields truncated toward zero, the 25 flag fields
kept), clear hitboxes, hurtbox state, effects, sounds, shakes, movement commands, goal X, stage notifications, cage
spawn and link events, rule enable / reset, target lock, status changes, death, and built-ins with a world-side effect.

Frame order inside `step`: frame input into the engine-maintained variables, the rule engine (a status change runs
the enter script at once), the status loop thread, the nine timeline threads (game 0-2, effect 0-2, sound 0-2), the
two cages' linked threads, then the frame counter. All randomness is drawn from the caller's `RandomSource`, one
`next_u32` per draw, in this fixed order: the rule engine's slot draw (only when the lowest-priority true rules span
two or more slots), each weighted switch the chosen action executes (none when every weight is zero), each 6507 range
in the enter script that follows (always one draw, even for an empty range), then the loop thread and the timelines.

## Where the spec was unclear: the switches and assumptions

Each uncertain reading is a named `Config` field; the default is the best reading.

| switch / assumption | default | why |
|---|---|---|
| `AsyncWaitInLoop` (0002 inside a loop) | `Absolute` | The format spec reads the basket's `20 x {shake, wait-until 6}` as "every 6 frames". Taken literally that loop runs to frame 190, which pushes the cage's frame-115 hitbox clear past the 160-frame motion, contradicting the observed 70-114 hitbox window (spec.md 8). The absolute reading keeps 70-114 (and puts the 20 shakes on frame 70; JumpLoop's shakes on 0, 10, 10). `IterationRelative` is the alternative. |
| `WeightRounding` | `Exact` | Whether switch weights are truncated before or after the 0.3 cut (format spec 4.3.5). Exact fixed point = float weights. |
| `RuleTieBreak` | `SlotDrawThenTableOrder` | Equal-priority ties (format spec 9.4). Slotless true rules stay candidates next to the drawn slot's rules; the first in table order fires. The repeat cut uses the slot the engine last fired a rule for. |
| `common_slot_fallback` | on | A null status entry uses the common slot 16 (enter / loop / exit). The spec says slot 16 is common but not whether it is a fallback or always runs. |
| `initial_status` | 2 (Wait A) | The spec does not name the status the fight opens in. |
| conditions with side effects | run every frame | Every enabled rule's condition runs each frame in table order, side effects included: rule 28's condition does the whole phase-up, rule 30's notifies stage event 9, rule 5's runs the death check. The data is built that way. |
| a condition is true | temp flag 4 set at its end | Cleared before each condition. A switch entry with a condition script counts only if the condition sets flag 4. |
| status requests (0202) | applied after the script that issued them | The script continues (the basket actions set `side` after their 0202). A rule's own target status wins over a 0202 in its action; Petey never has both. |
| immediate scripts | a wait ends them | Conditions, actions, enter / exit and init run to completion in the frame; none of Petey's wait. |
| loop motions | timelines do not restart on wrap | Only 0409 restarts them. "Motion ended" is true on the last frame of each pass. |
| cage L at spawn | the boss's L truncated | The init script never sends link 5112, yet spec.md 8's phase-0 basket damage uses the cage's truncated L. |
| 6900 (lock) | locks the slot being evaluated | The argument names the target handle (21004); reading 21004 gives the locked slot, else the evaluated one, else -1. |
| 5106 (cage counter) | the last argument | Sent as (0, n); spec 9 item 9 says the second argument is unverified. |
| 0010's first argument (1000), 1B02, 0808 / 0900, 0410 | decoded and kept; no runner effect except events | spec 9 item 6. |

## The 16 built-ins (format spec 4.6)

| import | native behaviour | confidence in the prose |
|---|---|---|
| `CommonStandBy`, `...Init_...`, `...Exit_...` | nothing (idle placeholder) | high: only on the unused common slot |
| `CommonAICheckDamageAir / Ground / Turn / NoReaction`, `CommonAICheckReactionEnd` | condition true when `World::damage_check` says so this frame | medium: "true when hit this frame in that situation"; the split between the four is the game's to decide |
| `CommonAICheckMotionEnd` | true on the motion's last frame | high (matches requirement 2 and the observations) |
| `CommonAICheckDeathFigure` | true once the death check has fired | medium: the death check itself already sets the condition |
| `CommonNoneFlyDamageProc` | `Event::Builtin`: the world takes the hit with no knockback and no status change | medium |
| `CommonAIExecDamageTurnNoRot` | `Event::Builtin`: turn without rotating | low: the boss never rotates, so it may be a no-op |
| `CommonDyingFigureProcInit` | all rules off, target cleared, intangible, `Event::Builtin` | medium |
| `CommonDyingFigureProc` | `Event::Builtin`: the trophy sequence and removal are the world's | low (behaviour only from observation) |
| `CommonDyingNoneEffProc` | rules off, target cleared, stop movement, stage event 0, intangible, `Event::Builtin` (removal) | medium (spec.md 4.3 status 0) |
| `gameAnimCmd_CommonBossDown` | a native script: stage events 5, 6, 7, 8 at frames 0, 1, 180, 420 | high (observed) |

## Tests

`tests/synthetic.rs` builds a small boss file in memory (no real bytes): structure, nested compressed archives,
timeline timing (waits, loops, wait-until in both readings), the scaled hitbox, the init script and the cage's
truncated L, the BossDown built-in, a damage rule, the slot draw with its repeat cut, the weighted switch with its
repeat cut under all three rounding switches, determinism, and the truncation / corruption fuzz.

`tests/disc.rs` runs only when the player's files are present (it skips otherwise and prints no path): the file is
found in the stage archive; header, roots, imports, the record, all tables and every one of the reachable scripts,
command by command, match the reader's decode; the spec's cross-check numbers; each of the 12 motions' timelines
played and checked against the spec's description; a long fight (fixed seeds, scripted hits) checking every status
transition against the spec's table and that the same seed gives the same event log; the RNG draw count of a menu
frame; death on a tie; and fuzz over the real file. Both the reader's decode and the archive stay outside the repo.

`brawl_boss_dump <archive or file> [out]` prints a listing (to the terminal, or to a path that must be under
`_build`).

## Phase 2: feeding Petey from the file, then the boss-fighter library

Step 1, constants (`petey.rs` keeps its structure; values come from a `BossFile` loaded beside the model in
`petey_model.rs`, through the same `local_root()` path, decoded once on the loader thread):

| `petey.rs` today | from the file |
|---|---|
| `CAGE_HP` | cage spawn record word 0 (`params.cage_hp`) |
| `DEATH_PCT` | `death_threshold()` (3057) |
| `PHASE1_PCT`, `PHASE2_PCT` | `params.cage_proc(side)` |
| `STRENGTH` (phases 1, 2) | params 24001 / 24002 |
| the wait ranges, the hover time | 44003 / 44004 / 44005 |
| `pick` weights (menu, sides, after-Angry, wait motion) | 44009 / 44008 / 44010 / 44011, with the switch running their condition scripts |
| slot weights `W` in `target_x` | `slot_weights()`; plus `slot_repeat_factor()` |
| `JUMP_GOAL` | 44006 / 44007 first floats |
| the dive / landing kinetics | 44000 kinds 15 and 16, raw (no scale: the 3056 bounce factor stays out, as the merged port/petey6 already does) |
| hitbox tables (`hit_list`) | the `Event::Hitbox` values for the current L |
| body / cage hurtboxes | `body_hurtboxes`, `cage_hurtboxes` |
| `MOT_LEN`, `L_TABLE`, `DIFFICULTY`, `ARENA` | stay: not in the file |

Fixes the decoded data brings (the spec's disagreement list):
- the 0.3 repeat cut on the basket side (44008) and the attack menu (44009), with `menuPick` kept;
- the 0.3 repeat factor on the last used target slot (`last_slot` is stored today but not used for weighting);
- rule 13 locks the target once per jump (its condition is "no target locked"); the lock is cleared on landing and in
  the menu. Today the goal can flip mid-jump because `target_x` re-picks every frame of JumpStart / JumpLoop;
- spawn state `jumpCount = 0`, `jumpsWanted = 1`; a cage tie at death counts as the right cage;
- speeds without `KIN_SCALE` (already removed on `rust-port`).

Step 2, the runner drives him: replace `rules()` / `enter()` / `advance()` with `Runner::step`, a `World` impl over
the fighter / camera / stage code (movement kinds, goal X, shakes, effects through the Brawl effect reader, sounds
through `melee_audio::brawl`), and the runner state stored in the rollback-snapshotted game state (the runner is plain
data plus the `&BossFile`; the file itself is decoded outside the arena, like the model). The RNG is the game's.

Step 3, the boss-fighter library: move Petey onto `ftbosslib.rs` as a boss kind beside Master Hand / Crazy Hand, the
cages as articles owning their own hitboxes (the cage's script runs on the article, with its own L), the three cage
procs (enter / monitor / broken) native in the article, and the phase-up / break / stun events going through
`cage_event`.

## Questions for the reader

1. Wait-until inside a loop (format 3.12, 9 item 8): the "+6" reading moves the basket's frame-115 cage clear to frame
   190, against the observed 70-114 window. Is 0002 the absolute motion frame (all 20 basket shakes on frame 70, and
   JumpLoop's on 0, 10, 10), or does something else make "every 6 frames" and "clear at 115" both true?
2. The command count: 3.11 says 64 distinct ids, but the table lists 63 (and 63 occur in the file). Is one missing?
3. Which status does the fight open in after the init script (2, or an entry status)?
4. Is the common status slot 16 a fallback for null entries, or does it run in addition to them?
5. The cage's L before the first phase-up: the init script sends no 5112. Does the cage get the boss's (truncated) L
   at spawn natively, or is its L 0 until phase 1 (basket damage 10 at phase 0)?
6. A switch entry whose condition does not set flag 4: skipped, or kept with its weight?
7. 0202 and the rule's target status: when an action requests a status and the rule also has one, which wins, and
   does the script continue after a 0202 (the data suggests it does)?
8. The lock flag on rule 26 (mask 0x10, no slot): what does it lock?
9. Do all enabled rules' conditions run each frame even when a lower-priority-value rule fires (rule 28's condition
   carries the phase-up's side effects, rule 30's the stage event)?
10. Requirement 18's first argument (always 0) and 6901's two zero scalars: meanings.
11. 6907 vs 6908: both read the target's x relative to the boss in the spec; what is the difference?
12. Loop-flagged motions: do the timelines restart when the animation wraps?
