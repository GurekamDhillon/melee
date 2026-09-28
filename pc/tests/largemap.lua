-- @name: largemap
-- @gameplay: true
-- @rollback_safe: false
-- MELEE_SCENE='mode=lab;p1=fox;p2=none;stage=bf;time=0'
-- MELEE_PAD_SCRIPT=<absolute path>/pc/tests/largemap.lua
-- Install exported bf_interior_room/models as pc/tests/models (no room-building script).
local section, failures, ticks, finished = 0, 0, 0, false
local function check(ok, detail)
    section = section + 1
    if not ok then failures = failures + 1 end
    gd.log(('TEST largemap section %d: %s %s'):format(section, ok and 'PASS' or 'FAIL', detail))
    return ok
end
local function hold(x, frames)
    gd.input(1, {x=x}, frames)
    gd.wait(frames)
end
-- A hook watchdog also terminates startup failures and a coroutine disabled by an API error.
function on_tick()
    ticks = ticks + 1
    if not finished and ticks > 30000 then
        finished = true
        check(false, 'watchdog expired')
        gd.quit()
    end
end
gd.run(function()
    local ok, why = pcall(function()
        assert(gd.wait_until(function() return gd.match().active and gd.match().frame > 90 end, 3000), 'match timeout')
        check(gd.lab_mode() and gd.player(1).char_name == 'Fox', 'LAB with Fox')
        gd.history(0) -- Lua builders are not replayed by rewind/resimulation.
        gd.input(1, {}, 1)
        local original = gd.stage_bounds()
        local width = original.camera.right - original.camera.left
        assert(width > 0, 'invalid Battlefield width')
        local span, y = width * 20, 100
        local bounds = {left=-span/2-width, right=span/2+width, bottom=-1000, top=2000}
        gd.stage_bounds({camera=bounds, blast=bounds})
        local b = gd.stage_bounds()
        check(b.camera.right == bounds.right and b.blast.left == bounds.left, '20x Battlefield camera and blast bounds')
        local invalid = pcall(gd.stage_bounds, {camera={left=0,right=10,bottom=0,top=10},blast={left=1,right=0,bottom=0,top=10}})
        check(not invalid and gd.stage_bounds().camera.right==bounds.right, 'invalid rectangle leaves both bounds unchanged')
        local before = gd.stage_stats()
        check(before.line_capacity > 200 and before.instance_capacity > 128 and before.target_capacity > 32, 'raised pools')
        local floor = gd.model_load('bf_floor_4m')
        local wall = gd.model_load('bf_wall_solid_4m')
        local beam = gd.model_load('bf_beam_4m')
        -- Cross both old ceilings, and exercise transactional cleanup after a builder error.
        local stale
        gd.area_load('capacity', function()
            for i=1,210 do
                stale = assert(gd.model_spawn(floor, {x=i*2, y=500, collision=false}))
            end
            for i=1,300 do assert(gd.stage_add_line(i*2, 600, i*2+1, 600, 'floor')) end
        end)
        local full = gd.stage_stats()
        check(full.instances == before.instances+210 and full.lines == before.lines+300, '210 instances and 300 lines simultaneously')
        check(gd.floor_below(600.5, 620, 40) ~= nil, 'collision beyond old line limit is queryable')
        gd.area_unload('capacity')
        check(gd.model_get(stale) == nil and gd.floor_below(600.5,620,40) == nil, 'unload removes draws and collision handles')
        local failed = pcall(gd.area_load, 'abort', function()
            assert(gd.model_spawn(floor, {x=0,y=500}))
            error('intentional builder failure')
        end)
        check(not failed and not gd.area_loaded('abort') and gd.stage_stats().lines == before.lines, 'failed load rolls back created content')
        local saved
        gd.area_load('snapshot', function() saved=assert(gd.model_spawn(floor,{x=500,y=500})) end)
        local duplicate = gd.area_load('snapshot', function() error('must not run twice') end)
        check(not duplicate and gd.area_loaded('snapshot'), 'loading an active name is idempotent')
        gd.savestate(4) gd.wait(3)
        gd.area_unload('snapshot')
        check(gd.model_get(saved)==nil, 'snapshot area unloaded')
        gd.loadstate(4) gd.wait(3)
        check(gd.area_loaded('snapshot') and gd.model_get(saved)~=nil and gd.floor_below(500,520,40)~=nil,
              'savestate restores membership, instance and collision')
        gd.area_unload('snapshot')
        -- Actual target allocation is subject to ItCo's category ceiling as well as this pool.
        gd.area_load('targets', function()
            for i=1,40 do assert(gd.spawn_target(-200+i*10, 800), 'target '..i..' refused') end
        end)
        check(gd.stage_stats().targets == before.targets+40, '40 real targets beyond old limit')
        gd.area_unload('targets')
        gd.fly(1, true)
        gd.fly_speed(width/80)
        gd.teleport(1, -span/2+width/2, y+35)
        local function room_x(n) return -span/2+(n-0.5)*width end
        local function room(n)
            gd.area_load('room'..n, function()
                -- Six exported Blender kit parts plus a continuous, explicitly sized floor.
                for _, dx in ipairs({-width/3,0,width/3}) do
                    assert(gd.model_spawn(floor, {x=room_x(n)+dx,y=y,scale=width/78,collision=false}))
                end
                assert(gd.model_spawn(wall, {x=room_x(n),y=y,z=-10,scale=3,collision=false}))
                assert(gd.model_spawn(beam, {x=room_x(n),y=y+70,scale=3,collision=false}))
                assert(gd.model_spawn(floor, {x=room_x(n),y=y+60,floor_flags=1}))
                assert(gd.stage_add_line(room_x(n)-width/2,y,room_x(n)+width/2,y,'floor'))
            end)
        end
        local function window(n)
            for i=1,20 do
                if math.abs(i-n)>1 then gd.area_unload('room'..i) end
            end
            for i=math.max(1,n-1),math.min(20,n+1) do room(i) end
        end
        window(1)
        hold(0,30)
        local warm = gd.stage_stats()
        check(warm.heap_free>0 and warm.asset_bytes<=warm.asset_budget, 'valid heap and asset budget diagnostics')
        local min_x, max_x = gd.player(1).x, gd.player(1).x
        -- Both directions reload previously unloaded names and reuse pool slots.
        for lap=1,2 do
            for step=1,20 do
                local n = lap == 1 and step or 21-step
                window(n)
                local target = room_x(n)
                local limit = 0
                while math.abs(gd.player(1).x-target)>width/100 and limit<140 do
                    hold(gd.player(1).x<target and 127 or -127,1)
                    limit=limit+1
                end
                hold(0,25) -- normal camera smoothing settles; no scripted camera_follow.
                local p, c, s = gd.player(1), gd.camera_get(), gd.stage_stats()
                min_x, max_x = math.min(min_x,p.x), math.max(max_x,p.x)
                check(math.abs(p.x-target)<width/40, ('flight lap %d room %d x=%.2f'):format(lap,n,p.x))
                check(c and math.abs(c.interest.x-p.x)<width/2, ('camera follows room %d'):format(n))
                check(s.instances<=before.instances+18 and s.visible_instances<=18 and s.lines<=before.lines+6 and s.areas<=3, 'stream window stays at three rooms and eighteen model draws')
                check(s.asset_bytes==warm.asset_bytes and s.state_bytes==warm.state_bytes and s.heap_free>0 and s.heap_free>=warm.heap_free-65536,
                      ('bounded memory heap=%d assets=%d state=%d'):format(s.heap_free,s.asset_bytes,s.state_bytes))
                check(gd.floor_below(target,y+10,20) ~= nil, 'loaded room floor exists')
                local absent = n<10 and 20 or 1
                check(gd.floor_below(room_x(absent),y+10,20) == nil, 'distant room collision absent')
            end
        end
        check(max_x-min_x>width*18.5, 'traversed corridor spanning twenty Battlefield camera widths')
        -- Flight disables KOs itself: turn it off to test the extended blast rectangle.
        gd.teleport(1,room_x(1),y+20)
        gd.fly(1,false)
        hold(0,60)
        check(gd.player(1).action>10 and math.abs(gd.player(1).x-room_x(1))<10, 'alive outside retail blast zone with flight disabled')
        check(math.abs(gd.camera_get().interest.x-gd.player(1).x)<width/2, 'normal camera follows extended bounds with flight disabled')
        for n=1,20 do gd.area_unload('room'..n) end
        local empty = gd.stage_stats()
        check(empty.instances==before.instances and empty.lines==before.lines and empty.targets==before.targets and empty.areas==0, 'all areas reclaimed')
        gd.model_release(floor) gd.model_release(wall) gd.model_release(beam)
        gd.stage_bounds(false)
        check(gd.stage_bounds().blast.left==original.blast.left, 'bounds restored')
    end)
    if not ok then check(false, tostring(why)) end
    check(failures==0, ('complete; %d failures'):format(failures))
    finished=true
    gd.quit()
end)
