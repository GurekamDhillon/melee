# Scripted stage platforms

`gd.stage_add_platform(x, y, width, options)` creates a floor collision line centred at
`(x, y)` in game world units. `gd.stage_add_line(x0, y0, x1, y1, "floor", options)`
can use the same visual option for a sloped floor. The calls remain offline gameplay
mod APIs; `gd.stage_remove(handle)` and `gd.stage_move(handle, x, y)` act on the
collision line and its visual together.

Set `options.model` to the basename of an exported stage model:

```lua
local h = assert(gd.stage_add_platform(0, 42, 40, {
  passthrough = true, ledges = true, model = "bf_platform"
}))
```

Put `bf_platform.gxmesh` and `bf_platform.gxtex` in `<mod>/models/` beside
`main.lua`, or at the mod root when the entry is `<mod>/scripts/main.lua`. The
name accepts letters, digits, `_`, and `-` only. A missing, invalid, or
unloadable pair gives a Lua error before any collision line is added.
`model` applies only to floor lines.

The mesh's top is aligned with the collision line. Its X axis scales to the
line's current width, including after `gd.stage_move`. Authored depth and
thickness stay fixed. Each modeled platform uses one indexed GX triangle draw
with the baked colour atlas. The atlas contains the source material emission;
the optional `.glow.gxtex` retains it separately for future bloom work. Model
selection and asset storage are native and do not enter gameplay or rollback
snapshots.
Assets load once per process; restart the port after replacing an exported
file. `gd.stage_view(false)` hides all scripted stage geometry, including models.

## Building the example art

From the game repo root, run Blender in the background with the original
source generator and this exporter:

```powershell
& 'D:\SteamLibrary\steamapps\common\Blender\blender.exe' --factory-startup --background --python pc/assets_src/bf_platform/export.py -- --width 8 --depth 2.4 --thickness 0.4 --units-per-meter 5 --texture-size 512 --output pc/scripts/examples/bf_platform/models/bf_platform
```

The exporter rebuilds `BF_Platform`, sets the parametric dimensions, evaluates
the Geometry Nodes mesh, bakes all six materials on `UVBake`, and writes
`.gxmesh`, `.gxtex`, `.glow.gxtex`, plus PNG previews. It uses the port's v1
GXTX RGBA8 texture layout and an indexed GXMS mesh with big-endian positions,
UVs, and triangle indices. The separate glow texture is not used by the current
renderer. At the default scale, 8 Blender metres produce 40 game world units
of width; the Lua platform can choose any collision width.

The example mod is [pc/scripts/examples/bf_platform](../pc/scripts/examples/bf_platform/).
Copy that folder to `scripts/bf_platform` beside `melee-pc.exe`, then enable
the mod. It puts three modelled platforms on Final Destination in an offline
match. The port log should contain `script stage: model ... loaded`,
three `script stage: model 1 attached` lines, and
`script stage: model 1 first world draw`.
