# Fighter state and control

Shows State; Gameplay. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: T: teleport P1; P: P2 percent +25; C: stand/fight CPU.
`unload demo_fighters` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Unload restores recorded positions/percent while the match is live. CPU control has no readback setter inverse; the documented demo baseline is fight.

API reading: workspace `docs/scripting.md`, **State; Gameplay**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
