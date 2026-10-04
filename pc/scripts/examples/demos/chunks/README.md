# Mission folder, chunks and reload

Shows Mission model paths and reloads; Stage content. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: R: reread tiny level; arrows choose chunk; fresh handles per area.
`unload demo_chunks` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Tiny is a contained teaching schema, not the full mission runtime. The catalogue also links missions/first for production validation, atomic staging, automatic 3x3 streaming and its follow/chunk/shaft modes.

API reading: workspace `docs/scripting.md`, **Mission model paths and reloads; Stage content**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
