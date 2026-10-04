-- Pure bounded rule evaluation. Native adapter owns checkpointed gameplay writes.
return function(D)
 local S,C=D.mod_schema,D.mod_codec;local E={};E.__index=E
 local ordered={'burn','shock','chill','curse','haste','guarded','momentum'}
 local status_tags={burn='burning',chill='chilled',curse='cursed',haste='hasted',guarded='guarded',momentum='momentum'}
 local function copy(v) return C.decode(C.encode(v)) end
 local function append(out,text) for _,v in ipairs(out) do if v==text then return end end;if #out<24 then out[#out+1]=text end end
 function E.new(seed,pool,limits)
  seed=seed or 1;limits=limits or {};assert(type(seed)=='number' and seed%1==0 and seed>=1 and seed<2147483647,'invalid seed')
  local budget,depth=limits.budget or 64,limits.depth or 8
  assert(type(budget)=='number' and budget%1==0 and budget>=1 and budget<=128 and type(depth)=='number' and depth%1==0 and depth>=1 and depth<=16,'invalid limits')
  D.mod_budget.validate_pool(pool,{});local rules={};local list={};for _,m in ipairs(pool) do S.validate(m);assert(not rules[m.id],'duplicate modifier');rules[m.id]=m;list[#list+1]=m end
  table.sort(list,function(a,b) return a.id<b.id end)
  return setmetatable({rules=rules,list=list,seed=seed or 1,frame=0,equipped={},implicits={},statuses={},recent={},players={},queue={},damage={},trace={},used=0,dropped=0,
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
 function E:begin_frame(players)
  self.frame=self.frame+1;self.players=copy(players or {});self.damage={};self.sustain={};self.used=0
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
  for port=1,6 do if self.players[port] then self:emit{kind='interval',port=port,tags={}} end end
 end
 function E:matches(m,e,tier)
  if m.trigger~=e.kind then return false end
  if m.trigger=='interval' and self.frame%S.resolve(m.interval,m,tier)~=0 then return false end
  local own=e.self_context or self.players[e.port] or {};local target=e.target_context or self.players[e.target] or {}
  for _,c in ipairs(m.conditions or {}) do for k,v in pairs(c) do
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
    if port and self.players[port] then
     local value=function(v) return S.resolve(v,m,tier) end;local label=m.label
     if effect.op=='status' or effect.op=='stacks' then
      local duration=math.max(1,math.min(3600,math.floor(value(effect.duration)*(self:values(e.port).status_duration or 1))));assert(duration>=1 and duration<=3600,'status duration out of bounds')
      local at=self.statuses[port] or {};self.statuses[port]=at;local v=at[effect.status];local expires=self.frame+duration
      local amount=math.max(0,math.min(100,value(effect.amount or 1)))
      if v then
       v.stacks=math.min(effect.max,v.stacks+1)
       if effect.refresh=='refresh' then v.expires=expires elseif effect.refresh=='extend' then v.expires=math.min(self.frame+3600,v.expires+duration) end
       v.amount=math.max(v.amount,amount)
      else v={expires=expires,stacks=1,max=effect.max,amount=amount,next_tick=self.frame+60,origin={}};at[effect.status]=v end
      append(origin,effect.status..' applied by '..label);v.origin=copy(origin)
      self:emit{kind='status_applied',port=port,target=e.target,status=effect.status,tags={[status_tags[effect.status]]=true},depth=e.depth+1,origin=origin}
      if effect.op=='stacks' or v.stacks>1 then self:emit{kind='stacks_changed',port=port,status=effect.status,tags={[status_tags[effect.status]]=true},depth=e.depth+1,origin=origin} end
     elseif effect.op=='remove_status' then
      if self:status(port,effect.status) then self.statuses[port][effect.status]=nil;append(origin,effect.status..' spent by '..label)
       self:emit{kind='status_removed',port=port,status=effect.status,tags={},depth=e.depth+1,origin=origin}
      end
     elseif effect.op=='clank_damage' then
      if e.damage_a~=nil and e.damage_b~=nil then local amount=e.damage_a+e.damage_b;self.damage[port]=(self.damage[port] or 0)+amount;append(origin,'damage '..amount..' from '..label) end
     elseif effect.op=='heal' or effect.op=='damage' then
      local amount=math.max(0,math.min(100,value(effect.amount)));self:sustain_damage(port,effect.op=='heal' and -amount or amount)
      append(origin,(effect.op=='heal' and 'heal ' or 'damage ')..amount..' from '..label)
     elseif effect.op=='emit' then
      append(origin,'event from '..label);self:emit{kind=effect.event,port=port,target=e.target,tags={[effect.tag]=true},depth=e.depth+1,origin=origin}
     end
    end
   end
  end
  if #origin>0 then self.trace=origin;self.display.trace_generation=(self.display.trace_generation or 0)+1 end
 end
 function E:drain()
  local at,lost=1,{}
  while at<=#self.queue do
   local e=self.queue[at];at=at+1
   if not (lost[e.port] and e.depth>1) then
   if e.native_trace and #e.origin>0 then
    self.trace=copy(e.origin);self.display.trace_generation=(self.display.trace_generation or 0)+1
    -- Remember real native use of a status for later KO ancestry; no new status
    -- or gameplay effect is invented by this provenance annotation.
    for _,name in ipairs(e.native_statuses or {}) do local v=self:status(e.target,name);if v then v.origin=copy(e.origin) end end
   end
   if self.used>=self.limit then self.dropped=self.dropped+1 end
   self.recent[e.port]=self.recent[e.port] or {};self.recent[e.port][e.kind]=self.frame
   for _,m in ipairs(self.list) do local tier=(self.equipped[e.port] or {})[m.id]
    if self.used<self.limit and tier then for _,instance in ipairs(S.instances(tier))do if self:matches(m,e,instance)then self:apply(m,e,instance)end end end
   end
   -- A lost stock ends what was happening to the fighter (statuses, stacks, recent events), not the build:
   -- equipped modifiers stay and their steady effects, looks and hit rules are derived again from them.
   if e.kind=='stock_lost' then lost[e.port]=true;self.statuses[e.port]=nil;self.recent[e.port]=nil;self.damage[e.port]=nil end
   end
  end
  self.queue={}
 end
 function E:family_budget(port) return D.mod_budget.build(self.list,self.equipped[port],self.implicits[port],self.statuses[port]) end
 function E:values(port) return D.mod_budget.values(self.list,self.equipped[port],self.implicits[port],self.statuses[port]) end
 function E:native_rules(port)
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
  for key,field in pairs({damage_dealt='percent_damage',damage_taken='percent_damage',knockback_taken='launch'}) do
   local f=D.mod_budget.families[key];local budget=self:family_budget(port);local raw=budget[f].raw
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
 function E:export()
  assert(#self.queue==0,'checkpoint requires a drained event boundary')
  return C.encode{version=1,seed=self.seed,frame=self.frame,context=self.context,equipped=self.equipped,implicits=self.implicits,statuses=self.statuses,recent=self.recent,trace=self.trace,dropped=self.dropped,limit=self.limit,depth=self.depth,display=self.display}
 end
 function E:import(text)
  local at=C.decode(text);at.context=D.mod_progression.context(at.context);assert(at.version==1 and type(at.frame)=='number' and at.frame%1==0 and at.frame>=0,'invalid modifier snapshot')
  assert(type(at.seed)=='number' and at.seed%1==0 and at.seed>=1 and at.seed<2147483647,'invalid random state')
  assert(type(at.limit)=='number' and at.limit>=1 and at.limit<=128 and at.limit%1==0 and type(at.depth)=='number' and at.depth>=1 and at.depth<=16 and at.depth%1==0,'invalid snapshot budget')
  for p,mods in pairs(at.equipped) do assert(type(p)=='number' and p>=1 and p<=6 and p%1==0)
   for id,tier in pairs(mods) do assert(self.rules[id] and S.level(tier),'unknown snapshot modifier') end
  end
  at.implicits=at.implicits or {}
  for p,base in pairs(at.implicits) do assert(type(p)=='number' and p%1==0 and p>=1 and p<=6);for key,value in pairs(base) do assert(S.values[key] and type(value)=='number' and value==value and value>=-1000000 and value<=1000000,'invalid snapshot implicit') end end
  for p,statuses in pairs(at.statuses) do assert(type(p)=='number' and p>=1 and p<=6 and p%1==0)
   for name,v in pairs(statuses) do assert(S.statuses[name] and type(v)=='table' and v.stacks>=1 and v.stacks<=8 and v.stacks%1==0 and v.max>=v.stacks and v.max<=8 and v.expires>=at.frame and v.expires%1==0 and v.next_tick%1==0 and v.amount>=0 and v.amount<=100 and type(v.origin)=='table','invalid snapshot status') end
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
 return E
end
