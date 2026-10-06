local T=dofile('melee/pc/tests/envoy_testlib.lua');local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_budget','mod_pool','mod_engine','drive_loot','drive_bag'})do D[n]=T.module(n,D)end
local function close(a,b)assert(math.abs(a-b)<1e-8,tostring(a)..' != '..tostring(b))end
local tests={}
function tests.mixed()
 local loot=D.drive_loot.new(D.mod_pool);local bag=D.drive_bag.new(loot,{context={depth=12,loop=3}})
 local low={seed=1,depth=20,colour='white',rarity='unique',unique='glass_core',affixes={{id='glass_core',tier=1}}}
 local high={seed=2,depth=12,loop=3,colour='white',rarity='unique',unique='glass_core',affixes={{id='glass_core',tier=11}}}
 for i,r in ipairs({low,high})do assert(bag:give(r));assert(bag:equip(1,i))end
 local mods,imp=bag:derive();local e=D.mod_engine.new(1,D.mod_pool,{context=bag.context});e:set_build(1,mods,imp)
 local expected=.6+.6*D.mod_progression.growth(11);close(e:family_budget(1).damage_dealt.raw,expected)
 assert(mods.glass_core.tiers[1]==1 and mods.glass_core.tiers[2]==11 and D.mod_schema.level(mods.glass_core)==11)
 local snap=e:export();local restored=D.mod_engine.new(2,D.mod_pool);restored:import(snap);assert(restored:export()==snap)
 for _,bad in ipairs({{tier=11,copies=2,tiers={1,0}},{tier=11,copies=2,tiers={1,11,11}},{tier=10,copies=2,tiers={1,11}}})do
  T.refuses(function()e:set_build(1,{glass_core=bad},{})end);assert(e:export()==snap)
 end
 local homogeneous=D.mod_schema.ratio(1.6,e.rules.glass_core,{tier=11,copies=2},'damage_dealt')
 close(homogeneous-1,1.2*D.mod_progression.growth(11))
end
function tests.raw()
 local e=D.mod_engine.new(1,D.mod_pool,{context={depth=12,loop=3}})
 e:set_build(1,{},{});e:set_build(2,{storm_shell=11,armoured=11,frozen_oath=11},{})
 local delta=(-.08)*D.mod_progression.growth(11);close(e:family_budget(2).damage_taken.raw,delta)
 local native=e:native_rules(2);local aggregate;for _,r in ipairs(native)do if r.id==1002 then aggregate=r.change.percent_damage end end;close(aggregate,1+delta)
 local hit=e:contact_ratios(1,2,'fire');close(hit.incoming,.15)
 e:set_build(2,{glass_core={tier=11,copies=4},storm_shell=11,armoured=11},{})
 close(e:family_budget(2).damage_taken.raw,delta+2.4);close(e:contact_ratios(1,2).incoming,1+delta+2.4)
 e:set_build(1,{glass_core={tier=1000000,copies=4},storm_shell=1000000},{})
 local h=e:contact_ratios(1,2);close(h.outgoing,64);assert(h.incoming>=.15)
 for _,r in ipairs(e:native_rules(1))do for _,v in pairs(r.change)do if type(v)=='number' then assert(v==v and math.abs(v)<=1e9)end end end
end
function tests.element()
 local e=D.mod_engine.new(1,D.mod_pool,{context={depth=12,loop=3}})
 e:set_build(1,{pyromancer=11},{});e:set_build(2,{frozen_oath=11},{})
 local h=e:contact_ratios(1,2);assert(h.element=='fire');close(h.incoming,1)
 close(e:contact_ratios(1,2,'fire').incoming,1.6)
 local reverse=e:contact_ratios(2,1);assert(reverse.element=='ice');close(reverse.incoming,1)
 close(e:contact_ratios(2,1,'ice').incoming,1.6)
end
function tests.upper()
 local record=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[3]));record.id='discount';record.group='discount';record.effects={{op='versus-status',status='burn',match={},change={percent_damage=-20}}};record.families={'damage_dealt'}
 local pool={record};for _,m in ipairs(D.mod_pool)do pool[#pool+1]=m end
 local e=D.mod_engine.new(1,pool,{context={depth=12,loop=3}});e:set_build(1,{glass_core=15,discount=1},{})
 e.statuses[2]={burn={amount=1,origin={}}}
 local expected=1+.6*D.mod_progression.growth(15)-21;assert(expected>.05 and expected<64);close(e:contact_ratios(1,2).outgoing,expected)
end
function tests.tooltip()
 local loot=D.drive_loot.new(D.mod_pool);local r={seed=1,depth=12,loop=3,colour='yellow',rarity='unique',unique='storm_shell',affixes={{id='storm_shell',tier=11}}}
 local text=table.concat(loot:tooltip(r),' / ');assert(not text:find('x-',1,true),'negative physical ratio shown');assert(text:find('Deal 20% less attack damage',1,true))
end
function tests.effects()
 local m=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[6]));m.id='mixed_heal';m.kind='unique';m.min_depth=nil;m.cost='Requires a KO';m.affix=nil;m.group=nil;m.weight=nil;m.conditions={};m.tiers={{heal=2},{heal=10}}
 local e=D.mod_engine.new(1,{m});e:set_build(1,{mixed_heal={tier=2,copies=2,tiers={1,2}}},{})
 e:begin_frame({[1]={percent=50},[2]={percent=0}});e:emit{kind='ko_dealt',port=1,target=2,tags={}};e:drain();close(e.damage[1],-12);assert(e.used==2)
 m.trigger='interval';m.interval='$frames';m.tiers={{heal=2,frames=60},{heal=10,frames=90}}
 e=D.mod_engine.new(1,{m});e:set_build(1,{mixed_heal={tier=2,copies=2,tiers={1,2}}},{})
 e.frame=59;e:begin_frame({[1]={percent=50}});e:drain();close(e.damage[1],-2);assert(e.used==1)
end
if arg[1] then T.test(arg[1],assert(tests[arg[1]]))else for _,n in ipairs({'mixed','raw','element','upper','tooltip','effects'})do T.test(n,tests[n])end end
T.done()
