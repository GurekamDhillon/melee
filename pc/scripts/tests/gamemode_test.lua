-- Host Lua only: no game process, no assets, no native code executed.
local blob, serial, frame = nil, 0, 0
local removed, releases, holds = {}, 0, 0
local player = {x = 0, y = 0, dead = false}
local fail_spawn = false
local function handle() serial = serial + 1; return serial end
gd = {
    mode_blob = function(...)
        if select('#', ...) > 0 then blob = ... end
        return blob
    end,
    match = function() return {frame = frame, netplay = false} end,
    player = function() return player end,
    log = function() end,
    model_load = handle, model_release = function() end,
    model_spawn = handle, model_despawn = function(h) removed[h] = true end,
    spawn_enemy = function() if fail_spawn then return nil, 'full' end; return handle() end,
    enemy_remove = function(h) removed[h] = true end,
    spawn_target = handle, stage_remove = function(h) removed[h] = true end,
    camera_set = function() end, camera_attach = function() end, camera_detach = function() end,
    teleport = function(_, x, y) player.x, player.y = x, y; return true end,
    boss_hold = function() holds = holds + 1; return true end,
    boss_release = function() releases = releases + 1; return true end,
    fill = function(_, _, _, _, rgba) assert(rgba >= 0 and rgba <= 255) end,
    kit = {
        available = function() return true end,
        panel = function() end,
        text = function(_, _, _, role, color)
            assert(role == 'label' and (color == 'gold' or color == 'bone'))
        end,
    },
}
dofile('pc/scripts/lib/gamemode.lua')
local spec = {
    id = 'test', version = 1, start = 'a', fade_frames = 2,
    areas = {
        a = {entries = {start = {x = 0, y = 0}}, checkpoint = true,
             layout = {{model = 'floor', x = 0, y = 0}},
             waves = {{{kind = 'goomba', x = 4, y = 1}}, {{kind = 'redead', x = 5, y = 1}}},
             goals = {{kind = 'defeat_all'}},
             doors = {{box = {8, -2, 12, 2}, to = 'b', entry = 'start'}}},
        b = {entries = {start = {x = 0, y = 0}},
             targets = {{x = 4, y = 1}},
             goals = {{kind = 'break_targets'}, {kind = 'survive', seconds = 0.05}},
             doors = {{box = {8, -2, 12, 2}}}},
    },
}
local mode = Gamemode.new(spec)
local function tick(n)
    for _ = 1, n or 1 do frame = frame + 1; mode:frame() end
end
local function first(t) for k in pairs(t) do return k end end
mode:start()
assert(mode.state.area == 'a' and mode.state.checkpoint.area == 'a')
local initial = blob
local enemy = first(mode.state.enemies)
mode:enemy_defeated({handle = -10})
assert(mode.state.enemies[enemy])
mode:enemy_defeated({handle = enemy})
mode:enemy_defeated({handle = enemy}) -- duplicate must not finish wave 2
tick()
assert(mode.state.wave == 2 and not mode:ready())
mode:enemy_defeated({handle = first(mode.state.enemies)})
tick()
assert(mode:ready())
mode:progress_set('key', 7)
player.x = 10
tick()
assert(mode.state.phase == 'out')
mode:draw()
local fade = blob
tick(2)
assert(mode.state.area == 'b')
tick(2)
assert(mode.state.phase == 'play' and not mode:ready())
mode:target_broken(first(mode.state.targets))
tick(3)
assert(mode:ready())
player.x = 10
tick(5)
assert(mode.state.phase == 'complete')
-- Native save/load restores blob and objects before notifying Lua. No respawns on load.
local serial_before = serial
blob = fade
mode:load()
assert(mode.state.area == 'a' and mode.state.phase == 'out')
assert(serial == serial_before and mode.state.progress.key == 7)
blob = initial
mode:load()
assert(mode.state.area == 'a' and mode.state.enemies[enemy])
assert(mode.state.progress.key == nil)
mode:retry()
tick(4)
assert(mode.state.area == 'a' and mode.state.wave == 1)
-- Errors retire only owned objects and never grant an objective.
fail_spawn = true
mode:retry()
tick(2)
assert(mode.state.phase == 'error' and not mode:ready())
fail_spawn = false
mode:stop()
assert(blob == '')
-- Boss hooks are filtered, held once, and released by a logic-frame timeout.
spec.areas.a.boss = {kind = 'master_hand', port = 3, hold_seconds = 1}
spec.areas.a.goals = {{kind = 'boss'}}
mode:start()
mode:boss_defeated({kind = 'crazy_hand', port = 2})
assert(holds == 0)
mode:boss_defeated({kind = 'master_hand', port = 3})
mode:boss_defeated({kind = 'master_hand', port = 3})
assert(holds == 1 and mode:ready())
tick(60)
assert(releases == 1)
mode:stop()
-- Checkpoint retry restores entry-time progress, not the failed attempt's edits.
spec.areas.a.goals = {{kind = 'reach_exit', box = {-2, -2, 2, 2}}}
spec.areas.a.waves = {}
spec.areas.b.checkpoint = true
mode:start()
assert(not mode:ready())
tick()
assert(mode:ready())
mode:progress_set('coins', 3)
player.x = 10
tick(5)
assert(mode.state.area == 'b')
mode:progress_set('coins', 8)
mode:retry()
tick(4)
assert(mode.state.progress.coins == 3 and mode.state.area == 'b')
-- Blob keys have canonical ordering and strings remain data, including NUL/delimiters.
mode:progress_set('data', {z = 'x\0:t2:', a = true, [7] = -9})
local canonical = blob
mode:progress_set('data', {[7] = -9, a = true, z = 'x\0:t2:'})
assert(blob == canonical)
mode:load()
assert(mode.state.progress.data.z == 'x\0:t2:')
mode:stop()
-- Checkpoint copies can overflow a blob even when a progress update fit alone.
spec.areas.a.boss, spec.areas.a.goals, spec.areas.a.waves = nil, {}, {}
spec.areas.b.checkpoint = true
mode:start()
assert(mode:progress_set('large', string.rep('x', 4500)))
player.x = 10
tick(3)
assert(mode.state.phase == 'error')
mode:load()
assert(mode.state.phase == 'error' and mode.state.area == 'b')
mode:stop()
print('gamemode stub: PASS')
