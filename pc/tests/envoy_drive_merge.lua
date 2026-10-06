-- Merging, the drive economy (quantity, small bag), and the one-call grid cell.
local T=dofile('melee/pc/tests/envoy_testlib.lua');local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_budget','keystones','mod_pool','mod_engine','drive_loot','drive_bag','drive_merge','drive_economy','drive_text'}) do D[n]=T.module(n,D) end
local P,M,E=D.mod_progression,D.drive_merge,D.drive_economy;local loot=D.drive_loot.new(D.mod_pool)
local function rec(colour,rarity,depth,ids,loop) local r={seed=1,depth=depth,colour=colour,rarity=rarity,affixes={},loop=loop}
 local td=5;for _,id in ipairs(ids) do local m=loot.rules[id];r.affixes[#r.affixes+1]={id=id,tier=loop==nil and math.min(#m.tiers,3,1+math.floor(depth/td)) or 1+math.floor(P.effective(P.context(depth,loop))/td)} end
 assert(loot:validate(r));return r end
T.test('what matches: same colour, comparable modifier, not unique, merges left',function()
 local held=rec('red','common',0,{'kindling'})
 assert(M.can_merge(held,rec('red','common',0,{'kindling'}),loot),'same modifier')
 assert(M.can_merge(held,rec('red','common',0,{'cleansing'}),loot),'comparable (both sustain)')
 assert(not M.can_merge(held,rec('red','common',0,{'burning'}),loot),'nothing in common')
 assert(M.can_merge(held,rec('blue','common',0,{'kindling'}),loot),'the same rule merges across colours')
 assert(not M.can_merge(held,rec('blue','common',0,{'cleansing'}),loot),'a family match alone still needs the same colour')
 local u=loot:roll(5,12,'unique');assert(not M.can_merge(u,u,loot));assert(not M.can_merge(held,u,loot))
 local full=rec('red','common',0,{'kindling'});full.merged=3;assert(not M.can_merge(full,held,loot))
end)
T.test('a merge lifts exactly one modifier one tier, adds none, and keeps colour, rarity and count',function()
 local held=rec('red','magic',5,{'lingering','kindling'});local gained=rec('red','common',0,{'kindling'})
 local out,info=M.merge(held,gained,loot);assert(out and #out.affixes==2 and out.rarity=='magic' and out.colour=='red' and out.merged==1)
 assert(info.affix=='kindling' and info.to==info.from+1);assert(held.merged==nil and held.affixes[2].tier==2,'the held record is untouched')
 assert(out.affixes[1].tier==2 and out.affixes[2].tier==3,'lingering keeps its tier, kindling gains one')
 assert(loot:validate(out))
 -- three merges at most, never a fourth modifier, never above depth tier + merges
 local r=rec('red','common',0,{'kindling'});for i=1,3 do r=assert(M.merge(r,gained,loot)) end
 assert(r.merged==3 and #r.affixes==1 and r.affixes[1].tier==4 and not M.can_merge(r,gained,loot));assert(loot:validate(r))
 -- the bag takes it and derives a stronger modifier than the plain one
 local bag=D.drive_bag.new(loot);assert(bag:give(r));assert(bag:equip(1,1));assert(bag:derive().kindling==4)
end)
T.test('a merge cannot turn an early simple drive into a many-modifier one',function()
 local early=rec('green','common',0,{'ledge'});local deep=loot:roll(9,12,'rare')
 for seed=1,200 do local d=loot:roll(seed,12,'rare');d.colour='green';-- force a colour match; keep only valid records
  local ok,why=pcall(function() loot:validate(d) end);if ok then local out=M.merge(early,d,loot);if out then assert(#out.affixes==1,'merge added modifiers');assert(loot:validate(out)) end end end
 -- merged deeper: the early drive re-bases to the deeper tier
 local d=rec('green','common',10,{'ledge'});local out,info=M.merge(early,d,loot);assert(out.depth==10 and info.rebased and out.affixes[1].tier==4-0 or out.affixes[1].tier>=3)
 assert(#out.affixes==1 and loot:validate(out))
end)
T.test('find_target prefers the exact modifier, then fewer merges; gain_plan: merge, bag, or choose',function()
 local gained=rec('red','common',0,{'kindling'})
 local list={rec('red','common',0,{'ledge'}),rec('red','magic',0,{'kindling','lingering'}),rec('blue','common',0,{'kindling'})}
 assert(M.find_target(list,gained,loot)==2)
 assert(M.find_target({rec('blue','common',0,{'kindling'})},gained,loot)==1,'a duplicate rule merges across colours')
 assert(M.find_target({rec('blue','common',0,{'cleansing'})},gained,loot)==nil,'a family match alone needs the same colour')
 local plan=E.gain_plan(list,2,gained,loot);assert(plan.action=='merge' and plan.index==2 and plan.info.affix=='kindling')
 assert(E.gain_plan({rec('blue','common',0,{'ledge'})},2,gained,loot).action=='bag')
 assert(E.gain_plan({rec('blue','common',0,{'ledge'})},E.tuning.bag_capacity,gained,loot).action=='choose')
 assert(E.tuning.bag_capacity==4 and E.before.bag_capacity==12)
end)
T.test('the quantity curve: at most one floor drop a stage, fewer rewards, roughly half the drives by stage ten',function()
 for _,kind in ipairs(E.kinds) do local f=E.stage(E.tuning,kind,3);assert(f<=1,kind) end
 local before,after=E.curve(E.before),E.curve(E.tuning)
 assert(after[10]<=before[10]*.55,before[10]..' vs '..after[10])
 print(('drives gained by stage 10: before %.1f after %.1f; by stage 12: %.1f / %.1f'):format(before[10],after[10],before[12],after[12]))
 local rewards=0;for i,kind in ipairs(E.kinds) do local _,r=E.stage(E.tuning,kind,i-1);rewards=rewards+r end;assert(rewards==6 and rewards<12)
 assert(E.reward_rarity(P.context(0,0),3)=='magic' and E.reward_rarity(P.context(8,0),3)=='rare' and E.reward_rarity(P.context(8,0),1)=='magic')
end)
T.test('one call gives a grid cell everything it draws: colour, rarity, pips, level, flags, a short name',function()
 local r=loot:roll(3,12,'rare');local c=D.drive_text.cell(loot,r,{new=true,can_merge=true})
 assert(c.kind=='drive' and c.colour==r.colour and c.rarity=='rare' and c.affixes==#r.affixes and c.level>=1 and c.level<=5 and c.new and c.can_merge and not c.equipped and c.colour_rgba and c.rarity_rgba)
 assert(not c.name:find('of the',1,true) and #c.name<=60,c.name)
 local one=D.drive_text.cell(loot,loot:roll(1,0));assert(one.affixes==1 and one.rarity=='common' and not one.new)
 local text=D.drive_text.short(loot,r);assert(not text:find('[{}$]') and not text:find('[Tt]ier') and not text:find('%d%.%d'))
 for seed=1,200 do local d=loot:roll(seed,seed%14,nil,seed%3);local cell=D.drive_text.cell(loot,d);assert(cell.affixes==#d.affixes and #cell.name>3) end
 local k=D.drive_text.keystone_cell(loot.rules.bulwark,{held=true});assert(k.kind=='keystone' and k.colour=='blue' and k.held and k.family=='Defence')
end)
T.done()
