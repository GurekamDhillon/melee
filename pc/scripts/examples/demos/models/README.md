# Model handles and labels

Shows Stage content; Mission model paths and reloads; Contacts and traces. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: R: reload optional models/demo.gxmesh; cyan slab is the no-export fallback.
`unload demo_models` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Export your own GXMS visual as models/demo.gxmesh, with its atlas/sidecars. Model basenames resolve inside the mod; built-in kit pieces require the user-installed kit model mod and are not global names. The fallback slab is deliberate; no model is fabricated.

API reading: workspace `docs/scripting.md`, **Stage content; Mission model paths and reloads; Contacts and traces**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
