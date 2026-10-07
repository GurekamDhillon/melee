-- @name: motion_fix1_acceptance
-- @gameplay: true
-- @rollback_safe: false
-- UNRUN native fixture. Offline LAB, P1 Fox, FD, portable items enabled.
-- No other afterimage script should own P1 while this fixture runs.
local function warm()
  local w=gd.warm{fighters={1}}
  assert(gd.wait_until(function() local done,e=gd.warm_done(w);assert(not e,e);return done end,1800),'motion warm timeout')
  gd.warm_release(w)
end
gd.run(function()
  local ok,why=pcall(function()
    assert(gd.wait_until(function() return gd.match().active and gd.match().frame>180 and gd.player(1)~=nil end,3000),'P1 timeout')
    local h=assert(gd.afterimage_add(1,{copies=12,lifetime=60,spacing=4,surface='silhouette',trigger='always',debug=true,intensity=0}))
    warm();local before=gd.motion_stats();assert(gd.afterimage_set(h,{intensity=1}))
    gd.wait(90);local first=gd.motion_stats()
    assert(first.poses>before.poses,'no complete fighter poses retained')
    assert(first.copy_draws>before.copy_draws,'no historical draws replayed')
    for name,value in pairs(first) do
      if name:match('^fail_') then assert(value==(before[name] or 0),'capture/replay failure '..name) end
    end
    assert(gd.afterimage_set(h,{intensity=0}))
    do local pl=gd.player(1);local ks={};for k,v in pairs(pl) do if type(v)~='table' then ks[#ks+1]=k..'='..tostring(v) end end;table.sort(ks);gd.log('PRE-GIVE '..table.concat(ks,' ')) end
    assert(gd.give_item(1,'random'),'requires portable items enabled, eligible empty-handed P1')
    warm();local held=gd.motion_stats();assert(gd.afterimage_set(h,{intensity=1}))
    gd.wait(90);local after=gd.motion_stats()
    assert(after.poses>held.poses and after.copy_draws>held.copy_draws,'held item stopped pose capture/replay')
    assert(after.held_draw_scopes>held.held_draw_scopes,'held geometry scope was never captured')
    assert(after.held_retained_draws>held.held_retained_draws,'scope retained no actual held-item geometry')
    for name,value in pairs(after) do
      if name:match('^fail_') then assert(value==(held[name] or 0),'held capture/replay failure '..name) end
    end
    assert(gd.history(300,1),'history refused');gd.wait(4);assert(gd.rewind_test(90,true),'rewind request refused')
    assert(gd.wait_until(function() return gd.rewind_test_result().pass~=nil end,1800),'rewind timeout')
    local replay=gd.rewind_test_result();assert(replay.pass and replay.diff_compared==0,'held draw traversal changed replay state')
    assert(gd.afterimage_remove(h))
    gd.log('motion: fix1 acceptance poses='..after.poses..' copy_draws='..after.copy_draws..' held_draw_scopes='..after.held_draw_scopes)
  end)
  do local s=gd.motion_stats();local k={};for n in pairs(s) do k[#k+1]=n end;table.sort(k);local o={};for _,n in ipairs(k) do o[#o+1]=n..'='..tostring(s[n]) end;gd.log('STATS '..table.concat(o,' '));local r=gd.rewind_test_result();gd.log('REWIND pass='..tostring(r.pass)..' diff='..tostring(r.diff)..' cmp='..tostring(r.diff_compared)..' '..tostring(r.text)) end
  gd.log('TEST motion_fix1_acceptance '..(ok and 'PASS' or 'FAIL '..tostring(why)))
  gd.quit()
end)
