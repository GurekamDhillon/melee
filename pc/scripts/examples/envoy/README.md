# SuperTime Envoy ? retail Classic / Adventure

2026-10-04 fix1 supersedes the earlier retail source tuning. Source and
offline tests are not native gameplay acceptance. Open `envoy menu`, continue to
the asset-free menu, choose a fighter and start Classic (default) or Adventure from setup.
Mode, difficulty and stocks can be selected with the controller. Normal is 2;
Mario and three stocks are the initial selections. Nothing runs in netplay.

The game's own stage tables, bonus stages, opponents, handicaps, AI, continues
and boss conditions remain retail. After each cleared stage, choose one of three
drives in an existing kit panel. The final reward is three times larger. Adventure
can give a base boss-clear choice before its Giga decision, then a definitive
completion bonus. The chosen reward shows before/after values and is atomically
saved before acknowledgement; a refused write can be retried with A. Uncommitted
choices expire without awarding. A broken script cannot hold the scene forever.

CPU stat spreads are seeded by run, stage, loop and port. Single-opponent budgets
center on the player's total stat levels, with ?15% width, bounded without clipping
bias. Teams divide each opponent budget by the square root of opponent count.
NG+ adds 15% of baseline budget per loop. The same native damage, movement and
shield, jump-height and knockback multipliers stack with giant/metal/retail handicaps; CPU AI is unchanged.
An opponent budget floor of four levels makes a new companion's opponents distinct.
A four-second stage-start tag and coloured marker show each opponent's leading stat. Ice Climbers share
port stat modifiers with their follower; they cannot have independent spreads.

Completion evolves a young companion using existing life gains, saves an Envoy
win, and loops with the same fighter, difficulty, stocks, companion stats and age.
Age advances once when the initial chain starts; NG+ preserves age. Game over or
abandon settles the chain and reincarnates if its life ended. Each completed
playthrough has its own Envoy ledger win; the next loop is counted only at its
actual stage start. Duplicate events and write retries do not count twice.

Scripted completion skips retail trophy, credits and congratulations to protect
the card's scores, clear counts, target-test records, trophies and unlocks. This
is an explicit presentation departure. Ordinary retail runs keep their records
and ending. Crazy Hand and Adventure Giga Bowser conditions remain retail.

ENVOY4 remains the save format; ENVOY1/2/3 still migrate in memory. Interrupted
runs reuse the pending seed, saved stats and age, restarting at retail stage zero;
this is not a retail scene snapshot, and NG+ loop index is session-local. Saves
at rewards/completion/game over replace the whole profile atomically. Corrupt,
future or unreadable profiles are refused without replacement.

All balance and cadence settings live in `scripts/companion.lua` under `retail`.
Set `reward_every=0` for completion rewards only. Physical CPU KO drives default
off; the optional switch uses real native items and native collection. Retail picks
offer 120/180/240 growth points before grade multiplication. Effects use L/(L+2):
Power caps at +24% damage, Speed +30% run/air speed, Guard 24% damage resistance,
20% knockback resistance and +35% shield, Jump +30% ground height, +25% air-jump
height and +15% air speed. A C-grade smallest Speed pick gives +18%, three give
+23.3%, seven +25.4%; distributing picks spreads those benefits. Reward choices
show their concrete change, reuse pickup sounds/FX and animate bars across level-ups.
Guard briefly flashes a blue marker when damage percent rises.

The authored physical garden, DNA, grades, reincarnation, stat HUD and optional
original drive-model adapter are retained. Existing kit components and coloured
drive glyphs draw the reward panel; the kit has no 3D-model icon preview API.
Optional local `envoy_drives_sa2` model art stays separate and is not copied here.
Results return to the asset-free menu. A walkable garden is offered only when
its model resolves; a failed garden spawn falls back to that menu. No disc-derived
assets are included. Optional garden assets can be prepared into one mod:

```
python tools/port/envoy_prepare.py --kit menu/out_roguelite/room-kit --output _build/tmp/envoy-classic-native
```

Use a new destination. The tool preserves existing folders and includes the
unchanged shared mission runtime and original kit dependencies for the garden.

Commands: `envoy menu`, `hub`, `start`, `classic`, `adventure`, `retry`, `stop`,
`status`, `menudump`, `dump`; `give` and `reset-profile confirm` are diagnostics.
`stop` retries a pending settlement. Missing retail APIs show unavailable, without
a maze fallback. `envoy campaign` explicitly selects the parked mission diagnostic
flow; its historical tests remain, but it is not the default.

See MENUS.md, PLAYTEST.md and NATIVE-TEST-PLAN.md for controller/native acceptance.

EM3 adds rolled drives and a controller bag in the offline LAB debugger. The
pool contains30 records (22normal,5fixed uniques,3separate keystones). `drive give
rare 42` grants loot; `drive drop magic 123` spawns the existing physical pickup
near P2. `bag` or Z+START opens four equipped slots and a twelve-drive bag.
Select a bag drive, then an equipped slot to equip/swap; select an occupied slot
without a bag selection to unequip. Discard and keystone choices are menu rows.
Details include rule text and equip changes; Left/Right pages long details.

Lost stocks now clear temporary statuses/stacks/events, while equipment,
implicits, steady values, looks and native rules persist through respawn.
`mod add <id>` still works beside loot and is merged at the highest tier per ID.
One keystone and eight native rules are enforced before a build edit. Defaults
start fresh on scene/run; drive_lab.tuning.persist controls retained inventory.
No opponent rolls or Classic loot attachment are enabled in this packet.

Normal elements convert ahead of combat; protected originals are immune.
Pyre is a real1.25x contact launch multiplier against Burning targets, without
inventing Curse. Pyromancer converts ordinary owned attacks to Fire and doubles
damage taken from original Ice hits. Native owner/ID traces name actual changes.
Split damage is refused. The generator, bag, rules/status outputs and look
summary share checkpoint state; live LAB rewind and screen acceptance remain
integrator work. See PLAYTEST.md and the EM3 report for scripts and exclusions.
