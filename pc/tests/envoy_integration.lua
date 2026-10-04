-- Loads the real generated entry and unchanged mission runtime, not a fake coordinator.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local root=T.root:gsub('scripts/$','')
local function fixture(fresh,no_kit)
  local D=T.rules();local S=T.module('save',D)
  local s={text=S.encode(S.new_profile()),commands={},logs={},enemies={},id=0,areas={},spawns={},writes=0,
    p={{x=-160,y=12,falls=0,percent=39},{x=-100,y=12,falls=0,cpu=true,percent=116}},active=true,masks={},models={},assets={},area_models={}}
  if fresh then s.text=nil end
  local function id() s.id=s.id+1;return s.id end
  local g={}
  g.mod_read=function(p) local f=io.open(root..p);if not f then return nil,'missing' end;local t=f:read('a');f:close();return t end
  g.mod_stamp=function(p) local f=io.open(root..p);if f then f:close();return 1 end end
  g.mod_list=function(p) assert(p:sub(1,9)=='missions/' and not p:find('..',1,true) and not p:find('\\',1,true) and not p:find(':',1,true),'engine path refused: '..p);if p=='missions/' then return {{name='path',dir=true},{name='boss',dir=true},{name='hub',dir=true}} end;if (p=='missions/models/' or p=='missions/hub/models/') and not no_kit then
    local rows={};for _,name in ipairs({'bf_floor_4m','bf_wall_solid_4m','bf_wall_doorway_4m','bf_door_leaf','bf_beam_4m'}) do rows[#rows+1]={name=name..'.gxmesh',dir=false} end;return rows
    end;local marker=io.open(root..p..'README.md');if marker then marker:close();return {} end;return nil,'directory missing' end
  g.command=function(n,f) s.commands[n]=f end
  g.data_read=function() if s.text==nil then return nil,'missing' end;return s.text end
  g.data_exists=function() return s.text~=nil end
  g.data_write_atomic=function(_,v) s.writes=s.writes+1;if s.disk then return false,'disk full' end;s.text=v;return true end
  g.log=function(t) s.logs[#s.logs+1]=t end
  g.match=function() return {active=s.active,netplay=false} end
  g.player=function(n) if not s.p[n] then return end;local p={};for k,v in pairs(s.p[n]) do p[k]=v end;return p end
  g.enemy_state=function(h) local e=s.enemies[h];return e and e.status=='alive' and e or nil end
  g.enemy_status=function(h) return s.enemies[h] and s.enemies[h].status or 'gone' end
  g.enemy_remove=function(h) s.enemies[h]=nil;return true end
  g.spawn_enemy=function(k,x,y) local h=id();s.enemies[h]={kind=k,x=x,y=y,status='alive'};return h end
  g.area_load=function(n,f) s.area=n;s.area_models[n]={};f();s.area=nil;s.areas[n]=true;return true end
  g.area_unload=function(n) for h in pairs(s.area_models[n] or {}) do s.models[h]=nil end;s.area_models[n]=nil;s.areas[n]=nil;return true end
  g.stage_add_line=function() return id() end
  g.stage_move=function() return true end
  g.stage_bounds=function() return {camera={left=-300,right=300,top=300,bottom=-300},blast={left=-400,right=400,top=400,bottom=-400}} end
  g.stage_spawn=function(n) local p=s.spawns[n] or {x=0,y=10};return p.x,p.y end
  g.stage_set_spawn=function(n,x,y) s.spawns[n]={x=x,y=y};return true end
  g.stage_set_camera_bounds=function() return true end
  g.stage_set_blast_bounds=function() return true end
  g.stage_restore_bounds=function() return true end
  g.floor_below=function(_,y)return y-0.25 end
  g.stage_hide=function() return true end
  g.teleport=function(n,x,y) s.p[n].x=x;s.p[n].y=y;return true end
  g.fly=function() return true end;g.fly_attack=function() return true end
  g.fly_target=function(n,x,y) s.p[n].x=x;s.p[n].y=y;return true end
  g.set_stocks=function() return true end;g.cpu_mode=function() return true end
  g.set_damage=function(n,v) s.p[n].percent=v end
  g.fighter_mod=function(_,v) s.mod=v;return true end
  s.fx={}
  g.fx_world=function(name,x,y,z) local h=id();s.fx[h]={name=name};return h end
  g.fx_move=function(h) assert(s.fx[h]);return true end
  g.fx_control=function(h) assert(s.fx[h]);return true end
  g.fx_end=function(h) s.fx[h]=nil;return true end
  g.play_sound=function() end
  s.items={};s.item_spawn=function(_,x,y,o) local h=id();s.spawned=(s.spawned or 0)+1;o.x=x;o.y=y;s.items[h]=o;return h end
  g.items=function() local rows={};for h,o in pairs(s.items) do rows[#rows+1]={handle=h,x=o.x,y=o.y,z=0,visual_y=o.y+6,rotation=90,age=0,visible=true} end;return rows end
  g.item_despawn=function(h) s.items[h]=nil;return true end
  g.parts=function() return {geometry_signature='stub',{index=0,path='body',joint=1,source=0,regions={torso=1}}} end
  g.parts_tint=function() s.tint=true;return true end
  g.parts_clear=function() s.tint=false;return true end
  g.model_load=function(path) if no_kit or not path:find('bf_',1,true) then return nil,'missing fixture asset' end;local h=id();s.assets[h]=path;return h end
  g.model_spawn=function(a,o) assert(s.assets[a]);local h=id();s.models[h]=o;if s.area then s.area_models[s.area][h]=true end;return h end
  g.model_get=function(h) return s.models[h] end
  g.model_set=function(h,o) if not s.models[h] then return false end;for k,v in pairs(o) do s.models[h][k]=v end;return true end
  g.model_label=function() return true end
  g.model_despawn=function(h) s.models[h]=nil;return true end
  g.model_release=function(h) s.assets[h]=nil;if h==77 then s.assets.old=nil end;return true end
  s.reserve={}
  g.fighter_benched=function(n) return s.reserve[n],{{present=true,refusal_code=0}} end
  g.fighter_bench=function(n) s.reserve[n]=true;return true end
  g.fighter_call=function(n,x,y) s.reserve[n]=false;s.p[n].x=x;s.p[n].y=y;return true end
  g.camera_params=function() return {} end;g.stage_set_origin=function() return true end
  g.pad=function() return {} end;g.input_mask=function(_,mask) s.masks[#s.masks+1]=mask end
  g.paused=function() return s.paused end;g.pause=function() s.paused=true end;g.resume=function() s.paused=false end
  g.time=function() return 0 end
  g.safe_area=function() return {x=0,y=0,w=853,h=480,right=853,bottom=480} end
  g.fill=function() end;g.text=function() end;g.project=function(x,y) return x,y,true end
  local env=setmetatable({gd=g,math=setmetatable({random=function() return .1 end},{__index=math})},{__index=_G})
  local f=assert(io.open(os.getenv('ENVOY_TEST_ENTRY') or T.root..'main.lua'));local source=f:read('a');f:close();source=source:gsub('local app = envoy.app.new%(gd, mission%)','local app = envoy.app.new(gd, mission, function() return .1 end)\n__envoy_test_app = app');local ok,why=pcall(assert(load(source,'@envoy-test','t',env)));assert(ok,why)
  env.__envoy_test_app.menu.run_type="campaign" -- Explicit parked mission regression lane.
  return s,env
end
local function ready(s,e)
  local app=e.__envoy_test_app
  for _=1,600 do
    if app.run.index>0 and not app.run.request and not app.mission.staging then e.on_frame();return end
    e.on_draw();e.on_frame()
  end
  error(table.concat(s.logs,'\n'))
end
local function encounter(s,e)
  if next(s.enemies) then return end
  local current=e.__envoy_test_app.mission.current
  for _,enemy in ipairs(current.doc.mission.enemies) do
    s.p[1].x=enemy.x;s.p[1].y=enemy.y
    for _=1,40 do e.on_frame();if next(s.enemies) then return end end
  end
  error('no spatial encounter installed')
end
local function collect(s,e)
  for h,o in pairs(s.items) do
    e.on_item_collect{name='drive',port=1,item=h,payload=o.payload}
    e.on_item_collect{name='drive',port=1,item=h,payload=o.payload}
    s.items[h]=nil
  end
end
local function complete_room(s,e)
  local app=e.__envoy_test_app;local index=app.run.index
  for _=1,1000 do
    local current=app.mission.current
    if not app.run.active or app.run.interlude or app.run.index~=index then return end
    for h,en in pairs(s.enemies) do if en.status=='alive' then
      s.p[1].x=en.x;s.p[1].y=en.y;e.on_frame()
      e.on_enemy_defeated{handle=h,kind=en.kind};en.status='defeated'
    end end
    collect(s,e)
    local z=current.doc.mission.goal;s.p[1].x=z.x;s.p[1].y=z.y
    e.on_frame()
  end
  error('room did not complete '..index..'\n'..table.concat(s.logs,'\n'))
end
local function boss(s,e)
  local app=e.__envoy_test_app
  while app.run.index<#app.run.levels do
    complete_room(s,e);assert(app.run.interlude)
    app:menu_effect{type='continue'};ready(s,e)
  end
end
local function drain(e) for _=1,60 do e.on_draw();e.on_frame() end end
T.test('one folder boots lists data starts native-defeat pickup and quit persists',function()
  local s,e=fixture();assert(s.commands.mission('list'));assert(s.commands.envoy('campaign'))
  ready(s,e)
  assert(s.commands.envoy('give yellow 50'))
  encounter(s,e);local h,en=next(s.enemies);assert(h)
  s.p[1].x=en.x;s.p[1].y=en.y
  -- Repeated events cannot award the same actor twice; a white is also valid.
  e.on_enemy_defeated({handle=h,kind=en.kind});e.on_enemy_defeated({handle=h,kind=en.kind})
  e.on_frame();e.on_draw();assert(s.commands.envoy('stop'));assert(s.writes==2)
  e.on_match_end();e.on_unload();assert(s.writes==2)
  assert(e.__envoy_test_app.run.profile.companions[1].stats.jump.points>0)
end)
T.test('real path complete flows boss; earlier CPU KO never wins',function()
  local s,e=fixture();assert(s.commands.envoy('campaign'))
  ready(s,e)
  s.p[2].falls=2
  boss(s,e);assert(s.writes==1)
  local app=e.__envoy_test_app;local z=app.mission.current.doc.mission.goal
  s.p[1].x=z.x;s.p[1].y=z.y;e.on_frame();assert(s.writes==1)
  s.p[2].falls=3;e.on_frame();assert(s.writes==2 and app.run.profile.last_result=='win')
end)
T.test('settlement refusal retries without duplicate progress',function()
  local s,e=fixture();s.commands.envoy('campaign');ready(s,e);s.commands.envoy('give red 5');s.disk=true
  assert(not s.commands.envoy('stop'));assert(not s.commands.envoy('campaign'))
  assert(not s.commands.envoy('give red 5'));s.disk=false;assert(s.commands.envoy('stop'))
  assert(s.writes==3 and e.__envoy_test_app.run.profile.companions[1].stats.power.points>0)
end)
T.test('fresh install stays closed and persists the first accepted start',function()
  local s,e=fixture(true);e.on_tick();e.on_match_start();assert(not s.paused and #s.masks==0 and s.writes==0 and not s.text)
  assert(s.commands.envoy('campaign'));assert(s.p[1].percent==0 and s.p[2].percent==0 and s.writes==0)
  ready(s,e)
  assert(s.commands.envoy('stop'));assert(s.writes==2 and s.text:find('profile 1 2 1 quit',1,true))
end)
T.test('mission defeat signals award all seven kinds including shell-only Koopa once',function()
  local s,e=fixture();assert(s.commands.envoy('campaign'))
  ready(s,e);complete_room(s,e);local logStart=#s.logs
  e.__envoy_test_app:menu_effect{type='continue'};ready(s,e)
  -- Shared runtime queues native spawns over several frames.
  for _=1,20 do
    for _,en in pairs(s.enemies) do if en.status=='alive' then en.status=en.kind=='koopa' and 'shell' or 'defeated' end end
    e.on_frame()
  end
  local log=table.concat(s.logs,'\n',logStart+1);local n=0
  for _ in log:gmatch('envoy: drop ') do n=n+1 end
  assert(n==7,log)
  for h,en in pairs(s.enemies) do e.on_enemy_defeated({handle=h,kind=en.kind}) end
  e.on_enemy_defeated({handle=999,kind='koopa'});e.on_frame()
  log=table.concat(s.logs,'\n',logStart+1);n=0;for _ in log:gmatch('envoy: drop ') do n=n+1 end;assert(n==7)
  for _,en in pairs(s.enemies) do s.p[1].x=en.x;s.p[1].y=en.y;e.on_frame() end
  s.commands.envoy('stop');assert(s.writes==2 and e.__envoy_test_app.run.profile.companions[1].stats.guard.points>0)
end)
T.test('mission lost signal never awards a drive',function()
  local s,e=fixture();assert(s.commands.envoy('campaign'));local h,en
  ready(s,e);complete_room(s,e);e.__envoy_test_app:menu_effect{type='continue'};ready(s,e)
  s.logs={}
  for k,v in pairs(s.enemies) do if v.kind=='goomba' then h,en=k,v end end
  assert(h);en.y=-115;e.on_frame();en.status='gone';e.on_frame()
  assert(not table.concat(s.logs,'\n'):find('envoy: drop ',1,true))
  assert(table.concat(s.logs,'\n'):find('lost goomba',1,true));s.commands.envoy('stop')
end)
T.test('retired resources drain and pause freezes a live run',function()
  local s,e=fixture();assert(s.commands.envoy('campaign'));ready(s,e)
  local app=e.__envoy_test_app;assert(app,'test app access')
  s.areas.old=true;s.assets.old=true
  app.mission.retiring={areas={{name='old',replacements={},assets={},borrowed=true}},index=1,assets={old=77}}
  app.mission.retire_queue={}
  app:frame();app:frame();app:frame()
  assert(not s.areas.old and not s.assets.old and not app.mission.retiring)
  app.mission.retiring={areas={},index=1,assets={}}
  assert(s.commands.envoy('menu pause'));e.on_tick();assert(s.paused and app.owns_pause)
  local frames=app.run.elapsed;for _=1,10 do if not s.paused then e.on_frame() end;e.on_tick() end
  assert(app.run.elapsed==frames);app.menu:show('playing');e.on_tick();assert(not s.paused)
  e.on_unload()
end)
local function repeated_runs(count)
  local s,e=fixture();e.gd.item_spawn=s.item_spawn;local app=e.__envoy_test_app;assert(app)
  for run=1,count do
    app:menu_effect{type='start',fighter='fox'};assert(app.run.active,table.concat(s.logs,'\n'));ready(s,e);boss(s,e)
    assert(app.run.index==4);assert((s.spawned or 0)>=run*7,'native drives exercised')
    local h=e.gd.item_spawn('drive',0,0,{payload={colour='red',amount=20}});app.drives.native[h]={colour='red',amount=20}
    local z=app.mission.current.doc.mission.goal
    s.p[1].x=z.x;s.p[1].y=z.y;s.p[2].falls=s.p[2].falls+1;e.on_frame();drain(e)
    assert(not app.run.active and app.run.index==0 and s.writes==run*2)
    assert(not next(s.fx),'pickup effects released')
    assert(not next(s.items) and not s.tint and not s.mod and not s.reserve[1] and not s.reserve[2])
    assert(not app.mission.retiring and #(app.mission.retire_queue or {})==0 and not next(s.areas))
    assert(not next(s.models) and not next(s.assets),'all geometry and asset handles released '..tostring(next(s.models))..' asset '..tostring(next(s.assets))..' path '..tostring(s.assets[next(s.assets)]))
    assert(not next(app.drives.native) and #app.drives.pickups==0)
    assert(app.run.profile.records.wins==run and app.run.profile.records.best_frames>0)
    local rows=table.concat(app:context().records,'\n');assert(rows:find('Wins: '..run,1,true))
    assert(s.commands.envoy('hub'));drain(e);assert(app.hub.active and not app.hub.waiting and app.hub.object,table.concat(s.logs,'\n'))
    assert(app.mission.current.doc.name=='hub')
    local D=T.rules();D.save=T.module('save',D);assert(D.save.decode(s.text))
    if run==6 then
      local c=app.run.profile.companions[1];assert(app.run.results.reincarnation and c.type=='egg' and c.age==0 and c.lives==1)
      for _,k in ipairs(D.companion.stats) do assert(c.stats[k].level==0 and c.stats[k].points==math.floor(app.run.results.after[k].points*.1)) end
    end
  end
  e.on_unload();assert(s.writes==count*2)
  assert(not next(s.areas) and not next(s.models) and not next(s.assets) and not next(s.fx))
end
T.test('three sequential full campaigns release every owned resource',function() repeated_runs(3) end)
T.test('full six run life reincarnates and hub cycles release every resource',function() repeated_runs(6) end)
T.test('scene, retirement, recovery and native cleanup waits name their stalls',function()
  local s,e=fixture();local app=e.__envoy_test_app;local armed,done={},{}
  e.gd.deadline=function(n) armed[n]=(armed[n] or 0)+1;return true end
  e.gd.deadline_done=function(n) done[n]=(done[n] or 0)+1;return true end
  e.gd.scene_launch=function() return true end
  app:menu_effect{type='start',fighter='fox'};app:tick();app:tick()
  assert(armed['envoy/scene-ready']==1)
  e.on_match_start();e.on_tick();assert(done['envoy/scene-ready']==1)
  ready(s,e)
  local retired=armed['envoy/retirement'] or 0
  app.mission.retiring={areas={},index=1,assets={}};app:frame()
  assert(armed['envoy/retirement']==retired+1 and done['envoy/retirement']==retired+1)
  app.mission.recovery={};app:watch_waits();app:watch_waits();assert(armed['envoy/recovery']==1)
  app.mission.recovery=nil;app:watch_waits();assert(done['envoy/recovery']==1)
  app:stop('quit');app.recolour.diagnostic='cleanup pending'
  app:watch_waits();app:watch_waits();assert(armed['envoy/native-cleanup']==1)
  app.recolour.diagnostic='inactive';app:watch_waits();assert(done['envoy/native-cleanup']==1)
  e.on_unload();assert(not next(app.run.waits))
end)
T.test('unload abandons staged installation without requiring another frame',function()
  local s,e=fixture();local app=e.__envoy_test_app
  assert(s.commands.envoy('campaign'));e.on_frame();e.on_frame()
  assert(app.mission.staging,'staged fixture');e.on_unload()
  assert(not app.mission.staging and not app.mission.recovery and not app.mission.retiring)
  assert(not next(s.areas) and not s.reserve[1] and not s.reserve[2] and not s.mod and not s.tint)
  assert(not next(app.run.waits))
end)
T.test('native collect authenticates owner and payload and awards once',function()
  local s,e=fixture();e.gd.item_spawn=s.item_spawn;assert(s.commands.envoy('campaign'));ready(s,e)
  local app=e.__envoy_test_app;encounter(s,e);local h,en=next(s.enemies);assert(h)
  e.on_enemy_defeated{handle=h,kind=en.kind}
  local item,o=next(s.items);assert(item)
  local function points() local n=0;for _,v in pairs(app.run.companion.stats) do n=n+v.points end;return n end
  local before=points()
  e.on_item_collect{name='drive',port=2,item=item,payload=o.payload}
  e.on_item_collect{name='drive',port=1,item=item,payload={colour=o.payload.colour,amount=o.payload.amount+1}}
  assert(points()==before)
  e.on_item_collect{name='drive',port=1,item=item,payload=o.payload};local awarded=points();assert(awarded>before)
  e.on_item_collect{name='drive',port=1,item=item,payload=o.payload};assert(points()==awarded)
  s.items[item]=nil;e.on_unload();assert(not next(s.items))
end)
T.test('fresh install without maze assets completes authored campaign',function()
  local s,e=fixture(true,true);local app=e.__envoy_test_app
  assert(s.commands.envoy('campaign'));ready(s,e)
  assert(app.run.levels[1].command=='play path' and app.run.levels[3].command=='play path')
  boss(s,e);local z=app.mission.current.doc.mission.goal
  s.p[1].x=z.x;s.p[1].y=z.y;s.p[2].falls=s.p[2].falls+1;e.on_frame();drain(e)
  assert(app.run.profile.last_result=='win' and s.writes==2)
  e.on_unload();assert(not next(s.models) and not next(s.fx) and not s.mod and not s.tint)
end)
T.test('real loader refused start keeps profile unchanged and absent kit returns menu',function()
  local s,e=fixture(false,true);local app=e.__envoy_test_app;local before=s.text
  local read=e.gd.mod_read;e.gd.mod_read=function(p) if p=='missions/path/level.lua' then return nil,'missing' end;return read(p) end
  s.commands.envoy('campaign')
  for _=1,60 do e.on_draw();e.on_frame();e.on_tick() end
  assert(not app.run.active and s.writes==0 and s.text==before and app.notice)
  assert(app.menu.screen=='setup' and not s.paused and not s.mod and not s.tint and not s.reserve[2])
  assert(s.p[1].percent==39 and s.p[2].percent==116,'failed start restores damage')
  e.gd.mod_read=read;assert(s.commands.envoy('hub'));drain(e)
  assert(not app.hub.active and app.menu.screen=='hub' and not app.garden_available)
  e.on_unload();assert(not next(s.models) and not next(s.areas))
end)
T.done()




