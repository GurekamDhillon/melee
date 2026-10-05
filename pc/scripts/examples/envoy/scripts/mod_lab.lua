-- The rule host: pool, bag, slots, opponent rolls and looks, evaluated at the checkpoint boundary.
-- Two adapters install it. The LAB (debug) needs an offline LAB match and no run. A retail run (Classic /
-- Adventure, run_host() true) needs only an active offline match: same engine, same ops, same looks.
return function(D)
 local L={};L.__index=L
 local function port(n) n=tonumber(n);assert(n and n%1==0 and n>=1 and n<=6,'port 1..6 required');return n end
 function L.new(g,options)
  local self=setmetatable({g=g,options=options or {},enabled=false,owned={},hit_owned={},pending={},observed={},debug_equipped={}},L)
  self.engine=D.mod_engine.new(104729,D.mod_pool)
  self.display=D.mod_display.new(g,self.engine)
  if D.mod_echo_lab then self.echoes=D.mod_echo_lab.new(self);g.command('echo',function(arg)return self.echoes:command(arg or '')end,'add <delay> [move] [scale] | clear') end
  if D.drive_lab then self.drives=D.drive_lab.new(g,self) end
  if D.foe_lab then self.foes=D.foe_lab.new(g,self) end
  g.command('mod',function(arg) return self:command(arg or '') end,'list | add <id> [port] | clear | trace | intensity <0..1>')
  g.command('depth',function(arg) return self:depth_command(arg or '') end,'<nonnegative depth> [New Game+ loop]')
  return self
 end
 function L:hosted() return self.options.run_host~=nil and self.options.run_host()==true end
 function L:allowed()
  local m=self.g.match()
  if self:hosted() then
   if not m or not m.active or m.netplay then return false,'offline active match required' end
  else
   if not m or not m.active or m.netplay or not self.g.lab_mode or not self.g.lab_mode() then return false,'offline active LAB match required' end
   if self.options.blocked and self.options.blocked() then return false,'stop the Envoy run/director before using LAB modifiers' end
  end
  if not self.g.sim_supported or not self.g.sim_commit or not self.g.sim_clear then return false,'modifier checkpoint engine unavailable' end
  if not self.g.hit_rule_add or not self.g.fighter_status then return false,'native hit-rule engine unavailable; rebuild required' end
  local caps=self.g.hit_rules and self.g.hit_rules(1) -- one table-building call, both flags read from it
  if not caps or caps.percent_only~=true then return false,'percent-only hit-rule engine unavailable; rebuild required' end
  if caps.progression~=true then return false,'progression hit-rule engine unavailable; rebuild required' end
  return true
 end
 function L:replaying() return self.g.sim_replaying and self.g.sim_replaying() end
 -- A retail run keeps its bag, slots and progression across stages (persistent build); everything a stage
 -- produced (statuses, history, ground drops, opponent rolls, looks) is transient and is rebuilt. `full`
 -- ends the run's build too. The LAB is a debug sandbox and always clears fully.
 function L:reset(full)
  local keep=not full and self:hosted() and self.drives
  if self.echoes then self.echoes:reset() end
  if self.foes then self.foes:reset() end
  if keep then self.drives:soft_clear() elseif self.drives then self.drives:clear() end
  local ctx=self.drives and self.drives.bag.context
  self.engine=D.mod_engine.new(104729,D.mod_pool,{context=ctx});self.enabled=false;self.owned={};self.hit_owned={};self.pending={};self.observed={};self.debug_equipped={}
  self.display:clear();self.display.engine=self.engine
  if self.drives and (self.drives:has_build() or #self.drives.bag.items>0) then self.enabled=true end
 end
 function L:depth_command(arg)
  local ok,why=pcall(function()
   local allowed,reason=self:allowed();assert(allowed,reason);assert(not self:replaying(),'depth edit refused during rewind')
   local w={};for v in arg:gmatch('%S+') do w[#w+1]=v end
   assert(#w==1 or #w==2,'usage: depth <n> [loop]');local ctx=D.mod_progression.context(assert(tonumber(w[1]),'numeric depth required'),w[2] and assert(tonumber(w[2]),'numeric loop required') or 0)
   assert(#self.engine.queue==0,'wait for combat events to commit before changing depth')
   local probe=D.mod_engine.new(self.engine.seed,D.mod_pool);probe:import(self.engine:export());probe:set_context(ctx)
   for _,e in ipairs(self.pending) do probe:equip(e.port,e.id) end
   if self.foes then
    local function check(r)
     local build=D.mod_codec.decode(D.mod_codec.encode(r.build));build.context=ctx
     local bag=D.drive_bag.new(self.foes.roller.loot,{context=ctx});local mods,implicit=bag:validate(build);probe:set_build(r.port,mods,implicit)
    end
    for _,r in pairs(self.foes.builds) do check(r) end
    for _,e in ipairs(self.foes.pending) do if e.op=='roll' then check(e.record) end end
   end
   if self.drives then
    local state=self.drives:snapshot();state.bag.context=ctx
    self.drives:validate(state,{engine=probe,debug_equipped=self.debug_equipped,pending=self.pending})
   end
   local drives=self.drives and self.drives:snapshot();if drives then drives.bag.context=ctx end
   self:prospective(probe,self.debug_equipped,self.pending,nil,drives)
   -- All roots (including queued edits) passed; physical rolls keep their tiers.
   self.engine.context=ctx;if self.drives then self.drives.bag.context=D.mod_progression.context(ctx) end
   self.enabled=true;if self.options.activate then self.options.activate() end
   self.g.log(('depth: %d / loop %d / effective %d'):format(ctx.depth,ctx.loop,D.mod_progression.effective(ctx)))
  end)
  if not ok then self.g.log('depth: refused '..tostring(why));return false,why end;return true
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
     for _,effect in ipairs(self.engine.rules[w[2]].effects)do if effect.op=='echo' then assert(self.echoes and self.echoes:capable(),'native echo journal unavailable; rebuild required') end end
     if p>1 and self.foes then for _,e in ipairs(self.foes.pending) do assert(e.op~='roll','wait for pending foe edits to commit') end end
     -- Validate the command without mutating the live rule root before warmup.
     local pending=D.mod_codec.decode(D.mod_codec.encode(self.pending));pending[#pending+1]={port=p,id=w[2]}
     self:prospective(self.engine,self.debug_equipped,pending,self.foes,self.drives and self.drives:snapshot())
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
  local origin,used=self.engine:native_origin(ids,victim,attacker)
  if attacker and self.foes and self.foes.builds[attacker] then table.insert(origin,1,'Envoy foe P'..attacker) end
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
 function L:pickup_expire(e) if self.drives and not self:replaying() then self.drives:expire(e) end end
 function L:pickup(e) if self.drives then self.drives:pickup(e) end;if e.port then self:event{kind='item_pickup',port=e.port,tags={}} end end
 function L:sample()
  local players,life={},{}
  for p=1,6 do local v=self.g.player(p)
   if v then players[p]={percent=v.percent or 0,grounded=not v.airborne,stocks=v.stocks or 0}
    life[p]={falls=v.falls or 0,stocks=v.stocks or 0,char=v.char or -1,action=v.action or 14,entity_ref=v.entity_ref}
   end
  end
  return players,life
 end
 function L:export()
  return D.mod_codec.encode{version=1,echoes=self.echoes and self.echoes:snapshot(true),foes=self.foes and self.foes:snapshot(true),debug_equipped=self.debug_equipped,drives=self.drives and self.drives:snapshot(true),engine=self.engine:export(),enabled=self.enabled,owned=self.owned,hit_owned=self.hit_owned,pending=self.pending,observed=self.observed}
 end
 function L:check_echo_capacity(engine,manual)
  if not self.echoes then return end;manual=manual or self.echoes.manual
  for p=1,6 do local desc=engine:echo_description(p)
   assert(#desc.rules+#(manual[p] or {})<=8,'eight echo rules per fighter maximum')
   if #desc.copies>0 or #(manual[p] or {})>0 then assert(self.echoes:capable(),'native echo journal unavailable; rebuild required')end
  end
 end
 function L:prospective(engine,debug,pending,foes,drives,players,manual)
  local function clone(v) return D.mod_codec.decode(D.mod_codec.encode(v)) end
  manual=manual or (self.echoes and self.echoes.manual) or {}
  local probe=D.mod_engine.new(engine.seed,D.mod_pool,{context=engine.context})
  probe.equipped=clone(engine.equipped);probe.implicits=clone(engine.implicits);probe.statuses=clone(engine.statuses);debug=clone(debug)
  local foe_ports={};for p in pairs(foes and foes.builds or {}) do foe_ports[p]=true end
  -- Match actual publication order, including rolls subsequently retired by clear.
  for _,e in ipairs(foes and foes.pending or {}) do
   if e.op=='roll' then
    if players then self.foes:cpu(e.record.port) end
    local mods,implicit=self.foes.roller:validate(e.record);probe:set_build(e.record.port,mods,implicit);foe_ports[e.record.port]=true
   elseif e.op=='clear' then
    for p in pairs(foe_ports) do probe:clear(p);debug[p]=nil end;foe_ports={}
    for _,at in pairs(probe.statuses) do for name,v in pairs(at) do for _,line in ipairs(v.origin or {}) do if line:match('^Envoy foe P[2-6]$') then at[name]=nil;break end end end end
   end
  end
  for _,e in ipairs(pending) do if not players or players[e.port] then probe:equip(e.port,e.id) end end
  local staged
  if drives then
   local roots={engine=probe,debug_equipped=debug,pending=pending,echo_manual=manual,check_echo_capacity=function(_,e)self:check_echo_capacity(e,manual)end};staged=self.drives:validate(drives,roots)
   local draft=D.drive_bag.new(self.drives.loot);assert(draft:restore(staged.bag))
   for _,e in ipairs(staged.pending or {}) do assert(draft[e.op](draft,e.a,e.b)) end
   if #(staged.pending or {})>0 or next(draft.equipped) or draft.keystone or #(draft.keystones or {})>0 then
    local mods,implicit=draft:derive();probe:set_build(1,self.drives:combined(mods,roots),implicit)
   end
  end
  for p=1,6 do probe:native_rules(p) end;self:check_echo_capacity(probe,manual)
  return staged
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
  local probe=D.mod_engine.new(self.engine.seed,D.mod_pool);probe:import(s.engine)
  local foes=self.foes and s.foes and self.foes:validate(s.foes)
  assert(type(s.pending)=='table' and type(s.owned)=='table' and type(s.observed)=='table','invalid adapter roots')
  for p,v in pairs(s.observed) do
   port(p);assert(type(v)=='table','invalid observed fighter')
   if v.entity_ref then
    assert(self.g.entity_valid and self.g.entity_resolve,'entity reference validation unavailable; rebuild required')
    assert(self.g.entity_valid(v.entity_ref),'stale checkpoint fighter reference')
    local ref=self.g.entity_resolve(v.entity_ref);assert(ref and ref.kind=='fighter' and ref.port==p and not ref.sub,'checkpoint fighter binding mismatch')
   end
  end
  local n=0;for i,e in pairs(s.pending) do
   assert(type(i)=='number' and i%1==0 and i>=1 and i<=12 and type(e)=='table','invalid pending equip')
   for k in pairs(e) do assert(k=='port' or k=='id','unknown pending equip field') end
   port(e.port);assert(probe.rules[e.id],'unknown pending modifier');n=n+1
  end;assert(n==#s.pending,'sparse pending equips')
  for _,root in ipairs({s.owned,s.hit_owned or {}}) do for p,v in pairs(root) do port(p);assert(v==true,'invalid ownership') end end
  local debug=s.debug_equipped or probe.equipped;for p,mods in pairs(debug) do local check=D.mod_engine.new(1,D.mod_pool,{context=probe.context});check:set_build(port(p),mods,{}) end
  local echoes=self.echoes and self.echoes:validate(s.echoes or {manual={},owned={}})
  local drives=self:prospective(probe,debug,s.pending,foes,self.drives and s.drives,nil,echoes and echoes.manual or {})
  if self.echoes then self.echoes:restore(echoes) end
  self.debug_equipped=debug;self.pending=s.pending
  if drives then self.drives:publish(drives) end
  self.engine:import(s.engine);self.enabled=s.enabled;self.owned=s.owned;self.hit_owned=s.hit_owned or {};self.pending=s.pending;self.observed=s.observed
  if self.foes then if foes then self.foes:restore(foes) else self.foes:reset() end end
  if self.enabled then self.display:on_loadstate(self.engine) else self.display:clear() end
 end
 function L:frame()
  if self:replaying() then return true end
  -- In a run the host waits for the stage's entrance to finish before it draws or commits anything.
  if self.options.run_ready and self:hosted() and not self.options.run_ready() then return false end
  if not self.enabled and not next(self.owned) and not next(self.hit_owned) then return false end
  local allowed=self:allowed()
  if not allowed then
   if not (self.g.match() or {}).netplay and self.g.sim_clear then self.g.sim_clear() end
   self:reset();return false
  end
  local players,life=self:sample()
  local ready=self.display:warm(self.engine,players)
  -- The staged-edit publication (validate the whole prospective state, then apply it) is only needed when an
  -- edit is staged or the bag's derived build is stale; an unchanged, already validated state is not
  -- re-validated every frame.
  local staged=#self.pending>0 or (self.foes and #self.foes.pending>0) or (self.drives and (#self.drives.pending>0 or self.drives:stale()))
  if ready and staged then
   local bag=self.drives and self.drives.bag:snapshot();local debug=D.mod_codec.decode(D.mod_codec.encode(self.debug_equipped))
   local accepted,why=pcall(function()
    self:prospective(self.engine,self.debug_equipped,self.pending,self.foes,self.drives and self.drives:snapshot(),players)
    if self.foes then self.foes:apply() end
    for _,e in ipairs(self.pending) do if players[e.port] then self.engine:equip(e.port,e.id);self.debug_equipped[e.port]=self.debug_equipped[e.port] or {};self.debug_equipped[e.port][e.id]=1 end end
    self.pending={}
    if self.drives and (#self.drives.pending>0 or self.drives:has_build()) then self.drives:apply() end
   end)
   if not accepted then
    pcall(self.g.sim_clear);if self.echoes then self.echoes:reset() end;self.display:clear();self.engine=D.mod_engine.new(104729,D.mod_pool,{context=self.engine.context});self.display.engine=self.engine
    self.enabled=false;self.owned={};self.hit_owned={};self.pending={};self.observed={};self.debug_equipped=debug
    if self.foes then self.foes:reset() end
    if self.drives then self.drives.bag.items=bag.items;self.drives.bag.equipped=bag.equipped;self.drives.bag.keystone=bag.keystone;self.drives.bag.keystones=bag.keystones or {};self.drives.bag.context=bag.context;self.drives.pending={} end
    self.g.log('mod: disabled after pending publication refusal '..tostring(why));return true
   end
  end
  if self.foes then self.foes:frame() end
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
   if not now or before and (now.char~=before.char or before.entity_ref and now.entity_ref~=before.entity_ref or (now.action==12 or now.action==13) and before.action~=12 and before.action~=13) then
    stock_queued[p]=true;self.engine.statuses[p]=nil;self.engine.recent[p]=nil;self.engine.damage[p]=nil;self.display:clear(p)
   end
  end
  self.observed=life
  local ops,new_owned,new_hit_owned={},{},{}
  for p=1,6 do
   local values=self.engine:values(p);values.status_duration=nil;values.damage_dealt=nil;values.damage_taken=nil;values.knockback_taken=nil
   if players[p] and next(values) then ops[#ops+1]={op='fighter_mod',port=p,values=values};new_owned[p]=true
   elseif self.owned[p] then ops[#ops+1]={op='fighter_mod',port=p} end
   local rules,bits=self.engine:native_rules(p)
   if players[p] and (#rules>0 or bits~=0) then
    ops[#ops+1]={op='hit_rules',port=p,rules=rules,status_bits=bits};new_hit_owned[p]=true
   elseif self.hit_owned[p] then ops[#ops+1]={op='hit_rules',port=p,rules={},status_bits=0} end
   local damage=self.engine.damage[p]
   if players[p] and damage and damage~=0 then ops[#ops+1]={op='damage',port=p,value=math.max(0,math.min(999,players[p].percent+damage))} end
  end
  if self.echoes then self.echoes:ops(ops,players,stock_queued) end
  self.owned=new_owned;self.hit_owned=new_hit_owned
  for p=1,6 do if self.engine.statuses[p] and not next(self.engine.statuses[p]) then self.engine.statuses[p]=nil end end
  self.enabled=(self.echoes and self.echoes:active()) or D.mod_progression.effective(self.engine.context)>0 or (self.foes and (#self.foes.pending>0 or next(self.foes.builds)~=nil)) or (self.drives and (#self.drives.pending>0 or self.drives.drops:count()>0 or #self.drives.bag.items>0 or self.drives:has_build())) or #self.pending>0 or next(self.engine.equipped)~=nil or next(self.engine.statuses)~=nil
  -- Visual pulse/cooldown metadata is pure state and belongs in the checkpoint.
  if self.enabled then self.display:update(self.engine,players) else self.display:clear() end
  local committed,why=pcall(function() return self.g.sim_commit(self:export(),ops) end)
  if not committed or not why then
   -- Validation/allocation failures are atomic natively. Retire old effects
   -- too, rather than letting mutated Lua timers diverge from the game.
   local cleared,detail=pcall(self.g.sim_clear)
   self:reset()
   self.g.log('mod: disabled after checkpoint refusal '..tostring(why)..(cleared and '' or '; clear refused '..tostring(detail)))
  end
  if committed and why and self.echoes then self.echoes:present(players) end
  return true
 end
 function L:tick()
  if self.drives then self.drives:tick() end
  if self.enabled and self:allowed() and not self:replaying() and self.display.tick then self.display:tick(self.engine) end
  return self.drives and self.drives.menu.active or false
 end
 function L:scene() if D.menu_input then D.menu_input.reset() end;self:reset();self.display=D.mod_display.new(self.g,self.engine) end -- scene invalidates shader handles
 function L:draw()
  if self.drives and self.drives.menu.active then self.drives:draw();return end
  local plate=self.foes and self.g.kit and next(self.foes.labels)~=nil
  -- The Modifier LAB debug text (ids, last chain) is a LAB tool: a run's own strip and plates replace it.
  if plate then self.foes:draw() elseif self.enabled and not self:hosted() then self.display:draw(self.engine) end
  if self.drives and not plate then self.drives:draw() end
 end
 function L:unload()
  if self.g.sim_clear and not (self.g.match() or {}).netplay then self.g.sim_clear() end
  self:reset(true)
 end
 -- Run adapter entry points (called by run_host.lua at the director's lifecycle boundaries).
 function L:run_end()
  if self.g.sim_clear and not (self.g.match() or {}).netplay then pcall(self.g.sim_clear) end
  self:reset(true)
 end
 function L:set_context(ctx)
  ctx=D.mod_progression.context(ctx.depth,ctx.loop)
  self.engine.context=ctx;if self.drives then self.drives.bag.context=D.mod_progression.context(ctx) end
 end
 return L
end
