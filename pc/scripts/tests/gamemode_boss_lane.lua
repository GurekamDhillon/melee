-- Optional in-game integration fixture; bundle AFTER lib/gamemode.lua.
-- This is not the two-room demo and has not been run in the game.
local mode
function on_match_start()
    if gd.match().netplay then return end
    local p = gd.player(1)
    if not p then return end
    mode = Gamemode.new{
        id = 'boss_lane', version = 1, start = 'boss', fade_frames = 20,
        areas = {
            boss = {
                entries = {start = {x = p.x, y = p.y}},
                boss = {kind = 'master_hand', port = 3, hold_seconds = 20},
                goals = {{kind = 'boss'}},
                doors = {{box = {-150, -100, 150, 150}, to = 'bonus'}},
            },
            bonus = {
                entries = {start = {x = -42, y = 16}},
                camera = {eye = {x = 0, y = 45, z = 180},
                          interest = {x = 0, y = 25, z = 0}, fov = 45},
                targets = {{x = -30, y = 23}, {x = 0, y = 32}, {x = 30, y = 23}},
                goals = {{kind = 'break_targets'}},
                doors = {{box = {-150, -100, 150, 150}}},
            },
        },
    }
    mode:start()
end
function on_frame() if mode then mode:frame() end end
function on_draw() if mode then mode:draw() end end
function on_boss_defeated(e) if mode then mode:boss_defeated(e) end end
function on_target_broken(h) if mode then mode:target_broken(h) end end
function on_loadstate() if mode then mode:load() end end
function on_match_end() if mode then mode:stop(true) end end
function on_unload() if mode then mode:stop() end end
gd.command('gmb_hit', function()
    assert(mode and mode.state and mode.state.area == 'boss', 'start the Classic final fight first')
    gd.set_damage(3, 999)
    assert(gd.hit(3, {damage = 1, angle = 45, kbg = 0, bkb = 0, from = 1}), 'Master Hand not live')
end, 'Final blow through real fighter damage; wait until the Hand is active')
gd.command('gmb_status', function()
    if not mode or not mode.state then gd.log('boss lane idle'); return end
    local s, targets = mode.state, 0
    for _ in pairs(s.targets) do targets = targets + 1 end
    gd.log(string.format('boss lane: area=%s phase=%s holding=%s targets=%d',
        s.area, s.phase, tostring(s.holding), targets))
end, 'Inspect the boss integration fixture')
