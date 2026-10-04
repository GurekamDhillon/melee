return function(test,fixture)
  local function scene()
    local s,r,d=fixture();assert(r:command('play test'));return s,r,d
  end
  test('clear pulses 28 on 12 off and bounds an immortal target',function()
    local s,r=scene();local _,e=next(s.enemies);e.kind='redead';e.state=7;e.vulnerable=false
    local opened,closed=0,0
    r.g.deadline=function(_,n)opened=opened+1;assert(n>1500);return true end
    r.g.deadline_done=function()closed=closed+1;return true end
    r.g.fly_state=function()return {attacking=s.attacking}end
    r.g.fly_clear=function()s.target=nil;s.attacking=false;s.released=true end
    assert(r:command('clear'));local off,on=0,0
    for i=1,40 do r:frame();if s.attacking then on=on+1 else off=off+1 end end
    assert(on==28 and off==12,'pulse '..on..'/'..off)
    for _=1,1500 do r:frame()end
    assert(not r.current.clear and opened==1 and closed==1 and s.released and not s.target)
    assert(table.concat(s.logs,';'):find('clear: redead not defeated after 1500 frames (state 7, vulnerable=false)',1,true))
  end)
  test('pulsed target can finish during a rest window and release cursor immediately',function()
    local s,r=scene();local h,e=next(s.enemies);local opened=0;local closed=0
    r.g.deadline=function()opened=opened+1;return true end
    r.g.deadline_done=function()closed=closed+1;return true end
    r.g.fly_clear=function()s.target=nil;s.attacking=false end
    r:command('clear');local rests=0
    for _=1,100 do
      r:frame()
      if not s.attacking then rests=rests+1 end
      if rests==12 then e.status='defeated' end
      if not r.current.clear then break end
    end
    assert(not r.current.clear and r.current.run.state.defeated==1 and not s.target)
    assert(opened==1 and closed==1)
    assert(r.g.fly_target(1,10,10),'first console-equivalent target failed')
  end)
  test('clear escalation shifts the cursor and target deadline cannot be restarted',function()
    local s,r=scene();local h,e=next(s.enemies);r:command('clear')
    for _=1,650 do r:frame()end
    assert(s.target and math.abs(s.target.x-e.x)==8 and r.current.clear.age>=650)
    local age=r.current.clear.age;s.p.action=12
    for _=1,10 do r:frame()end;assert(r.current.clear.age==age+10)
  end)
  test('clear deadline and cursor close on replacement and stop',function()
    local s,r=scene();local pending=0;local releases=0
    r.g.deadline=function()pending=pending+1;return true end
    r.g.deadline_done=function()pending=pending-1;return true end
    r.g.fly_clear=function()releases=releases+1;s.target=nil;s.attacking=false end
    r:command('clear');r:command('tour');assert(pending==0 and releases>=1)
    r:command('clear');r:command('stop');assert(pending==0 and releases>=2)
  end)
  test('P1 damaged escaped enemies count as defeats but untouched escapes do not',function()
    local s,r=scene();local h,e=next(s.enemies);e.x=199;e.received=1;e.last_attacker=1;r:frame()
    e.last_attacker=2;r:frame();e.status='removed';r:frame()
    assert(r.current.run.state.defeated==1 and r.current.run.state.vanished==0)
    local _,other=next(s.enemies,h);assert(other);other.x=199;other.received=1;other.last_attacker=2;r:frame()
    other.status='removed';r:frame();assert(r.current.run.state.vanished==1)
  end)
  test('handover samples live falls and holds P1 until host isolation',function()
    local s,r=fixture();s.manual_stage=true;s.p.falls=3
    local hide=r.g.stage_hide
    r.g.stage_hide=function(v)if v then assert(s.holding and not s.reserve_calls,'P1 unsafe during host isolation');s.p.falls=4 end;return hide(v)end
    assert(r:command('play test'));for _=1,40 do r:frame()end
    assert(r.current and r.current.run.state.deaths==0 and r.current.run.state.falls==4)
    assert(s.reserve_calls==1 and not table.concat(s.logs,';'):find('mission: death',1,true))
    s.p.falls=5;r:frame();assert(r.current.run.state.deaths==1,'real post-load death ignored')
  end)
end
