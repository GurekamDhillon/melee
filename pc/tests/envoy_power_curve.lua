local T=dofile('melee/pc/tests/envoy_testlib.lua');local D={}
T.test('progression owns monotonic tiers slots and independent difficulty',function()
 local f=io.open(T.root..'mod_progression.lua');assert(f,'missing progression authority');f:close()
 D.mod_progression=T.module('mod_progression',D)
 local P=D.mod_progression;local last=0
 for e=0,120 do local c=P.context(e,0);assert(P.growth(P.tier(c))>=last);last=P.growth(P.tier(c)) end
 assert(P.slots(P.context(0,0))==4 and P.slots(P.context(5,0))==5 and P.slots(P.context(10,0))==6)
 assert(P.keystones(P.context(12,3))==11 and P.keystones(P.context(0,0))==1 and P.keystones(P.context(5,0))==2 and P.keystones(P.context(10,0))==3 and P.keystones(P.context(0,5))==14 and P.factor(P.context(12,3),'boss')>P.factor(P.context(12,3)))
 for _,v in ipairs({-1,1.5,math.huge,0/0,2147483647}) do T.refuses(function()P.context(v,0)end) end
end)
for _,n in ipairs({'mod_schema','mod_codec','mod_budget','mod_pool','mod_engine','drive_loot','drive_bag','foe_roll'})do D[n]=T.module(n,D)end
T.test('late tiers multiple uniques and keystones contribute and restore atomically',function()
 local found={};for _,m in ipairs(D.mod_pool)do found[m.id]=m end;assert(found.armoured and found.cleansing,'same-pool defensive answers missing')
 local c=D.mod_progression.context(12,3);local loot=D.drive_loot.new(D.mod_pool);local bag=D.drive_bag.new(loot,{context=c})
 local r
 for seed=1,1000 do local v=loot:roll(seed,c,'unique');if v.unique=='glass_core' then r=v;break end end
 assert(r and r.affixes[1].tier>3);assert(bag:give(r));assert(bag:equip(1,1));local one=bag:derive()
 assert(table.concat(loot:tooltip(r),' / '):find('Tier 11: Attack percent damage +1758.7%',1,true),'late unique tooltip hides its resolved attack ratio')
 assert(bag:give(r));assert(bag:equip(1,2));local two=bag:derive();assert(two.glass_core.copies==2)
 local e=D.mod_engine.new(1,D.mod_pool,{context=c});e:set_build(1,two,{})
 local b,s=e:family_budget(1);assert(b.damage_dealt.value>10 and s>10)
 assert(bag:choose_keystone('pyromancer'));assert(bag:choose_keystone('still_heart'));assert(bag:choose_keystone('frozen_oath'))
 local mods,imp=bag:derive();e:set_build(1,mods,imp);assert(e.equipped[1].still_heart and e.equipped[1].frozen_oath)
 local snap=e:export();local bad=D.mod_codec.decode(snap);bad.context.loop=-1;T.refuses(function()e:import(D.mod_codec.encode(bad))end);assert(e:export()==snap)
end)
T.test('engine emits widened safe values preserves legal additions and rejects malformed copies',function()
 local _,plain=D.mod_budget.build(D.mod_pool,{kindling=1},{},{})
 local _,curse=D.mod_budget.build(D.mod_pool,{kindling=1,malice=1},{},{})
 assert(curse>plain,'offensive curse was counted as wearer vulnerability')
 local e=D.mod_engine.new(1,D.mod_pool,{context={depth=12,loop=3}})
 e:set_build(1,{glass_core={tier=11,copies=2},cinder=11,pyre=11,storm_shell={tier=11,copies=3}},{damage_dealt=1.4})
 local b=e:family_budget(1);assert(b.damage_dealt.raw>10 and b.damage_dealt.value>10)
 for _,r in ipairs(e:native_rules(1))do for k,v in pairs(r.change)do if k=='percent_damage' or k=='launch' then assert(v==v and math.abs(v)<=1e9)end end end
 local before=e:export();T.refuses(function()e:set_build(1,{glass_core={tier=11,copies=0/0}},{})end);assert(e:export()==before)
 T.refuses(function()e:set_build(1,{cinder={tier=11,copies=2}},{})end);assert(e:export()==before)
 e:set_build(1,{glass_core=1000000,storm_shell=1000000},{})
 for _,r in ipairs(e:native_rules(1))do for k,v in pairs(r.change)do if type(v)=='number' then assert(v==v and v<math.huge)end end end
end)
T.test('lowering context refuses occupied locked slots and malformed keystones atomically',function()
 local loot=D.drive_loot.new(D.mod_pool);local bag=D.drive_bag.new(loot,{context={depth=12,loop=3}})
 assert(bag:give(loot:roll(1,{depth=12,loop=3},'common')));assert(bag:equip(1,6));local before=D.mod_codec.encode(bag:snapshot())
 assert(not bag:set_context({depth=0,loop=0}));assert(D.mod_codec.encode(bag:snapshot())==before)
 local bad=bag:snapshot();bad.keystones={[2]='pyromancer'};assert(not bag:restore(bad));assert(D.mod_codec.encode(bag:snapshot())==before)
end)
T.test('seeded scalar opponents track all 52 contexts without consulting a reference',function()
 local roll=D.foe_roll.new(D.mod_pool);local P=D.mod_progression
 for loop=0,3 do for depth=0,12 do local c=P.context(depth,loop)
  local player=roll:sample(401+depth+loop*13,c);local target=player.strength*P.factor(c)
  local foe=roll:roll(player.strength,500+depth+loop*13,depth,2,c)
  roll:validate(foe);assert(foe.strength/target>=roll.band[1] and foe.strength/target<=roll.band[2],depth..'/'..loop)
  assert(D.mod_codec.encode(foe)==D.mod_codec.encode(roll:roll(player.strength,500+depth+loop*13,depth,2,c)))
  assert(not foe.reference and not foe.mods)
 end end
 local poison=setmetatable({},{__index=function()error('reference consulted')end})
 T.refuses(function()roll:roll(2,1,0,2,poison)end)
end)
T.test('late balanced unique and three-keystone witness retains readable exchanges',function()
 -- witness re-seeded and widened when the pool grew (41 keystones, technique modifiers): strength now includes crit and armour, which the damage-and-launch exchange model does not play out
 local roll=D.foe_roll.new(D.mod_pool);local c={depth=12,loop=3};local player=roll:sample(303,c)
 local foe=roll:roll(player.strength,250,12,2,c);local a,b=roll:exchange(player.build,foe.build)
 assert(player.strength>10 and a>=2 and a<=100 and b>=2 and b<=30) -- the exchange model counts damage and launch only, not crit or armour
 assert(player.build.equipped[5].unique and #player.build.keystones>=3)
 local boss=roll:roll(player.strength,250,12,2,c,'boss');local final=roll:roll(player.strength,250,12,2,c,'finalboss')
 assert(boss.target>foe.target and final.target>boss.target)
 local bad=D.mod_codec.decode(D.mod_codec.encode(foe));bad.build.context.depth=11;T.refuses(function()roll:validate(bad)end)
 local empty=D.mod_engine.new(1,D.mod_pool);local hit=empty:contact_ratios(1,2);assert(D.mod_progression.exchange(hit)==7)
end)
T.test('the opponent edge grows +1% per effective depth and stops at +25%',function()
 local P=D.mod_progression
 assert(math.abs(P.factor({depth=10,loop=0})-1.10)<1e-9 and math.abs(P.factor({depth=12,loop=1})-1.25)<1e-9)
 assert(math.abs(P.factor({depth=10,loop=3})-1.25)<1e-9,'capped')
 assert(math.abs(P.factor({depth=10,loop=3},'boss')-1.25*1.15)<1e-9 and math.abs(P.factor({depth=10,loop=3},'finalboss')-1.25*1.3)<1e-9,'roles still multiply the capped edge')
end)
T.done()
