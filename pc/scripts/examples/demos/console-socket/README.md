# External console socket

Shows The console (localhost socket). Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: Python client.py <port> demo_ping; external commands read player state.
`unload demo_console_socket` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Start the integrator's game with MELEE_CONSOLE_PORT=51707, load this folder, then
run `python client.py 51707 demo_ping "= gd.player(1).x"` from this folder.
The client reads the initial banner, sends one line at a time and drains replies through
the exact ok/error terminator. Connections are localhost-only; timeout or EOF is failure.
Use `--help` for details. Technique credit: pc/scripts/console.py and docs/scripting.md.

API reading: workspace `docs/scripting.md`, **The console (localhost socket)**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
