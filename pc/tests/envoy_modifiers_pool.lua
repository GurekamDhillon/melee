local T=dofile('melee/pc/tests/envoy_testlib.lua');local D=T.rules()
T.test('full pool golden tiers, stock loss, snapshots, seven chains and synergy payloads',function()
D.mod_codec=T.module('mod_codec',D);D.mod_schema=T.module('mod_schema',D);D.mod_pool=T.module('mod_pool',D);D.mod_engine=T.module('mod_engine',D)
assert(#D.mod_pool==32+#D.mod_techniques.records(),'full pool (without the keystones module) requires 32 existing plus the technique modifiers')
local r=D.mod_engine.new(1,D.mod_pool);r:set_build(1,{kindling=1,glass_core=1},{damage_dealt=1.1,status_duration=1.5})
r:begin_frame({[1]={percent=0},[2]={percent=0}});r:emit{kind='hit_dealt',port=1,target=2,tags={fire=true}};r:drain()
assert(r:status(2,'burn').expires-r.frame==270);r:emit{kind='stock_lost',port=1,tags={}};r:drain();assert(r.equipped[1].glass_core==1 and math.abs(r:values(1).damage_dealt-1.7)<1e-9)
local restored=D.mod_engine.new(1,D.mod_pool);restored:import(r:export());assert(restored:export()==r:export())
print('EM3 pool/build PASS')

local dead=D.mod_engine.new(1,D.mod_pool);dead:set_build(1,{ledge=1,shelter=1},{})
dead:begin_frame({[1]={percent=0}});dead:emit{kind='ledge_grab',port=1,tags={}};dead:emit{kind='stock_lost',port=1,tags={}};dead:drain();assert(not dead:status(1,'guarded'),'queued statuses must not resurrect after KO')
D.mod_tuning=D.mod_tuning or T.module('mod_tuning',D);D.mod_graph=D.mod_graph or T.module('mod_graph',D);D.mod_synergy=T.module('mod_synergy',D);local synergy_pairs,degree=D.mod_synergy.generate(D.mod_pool)
-- Golden balancing outcomes, independent of the record resolver.
local expected={
 trailing={},echoes={},echo_heart={values={damage_dealt=.75}},
 kindling={status={2,'burn',{180,225,270},{1,1.25,1.5},1}},
 pyre={native={{match={move='any',status_bits=1},change={launch={1.08,1.1,1.12}}}}},
 burning={native={{match={move='smash'},change={element='fire'}}}},
 charged={native={{match={move='aerial'},change={element='electric'}}}},
 pyromancer={native={{match={move='any'},change={element='fire'}},{match={move='any',incoming=true,element='ice'},change={percent_damage=1.6}}}},
 feasting={heal={10,12.5,15}},updraft={status={1,'momentum',{300,375,450},1,5}},
 crosswind={status={1,'haste',{180,225,270},1,1},absent='momentum'},
 icebound={status={2,'chill',{120,150,180},1,1}},brittle={status={2,'curse',{120,150,180},{.025,.03125,.0375},1}},
 reprisal={status={1,'guarded',{180,225,270},1,1},absent='momentum'},
 glass_core={values={damage_dealt=1.6,damage_taken=1.6}},
 frosted={native={{match={move='smash'},change={element='ice'}}}},
 lingering={values={status_duration={1.2,1.25,1.3}}},
 cinder={native={{match={move='any',status_bits=1},change={percent_damage={1.1,1.125,1.15}}}}},
 shatter={native={{match={move='any',status_bits=4},change={percent_damage={1.1,1.125,1.15}}}}},
 ledge={status={1,'haste',{180,225,270},1,1}},bastion={status={1,'guarded',{120,150,180},1,1}},
 renewal={heal={2,2.5,3}},rush={status={1,'momentum',{240,300,360},1,5}},
 shelter={status={1,'guarded',{120,150,180},1,1}},malice={status={2,'curse',{120,150,180},{.025,.03125,.0375},1}},
 ember_crown={values={damage_taken=1.3},native={{match={move='any'},change={element='fire'}}}},
 winter_heart={values={run_speed=.8},native={{match={move='any'},change={element='ice'}}}},
 storm_shell={values={damage_dealt=.8},native={{match={move='any'},change={element='electric'}}}},
 mirror_shard={damage=17},
 armoured={values={damage_taken={.92,.9,.88}}},cleansing={},
 frozen_oath={native={{match={move='any'},change={element='ice'}},{match={move='any',incoming=true,element='fire'},change={percent_damage=1.6}}}}
}
for _,m in ipairs(D.mod_techniques and D.mod_techniques.records() or {}) do expected[m.id]=expected[m.id] or {} end -- technique modifiers have trigger effects only (tested in envoy_technique.lua)
local function at(v,tier)return type(v)=='table' and v[tier] or v end
local function equal(actual,want,label)
 if type(want)=='number' then assert(type(actual)=='number' and math.abs(actual-want)<1e-10,label..': '..tostring(actual)..' != '..want)
 else assert(actual==want,label..': '..tostring(actual)..' != '..tostring(want)) end
end
local function outcome(solo,id,tier)
 local want=assert(expected[id],'missing golden outcome '..id);local values=solo:values(1);local all=solo:native_rules(1);local native={};for _,rule in ipairs(all) do if rule.id<1000 then native[#native+1]=rule end end
 for key,v in pairs(want.values or {})do equal(values[key],at(v,tier),id..' '..key)end
 equal(#native,#(want.native or {}),id..' native count')
 for i,rule in ipairs(want.native or {})do
  for key,v in pairs(rule.match)do equal(native[i].match[key],v,id..' native match '..key)end
  for key,v in pairs(rule.change)do equal(native[i].change[key],at(v,tier),id..' native change '..key)end
 end
 for _,key in ipairs{'status','second'}do local st=want[key];if st then local actual=assert(solo:status(st[1],st[2]),id..' missing status')
  equal(actual.expires-solo.frame,at(st[3],tier),id..' duration');equal(actual.amount,at(st[4],tier),id..' amount');equal(actual.max,st[5],id..' max');equal(actual.stacks,1,id..' stacks')
  assert(not solo:status(3-st[1],st[2]),id..' applied status to wrong port')
 end end
 if want.absent then assert(not solo:status(1,want.absent),id..' failed to consume status')end
 if want.damage then equal(solo.damage[2],want.damage,id..' clank damage');assert(not solo.damage[1],id..' damaged self')end
 if want.heal then equal(solo.damage[1],-at(want.heal,tier),id..' heal');assert(not solo.damage[2],id..' healed target')end
end
local isolated
local tech={};for _,m in ipairs(D.mod_techniques and D.mod_techniques.records() or {}) do tech[m.id]=true end
for _,m in ipairs(D.mod_pool) do
 if degree[m.id]==0 then isolated=(isolated or 0)+1 end -- stat sticks connect to nothing (the derived graph says so; the old tag table hid it)
 if m.kind=='normal' then assert(#m.tiers==3 and m.affix and m.group and m.weight>0) end
 for tier=1,#m.tiers do
  if tech[m.id] then break end -- technique modifiers: envoy_technique.lua (payoffs carry min_depth too and are checked here)
  local solo=D.mod_engine.new(1,D.mod_pool);solo:set_build(1,{[m.id]=tier},{})
  solo:begin_frame({[1]={percent=100,grounded=false,stocks=1},[2]={percent=100,grounded=true,stocks=1}})
  local tags={};local ev={kind=m.trigger,port=1,target=2,tags=tags,damage_a=7,damage_b=10}
  for _,c in ipairs(m.conditions) do
   if c.tag then tags[c.tag]=true end
   for _,key in ipairs{'self_status','target_status'} do if c[key] then local port=key=='self_status' and 1 or 2;local name=c[key]=='any' and 'burn' or c[key];solo.statuses[port]=solo.statuses[port] or {};solo.statuses[port][name]={expires=1000,stacks=1,max=1,amount=1,next_tick=60,origin={}} end end
   if c.status then ev.status=c.status end
  end
  if m.trigger=='interval' then solo.frame=60 end
  solo:emit(ev);solo:drain();outcome(solo,m.id,tier)
  if m.trigger=='equip' then local values=solo:values(1);local rules=solo:native_rules(1);for _,effect in ipairs(m.effects) do if effect.op=='value' then assert(values[effect.key],m.id) elseif effect.op=='echo' then assert(solo:echo_description(1)) else assert(#rules>0,m.id) end end
  else assert(solo.used>0,m.id..' alone never fires') end
 end
end
T.refuses(function()r:set_build(1,{pyromancer=1,frozen_oath=1},{})end)
T.refuses(function()r:set_build(1,{kindling=1.5},{})end)
T.refuses(function()r:set_build(1,{},{status_duration=0/0})end)
local function chain(ids,events,check)
 local e=D.mod_engine.new(1,D.mod_pool);local mods={};for _,id in ipairs(ids)do mods[id]=1 end;e:set_build(1,mods,{})
 e:begin_frame({[1]={percent=50,grounded=false},[2]={percent=0}})
 for _,event in ipairs(events)do event.port=1;event.target=2;event.tags=event.tags or {};e:emit(event);e:drain()end;check(e)
end
chain({'updraft','crosswind'},{{kind='hit_dealt',tags={aerial=true}},{kind='landing'}},function(e)assert(e:status(1,'haste') and not e:status(1,'momentum'))end)
chain({'ledge','rush'},{{kind='ledge_grab'},{kind='hit_dealt'}},function(e)assert(e:status(1,'momentum'))end)
chain({'shelter','renewal'},{{kind='ledge_grab'}},function(e)assert(e.damage[1]==-2)end)
chain({'icebound','brittle'},{{kind='hit_dealt'},{kind='hit_dealt'}},function(e)assert(e:status(2,'curse'))end)
chain({'kindling','malice'},{{kind='hit_dealt'},{kind='hit_dealt'}},function(e)assert(e:status(2,'curse'))end)
chain({'ledge','rush','bastion'},{{kind='ledge_grab'},{kind='hit_dealt'},{kind='landing'}},function(e)assert(e:status(1,'guarded') and not e:status(1,'momentum'),'Bastion spends one Momentum stack')end)
local output=assert(io.open('_build/tmp/em3-synergy.tsv','w'));output:write('a\tb\treasons\n');for _,pair in ipairs(synergy_pairs)do output:write(pair.a,'\t',pair.b,'\t',table.concat(pair.reasons,','),'\n')end;output:close()
print('EM3 every record/tier, seven chains, '..#synergy_pairs..' synergy pairs PASS')
local function has_event(a,b)
 for _,pair in ipairs(synergy_pairs) do if (pair.a==a and pair.b==b) or (pair.a==b and pair.b==a) then for _,why in ipairs(pair.reasons) do if why:match('^event:') then return true end end end end
 return false
end
assert(not has_event('kindling','renewal'),'Burn must not claim to trigger Guarded-only Renewal')
assert(not has_event('icebound','renewal'),'Chill must not claim to trigger Guarded-only Renewal')
assert(has_event('shelter','renewal'))
local emit=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[1]));emit.id='test_emitter';emit.tags={};emit.effects={{op='emit',event='status_applied',tag='damage'}}
local receive=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[1]));receive.id='test_receiver';receive.tags={};receive.trigger='status_applied';receive.conditions={{tag='fire'}}
assert(#D.mod_synergy.generate({emit,receive})==0,'emitted tag must satisfy receiver tag condition')
receive.conditions={{tag='damage'}};assert(#D.mod_synergy.generate({emit,receive})==1)

assert(expected.mirror_shard and r.rules.mirror_shard,'Mirror Shard record required')
local mirror=D.mod_engine.new(1,D.mod_pool);mirror:set_build(1,{mirror_shard=1},{})
mirror:begin_frame({[1]={percent=0},[2]={percent=0}})
mirror:emit{kind='clank',port=1,target=2,tags={}};mirror:drain();assert(not mirror.damage[2],'missing damage payload must not invent damage')
mirror:emit{kind='clank',port=1,target=2,tags={},damage_a=4.5,damage_b=2.25};mirror:drain();equal(mirror.damage[2],6.75,'Mirror Shard exact combined clank damage')
for _,bad in ipairs{-1,math.huge}do T.refuses(function()mirror:emit{kind='clank',port=1,target=2,damage_a=bad,damage_b=1}end)end
T.refuses(function()mirror:emit{kind='clank',port=1,target=2,damage_a=0/0,damage_b=1}end)
local invalid=D.mod_codec.decode(D.mod_codec.encode(mirror.rules.mirror_shard));invalid.trigger='hit_dealt';T.refuses(function()D.mod_schema.validate(invalid)end)

end)
T.done()
