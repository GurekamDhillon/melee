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
 function P.keystones(c) return math.min(3,1+math.floor(P.effective(c)/5)) end
 function P.factor(c,role)
  assert(role==nil or role=='normal' or role=='boss' or role=='finalboss','unknown opponent role')
  return (1+.003*P.effective(c))*(role=='boss' and 1.15 or role=='finalboss' and 1.3 or 1)
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
