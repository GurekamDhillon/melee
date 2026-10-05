# Envoy rule-host run: what to see, stage by stage (ux pass 2026-10-04)

Setup: `envoy rules on`, then `envoy classic` (console), vanilla disc, offline. Controller: A / B / X / Y, D-pad or stick.
1. Stage 1 start: a panel says "Your starter drive" and names it; the strip top-left shows one filled pip and three dark ones, Strength, Depth 0.
2. Fight: the opponent's plate lists its modifiers (probably "No modifiers" this early). Hit it past 50%: "A drive dropped!" appears, a drive with a beam sits on the floor near you. Walk over it: a pickup card, the strip flashes.
3. Stage clear: the reward screen. Left rows: two offered drives, Skip, your four slots, your bag (new drives marked NEW), keystones. Press A on an offered drive: it goes into slot 2. Watch the totals line. You have about 15 s (countdown at the top); if it runs out, nothing is lost: the first offer is taken, free slots fill, the rest stays in the bag.
4. Stages 2-4 (team stage drops more): fill all four slots, then choose an offer: you are asked which drive to swap out, with before -> after totals. Try "Keep it in the bag", Skip (A twice), and X.
5. Stage 6 (depth 5): "Fifth slot unlocked", "Keystone allowance: 2", "Drive tier 2". Open the bag (Z+START) and pick a keystone; each shows its drawback.
6. Bonus stage: three offered drives. Master Hand: three offered, one Unique. After it: "New Game+ 1" and your build carries over.
Judge: can you tell in five seconds what a drive does; is 15 s enough (the engine hold is capped at 1800 host ticks); strip position vs the retail percent display; drop frequency. The logs (`melee-pc.log`, lines `envoy rules:`) say exactly what was equipped, swapped, bagged or discarded.

EM5 echo source pass (2026-10-04) is the current addition below. Its36-record
pool changes deterministic loot/foe recipes from the preceding32-record EM4
follow-up. Preserve those sections as historical model evidence, not current
seed guarantees. Echo source fixtures do not establish live gameplay acceptance.

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
Resume; open bag, equip all six; choose Frozen Oath, Pyromancer and Still Heart; close/resume.
```text
foe roll - 2144865533 2
foe fight 2
```
The late records are tier11, include Glass Core and unlock all three keys. Strength226.890 versus257.161; model4/4 attacks. For a boss/final boss, clear/commit before `foe roll - 2144865533 2 boss` or `foe roll - 2144865533 2 finalboss`. Expected strengths287.690/348.456 and modeled4/3 or3/3 attacks. Recreate a fresh stock between directions; statuses, healing, spacing and ledge triggers change actual results.

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
