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
  local rules={};local list={};for _,m in ipairs(pool) do S.validate(m);assert(not rules[m.id],'duplicate modifier');rules[m.id]=m;list[#list+1]=m end
  table.sort(list,function(a,b) return a.id<b.id end)
  return setmetatable({rules=rules,list=list,seed=seed or 1,frame=0,equipped={},statuses={},recent={},players={},queue={},damage={},trace={},used=0,dropped=0,
   limit=budget,depth=depth,display={last_pulse=-30,pulse_start=-100,pulse_strength=0,trace_key='',intensity=.65}},E)
 end
 function E:random() self.seed=self.seed*48271%2147483647;return self.seed/2147483647 end
 function E:equip(port,id,tier)
  assert(type(port)=='number' and port%1==0 and port>=1 and port<=6,'port 1..6 required')
  local m=assert(self.rules[id],'unknown modifier');tier=tier or 1;assert(m.tiers[tier],'unknown tier')
  local at=self.equipped[port] or {};if m.kind=='keystone' then for other in pairs(at) do assert(self.rules[other].kind~='keystone' or other==id,'one keystone') end end
  at[id]=tier;self.equipped[port]=at
 end
 function E:status(port,name) return (self.statuses[port] or {})[name] end
 function E:clear(port)
  if port then self.equipped[port]=nil;self.statuses[port]=nil;self.recent[port]=nil;self.damage[port]=nil
  else self.equipped={};self.statuses={};self.recent={};self.damage={};self.queue={};self.trace={} end
 end
 function E:emit(e)
  assert(S.events[e.kind],'unsupported modifier event')
  e=copy(e);e.depth=e.depth or 1;e.tags=e.tags or {};e.origin=e.origin or {}
  assert(type(e.port)=='number' and e.port%1==0 and e.port>=1 and e.port<=6,'invalid event port')
  if e.target then assert(type(e.target)=='number' and e.target%1==0 and e.target>=1 and e.target<=6,'invalid event target') end
  for tag,v in pairs(e.tags) do assert(S.tags[tag] and type(v)=='boolean','unsupported event tag') end
  assert(type(e.depth)=='number' and e.depth%1==0 and e.depth>=1,'invalid event depth')
  if e.depth>self.depth or #self.queue>=128 then self.dropped=self.dropped+1;return false end
  self.queue[#self.queue+1]=e;return true
 end
 function E:begin_frame(players)
  self.frame=self.frame+1;self.players=copy(players or {});self.damage={};self.used=0
  for port=1,6 do local statuses=self.statuses[port]
   if statuses then for _,name in ipairs(ordered) do local v=statuses[name]
    if v then
     if name=='burn' and v.next_tick<=self.frame then
      self.damage[port]=(self.damage[port] or 0)+v.amount*v.stacks;v.next_tick=v.next_tick+60
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
 function E:apply(m,e,tier)
  local origin=ancestry(self,m,e)
  for _,effect in ipairs(m.effects) do
   if self.used>=self.limit then self.dropped=self.dropped+1;break end
   if not effect.when or effect.when==e.kind then
    self.used=self.used+1;local port=effect.subject=='target' and e.target or e.port
    if port and self.players[port] then
     local value=function(v) return S.resolve(v,m,tier) end;local label=m.label
     if effect.op=='status' or effect.op=='stacks' then
      local duration=math.floor(value(effect.duration));assert(duration>=1 and duration<=3600,'status duration out of bounds')
      local at=self.statuses[port] or {};self.statuses[port]=at;local v=at[effect.status];local expires=self.frame+duration
      local amount=value(effect.amount or 1)
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
     elseif effect.op=='heal' or effect.op=='damage' then
      local amount=value(effect.amount);self.damage[port]=(self.damage[port] or 0)+(effect.op=='heal' and -amount or amount)
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
  local at=1
  while at<=#self.queue do
   local e=self.queue[at];at=at+1
   if e.native_trace and #e.origin>0 then
    self.trace=copy(e.origin);self.display.trace_generation=(self.display.trace_generation or 0)+1
    -- Remember real native use of a status for later KO ancestry; no new status
    -- or gameplay effect is invented by this provenance annotation.
    for _,name in ipairs(e.native_statuses or {}) do local v=self:status(e.target,name);if v then v.origin=copy(e.origin) end end
   end
   if self.used>=self.limit then self.dropped=self.dropped+1 end
   self.recent[e.port]=self.recent[e.port] or {};self.recent[e.port][e.kind]=self.frame
   for _,m in ipairs(self.list) do local tier=(self.equipped[e.port] or {})[m.id]
    if self.used<self.limit and tier and self:matches(m,e,tier) then self:apply(m,e,tier) end
   end
   -- A lost stock ends what was happening to the fighter (statuses, stacks, recent events), not the build:
   -- equipped modifiers stay and their steady effects, looks and hit rules are derived again from them.
   if e.kind=='stock_lost' then self.statuses[e.port]=nil;self.recent[e.port]=nil;self.damage[e.port]=nil end
  end
  self.queue={}
 end
 function E:values(port)
  local out={};local function mul(key,value) out[key]=math.max(.1,math.min(4,(out[key] or 1)*value)) end
  for _,m in ipairs(self.list) do local tier=(self.equipped[port] or {})[m.id]
   if tier and m.trigger=='equip' then for _,effect in ipairs(m.effects) do if effect.op=='value' then mul(effect.key,S.resolve(effect.value,m,tier)) end end end
  end
  for _,name in ipairs(ordered) do local v=self:status(port,name)
   if v then
    if name=='chill' then mul('run_speed',.8);mul('air_speed',.8)
    elseif name=='curse' then mul('knockback_taken',1+v.amount)
    elseif name=='haste' then mul('run_speed',1.2);mul('air_speed',1.2)
    elseif name=='guarded' then mul('damage_taken',.75);mul('knockback_taken',.85) end
   end
  end
  return out
 end
 function E:native_rules(port)
  local out,bits={},0
  for _,name in ipairs(ordered) do if self:status(port,name) then bits=bits+(S.status_bits[name] or 0) end end
  for i,m in ipairs(self.list) do local tier=(self.equipped[port] or {})[m.id]
   if tier then for j,effect in ipairs(m.effects) do
    if effect.op=='convert' or effect.op=='versus-status' then
     local match,change=copy(effect.match),{}
     match.move=match.move or 'any';if effect.status then match.status_bits=S.status_bits[effect.status] end
     for k,v in pairs(effect.change) do change[k]=k=='element' and v or S.resolve(v,m,tier) end
     out[#out+1]={id=i*8+j,match=match,change=change}
    end
   end end
  end
  assert(#out<=8,'native hit rule capacity exceeded');return out,bits
 end
 function E:native_origin(ids,target)
  local out,used={},{}
  for _,id in ipairs(ids or {}) do
   if type(id)=='number' and id%1==0 then local m=self.list[math.floor((id-1)/8)];local effect=m and m.effects[(id-1)%8+1]
    if effect and (effect.op=='convert' or effect.op=='versus-status') then
     if effect.status then used[#used+1]=effect.status;for _,line in ipairs((self:status(target,effect.status) or {}).origin or {}) do append(out,line) end end
     local action=effect.change.element and 'hit converted to '..effect.change.element or effect.change.knockback_taken and 'launch boosted' or 'hit damage scaled'
     append(out,action..' by '..m.label)
    end
   end
  end
  return out,used
 end
 function E:export()
  assert(#self.queue==0,'checkpoint requires a drained event boundary')
  return C.encode{version=1,seed=self.seed,frame=self.frame,equipped=self.equipped,statuses=self.statuses,recent=self.recent,trace=self.trace,dropped=self.dropped,limit=self.limit,depth=self.depth,display=self.display}
 end
 function E:import(text)
  local at=C.decode(text);assert(at.version==1 and type(at.frame)=='number' and at.frame%1==0 and at.frame>=0,'invalid modifier snapshot')
  assert(type(at.seed)=='number' and at.seed%1==0 and at.seed>=1 and at.seed<2147483647,'invalid random state')
  assert(type(at.limit)=='number' and at.limit>=1 and at.limit<=128 and at.limit%1==0 and type(at.depth)=='number' and at.depth>=1 and at.depth<=16 and at.depth%1==0,'invalid snapshot budget')
  for p,mods in pairs(at.equipped) do assert(type(p)=='number' and p>=1 and p<=6 and p%1==0)
   for id,tier in pairs(mods) do assert(self.rules[id] and self.rules[id].tiers[tier],'unknown snapshot modifier') end
  end
  for p,statuses in pairs(at.statuses) do assert(type(p)=='number' and p>=1 and p<=6 and p%1==0)
   for name,v in pairs(statuses) do assert(S.statuses[name] and type(v)=='table' and v.stacks>=1 and v.stacks<=8 and v.stacks%1==0 and v.max>=v.stacks and v.max<=8 and v.expires>=at.frame and v.expires%1==0 and v.next_tick%1==0 and v.amount>=0 and v.amount<=100 and type(v.origin)=='table','invalid snapshot status') end
  end
  for p,events in pairs(at.recent) do assert(type(p)=='number' and p%1==0 and p>=1 and p<=6);for event,frame in pairs(events) do assert(S.events[event] and type(frame)=='number' and frame%1==0 and frame>=0 and frame<=at.frame,'invalid recent event') end end
  assert(type(at.trace)=='table' and #at.trace<=24 and type(at.dropped)=='number' and at.dropped>=0 and at.dropped%1==0,'invalid snapshot trace');for _,line in ipairs(at.trace) do assert(type(line)=='string' and #line<=160) end
  assert(type(at.display)=='table' and type(at.display.intensity)=='number' and at.display.intensity>=0 and at.display.intensity<=1,'invalid display snapshot')
  for _,key in ipairs({'last_pulse','pulse_start','pulse_strength'}) do assert(type(at.display[key])=='number','invalid visual clock') end
  assert(at.display.pulse_strength>=0 and at.display.pulse_strength<=.06 and type(at.display.trace_key)=='string','invalid visual pulse')
  if at.display.trace_generation then assert(type(at.display.trace_generation)=='number' and at.display.trace_generation%1==0 and at.display.trace_generation>=0) end
  for _,k in ipairs({'seed','frame','equipped','statuses','recent','trace','dropped','limit','depth','display'}) do self[k]=at[k] end
  self.queue={};self.damage={};self.players={};self.used=0
 end
 return E
end
