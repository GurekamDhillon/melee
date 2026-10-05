-- Progression grows until numerical/physics safety, independently of event depth.
return function()
 local P={}
 local function integer(n) return type(n)=='number' and n==n and n%1==0 and n>=0 and n<=2147483646 end
 function P.context(depth,loop)
  if type(depth)=='table' then
   assert(not getmetatable(depth),'plain progression context required')
   for k in pairs(depth)do assert(k=='depth' or k=='loop','unknown progression field')end
   loop=depth.loop;depth=depth.depth
  end
  depth=depth or 0;loop=loop or 0;assert(integer(depth) and integer(loop),'depth/loop must be bounded nonnegative integers')
  return {depth=depth,loop=loop}
 end
 function P.effective(c) c=P.context(c);return c.depth+13*c.loop end
 function P.tier(c) return 1+math.floor(P.effective(c)/5) end
 function P.growth(t)
  assert(type(t)=='number' and t%1==0 and t>=1 and t<=6500000000,'invalid tier')
  -- Preserve authored EM4 tiers 1..3; logarithmic guard prevents overflow.
  if t<=3 then return 1+.25*(t-1)end
  return math.exp(math.min(math.log(1.5)+(t-3)*math.log(1.45),math.log(1000000)))
 end
 function P.launch_growth(t)
  local growth=P.growth(t);return growth<=1.5 and growth or 1.5*(growth/1.5)^.15
 end
 function P.slots(c) return math.min(6,4+math.floor(P.effective(c)/5)) end
 -- Keystone allowance: one at the start, one more every five effective depth, no ceiling (the pool, the
 -- exclusion rules and the drawback floor in `keystones.lua` are the limits). Kept under the old name too:
 -- drive_bag.lua and mod_engine.lua assert against P.keystones.
 P.keystone_step=5
 function P.allowance(c) return 1+math.floor(P.effective(c)/P.keystone_step) end
 function P.keystones(c) return P.allowance(c) end
 -- Affix count grows with depth: how many modifiers one drive may carry, and which rarities roll naturally.
 -- Bands are in EFFECTIVE depth (depth + 13 per New Game+ loop), so every loop is in the top band.
 P.affix_bands={{top=2,cap=1},{top=5,cap=2},{top=9,cap=3},{top=math.huge,cap=4}}
 P.rarity_from={common=0,magic=3,rare=6,unique=10}
 function P.affix_cap(c) local e=P.effective(c);for _,b in ipairs(P.affix_bands) do if e<=b.top then return b.cap end end end
 function P.band_top(c) local e=P.effective(c);for _,b in ipairs(P.affix_bands) do if e<=b.top then return b.top end end end
 function P.rarity_allowed(c,rarity) return P.effective(c)>=P.rarity_from[rarity] end
 P.rarity_affixes={common=1,magic=2,rare=4}
 -- Natural drop weights by band. Tuned so the build power of a filled set of slots stays within about 10% of the
 -- pre-curve game from depth 5 on (see PLAYTEST: affix-count curve), while early drives are one plain effect.
 P.rarity_bands={{top=2,w={common=100,magic=0,rare=0,unique=0}},{top=5,w={common=85,magic=15,rare=0,unique=0}},
  {top=9,w={common=82,magic=15,rare=3,unique=0}},{top=math.huge,w={common=80,magic=15,rare=4,unique=1}}}
 function P.rarity_weights(c) local e=P.effective(c);for _,b in ipairs(P.rarity_bands) do if e<=b.top then return b.w end end end
 -- The one count rule: rarity's own count, held down by the depth band; White's extra modifier only past depth 2.
 function P.affix_count(c,rarity,white)
  local n=math.min(P.rarity_affixes[rarity] or 1,P.affix_cap(c))
  return n+((white and P.effective(c)>=3) and 1 or 0)
 end
 -- Opponents roll against the player's ACTUAL build strength times this edge, which grows with effective depth
 -- (was .003: opponents were barely ahead; .010 gives +5% at depth 5, +10% at depth 10, +13% in New Game+ 1, +39% in NG+3; a bigger edge makes the late exchanges lopsided (power_curve test)).
 P.opponent_edge=.010
 function P.factor(c,role)
  assert(role==nil or role=='normal' or role=='boss' or role=='finalboss','unknown opponent role')
  return (1+P.opponent_edge*P.effective(c))*(role=='boss' and 1.15 or role=='finalboss' and 1.3 or 1)
 end
 function P.exchange(attacker,defender)
  -- Mario sweetspot forward smash, raw/current-hit18, weight100, KBG95 BKB25.
  -- Percent-only bonuses enter the next hit's starting percent, never raw damage.
  -- FD no-DI continuous horizontal-distance proxy at44deg, threshold196.8699.
  local percent=0
  local damage,scale
  if attacker.percent_damage then damage=attacker.percent_damage;scale=attacker.launch
  else damage=attacker.damage_dealt.potential*defender.damage_taken.potential;scale=attacker.launch_dealt.potential*math.max(.05,math.min(4,defender.launch_taken.potential+(attacker.curse_dealt and attacker.curse_dealt.potential-1 or 0)))end
  for hits=1,1000 do
   local after=percent+18
   local kb=((after/10+after*18/20)*1.4+18)*.95+25
   if kb*scale>=196.8699 then return hits end
   percent=math.min(999,percent+18*damage)
  end
  return nil,'launch proxy unreachable at percent display ceiling'

 end
 return P
end
