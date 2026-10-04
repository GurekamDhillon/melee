local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();local E=T.module('mission_events',D);D.drives=T.module('drives',D)
T.test('observer forwards real machine signals without changing step results',function()
  local actions={{type='spawn'}};local events={{type='defeat',handle=1,kind='koopa'},{type='vanish',handle=2,kind='goomba'}}
  local machine={step=function(st,obs) assert(st==3 and obs==4);return actions,events end}
  local original=machine.step;local got={}
  local detach=E.observe(machine,function(e) got[#got+1]=e end)
  local a,b=machine.step(3,4);assert(a==actions and b==events and got[1]==events[1] and got[2]==events[2])
  detach();assert(machine.step==original);detach();assert(machine.step==original)
end)
T.test('mission-only Koopa defeat drops once; lost and foreign handles do not',function()
  local v=D.drives.new(function() return .1 end);local c=D.companion.new()
  v:track(1,{kind='koopa',x=0,y=0});v:track(2,{kind='goomba',x=0,y=0})
  local machine={step=function() return {},{{type='defeat',handle=1,kind='koopa'},{type='vanish',handle=2,kind='goomba'},{type='defeat',handle=8,kind='redead'}} end}
  E.observe(machine,function(e) if e.type=='defeat' then v:defeat(e,c) end end)
  machine.step();machine.step();assert(#v.pickups==1 and v.pickups[1].colour=='blue')
end)
T.test('observer failures cannot alter mission state or returned actions',function()
  local machine={step=function() return {5},{{type='defeat'}} end};local logs={}
  E.observe(machine,function() error('refused listener') end,function(s) logs[#logs+1]=s end)
  local actions,events=machine.step();assert(actions[1]==5 and events[1].type=='defeat' and #logs==1)
end)
T.test('runtime adapter normalizes void damage setter without hiding refusal',function()
  local count=0;local g={set_damage=function(port,value) assert(port==1 and value==0);count=count+1 end,player=function() return 5 end}
  local adapter=E.engine(g);assert(adapter.set_damage(1,0)==true and count==1 and adapter.player()==5)
  g.set_damage=function() return nil,'refused' end
  local ok,why=adapter.set_damage(1,0);assert(ok==nil and why=='refused')
  g.set_damage=function() return false end;assert(adapter.set_damage(1,0)==false)
  g.set_damage=function() error('offline') end;assert(not pcall(adapter.set_damage,1,0))
  assert(E.engine({}).set_damage==nil)
end)
T.done()
