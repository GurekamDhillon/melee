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
 -- STRENGTH v2 (the synergy-aware number; `mod_tuning` strength=v2, the default; v1 is the old additive one, kept for comparison).
 -- A record counts for what it actually does given what the build feeds it: a reader counts 0.1 + 0.9 x (what the OTHER held records provide of what
 -- it reads, from the derived graph, mod_graph); a status a trigger grants counts for rate x duration of its trigger, not for ever; Echo is capped to
 -- what it measures; speed counts a little; cleansing counts less. Constants are chosen from the synergy analysis (measured rates), not tuned by play.
 B.rates=B.rates or {hit_dealt=.8,landing=.45,hit_taken=.45,jump=.09,air_jump=.035,ko_dealt=.018,clank=.02,ledge_grab=.006,perfect_shield=.018,interval=1,status_applied=.3,crit=.05,combo=.25,combo_end=.05,lcancel_hit=.002,wavedash=.005,tech=.01,air_dodge=.02,armor=.02}
 B.crit_w=B.crit_w or 1;B.skill=B.skill or .6;B.speed_w=B.speed_w or .25;B.echo_cap=B.echo_cap or .6;B.cleanse_w=B.cleanse_w or .03
 B.human=B.human or {lcancel_hit=.15,wavedash=.08,perfect_shield=.05,tech=.05,air_dodge=.05}   -- placeholder technique rates a person performs, x skill; needs the owner's counters
 function B.mode() if D.mod_tuning and D.mod_tuning.get('strength')=='v1' then return 'v1' end;return D.mod_graph and 'v2' or 'v1' end
 function B.uptime(m,duration)
  local t=m.trigger;if t=='interval' then return 1 end;if t=='status_applied' then return .5 end
  local r=B.rates[t] or .3;if B.human[t] then r=B.human[t]*B.skill end
  return math.max(.05,math.min(1,r*(duration or 60)/60))
 end
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
  elseif e.op=='status' or e.op=='stacks' or e.op=='chain_status' then out=status(e.status,value(e.amount or 1)*(D.mod_tuning and D.mod_tuning.amount_scale(m,e) or 1),e.max);if e.op=='chain_status' then out.conversion=(out.conversion or 0)+S.copies(tier) end
   if B.mode()=='v2' then local u=B.uptime(m,value(e.duration or 60));for k,v in pairs(out) do out[k]=v*u end end
  elseif e.op=='remove_status' then out=status(e.status,0,0);for k in pairs(out) do out[k]=0 end;out.cleanse=e.count and 0 or S.copies(tier)   -- spending a stack of your own buff (count) is not cleansing
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
    -- v2: the engine draws a crit uniformly up to multiplier_max, so its mean is the midpoint (Gambler measured x1.25, v1 said x0.94)
    if e.multiplier_max and B.mode()=='v2' then mult=(mult+math.min(4,S.resolve(e.multiplier_max,m,instance)))/2 end
    local n
    if c then n=c*(mult-1) elseif mult>1 then n=(mult-1)*.15 else n=0 end
    if e.tag then n=n*.35 end;if e.status then n=n*.6 end;if e.min_percent and e.min_percent>0 then n=n*(B.mode()=='v2' and .4 or .6) end   -- v2: a target above 100% is a minority of a fight
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
  local v2=B.mode()=='v2';local counts
  if v2 then counts={};for _,m in ipairs(pool) do if (mods or {})[m.id] then for k in pairs(D.mod_graph.profile(m).gives) do counts[k]=(counts[k] or 0)+1 end end end end
  for _,m in ipairs(pool) do local tier=(mods or {})[m.id];if tier then
   S.level(tier);found[m.id]=true
   local provided
   if v2 then local own=D.mod_graph.profile(m).gives
    provided=setmetatable({},{__index=function(_,k) local n=counts[k] or 0;if own[k] then n=n-1 end;return n>0 end}) end
   for i,e in ipairs(m.effects) do
    local contributions=effect(e,m,tier)
    if v2 then local sup=D.mod_graph.supply(m,i,provided,m.trigger~='equip')
     if sup then local gw=.1+.9*sup;for f,n in pairs(contributions) do contributions[f]=n*gw end end end
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
  if B.mode()=='v2' then
   local cw=B.cleanse_w-.1  -- v1 weights cleansing .1 a point; v2 .03
   utility=math.max(.5,utility+cw*math.max(0,out.cleanse.potential-1))
   offence=out.damage_dealt.potential*(1+(out.crit.potential-1)*B.crit_w)   -- v1 counted a crit at half weight; measured, an expectation counts in full
   local echoterm=1+math.min(B.echo_cap,math.max(0,out.echo.potential-1));local speedterm=1+B.speed_w*math.max(0,out.speed.potential-1)
   strength=echoterm*offence*toughness^.25*math.max(.25,launch)^.25*utility*speedterm
  else strength=(1+math.max(0,out.echo.potential-1))*offence*toughness^.25*math.max(.25,launch)^.25*utility end
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
 -- ---- content memo ----------------------------------------------------------------------------------------------------------------
 -- build/values are pure functions of (pool, mods, implicits, statuses, strength mode, tuning revision). Probe engines, the bag's draft views,
 -- the foe roller and the publication check all ask the same question many times a frame at deep loops (the campaigns hit the 2 M instruction
 -- budget there); one shared, bounded cache answers repeats, and keeps the work an interrupted publication already did, so a retry makes
 -- progress. Results are shared tables: callers read them, none writes them.
 local pool_ids=setmetatable({},{__mode='k'});local npool=0
 local cache,cache_n={},0
 local function tier_text(t)
  if type(t)~='table' then return tostring(t) end
  local o={};if t.tier then o[#o+1]='t'..tostring(t.tier) end;if t.copies then o[#o+1]='c'..tostring(t.copies) end
  if t.tiers then o[#o+1]='['..table.concat(t.tiers,',')..']' end;return table.concat(o,'/')
 end
 local function content_key(kind,pool,mods,implicits,statuses)
  local id=pool_ids[pool];if not id then npool=npool+1;id=npool;pool_ids[pool]=id end
  local parts,n={},0
  for mid,t in pairs(mods or {}) do n=n+1;parts[n]=mid..'='..tier_text(t) end;table.sort(parts)
  local out={kind,id,B.mode(),D.mod_tuning and D.mod_tuning.rev or 0,table.concat(parts,';')}
  local im={};for k,v in pairs(implicits or {}) do im[#im+1]=k..'='..tostring(v) end;table.sort(im);out[#out+1]=table.concat(im,';')
  local st={};for name,v in pairs(statuses or {}) do st[#st+1]=name..':'..tostring(type(v)=='table' and v.stacks)..':'..tostring(type(v)=='table' and v.amount) end;table.sort(st);out[#out+1]=table.concat(st,';')
  return table.concat(out,'|')
 end
 local function remember(key,a,b)
  if cache_n>=600 then cache,cache_n={},0 end
  cache[key]={a,b};cache_n=cache_n+1
 end
 local raw_build,raw_values=B.build,B.values
 B.cache_stats={hits=0,misses=0}
 function B.build(pool,mods,implicits,statuses)
  local key=content_key('b',pool,mods,implicits,statuses);local hit=cache[key]
  if hit then B.cache_stats.hits=B.cache_stats.hits+1;return hit[1],hit[2] end
  B.cache_stats.misses=B.cache_stats.misses+1
  local a,b=raw_build(pool,mods,implicits,statuses);remember(key,a,b);return a,b
 end
 function B.values(pool,mods,implicits,statuses)
  local key=content_key('v',pool,mods,implicits,statuses);local hit=cache[key]
  if hit then B.cache_stats.hits=B.cache_stats.hits+1;local out={};for k,v in pairs(hit[1]) do out[k]=v end;return out end -- callers edit their copy
  B.cache_stats.misses=B.cache_stats.misses+1
  local a=raw_values(pool,mods,implicits,statuses);remember(key,a);local out={};for k,v in pairs(a) do out[k]=v end;return out
 end
 function B.cache_clear() cache,cache_n={},0 end
 return B
end
