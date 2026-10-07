-- @name: motion_visual_acceptance
-- @gameplay: true
-- @rollback_safe: false
-- Native game acceptance fixture, UNRUN. Requires offline match and rebuilt Aurora.
gd.run(function()
  local ok,why=pcall(function()
    assert(gd.wait_until(function() return gd.match().active and gd.player(1)~=nil end,3000),'match timeout')
    local before=gd.motion_stats()
    local pose=assert(gd.afterimage_add(1,{copies=6,spacing=1,lifetime=31,surface='silhouette',trigger='always',debug=true}))
    local handles={}
    for i=1,64 do handles[i]=assert(gd.tracer_add{port=1,anchor='right_hand'}) end
    local extra,reason=gd.tracer_add{port=1,anchor='left_hand'}
    assert(extra==nil and reason,'capacity must refuse')
    for _,h in ipairs(handles) do assert(gd.tracer_remove(h)) end
    local a=assert(gd.tracer_hitboxes(1,{length=12,width=1}))
    local warm=gd.warm{fighters={1},tracers=true}
    assert(gd.wait_until(function() local done,e=gd.warm_done(warm);assert(not e,e);return done end,1200),'warm timeout')
    gd.warm_release(warm)
    gd.wait(60)
    assert(gd.history(300,1),'history refused');gd.wait(4);assert(gd.rewind_test(90,true))
    assert(gd.wait_until(function() local r=gd.rewind_test_result();return r.pass~=nil end,1200),'rewind timeout')
    local r=gd.rewind_test_result();assert(r.pass and r.diff_compared==0,'effects changed replay state bytes')
    assert(gd.afterimage_remove(pose));assert(gd.tracer_remove(a))
    local after=gd.motion_stats();assert(after.afterimages==before.afterimages and after.tracers==before.tracers,'remove cleanup')
    assert(not pcall(gd.afterimage_set,pose,{copies=1}),'stale handle accepted')
  end)
  do local s=gd.motion_stats();local k={};for n in pairs(s) do k[#k+1]=n end;table.sort(k);local o={};for _,n in ipairs(k) do o[#o+1]=n..'='..tostring(s[n]) end;gd.log('STATS '..table.concat(o,' '));local r=gd.rewind_test_result();gd.log('REWIND pass='..tostring(r.pass)..' diff='..tostring(r.diff)..' cmp='..tostring(r.diff_compared)..' '..tostring(r.text)) end
  gd.log('TEST motion_visual_acceptance '..(ok and 'PASS' or 'FAIL '..tostring(why)))
  gd.quit()
end)
