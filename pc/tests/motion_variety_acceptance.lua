-- @name: motion_variety_acceptance
-- @gameplay: true
-- @rollback_safe: false
-- Native game fixture for the FX colour/shape variety options (palette, hue_shift, scale_falloff, gradient, pulse,
-- hue_drift, hue_span, swell). Offline LAB, P1 Fox, FD, CPU idle. Run unattended (see tools/port/README.md); it
-- proves the options reach the renderer (poses, copy draws and ribbon vertices grow, no new fail_* counter) and that
-- a rewind compares 0 simulation bytes with them on. The look itself is for a human on a monitor.
gd.run(function()
  local ok,why=pcall(function()
    assert(gd.wait_until(function() return gd.match().active and gd.match().frame>120 and gd.player(1)~=nil end,3000),'P1 timeout')
    local ghost=assert(gd.afterimage_add(1,{copies=6,spacing=3,lifetime=31,surface='silhouette',blend='additive',trigger='always',debug=true,intensity=0,
      palette={{1,0.2,0.2,0.9},{0.2,1,0.3,0.9},{0.2,0.4,1,0.9}},hue_shift=40,scale_falloff=0.5}))
    local trail=assert(gd.tracer_add{port=1,anchor='right_foot',width=6,length=40,smoothing=6,shader='glow',intensity=0,
      gradient={{1,0.9,0.4,1},{1,0.3,0.1,0.9},{0.3,0.3,1,0.6},{0.1,0.1,0.4,0}},pulse={3,0.6},hue_drift=90,hue_span=180,swell=1.5,params={1,2,0.5,2}})
    local boxes=assert(gd.tracer_hitboxes(1,{width=3,length=20,gradient={{1,1,1,1},{0.5,0.5,1,0}},swell=-0.5}))
    local w=gd.warm{fighters={1},tracers=true}
    assert(gd.wait_until(function() local done,e=gd.warm_done(w);assert(not e,e);return done end,1800),'warm timeout')
    gd.warm_release(w)
    local before=gd.motion_stats()
    assert(gd.afterimage_set(ghost,{intensity=1}));assert(gd.tracer_set(trail,{intensity=1}));assert(gd.tracer_set(boxes,{intensity=1}))
    for i=1,6 do   -- run back and forth; change the options live mid-run
      gd.input(1,{x=(i%2==1) and 127 or -127},20);gd.wait(20)
      if i==2 then assert(gd.afterimage_set(ghost,{palette={},hue_shift=-60}));assert(gd.tracer_set(trail,{gradient={},hue_span=0,swell=-0.6})) end
      if i==4 then assert(gd.afterimage_set(ghost,{palette={{1,1,0,1},{0,1,1,1}},scale_falloff=-0.5}));assert(gd.tracer_set(trail,{gradient={{1,0,0,1},{0,0,1,0}},pulse={0,0}})) end
    end
    gd.input(1,{buttons='A'},4);gd.wait(30)
    local after=gd.motion_stats()
    assert(after.poses>before.poses,'no poses retained');assert(after.copy_draws>before.copy_draws,'no copies replayed')
    assert(after.ribbon_vertices>before.ribbon_vertices,'no ribbon vertices drawn')
    for name,value in pairs(after) do if name:match('^fail_') then assert(value==(before[name] or 0),'new failure '..name) end end
    assert(gd.history(300,1),'history refused');gd.wait(4);assert(gd.rewind_test(90,true),'rewind refused')
    assert(gd.wait_until(function() return gd.rewind_test_result().pass~=nil end,1800),'rewind timeout')
    local r=gd.rewind_test_result();assert(r.pass and r.diff_compared==0,'variety options changed simulation bytes')
    assert(gd.afterimage_remove(ghost));assert(gd.tracer_remove(trail));assert(gd.tracer_remove(boxes))
    gd.log(('motion: variety acceptance poses=%d copy_draws=%d ribbon_vertices=%d'):format(after.poses-before.poses,after.copy_draws-before.copy_draws,after.ribbon_vertices-before.ribbon_vertices))
  end)
  do local s=gd.motion_stats();local k={};for n in pairs(s) do k[#k+1]=n end;table.sort(k);local o={};for _,n in ipairs(k) do o[#o+1]=n..'='..tostring(s[n]) end;gd.log('STATS '..table.concat(o,' '));local r=gd.rewind_test_result();gd.log('REWIND pass='..tostring(r.pass)..' diff='..tostring(r.diff)..' cmp='..tostring(r.diff_compared)..' '..tostring(r.text)) end
  gd.log('TEST motion_variety_acceptance '..(ok and 'PASS' or 'FAIL '..tostring(why)))
  gd.quit()
end)
