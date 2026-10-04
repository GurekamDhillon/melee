-- Offline contracts. Removing validation, rollback, streaming or command routing must fail these.
local base = 'pc/scripts/examples/missions/scripts/'
if not io.open(base .. 'mission.lua') then base = 'melee/' .. base end
local passed, failed = 0, 0
local function test(name, fn)
  local ok, why = pcall(fn)
  if ok then passed = passed + 1 else failed = failed + 1 end
  print((ok and 'PASS ' or 'FAIL ') .. name .. (ok and '' or ': ' .. tostring(why)))
end
local function read(path) local f=assert(io.open(path)); local s=f:read('a'); f:close(); return s end
local function modules()
  local d = {}
  for _, name in ipairs({'mission','validator','loader','world','zones','chunks','glue','fighters','camera','commands','mission_finish','hud','mission_warm','install','mission_launch','runtime'}) do
    local f = loadfile(base .. name .. '.lua')
    assert(f, 'missing runtime module ' .. name)
    d[name] = name == 'mission' and f() or f()(d)
  end
  return d
end
local level = 'return {version=2,units=6.5,parts={{part="custom",x=0,y=0,z=0,rot=0,collision=true,floor_flags=3}},camera={left=-100,right=100,top=100,bottom=-100},blast={left=-200,right=200,top=200,bottom=-200},spawn={[0]={x=0,y=10}}}'
local mission = 'return {start={x=0,y=10},enemies={{kind="goomba",x=20,y=10,wave=1},{kind="koopa",x=60,y=10,wave=2}},checkpoints={{x=40,y=10,w=10,h=10}},goal={x=90,y=10,w=10,h=10},objective={type="defeat_then_goal"}}'
local function fixture()
  local s={files={['missions/test/level.lua']=level,['missions/test/mission.lua']=mission}, stamps={},
    models={}, areas={}, enemies={}, logs={}, loads=0, unloads=0, serial=0, spawns={},
    p={x=0,y=10,falls=0,percent=0}, flying=false, attacking=false, hidden=false, now=0}
  local function id() s.serial=s.serial+1 return s.serial end
  local g={}
  g.mod_read=function(p) return s.files[p], 'missing file' end
  g.mod_list=function(p)
    if p=='missions/' then return {{name='test',dir=true}} end
    if p:match('/models/$') then return {{name='custom.gxmesh',dir=false},{name='atlas.gxtex',dir=false}} end
    return nil,'missing directory'
  end
  g.mod_stamp=function(p) return s.stamps[p] or (s.files[p] and 1 or nil) end
  g.match=function() return {active=not s.ended,netplay=s.online} end
  g.player=function() local p={};for k,v in pairs(s.p) do p[k]=v end;return p end
  g.log=function(t) s.logs[#s.logs+1]=t end
  g.time=function() s.now=s.now+0.001 return s.now end
  g.model_load=function(p) if s.reject_asset then return nil,'bad asset' end return p end
  g.model_release=function() end
  g.model_spawn=function(a,o)
    if s.reject_model then return nil,'capacity' end
    local h=id(); s.models[h]={asset=a,opts=o}; return h
  end
  g.model_get=function(h)local m=s.models[h];return m and m.opts end
  g.model_despawn=function(h) s.models[h]=nil return true end
  g.model_set=function(h,o) assert(s.models[h]); s.models[h].opts=o return true end
  g.area_load=function(n,build)
    assert(not s.building,'nested builder'); assert(not s.areas[n],'duplicate area')
    local before={}; for h in pairs(s.models) do before[h]=true end
    s.building=true; local ok,why=pcall(build); s.building=false
    local hs={}; for h in pairs(s.models) do if not before[h] then hs[#hs+1]=h end end
    if not ok then for _,h in ipairs(hs) do s.models[h]=nil end error(why) end
    s.areas[n]=hs; s.loads=s.loads+1; return true
  end
  g.area_unload=function(n)
    assert(not s.building); for _,h in ipairs(s.areas[n] or {}) do s.models[h]=nil end
    s.areas[n]=nil; s.unloads=s.unloads+1; return true
  end
  g.stage_add_line=function() if s.reject_line then return nil,'line capacity' end return id() end
  g.stage_set_spawn=function(slot,x,y) if s.reject_spawn then return false,'spawn refused' end s.spawns[slot]={x=x,y=y} return true end
  g.stage_set_camera_bounds=function(...) if s.reject_bounds then s.reject_bounds=false;return false,'bounds refused' end s.camera={...} return true end
  g.stage_set_blast_bounds=function(...) s.blast={...} return true end
  g.stage_restore_bounds=function() s.camera=nil;s.blast=nil;s.origin={x=0,y=0};s.spawns={} return true end
  g.stage_set_origin=function(x,y,o)s.origin={x=x,y=y,frames=o and o.frames or 0};return true end
  g.safe_area=function()return {w=480*16/9,h=480}end
  g.camera_params=function(v)
    local old=s.params or {};if v==nil then s.params=nil elseif next(v) then s.params=v end;return old
  end
  g.stage_bounds=function()
    local function rect(t,default) if not t then return default end local o=s.origin or {x=0,y=0};return {left=t[1]+o.x,right=t[2]+o.x,top=t[3]+o.y,bottom=t[4]+o.y} end
    return {camera=rect(s.camera,{left=-300,right=300,top=300,bottom=-300}),blast=rect(s.blast,{left=-400,right=400,top=400,bottom=-400}),origin=s.origin or {x=0,y=0}}
  end
  g.stage_spawn=function(slot) local p=s.spawns[slot] or {x=0,y=30};return p.x,p.y,0 end
  g.stage_hide=function(v) s.hidden=v return true end
  g.teleport=function(_,x,y) if s.reject_teleport then error('gd.teleport: dead fighter') end s.p.x=x;s.p.y=y end
  g.fly=function(_,v)
    if v~=nil then if v=='place' then s.flying=false else s.flying=v end;s.placed=v=='place' end
    return s.flying
  end
  g.fly_target=function(_,x,y) s.target={x=x,y=y};s.flying=true; return true end
  g.fly_attack=function(_,v) s.attacking=v;if v then s.flying=true end;return s.attacking end
  g.spawn_enemy=function(k,x,y) if s.reject_enemy then return nil,'enemy capacity' end local h=id();s.enemies[h]={kind=k,x=x,y=y,status='alive'};return h end
  g.enemy_remove=function(h) s.enemies[h]=nil return true end
  g.enemy_status=function(h) return s.enemies[h] and s.enemies[h].status or 'removed' end
  g.enemy_state=function(h)
    local e=s.enemies[h];if not e or e.status~='alive' then return nil end
    local copy={};for k,v in pairs(e) do copy[k]=v end;return copy
  end
  g.floor_below=function(_,y)return y-0.25 end;g.stage_isolate=function()return s.hidden end
  g.stage_move=function()return true end
  g.fighter_bench=function(port)if port==1 then s.reserve_benches=(s.reserve_benches or 0)+1;s.holding=true;return true end;return false end
  g.fighter_benched=function(port)return port==1 and s.holding or false,{{entity_index=0,present=port==1,refusal_code=s.reserve_refusal_code or 0}}end
  g.fighter_call=function(port,x,y)if port==1 then s.reserve_calls=(s.reserve_calls or 0)+1;if s.reject_teleport then s.reject_teleport=false;return false end;s.holding=false;s.p.x=x;s.p.y=y;return true end;return false end
  g.set_damage=function(_,value)s.p.percent=value end
  g.set_stocks=function() end
  g.command=function(_,fn) s.command=fn end
  g.text=function(_,_,t) s.hud=t end
  g.fill=function()end
  g.warm=function(q)s.warm_declaration=q;return id()end
  g.warm_done=function()return true end
  g.warm_release=function()end
    local api=g;local reserve={fighter_bench=api.fighter_bench,fighter_benched=api.fighter_benched,fighter_call=api.fighter_call}
  g=setmetatable({},{__index=function(_,key)
    if reserve[key] then return function(port,...)if port==1 then return reserve[key](port,...)end;return api[key](port,...)end end
    return api[key]
  end,__newindex=function(_,key,value)api[key]=value end})
  local d=modules();local r=d.runtime.new(g)
    local command,frame=r.command,r.frame;local draining=false
  local function drain()
    if s.manual_stage or draining then return end
    draining=true
    for _=1,300 do if not r.staging and not r.retiring then break end;if r.staging then frame(r) else d.install.retire(r) end end
    draining=false;assert(not r.staging and not r.retiring,'fixture staging never completed')
  end
  r.command=function(self,arg)local ok,why=command(self,arg);drain();if self.last_install_error then return nil,self.last_install_error end;return ok,why end
  r.frame=function(self)
    self:pre_frame();frame(self);drain()
    if not s.manual_stream then
      for _=1,30 do if not self.current or not self.current.stream.pending then break end
        self:pre_frame();frame(self);drain()
      end
    end
  end
  return s,r,d
end
local function count(t) local n=0;for _ in pairs(t) do n=n+1 end;return n end
local function play(s,r) assert(r:command('play test')); assert(s.hidden); assert(count(s.models)==1) end
test('module copy unchanged',function()
  assert(read(base..'mission.lua')==read(base:gsub('missions/scripts/','map_editor/scripts/')..'mission.lua'))
end)
test('folder catalogue and world transforms',function()
  local s,r=fixture();play(s,r);local _,v=next(s.models)
  assert(v.asset=='missions/test/models/custom.gxmesh');assert(v.opts.collision and v.opts.floor_flags==3)
  assert(s.camera[1]==-0.5 and r.g.stage_bounds().blast.right==200 and s.spawns[0].y==10)
  assert(count(s.enemies)==1);r:draw();assert(s.hud:find('Defeat all',1,true))
end)
local refusals={
  {'version', 'version=2','version=3'}, {'units','units=6.5','units=7'},
  {'part','part="custom"','part="absent"'}, {'finite','x=0','x=0/0'},
  {'rotation','rot=0','rot=361'}, {'collision','collision=true','collision=1'},
  {'flags','floor_flags=3','floor_flags=4'}, {'scale','rot=0','rot=0,scale=0'},
  {'bounds','left=-100','left=101'}, {'spawn','[0]','[8]'},
  {'sparse','parts={{','parts={[2]={'},
}
for _,case in ipairs(refusals) do test('atomic validation refusal '..case[1],function()
  local s,r=fixture();play(s,r);local old=r.current;local loads=s.loads
  s.files['missions/test/level.lua']=level:gsub(case[2]:gsub('(%W)','%%%1'),case[3],1)
  assert(not r:command('reload'));assert(r.current==old and s.loads==loads and count(s.models)==1)
end) end
test('sandbox and mission refusals',function()
  local s,r=fixture();play(s,r);local old=r.current
  for _,bad in ipairs({'gd.quit(); return {}','return {objective={type="wrong"}}','return {start={x=0,y=0}}','return {start={x=0,y=0},objective={type="defeat_all"}}','return {start={x=0,y=0},objective={type="reach_goal"}}','return {start={x=0,y=0},enemies={{kind="wrong",x=0,y=0}},objective={type="defeat_all"}}','not lua'}) do
    s.files['missions/test/mission.lua']=bad;assert(not r:command('reload'));assert(r.current==old)
  end
  assert(not r:command('play ../test'));assert(not r:command('play test from absent'))
end)
test('legacy embedded mission',function()
  local s,r=fixture();s.files['missions/test/mission.lua']=nil
  s.files['missions/test/level.lua']=level:gsub('parts=', 'mission='..mission:sub(8)..',parts=',1)
  play(s,r)
end)
test('asset and instance refusal roll back',function()
  local s,r=fixture();play(s,r);local old=r.current
  s.reject_asset=true;assert(not r:command('reload'));assert(r.current==old)
  s.reject_asset=false;s.reject_model=true;assert(not r:command('reload'));assert(r.current==old and count(s.models)==1)
end)
test('stamp reload every fifteen frames preserves player position',function()
  local s,r=fixture();play(s,r);s.p.x=33;s.stamps['missions/test/mission.lua']=2;local old=r.current
  for _=1,14 do r:frame() end;assert(r.current==old)
  r:frame();assert(r.current~=old and s.p.x==33)
  old=r.current;s.files['missions/test/level.lua']='return {}';s.stamps['missions/test/level.lua']=2
  for _=1,15 do r:frame() end;assert(r.current==old)
end)
test('play from checkpoint clears earlier waves',function()
  local s,r=fixture();assert(r:command('play test from checkpoint1'))
  assert(s.p.x==40 and r.current.run.state.cp==1 and r.current.run.state.begun[1])
  assert(r.current.run.state.defeated==1 and count(s.enemies)==1)
  local _,e=next(s.enemies);assert(e.kind=='koopa')
end)
test('fly next prev named drop clear restart list and stop',function()
  local s,r=fixture();play(s,r)
  assert(r:command('fly next'));assert(s.target.x==0)
  assert(r:command('fly next'));assert(s.target.x==20)
  assert(r:command('fly prev'));assert(s.target.x==0)
  assert(r:command('fly goal'));assert(s.target.x==90)
  assert(r:command('drop'));assert(s.placed and not s.flying and not s.attacking)
  assert(r:command('clear'));assert(s.attacking)
  r:frame();assert(s.target.x==20)
  for _,e in pairs(s.enemies) do e.status='defeated' end
  r:frame();assert(not s.attacking and r.current.run.state.defeated==1)
  assert(r:command('restart'));assert(s.p.x==0)
  assert(r:command('list'));assert(table.concat(s.logs,'\n'):find('test',1,true))
  assert(r:command('stop'));assert(not s.hidden and count(s.models)==0 and count(s.enemies)==0)
end)
local function chunks(s)
  local specs={}
  for x=0,4 do for y=0,2 do
    local name='c'..x..'_'..y
    specs[#specs+1]=('{id="%s",rect={left=%d,right=%d,bottom=%d,top=%d},spawn={x=%d,y=%d}}'):format(name,x*100,(x+1)*100,y*100,(y+1)*100,x*100+10,y*100+10)
    s.files['missions/test/chunks/'..name..'/level.lua']='return {version=2,units=6.5,parts={{part="custom",x='..(x*100)..',y='..(y*100)..',z=0,rot=0,collision=true,floor_flags=0}}}'
  end end
  s.files['missions/test/level.lua']=level:gsub('parts={{.-}},camera=', 'parts={},chunks={'..table.concat(specs,',')..'},camera=')
end
test('three by three streaming and chunk respawn',function()
  local s,r=fixture();chunks(s);s.p.x=150;s.p.y=150
  assert(r:command('play test'));s.p.x=150;s.p.y=150;r:frame()
  assert(count(r.current.stream.loaded)==9 and s.spawns[4].x==110)
  local loads,unloads=s.loads,s.unloads;s.p.x=250;for _=1,8 do r:frame()end
  assert(s.loads-loads>=3 and s.unloads-unloads>=3 and s.spawns[4].x==210)
  s.p.y=-999;s.p.falls=1;r:frame();assert(s.spawns[4].x==210)
  r:stop('match end');assert(count(s.areas)==0)
end)
test('tour visits every marker and chunk and records timing',function()
  local s,r=fixture();chunks(s);assert(r:command('play test'));assert(r:command('tour'))
  for _=1,100 do if s.target then s.p.x=s.target.x;s.p.y=s.target.y end;r:frame() end
  local log=table.concat(s.logs,'\n');assert(log:find('tour chunk:c4_2',1,true) and log:find('seconds=',1,true))
  assert(log:find('tour complete',1,true))
end)
test('cleanup on unload and offline guard',function()
  local s,r=fixture();play(s,r);s.ended=true;r:frame()
  assert(not r.current and count(s.models)==0 and count(s.enemies)==0 and not s.attacking)
  s.ended=false;s.online=true;assert(not r:command('play test'))
  s.online=false;play(s,r);r:stop('unload');assert(count(s.areas)==0)
end)
test('collision trigger replacements belong to areas and clean up',function()
  local s,r=fixture()
  s.files['missions/test/mission.lua']=mission:gsub('objective=', 'triggers={{x=0,y=10,w=4,h=4,action="collision",at={x=0,y=0},r=20,open=true}},objective=')
  play(s,r);r:frame() -- collision edits wait for a frame without wave spawning
  local _,v=next(s.models);assert(v.opts.collision==false)
  r:stop('unload');assert(count(s.models)==0,'replacement leaked outside owning area')
end)
test('bounds teleport and initial enemy refusals retain running objects',function()
  for _,kind in ipairs({'reject_bounds','reject_teleport','reject_enemy'}) do
    local s,r=fixture();play(s,r);s.p.x=33;local old=r.current;local h=next(s.enemies)
    s[kind]=true;local accepted=r:command('play test');assert(not accepted);assert(r.current==old and s.enemies[h])
    assert(count(s.models)==1 and count(s.enemies)==1 and s.p.x==33)
    assert(s.spawns[4].x==0 and s.hidden)
  end
end)
test('chunk load refusal retains previous window and respawn',function()
  local s,r=fixture();chunks(s);assert(r:command('play test'));s.p.x=150;s.p.y=150;r:frame()
  local n=count(s.models);local oldloads=s.loads;s.reject_model=true;s.p.x=250;r:frame()
  assert(count(s.models)==n and s.loads==oldloads and s.spawns[4].x==110)
  assert(r.current.stream.current.id=='c1_1')
end)
test('chunk spawn refusal retains previous window',function()
  local s,r=fixture();chunks(s);assert(r:command('play test'));s.p.x=150;s.p.y=150;r:frame()
  local n=count(s.models);s.reject_spawn=true;s.p.x=250;r:frame()
  assert(r.current.stream.loaded.c0_1 and not r.current.stream.loaded.c3_1 and count(s.models)==n)
end)
test('chunk validation refuses duplicate overlap missing and invalid child',function()
  local s,r=fixture();play(s,r);local old=r.current
  for _,bad in ipairs({
    '{{id="../bad",rect={},spawn={}}}',
    '{{id="a",rect={left=0,right=10,bottom=0,top=10},spawn={x=20,y=1}}}',
    '{{id="missing",rect={left=0,right=10,bottom=0,top=10},spawn={x=1,y=1}}}',
    '{{id="a",rect={left=0,right=10,bottom=0,top=10},spawn={x=1,y=1}},{id="a",rect={left=0,right=10,bottom=0,top=10},spawn={x=1,y=1}}}',
    '{{id="a",rect={left=0,right=10,bottom=0,top=10},spawn={x=1,y=1}},{id="b",rect={left=5,right=15,bottom=0,top=10},spawn={x=6,y=1}}}'}) do
    s.files['missions/test/chunks/a/level.lua']=level
    s.files['missions/test/level.lua']=level:gsub('units=6.5','units=6.5,chunks='..bad)
    assert(not r:command('reload'));assert(r.current==old and count(s.models)==1)
  end
  chunks(s);s.files['missions/test/chunks/c4_2/level.lua']='return {}'
  assert(not r:command('reload'));assert(r.current==old)
end)
test('generated entry hooks execute without require',function()
  local s,r=fixture();local env={gd=r.g};setmetatable(env,{__index=_G})
  assert(loadfile(base..'main.lua','t',env))()
  assert(s.command('play test'));for _=1,30 do env.on_frame()end;env.on_draw();assert(s.hud)
  env.on_match_end();assert(count(s.models)==0 and not s.hidden)
  assert(s.command('play test'));env.on_unload();for _=1,30 do env.on_frame()end;assert(count(s.enemies)==0)
end)
test('sample folder uses kit parts and a playable mission',function()
  local s,r=fixture();local folder=base:gsub('scripts/$','missions/first/')
  s.files['missions/first/level.lua']=read(folder..'level.lua')
  s.files['missions/first/mission.lua']=read(folder..'mission.lua')
  s=r.g.mod_list
  r.g.mod_list=function(path) if path=='missions/first/models/' then return {{name='bf_floor_4m.gxmesh',dir=false}} end return s(path) end
  assert(r:command('play first'))
end)
test('long range fly uses teleport fixture within native coordinate cap',function()
  local s,r=fixture();s.files['missions/test/mission.lua']=mission:gsub('x=90','x=20000')
  play(s,r);assert(r:command('fly goal'));assert(s.p.x==20000 and s.flying==true)
end)
test('authored marker progress handles vertical and nonmonotonic missions',function()
  local s,r=fixture()
  s.files['missions/test/level.lua']=level:gsub('units=6.5','units=6.5,markers={{name="upper",x=5,y=200,cleared_waves={1},checkpoint=1,frames=300}}')
  assert(r:command('play test from upper'))
  local st=r.current.run.state
  assert(st.cp==1 and st.defeated==1 and st.frames==300 and s.p.y==200)
  assert(count(s.enemies)==1);local _,e=next(s.enemies);assert(e.kind=='koopa')
end)
test('chunk window rejects mixed sizes and off-grid rectangles',function()
  local s,r=fixture();play(s,r);local old=r.current;chunks(s)
  s.files['missions/test/level.lua']=s.files['missions/test/level.lua']:gsub('right=500','right=550',1)
  assert(not r:command('reload'));assert(r.current==old)
end)
test('missing folder lists and unstable stamps refuse before world writes',function()
  local s,r=fixture();play(s,r);local old=r.current;local n=s.loads
  local list=r.g.mod_list;r.g.mod_list=function()return nil,'refused'end
  assert(not r:command('reload'));assert(r.current==old and s.loads==n)
  r.g.mod_list=list
  local calls=0;r.g.mod_stamp=function()calls=calls+1;return calls end
  assert(not r:command('reload'));assert(r.current==old and s.loads==n)
end)
test('match-end cleanup tolerates engine scene resources already reclaimed',function()
  local s,r=fixture();play(s,r);s.ended=true;s.models={};s.areas={};s.enemies={}
  r.g.area_unload=function()error('active match required')end
  r.g.model_release=function()error('active match required')end
  r:stop('match end');assert(r.current==nil)
end)
test('new layout restores host blast and default follow window',function()
  local s,r=fixture();play(s,r)
  s.files['missions/test/level.lua']='return {version=1,units=6.5,parts={}}'
  assert(r:command('reload'));assert(s.camera[1]==-0.5 and r.g.stage_bounds().blast.right==400 and not s.spawns[0])
end)
test('repeated collision triggers retain only one replacement per part',function()
  local s,r,d=fixture();play(s,r)
  for _=1,70 do d.world.collision(r.g,{r.current.root},{x=0,y=0,r=10,open=true}) end
  assert(count(s.areas)==2 and count(s.models)==1)
  r:stop('stop');assert(count(s.areas)==0 and count(s.models)==0)
end)
test('steady chunks and fixed clear targets do not flood native logs',function()
  local s,r=fixture();chunks(s);assert(r:command('play test'));assert(r:command('clear'))
  local writes,targets=0,0;local spawn=r.g.stage_set_spawn;local target=r.g.fly_target
  r.g.stage_set_spawn=function(...)writes=writes+1;return spawn(...)end
  r.g.fly_target=function(...)targets=targets+1;return target(...)end
  for _=1,600 do r:frame() end
  assert(writes==0,'unchanged chunk respawn rewritten '..writes..' times')
  assert(targets<=1,'fixed target reissued '..targets..' times')
end)
test('fly goal cancels clear pursuit and finishes on terminal enemy observation',function()
  local s,r=fixture();play(s,r);assert(r:command('clear'));assert(r:command('fly goal'))
  assert(not r.current.clear and not s.attacking)
  for _,e in pairs(s.enemies) do e.status='removed';e.x=195 end
  r:frame();assert(r.current.run.state.begun[2])
  for _,e in pairs(s.enemies) do e.status='defeated' end
  s.p.x=90;s.p.y=10;r:frame()
  assert(r.current.run.state.result.status=='complete')
  assert(table.concat(s.logs,'\n'):find('mission: complete',1,true))
  local n=#s.logs;r:frame();assert(#s.logs==n,'terminal event repeated')
end)
test('checkpoint goal and last defeat are observed together without one-step delay',function()
  local s,r=fixture();assert(r:command('play test from checkpoint1'))
  s.p.x=90;s.p.y=10;for _,e in pairs(s.enemies) do e.status='defeated' end
  r:frame();assert(r.current.run.state.cleared and r.current.run.state.result)
end)
test('fly destination preloads without replacing actual chunk or thrashing',function()
  local s,r=fixture();chunks(s);assert(r:command('play test'))
  s.p.x=150;s.p.y=150;r:frame();local stream=r.current.stream
  assert(r:command('fly goal')) -- goal remains in c0_0; real player is c1_1
  assert(stream.current.id=='c1_1' and stream.loaded.c0_0)
  local loads,unloads=s.loads,s.unloads
  for _=1,10 do r:frame() end
  assert(s.loads==loads and s.unloads==unloads and stream.current.id=='c1_1')
end)
test('chunk border hysteresis reports actual crossing position',function()
  local s,r=fixture();chunks(s);assert(r:command('play test'));s.p.x=150;s.p.y=150;r:frame()
  s.p.x=200.5;r:frame();assert(r.current.stream.current.id=='c1_1')
  s.p.x=199.5;r:frame();assert(r.current.stream.current.id=='c1_1')
  s.p.x=209;r:frame();assert(r.current.stream.current.id=='c2_1')
  assert(table.concat(s.logs,'\n'):find('player=209.0,150.0',1,true))
end)
test('chunk models prefer shared root catalogue and allow root-only exports',function()
  local s,r=fixture();chunks(s)
  local list=r.g.mod_list;r.g.mod_list=function(path)
    if path:find('/chunks/',1,true) then return nil,'missing directory' end
    return list(path)
  end
  assert(r:command('play test'))
  for _,model in pairs(s.models) do assert(model.asset=='missions/test/models/custom.gxmesh') end
end)
test('reload refusal includes engine reason and keeps old world',function()
  local s,r=fixture();play(s,r);local old=r.current
  r.g.model_load=function()return nil,'model scene asset budget exceeded (64 MiB)'end
  local ok,why=r:command('reload');assert(not ok and tostring(why):find('64 MiB',1,true))
  assert(r.current==old)
end)
test('optional labels include authored instance name and fallback part name',function()
  local s,r=fixture();s.files['missions/test/level.lua']=level:gsub('part="custom"','part="custom",name="floor entry"')
  local labels={};r.g.model_label=function(h,name)assert(s.models[h]);labels[h]=name;return true end
  play(s,r);local _,name=next(labels);assert(name=='floor entry')
end)
test('blast-zone vanish advances waves and completes without being counted as defeat',function()
  local s,r=fixture();play(s,r);local h,e=next(s.enemies);e.x=195;r:frame()
  e.status='removed';r:frame();assert(r.current.run.state.vanished==1 and r.current.run.state.begun[2])
  for _,enemy in pairs(s.enemies) do enemy.status='defeated' end
  s.p.x=90;r:frame();assert(r.current.run.state.result.status=='complete')
end)
test('moving clear targets retarget at bounded cadence and survive temporary refusal',function()
  local s,r=fixture();play(s,r);assert(r:command('clear'));local calls=0;local target=r.g.fly_target
  r.g.fly_target=function(...)calls=calls+1;if calls==1 then error('fighter is respawning')end;return target(...)end
  for i=1,60 do for _,e in pairs(s.enemies)do e.x=20+i end;r:frame()end
  assert(calls<=10 and calls>=2 and s.target and r.current.clear)
end)
test('labels fall back to part and persist on collision replacements',function()
  local s,r,d=fixture();local labels={}
  r.g.model_label=function(h,name)labels[h]=name end -- optional setter return is ignored
  play(s,r);assert(labels[next(s.models)]=='custom')
  d.world.collision(r.g,{r.current.root},{x=0,y=0,r=10,open=true})
  assert(labels[next(s.models)]=='custom')
end)
assert(loadfile((base:gsub('scripts/examples/missions/scripts/$', 'tests/missions_fix2.lua'))))()(test,fixture,chunks,level,mission,count)
assert(loadfile((base:gsub('scripts/examples/missions/scripts/$', 'tests/missions_fix3.lua'))))()(test,fixture)
assert(loadfile((base:gsub('scripts/examples/missions/scripts/$', 'tests/missions_fix4.lua'))))()(test,fixture)
assert(loadfile((base:gsub('scripts/examples/missions/scripts/$', 'tests/missions_fix5.lua'))))()(test,fixture)
assert(loadfile((base:gsub('scripts/examples/missions/scripts/$', 'tests/missions_fix6.lua'))))()(test,fixture,chunks)
assert(loadfile((base:gsub('scripts/examples/missions/scripts/$', 'tests/missions_fix7.lua'))))()(test,fixture)
assert(loadfile((base:gsub('scripts/examples/missions/scripts/$', 'tests/missions_fix8.lua'))))()(test,fixture);assert(loadfile((base:gsub('scripts/examples/missions/scripts/$', 'tests/missions_play_crash.lua'))))()(test,fixture)
assert(loadfile((base:gsub('scripts/examples/missions/scripts/$', 'tests/missions_fix9.lua'))))()(test,fixture,chunks);assert(loadfile((base:sub(1,6)=='melee/' and '' or '../')..'tools/maze/runtime_fix3.lua'))()(test,fixture,chunks);print(('missions_runtime: %d passed, %d failed'):format(passed,failed));assert(failed==0, 'missions runtime contract failures')
