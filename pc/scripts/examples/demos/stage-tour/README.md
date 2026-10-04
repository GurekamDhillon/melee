# Stage tour

Shows Combined stage slots, queue, transition events, surface shaders and HUD. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: N: next stage; cycles FD/BF/YS every 8s; wipe/flash/morph.
`unload demo_stage_tour` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

A presentation reel of three static destinations with independent RGB looks and
wipe/flash/morph transitions. Preloads synchronously before installing a deterministic loop.
Captions report shader and preload refusal. No stage hazard code runs in destination slots.
Read stage_switch_demo, surface-stage and events for its components. Captured/dead fighters
can defer switches; the queue's native retry rules apply. Stage-slot meshes taking the surface
shader is an integrator acceptance check, because the original-stage shader contract alone
does not establish static DAT-slot draw coverage.

Fix2 defers the surface look until the frame after the `after` event and temporarily omits post
tints entirely. Clearing all passes expired the engine cover; removing only owned handles then
hung the tested engine. This mod performs neither operation while that engine repair is pending.

API reading: workspace `docs/scripting.md`, **Combined stage slots, queue, transition events, surface shaders and HUD**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
