-- Executes the actual demo against a narrow drawing/API stub; no game launch.
local calls,events,draws=0,{},0
gd={
  match=function() return {active=true} end,
  zone_add=function(z) assert(z.name and z.x0<z.x1 and z.y0<z.y1);calls=calls+1;return calls end,
  contact_overlay=function(v) assert(type(v)=='boolean') end,
  log=function(...) events[#events+1]={...} end,
  project=function(x,y) return x,y,true,0 end,
  line=function(...) draws=draws+1 end,
  safe_area=function() return {x=0,y=0,w=640,h=480} end,
  text=function(...) draws=draws+1 end,
  player=function(port) return port==1 and {} or nil end,
  zones_at=function(port,sub) return sub==0 and {{label='Doorway',frames=20}} or {} end,
}
dofile('melee/pc/scripts/examples/demos/zones/scripts/main.lua')
on_scene();on_match_start();assert(calls==3)
on_draw();assert(draws==14)
local e={port=1,entity=1,sub=0,label='Doorway',x=0,y=0}
on_zone_enter(e);on_zone_exit(e);on_zone_none(e);on_zone_some(e)
assert(#events==4)
on_unload();on_match_end()
print('zones demo: PASS (setup, outlines, membership text, four event hooks)')
