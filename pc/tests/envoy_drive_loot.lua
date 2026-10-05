local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={};for _,n in ipairs({'mod_schema','mod_codec','mod_pool','drive_loot','drive_bag'}) do D[n]=T.module(n,D) end
local loot=D.drive_loot.new(D.mod_pool)
local function equal(a,b)
 if type(a)~=type(b) then return false end;if type(a)~='table' then return a==b end
 for k,v in pairs(a) do if not equal(v,b[k]) then return false end end
 for k in pairs(b) do if a[k]==nil then return false end end;return true
end
T.test('10000 deterministic valid rolls and weighted rarity/colour distributions (ungated: the weights themselves)',function()
 local loot=D.drive_loot.new(D.mod_pool,{ungated=true});local counts={common=0,magic=0,rare=0,unique=0};local colours={}
 for seed=0,9999 do
  local s=seed;local r=loot:roll(s,seed%21)
  assert(equal(r,loot:roll(s,seed%21)));assert(loot:validate(r));counts[r.rarity]=counts[r.rarity]+1
  colours[r.colour]=(colours[r.colour] or 0)+1;assert(#loot:name(r)>0 and #loot:tooltip(r)>0)
 end
 for rarity,want in pairs({common=6000,magic=2800,rare=1000,unique=200}) do assert(math.abs(counts[rarity]-want)<120,rarity..' '..counts[rarity]) end
 for _,c in ipairs(loot.colours) do assert(colours[c]>1400 and colours[c]<1900,c) end
 print('distribution '..counts.common..'/'..counts.magic..'/'..counts.rare..'/'..counts.unique)
end)
T.test('depth floor, forced rarity, fixed unique and white extra',function()
 for seed=1,100 do for _,rarity in ipairs({'common','magic','rare','unique'}) do
  local r=loot:roll(seed*104729,20,rarity)
  for _,a in ipairs(r.affixes) do assert(a.tier==5) end
  if rarity~='unique' then assert(#r.affixes==({common=1,magic=2,rare=4})[rarity]+(r.colour=='white' and 1 or 0)) end
 end end
 T.refuses(function() loot:roll(1,1,'keystone') end)
 for depth=0,10,5 do local r=loot:roll(1,depth,'rare');for _,a in ipairs(r.affixes) do assert(a.tier==1+depth/5) end end
 T.refuses(function() D.drive_loot.new(D.mod_pool,{tier_depth=0}) end)
 T.refuses(function() D.drive_loot.new(D.mod_pool,{rarity_weights={common=math.huge,magic=1,rare=1,unique=1}}):roll(1,0) end)
end)
T.test('every bag record is validated and malformed arrays/groups refused',function()
 local bag=D.drive_bag.new(loot);for i=1,12 do assert(bag:give(loot:roll(i*104729,10,'rare'))) end
 local baseline=bag:snapshot()
 for i=1,12 do local bad=bag:snapshot();bad.items[i].affixes[1].id='invalid';assert(not bag:restore(bad));assert(equal(baseline,bag:snapshot())) end
 local r=loot:roll(104729,1,'rare');r.affixes[2]=r.affixes[1];T.refuses(function() loot:validate(r) end)
 r=loot:roll(104729,1,'rare');r.affixes[9]={id='kindling',tier=1};T.refuses(function() loot:validate(r) end)
 r=loot:roll(104729,1,'rare');r.affixes[1].tier=1.5;assert(not bag:give(r))
 -- Exact review regression: keys 2..5, four valid normal affixes, no index1.
 local dense=loot:roll(1,12,'rare');dense.colour='red'
 local hole={false,dense.affixes[1],dense.affixes[2],dense.affixes[3],dense.affixes[4]}
 hole[1]=nil;assert(#hole==5,'regression must retain a length above its sparse count');dense.affixes=hole
 T.refuses(function() loot:validate(dense) end);assert(not bag:give(dense))
end)
T.test('full bag slots swap discard and checkpoint exactness',function()
 local bag=D.drive_bag.new(loot);for i=1,12 do assert(bag:give(loot:roll(i*104729,10,'common'))) end
 assert(not bag:give(loot:roll(2,1)));for slot=1,4 do assert(bag:equip(1,slot)) end
 assert(#bag.items==8);assert(bag:equip(1,1));assert(#bag.items==8)
 for i=1,4 do assert(bag:give(loot:roll(i,0,'common'))) end
 local codec=T.module('mod_codec');local snap=bag:snapshot();local bytes=codec.encode(snap);local again=D.drive_bag.new(loot);assert(again:restore(codec.decode(bytes)));assert(equal(snap,again:snapshot()));assert(codec.encode(again:snapshot())==bytes)
 snap.items[1].colour='oops';assert(again.items[1].colour~='oops')
 assert(not bag:unequip(1))
 assert(bag:discard(1));assert(bag:unequip(1));assert(#bag.items==12 and not bag.equipped[1])
 assert(bag:new_run());assert(#bag.items==0 and not next(bag.equipped));local m,imp=bag:derive();assert(not next(m) and not next(imp))
end)
T.test('atomic preflight refusal, highest tier and separate keystone',function()
 local reject=false;local bag=D.drive_bag.new(loot,{preflight=function() return not reject,'fixture refusal' end})
 local r=loot:roll(12345,20,'rare');assert(bag:give(r));local baseline=bag:snapshot();reject=true
 assert(not bag:equip(1,1));assert(equal(baseline,bag:snapshot()));reject=false;assert(bag:equip(1,1))
 assert(bag:give(r));assert(bag:equip(1,2));local mods=bag:derive();for _,a in ipairs(r.affixes) do assert(mods[a.id]==a.tier) end
 local key;for _,m in ipairs(D.mod_pool) do if m.kind=='keystone' then key=m.id;break end end
 assert(bag:choose_keystone(key));assert(bag:choose_keystone(nil));assert(not bag:choose_keystone('kindling'))
 local persistent=D.drive_bag.new(loot,{persist=true});assert(persistent:give(r));assert(persistent:new_run());assert(#persistent.items==1)
end)
T.test('complete native rule budget rejects more than thirty regular rules without editing',function()
 local pool={};local codec=T.module('mod_codec');for i=1,12 do
  local m=codec.decode(codec.encode(D.mod_pool[3]));m.id='rule_'..string.char(96+i);m.label='Rule '..i;m.affix=i%2==0 and 'prefix' or 'suffix';m.group=m.id;m.effects={m.effects[1],m.effects[1],m.effects[1],m.effects[1]};pool[i]=m
 end
 local l=D.drive_loot.new(pool);local bag=D.drive_bag.new(l)
 for start=1,9,4 do local r={seed=start,depth=0,colour='white',rarity='rare',affixes={}};for i=start,math.min(start+4,12) do r.affixes[#r.affixes+1]={id=pool[i].id,tier=1} end
  if #r.affixes==5 then assert(bag:give(r)) end
 end
 assert(bag:equip(1,1));local before=bag:snapshot();assert(not bag:equip(1,2));assert(equal(before,bag:snapshot()))
end)
T.test('duplicate IDs retain the higher tier and unequip restores the lower tier',function()
 local bag=D.drive_bag.new(loot)
 local low=loot:roll(72,0,'rare');local high=loot:roll(72,10,'rare')
 assert(#low.affixes==1 and #high.affixes==4) -- the curve: the same seed is one affix early and four deep
 for i,a in ipairs(low.affixes) do assert(a.id==high.affixes[i].id and high.affixes[i].tier==3 and a.tier==1) end
 assert(bag:give(low));assert(bag:equip(1,1));assert(bag:give(high));assert(bag:equip(1,2))
 local mods=bag:derive();for _,a in ipairs(low.affixes) do assert(mods[a.id]==3) end
 assert(bag:unequip(2));mods=bag:derive();for _,a in ipairs(low.affixes) do assert(mods[a.id]==1) end
end)
T.done()
