-- Lead-run integration check. Use the same data-dir setup as workbench_check.lua.
-- Detects render changing the simulation camera, incomplete/stale sidecars, and
-- requests that never finish. Inspect each logged capture for visual coverage.
local cases = {
    {},
    {camera={preset='front'}},
    {camera={preset='side'}},
    {camera={preset='top'}},
    {camera={preset='three_quarter',yaw=30,pitch=20,distance=80}},
    {overlays={hitboxes=true}},
    {overlays={hurtboxes=true}},
    {overlays={ecb=true}},
    {overlays={bones=true}},
    {overlays={floor=true}},
    {overlays={stage=false,model=false,bones=true}},
    {overlays={hud=true}},
    {camera={preset='front'},overlays={model=false}},
    {camera={preset='side'},overlays={model=false}},
    {camera={preset='three_quarter'},overlays={model=false}},
    {camera={preset='top'},overlays={model=false}},
    {camera={preset='front',fov=30}}, -- explicit lens must stay narrow
    {camera={preset='side'},overlays={stage=false}},
}
local index, pending, ticks = 1, false, 0
local before, draw_flags, stage_flags, prepared, preparing
local function check_fighter_height()
    local lo,hi=math.huge,-math.huge
    for _,j in ipairs(gd.joints(1,true)) do
        if j.valid then lo,hi=math.min(lo,j.y),math.max(hi,j.y) end
    end
    -- Goku fixture: interrupted Entry left every bone within 0.2 vertical units.
    assert(hi-lo>1,'fighter retained Entry squash after workbench preparation')
end
local function same_pose(a,b)
    assert(a.mode == b.mode, 'render changed camera mode')
    for _,k in ipairs({'eye','interest'}) do
        for _,axis in ipairs({'x','y','z'}) do
            assert(math.abs(a[k][axis]-b[k][axis])<0.001, 'render changed camera '..k)
        end
    end
    assert(a.fov==b.fov and a.roll==b.roll, 'render changed camera lens')
end
function on_tick()
    if not wb or not gd.player(1) or not gd.paused() then return end
    ticks=ticks+1
    assert(ticks<20000,'workbench render timeout')
    if not prepared then
        assert(not gd.player(1).hidden,'workbench paused with an invisible Entry fighter')
        check_fighter_height()
        local dair
        for _,row in ipairs(gd.motion_list(1)) do
            if row.name=='AttackAirLw' then dair=row.id end
        end
        assert(dair,'Goku fixture needs AttackAirLw')
        assert(wb.frame{motion=dair,frame=14,lift=40}=='pending')
        prepared,preparing=true,true
        return
    end
    if preparing then
        local json=gd.data_read('frame.json')
        if not json or tonumber(json:match('"frame":([%-0-9]+)'))~=14 then return end
        check_fighter_height()
        preparing=false
    end
    if pending then
        local json=gd.data_read('view.json')
        if not json then return end
        local w=tonumber(json:match('"w":(%d+)'))
        local h=tonumber(json:match('"h":(%d+)'))
        assert(w and w>0 and h and h>0,'missing actual PNG dimensions')
        assert(json:find('"requested_w":1280',1,true))
        assert(json:find('"requested_h":720',1,true))
        same_pose(before,gd.camera_get())
        assert(draw_flags == gd.debug_draw(1), 'render changed fighter debug flags')
        assert(stage_flags == gd.debug_stage(), 'render changed stage debug flags')
        gd.log('WBRENDER case='..index..' actual='..w..'x'..h..' restored')
        index,pending=index+1,false
        return
    end
    if not cases[index] then gd.log('WBRENDER checks complete'); gd.quit(); return end
    assert(not pcall(wb.render,{w=0}), 'accepted zero capture width')
    assert(not pcall(wb.render,{camera={distance=0}}), 'accepted zero camera distance')
    assert(not pcall(wb.render,{camera={target_joint=999999}}), 'accepted nonexistent joint')
    local opts=cases[index]
    opts.w,opts.h=1280,720
    before=gd.camera_get()
    draw_flags,stage_flags=gd.debug_draw(1),gd.debug_stage()
    local result=wb.render(opts)
    assert(result:match('^@.*[/\\]view%.png$'),'render reply changed')
    assert(not pcall(wb.render,opts),'overlapping capture was accepted')
    pending=true
end
