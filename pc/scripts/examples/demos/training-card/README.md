# Training card

Shows Combined LAB timeline, hitboxes, contacts, trace and wait_until. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: LEFT/RIGHT: move; M: inspect chosen move; D: landing drill; SPACE: resume.
`unload demo_training_card` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

A usable LAB inspection card: select an engine-listed move, request its numeric motion,
step/resume to inspect active hitboxes, read the script timeline, and retain five contact events.
D starts an input-free native watcher for the real cyan platform, with a 180-frame landing limit.
PASS/FAIL includes the native diagnostic reason; tracing writes JSONL to this mod's data folder.
Read lab-inspection, contacts and collision for each piece. set_motion bypasses entry setup,
so special-move behavior must still be tested through real controller input.

API reading: workspace `docs/scripting.md`, **Combined LAB timeline, hitboxes, contacts, trace and wait_until**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
