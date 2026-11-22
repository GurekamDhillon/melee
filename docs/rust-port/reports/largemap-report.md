# largemap — batch integration report

Code only. No build, game run, or commit was performed. The Lua test has been
syntax-checked; its runtime assertions have **not** been executed.

## Changes

- Script collision pool: **200 → 768** lines, subject to the stage's remaining
  vertex/line/joint capacity. The joint ceiling is now 1,024. `mpLibLoad` allocates
  `max(256, base_joints + reserved_lines)` joints for a prepared script map;
  other scenes and online retain the retail allocation. Vertex and line arrays
  remain 2,048 and 1,536: **mpIsland's `visited[0x600]` requires that line limit**.
- Script targets: **32 → 128**. Normal item creation, destruction and category
  accounting remain in use; this does not bypass allocation failures.
- Runtime `gd.model_*` instances: **128 → 256**. Per-asset collision sidecars
  remain limited to 32 lines; immutable asset slots remain limited to 32.
- Named areas reclaim runtime instances, their attached collision, standalone
  script lines/platforms, and script targets. Unloaded objects stop drawing and
  stop colliding. Slots are reused; handles are not. Area names and membership
  live in the existing `script_game.c` snapshot domain. Nonreused native owner
  tokens prevent an old snapshot from assigning areas to a replacement script.
- `gd.stage_bounds` sets live camera and blast rectangles in world coordinates,
  within the existing finite ±100,000 coordinate range. Stage offsets are
  preserved. The normal camera reads these bounds; no cinematic camera claim
  is required. Both match views get a 200,000-unit far plane while bounds are
  overridden; both original far planes are restored on reset/scene end, including
  debug camera modes. Existing debug flight already follows flying fighters alone.
- Native immutable mesh/atlas pins have a **64 MiB** scene budget. Shared
  atlases are counted once. Pins remain until scene end so old snapshots cannot
  refer to freed assets. Area unload reclaims live content, **not asset pins**.
- `gd.stage_stats()` reports live counts and capacities, visible instances,
  script state bytes, collision reservation bytes, HSD heap free bytes, and
  pinned asset bytes/budget. Existing debug overlays use the new capacities.

Most new logic is in `pc/gameworld/script_largemap.inc` and
`pc/platform/gw_script_largemap.inc`, included by the existing TUs. There are no
new compilation units or response-file entries. Shared edits are marked
`largemap`; the required engine hooks are in `mplib.c` and `camera.c`.

## API and ownership

All new APIs require a gameplay script in an active offline match and refuse
netplay/rollback sessions. Writes fork the LAB timeline through `gs_rw_branch`.
Names are scoped to the calling script, 1–47 bytes, with at most 64 loaded areas.

```lua
local floor = gd.model_load("bf_floor_4m") -- retain reusable assets outside builders
local function load_room()
  return gd.area_load("west-room", function()
    assert(gd.model_spawn(floor, {x=-500, y=100}))
    assert(gd.stage_add_line(-550, 80, -450, 80, "floor"))
    assert(gd.spawn_target(-500, 120))
  end)
end
load_room()                       -- true: created
assert(gd.area_loaded("west-room"))
load_room()                       -- false: already loaded; callback is not called
gd.area_unload("west-room")       -- true; false when absent
load_room()                       -- new handles; invokes builder again
gd.area_unload("west-room")
gd.model_release(floor)

local old = gd.stage_bounds()     -- {camera={left,right,bottom,top}, blast={...}}
gd.stage_bounds{
  camera={left=-8000, right=8000, bottom=-1000, top=2000},
  blast={left=-8500, right=8500, bottom=-1500, top=2500},
}
gd.stage_bounds(false)            -- restore the original rectangles
```

Builders are synchronous: no `gd.wait`, nested loads, or area unloads inside
them. An error removes the instances/lines/targets created by that builder and
rethrows the error. Use `assert` on creation calls that return `nil, reason`.
This is content-lifetime cleanup, not a transaction for arbitrary Lua/game
side effects. Asset retain/release, legacy `stage_add_model` GObjs, enemies,
camera changes, fighter changes and Lua tables are not part of area ownership.
Load shared assets outside builders. Instances may still be moved or despawned
individually. Script unload destroys its areas; scene end destroys all areas and
restores the original bounds. Bounds are stage-wide, not owned by an area.

Reload recipes are ordinary Lua functions, not retained by the engine: call
`area_load(name, builder)` again after unloading. After loading a savestate, use
`area_loaded`/`model_get` to reconcile Lua bookkeeping. Keep the same scripts
loaded when restoring states: content from an unloaded script retains its retired
identity rather than becoming owned by a replacement script. Lua state is not saved,
and builders are not replayed on resimulation frames. These APIs are offline
only; this does not add deterministic online streaming.

## Memory and snapshot considerations

The 40 MiB MEM1 size is unchanged. Larger script pools add approximately
**147 KiB of snapshotted game globals** (32-bit layout arithmetic, not a linked
measurement). Collision arrays are reserved once per scene; compared with the
old 200-line reservation, the copied map grows about 40 KiB and the joint array
by at most 39 KiB. The engine's collision-island nodes retain/reuse their
high-water allocation. Unloading does not shrink those pools or the global
arrays. Target objects additionally consume normal game heaps while alive.

Mesh/texture bytes are native allocations, outside MEM1 and snapshots. Their
64 MiB budget covers persistent mesh and unique atlas image buffers, not
temporary loader copies, Lua memory, renderer caches or the whole process.
The existing page-sharing snapshot implementation is unchanged. Area metadata
and instances occupy a fixed amount of snapshot state; loading more historical
room names does not grow a native registry. Streaming different unique assets
can still exhaust the 32-slot/64 MiB scene cache; use a reusable kit or change
scenes. No unsafe eviction of snapshot dependencies was introduced.

## Self-checking test and launch

Test: **`pc/tests/largemap.lua`**. It logs
`TEST largemap section <n>: PASS|FAIL <detail>` for each check, catches errors,
has a startup/task watchdog, and calls `gd.quit()` on completion or failure.

It tests 210 simultaneous model instances, 300 collision lines (crossing the
retail joint limit), 40 targets, failed-builder cleanup, idempotent loads,
snapshot restoration of an unloaded area, and atomic bounds validation. It
then creates a corridor spanning **20 times the actual Battlefield camera
width** using exported Blender kit floors/walls/beams. Scripted debug-flight
input takes Fox to all 20 room centres and back. Only three adjacent rooms
remain loaded (18 visible instances, six lines). Each stop checks position,
camera following, live counts, loaded/absent floor queries, stable pinned asset
bytes, fixed state size, and HSD free memory within 64 KiB of the warmed value.
It also checks survival and normal camera following far outside retail bounds
after flight is disabled, complete cleanup and bounds restoration.

Required fixtures are the **real exported BF interior kit**, not placeholder
art. This checkout contains source/sidecars; generated GXMS/GXTX files are not
assumed to exist. The batch lane must provide the existing kit export first
(`pc/assets_src/bf_interior/export_kit.py`, instructions in its docstring), then
stage only the following assets beside the test. Do not load the example's
room-building script as well.

From the workspace root in Git Bash, after the batch's single build and kit
asset preparation (commands below were **not run here**):

```bash
# GW_MELEE and GW_BUILD_ROOT select the integrated checkout/build, as usual.
fixture="$GW_BUILD_ROOT/largemap-fixture"
kit="$GW_MELEE/pc/scripts/examples/bf_interior_room/models"
mkdir -p "$fixture/models"
cp "$GW_MELEE/pc/tests/largemap.lua" "$fixture/largemap.lua"
for part in bf_floor_4m bf_wall_solid_4m bf_beam_4m; do
  cp "$kit/$part.gxmesh" "$kit/$part.coll.json" "$fixture/models/"
done
cp "$kit/bf_kit.gxtex" "$kit/bf_kit.glow.gxtex" "$fixture/models/"

MELEE_SCENE='mode=lab;p1=fox;p2=none;stage=bf;time=0' \
MELEE_PAD_SCRIPT="$fixture/largemap.lua" MELEE_SCRIPTS=0 \
MELEE_PAD_IGNORE_ADAPTER=1 MELEE_INPUT=none \
MELEE_SCRIPT_MS=1000 MELEE_TURBO_DRAWS=1 MELEE_TURBO_RENDER=8 \
bash tools/port/run.sh --test largemap --iso "$GW_ISO_VANILLA"
```

Use Windows-style absolute paths for `GW_MELEE`, `GW_BUILD_ROOT` and therefore
`MELEE_PAD_SCRIPT`. The generous per-call script wall-time budget accommodates
the capacity/target setup and logging on slower batch hosts; traversal is
frame-driven. In a combined batch driver, preserve the LAB scene, fixtures,
pad ownership, error watchdog and assertions, and let the driver perform the
single final quit. A pass requires the terminal `complete; 0 failures` line
and no `FAIL` lines, not merely process exit zero.

## Verification and what remains untested

- `luac -p pc/tests/largemap.lua`: passed.
- `git diff --check`: passed (Git emitted line-ending warnings).
- Attempted the documented PowerPC `clang -fsyntax-only` command on
  `pc/gameworld/script_game.c`: execution returned **permission denied** for
  `../../_toolchains/llvm/bin/clang.exe`. No escalation was requested. C syntax
  is **unverified**, including the native Windows TU and other changed game TUs.
- Independent source review found far-plane restoration and reusable-owner
  issues; both were corrected.
- No build, bridge/link validation, game run, rendered-image inspection,
  performance measurement, or runtime test pass is claimed. The batch lane must
  validate snapshot restoration, actual heap drift, extended camera/blast
  behaviour, all capacity checks, and integration with the other branches.
- Rewind exactness across script mutations, online interoperability, other
  stages' offset animation, every camera mode, maximum target memory use,
  unique-asset cache exhaustion and scene transitions have not been exercised.

Command evidence: `.omo/evidence/task-largemap.log`.
