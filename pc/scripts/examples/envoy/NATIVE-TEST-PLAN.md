# Envoy Classic/Adventure native acceptance - 2026-10-04 fix1

Source-only work does not establish native acceptance. This plan supersedes the
maze campaign as the default acceptance route; historical mission checks follow.
Build the engine through tools/port/build.sh, including the bridge fixpoint and
ABI audit, only after this no-build packet is integrated. Use the vanilla disc,
a current stamped EXE, an isolated run folder and backed-up Envoy script data.

1. With Envoy enabled but unopened, play an ordinary offline match and ordinary
   retail Classic. Verify no modifier, tint, hold or retail record suppression.
   Also verify that every new gameplay mutation is refused in netplay/rollback.
2. Open Envoy, choose Mario, Classic, Normal (2), three stocks. Inspect stage-one
   opponent tag and player HUD. Confirm the shipped stage order, opponents,
   bonus stages, intermissions, giant, metal and wireframes. CPU allies must not
   receive enemy budgets. Check replacement wireframes and sub-fighters.
3. Clear each stage. A controller alone must navigate three drive choices; choose
   each colour in turn, verify one award, pickup sound/FX and 48-tick before/after
   bar animation with level-up crossings. Read the concrete percentage changes. Hold
   A through transitions: no repeated reward. Retry after continuing: same spread.
4. Reach Master Hand; also test retail Crazy Hand conditions on an eligible
   difficulty/time. Check the larger final reward, evolution and two NG+ loops.
   Fighter/difficulty/stocks and companion age/stats carry; enemy budgets scale.
5. Decline a continue halfway through a run. Growth is retained, results settle
   once, owned stats/tints/holds clear, and results return to a working menu with
   no garden assets. With resolved assets, explicitly enter the optional garden. Repeat with
   abandon, script unload and scene change. Check all six ports and sub-fighters.
6. Back up the memory card and compare retail Classic/Adventure/Target Test
   scores, clear counts, trophies and unlocks before/after scripted runs.
   Scripted progression must not inflate retail records. Verify the explicitly
   documented ending presentation policy; normal retail completion stays normal.
7. Refuse profile writes at reward, completion and game over. Retry a prepared
   write without duplicate drives/evolution/aging. Break/unload the reward script:
   engine timeout releases the flow. Restore an old engine: clear unavailable
   message, no generated-level fallback.
8. Play Adventure from the hub. Check its traversal/enemy/boss transitions, bonus
   and continue flows, final Bowser/Giga Bowser conditions and NG+ restart.
9. Judge panel readability, stage-start tags, modifier feel, team balance, NG+
   scaling, reward cadence and whether progression feels useful and fair.
10. Check one start and accepted clear per fight/bonus (including every single,
    team, giant and metal fight). Each CPU enemy receives its spread; allies do not.
    Read the four-second coloured marker even on meshes without working tint.
    Compare green/yellow/blue after 1/3/7 picks; test full hop, short hop, double and
    multijumps/animation-driven jumps. Guard should resist launch and flash blue
    on damage. Respawn/replacement must retain effects; mode exit restores them.

## Parked mission acceptance (historical)

# Envoy slices 2?4 native acceptance ? not run by this job

Supersedes the slice1/fix acceptance plan. Use a current stamped executable,
vanilla disc, offline LAB on FD, human P1 and match-start P2 CPU, isolated run
folder and backed-up script data. Do not combine the mission demo with Envoy.

Prepare a local mod with existing authored room-kit assets:
`python tools/port/envoy_prepare.py --kit menu/out_roguelite/room-kit --output _build/tmp/envoy-slices-2-4-native`
Choose a new output each time. The tool refuses existing output and incomplete kit
assets, assembles a complete folder before publishing, and embeds current shared
sources. Root-only source folders intentionally contain no copied mesh/texture
assets. Keep the same mounted mod id and script-data owner across reloads.

1. Enable Envoy in a normal match. No pause, pad mask, modifiers, tint or garden
   until explicitly opened. Open title/Profile, then Continue. If no scene is
   active, confirm LAB launch. Staging must retain P1 until a live floor probe
   succeeds; Entry/Landing waits must remain intact. Test missing kit: clear
   refusal, no substituted geometry or soft lock.
2. Walk the garden with Fox, Falco, Kirby and Ganondorf. Inspect all floor seams,
   walls, labels and the colour-tinted companion placeholder. Open each station
   by proximity + A; hold A to check no repeated activation. Close/back to the
   garden and resume movement. Nest stays closed; C-stick still attacks.
3. Start at the exit, play maze8 ? authored crossing ? maze16 ? CPU boss. Inspect
   map/seed logs and actual connections. Real controller traversal, including
   drops/ascent, is required; fly/tour only establishes a fixture. Interludes
   pause logic, retain drives/levels, show the next theme, and advance once on A.
4. Check payload rewards 20/25/30/35 points by depth, before grade multiplication.
   Watch bars, 24-frame receiving-bar flash and 48-frame level-up text. Compare
   repeated pickup sounds for the existing controller's bounded pitch streak.
   Native hover/size/spin are the drive-polish definition, not new Lua physics.
5. KO P2, then reach the boss exit: one evolution for a young companion. Check
   named passive on Companion/results, raised grade and DNA. Power/Speed/Guard
   are native fighter_mod ratios; Jump/Skybound changes air speed. Balanced/Sure
   Footed changes run and air speed. Pickup radius17.5 and drop chance80% stay
   fixed at all levels and depths. Jump height remains inactive pending an API.
6. Compare fresh level0 with levels1,5,10,25 using PLAYTEST.md. Check damage also
   scales knockback; total Power never exceeds10%. Judge movement, survivability,
   drop flow and whether the larger bonuses remain fair.
7. Complete three consecutive runs including garden returns and fighter changes.
   Inspect native items, FX/post handles, tints, modifiers, benched ports, areas,
   model references and retirement queues between runs and after unload. Repeat
   the original Fox win ? Falco relaunch crash sequence, including a grounded
   old actor and new actors in Entry/Landing.
8. Complete a six-start life including a failed/abandoned run. Age increments once
   per accepted start; collected growth survives loss. Sixth settlement becomes
   an egg in the hub. Inherited grades/DNA/colours remain, level0,10% points carried.
   Temporary white boosts disappear; evolution's one inherited step remains.
9. Force disk refusal on start: refuse the run and clean ownership. Force refusal
   on settlement: retries write the same prepared outcome without aging, awarding
   drives, evolving or reincarnating twice. Interrupt after a successful start
   save, reload the profile, and resume the same pending seed with unchanged age.
   `envoy retry` after settlement reconstructs the previous seeded geometry.
10. Migrate copies of ENVOY1/2/3 saves to ENVOY4. Existing levels and fractional progress should
    remain coherent; legacy reach traits, DNA, grades, points, life gains and evolved
    type become Jump one-to-one. Unknown old history stays unknown. Refuse
    empty, corrupt, future-version and unreadable profiles without replacing them.
11. Force a staging/readiness/retirement/cleanup stall and inspect its named script
    deadline. Check pause with retired resources pending, resume and drain. Unload
    during staged install must release resources without waiting for another frame.

Offline checks prove rules, deterministic generation and stub ownership. They do
not prove native floor/ECB safety, traversal, renderer quality, sound, performance
or tuning feel. Those remain owner/integrator acceptance.

Engine request: expose validated ground/air jump-height fighter modifiers with
reset, cleanup and snapshot/rewind semantics. Desired height cap is15%, currently
inactive; use registered air speed only. No teleports or velocity pokes. Maze
reachability must assume Jump0: Mario full hop about29 units, double jump about52.
The previous per-instance collection-radius engine request is withdrawn.

## Drive-polish merge follow-up

All nine `companion.tuning.juice` presentation switches now default on. Inspect all
five colours: core halo, floor pool, sparkles, highlight following native spin,
pop trail, collect burst toward the fighter, escalating sound streak and HUD bar
flash. White must use its distinct sound and label flash. Collect, expiry, room
transition, abandon, match end and unload must leave no resting FX; collect tails
finish within 24 logic frames. Try each switch off independently. Compare the
existing pickup-juice demo I/J before changing balance; no new demo was authored.

Missing/refusing FX or sound calls must degrade presentation safely while native
collection remains the sole reward authority. The native definition retains
6-unit hover, 2.5 scale, 12-degree tilt, 6 degrees/frame spin, 900-frame life and
120-frame blink. Inspect the glass/core and judge sounds/performance by eye/ear.

Compare Jump0 with Jump1/5/10/25 for air control. Native collection radius stays
17.5 world units, drop chance80%, including evolutions and every room depth.

## Slices fix1 acceptance

Copy the source mod without any additional kit assets and start from a fresh
profile. Expect one authored-room fallback log and four depths ending in the boss.
Return to the garden and confirm stations work without a model kit. Install the
complete kit under missions/models/ to exercise generated rooms instead. Remove
one required chunk mesh or its sidecar: the entire campaign should choose authored
rooms before gameplay begins.

Refuse the first room (remove path/level.lua), including after a queued readiness
wait. No new age, records, seed or failure result may be written. The setup menu
must explain the failure; no owned pause, tint, fighter modifier or boss reserve
may remain. Restore the room, retry and return to garden. Verify mounted SA2 assets
load drive_blue and drive_white; a missing colour logs its exact filename once and
retains that colour's HUD marker without hiding other colours.
