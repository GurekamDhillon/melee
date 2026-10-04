# LAB hitboxes and frame timeline

Shows LAB inspection and control. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: J: request jab frame 1 (pauses); SPACE: resume; B: debug draw toggle.
`unload demo_lab_inspection` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Watch the status caption for acceptance/refusal; gameplay writes are offline only.

API reading: workspace `docs/scripting.md`, **LAB inspection and control**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
