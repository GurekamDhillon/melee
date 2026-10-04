-- Dormant LAB debug adapter. Combat hooks only enqueue pure-data events.
return function(D)
 local L={};L.__index=L
 local function port(n) n=tonumber(n);assert(n and n%1==0 and n>=1 and n<=6,'port 1..6 required');return n end
 function L.new(g,options)
  local self=setmetatable({g=g,options=options or {},enabled=false,owned={},hit_owned={},pending={},observed={},debug_equipped={}},L)
  self.engine=D.mod_engine.new(104729,D.mod_pool)
  self.display=D.mod_display.new(g,self.engine)
  if D.drive_lab then self.drives=D.drive_lab.new(g,self) end
  g.command('mod',function(arg) return self:command(arg or '') end,'list | add <id> [port] | clear | trace | intensity <0..1>')
  return self
 end
 function L:allowed()
  local m=self.g.match()
  if not m or not m.active or m.netplay or not self.g.lab_mode or not self.g.lab_mode() then return false,'offline active LAB match required' end
  if self.options.blocked and self.options.blocked() then return false,'stop the Envoy run/director before using LAB modifiers' end
  if not self.g.sim_supported or not self.g.sim_commit or not self.g.sim_clear then return false,'modifier checkpoint engine unavailable' end
  if not self.g.hit_rule_add or not self.g.fighter_status then return false,'native hit-rule engine unavailable; rebuild required' end
  return true
 end
 function L:replaying() return self.g.sim_replaying and self.g.sim_replaying() end
 function L:reset()
  if self.drives then self.drives:clear() end
  self.engine=D.mod_engine.new(104729,D.mod_pool);self.enabled=false;self.owned={};self.hit_owned={};self.pending={};self.observed={};self.debug_equipped={}
  self.display:clear();self.display.engine=self.engine
  if self.drives and (self.drives:has_build() or #self.drives.bag.items>0) then self.enabled=true end
 end
 function L:command(arg)
  local ok,result=pcall(function()
   local w={};for word in arg:gmatch('%S+') do w[#w+1]=word end
   if w[1]=='list' then
    assert(#w==1,'usage: mod list')
    for _,m in ipairs(self.engine.list) do self.g.log('mod '..m.id..': '..D.mod_schema.tooltip(m,1)) end
   elseif w[1]=='trace' then
    assert(#w==1,'usage: mod trace');self.g.log(#self.engine.trace>0 and table.concat(self.engine.trace,', ') or 'mod: no chain yet')
   elseif w[1]=='intensity' then
    assert(#w==2,'usage: mod intensity <0..1>');local n=tonumber(w[2]);assert(n and n>=0 and n<=1,'intensity 0..1 required')
    self.display:intensity(n);self.engine.display.intensity=n
   else
    local allowed,why=self:allowed();assert(allowed,why);assert(not self:replaying(),'modifier edit refused during rewind')
    if w[1]=='add' then
     assert(#w==2 or #w==3,'usage: mod add <id> [port]');local p=port(w[3] or 1)
     assert(self.engine.rules[w[2]],'unknown modifier; mod list');assert(self.g.player(p),'fighter absent')
     -- Validate the command without mutating the live rule root before warmup.
     local probe=D.mod_engine.new(self.engine.seed,D.mod_pool);probe:import(self.engine:export());for _,e in ipairs(self.pending) do probe:equip(e.port,e.id) end;probe:equip(p,w[2]);if self.drives and p==1 then local mods,implicit=self.drives:view():derive();mods=self.drives:combined(mods);mods[w[2]]=math.max(mods[w[2]] or 0,1);probe:set_build(1,mods,implicit) end
     local ready,detail=self.display:warm(self.engine)
     assert(not self.display.error,detail or 'modifier shader warmup unavailable')
     assert(#self.pending<12,'modifier pending equip budget exhausted')
     self.pending[#self.pending+1]={port=p,id=w[2]};self.enabled=true
     if self.options.activate then self.options.activate() end
     self.g.log('mod: queued '..w[2]..' on P'..p..(ready and '' or ' (waiting for shader warmup)'))
    elseif w[1]=='clear' then
     assert(#w==1,'usage: mod clear')
     self.g.sim_clear();self:reset()
     self.g.log('mod: cleared')
    else error('usage: mod list|add <id> [port]|clear|trace|intensity <0..1>') end
   end
   return true
  end)
  if not ok then self.g.log('mod: refused '..tostring(result));return false,result end
  return result
 end
 function L:event(e)
  if not self.enabled or self:replaying() or not self:allowed() then return end
  if type(e.port)~='number' or e.port<1 or e.port>6 then return end
  self.engine:emit(e)
 end
 local function context(g,p,damage,grounded)
  local player=p and g.player(p);local out={}
  if type(damage)=='number' then out.percent=damage end
  if type(grounded)=='boolean' then out.grounded=grounded end
  if player then out.stocks=player.stocks end
  return out
 end
 function L:hit(attacker,victim,e)
  if not self.enabled or self:replaying() or not self:allowed() or not victim then return end
  e=e or {};local valid=e.context_valid==true
  local tags={}
  if valid then
   for _,tag in ipairs({e.move_tag or '',e.element_tag or ''}) do if D.mod_schema.tags[tag] then tags[tag]=true end end
   if type(e.attacker_grounded)=='boolean' then tags[e.attacker_grounded and 'grounded' or 'airborne']=true end
  end
  local ad,vd,ag,vg
  if valid then ad,vd,ag,vg=e.attacker_damage,e.victim_damage,e.attacker_grounded,e.victim_grounded end
  local attacker_context=context(self.g,attacker,ad,ag)
  local victim_context=context(self.g,victim,vd,vg)
  local ids,owner={},nil
  if valid and self.g.hit_rules then
   for p=1,6 do if self.hit_owned[p] then owner=self.g.hit_rules(p).owner;break end end
   for i,id in ipairs(e.hit_rule_ids or {}) do
    if owner and owner~=0 and (e.hit_rule_owners or {})[i]==owner then ids[#ids+1]=id end
   end
  end
  local origin,used=self.engine:native_origin(ids,victim)
  if attacker then self:event{kind='hit_dealt',port=attacker,target=victim,tags=tags,
   self_context=attacker_context,target_context=victim_context,origin=origin,native_trace=true,native_statuses=used} end
  local taken={};for key,value in pairs(tags) do if key~='grounded' and key~='airborne' then taken[key]=value end end
  if valid and type(e.victim_grounded)=='boolean' then taken[e.victim_grounded and 'grounded' or 'airborne']=true end
  self:event{kind='hit_taken',port=victim,target=attacker,tags=taken,self_context=victim_context,target_context=attacker_context,origin=origin,native_trace=not attacker}
 end
 function L:ko(attacker,victim)
  if attacker then self:event{kind='ko_dealt',port=attacker,target=victim,tags={}} end
 end
 function L:stock_lost(p) self:event{kind='stock_lost',port=p,tags={}} end
 function L:action(kind,p,_,sub) if not sub then self:event{kind=kind,port=p,tags={}} end end
 function L:clank(e)
  local a,b=e.port_a,e.port_b
  if type(a)~='number' or type(b)~='number' or a%1~=0 or b%1~=0 or a<1 or a>6 or b<1 or b>6 or a==b then return end
  local function damage(n) if type(n)=='number' and n==n and n>=0 and n<=100000 then return n end end
  local da,db=damage(e.damage_a),damage(e.damage_b)
  self:event{kind='clank',port=a,target=b,tags={},damage_a=da,damage_b=db}
  self:event{kind='clank',port=b,target=a,tags={},damage_a=da,damage_b=db}
 end
 function L:pickup_expire(e) if self.drives and not self:replaying() then self.drives.drops:expire(e) end end
 function L:pickup(e) if self.drives then self.drives:pickup(e) end;if e.port then self:event{kind='item_pickup',port=e.port,tags={}} end end
 function L:sample()
  local players,life={},{}
  for p=1,6 do local v=self.g.player(p)
   if v then players[p]={percent=v.percent or 0,grounded=not v.airborne,stocks=v.stocks or 0}
    life[p]={falls=v.falls or 0,stocks=v.stocks or 0,char=v.char or -1,action=v.action or 14}
   end
  end
  return players,life
 end
 function L:export()
  return D.mod_codec.encode{version=1,debug_equipped=self.debug_equipped,drives=self.drives and self.drives:snapshot(),engine=self.engine:export(),enabled=self.enabled,owned=self.owned,hit_owned=self.hit_owned,pending=self.pending,observed=self.observed}
 end
 function L:loadstate()
  local blob=self.g.sim_read and self.g.sim_read()
  if not blob then
   self:reset()
   -- Native clear branches only if this owner has storage/overlays, so rewind
   -- before first equip preserves untouched future history. Old generations
   -- can restore numeric-slot overlays; those must be retired immediately.
   if self.g.sim_clear and not (self.g.match() or {}).netplay then self.g.sim_clear() end
   return
  end
  local s=D.mod_codec.decode(blob);assert(s.version==1 and type(s.enabled)=='boolean','invalid LAB modifier checkpoint')
  self.debug_equipped=s.debug_equipped or s.engine.equipped or {};self.pending=s.pending
  if self.drives and s.drives then self.drives:restore(s.drives) end
  self.engine:import(s.engine);self.enabled=s.enabled;self.owned=s.owned;self.hit_owned=s.hit_owned or {};self.pending=s.pending;self.observed=s.observed
  if self.enabled then self.display:on_loadstate(self.engine) else self.display:clear() end
 end
 function L:frame()
  if self:replaying() then return true end
  if not self.enabled and not next(self.owned) and not next(self.hit_owned) then return false end
  local allowed=self:allowed()
  if not allowed then
   if not (self.g.match() or {}).netplay and self.g.sim_clear then self.g.sim_clear() end
   self:reset();return false
  end
  local players,life=self:sample()
  local ready=self.display:warm(self.engine)
  if ready then
   for _,e in ipairs(self.pending) do if players[e.port] then self.engine:equip(e.port,e.id);self.debug_equipped[e.port]=self.debug_equipped[e.port] or {};self.debug_equipped[e.port][e.id]=1 end end
   self.pending={}
   if self.drives and (#self.drives.pending>0 or next(self.drives.bag.equipped) or self.drives.bag.keystone) then self.drives:apply() end
  end
  if self.drives then self.drives:frame() end
  local stock_queued={};for _,e in ipairs(self.engine.queue) do if e.kind=='stock_lost' then stock_queued[e.port]=true end end
  for p=1,6 do local before,now=self.observed[p],life[p]
   if before and now and (now.falls>before.falls or now.stocks<before.stocks) and not stock_queued[p] then self:stock_lost(p);stock_queued[p]=true end
  end
  self.engine:begin_frame(players);self.engine:drain()
  for p=1,6 do if stock_queued[p] then
   self.display:clear(p);self.engine.display.pulse_start=-100;self.engine.display.pulse_strength=0
  end end
  for p=1,6 do local before,now=self.observed[p],life[p]
   if not now or before and (now.char~=before.char or (now.action==12 or now.action==13) and before.action~=12 and before.action~=13) then
    self.engine.statuses[p]=nil;self.engine.recent[p]=nil;self.engine.damage[p]=nil;self.display:clear(p)
   end
  end
  self.observed=life
  local ops,new_owned,new_hit_owned={},{},{}
  for p=1,6 do
   local values=self.engine:values(p);values.status_duration=nil
   if players[p] and next(values) then ops[#ops+1]={op='fighter_mod',port=p,values=values};new_owned[p]=true
   elseif self.owned[p] then ops[#ops+1]={op='fighter_mod',port=p} end
   local rules,bits=self.engine:native_rules(p)
   if players[p] and (#rules>0 or bits~=0) then
    ops[#ops+1]={op='hit_rules',port=p,rules=rules,status_bits=bits};new_hit_owned[p]=true
   elseif self.hit_owned[p] then ops[#ops+1]={op='hit_rules',port=p,rules={},status_bits=0} end
   local damage=self.engine.damage[p]
   if players[p] and damage and damage~=0 then ops[#ops+1]={op='damage',port=p,value=math.max(0,math.min(999,players[p].percent+damage))} end
  end
  self.owned=new_owned;self.hit_owned=new_hit_owned
  for p=1,6 do if self.engine.statuses[p] and not next(self.engine.statuses[p]) then self.engine.statuses[p]=nil end end
  self.enabled=(self.drives and (#self.drives.pending>0 or self.drives.drops:count()>0 or #self.drives.bag.items>0)) or #self.pending>0 or next(self.engine.equipped)~=nil or next(self.engine.statuses)~=nil
  -- Visual pulse/cooldown metadata is pure state and belongs in the checkpoint.
  if self.enabled then self.display:update(self.engine) else self.display:clear() end
  local committed,why=pcall(function() return self.g.sim_commit(self:export(),ops) end)
  if not committed or not why then
   -- Validation/allocation failures are atomic natively. Retire old effects
   -- too, rather than letting mutated Lua timers diverge from the game.
   local cleared,detail=pcall(self.g.sim_clear)
   self:reset()
   self.g.log('mod: disabled after checkpoint refusal '..tostring(why)..(cleared and '' or '; clear refused '..tostring(detail)))
  end
  return true
 end
 function L:tick()
  if self.drives then self.drives:tick() end
  if self.enabled and self:allowed() and not self:replaying() and self.display.tick then self.display:tick(self.engine) end
  return self.drives and self.drives.menu.active or false
 end
 function L:scene() self:reset();self.display=D.mod_display.new(self.g,self.engine) end -- scene invalidates shader handles
 function L:draw() if self.enabled then self.display:draw(self.engine) end;if self.drives then self.drives:draw() end end
 function L:unload()
  if self.g.sim_clear and not (self.g.match() or {}).netplay then self.g.sim_clear() end
  self:reset()
 end
 return L
end
