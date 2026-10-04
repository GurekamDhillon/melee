-- The one declaration of the named statuses. Their mirrored hit-rule bits, event tags, loot-budget
-- contributions, fighter-value effects and looks are all read from here, so a status is added or
-- retuned in one place. The live instances (stacks, origin, expiry) stay in mod_engine.statuses; the
-- native numeric timed channels (gd.fighter_timed_status) are a separate, private mechanism.
-- Names are mechanics, not marketing: 'guarded' is a percent resistance, not reaction armour.
return function(D)
 local M={}
 M.defs={
  {name='burn',bit=1,tag='burning',look='burn',implemented=true,
   budget=function(amount,max) return {sustain=amount*(max or 1)} end},
  -- Shock's hitstun effect awaits safe hit mutation: declared for the bit and look, not applicable.
  {name='shock',bit=2,tag='shocked',look='shock',implemented=false},
  {name='chill',bit=4,tag='chilled',look='chill',implemented=true,
   budget=function() return {speed=-.2} end,values={run_speed=-.2,air_speed=-.2}},
  {name='curse',bit=8,tag='cursed',look='curse',implemented=true,
   budget=function(amount) return {launch_taken=amount} end,values=function(v) return {knockback_taken=v.amount} end},
  {name='haste',bit=16,tag='hasted',look='haste',implemented=true,
   budget=function() return {speed=.2} end,values={run_speed=.2,air_speed=.2}},
  {name='guarded',bit=32,tag='guarded',look='guarded',implemented=true,
   budget=function() return {damage_taken=-.25,launch_taken=-.15} end,values={damage_taken=-.25,knockback_taken=-.15}},
  {name='momentum',bit=64,tag='momentum',look='momentum',implemented=true,
   budget=function(_,max) return {momentum=max or 5} end},
 }
 M.order,M.by_name,M.bits,M.tags,M.implemented={},{},{},{},{}
 for _,d in ipairs(M.defs) do
  M.order[#M.order+1]=d.name;M.by_name[d.name]=d
  if d.implemented then M.bits[d.name]=d.bit;M.tags[d.name]=d.tag;M.implemented[d.name]=true end
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
