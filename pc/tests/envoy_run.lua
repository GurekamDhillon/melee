local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();D.save=T.module('save',D);D.fighter=T.module('fighter',D);D.campaign=T.module('campaign');local R=T.module('run',D)
local function fixture()
  local s={writes=0,plays={},p2={cpu=true,falls=0},p1={x=0,y=0,falls=0},active=true}
  local g={match=function() return {active=s.active,netplay=s.online} end,
    player=function(n) if n==1 then return s.p1 elseif n==2 then return s.p2 end end,log=function() end,
    fighter_mod=function() return true end,cpu_mode=function() return true end,set_damage=function() end,
    fighter_benched=function() return s.benched,{{present=true,refusal_code=0}} end,
    fighter_bench=function() s.benched=true;return true end,
    fighter_call=function() s.benched=false;return true end,stage_spawn=function() return 0,30 end}
  local mission={command=function(self,a)
    s.plays[#s.plays+1]=a;if s.reject then return nil,'load refused' end
    if s.stage then self.staging={name=a:match('play (.+)') or a:gsub('maze (%d+) (%d+)','maze_%1_%2')};return true end
    if s.delay then self.pending={arg=a};return true end
    self.current={doc={name=a:match('play (.+)') or a:gsub('maze (%d+) (%d+)','maze_%1_%2'),mission={goal={x=0,y=0,w=20,h=20},objective={time=180,lives=3}}},run={state={}}};return true
  end,stop=function(self) self.current=nil end}
  local r=R.new(g,mission,D.save.new_profile(),function(p)
    if p.last_settled>0 then s.writes=s.writes+1 end;if s.disk then return false,'disk full' end;s.saved=D.save.encode(p);return true
  end)
  return s,r,mission
end
local function boss(r,m)
  while r.index<#r.levels do
    m.current.run.state.result={status='complete'};r:frame();assert(r.interlude);assert(r:continue());r:frame()
  end
end
T.test('level then boss via runtime commands and falls gate',function()
  local s,r,m=fixture();local armed,done={},{}
  r.g.deadline=function(n) armed[n]=true;return true end
  r.g.deadline_done=function(n) done[n]=true;return true end
  assert(r:start());assert(s.plays[1]=='play path')
  boss(r,m);assert(s.plays[4]=='play boss')
  m.current.run.state.result={status='complete'};s.p1.x=0;s.p1.y=0;r:frame();assert(s.writes==0)
  s.p2.falls=1;r:frame();assert(s.writes==1 and r.profile.last_result=='win')
  r:finish('quit');r:frame();assert(s.writes==1)
  assert(armed['envoy/boss-call'] and done['envoy/boss-call'])
  assert(armed['envoy/settle'] and done['envoy/settle'])
end)
T.test('fail and quit retain picked up stats exactly once',function()
  for _,reason in ipairs({'fail','quit'}) do
    local s,r=fixture();assert(r:start());D.companion.feed(r.companion,'red',100)
    assert(r:finish(reason));assert(r.results.before.power.points==0 and r.results.after.power.points==100);assert(s.writes==1 and D.save.decode(s.saved).companions[1].stats.power.points==100)
    assert(r:finish(reason));assert(s.writes==1)
  end
end)
T.test('settlement failure locks new run and retries same action',function()
  local s,r=fixture();r:start();D.companion.feed(r.companion,'green',100);s.disk=true
  assert(not r:finish('fail'));assert(not r:start());assert(r.pending=='fail' and r.companion.stats.speed.points==100)
  s.disk=false;assert(r:finish('quit'));assert(r.profile.last_result=='fail' and r.profile.next_run==2)
end)
T.test('transition refusal and match end settle as failure',function()
  local s,r,m=fixture();r:start();m.current.run.state.result={status='complete'};r:frame();s.reject=true;r:continue()
  assert(s.writes==1 and r.profile.last_result=='fail')
  s,r=fixture();r:start();s.active=false;r:frame();assert(s.writes==1)
end)
T.test('early boss exit does not freeze failure checks or count while away',function()
  local s,r,m=fixture();r:start();boss(r,m)
  m.current.run.state.result={status='complete'};s.p1.x=100;s.p1.y=0;s.p2.falls=1;r:frame()
  assert(s.writes==0);s.p1.falls=3;r:frame();assert(s.writes==1 and r.profile.last_result=='fail')
  s,r,m=fixture();r:start();boss(r,m)
  m.current.run.state.result={status='complete'};s.p2=nil
  for _=1,10800 do r:frame() end
  assert(s.writes==1 and r.profile.last_result=='fail')
end)
T.test('missing CPU online and repeated starts refused',function()
  local s,r=fixture();s.p2=nil;assert(not r:start());assert(s.writes==0)
  s,r=fixture();s.online=true;assert(not r:start())
  s,r=fixture();assert(r:start());assert(not r:start());assert(#s.plays==1)
end)
T.test('queued mission start and transition are adopted only when installed',function()
  local s,r,m=fixture();s.delay=true;assert(r:start());r:frame();assert(r.active and r.index==0)
  assert(m.pending);s.delay=false;m.pending=nil;m:command(r.levels[1].command);r:frame();assert(r.index==1)
  m.current.run.state.result={status='complete'};r:frame();s.delay=true;r:continue()
  assert(r.index==1 and r.request and s.benched)
  s.delay=false;m.pending=nil;m:command(r.levels[2].command);r:frame();assert(r.index==2 and s.benched)
end)
T.test('queued mission timeout settles without a second play request',function()
  local s,r,m=fixture();s.delay=true;r:start();r:frame();m.pending=nil;r:frame()
  assert(s.writes==0 and r.profile.last_result~='fail' and #s.plays==1 and r.profile.records.runs==0 and r.profile.companions[1].age==0)
end)
T.test('staged install remains active until commit and forbids new runs during cleanup',function()
  local s,r,m=fixture();s.stage=true;assert(r:start());assert(r.active and r.index==0 and m.staging)
  for _=1,20 do r:frame() end;assert(r.active and r.index==0)
  s.stage=false;m.staging=nil;m:command(r.levels[1].command);r:frame();assert(r.index==1)
  m.current.run.state.result={status='complete'};r:frame();s.stage=true;r:continue()
  for _=1,20 do r:frame() end;assert(r.active and r.index==1 and r.request)
  s.stage=false;m.staging=nil;m:command(r.levels[2].command);r:frame();assert(r.index==2 and not r.fighter.boss)
  r:finish('quit');m.staging={phase='rollback'};assert(not r:start())
end)
T.test('wait deadlines arm once, finish on progress, and survive settlement retries',function()
  local s,r,m=fixture();local armed,done={},{}
  r.g.deadline=function(n,f) armed[n]=(armed[n] or 0)+1;assert(f==600);return true end
  r.g.deadline_done=function(n) done[n]=(done[n] or 0)+1;return true end
  s.stage=true;assert(r:start());for _=1,8 do r:frame() end
  assert(armed['envoy/staging']==1 and done['envoy/bench-readiness']==1)
  s.stage=false;m.staging=nil;m:command(r.levels[1].command);r:frame()
  s.disk=true;assert(not r:finish('quit'));assert(not r:finish('quit'))
  assert(armed['envoy/settle']==1 and not done['envoy/settle'])
  s.disk=false;assert(r:finish('quit'));assert(done['envoy/settle']==1 and r.index==0)
  r:wait('optional',true);r:wait('optional',true);r:clear_waits();assert(armed['envoy/optional']==1 and done['envoy/optional']==1)
  r.g.deadline=function() error('optional API refusal') end;r.g.deadline_done=nil
  r:wait('absent',true);r:clear_waits()
end)
T.test('interlude pauses elapsed and sends one command on continue',function()
  local s,r,m=fixture();r:start();m.current.run.state.result={status='complete'};r:frame()
  local elapsed=r.elapsed;for _=1,100 do r:frame() end
  assert(r.interlude and r.elapsed==elapsed and #s.plays==1)
  assert(r:continue());assert(#s.plays==2 and not r.interlude);assert(not r:continue())
end)
T.test('accepted start persists seed and can resume without aging twice',function()
  local s,r=fixture();local writes=0
  r.commit=function(p) writes=writes+1;s.saved=D.save.encode(p);return true end
  assert(r:start(42));assert(writes==1 and r.profile.pending_seed==42)
  local age=r.companion.age;local crash=s.saved
  local s2,r2=fixture();r2.profile=D.save.decode(crash);local resumedWrites=0
  r2.commit=function() resumedWrites=resumedWrites+1;return true end
  assert(r2:start(999));assert(r2.seed==42 and r2.companion.age==age and resumedWrites==0)
  r:finish('quit');assert(writes==2 and r.profile.last_seed==42)
end)
T.test('start disk refusal accepts no age and invalid seed does not reserve CPU',function()
  local s,r=fixture();s.disk=true;assert(not r:start(42));assert(not r.active and not s.benched and r.profile.pending_seed==nil)
  s.disk=false;assert(not r:start(-1));assert(not s.benched)
end)
T.test('immediate refused first room writes no age or record and unbenches',function()
  local s,r,m=fixture();s.p1.percent=39;s.p2.percent=116
  r.g.set_damage=function(port,v) (port==1 and s.p1 or s.p2).percent=v end
  s.reject=true;local before=D.save.encode(r.profile);local writes=0
  r.commit=function() writes=writes+1;return true end
  assert(not r:start(42));assert(writes==0 and D.save.encode(r.profile)==before)
  assert(not r.active and not s.benched and not r.fighter.started and not r.pending)
  assert(s.p1.percent==39 and s.p2.percent==116)
  s.reject=false;assert(r:start(42));assert(r.profile.records.runs==1)
end)
T.test('queued first room has no durable start until installation',function()
  local s,r,m=fixture();s.stage=true;local writes=0
  r.commit=function() writes=writes+1;return true end
  assert(r:start(42));assert(writes==0 and r.profile.records.runs==0)
  m.staging=nil;r:frame();assert(not r.active and writes==0 and not s.benched)
  assert(r.profile.companions[1].age==0 and r.last_start_error)
end)
T.test('first room timeout restores damage after asynchronous rollback',function()
 local s,r,m=fixture();s.stage=true;s.p1.percent=39;s.p2.percent=116
 r.g.set_damage=function(port,v) (port==1 and s.p1 or s.p2).percent=v end
 m.stop=function(self) self.current=nil;self.staging={phase='rollback'} end
 assert(r:start(42));r.request.frames=600;r:frame();assert(not r.active)
 s.p1.percent=0;m.staging=nil;r:frame()
 assert(s.p1.percent==39 and s.p2.percent==116 and r.profile.records.runs==0)
end)
T.test('partial damage reset failure restores earlier reset ports',function()
 local s,r=fixture();s.p1.percent=39;s.p2.percent=116
 r.g.set_damage=function(port,v) if port==2 and v==0 then return false,'refused' end;(port==1 and s.p1 or s.p2).percent=v end
 assert(not r:start(42));assert(s.p1.percent==39 and s.p2.percent==116 and not s.benched)
end)
T.done()
