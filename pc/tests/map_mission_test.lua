-- Mission layer tests for the kit map editor. Run from melee/: lua pc/tests/map_mission_test.lua
-- Part 1 exercises the pure state machine (scripts/mission.lua). Part 2 reuses map_editor_test.lua's
-- gd stub (the text before "local chunk = loadfile", as tools/roguelite/test_audit_repairs.py does)
-- and drives the editor commands and the runtime glue. Stubs only: nothing here ran in the engine.
local function read(path) local f=assert(io.open(path,'rb')) local s=f:read('a') f:close() return s end
local DIR='pc/scripts/examples/map_editor/scripts/'
local passed,failed=0,0
local function test(name,fn)
  local ok,why=pcall(fn)
  if ok then passed=passed+1 else failed=failed+1 print('FAIL '..name..': '..tostring(why)) end
end

local Mission=assert(loadfile(DIR..'mission.lua'),'scripts/mission.lua is missing')()

local function sample()
  return {start={x=0,y=0},
    enemies={{kind='goomba',x=50,y=0,wave=1},{kind='koopa',x=60,y=0,wave=1},{kind='redead',x=80,y=0,wave=2}},
    goal={x=200,y=0,w=20,h=40},checkpoints={{x=100,y=0,w=10,h=40}},
    objective={type='defeat_then_goal',time=60,lives=2}}
end
local function obs(frames,px,falls,alive,defeated)
  return {frames=frames,player=px and {x=px,y=0} or nil,falls=falls or 0,alive=alive or {},defeated=defeated}
end
local function of_type(list,t) local out={} for _,v in ipairs(list) do if v.type==t then out[#out+1]=v end end return out end
local function report(st,actions,base)
  for _,a in ipairs(actions) do if a.type=='spawn' then Mission.spawned(st,a.index,base+a.index) end end
end
local function refuses(m,pattern)
  local ok,why=pcall(Mission.validate,m)
  assert(not ok,'validate accepted a bad mission (wanted '..pattern..')')
  assert(tostring(why):find(pattern,1,true),'wrong refusal: '..tostring(why)..' (wanted '..pattern..')')
end

test('validate normalizes and defaults the wave',function()
  local m=sample() m.enemies[3].wave=nil
  local n=Mission.validate(m)
  assert(n.enemies[3].wave==1 and n.start.x==0 and #n.checkpoints==1 and n.objective.type=='defeat_then_goal')
  assert(n~=m,'validate must return a copy')
end)
test('validate refuses malformed missions',function()
  local function with(f) local m=sample() f(m) return m end
  refuses(with(function(m) m.enemies[1].kind='dragon' end),'unknown enemy kind')
  refuses(with(function(m) m.enemies[1].x=0/0 end),'finite')
  refuses(with(function(m) m.enemies[1].x=math.huge end),'finite')
  refuses(with(function(m) m.enemies[1].x='5' end),'finite')
  refuses(with(function(m) m.enemies[1].wave=0 end),'wave')
  refuses(with(function(m) m.enemies[1].wave=9 end),'wave')
  refuses(with(function(m) m.enemies[1].wave=1.5 end),'wave')
  refuses(with(function(m) m.goal.w=0 end),'goal')
  refuses(with(function(m) m.goal.h=-1 end),'goal')
  refuses(with(function(m) m.goal=5 end),'goal')
  refuses(with(function(m) m.start.y=nil end),'start')
  refuses(with(function(m) m.objective.type='win' end),'objective')
  refuses(with(function(m) m.objective.time=0 end),'time')
  refuses(with(function(m) m.objective.time=3601 end),'time')
  refuses(with(function(m) m.objective.lives=0 end),'lives')
  refuses(with(function(m) m.objective.lives=2.5 end),'lives')
  refuses(with(function(m) m.objective.lives=100 end),'lives')
  refuses(with(function(m) m.bogus=1 end),'unknown mission field')
  refuses(with(function(m) m.enemies[1].hp=3 end),'unknown enemy field')
  refuses(with(function(m) m.enemies={[2]={kind='goomba',x=0,y=0}} end),'list')
  refuses(with(function(m) m.checkpoints={} for i=1,17 do m.checkpoints[i]={x=i,y=0,w=1,h=1} end end),'checkpoint')
  refuses(with(function(m) m.enemies={} for i=1,33 do m.enemies[i]={kind='goomba',x=i,y=0,wave=1} end end),'wave 1')
  refuses(with(function(m) m.enemies={} for i=1,65 do m.enemies[i]={kind='goomba',x=i,y=0,wave=(i%2)+1} end end),'enemies')
  refuses(setmetatable({},{}),'table')
  refuses('mission','table')
end)
test('validate accepts every engine enemy kind and the limits',function()
  local m=sample() m.enemies={}
  for i,k in ipairs(Mission.KINDS) do m.enemies[i]={kind=k,x=i,y=0} end
  assert(#Mission.KINDS==7)
  Mission.validate(m)
  m.enemies={} for i=1,32 do m.enemies[i]={kind='goomba',x=i,y=0,wave=1} end
  for i=33,64 do m.enemies[i]={kind='goomba',x=i,y=0,wave=2} end
  Mission.validate(m)
end)
test('check_playable names what is missing',function()
  local m=Mission.validate(sample())
  assert(Mission.check_playable(m)==nil)
  local n=Mission.validate({}) assert(Mission.check_playable(n):find('start',1,true))
  n=Mission.validate({start={x=0,y=0}}) assert(Mission.check_playable(n):find('objective',1,true))
  n=Mission.validate({start={x=0,y=0},objective={type='reach_goal'}}) assert(Mission.check_playable(n):find('goal',1,true))
  n=Mission.validate({start={x=0,y=0},objective={type='defeat_all'}}) assert(Mission.check_playable(n):find('enemy',1,true))
  n=Mission.validate({start={x=0,y=0},objective={type='defeat_then_goal'},goal={x=1,y=1,w=1,h=1}})
  assert(Mission.check_playable(n):find('enemy',1,true))
end)
test('waves spawn in order and the next one waits for the current to be defeated',function()
  local st=Mission.new(Mission.validate(sample()))
  local a,e=Mission.step(st,obs(1,0))
  local sp=of_type(a,'spawn')
  assert(#sp==2 and sp[1].index==1 and sp[2].index==2 and sp[1].kind=='goomba')
  assert(sp[1].facing==-1,'enemies face the player')
  assert(of_type(e,'wave')[1].n==1)
  report(st,a,10)
  a=Mission.step(st,obs(2,0,0,{[11]=true,[12]=true})) assert(#a==0)
  a,e=Mission.step(st,obs(3,0,0,{[12]=true})) assert(#a==0 and #of_type(e,'defeat')==1)
  a=Mission.step(st,obs(4,0,0,{})) sp=of_type(a,'spawn')
  assert(#sp==1 and sp[1].index==3 and sp[1].kind=='redead','wave 2 starts when wave 1 is gone')
  report(st,a,10)
  assert(Mission.hud(st):find('enemies 1',1,true))
end)
test('a new enemy is not judged on the frame it spawned',function()
  local st=Mission.new(Mission.validate(sample()))
  local a=Mission.step(st,obs(1,0)) report(st,a,10)
  local _,e=Mission.step(st,obs(1,0,0,{})) -- the pool has not caught up yet: same frame
  assert(#of_type(e,'defeat')==0)
end)
test('defeat_then_goal: the goal opens only after the last wave',function()
  local st=Mission.new(Mission.validate(sample()))
  local a=Mission.step(st,obs(1,0)) report(st,a,10)
  local _,e=Mission.step(st,obs(2,200,0,{[11]=true,[12]=true})) -- standing in the goal early
  assert(#of_type(e,'complete')==0)
  a=Mission.step(st,obs(3,200,0,{})) report(st,a,10) -- wave 2 spawns
  _,e=Mission.step(st,obs(4,200,0,{[13]=true})) assert(#of_type(e,'complete')==0)
  a,e=Mission.step(st,obs(5,0,0,{})) assert(#of_type(e,'complete')==0,'cleared but not in the goal')
  assert(Mission.hud(st):find('goal',1,true))
  a,e=Mission.step(st,obs(6,205,0,{}))
  local c=of_type(e,'complete')
  assert(#c==1 and c[1].defeated==3 and #of_type(a,'cleanup')==1)
  assert(st.result.status=='complete')
  a,e=Mission.step(st,obs(7,205,0,{})) assert(#a==0 and #e==0,'a finished mission is inert')
end)
test('defeat_all completes on the last defeat; reach_goal ignores enemies',function()
  local m=sample() m.objective={type='defeat_all'}
  local st=Mission.new(Mission.validate(m))
  local a=Mission.step(st,obs(1,0)) report(st,a,10)
  Mission.step(st,obs(2,0,0,{[11]=true,[12]=true}))
  a=Mission.step(st,obs(3,0,0,{})) report(st,a,10)
  local _,e=Mission.step(st,obs(4,0,0,{})) assert(#of_type(e,'complete')==1)
  m=sample() m.objective={type='reach_goal'}
  st=Mission.new(Mission.validate(m))
  Mission.step(st,obs(1,0))
  _,e=Mission.step(st,obs(2,195,0,{})) assert(#of_type(e,'complete')==1)
end)
test('a mission with no enemies is cleared immediately',function()
  local st=Mission.new(Mission.validate({start={x=0,y=0},goal={x=5,y=0,w=4,h=4},objective={type='reach_goal'}}))
  local a,e=Mission.step(st,obs(1,0)) assert(#a==1 and a[1].type=='respawn_point' and #e==0)
  _,e=Mission.step(st,obs(2,5)) assert(#of_type(e,'complete')==1)
end)
test('the start is the first respawn point and each new checkpoint moves it',function()
  local st=Mission.new(Mission.validate(sample()))
  local a=Mission.step(st,obs(1,0,0))
  local r=of_type(a,'respawn_point') assert(#r==1 and r[1].x==0 and r[1].y==0,'start is the first respawn point')
  a=Mission.step(st,obs(2,0,0)) assert(#of_type(a,'respawn_point')==0,'announced once')
  local _,e
  a,e=Mission.step(st,obs(3,100,0)) assert(#of_type(e,'checkpoint')==1)
  r=of_type(a,'respawn_point') assert(#r==1 and r[1].x==100 and r[1].y==0)
  a,e=Mission.step(st,obs(4,100,0)) assert(#of_type(e,'checkpoint')==0 and #of_type(a,'respawn_point')==0,'one event per touch')
  _,e=Mission.step(st,obs(10,50,1)) -- falls 0 -> 1
  local d=of_type(e,'death') assert(#d==1 and d[1].deaths==1 and d[1].remaining==1)
  a=Mission.step(st,obs(400,nil,1)) assert(#a==0,'the engine respawns P1; the state machine does not teleport')
end)
test('overlapping checkpoints do not flip-flop',function()
  local m=sample() m.checkpoints={{x=100,y=0,w=20,h=40},{x=105,y=0,w=20,h=40}}
  local st=Mission.new(Mission.validate(m))
  Mission.step(st,obs(1,0,0))
  local n=0
  for f=2,10 do local a=Mission.step(st,obs(f,102,0)) n=n+#of_type(a,'respawn_point') end
  assert(n==1,'standing in two overlapping zones moves the respawn point once, got '..n)
end)
test('an enemy that leaves without a defeat is lost, not defeated, and does not stall the objective',function()
  local m=sample() m.enemies={{kind='goomba',x=50,y=0,wave=1}} m.objective={type='defeat_all'}
  local st=Mission.new(Mission.validate(m))
  local a=Mission.step(st,obs(1,0)) report(st,a,10)
  local _,e=Mission.step(st,obs(5,0,0,{},{})) -- gone, not in the defeated set
  assert(#of_type(e,'vanish')==1 and #of_type(e,'defeat')==0 and st.defeated==0 and st.vanished==1)
  local c=of_type(e,'complete')
  assert(#c==1 and c[1].defeated==0 and c[1].vanished==1,'defeat_all still completes')
end)
test('a stock-defeated enemy counts as defeated even while it still reports alive no more',function()
  local m=sample() m.enemies={{kind='goomba',x=50,y=0,wave=1}} m.objective={type='defeat_all'}
  local st=Mission.new(Mission.validate(m))
  local a=Mission.step(st,obs(1,0)) report(st,a,10)
  local _,e=Mission.step(st,obs(5,0,0,{},{[11]=true}))
  assert(#of_type(e,'defeat')==1 and st.defeated==1 and st.vanished==0)
end)
test('time runs out exactly at the limit',function()
  local st=Mission.new(Mission.validate(sample()))
  local _,e=Mission.step(st,obs(60*60-1,0)) assert(#of_type(e,'failed')==0)
  local a
  a,e=Mission.step(st,obs(60*60,0))
  local f=of_type(e,'failed') assert(#f==1 and f[1].reason=='time' and #of_type(a,'cleanup')==1)
  assert(Mission.result_text(st):find('time',1,true))
end)
test('lives run out on the last allowed death',function()
  local st=Mission.new(Mission.validate(sample()))
  Mission.step(st,obs(1,0,0))
  local _,e=Mission.step(st,obs(2,0,1)) assert(#of_type(e,'failed')==0)
  local d=of_type(e,'death') assert(d[1].remaining==1)
  _,e=Mission.step(st,obs(3,0,2))
  local f=of_type(e,'failed') assert(#f==1 and f[1].reason=='lives' and f[1].deaths==2)
end)
test('no lives set: deaths never fail the mission',function()
  local m=sample() m.objective={type='reach_goal'}
  local st=Mission.new(Mission.validate(m))
  Mission.step(st,obs(1,0,0))
  local _,e=Mission.step(st,obs(2,0,5)) assert(#of_type(e,'failed')==0 and #of_type(e,'death')==1)
end)
test('a refused spawn fails the mission with the reason',function()
  local st=Mission.new(Mission.validate(sample()))
  local a=Mission.step(st,obs(1,0))
  Mission.spawned(st,1,11) Mission.spawn_failed(st,2,'capacity')
  local a2,e=Mission.step(st,obs(2,0,0,{[11]=true}))
  local f=of_type(e,'failed') assert(#f==1 and f[1].reason=='spawn' and f[1].detail:find('capacity',1,true))
  assert(#of_type(a2,'cleanup')==1)
end)
test('hud summarizes objective, enemies, time and lives',function()
  local st=Mission.new(Mission.validate(sample()))
  local a=Mission.step(st,obs(60,0)) report(st,a,10)
  local h=Mission.hud(st)
  assert(h:find('enemies 3',1,true) and h:find('0:59',1,true) and h:find('lives 2',1,true),h)
  local m=sample() m.objective={type='reach_goal'} m.enemies={}
  st=Mission.new(Mission.validate(m)) Mission.step(st,obs(90,0))
  h=Mission.hud(st) assert(h:find('0:01',1,true) and not h:find('enemies',1,true) and not h:find('lives',1,true),h)
end)
test('main.lua embeds mission.lua verbatim (run tools/port/map_mission_sync.py)',function()
  local main=read(DIR..'main.lua')
  local body=read(DIR..'mission.lua')
  local block=main:match('%-%- BEGIN GENERATED MISSION[^\n]*\n(.-)%-%- END GENERATED MISSION')
  assert(block,'main.lua has no generated mission block')
  assert(block=='local Mission = (function()\n'..body..'end)()\n','embedded mission module is stale')
end)

-- Wave rules and trigger zones (pure). ------------------------------------------------------------
local function ruled()
  local m=sample()
  m.enemies={{kind='goomba',x=50,y=0,wave=1},{kind='koopa',x=60,y=0,wave=2},{kind='redead',x=80,y=0,wave=3}}
  m.waves={{wave=2,time=5},{wave=3,x=150}}
  m.objective={type='defeat_all'}
  return m
end
test('validate accepts wave rules and triggers and fills their defaults',function()
  local m=ruled()
  m.triggers={{x=0,y=0,w=10,h=10,action='wave',wave=2},{x=5,y=5,w=10,h=10,action='message',text='Go!',once=false},
              {x=1,y=1,w=4,h=4,action='collision',at={x=3,y=4},open=false},{x=0,y=0,w=2,h=2,action='complete'},
              {x=0,y=0,w=2,h=2,action='fail'}}
  local n=Mission.validate(m)
  assert(n.waves[2].dir==1 and n.waves[1].dir==nil and n.waves[1].time==5)
  assert(n.triggers[1].once==true and n.triggers[2].once==false and n.triggers[3].r==6.5 and n.triggers[3].open==false)
  assert(#n.triggers==5 and n.triggers[2].text=='Go!')
  local plainm=Mission.validate({start={x=0,y=0}}) assert(#plainm.waves==0 and #plainm.triggers==0)
end)
test('validate refuses malformed wave rules and triggers',function()
  local function with(f) local m=ruled() m.triggers={} f(m) return m end
  refuses(with(function(m) m.waves[1].time=nil end),'exactly one')
  refuses(with(function(m) m.waves[1].x=3 end),'exactly one')
  refuses(with(function(m) m.waves[1].wave=0 end),'wave rule')
  refuses(with(function(m) m.waves[2].wave=2 end),'two rules')
  refuses(with(function(m) m.waves[1].time=-1 end),'time')
  refuses(with(function(m) m.waves[1].time=3601 end),'time')
  refuses(with(function(m) m.waves[2].dir=2 end),'dir')
  refuses(with(function(m) m.waves[1].dir=1 end),'dir')
  refuses(with(function(m) m.waves[2].x=0/0 end),'finite')
  refuses(with(function(m) m.waves[1].bogus=1 end),'unknown wave rule field')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='explode'}} end),'action')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=0,h=10,action='complete'}} end),'trigger')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='wave'}} end),'trigger wave')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='wave',wave=9}} end),'trigger wave')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='message'}} end),'text')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='message',text=''}} end),'text')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='message',text='a\nb'}} end),'text')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='message',text=('x'):rep(81)}} end),'text')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='collision',open=true}} end),'trigger at')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='collision',at={x=0,y=0}}} end),'open')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='collision',at={x=0,y=0},open=true,r=0}} end),'trigger r')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='complete',text='no'}} end),'only goes with')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='complete',once='yes'}} end),'once')
  refuses(with(function(m) m.triggers={{x=0,y=0,w=10,h=10,action='complete',hp=1}} end),'unknown trigger field')
  refuses(with(function(m) m.triggers={} for i=1,17 do m.triggers[i]={x=0,y=0,w=1,h=1,action='complete'} end end),'triggers')
end)
test('check_playable rejects rules and triggers that point at empty waves',function()
  local m=ruled() m.waves[1].wave=5
  local why=Mission.check_playable(Mission.validate(m)) assert(why and why:find('wave 5',1,true),why)
  m=ruled() m.triggers={{x=0,y=0,w=5,h=5,action='wave',wave=7}}
  why=Mission.check_playable(Mission.validate(m)) assert(why and why:find('wave 7',1,true),why)
  assert(Mission.check_playable(Mission.validate(ruled()))==nil)
end)
test('a timed wave appears at its time even while wave 1 is still alive',function()
  local st=Mission.new(Mission.validate(ruled()))
  local a=Mission.step(st,obs(1,0)) report(st,a,10)
  assert(#of_type(a,'spawn')==1 and of_type(a,'spawn')[1].kind=='goomba')
  a=Mission.step(st,obs(299,0,0,{[11]=true})) assert(#of_type(a,'spawn')==0)
  local e
  a,e=Mission.step(st,obs(300,0,0,{[11]=true}))
  local sp=of_type(a,'spawn') assert(#sp==1 and sp[1].kind=='koopa','wave 2 starts at 5 s, got '..#sp)
  local w=of_type(e,'wave') assert(#w==1 and w[1].wave==2 and w[1].n==2)
  report(st,a,20)
  a=Mission.step(st,obs(301,0,0,{[11]=true,[22]=true})) assert(#of_type(a,'spawn')==0,'a wave starts once')
end)
test('a conditional wave starts when P1 crosses x and not before; dir=-1 watches the other way',function()
  local st=Mission.new(Mission.validate(ruled()))
  local a=Mission.step(st,obs(1,0)) report(st,a,10)
  a=Mission.step(st,obs(2,149.9,0,{[11]=true})) assert(#of_type(a,'spawn')==0)
  a=Mission.step(st,obs(3,150,0,{[11]=true}))
  local sp=of_type(a,'spawn') assert(#sp==1 and sp[1].kind=='redead')
  local m=ruled() m.waves[2].dir=-1 m.waves[2].x=-50
  st=Mission.new(Mission.validate(m)) a=Mission.step(st,obs(1,0)) report(st,a,10)
  a=Mission.step(st,obs(2,-49,0,{[11]=true})) assert(#of_type(a,'spawn')==0)
  a=Mission.step(st,obs(3,-50,0,{[11]=true})) assert(#of_type(a,'spawn')==1)
  a=Mission.step(st,obs(4,nil,0,{[11]=true,[33]=true})) assert(#of_type(a,'spawn')==0,'no player: no condition')
end)
test('unruled waves still go in order and a ruled one that has not started blocks the clear',function()
  local m=ruled() m.waves={{wave=2,time=5}}
  local st=Mission.new(Mission.validate(m))
  local a=Mission.step(st,obs(1,0)) report(st,a,10) -- wave 1
  a=Mission.step(st,obs(3,0,0,{})) -- wave 1 dead: wave 3 is unruled and follows at once
  local sp=of_type(a,'spawn') assert(#sp==1 and sp[1].kind=='redead',#sp) report(st,a,30)
  assert(not st.cleared)
  a=Mission.step(st,obs(4,0,0,{[33]=true}))
  assert(not st.cleared,'wave 2 has not started, so the mission is not cleared')
  a=Mission.step(st,obs(300,0,0,{[33]=true})) report(st,a,20)
  a=Mission.step(st,obs(301,0,0,{})) -- everything gone
  assert(st.cleared and st.result and st.result.status=='complete')
end)
test('trigger zones fire on entry once and can be repeatable',function()
  local m=sample() m.triggers={{x=100,y=0,w=10,h=10,action='message',text='hi'},{x=100,y=0,w=10,h=10,action='message',text='again',once=false}}
  m.objective={type='reach_goal'} m.enemies={}
  local st=Mission.new(Mission.validate(m))
  local _,e=Mission.step(st,obs(1,0)) assert(#of_type(e,'message')==0)
  _,e=Mission.step(st,obs(2,100)) local ms=of_type(e,'message') assert(#ms==2 and ms[1].text=='hi')
  _,e=Mission.step(st,obs(3,101)) assert(#of_type(e,'message')==0,'staying inside does not refire')
  _,e=Mission.step(st,obs(4,50)) _,e=Mission.step(st,obs(5,100))
  ms=of_type(e,'message') assert(#ms==1 and ms[1].text=='again','only the repeatable one fires again')
end)
test('a wave trigger spawns its wave at once and removes it from the sequence',function()
  local m=sample() m.enemies={{kind='goomba',x=50,y=0,wave=1},{kind='koopa',x=60,y=0,wave=2}}
  m.triggers={{x=100,y=0,w=10,h=10,action='wave',wave=2}} m.objective={type='defeat_all'}
  local st=Mission.new(Mission.validate(m))
  local a=Mission.step(st,obs(1,0)) report(st,a,10)
  a=Mission.step(st,obs(3,0,0,{})) -- wave 1 dead, wave 2 is trigger-owned: nothing spawns, not cleared
  assert(#of_type(a,'spawn')==0 and not st.cleared)
  local e
  a,e=Mission.step(st,obs(4,100,0,{}))
  local sp=of_type(a,'spawn') assert(#sp==1 and sp[1].kind=='koopa')
  assert(#of_type(e,'trigger')==1 and of_type(e,'trigger')[1].action=='wave')
end)
test('a collision trigger asks the glue to open or close the parts near a point',function()
  local m=sample() m.objective={type='reach_goal'} m.enemies={}
  m.triggers={{x=100,y=0,w=10,h=10,action='collision',at={x=7,y=8},r=20,open=true}}
  local st=Mission.new(Mission.validate(m))
  Mission.step(st,obs(1,0))
  local a=Mission.step(st,obs(2,100)) local c=of_type(a,'collision')
  assert(#c==1 and c[1].x==7 and c[1].y==8 and c[1].r==20 and c[1].open==true)
end)
test('complete and fail triggers end the mission whatever the objective says',function()
  local m=sample() m.triggers={{x=100,y=0,w=10,h=10,action='complete'}}
  local st=Mission.new(Mission.validate(m)) Mission.step(st,obs(1,0))
  local a,e=Mission.step(st,obs(2,100))
  assert(#of_type(e,'complete')==1 and #of_type(a,'cleanup')==1 and st.result.status=='complete')
  m.triggers={{x=100,y=0,w=10,h=10,action='fail'}}
  st=Mission.new(Mission.validate(m)) Mission.step(st,obs(1,0))
  a,e=Mission.step(st,obs(2,100)) local f=of_type(e,'failed')
  assert(#f==1 and f[1].reason=='trigger' and Mission.result_text(st):find('FAILED',1,true))
end)
test('empty ignores nothing: rules and triggers make a mission non-empty',function()
  assert(Mission.empty(Mission.validate({})))
  assert(not Mission.empty(Mission.validate({triggers={{x=0,y=0,w=1,h=1,action='complete'}}})))
  assert(not Mission.empty(Mission.validate({waves={{wave=1,time=2}}})))
end)

-- Part 2: the editor commands and the runtime glue against the editor's gd stub. ------------------
local fixture=read('pc/tests/map_editor_test.lua')
local prefix=assert(fixture:match('^(.-)local chunk = loadfile'),'stub prefix not found')
local EXTRA=[==[
local live_spawns={}
gd.stage_set_spawn=function(slot,x,y)live_spawns[slot]={x=x,y=y};stub_spawns[#stub_spawns+1]={slot,x,y};return true end
gd.stage_spawn=function(slot)local s=live_spawns[slot] or{x=12,y=34};return s.x,s.y,0 end
gd.stage_restore_bounds=function()stub_camera=nil;stub_blast=nil;live_spawns={};return true end
local spawned,alive_set,removed,stocks_set,fail_enemy,next_enemy={}, {}, {}, nil, false, 100
gd.spawn_enemy=function(kind,x,y,o)
  if fail_enemy then return nil,'enemy capacity or item data unavailable' end
  next_enemy=next_enemy+1 spawned[#spawned+1]={kind=kind,x=x,y=y,facing=o and o.facing,handle=next_enemy}
  alive_set[next_enemy]=true return next_enemy
end
gd.enemy_alive=function(h) return alive_set[h]==true end
local defeated_set={}
gd.enemy_status=function(h) return alive_set[h]==true and 'alive' or defeated_set[h] and 'defeated' or 'removed' end
local enemy_at={}
gd.enemy_state=function(h) local a=enemy_at[h] return alive_set[h]==true and {x=a and a.x or 0,y=a and a.y or 0} or nil end
gd.enemy_remove=function(h) local was=alive_set[h] alive_set[h]=nil removed[#removed+1]=h return was==true end
gd.set_stocks=function(_,n) stocks_set=n return true end
p.falls,p.stocks=0,4
assert(loadfile('pc/scripts/examples/map_editor/scripts/main.lua'))()
]==]
local BODY=[==[
local failed_n,passed_n=0,0
local function check(name,fn)
  local ok,why=pcall(fn) if ok then passed_n=passed_n+1 else failed_n=failed_n+1 print('FAIL editor: '..name..': '..tostring(why)) end
end
local function command(s) commands.map(s) end
local function doc() command('save snap.lua') return files['snap.lua'] end
local function at(x,y) p.x,p.y=x,y end
local function refused(cmd,pat)
  logs={} local before=doc() command(cmd)
  assert(logged('error:'),'expected a refusal for: '..cmd)
  assert(pat==nil or logged(pat),'refusal should mention "'..tostring(pat)..'" for: '..cmd)
  assert(doc()==before,'a refused command changed the document: '..cmd)
end
local function reset_run() spawned,alive_set,removed={}, {}, {} defeated_set={} stocks_set=nil end
local function kill(h) alive_set[h]=nil defeated_set[h]=true end
local function lose(h) alive_set[h]=nil end
local function fresh() reset_run() logs={} p.falls=0 end
local function frames(n,fn) for i=1,n do if fn then fn(i) end on_frame() end end
local function build_full()
  command('mission clear')
  at(0,0) command('mission start')
  at(13,26) command('mission enemy goomba') command('mission enemy koopa 1') command('mission enemy redead 2')
  at(130,26) command('mission checkpoint 10 40')
  at(260,26) command('mission goal 20 40')
  command('mission objective defeat_then_goal time=60 lives=2')
end
command('on') command('ghost off')

command('mission clear')
check('start defaults to the cursor, uses the spawn mechanism and is one undo step',function()
  command('mission clear') at(13,26) local base=doc()
  command('mission start')
  local s=live_spawns[0] assert(s and s.x==13 and s.y==26,'start must reach stage_set_spawn slot 0')
  local saved=doc()
  assert(saved:find('version=2',1,true) and saved:find('mission=',1,true))
  command('undo') assert(doc()==base and live_spawns[0]==nil,'undo restores the document and the native spawn')
  command('redo') assert(live_spawns[0].x==13 and doc()==saved)
  command('mission start 65 52') assert(live_spawns[0].x==65 and live_spawns[0].y==52)
  command('undo') assert(live_spawns[0].x==13)
  command('mission clear')
end)

check('enemies, goal, checkpoints and objective edit the document, each as one undo step',function()
  local base=doc()
  at(0,0) command('mission start') at(13,26) command('mission enemy goomba') command('mission enemy koopa 2')
  at(130,26) command('mission checkpoint 10 40') at(260,26) command('mission goal 20 40')
  command('mission objective reach_goal time=90 lives=3')
  local text=doc()
  assert(text:find('goomba',1,true) and text:find('koopa',1,true) and text:find('reach_goal',1,true))
  for _=1,6 do command('undo') end
  assert(doc()==base,'six mission edits are six undo steps')
  for _=1,6 do command('redo') end
  assert(doc()==text)
end)
check('bad commands are refused and leave the document alone',function()
  build_full()
  refused('mission enemy dragon','unknown enemy kind')
  refused('mission enemy goomba 0','wave')
  refused('mission enemy goomba 9','wave')
  refused('mission enemy goomba x','wave')
  refused('mission enemy','kind')
  refused('mission goal 0 5','goal')
  refused('mission goal 5','goal')
  refused('mission checkpoint -1 5','checkpoint')
  refused('mission objective fly','objective')
  refused('mission objective reach_goal time=0','time')
  refused('mission objective reach_goal lives=100','lives')
  refused('mission objective reach_goal lives=abc','lives')
  refused('mission objective reach_goal bogus=1','option')
  refused('mission start 1','x y')
  refused('mission start nan nan','finite')
  refused('mission delete 99','index')
  refused('mission delete x','index')
  refused('mission frobnicate','mission')
end)
check('enemy and checkpoint limits hold',function()
  command('mission clear')
  for _=1,32 do command('mission enemy goomba 1') end
  refused('mission enemy goomba 1','wave 1')
  for _=1,32 do command('mission enemy goomba 2') end
  refused('mission enemy goomba 3','enemies')
  command('mission clear')
  for _=1,16 do command('mission checkpoint 5 5') end
  refused('mission checkpoint 5 5','checkpoint')
  command('mission clear')
end)
check('list and delete work by index',function()
  build_full() logs={}
  command('mission list')
  assert(logged('start') and logged('goomba') and logged('checkpoint') and logged('goal') and logged('defeat_then_goal'))
  local before=doc()
  command('mission delete 1') assert(live_spawns[0]==nil,'deleting the start frees the native spawn')
  assert(not doc():find('start=',1,true))
  command('undo') assert(doc()==before and live_spawns[0])
  command('mission delete 3') assert(doc()~=before)
  command('undo') assert(doc()==before)
  command('mission clear') assert(not doc():find('mission=',1,true) and live_spawns[0]==nil)
  assert(doc():find('version=1',1,true),'a layout without mission or arena fields is v1 again')
  command('undo') assert(doc()==before)
  command('mission clear')
end)
check('save and load round trip a mission byte for byte',function()
  build_full() command('save m.lua') local text=files['m.lua']
  assert(select(2,text:gsub('mission={',''))==1,'the mission is written exactly once')
  command('mission clear') assert(not doc():find('mission=',1,true))
  command('load m.lua')
  assert(doc()==text,'load then save must reproduce the file') assert(live_spawns[0] and live_spawns[0].x==0)
  local data=assert(load(text,'m','t',{}))()
  assert(data.version==2 and data.mission.objective.type=='defeat_then_goal' and #data.mission.enemies==3)
  assert(data.mission.enemies[3].wave==2 and data.mission.goal.w==20 and data.mission.checkpoints[1].x==130)
  command('undo') assert(not doc():find('mission=',1,true),'load is one undo step')
  command('mission clear')
end)
check('a bad mission table refuses the load and keeps the current document',function()
  build_full() local keep=doc()
  local function bad(name,mission,version)
    files[name]='return {version='..(version or 2)..',units=6.5,parts={},mission='..mission..'}'
    logs={} command('load '..name)
    assert(logged('error:'),'load should refuse '..name) assert(doc()==keep,'refused load changed the document: '..name)
  end
  bad('b1.lua','{enemies={{kind="dragon",x=0,y=0}}}')
  bad('b2.lua','{goal={x=0/0,y=0,w=1,h=1}}')
  bad('b3.lua','{objective={type="win"}}')
  bad('b4.lua','{start={x=0}}')
  bad('b5.lua','{surprise=true}')
  bad('b6.lua','"text"')
  bad('b7.lua','{objective={type="reach_goal",lives=0}}')
  bad('b8.lua','{start={x=0,y=0}}',1) -- a v1 file has no mission
  bad('b9.lua','{enemies={{kind="goomba",x=0,y=0,wave=99}}}')
  files['b10.lua']='return {version=2,units=6.5,parts={},mission={checkpoints=setmetatable({},{})}}'
  logs={} command('load b10.lua') assert(logged('error:') and doc()==keep)
  command('mission clear')
end)
check('old v1 and v2 layouts load unchanged and carry no mission',function()
  files['v1.lua']='return {version=1,units=6.5,parts={{part="bf_floor_4m",x=0,y=0,z=0,rot=0,collision=true,floor_flags=3}}}'
  files['v2.lua']='return {version=2,units=6.5,spawn={[4]={x=-10,y=5}},parts={}}'
  command('load v1.lua') local t1=doc() assert(t1:find('version=1',1,true) and not t1:find('mission',1,true))
  command('load v2.lua') local t2=doc() assert(t2:find('version=2',1,true) and not t2:find('mission',1,true))
  assert(live_spawns[4].x==-10)
  command('mission list') assert(logged('no mission'))
end)

check('runtime: waves, checkpoint respawn, win, restart',function()
  command('load v1.lua') build_full() reset_run() logs={}
  command('mission play')
  assert(not flying,'play leaves editing') assert(p.x==0 and p.y==0,'P1 is placed at the start')
  assert(#spawned==2 and spawned[1].kind=='goomba' and spawned[2].kind=='koopa','wave 1 spawns at once')
  assert(spawned[1].facing==-1 and spawned[1].x==13 and spawned[1].y==26)
  assert(stocks_set==3,'lives+1 stocks: the engine game-over never beats the mission failure')
  on_draw()
  frames(3) assert(#spawned==2)
  kill(spawned[1].handle) frames(2) assert(#spawned==2,'wave 2 waits for the whole wave')
  kill(spawned[2].handle) frames(1) assert(#spawned==3 and spawned[3].kind=='redead')
  kill(spawned[3].handle) frames(2)
  at(260,26) frames(2) assert(logged('mission: complete'),'cleared then in the goal completes the mission')
  on_draw()
  command('mission restart') assert(p.x==0 and #spawned==5,'restart replays from the start')
  assert(live_spawns[4] and live_spawns[4].x==0 and live_spawns[4].y==0,'the start is the respawn point before any checkpoint')
  logs={} p.falls=0
  at(130,26) frames(2) assert(logged('checkpoint'))
  at(130,26) frames(2)
  assert(live_spawns[4].x==130 and live_spawns[4].y==26,'a touched checkpoint becomes the respawn slot 4 point')
  p.falls=1 at(5000,5000) frames(300)
  assert(p.x==5000,'the mission never teleports P1 itself: the engine respawn does the move')
  command('mission stop')
  assert(live_spawns[4]==nil,'stopping gives the respawn slot back to the document')
end)
check('runtime: an enemy that ends inside the stage (a Koopa killed into a shell) is a defeat',function()
  command('on') command('mission clear')
  at(0,0) command('mission start') at(13,26) command('mission enemy koopa') at(260,26) command('mission goal 20 40')
  command('mission objective defeat_then_goal') reset_run() enemy_at={} logs={}
  command('mission play') local h=spawned[1].handle
  enemy_at[h]={x=13,y=26} frames(2) lose(h) frames(2)
  assert(logged('mission: defeated koopa') and not logged('mission: lost'),'a removal mid-stage is a defeat')
  command('mission stop') command('on')
end)
check('runtime: an enemy that leaves the stage is lost, not defeated',function()
  command('on') command('mission clear')
  at(0,0) command('mission start') at(13,26) command('mission enemy goomba') at(260,26) command('mission goal 20 40')
  command('mission objective defeat_then_goal') reset_run() logs={}
  command('mission play') local h=spawned[1].handle
  enemy_at[h]={x=-110,y=60} frames(2) -- last seen 10 units from the blast zone's left edge (-120)
  lose(h) frames(2)
  assert(logged('mission: lost goomba'),'a vanished enemy is logged as lost '..table.concat(logs,' / '))
  at(260,26) frames(2) assert(logged('mission: complete') and logged('defeated=0 vanished=1'),table.concat(logs,' / '))
  command('mission stop') command('on')
end)
check('runtime: time-out, lives-out and cleanup',function()
  build_full() fresh()
  command('mission play') local first=spawned[1].handle
  frames(60*60-1) assert(not logged('mission: failed'))
  frames(1) assert(logged('mission: failed') and logged('time'))
  assert(removed[1]==first and not alive_set[first],'its enemies are removed on failure')
  on_draw()
  fresh() command('mission restart')
  p.falls=1 frames(1) p.falls=2 frames(1)
  assert(logged('mission: failed') and logged('lives'))
  assert(next(alive_set)==nil,'no enemy outlives the mission')
end)
check('runtime: leaving the match, editing again and unloading clean up',function()
  build_full() reset_run() command('mission play') assert(next(alive_set))
  on_match_end() assert(next(alive_set)==nil,'match end removes enemies') assert(logged('mission: aborted'))
  reset_run() command('mission play') assert(next(alive_set))
  command('on') assert(next(alive_set)==nil and flying,'returning to edit mode removes enemies')
  reset_run() command('mission play') assert(next(alive_set))
  command('off') assert(next(alive_set)==nil and logged('mission: aborted (map off)'),'map off ends a running mission too')
  command('on')
  reset_run() on_savestate(1) command('mission play') assert(next(alive_set))
  on_loadstate(1) assert(next(alive_set)==nil,'a savestate load drops the mission')
  reset_run() command('mission play') assert(next(alive_set))
  on_unload() assert(next(alive_set)==nil)
  command('on')
end)
check('runtime: a refused spawn fails the mission rather than hanging it',function()
  build_full() reset_run() logs={} fail_enemy=true command('mission play') fail_enemy=false
  frames(1) assert(logged('mission: failed') and logged('spawn'))
end)
check('map play with a mission layout starts the mission; an unplayable one only loads',function()
  build_full() command('save play.lua') reset_run() logs={}
  command('play play.lua') assert(#spawned==2 and not flying,'map play runs the mission')
  command('mission stop')
  files['half.lua']='return {version=2,units=6.5,parts={},mission={start={x=0,y=0}}}'
  reset_run() logs={} command('play half.lua')
  assert(#spawned==0 and logged('objective'),'an incomplete mission refuses to run but the layout loads')
  command('on')
end)
check('mission play [file] loads the file first; online and unknown states refuse',function()
  command('on') build_full() command('save direct.lua') command('mission clear') reset_run()
  command('mission play direct.lua') assert(#spawned==2)
  command('mission stop') command('on')
  command('mission clear') fresh() command('mission play') assert(logged('start') and #spawned==0)
  online=true logs={} command('mission play') assert(logged('error:'),'online refuses') online=false
  end)
check('on_match_start with a mission autoload begins the mission',function()
  command('on') build_full() command('save auto.lua') reset_run() command('play auto.lua') command('mission stop')
  on_match_end() reset_run() on_match_start()
  assert(#spawned==2,'a new match runs the autoloaded mission')
  command('mission stop')
end)
check('the shipped sample loads, validates and plays',function()
  files['first_mission.lua']=SAMPLE_TEXT command('on') logs={}
  command('load first_mission.lua') assert(not logged('error:'),'sample refused to load')
  local data=assert(load(SAMPLE_TEXT,'s','t',{}))()
  assert(#data.parts==11 and #data.mission.enemies==3 and data.mission.enemies[3].wave==2)
  assert(live_spawns[0] and live_spawns[0].x==-120)
  reset_run() command('play first_mission.lua')
  assert(#spawned==2 and p.x==-120 and not flying,'the sample starts with wave 1')
  command('mission stop') command('on')
end)
check('the editor overlay draws markers',function()
  command('on') build_full()
  local drawn={}
  local keep_box,keep_text=gd.box,gd.kit.text
  gd.box=function(...) drawn[#drawn+1]=select('#',...) end
  gd.kit.text=function(_,_,t) drawn[#drawn+1]=t end
  on_draw()
  gd.box,gd.kit.text=keep_box,keep_text
  local labels={} for _,v in ipairs(drawn) do if type(v)=='string' then labels[#labels+1]=v end end
  labels=table.concat(labels,'|')
  assert(labels:find('START',1,true) and labels:find('GOAL',1,true) and labels:find('CP1',1,true) and labels:find('goomba',1,true),labels)
end)
check('the action menu offers the mission actions',function()
  command('on') build_full()
  for _,name in ipairs({'mission: start at cursor','mission: play','mission: restart'}) do
    reset_run() logs={} command('run '..name) assert(not logged('No matching action'),name)
  end
  command('mission stop') command('on')
end)

check('wave rules: set, replace, clear, list, undo',function()
  command('on') command('mission clear') command('clear')
  at(0,0) command('mission start') at(13,26) command('mission enemy goomba 1') command('mission enemy koopa 2')
  local base=doc()
  command('mission wave 2 time 12')
  assert(doc():find('time=12',1,true) and doc():find('waves=',1,true))
  command('mission wave 2 x 150 left') assert(doc():find('x=150',1,true) and doc():find('dir=-1',1,true) and not doc():find('time=12',1,true),'a new rule replaces the old one')
  at(65,0) command('mission wave 2 x') assert(doc():find('x=65',1,true) and doc():find('dir=1',1,true),'x defaults to the cursor')
  logs={} command('mission list') assert(logged('wave 2 when P1 passes x=65 going right'))
  command('mission wave 2 clear') assert(not doc():find('waves=',1,true))
  command('undo') assert(doc():find('x=65',1,true))
  for _=1,3 do command('undo') end assert(doc()==base,'rule edits are undo steps')
  refused('mission wave 2','wave')
  refused('mission wave 2 time abc','number')
  refused('mission wave 2 time 99999','time')
  refused('mission wave 2 x 5 up','direction')
  refused('mission wave 2 sideways','time, x or clear')
  command('mission clear')
end)
check('trigger commands: every action, repeat, refusals, list and delete',function()
  command('mission clear') command('clear')
  at(13,26) command('place') command('select')
  at(100,40) command('mission trigger wave 20 20 2')
  command('mission trigger message 20 20 Watch out below')
  command('mission trigger message 20 20 again repeat')
  command('mission trigger collision 20 20 open 30')
  command('mission trigger complete 8 8')
  command('mission trigger fail 8 8')
  local text=doc()
  assert(text:find('action="wave"',1,true) and text:find('text="Watch out below"',1,true))
  assert(text:find('once=false',1,true) and text:find('open=true',1,true) and text:find('action="fail"',1,true))
  local data=assert(load(text,'t','t',{}))()
  assert(#data.mission.triggers==6 and data.mission.triggers[4].at.x==13 and data.mission.triggers[4].r==30)
  assert(data.mission.triggers[3].text=='again' and data.mission.triggers[3].once==false)
  logs={} command('mission list') assert(logged('trigger message "Watch out below"') and logged('trigger collision open near 13.0'))
  command('mission delete 1') assert(not doc():find('action="wave"',1,true)) command('undo')
  refused('mission trigger','trigger')
  refused('mission trigger wave 20 20','wave number')
  refused('mission trigger message 20 20','text')
  refused('mission trigger collision 20 20 sideways','open or close')
  refused('mission trigger complete 20 20 extra','no arguments')
  refused('mission trigger explode 5 5','action')
  refused('mission trigger wave 0 20 2','trigger')
  command('clear') refused('mission trigger collision 20 20 open','select a part')
  command('mission clear') command('clear')
end)
check('runtime: a timed wave, a wave trigger and a message trigger',function()
  command('mission clear') command('clear')
  at(0,0) command('mission start')
  at(13,26) command('mission enemy goomba 1') at(39,26) command('mission enemy koopa 2') at(65,26) command('mission enemy redead 3')
  command('mission wave 2 time 2')
  at(130,26) command('mission trigger wave 20 40 3')
  at(60,26) command('mission trigger message 30 40 Halfway there')
  at(260,26) command('mission goal 20 40')
  command('mission objective defeat_then_goal')
  reset_run() logs={} command('mission play')
  assert(#spawned==1 and spawned[1].kind=='goomba')
  frames(119) assert(#spawned==1,'wave 2 waits for its time')
  frames(1) assert(#spawned==2 and spawned[2].kind=='koopa','wave 2 appears after 2 s with wave 1 still alive')
  at(60,26) frames(2) assert(logged('mission: message Halfway there'))
  on_draw() -- the message line draws
  at(0,0) frames(1) at(130,26) frames(2)
  assert(#spawned==3 and spawned[3].kind=='redead','the trigger spawns wave 3')
  command('mission stop') command('on')
end)
check('runtime: a collision trigger opens a part for the run only',function()
  command('mission clear') command('clear')
  at(13,26) command('part bf_floor_4m') command('place') command('select')
  local pid=nil
  for h,m in pairs(models) do if m.collision then pid=h end end
  assert(pid and models[pid].collision==true,'the placed part owns collision')
  at(0,0) command('mission start')
  at(100,40) command('mission trigger collision 20 20 open')
  at(260,26) command('mission goal 20 40') command('mission objective reach_goal')
  local before=doc()
  reset_run() logs={} command('mission play')
  at(100,40) frames(2)
  assert(logged('mission: collision open on 1 part'),'the trigger found the part')
  local open=false for _,m in pairs(models) do if m.collision==false and m.floor_flags~=0 then open=true end end
  assert(open,'its instance now has no collision')
  command('mission stop')
  local closed=false for _,m in pairs(models) do if m.collision==true then closed=true end end
  assert(closed,'stopping restores the document collision')
  assert(doc()==before,'the document was never touched')
  command('mission clear') command('clear')
end)
check('map mission test: starts at the cursor, leaves the document alone and returns to the cursor',function()
  command('on') build_full() at(130,26)
  local before=doc()
  reset_run() logs={} command('mission test')
  assert(not flying and p.x==130 and p.y==26,'P1 starts at the cursor, not at the authored start')
  assert(#spawned==2 and logged('test from the cursor'),'the mission itself runs')
  assert(live_spawns[4].x==130 and live_spawns[4].y==26,'the test start is the first respawn point')
  at(500,500) frames(2)
  assert(doc()==before,'testing never changes the document')
  command('mission stop')
  assert(flying and p.x==130 and p.y==26,'stopping returns to editing at the cursor the test left from')
  assert(next(alive_set)==nil,'its enemies are gone')
  assert(doc()==before)
  -- F6 / map on during a test does the same
  at(40,26) command('mission test') at(300,300) assert(not flying)
  command('on') assert(flying and p.x==40 and p.y==26 and next(alive_set)==nil,'map on ends a test and returns')
  -- restart keeps testing from the same place
  at(70,26) command('mission test') at(200,200) command('mission restart')
  assert(p.x==70 and p.y==26 and #spawned>=2,'restart repeats the test from the same spot')
  command('mission stop')
  assert(flying and p.x==70)
  -- a normal play is not a test: stop does not re-enter the editor
  command('mission play') command('mission stop') assert(not flying)
  command('on') at(5,5) command('mission clear') command('mission test') assert(logged('error:') and flying,'no objective: refused, still editing')
  refused('mission test','objective')
  command('mission clear')
end)
check('map mission test: flight refused for a few frames after the test is retried, then granted',function()
  command('on') build_full() at(130,26)
  command('mission test')
  local real=gd.fly local refusals=13 local nudges,released=0,0
  gd.fly=function(port,v) if v==true and refusals>0 then refusals=refusals-1 error('gd.fly: that fighter state cannot fly') end return real(port,v) end
  gd.input=function(port,spec,n) nudges=nudges+1 assert(port==1 and spec.y==-100) return true end
  gd.release_pad=function() released=released+1 end
  command('mission stop') assert(not flying,'first attempt refused')
  frames(20) gd.fly=real gd.input=nil gd.release_pad=nil
  assert(flying and p.x==130 and p.y==26,'a later frame gets the editor back at the cursor')
  assert(nudges>=1 and released==1,'a stuck state is nudged with the stick, and the pad claim is released once editing resumed')
  command('mission clear')
end)
-- Mouse authoring. The stub projects world to screen 1:1, so a click at (x, y) is the world point (x, y).
-- Positions are multiples of 13 (the grid); the side panels own x < 254 and x >= 400.
local function pointer(x,y,b) mouse_state={x=x,y=y,buttons=b} on_frame_pre() end
local function click(x,y) pointer(x,y,1) pointer(x,y,0) end
local function drag(x1,y1,x2,y2) pointer(x1,y1,1) pointer(x2,y2,1) pointer(x2,y2,0) end
local function mdata() return assert(load(doc(),'d','t',{}))().mission end
local function fresh_mission() command('on') command('mission clear') command('clear') command('tool select') end
check('mouse mapping inverts a real perspective projection (not only the identity stub)',function()
  command('on') command('clear') command('tool place')
  local keep_project,keep_cam=gd.project,camera
  -- an asymmetric homography, the shape of the engine's projection of the z = 0 plane
  gd.project=function(x,y) local w=1+0.001*x+0.002*y return (6*x+0.5*y+300)/w,(0.3*x-5*y+260)/w,true end
  camera={interest={x=11,y=22,z=0},eye={x=13,y=24,z=140},fov=30,roll=0,mode=0}
  local ok,why=pcall(function()
  for _,w in ipairs({{13,13},{0,26},{13,0}}) do
    local sx,sy=gd.project(w[1],w[2],0)
    command('clear') pointer(sx,sy,1) pointer(sx,sy,0)
    local part=assert(load(doc(),'d','t',{}))().parts[1]
    assert(part and math.abs(part.x-w[1])<1e-6 and math.abs(part.y-w[2])<1e-6,
           ('click at the projection of %d,%d placed %s,%s'):format(w[1],w[2],tostring(part and part.x),tostring(part and part.y)))
  end
  end)
  gd.project,camera=keep_project,keep_cam
  command('clear')
  assert(ok,why)
end)
check('mission tool: click places each picked kind, one undo step each, and selects it',function()
  fresh_mission() local base=doc()
  command('mission pick enemy koopa') assert(logged('Click places: enemy'))
  click(286,208) local m=mdata()
  assert(#m.enemies==1 and m.enemies[1].kind=='koopa' and m.enemies[1].x==286 and m.enemies[1].y==208 and m.enemies[1].wave==1)
  command('undo') assert(doc()==base,'a click is one undo step') command('redo')
  command('mission pick start') click(273,260) m=mdata()
  assert(m.start.x==273 and m.start.y==260 and live_spawns[0].x==273,'start uses the spawn mechanism')
  command('mission pick checkpoint') click(325,208) m=mdata()
  assert(#m.checkpoints==1,'cp count '..#m.checkpoints..' '..doc())
  assert(m.checkpoints[1].w==26 and m.checkpoints[1].h==60 and m.checkpoints[1].x==325,'cp geometry')
  command('mission pick goal') click(377,208) m=mdata() assert(m.goal and m.goal.x==377 and m.goal.w==26,'goal: '..doc())
  command('mission pick trigger complete') click(299,299) m=mdata()
  assert(#m.triggers==1 and m.triggers[1].action=='complete' and m.triggers[1].once~=false)
  command('mission pick trigger message') click(325,299) m=mdata()
  assert(m.triggers[2].text=='Message')
  refused('mission pick dragon','pick')
  refused('mission pick enemy dragon','enemy kind')
  refused('mission pick trigger explode','trigger action')
  refused('mission pick start x','second word')
  command('mission clear')
end)
check('mission tool: clicking a marker selects it and dragging moves it as one undo step',function()
  fresh_mission()
  command('mission pick enemy goomba') click(286,208) command('mission pick start') click(273,260)
  local before=doc()
  drag(286,208,338,234) -- onto the enemy, then to (338, 234) (snapped)
  local m=mdata()
  assert(m.enemies[1].x==338 and m.enemies[1].y==234,'the enemy follows the pointer, got '..m.enemies[1].x..','..m.enemies[1].y)
  assert(#m.enemies==1,'dragging a marker does not place another one')
  command('undo') assert(doc()==before,'the whole drag is one undo step') command('redo')
  -- dragging the start moves the native spawn
  drag(273,260,299,286) m=mdata() assert(m.start.x==299 and m.start.y==286 and live_spawns[0].x==299)
  command('undo') assert(live_spawns[0].x==273)
  -- a click without movement only selects
  local text=doc() click(338,234) assert(doc()==text,'a click on a marker changes nothing')
  command('mission clear')
end)
check('mission tool: dragging a selected zone edge resizes it',function()
  fresh_mission()
  command('mission pick checkpoint') click(325,260) -- zone 312..338 x 230..290
  local before=doc()
  drag(338,260,390,260) -- the right edge to x=390
  local m=mdata()
  assert(m.checkpoints[1].w==78 and m.checkpoints[1].x==351 and m.checkpoints[1].h==60,'right edge resize: w=78 x=351, got '
         ..m.checkpoints[1].w..' '..m.checkpoints[1].x)
  command('undo') assert(doc()==before)
  command('redo')
  drag(351,230,351,195) -- the bottom edge (world y 230) moves down to 195
  m=mdata()
  assert(m.checkpoints[1].h==95 and m.checkpoints[1].w==78,'bottom edge resize: h=95, got '..m.checkpoints[1].h)
  -- an edge cannot be dragged past the opposite one
  drag(390,242,260,242) m=mdata()
  assert(m.checkpoints[1].w==13,'the zone keeps a minimum size of one grid step, got '..m.checkpoints[1].w)
  command('mission clear')
end)
check('mission tool: the inspector edits the selected marker; Delete removes it',function()
  fresh_mission()
  command('mission pick enemy goomba') click(286,208)
  -- inspector rows (x = 400..624): picks at y 70/90/110, then the marker block from y 136
  click(500,196) assert(mdata().enemies[1].kind=='koopa','the kind row cycles the enemy kind')
  click(500,216) assert(mdata().enemies[1].wave==2,'the wave row steps the wave')
  click(500,156) -- the x row: type a value
  keys={['3']=true} on_tick() keys={['6']=true} on_tick() keys={['5']=true} on_tick() keys={ENTER=true} on_tick() keys={}
  assert(mdata().enemies[1].x==365,'typed x applies, got '..tostring(mdata().enemies[1].x))
  click(500,176) keys={['9']=true} on_tick() keys={ENTER=true} on_tick() keys={}
  assert(mdata().enemies[1].y==9)
  command('undo') command('undo') command('undo') command('undo')
  assert(mdata().enemies[1].kind=='goomba' and mdata().enemies[1].wave==1 and mdata().enemies[1].x==286,'each edit is one undo step')
  keys={DELETE=true} on_tick() keys={}
  assert(not doc():find('enemies=',1,true),'Delete removes the selected marker')
  command('undo') assert(doc():find('enemies=',1,true))
  -- a zone shows its size rows, and its delete row removes it
  command('mission pick checkpoint') click(325,260)
  click(500,196) -- with only the 'click places' row above: header 96, x 116, y 136, w 156, h 176, delete 196
  assert(not doc():find('checkpoints=',1,true),'the delete row removes the marker')
  command('mission clear')
end)
check('controller path: the Z-menu actions place, select, move and delete markers at the flight cursor',function()
  fresh_mission()
  command('mission pick enemy') command('run next marker kind') -- enemy -> checkpoint
  at(260,208) command('run place marker at cursor')
  local m=mdata() assert(#m.checkpoints==1 and m.checkpoints[1].x==260 and m.checkpoints[1].y==208,'placed a checkpoint at the cursor')
  command('mission pick start') at(299,260) command('run place marker at cursor') assert(mdata().start.x==299)
  at(263,210) command('run select nearest marker') assert(logged('Selected checkpoint 1'))
  at(351,234) command('run move marker to cursor') m=mdata()
  assert(m.checkpoints[1].x==351 and m.checkpoints[1].y==234 and m.start.x==299,'only the selected marker moved')
  command('run delete marker') assert(#(mdata().checkpoints or {})==0 and mdata().start,'deleted the selection')
  logs={} command('run delete marker') assert(logged('error:'),'nothing selected: refused')
  command('mission clear')
end)
check('the multi-level sample loads, validates and its rules and triggers are live',function()
  local text=MULTI_TEXT
  files['multi_level_mission.lua']=text command('on') logs={}
  command('load multi_level_mission.lua') assert(not logged('error:'),'sample refused to load')
  local data=assert(load(text,'s','t',{}))()
  assert(#data.parts==7 and #data.mission.enemies==4 and #data.mission.triggers==3 and #data.mission.waves==1)
  reset_run() logs={} command('play multi_level_mission.lua')
  assert(#spawned==1 and spawned[1].kind=='goomba' and logged('message Climb to the top'),'wave 1 and the start message')
  at(20,70) frames(2) assert(#spawned==3 and logged('trigger 1 wave'),'the ramp-top trigger spawns wave 2')
  at(131,60) frames(2) assert(#spawned==4 and spawned[4].kind=='redead','crossing x=130 starts wave 3')
  command('mission stop') command('on')
end)
check('mission tool: draws selection handles and survives a missing selection',function()
  fresh_mission()
  command('mission pick checkpoint') click(325,260)
  local boxes=0 local keep=gd.fill gd.fill=function(...) boxes=boxes+1 end
  on_draw() gd.fill=keep assert(boxes>8,'the selected zone draws its handles')
  command('undo') on_draw() -- the selection points at nothing now
  command('mission clear')
end)
check('the overlay draws trigger zones and rule hints',function()
  command('on') command('mission clear') command('clear')
  at(0,0) command('mission start') at(13,26) command('mission enemy goomba 2') command('mission wave 2 time 5')
  at(60,26) command('mission trigger message 20 20 hi')
  local drawn={}
  local keep=gd.kit.text
  gd.kit.text=function(_,_,t) drawn[#drawn+1]=t end
  on_draw()
  gd.kit.text=keep
  local labels=table.concat(drawn,'|')
  assert(labels:find('T1 message',1,true) and labels:find('@5s',1,true),labels)
  command('mission clear')
end)

print(('map_mission_test: %d pure passed, %d editor passed, %d failed'):format(PURE.passed,passed_n,PURE.failed+failed_n))
assert(PURE.failed+failed_n==0,'map_mission_test has failures')
]==]
_G.SAMPLE_TEXT=read('pc/scripts/examples/map_editor/samples/first_mission.lua')
_G.MULTI_TEXT=read('pc/scripts/examples/map_editor/samples/multi_level_mission.lua')
_G.PURE={passed=passed,failed=failed,}
local chunk,why=load(prefix..EXTRA..BODY,'=map_mission_editor','t')
assert(chunk,why)
chunk()
