# Screen models

Shows `gd.kit.model(model, x, y, w, h [, opts])`: a script model drawn into a rectangle of the 640x480-based script
canvas (width from `gd.safe_area()`), in call order with `gd.fill`, `gd.box`, `gd.text` and `gd.kit.*`. The point is a menu
cell that shows a real 3D model. The API reference is the "Screen-space models" section of `docs/scripting.md`.

Launch (offline LAB, idle CPU; the demo only draws, it consumes P1's D-pad while shown):

    MELEE_SCENE="mode=lab;stage=fd;p1=fox/hu;p2=falco/cpu0;cpus=idle"
    # then, in the console:  load <absolute path of this folder>

Mode 1, cells: a 4x3 grid, one model per cell, the focused cell spinning. Every cell draws `gd.fill` (background),
`gd.kit.model`, `gd.box` (frame), a `gd.fill` badge over the model's corner and `gd.text` over its foot: the badge and the
frame sitting on top of the model are the ordering proof. Cell 7 is locked (`dim`), cell 8 a ghost (`alpha`), cell 12 draws
with `clip = false`.

Mode 2, grid: the real grid component (`scripts/grid.lua`, a copy of `demos/grid-inventory/scripts/grid.lua`, embedded
into `main.lua` by `scripts/embed.py`) with `icon_draw` set to one call (see `boot_grid` in `main.lua`; that is the
adoption line).

Keys: arrows / D-pad move the focus, TAB or pad L switches mode, F6 hides it.
Console: `sm_mode <1|2>`, `sm_focus <n>`, `sm_press <dir>...`, `sm_bench [all|n [frames]]` (script time of on_draw for 16 and 48
models, and the host's fps), `sm_shot <name>` (`gd.screenshot` leaves the interface overlay out: use an OS window capture).

## Models and their origin

The twelve models (`models/gem_*.gxmesh`, GXMS v2, 4 to 224 triangles) and their shared atlas (`models/shapes.gxtex`,
`shapes.glow.gxtex`, 256x256 GXTX RGBA8) are original geometry and a procedural texture written by
`tools/make_models.py` (Python standard library only; run it to regenerate). They are not derived from any game or any
other project's assets. The Envoy drive models (`envoy_drives_sa2`) are Sonic Adventure 2 data (Sega / Sonic Team,
extracted locally, "never commit" in that mod's README) and are deliberately not used or copied here.

Credit: the light is the world model draw's own key light (`gs_model_draw`, `gw_script.c`); the grid component is this
repository's `demos/grid-inventory`; no third-party code or assets.
