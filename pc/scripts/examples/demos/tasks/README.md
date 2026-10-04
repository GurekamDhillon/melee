# Coroutine input task

Shows Tasks (scripts that wait); Input. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: Q: jump, drift, wait for landing; task releases its pad claim when complete.
`unload demo_tasks` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Uses the function form of wait_until in a yielding task, distinct from the nonblocking native contact watcher. Tasks advance on logic frames; they must never spin in a blocking loop.

API reading: workspace `docs/scripting.md`, **Tasks (scripts that wait); Input**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
