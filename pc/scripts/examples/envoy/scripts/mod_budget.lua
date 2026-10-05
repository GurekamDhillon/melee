-- One authority: additive deltas, numerical safety families, bounded utility, conservative pool audit.
return function(D)
 local S=D.mod_schema;local B={}
 -- Every registered effect needs a budget answer; a field the schema admits cannot be silently unbudgeted.
 B.effect_ops={echo=true,value=true,convert=true,['versus-status']=true,status=true,stacks=true,remove_status=true,heal=true,damage=true,clank_damage=true,emit=true,armor=true,intangible=true,interrupt=true,chain_status=true,crit_next=true,air_jumps=true,restrict=true,crit=true}
 for op in pairs(D.mod_registry.effects) do assert(B.effect_ops[op],'registered effect has no budget rule: '..op) end
 for op in pairs(B.effect_ops) do assert(D.mod_registry.effects[op],'budget rule for an unregistered effect: '..op) end
 B.order={'echo','damage_dealt','launch_dealt','damage_taken','launch_taken','speed','jump','status_duration','sustain','conversion','momentum','clank','cleanse','curse_dealt','armor','intangible','air_jumps','restrict','interrupt','crit','fall','weight'}
 B.caps={echo={0,12},damage_dealt={-.95,63},launch_dealt={-.95,3},damage_taken={-.85,63},launch_taken={-.95,3},speed={-.8,1},jump={-.8,1},status_duration={-.95,19},sustain={0,100},conversion={0,30},momentum={0,40},clank={0,6},cleanse={0,30},curse_dealt={0,3},armor={0,12},intangible={0,3},air_jumps={-1,5},restrict={0,4},interrupt={0,2},crit={0,3},fall={-.6,1.5},weight={-.5,1.5}}
 B.families={damage_dealt='damage_dealt',launch_dealt='launch_dealt',damage_taken='damage_taken',knockback_taken='launch_taken',run_speed='speed',air_speed='speed',jump_height='jump',air_jump_height='jump',status_duration='status_duration',fall_speed='fall',weight='weight'}
 B.implicit_families={red={'damage_dealt'},green={'speed'},blue={'launch_taken'},yellow={'jump'},purple={'status_duration'},white={}}
 local function clamp(f,n) local c=assert(B.caps[f],'uncapped family');return math.max(c[1],math.min(c[2],n)) end
 local function add(t,f,n) t[f]=(t[f] or 0)+n end
 local status=D.mod_status.budget
 local function effect(e,m,tier)
  local out={};local function value(v) return S.resolve(v,m,tier) end
  if e.op=='echo' then
   local d=D.mod_echo.resolve(e,m,tier);out.echo=0;for _,c in ipairs(d.copies)do if c.echo then out.echo=out.echo+c.echo.damage*c.echo.knockback end end;out.echo=out.echo*S.copies(tier)
  elseif e.op=='value' then add(out,assert(B.families[e.key],'unbudgeted fighter value'),S.ratio(e.value,m,tier,B.families[e.key])-1)
  elseif e.op=='convert' or e.op=='versus-status' then
   for k,v in pairs(e.change) do
    if k=='element' then out.conversion=S.copies(tier)
    elseif k=='percent_damage' then add(out,e.match.incoming and 'damage_taken' or 'damage_dealt',S.ratio(v,m,tier,e.match.incoming and 'damage_taken' or 'damage_dealt')-1)
    elseif k=='launch' then add(out,e.match.incoming and 'launch_taken' or 'launch_dealt',S.ratio(v,m,tier,e.match.incoming and 'launch_taken' or 'launch_dealt')-1)
    else error('unbudgeted native field '..k) end
   end
  elseif e.op=='status' or e.op=='stacks' or e.op=='chain_status' then out=status(e.status,value(e.amount or 1),e.max);if e.op=='chain_status' then out.conversion=(out.conversion or 0)+S.copies(tier) end
  elseif e.op=='remove_status' then out=status(e.status,0,0);for k in pairs(out) do out[k]=0 end;out.cleanse=S.copies(tier)
  elseif e.op=='heal' or e.op=='damage' then out.sustain=0;for _,instance in ipairs(S.instances(tier))do out.sustain=out.sustain+math.min(100,S.resolve(e.amount,m,instance))end
  elseif e.op=='clank_damage' then out.clank=S.copies(tier)
  elseif e.op=='emit' then out.conversion=S.copies(tier)
  elseif e.op=='armor' then
   -- Protection-seconds: a type's weight times how long it stands. A permanent threshold counts as two seconds a point per ten.
   out.armor=0;for _,instance in ipairs(S.instances(tier)) do
    local v=e.value and S.resolve(e.value,m,instance) or 1;local f=e.frames and S.resolve(e.frames,m,instance)
    v=math.min(v,e.type=='hit_count' and 3 or 40);if f then f=math.min(f,e.type=='super' and 30 or 600) end
    local n
    if e.type=='super' then n=f/60 elseif e.type=='damage_threshold' then n=f and v/10*f/60*.8 or v/10*2
    elseif e.type=='hit_count' then n=v*.35 elseif e.type=='damage_pool' then n=v/30*1.5 else n=v/40 end
    out.armor=out.armor+n
   end
  elseif e.op=='intangible' then out.intangible=0;for _,instance in ipairs(S.instances(tier)) do out.intangible=out.intangible+math.min(24,S.resolve(e.frames,m,instance))/60 end
  elseif e.op=='interrupt' then out.interrupt=0;for _,instance in ipairs(S.instances(tier)) do out.interrupt=out.interrupt+math.min(20,S.resolve(e.frames,m,instance))/60 end
  elseif e.op=='air_jumps' then out.air_jumps=(S.resolve(e.count,m,tier)-1)*S.copies(tier)
  elseif e.op=='restrict' then out.restrict=#e.forbid*S.copies(tier)
  elseif e.op=='crit_next' then out.crit=0;for _,instance in ipairs(S.instances(tier)) do out.crit=out.crit+math.min(3,S.resolve(e.count,m,instance))*(math.min(4,S.resolve(e.multiplier,m,instance))-1)*.12 end
  elseif e.op=='crit' then
   -- Expected extra damage: a chance times its multiplier gain, scaled by how often it can apply. A multiplier alone is worth a
   -- fraction (it needs a chance from elsewhere); a tag, a status gate or a percent floor each narrow it.
   out.crit=0;for _,instance in ipairs(S.instances(tier)) do
    local c=e.chance and math.min(1,S.resolve(e.chance,m,instance));local mult=math.min(4,e.multiplier and S.resolve(e.multiplier,m,instance) or 1.5)
    local n
    if c then n=c*(mult-1) elseif mult>1 then n=(mult-1)*.15 else n=0 end
    if e.tag then n=n*.35 end;if e.status then n=n*.6 end;if e.min_percent and e.min_percent>0 then n=n*.6 end
    out.crit=out.crit+n
   end
  else error('unbudgeted effect') end
  return out
 end
 -- The families a record's effects touch at any tier (what its `families` declaration must say). Used by pool
 -- builders that write many records (keystones.lua) so a declaration is derived, never typed twice.
 function B.effect_families(m)
  local seen,out={},{}
  for tier=1,#m.tiers do for _,e in ipairs(m.effects) do for f in pairs(effect(e,m,tier)) do if not seen[f] then seen[f]=true;out[#out+1]=f end end end end
  table.sort(out);return out
 end
 -- A pool is data fixed at load: validating the same table against the same implicit definitions twice is pure
 -- repetition (every drive_loot.new, i.e. every candidate an opponent roll constructs, did it: ~half the
 -- instructions of a deep roll). Keyed by the pool and implicit-definition tables' identity; a failure is never remembered.
 local validated=setmetatable({},{__mode='k'})
 function B.validate_pool(pool,implicitdefs)
  local seen_defs=validated[pool]
  if seen_defs and seen_defs[implicitdefs or false] and #pool==seen_defs.n then return true end
  local ok=B.validate_pool_uncached(pool,implicitdefs)
  seen_defs=seen_defs or {n=#pool};seen_defs.n=#pool;seen_defs[implicitdefs or false]=true;validated[pool]=seen_defs
  return ok
 end
 function B.validate_pool_uncached(pool,implicitdefs)
  local ids={}
  for _,m in ipairs(pool) do
   S.validate(m);assert(not ids[m.id],'duplicate modifier');ids[m.id]=true
   assert(type(m.families)=='table','family declaration required');local declared,seen={},{}
   for _,f in ipairs(m.families) do assert(B.caps[f] and not declared[f],'unknown/duplicate family');declared[f]=true end
   for tier=1,#m.tiers do
    local totals={}
    for _,e in ipairs(m.effects) do for f,n in pairs(effect(e,m,tier)) do
     assert(declared[f],'undeclared effect family');seen[f]=true
     assert(n==n and math.abs(n)<=1e9,'individual contribution exceeds finite safety');add(totals,f,n)
    end end
    for f,n in pairs(totals) do assert(n==n and math.abs(n)<=1e9,'record contribution exceeds finite safety') end
   end
   for f in pairs(declared) do assert(seen[f],'lying family declaration') end
  end
  for colour,base in pairs(implicitdefs or {}) do
   local allowed={};for _,f in ipairs(assert(B.implicit_families[colour],'unknown implicit colour')) do allowed[f]=true end
   local seen={};for key,v in pairs(base) do local f=assert(B.families[key],'unknown implicit family');assert(allowed[f],'undeclared implicit');assert(type(v)=='number' and v==v and v-1>=B.caps[f][1] and v-1<=B.caps[f][2],'implicit over budget');seen[f]=true end
   for f in pairs(allowed) do assert(seen[f],'missing implicit family') end
  end
  return true
 end
 local function compose(pool,mods,implicits,statuses)
  local raw,potential,keys,power={},{},{},{}
  -- Held keystones must be compatible (no exclusive pair, stacked drawbacks inside the limits): one rule for the
  -- bag, the foe roll and the engine, since all of them reach the budget.
  if D.keystones then local held={};for id,t in pairs(mods or {}) do if D.keystones.meta[id] then held[#held+1]=id end end
   table.sort(held);if #held>1 then local ok,why=D.keystones.check(held);assert(ok,why or 'incompatible keystones') end end
  for k,v in pairs(implicits or {}) do local f=assert(B.families[k],'unknown implicit');assert(type(v)=='number' and v==v and math.abs(v)<100,'invalid implicit');keys[k]=v-1 end
  local found={}
  for _,m in ipairs(pool) do local tier=(mods or {})[m.id];if tier then
   S.level(tier);found[m.id]=true
   for _,e in ipairs(m.effects) do
    local contributions=effect(e,m,tier)
    for f,n in pairs(contributions) do local pf=e.subject=='target' and f=='launch_taken' and 'curse_dealt' or f;add(potential,pf,n);local incoming=f=='damage_taken' or f=='launch_taken';local score=incoming and -n or n;if e.subject=='target' and (incoming or f=='speed') then score=-score end;add(power,f,score) end
    if e.op=='value' then add(keys,e.key,S.ratio(e.value,m,tier,B.families[e.key])-1)
    elseif e.op=='versus-status' or e.op=='convert' then
     if not e.status and not e.match.incoming and (not e.match.move or e.match.move=='any') and not e.match.element and e.change.launch then add(raw,'launch_dealt',S.ratio(e.change.launch,m,tier,'launch_dealt')-1) end
    end
   end
  end end
  for id in pairs(mods or {}) do assert(found[id],'unknown modifier') end
  for name,v in pairs(statuses or {}) do
   for k,n in pairs(D.mod_status.values(name,v)) do add(keys,k,n) end
  end
  -- Multiple keys in speed/jump share caps but do not double-count one implicit.
  local familykeys={}
  for k,n in pairs(keys) do local f=B.families[k];if familykeys[f]==nil or math.abs(n)>math.abs(familykeys[f]) or (math.abs(n)==math.abs(familykeys[f]) and n>familykeys[f]) then familykeys[f]=n end end
  for f,n in pairs(familykeys) do add(raw,f,n) end
  for k,v in pairs(implicits or {}) do local f=B.families[k];if k~='air_speed' and k~='air_jump_height' then add(potential,f,v-1);add(power,f,(f=='damage_taken' or f=='launch_taken') and 1-v or v-1) end end
  if D.mod_echo then
   local d=D.mod_echo.engine({list=pool,equipped={[1]=mods or {}},status=function(_,_,name)return (statuses or {})[name]end},1);local n=0
   for _,r in ipairs(d.rules)do n=n+r.damage*r.knockback end;potential.echo=n;power.echo=n
  end
  return raw,potential,keys,power
 end
 -- Restrictions are only ever a price. At most two at once, and never the pair that leaves a fighter with no defence at all.
 function B.restrictions_ok(list)
  local n,has=0,{};for _,x in ipairs(list or {}) do n=n+1;has[x]=true end
  if n>2 then return false,'Those keystones forbid too many actions.' end
  if has.shield and has.air_dodge then return false,'Those keystones leave you with no defence.' end
  if has.run and has.air_dodge and has.specials then return false,'Those keystones forbid too many actions.' end
  return true
 end
 function B.sustain_delta(previous,delta) local cap=B.caps.sustain[2];local next=math.max(-cap,math.min(cap,(previous or 0)+delta));return next,next-(previous or 0) end
 function B.build(pool,mods,implicits,statuses)
  local raw,potential,_,scores=compose(pool,mods,implicits,statuses);local out,strength={},1
  for _,f in ipairs(B.order) do local c=B.caps[f]
   local n=raw[f] or 0;local p=potential[f] or 0;out[f]={raw=n,value=1+clamp(f,n),potential=1+clamp(f,p)}
  end
  -- A crit is an expectation, not a certainty: it counts at half weight in build strength.
  local offence=out.damage_dealt.potential*(1+(out.crit.potential-1)*.5)
  local toughness=1/out.damage_taken.potential
  -- Weight divides the launch taken (a heavier fighter flies less far); more protection and mobility add to utility.
  local launch=out.launch_dealt.potential*out.curse_dealt.potential/math.max(.05,out.launch_taken.potential/math.max(.2,out.weight.potential))
  local utility=math.max(.5,1+.025*math.max(0,out.sustain.potential-1)+.04*math.max(0,out.momentum.potential-1)+.12*math.max(0,out.conversion.potential-1)+.2*math.max(0,out.clank.potential-1)+.1*math.max(0,out.cleanse.potential-1)+.08*math.max(0,out.armor.potential-1)+.15*math.max(0,out.intangible.potential-1)+.12*math.max(0,out.interrupt.potential-1)+.03*math.max(0,out.air_jumps.potential-1)-.08*math.max(0,out.restrict.potential-1)+.06*math.max(0,1-out.fall.potential)-.02*math.max(0,out.fall.potential-1))
  strength=(1+math.max(0,out.echo.potential-1))*offence*toughness^.25*math.max(.25,launch)^.25*utility
  return out,math.max(1,strength)
 end
 function B.values(pool,mods,implicits,statuses)
  local _,_,keys=compose(pool,mods,implicits,statuses);local out={}
  for k,n in pairs(keys) do out[k]=1+clamp(B.families[k],n) end
  return out
 end
 function B.worst_case(pool,implicitdefs)
  B.validate_pool(pool,implicitdefs);local low,high={},{}
  -- Superset of all reachable distinct highest-tier IDs (including every unique and
  -- every keystone) plus four strongest implicits. This is conservative, not sampling.
  for _,m in ipairs(pool) do
   local lo,hi={},{}
   for tier=1,#m.tiers do local sums={};for _,e in ipairs(m.effects) do for f,n in pairs(effect(e,m,tier)) do add(sums,f,n) end end
    for f,n in pairs(sums) do lo[f]=math.min(lo[f] or 0,n);hi[f]=math.max(hi[f] or 0,n) end
   end
   for f,n in pairs(lo) do add(low,f,n) end;for f,n in pairs(hi) do add(high,f,n) end
  end
  for f in pairs(B.caps) do local lo,hi=0,0
   for _,base in pairs(implicitdefs or {}) do for k,v in pairs(base) do if B.families[k]==f then lo=math.min(lo,v-1);hi=math.max(hi,v-1) end end end
   add(low,f,4*lo);add(high,f,4*hi)
  end
  local result={};for f in pairs(B.caps) do result[f]={raw_min=low[f] or 0,raw_max=high[f] or 0,min=1+clamp(f,low[f] or 0),max=1+clamp(f,high[f] or 0)} end
  return result
 end
 return B
end
