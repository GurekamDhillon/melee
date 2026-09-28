-- Same-chunk test for examples/boss_gamemode_bonus: bundle.py --test pc/tests/boss_gamemode_bonus.lua
-- Scene: MELEE_SCENE='mode=classic;p1=fox;step=10;difficulty=0' (vanilla disc). Logs BBG PASS/FAIL, gd.quit().
-- Master Hand is dropped to 1 HP and KO'd with gd.hit; the rooms' enemies/targets are cleared with
-- gd.hold_hitbox; afterwards the results screen (Classic trophy fall after Start) must be reached.
local bbg_done, bbg_ticks = false, 0
local function bbg_finish(ok, why)
    if bbg_done then return end
    bbg_done = true
    gd.log('BBG ' .. (ok and 'PASS ' or 'FAIL ') .. why)
    gd.quit()
end
function on_tick()
    bbg_ticks = bbg_ticks + 1
    if bbg_ticks > 30000 then bbg_finish(false, 'deadline') end
end
local function bbg_items(targets)
    local out = {}
    for _, it in ipairs(gd.items() or {}) do
        if (targets and it.kind == 0xD1) or (not targets and (it.kind == 0x2B or it.kind == 0x2C)) then
            out[#out + 1] = it
        end
    end
    return out
end
local function bbg_tp(x, y)
    for _ = 1, 40 do
        if pcall(gd.teleport, 1, x, y) then return end
        gd.wait(15)
    end
    error('teleport never became available')
end
local function bbg_swing(x, y)
    if not gd.fly(1) then gd.fly(1, true) end
    if not gd.hold_hitbox(1) then gd.hold_hitbox(1, true, {action = 'nair'}) end
    bbg_tp(x, y)
    gd.wait(24)
end
local function bbg_section(n, text) gd.log('BBG section ' .. n .. ': PASS ' .. text) end

gd.run(function()
    local ok, why = pcall(function()
        assert(gd.wait_until(function() return mode and mode.state and mh_port end, 6000), 'mode never armed')
        local start = {x = gd.player(1).x, y = gd.player(1).y}
        gd.wait(90)
        gd.set_damage(mh_port, 299) -- 1 HP left of 300
        gd.wait(30)
        assert(gd.hit(mh_port, {damage = 20, angle = 90, kbg = 50, bkb = 40}), 'environment hit rejected')
        assert(gd.wait_until(function() return mode.state.boss_done end, 1800), 'boss defeat hook never reached the mode')
        bbg_section(1, 'Master Hand KO held by the mode')
        assert(gd.wait_until(function() return mode.state.area == 'hall' and mode.state.phase == 'play' end, 1800),
               'never entered the hall')
        assert(dx - 52 - 85 >= 200, 'rooms not 200 units clear of the stage')
        assert(gd.stage_bounds().camera.right >= dx + 80 and gd.stage_bounds().blast.right >= dx + 110,
               'camera/blast bounds not widened')
        assert(gd.player(1).x > dx - 60, 'fighter was not moved to the rooms')
        bbg_section(2, 'rooms at x=' .. dx .. ', bounds widened, fighter inside')
        for _ = 1, 40 do
            local e = bbg_items(false)
            if #e == 0 then break end
            bbg_swing(e[1].x, e[1].y)
        end
        assert(gd.wait_until(function() return mode:ready() end, 180), 'hall goal stranded')
        local _, hits = gd.hold_hitbox(1)
        assert(hits >= 5, 'held hitbox re-armed only ' .. tostring(hits) .. ' times')
        bbg_section(3, 'hall wave cleared with gd.hold_hitbox')
        gd.hold_hitbox(1, false)
        bbg_tp(dx + 42, 26)
        assert(gd.wait_until(function() return mode.state.area == 'gallery' and mode.state.phase == 'play' end, 240),
               'door never entered the gallery')
        for _ = 1, 30 do
            local t = bbg_items(true)
            if #t == 0 then break end
            bbg_swing(t[1].x, t[1].y)
        end
        assert(gd.wait_until(function() return mode:ready() end, 300), 'gallery goals incomplete')
        bbg_section(4, 'gallery targets broken')
        gd.hold_hitbox(1, false)
        bbg_tp(dx + 42, 26)
        assert(gd.wait_until(function() return bbg_done == false and done end, 400), 'sequence never finished/restored')
        gd.fly(1, false)
        assert(not widened and gd.stage_bounds().camera.right < dx, 'bounds not restored')
        gd.wait(60)
        assert(math.abs(gd.player(1).x - start.x) < 150, 'fighter not put back near the stage: x=' .. tostring(gd.player(1).x) .. ' y=' .. tostring(gd.player(1).y) .. ' start=' .. tostring(start.x) .. ' saved=' .. tostring(saved and saved.x) .. ' back=' .. tostring(back))
        bbg_section(5, 'camera/bounds restored, fighter back on the stage')
        -- The hold is released: the normal clear + results screen must follow.
        gd.wait(600)
        local reached = false
        for _ = 1, 60 do
            gd.press(1, 'START', 2)
            gd.wait(28)
            if gd.scene().name == 'GS_REGEND_TOYFALL' then reached = true break end
        end
        assert(reached, 'results screen / Classic trophy fall not reached after release')
        bbg_section(6, 'results reached')
    end)
    bbg_finish(ok, ok and 'boss -> rooms -> restore -> results' or tostring(why))
end)
