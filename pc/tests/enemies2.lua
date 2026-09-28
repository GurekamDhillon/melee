-- @name: enemies2
-- @gameplay: true
-- Unattended LAB/FD fixture. Launch as MELEE_PAD_SCRIPT; see enemies2-report.md.
local section, failures, done = 0, 0, false
local events, expected = {}, {}
local started = gd.time()
local function check(ok, detail)
    section = section + 1
    if not ok then failures = failures + 1 end
    gd.log(('TEST enemies2 section %d: %s %s'):format(
        section, ok and 'PASS' or 'FAIL', detail))
    return ok
end
local function finish()
    if done then return end
    done = true
    check(failures == 0, 'complete; failures=' .. failures)
    gd.quit()
end
function on_enemy_defeated(e)
    check(expected[e.handle] == e.kind, 'defeat identity ' .. tostring(e.kind))
    events[e.handle] = (events[e.handle] or 0) + 1
end
function on_tick()
    if done then return end
    -- Boot prompts and real controllers require no human intervention.
    if gd.scene().name == 'GS_MEMCARD' then
        gd.input(1, {buttons=(gd.time() - started) % 0.5 < 0.1 and gd.buttons.A or 0}, 1)
    end
    if gd.time() - started > 120 then
        check(false, 'watchdog: match or task stalled')
        finish()
    end
end
local function item(id)
    for _, v in ipairs(gd.items()) do if v.id == id then return v end end
end
gd.run(function()
    if not check(gd.wait_until(function() return gd.match().active end, 1800),
                 'match entered') then finish(); return end
    gd.input(1, {}, 1)
    gd.input(2, {}, 1)
    -- gd.match().stage is the internal Gr_Kind_Last (0x25), not external stage 32.
    if not check(gd.lab_mode() and gd.match().stage == 0x25, 'LAB on FD') then
        finish(); return
    end
    gd.wait(90)
    for _, name in ipairs({'like_like', 'octorok', 'polar_bear', 'topi'}) do
        gd.teleport(1, -65, 0)
        gd.teleport(2, 65, 0)
        local before = {}
        for _, v in ipairs(gd.items()) do before[v.id] = true end
        local h, why = gd.spawn_enemy(name, 0, 20, {facing=-1})
        check(h ~= nil, name .. ' spawn: ' .. tostring(why or h))
        if h then
            expected[h] = name
            local born
            for _, v in ipairs(gd.items()) do
                if not before[v.id] then born = v; break end
            end
            check(born ~= nil, name .. ' has live item')
            gd.wait(45)
            local live = born and item(born.id)
            check(live ~= nil and (live.state ~= born.state or
                  live.x ~= born.x or live.y ~= born.y), name .. ' AI/physics advances')
            check(live ~= nil and live.y >= -5 and live.y < 25,
                  name .. ' settles onto FD (not ceiling flight)')
            check(events[h] == nil, name .. ' no defeat before hit')
            local percent = gd.player(1).percent
            check(gd.hit({enemy=h}, {damage=500, angle=90, kbg=0, bkb=0, from=1}),
                  name .. ' hit accepted')
            check(gd.player(1).percent == percent, name .. ' hit does not target fighter port')
            check(gd.wait_until(function() return events[h] ~= nil end, 180),
                  name .. ' stock defeat callback delivered')
            check(not gd.hit({enemy=h}, {damage=1, angle=90, kbg=0, bkb=0}),
                  name .. ' defeated handle rejects another hit')
            gd.wait(30)
            check(events[h] == 1, name .. ' exactly one defeat')
            gd.enemy_remove(h) -- clean up any remaining death presentation
        end
        local removed = gd.spawn_enemy(name, 0, 20)
        check(removed ~= nil, name .. ' removal fixture spawn')
        if removed then
            expected[removed] = name
            check(gd.enemy_remove(removed), name .. ' explicit removal succeeds')
            check(not gd.enemy_remove(removed), name .. ' stale removal rejected')
            check(not gd.hit({enemy=removed}, {damage=500, angle=90, kbg=0, bkb=0}),
                  name .. ' stale hit rejected')
            gd.wait(5)
            check(events[removed] == nil, name .. ' removal emits no defeat')
        end
    end
    finish()
end)
