# Envoy retail playtest - 2026-10-04 fix1

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
