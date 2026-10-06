-- The one declaration of the named statuses. Their mirrored hit-rule bits, event tags, loot-budget
-- contributions, fighter-value effects and looks are all read from here, so a status is added or
-- retuned in one place. The live instances (stacks, origin, expiry) stay in mod_engine.statuses; the
-- native numeric timed channels (gd.fighter_timed_status) are a separate, private mechanism.
-- Names are mechanics, not marketing: 'guarded' is a percent resistance, not reaction armour.
--
-- THE WORDS (readability split, 2026-10-05). Five core statuses everyone learns (Burning, Chilled, Haste, Guarded, Marked), one
-- counter (Momentum: a stored number, not a status look) and one private status (Shock, the electric theme's own). Each has ONE word
-- in every text a player reads; its numbers live in the glossary below, not in every sentence. The internal ids never change
-- (`curse` is still the id of Marked, so saves and build records stay valid). `label` is the word; `puts` says it on a target.
return function(D)
 local M={}
 M.defs={
  {name='burn',bit=1,tag='burning',look='burn',implemented=true,label='Burning',puts='sets the target Burning',gloss='Burning: the target takes damage every second.',core=true,
   budget=function(amount,max) return {sustain=amount*(max or 1)} end},
  -- Shock is native now (gd.shock, journal op `shock`): the target's next hit taken has more hitstun. The host mirrors the Lua
  -- status into that native state. It has no hit-rule bit (versus-status rules cannot test it), so it stays out of M.bits.
  {name='shock',bit=2,tag='shocked',look='shock',implemented=true,no_bit=true,label='Shock',puts='Shocks the target',private=true,
   gloss='Shock: the target\'s next hit taken stuns longer (private to the electric theme).',
   budget=function() return {launch_taken=.06} end},
  {name='chill',bit=4,tag='chilled',look='chill',implemented=true,label='Chilled',puts='Chills the target',gloss='Chilled: 20% slower.',core=true,
   budget=function() return {speed=-.2} end,values={run_speed=-.2,air_speed=-.2}},
  {name='curse',bit=8,tag='cursed',look='curse',implemented=true,label='Marked',puts='Marks the target',gloss='Marked: your later hits launch the target farther.',core=true,
   budget=function(amount) return {launch_taken=amount} end,values=function(v) return {knockback_taken=v.amount} end},
  {name='haste',bit=16,tag='hasted',look='haste',implemented=true,label='Haste',puts='gives you Haste',gloss='Haste: 20% faster run and air speed.',core=true,
   budget=function() return {speed=.2} end,values={run_speed=.2,air_speed=.2}},
  {name='guarded',bit=32,tag='guarded',look='guarded',implemented=true,label='Guarded',puts='gives you Guarded',gloss='Guarded: 25% less damage and 15% less launch taken.',core=true,
   budget=function() return {damage_taken=-.25,launch_taken=-.15} end,values={damage_taken=-.25,knockback_taken=-.15}},
  -- Momentum is a COUNTER: up to 5 stored stacks that a landing rule spends. It keeps its engine slot (the surface channel still has a
  -- lane for it) but it is not one of the five statuses a player learns; the later visual pass draws it as 1 to 5 orbs from `counter()`.
  {name='momentum',bit=64,tag='momentum',look='momentum',implemented=true,label='Momentum',puts='gives you Momentum',counter=true,
   gloss='Momentum: a counter, up to 5, spent when you land.',
   budget=function(_,max) return {momentum=max or 5} end},
 }
 M.order,M.by_name,M.bits,M.tags,M.implemented={},{},{},{},{}
 M.core,M.counter_name,M.private_name={},'momentum','shock'
 for _,d in ipairs(M.defs) do
  M.order[#M.order+1]=d.name;M.by_name[d.name]=d
  if d.core then M.core[#M.core+1]=d.name end
  if d.implemented then if not d.no_bit then M.bits[d.name]=d.bit end;M.tags[d.name]=d.tag;M.implemented[d.name]=true end
 end
 -- The one word for a status in player-read text.
 function M.label(name) local d=M.by_name[name];return d and d.label or tostring(name) end
 -- Words a text may never use for these (the old synonyms): the tests scan every line a player reads for them.
 M.banned_words={'Curse','Cursed','curse','cursed','moving fast','speed boost','Hasted','hasted'}
 -- The glossary: one line per word a player learns, the crit rule included (a crit is x1.5 unless a piece says otherwise).
 function M.glossary()
  local out={}
  for _,name in ipairs({'burn','chill','haste','guarded','curse','momentum','shock'}) do out[#out+1]=M.by_name[name].gloss end
  out[#out+1]='Crit: a hit that deals x1.5 damage.'
  return out
 end
 function M.budget(name,amount,max)
  local d=M.by_name[name];assert(d and d.budget,'unbudgeted status');return d.budget(amount,max)
 end
 -- Fighter-value keys a live status contributes (v is the engine's instance record).
 function M.values(name,v)
  local d=M.by_name[name];local x=d and d.values
  if type(x)=='function' then return x(v) end
  return x or {}
 end
 return M
end
