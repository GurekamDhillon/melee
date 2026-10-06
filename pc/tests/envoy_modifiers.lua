local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();D.mod_codec=T.module('mod_codec',D)
T.test('modifier schema rejects names and specific moves',function()
 D.mod_schema=T.module('mod_schema',D);D.mod_pool=T.module('mod_pool',D)
 for _,m in ipairs(D.mod_pool) do assert(D.mod_schema.validate(m)) end
 local m={id='bad',tags={'fox_nair'},tiers={{}},trigger='hit_dealt',conditions={},effects={},stacking={max=1},text='Bad',visual={look='burn'}}
 T.refuses(function() D.mod_schema.validate(m) end)
 m.tags={'fire'};m.conditions={{modifier='kindling'}};T.refuses(function() D.mod_schema.validate(m) end)
 m.conditions={{move='AttackAirN'}};T.refuses(function() D.mod_schema.validate(m) end)
end)
T.test('deterministic statuses and intended tagged chains',function()
 D.mod_engine=T.module('mod_engine',D)
 local function trial()
  local r=D.mod_engine.new(41,D.mod_pool);r:equip(1,'kindling');r:equip(1,'pyre');r:equip(1,'feasting')
  r:begin_frame({[1]={percent=70,stocks=3,grounded=true},[2]={percent=0,stocks=3,grounded=true}})
  r:emit{kind='hit_dealt',port=1,target=2,tags={fire=true},damage=8};r:drain()
  assert(r:status(2,'burn'));local rules=r:native_rules(1);assert(rules[1].change.launch==1.08)
  local origin,used=r:native_origin({rules[1].id},2)
  r:emit{kind='hit_dealt',port=1,target=2,tags={normal=true},damage=8,origin=origin,native_trace=true,native_statuses=used};r:drain()
  assert(not r:status(2,'curse'));r:emit{kind='ko_dealt',port=1,target=2,tags={}};r:drain()
  assert(r.damage[1]==-10 and #r.trace>=3)
  return r:export()
 end
 assert(trial()==trial())
end)
T.test('two other chains plus unique and keystone use only supported effects',function()
 local r=D.mod_engine.new(6,D.mod_pool)
 r:equip(1,'updraft');r:equip(1,'crosswind');r:equip(1,'pyromancer');r:equip(1,'glass_core')
 r:begin_frame({[1]={percent=0,grounded=false},[2]={percent=0,grounded=true}})
 r:emit{kind='hit_dealt',port=1,target=2,tags={aerial=true}};r:drain();assert(r:status(1,'momentum'))
 r:emit{kind='landing',port=1,tags={grounded=true}};r:drain()
 assert(r:status(1,'haste') and not r:status(1,'momentum'),'landing spends the Momentum stack and gives Haste')
 local v=r:values(1);assert(v.damage_dealt==1.6 and v.damage_taken>1)
 r:equip(2,'icebound');r:equip(2,'brittle');r:emit{kind='hit_dealt',port=2,target=1,tags={ice=true}};r:drain()
 r:emit{kind='hit_dealt',port=2,target=1,tags={normal=true}};r:drain();assert(r:status(1,'curse'))
end)
T.test('budget depth expiry snapshots and cleanup are bounded',function()
 local r=D.mod_engine.new(7,D.mod_pool,{budget=3,depth=2})
 r:equip(1,'kindling');r:begin_frame({[1]={percent=0},[2]={percent=0}})
 for _=1,20 do r:emit{kind='hit_dealt',port=1,target=2,tags={fire=true}} end;r:drain()
 assert(r.used<=3 and r.dropped>0 and #r.queue==0)
 local text=r:export();local restored=D.mod_engine.new(7,D.mod_pool);restored:import(text);assert(restored:export()==text)
 restored:clear(2);assert(not restored:status(2,'burn'));restored:clear();assert(next(restored.equipped)==nil and next(restored.statuses)==nil)
end)
local function fresh(id)
 local r=D.mod_engine.new(11,D.mod_pool);if id then r:equip(1,id) end
 r:begin_frame({[1]={percent=50,stocks=3,grounded=true},[2]={percent=0,stocks=3,grounded=true}});r:drain();return r
end
local function hit(r,tags) r:emit{kind='hit_dealt',port=1,target=2,tags=tags or {}};r:drain() end
for _,id in ipairs({'kindling','pyre','feasting','updraft','crosswind','icebound','brittle','reprisal','glass_core'}) do
 T.test('standalone rule '..id,function()
  local r=fresh(id)
  if id=='kindling' then hit(r,{});assert(r:status(2,'burn'),'Burn applies on any hit')
  elseif id=='icebound' then hit(r,{});assert(r:status(2,'chill'),'Chill applies on any hit')
  elseif id=='updraft' then hit(r,{aerial=true});assert(r:status(1,'momentum'))
  elseif id=='glass_core' then local v=r:values(1);assert(v.damage_dealt==1.6 and v.damage_taken==1.6)
  elseif id=='reprisal' then r:emit{kind='perfect_shield',port=1};r:drain();assert(r:status(1,'guarded') and not r:status(1,'momentum'),'Reprisal is one effect: Guarded')
  else
   local status=({pyre='burn',feasting='chill',crosswind='momentum',brittle='chill'})[id]
   local port=(id=='crosswind') and 1 or 2
   r.statuses[port]={[status]={stacks=1,max=1,expires=181,next_tick=61,amount=1,origin={}}}
   if id=='crosswind' then r:emit{kind='landing',port=1};r:drain();assert(r:status(1,'haste') and not r:status(1,'momentum'))
   elseif id=='feasting' then r:emit{kind='ko_dealt',port=1,target=2};r:drain();assert(r.damage[1]==-10)
   elseif id=='pyre' then local rules=r:native_rules(1);assert(rules[1].match.status_bits==1 and rules[1].change.launch==1.08);hit(r);assert(not r:status(2,'curse'))
   else hit(r);assert(r:status(2,'curse')) end
  end
  assert(#D.mod_schema.tooltip(r.rules[id])>10)
 end)
end
T.test('strict schema and codec reject hidden fields malformed bounds and payloads',function()
 local function clone(v) return D.mod_codec.decode(D.mod_codec.encode(v)) end
 local base=clone(D.mod_pool[1])
 for _,change in ipairs({function(m)m.tags.hidden='fire'end,function(m)m.effects[1].duration=0 end,
  function(m)m.effects[1].max=1.5 end,function(m)m.tiers[1].duration=4000 end,function(m)m.visual.strength=0/0 end,
  function(m)m.effects[1]={op='value',key='weight',value=2} end,function(m)m.text='{undefined}'end}) do
  local m=clone(base);change(m);T.refuses(function()D.mod_schema.validate(m)end)
 end
 for _,text in ipairs({'s999:short','{1:s1:x','d3:nan','{2:s1:xd1:1s1:xd1:2','tTAIL'}) do T.refuses(function()D.mod_codec.decode(text)end) end
 local cyc={};cyc[1]=cyc;T.refuses(function()D.mod_codec.encode(cyc)end)
 T.refuses(function()D.mod_engine.new(0,D.mod_pool)end)
end)
T.test('status duration refresh stacks exact expiration and KO cleanup',function()
 local r=fresh('updraft');for _=1,9 do hit(r,{aerial=true}) end;assert(r:status(1,'momentum').stacks==5)
 local players=r.players;for _=1,299 do r:begin_frame(players);r:drain() end
 assert(r:status(1,'momentum'));hit(r,{aerial=true});assert(r:status(1,'momentum').expires==600)
 for _=1,300 do r:begin_frame(players);r:drain() end;assert(not r:status(1,'momentum'))
 r:equip(1,'glass_core');r:emit{kind='stock_lost',port=1};r:drain();assert(r:values(1).damage_dealt==1.6 and r.equipped[1].glass_core==1 and not r.statuses[1] and not r.recent[1])
 r=fresh('kindling');hit(r,{fire=true});players=r.players;local total=0
 for _=1,180 do r:begin_frame(players);r:drain();total=total+(r.damage[2] or 0) end
 assert(total==3 and not r:status(2,'burn'),'Kindling burns 1 damage a second for 3 seconds')
end)
T.test('synthetic event cycle obeys depth and interval is logic-frame deterministic',function()
 local m=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[1]));m.families={'conversion'};m.id='cycle';m.trigger='interval';m.interval=2;m.conditions={};m.effects={{op='emit',event='interval',tag='damage'}}
 local r=D.mod_engine.new(17,{m},{budget=128,depth=3});r:equip(1,'cycle');r:begin_frame({[1]={percent=0}});r:drain();assert(r.used==0)
 r:begin_frame(r.players);r:drain();assert(r.used==3 and r.dropped==1)
 local saved=r:export();local rr=D.mod_engine.new(1,{m});rr:import(saved)
 for _=1,20 do local players={[1]={percent=0}};r:begin_frame(players);rr:begin_frame(players);r:drain();rr:drain();assert(r:export()==rr:export()) end
end)
T.test('contact conditions use captured context rather than deferred fighter state',function()
 local m=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[1]));m.conditions={{grounded=false},{self_damage_above=40}}
 local r=D.mod_engine.new(1,{m});r:equip(1,'kindling');r:begin_frame({[1]={percent=0,grounded=true},[2]={percent=0}})
 r:emit{kind='hit_dealt',port=1,target=2,tags={},self_context={percent=50,grounded=false}};r:drain();assert(r:status(2,'burn'))
end)
T.test('budget exhaustion never prevents stock lifetime cleanup',function()
 local r=D.mod_engine.new(2,D.mod_pool,{budget=1});r:equip(1,'kindling');r:equip(2,'glass_core')
 r:begin_frame({[1]={percent=0},[2]={percent=0}});r:emit{kind='hit_dealt',port=1,target=2,tags={fire=true}};r:emit{kind='stock_lost',port=2};r:drain()
 assert(r.equipped[2].glass_core==1 and not r:status(2,'burn') and r:values(2).damage_dealt==1.6)
end)
T.test('unknown contact context never means zero damage or grounded',function()
 local m=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[1]));m.conditions={{self_damage_below=40}}
 local r=D.mod_engine.new(1,{m});r:equip(1,'kindling');r:begin_frame({[1]={percent=0,grounded=true},[2]={percent=0}})
 r:emit{kind='hit_dealt',port=1,target=2,self_context={},tags={}};r:drain();assert(not r:status(2,'burn'))
 m.conditions={{airborne=false}};r:emit{kind='hit_dealt',port=1,target=2,self_context={},tags={}};r:drain();assert(not r:status(2,'burn'))
end)
T.done()
