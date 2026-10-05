-- One authority: additive deltas, numerical safety families, bounded utility, conservative pool audit.
return function(D)
 local S=D.mod_schema;local B={}
 -- Every registered effect needs a budget answer; a field the schema admits cannot be silently unbudgeted.
 B.effect_ops={echo=true,value=true,convert=true,['versus-status']=true,status=true,stacks=true,remove_status=true,heal=true,damage=true,clank_damage=true,emit=true}
 for op in pairs(D.mod_registry.effects) do assert(B.effect_ops[op],'registered effect has no budget rule: '..op) end
 for op in pairs(B.effect_ops) do assert(D.mod_registry.effects[op],'budget rule for an unregistered effect: '..op) end
 B.order={'echo','damage_dealt','launch_dealt','damage_taken','launch_taken','speed','jump','status_duration','sustain','conversion','momentum','clank','cleanse','curse_dealt'}
 B.caps={echo={0,12},damage_dealt={-.95,63},launch_dealt={-.95,3},damage_taken={-.85,63},launch_taken={-.95,3},speed={-.8,1},jump={-.8,1},status_duration={-.95,19},sustain={0,100},conversion={0,30},momentum={0,40},clank={0,6},cleanse={0,30},curse_dealt={0,3}}
 B.families={damage_dealt='damage_dealt',launch_dealt='launch_dealt',damage_taken='damage_taken',knockback_taken='launch_taken',run_speed='speed',air_speed='speed',jump_height='jump',air_jump_height='jump',status_duration='status_duration'}
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
  elseif e.op=='status' or e.op=='stacks' then out=status(e.status,value(e.amount or 1),e.max)
  elseif e.op=='remove_status' then out=status(e.status,0,0);for k in pairs(out) do out[k]=0 end;out.cleanse=S.copies(tier)
  elseif e.op=='heal' or e.op=='damage' then out.sustain=0;for _,instance in ipairs(S.instances(tier))do out.sustain=out.sustain+math.min(100,S.resolve(e.amount,m,instance))end
  elseif e.op=='clank_damage' then out.clank=S.copies(tier)
  elseif e.op=='emit' then out.conversion=S.copies(tier) else error('unbudgeted effect') end
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
 function B.sustain_delta(previous,delta) local cap=B.caps.sustain[2];local next=math.max(-cap,math.min(cap,(previous or 0)+delta));return next,next-(previous or 0) end
 function B.build(pool,mods,implicits,statuses)
  local raw,potential,_,scores=compose(pool,mods,implicits,statuses);local out,strength={},1
  for _,f in ipairs(B.order) do local c=B.caps[f]
   local n=raw[f] or 0;local p=potential[f] or 0;out[f]={raw=n,value=1+clamp(f,n),potential=1+clamp(f,p)}
  end
  local offence=out.damage_dealt.potential
  local toughness=1/out.damage_taken.potential
  local launch=out.launch_dealt.potential*out.curse_dealt.potential/math.max(.05,out.launch_taken.potential)
  local utility=1+.025*math.max(0,out.sustain.potential-1)+.04*math.max(0,out.momentum.potential-1)+.12*math.max(0,out.conversion.potential-1)+.2*math.max(0,out.clank.potential-1)+.1*math.max(0,out.cleanse.potential-1)
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
