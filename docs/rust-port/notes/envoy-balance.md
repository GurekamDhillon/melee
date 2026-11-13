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
a player takes from a foe (damage, percent, launch speed^2 on the next frame) and the percent of every stock lost.
Both are logs or test-only. Without the knobs, nothing that is hashed changes besides the new rules.

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

## Checks

See the branch's commit message for results: v2_drives d4/d7, loop_run, switch_torture (RUN_TIMEOUT 6000), gate.sh.
