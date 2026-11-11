# Envoy V3 rules: Heat (port/v3sim, built 2026-11-10 overnight)

The rules slice of `envoy-synergy-v3.md` section 12 (branch `port/synergy3`, commit 4e5338a4f), built on `port/v2`
(22f4057d4). The UI for it is a separate branch (`port/v3ui`); this branch keeps V2's look and adds the smallest
text HUD so it can be played on its own. The view contract is `envoy-v3-view.md` (this branch). Decisions taken
here, in the owner's pattern, are marked **[decided]**.

## What was built

| piece | where | state |
|---|---|---|
| Heat | `envoy_data/src/heat.rs` (the integer rules, shared with the tests), `envoy_sim` | built: per player, +1 per 1% dealt by damage records (never by hit count), -1 per 1% taken, no decay, 0 at each fight, marks 40 / 90 / 165, cap 225, the 150% overkill half on the sim's own percent, fires x1 / x1.25 / x1.5 / x2 by state, a lost stock halves it |
| the cash-out | `envoy_sim` `foe_ko`, `cash_out` | built: every player with 10%+ on that foe's stock, player 1 first: coins into the one purse (and the player's earned share), pips, the flare (Heat / 15 percent to every other foe, max 15, no launch, feeds nothing), Wildfire, then half kept; pay x0.5 minion / x1 duel / x1.5 elite / x1 boss stock / x2 the boss's last |
| damage charge | `envoy_sim` `deal`, `take` | built: every trigger's need in percent (x1e9 units per 1%), remainders carried; time, tag-in, shift and cash-out triggers |
| chains | `envoy_sim` `chain_frame` | built: leftmost full trigger starts; a fire's damage charges only the triggers to its right by its coefficient (x parent, rounded once per link); 4 links, 6 frames apart, 20 frames between chains; the fire log carries link number, parent index and the Heat state |
| ordered slots | `open_slots` | built: 3, 4 after boss 1, 5 after boss 2, in order; full rows replace a chosen drive (pick screen step 2, shop step 1) |
| Echo | the neighbour drive | built: the drive to its right fires again 6 frames later at x0.5 / x0.8 / x1.2 (not a link, never launches); Echo Oath (keystone) kept |
| +Heat / xGain / shaper | `heat_add_drives`, `gain_now`, `take` | built: Clean Landing (L-cancel after an aerial that dealt damage), Riposte (tech), Metronome (a move not among the last three, once per move); Tailwind (Haste), Perfect Form (three L-cancelled aerial landings), Gatekeeper (offstage), Glass Core x2; capped x4; Insulate's allowance |
| technique | engine `ENVOY_LCANCEL` hook, `tech_frame` | built: the L-cancel bit (`gd_envoy_events`), wavedash (LandingFallSpecial within 3 frames of EscapeAir), tech (Passive states), offstage damage (a ledge table for the five stages; any other stage never counts as offstage: the engine exposes no stage-extent read to mods, only the stage kind); each writes `EV_SPARK` |
| Volley | D-pad right, 3 pips | built: given at the run's start full; every trigger of the fighter in play fires left to right as one chain with no link limit |
| Inferno | three different RED drives on one fighter | built: Burning stacks to 6, its ticks feed Heat at 0.5; and (loud) every RED fire adds a Burning stack and Ember splashes |
| Grand Finale | `gd_envoy_kb_scale` | built: a smash attack's knockback x(1 + Heat/450), to x1.5; a smash KO pays x1.5 and keeps no Heat |
| the boss | `gd_envoy_boss_dmg` | built: Petey's damage per slot (x1000) and HP; each quarter of HP is a break (a cash-out); the stand-in boss (Ganondorf) cashes out per stock |
| coins | `cash_out`, `fight_won`, `on_new_match` | built: cash-outs, +20 a fight cleared (+10 with no stock lost), +10 an edgeguard KO (offstage damage within 2 s of the KO), interest +1 per 10 held to +5 |
| the shop | `shop_frame` | built: after fights 1, 3 and 4 (a screen after the picks): three drives (Common 30 / Rare 80), level a drive (60 / 120), +1 stock (100), reroll (10, +10 each in a visit), reorder (free), leave; a buy onto a full row picks the slot it replaces |
| the view | `envoy_data/src/view.rs`, `envoy-v3-view.md` | 768 words, "ENV3", record v5 |
| the old HUD | `envoy_ui` | minimal: a Heat bar with its marks, the state and its fire scale, the gain multiplier, coins, Volley pips; the full-row step; the shop as a text list |
| tests | `tools/rust-port/envoy/v3_slice.sh`, `v2_drives.sh` | the lint and split test, sim.py, nine V3 synctests, V2's six on the new rules |

## Content (the pool)

26 drives (the slice's 16 plus V2's other ten re-derived on damage) and 7 keystones (the slice's three plus V2's other
four). **[decided]** V2's ten stay because the owner asked for "big game-changing effects" and more variety; none of
them counts hits any more (the lint rejects a count source): Spire and Comet are timed launches, Bulwark timed,
Slipstream / Relay tag-in, Surge a shift, Thorns every 25% taken, Leech every 60% dealt, Hex every 50% dealt,
Updraft every 35% of aerial damage. V2's keystones keep their rules on the new charge (Overclock doubles the charge
rate; Winter Oath Chills on any damage; Twin Soul's bench row is live; Echo Oath repeats every fire at half).

### Numbers changed from the spec (12.6), and why

The owner's standing wish is loud, visible power; the spec's numbers were first guesses. Every change keeps the 4.3
bound (a fire returns at most 0.1 of its need at level 1 Cold, 0.5 at level 3 Blazing), which the lint checks.

| drive | spec | built | why |
|---|---|---|---|
| Ember | 4%, coef 0.5 | **6%**, 0.5 | a puff at Cold in play; 6% reads as a hit |
| Bolt | 5%, coef 0.6 | **6%, coef 0.4** | louder; coefficient lowered to keep 4.3 |
| Nova | 8% | **10%** | the timed launcher is the chain starter; it should launch visibly |
| Wildfire | 10% | **12%** | a cash-out moment |
| Rush | every 50%, 4 s | **every 40%, 5 s** | Heat drives were too quiet in sim.py (heat spread 1.2) |
| Tailwind / Perfect Form | x1.5 | **x1.75** | the same |
| Clean Landing | +6 | **+10** | the same |
| Riposte | +10 | **+15** | the same |
| Metronome | +5 | **+6** | the same |
| Feather Storm | 2% x 3, coef 0.5 | **3% x 3, coef 0.3**; 4 feathers when Hot, 5 when Blazing | louder; coefficient for 4.3 |
| Frost | 2% | **3%**, coef 0.3 | the status fire carries a little damage |
| Shatter | 5%, coef 0.4 | **7%, coef 0.25** | louder; coefficient for 4.3 |
| Burning | 1% a second a stack | **1.5%** | Burning was the quietest word |
| fire caps | 30% / 40% | unchanged | |

Rule texts were shortened where the lint's 12 words / 2 numbers rule rejected the spec's (Feather Storm, Insulate,
Echo, Metronome, Frost: the Chilled length moved to the glossary).

### Hot and Blazing look different (the owner's wish) **[decided]**

The state already scales every fire's damage x1.25 / x1.5 / x2. On top, deterministically (no chance, no hit count):
launching blasts (Nova, Comet, Spire) get +20% radius and +6 knockback growth / base per state; Bolt chains to a
second foe at Hot and a third at Blazing; Feather Storm throws 4 feathers at Hot and 5 at Blazing; Ember splashes at
Blazing (as at level 3); Shatter's shards hit every Chilled foe at Blazing; Frost's Chill lasts a second longer per
state. The fire log carries the state (bits 16-17) and the dealt damage (the level word >> 4), so the UI can size
every effect.

## Determinism

Two `#[repr(C)]` structs, one of i32 words and one of i64 words (Heat, charge, the purse, the pips), both hashed into
`gd_hash_word` every frame with the plan and the pool digest; the Heat constants are folded into `POOL_DIGEST`. The
engine's f32 damage is rounded once at the hook (`(d * 1000 + 0.5) as i32`). Records are buffered in the hook and
processed at frame end in buffer order (12.4). The split test (`heat.rs`) proves one record of N and 2 / 7 / 30
records adding to N give bit-identical Heat, charge, tally, coins and pips, across the 150% line, for damage taken
with and without Insulate, and through a cash-out.

## Endless loops (NG+, 2026-11-11)

Clearing the boss no longer ends the run. After the clear: the drive pick, the keystone pick if one is due, and the
road's "Run cleared! Loop N begins" moment; the run goes on at fight 1 of the next loop with drives, levels,
keystones, coins and slots kept, and one more stock in the pool (to 8). A run ends only when the pool is empty or
the player abandons it at the camp; there is no cap on loops (the scaling stops counting at loop 50).

Everything scales from one integer loop number in the sim state (`St::lp`, in the snapshot and the record). With
e = loop - 1 (loop 1: all terms 0, and the seeded rolls are loop 1's own, so loop 1 plays as before):

| what | per loop |
|---|---|
| foe damage dealt (attack ratio) | +20% (x1000 + 200e) |
| foe damage taken (defence ratio; stands in for weight / knockback resistance) | / (1 + 0.15e) |
| CPU level | 9 from loop 2 |
| elite / boss stocks | +1 every second loop |
| pack fight, solo | +1 foe from loop 2 (3 CPUs) |
| ordinary fights | loop 2: the first foe carries 1 drive; loop 3 on: the first foe is an elite (size 1.15, the elite damage terms) with 2 drives, the others carry 1 |
| elite / boss drives | +1 per loop (to 2) |
| foe drive level | +1 per loop (to the level cap) |
| Petey's cage HP (and so the HP between his four breaks) | +25% (plan word 30, `P_BOSS_HP`) |
| shop prices: drives, level-ups, rerolls, a stock | +25% |
| drive level cap (players and foes) | 3 + e, to 9; strength 1000 / 1600 / 2400, then +1000, +1200, +1400 ... (`level_scale`) |

Heat's cap and the fire caps are unchanged (no instant kills). Slots stay at 5. The shop restocks every loop (the
same fights open it). Shown: the fight HUD and road ("Loop 3 - Fight 2 of 5"), the saved-run sticker and Continue
card, the abandon dialog, RUN OVER ("Reached loop 4, fight 3"). The best loop a save reached is kept by the host in
`best.bin` beside `run.bin` and shown at the camp ("Best: loop 4"). The camp's NG+ row stays hidden.

Test knob: the `MELEE_ENVOY_TEST_DRIVES` item `@N` starts a new run in loop N (with both bosses' slots and keystones
counted).

## Engine hooks (all inert unless a run is armed)

| hook | where |
|---|---|
| L-cancel event bit | `melee_fighter::hooks::ENVOY_LCANCEL`, called in `ftCo_LandingAir.rs` inside the `x67F < xE4` branch; `melee_modes::tu::envoy::{EVENTS, on_lcancel, events}` (writes only while `armed()`), installed by `match_begin`; import `gd_envoy_events` |
| boss damage | `petey::DMG` (x1000 per slot, bumped where `HITS` is, from the same `dmg`); import `gd_envoy_boss_dmg` (4 damage words, HP x1000, full HP x1000) |
| knockback write | `envoy::kb_scale` (`dmg.kb_applied *= m`, m clamped 1.0-1.5); import `gd_envoy_kb_scale` |
| host constants | `VIEW_WORDS` 768, `REC_AT` 704, `REC_VERSION` 5 |

### Engine facts: verified / unverified

| fact (12.5) | status |
|---|---|
| the L-cancel test is `x67F < xE4` at `ftCo_LandingAir.rs` in `EnterWithLag` | read; the hook sits in that branch. Only that branch was found. seen firing on a scripted pad (tech_pad.txt: 90 L-cancels and 45 wavedashes at depth 4, +Heat paid) |
| a write to `kb_applied` in the take-damage post hook takes effect | verified (A/B, v3_slice.sh 7): at one Heat (+30%) the first hit's launch speed with the write on / off is 1.30 (speed^2 35217 vs 20838), depths 4 and 7 |
| `dmg` at the Petey hit test is what the cages lose | read only; the HP words are `petey::view()`'s |
| the ledge x of the five stages | [K] from knowledge of the stages (BF 68.4, YS 56, DL 77.3, FoD 63.4, FD 85.6), not read from stage data |
| `foe_m` tracks the percent the player sees | approximately: it sums the hook's damage, Marked's extra, fires, flares and Burning; heals on foes are not tracked |

## What was cut or simplified

- **The full three-act run** behind the camp's Run length setting: not built (the Short run's five fights only).
- **Reorder on the pick screen** while placing a drive: the pick only chooses which drive a new one replaces; free
  reordering is in the shop (5.1 says the rest stop; the shop is the rest stop here).
- **The map screen**: none (V2 has none); fights follow each other as in V2.
- **Keystone pick order**: the spec's since 2026-11-11: after a boss the drive pick comes first, then the keystone
  pick, then the shop (a run started with fewer keystones than due gets the keystone pick at fight 1).
- **The taunt coin bonus** and the pact: not built.
- **Volley "the fighter actionable"**: Volley fires on D-pad right whenever the player is alive with 3 pips.
- **Player 2's pips and earned coins** in the record: kept (packed into words 44-45), unlike the spec's plan.
- **A Marked foe's extra damage** is word damage (x0.2) for the attacker, as 4.1 says; Marked has two givers (Hex,
  Thorns) and no reader in the pool.
- Foe drives charge by the damage the foe deals (dealt) or by time; foe fires are x1 (no foe meter).

## How to run

- Build: the mods (`tools/rust-port/modhost/build_mods.sh`, or envoy_sim + envoy_ui) and `cargo build -p melee
  --release`; one cargo build machine-wide (`E:/Projects/Melee Workspace/scratch/.cargo-build.lock`; the test
  scripts take it around their cargo calls).
- Checks: `bash tools/rust-port/envoy/v3_slice.sh` (lint + split test, sim.py, nine synctests at depths 4 and 7),
  `bash tools/rust-port/envoy/v2_drives.sh` (V2's six on the new rules), `python tools/rust-port/envoy/sim.py --v3`.
- Test knobs: `MELEE_ENVOY_TEST_DRIVES` takes `$N` (coins), `noshop` (no shop), `^N` (every fight starts at the Heat
  that gives Grand Finale +N/1000), `finaleall` (every player hit counts as a smash for Grand Finale), `finaleoff`
  (the write logged, not made), `aslog` (player 1's action states and hits logged) besides drives and keystones; a
  row of more than the open slots opens them for that run.
- Play: `E:\Projects\Melee Workspace\scratch\envoy-demo\play-v3sim-solo.cmd` (old look, new rules).

## Results (2026-11-10, commit on port/v3sim)

| check | result |
|---|---|
| gate (`gate.sh --skip-build`) | GATE OK |
| replays vs V2 baseline | hero 392/428, local 125/125, own 917/917 (all = baseline) |
| envoy_data lint + tests (incl. the split test, both directions, Insulate, through a cash-out) | 7 passed |
| V3 synctests (v3_slice.sh), depths 4 and 7 | all nine 0 mismatches. Seen: chains to link 3, Echo repeats, cash-outs (5-16 a run) with flares, states up and down, Volley, the shop (buy onto a full row, level, reroll, reorder, full-row pick replace), co-op both firing. Not seen in CPU play (informational): a technique mark (the CPUs did not L-cancel, wavedash or tech in 3600 frames), Grand Finale's knockback write, a Petey break inside 1846 frames |
| V2's six (v2_drives.sh) on the new rules, depths 4 and 7 | all 0 mismatches |
| camp_run.sh (no MELEE_ENVOY_ARM, the camp door) | CAMP RUN OK: fights 1, 2, 3 rewritten and won, a shop after fight 1, keystones. A longer run (110000 frames) stops in fight 3's results only because camp_pad.txt runs out of presses; fights 4-5 were not reached headless |
| sim.py --v3 (120 seeds, model) | pace met (reference 90 at the first KO, never Blazing by the 4th; two-for-one 75 = Warm; even trade 30), split identical, use 16/16, character spread 1.07 (<= 1.25); build spread 2.1-2.9 (band 2.5-6: aerial and tagger just under), heat spread 1.1-1.3 (band 1.5-2.5: missed; Heat drives are a small share once halving and the cap shape the meter), KO delay +16% (band < 15%). The bands are report-only in the scripts (`--no-fail`) until the owner's play sets the dial |

### Follow-up round (2026-11-11)

| check | result |
|---|---|
| full Short run from the camp (camp_run.sh, 216000 frames, CPU player) | fights 1-5 rewritten in order, picks, shops (fights 2, 4, 5), keystones after bosses 1-2; the CPU lost fight 5 (RUN OVER, then the camp over the CSS, disarmed). Loss also seen at fight 4 |
| the win (camp run from fight 5, strong drives) | the boss fight cleared, RUN CLEAR, pool 3 left, disarmed |
| saved run | **was broken in V3** (the host checked the V2 view words); fixed. Run 1 saved after fight 1; run 2's camp Continue: "armed, run continued at fight 2" |
| synctests C1-C8, depths 4 and 7 | all 0 mismatches; tech 90/144 L-cancels, 45/72 wavedashes; tech marks seen in a camp run's fight, not on the pad (Falcon is never knocked down there) |
| Grand Finale A/B | 1.30 launch speed ratio at +30% (required 1.25-1.35) |
| Petey stall (round 3) | a rollback-only bug: draw_model grew vertex buffers inside the game scope; with the Brawl files present a snapshot load rewrote the arena under them (ARENA MISUSE, 196 mismatches, a stalled synctest). Buffers now made once on a helper thread. Offline play was never affected |
| Petey breaks (round 3) | **were never counted, in play too**: the carrier was looked up before Petey's fight began (boss_slot -1). Fixed; with `*6000` breaks 1-4 fire, cash out, RUN CLEAR; slice C9 requires them |
| full win from fight 1 (round 3) | camp_run with `ember:3,nova:3,bolt:3,*4000\|grand_finale`: fights 1-5 won, RUN CLEAR. One earlier identical run crashed once in fight 2 (access violation in HSD_PObjLoadDesc / geno_pal_desc strcmp on a bad class_name, a JObj load mid-match); not reproduced |

### Heat spread: proposals (not applied; the owner's rules stay: half kept on a KO, no decay, 1 Heat per 1%)

sim.py --v3 puts every build at 1.1-1.3x the reference's Heat (band 1.5-2.5). Damage is the same for every build, so
damage-driven Heat cannot spread builds; only the build-specific terms can. Options, smallest first:
1. **State-keyed +Heat**: Heat drives pay more in the state they are built for (e.g. Clean Landing x2 while Warm or
   below, Gatekeeper x2 at Hot+), so a Heat build climbs out of the sawtooth sooner.
2. **Bigger gain multipliers on Heat drives** (xGain 1.5 -> 2, cap x4 kept): moves only builds that take them.
3. **A drive that keeps more on a KO** (e.g. a keystone or level-3 effect: keep 70% instead of 50%) - a build choice,
   not a rule change for everyone.
4. **State-keyed drives** (fire or charge only at a Heat state) make the Heat a build cares about visible as a spread
   in outcomes even when the meters look alike.
Each can be tried in sim_v3.py first; none needs an engine change.

### Known gaps for the playtest

- Heat spread is structurally low: every build sits near the same sawtooth (half kept at each KO). If builds feel
  alike in Heat, the levers are the gain multipliers and +Heat sizes, or a smaller keep (e.g. keep 60% with Heat
  drives).
- After the run ends the camp's Done leaves the player on the VS CSS (envoy_ui's flow); the main menu may suit better.
