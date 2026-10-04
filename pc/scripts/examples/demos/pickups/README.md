# Standalone Geno pickup

Shows Standalone items and model rotation. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: I: drop a native coloured pickup by P1; touch it to collect.
`unload demo_pickups` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Primitive native pickup visuals need no model. Mount at boot or use item_define from the contained definition. Events for unrelated items and duplicate collection are ignored.

API reading: workspace `docs/scripting.md`, **Standalone items and model rotation**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
