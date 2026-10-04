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

Enter any offline match with P1. Press **F6** or **Z + D-pad Up** to start editing. **F1** opens
the keybind help; the left panel's rows are clickable with the mouse.
P1 becomes the cursor using the existing debug fly mode and its camera beyond stage bounds.
The gold cross is the snapped placement origin; the cyan square is the selected part origin.
Select uses nearest origin in XYZ, including the selected depth plane. It is not a mesh raycast.

| Action | Keyboard | Controller (P1) |
|---|---|---|
| Flight | WASD; Shift fast | Left stick; A slow, B fast (existing fly controls) |
| Tool | 1..5: place, select, move, rotate, scale | Z menu: Tool: ... / Next tool |
| Palette part | Up/Down | D-pad Up/Down |
| Depth | PageUp/PageDown, or wheel | D-pad Right/Left |
| Use the tool at the pointer | LMB (drag with move) | — (pad keeps the fly cursor) |
| Place | Insert | A |
| Select nearest | Tab | X |
| Move selected to cursor | M | Y |
| Rotate selected about Z | R / T (+/-15 degrees); rotate tool: drag to face the pointer | R / L |
| Scale selected | F7 / F8 (-10% / +10%), clamped 0.25..4 | Z menu: Scale -/+ |
| Mirror selected | Shift+X / Shift+Y | Z menu: Mirror X / Y |
| Transform (hold) | G / E / C hold: modal move / rotate / scale; tap: switch tool | — (pad holds LT/RT) |
| Frame selection | F | — |
| Move constraint | Shift+C: free / X only / Y only | Z menu: Move constraint |
| Snap | Z: on / off | Z menu: Snap on/off |
| Duplicate at cursor | Ctrl+D | Z menu: Duplicate |
| Duplicate xN | `map duplicate <n>`; action row "Duplicate x4 at cursor" (one undo step) | — |
| Delete | Delete | Z menu: Delete |
| Undo / redo | Ctrl+Z / Ctrl+Y | Z menu: Undo / Redo |
| Save / load current file | Ctrl+S / Ctrl+O | Z menu: Save / Load |
| Collision overlay | F3 | Z menu: Collision overlay |
| Action menu | F2; Up/Down, Enter; Esc closes | Z; D-pad, A; B closes |
| Help / keybinds | F1 or H (Esc or a click closes) | Z menu: Help / keybinds |
| Palette filter | F4, then type; Enter done, Esc clears; `map filter <text>` | — |
| Selection inspector | right panel: drag x/y/z/rot/scale to scrub, click a field to type a value (Enter applies, Esc cancels); click collision / floor flags | — |
| Gizmo handles | drag the selected part's handles: red X / green Y = move, cyan = scale, gold ring = rotate; hovering names the handle in the status line and rings it white | — |
| Placement ghost | translucent preview at the cursor while placing; `map ghost on\|off` | — |
| Action search | Space: type to filter actions, Up/Down, Enter runs; `map run <text>` | — |
| Action log | `map log on`: named steps; click one to step back or a `redo:` row to step forward (`map history <n>` / `map redo <n>`) | — |
| Stage bounds | `map bounds capture` stores the live camera/blast bounds in the layout (v2, drawn green/red); `bounds restore` clears them; `bounds camera l r t b` / `bounds blast l r t b` set them; drag a green edge handle to move a camera edge (one undo step) | — |
| Spawns | `map spawn <slot>` reads a spawn point; `map spawn <slot> <x> <y>` moves it (saved in the v2 layout). Slots: 0-3 starts, 4-7 respawns, 127-146 item spawns | — |
| Out-of-bounds warning | placing or duplicating outside the blast zone / camera bounds raises a toast and a log line (uses the layout bounds, else the live stage) | — |
| Multi-select | Shift+Tab or `map select add` joins the nearest part to the anchor; Ctrl+Tab or `select remove` drops one; `select all` / `select clear`; Move, Rotate, Scale and Delete apply to the whole selection as one undo step | — |
| Panel rows | click a tool, part or action row | — |
| Exit editing, keep map live | F6 | Z menu: Exit editor / play |

## Tools and the mouse

The left panel is clickable: the five tool rows, the palette (place tool), grouped under
`floor`/`wall`/`trim`/`glass`/`corner`/`door`/`balcony` category headers with the recent parts pinned
first, the action list (every other tool, the row the keyboard menu is on), and the Help row. The top bar shows the
active tool and part; the bottom bar shows the document, cursor, grid, snap and part count.

With the mouse, a plain click uses the current tool at the pointer: **place** drops a part there,
**select** picks the nearest part, **move** starts a drag that follows the pointer (one undo entry
for the whole drag; `C` constrains it to one axis), **rotate** faces the part at the pointer
(snapped to 15 degrees while snap is on) and **scale** steps +10%. The right button opens the
action menu, the wheel changes depth by one grid step, and the mapping is a homography solved
from four projected plane samples each time the camera or depth changes, so it follows the real
projection (widescreen included) without assuming camera constants.

Rotation and mirroring follow the model API: scenery rotates freely; mirroring an instance that
owns collision goes through despawn/respawn because the engine rejects effective axis-sign
changes in place. Scale magnitudes and translations stay atomic updates. Scale fields are written
to the layout only when they differ from 1, so existing v1 files stay valid.


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

## Missions

A layout can carry a **mission**: a start, enemies in waves, checkpoints, a goal zone, trigger zones and
an objective. Author it on any map built here, then `map mission play`. Everything is Lua on the
existing `gd.*` calls (`gd.spawn_enemy`, `gd.enemy_status`, `gd.enemy_state`, `gd.enemy_remove`,
`gd.stage_set_spawn`, `gd.model_set` through the editor's own sync); there is no engine change.

Try the samples: copy `samples/first_mission.lua` (flat strip: two waves, a checkpoint, a goal) or
`samples/multi_level_mission.lua` (two ramps up to a goal at the top: a trigger-spawned wave, a
position-started wave, a mid-way checkpoint, messages) to `scripts-data/map_editor_main/`, start an
offline match on Final Destination (the LAB works) and enter `map play first_mission.lua`.

| Command | Does |
|---|---|
| `map mission start [x y]` | where P1 begins (default: the cursor). Goes through the editor's spawn mechanism: slot 0, so it wins over a hand-set `map spawn 0` |
| `map mission enemy <kind> [wave]` | an enemy marker at the cursor, wave 1 by default |
| `map mission goal <w> <h>` | the goal zone, centred on the cursor (replaces the old one) |
| `map mission checkpoint <w> <h>` | a checkpoint zone centred on the cursor |
| `map mission objective <type> [time=<s>] [lives=<n>]` | `reach_goal`, `defeat_all` or `defeat_then_goal`; replaces the previous objective, so omitted `time`/`lives` are cleared |
| `map mission wave <n> time <s>` / `x [<x>] [left\|right]` / `clear` | a **wave rule**: wave `n` starts after `s` seconds, or when P1 crosses `x` (default: the cursor; `right` unless `left`), without waiting for the earlier waves. One rule per wave; a new one replaces it |
| `map mission trigger wave <w> <h> <n>` | a **trigger zone** at the cursor that spawns wave `n` when P1 enters |
| `map mission trigger message <w> <h> <text>` | shows `text` (up to 80 characters) on the HUD for 4 s |
| `map mission trigger collision <w> <h> open\|close [r]` | for this run only, removes (`open`) or restores (`close`) the collision of every part within `r` units (default 6.5) of the **selected part**. Only floors and ramps carry collision in the kit |
| `map mission trigger complete\|fail <w> <h>` | ends the mission |
| (any trigger) `... repeat` | fire on every entry instead of once |
| `map mission pick <start\|enemy\|checkpoint\|goal\|trigger> [enemy kind \| trigger action]` | choose what a click places with the mission tool |
| `map mission list` / `delete <index>` / `clear` | numbered items (start, goal, checkpoints, enemies, wave rules, triggers) and the objective, to the log / remove one / remove the whole mission |
| `map mission test` | start the mission from the editing cursor without leaving the document; the cursor's position is kept |
| `map mission play [file]` | load `file` first if given, leave editing, place P1 at the start, spawn wave 1 and run |
| `map mission restart` / `stop` | start over (a test restarts from the same spot) / end the run and remove its enemies; stopping a test hands the editor back at the cursor (the engine refuses flight in some poses, e.g. teetering at a floor edge, so for up to 3 s the editor retries and nudges the stick down) |

Each edit is one undo step and a refused one changes nothing. The action menu has Mission: start at
cursor, play, restart and test from cursor.

**Mouse and the mission tool (key 6).** With the mission tool a click on empty space places the picked
kind (Up/Down or the inspector's first row changes it; `map mission pick`), a click on a marker selects
it, dragging moves it, and dragging an edge or corner of the *selected* zone resizes it (minimum one grid
step). Every gesture is one undo step; Delete removes the selected marker. The right-hand inspector
shows the selected marker's x, y (and w, h for zones) (click a number to type a value), an enemy's kind
and wave (click to step), a trigger's firing mode, and a delete row. Markers are drawn in the overlay:
green START, red enemies (`E1 goomba w1`, with the wave's rule, `@5s` or `x>130`), blue `CP<n>` zones,
gold GOAL, purple `T<n>` trigger zones (a collision trigger also marks its target). The selected marker gets
a white outline and handles.

`map play <file>` also runs the mission if the layout has a playable one (and again on each later
offline match); an unplayable one still loads the map and says what is missing.

**Layout.** An optional `mission` table, only in `version=2` files (a mission makes the saved file v2;
v1 files, and v2 files without one, load as before; a v1 file that has a `mission` key is refused):

```lua
mission={
  start={x=-120,y=40},
  enemies={{kind="goomba",x=-50,y=36,wave=1}, ...},       -- wave defaults to 1
  goal={x=125,y=45,w=24,h=60},                            -- x,y is the centre, w,h the full size
  checkpoints={{x=20,y=45,w=16,h=60}},
  objective={type="defeat_then_goal",time=120,lives=3},   -- time and lives optional
  waves={{wave=2,time=5},{wave=3,x=130,dir=1}},           -- optional rules: time (s) XOR x (+ dir 1 or -1)
  triggers={                                              -- optional; zones are x,y centre + w,h
    {x=20,y=70,w=24,h=60,action="wave",wave=2},
    {x=-130,y=60,w=40,h=70,action="message",text="Climb",once=false},
    {x=0,y=60,w=20,h=60,action="collision",at={x=26,y=26},r=30,open=false},
    {x=190,y=100,w=30,h=70,action="complete"},            -- or "fail"
  },
}
```

Loading is strict: unknown keys, fields that do not belong to a trigger's action, wrong types,
non-finite numbers, unknown kinds, actions or objective types and over-limit counts refuse the whole load
with a message and leave the current document untouched. Whether the mission is *complete enough to
play* is checked at play time, so you can author step by step: `reach_goal` needs a goal, `defeat_all` an
enemy, `defeat_then_goal` both; every mission needs a start and an objective; a rule or a trigger must
name a wave that has enemies.

**Limits.** Kinds: `goomba`, `koopa`, `redead`, `like_like`, `octorok`, `polar_bear`, `topi` (the
engine's `gd.spawn_enemy` list). Waves 1-8, 32 enemies per wave (the engine allows 32 live script
enemies at once, shared with other scripts), 64 enemies, 16 checkpoints, 16 triggers, zones up to 2000
units a side, `time` 1-3600 s, `lives` 1-99.

**Rules.**
- *Waves.* A wave with no rule and no trigger is sequential: the next one spawns when every enemy
  spawned so far is gone, in ascending order. A wave with a rule starts on its own condition, a wave named
  by a `wave` trigger starts when that trigger fires; both start whatever is still alive. The enemies are
  "all gone" only once every wave has started, so a defeat objective cannot finish while a rule or trigger
  wave is still waiting.
- *Zones* test P1's origin (the feet), so make them tall enough. Triggers fire on entry (outside to
  inside), once by default.
- *Checkpoints and death.* Touching a checkpoint makes it the respawn point (the latest touch wins; the
  start is the first one). The point is written to stage spawn slot 4, which is where the engine puts P1
  after a KO: P1 comes back on the engine's own rebirth platform at the checkpoint and the engine moves
  it, with no script teleport. Slot 4 is given back to the document when the mission stops.
- *Lives* count P1's falls themselves (the LAB has infinite respawn and no stocks, so `gd.set_stocks`
  means nothing there; elsewhere P1's stocks are set to `lives + 1` so the engine's game-over cannot end
  the match before the mission's own failure). The KO that uses the last life fails the mission
  (`reason=lives`); the engine then respawns P1 as usual and the match goes on. Without `lives`, KOs never
  fail it.
- *Time* counts logic frames at 60 per second, so it follows pause and turbo.
- *Defeats.* An enemy is **defeated** when the engine reports a stock defeat (`gd.enemy_status`
  `defeated`; this is what `on_enemy_defeated` fires on), or when it ends without that event while last
  seen well inside the stage: a Koopa killed into its shell is reported only as removed. An enemy that ends
  within 30 units of the blast zone is **lost** (it fell out of the stage): it leaves the fight so the
  objective cannot stall, is logged `mission: lost <kind>`, and is not counted as a defeat (`defeated=` and
  `vanished=` in the result line).
- *Result.* A result ends the mission, removes its enemies, shows MISSION COMPLETE or MISSION FAILED and logs
  one line, `mission: complete time=...` or `mission: failed reason=<time|lives|spawn|trigger> ...`. A HUD
  line shows objective, enemies left, time and lives. `map mission restart` retries.
- *Cleanup.* Returning to edit mode (`map on`, F6 into editing), `map off`, match end, a savestate load
  and unload remove what the mission spawned, restore collision a trigger changed and log
  `mission: aborted (...)`.

**Verified in the real engine** (isolated mods root, LAB, Falco, ACE disc, scripted pad input;
`_build/audit-20261003/mission-native2/RESULTS.md` has the log lines): start, waves, checkpoint, goal and
complete; death and checkpoint respawn through slot 4; lives failing on the last KO and the match going on;
the `time` limit failing at exactly 300 frames for `time=5`; restart, stop, returning to editing and `map
off` leaving no enemy alive; the whole editing command set from an empty document (place, mission start /
enemy / goal / checkpoint / trigger / wave / objective, undo, redo, save, load), then playing the result;
`reach_goal`, `defeat_all` and `defeat_then_goal` completing; a killed Goomba counted as defeated and one
that fell off the stage counted as lost; a Koopa killed into its shell counted as defeated; a timed wave
appearing at its second, a conditional wave on crossing x, a wave trigger, a message trigger, a collision
trigger (a gap bridged for the run, restored after); `topi` spawning; `map mission test` and its return
to the cursor; the mission tool's click / drag / resize / undo with a synthetic pointer computed from
`gd.project` (the editor's own screen-to-world mapping, hit tests and drags ran; the real OS mouse was not
driven); the multi-level sample completed with real pad input; no script error in any run.

**Controller path.** The Z menu (or F2) has Mission: next marker kind, place marker at cursor, select nearest
marker, move marker to cursor and delete marker, which do the same edits at the flight cursor (resizing a
zone from the pad is not possible: type w and h in the inspector, or use the mouse). Stub-tested only.

**Not verified.** How anything *looks*: the overlay markers, handles, HUD, message line and result banner
were drawn without a Lua error but nobody looked at them. A physical mouse. The Z-menu controller path in the engine. Netplay (missions are offline only). Rollback or savestate
interplay beyond "loading a state ends the mission". More than two or three enemies at once, or the other
four enemy kinds in a fight (`like_like`, `octorok`, `polar_bear` were not used in a mission run).

**Fixed on the way.** The editor's screen-to-world mouse mapping (`inv3`, used by every mouse tool, not
only missions) returned the cofactor matrix instead of its transpose, so on the real, non-symmetric
camera projection every click mapped to about (0, 0): the existing click-to-place tool could not have
worked in the engine. The stub's identity projection hid it; there is now a test with a real homography.

**Not supported.** Doors that open on a key or kill (use a collision trigger), items, dialogue, per-enemy
behaviour or health tweaks, more than one player, netplay, editing while a mission runs, a mission in a
savestate (loading one ends it), controller-driven marker authoring.

**Module layout.** The state machine is `scripts/mission.lua` (no `gd` calls; loads under plain `lua`).
The engine loads one entry file per mod and has no `require`, so `main.lua` embeds it verbatim between
`BEGIN/END GENERATED MISSION` markers, like the kit block. After editing `mission.lua`, run
`python tools/port/map_mission_sync.py` from the workspace (`--check` verifies; the tests do; it refuses a
`main.lua` with more or fewer than one block). Copying the folder to `mods/map_editor` is enough;
`mission.lua` is only the source.

Tests: `lua pc/tests/map_mission_test.lua` from `melee/` (pure module plus editor commands, the mouse
tool and the runtime glue against the `gd` stub), wrapped by `tools/port/map_mission_sync.py --check` and
`tools/port/test_map_mission.py`.

Validation and the unexecuted Windows lane are recorded in `map-editor-report.md` at repo root.
