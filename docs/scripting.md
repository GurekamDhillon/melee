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
Its collision remains that one stage line; use `gd.model_spawn` to instantiate
the model's complete collision sidecar.

The mesh's top is aligned with the collision line. Its X axis scales to the
line's current width, including after `gd.stage_move`. Authored depth and
thickness stay fixed. Each modeled platform uses one indexed GX triangle draw
with the baked colour atlas. The atlas contains the source material emission;
the optional `.glow.gxtex` adds unlit emission in a second TEV stage. Model
selection and asset storage are native and do not enter gameplay or rollback
snapshots.
Assets load once per scene; start a new scene after replacing an exported
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
UVs, and triangle indices. The separate glow texture is added by the current renderer. At the default scale, 8 Blender metres produce 40 game world units
of width; the Lua platform can choose any collision width.

The example mod is [pc/scripts/examples/bf_platform](../pc/scripts/examples/bf_platform/).
Copy that folder to `scripts/bf_platform` beside `melee-pc.exe`, then enable
the mod. It puts three modelled platforms on Final Destination in an offline
match. The port log should contain `script stage: model ... loaded`,
three `script stage: model 1 attached` lines, and
`script stage: model 1 first world draw`.

# Runtime models

`gd.model_load(path)` loads a GXMS mesh, its GXTX colour atlas, an optional glow
atlas, and an optional collision sidecar. It returns an asset handle. Load in
`on_match_start` or during gameplay, then spawn any number of instances:

```lua
local deck = gd.model_load("stage_kit/models/deck")
local part = assert(gd.model_spawn(deck, {x = 0, y = 40, z = 0,
                                        rot = 0, scale = 1, layer = 0}))
gd.model_release(deck) -- the instance retains its asset
gd.model_move(part, 25, 45, 0)
gd.model_set(part, {visible = true, tint = 0x80C0FFFF})
-- gd.model_despawn(part)
```

These are offline gameplay APIs. A script needs `gameplay: true` in its manifest
(whether installed as a script or shipped by a mod) and an active gameplay scene.
They have no stage-ID or VS/LAB/1P/Classic/Adventure/event mode filter. The console
cannot create or change models. Every call is refused during netplay or rollback
sessions, even for a script marked `rollback_safe`. Online asset agreement and
deterministic script replay have not been validated.

## Paths and ownership

A qualified path is `mounted-mod-id/relative/path/basename`, without `.gxmesh`.
It resolves inside that active mod's payload directory: `<mod>/files/` when
present, otherwise the legacy mod root. Any mounted mod can provide the model.
Only letters, digits, `_`, `-`, and single `/` separators are accepted; no drive
names, `..`, backslashes, empty components, or extensions. The full argument is
at most 180 bytes; its basename is at most 48 bytes. Windows paths must fit
`MAX_PATH`. Mod IDs must use the same character set to be addressed by this API.

A plain basename, such as `gd.model_load("deck")`, uses the existing `model=`
lookup: `models/` beside the calling script, then one directory above it. This
also works for script mods installed under `scripts/`, without a DVD payload.

The cache uses case-insensitive absolute paths. Loading the same path returns
the same handle and increments its load reference count. Each successful spawn
owns another reference. `gd.model_release(handle)` releases one load reference;
it does not despawn instances. Release each successful load once. A released
handle cannot spawn again until another load retains it. Excess releases error.
Despawn releases the instance's reference. All references are cleared at scene end.

Immutable asset slots stay pinned until scene end even with zero references:
an older savestate may still reference them. This is deliberate, bounded retention,
not an eviction cache. A full cache reports `model cache full (32 scene-pinned
assets); start a new scene`. New scenes free mesh/atlas bytes and invalidate all
old handles. Reload files by starting a new scene. Do not retain handles across
scene transitions.

## Instances

| Call | Result |
|---|---|
| `gd.model_spawn(asset, options)` | Instance handle, or `nil, reason` for unavailable renderer, instance/line capacity, or an invalid collision transform |
| `gd.model_move(instance, x, y [, z])` | Moves the model and all owned collision lines; omitted Z is preserved |
| `gd.model_move(instance, options)` | Updates any instance fields, like `model_set` |
| `gd.model_set(instance, options)` | Updates supplied fields atomically; returns `true` |
| `gd.model_get(instance)` | Current snapshotted fields, or `nil` for a stale instance |
| `gd.model_despawn(instance)` | Removes the instance and its collision; `true`, or `false` if already absent |
| `gd.model_release(asset)` | Releases one load reference; no return value |

Options are `x`, `y`, `z` (default 0), `rot` (degrees about +Z, default 0),
`scale` (uniform, default 1), `layer` (integer -8..8, default 0), `visible`
(boolean, default true), and `tint` (packed `0xRRGGBBAA`, default `0xFFFFFFFF`).
Fields are raw table entries; extra fields are ignored so data-table rows can
include script metadata. Coordinates must be finite and within ±100000; scale
is 0.001..100 and rotation is -360..360. Invalid options and stale mutation
handles raise Lua errors. A rejected transform leaves the instance unchanged.

The mesh uses its authored origin and dimensions, with no automatic centering
or fit-to-floor scaling. X/Y are the collision plane; Z is visual depth only.
Collision transforms are `world = translation + scale * rotationZ * local`.
Changing visibility or tint does not remove collision. Despawn to remove it.
Owned sidecar lines cannot be individually moved or removed through `gd.stage_*`;
use the instance APIs so the model and collision remain together.

There are 128 instance slots and 32 scene-pinned assets, shared with the older
stage `model=` cache. Collision shares the existing stage pool: at most 200
lines in total, reduced by the map's available vertex/line/joint room. Visual
instances with no collision still work when that pool is unavailable. Collision
spawns reserve all required lines or fail without leaving a partial assembly.
Offline stages reserve the collision headroom and world renderer at stage load,
even before a gameplay script is loaded, so scripts loaded mid-match can use them.

Instance handles, transform, tint, visibility, layer, local collision definitions,
line ownership, and reference counts live in `script_game.c` game memory, already
registered in the snapshot set. Restoring an in-scene snapshot restores these
fields, including despawned objects, and the scene pin preserves their assets.
Lua locals are not snapshotted; use `model_get` for current state and account for
handles from discarded futures. This is snapshot storage coverage, not a claim
of tested rewind/resimulation parity. Cross-scene/cross-process asset resurrection
is not supported. No online/rollback-safety claim is made.

## Collision sidecar v1 (exporter contract)

Beside `deck.gxmesh`, write `deck.coll.json` as UTF-8 JSON without a BOM:

```json
{
  "version": 1,
  "atlas": "kit",
  "lines": [
    ["floor", -20, 0, 20, 0, 3],
    ["floor", 20, 0, 40, 8, 0],
    ["left_wall", -20, -12, -20, 0, 0],
    ["right_wall", 40, 8, 40, -12, 0],
    ["ceiling", 40, -12, -20, -12, 0]
  ]
}
```

`version` and `lines` are required; `atlas` is optional. Each line is exactly
`[kind, x0, y0, x1, y1, flags]`, in the **same local game units and origin as the
mesh**, before instance scale/rotation. Flags are integer 0..3: bit 0 (`1`)
makes a floor pass-through; bit 1 (`2`) enables the existing scripted floor
ledge flag. Use `3` for a droppable platform with ledges. A sloped floor is just
a floor with differing endpoint Y values. A horizontal platform is a floor
with equal Y values; there is no separate platform record.

Floors must run left to right, ceilings right to left, `left_wall` bottom to
top, and `right_wall` top to bottom. Only floors may have nonzero flags. These
directions must still hold after a transform: turning a floor upside down is
rejected, rather than changing its collision kind. Endpoint coordinates must
remain within ±100000 in local and world space. Segments are independent stage
lines; sidecars do not weld islands, define arbitrary polygon solids, or author
per-endpoint ledge metadata. The ledge behavior is exactly that of existing
`gd.stage_add_line(..., {ledges=true})`.

At most 32 lines per model; sidecar size at most 16 KiB. Empty `lines` is valid.
A missing sidecar means visual-only and the default atlas name. Invalid JSON,
unknown or repeated object fields, unsupported version, invalid direction,
invalid flag, or excess lines rejects the entire load. Field order is arbitrary.
Numbers accept JSON decimal/exponent notation; version and flags are integers.
Strings use the literal ASCII names shown here, without JSON escapes.

`atlas` selects `<atlas>.gxtex` and optional `<atlas>.glow.gxtex` in the mesh's
directory. It is a basename of up to 48 letters/digits/underscores/hyphens.
Omit it to use the model basename. Different parts referencing the same atlas
path share texture bytes and GX texture objects. Export parts with common UVs
against that atlas and `"atlas": "kit"`; a visual-only part can specify
`{"version":1,"atlas":"kit","lines":[]}`.

## Mesh, textures, and draw

GXMS uses a 36-byte big-endian header:

| Offset | Type | Meaning |
|---|---|---|
| 0 | 4 bytes | `GXMS` |
| 4 | u32 | Version 1 or 2; export v2 for lighting |
| 8, 12 | u32 each | Vertex count, index count |
| 16, 20, 24 | f32 each | Positive authored width (X), depth (Z), thickness/height (Y) |
| 28, 32 | u32 each | Vertex offset (36), index offset |

Version 2 vertices are eight BE float32 values `(x,y,z,u,v,nx,ny,nz)`, 32 bytes
each; version 1 omits normals and remains unlit. Export split corner normals
at hard edges. Indices are BE u16 triangle indices. Vertex and index counts
are each 3..65535; the index count is divisible by 3. Arrays are contiguous,
without trailing data; every index and float is validated. Mesh files are
limited to 4 MiB by the shared file reader.

GXTX reuses the existing v1 64-byte BE header and tiled RGBA8 payload. Dimensions
are 4..4096 and divisible by 4. Mip levels are consecutive payloads, halving
dimensions while both are at least 4; each included level must fit the GX RGBA8
tile layout. The loader validates a complete byte count for the supplied chain;
the shader uses trilinear filtering when multiple levels exist. For exported
mip chains use power-of-two dimensions. The optional glow atlas uses the same
format and normalized UVs; malformed glow rejects the load.

The existing indexed GX renderer provides a fixed directional light plus ambient
for v2 normals, atlas mips, additive unlit glow, and final RGBA tint. Each visible
instance issues one triangle draw, even with glow. Within each layer, opaque
instances group by asset and reuse material setup; atlas storage is shared
across parts. Matrices remain per-instance: this GX path has no hardware
instancing. Alpha-tinted instances follow opaque instances in slot order within
the layer. Layer controls submission order, with ordinary depth testing; it
does not override depth or perform automatic transparent depth sorting.
`gd.stage_view(false)` hides runtime models along with scripted stage geometry.

## Example and checks

[runtime_models](../pc/scripts/examples/runtime_models/) assembles parts from a
Lua data table and moves one using its snapshotted position. Run its
`make_models.py` once to generate original placeholder GXMS v2 parts, sidecars,
and a shared atlas; replace those assets with the Blender kit exports as desired.
Install the folder as `scripts/runtime_models` beside the executable and enable it.
It deliberately has no map or mode filter; placement is a demonstration and may
overlap a particular map's existing geometry.

Headless API/validation test: workspace `run.sh --test script_model_api`.
Standalone C tests: `pc/tests/model_format_test.c` and
`pc/tests/model_instance_test.c` (compile as ordinary host C; neither runs Melee).
The instance harness uses fake collision calls, so it tests transactions and
snapshot data, not fighter contact behavior. See [model-api-report.md](../model-api-report.md)
for what was checked and the required stage/mode/savestate lane test plan.
