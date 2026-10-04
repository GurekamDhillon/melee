local T=dofile('melee/pc/tests/envoy_testlib.lua');local D=T.rules()
D.mod_codec=T.module('mod_codec',D);D.mod_schema=T.module('mod_schema',D)
T.test('budget authority exists and additive families cap once',function()
 local f=io.open(T.root..'mod_budget.lua');assert(f,'missing additive budget authority');f:close()
 D.mod_budget=T.module('mod_budget',D);D.mod_pool=T.module('mod_pool',D)
 local b,s=D.mod_budget.build(D.mod_pool,{glass_core=1,storm_shell=1},{damage_dealt=1.16},{})
 assert(math.abs(b.damage_dealt.value-1.56)<1e-9 and s>=1)
 local v=D.mod_budget.values(D.mod_pool,{}, {run_speed=1.32,air_speed=1.32},{haste={amount=1}})
 assert(math.abs(v.run_speed-1.52)<1e-9 and math.abs(v.air_speed-1.52)<1e-9)
 local bad={D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[1]))};bad[1].families={'unknown'}
 T.refuses(function()D.mod_budget.validate_pool(bad,{})end)
 bad={D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[12]))};bad[1].effects[1].value=65
 T.refuses(function()D.mod_budget.validate_pool(bad,{})end)
end)
T.test('four actual drives reach representative Rare plus Glass native budget',function()
 D.mod_engine=T.module('mod_engine',D);D.drive_loot=T.module('drive_loot',D);D.drive_bag=T.module('drive_bag',D)
 local loot=D.drive_loot.new(D.mod_pool);local bag=D.drive_bag.new(loot)
 local function rare(seed,colour,ids)
  local r={seed=seed,depth=10,colour=colour,rarity='rare',affixes={}}
  for _,id in ipairs(ids) do r.affixes[#r.affixes+1]={id=id,tier=3} end
  assert(loot:validate(r));return r
 end
 local records={rare(401,'red',{'heavy','pyre','kindling','malice'}),rare(402,'green',{'cinder','lingering','feasting','renewal'}),rare(403,'blue',{'featherweight','shatter','updraft','icebound'}),{seed=404,depth=10,colour='white',rarity='unique',unique='glass_core',affixes={{id='glass_core',tier=1}}}}
 for slot,r in ipairs(records) do assert(bag:give(r));assert(bag:equip(1,slot)) end
 local mods,implicit=bag:derive();local e=D.mod_engine.new(1,D.mod_pool);e:set_build(1,mods,implicit)
 e.statuses[2]={};for name,amount in pairs({burn=4.5,curse=.0375}) do e.statuses[2][name]={amount=amount,expires=1000,next_tick=60,stacks=1,max=1,origin={}} end
 local rules=e:native_rules(1);local launch,damage=1,1
 for _,r in ipairs(rules) do if not r.match.incoming then
  if not r.match.status_bits or r.match.status_bits==1 then launch=launch+(r.change.launch or 1)-1;damage=damage+(r.change.percent_damage or 1)-1 end
 end end
 assert(math.abs(launch-1.195)<1e-9);assert(math.min(1.6,damage)==1.6);assert(#rules<=8)
 local target=e:native_rules(2);assert(#target==1 and target[1].match.incoming and math.abs(target[1].change.launch-1.0375)<1e-9)
 assert(math.abs(launch*target[1].change.launch-1.2398125)<1e-9)
 local b=e:family_budget(1);assert(b.damage_dealt.raw>.6 and b.damage_dealt.value==1+b.damage_dealt.raw)
 local before=e:export();e:emit{kind='stock_lost',port=1};e:drain();assert(e.equipped[1].glass_core and e:values(1).damage_dealt>1.6)
 local restored=D.mod_engine.new(2,D.mod_pool);restored:import(e:export());assert(restored:export()==e:export())
 local audit=D.mod_budget.worst_case(D.mod_pool,loot.implicits)
 for f,c in pairs(D.mod_budget.caps) do assert(audit[f].min>=1+c[1]-1e-9 and audit[f].max<=1+c[2]+1e-9) end
end)
T.test('two Red drives add eight percent twice and costs offset before cap',function()
 local loot=D.drive_loot.new(D.mod_pool);local bag=D.drive_bag.new(loot)
 for slot=1,2 do assert(bag:give{seed=slot,depth=0,colour='red',rarity='common',affixes={}});assert(bag:equip(1,slot)) end
 local mods,imp=bag:derive();assert(math.abs(imp.damage_dealt-1.16)<1e-9)
 local b=D.mod_budget.build(D.mod_pool,{glass_core=1,storm_shell=1},imp,{})
 assert(math.abs(b.damage_dealt.value-1.56)<1e-9)
 local base=D.mod_budget.build(D.mod_pool,{cinder=3},{damage_dealt=1.7},{})
 assert(base.damage_dealt.raw>.6)
end)
T.test('combined ordinary heal and DOT retain healing until the numeric frame safety boundary',function()
 local e=D.mod_engine.new(1,D.mod_pool);e:set_build(1,{feasting=3,renewal=3},{})
 e:begin_frame({[1]={percent=100},[2]={percent=0}});e.statuses[2]={burn={amount=3,expires=1000,next_tick=60,stacks=1,max=1,origin={}}}
 e:emit{kind='ko_dealt',port=1,target=2};e:emit{kind='status_applied',port=1,status='guarded'};e:drain()
 assert(e.damage[1]==-18,'ordinary healing was silently capped at the old balance ceiling')
end)
T.test('unsupported outgoing launch fighter value is refused rather than silently ignored',function()
 local m=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[15]));m.effects={{op='value',key='launch_dealt',value=1.1}};m.families={'launch_dealt'}
 T.refuses(function()D.mod_budget.validate_pool({m},{})end)
end)
T.test('aggregate IDs retain actor and victim passive/status provenance',function()
 local e=D.mod_engine.new(1,D.mod_pool);e:set_build(1,{glass_core=1},{damage_dealt=1.08});e:set_build(2,{ember_crown=1},{knockback_taken=.92})
 e.statuses[2]={guarded={amount=1,origin={'Guarded applied by Shelter'}}}
 local out=e:native_origin({1001},2,1);assert(table.concat(out,' / '):find('Glass Core',1,true) and table.concat(out,' / '):find('Drive implicit',1,true))
 local incoming,used=e:native_origin({1002},2,1);local text=table.concat(incoming,' / ')
 assert(text:find('Ember Crown',1,true) and text:find('Guarded applied by Shelter',1,true) and used[1]=='guarded')
 assert(not text:find('Glass Core',1,true),'incoming budget used attacker passives')
end)
T.test('native rules reserve both aggregate slots before transient status arrives',function()
 local pool,mods={},{};for i=1,31 do local m=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[3]));m.id='reserve_'..string.char(97+math.floor((i-1)/26))..string.char(97+(i-1)%26);m.group=m.id;pool[i]=m;mods[m.id]=1 end
 local e=D.mod_engine.new(1,pool);T.refuses(function()e:set_build(1,mods,{})end);assert(not e.equipped[1],'capacity refusal mutated build')
end)
T.done()
