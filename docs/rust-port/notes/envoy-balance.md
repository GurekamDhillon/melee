# Envoy balance pass 1: no more one-shots (port/envoy-balance1, 2026-11-13)

The owner reported that players "get one shot because knockback scaling is too high", from Petey (a separate lane:
`petey.rs` is not touched here) and from the drive CPUs. A laptop soak also lost the elite room 3 times out of 3 with
a strong build, its stocks drained by wave after wave. This pass changes only `rust/mods/envoy_sim`, so the engine and
vanilla play stay bit-identical.

## Where the extra damage and knockback came from

| source | where | before |
|---|---|---|
| foe attack ratio (the engine's `attack_ratio`: damage, and so knockback) | `make_plan` p[17] | elite / boss **size x loop**, multiplied: elite 1.2, boss 1.25, loop terms `foe_atk` 1.05 / 1.15 / 1.25 / 1.35 / +0.15 a loop, so loop 5's boss was x1.69 |
| foe drive level | `foe_roll` | elite / boss level 2, **+1 per loop up to the player cap (to 9)**; ordinary fights' first foe from loop 2 |
| drive level scale | `codes::level_scale` | 1.0 / 1.6 / 2.4, then superlinear: level 4 3.4, 5 4.6, 6 6.0 ... 9 11.4 |
| foe blast damage (Nova 10%, Comet 7%: real hitboxes with knockback 70/35 and 55/30) | `fire` -> `launch` | **uncapped**: `LAUNCH_CAP` was applied only on the percent fallback, never on the blast itself. The engine then multiplies the blast by the foe's attack ratio |
| foe percent fires (Ember, Bolt, Frost, Hex) | `hurt` (FX_FOE) | capped at 30% each (the player cap) |
| player's own keystone prices | Glass Core x1.6 taken, Fragile x1.25 | unchanged (a choice the player makes) |
| CPU level | plan p[11] | loop 1: 3 / 5-6 / 6+8 / 9; loop 2: +3; loop 3 on: 9 (unchanged) |
| waves | `room_waves` | 2 + loop - 1, to 4; percent carries between waves |
| elite / boss stocks | plan | +1 every second loop (unchanged) |
| stages | `STAGES` | BF, Yoshi's Story, Dream Land, FoD, FD: retail stages with their retail blast zones (the coordinator's Petey finding, that one-shots came from close blast zones, does not apply here) |

What stacked was the attack ratio (size x loop) on top of a foe drive whose level, and so its blast damage, grew by
the superlinear level scale with no cap. A loop-3 elite's Nova landed 10% x 3.4 x 1.38 = 47%, with knockback 70/35. It
KOs a mid-weight at **about 3%**.

## The rules (four, in `envoy_sim/src/lib.rs`, "balance pass 1")

1. **Foe bonuses add, never multiply, and are capped.** attack = 1 + half the size bonus (elite 0.2 -> +0.1, boss
   0.25 -> +0.125) + the loop bonus, to at most **x1.5** (`FOE_ATK_CAP`). The size already gives more reach.
2. **Foe drives fire at most at level-3 strength, and one foe fire lands at most 12%** (`FOE_FIRE_CAP`), blasts
   included. Foe drive *levels* still rise per loop (the HUD shows them). The challenge now comes from the number of
   drives, CPU level, stocks and waves, not from bigger numbers.
3. **A room has at most 3 waves** (`WAVES_CAP`, was 4).
4. **Clearing a wave heals every player 25%** (`WAVE_HEAL`, never below 0%). This makes the elite room survivable.

The challenge still rises every loop: the attack ratio (+5% / +10% / +10% / +10% / +15% ... to the cap), foe
toughness (unchanged, uncapped), CPU level 9, +1 elite / boss stock every second loop, +1 pack foe, more foe drives,
a third wave from loop 2, and Petey's HP.

Test knobs (`MELEE_ENVOY_TEST_DRIVES`): `oldscale` restores the old numbers for an A/B, and `kblog` logs every hit
a player takes from a foe (damage, percent, launch speed^2 on the next frame) and the percent of every stock lost
(its exact lines: round 3 below). Both are logs or test-only. Without the knobs, nothing that is hashed changes
besides the new rules.

## Before / after: model (Melee's formula, Mario weight 100, centre of FD, KO knockback 167)

A typical strong CPU finisher (a 16% smash, 90/30) and the elite's Nova:

| fight | atk before | atk after | smash KO% before / after | foe Nova dmg before / after | Nova KO% before / after |
|---|---|---|---|---|---|
| vanilla level-9 CPU | 1.00 | 1.00 | 91 | - | - |
| fight 1 duel, fight 2 pack | 1.00 | 1.00 | 91 / 91 | - | - |
| fight 3 elite (wave 2) | 1.20 | 1.10 | 72 / 81 | 19.2 / 13.2 | 96 / 148 |
| fight 4 boss | 1.25 | 1.125 | 68 / 78 | 20.0 / 13.5 | 91 / 144 |
| loop 2 elite | 1.26 | 1.15 | 67 / 76 | 30.2 / 13.8 | 46 / 141 |
| loop 3 elite | 1.38 | 1.25 | 58 / 68 | 46.9 / 15.0 | **3** / 129 |
| loop 5 elite | 1.62 | 1.45 | 43 / 53 | 97.2 / 17.4 | **0** / 109 |
| loop 5 boss | 1.69 | 1.475 | 40 / 52 | 101.2 / 17.7 | **0** / 106 |

## Before / after: measured in engine

A level-9 CPU Falcon (weight 104) stands in for the player, with Ember, Bolt and Nova. Each fight was started directly
(`MELEE_ENVOY_START_FIGHT`), two seeds each, 10800 frames, at synctest depth 1 (0 mismatches in every run). The
`kblog` lines give each stock lost: the percent, and the hit that took it.

| fight (stage) | before: KOs, percent at KO | before: biggest single hit | after: KOs, percent at KO | after: biggest single hit |
|---|---|---|---|---|
| 1 duel (BF) | none | 4% | none | 4% |
| 2 pack (YS) | 62% (a 2.7% hit, low speed: walked off) | 16% | 62% (same), 180% | 16% |
| 3 elite room (DL) | 115, 121, 160, 168% | 18.9% | 176, 217% | 15.8% |
| 4 boss Ganondorf (FoD) | **70% (31.9% hit), 85% (34%)**, 102% (23.4%), 115-189% | 34% | 80% (12%), 114-188% (last hits 12.8-18.1%) | 34% (one hit, not a KO) |
| loop 2 elite room (DL) | 126, 126, 164, 166% | 17% | 96-155% (6 KOs, last hits 9.7-19.9%) | 19.9% |
| loop 2 boss (FoD) | **58% (28.7% hit)**, 89, **99% (34%)**, 115, 116% (24%), 131, 132, 146% | 34% | 88, 90, 91, 105, 106, 137, 185% (last hits 12-14.5%) | 23.4% |

Before the pass, single hits of 24-34% took stocks at 58-99%. After it, no stock is lost below 80%, and every KO hit
is 9.7-19.9%: a normal Melee finisher at a normal Melee percent (a vanilla level-9 CPU smash KOs a mid-weight at
about 91% at FD's centre). Loop 2 still takes more stocks than loop 1 (more waves, level 9 CPUs), which is the
intended per-loop rise. The headless runs are too short to show the loop-3+ blasts the model flags (3% / 0%).
`kblog` on a loop-3 run is the follow-up check.

## Round 2 (laptop feedback on 342340622)

- **The 12% cap leaked on blasts.** The engine scales a blast hitbox by its owner's attack ratio after the sim armed
  it, so a capped 12% Nova landed up to 18%. Besides that, the `kblog` blast flag (any foe hit within 4 frames of
  any foe blast) also marked ordinary moves, which were the laptop's 24-30% "blast" hits. Fixed: a foe blast is
  armed at cap / attack ratio, rounded down, so what lands is at most 12%. The flag now needs the hit to come from
  the slot that fired the blast.
- **Re-measured** (kblog, level-9 CPU Falcon, two seeds each, 10800 frames):

| fight | KOs, percent at KO | KO hits | foe blasts landed (max) |
|---|---|---|---|
| loop 3 elite room | 72, 76, 101, 112, 133, 133, 162, 177% | 2.1-28.4% | 9% |
| loop 3 boss | 95, 102, 109, 112, 122, 145, 147, 164% | 12.6-28.7% | 9% |
| loop 5 elite room | 74, 91, 98, 120, 158, 167, 203% | 1-16.8% | 9% |
| loop 5 boss | 77, 83, 88, 103, 103, 114, 125, 126% | 14-18.1% | none |

  Across 44 blast hits, none landed more than 9%. The remaining hits above 20% are CPU smashes at the capped attack
  ratio (x1.25-x1.5: for example a 19% smash landing 26.6%), so later loops do KO earlier (72-77% at the lowest),
  and that is the intended per-loop rise.
- **A level-9 CPU with the strong row clears to loop 11. Is it too easy? No change, by decision.** That row is
  `ember:3,nova:3,bolt:3,*4000`: `*4000` is a test knob that multiplies the pilot's damage by 4, so the run measures
  stability, not difficulty. Without it, the measured runs above still lose 6-8 stocks per fight from loop 2 on. If
  the owner finds later loops too easy, the lever is the uncapped foe toughness (`foe_tough`) or +1 foe a loop, not
  knockback.
- `loop_run.sh`: run a passes on reaching loop 3, and a RUN OVER is reported but no longer required. Run c must
  continue in the loop run b last reached, whichever loop that is. The host's "saved run loaded" line masks the loop
  word to its low byte (it printed 520 = 0x208, which is loop 8 plus the stop-done bit).

## Round 3 (the laptop still saw "blast" hits of up to 32% on b990e17df)

The laptop's run (`loop_run` run a, `ember:3,nova:3,bolt:3,*4000,@3,kblog|grand_finale`, 300000 frames, from loop 3)
was repeated on the desktop on the same build and gave the same numbers: 325 hits flagged as blasts, 0.9-32.0%, and
stocks lost at 43.6, 53.3, 57.6 and 63.2%.

**The cap had not leaked. The log was wrong, in two ways, and round 2's reading of the engine was wrong too.**

1. **The blast flag went stale between fights.** `kblog` kept the play frame of the last foe blast in a static, and
   the play counter starts again at 0 in every fight. "Within 4 frames of the blast" was `play - blast_at <= 4`, which
   is true for every frame of every later fight until its counter passes the old blast's frame. So after one foe
   blast, every hit from that foe slot was flagged, for whole fights. In that run only 21 foe blasts were armed (13
   Nova, 8 Comet). 18 flagged hits were real blasts and every one landed **9.0%**; the other 307 were ordinary moves
   and Petey's attacks. The desktop's round-2 measurement started every fight in a fresh process
   (`MELEE_ENVOY_START_FIGHT`), so its static was always fresh: that is the whole desktop / laptop difference.
2. **The logged damage was the frame's sum.** The engine adds every hit a fighter takes in a frame and applies the
   sum once (`dmg.x1838_percentTemp`), and the take-damage hook sees only the sum and the source of the hit whose
   knockback won. A blast and a move landing on the same frame were one line.
3. **The engine never scaled a blast's damage by the attack ratio.** The attack ratio, and the victim's defense
   ratio, scale *knockback* only (`ftColl_80079AB0`). A move's damage grows with the fighter's size where the
   hitbox is made (`ftColl_8007ABD0`: an elite of size 1.15 deals x1.0375), and a blast capsule does not go through
   that. Round 2's "a capped 12% Nova landed up to 18%" was the same stale flag.

**Every way a foe drive's damage reaches a player, and where each is capped now:**

| path | drives | cap |
|---|---|---|
| a blast capsule on the engine's hit path | Nova, Comet (Spire, not in the foe pool) | **in the engine, where it lands**: `ftColl_80076ED8` clamps that capsule's damage on the victim to the blast's cap after staling and the victim's own multipliers, right before it joins the frame's damage (`hooks::ENVOY_BLAST_LAND` -> `envoy::blast_land`). The sim gives every foe blast the cap with `gd_envoy_blast_cap(slot, 12000, tag)`. Whatever multiplies the armed number on the way can no longer pass 12% |
| a blast that found no free capsule | Nova, Comet | percent through `hurt`, below |
| a percent fire (`hurt` -> `gd_envoy_percent`) | Ember, Bolt and its chain, Frost, Hex | in `hurt`, at the call that adds the percent: a foe's fire lands at most `FOE_FIRE_CAP` whoever set the fire context (it used to trust the cap the last fire left behind) |
| Mark: a marked player takes +30% of every hit as extra percent | Hex | the extra, together with the blast part of the same hit, is at most `FOE_FIRE_CAP` (it was uncapped) |
| Burning | Ember | 1.5% a second per stack; a foe's Ember holds one stack |

A foe's blast is still armed at 12% / its attack ratio (9% at x1.25-x1.3, 8% at x1.5). Since the ratio scales the
blast's knockback, that keeps a later loop's blast from launching much harder. It is a balance choice, not the cap:
the cap is the engine's.

**`kblog` now logs what was applied, per source.** A hit's `dmg` is the percent the fighter gained (read before and
after the hit lands). The engine records each blast capsule's part of the frame (`gd_envoy_landed`), so a frame with
a foe blast in it is logged as two records: the blast (flagged, then `kblog blast <drive>` = owner slot x1000 +
level x100 + capsule x10 + capsules landed, `kblog blast raw` = before the cap, `kblog blast launches` = its
knockback won the frame) and the rest. Percent fires log `kblog fire <drive>`, Mark `kblog mark extra`, Burning
`kblog burn tick`. A KO adds `kblog KO last blast`. Each hit also logs the knockback that won the frame.
The test knob `blastover` arms every foe blast at 30%, to show the engine's cap holding it.
`tools/rust-port/envoy/kblog_summary.py a.log` prints the per-source table and the KO split from a run's log.

**Measured** (the laptop's setup, 300000 frames from loop 3, Petey as the boss, two seeds; seed 1 is `#1`):

| run | loops | foe blasts landed | percent fires | Mark extra | foe-drive hits over 12% |
|---|---|---|---|---|---|
| seed 0 | 3-4 | 18 (12 Nova, 6 Comet): all 9.0% | 46: Ember / Bolt 12.0%, Frost / Hex 7.2% | 25, max 4.7% | **0** |
| seed 1 | 3-5 | 26 (16 Nova, 10 Comet): all 9.0% | 44: Ember / Bolt / a capsule-less Nova and Comet 12.0%, Frost / Hex 7.2% | 26, max 4.5% | **0** |
| `blastover` (armed at 30%), 150000 frames | 3-5 | 24: armed 30.0%, **landed 12.0%** every time | 37, max 12.0% | 28, max 5.1% | **0** |

Stocks lost (a fall: launch speed^2 under 100k in the log's units, so the player was not launched off):

| run | falls | launches | lost to a blast |
|---|---|---|---|
| seed 0 | 19: 39-141%, median 77% | 30: 44-177%, median 106%; four under 60% (44, 46, 53, 58%) | 2: a launch at 88%, a fall at 63% |
| seed 1 | 11: 10-145%, median 77% | 25: 44-212%, median 127%; one under 60% (44%) | 1: a launch at 212% |

No stock is lost early to a foe drive. The early launches that remain (44-58%) are ordinary CPU moves, mostly an
elite's 14.7-16.6% hit with knockback 185-237 (the attack ratio scales a move's knockback, the elite's size its
damage). The hits of 30% and more are Petey's (30-40%), which is the Petey lane's. Those are rule 1's numbers, not rule 2's; if the
owner still finds loop 3+ too deadly, the lever is `foe_atk` or the elite size, not the drives.

## Checks

See the branch's commit message for results: v2_drives d4/d7, loop_run, switch_torture (RUN_TIMEOUT 6000), gate.sh.

Round 3, on the round-3 build (vanilla disc): `v2_drives` every row at depth 4 and 7 with 0 mismatches; `loop_run`
with `SAVE_TEST=1` on a per-run copy of the test card (run a to loop 8 and RUN OVER, run c continued run b's loop);
`switch_torture` (RUN_TIMEOUT 6000) fights 1, 2, 3c, 5 with 0 switch KOs and 0 misplaced; `gate.sh` OK (the four
goldens match at their recorded counts, so vanilla is bit-identical: the two new hooks in `ftcoll.rs` are `None`
until an Envoy run begins a match).
