# Native effects and emitter controls

Shows State (gd.fx); LAB effect control. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: E: play square sparks; C: slow/brighten; R: fade out; F: attach package.
`unload demo_effects` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Mount this folder as an enabled mod before boot so the package loader can find fx/DemoSquares. The supplied package contains only original JSON and draws texture-free coloured square primitives. No converted effects, textures or models are included.

API reading: workspace `docs/scripting.md`, **State (gd.fx); LAB effect control**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
