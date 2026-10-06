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
  self.seats={} -- extra build seats (co-op): port -> drive host sharing this engine and the ground; empty in every one-player use
  if D.foe_lab then self.foes=D.foe_lab.new(g,self) end
  self.tech={crit={},caps={},armor={},timed={},shock={}}
  if D.earned_fx then self.fx=D.earned_fx.new(self);g.command('critfx',function(arg) return self.fx:command(arg or '') end,'preview <0..1> | intensity <0..1> | off | (status)') end
  -- A read-only probe of the technique layer for tests and the owner: statuses with their cause, what was last written natively, the crit and presentation counters.
  g.command('techprobe',function(arg)
   if arg=='cost reset' then self.cost={} return true end
   if arg=='prof on' or arg=='prof reset' then -- diagnostic wall-clock profile of the rule host's functions (inclusive time, so nested calls count twice)
    self.prof=self.prof or {};if arg=='prof reset' then for _,r in pairs(self.prof) do r.n=0;r.sum=0;r.max=0 end end
    if not self.prof_wrapped then
     self.prof_wrapped={};local prof=self.prof_wrapped
     local function done(r,t0,...) local ms=(g.time()-t0)*1000;r.n=r.n+1;r.sum=r.sum+ms;if ms>r.max then r.max=ms end;return ... end
     local function wrap(prefix,cls) if type(cls)~='table' then return end
      for k,f in pairs(cls) do if type(f)=='function' and k~='__index' then
       local name=prefix..'.'..tostring(k);local r={n=0,sum=0,max=0};self.prof[name]=self.prof[name] or r;local rec=self.prof[name]
       if name=='lab.frame' then -- a slow frame says which functions it spent its time in (inclusive, so nested calls overlap)
        cls[k]=function(...)
         local before={};for n,r in pairs(self.prof) do before[n]=r.sum end
         local t0=g.time();local function fin(...)
          local ms=(g.time()-t0)*1000;rec.n=rec.n+1;rec.sum=rec.sum+ms;if ms>rec.max then rec.max=ms end
          if ms>12 then local d={};for n,r in pairs(self.prof) do local x=r.sum-(before[n] or 0);if x>1 and n~='lab.frame' then d[#d+1]={n,x} end end
           table.sort(d,function(a,b) return a[2]>b[2] end);local o={};for i=1,math.min(8,#d) do o[i]=('%s=%.1f'):format(d[i][1],d[i][2]) end
           g.log(('techprobe slow frame %.1f ms (engine frame %s): %s'):format(ms,tostring(self.engine.frame),table.concat(o,' ')))
          end
          return ...
         end
         return fin(f(...))
        end
       else
       cls[k]=function(...) return done(rec,g.time(),f(...)) end
       end
      end end
     end
     for _,n in ipairs({'mod_budget','mod_codec','mod_graph','mod_echo','mod_status','drive_loot','drive_merge','foe_roll','keystones','drive_text'}) do wrap(n,D[n]) end
     wrap('engine',getmetatable(self.engine));wrap('lab',getmetatable(self));if self.drives then wrap('drives',getmetatable(self.drives));wrap('bag',getmetatable(self.drives.bag));wrap('loot',getmetatable(self.drives.loot)) end
     if self.foes then wrap('foes',getmetatable(self.foes)) end
     g.log('techprobe prof: wrapping installed')
    end
    return true
   end
   if arg=='prof' then
    local list={};for name,r in pairs(self.prof or {}) do if r.n>0 then list[#list+1]={name=name,r=r} end end
    table.sort(list,function(a,b) return a.r.sum>b.r.sum end)
    for i=1,math.min(30,#list) do local e=list[i];g.log(('techprobe prof: %-34s n=%6d total=%9.1f ms mean=%7.3f max=%7.2f'):format(e.name,e.r.n,e.r.sum,e.r.sum/e.r.n,e.r.max)) end
    return true
   end
   if arg=='ops' or arg=='ops reset' then -- which operations the per-frame commit carries (a run commits a quiet frame only every tenth)
    local st=self.opstats or {frames=0,empty=0,kinds={}}
    local o={};for k,n in pairs(st.kinds) do o[#o+1]=k..'='..n end;table.sort(o)
    g.log(('techprobe ops: frames=%d quiet=%d %s'):format(st.frames,st.empty,table.concat(o,' ')));if arg=='ops reset' then self.opstats=nil end;return true
   end
   if arg=='size' then g.log('techprobe size: '..self:size_report());return true end
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
  g.command('mod',function(arg) return self:command(arg or '') end,'list | add <id> [port] | clear | trace | intensity <0..1> | box on|off | status <name> [port] [frames] [cause] [stacks] [max]')
  g.command('depth',function(arg) return self:depth_command(arg or '') end,'<nonnegative depth> [New Game+ loop]')
  g.command('envoynet',function(arg) return self:net_command(arg or '') end,'status | auto <0..3|x> | pick <0..3> | stage <seed> [game] | tamper (test hooks)')
  return self
 end
-- A co-op run adds one drive host per further local player (port 2..): own bag, slots and keystones, the same engine and the same ground.
 function L:add_seat(port)
  assert(self.drives and port>=2 and port<=6 and not self.seats[port],'seat port 2..6, once')
  local seat=D.drive_lab.new(self.g,self,{port=port,drops=self.drives.drops});self.seats[port]=seat;return seat
 end
 function L:seat_ports() local out={};for p in pairs(self.seats) do out[#out+1]=p end;table.sort(out);return out end
 function L:seats_busy() for _,seat in pairs(self.seats) do if #seat.pending>0 or seat:stale() then return true end end;return false end
 function L:seats_enable() for _,seat in pairs(self.seats) do if #seat.pending>0 or #seat.bag.items>0 or seat:has_build() then return true end end;return false end
 function L:seat_snapshots(raw) local out;for p,seat in pairs(self.seats) do out=out or {};out[p]=seat:snapshot(raw) end;return out end
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
  self:release_native();self.op_sig=nil;self.blob_cache=nil
  local keep=not full and self:hosted() and self.drives
  if self.echoes then self.echoes:reset() end
  if self.foes then self.foes:reset() end
  if keep then self.drives:soft_clear() elseif self.drives then self.drives:clear() end
  for _,seat in pairs(self.seats) do if keep then seat:soft_clear() else seat:clear() end end
  local ctx=self.drives and self.drives.bag.context
  self.engine=D.mod_engine.new(104729,D.mod_pool,{context=ctx});self.enabled=false;self.owned={};self.hit_owned={};self.pending={};self.observed={};self.debug_equipped={}
  self.tech={crit={},caps={},armor={},timed={},shock={}};if self.fx then self.fx:reset() end
  self.display:clear();self.display.engine=self.engine
  if self.drives and (self.drives:has_build() or #self.drives.bag.items>0) then self.enabled=true end
  if self:seats_enable() then self.enabled=true end
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
   elseif w[1]=='box' then -- LAB debug text on/off (captures); look only
    assert(#w==2 and (w[2]=='on' or w[2]=='off'),'usage: mod box on|off');self.hide_box=(w[2]=='off')
   elseif w[1]=='status' then -- debug (captures, looks): put a status on a fighter without its trigger. mod status <name> [port] [frames] [cause] [stacks]
    local allowed,why=self:allowed();assert(allowed,why);assert(not self:replaying(),'modifier edit refused during rewind')
    local name=w[2];local known=false;for _,n in ipairs(D.mod_status.order) do if n==name then known=true end end
    assert(known,'usage: mod status <'..table.concat(D.mod_status.order,'|')..'> [port] [frames] [cause] [stacks] [max]')
    local p=port(w[3] or 1);assert(self.g.player(p),'fighter absent');local n=math.max(1,math.min(3600,math.floor(tonumber(w[4] or 600) or 600)))
    local cause=w[5];if cause=='-' then cause=nil end;assert(cause==nil or D.mod_skill.cause[cause],'cause: '..table.concat((function() local t={} for k in pairs(D.mod_skill.cause) do t[#t+1]=k end table.sort(t) return t end)(),'|'))
    local at=self.engine.statuses[p] or {};self.engine.statuses[p]=at
    at[name]={expires=self.engine.frame+n,stacks=math.max(1,math.floor(tonumber(w[6] or 1) or 1)),max=math.max(1,math.floor(tonumber(w[7] or w[6] or 1) or 1)),amount=1,next_tick=self.engine.frame+60,origin={'debug status'},cause=cause}
    self.enabled=true;if self.options.activate then self.options.activate() end
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
  if self.tap and attacker and victim and not self:replaying() then self.tap('hit',attacker,victim) end -- the co-op run credits damage and drops to the last player who hit
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
  if self.foes and self.foes.driver and type(e)=='table' and not e.subfighter then self.foes.driver:on_skill(e) end
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
 function L:pickup(e) if self.drives then self.drives:pickup(e) end;for _,seat in pairs(self.seats) do seat:pickup(e) end;if e.port then self:event{kind='item_pickup',port=e.port,tags={}} end end
 function L:sample()
  local players,life={},{}
  for p=1,6 do local v=self.g.player(p)
   if v then players[p]={percent=v.percent or 0,grounded=not v.airborne,stocks=v.stocks or 0,x=type(v.x)=='number' and v.x or nil,y=type(v.y)=='number' and v.y or nil}
    life[p]={falls=v.falls or 0,stocks=v.stocks or 0,char=v.char or -1,action=v.action or 14,entity_ref=v.entity_ref}
   end
  end
  return players,life
 end
 -- The checkpoint blob is capped at 16384 bytes natively: which part of it is how big (diagnostic, also logged when a commit is refused).
 function L:size_report()
  local parts={echoes=self.echoes and self.echoes:snapshot(true),foes=self.foes and self.foes:snapshot(true),debug_equipped=self.debug_equipped,drives=self.drives and self.drives:snapshot(true),seats=self:seat_snapshots(true),engine=self.engine:export(),owned=self.owned,hit_owned=self.hit_owned,pending=self.pending,observed=self.observed,tech=self.tech}
  local names={};for k in pairs(parts) do names[#names+1]=k end;table.sort(names);local out={}
  for _,k in ipairs(names) do local ok,t=pcall(D.mod_codec.encode,{[k]=parts[k]});out[#out+1]=k..'='..(ok and #t or 'over') end
  if self.foes then local fs=self.foes:snapshot(true);local sub={};for _,k in ipairs({'builds','pending','labels'}) do local ok,t=pcall(D.mod_codec.encode,{fs[k]});sub[#sub+1]=k..'='..(ok and #t or 'over') end;out[#out+1]='foes{'..table.concat(sub,' ')..'}' end
  if self.foes then for p,r in pairs(self.foes.builds) do local ok,t=pcall(D.mod_codec.encode,r);if ok then out[#out+1]='sample P'..p..'='..t:sub(1,1100) break end end end
  do local sub={};for _,k in ipairs({'equipped','implicits','statuses','recent','trace','display','damage'}) do local ok,t=pcall(D.mod_codec.encode,{self.engine[k]});sub[#sub+1]=k..'='..(ok and #t or 'over') end;out[#out+1]='engine{'..table.concat(sub,' ')..'}' end
  local eq={};for p=1,6 do local ok,t=pcall(D.mod_codec.encode,{self.engine.equipped[p] or {}});if ok then eq[#eq+1]='P'..p..'='..#t end end
  local ok,t=pcall(function() return self:export() end)
  return 'total='..(ok and #t or 'over')..' '..table.concat(out,' ')..' | equipped '..table.concat(eq,' ')
 end
 function L:export()
  return D.mod_codec.encode{version=1,echoes=self.echoes and self.echoes:snapshot(true),foes=self.foes and self.foes:snapshot(true),debug_equipped=self.debug_equipped,drives=self.drives and self.drives:snapshot(true),seats=self:seat_snapshots(true),engine=self.engine:export(self:hosted()),enabled=self.enabled,owned=self.owned,hit_owned=self.hit_owned,pending=self.pending,observed=self.observed,tech=self.tech}
 end
 function L:check_echo_capacity(engine,manual)
  if not self.echoes then return end;manual=manual or self.echoes.manual
  for p=1,6 do local desc=engine:echo_description(p)
   assert(#desc.rules+#(manual[p] or {})<=8,'eight echo rules per fighter maximum')
   if #desc.copies>0 or #(manual[p] or {})>0 then assert(self.echoes:capable(),'native echo journal unavailable; rebuild required')end
  end
 end
 function L:prospective(engine,debug,pending,foes,drives,players,manual,seats)
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
  for p,snap in pairs(seats or {}) do -- co-op seats: the same validation and publication order, one bag each
   local seat=self.seats[p];if seat then
    local roots={engine=probe,debug_equipped=debug,pending=pending,echo_manual=manual,check_echo_capacity=function(_,e)self:check_echo_capacity(e,manual)end};local st=seat:validate(snap,roots)
    local draft=D.drive_bag.new(seat.loot);assert(draft:restore(st.bag))
    for _,e in ipairs(st.pending or {}) do assert(draft[e.op](draft,e.a,e.b)) end
    if #(st.pending or {})>0 or next(draft.equipped) or draft.keystone or #(draft.keystones or {})>0 then local mods,implicit=draft:derive();probe:set_build(p,seat:combined(mods,roots),implicit) end
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
  local drives=self:prospective(probe,debug,s.pending,foes,self.drives and s.drives,nil,echoes and echoes.manual or {},s.seats)
  if self.echoes then self.echoes:restore(echoes) end
  self.debug_equipped=debug;self.pending=s.pending
  if drives then self.drives:publish(drives) end
  for p,snap in pairs(s.seats or {}) do local seat=self.seats[p];if seat then seat:restore(snap) end end
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
  local staged=#self.pending>0 or (self.foes and #self.foes.pending>0) or (self.drives and (#self.drives.pending>0 or self.drives:stale())) or self:seats_busy()
  if ready and staged then
   local bag=self.drives and self.drives.bag:snapshot();local seat_bags={};for p,seat in pairs(self.seats) do seat_bags[p]=seat.bag:snapshot() end;local debug=D.mod_codec.decode(D.mod_codec.encode(self.debug_equipped))
   local phase='check'
   local accepted,why=pcall(function()
    self:prospective(self.engine,self.debug_equipped,self.pending,self.foes,self.drives and self.drives:snapshot(),players,nil,self:seat_snapshots())
    phase='apply'
    if self.foes then self.foes:apply() end
    for _,e in ipairs(self.pending) do if players[e.port] then self.engine:equip(e.port,e.id);self.debug_equipped[e.port]=self.debug_equipped[e.port] or {};self.debug_equipped[e.port][e.id]=1 end end
    self.pending={}
    if self.drives and (#self.drives.pending>0 or self.drives:has_build()) then self.drives:apply() end
    for _,seat in pairs(self.seats) do if #seat.pending>0 or seat:has_build() then seat:apply() end end
   end)
   if accepted then self.publish_tries=0;self.publish_refusals=0
   elseif phase=='check' and tostring(why):find('ran too long',1,true) and (self.publish_tries or 0)<60 then
    -- The check ran out of script budget before anything was applied: nothing is discarded. The work done so far is memoised
    -- (mod_budget), so the next frame's retry gets further. Said in the log and on screen; never a silent drop.
    self.publish_tries=(self.publish_tries or 0)+1
    if self.publish_tries==1 or self.publish_tries%10==0 then self.g.log('mod: build publication deferred (script budget), retry '..self.publish_tries..': '..tostring(why):sub(1,120)) end
    if self.publish_tries==1 and self.toast then pcall(self.toast,'Build update delayed: retrying') end
    return true
   end
   if not accepted then
    self.publish_tries=0
    -- A refused publication is retried (twice, a frame apart) before anything is discarded: a transient refusal must not cost the build.
    self.publish_refusals=(self.publish_refusals or 0)+1
    if self.publish_refusals<=2 and self:hosted() then
     self.g.log('mod: build publication refused, retrying ('..self.publish_refusals..'/2): '..tostring(why):sub(1,160))
     if self.toast then pcall(self.toast,'Build update refused: retrying') end
     return true
    end
    self.publish_refusals=0
    pcall(self.g.sim_clear);self.op_sig=nil;if self.echoes then self.echoes:reset() end;self.display:clear();self.engine=D.mod_engine.new(104729,D.mod_pool,{context=self.engine.context});self.display.engine=self.engine
    self.enabled=false;self.owned={};self.hit_owned={};self.pending={};self.observed={};self.debug_equipped=debug
    if self.foes then self.foes:reset() end
    if self.drives then self.drives.bag.items=bag.items;self.drives.bag.equipped=bag.equipped;self.drives.bag.keystone=bag.keystone;self.drives.bag.keystones=bag.keystones or {};self.drives.bag.context=bag.context;self.drives.pending={} end
    for p,seat in pairs(self.seats) do local b=seat_bags[p];seat.bag.items=b.items;seat.bag.equipped=b.equipped;seat.bag.keystone=b.keystone;seat.bag.keystones=b.keystones or {};seat.bag.context=b.context;seat.pending={} end
    self.g.log('mod: disabled after pending publication refusal '..tostring(why)..' (bags kept; the builds are published again from them)')
    if self.toast then pcall(self.toast,'Build update refused: '..tostring(why):sub(1,60)) end
    if self:hosted() then self.enabled=true end -- the run's bags are intact: the next frame publishes the builds from them
    return true
   end
  end
  if self.foes then self.foes:frame() end
  if self.drives then self.drives:frame() end
  for _,seat in pairs(self.seats) do seat:frame() end
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
   -- A boss's remaining HP is its stamina minus its damage and the game ends the fight when it reaches 0. Only a hit can start the boss's
   -- death, so (char 26 / 27: Master Hand, Crazy Hand) a damage-over-time tick (burn) must never take a boss's damage up: it would end the fight with the boss alive (the 2026-10-05 softlock).
   if players[p] and damage and damage>0 and life[p] and (life[p].char==26 or life[p].char==27) then
    self.boss_dot_skipped=(self.boss_dot_skipped or 0)+1
    if self.boss_dot_skipped==1 then self.g.log('envoy: damage over time does not hurt a boss (only hits do)') end
    damage=nil
   end
   if players[p] and damage and damage~=0 then ops[#ops+1]={op='damage',port=p,value=math.max(0,math.min(999,players[p].percent+damage))} end
  end
  if self.echoes then self.echoes:ops(ops,players,stock_queued) end
  local interrupts=self:technique_ops(ops,players,stock_queued)
  self.owned=new_owned;self.hit_owned=new_hit_owned
  for p=1,6 do if self.engine.statuses[p] and not next(self.engine.statuses[p]) then self.engine.statuses[p]=nil end end
  self.enabled=(self.echoes and self.echoes:active()) or D.mod_progression.effective(self.engine.context)>0 or (self.foes and (#self.foes.pending>0 or next(self.foes.builds)~=nil)) or (self.drives and (#self.drives.pending>0 or self.drives.drops:count()>0 or #self.drives.bag.items>0 or self.drives:has_build())) or self:seats_enable() or #self.pending>0 or next(self.engine.equipped)~=nil or next(self.engine.statuses)~=nil
  -- Visual pulse/cooldown metadata is pure state and belongs in the checkpoint.
  if self.enabled then self.display:update(self.engine,players) else self.display:clear() end
  -- The journal keeps the last frames' commits, one 2.4 KB slot per operation, under a 128 MiB budget: a commit that re-sends every fighter's overlay
  -- and hit rules each frame (six fighters: 12 operations, 30 KB a frame) overran it within a long stage ("sim_commit journal memory budget
  -- exhausted", found in the co-op campaigns at about 3500 frames). Overlays and hit rules stay in place until replaced, so a run (which has no
  -- rewind) sends only the ones that changed; the LAB keeps sending all of them, as the rewind journal is built on that.
  local sent
  if self:hosted() then
   -- cheap signatures (no encoding): a hit-rule list is the engine memo's own table, so identity says it is unchanged; an overlay is eleven numbers
   local sig=self.op_sig;if not sig then sig={};self.op_sig=sig end
   local m=self.g.match and self.g.match();local f=m and m.frame or 0
   if f<(self.op_frame or 0) or (f>0 and f%120==0) then sig={};self.op_sig=sig end -- a new scene (frame count restarted), and a full refresh every 120 frames as a safeguard
   self.op_frame=f
   local kept={};sent={}
   for _,o in ipairs(ops) do
    local skip=false
    if o.op=='fighter_mod' then
     local prev=sig['m'..o.port]
     if prev then
      skip=true;local n=0
      if o.values then for k,v in pairs(o.values) do n=n+1;if prev.values==nil or prev.values[k]~=v then skip=false;break end end end
      if skip then local pn=0;if prev.values then for _ in pairs(prev.values) do pn=pn+1 end end;if pn~=n or (prev.values==nil)~=(o.values==nil) then skip=false end end
     end
     if not skip then sent['m'..o.port]={values=o.values and (function() local c={};for k,v in pairs(o.values) do c[k]=v end;return c end)() or nil} end
    elseif o.op=='hit_rules' then
     local prev=sig['h'..o.port]
     if prev and prev.rules==o.rules and prev.bits==o.status_bits then skip=true else sent['h'..o.port]={rules=o.rules,bits=o.status_bits} end
    end
    if not skip then kept[#kept+1]=o end
    if #kept>=24 then break end
   end
   ops=kept
  end
  do local st=self.opstats;if not st then st={frames=0,empty=0,kinds={}};self.opstats=st end
   st.frames=st.frames+1;if #ops==0 then st.empty=st.empty+1 end;for _,o in ipairs(ops) do st.kinds[o.op]=(st.kinds[o.op] or 0)+1 end end
  local committed,why
  if self:hosted() and #ops==0 and (self.engine.frame%10~=0) then committed,why=true,true
  else
   -- the operations are applied every frame they exist; the blob (state for a rewind that a run never does) is rebuilt every tenth frame
   committed,why=pcall(function()
    local blob=self.blob_cache
    if not (self:hosted() and blob and self.engine.frame%10~=0) then blob=self:export();self.blob_cache=blob end
    return self.g.sim_commit(blob,ops)
   end)
  end
  if sent and committed and why then for key,text in pairs(sent) do self.op_sig[key]=text end end
  if not committed or not why then
   -- Validation/allocation failures are atomic natively. Retire old effects
   -- too, rather than letting mutated Lua timers diverge from the game.
   local ok2,t=pcall(function() return self:size_report() end);local sizes=ok2 and t or nil
   local cleared,detail=pcall(self.g.sim_clear)
   self:reset()
   if sizes then self.g.log('mod: checkpoint sizes: '..sizes) end
   self.g.log('mod: disabled after checkpoint refusal '..tostring(why)..(cleared and '' or '; clear refused '..tostring(detail)))
   if self:hosted() then if self.toast then pcall(self.toast,'Build update refused (checkpoint): publishing again') end;self.g.log('mod: the run republishes the builds from the bags next frame') end
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
 -- ---- the online Envoy set (stage 3; _research/envoy-netplay-scoping-2026-10-05.md) ----------------------------------------------------------------
 -- Runs in the ONLINE LOBBY, never in a match, and writes no gameplay state. It reads gd.netplay().envoy (the host-arbitrated state the native lobby
 -- keeps), computes BOTH players' builds from the set seed (mod_progression.set_*: pure), stages them natively (gd.netbuild_stage: the native side
 -- applies them once before frame 0 and reports their word to the other side), draws this player's reward pick and sends it (gd.netplay_act 'rpick').
 -- The look of the pick is unverified: nobody has seen it.
 function L:net_log(text) self.g.log('envoy net: '..text) end
 local function net_label(id) for _,m in ipairs(D.mod_pool) do if m.id==id then return m.label end end;return id end
 -- This seat's three offers for the game about to be played: computed from the build it ended the previous game with.
 function L:net_offers(env,np)
  local seat=np.me+1;local n=self.net
  local prior=D.mod_progression.set_build(D,env.seed,np.game-1,seat,n.picks)
  return D.mod_progression.set_offers(D,env.seed,np.game,seat,prior),seat
 end
 function L:net_sync()
  local g=self.g
  if not (g.netplay and g.netbuild_stage and D.mod_progression and D.mod_progression.set_stage) then return end
  self.net_ticks=(self.net_ticks or 0)+1;if not self.net and self.net_ticks%20~=0 then return end -- idle: look every 20 ticks (gd.netplay builds a table), active: every tick
  local np=g.netplay();local env=np and np.envoy;local n=self.net
  if not (env and env.on and env.seed>0 and np.phase=='lobby') then
   if n and n.input then n.input:close();n.input=nil end -- (a reconnect between games makes env.on blink off: the state is kept, the native history is the memory)
   return
  end
  if not n or n.seed~=env.seed then n={seed=env.seed,picks={}};self.net=n end
  local game=np.game;if game<1 then return end
  -- the picks of every resolved reward come from the lobby's own history (host-resolved, mirrored to the guest), never from this script's memory
  local picks={};for g,p in pairs(env.history) do picks[g]={[1]=p[1],[2]=p[2]} end;n.picks=picks
  if n.staged and game<n.staged then n.staged=nil end -- a new set with the same seed (or a restart): stage again
  if n.staged~=game and (game==1 or n.picks[game]) then
   local ok,st=pcall(D.mod_progression.set_stage,D,env.seed+(self.net_tamper and 1 or 0),game,n.picks) -- tamper: a TEST hook that makes this client stage a different build
   if ok then
    for seat=1,2 do
     local sok,why=pcall(g.netbuild_stage,seat,st[seat].record,st[seat].ops)
     if not sok then ok=false;st=why;break end
    end
   end
   if ok then
    n.staged=game;n.builds={st[1].build,st[2].build}
    self:net_log(('game %d staged: host %s | guest %s | word %s'):format(game,st[1].digest,st[2].digest,g.netbuild().word))
   elseif n.fail~=tostring(st) then n.fail=tostring(st);self:net_log('staging failed: '..n.fail) end
  end
  if env.open then
   local offers,seat=self:net_offers(env,np);n.offers=offers;n.seat=seat
   if env.picks[seat]<0 then
    if self.net_auto~=nil then
     n.wait=(n.wait or 0)+1
     if n.wait>=30 then n.wait=nil;g.netplay_act('rpick',self.net_auto) end
    else
     n.input=n.input or D.menu_input.new(g,1);n.input:set_active(true)
     for _,a in ipairs(n.input:poll()) do
      if a=='up' then n.cursor=math.max(0,(n.cursor or 0)-1) elseif a=='down' then n.cursor=math.min(3,(n.cursor or 0)+1)
      elseif a=='accept' then g.netplay_act('rpick',n.cursor or 0) elseif a=='back' then g.netplay_act('rpick',3) end
     end
    end
   elseif n.input then n.input:close();n.input=nil end
  elseif n.input then n.input:close();n.input=nil end
 end
 function L:net_draw()
  local g=self.g;local n=self.net
  if not n or not n.offers or not g.netplay or not g.text then return end
  local np=g.netplay();local env=np.envoy
  if not (env and env.on and env.open) or env.picks[n.seat]>=0 then return end
  g.fill(150,110,340,190,0x0A1018E8);g.box(150,110,340,190,0xE8EEF4FF)
  g.text(162,118,('ENVOY REWARD  -  game %d  -  %d s left'):format(np.game,math.ceil(env.left/60)),0xFFE070FF,1)
  for i,id in ipairs(n.offers) do
   g.text(170,146+(i-1)*24,((n.cursor or 0)==i-1 and '> ' or '  ')..net_label(id),(n.cursor or 0)==i-1 and 0xFFFFFFFF or 0xB0B8C4FF,1)
  end
  g.text(170,146+72,((n.cursor or 0)==3 and '> ' or '  ')..'Keep my build',(n.cursor or 0)==3 and 0xFFFFFFFF or 0xB0B8C4FF,1)
  g.text(162,270,'Up/Down choose, A takes it, B keeps. No pick: the first offer.',0x8090A0FF,1)
 end
 function L:net_command(arg)
  local g=self.g;local word,rest=arg:match('^(%S*)%s*(.*)$')
  if word=='status' or word=='' then
   local n=self.net;local np=g.netplay and g.netplay();local env=np and np.envoy
   return ('envoy net: on=%s seed=%s game=%s open=%s picks=%s,%s round=%s staged=%s word=%s peer=%s refused=%s auto=%s'):format(tostring(env and env.on),tostring(env and env.seed),tostring(np and np.game),tostring(env and env.open),tostring(env and env.picks[1]),tostring(env and env.picks[2]),tostring(env and env.round),tostring(n and n.staged),tostring(env and env.word),tostring(env and env.peer_word),tostring(env and env.refused),tostring(self.net_auto))
  elseif word=='auto' then local v=tonumber(rest);self.net_auto=v and math.max(0,math.min(3,math.floor(v))) or nil;return 'auto pick '..tostring(self.net_auto)
  elseif word=='pick' then return tostring(g.netplay_act('rpick',tonumber(rest) or 0))
  elseif word=='stage' then -- TEST hook (offline or lobby): stage both seats of a set at <seed> [game] with the default picks, as the lobby would
   local seed,game=rest:match('^(%d+)%s*(%d*)$');seed=tonumber(seed);game=tonumber(game) or 1;if not seed then return 'envoynet stage <seed> [game]' end
   local st=D.mod_progression.set_stage(D,seed,game,{})
   for seat=1,2 do g.netbuild_stage(seat,st[seat].record,st[seat].ops) end
   return ('staged game %d of seed %d: %s | %s word %s'):format(game,seed,st[1].digest,st[2].digest,g.netbuild().word)
  elseif word=='tamper' then self.net_tamper=true;if self.net then self.net.staged=nil end;return 'tampered: this client stages a different build (test hook)'
  end
  return 'envoynet status | auto <0..3|x> | pick <0..3> | tamper'
 end
 function L:tick()
  self:net_sync()
  if self.drives then self.drives:tick() end
  for _,seat in pairs(self.seats) do seat:tick() end
  if self.fx then self.fx:tick() end
  if self.enabled and self:allowed() and not self:replaying() and self.display.tick then self.display:tick(self.engine) end
  return self.drives and self.drives.menu.active or false
 end
 function L:scene() if D.menu_input then D.menu_input.reset() end;self:reset();self.display=D.mod_display.new(self.g,self.engine) end -- scene invalidates shader handles
 function L:draw()
  self:net_draw()
  if self.drives and self.drives.menu.active then self.drives:draw();return end
  local plate=self.foes and self.g.kit and next(self.foes.labels)~=nil
  -- The Modifier LAB debug text (ids, last chain) is a LAB tool: a run's own strip and plates replace it.
  if plate then self.foes:draw() elseif self.enabled and not self:hosted() and not self.hide_box then self.display:draw(self.engine) end
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
  for _,seat in pairs(self.seats) do seat.bag.context=D.mod_progression.context(ctx) end
  -- fewer slots than before (never in real play): drives in slots that no longer exist go to the bag when it has room, else stay equipped
  -- in their slot until the player frees space. Said in the log and on screen either way; nothing is destroyed.
  local bags={};if self.drives then bags[#bags+1]=self.drives.bag end;for _,seat in pairs(self.seats) do bags[#bags+1]=seat.bag end
  local function nm(r) local ok,n=pcall(function() return self.drives.loot:name(r) end);return ok and n or 'a drive' end
  for _,bag in ipairs(bags) do
   bag.overflow=true
   local moved,kept=bag:settle_overflow()
   for _,r in ipairs(moved) do self.g.log('slots reduced to '..bag:slots()..': '..nm(r)..' moved to the bag');if self.toast then pcall(self.toast,'A drive moved to the bag: fewer slots') end end
   for _,r in ipairs(kept) do self.g.log('slots reduced to '..bag:slots()..' and the bag is full: '..nm(r)..' stays in its slot until you free space');if self.toast then pcall(self.toast,'Bag full: a drive stays in its extra slot') end end
   local dropped=bag:settle_keystones()
   for _,id in ipairs(dropped) do self.g.log('keystone allowance reduced to '..D.mod_progression.keystones(bag.context)..': keystone '..tostring(id)..' removed (the pick is owed again)');if self.toast then pcall(self.toast,'A keystone was removed: lower allowance') end end
  end
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
