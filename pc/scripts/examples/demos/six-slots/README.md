# Six fighter launch and recycling

After R succeeds, wait two logic frames for the explicit PROVED/FAIL message. The demo reads
the primary entity's visibility/dormant flags and zero damage; a true recycle return is insufficient.
If P6 remains hidden, this is a native engine failure to repair/retest, not a Lua visibility toggle.

Shows Six-fighter direct matches. Read `scripts/main.lua`; no external art or disc data is included.

Run on an offline vanilla LAB match on FD with P1 human and P2 CPU:
`load <absolute-path-to-this-folder>` in the console. Controls: L: launch six-slot LAB; K: send P6 below blast; R: recycle after KO.
`unload demo_six_slots` releases the demo. Use a fresh match between demos.
For native item/FX packages, mount this folder as a mod at boot (see the catalogue guide).

L performs a scene reset by explicit user hotkey. Slots 5/6 are CPU slots; no physical controllers are added. Recycle only after retained CPU KO reaches Sleep/Rebirth; paired/transformed/character changes are refused.

API reading: workspace `docs/scripting.md`, **Six-fighter direct matches**.
Verification: initial version ran in the external vanilla audit; **fix2 changes not run in game**. See the catalogue for acceptance limits.
Technique credit: GD scripting reference and engine registration/implementation sources;
project samples informed lifecycle/ownership handling. No third-party source or assets consulted.
