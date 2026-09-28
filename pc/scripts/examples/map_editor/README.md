# Kit map editor (offline)

Copy this folder to `mods/map_editor` beside the game. Copy the **contents** of an exported
`bf_interior_room/models/` to `map_editor/models/` (all 17 meshes, sidecars and shared atlases).
Generated model binaries are not included in this checkout. Enable only this kit mod while
testing; the room example also spawns 41 instances and consumes the same pools.

For a fresh kit export, from the game checkout, in Blender's Python lane:

```powershell
blender --factory-startup --background --python pc/assets_src/bf_interior/export_kit.py -- --output pc/scripts/examples/map_editor/models --editor pc/scripts/examples/map_editor/scripts/main.lua
```

The exporter updates the delimited palette/grid block in `scripts/main.lua` with the parts it
actually emitted and `UNIT = 5 * KIT_SCALE`. The checked-in catalog matches the current export:
6.5 game units per kit metre. Recopy the script and models together after changing kit scale.

Enter any offline match with P1. Press **F6** or **Z + D-pad Up** to start editing.
P1 becomes the cursor using the existing debug fly mode and its camera beyond stage bounds.
The gold cross is the snapped placement origin; the cyan square is the selected part origin.
Select uses nearest origin in XYZ, including the selected depth plane. It is not a mesh raycast.

| Action | Keyboard | Controller (P1) |
|---|---|---|
| Flight | WASD; Shift fast | Left stick; A slow, B fast (existing fly controls) |
| Palette part | Up/Down | D-pad Up/Down |
| Depth | PageUp/PageDown | D-pad Right/Left |
| Place | Insert | A |
| Select nearest | Tab | X |
| Move selected to cursor | M | Y |
| Rotate selected about Z | R / T (+/-15 degrees) | R / L |
| Duplicate at cursor | Ctrl+D | Z menu: Duplicate |
| Delete | Delete | Z menu: Delete |
| Undo / redo | Ctrl+Z / Ctrl+Y | Z menu: Undo / Redo |
| Save / load current file | Ctrl+S / Ctrl+O | Z menu: Save / Load |
| Collision overlay | F3 | Z menu: Collision overlay |
| Action menu | F2; Up/Down, Enter; Esc closes | Z; D-pad, A; B closes |
| Exit editing, keep map live | F6 | Z menu: Exit editor / play |

Every operation is available from the kit action menu, including grid subdivisions (1, 0.5,
0.25 metre), new-part rotation, collision enable and floor flags. Flags are the native sidecar
override: bit 1 = drop-through, bit 2 = ledges. Defaults are collision on, flags 3. Options apply
to **new** placements; duplicates retain their source options. Fly A/B speed modifiers also
apply while pressing those buttons for UI; use the stick at rest when placing precisely.
The menu does not pause the game; controller flight remains live. Do not toggle F11 off while
editing. Keyboard flight advances only on game logic frames, so resume a paused match first.

Rotation follows the model API: scenery can rotate fully, but collision transforms that reverse
floor/wall direction or leave world bounds are refused. The map and history stay unchanged.
Depth moves the visual model only: Melee collision remains in the XY fighter plane. Collision
overlay displays the engine's actual scripted lines at Z=0, including other mods' lines.

## Layout files and the map loader

The console command `map save castle.lua` saves a **Lua data file**, with read-back verification
and the previous contents at `castle.lua.bak`. It lives in
`scripts-data/map_editor_main/castle.lua` when installed as a `mods/` script. The script id shown
by the `scripts` console command determines the data directory (slashes become underscores).

```lua
return {version=1, units=6.5, parts={
  {part="bf_floor_4m",x=0,y=40,z=0,rot=0,collision=true,floor_flags=2},
}}
```

`map load castle.lua` loads and resumes editing. Loading replaces the current map and is undoable.
`map play castle.lua` uses this mod as the **map script**: loads that data file, closes the editor,
restores the prior fly/overlay state, and loads that file on subsequent offline matches in this
script session. `map on` resumes editing the live map and cancels play autoload so unsaved edits
survive a stage change. After relaunch/reload, issue `map play`
again; no preference is silently persisted. To share, ship the script, matching models, and the
layout; the recipient copies the layout into that script's own data folder. No cross-mod file API
or unrestricted filesystem access is required. Loading text runs with an empty Lua environment
and validates the returned schema before changing any instances.

Other console actions: `map place`, `select`, `move`, `rotate -15`, `duplicate`, `delete`,
`undo`, `redo`, `clear`, `off`. `clear` is undoable. Save/load accept plain `.lua` basenames.
All edits require offline editing mode. Save also works after a match ends for recovery.

## Limits and lifecycle

- 128 models **total across all scripts**; up to 200 scripted collision lines, further limited
  by the host stage's remaining collision joints. This is a kit assembly editor within existing
  engine limits, not an unlimited world streamer. Native blast zones remain unchanged.
- Undo/redo retains 64 edits. A failed operation does not advance history. If restoring native
  instances also fails, editing is refused; save the retained document and restart the match.
- Changing stages retains the document, rebuilds its instances at the same absolute coordinates,
  and clears history. A map with no room on the new stage may fail restoration; the document can
  still be saved. It does not silently drop parts from the saved layout.
- Manual savestate slots pair with Lua document snapshots and clear undo on restore. Untracked
  persistent-state/rewind loads require saving and restarting the match. Do not edit while
  rewinding; Lua state is not part of the native snapshot. Hot script reload loses unsaved edits.
- Exiting editing restores the previous P1 fly state and scripted collision overlay. Unloading
  removes this script's known instances and releases its asset references. Scene end frees pools.
- Offline gating also exists in the model/fly engine APIs. This manifest is never rollback-safe.

Validation and the unexecuted Windows lane are recorded in `map-editor-report.md` at repo root.
