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
  self.tech={crit={},caps={},armor={},timed={},shock={}}
  if D.earned_fx then self.fx=D.earned_fx.new(self);g.command('critfx',function(arg) return self.fx:command(arg or '') end,'preview <0..1> | intensity <0..1> | off | (status)') end
  -- A read-only probe of the technique layer for tests and the owner: statuses with their cause, what was last written natively, the crit and presentation counters.
  g.command('techprobe',function(arg)
   if arg=='cost reset' then self.cost={} return true end
   if arg=='cost' then
    for name,r in pairs(self.cost or {}) do local n=math.min(r.n,240);local sum,sorted=0,{};for i=1,n do sum=sum+r[i];sorted[i]=r[i] end;table.sort(sorted)
     g.log(('techprobe cost: %s n=%d mean=%.3f ms p95=%.3f ms max=%.3f ms'):format(name,r.n,n>0 and sum/n or 0,sorted[math.max(1,math.ceil(n*.95))] or 0,r.max)) end
    return true
   end
   if arg=='watch' then self.watch=true;return true end
   if arg=='armor' then for p=1,6 do if g.player(p) then local rows=g.fighter_armor and g.fighter_armor(p);local t={};for _,r in ipairs(rows or {}) do t[#t+1]=('%s v=%s left=%s enabled=%s'):format(r.type,tostring(r.value),tostring(r.remaining),tostring(r.enabled)) end;g.log('techprobe armor P'..p..': '..table.concat(t,' | ')) end end;return true end
   local out={'techprobe frame='..tostring(self.engine.frame)..' enabled='..tostring(self.enabled)}
   for p=1,6 do
    local at=self.engine.statuses[p];local names={};for name,v in pairs(at or {}) do names[#names+1]=name..'('..tostring(v.cause or '-')..','..tostring(v.expires-self.engine.frame)..')' end
    table.sort(names);local cfg=self.engine:crit_config(p)
    if #names>0 or cfg or self.tech.caps[p] or self.tech.armor[p] then out[#out+1]=('  P%d statuses=%s crit=%s caps=%s armor=%s earned=%s'):format(p,table.concat(names,','),cfg and ('%.2f x%.2f floor %d'):format(cfg.slots.default.chance,cfg.slots.default.multiplier,cfg.min_percent) or '-',tostring(self.tech.caps[p] and 'on'),tostring(self.tech.armor[p]),tostring(self.engine:earned(p) and self.engine:earned(p).cause))end
   end
   if self.fx then local st=self.fx.stat;out[#out+1]=('  fx crits=%d passes=%d restarts=%d tracers=%d toasts=%d bound=%d'):format(st.crits,st.passes,st.restarts,st.tracers,st.toasts,st.bound or 0) end
   for _,l in ipairs(out) do g.log(l) end;return true
  end,'log the technique layer state')
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
 -- sim_clear does not release every native technique state (caps, typed armour, crit configuration, Shock stay with their owner),
 -- so a reset releases what this script wrote, by the documented direct calls (offline only; a missing fighter is ignored).
 function L:release_native()
  local g=self.g;local t=self.tech;if not t then return end
  local m=g.match and g.match();if not m or m.netplay then return end
  for p=1,6 do
   if t.caps[p] and g.fighter_caps then pcall(g.fighter_caps,p,nil) end
   if t.armor[p] and g.fighter_armor then pcall(g.fighter_armor,p,{type=t.armor[p]:match('^[a-z_]+'),clear=true}) end
   if t.crit[p] and g.crit then pcall(g.crit,p,nil) end
   if t.shock[p] and g.shock then pcall(g.shock,p,nil) end
  end
 end
 function L:reset(full)
  self:release_native()
  local keep=not full and self:hosted() and self.drives
  if self.echoes then self.echoes:reset() end
  if self.foes then self.foes:reset() end
  if keep then self.drives:soft_clear() elseif self.drives then self.drives:clear() end
  local ctx=self.drives and self.drives.bag.context
  self.engine=D.mod_engine.new(104729,D.mod_pool,{context=ctx});self.enabled=false;self.owned={};self.hit_owned={};self.pending={};self.observed={};self.debug_equipped={}
  self.tech={crit={},caps={},armor={},timed={},shock={}};if self.fx then self.fx:reset() end
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
 -- Engine skill events (on_skill): the rules react to the engine's own decisions, never to raw inputs. perfect_shield arrives by the
 -- legacy hook (L:action), so the skill copy of it is ignored here. A combo belongs to its attacker.
 function L:skill(e)
  if type(e)~='table' or e.kind=='perfect_shield' or e.subfighter then return end
  if not self.enabled or self:replaying() or not self:allowed() or not D.mod_skill.is_skill(e.kind) then return end
  local port,target=e.port,nil
  if e.kind=='combo' or e.kind=='combo_end' then port,target=e.attacker,e.port end
  if type(port)~='number' or port<1 or port>6 or not self.engine:listens(port,e.kind) then return end
  local own=context(self.g,port)
  if self.g.skill_state and type(e.entity)=='number' then
   local ok,st=pcall(self.g.skill_state,e.entity)
   if ok and type(st)=='table' then if type(st.air_frames)=='number' then own.air_frames=st.air_frames end;own.aerial_hit=st.aerial_hit==true;if type(st.combo_count)=='number' then own.combo_count=st.combo_count end end
  end
  local ev={kind=e.kind,port=port,target=target,tags={},self_context=own,hit=e.hit==true}
  if type(e.aerial)=='string' then ev.aerial=e.aerial end
  if type(e.direction)=='string' then ev.direction=e.direction end
  if type(e.count)=='number' then ev.count=e.count end
  if type(e.damage)=='number' and e.damage==e.damage and e.damage>=0 and e.damage<=100000 then ev.damage=e.damage end
  self.engine:emit(ev)
 end
 -- A crit: a trigger for the rules, and the presentation moment (impact frames, tracer, toast), per peer.
 function L:crit(e)
  if type(e)~='table' then return end
  if self.g.log then self.g.log(('technique: crit P%s->P%s tag=%s strength=%.2f mult=%.2f %.1f -> %.1f forced=%s'):format(tostring(e.attacker),tostring(e.victim),tostring(e.move_tag),tonumber(e.strength) or 0,tonumber(e.multiplier) or 0,tonumber(e.base_damage) or 0,tonumber(e.final_damage) or 0,tostring(e.forced))) end
  if self.fx and not self:replaying() then self.fx:crit(e) end
  if not self.enabled or self:replaying() or not self:allowed() then return end
  local a,v=e.attacker,e.victim
  if type(a)~='number' or a<1 or a>6 or not self.engine:listens(a,'crit') then return end
  self.engine:emit{kind='crit',port=a,target=(type(v)=='number' and v>=1 and v<=6) and v or nil,tags={critical=true},strength=tonumber(e.strength) or 0,count=nil}
 end
 -- Native Shock ended (spent, expired, cleared): the Lua status ends with it.
 function L:shock_end(e)
  if type(e)~='table' or e.subfighter or not self.enabled or self:replaying() then return end
  local p=e.port;if type(p)~='number' or p<1 or p>6 then return end
  local at=self.engine.statuses[p];if at and at.shock and e.reason=='spent' then at.shock=nil;self.tech.shock[p]=nil end
 end
 function L:armor(e)
  if type(e)~='table' or e.subfighter then return end
  if self.g.log and self.enabled then self.g.log(('technique: armour event P%s type=%s absorbed=%s broke=%s'):format(tostring(e.port),tostring(e.type),tostring(e.absorbed),tostring(e.broke))) end
  local p=e.port
  if not self.enabled or self:replaying() or not self:allowed() or type(p)~='number' or p<1 or p>6 or not self.engine:listens(p,'armor') then return end
  self.engine:emit{kind='armor',port=p,tags={},absorbed=e.absorbed==true,broke=e.broke==true}
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
   if v then players[p]={percent=v.percent or 0,grounded=not v.airborne,stocks=v.stocks or 0,x=type(v.x)=='number' and v.x or nil,y=type(v.y)=='number' and v.y or nil}
    life[p]={falls=v.falls or 0,stocks=v.stocks or 0,char=v.char or -1,action=v.action or 14,entity_ref=v.entity_ref}
   end
  end
  return players,life
 end
 function L:export()
  return D.mod_codec.encode{version=1,echoes=self.echoes and self.echoes:snapshot(true),foes=self.foes and self.foes:snapshot(true),debug_equipped=self.debug_equipped,drives=self.drives and self.drives:snapshot(true),engine=self.engine:export(),enabled=self.enabled,owned=self.owned,hit_owned=self.hit_owned,pending=self.pending,observed=self.observed,tech=self.tech}
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
  self.tech=type(s.tech)=='table' and {crit=s.tech.crit or {},caps=s.tech.caps or {},armor=s.tech.armor or {},timed=s.tech.timed or {},shock=s.tech.shock or {}} or {crit={},caps={},armor={},timed={},shock={}}
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
  local interrupts=self:technique_ops(ops,players,stock_queued)
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
  if committed and why then
   for _,f in ipairs(interrupts or {}) do
    local o={frames=f.frames,exits=f.exits and (function() local t={};for _,x in ipairs(f.exits) do t[x]=true end;return t end)() or nil,guard=f.guard,restore_jumps=f.restore_jumps}
    if self.g.fighter_interrupt then local ok,err=pcall(self.g.fighter_interrupt,f.port,o);if not ok then self.g.log('mod: interrupt refused '..tostring(err)) end end
   end
   if self.fx then self.fx:frame(players) end
   self:announce_first()
   -- Diagnostic (`techprobe watch`): read the native armour rows for ten frames after a timed armour was written.
   if self.watch then
    if #(self.engine.fx)>0 then for _,f in ipairs(self.engine.fx) do if f.op=='armor' then self.watch_left=10;self.watch_port=f.port end end end
    if (self.watch_left or 0)>0 and self.g.fighter_armor then
     local rows=self.g.fighter_armor(self.watch_port);local t={};for _,r in ipairs(rows or {}) do t[#t+1]=('%s left=%s on=%s'):format(r.type,tostring(r.remaining),tostring(r.enabled)) end
     self.g.log(('technique watch: P%d frame %d armour %s'):format(self.watch_port,self.engine.frame,table.concat(t,' | ')));self.watch_left=self.watch_left-1
    end
   end
  end
  return true
 end
 -- ---- technique ops: one-shot effects on the event frame, passive state written only when it changes ------------------------------
 -- Armour, intangibility, forced crits, crit configuration, fighter caps and the timed channels are native state (snapshot
 -- covered) written through sim_commit ops, so the LAB journal replays them. The interrupt window has no journal operation: it is
 -- the one direct write (returned to the caller, applied after the commit). The `tech` table is only the record of what was last
 -- written, so a changed build writes a diff; it is in the checkpoint so a rewind restores it with the native state.
 local armor_limit={super=30,damage_threshold=300,knockback_threshold=300,hit_count=600,damage_pool=600}
 function L:technique_ops(ops,players,lost)
  local t=self.tech;local engine=self.engine;local out={};local interrupts={}
  local function add(o)
   local ok,err=pcall(D.mod_registry.operation,o)
   if ok then out[#out+1]=o else self.g.log('mod: technique op refused '..tostring(err)) end
  end
  local function enc(v) return D.mod_codec.encode(v) end
  -- crit configuration (a diff against what was written)
  for p=1,6 do
   if lost[p] then t.crit[p]=nil;t.caps[p]=nil;t.armor[p]=nil;t.timed[p]=nil;t.shock[p]=nil end
   local cfg=players[p] and engine:crit_config(p) or nil;local w=t.crit[p]
   if cfg then
    local slots={};for tag,s in pairs(cfg.slots) do slots[tag]=enc(s) end
    local full=not w or w.floor~=cfg.min_percent
    local function slot(tag,s) local o={op='crit',entity=p,slot=tag,chance=s.chance,multiplier=s.multiplier,launch=s.launch};if s.multiplier_max then o.multiplier_max=s.multiplier_max end;add(o) end
    if full then
     add({op='crit',entity=p,begin=true,min_percent=cfg.min_percent})
     slot('default',cfg.slots.default);for _,tag in ipairs({'jab','dash_attack','tilt','smash','aerial','grab','throw','special','projectile'}) do if cfg.slots[tag] then slot(tag,cfg.slots[tag]) end end
    else
     for tag,s in pairs(cfg.slots) do if w.slots[tag]~=slots[tag] then slot(tag,s) end end
     for tag in pairs(w.slots) do if not cfg.slots[tag] then add({op='crit',entity=p,slot=tag,chance=0,multiplier=1}) end end
    end
    t.crit[p]={floor=cfg.min_percent,slots=slots}
   elseif w then add({op='crit',entity=p,release=true});t.crit[p]=nil end
  end
  -- one-shot effects the rules fired this frame
  for _,f in ipairs(engine.fx) do if players[f.port] then
   if f.op=='armor' then
    local ty=f.type;local o={op='fighter_armor',entity=f.port,type=ty,frames=math.max(1,math.min(armor_limit[ty] or 30,math.floor(f.frames or 1)))}
    o.value=ty=='super' and 1 or math.max(1,math.min(ty=='hit_count' and 3 or 40,f.value or 1));if ty=='hit_count' then o.value=math.floor(o.value) end
    if f.direction and f.direction~='any' then o.direction=f.direction end
    add(o);self.g.log(('technique: armour %s %s frames=%d on P%d (frame %d)'):format(ty,tostring(o.value),o.frames,f.port,engine.frame))
   elseif f.op=='intangible' then self.g.log(('technique: intangible %d frames on P%d (frame %d)'):format(f.frames,f.port,engine.frame));add({op='fighter_effect',entity=f.port,effect='intangible',value=1,frames=math.max(1,math.min(24,f.frames))})
   elseif f.op=='crit_next' then
    local cfg=engine:crit_config(f.port)
    if cfg then self.g.log(('technique: next %d hit(s) crit on P%d (frame %d)'):format(f.count,f.port,engine.frame));add({op='crit',entity=f.port,force=math.max(1,math.min(3,f.count)),min_percent=cfg.min_percent}) end
   elseif f.op=='interrupt' then
    interrupts[#interrupts+1]={port=f.port,frames=math.max(1,math.min(20,f.frames)),exits=f.exits,guard=f.guard,restore_jumps=f.restore_jumps}
   end
  end end
  -- passive caps and permanent armour (equip rules)
  for p=1,6 do
   local ps=players[p] and engine:passive_state(p) or nil
   local values={}
   if ps then
    if ps.air_jumps then values.air_jumps=math.max(0,math.min(6,ps.air_jumps)) end
    for _,x in ipairs(ps.forbid) do values[x]=true end
   end
   local key=next(values) and enc(values) or nil
   if key~=t.caps[p] then if key then add({op='fighter_caps',entity=p,values=values}) else add({op='fighter_caps',entity=p,values={}}) end;t.caps[p]=key end
   local akey=ps and ps.armor and (ps.armor.type..':'..tostring(math.min(10,ps.armor.value))) or nil
   if akey~=t.armor[p] then
    if akey then add({op='fighter_armor',entity=p,type=ps.armor.type,value=math.min(10,ps.armor.value),frames=0})
    else add({op='fighter_armor',entity=p,type=t.armor[p]:match('^[a-z_]+'),clear=true}) end
    t.armor[p]=akey
   end
  end
  -- Shock: the Lua status mirrored into the native one (gd.shock): written when it starts or its charges change, cleared when it ends.
  for p=1,6 do
   local st=players[p] and engine.statuses[p] and engine.statuses[p].shock or nil;local w=t.shock[p]
   if st then
    local left=math.max(1,math.min(3600,st.expires-engine.frame));local charges=math.max(1,math.min(8,st.stacks))
    if not w or w.charges~=charges or st.expires>w.expires then add({op='shock',entity=p,frames=left,charges=charges,hitstun=1.5,stack=false});t.shock[p]={charges=charges,expires=st.expires} end
   elseif w then add({op='shock',entity=p,clear=true});t.shock[p]=nil end
  end
  -- timed channels: 1 = the earned status (value = the cause's colour index), 2 = the echo picture window. The remaining
  -- frames are written every frame while on, so the native countdown can never outlive the Lua status.
  for p=1,6 do
   local on=t.timed[p] or {}
   local e=players[p] and engine:earned(p) or nil;local ew=players[p] and engine:echo_window(p) or 0
   local want={[1]=e and e.frames or 0,[2]=ew}
   for ch=1,2 do
    local frames=want[ch]
    if frames>0 then add({op='timed_status',entity=p,channel=ch,value=ch==1 and D.mod_skill.cause[e.cause].index or 1,frames=math.min(3600,frames)});on[ch]=true
    elseif on[ch] then if players[p] then add({op='timed_status',entity=p,channel=ch,value=0,frames=0}) end;on[ch]=nil end
   end
   t.timed[p]=next(on) and on or nil
  end
  for _,o in ipairs(out) do if #ops<24 then ops[#ops+1]=o else self.g.log('mod: technique op deferred (24 operation limit)') end end
  return interrupts
 end
 -- The first time a technique or crit rule fires in a run, one short line (the HUD toast), said once per rule.
 function L:announce_first()
  local log=self.engine.fired_log;if not log or #log==0 then return end
  self.engine.fired_log=nil
  if not self.announced then self.announced={} end
  local first=log[1]
  if self.toast and not self.announced.any then
   self.announced.any=true
   self.toast('Technique rule fired: '..first.label..'. Techniques show as a coloured afterimage while their reward lasts.')
  end
 end
 function L:tick()
  if self.drives then self.drives:tick() end
  if self.fx then self.fx:tick() end
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
 -- Script cost of the per-frame host and of one skill event (wall clock, diagnostic only: `techprobe cost`).
 do
  local function timed(name,fn)
   return function(self,...)
    local g=self.g;local t0=g.time and g.time()
    local a,b=fn(self,...)
    if t0 then local c=self.cost;if not c then c={};self.cost=c end;local r=c[name];if not r then r={n=0,max=0};c[name]=r end
     local ms=(g.time()-t0)*1000;r.n=r.n+1;r[(r.n-1)%240+1]=ms;if ms>r.max then r.max=ms end end
    return a,b
   end
  end
  L.frame=timed('frame',L.frame);L.skill=timed('skill',L.skill);L.technique_ops=timed('technique_ops',L.technique_ops);L.export=timed('export',L.export);L.sample=timed('sample',L.sample)
 end
 return L
end
