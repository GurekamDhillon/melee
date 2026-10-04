# Input and pad reads

Shows Input; Tasks (scripts that wait). Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: I: hold right 30 frames; R: release; mouse/pad readout.
`unload demo_input` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Input claims the whole pad. After the hold it stays neutral until R or unload; extra paused PADReads do not consume frames.

API reading: workspace `docs/scripting.md`, **Input; Tasks (scripts that wait)**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
