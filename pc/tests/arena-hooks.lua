-- @name: arena-hooks
-- @gameplay: true
-- Run with MELEE_SCENE=mode=lab;p1=fox;p2=marth;stage=bf;time=0
local section, failed, ticks, done = 0, 0, 0, false
local function check(ok, detail)
    section = section + 1
    if not ok then failed = failed + 1 end
    gd.log(('TEST arena-hooks section %d: %s %s'):format(section, ok and 'PASS' or 'FAIL', detail))
end
local function near(a, b) return math.abs(a - b) < 0.001 end
local function same(a, b)
    return near(a.left,b.left) and near(a.right,b.right) and near(a.top,b.top) and near(a.bottom,b.bottom)
end
function on_tick()
    ticks = ticks + 1
    if not done and ticks > 6000 then
        check(false, 'watchdog timeout (including script errors)')
        done = true
        pcall(gd.stage_restore_bounds)
        gd.quit()
    end
end
gd.run(function()
    gd.input(1, 0, 1) gd.input(2, 0, 1) -- persistent neutral pad claims
    gd.wait_until(function() return gd.match().active and gd.match().frame > 150 end, 3000)
    assert(gd.match().active, 'LAB did not start')
    check(gd.lab_mode(), 'running in LAB mode')
    local original = gd.stage_bounds()
    check(original ~= nil, 'bounds readback available')
    check(not pcall(gd.stage_set_camera_bounds, 10, -10, 50, -50), 'inverted bounds rejected')
    check(not pcall(gd.stage_set_origin, 0/0, 0), 'nonfinite origin rejected')
    check(not pcall(gd.stage_set_origin, 0, 0, {frames=-1}), 'negative transition rejected')
    check(not pcall(gd.stage_collision_group, -1, false), 'invalid group rejected')
    local groups = gd.stage_collision_groups()
    check(#groups > 0, 'retail collision groups enumerated')
    if #groups > 0 then
        local g = groups[1]
        gd.stage_collision_group(g.id, not g.enabled)
        check(gd.stage_collision_groups()[1].enabled == not g.enabled, 'group toggled')
        gd.stage_collision_group(g.id, g.enabled)
        check(gd.stage_collision_groups()[1].enabled == g.enabled, 'group restored')
    end
    gd.stage_set_origin(0, 0)
    gd.stage_set_camera_bounds(-60, 60, 90, -35)
    gd.stage_set_blast_bounds(-90, 90, 120, -65)
    gd.wait(2)
    local locked = gd.stage_bounds()
    check(same(locked.camera, {left=-60,right=60,top=90,bottom=-35}), 'camera locked around centre')
    check(same(locked.blast, {left=-90,right=90,top=120,bottom=-65}), 'blast bounds locked around centre')
    gd.stage_set_origin(12, 6, {frames=60})
    gd.wait(30)
    local halfway = gd.stage_bounds()
    check(halfway.frames == 30 and near(halfway.origin.x,6) and near(halfway.origin.y,3),
          'origin transition reaches halfway after exactly 30 logic ticks')
    local saved_group = groups[#groups]
    if saved_group then gd.stage_collision_group(saved_group.id, not saved_group.enabled) end
    gd.savestate(1) gd.wait(2)
    local saved
    -- Save callback observes the exact frame-boundary snapshot, before its next tick.
    saved = arena_saved
    if saved_group then gd.stage_collision_group(saved_group.id, saved_group.enabled) end
    gd.wait(65)
    local finish = gd.stage_bounds()
    check(finish.frames == 0 and near(finish.origin.x,12) and near(finish.origin.y,6), '60-tick origin reaches target')
    check(near(finish.camera.left,-48) and near(finish.blast.left,-78), 'origin shifts camera and blast offsets together')
    gd.loadstate(1) gd.wait(2)
    local loaded = arena_loaded
    check(saved and loaded and loaded.frames == saved.frames and near(loaded.origin.x,saved.origin.x), 'savestate restores transition exactly at load callback')
    if saved_group then
        check(arena_loaded_groups[#groups].enabled == not saved_group.enabled, 'savestate restores collision group enabled state')
        gd.stage_collision_group(saved_group.id, saved_group.enabled)
    end
    gd.wait(65)
    check(near(gd.stage_bounds().origin.x,12) and gd.stage_bounds().frames == 0, 'restored transition completes')
    gd.stage_set_origin(0, 0)
    gd.teleport(1, 0, 10) gd.wait(5)
    check(gd.player(1).action > 10, 'fighter alive inside arena before KO check')
    -- This point is below the new bottom (-65), but inside Battlefield's original zone.
    local y = (-65 + original.blast.bottom) / 2
    check(y < -65 and y > original.blast.bottom, 'KO probe lies inside original blast zone')
    gd.teleport(1, 0, y)
    local died = false
    for _ = 1, 60 do
        gd.wait(1)
        local p = gd.player(1)
        if p and p.action <= 10 then died = true break end
    end
    check(died, 'fighter pushed past narrowed blast zone enters Dead motion')
    gd.stage_set_origin(3, 4, {frames=60})
    gd.stage_restore_bounds()
    gd.wait(2)
    local restored = gd.stage_bounds()
    check(same(restored.camera,original.camera) and same(restored.blast,original.blast)
          and near(restored.origin.x,original.origin.x) and near(restored.origin.y,original.origin.y)
          and restored.frames == 0, 'original bounds and origin restored')
    gd.stage_restore_bounds()
    check(same(gd.stage_bounds().blast,original.blast), 'restore is idempotent')
    check(failed == 0, ('completed with %d failed checks'):format(failed))
    done = true
    gd.quit()
end)
function on_savestate(slot)
    if slot == 1 then arena_saved = gd.stage_bounds() end
end
function on_loadstate(slot)
    if slot == 1 then
        arena_loaded = gd.stage_bounds()
        arena_loaded_groups = gd.stage_collision_groups()
    end
end
