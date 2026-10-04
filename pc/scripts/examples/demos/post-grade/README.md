# Colour grade

Shows Custom shaders, post passes and model materials. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: P: enable/disable pass; compare the same view.
`unload demo_post_grade` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Uses the existing shader-demo body, preserved independently for one-feature study. Depth outline follows visible depth discontinuities, not object IDs. Bloom uses half-resolution extraction/blur then full-resolution composition; normalized targets are not HDR.

API reading: workspace `docs/scripting.md`, **Custom shaders, post passes and model materials**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
