# Custom material, glass and light

Shows Custom shaders, post passes and model materials. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: L: warm/cool light; optional lit/glass/custom meshes; fallback slabs.
`unload demo_materials` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Material/light effects need your own exported mesh. The lit, glass and custom material templates belong beside models/lit.gxmesh, models/glass.gxmesh and models/custom.gxmesh. Keep glass alpha=1 in the collision sidecar for translucent sorting. The custom material uses the supplied original normal-colour WGSL body. Without meshes the slabs and light-state caption remain visible; plain slabs do not receive the material light.

API reading: workspace `docs/scripting.md`, **Custom shaders, post passes and model materials**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
