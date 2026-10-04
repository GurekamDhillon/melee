# Save, step and rewind

S receives an actual `on_savestate(1)` acknowledgement, remembers damage, then adds 25 damage.
L must restore the remembered value; the next frame prints PROVED or FAIL. A history depth of
zero alone says nothing about snapshot success. This demo enables 120-frame history, rearms it
after load, and restores the prior depth/interval on unload.

Shows Rewind, reload and persistent states. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: S: snapshot 1; L: load; P: pause; N: step; R: resume.
`unload demo_rewind` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

Do not run alongside stage slots: native slots refuse snapshots/rewind. Persistent state-library APIs and hot reload are described in the linked LAB reference; neither is needed for this minimal snapshot demo.

API reading: workspace `docs/scripting.md`, **Rewind, reload and persistent states**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
