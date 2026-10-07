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
  pause=function() s.paused=true end,resume=function() s.paused=false end,
  fx_world=function(name) s.fx=(s.fx or 0)+1;s.fx_name=name;return s.fx end,
  fx_move=function() end,fx_control=function() end,fx_end=function() end}
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
T.test('retail results returns asset-free menu without scene or mission launch',function()
 local s,a=fixture(true);assert(a:command('start'));a:retail_event('game_over',s.mode)
 local launches=0;a.mission.command=function() launches=launches+1;error('asset-free return must not launch garden') end
 a:menu_effect(a.menu:input('accept',a:context()))
 assert(a.menu.screen=='hub' and a.visible and not a.hub.active and launches==0)
 assert(a.menu:entries(a:context())[1].target=='setup')
end)
T.test('retail reward invokes existing juice and visual once after save success',function()
 local s,a=fixture(true);assert(a:command('start'));a:retail_event('stage_clear',s.mode)
 local sounds,pulses=0,0;s.a.g.play_sound=function(id) assert(id==170 or id==250);sounds=sounds+1 end
 a.visual.pickup=function() pulses=pulses+1 end
 s.disk=true;a:menu_effect{type='reward',index=1};assert(sounds==0 and pulses==0 and not s.fx)
 s.disk=false;a:menu_effect{type='reward',index=1};assert(sounds==1 and pulses==1 and next(a.flashes) and s.fx==1 and s.fx_name:find('collect_burst',1,true))
 a:menu_effect{type='reward',index=1};assert(sounds==1 and pulses==1 and s.fx==1)
end)
T.test('garden is offered only after model resolution and spawn refusal returns menu',function()
 local s,a=fixture(true);a.menu:show('hub');assert(a:enter_hub());assert(not a:context().garden_available)
 local releases=0;s.a.g.model_load=function() return 77 end;s.a.g.model_release=function() releases=releases+1;return true end
 assert(a.hub:resolve() and a.hub.asset==77);a.hub:clear();assert(releases==1)
 a.hub.active=true;a.hub.waiting=true;a.mission.current={doc={name='hub'}};a.g.model_spawn=function() return false end;a.mission.stop=function() s.stopped=true end
 a:frame();assert(a.menu.screen=='hub' and not a.hub.active and s.stopped and not a.garden_available)
end)
T.test('stage tag and opponent marker draw in rolled colour plus Guard hit flash',function()
 local s,a=fixture(true);assert(a:command('start'));a:retail_event('stage_start',s.mode)
 local texts={};a.g.kit={available=function() return true end,panel=function() end,text=function(_,_,label,_,colour) texts[#texts+1]={label=label,colour=colour} end}
 a.g.safe_area=function() return {x=0,y=0,w=960,h=540,right=960} end;a.g.fill=function() end
 a.g.player=function() return {x=0,y=0} end;a.g.project=function() return 100,100,true end
 a.retail.guard_flash=12;a:draw();local marker,guard=false,false
 for _,v in ipairs(texts) do if v.label==a.retail.tags[1].label and v.colour==a.retail.tags[1].colour then marker=true end;if v.label=='GUARD' then guard=true end end
 assert(marker and guard)
end)
T.test('final acknowledged reward returns menu even with resolved garden',function()
 local s,a=fixture(true);assert(a:command('start'));s.mode.final=true
 a:retail_event('stage_clear',s.mode);a:retail_event('complete',s.mode);a:menu_effect{type='reward',index=1}
 local ngplus=D.companion.tuning.retail.ngplus;D.companion.tuning.retail.ngplus=false;a:menu_effect{type='reward_done'};D.companion.tuning.retail.ngplus=ngplus
 assert(a.menu.screen=='results' and not a.retail.active)
 a.g.model_load=function() return 77 end;a.mission.command=function() error('return menu must not enter resolved garden') end
 a:menu_effect(a.menu:input('accept',a:context()));assert(a.menu.screen=='hub' and not a.hub.active)
end)
T.test('START at the retail results screen and on a stage start already held does not open Envoy menu or pause',function()
 local s,a=fixture(true);assert(a:command('start'));a:retail_event('stage_start',s.mode)
 -- stage clear with the results up: one START must not open the menu, must not pause
 a.menu:show('playing');a.visible=true;a:retail_event('stage_clear',{stage_index=0,loop=0});a.menu:show('playing');a.retail.reward=nil
 a.results_up=true;s.pad={START=true};a:tick();assert(a.menu.screen=='playing' and not s.paused,'START at the results screen opened the menu: '..a.menu.screen)
 s.pad={};a:tick();s.pad={START=true};a:tick();assert(a.menu.screen=='playing' and not s.paused)
 -- the next stage starts with START still held (scene change): no menu until it is released and pressed again
 s.mode.held=false;s.pad={START=true};a:retail_event('stage_start',s.mode);a.menu:show('playing');a.visible=true;a:tick();a:tick()
 assert(a.menu.screen=='playing' and not s.paused,'held START at a stage start opened the menu')
 s.pad={};a:tick();assert(a.menu.screen=='playing');s.pad={START=true};a:tick()
 assert(a.menu.screen=='pause' and s.paused,'a fresh START during the stage must still open the pause menu')
end)
T.test('a team stage is decided when its foes are out, though the human CPU teammate still has stocks (Atlas proof D11)',function()
 local s,a=fixture(true);assert(a:command('start'));a:retail_event('stage_start',s.mode)
 local pl={[1]={stocks=3,team=0},[2]={stocks=2,team=0},[3]={stocks=1,team=1}};a.g.player=function(p) return pl[p] end
 s.pad={};a:tick();a:tick();assert(a.menu.screen=='playing' and not a.results_up,'a live foe: not decided')
 pl[3].stocks=0;a:tick();assert(a.results_up,'the foe is out; the teammate still stands: decided')
 s.pad={START=true};a:tick();assert(a.menu.screen=='playing' and not s.paused,'START at the team stage results reached the game, not the Envoy pause')
 -- an engine without teams (no team field): a foe-less field still counts the old way
 a:retail_event('stage_start',s.mode);pl={[1]={stocks=3},[2]={stocks=1}};s.pad={};a:tick();a:tick();assert(not a.results_up);pl[2].stocks=0;a:tick();assert(a.results_up)
end)
T.test('a clear the game accepted closes a pause Envoy opened a few frames before (Atlas proof D11)',function()
 local s,a=fixture(true);assert(a:command('start'));a:retail_event('stage_start',s.mode)
 s.pad={};a:tick();a:tick();s.pad={START=true};a:tick();assert(a.menu.screen=='pause' and s.paused,'a live stage pauses')
 a:retail_event('stage_clear',s.mode);assert(a.menu.screen~='pause' and not s.paused,'the pause did not outlive the stage it was opened in: '..a.menu.screen)
end)
T.test('the results screen is read from game state: a decided stage ignores START, a live one still pauses',function()
 local s,a=fixture(true);assert(a:command('start'));a:retail_event('stage_start',s.mode)
 local pl={[1]={stocks=3},[2]={stocks=1}};a.g.player=function(p) return pl[p] end
 s.pad={};a:tick();a:tick();assert(a.menu.screen=='playing' and not a.results_up)
 s.pad={START=true};a:tick();assert(a.menu.screen=='pause' and s.paused,'a live stage opens the pause menu on a fresh START')
 a.menu:show('playing');a.g.resume();s.paused=false;a.owns_pause=nil;s.pad={};a:tick()
 pl[2].stocks=0;a:tick();assert(a.results_up,'every opponent out of stocks: the stage is decided')
 s.pad={START=true};a:tick();assert(a.menu.screen=='playing' and not s.paused,'one START at the results screen must reach the game')
 s.pad={};a:tick();a:retail_event('stage_start',s.mode);assert(not a.results_up and not a.seen_foe)
 -- P1 out of stocks, and an engine hold, also count
 pl={[1]={stocks=0},[2]={stocks=2}};a:tick();assert(a.results_up)
 a:retail_event('stage_start',s.mode);pl={[1]={stocks=3}};a.g.mode_1p=function() return {held=true} end;s.pad={};a:tick();a:tick();assert(a.results_up)
end)
T.test('Adventure: an intro-skip START still held at every stage start never opens the pause menu (one fresh press does)',function()
 local s,a=fixture(true);a.menu.run_type='adventure';assert(a:command('adventure'));s.mode.mode='adventure'
 for stage=0,3 do
  s.mode.stage_index=stage;s.mode.held=false;s.pad={START=true};a:retail_event('stage_start',s.mode);a.menu:show('playing');a.visible=true;a:tick();a:tick()
  assert(a.menu.screen=='playing' and not s.paused,'held START at Adventure stage '..stage..' paused the game')
  s.pad={};a:tick();assert(a.menu.screen=='playing')
 end
 s.pad={START=true};a:tick();assert(a.menu.screen=='pause' and s.paused,'a fresh START must still pause')
end)
T.test('a run start issued in the opening movie / demo waits for a settled title, survives the scene changes, then launches (2026-10-05 hang)',function()
 local s,a=fixture(true);local scene={name='GS_MOVIE_OPENING',mode_name='GM_OPENING_MV',epoch=1};local frame=100
 a.g.scene=function() return scene end;a.g.frame=function() return frame end
 local function logs() local t={};a.g.log=function(x) t[#t+1]=x end;return t end;local log=logs()
 assert(a:command('classic'));assert(not s.launch and a.retail_request and a.retail_request.scene_wait,'deferred in the movie')
 assert(table.concat(log,' '):find('run start deferred',1,true),'said so')
 for _=1,10 do a:tick() end;assert(not s.launch)
 -- the movie ends: the scene change reaches the app as a match end, which must not drop the request nor count as a run's match end
 scene={name='GS_VS',mode_name='GM_OPENING_MV',epoch=2};frame=700;a:match_end();assert(a.retail_request and not s.launch and not a.retail.active,'request survives the scene change')
 for _=1,10 do frame=frame+1;a:tick() end;assert(not s.launch,'not in the attract demo')
 scene={name='GS_TITLE',mode_name='GM_OPENING_MV',epoch=3};frame=1500;a:match_end();a:tick();assert(not s.launch,'title just entered: settling')
 frame=1500+30;a:tick();assert(not s.launch);frame=1500+470;a:tick();assert(not s.launch,'title about to give way to the demo')
 frame=1500+120;a:tick();assert(s.launch and s.launch.mode=='classic' and a.retail.active,'launched from the settled title')
 assert(not a.retail_request)
end)
T.test('a scene end before the run reached a stage is not a match end of the run',function()
 local s,a=fixture(true);assert(a:command('start'));assert(a.retail.active and not a.retail.stage_started)
 a.results_up=nil;a:match_end();assert(not a.results_up and a.retail.active,'ignored')
 a:retail_event('stage_start',s.mode);assert(a.retail.stage_started);a:match_end();assert(a.results_up,'a real match end')
end)
T.test('envoy start: depth/loop/build give a consistent run (slots, keystone allowance, rolled build) and bad arguments are refused',function()
 local s,a=fixture(true)
 assert(not a:command('start classic mario depth=x'));assert(not a:command('start classic mario bogus=1'));assert(not a:command('start sprint mario'));assert(not s.launch)
 local ok,why=a:command('start classic mario depth=12 loop=1 build=77');assert(ok,why);assert(s.launch.mode=='classic' and a.retail.rules)
 local h=a.retail.host
 if h then
  local b=h:bag();local ctx=h.mods.engine.context
  assert(ctx.depth==12 and ctx.loop==1,'context follows the request');assert(b:slots()==6)
  assert(h:equipped_count()==6,'every slot filled');assert(#h:keystone_ids()==D.mod_progression.keystones(ctx),'keystones up to the allowance')
 end
end)
T.done()
