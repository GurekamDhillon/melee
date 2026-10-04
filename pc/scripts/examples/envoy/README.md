# SuperTime Envoy ? retail Classic / Adventure

2026-10-03 source packet. This default replaces generated campaigns; source and
offline tests are not native gameplay acceptance. Open `envoy menu`, continue to
the garden, choose a fighter and start Classic (default) or Adventure from setup.
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
shield multipliers stack with giant/metal/retail handicaps; CPU AI is unchanged.
A short stage-start tag shows each opponent's leading stat. Ice Climbers share
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
off; the optional switch uses real native items and native collection. Jump is
still capped air-speed growth; jump-height caps await an engine API.

The authored physical garden, DNA, grades, reincarnation, stat HUD and optional
original drive-model adapter are retained. Existing kit components and coloured
drive glyphs draw the reward panel; the kit has no 3D-model icon preview API.
Optional local `envoy_drives_sa2` model art stays separate and is not copied here.
No disc-derived assets are included. Garden assets can be prepared into one mod:

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
