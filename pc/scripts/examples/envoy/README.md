# SuperTime Envoy ? retail Classic / Adventure

Menu entry (Atlas step 2, 2026-10-06): **SOLO > ENVOY**. `mod.json` `menus` names the entry and the host calls `on_entry("envoy")` (generated into
`main.lua` by `tools/port/envoy_bundle.py`). Branch A: it opens a small Atlas screen (START CLASSIC, START ADVENTURE, ENVOY MENU, BACK) in front of the
native menu; ENVOY MENU opens the legacy menu below and holds the native menu with `gd.ui.hold_menu` until it closes. Whether a run starts from the
front end (Branch B is the fallback: launch the same offline scene `envoy start` uses) is checked in the game, not by the offline tests. Under
`MELEE_ATLAS=0` the legacy SOLO hub lists the entry too. Offline test: `pc/tests/envoy_atlas_entry.lua`.

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
Drive model art is the original `envoy_drives` mod (mount it beside this one); without it floor drops fall back to the engine's plain item and cells to flat colours.
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

## Builds for CPUs in an ordinary VS match (`envoy vs`)

Offline only, never in netplay, never under an Envoy run, and off until asked. In a normal offline VS match (stocks, timer, any stage):

```
envoy vs on                       the explicit switch (this match only; a new scene clears it)
envoy vs max [seed] [depth [loop]]   a max build for EVERY present CPU port (1..6), sliced across frames; the CPUs are set to fight
envoy vs <strength> [seed] ...    the same with a numeric strength (the whole bounded search, sliced)
foe roll <max|strength|-> [seed] [port 1..6] [role]    one port, by hand (the LAB's command, now also here)
```

`max` is the highest bounded build of the depth and loop: every slot, the keystone allowance the rules permit, drives rolled at the deepest tier the
depth band allows, best of 8 full builds by strength. With no depth set it means depth 12 / loop 3. Seeds above 2^31 are folded (logged). Without
the switch, nothing here changes: a plain VS match refuses `foe`, the LAB works as before, and a run's host is untouched. Tests: `pc/tests/envoy_vs_*.lua`.

## The pieces (the readability split, 2026-10-05)

A **drive** carries at most TWO rules: one below effective depth 5, two from depth 5 on (a standing rule and a trigger rule), and a white
drive gets no extra one. It is named by its colour and its rules (`Green Drive: Kindling + Updraft`), never by grammar. A **keystone** is one
rule line and one price line; the price is one of seven learned once: Slow, Low, Fragile, Weak, Vulnerable, Lock, and Bleed (lose damage
points each time an event keystone fires). No keystone pays with a status on its own fighter. The pool is 72 pieces: 36 drive rules (13
standing, 23 trigger), six uniques and 30 keystones. The rule payoffs (Pyre, Cinder, Shatter, Brittle, Malice, Feasting, Renewal, Rush,
Crosswind, Bastion, Trailing) open at effective depth 5. A second copy of a rule you hold merges into it whatever its colour.

Five statuses are learned, each with one word everywhere: **Burning**, **Chilled**, **Haste**, **Guarded**, **Marked** (the old Curse; its internal
id is still `curse`, so saves are valid). **Momentum** is a counter (up to 5, spent when you land: `engine:momentum(port)` gives the count a
later pass draws as orbs) and **Shock** is private to the electric theme. A crit is x1.5 unless the piece says otherwise (Brutal, Gambler,
Executioner). The glossary is `drive_text.glossary()` / `mod_status.glossary()`. The pieces' one-line texts are in the research notes
(`_build/audit-20261003/envoy-split/APPENDIX-pieces.md` in the workspace). A saved run that holds a piece the split cut drops it with a notice
(`drive_bag.migrate`); Heavy, Featherweight and twelve keystones are the retired ids.

**A run started from the menus or the console uses the rule host** (this pool, bag and opponent rolls) by default; `envoy rules off` selects the
older companion-stat route for the next run, which is unchanged. `envoy devui on|off` shows the developer figures (see PLAYTEST.md).

Rolled drives and a controller bag are available in the offline LAB debugger.
The shared pool has 72 records: 36 drive rules, six uniques and 30
keystones (the LAB debugger alone, without the keystone module, has fewer). `drive give rare 42` grants loot; `drive drop magic 123` spawns a
physical pickup near P2. `bag` or Z+START opens the twelve-drive bag. Select a
drive, then a slot to equip/swap; select an occupied slot to unequip. Left/Right
pages details, including current family totals, safety limits and equip previews.

`depth <n> [loop]` sets the LAB progression dial. Effective depth is depth +
13 times the New Game+ loop. Four slots and one keystone grow to five/two at
effective depth 5 and six/three at 10. Tiers keep growing after three; repeated
uniques stack their benefits. Physical drives retain their rolled tiers, while
chosen keystones follow the current dial. A downshift that cannot hold existing
slots or keystones refuses without discarding equipment. Collect ground drops
and commit pending edits before changing depth.

`foe roll [strength] [seed] [port] [normal|boss|finalboss]` rolls an independent
same-pool CPU build. Omit strength, or use `-`, to target the player strength.
The opponent receives only that number, progression and role; it never copies
the player's modifier identities. Difficulty rises with depth/loop; bosses get
larger factors. High-strength rolls weight Damage resistant, Cleansing and defensive
chains more heavily than raw launch. `foe list` shows the actual result;
`foe fight [port]` / `foe stand [port]` switch CPU mode; `foe clear` cleans up.
The same shaders and timed nameplates show the rolled modifiers. CPU modifier
edits and foe rolls must commit separately; timed plates replace the debug HUD.

Percent damage remains separate from current-hit launch. Bonuses add within a
family, while interactions between offensive, defensive and status families can
become extreme. EM4 follow-up replaces the former tight balance ceilings with
numerical/physics safety bounds. Event-chain depth and per-frame work stay
bounded. Lost stocks clear transient statuses while equipment, steady values,
looks and native rules persist. Scene/run defaults start fresh; the LAB debugger
remains separate from Classic loot integration.

This follow-up requires a native rebuild: 32-rule capacity, percent/launch
safety and snapshot/journal layouts changed. Older engines refuse progression
edits. Source tests and formula models do not establish live KO, controller,
shader or LAB rewind acceptance. See PLAYTEST.md for the current owner script.

Echo upgrades (EM5, source pass): of Echoes arms aerial copies at ages8/16/24,
with higher tiers arming more copies. Trailing leaves pictures while Hasted;
Echo Heart repeats all supported fighter hitboxes at a global25% attack-damage
cost; Echo Oath trades global50% attack damage for stronger echoes. The visible
echo family uses assembled outputs and is shared by independently rolled CPUs.
Different aerial/all-move filters remain separate; matching copies add within
the final bounded rule. Article/projectile hitboxes are outside this carrier.

`echo add 8 nair .4`, then `echo add 16 nair .4`, arms neutral-air echoes for
the primary player; `echo clear` removes manual rules. Manual arbitrary-delay
commands currently create collision rules without their own picture emitter.
The catalogue Echoes demo has three pictures: E arms aerial copy2, N arms only
neutral-air on copies1/2. Those use a joint native description for picture age
and collision delay. Copy tint/flash data is implemented, but its renderer hook
is an integration request to the afterimage lane; visible armed styling remains
pending. See PLAYTEST.md for rebuilding and actual collision/rewind acceptance.
