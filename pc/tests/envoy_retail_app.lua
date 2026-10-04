local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();for _,k in ipairs({'save','drives','drive_models','fighter','campaign','hub','run','classic','hud','menu','menu_draw','menu_input','recolour','visual'}) do D[k]=T.module(k,D) end
T.test('retail app integration exists',function() local f=io.open(T.root..'retail_app.lua');assert(f,'retail app glue missing');f:close() end)
D.retail_app=T.module('retail_app',D);D.app=T.module('app',D)
local function fixture(engine)
 local s={text=D.save.encode(D.save.new_profile()),pad={},commands={},writes=0,mods={},mode={mode='classic',stage_index=0,loop=0,player_port=1,opponents={{port=2}}}}
 local g={command=function(n,f) s.commands[n]=f end,log=function() end,data_read=function() return s.text end,
  data_write_atomic=function(_,v) if s.disk then return false,'disk refused' end;s.writes=s.writes+1;s.text=v;return true end,
  match=function() return {active=true,netplay=false} end,player=function() return {} end,
  pad=function() return s.pad end,input_mask=function() end,paused=function() return false end,
  pause=function() s.paused=true end,resume=function() s.paused=false end}
 if engine then
  g.mode_1p=function() return s.mode end;g.start_1p=function(v) s.launch=v;return true end
  g.spawn_1p=function(p,v) s.mods[p]=v;return true end;g.loop_1p=function(v) s.loop=v;return true end;g.end_1p=function() s.ended=true;return true end
  g.hold_1p=function() s.mode.held=true;return true end;g.release_1p=function() s.mode.held=false;return true end
 end
 local mission={frame=function() end,draw=function() end,stop=function(m) m.current=nil end}
 s.a=D.app.new(g,mission);return s,s.a
end
T.test('hub start launches Classic Mario Normal three stocks',function()
 local s,a=fixture(true);a:menu_effect{type='start',fighter='mario',mode='classic',difficulty=2,stocks=3}
 assert(s.launch.mode=='classic' and s.launch.fighter=='mario' and s.launch.difficulty==2 and s.launch.stocks==3)
 assert(a.retail.active and not a.run.active);a:retail_event('stage_start',s.mode);a:match_end()
 assert(a.retail.active and s.writes==1,'retail match end must not settle a stage')
end)
T.test('controller reward uses hold not pause and ignores B then previews once',function()
 local s,a=fixture(true);assert(a:command('start'));a:retail_event('stage_clear',s.mode);a:tick()
 assert(a.menu.screen=='reward' and not s.paused);s.pad={B=true};a:tick();assert(a.retail.reward)
 s.pad={};a:tick();s.pad={A=true};a:tick();assert(a.retail.reward.preview);local writes=s.writes
 a:tick();assert(s.writes==writes);s.pad={};a:tick();s.pad={A=true};a:tick();assert(not a.retail.reward and a.menu.screen=='playing')
end)
T.test('old engine shows unavailable no mission or LAB fallback',function()
 local s,a=fixture(false);local ok=a:command('start');assert(not ok and not a.run.active and not s.launch)
 assert(a.notice:find('unavailable',1,true) and a.menu.screen=='setup' and s.writes==0)
end)
T.test('mode selection switches Classic Adventure with controller',function()
 local _,a=fixture(true);a.menu:show('setup');a.menu.focus.setup=2;a:menu_effect(a.menu:input('accept',a:context()))
 assert(a.menu.run_type=='adventure');a.menu.focus.setup=1;a:menu_effect(a.menu:input('accept',a:context()));assert(a.retail.mode=='adventure')
end)
T.test('game over shows results and unload clears every template',function()
 local s,a=fixture(true);assert(a:command('start'));a:retail_event('stage_start',s.mode);assert(next(s.mods))
 a:retail_event('game_over',s.mode);assert(not a.retail.active and a.menu.screen=='results' and next(s.mods)==nil)
 local writes=s.writes;a:unload();assert(s.writes==writes and next(s.mods)==nil)
end)
T.test('completion disk refusal exposes controller retry with no phantom loop',function()
 local s,a=fixture(true);assert(a:command('start'));s.disk=true;a:retail_event('complete',s.mode)
 assert(a.menu.screen=='results' and a.retail.pending and a:context().retail_pending and s.ended)
 local e=a.menu:input('accept',a:context());assert(e.type=='retail_retry');a:menu_effect(e);assert(a.retail.pending)
 s.disk=false;a:menu_effect(e);assert(not a.retail.pending and a.run.profile.pending_seed==nil and a.run.profile.records.wins==1)
end)
T.test('companion recolour refreshes after respawn during retail frames',function()
 local _,a=fixture(true);assert(a:command('start'));local ticks=0
 a.recolour.tick=function() ticks=ticks+1 end;a:frame();a:frame();assert(ticks==2)
end)
T.test('final reward timeout becomes terminal win and exposes refused save retry',function()
 local s,a=fixture(true);assert(a:command('start'));s.mode.final=true
 a:retail_event('stage_clear',s.mode);a:retail_event('complete',s.mode);s.disk=true;s.mode.held=false;a:tick()
 assert(not a.retail.active and not a.retail.awaiting_loop and a.retail.pending and a.menu.screen=='results')
 s.disk=false;s.pad={A=true};a:tick();assert(not a.retail.pending and a.run.profile.records.wins==1)
end)
T.done()
