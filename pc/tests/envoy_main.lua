local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();for _,k in ipairs({'save','drives','drive_models','fighter','campaign','hub','run','hud','menu','menu_draw','menu_input','recolour','visual','app'}) do D[k]=T.module(k,D) end
local function fixture(text)
  local s={text=text or D.save.encode(D.save.new_profile()),writes=0,commands={},logs={},p1={x=0,y=0,falls=0},p2={cpu=true,falls=0},pad={},mods={},masks={}}
  local g={data_read=function() return s.text end,data_write_atomic=function(_,v) s.writes=s.writes+1;s.text=v;return true end,
    command=function(k,f) s.commands[k]=f end,log=function(t) s.logs[#s.logs+1]=t end,
    match=function() return {active=true,netplay=s.online} end,player=function(n) if n==1 then return s.p1 elseif n==2 then return s.p2 end end,
    enemy_state=function(h) return {kind='goomba',x=0,y=0} end,cpu_mode=function() return true end,
    teleport=function() return true end,fighter_mod=function(_,v) s.mods[#s.mods+1]={value=v};return true end,
    fighter_benched=function() return s.benched,{{present=true,refusal_code=0}} end,
    fighter_bench=function() s.benched=true;return true end,fighter_call=function() s.benched=false;return true end,
    stage_spawn=function() return 0,30 end,pad=function() return s.pad end,input_mask=function(_,mask) s.masks[#s.masks+1]=mask end,
    set_damage=function(n,v) (n==1 and s.p1 or s.p2).percent=v end,
    paused=function() return s.paused end,pause=function() s.paused=true end,resume=function() s.paused=false end}
  local m={command=function(self,a) self.current={doc={name=a:match('play (.+)') or ('maze_'..assert(a:match('maze (%d+)'))..'_'..assert(a:match('maze %d+ (%d+)')))},run={state={tracked={[1]={}}}}};return true end,
    frame=function() end,stop=function(self) self.current=nil end,draw=function() end}
  s.g=g;local a=D.app.new(g,m,function() return .1 end);return s,a
end
T.test('console start give status stop and confirm reset',function()
  local s,a=fixture();assert(s.commands.envoy('start'));assert(s.commands.envoy('give red 5'))
  assert(a.run.companion.stats.power.points==100);assert(s.commands.envoy('status'))
  assert(s.commands.envoy('stop'));assert(s.writes==2)
  assert(not s.commands.envoy('reset-profile'));assert(s.writes==2)
  assert(s.commands.envoy('reset-profile confirm'));assert(s.writes==3)
end)
T.test('native defeat feeds by touch; lifecycle cannot settle twice',function()
  local s,a=fixture();a:command('start');a:defeated({handle=1,kind='goomba'});a:frame()
  assert(a.run.companion.stats.power.points>0);a:stop('quit');a:stop('quit');assert(s.writes==2)
end)
T.test('corrupt profile blocks start give and even confirmed reset',function()
  local s,a=fixture('bad');assert(not a:command('start'));assert(not a:command('give red 2'))
  assert(not a:command('reset-profile confirm'));assert(s.writes==0)
  assert(a:command('menudump') and a:command('dump'));assert(s.writes==0)
end)
T.test('invalid and online commands do not mutate state',function()
  local s,a=fixture();for _,arg in ipairs({'give red -1','give red 1.5','give purple 1','give red 2 junk','start junk','wat'}) do
    assert(not a:command(arg))
  end
  s.online=true;assert(not a:command('give red 1'));assert(s.writes==0)
end)
T.test('menus navigate while paused and release gameplay on resume',function()
  local s,a=fixture();a:command('menu');a:tick();assert(s.paused)
  a.menu:show('hub');a.menu.focus.hub=2;a.menu:input('accept',a:context());a.menu:input('accept',a:context())
  local effect=a.menu:input('accept',a:context());a:menu_effect(effect)
  assert(a.run.active and not s.paused and a.menu.screen=='playing' and s.benched)
  s.pad={START=true};a:tick();assert(s.paused and a.menu.screen=='pause')
  s.pad={};a:tick();s.pad={A=true};a:tick();assert(not s.paused and a.menu.screen=='playing')
  a:stop('quit');assert(a.menu.screen=='results' and s.mods[#s.mods].value==nil)
end)
T.test('native pickup feeds results and immediately reapplies a first level',function()
  local s,a=fixture();a:command('start');local n=#s.mods
  a:defeated({handle=1,kind='goomba'});a:frame();assert(#s.mods==n+1)
  a:stop('quit');assert(a.run.results.drives.power==1 and a.run.results.points.power==20)
end)
T.test('quit hides navigation until explicitly reopened',function()
  local s,a=fixture();a:menu_effect({type='quit'})
  for _=1,10 do s.pad={A=true};a:tick();s.pad={};a:tick() end
  assert(a.menu.screen=='title' and not a.run.active and not s.paused)
  assert(a:command('menu'));a:tick();assert(s.paused)
end)
T.test('scene launch waits for both actors and recovers refused start',function()
  local s,a=fixture();s.p1.char_name='Fox'
  s.g.scene_launch=function(v) s.scene=v;return true end
  a:menu_effect({type='start',fighter='marth'});assert(s.scene.p1=='marth' and s.scene.mode=='lab' and s.scene.p2=='falco/cpu0' and not a.run.active)
  a:match_end();assert(s.writes==0)
  a:match_start();local p2=s.p2;s.p2=nil;a:tick();assert(not a.run.active and a.launching)
  s.p2=p2;a:tick();assert(a.run.active and not a.launching)
  a:stop('quit')
  a:menu_effect({type='start',fighter='marth'});a:match_start()
  s.g.fighter_mod=function() return false end;a:tick();assert(not a.run.active and a.menu.screen=='setup')
end)
T.test('enabled mod leaves unrelated offline matches untouched',function()
  local s,a=fixture();s.pad={A=true,UP=true,START=true};a:tick();a:frame()
  assert(a.visible==false and not s.paused and #s.masks==0 and not a.run.active and #s.mods==0)
  a:match_start();a:match_end();a:tick();assert(not s.paused and #s.masks==0 and s.writes==0)
end)
T.test('corrupt profile close releases its owned pause and mask',function()
  local s,a=fixture('bad');a:command('menu profile');a:tick();assert(s.paused)
  a.menu:input('down',a:context());a:menu_effect(a.menu:input('accept',a:context()))
  assert(a.visible==false and not s.paused);s.pad={};a:tick();assert(s.masks[#s.masks]==0 and s.writes==0)
end)
T.test('supported dumps are read only and records include ledger and growth',function()
  local s,a=fixture();assert(a:command('menudump'));assert(a:command('dump'))
  local log=table.concat(s.logs,'\n');assert(log:find('screen=title',1,true) and log:find('disabled=',1,true))
  assert(log:find('modifiers applied=false',1,true) and log:find('tint=',1,true))
  assert(s.writes==0 and #s.mods==0 and #s.masks==0)
  a:command('start');a:command('give blue 50');a:effects()
  assert(D.companion.effects(a.run.companion).shield_max>1.1 and #s.mods>1)
  a:stop('quit');a:command('status');assert(s.logs[#s.logs-5]:find('room=0',1,true) or table.concat(s.logs,'\n'):find('active=false room=0',1,true))
  assert(#a:context().records>=6)
end)
T.test('optional models replace glyph per pickup and pulse collection then clear',function()
  local s,a=fixture();local serial=0;local models={};local glyphs=0
  s.g.model_load=function(path) assert(path:find('envoy_drives_sa2/',1,true));return path end
  s.g.model_spawn=function() serial=serial+1;models[serial]=true;return serial end
  s.g.model_set=function(h) assert(models[h]);return true end
  s.g.model_despawn=function(h) models[h]=nil;return true end
  s.g.model_release=function() end
  s.g.safe_area=function() return {x=0,y=0,w=853,h=480,right=853,bottom=480} end
  s.g.fill=function() end;s.g.text=function(_,_,text) if text=='<>' then glyphs=glyphs+1 end end
  s.g.project=function(x,y) return x,y,true end
  a:command('start');s.p1.x=100;a:defeated({handle=1,kind='goomba'});a:frame()
  assert(serial==2);a:draw();assert(glyphs==0)
  s.p1.x=0;a:frame();assert(#a.drives.pickups==0 and #a.models.dying==1)
  for _=1,14 do a:frame() end;assert(next(models)==nil)
  a:stop('quit');a:unload();assert(next(models)==nil and s.writes==2)
end)
T.test('menu entry always launches LAB even with selected fighter already present',function()
  local s,a=fixture();s.p1.char_name='Fox';s.g.scene_launch=function(v) s.scene=v;return true end
  a:menu_effect({type='start',fighter='fox'})
  assert(s.scene and s.scene.mode=='lab' and s.scene.p2=='falco/cpu0' and not a.run.active)
end)
T.test('two scene generations reload optional model assets instead of stale handles',function()
  local s,a=fixture();local generation=1;local loads=0;local id=0;local models={}
  s.g.model_load=function(path) loads=loads+1;return {generation=generation,path=path} end
  s.g.model_spawn=function(asset) assert(asset.generation==generation,'stale or released model handle');id=id+1;models[id]=true;return id end
  s.g.model_set=function(h) assert(models[h]);return true end
  s.g.model_despawn=function(h) models[h]=nil;return true end
  s.g.model_release=function(asset) if asset.generation~=generation then error('stale or released model handle') end end
  a:command('start');s.p1.x=100;a:defeated({handle=1,kind='goomba'});a:frame();assert(loads==6 and id==2)
  generation=2;models={};a:match_end();a:match_start()
  a:command('start');s.p1.x=100;a:defeated({handle=1,kind='goomba'});a:frame()
  assert(loads==12 and id==4 and a.models:active())
  a:stop('quit');a:unload();assert(s.writes==4 and next(models)==nil)
end)
T.test('cancelled staging drains cleanup before results pause can hold it',function()
  local s,a=fixture();a:command('start');a.menu:show('pause');a:tick();assert(s.paused)
  local frames=0;a.mission.stop=function(m) m.staging={phase='rollback'};m.current=nil end
  a.mission.frame=function(m) frames=frames+1;if frames==3 then m.staging=nil end end
  a:stop('quit');a:tick();assert(not s.paused)
  for _=1,3 do a:frame();a:tick() end
  assert(frames==3 and not a.mission.staging and s.paused and s.writes==2)
end)
T.test('second scene does not start run while actors are entering or landing',function()
  local s,a=fixture();a.launching='falco';a:match_start()
  s.p1.action=323;s.p2.action=14;a:tick();assert(not a.run.active and a.launch_ready)
  s.p1.action=42;a:tick();assert(not a.run.active and a.launch_ready)
  s.p1.action=14;s.p2.action=324;a:tick();assert(not a.run.active and a.launch_ready)
  s.p2.action=14;a:tick();assert(a.run.active and not a.launch_ready)
end)
T.test('garden stations pause screens and return to physical garden',function()
 local s,a=fixture();assert(a:command('hub'));a:frame();a:tick()
 assert(a.hub.active and not a.hub.waiting and a.menu.screen=='garden' and not s.paused)
 s.pad={A=true};a:tick();assert(a.menu.screen=='companion' and s.paused)
 s.pad={};a:tick();s.pad={B=true};a:tick();assert(a.menu.screen=='garden' and not s.paused)
 s.pad={};a:tick();s.p1.x=180;s.pad={A=true};a:tick()
 assert(a.menu.screen=='garden' and a.notice:find('slice 5',1,true))
 s.pad={};a:tick();s.p1.x=90;s.pad={A=true};a:tick();assert(a.menu.screen=='records' and s.paused)
 a:menu_effect({type='hub'});assert(a.menu.screen=='garden' and not s.paused and s.writes==0)
end)
T.test('app interlude continue advances room and depth with visible levelup flash',function()
 local s,a=fixture();assert(a:command('start'));assert(a.run.index==1 and s.writes==1)
 a:defeated({handle=1,kind='goomba'});a:frame();assert(a.flashes.power==D.companion.tuning.levelup_frames)
 local texts={};s.g.safe_area=function() return {x=0,y=0,w=853,h=480,right=853} end
 s.g.fill=function() end;s.g.text=function(x,y,t) texts[#texts+1]=t end;s.g.project=function(x,y) return x,y,true end
 a:draw();assert(table.concat(texts,' '):find('LEVEL UP!',1,true))
 a.mission.current.run.state.result={status='complete'};a:frame()
 assert(a.menu.screen=='interlude' and s.paused and a.run.index==1)
 a:menu_effect(a.menu:input('back',a:context()));assert(a.menu.screen=='interlude' and a.run.index==1)
 a:menu_effect(a.menu:input('accept',a:context()));a:frame()
 assert(a.menu.screen=='playing' and not s.paused and a.run.index==2 and a.drives.depth==2)
 s.p1.x=100;a.mission.current.run.state.tracked={[2]={}};a:sample();a:defeated({handle=2,kind='goomba',x=0,y=0});assert(a.drives.pickups[1].amount==25)
 s.p1.x=0;local before=a.run.companion.stats.power.points;a:frame();assert(a.run.companion.stats.power.points-before==25)
 for _=1,D.companion.tuning.levelup_frames do a:frame() end;assert(a.flashes.power==nil)
 a:stop('quit');assert(s.writes==2 and a.menu.screen=='results')
 a:menu_effect(a.menu:input('accept',a:context()));assert(a.hub.active and a.menu.screen=='garden' and not s.paused)
end)
T.test('quitting garden releases its level and prevents station input',function()
 local s,a=fixture();assert(a:command('hub'));a:frame();a:tick();a.menu:show('hub');a:sync_pause();assert(s.paused)
 a:menu_effect({type='quit'});assert(not a.visible and not a.hub.active and not a.mission.current and not s.paused)
 for _=1,3 do s.pad={A=true};a:tick();a:frame();s.pad={};a:tick() end
 assert(not a.hub.active and s.writes==0)
end)
T.test('failed start returns an explanatory usable menu without owned state',function()
  local s,a=fixture();a.mission.command=function() return false,'room assets missing' end
  a:command('menu');a:tick();assert(s.paused)
  assert(not a:command('start'));a:frame();a:tick()
  assert(a.notice and a.notice:find('Run could not start',1,true))
  assert(a.menu.screen=='setup' and not s.paused and not s.benched)
  assert(not a.run.fighter.started and not a.run.active and s.writes==0)
  assert(a:command('menu hub'))
end)
T.test('shared staging cover draws while the garden menu is open',function()
 local s,a=fixture();local draws=0;a.mission.draw=function() draws=draws+1 end
 a.mission.staging={phase='warm'};a.menu:show('garden');a.hub.active=true;a.visible=true
 a:draw();assert(draws==1,'shared staging needs its loading cover')
end)
T.done()
