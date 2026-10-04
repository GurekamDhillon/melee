-- @name: motion_worst_case
-- @gameplay: true
-- @rollback_safe: false
-- UNRUN benchmark workload. Six visible fighters required, no roster substitutions.
local started=false
gd.run(function()
  assert(gd.wait_until(function() return gd.match().active and gd.player(6)~=nil end,3000),'six fighters required')
  local ports={1,2,3,4,5,6}
  for _,p in ipairs(ports) do
    assert(gd.afterimage_add(p,{copies=6,spacing=1,lifetime=31,surface='silhouette'}))
    for _,anchor in ipairs({'right_hand','left_foot'}) do assert(gd.tracer_add{port=p,anchor=anchor,length=31,smoothing=8,width=2,shader='glow',params={1,1,0,0}}) end
  end
  local w=gd.warm{fighters=ports,tracers=true}
  assert(gd.wait_until(function() local done,e=gd.warm_done(w);assert(not e,e);return done end,1800),'warm timeout')
  gd.warm_release(w);started=true;gd.log('motion worst-case ready: begin profiler measurement, exercise all six fighters')
end)
function on_draw() if started then local a=gd.safe_area();gd.text(a.x+16,a.y+16,'Motion worst case: six fighters / six copies / two tracers each') end end
