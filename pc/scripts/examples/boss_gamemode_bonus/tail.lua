-- Source tail for bundle.py (lib/gamemode.lua goes first). Only scripts/main.lua is installed.
-- Classic step 10 (or a VS boss stage): Master Hand falls -> gd.boss_hold -> the demo's rooms are built
-- past the right blast/camera edge (widened with gd.stage_set_*_bounds) -> hall, door, gallery, exit ->
-- camera, bounds and fighter restored, gd.boss_release, normal results. 90 s wedge fallback.
local U, Y = 6.5, 20 -- BF interior kit scale; room floor height
local HOLD_S = 90
local mode, started, done, mh_port, orig, saved, defeat_clock, widened, dx, back, moved
BOSSBONUS_ONE_HP = BOSSBONUS_ONE_HP or false -- the play bundle sets this so a human can land the KO

local function layout(second)
    local parts = {}
    for _, x in ipairs({-6, -2, 2, 6}) do
        parts[#parts + 1] = {model = 'bf_floor_4m', x = x * U + dx, y = Y, floor_flags = 0}
        parts[#parts + 1] = {model = second and 'bf_wall_window_4m' or 'bf_wall_solid_4m',
                            x = x * U + dx, y = Y, collision = false}
    end
    parts[#parts + 1] = {model = 'bf_wall_doorway_4m', x = 6 * U + dx, y = Y, z = 1, collision = false}
    return parts
end

local function build_def(px, py)
    local camera = {eye = {x = dx, y = 52, z = 175}, interest = {x = dx, y = 32, z = 0}, fov = 45}
    return {
        id = 'boss_bonus_rooms', version = 1, start = 'boss', port = 1, fade_frames = 24,
        areas = {
            boss = {
                entries = {start = {x = math.floor(px + 0.5), y = math.floor(py + 0.5)}},
                boss = {kind = 'master_hand', port = mh_port, hold_seconds = HOLD_S},
                goals = {{kind = 'boss'}},
                doors = {{box = {-1e5, -1e5, 1e5, 1e5}, to = 'hall'}},
            },
            hall = {
                title = 'Hall: defeat the wave, then go right', layout = layout(false), camera = camera,
                entries = {start = {x = dx - 38, y = Y + 6}}, checkpoint = true,
                waves = {{{kind = 'goomba', x = dx - 4, y = Y + 6},
                          {kind = 'redead', x = dx + 24, y = Y + 6, facing = -1}}},
                goals = {{kind = 'defeat_all'}, {kind = 'reach_exit'}},
                doors = {{box = {dx + 35, Y - 2, dx + 49, Y + 22}, to = 'gallery'}},
            },
            gallery = {
                title = 'Gallery: break targets, then go right', layout = layout(true), camera = camera,
                entries = {start = {x = dx - 38, y = Y + 6}}, checkpoint = true,
                targets = {{x = dx - 12, y = Y + 14}, {x = dx + 16, y = Y + 22}},
                goals = {{kind = 'break_targets'}, {kind = 'survive', seconds = 3}, {kind = 'reach_exit'}},
                doors = {{box = {dx + 35, Y - 2, dx + 49, Y + 22}}},
            },
        },
    }
end

local function try_start()
    if started or gd.match().netplay or not gd.match().active then return end
    local p = gd.player(1)
    if not p then return end
    for q = 2, 4 do
        local f = gd.player(q)
        if f and f.kind == 0x1B then mh_port = q end
    end
    if not mh_port then return end
    started = true
    orig = gd.stage_bounds()
    if not orig then started = false return end
    local floor_right = orig.main_floor and orig.main_floor.right or 90
    local right = math.max(orig.camera.right, orig.blast.right)
    dx = math.max(right + 60, floor_right + 260) -- rooms span dx +- 52: >= 200 clear of the stage
    mode = Gamemode.new(build_def(p.x, p.y))
    local ok, err = mode:start()
    if not ok then gd.log('boss_gamemode_bonus: start failed: ' .. tostring(err)) mode = nil return end
    if BOSSBONUS_ONE_HP then gd.set_damage(mh_port, 299) end
    gd.log(string.format('boss_gamemode_bonus: armed (Master Hand P%d, rooms at x=%.0f)', mh_port, dx))
end

local function widen()
    widened = true
    gd.stage_set_camera_bounds(orig.camera.left, dx + 90, orig.camera.top, orig.camera.bottom)
    gd.stage_set_blast_bounds(orig.blast.left, dx + 120, orig.blast.top, orig.blast.bottom)
    gd.log('boss_gamemode_bonus: camera/blast bounds widened')
end

local function put_back()
    -- Before the hold is released: the native clear pose refuses a teleport.
    if moved or not saved then return end
    if pcall(gd.teleport, 1, saved.x, saved.y) then moved = true end
end

local function finish(why)
    if done then return end
    done = true
    put_back()
    if mode then pcall(function() mode:stop() end) end -- releases the hold, removes rooms, re-attaches camera
    pcall(gd.boss_release)
    if widened then pcall(gd.stage_restore_bounds); widened = false end
    if saved and not moved then back = {x = saved.x, y = saved.y, tries = 0} end
    gd.log('boss_gamemode_bonus: ' .. why .. '; restored')
end

function on_match_start() started, done, mode, widened, saved, defeat_clock, back, moved = false, false, nil, false, nil, nil, nil, false end
function on_boss_defeated(e)
    if not mode or done or e.kind ~= 'master_hand' then return end
    local p = gd.player(1)
    if p then saved = {x = p.x, y = p.y} end
    mode:boss_defeated(e)
end
function on_frame()
    if not mode then try_start() end
    if back then
        back.tries = back.tries + 1
        local okk, err = pcall(gd.teleport, 1, back.x, back.y)
        if not okk and back.tries % 60 == 1 then gd.log('boss_gamemode_bonus: put-back retry: ' .. tostring(err)) end
        if okk or back.tries > 300 then back = nil end
    end
    if not mode or done then return end
    mode:frame()
    local s = mode.state
    if not s then return end
    -- last frames of the final fade-out: hold is still on, so the fighter can be moved home
    if s.phase == 'out' and s.destination == '' and s.fade_left <= 2 then put_back() end
    if s.boss_done and not defeat_clock then defeat_clock = s.clock end
    if s.area ~= 'boss' and not widened then widen() end
    if s.phase == 'complete' then finish('sequence complete')
    elseif s.phase == 'error' then finish('mode error')
    elseif defeat_clock and s.clock - defeat_clock > HOLD_S * 60 then finish('90 s fallback') end
end
function on_draw() if mode and not done then mode:draw() end end
function on_enemy_defeated(e) if mode then mode:enemy_defeated(e) end end
function on_enemy_removed(e) if mode then mode:enemy_removed(e) end end
function on_target_broken(h) if mode then mode:target_broken(h) end end
function on_loadstate() if mode then mode:load() end end
function on_match_end()
    if mode then mode:stop(true) end
    if widened then pcall(gd.stage_restore_bounds) end
    mode, widened = nil, false
end
function on_unload()
    if mode then mode:stop() end
    if widened then pcall(gd.stage_restore_bounds) end
end
