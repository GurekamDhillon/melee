-- Independent seeded same-pool construction. Only scalar strength/context/role enter.
return function(D)
 local P=D.mod_progression;local R={minimum=1,maximum=1e12,band={.8,1.2},tolerance=.2,candidates=128};R.__index=R
 local function integer(n) return type(n)=='number' and n==n and n%1==0 and n>=0 and n<=2147483646 end
 local function random(seed)
  local state=seed%2147483646+1
  return function(n)state=state*16807%2147483647;return (state-1)/2147483646*n end
 end
 function R.new(pool)
  local self=setmetatable({pool=pool,loot=D.drive_loot.new(pool),keys={},probe=D.mod_engine.new(1,pool)},R)
  for _,m in ipairs(pool)do if m.kind=='keystone' then self.keys[#self.keys+1]=m.id end end
  return self
 end
 function R:weights(strength)
  local high=math.min(12,math.max(0,math.log(strength)))
  local w={armoured=1+3*high,cleansing=1+2*high,bastion=1+high,shelter=1+high,reprisal=1+high,renewal=1+high,pyre=1/(1+high)^3}
  -- A technique record whose trigger nobody performs for the opponent is inert on it: weight it down (it stays rollable, a deliberate
  -- dead roll like the rest of the pool). 'driven' triggers (foe_driver.lua performs them) and 'maybe' ones are left at full weight.
  if D.mod_skill then for _,m in ipairs(self.pool) do if m.min_depth and D.mod_skill.cpu(m.trigger)=='dead' then w[m.id]=.25 end end end
  return w
 end
 function R:construct(rand,context,strength,full,relaxed,cap,maxed)
  local count=full and P.slots(context) or math.floor(rand(P.slots(context)+1))
  if cap then count=math.min(count,math.max(1,cap.drives)) end
  local b={items={},equipped={},keystones={},context=P.context(context)}
  local h=math.max(0,math.log(strength));local loot=D.drive_loot.new(self.pool,{affix_weights=self:weights(strength),unique_weights={glass_core=1+2*h,storm_shell=1+h,mirror_shard=1/(1+h)}})
  -- Opponents follow the player's curve: the best rarity the depth has reached (a unique only once uniques roll),
  -- and the drive's affix count is held down by the depth band exactly as for the player's drives.
  -- `maxed` (the max build): a unique where uniques roll, half the time, else the best rarity the depth allows.
  local function rarity(c) if maxed then return P.rarity_allowed(c,'unique') and rand(2)<1 and 'unique' or P.rarity_allowed(c,'rare') and 'rare' or P.rarity_allowed(c,'magic') and 'magic' or 'common' end;if relaxed then return rand(5)<1 and 'unique' or 'rare' end;local top=P.rarity_allowed(c,'unique') and rand(5)<1 and 'unique' or P.rarity_allowed(c,'rare') and 'rare' or P.rarity_allowed(c,'magic') and 'magic' or 'common';return top end
  for slot=1,count do b.equipped[slot]=loot:roll(math.floor(rand(2147483646)),context,rarity(P.context(context)))end
  local keys={};for _,id in ipairs(self.keys)do keys[#keys+1]=id end
  local keycount=full and P.keystones(context) or math.floor(rand(P.keystones(context)+1))
  if cap then keycount=math.min(keycount,math.max(1,cap.keystones)) end
  -- Held keystones follow the allowance and the same exclusion/drawback rules as the player's (keystones.lua).
  while #b.keystones<keycount and #keys>0 do
   local id=table.remove(keys,math.floor(rand(#keys))+1)
   local trial={};for _,k in ipairs(b.keystones)do trial[#trial+1]=k end;trial[#trial+1]=id
   if not D.keystones or D.keystones.check(trial) then b.keystones=trial end
  end
  if #b.keystones==1 then b.keystone=b.keystones[1]end
  return b
 end
 function R:evaluate(build,port)
  local mods,implicits=D.drive_bag.new(self.loot,{context=build.context}):validate(build)
  self.probe:clear();self.probe:set_context(build.context);self.probe:set_build(port or 2,mods,implicits)
  local families,strength=D.mod_budget.build(self.pool,mods,implicits,{})
  return strength,families,mods,implicits
 end
 function R:sample(seed,context)
  assert(integer(seed),'invalid sample seed');context=P.context(context)
  local build=self:construct(random(seed),context,1,true)
  local strength,families=self:evaluate(build)
  return {seed=seed,context=context,build=build,strength=strength,families=families}
 end
 function R:exchange(player,foe)
  local function trial(attacker,defender)
   local engine=D.mod_engine.new(1,self.pool,{context=player.context});local players={[1]={percent=0,grounded=true,stocks=1},[2]={percent=0,grounded=true,stocks=1}}
   for port,build in ipairs({player,foe})do local mods,imp=D.drive_bag.new(self.loot,{context=build.context}):validate(build);engine:set_build(port,mods,imp)end
   local previous={}
   local function damage()
    for p=1,2 do local n=engine.damage[p] or 0;players[p].percent=math.max(0,math.min(999,players[p].percent+n-(previous[p] or 0)));previous[p]=n end
   end
   engine:begin_frame(players);for p=1,2 do engine:emit{kind='ledge_grab',port=p,tags={grounded=true}}end;engine:drain();damage()
   local ratio
   for hits=1,1000 do
    -- Fixed scenario: initial ledge setup, then one grounded smash per60 frames.
    -- Real event drain handles Guarded/Haste conversions, DOT, healing and cleanses.
    for _=1,60 do previous={};engine:begin_frame(players);engine:drain();damage()end
    ratio=engine:contact_ratios(attacker,defender)
    local after=players[defender].percent+18
    local kb=((after/10+after*18/20)*1.4+18)*.95+25
    if kb*ratio.launch>=196.8699 then return hits,ratio end
    if (((1017/10+1017*18/20)*1.4+18)*.95+25)*ratio.launch<196.8699 and hits>20 then return nil,ratio end
    players[defender].percent=math.min(999,players[defender].percent+18*ratio.percent_damage)
    local tags={smash=true,grounded=true};tags[ratio.element]=true
    engine:emit{kind='hit_dealt',port=attacker,target=defender,tags=tags};engine:emit{kind='hit_taken',port=defender,target=attacker,tags=tags};engine:drain();damage()
   end
   return nil,ratio
  end
  local a,ra=trial(1,2);local b,rb=trial(2,1);return a,b,ra,rb
 end
 function R:validate(r)
  assert(type(r)=='table' and not getmetatable(r),'plain foe record required')
  for k in pairs(r)do assert(({capped=true,seed=true,stage=true,port=true,requested=true,target=true,strength=true,build=true,context=true,role=true})[k],'unknown foe record field')end
  assert(integer(r.seed) and integer(r.stage) and integer(r.port) and r.port>=1 and r.port<=6,'invalid foe identity')
  assert(type(r.requested)=='number' and r.requested==r.requested and r.requested>=1 and r.requested<=self.maximum,'invalid requested strength')
  local c=P.context(r.context);local bc=P.context(r.build.context);assert(c.depth==bc.depth and c.loop==bc.loop,'foe build context mismatch');local target=r.requested*P.factor(c,r.role)
  assert(r.target==target,'invalid difficulty target')
  local strength,_,mods,implicits=self:evaluate(r.build,r.port)
  assert(type(r.strength)=='number' and math.abs(strength-r.strength)<1e-9,'invalid foe strength')
  assert(strength<=target*self.band[2] and (r.capped or strength>=target*self.band[1]),'opponent outside tracking band')
  return mods,implicits
 end
 -- A roll is a bounded search over candidate builds. roll_job/roll_step expose it in slices so a run can spend
 -- a few attempts per script call (one call is limited to 2M instructions or 50 ms); roll() runs them all.
 -- `held` ({drives=,keystones=}: what the player holds) caps an EARLY opponent (effective depth below 10, the affix bands that
 -- keep early play tame) at the player's own counts: it reaches the target through tier or settles for a lower strength.
 function R:roll_job(strength,seed,stage,port,context,role,held)
  assert(strength=='max' or type(strength)=='number' and strength==strength and strength>=self.minimum and strength<=self.maximum,'strength request must be max or finite 1..1e12')
  assert(integer(seed),'seed must be a whole number from 0 to 2147483646 (the roll folds larger ones: use the console)')
  assert(integer(stage) and integer(port) and port>=1 and port<=6,'invalid foe stage/port (port 1..6)')
  context=P.context(context)
  if strength=='max' then -- the highest bounded build of this context: best of `max_tries` full candidates (R:max_step)
   P.factor(context,role)
   return {max=true,tries=self.max_tries,seed=seed,stage=stage,port=port,context=context,role=role,rand=random((seed+stage*104729+port*8191)%2147483646),attempt=0}
  end
  local target=strength*P.factor(context,role)
  local cap=held and P.effective(context)<10 and {drives=held.drives or 1,keystones=held.keystones or 1} or nil
  return {cap=cap,strength=strength,seed=seed,stage=stage,port=port,context=context,role=role,target=target,rand=random((seed+stage*104729+port*8191)%2147483646),attempt=0}
 end
 -- Runs up to `attempts` candidates; returns the finished record once the search and its validation are done.
 function R:roll_step(job,attempts)
  if job.max then return self:max_step(job,attempts) end
  local strength,seed,stage,port,context,role,target,rand=job.strength,job.seed,job.stage,job.port,job.context,job.role,job.target,job.rand
  local best,distance=job.best,job.distance
  local top=job.relaxed and job.top2 or job.top1 or self.candidates -- a one-call roll splits its few tries over both passes (R:run)
  while job.attempt<=top and attempts>0 and not job.found do
   local attempt=job.attempt;job.attempt=attempt+1;attempts=attempts-1
   local candidate=P.context(context)
   if attempt>0 then
    -- A deeper roll buys tier, but never leaves the context's affix-count band (a depth-0 opponent stays simple).
    local room=job.relaxed and 15 or math.min(15,P.band_top(context)-P.effective(context))
    candidate.depth=math.min(2147483646,candidate.depth+math.min(room,math.floor(rand(16))))
   end
   local build=attempt==0 and {items={},equipped={},keystones={},context=context} or self:construct(rand,candidate,target,false,job.relaxed,job.cap)
   build.context=context
   for slot in pairs(build.equipped)do if slot>P.slots(context)then build.equipped[slot]=nil end end
   while #build.keystones>P.keystones(context)do table.remove(build.keystones)end
   build.keystone=#build.keystones==1 and build.keystones[1] or nil
   local ok,power,families=pcall(self.evaluate,self,build,port)
   if ok then local diff=math.abs(power-target)
    local quality=target>10 and (math.max(0,families.launch_dealt.potential-1.6)+math.max(0,.5*math.sqrt(target)-families.damage_dealt.value)) or 0
    diff=diff+target*.2*quality
    if power<target*self.band[1] or power>target*self.band[2]then diff=diff+target*100 end
    if not distance or diff<distance then best={seed=seed,stage=stage,port=port,requested=strength,target=target,strength=power,build=build,context=context,role=role,capped=job.cap and true or nil};distance=diff end
    if diff<=target*.04 then job.found=true end
   end
   job.best,job.distance=best,distance
  end
  if job.attempt<=top and not job.found then return nil end
  -- The curve bounds ordinary play. A strength it cannot reach (a LAB build far above the depth) gets a second
  -- pass with the old unbanded rolls instead of a refusal; a normal run never reaches this.
  if not job.found and not job.relaxed and not job.cap and (not best or best.strength<target*self.band[1] or best.strength>target*self.band[2]) then
   job.relaxed=true;job.attempt=1;job.top2=job.top2 or self.candidates;return nil
  end
  assert(best,'no valid independent opponent build')
  -- Both passes spent and the band still missed: the best-effort record (logged), not a refusal.
  if not job.found and not job.cap and best.strength<target*self.band[1] then return self:settle(job) end
  -- Every candidate over the band (a small request at a depth whose cheapest rolled build is already above it): restated, never refused.
  if best.strength>target*self.band[2] then return self:settle(job) end
  assert(job.cap or best.strength>=target*self.band[1] or best.capped,'requested strength unreachable by bounded same-pool search')
  self:validate(best);return best
 end
 -- One script call is limited to 2M instructions or 500 ms and a candidate build costs up to ~45k, so a roll made in ONE call (the console's `foe roll`) gets `sync_attempts` candidates in all, across both passes. The old roll() ran the whole 128 + 128 search, which blew the budget for
 -- seeds whose target was not met early (the depth 12 / loop 3 normal roll of seed 2144865533, 2026-10-06). Out of tries it settles for the best
 -- candidate so far: its strength is within the upper band always (the empty build is candidate 0), and below the lower band the record is marked
 -- `capped` (the same marking a held-down early opponent gets), so it validates. R.log (set by the host) says when it fell back.
 R.sync_attempts=32;R.hopeless=.5;R.impossible=1e6
 function R:settle(job)
  local best=assert(job.best,'no valid independent opponent build')
  -- A request the pool cannot come near (a strength above what the context's full builds reach): the best candidate found is restated at
  -- its own strength, so the roll ends at the ceiling instead of refusing. (max_step does the same on purpose.)
  -- (A request beyond `impossible` is nonsense, not a ceiling, and stays refused.)
  assert(job.target<=self.impossible or best.strength>=job.target*self.hopeless,'requested strength unreachable by bounded same-pool search')
  if best.strength<job.target*self.hopeless or best.strength>job.target*self.band[2] then
   best=self:restate(best)
   if self.log then self.log(('foe roll: strength %.2f is out of reach at depth %d / loop %d: built the nearest candidate, strength %.2f'):format(job.strength,job.context.depth,job.context.loop,best.strength)) end
   self:validate(best);return best
  end
  if best.strength<job.target*self.band[1] then best.capped=true end
  if self.log then self.log(('foe roll: fell back to the best of %d tries (%s / strength %.2f for target %.2f%s)'):format(job.attempt,job.role or 'normal',best.strength,job.target,best.capped and ', below the band' or '')) end
  self:validate(best);return best
 end
 -- A candidate's record restated as a request for its own strength (the max build, and a request beyond the ceiling).
 function R:restate(best)
  local factor=P.factor(best.context,best.role);local requested=math.max(1,math.min(self.maximum,best.strength/factor));local target=requested*factor
  return {seed=best.seed,stage=best.stage,port=best.port,requested=requested,target=target,strength=best.strength,build=best.build,context=best.context,role=best.role,capped=best.strength<target*self.band[1] and true or nil}
 end
 -- The max build: every slot, the whole keystone allowance the pool's rules permit, drives rolled at the deepest tier the context's affix band
 -- allows (the roll's own `room`), best of `max_tries` candidates by strength. Each candidate is one full build (~65k instructions), so a one-call roll
 -- and a sliced one (the host's few per frame) both fit a script call.
 R.max_tries=8
 function R:max_step(job,attempts)
  local context=job.context
  while job.attempt<job.tries and attempts>0 do
   job.attempt=job.attempt+1;attempts=attempts-1
   local candidate=P.context(context);candidate.depth=math.min(2147483646,candidate.depth+math.min(15,P.band_top(context)-P.effective(context)))
   local build=self:construct(job.rand,candidate,2,true,false,nil,true);build.context=context
   for slot in pairs(build.equipped)do if slot>P.slots(context)then build.equipped[slot]=nil end end -- the deeper tier never buys a slot or a keystone
   while #build.keystones>P.keystones(context)do table.remove(build.keystones)end
   build.keystone=#build.keystones==1 and build.keystones[1] or nil
   local ok,power=pcall(self.evaluate,self,build,job.port)
   if not ok then job.why=power end
   if ok and (not job.power or power>job.power) then job.power,job.build=power,build end
  end
  if job.attempt<job.tries then return nil end
  assert(job.build,'no valid independent opponent build: '..tostring(job.why))
  local r=self:restate({seed=job.seed,stage=job.stage,port=job.port,strength=job.power,build=job.build,context=context,role=job.role})
  self:validate(r);return r
 end
 function R:run(job,limit)
  local r
  if job.max then repeat r=self:roll_step(job,1) until r;return r end
  limit=limit or self.sync_attempts
  if limit<2*self.candidates then job.top1=limit//4;job.top2=limit-limit//4 end
  for _=1,limit+1 do r=self:roll_step(job,1);if r then return r end end -- +1: the switch to the relaxed pass takes a call but no try
  return self:settle(job)
 end
 -- roll() is the full search (at most 2 x `candidates` tries, then the logged best-effort); the console passes `tries` = R.sync_attempts so that
 -- the whole roll fits one script call.
 -- A candidate costs more the stronger the request (a full build of many rules): above `sync_over` a roll limited to one call (the console passes `tries`)
 -- gets `sync_high` tries in all, so no strength can blow the script call (it settles for the best of them, logged); a caller that wants the whole
 -- search slices it (roll_job/roll_step: `envoy vs`, `foe sliced on`) or passes no `tries`.
 R.sync_over=10;R.sync_high=8
 function R:roll(strength,seed,stage,port,context,role,held,tries)
  if tries and type(strength)=='number' and strength>self.sync_over then tries=math.min(tries,self.sync_high) end
  tries=tries or 2*self.candidates+2
  return self:run(self:roll_job(strength,seed,stage,port,context,role,held),tries)
 end
 return R
end
