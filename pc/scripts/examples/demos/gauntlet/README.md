# Gauntlet

Shows Combined missions, maze, waves, reserve, pickups, cameras, slots, post and data. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: Fight room 1; reach maze goal in room 2; KO boss in room 3; ENTER retries.
`unload demo_gauntlet` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Three playable rooms: an authored primitive arena with two waves; a seed-42 generated
eight-cell platform maze; Battlefield with a pre-benched CPU finale. Native defeat drops are
owned, deduplicated pickups that scale attack/run speed. Camera starts in follow mode, then
45-frame chunk framing. Boss entry uses a static stage slot; victory has a timed post flourish,
kit result card and atomically saved best logic-frame time. No art export required.

Run on fresh vanilla FD LAB, P1 Fox human/P2 Marth CPU. Death before the boss fails the run;
ENTER restarts via the scene grammar. Setup yields after the entry area and each maze cell;
the persistent camera is claimed after the construction task ends. Setup failure/cancellation
retires partial geometry and releases the reserve; the boss slot loads only after the arena is gone.

Play room 1: attack the two Goombas, then the Redead; walk over the dropped buff tokens.
Room 2 starts at (240,8). Seed 42's intended path is c1 â†’ c2 â†’ c3 â†’ c4: walk right through
the doors at x=350 and x=480, then drop through c3's central floor gap x=533..557 into c4.
Walk right to the gold goal near (580,-96). Cyan lines show the original primitive geometry;
stairs serve optional vertical branches. No key teleports you to the maze goal or skips enemies.
The ordinary stage transition places you in the finale; Marth starts at zero damage in fight mode.
Build damage and KO him through actual combat to earn victory. The audit's teleported maze/boss
completion does not accept this route or fight; both need a fresh manual run.
This is a small mission runner, not the full missions schema. Read demos/enemies, reserve,
pickups, modifiers, chunks, mission-cameras, data and post-custom; read missions for production
streaming/reload. The contained pure maze generator/recipe are copied from missions with credit.
Stage-slot switching with a benched fighter is an explicit native acceptance item.


API reading: workspace `docs/scripting.md`, **Combined missions, maze, waves, reserve, pickups, cameras, slots, post and data**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
