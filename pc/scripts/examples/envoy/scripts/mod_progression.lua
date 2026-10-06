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
 -- A run's own length sets the New Game+ offset: effective depth = stage + 13 x units x loop, so the loop never starts below the
 -- run's last stage. Classic has 11 stages (one unit, 13); Adventure has 22 (two units, 26). The stage is NOT capped: depth follows
 -- the stage index through the whole run.
 P.run_units={classic=1,adventure=2}
 function P.units(mode) return P.run_units[mode] or 1 end
 function P.run_context(mode,stage,loop) return P.context(stage,(loop or 0)*P.units(mode)) end
 function P.run_loop(mode,context) return math.floor(P.context(context).loop/P.units(mode)) end
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
 -- RULES PER DRIVE (readability split, 2026-10-05): never more than two. ONE rule below effective depth 5, two from depth 5 on, and no
 -- extra rule for a white drive (the roller alternates a standing rule and a trigger rule, so a two-rule drive is one of each). Rarity stops
 -- meaning "more rules": a magic and a rare drive both carry up to two; the better tier comes from merging and from depth.
 -- Bands are in EFFECTIVE depth (depth + 13 per New Game+ loop), so every loop is in the top band.
 P.max_rules=2
 P.affix_bands={{top=4,cap=1},{top=math.huge,cap=P.max_rules}}
 P.rarity_from={common=0,magic=3,rare=6,unique=10}
 function P.affix_cap(c) local e=P.effective(c);for _,b in ipairs(P.affix_bands) do if e<=b.top then return b.cap end end end
 function P.rarity_allowed(c,rarity) return P.effective(c)>=P.rarity_from[rarity] end
 P.rarity_affixes={common=1,magic=2,rare=2}
 -- Natural drop weights by band. Tuned so the build power of a filled set of slots stays within about 10% of the
 -- pre-curve game from depth 5 on (see PLAYTEST: affix-count curve), while early drives are one plain effect.
 P.rarity_bands={{top=2,w={common=100,magic=0,rare=0,unique=0}},{top=5,w={common=85,magic=15,rare=0,unique=0}},
  {top=9,w={common=82,magic=15,rare=3,unique=0}},{top=math.huge,w={common=80,magic=15,rare=4,unique=1}}}
 function P.rarity_weights(c) local e=P.effective(c);for _,b in ipairs(P.rarity_bands) do if e<=b.top then return b.w end end end
 -- The top of the band an effective depth sits in: the room an opponent's roll may borrow in tier without leaving the depth's rarity band (it keeps an early
 -- opponent from rolling a unique at depth 5). Rarity bands, since the rule-count bands are only two now.
 function P.band_top(c) local e=P.effective(c);for _,b in ipairs(P.rarity_bands) do if e<=b.top then return b.top end end end
 -- The one count rule: rarity's own count, held down by the depth band. `white` is accepted and ignored (White no longer adds a rule).
 function P.affix_count(c,rarity,white)
  return math.min(P.rarity_affixes[rarity] or 1,P.affix_cap(c))
 end
 -- Opponents roll against the player's ACTUAL build strength times this edge, which grows with effective depth
 -- (was .003: opponents were barely ahead; .010 gives +5% at depth 5, +10% at depth 10, +13% in New Game+ 1, +39% in NG+3; a bigger edge makes the late exchanges lopsided (power_curve test)).
 P.opponent_edge=.010
 -- The edge stops growing at +25%: AI-versus-AI runs (envoy-foes harness, 126 trials) showed opponents out-dealing the player 1.0x at loop 0,
 -- 1.3x at NG+1 (edge 13-23%), 2.5x at NG+2 (26-36%) and 4.6x at NG+3 (39-49%) in damage dealt.
 P.opponent_edge_cap=.25
 function P.factor(c,role)
  assert(role==nil or role=='normal' or role=='boss' or role=='finalboss','unknown opponent role')
  return (1+math.min(P.opponent_edge_cap,P.opponent_edge*P.effective(c)))*(role=='boss' and 1.15 or role=='finalboss' and 1.3 or 1)
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
 -- ---- the Versus set (online Envoy, stage 3) -------------------------------------------------------------------------------------
 -- Pure functions of (set seed, game, seat, the picks so far): every client computes BOTH players' builds and offers the same way, so nothing but the
 -- seed and each pick (an index) crosses the network. The records are passive only (see mod_engine E.online_safe); a build is one starter record,
 -- one keystone, and what the picks added. Needs D.mod_pool, D.mod_codec and D.mod_engine at call time (this module loads first).
 -- Picks use the native lobby's numbering: 0..2 = the offer shown, 3 = keep the build (also the late default would be 0, see gw_netplay.c).
 P.set={offers=3,keep=3,max_tier=3}
 local function set_ids(D,kind)
  local ids={}
  for _,m in ipairs(D.mod_pool) do if m.kind==kind and D.mod_engine.online_safe(m) then ids[#ids+1]=m.id end end
  table.sort(ids);return ids
 end
 -- The starter record and the keystone of one seat. Seats differ (the seed is mixed with the seat) and the choice is a pure function of the seed.
 function P.set_starter(D,seed,seat)
  local r=D.mod_codec.rng(D.mod_codec.seed_for(seed,0,0,seat*7+1))
  local normal,keys=set_ids(D,'normal'),set_ids(D,'keystone')
  assert(#normal>=3 and #keys>=1,'no online-safe records to start a set from')
  return normal[math.floor(r()*#normal)+1],keys[math.floor(r()*#keys)+1]
 end
 -- Three distinct offers for a seat at a game: online-safe normal records, those already at the top tier last.
 function P.set_offers(D,seed,game,seat,eq)
  local r=D.mod_codec.rng(D.mod_codec.seed_for(seed,game,0,seat*7+3))
  local pool=set_ids(D,'normal');local fresh,full={},{}
  for _,id in ipairs(pool) do local t=eq and eq[id] or 0;if type(t)~='number' then t=0 end;if t>=P.set.max_tier then full[#full+1]=id else fresh[#fresh+1]=id end end
  local out={}
  while #out<P.set.offers do
   local from=#fresh>0 and fresh or full;assert(#from>0,'offer pool empty')
   local i=math.floor(r()*#from)+1;out[#out+1]=table.remove(from,i)
  end
  return out
 end
 -- The build after a pick: the offered record at tier 1, or one tier up if it is held; keep (3) or anything outside the offer changes nothing.
 function P.set_apply(eq,offers,pick)
  local out={};for k,v in pairs(eq) do out[k]=v end
  local id=offers[(pick or P.set.keep)+1];if not id or pick==P.set.keep then return out end
  local t=out[id];out[id]=math.min(P.set.max_tier,(type(t)=='number' and t or 0)+1)
  return out
 end
 -- The build of a seat at the start of `game` (1 = the starters). picks[g] = {[1]=host's pick,[2]=guest's pick} for g = 2..game.
 function P.set_build(D,seed,game,seat,picks)
  local starter,key=P.set_starter(D,seed,seat);local eq={[starter]=1,[key]=1}
  for g=2,game do
   local pick=picks and picks[g] and picks[g][seat]
   if pick==nil then pick=0 end -- a missing pick is the deterministic default: the first offer
   eq=P.set_apply(eq,P.set_offers(D,seed,g,seat,eq),pick)
  end
  return eq,{}
 end
 -- Everything a client stages for the next game: both seats' records and passive ops. Returns {[seat]={record=,digest=,ops=,build=}}.
 function P.set_stage(D,seed,game,picks)
  local out={}
  for seat=1,2 do
   local eq,imp=P.set_build(D,seed,game,seat,picks)
   local record,digest=D.mod_codec.build_record({seed=seed,game=game,loop=0,port=seat},eq,imp)
   local engine=D.mod_engine.new(seed,D.mod_pool);engine:set_build(seat,eq,imp)
   out[seat]={record=record,digest=digest,ops=engine:passive_ops(seat,seat),build=eq}
  end
  return out
 end
 return P
end
