# Roguelite room certification probe

`main.lua` is the game-side half of `tools/roguelite/certify_rooms.py`. It is not
loaded on its own: the Python installer prepends the reviewed pure modules
(`RoomCatalogue`, `RoomRecipes`, `Rooms`) exactly as
`tools/roguelite/prepare.py` does, then installs the bundle as an isolated,
gameplay-enabled script mod. It exists so the coordinator can inspect and drive
an **uncertified** recipe without touching production certification or the
production adapter.

## What it does

1. `certify_recipe <template>` resolves a template through
   `RoomRecipes.resolve` directly (no certification gate) and prints the recipe
   version, `certified` flag, sockets, anchors and floor openings.
2. `certify_build <template>` preloads the BF kit one model per tick
   (`Rooms.preload_step`), spawns the reviewed visual layout (`Rooms.enter`),
   builds explicit collision from `Rooms.collision` (segmented floors via
   `gd.stage_add_platform`, slopes via `gd.stage_add_line`), then acquires FD
   isolation (`gd.stage_isolate(true)`). It never spawns actors or encounters.
3. `certify_place` / `certify_place_at` teleport the fighter as a **labelled
   fixture** (`fixture=true`) for initial placement and inspection only.
4. `certify_arm` / `certify_result` record a bounded per-frame trace
   (`x`, `y`, `vx`, `vy`, `airborne`, `action`) for the Python driver to
   classify arrival, fall, stuck and seam-pop candidates. At most 900 samples.
5. `certify_cleanup` removes every collider, releases the visual assets and
   restores the original stage.

## What it never does

- It never sets `recipe.certified`.
- It never calls the production `Adapter`, so it cannot make an uncertified
  recipe admissible in the live runtime.
- It never sends input and never owns a pad; the Python driver replays ordinary
  controller samples at normal 60 Hz through the real console `input` command.
- A teleport is never traversal evidence.

## Console commands

| command | purpose |
| --- | --- |
| `certify_status` | phase, template, recipe version, certified flag, colliders, samples, error |
| `certify_apis` | presence/absence of each required native API |
| `certify_recipe <template>` | resolved uncertified recipe rows and sockets |
| `certify_build <template>` | load, build and isolate the recipe |
| `certify_place <socket>` | fixture teleport to a socket arrival (inspection only) |
| `certify_place_at <x> <y>` | fixture teleport to a coordinate (inspection only) |
| `certify_arm <socket> <label>` | open a bounded trace window |
| `certify_result` | close the window and emit the summary plus first 200 trace rows |
| `certify_trace <offset> <count>` | page further trace rows |
| `certify_sample` | read port 1 once |
| `certify_bounds` | stage blast/camera/main-floor bounds when `gd.stage_bounds` exists |
| `certify_cleanup` | remove colliders, visuals and isolation |
| `certify_scene <fighter> [costume]` | launch the isolated FD match for a mobility profile |
| `certify_hud on\|off` | hide/restore the native status HUD |

## Coordinator prerequisites

- A native build with the current API (`gd.model_load`/`model_spawn`/
  `model_despawn`/`model_release`, `gd.stage_add_platform`, `gd.stage_add_line`,
  `gd.stage_isolate`), started at **normal speed** with `MELEE_CONSOLE_PORT` and
  an NTSC 1.02 disc. Turbo/uncapped runs are refused by the driver.
- Install into a dedicated app directory, never the shared review directory:
  `python3 tools/roguelite/certify_rooms.py install --app-dir <isolated-dir>`.

## Limitations

Collision placement, slope seams, body clearance, camera/blast margins and
seam-pop timing can only be judged from an actual normal-speed run and captures.
This probe produces observations; a human reviews them. No output here marks a
recipe certified.
