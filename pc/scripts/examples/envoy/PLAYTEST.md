# Envoy rule-host run: what to see, stage by stage (the grid pass, 2026-10-05)

**The readability split (2026-10-05) supersedes the numbers and wording below where they differ** (one or two rules per drive, 72 pieces, 30
keystones with one of seven prices, five statuses and a counter, the HUD of MENUS.md). The old sections stay as history.

Setup: `envoy classic` (console) or START in the Envoy menu (the rule host is the default; `envoy rules off` selects the older companion-stat
route). Vanilla disc, offline. Controller: A / B / X / Y, D-pad or stick. See MENUS.md for the grid.

## The developer overlay (off by default)
`envoy devui on|off` (or the `ENVOY_DEVUI=1` environment flag where the mod can read it) is ONE predicate (`mod_tuning.dev_ui()`). On it shows: the
strength percentage and the depth text on the build strip, the strip's flash text (a modifier's name, "Build ready"), the opponent-strength figure on
the opponent card, the mod's teaching and error panels ("Technique rule fired", "Build update refused"; they are always in the log), the Modifier LAB
text box and the Drive LAB card. Owed on the engine side (not in the mod): the FLY readout, the LAB mod's default mode and a settings key for the switch.

## What to check after the split (needs a game window: not done in the session that wrote it)
1. `envoy start classic <fighter>`: one drive with ONE rule, one keystone with one rule line and one price line. Bar: squares and a keystone letter, no `%`, no `Depth`.
2. `envoy start classic <fighter> depth=6 loop=0 build=7`: five drives with at most two rules each; the reward screen title says "Unlocked: ..." after the first reward.
3. `drive grant rare 11` twice: the second copy of a held rule merges ("Merged!", a one-line note bottom-left) even when its colour differs.
4. A fire build (Kindling applies Burning on any hit, 1 damage a second); Plague Bearer 1.5 a second per stack; no keystone puts a status look on yourself.
5. `envoy devui on`: the figures return; `envoy devui off`: they go.
6. Complete an archetype: a small corner note, not a banner; link flash and tints unchanged. Crits: impact frame and tracer play, no text.
7. Opponent card: top right, keystones and `N drive rules`, clear of the timer. Win with a drive on the floor: "Collect the drives" sits below the timer.

## The grid pass, stage by stage (history)
1. Stage 1 start: ONE panel: "Your starter drive" (one modifier, e.g. `Green Drive: Lingering`) and "Your keystone: <name>" with its effect and drawback. The strip top-left: one filled pip, a `+n%` strength, Depth 0, one small keystone cell.
2. Fight: the opponent plate lists its modifiers one line each (early on, one or two). Hit it past 50%: a drive may drop (70% on a battle stage, never more than one). The match does not end at the last KO while a drive lies on the floor: pick it up (a merge says "Merged!", otherwise it goes in the bag, or a free slot when the bag is full, or a grid asks which drive to give up).
3. Stages 1 and 2 give no reward screen; the third stage (index 2) does: three offered cells, your six equipped cells (two locked), your four bag cells, the keystone you hold. Focus a cell: the panel shows its lines and `Build strength a -> b`; A does the obvious thing (merge, else equip, else bag, else ask).
4. Merging: when an offered drive shares a colour and a modifier with one you hold, the held cell shows a plus and A says Merge: one modifier of the held drive goes up a tier, nothing is added.
5. Depth 5 (stage 6): no banner any more; the next reward screen's title says "Unlocked: fifth slot, keystone allowance 2, drive tier 2". At that clear the keystone row appears next to the offered drives: three cells from three colours, pick one (B twice skips; it stays owed).
6. Bag screen: Z+START in a fight opens the same grid and pauses; B or START closes it without pausing the match. Four bag places; a fifth drive asks which to give up.
7. Bonus stage: three offered drives. Master Hand: two Rare and one Unique. After it: "New Game+ 1", your build carries over, opponents are rolled a step above your actual strength.
Judge: are the cells readable at a glance (colour, border, pips); is the one detail panel enough; do early drops feel tame; is 45 s enough (the engine hold is 2850 units of 1/60 s); is a drive per stage the right amount.

## What to expect on the next run
- You start with one drive (one modifier) and one random keystone; the first drops are single-modifier. Two-modifier drives appear from stage 4, three from stage 7, four only deep (NG+ is always the top band).
- Far fewer drives: at most one per stage on the floor, a reward screen only every third stage, bonus stages and the boss; a four-place bag. Duplicates merge instead of piling up.
- 30 keystones, a new one each five depth, chosen from three; you only ever see the ones you hold and the three on offer.
- Opponents track your actual strength and sit a little above it (about +5% at depth 5, +10% at 10, +13% in NG+1, +39% in NG+3).
- The reward screen is a grid, not a list; the drive models show in the cells when the local model mod is installed, flat coloured cells otherwise.
- Known, not mine: the retail results screen needs START (the first press pauses, the second advances), and bonus-stage targets cannot be enumerated by scripts.

## Loot pacing, keystones, merging and the small bag (pacing lane, joined 2026-10-05; Lua-tested, numbers from its simulations)

### 1. The affix-count curve (PLAYTEST table)

A drive's modifier count is `min(rarity count, depth cap)`, plus one for a White drive from depth 3. Rarity counts are the old
ones (Common 1, Magic 2, Rare 4; Unique is one fixed rule). "Depth" is effective depth: stage depth + 13 per New Game+ loop.

| effective depth | cap | natural rarity weights (common/magic/rare/unique) | what a drop reads like |
|---|---|---|---|
| 0-2 | 1 | 100/0/0/0 | `Green Drive: Heavy`, one effect |
| 3-5 | 2 | 85/15/0/0 | one or two; Magic appears |
| 6-9 | 3 | 82/15/3/0 | up to three; Rare appears |
| 10+ and every NG+ loop | 4 | 80/15/4/1 | up to four; Uniques appear |

A forced rarity (a reward) is not gated by the weights but its count still is (a lucky early Rare is one modifier). Names:
one modifier is `<Colour> Drive: <Label>`; two or more keep the old `Heavy Green Drive of Kindling` form. The starter drive is
one modifier (depth 0). Opponents roll from the same rules: early opponents carry one-modifier drives, no uniques before depth 10
(a LAB build far stronger than the depth falls back to the old unbanded rolls instead of refusing). Held keystones for opponents
follow the allowance and the same exclusion rules.

Power, mean build strength with every slot filled, natural rolls, 120-300 builds each (`envoy_loot_pacing.lua`, `sim_curve.lua`):

| context | slots | before | after | ratio |
|---|---|---|---|---|
| depth 0 | 4 | 1.55 | 1.40 | 0.90 |
| depth 5 | 5 | 1.85 | 1.88 | 1.02 |
| depth 10 | 6 | 2.21 | 2.44 | 1.10 |
| NG+ loop 1 (depth 0) | 6 | 2.22 | 2.44 | 1.10 |
| depth 12, loop 3 | 6 | 9.79 | 9.97 | 1.02 |

Common drives now carry one modifier, so the weights above were tuned (they were 60/28/10/2) to keep late power within about 10%
of before. Knobs: `mod_progression.lua` `P.affix_bands`, `P.rarity_bands`, `P.rarity_affixes`.

### 2. Keystones

About thirty (`keystones.lua`), six drive colours: red damage, green speed, blue defence, yellow air, purple status, white wild.
Every one has a drawback line. Rules:
- **Allowance** = 1 + floor(effective depth / 5), no ceiling (depth 5: 2, depth 10: 3, NG+ loop 1: 3, loop 3: 11). Shown "Keystones n/allowed".
- **Exclusive pairs** are refused by the bag, the engine and the opponent roll: Smasher's Creed / Skybreaker, Pyromancer / Frozen Oath,
  Echo Oath / Echo Weaver, Iron Resolve / Banked Momentum.
- **Stacked drawbacks add and are floored**: damage dealt not below x0.6 overall, run/air speed and jump not below x0.55, damage taken
  not above x1.8, launch taken not above x1.6 (so a pile of keystones cannot build an unplayable fighter).
- **Starting keystone**: one random keystone from the run seed, among those playable from stage one (a keystone that needs a rare trigger,
  Clash King, Echo Weaver, Echo Oath, Last Stand, Desperado, is never the starting one).
- **How more are gained**: at a stage clear where the allowance is above the keystones held, offer a choice of three from three different colours,
  legal with what is held; the same offer on a retry; skipping keeps the allowance owed. Never a hidden list in the bag.
- Frame cost (standalone Lua, two fighters, busy event stream, `bench_keystones.lua`): 0 keystones 0.009 ms/frame, 8 keystones 0.075 ms, 12
  keystones 0.111 ms against the 8.3 ms budget (about 1.3%); no effect was dropped by the per-frame effect limit.

### 3. Fewer drives, merging, a small bag

Quantity, expected drives gained (starter included; the run host's numbers before; a 12-stage Classic run assumed:
battle, battle, team, battle, bonus, battle, giant, battle, metal, battle, bonus, boss; `sim_quantity.lua`, 400 simulated runs, real rolls and the real merge rule):

| by stage | gained before | gained after | held before (equipped+bag) | held after | merges after | full-bag choices after |
|---|---|---|---|---|---|---|
| 3 | 8.5 | 4.4 | 8.5 | 3.9 | 0.5 | 0 |
| 5 | 11.8 | 6.1 | 11.8 | 4.8 | 1.3 | 0 |
| 8 | 18.5 | 9.2 | 17.0 (bag full, 1.5 lost) | 6.6 | 2.6 | 0 |
| 10 | 23.0 | 11.6 | 17.0 (6 lost) | 7.6 | 3.8 | 0.1 |
| 12 | 25.0 | 13.6 | 18.0 | 7.9 | 5.5 | 0.2 |

The proposed numbers (`drive_economy.lua` `E.tuning`, the old ones in `E.before`): at most ONE floor drop per stage (battle, giant, metal 70%,
team one, bonus and boss none); stage rewards only at a bonus stage, the boss and every third stage, a pick of three (first two Magic, the
third Rare once rares roll: `E.reward_rarity`); bag of 4. The run host still owns `H.tuning` (`drop_chance`, `battle_drops_max`, `team_drops_max`,
`offer_count`, `bonus_offer_count`, `bag_capacity`): point them at `E.tuning` (`floor_chance`, `floor_max`, `team_max`, `reward_every`, `offers`,
`bag_capacity`). The bag size is `bag_capacity` in `drive_economy.lua` and must reach `drive_bag`'s `config.capacity` (default 12), the header
`BAG n/12` in `run_screen.lua`, and the hard-coded 12s in `drive_lab.lua` lines 44, 55, 67, 127.

**Merging** (`drive_merge.lua`): a gained drive MATCHES a held one if same colour, neither Unique, they share a modifier or touch a common
budget family (two "damage taken" modifiers), and the held drive has merges left (3 per drive). A merge lifts exactly ONE held modifier one
tier (the exact one if shared), records one merge, and re-bases the held drive to the gained drive's depth if that is deeper. It never adds a
modifier or changes colour, rarity or count, so merging cannot turn an early simple drive into a four-modifier one. `can_merge(held, gained, loot)`,
`merge(held, gained, loot)`, `find_target(list, gained, loot)`. A merged record carries `merged = n` (new record field; `drive_loot` validates
the tier lift against it).

**Gaining a drive** (`E.gain_plan(held, bag_count, gained, loot)`): merge if any held drive matches (equipped first, exact match first), else bag
if there is room, else `choose` (an immediate keep/replace: keep it and pick a drive to give up, or leave it).

### 4. Text economy (`drive_text.lua`)

One short line per drive in lists: `T.short(loot, drive)` = `Red Drive: Heavy, Shatter +1` (max two labels). Full detail only for the focused item
(`drive_lines`, `header`). No ids, tier codes or budget numbers: "Build strength" now reads as a percentage over an empty build (`+56%`), not the
budget's 1.56; a unique's fallback no longer prints `Tier n:`. Keystones read as two lines, effect then `Drawback: ...`.

(The run host now reads `drive_economy.lua` `E.tuning` directly; `bag_capacity` 4 reaches `drive_bag` and `drive_lab`.)

The screens that carry these rules are the grid in MENUS.md; the screen proposal in the pacing lane's notes was adopted.

## Technique, crits and earned looks (skill layer, 2026-10-05; Lua-tested, in-game checks listed below)

The engine's skill events, crits and earned-look calls are wired into the modifier system. Everything here is data records in the existing schema
(no Lua callbacks): a trigger, conditions, effects. `scripts/mod_skill.lua` is the one declaration of the technique vocabulary.

### Triggers (`mod_skill.lua`)
| trigger | verified in the game (engine lane) | retail CPU performs it |
|---|---|---|
| lcancel, lcancel_hit (hit-confirmed), lcancel_miss | yes | dead (miss: maybe) |
| wavedash | yes | dead |
| perfect_shield | yes | maybe |
| tech, tech_miss | yes (directions partly measured) | maybe, live |
| short_hop, fast_fall | yes | maybe |
| dash_dance, jump_cancel_grab | yes | dead |
| combo (count condition), combo_end | yes | live |
| crit (strength condition), armor (absorbed / broke) | yes | live |
| waveland, ledge_dash, sdi, shield_drop, auto_cancel, air_dodge, full_hop, jump_cancel_usmash | FLAGGED: available, not confirmed in play | dead / maybe |

Conditions: `combo_at_least`, `combo_damage_above`, `hit`, `aerial`, `direction`, `strength_above`, `armor_result`, `air_frames_above`, `aerial_hit`.
Opponents roll technique modifiers too. A roll whose trigger the retail AI never performs is inert for now (it costs the opponent budget it cannot
use); opponents are not scripted. Foe rolls weight those records down (see the report).

### Effect kinds (registry, schema, budget families, safety floors)
armor (timed by type; permanent only as a threshold on an equip rule), intangible (<= 24 frames), interrupt (<= 20 frames, direct call, not journalled),
air_jumps and restrict (caps; at most two restrictions, never shield with air dodge), crit (chance, multiplier, per-tag slot, percent floor, status gate),
crit_next (forced crits), chain_status (Conductor's Shock to the nearest other opponent), and the fall-speed and weight value families.

### Where technique modifiers sit in the depth curve
Normal records carry `min_depth` (effective depth): Keen 4, Clean Landing 5, Tech 5, Brutal 6, Ruthless 6, Wave 6, Combo 6, Finish 6, Critical Flow 6,
Finishing 7, Stance 7, Retaliation 8. Below that depth they are never drawn (a redraw on a separate stream keeps the seed's other picks identical).

### Presentation: one meaning per output
surface treatment = a status you have; earned afterimage = a status you earned by technique, coloured by its cause, only while it lasts (blue L-cancel,
teal shield/tech, gold wave, red combo, white movement, violet a state paid for a miss); tracer = this hit crit; impact frame / particle = a moment.
The echo picture is also windowed (the status its rule needs, or 45 frames after your own hit): nothing is ever on continuously.

### Try it
`envoy rules on`, `envoy classic`, then `envoy grant wavedasher` (any keystone id: wavedasher clean_lander powershield_oath juggernaut executioner
critical_mass combo_conduit aerialist phase_dash gambler featherfall conductor). `critfx preview 0.2` / `critfx preview 1` plays the crit moment at a strength;
`critfx slow 4` stretches it for a look; `techprobe` logs statuses with their cause and the native writes; `techprobe cost` the script cost.

# Envoy retail playtest - 2026-10-04 fix1

EM4 follow-up supersedes the tight EM4 LAB ceilings and mirror fallback below. Opponent rolls are
LAB diagnostics; the earlier retail Classic stat/reward flow remains separate.
Use the follow-up acceptance section at the end for current progression and opponents. Earlier EM4 sections are historical acceptance recipes.

Classic is now the intended default; Adventure follows. The historical maze
playtest below is parked. Start Mario on Normal with three stocks, on vanilla,
offline. Source/stub tests do not establish controller or visual acceptance.

Play stage one normally. Read the opponent's leaning-stat tag. Clear it and
choose a reward with the controller; verify the before/after values and that
holding A cannot feed twice. Continue through the shipped bonus/team/giant/metal
stages to Master Hand, choose the larger final reward, and play into NG+1 and
NG+2. Companion stats and age must carry. Retry once via retail continue and
check that the opponent spread stays identical; then decline a continue mid-run
and inspect growth retention and settlement on results, then the asset-free menu.

Judge: three rewards versus end-of-run-only cadence; early Power/Guard feel;
Speed and Jump control changes; tag clarity; team budget scaling; later NG+
challenge; and the ending presentation departure needed to protect retail card
records. Compare Mario jumps before/after yellow, green running after one/three/seven
picks, and blue knockback at the same percent. Each opponent-bearing stage must show
a tag and coloured marker for four seconds. A bonus stage shows its own start tag.
Listen for drive pickup sound, watch the 48-tick growth/level-up animation and the
blue Guard flash on a damaging hit. Test with the optional garden kit absent.

## Parked mission playtest (historical)

# Try raising an Envoy

Enter the garden, visit Companion, then start at the exit. Fight for the colours
you want: red Power, green Speed, blue Guard, yellow Jump. Complete the two mazes,
the crossing and the boss; stop at the interludes to see the growth you kept.
A loss keeps collected drives, but evolution requires a boss win.

At grade C the first20-point drive reaches level1. Level5 needs200 total points,
level10 needs650, and level25 needs3500. Later rooms give25/30/35 points per drive.
Grades E/D/C/B/A/S multiply points by0.5/0.75/1/1.25/1.5/2. Look for a receiving-bar
flash and LEVEL UP, then compare before/after on results.

| Level | Power damage/knockback | Speed run/air | Guard damage taken | Shield capacity | Jump air speed |
| --- | --- | --- | --- | --- | --- |
| 1 | +2% | +4% | -3% | +4% | +1.6% |
| 5 | +5.6% | +11.1% | -8.3% | +11.1% | +4.4% |
| 10 | +7.1% | +14.3% | -10.7% | +14.3% | +5.7% |
| 25 | +8.6% | +17.2% | -12.9% | +17.2% | +6.9% |

These are young-companion bonuses. Early movement and shield changes should be
noticeable; Power stays modest because it affects launch strength too. Total
caps including passives are Power10%, Speed20%, damage reduction15%, shield20%.
Jump currently changes air speed only, up to8%; ground/air jump height is not
registered by fighter_mod. A desired15% height bonus is inactive pending an engine
API. Maze levels must always be solvable at Jump0 (Mario full hop about29 units,
double jump about52); bonuses must never be required.
Pickup radius stays17.5 world units and drop chance80% at every level/depth.

Your first boss win chooses the stat with most points gained this life. Tied
leaders become Balanced. Try:

- Power: Heavy Hands, +2 percentage points damage, within the10% total cap.
- Speed: Tailwind, +3 percentage points movement, within the20% cap.
- Guard: Soft Landing,3 percentage points less incoming damage, within15% cap.
- Jump: Skybound, +2 percentage points air speed, within8% Jump cap.
- Balanced: Sure Footed, +1 percentage point run speed and +2 percentage points
  air speed, within total run20%/air28% caps.

The leading stat's inherited grade rises one step; a mixed hidden allele remains.
A white drive boosts a lowest uncapped grade for this life. On your sixth accepted
start, finish or abandon and return to the garden: the companion becomes an egg.
Its inherited grades, DNA and colours remain; levels reset and10% of points carry
as a separate baseline. The next started run hatches it and a new growth contest
begins. Whites never become permanent through the evolution calculation.

Please judge: can you feel early growth without the game playing itself? Does the
four-room length drag? Are maze climbs and garden stations comfortable with your
fighter? Do the result/evolution/egg moments explain what happened? The existing
pickup controller's sound candidates still need listening; no new art was made.


EM2 native hit-rule acceptance (unrun in the source-only packet):

1. Build through tools/port/build.sh after integration, verify new log strings in
   the actual EXE, and run the registered script_hit_rules fixture.
2. In offline LAB Mario P1 vs stationary P2, use `mod clear`, then `mod add burning`.
   Use up-smash (original Normal), not forward-smash (already Fire): look for the
   ember body treatment and retail Fire contact burst. Original sound remains.
3. Clear and add charged; use a Mario aerial. Look for electric body arcs, retail
   electric contact and increased retail hitlag, with original move sound.
4. Add burning, kindling and pyre. Land a Fire hit to apply Burn, then another
   hit while Burn is active. The second hit must launch25% harder before selection
   of the winning simultaneous hit. Trace should name native Pyre and ancestry
   from Kindling; purple shader treatment must not create a Curse gameplay bit.
5. Clear and add pyromancer. Test normal jab, aerial and owned projectile conversion.
   Hit P1 with an original Ice capsule: damage must double. Sing Sleep, grab Catch
   and other protected original elements must stay unchanged. Confirm actual
   elemental Fire/Electric/Ice/Darkness behavior separately with the API fixture.
6. Exercise fighter scripts, throw descriptors/releases/secondary throws, item
   scripts, Geno articles and Samus grapple. Charge/damage updates and reflected
   projectiles must not compound the creation multiplier.
7. Assign different primary/partner tables and masks using the optional sub
   selector. Test Popo/Nana independently plus all six slots and CPUs.
8. Snapshot active rules, status masks and in-flight metadata; run actual LAB
   rewind_test and require differing_bytes=0. Isolated helper tests are not this
   acceptance. Test clear, owner unload, scene change, KO/respawn and expiry.
   Existing capsule creation values stay latched; contact rules clear immediately.
9. `mod clear` must retire native rules, statuses and shader/post treatments;
   campaign/Classic/Adventure activation and netplay writes remain refused.

The hit-rules catalogue scenario only checks the real table toggle; collision,
particles, sound and timing require the operator steps above.


EM3 drive loot acceptance (2026-10-04; source tests only):

1. Mount the refreshed Envoy folder on an EM2-capable build. Restart the game
   to register new DriveLoot FX declarations. This packet adds no native C.
2. Offline LAB: Mario P1 and a standing level0 P2, Final Destination. Run
   `mod clear`, `drive give rare 45`, `drive give unique 6`, then `bag`.
   A selects a drive; A on a slot equips/swaps. A on an occupied slot without
   selection unequips. Discard and keystone are explicit rows. Z+START also
   opens the bag; B closes. Left/Right pages details. All text must be readable
   at853x480 and smaller640x360, including long names/rules and equip deltas.
3. Resume simulation and use `drive drop magic 123`, `drive drop rare 456`,
   `drive drop unique 789`. Walk P1 over each. Check native hover/spin, core
   glow, rarity beam/sparkle, pickup sound and short generated-name card.
   Duplicate callbacks cannot award twice. Ground drops reserve bag capacity;
   full bag must refuse gracefully, including pending paused inventory edits.
   Collect all ground drops before editing inventory; pending edits must finish
   before another drop. Paused drops refuse until simulation resumes.
4. Test red damage, green run/air speed, blue launch resistance, yellow jump,
   purple applied-status duration, and white extra affix/no implicit. Tier
   depth bands are1 at0..4,2 at5..9,3 at10+. Keystones never roll as loot.
5. Equip Glass Core and other steady effects; lose a stock. The same build
   must remain, and damage modifiers/native rules/looks return on respawn.
   Burn, Momentum, Haste, Guarded, Curse, recent events and transient values
   must clear. Repeat with twelve bag drives plus four equipped drives.
6. Equip/unequip/swap/discard while paused repeatedly; previews must track
   the draft, refused edits must leave everything intact, and closing commits
   once warm. More than twelve queued edits must refuse without disabling Lua.
7. Snapshot a full bag/equipment/keystone and an active physical drop after
   the first post-drop frame checkpoint. Require live rewind_test
   differing_bytes=0 and same inventory/rules/statuses after seek. Manual
   drops branch history and record their map at the next checkpoint; avoid
   saving in the interval before that checkpoint. Isolated roundtrip evidence
   is not acceptance of that manual boundary or real LAB rewind.
8. Clear, unload/reload and change scene. Check native rules/overlays/shaders
   and physical drops clean up; default bag starts fresh. Change the single
   drive_lab.tuning.persist switch to test retained bag/build across scenes.
   Classic/Adventure and netplay must refuse these LAB diagnostics.

Five two-modifier scripts (each begins with mod clear):
- `mod add updraft`; `mod add crosswind`: aerial hit then land, Momentum -> Haste.
- `mod add ledge`; `mod add rush`: grab ledge then hit, Haste -> Momentum.
- `mod add shelter`; `mod add renewal`: grab ledge, Guarded -> heal2.
- `mod add icebound`; `mod add brittle`: an Ice hit then another hit, Chill -> Curse.
- `mod add kindling`; `mod add malice`: Fire hit then another hit, Burn -> Curse.

Two three-modifier scripts:
- Updraft + Crosswind + Still Heart: aerial hit then land, Momentum -> Haste -> Guarded.
- Ledge + Rush + Bastion: ledge grab, hit, land, Haste -> Momentum -> Guarded.

Use Mario forward smash for Fire in Kindling's script; use Ice Climbers' native
Ice attack for the Icebound/Brittle pair.
The generated146-pair TSV describes vocabulary overlaps and filtered directional
status/event links. It does not establish146 tested causal combinations;
classification-tag overlap alone, including two keystones, is not a legal
single-player chain. The seven scripts above are separately tested causal chains.

EM4 balance and opponent acceptance (2026-10-04; native rebuild required):

1. Integrator: rebuild through `tools/port/build.sh`, complete the bridge ABI
   audit, then mount the refreshed Envoy folder. The older EXE must refuse LAB
   edits with a rebuild message. Native hit rules now expose percent-only damage
   and final launch fields; old native snapshots/journals cannot cross this
   binary change. Run the registered hit-rule tests before owner play.
2. Offline LAB, Mario P1 and standing CPU P2 on Final Destination: `mod clear`;
   `drive give rare 45`; `drive give rare 8`; `drive give rare 76`;
   `drive give unique 6`; `bag`. Equip all four slots, close with B and resume
   until the pending edits commit. This is a real depth-1 seeded build: Pyre /
   Burning / Crosswind / Bastion / Updraft; Lingering / Kindling / Malice;
   Heavy / Rush; Glass Core. Duplicate IDs contribute once. Check every family
   total/cap and equip preview at 640x360 and 853x480 with controller paging.
3. Glass Core adds 60% percent damage dealt and taken, with current launch
   unchanged. Fire smashes start Burn, later hits activate Pyre and Malice's
   small Curse. Aerial hits then landing retain Momentum/Haste/Guarded chains.
   Percent should climb visibly; launch should follow the stated launch rules
   rather than suddenly doubling with the damage bonus. Glass remains risky.
4. `foe roll` matches P2 to the current player strength (about 2.06 for the
   recipe above). Resume for warmup/application. Read its timed nameplate,
   shader treatments and `foe list`. A difficult target can use a logged exact
   player-build fallback; it must not silently pretend that was a different
   random roll. `foe fight 2` makes the existing CPU fight without losing its
   build; `foe stand 2` returns it to standing, preserving the native CPU level.
5. For an explicit deterministic comparison: `foe roll 1.4 42 2`; resume;
   inspect `foe list`; clear/repeat the same seed/stage/port. It must reproduce.
   Change seed or stage to explore different modifiers. Uniques and keystones
   are legal. Impossible high explicit strengths should refuse visibly.
6. Let the opponent hit P1. Its conversions, statuses, healing/other chains
   and native damage/launch rules must affect P1 through the same engine. Lose
   a stock on each side: equipped builds persist; transient statuses clear.
   `foe clear` must remove its builds/nameplates/native rules and descended
   statuses while retaining P1 bag and equipment. Scene/unload must clean up.
7. Check native semantics with matched attacks at the same starting percent:
   extra percent must not change the current launch inputs, shield damage,
   clank priority, staling or hitlag inputs. Test charged/staled attacks, an
   owned projectile, throws, phantom hits and a zero-damage detector. Test two
   simultaneous victims with different statuses; no shared-hit mutation.
8. Complete a checkpoint with twelve bag drives, four equipped slots, multiple
   modded CPUs and nameplate timers. Require live LAB `rewind_test`
   differing_bytes=0 through hits, stock loss, stand/fight and clear. Source
   codec/helper roundtrips are not this acceptance. For manual physical drops,
   wait for the completed following checkpoint before saving.

Numeric budget witness: three valid depth-10 Rare records plus Glass Core
produce Heavy +7.5%, Pyre +12% and target Curse +3.75%: final launch 1.2398125.
The real-weight retail-formula FD-centre no-DI launch-distance proxy passes
Mario 99% -> 70% and Peach 93% -> 66%, about 29% earlier. It excludes gravity,
drift, DI, animation geometry, stale moves and hitstun exits; it is a tuning
test, not a measured in-game KO. The four-drive test's fixed labels 401..404
are fixture identities, not commands that reproduce those exact generated
records. Live KO feel, controller layout and rewind remain owner acceptance.

Queued CPU modifier edits and foe rolls must commit separately: resume and wait
for one edit before issuing the other. During the four-second foe nameplate,
the debug HUD is temporarily hidden; it returns when the nameplate expires.


## EM4 follow-up1: escalating depth and independent opponents

## Owner play script (after rebuilding)

Use Mario P1 and Mario P2 on Final Destination, enter LAB and allow queued changes to commit by resuming a logic frame. These commands assume a clean bag. Give commands and subsequent bag equipment must commit separately (the pending edit limit is12). Open the drive bag after the gives commit, equip the listed records in order, and select the named keystones. Close it and resume before rolling the CPU.

Early:
```text
foe clear
mod clear
depth 0 0
drive give rare 6756413
drive give rare 1617784304
drive give unique 370289401
drive give rare 1958367041
```
Resume; open bag, equip all four and choose Pyromancer; close/resume.
```text
foe roll - 2143609173 2
foe fight 2
```
Strength1.466 against1.449; model8/7 attacks. Observe percent versus launch, names and visible family totals.

Then clear the CPU and player build and commit the clears. Set the late dial and give:
```text
depth 12 3
drive give rare 218490
drive give rare 1767098125
drive give rare 1988514207
drive give rare 1313358071
drive give unique 1784243822
drive give rare 2116403131
```
Resume; open bag, equip all six; choose Frozen Oath, Bulwark and Sprinter (Pyromancer excludes Frozen Oath; Still Heart is retired); close/resume.
```text
foe roll - 2144865533 2
foe fight 2
```
The late records are tier11, include Glass Core and unlock all three keys. The opponent's target is your strength times the difficulty factor (opponent edge .010 per effective depth, capped at +25%; the old plan's .003 is gone), so the figures in this section are order-of-magnitude, not pinned (the 2026-10-03 pair was 226.890 versus 257.161; model4/4 attacks). A console `foe roll` is one script call, so it tries at most `sync_attempts` (32) candidates and, if the band is missed, settles for the best one with a `foe roll: fell back` log line. For a boss/final boss, clear/commit before `foe roll - 2144865533 2 boss` or `foe roll - 2144865533 2 finalboss`. Expected strengths287.690/348.456 and modeled4/3 or3/3 attacks. Recreate a fresh stock between directions; statuses, healing, spacing and ledge triggers change actual results.

FD seeds compensate the model fixtures' stage0/12 against gd.match().stage's internal FD kind37, preserving the exact independent rolled build. Verified by codec equality, not assumed from the external stage ID. Model/player fixtures use sample401 early and sample12 late.

## EM5 echoes owner acceptance

Rebuild through the normal port pipeline after integrating the afterimage-lane
renderer patch described in the EM5 report. Mount the Echoes catalogue demo by
itself, choose Mario P1 and Mario CPU P2 on Final Destination in offline LAB.
Allow several frames for history/pictures to fill. P2 stands automatically.
Jump and neutral-air past P2: three pictures are spaced4 frames apart, with
copy2's aerial capsule replaying at its recorded world position at age8.
E restarts this setup. N switches to the owner's example: three pictures with
only neutral-air on copies1/2, ages4/8. Verify copy3 stays unarmed. Direct hit and
echo are separate hits with their own target memory; each echo consumes a target
once per move by default. Owner hitlag/rebound is suppressed; target reactions,
shields, clanks, team rules, stale history and KO credit use retail paths.

Check an aerial that hits live and delayed; an aerial that misses live but whose
old capsule catches P2; shield and clank interactions; P1 immunity and team attack
off/on; Ice Climbers partners; a thrown-body capsule credited to the thrower;
stock loss/respawn; scene change and unload. Echoes repeat recorded fighter
capsules, not an article/projectile's hitboxes or a new animation on old bones.

In Envoy, stop the director, use `mod clear`, then:
```text
echo add 8 nair .4
echo add 16 nair .4
```
Resume a logic frame to publish rules. These manual commands are collision-only;
use the demo for the three-picture example. `echo clear` retires manual rules.
Use `mod add echoes` to test the tier1 aerial suffix; use loot and depth for
higher tiers and the shared family totals. Commit edits separately if capacity
would overflow; an invalid combined edit refuses without replacing the build.

Start LAB history, perform repeated aerials, save/query `gd.fighter_history`,
seek backward and resume. Require actual `gd.rewind_test` to report0 differing
bytes for the rebuilt EXE; compare delayed contacts after replay. The standalone
source fixture's zero-byte comparison is separate evidence. Presentation caches
refill after rewind and may temporarily lack old poses. Check visible armed
element tint/contact flash only after the deferred renderer hook is integrated.
