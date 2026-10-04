# Mission camera modes

Shows Camera; missions README Camera. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: F: follow; C: chunk; V: shaft; R: retail; jump along primitive route.
`unload demo_mission_cameras` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

A minimal visual illustration of the three modes, using direct camera poses. Production mission camera logic adds footprint/outer-edge clamps, leads, respawn cuts and projection correction; read missions/scripts/camera.lua for that version.

API reading: workspace `docs/scripting.md`, **Camera; missions README Camera**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
