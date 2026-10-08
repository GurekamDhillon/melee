-- Pure bounded rule evaluation. Native adapter owns checkpointed gameplay writes.
return function(D)
 local S,C=D.mod_schema,D.mod_codec;local E={};E.__index=E
 local ordered,status_tags=D.mod_status.order,D.mod_status.tags
 local function copy(v) return C.decode(C.encode(v)) end
 local function append(out,text) for _,v in ipairs(out) do if v==text then return end end;if #out<24 then out[#out+1]=text end end
 function E.new(seed,pool,limits)
  seed=seed or 1;limits=limits or {};assert(type(seed)=='number' and seed%1==0 and seed>=1 and seed<2147483647,'invalid seed')
  local budget,depth=limits.budget or 64,limits.depth or 8
  assert(type(budget)=='number' and budget%1==0 and budget>=1 and budget<=128 and type(depth)=='number' and depth%1==0 and depth>=1 and depth<=16,'invalid limits')
  -- Validating a pool is expensive and a pool never changes after load. The bag screen builds a draft
  -- engine several times per drawn frame; revalidating each time blew the per-call script budget and ended a run.
  D._pool_checked=D._pool_checked or setmetatable({},{__mode='k'})
  local checked=D._pool_checked[pool]
  if not checked then
   D.mod_budget.validate_pool(pool,{});local r,l={},{};for _,m in ipairs(pool) do S.validate(m);assert(not r[m.id],'duplicate modifier');r[m.id]=m;l[#l+1]=m end
   table.sort(l,function(a,b) return a.id<b.id end)
   checked={rules=r,list=l};D._pool_checked[pool]=checked
  end
  local rules,list={},{};for id,m in pairs(checked.rules) do rules[id]=m end;for i,m in ipairs(checked.list) do list[i]=m end
  checked.tag=checked.tag or (function() D._pool_tags=(D._pool_tags or 0)+1;return D._pool_tags end)()
  D._pool_checked[list]=checked -- an engine's own list is a validated pool too (import() builds its probe from it)
  return setmetatable({pool_tag=checked.tag,rules=rules,list=list,seed=seed or 1,frame=0,equipped={},implicits={},statuses={},recent={},players={},queue={},damage={},trace={},used=0,fx={},dropped=0,memo={},
   context=D.mod_progression.context(limits.context),limit=budget,depth=depth,display={last_pulse=-30,pulse_start=-100,pulse_strength=0,trace_key='',intensity=.65}},E)
 end
 function E:random() self.seed=self.seed*48271%2147483647;return self.seed/2147483647 end
 function E:equip(port,id,tier)
  assert(type(port)=='number' and port%1==0 and port>=1 and port<=6,'port 1..6 required')
  local m=assert(self.rules[id],'unknown modifier');tier=tier or 1;S.level(tier)
  local at=copy(self.equipped[port] or {});if m.kind=='keystone' then for other in pairs(at) do if self.rules[other].kind=='keystone' and other~=id then assert(D.mod_progression.keystones(self.context)>1,'one keystone at depth zero')end end end
  at[id]=tier;self:set_build(port,at,self.implicits[port])
 end
 function E:set_build(port,mods,implicits)
  assert(type(port)=='number' and port%1==0 and port>=1 and port<=6,'port 1..6 required')
  local checked,base,keys={},{},0
  for id,tier in pairs(mods or {}) do local m=assert(self.rules[id],'unknown modifier');S.level(tier);assert(m.kind=='unique' or S.copies(tier)==1,'only physical uniques stack copies');checked[id]=copy(tier);if m.kind=='keystone' then keys=keys+1 end end
  assert(keys<=D.mod_progression.keystones(self.context),'keystone allowance exceeded')
  for key,value in pairs(implicits or {}) do assert(S.values[key] and type(value)=='number' and value==value and value>=-1000000 and value<=1000000,'invalid implicit');base[key]=value end
  local old,oldbase=self.equipped[port],self.implicits[port];self.equipped[port],self.implicits[port]=checked,base;local ok,err=pcall(self.native_rules,self,port);self.equipped[port],self.implicits[port]=old,oldbase;assert(ok,err)
  self.equipped[port]=checked;self.implicits[port]=base
 end
 function E:status(port,name) return (self.statuses[port] or {})[name] end
 -- Momentum is a COUNTER, not a status look: how many stacks (0..5) a fighter holds right now. The later visual pass draws it as orbs
 -- from this one accessor (a pure read of the checkpointed status table; the stack cap is the status record's own `max`).
 function E:momentum(port) local v=(self.statuses[port] or {}).momentum;return v and v.stacks or 0 end
 function E:momentum_max(port) local v=(self.statuses[port] or {}).momentum;return v and v.max or 5 end
 function E:clear(port)
  if port then self.equipped[port]=nil;self.implicits[port]=nil;self.statuses[port]=nil;self.recent[port]=nil;self.damage[port]=nil
  else self.equipped={};self.implicits={};self.statuses={};self.recent={};self.damage={};self.queue={};self.trace={} end
 end
 function E:emit(e)
  assert(S.events[e.kind],'unsupported modifier event')
  e=copy(e);e.depth=e.depth or 1;e.tags=e.tags or {};e.origin=e.origin or {}
  assert(type(e.port)=='number' and e.port%1==0 and e.port>=1 and e.port<=6,'invalid event port')
  if e.target then assert(type(e.target)=='number' and e.target%1==0 and e.target>=1 and e.target<=6,'invalid event target') end
  for _,key in ipairs({'damage_a','damage_b'}) do local v=e[key];if v~=nil then assert(type(v)=='number' and v==v and v>=0 and v<=100000,'invalid clank damage') end end
  for tag,v in pairs(e.tags) do assert(S.tags[tag] and type(v)=='boolean','unsupported event tag') end
  assert(type(e.depth)=='number' and e.depth%1==0 and e.depth>=1,'invalid event depth')
  if e.depth>self.depth or #self.queue>=128 then self.dropped=self.dropped+1;return false end
  self.queue[#self.queue+1]=e;return true
 end
 -- Events the engine itself builds (fresh tables, known-valid kind and port) skip the codec copy and the checks.
 function E:emit_trusted(e)
  if #self.queue>=128 then self.dropped=self.dropped+1;return false end
  self.queue[#self.queue+1]=e;return true
 end
 function E:begin_frame(players)
  self.frame=self.frame+1;local own={};for port,v in pairs(players or {}) do local t={};for k,x in pairs(v) do t[k]=x end;own[port]=t end;self.players=own;self.damage={};self.sustain={};self.used=0;self.fx={}
  for port=1,6 do local statuses=self.statuses[port]
   if statuses then for _,name in ipairs(ordered) do local v=statuses[name]
    if v then
     if name=='burn' and v.next_tick<=self.frame then
      self:sustain_damage(port,v.amount*v.stacks);v.next_tick=v.next_tick+60
     end
     if v.expires<=self.frame then
      statuses[name]=nil;self:emit{kind='status_removed',port=port,status=name,tags={},origin=v.origin}
     end
    end
   end end
  end
  for port=1,6 do if self.players[port] then self:emit_trusted{kind='interval',port=port,tags={},depth=1,origin={}} end end
 end
 function E:matches(m,e,tier)
  local main=m.trigger==e.kind
  if not main then local any;for _,a in ipairs(m.also or {}) do if a.trigger==e.kind then any=true;break end end;if not any then return false end end
  if main and m.trigger=='interval' and self.frame%S.resolve(m.interval,m,tier)~=0 then return false end
  if main and self:conditions_hold(m,e,tier,m.conditions) then return true end
  -- A record can carry alternative triggers (`also`): any one whose own conditions hold fires the same effects.
  for _,a in ipairs(m.also or {}) do if a.trigger==e.kind and self:conditions_hold(m,e,tier,a.conditions) then return true end end
  return false
 end
 function E:conditions_hold(m,e,tier,conditions)
  local own=e.self_context or self.players[e.port] or {};local target=e.target_context or self.players[e.target] or {}
  for _,c in ipairs(conditions or {}) do for k,v in pairs(c) do
   if type(v)=='string' and v:sub(1,1)=='$' then v=S.resolve(v,m,tier) end
   if k=='tag' and not e.tags[v] then return false
   elseif k=='status' and (v=='any' and not e.status or v~='any' and e.status~=v) then return false
   elseif k=='self_status' or k=='target_status' then
    local at=self.statuses[k=='self_status' and e.port or e.target] or {}
    if v=='any' then if next(at)==nil then return false end elseif not at[v] then return false end
   elseif k=='self_damage_above' and (own.percent==nil or own.percent<=v) then return false
   elseif k=='self_damage_below' and (own.percent==nil or own.percent>=v) then return false
   elseif k=='target_damage_above' and (target.percent==nil or target.percent<=v) then return false
   elseif k=='grounded' and (own.grounded==nil or (own.grounded==true)~=v) then return false
   elseif k=='airborne' and (own.grounded==nil or (own.grounded==false)~=v) then return false
   elseif k=='last_stock' and ((own.stocks or 0)==1)~=v then return false
   elseif k=='stage_kind' and e.stage_kind~=v then return false
   elseif k=='combo_at_least' and not ((e.count or 0)>=v) then return false
   elseif k=='combo_damage_above' and not ((e.damage or 0)>v) then return false
   elseif k=='hit' and (e.hit==true)~=v then return false
   elseif k=='aerial' and e.aerial~=v then return false
   elseif k=='direction' and e.direction~=v then return false
   elseif k=='strength_above' and not ((e.strength or 0)>v) then return false
   elseif k=='armor_result' and not (v=='absorbed' and e.absorbed==true or v=='broke' and e.broke==true) then return false
   elseif k=='air_frames_above' and not ((own.air_frames or 0)>v) then return false
   elseif k=='aerial_hit' and (own.aerial_hit==true)~=v then return false
   elseif k=='recently' then local f=(self.recent[e.port] or {})[v.event];if not f or self.frame-f>S.resolve(v.frames,m,tier) then return false end end
  end end
  return true
 end
 local function ancestry(self,m,e)
  local out=copy(e.origin)
  for _,c in ipairs(m.conditions or {}) do
   for _,key in ipairs({'self_status','target_status'}) do local name=c[key]
    if name then local at=self.statuses[key=='self_status' and e.port or e.target] or {}
     for _,status in ipairs(ordered) do if name=='any' or name==status then for _,line in ipairs((at[status] or {}).origin or {}) do append(out,line) end end end
    end
   end
  end
  return out
 end
 function E:sustain_damage(port,amount)
  self.sustain=self.sustain or {};local next,delta=D.mod_budget.sustain_delta(self.sustain[port],amount);self.sustain[port]=next;self.damage[port]=(self.damage[port] or 0)+delta
 end
 function E:apply(m,e,tier)
  local origin=ancestry(self,m,e)
  for _,effect in ipairs(m.effects) do
   if self.used>=self.limit then self.dropped=self.dropped+1;break end
   if not effect.when or effect.when==e.kind then
    self.used=self.used+1;local port=effect.subject=='target' and e.target or e.port
    if effect.op=='chain_status' then port=self:nearest_other(e) end
    if port and self.players[port] then
     local value=function(v) return S.resolve(v,m,tier) end;local label=m.label
     if effect.op=='status' or effect.op=='stacks' or effect.op=='chain_status' then
      local duration=math.max(1,math.min(3600,math.floor(value(effect.duration)*(self:values(e.port).status_duration or 1))));assert(duration>=1 and duration<=3600,'status duration out of bounds')
      local at=self.statuses[port] or {};self.statuses[port]=at;local v=at[effect.status];local expires=self.frame+duration
      -- A status granted by a technique trigger is EARNED: it keeps its cause (the earned afterimage's colour) while it lasts.
      local cause=D.mod_skill.is_skill(e.kind) and D.mod_skill.cause_of(e.kind) or nil
      local amount=math.max(0,math.min(100,value(effect.amount or 1)*(D.mod_tuning and D.mod_tuning.amount_scale(m,effect) or 1)))
      if v then
       -- Two appliers of one status can differ in `max` (Plague Bearer stacks Burn to 3, a plain burn to 1): the instance keeps the
       -- LARGEST maximum any applier of it allows, so its stacks never exceed its own max (a snapshot of stacks 2, max 1 was refused,
       -- 2026-10-06, and the refusal cost the player the whole build).
       v.max=math.max(v.max,effect.max);v.stacks=math.min(v.max,v.stacks+1)
       if effect.refresh=='refresh' then v.expires=expires elseif effect.refresh=='extend' then v.expires=math.min(self.frame+3600,v.expires+duration) end
       v.amount=math.max(v.amount,amount)
      else v={expires=expires,stacks=1,max=effect.max,amount=amount,next_tick=self.frame+60,origin={}};at[effect.status]=v end
      if cause then v.cause=cause end
      append(origin,effect.status..' applied by '..label);v.origin=copy(origin)
      self:emit{kind='status_applied',port=port,target=e.target,status=effect.status,tags={[status_tags[effect.status]]=true},depth=e.depth+1,origin=origin}
      if effect.op=='stacks' or v.stacks>1 then self:emit{kind='stacks_changed',port=port,status=effect.status,tags={[status_tags[effect.status]]=true},depth=e.depth+1,origin=origin} end
     elseif effect.op=='remove_status' then
      local held=self:status(port,effect.status)
      if held and effect.count and held.stacks>effect.count then
       -- spend `count` stacks, keep the status (Momentum: a landing spends one, so a higher stack count still matters)
       held.stacks=held.stacks-effect.count;append(origin,effect.status..' stack spent by '..label)
       self:emit{kind='stacks_changed',port=port,status=effect.status,tags={[status_tags[effect.status]]=true},depth=e.depth+1,origin=origin}
      elseif held then self.statuses[port][effect.status]=nil;append(origin,effect.status..' spent by '..label)
       self:emit{kind='status_removed',port=port,status=effect.status,tags={},depth=e.depth+1,origin=origin}
      end
     elseif effect.op=='clank_damage' then
      if e.damage_a~=nil and e.damage_b~=nil then local amount=e.damage_a+e.damage_b;self.damage[port]=(self.damage[port] or 0)+amount;append(origin,'damage '..amount..' from '..label) end
     elseif effect.op=='heal' or effect.op=='damage' then
      local amount=math.max(0,math.min(100,value(effect.amount)));self:sustain_damage(port,effect.op=='heal' and -amount or amount)
      append(origin,(effect.op=='heal' and 'heal ' or 'damage ')..amount..' from '..label)
     elseif effect.op=='armor' then
      append(origin,'armour from '..label);self.fx[#self.fx+1]={op='armor',port=port,type=effect.type,value=effect.value~=nil and value(effect.value) or nil,frames=effect.frames and math.floor(value(effect.frames)) or nil,direction=effect.direction}
     elseif effect.op=='intangible' then
      append(origin,'intangibility from '..label);self.fx[#self.fx+1]={op='intangible',port=port,frames=math.floor(value(effect.frames))}
     elseif effect.op=='interrupt' then
      append(origin,'interrupt window from '..label);self.fx[#self.fx+1]={op='interrupt',port=port,frames=math.floor(value(effect.frames)),exits=effect.exits,guard=effect.guard,restore_jumps=effect.restore_jumps}
     elseif effect.op=='crit_next' then
      append(origin,'next hits crit from '..label);self.fx[#self.fx+1]={op='crit_next',port=port,count=math.floor(value(effect.count))}
     elseif effect.op=='emit' then
      append(origin,'event from '..label);self:emit{kind=effect.event,port=port,target=e.target,tags={[effect.tag]=true},depth=e.depth+1,origin=origin}
     end
    end
   end
  end
  if #origin>0 then self.trace=origin;self.trace_port=e.port;self.display.trace_generation=(self.display.trace_generation or 0)+1 end   -- trace_port: presentation only, not checkpointed
 end
 function E:drain()
  local at,lost=1,{}
  while at<=#self.queue do
   local e=self.queue[at];at=at+1
   if not (lost[e.port] and e.depth>1) then
   if e.native_trace and #e.origin>0 then
    self.trace=copy(e.origin);self.trace_port=e.port;self.display.trace_generation=(self.display.trace_generation or 0)+1
    -- Remember real native use of a status for later KO ancestry; no new status
    -- or gameplay effect is invented by this provenance annotation.
    for _,name in ipairs(e.native_statuses or {}) do local v=self:status(e.target,name);if v then v.origin=copy(e.origin) end end
   end
   if self.used>=self.limit then self.dropped=self.dropped+1 end
   self.recent[e.port]=self.recent[e.port] or {};self.recent[e.port][e.kind]=self.frame
   local build=self.equipped[e.port]
   if build and next(build) then for _,m in ipairs(self.list) do local tier=build[m.id]
    if self.used<self.limit and tier then for _,instance in ipairs(S.instances(tier))do if self:matches(m,e,instance)then
      self:apply(m,e,instance)
      -- The first time a technique or crit rule fires (per engine, i.e. per run) is logged for the host's announcement.
      -- (payoffs now carry min_depth too, so the test is the TRIGGER: a technique, a crit or armour absorbing a hit)
      if (m.min_depth and (D.mod_skill.is_skill(e.kind) or e.kind=='crit' or e.kind=='armor')) or m.kind=='keystone' and (D.mod_skill.is_skill(e.kind) or e.kind=='crit') then
       self.fired=self.fired or {};if not self.fired[m.id] then self.fired[m.id]=true;self.fired_log=self.fired_log or {};self.fired_log[#self.fired_log+1]={id=m.id,label=m.label,kind=e.kind,port=e.port} end
      end
     end end end
   end end
   -- A lost stock ends what was happening to the fighter (statuses, stacks, recent events), not the build:
   -- equipped modifiers stay and their steady effects, looks and hit rules are derived again from them.
   if e.kind=='stock_lost' then lost[e.port]=true;self.statuses[e.port]=nil;self.recent[e.port]=nil;self.damage[e.port]=nil end
   end
  end
  self.queue={}
 end
 -- Everything below is a pure function of one fighter's build, implicits and the statuses that matter to the
 -- derived values (name, stacks, amount; expiry and origin never enter a value, a rule or an echo). It is
 -- memoised by that content, so a fighter whose build and statuses did not change costs a key and a lookup
 -- instead of a re-derivation. The key is content, not identity: any write path invalidates it by changing it.
 local function eq_key(self,port)
  local eq=self.equipped[port]
  self.eqkeys=self.eqkeys or {}
  local ek=self.eqkeys[port]
  if not ek or ek.tbl~=eq then
   local parts,n={},0
   if eq then for id,tier in pairs(eq) do n=n+1;parts[n]=id..'='..(type(tier)=='table' and C.encode(tier) or tostring(tier)) end;table.sort(parts) end
   ek={tbl=eq,text=table.concat(parts,';')};self.eqkeys[port]=ek
  end
  return ek.text
 end
 local function memo_key(self,port)
  -- The equipped part of the key (every merged drive is a stack table that has to be encoded) is built once per equipped table, not on each of the
  -- ~20 memo lookups a frame makes (the deep-loop profile: 70 encodes a frame). set_build/import/clear replace the table, which is the invalidation.
  local out=eq_key(self,port)..'|t'..tostring(D.mod_tuning and D.mod_tuning.rev or 0)..'|'
  local im=self.implicits[port]
  if im then local k={};for key,v in pairs(im) do k[#k+1]=key..'='..tostring(v) end;table.sort(k);out=out..table.concat(k,';') end
  local at=self.statuses[port]
  if at then out=out..'|';for _,name in ipairs(ordered) do local v=at[name];if v then out=out..name..':'..tostring(v.stacks)..':'..tostring(v.amount)..';' end end end
  return out
 end
 -- The memo is shared between engine instances (the live one, the publication probe, the bag's preflight engines, the foe roller's): a
 -- derivation is a pure function of the content key, the pool and the progression context, and every probe used to repeat it from scratch
 -- (the campaigns' slow frames were all that: set_build -> native rules -> budget, 10 to 25 ms at depth). Bounded; cleared when it fills.
 D._memo_shared=D._memo_shared or {n=0,map={}}
 local function memo(self,port)
  local key=memo_key(self,port);local m=self.memo[port]
  if not m or m.key~=key then
   local sh=D._memo_shared
   local full=key..'|'..tostring(self.pool_tag or 0)..'|'..tostring(self.context and self.context.depth)..':'..tostring(self.context and self.context.loop)
   m=sh.map[full]
   if not m then if sh.n>=500 then sh.map,sh.n={},0 end;m={key=key};sh.map[full]=m;sh.n=sh.n+1 end
   self.memo[port]=m
  end
  return m
 end
 function E:family_budget(port)
  local m=memo(self,port)
  if not m.budget then m.budget,m.strength=D.mod_budget.build(self.list,self.equipped[port],self.implicits[port],self.statuses[port]) end
  return m.budget,m.strength
 end
 function E:values(port)
  local m=memo(self,port)
  if not m.values then m.values=D.mod_budget.values(self.list,self.equipped[port],self.implicits[port],self.statuses[port]) end
  local out={};for k,v in pairs(m.values) do out[k]=v end;return out -- callers edit their copy
 end
 -- The echo description depends on the build and on the statuses its echo rules wait for (Hasted for Trailing), not on every
 -- status: it is kept under that narrower key, so a status that no echo rule reads (a Guarded, a Burning) does not redescribe it.
 function E:echo_status_names()
  if not self.echo_names then local n={};for _,m in ipairs(self.list) do for _,e in ipairs(m.effects) do if e.op=='echo' and e.status then n[e.status]=true end end end;self.echo_names=n end
  return self.echo_names
 end
 function E:echo_description(port)
  local key=eq_key(self,port)
  for name in pairs(self:echo_status_names()) do key=key..'|'..name..'='..tostring(self:status(port,name) and true or false) end
  self.echo_memo=self.echo_memo or {}
  local c=self.echo_memo[port]
  if not c or c.key~=key then
   D._memo_shared=D._memo_shared or {n=0,map={}};local sh=D._memo_shared
   local full='echo|'..key..'|'..tostring(self.pool_tag or 0)..'|'..tostring(self.context and self.context.depth)..':'..tostring(self.context and self.context.loop)
   c=sh.map[full]
   if not c then c={key=key,value=D.mod_echo.engine(self,port)};if sh.n>=500 then sh.map,sh.n={},0 end;sh.map[full]=c;sh.n=sh.n+1 end
   self.echo_memo[port]=c
  end
  return c.value
 end
 function E:native_rules(port)
  local m=memo(self,port)
  if not m.rules then m.rules,m.bits=self:compute_native_rules(port) end
  return m.rules,m.bits
 end
 function E:compute_native_rules(port)
  if D.mod_echo then self:echo_description(port) end
  self:passive_state(port);self:crit_config(port)
  local out,bits={},0
  for _,name in ipairs(ordered) do if self:status(port,name) then bits=bits+(S.status_bits[name] or 0) end end
  for i,m in ipairs(self.list) do local tier=(self.equipped[port] or {})[m.id]
   if tier then for j,effect in ipairs(m.effects) do
    if effect.op=='convert' or effect.op=='versus-status' then
     local match,change=copy(effect.match),{}
     match.move=match.move or 'any';if effect.status then match.status_bits=S.status_bits[effect.status] end
     for k,v in pairs(effect.change) do change[k]=k=='element' and v or S.ratio(v,m,tier,k=='percent_damage' and (match.incoming and 'damage_taken' or 'damage_dealt') or (match.incoming and 'launch_taken' or 'launch_dealt')) end
     out[#out+1]={id=i*8+j,match=match,change=change}
    end
   end end
  end
  assert(#out<=30,'native hit rule capacity exceeded (two budget slots reserved)')
  local outgoing,incoming={},{}
  local budget
  for key,field in pairs({damage_dealt='percent_damage',damage_taken='percent_damage',knockback_taken='launch'}) do
   local f=D.mod_budget.families[key];budget=budget or self:family_budget(port);local raw=budget[f].raw
   if raw~=0 then
    local value=1+raw
    assert(value==value and math.abs(value)<=1e9,'aggregate native '..field..' must be finite and within native safety')
    local change=key=='damage_dealt' and outgoing or incoming;change[field]=value
   end
  end
  if next(outgoing) then out[#out+1]={id=1001,match={move='any'},change=outgoing} end
  if next(incoming) then out[#out+1]={id=1002,match={move='any',incoming=true},change=incoming} end
  assert(#out<=32,'native hit rule capacity exceeded');return out,bits
 end
 -- ---- technique-derived native state (pure functions of the build and the statuses) -----------------------------
 -- Passive caps and permanent armour come from equip rules. A restriction set is bounded here as well as in the
 -- keystone check, so a hand-made build cannot leave a fighter defenceless or unable to move.
 function E:passive_state(port)
  local m=memo(self,port)
  if m.passive then return m.passive end
  local out={forbid={},armor=nil};local seen={}
  for _,rule in ipairs(self.list) do local tier=(self.equipped[port] or {})[rule.id]
   if tier then for _,effect in ipairs(rule.effects) do
    for _,instance in ipairs(S.instances(tier)) do
     if effect.op=='air_jumps' then local n=math.floor(S.resolve(effect.count,rule,instance));out.air_jumps=math.max(out.air_jumps or n,n)
     elseif effect.op=='restrict' then for _,x in ipairs(effect.forbid) do if not seen[x] then seen[x]=true;out.forbid[#out.forbid+1]=x end end
     elseif effect.op=='armor' and effect.frames==nil then local v=S.resolve(effect.value,rule,instance);if not out.armor or v>out.armor.value then out.armor={type=effect.type,value=v} end end
    end
   end end
  end
  table.sort(out.forbid)
  local ok,why=D.mod_budget.restrictions_ok(out.forbid);assert(ok,why)
  m.passive=out;return out
 end
 -- The crit configuration one fighter should have right now, or nil: slots by move tag (default first), a percent floor.
 -- Chance adds (capped at .6 a slot); a chance rule's multiplier is the base (the largest), a multiplier-only rule adds its
 -- gain on top; a tag slot starts from the default and adds its own. A forced-crit rule (crit_next) needs a configuration
 -- to exist and raises the default multiplier to its own.
 function E:crit_config(port)
  local m=memo(self,port)
  local names={};for _,n in ipairs(ordered) do if (self.statuses[port] or {})[n] then names[#names+1]=n end end
  local key='crit:'..table.concat(names,',')
  if m[key] then return m[key].config end
  local slots,floor,force,any={},0,nil,false
  local function slot(tag) local s=slots[tag];if not s then s={chance=0,base=nil,add=0,max_add=0,launch=1};slots[tag]=s end;return s end
  for _,rule in ipairs(self.list) do local tier=(self.equipped[port] or {})[rule.id]
   if tier then for _,effect in ipairs(rule.effects) do
    if effect.op=='crit' and (not effect.status or self:status(port,effect.status)) then
     for _,instance in ipairs(S.instances(tier)) do
      any=true;local sl=slot(effect.tag or 'default')
      local c=effect.chance and S.resolve(effect.chance,rule,instance) or 0;local mult=effect.multiplier and S.resolve(effect.multiplier,rule,instance)
      sl.chance=sl.chance+c
      if effect.chance then local b=mult or 1.5;sl.base=math.max(sl.base or 0,b) elseif mult then sl.add=sl.add+(mult-1) end
      if effect.multiplier_max then sl.max_add=math.max(sl.max_add,S.resolve(effect.multiplier_max,rule,instance)-(mult or 1.5)) end
      if effect.launch then sl.launch=math.max(sl.launch,S.resolve(effect.launch,rule,instance)) end
      if effect.min_percent then floor=math.max(floor,S.resolve(effect.min_percent,rule,instance)) end
     end
    elseif effect.op=='crit_next' then
     for _,instance in ipairs(S.instances(tier)) do any=true;force=math.max(force or 1,S.resolve(effect.multiplier,rule,instance)) end
    end
   end end
  end
  local config
  if any then
   slot('default');local out={}
   local function finish(tag)
    local sl=slots[tag];local d=slots.default;local isd=tag=='default'
    local chance=math.min(floor>=80 and 1 or .6,(isd and 0 or d.chance)+sl.chance) -- a certain crit is only possible under a percent floor
    local base=sl.base or (not isd and d.base) or 1.5
    local mult=math.min(4,math.max(1,base+(isd and 0 or d.add)+sl.add))
    local mmax=math.min(4,mult+(isd and 0 or d.max_add)+sl.max_add)
    -- `mean` is the configured multiplier. The native draw is uniform in [low, high] around it (low = half the gain, high = one and a half
    -- times the gain, plus any explicit maximum), so crits differ in strength: the engine's strength is a crit's gain over the largest gain.
    local gain=mult-1;local extra=math.max(0,mmax-mult)
    return {chance=chance,mean=mult,multiplier=1+gain*.5,multiplier_max=math.min(4,1+gain*1.5+extra),launch=math.max(sl.launch,isd and 1 or d.launch)}
   end
   out.default=finish('default')
   if force and force>out.default.mean then local d=out.default;d.mean=force;d.multiplier_max=math.min(4,1+(force-1)*1.5);d.multiplier=math.min(d.multiplier_max,1+(force-1)*.5) end -- a forced crit above x4 must not leave multiplier above multiplier_max (the native config refuses it: found at deep loops)
   for tag in pairs(slots) do if tag~='default' then out[tag]=finish(tag) end end
   config={slots=out,min_percent=floor}
  end
  m[key]={config=config};return config
 end
 -- ---- online play: the passive build as native ops (Envoy netplay plan, stage 2) --------------------------------------------------
 -- Offline, mod_lab commits these ops after every logic frame. Online nothing may write from Lua during a match, so the SAME ops for the
 -- passive part of a build are compiled once, before the match, and staged (gd.netbuild_stage) for the native side to apply before frame 0.
 -- A record is online-safe today when it is passive (trigger 'equip') and every effect is one of the ops below, all of which have a native carrier
 -- already living in snapshotted game memory: value (fighter overlay), convert and versus-status (hit rules), crit (crit configuration),
 -- armor and air_jumps and restrict (armour and capability tables). Echoes and every triggered record need the native evaluator (stage 4).
 local online_ops={value=true,convert=true,['versus-status']=true,crit=true,armor=true,air_jumps=true,restrict=true}
 function E.online_safe(rule)
  if rule.trigger~='equip' then
   -- Stage 4: a triggered record is online-safe when the native evaluator has a form for its trigger, conditions and effects (E.native_support, below).
   local ok,why=E.native_support(rule)
   if not ok then return false,'triggered ('..tostring(rule.trigger)..'): '..tostring(why) end
   return true
  end
  for _,e in ipairs(rule.effects) do if not online_ops[e.op] then return false,'effect '..e.op..' has no native carrier yet' end end
  return true
 end
 -- Every record in the pool that is online-safe, in pool order.
 function E.online_pool(list)
  local out={};for _,m in ipairs(list) do if E.online_safe(m) then out[#out+1]=m.id end end;return out
 end
 -- The op list for one port's build, deterministic and canonical: fighter values, hit rules, crit configuration (begin, default, then the tags in a fixed
 -- order), capability caps, permanent armour. Raises for a record that is not online-safe. `slot` (default: the port) is the 1-based seat the ops are for.
 function E:passive_ops(port,slot)
  slot=slot or port
  for id in pairs(self.equipped[port] or {}) do
   local rule=assert(self.rules[id],'unknown modifier '..tostring(id));local ok,why=E.online_safe(rule)
   assert(ok,'record '..id..' is not online-safe: '..tostring(why))
  end
  local ops={}
  local values=self:values(port);values.status_duration=nil;values.damage_dealt=nil;values.damage_taken=nil;values.knockback_taken=nil
  if next(values) then ops[#ops+1]={op='fighter_mod',port=slot,values=values} end
  local rules,bits=self:native_rules(port)
  if #rules>0 or bits~=0 then ops[#ops+1]={op='hit_rules',port=slot,rules=rules,status_bits=bits} end
  local cfg=self:crit_config(port)
  if cfg then
   ops[#ops+1]={op='crit',entity=slot,begin=true,min_percent=cfg.min_percent}
   local function add(tag,s) local o={op='crit',entity=slot,slot=tag,chance=s.chance,multiplier=s.multiplier,launch=s.launch};if s.multiplier_max then o.multiplier_max=s.multiplier_max end;ops[#ops+1]=o end
   add('default',cfg.slots.default)
   for _,tag in ipairs({'jab','dash_attack','tilt','smash','aerial','grab','throw','special','projectile'}) do if cfg.slots[tag] then add(tag,cfg.slots[tag]) end end
  end
  local ps=self:passive_state(port)
  local caps={}
  if ps.air_jumps then caps.air_jumps=math.max(0,math.min(6,ps.air_jumps)) end
  for _,x in ipairs(ps.forbid) do caps[x]=true end
  if next(caps) then ops[#ops+1]={op='fighter_caps',entity=slot,values=caps} end
  if ps.armor then ops[#ops+1]={op='fighter_armor',entity=slot,type=ps.armor.type,value=math.min(10,ps.armor.value),frames=0} end
  for _,o in ipairs(ops) do D.mod_registry.operation(o) end
  return ops
 end
 -- The status a technique earned that an afterimage should show right now: the longest-lasting earned status of the fighter
 -- ({cause, status, frames}), or nil. Pure.
 function E:earned(port)
  local best
  for _,name in ipairs(ordered) do local v=(self.statuses[port] or {})[name]
   if v and v.cause then local left=v.expires-self.frame
    if left>0 and (not best or left>best.frames) then best={cause=v.cause,status=name,frames=math.min(left,D.mod_skill.max_frames)} end
   end
  end
  return best
 end
 -- How many frames an echo's picture may show: while the status its rule needs is on (the status ends it), else a short
 -- window after the fighter's own last hit. Never continuous. Pure function of checkpointed state.
 E.echo_window_after_hit=45
 function E:echo_window(port)
  local best=0
  for _,rule in ipairs(self.list) do local tier=(self.equipped[port] or {})[rule.id]
   if tier then for _,effect in ipairs(rule.effects) do if effect.op=='echo' then
    if effect.status then local v=self:status(port,effect.status);if v then best=math.max(best,v.expires-self.frame) end
    else local f=(self.recent[port] or {}).hit_dealt;if f then best=math.max(best,E.echo_window_after_hit-(self.frame-f)) end end
   end end end
  end
  return math.max(0,math.min(best,D.mod_skill.max_frames))
 end
 -- Whether any equipped rule of this fighter listens to a trigger (the host only reads skill state and builds an event then).
 -- The opponent nearest the victim of a hit, other than the attacker and the victim (frame positions come from the host sample).
 function E:nearest_other(e)
  local from=self.players[e.target] or {};if type(from.x)~='number' then return nil end
  local best,bd
  for p,v in pairs(self.players) do if p~=e.port and p~=e.target and type(v.x)=='number' and type(v.y)=='number' then
   local d=(v.x-from.x)^2+(v.y-(from.y or 0))^2;if not bd or d<bd or (d==bd and p<best) then best,bd=p,d end
  end end
  return best
 end
 function E:listens(port,kind)
  for _,rule in ipairs(self.list) do if (self.equipped[port] or {})[rule.id] then
   if rule.trigger==kind then return true end
   for _,a in ipairs(rule.also or {}) do if a.trigger==kind then return true end end
  end end
  return false
 end
 -- Whether the fighter carries a rule that can EARN a status (a technique trigger with a status effect): the host prepares its
 -- afterimage emitter at stage start so the first earned status shows at once.
 function E:earned_source(port)
  local m=memo(self,port)
  if m.earned_source==nil then
   m.earned_source=false
   for _,rule in ipairs(self.list) do if (self.equipped[port] or {})[rule.id] and D.mod_skill.is_skill(rule.trigger) then
    for _,effect in ipairs(rule.effects) do if effect.op=='status' or effect.op=='stacks' then m.earned_source=true end end
   end end
  end
  return m.earned_source
 end
 function E:native_origin(ids,target,actor)
  local out,used={},{}
  for _,id in ipairs(ids or {}) do
   if id==1001 or id==1002 then
    local incoming=id==1002;local port=incoming and target or actor
    local keys=incoming and {damage_taken=true,knockback_taken=true} or {damage_dealt=true}
    for _,m in ipairs(self.list) do if (self.equipped[port] or {})[m.id] then
     for _,effect in ipairs(m.effects) do if effect.op=='value' and keys[effect.key] then append(out,'additive '..effect.key..' from '..m.label) end end
    end end
    for _,key in ipairs({'damage_dealt','damage_taken','knockback_taken'}) do if keys[key] and (self.implicits[port] or {})[key] then append(out,'additive '..key..' from Drive implicit') end end
    if incoming then for _,name in ipairs({'curse','guarded'}) do local v=self:status(port,name);if v then
     used[#used+1]=name;for _,line in ipairs(v.origin or {}) do append(out,line) end;append(out,'incoming budget from '..name)
    end end end
   elseif type(id)=='number' and id%1==0 then local m=self.list[math.floor((id-1)/8)];local effect=m and m.effects[(id-1)%8+1]
    if effect and (effect.op=='convert' or effect.op=='versus-status') then
     if effect.status then used[#used+1]=effect.status;for _,line in ipairs((self:status(target,effect.status) or {}).origin or {}) do append(out,line) end end
     local action=effect.change.element and 'hit converted to '..effect.change.element or effect.change.launch and 'launch boosted' or 'hit damage scaled'
     append(out,action..' by '..m.label)
    end
   end
  end
  return out,used
 end
 function E:contact_ratios(attacker,defender,original_element)
  -- Pure evaluation of the emitted native packet for a grounded normal-origin smash.
  local attack=self:native_rules(attacker);local defence,bits=self:native_rules(defender)
  original_element=original_element or 'normal';assert(S.elements[original_element],'unsupported original element');local element=original_element
  for _,r in ipairs(attack)do local m=r.match
   if not m.incoming and r.change.element and (m.move=='any' or m.move=='smash') and m.grounded~=false and (not m.element or m.element==original_element)then element=r.change.element end
  end
  local function evaluate(rules,incoming)
   local percent,launch=1,1
   for _,r in ipairs(rules)do local m=r.match
    if (m.incoming==true)==incoming and (m.move=='any' or m.move=='smash') and m.grounded~=false and (not m.element or m.element==original_element) and (not m.status_bits or bits&m.status_bits~=0)then
     percent=percent+(r.change.percent_damage or 1)-1;launch=launch+(r.change.launch or 1)-1
    end
   end
   return math.max(incoming and .15 or .05,math.min(64,percent)),math.max(.05,math.min(4,launch))
  end
  local po,lo=evaluate(attack,false);local pt,lt=evaluate(defence,true)
  return {percent_damage=po*pt,launch=lo*lt,outgoing=po,incoming=pt,launch_out=lo,launch_in=lt,element=element,original_element=original_element}
 end
 -- `trim` (a run's checkpoint, which is never restored): each status keeps the first 6 lines of its origin list, not up to 24. At depth six
 -- fighters carry several statuses each and the origins were 5 KB of the 16 KiB blob (found by the campaigns).
 function E:export(trim)
  assert(#self.queue==0,'checkpoint requires a drained event boundary')
  local statuses=self.statuses
  if trim then
   statuses={}
   for p,at in pairs(self.statuses) do local t={};for name,v in pairs(at) do local c={};for k,x in pairs(v) do c[k]=x end;if type(v.origin)=='table' and #v.origin>6 then local o={};for i=1,6 do o[i]=v.origin[i] end;c.origin=o end;t[name]=c end;statuses[p]=t end
  end
  return C.encode{version=1,seed=self.seed,frame=self.frame,context=self.context,equipped=self.equipped,implicits=self.implicits,statuses=statuses,recent=self.recent,trace=self.trace,dropped=self.dropped,limit=self.limit,depth=self.depth,display=self.display}
 end
 -- The live statuses repaired in place (a max below its stacks is raised); the host logs what this returns. Cheap: a handful of instances.
 function E:sanitize_statuses()
  local out
  for p,at in pairs(self.statuses) do for name,v in pairs(at) do
   if type(v)=='table' and type(v.stacks)=='number' and type(v.max)=='number' and v.max<v.stacks and v.stacks<=8 then
    out=out or {};out[#out+1]=('P%d %s: max %d raised to its %d stacks'):format(p,tostring(name),v.max,v.stacks);v.max=v.stacks
   end
  end end
  return out
 end
 function E:import(text)
  local at=C.decode(text);at.context=D.mod_progression.context(at.context);assert(at.version==1 and type(at.frame)=='number' and at.frame%1==0 and at.frame>=0,'invalid modifier snapshot')
  assert(type(at.seed)=='number' and at.seed%1==0 and at.seed>=1 and at.seed<2147483647,'invalid random state')
  assert(type(at.limit)=='number' and at.limit>=1 and at.limit<=128 and at.limit%1==0 and type(at.depth)=='number' and at.depth>=1 and at.depth<=16 and at.depth%1==0,'invalid snapshot budget')
  -- A snapshot from before the readability split may hold a piece the split cut: it is dropped (and listed in self.retired_dropped for the
  -- host to say so), never a refusal of the whole snapshot.
  self.retired_dropped=nil;self.status_repairs=nil
  for p,mods in pairs(at.equipped) do assert(type(p)=='number' and p>=1 and p<=6 and p%1==0)
   for id in pairs(mods) do if not self.rules[id] and D.drive_loot and D.drive_loot.is_retired(id) then mods[id]=nil;self.retired_dropped=self.retired_dropped or {};self.retired_dropped[#self.retired_dropped+1]=id end end
   for id,tier in pairs(mods) do assert(self.rules[id] and S.level(tier),'unknown snapshot modifier') end
  end
  at.implicits=at.implicits or {}
  for p,base in pairs(at.implicits) do assert(type(p)=='number' and p%1==0 and p>=1 and p<=6);for key,value in pairs(base) do assert(S.values[key] and type(value)=='number' and value==value and value>=-1000000 and value<=1000000,'invalid snapshot implicit') end end
  for p,statuses in pairs(at.statuses) do assert(type(p)=='number' and p>=1 and p<=6 and p%1==0)
   for name,v in pairs(statuses) do
    -- A status instance that is slightly out of range is REPAIRED (its max raised to its stacks, its stacks and amount clamped, an expiry
    -- in the past made now), one that cannot be repaired is DROPPED: never a refusal of the whole snapshot, which costs the player the build.
    -- Both are listed in self.status_repairs for the host to say so.
    if type(v)=='table' and S.statuses[name] and type(v.stacks)=='number' and type(v.max)=='number' then
     local before=('stacks %s max %s'):format(tostring(v.stacks),tostring(v.max))
     local fixed=false
     if v.stacks%1~=0 or v.stacks<1 then v.stacks=math.max(1,math.floor(v.stacks));fixed=true end
     if v.stacks>8 then v.stacks=8;fixed=true end
     if v.max<v.stacks then v.max=v.stacks;fixed=true end
     if v.max>8 then v.max=8;v.stacks=math.min(v.stacks,8);fixed=true end
     if type(v.expires)=='number' and v.expires<at.frame then v.expires=at.frame;fixed=true end
     if type(v.amount)=='number' and (v.amount<0 or v.amount>100) then v.amount=math.max(0,math.min(100,v.amount));fixed=true end
     if type(v.next_tick)=='number' and v.next_tick%1~=0 then v.next_tick=math.floor(v.next_tick);fixed=true end
     if fixed then self.status_repairs=self.status_repairs or {};self.status_repairs[#self.status_repairs+1]=('P%d %s repaired (%s)'):format(p,name,before) end
    end
    local why
    if not S.statuses[name] then why='unknown status' elseif type(v)~='table' then why='not a table'
    elseif not (v.stacks>=1 and v.stacks<=8 and v.stacks%1==0) then why='stacks '..tostring(v.stacks)
    elseif not (v.max>=v.stacks and v.max<=8) then why='max '..tostring(v.max)..' stacks '..tostring(v.stacks)
    elseif not (v.expires>=at.frame and v.expires%1==0) then why='expires '..tostring(v.expires)..' frame '..tostring(at.frame)
    elseif not (v.next_tick%1==0) then why='next_tick '..tostring(v.next_tick)
    elseif not (v.amount>=0 and v.amount<=100) then why='amount '..tostring(v.amount)
    elseif type(v.origin)~='table' then why='origin'
    elseif not (v.cause==nil or D.mod_skill.cause[v.cause]~=nil) then why='cause '..tostring(v.cause) end
    if why then
     statuses[name]=nil;self.status_repairs=self.status_repairs or {};self.status_repairs[#self.status_repairs+1]=('P%d %s dropped (%s)'):format(p,tostring(name),why)
    end
   end
  end
  for p,events in pairs(at.recent) do assert(type(p)=='number' and p%1==0 and p>=1 and p<=6);for event,frame in pairs(events) do assert(S.events[event] and type(frame)=='number' and frame%1==0 and frame>=0 and frame<=at.frame,'invalid recent event') end end
  assert(type(at.trace)=='table' and #at.trace<=24 and type(at.dropped)=='number' and at.dropped>=0 and at.dropped%1==0,'invalid snapshot trace');for _,line in ipairs(at.trace) do assert(type(line)=='string' and #line<=160) end
  assert(type(at.display)=='table' and type(at.display.intensity)=='number' and at.display.intensity>=0 and at.display.intensity<=1,'invalid display snapshot')
  for _,key in ipairs({'last_pulse','pulse_start','pulse_strength'}) do assert(type(at.display[key])=='number','invalid visual clock') end
  assert(at.display.pulse_strength>=0 and at.display.pulse_strength<=.06 and type(at.display.trace_key)=='string','invalid visual pulse')
  if at.display.trace_generation then assert(type(at.display.trace_generation)=='number' and at.display.trace_generation%1==0 and at.display.trace_generation>=0) end
  local probe=E.new(1,self.list,{context=at.context});for p,mods in pairs(at.equipped)do probe:set_build(p,mods,at.implicits[p])end
  for _,k in ipairs({'seed','frame','context','equipped','implicits','statuses','recent','trace','dropped','limit','depth','display'}) do self[k]=at[k] end
  self.queue={};self.damage={};self.players={};self.used=0
 end
 function E:set_context(context)
  local c=D.mod_progression.context(context);local old=self.context;self.context=c
  local ok,err=pcall(function()for p,mods in pairs(self.equipped)do self:set_build(p,mods,self.implicits[p])end end)
  if not ok then self.context=old;error(err)end
  return c
 end
 -- ---- online play, stage 4: the TRIGGERED rules as a native program (Envoy netplay plan, stage 4) -----------------------------------------------
 -- The records are data, so everything that is a pure function of (build, tier) is resolved here ONCE, before the match: E:trigger_program flattens the
 -- equipped triggered records into numbers (every $tier resolved with S.resolve, durations scaled by the build's status_duration and clamped exactly as
 -- E:apply does, amounts clamped, technique effects clamped as mod_lab:technique_ops does) and the native evaluator (pc/gameworld/script_mods_core.h)
 -- runs them on the simulation's own events. The derived native tables that depend on WHICH statuses a fighter holds (fighter values, hit rules and their
 -- status bits, the crit configuration) are enumerated per status mask as VARIANTS, computed by the same E:values/native_rules/crit_config the offline
 -- host commits; the evaluator only picks a variant when the mask changes. The numbers below mirror script_mods_core.h (checked by the parity test).
 local NATIVE_EVENTS={'hit_dealt','hit_taken','ko_dealt','stock_lost','shield_hit','perfect_shield','clank','jump','air_jump','landing','ledge_grab','grab','throw','taunt','interval','status_applied','status_removed','stacks_changed','crit','armor','lcancel','lcancel_hit','lcancel_miss','wavedash','waveland','ledge_dash','air_dodge','tech','tech_miss','dash_dance','short_hop','full_hop','fast_fall','shield_drop','jump_cancel_grab','jump_cancel_usmash','sdi','auto_cancel','combo','combo_end'}
 local NATIVE_TAGS={'jab','tilt','smash','aerial','special','grab','throw','projectile','dash_attack','grounded','airborne','normal','fire','electric','ice','darkness','burning','shocked','chilled','cursed','hasted','guarded','momentum','damage','healing','unique','keystone','technique','critical'}
 local NATIVE_EXITS={'jab','tilt','smash','aerial','special','grab','jump','dash','crouch','turn','walk','escape','shield','air_dodge','air_jump'}
 local NATIVE_ARMOR={knockback=1,damage_threshold=2,knockback_threshold=3,super=4,hit_count=5,damage_pool=6}
 local NATIVE_OPS={status=1,stacks=2,chain_status=3,remove_status=4,clank_damage=5,heal=6,damage=7,armor=8,intangible=9,interrupt=10,crit_next=11,emit=12}
 local NATIVE_COND={tag=1,status=2,self_status=3,target_status=4,self_damage_above=5,self_damage_below=6,target_damage_above=7,grounded=8,airborne=9,last_stock=10,stage_kind=11,combo_at_least=12,combo_damage_above=13,hit=14,aerial=15,direction=16,strength_above=17,armor_result=18,air_frames_above=19,aerial_hit=20,recently=21}
 local NATIVE_AERIAL={nair=0,fair=1,bair=2,uair=3,dair=4}
 local NATIVE_DIRECTION={in_place=0,toward=1,away=2,wall=3,ceiling=4}
 local EV,TAGBIT,STATUS_ID,EXITBIT={},{},{},{}
 for i,n in ipairs(NATIVE_EVENTS) do EV[n]=i end
 for i,n in ipairs(NATIVE_TAGS) do TAGBIT[n]=1<<(i-1) end
 for i,n in ipairs(ordered) do STATUS_ID[n]=i end
 for i,n in ipairs(NATIVE_EXITS) do EXITBIT[n]=1<<(i-1) end
 E.native={status_bits=D.mod_status.bits,events=EV,event_names=NATIVE_EVENTS,tags=TAGBIT,status_ids=STATUS_ID,max_rules=24,max_variants=24,magic=0x534D5031,header=136,rule_words=268,masks=128}
 local armor_limit={super=30,damage_threshold=300,knockback_threshold=300,hit_count=600,damage_pool=600}
 local function fbits(x) return (string.unpack('<i4',string.pack('<f',x))) end
 local function clamp(lo,hi,x) return math.max(lo,math.min(hi,x)) end
 -- Why a triggered record cannot run natively, or nil. A pure function of the record.
 local function flatten(m,tier,conditions)
  local out={}
  for _,c in ipairs(conditions or {}) do
   local keys={};for k in pairs(c) do keys[#keys+1]=k end;table.sort(keys)
   for _,k in ipairs(keys) do
    local v=c[k];if type(v)=='string' and v:sub(1,1)=='$' then v=S.resolve(v,m,tier) end
    local kind=assert(NATIVE_COND[k],'condition '..k..' has no native form');local a,b=0,0
    if k=='tag' then a=assert(TAGBIT[v],'unknown tag')
    elseif k=='status' or k=='self_status' or k=='target_status' then a=v=='any' and 0 or assert(STATUS_ID[v],'unknown status')
    elseif k=='self_damage_above' or k=='self_damage_below' or k=='target_damage_above' or k=='combo_damage_above' or k=='strength_above' then b=fbits(v)
    elseif k=='grounded' or k=='airborne' or k=='last_stock' or k=='hit' or k=='aerial_hit' then a=v and 1 or 0
    elseif k=='combo_at_least' or k=='air_frames_above' then a=math.floor(v)
    elseif k=='aerial' then a=assert(NATIVE_AERIAL[v],'unknown aerial')
    elseif k=='direction' then a=assert(NATIVE_DIRECTION[v],'unknown direction')
    elseif k=='armor_result' then a=v=='absorbed' and 1 or 2
    elseif k=='recently' then a=assert(EV[v.event],'recently: no native source for '..tostring(v.event));b=math.floor(S.resolve(v.frames,m,tier))
    end
    out[#out+1]={kind,a,b}
   end
  end
  assert(#out<=8,'more than 8 conditions')
  return out
 end
 function E.native_support(rule)
  if rule.trigger=='equip' then return true end
  local ok,why=pcall(function()
   assert(EV[rule.trigger],'no native source for the trigger '..tostring(rule.trigger))
   for _,a in ipairs(rule.also or {}) do assert(EV[a.trigger],'no native source for the trigger '..tostring(a.trigger)) end
   assert(1+#(rule.also or {})<=4,'too many alternative triggers')
   flatten(rule,1,rule.conditions);for _,a in ipairs(rule.also or {}) do flatten(rule,1,a.conditions) end
   assert(#rule.effects<=8,'more than 8 effects')
   for _,e in ipairs(rule.effects) do
    assert(NATIVE_OPS[e.op],'effect '..e.op..' has no native form')
    if e.op=='emit' then assert(EV[e.event] and TAGBIT[e.tag],'emit has no native form') end
   end
  end)
  if ok then return true end
  return false,(tostring(why):gsub('^.-:%d+: ',''))
 end
 local function compile_effect(self,m,instance,effect,sd)
  local w={};for i=1,20 do w[i]=0 end
  local function set(i,v) w[i+1]=v end
  local function value(v) return S.resolve(v,m,instance) end
  set(0,NATIVE_OPS[effect.op]);set(1,effect.subject=='target' and 1 or 0);set(2,effect.when and EV[effect.when] or 0)
  local op=effect.op
  if op=='status' or op=='stacks' or op=='chain_status' then
   set(3,STATUS_ID[effect.status]);set(4,clamp(1,3600,math.floor(value(effect.duration)*sd)));set(5,effect.max)
   set(6,effect.refresh=='refresh' and 0 or effect.refresh=='extend' and 1 or 2)
   set(11,fbits(clamp(0,100,value(effect.amount or 1)*(D.mod_tuning and D.mod_tuning.amount_scale(m,effect) or 1))))
  elseif op=='remove_status' then set(3,STATUS_ID[effect.status]);set(7,effect.count or 0)
  elseif op=='heal' or op=='damage' then local a=clamp(0,100,value(effect.amount));set(11,fbits(op=='heal' and -a or a))
  elseif op=='armor' then
   local ty=effect.type;set(9,NATIVE_ARMOR[ty]);set(8,clamp(1,armor_limit[ty] or 30,math.floor(effect.frames and value(effect.frames) or 1)))
   local v=ty=='super' and 1 or clamp(1,ty=='hit_count' and 3 or 40,effect.value~=nil and value(effect.value) or 1);if ty=='hit_count' then v=math.floor(v) end
   set(10,fbits(v));set(12,effect.direction=='front' and 1 or effect.direction=='back' and -1 or 0)
  elseif op=='intangible' then set(8,clamp(1,24,math.floor(value(effect.frames))))
  elseif op=='interrupt' then
   set(8,clamp(1,20,math.floor(value(effect.frames))))
   local mask=0x7FFF;if effect.exits then mask=0;for _,x in ipairs(effect.exits) do mask=mask|EXITBIT[x] end end
   set(13,mask);set(14,(effect.guard and 1 or 0)|(effect.restore_jumps and 2 or 0))
  elseif op=='crit_next' then set(7,clamp(1,3,math.floor(value(effect.count))))
  elseif op=='emit' then set(15,EV[effect.event]);set(16,TAGBIT[effect.tag])
  end
  return w
 end
 -- The status-mask variants of one port's derived native tables (see the header). Pure; uses a private probe engine, never self's statuses.
 local function variants_of(self,port)
  local probe=E.new(self.seed,self.list,{context=self.context});probe:set_build(port,self.equipped[port] or {},self.implicits[port])
  -- Only statuses that change a derived table matter: those with fighter values, and those a crit effect waits for. (Status BITS are not in a variant.)
  local relevant=0
  for i,name in ipairs(ordered) do local d=D.mod_status.by_name[name];if d and d.values~=nil then relevant=relevant|(1<<(i-1)) end end
  for _,m in ipairs(self.list) do if (self.equipped[port] or {})[m.id] then for _,e in ipairs(m.effects) do if e.op=='crit' and e.status then relevant=relevant|(1<<(STATUS_ID[e.status]-1)) end end end end
  local list,index,map,of={},{},{},{}
  for mask=0,127 do
   local base=mask&relevant
   if of[base]~=nil then map[mask]=of[base]
   else
    local st={};for i,name in ipairs(ordered) do if base&(1<<(i-1))~=0 then st[name]={expires=3600,stacks=1,max=1,amount=0,next_tick=60,origin={}} end end
    probe.statuses={[port]=next(st) and st or nil}
    local ops={}
    local values=probe:values(port);values.status_duration=nil;values.damage_dealt=nil;values.damage_taken=nil;values.knockback_taken=nil
    if next(values) then ops[#ops+1]={op='fighter_mod',port=port,values=values} end
    local rules=probe:native_rules(port)
    local curse=0
    local copy={};for i,r in ipairs(rules) do copy[i]=r end
    if base&(1<<(STATUS_ID.curse-1))~=0 then
     for i,r in ipairs(copy) do if r.id==1002 then curse=i end end
     if curse==0 then copy[#copy+1]={id=1002,match={move='any',incoming=true},change={launch=1}};curse=#copy end
    end
    -- The status bits are not part of a variant: they are a function of the mask alone (E.native.status_bits), written by the host at every apply.
    if #copy>0 then ops[#ops+1]={op='hit_rules',port=port,rules=copy,status_bits=0} end
    local cfg=probe:crit_config(port)
    if cfg then
     ops[#ops+1]={op='crit',entity=port,begin=true,min_percent=cfg.min_percent}
     local function add(tag,s) local o={op='crit',entity=port,slot=tag,chance=s.chance,multiplier=s.multiplier,launch=s.launch};if s.multiplier_max then o.multiplier_max=s.multiplier_max end;ops[#ops+1]=o end
     add('default',cfg.slots.default)
     for _,tag in ipairs({'jab','dash_attack','tilt','smash','aerial','grab','throw','special','projectile'}) do if cfg.slots[tag] then add(tag,cfg.slots[tag]) end end
    end
    local key=C.encode({ops,curse})
    local at=index[key]
    if not at then
     assert(#list<E.native.max_variants,'more than '..E.native.max_variants..' derived variants')
     list[#list+1]={ops=ops,curse_rule=curse};at=#list-1;index[key]=at
    end
    of[base]=at;map[mask]=at
   end
  end
  return list,map
 end
 -- The program of one port's triggered build, or nil when it holds no triggered record. `slot` is the 1-based seat (default: the port).
 -- Returns {words=,variants=,mask_variant=,rules=n}. Deterministic: same build -> same words, whatever the process.
 function E:trigger_program(port,slot)
  slot=slot or port
  local eq=self.equipped[port] or {}
  local sd=self:values(port).status_duration or 1
  local rules={}
  for _,m in ipairs(self.list) do local tier=eq[m.id]
   if tier and m.trigger~='equip' then
    local ok,why=E.native_support(m);assert(ok,'record '..m.id..' has no native form: '..tostring(why))
    for _,instance in ipairs(S.instances(tier)) do
     local triggers={{EV[m.trigger],flatten(m,instance,m.conditions)}}
     for _,a in ipairs(m.also or {}) do triggers[#triggers+1]={EV[a.trigger],flatten(m,instance,a.conditions)} end
     local effects={};for _,e in ipairs(m.effects) do effects[#effects+1]=compile_effect(self,m,instance,e,sd) end
     rules[#rules+1]={id=m.id,interval=m.trigger=='interval' and math.floor(S.resolve(m.interval,m,instance)) or 0,triggers=triggers,effects=effects}
    end
   end
  end
  assert(#rules<=E.native.max_rules,'more than '..E.native.max_rules..' triggered rule instances in one build')
  return rules
 end
 -- Whether any equipped record of this port is triggered.
 function E:has_triggered(port) for _,m in ipairs(self.list) do if (self.equipped[port] or {})[m.id] and m.trigger~='equip' then return true end end;return false end
 -- The staged form for one port (slot = 1-based seat): flat words for the evaluator, the variants and the mask map for the host. Call once either seat has
 -- a triggered record: a seat without one still needs its variants, because the other seat's statuses land on it.
 function E:native_program(port,slot)
  slot=slot or port
  local rules=self:trigger_program(port,slot)
  local N=E.native;local words={}
  local function put(i,v) words[i+1]=v end
  for i=1,N.header+N.max_rules*N.rule_words do words[i]=0 end
  put(0,N.magic);put(1,#rules)
  local variants,map=variants_of(self,port)
  put(2,#variants)
  for mask=0,N.masks-1 do put(8+mask,map[mask]) end
  for r,rule in ipairs(rules) do
   local base=N.header+(r-1)*N.rule_words
   put(base,rule.interval);put(base+1,#rule.triggers);put(base+2,#rule.effects)
   for t,tr in ipairs(rule.triggers) do
    local tb=base+4+(t-1)*26;put(tb,tr[1]);put(tb+1,#tr[2])
    for c,cond in ipairs(tr[2]) do local cb=tb+2+(c-1)*3;put(cb,cond[1]);put(cb+1,cond[2]);put(cb+2,cond[3]) end
   end
   for e,ew in ipairs(rule.effects) do local eb=base+4+4*26+(e-1)*20;for k=1,20 do put(eb+k-1,ew[k]) end end
  end
  for i=1,#variants do for _,o in ipairs(variants[i].ops) do D.mod_registry.operation(o) end end
  for _,v in ipairs(variants) do for _,o in ipairs(v.ops) do if o.port then o.port=slot end;if o.entity then o.entity=slot end end end
  return {words=words,variants=variants,mask_variant=map,rules=#rules}
 end
 return E
end
