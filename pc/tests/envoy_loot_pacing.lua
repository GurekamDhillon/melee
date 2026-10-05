-- Loot pacing: the affix-count curve, rarity by depth, one-line names, the uncapped keystone allowance, opponents on the same curve.
local T=dofile('melee/pc/tests/envoy_testlib.lua');local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_budget','keystones','mod_pool','mod_engine','drive_loot','drive_bag','foe_roll'}) do D[n]=T.module(n,D) end
local P=D.mod_progression;local loot=D.drive_loot.new(D.mod_pool)
T.test('affix count is a function of rarity and depth (the curve table)',function()
 -- effective depth -> cap
 for e,cap in pairs({[0]=1,[2]=1,[3]=2,[5]=2,[6]=3,[9]=3,[10]=4,[13]=4,[500]=4}) do assert(P.affix_cap(P.context(e,0))==cap,e) end
 assert(P.affix_cap(P.context(0,1))==4,'every New Game+ loop is in the top band')
 for depth=0,12 do for _,rarity in ipairs({'common','magic','rare'}) do
  local want=math.min(({common=1,magic=2,rare=4})[rarity],P.affix_cap(P.context(depth,0)))
  for seed=1,40 do local r=loot:roll(seed*7919+depth,depth,rarity);assert(#r.affixes==want or r.colour=='white',depth..rarity);assert(loot:validate(r)) end
 end end
 -- a lucky early Rare is still simple; a white drive gets its extra modifier only past depth 2
 for seed=1,60 do local r=loot:roll(seed,1,'rare');assert(#r.affixes==1) end
 for seed=1,200 do local r=loot:roll(seed,0);assert(#r.affixes==1 and r.rarity=='common' and r.unique==nil) end
 local seen_white;for seed=1,300 do local r=loot:roll(seed,4,'rare');if r.colour=='white' then seen_white=true;assert(#r.affixes==3) end end;assert(seen_white)
end)
T.test('rarities appear by depth: common only to 2, magic from 3, rare from 6, unique from 10',function()
 local function kinds(depth,loop) local out={};for seed=1,3000 do out[loot:roll(seed,depth,nil,loop).rarity]=true end;return out end
 local a,b,c,d=kinds(2),kinds(5),kinds(9),kinds(10)
 assert(a.common and not a.magic and not a.rare and not a.unique)
 assert(b.magic and not b.rare and not b.unique);assert(c.rare and not c.unique);assert(d.unique and d.rare)
 assert(kinds(0,1).unique,'a New Game+ loop rolls uniques')
 for _,r in ipairs({'common','magic','rare','unique'}) do assert(P.rarity_allowed(P.context(10,0),r)) end
end)
T.test('one affix is one short name, no prefix and suffix chain',function()
 for seed=1,50 do local r=loot:roll(seed,0);local n=loot:name(r);assert(n:find('^%u%l+ Drive: '),n);assert(not n:find(' of the ',1,true),n) end
 local r=loot:roll(3,12,'rare');local n=loot:name(r);assert(#r.affixes==4 and (n:find(' of the ',1,true) or n:find('Drive$')),n)
end)
T.test('the power curve stays honest: filled slots within 12% of the pre-curve game from depth 5, lower early',function()
 -- Pre-curve shape (common 0 affixes, magic 2, rare 4, natural weights 60/28/10/2) rebuilt as records.
 local function old(seed,ctx)
  local x=seed*104729%2147483646+1;local function r(n) x=x*16807%2147483647;return (x-1)/2147483646*n end
  local pick=r(100);local rar=pick<60 and 'common' or pick<88 and 'magic' or pick<98 and 'rare' or 'unique'
  local rec={seed=seed,depth=ctx.depth,loop=ctx.loop>0 and ctx.loop or nil,colour=loot.colours[math.floor(r(6))+1],rarity=rar,affixes={}}
  local tier=1+math.floor(P.effective(ctx)/5);local groups={}
  if rar=='unique' then local m=loot.uniques[math.floor(r(#loot.uniques))+1];rec.unique=m.id;rec.colour=m.fixed_colour or 'white';rec.affixes={{id=m.id,tier=rec.loop==nil and 1 or P.tier(ctx)}};return rec end
  local function aff(kind) local c={};for _,m in ipairs(loot.normal) do if not groups[m.group] and (not kind or m.affix==kind) then c[#c+1]=m end end
   local m=c[math.floor(r(#c))+1];groups[m.group]=true;rec.affixes[#rec.affixes+1]={id=m.id,tier=rec.loop==nil and math.min(3,tier) or tier} end
  for _=1,rar=='rare' and 2 or rar=='magic' and 1 or 0 do aff('prefix');aff('suffix') end
  if rec.colour=='white' then aff() end;return rec
 end
 local function mean(make,ctx) local sum,N=0,120
  for s=1,N do local bag=D.drive_bag.new(loot,{context=ctx});for i=1,P.slots(ctx) do bag.equipped[i]=make(s*100+i,ctx) end
   local mods,imp=bag:derive();local _,st=D.mod_budget.build(D.mod_pool,mods,imp,{});sum=sum+st/N end;return sum end
 local ratios={}
 for _,c in ipairs({{0,0},{5,0},{10,0},{0,1}}) do local ctx=P.context(c[1],c[2])
  local o,n=mean(old,ctx),mean(function(s,x) return loot:roll(s,x) end,ctx);ratios[#ratios+1]=n/o;print(('depth %d loop %d old %.2f new %.2f ratio %.3f'):format(c[1],c[2],o,n,n/o))
 end
 assert(ratios[1]<1,'early drives must be tamer');for i=2,4 do assert(ratios[i]>=.95 and ratios[i]<=1.12,'late power drifted: '..ratios[i]) end
end)
T.test('keystone allowance has no ceiling: +1 every 5 effective depth, loops included',function()
 assert(P.allowance(P.context(0,0))==1 and P.allowance(P.context(4,0))==1 and P.allowance(P.context(5,0))==2 and P.allowance(P.context(10,0))==3)
 assert(P.allowance(P.context(0,1))==3 and P.allowance(P.context(0,5))==14 and P.allowance(P.context(12,3))==11)
 for e=0,400 do assert(P.keystones(P.context(e,0))==P.allowance(P.context(e,0)) and P.allowance(P.context(e+1,0))>=P.allowance(P.context(e,0))) end
 -- the bag and the engine read the same rule: depth 0 holds one, depth 5 two, an unrelated third is refused
 local bag=D.drive_bag.new(loot,{context=P.context(0,0)});assert(bag:choose_keystone('bulwark'));assert(not bag:choose_keystone('sprinter') or #bag.keystones==1)
 local deep=D.drive_bag.new(loot,{context=P.context(12,3)})
 for _,id in ipairs({'bulwark','sprinter','skyborne','pandemic','fury'}) do assert(deep:choose_keystone(id),id) end
 assert(#deep.keystones==5)
 local e=D.mod_engine.new(1,D.mod_pool,{context=P.context(0,0)});local mods={bulwark=1,sprinter=1}
 T.refuses(function() e:set_build(1,mods,{}) end)
end)
T.test('opponents follow the same curve: simple early, four affixes only deep, uniques only once they roll',function()
 local R=D.foe_roll.new(D.mod_pool)
 local function stats(ctx) local maxn,uniques,n=0,0,0
  for s=1,12 do local pl=R:sample(300+s,ctx);local foe=R:roll(pl.strength,700+s,ctx.depth,2,ctx)
   for _,rec in pairs(foe.build.equipped) do n=n+1;maxn=math.max(maxn,#rec.affixes);if rec.unique then uniques=uniques+1 end end end
  return maxn,uniques,n end
 for depth=0,2 do local m,u=stats(P.context(depth,0));assert(m<=1 and u==0,'depth '..depth..' foes: '..m..' affixes '..u..' uniques') end
 for depth=3,5 do local m,u=stats(P.context(depth,0));assert(m<=3 and u==0,'depth '..depth) end -- 2, +1 for a White drive
 for depth=6,9 do local m,u=stats(P.context(depth,0));assert(m<=5,'depth '..depth) end -- 3, +1 for White; 4+1 only when a foe must keep pace with a full player build and takes the unbanded fallback
 local m,u,n=stats(P.context(0,1));assert(m==4 or m==1 or m>=1);assert(n>0)
 -- held keystones follow the allowance and the exclusion rules
 local ctx=P.context(12,3);local pl=R:sample(5,ctx);assert(D.keystones.check(pl.build.keystones),'sampled build must be a legal keystone set')
end)
T.test('a depth the curve cannot reach falls back to unbanded rolls instead of refusing',function()
 local R=D.foe_roll.new(D.mod_pool);local ctx=P.context(0,0)
 local rec=R:roll(8,41,0,2,ctx);R:validate(rec);assert(rec.strength>=rec.target*.8)
end)
T.done()
