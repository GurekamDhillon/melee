-- Append after the real demo with bundle.py --test pc/tests/gamemode_demo.lua.
-- Same Lua chunk: reads the real local `mode`, never fabricates a terminal hook.
-- Offline VS, Fox P1, idle human P2, FD, time=0; install the kit assets as usual.
-- Bounded attack attempts and a tick deadline guarantee PASS/FAIL followed by gd.quit().
local gmtest_done, gmtest_ticks = false, 0
local gmtest_hall, gmtest_terminal = {}, {}
local gmtest_start = on_match_start
function on_match_start()
    gmtest_start()
    if mode.state then
        for h in pairs(mode.state.enemies) do gmtest_hall[#gmtest_hall + 1] = h end
    end
end
local gmtest_defeated, gmtest_removed = on_enemy_defeated, on_enemy_removed
local function gmtest_record(e)
    local t = gmtest_terminal[e.handle] or {count = 0}
    t.count, t.reason = t.count + 1, e.reason
    gmtest_terminal[e.handle] = t
end
function on_enemy_defeated(e) gmtest_record(e); gmtest_defeated(e) end
function on_enemy_removed(e) gmtest_record(e); gmtest_removed(e) end
local function gmtest_finish(ok, why)
    if gmtest_done then return end
    gmtest_done = true
    gd.log('GMD ' .. (ok and 'PASS ' or 'FAIL ') .. why)
    gd.quit()
end
function on_tick()
    gmtest_ticks = gmtest_ticks + 1
    if gmtest_ticks > 20000 then gmtest_finish(false, 'deadline before route completion') end
end
local function gmtest_items(targets)
    local out = {}
    for _, it in ipairs(gd.items() or {}) do
        if (targets and it.kind == 0xD1) or
           (not targets and (it.kind == 0x2B or it.kind == 0x2C)) then
            out[#out + 1] = it
        end
    end
    return out
end
local function gmtest_tp(x, y)
    for _ = 1, 40 do
        if pcall(gd.teleport, 1, x, y) then return end
        gd.wait(15)
    end
    error('teleport never became available')
end
local function gmtest_hold(spec, n) gd.input(1, spec, n); gd.wait(n) end
-- Attack: fly into the target with a held hitbox (gd.hold_hitbox re-hits every 8 frames).
local function gmtest_swing(x, y)
    if not gd.fly(1) then gd.fly(1, true) end
    if not gd.hold_hitbox(1) then gd.hold_hitbox(1, true, {action = 'nair'}) end
    gmtest_tp(x, y)
    gd.wait(24)
end
gd.run(function()
    local ok, why = pcall(function()
        assert(gd.wait_until(function()
            return gd.match().active and gd.match().frame > 150
        end, 3000), 'match did not start')
        assert(mode.state and mode.state.area == 'hall' and mode.state.phase == 'play', 'hall did not start')
        assert(#gmtest_hall == 2, 'hall did not spawn both enemies')
        for _ = 1, 40 do
            local enemies = gmtest_items(false)
            if #enemies == 0 then break end
            gmtest_swing(enemies[1].x, enemies[1].y)
        end
        assert(gd.wait_until(function() return mode:ready() end, 180), 'hall goal stranded')
        do
            local _, hits = gd.hold_hitbox(1)
            gd.log('GMD hold_hitbox rehit intervals: ' .. tostring(hits))
            assert(hits >= 5, 'held hitbox re-armed only ' .. tostring(hits) .. ' times')
        end
        for _, h in ipairs(gmtest_hall) do
            assert(gd.enemy_status(h) ~= 'alive', 'hall cleared with live owned enemy')
            local terminal = gmtest_terminal[h]
            assert(terminal and terminal.count == 1 and terminal.reason,
                   'hall enemy missing terminal event/reason, or delivered twice')
        end
        gd.log('GMD PASS hall goal cleared')
        gmtest_tp(42, 26)
        assert(gd.wait_until(function()
            return mode.state.area == 'gallery' and mode.state.phase == 'play'
        end, 240), 'hall door never entered gallery')
        assert(#gmtest_items(true) == 2, 'gallery did not spawn both targets')
        for _ = 1, 30 do
            local targets = gmtest_items(true)
            if #targets == 0 then break end
            gmtest_swing(targets[1].x, targets[1].y)
        end
        assert(gd.wait_until(function() return mode:ready() end, 300), 'gallery objectives not complete')
        assert(#gmtest_items(true) == 0, 'gallery targets still live')
        gd.log('GMD PASS gallery goals cleared')
        gmtest_tp(42, 26)
        assert(gd.wait_until(function() return mode.state.phase == 'complete' end, 240), 'exit never completed')
        assert(mode.state.cleared.hall and mode.state.cleared.gallery, 'route skipped an area')
        for _, h in ipairs(gmtest_hall) do
            assert(gmtest_terminal[h].count == 1, 'duplicate terminal event after item cleanup')
        end
    end)
    gmtest_finish(ok, ok and 'both areas to exit' or tostring(why))
end)
