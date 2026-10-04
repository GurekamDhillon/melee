local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();local F=T.module('fighter',D)
local function fixture()
  local s={mods={},calls={},bench=false,mode={},p={{cpu=false,falls=0,percent=39},{cpu=true,falls=0,percent=116},{cpu=true,falls=0}},safe=0}
  local g={player=function(n) return s.p[n] end,match=function() return {active=true} end,log=function() end,
    fighter_mod=function(n,v) s.mods[#s.mods+1]={port=n,value=v};return true end,
    fighter_benched=function() return s.bench,{{present=true,refusal_code=s.safe},{present=false}} end,
    fighter_bench=function() if s.safe~=0 then return false,'unsafe action' end;s.bench=true;return true end,
    fighter_call=function(n,x,y,o) if s.callfail then return false,'respawning' end;s.bench=false;s.calls[#s.calls+1]={x=x,y=y};return true end,
    cpu_mode=function(n,m) s.mode[n]=m;return true end,stage_spawn=function() return 0,30 end,
    set_damage=function(n,v) if s.damagefail then error('damage refused') end;if s.damagenil then return nil,'damage refused' end;s.p[n].percent=v end}
  return s,F.new(g),D.companion.new()
end
T.test('native ratios applied start and changed levels only; cap includes knockback',function()
  local s,f,c=fixture();assert(f:begin(c));assert(s.bench and s.mode[2]=='stand' and s.mode[3]=='stand')
  local e=s.mods[1].value;assert(e.damage_dealt==1 and e.shield_max==1 and e.run_speed==1 and e.air_speed==1)
  D.companion.feed(c,'red',100);assert(f:tick(c));assert(math.abs(s.mods[2].value.damage_dealt-D.companion.effects(c).damage_dealt)<1e-9 and s.mods[2].value.damage_dealt>=1.04)
  local n=#s.mods;f:tick(c);assert(#s.mods==n)
  D.companion.feed(c,'red',100000);f:tick(c);assert(s.mods[#s.mods].value.damage_dealt<=1.1)
end)
T.test('activate calls existing reserve grounded and only active boss fights',function()
  local s,f,c=fixture();f:begin(c);assert(f:activate(60,0));assert(not s.bench)
  f:tick(c);assert(s.mode[2]=='fight' and s.mode[3]=='stand' and s.calls[1].y==0)
  f:clear();assert(s.mods[#s.mods].value==nil and s.mode[2]=='stand')
  local n=#s.mods;f:clear();assert(#s.mods==n)
end)
T.test('cleanup releases benched reserve and clears exactly once',function()
  local s,f,c=fixture();f:begin(c);f:clear();assert(not s.bench and #s.calls==1)
  assert(s.mods[#s.mods].value==nil);f:clear();assert(#s.calls==1)
end)
T.test('unsafe bench retries bounded without starting boss or borrowing another owner',function()
  local s,f,c=fixture();s.safe=7;local ok,why=f:begin(c);assert(ok and why=='waiting')
  assert(not f.ready);s.safe=0;for _=1,6 do f:tick(c) end;assert(f.ready and s.bench)
  s,f,c=fixture();s.bench=true;assert(not f:begin(c));assert(#s.mods==0)
end)
T.test('start and boss call reset both percent; debug reserve damage cannot carry',function()
  local s,f,c=fixture();assert(f:begin(c));assert(s.p[1].percent==0 and s.p[2].percent==0)
  s.p[1].percent=24;s.p[2].percent=56;assert(f:activate(60,0))
  assert(s.p[1].percent==0 and s.p[2].percent==0 and #s.calls==1)
  f:clear();s.p[1].percent=39;s.p[2].percent=116;assert(f:begin(c))
  assert(s.p[1].percent==0 and s.p[2].percent==0)
end)
T.test('damage reset refusal cannot activate a damaged boss',function()
  local s,f,c=fixture();assert(f:begin(c));s.damagefail=true
  assert(not f:activate(60,0));assert(s.bench and not f.boss and #s.calls==0)
  s.damagefail=nil;s.damagenil=true;assert(not f:activate(60,0));assert(s.bench and #s.calls==0)
end)
T.done()
