# Atlas pause screen (kind = "pause")

Registers a pause screen (`kind = "pause"`: a list with **Resume** and **Log a line**) and names it with `gd.ui.pause_screen`. That is all the script does at load.

**Without the takeover, nothing changes.** Retail's pause looks and works as it always did. Run the game with `MELEE_ATLAS_PAUSE=1` (or the setting `atlas_pause`) and a retail pause in a VS-style match (START in a LAB match) pushes this list; the pausing port drives it. **Resume** (or B) calls `gd.ui.unpause()`: a one-shot request the game takes at its next unpause check, so retail's own routine runs for the pauser (audio, camera, the ten-frame unpause timer). The retail pause panel stays visible unless you also hide it: `MELEE_ATLAS_RETAIL=pause.panel` shows only this list.

A pause screen is **never opened online** and the takeover never runs in a netplay match.

Launch with `MELEE_SCENE="mode=lab;stage=fd;p1=fox/hu;p2=falco/cpu0;cpus=idle"` and `MELEE_ATLAS_PAUSE=1`, then in the console `load <absolute path of this folder>`; press START. Needs the Atlas font roles in `ui/`. Offline stand-in checked; not yet seen in the game. `unload demo_atlas_pause` releases it and clears the name.

API reading: workspace `docs/scripting.md`, **Atlas screens (`gd.ui`)**, the retail takeover.
