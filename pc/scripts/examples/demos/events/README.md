# Native game events

Shows Hooks; Clank observation and timed presentation; Stage slots and switching. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: Fight/clank; E: spawn enemy; events log after logic; no fabricated events.
`unload demo_events` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

For an actual stage-switch event run Stage tour or the existing stage_switch_demo. A second human sword fighter is useful for clanks. This observer does not manufacture an event to make its HUD change.

API reading: workspace `docs/scripting.md`, **Hooks; Clank observation and timed presentation; Stage slots and switching**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
