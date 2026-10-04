# Profiler and zone readout

Shows State (gd.perf); docs/profiling.md. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: O: prof on/off; console prof report / prof trace 8 / prof hitch 20.
`unload demo_profiler` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Reads native zones rather than inventing a Lua zone-begin API. Never add overlapping worker/CPU or parent/child times. See docs/profiling.md for startup GPU-query requirements and unavailable GPU timing.

API reading: workspace `docs/scripting.md`, **State (gd.perf); docs/profiling.md**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
