# Gamemode v1

An **offline, same-match** director for reusable room experiments. It does not choose a
campaign, install a native menu mode, launch scenes or replace the underlying match rules.
Use an offline gameplay manifest with `rollback_safe: false`. One director per script;
the director owns the camera while active. Compose the library before your script entry
as demonstrated by `../examples/gamemode_demo/bundle.py`; `require` is absent from the sandbox.

## Definitions

```lua
local mode = Gamemode.new{
    id = 'my_route', version = 1, start = 'room', port = 1, fade_frames = 20,
    areas = {
        room = {
            title = 'An experiment',
            entries = {start = {x = -30, y = 26}},
            layout = {{model = 'bf_floor_4m', x = 0, y = 20, floor_flags = 0}},
            camera = {eye = {x = 0, y = 50, z = 175},
                      interest = {x = 0, y = 30, z = 0}, fov = 45},
            checkpoint = true,
            waves = {
                {{kind = 'goomba', x = 8, y = 26, facing = -1}},
                {{kind = 'redead', x = 12, y = 26}},
            },
            goals = {{kind = 'defeat_all'}, {kind = 'reach_exit'}},
            doors = {{box = {20, 18, 35, 42}}}, -- no `to`: complete the route
        },
    },
}
```

Keep definitions immutable once constructed; bump the version after changing their meaning.
IDs and entry names are at most 64 bytes. Every area has `entries.start`; a door may select
another entry with `entry = 'name'`. Arrays must be dense and ordered. Overlapping doors use
the first matching definition. Model placements accept `x/y/z`, `scale`, `rot`, `collision`
and `floor_flags` with the existing model API meanings. Coordinates are game world units;
there is no automatic fit, bounds change, scaling or collision generation. Sidecars supply
collision. At most 128 models and 32 targets per room, further limited by native/global pools.

`waves` is an ordered list of enemy lists (1–32 per wave). A subsequent wave starts on the
next logic-frame hook after all **owned** enemies emit defeat events. Unrelated/duplicate
events do nothing. Explicit removal is not defeat; silently despawning a mode enemy elsewhere
can leave its objective blocked. Koopa's shell transition is not a defeat event. Spawn failure
puts the director in `error` and cleans its owned objects, rather than treating the wave as won.

Goals are ANDed; empty goals mean doors are immediately eligible:

| Goal | Satisfied when |
|---|---|
| `reach_exit` | An eligible door is entered; with `box={x0,y0,x1,y1}`, that region must first be reached |
| `defeat_all` | Every authored wave was spawned and all its owned enemies defeated |
| `break_targets` | Every target in `targets={{x=,y=},...}` was broken |
| `survive`, `seconds=N` | `ceil(N*60)` active `play` frames elapsed, excluding fades and pause |
| `boss` | The configured boss kind/port emitted its defeat hook and the native hold was accepted |

The survival goal is a room timer, not an automatic life-loss policy. The owning mod decides
whether death calls `retry()`, ends the route, or waits for the underlying match to respawn.

## Hooks and methods

Forward `on_match_start` to `start()`, `on_frame` to `frame()`, `on_draw` to `draw()`,
`on_enemy_defeated(e)` to `enemy_defeated(e)`, `on_target_broken(h)` to `target_broken(h)`,
`on_boss_defeated(e)` to `boss_defeated(e)`, and `on_loadstate` to `load()`.
Forward `on_match_end` to `stop(true)` (native teardown owns resources) and `on_unload` to
`stop()`. Never advance the director from `on_tick`/`on_draw`. Do not also forward
`on_all_targets_broken`: that hook covers every script's targets, not this area's ownership.

- `start()` allocates state, constructs the start area and moves the live player to its entry.
- `ready()` reports all current goals, before door contact.
- `retry()` fades to the latest checkpoint and rebuilds the encounter from its beginning.
  Checkpoints save the area, entry, progress and cleared-room table at **entry**. They do not
  restore health, stock, inventory, elapsed goal time or partially defeated waves.
- `progress_set(key, value)` sets/removes a progress field and writes its snapshot immediately.
  Values are integers, strings, booleans or nested acyclic tables with string/integer keys.
  Nil removes a field. Floats, functions, userdata and cycles are rejected. Depth is <16.
- `load()` replaces the Lua state from the native snapshot, without rebuilding objects or
  advancing time; it reclaims camera ownership at the restored pose if the room has a camera.
- `stop()` removes only owned objects and clears mode storage; a failed cleanup leaves an
  error state for inspection/retry. `stop(true)` is exclusively for scene teardown.

Read `mode.state` for diagnostics; do not mutate it directly. Public mutation methods return
`false, error` on a handled error. Definition/load validation errors raise Lua errors. The kit
HUD reports the room, exit readiness, completion or error; `gd.fill` supplies the fade. Doors
fade out, swap layout/enemies/targets, relocate the fighter, set the room camera, then fade in.
Physics continues during fades. Entry placement uses `gd.teleport`, resetting live movement;
it can reject dead, held or respawning fighters. It is not a new fighter-spawn/revive API.
On rejection, wait for a live fighter and retry. Completion leaves the room visible until stop
or scene teardown, attaches the normal camera and releases any boss hold.

## Boss continuation

An area may specify:

```lua
boss = {kind = 'master_hand', port = 3, hold_seconds = 20},
goals = {{kind = 'boss'}},
doors = {{box = {-60, -10, 60, 50}, to = 'bonus', entry = 'start'}},
```

Filter both kind and one-based port; Giga Bowser is `fighter_31`. The native boss hook must
already exist in the seeded encounter. This framework does **not** spawn bosses or configure
Classic's controller. A matching defeat requests a native hold of `hold_seconds + 1` as a
backstop and records its own logic-frame deadline. The hold can span a door into a post-boss
room; completion, retry, stop, error or deadline releases it. It is deliberately finite
(1/60–599 seconds); the ordinary controller may finish if the bonus exceeds the deadline.
Only one script should own the engine's global boss hold.

For an offline test of an already-present fighter boss, `gd.hit` uses the real damage/KO
path, e.g. `gd.hit(3, {damage=500, angle=45, kbg=0, bkb=0, from=1})`. It does not damage
Adventure item enemies. Test actual native events; calling a Lua hook manually is not an
integration test. See the report's boss lane.

## Snapshot contract and native API

`gd.mode_blob()` returns this script's bytes or nil. `gd.mode_blob(bytes)` replaces them;
`gd.mode_blob('')` clears them. The API is limited to gameplay scripts in an active offline
match (no console), even with `rollback_safe: true`. It stores up to 8,192 bytes in each of
16 source/id-hash-keyed slots in script_game.c's snapshotted BSS (~128 KiB total). Native
code transfers scalar words; pointers and host Lua memory never cross the boundary. Each
write validates before mutation, zeroes a shrinking payload's tail, and forks the timeline.
Unloading clears that source's live blob; stage teardown clears the entire store.

The library serializes progress, checkpoint, wave/target ownership, model handles, phase,
timers and hold state in a canonical length-prefixed data format. No code is evaluated on
decode. It writes after every logic step and relevant event; `on_savestate` runs too late to
be a reliable capture hook and is not used. The native store goes into quick slots and
persistent `.gdst` snapshots with the world, **only within the original live scene/process**.
Native mesh/atlas caches are scene-pinned and cannot be reconstructed by a file load after
scene teardown or restart; a host-only session cookie and scene epoch reject that case.
Mode-bearing states require the same set of
active gameplay source/version hashes before loading; incompatible quick slots and files
are rejected before restoring their world objects. No cross-scene/cross-process campaign persistence.

Keep progress small: a checkpoint copies it, and handles also consume space. If state growth
overflows the blob, cleanup enters an error state. If even that error state is too large,
the fallback explicitly discards progress/checkpoint/cleared history and stores the bounded
error plus remaining owned handles. A log records that loss; it never reports a completed goal.

**Rollback awareness is refusal, not online support.** Lua hooks do not execute in rollback
or rewind resimulation. Activating mode storage retires prior history and disables its
recording; `history(n>0)`, `step_back`, `rewind_to`, `rewind_test` and `hot_reload` refuse
while it is active. Loading a direct mode-bearing state also retires history. After stop,
history can be explicitly enabled again. Netplay must not use this director. Achieving replayable
online flow requires a native director or an engine facility to log/replay all mode mutations.

## Validation

`lua pc/scripts/tests/gamemode_test.lua` exercises state-machine behavior with stubs only.
`luac -p` checks the library, demo, generated entry and tests. Native `script_mode_blob` is
registered for the Windows headless suite but was not run in this code-only task. The report
lists the exact lane and everything that remains untested.
