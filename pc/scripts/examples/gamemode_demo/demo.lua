-- Source tail for bundle.py. Only scripts/main.lua is installed/executed.
local U, Y = 6.5, 20 -- existing BF interior kit export scale; room above FD
local function layout(second)
    local parts = {}
    for _, x in ipairs({-6, -2, 2, 6}) do
        parts[#parts + 1] = {model = 'bf_floor_4m', x = x * U, y = Y, floor_flags = 0}
        parts[#parts + 1] = {model = second and 'bf_wall_window_4m' or 'bf_wall_solid_4m',
                            x = x * U, y = Y, collision = false}
    end
    parts[#parts + 1] = {model = 'bf_wall_doorway_4m', x = 6 * U, y = Y, z = 1, collision = false}
    return parts
end
local camera = {eye = {x = 0, y = 52, z = 175}, interest = {x = 0, y = 32, z = 0}, fov = 45}
local mode = Gamemode.new{
    id = 'two_rooms', version = 1, start = 'hall', port = 1, fade_frames = 24,
    areas = {
        hall = {
            title = 'Hall: defeat the wave, then go right', layout = layout(false), camera = camera,
            entries = {start = {x = -38, y = Y + 6}}, checkpoint = true,
            waves = {{{kind = 'goomba', x = -4, y = Y + 6},
                      {kind = 'redead', x = 24, y = Y + 6, facing = -1}}},
            goals = {{kind = 'defeat_all'}, {kind = 'reach_exit'}},
            doors = {{box = {35, Y - 2, 49, Y + 22}, to = 'gallery'}},
        },
        gallery = {
            title = 'Gallery: break targets, then go right', layout = layout(true), camera = camera,
            entries = {start = {x = -38, y = Y + 6}}, checkpoint = true,
            targets = {{x = -12, y = Y + 14}, {x = 16, y = Y + 22}},
            goals = {{kind = 'break_targets'}, {kind = 'survive', seconds = 3}, {kind = 'reach_exit'}},
            doors = {{box = {35, Y - 2, 49, Y + 22}}},
        },
    },
}

function on_match_start()
    if gd.match().netplay then gd.log('gamemode demo: offline only'); return end
    -- This authored layout is inside FD's existing bounds, not an arbitrary-stage claim.
    if gd.match().stage ~= 0x25 then gd.log('gamemode demo: use Final Destination'); return end
    mode:start()
end
function on_frame() mode:frame() end
function on_draw() mode:draw() end
function on_enemy_defeated(e) mode:enemy_defeated(e) end
function on_enemy_removed(e) mode:enemy_removed(e) end
function on_target_broken(h) mode:target_broken(h) end
function on_boss_defeated(e) mode:boss_defeated(e) end
function on_loadstate() mode:load() end
function on_match_end() mode:stop(true) end
function on_unload() mode:stop() end

gd.command('gm_retry', function() mode:retry() end, 'Restart the latest checkpoint (live P1 required)')
gd.command('gm_stop', function() mode:stop() end, 'Remove the demo and release camera/boss hold')
gd.command('gm_status', function()
    local s = mode.state
    if not s then gd.log('gamemode demo: idle'); return end
    local enemies, targets = 0, 0
    for _ in pairs(s.enemies) do enemies = enemies + 1 end
    for _ in pairs(s.targets) do targets = targets + 1 end
    gd.log(string.format('gamemode status: area=%s phase=%s wave=%d enemies=%d targets=%d elapsed=%d ready=%s checkpoint=%s',
        s.area, s.phase, s.wave, enemies, targets, s.elapsed, tostring(mode:ready()),
        s.checkpoint and s.checkpoint.area or 'none'))
end, 'Inspect snapshotted mode state')
