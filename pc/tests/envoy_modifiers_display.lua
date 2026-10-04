local T=dofile('melee/pc/tests/envoy_testlib.lua');local D=T.rules()
for _,name in ipairs{'mod_codec','mod_schema','mod_pool','mod_engine','mod_display'} do D[name]=T.module(name,D) end
local function fixture()
 local e=D.mod_engine.new(7,D.mod_pool);local s={next=0,passes=0,surfaces={},posts={},params={},loads=0,done=false,warm_calls=0}
 local g={}
 function g.player(p) if p<=2 then return {stocks=4} end end
 function g.fighter_shader(p,path,opts) if path then s.loads=s.loads+1 end;s.surfaces[p]=path and opts.params or nil;return true end
 function g.fighter_shader_set(p,opts) assert(s.surfaces[p]);s.params[p]=opts.params;return true end
 function g.shader_load() s.next=s.next+1;return s.next end
 function g.shader_status() return {valid=true} end
 function g.post_add(shader,opts) s.next=s.next+1;opts.params._born=s.passes;s.posts[s.next]=opts.params;return s.next end
 function g.post_set(p,opts) assert(s.posts[p]);s.posts[p]=opts.params;return true end
 function g.post_remove(p) s.posts[p]=nil;return true end
 function g.post_ready(h) return s.passes>s.posts[h]._born end
 function g.perf() return {shaders={passes=s.passes}} end
 function g.warm(opts) s.warm_calls=s.warm_calls+1;assert(#opts.fighters==2);return 90 end
 function g.warm_done() return s.done end
 function g.warm_release() end
 function g.safe_area() return {x=0,y=0,w=640,h=480} end
 function g.fill() end;function g.text() end
 local d=D.mod_display.new(g,e)
 return e,s,d
end
local function ready(d,s) assert(not d:warm());s.done=true;assert(not d:warm());s.passes=s.passes+1;assert(d:warm());assert(not next(s.posts)) end
T.test('real pipeline gate and params-only named equipment treatments',function()
 local e,s,d=fixture();ready(d,s);e:equip(1,'kindling');e:equip(1,'glass_core');e.frame=10
 assert(d:update());assert(s.params[1][1]==10/60 and s.params[1][10]>=2 and s.params[1][10]<3)
 assert(s.params[1][12]>=1 and s.params[1][12]<2)
 local loaded=s.loads;e.frame=11;assert(d:update());assert(s.loads==loaded)
 d:draw();d:clear();assert(not next(s.surfaces) and not next(s.posts))
end)
T.test('solo multieffect no pulse; distinct labels pulse and fixedlogic cooldown',function()
 local e,s,d=fixture();ready(d,s)
 e.frame=1;e.trace={'guarded applied by Reprisal','momentum applied by Reprisal'};e.display.trace_generation=1;d:update();assert(not next(s.posts))
 e.frame=2;e.trace={'burn applied by Kindling','curse applied by Pyre'};e.display.trace_generation=2;d:update();assert(e.display.pulse_strength==.03 and e.display.pulse_start==2)
 e.frame=10;d:update();assert(s.posts[d.post].progress==8/18 and s.posts[d.post].strength<=.06)
 e.frame=20;e.display.trace_generation=3;d:update();assert(e.display.pulse_start==2 and not next(s.posts))
 e.frame=32;e.display.trace_generation=4;d:update();assert(e.display.pulse_start==32)
end)
T.test('rewind reconstructs pulse exactly; KO oneport preserves other display readiness',function()
 local e,s,d=fixture();ready(d,s);e:equip(2,'icebound')
 e.frame=40;e.trace={'burn applied by Kindling','curse applied by Pyre'};e.display.trace_generation=1;d:update()
 e.frame=45;d:update();local snap=e:export();local old=s.posts[d.post].progress
 e.frame=80;d:update();e:import(snap);assert(d:on_loadstate(e));assert(s.posts[d.post].progress==old)
 d:clear(1);assert(d.ready and not s.surfaces[1] and s.surfaces[2]);e.frame=46;assert(d:update())
 d:clear();assert(not d.ready and not next(s.posts) and not next(s.surfaces))
end)
T.test('paused restore after manualclear rehydrates cached shaders without advancing metadata',function()
 local e,s,d=fixture();ready(d,s);e:equip(1,'kindling');e.frame=12;d:update();local snap=e:export();local loads=s.loads
 local warms=s.warm_calls;d:clear();e:import(snap);assert(d:on_loadstate(e));d:tick(e);assert(s.warm_calls==warms,'restore must not traverse gd.warm');assert(s.params[1][1]==12/60,"clock");assert(e:export()==snap,"metadata changed: "..e:export().." vs "..snap);assert(s.loads==loads+1,"loads "..s.loads.." vs "..loads)
 d:scene_end();assert(not d.shader)
end)
T.test('intensity zero disables pulse; finite controls',function()
 local e,s,d=fixture();ready(d,s);d:intensity(0);e.frame=1;e.trace={'burn applied by Kindling','curse applied by Pyre'};e.display.trace_generation=1;d:update()
 assert(s.params[1][2]==0 and not next(s.posts));T.refuses(function() d:intensity(0/0) end);T.refuses(function() d:intensity(2) end)
end)
T.test('drive rarity colour and checkpoint preserve visible equipment',function()
 local e,s,d=fixture();ready(d,s)
 e.display.drive_build={[1]={{colour='red',rarity='common'}}};d:update();assert(s.params[1][11]==0)
 e.display.drive_build[1][1].rarity='magic';d:update();assert(s.params[1][11]==.15 and s.params[1][10]==1.02)
 e.display.drive_build[1][1].rarity='rare';d:update();assert(s.params[1][11]==.35)
 e.display.drive_build[1][1].rarity='unique';d:update();assert(s.params[1][11]==.6)
 local snap=e:export();e.display.drive_build={};e:import(snap);d:on_loadstate(e);assert(s.params[1][11]==.6)
end)
T.done()
