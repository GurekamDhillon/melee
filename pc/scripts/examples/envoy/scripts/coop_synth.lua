-- Synthetic players for the co-op run, for the lane that found out what the design needs: both players flown by the debug cursor
-- (gd.fly_target / gd.fly_attack, offline only), reward choices made by a seeded policy through the same host calls the screen's buttons end in.
-- Off unless started (`envoy synth start [policy]`); it is a test driver, never part of a normal run. Hurtboxes stay on while flying
-- (gd.fly_solid) so the players can take damage and lose stocks like anyone else.
return function(D)
 local S={};S.__index=S
 S.tuning={damage=18,radius=14,fly_speed=3.2,decide_ticks=24,collect_attack=false,stand_off=4,share_attack=false}
 function S.new(g,coop)
  local self=setmetatable({g=g,coop=coop,on=false,policy='seeded',waited={}},S)
  if g.command then
   g.command('synth',function(a) return self:command(a or '') end,'synthetic co-op players: synth start [first|seeded|best] [cpuonly] | stop | wait <ticks> | refusals')
  end
  return self
 end
 function S:command(arg)
  local w={};for x in arg:gmatch('%S+') do w[#w+1]=x end
  if w[1]=='refusals' then -- diagnostic: which engine calls return (nil|false, reason) and how often (the engine's per-callback watchdog counts them)
   local g=self.g
   if not self.counts then
    self.counts={};local counts=self.counts;local skip={log=true,command=true}
    for k,v in pairs(g) do if type(v)=='function' and not skip[k] then
     g[k]=function(...) local a,b=v(...);if (a==nil or a==false) and type(b)=='string' then counts[k]=counts[k] or {n=0,why=b};counts[k].n=counts[k].n+1 end;return a,b end
    end end
    g.log('synth: refusal tracing on');return true
   end
   local list={};for k,r in pairs(self.counts) do list[#list+1]=('%s x%d (%s)'):format(k,r.n,r.why) end;table.sort(list)
   for _,l in ipairs(list) do g.log('synth refused: '..l) end;if #list==0 then g.log('synth refused: none') end;return true
  end
  if w[1]=='gear' then -- synth gear <seat> <id> <id> ...: that seat's bag is replaced by one common drive per id, equipped (cross-player synergy experiments)
   local seat=tonumber(w[2]);local ids={};for k=3,#w do ids[#ids+1]=w[k] end;return self:gear(seat,ids)
  end
  if w[1]=='damage' then S.tuning.damage=math.max(1,math.min(30,math.floor(tonumber(w[2]) or 18)));self.g.log('synth: attack damage '..S.tuning.damage..' per burst');return true end
  if w[1]=='chains' then self:report_chains();return true end
  if w[1]=='chains_reset' then self.chains={total=0,cross=0,by_port={},pairs={}};return true end
  if w[1]=='trace' then self.trace_from=tonumber(w[2]);return true end
  if w[1]=='stop' then self:stop();return true end
  if w[1]=='start' then self:start(w[2] or 'seeded',w[3]=='cpuonly');return true end
  if w[1]=='wait' then S.tuning.decide_ticks=math.max(1,tonumber(w[2]) or 24);self.g.log('synth: a reward screen waits '..S.tuning.decide_ticks..' ticks before the choice');return true end
  self.g.log('synth: usage synth start [first|seeded|best] [cpuonly] | stop | wait <ticks> | refusals');return false
 end
 function S:start(policy,cpu_only) -- cpu_only: pick rewards only for the seats that are CPUs (a person plays the other seat)
  assert(policy=='first' or policy=='seeded' or policy=='best','policy is first, seeded or best')
  self.on=true;self.policy=policy;self.waited={};self.cpu_only=cpu_only==true
  if self.g.fly_speed then pcall(self.g.fly_speed,S.tuning.fly_speed) end
  if self.g.fly_solid then pcall(self.g.fly_solid,true) end
  self.g.log('synth: on, reward policy '..policy)
 end
 function S:stop()
  if not self.on then return end
  self.on=false
  for _,s in ipairs(self.coop.seats or {}) do if self.g.fly_clear then pcall(self.g.fly_clear,s.port) end;if self.g.fly then pcall(self.g.fly,s.port,false) end end
  self.g.log('synth: off')
 end
 local function nearest(list,x,y)
  local best,bd
  for _,e in ipairs(list) do local d=(e.x-x)^2+(e.y-y)^2;if not bd or d<bd then best,bd=e,d end end
  return best
 end
 -- Per logic frame, while a stage is fought: each human-policy seat flies to the nearest living opponent and attacks, or to the nearest floor
 -- drive while the run holds the end for collection.
 -- The synthetic players' attack grows with the loop, standing in for a team's offence (opponents at depth take a fraction of every hit): damage up
 -- to the engine's 30 per burst, and from NG+2 a burst every 15 frames instead of 27.
 function S:firepower(loop)
  local d=math.min(30,S.tuning.damage+4*(loop or 0))
  -- NG+3 opponents can stack knockback_taken to its cap: at 999% a default-knockback burst no longer moves them (found: a Ganondorf pinned at 999% for ever),
  -- so the burst's own knockback (base/growth, engine range 1..2000) is raised for that loop.
  local deep=(loop or 0)>=3
  return {damage=d,radius=S.tuning.radius,gap=((loop or 0)>=2) and 12 or 24,kb=deep and 600 or nil,kbg=deep and 1000 or nil}
 end
 function S:frame()
  if not self.on then return end
  local c=self.coop;if not c.active or c.state~='stage' then return end
  local g=self.g
  self:watch_chains()
  -- Target choice: the nearest living opponent, but one that has not taken damage for a while (invulnerable, out of reach, respawning) is
  -- left alone for a few seconds so both cursors do not sit on it for ever.
  self.seen=self.seen or {};if c.stage_frames==1 or self.seen_stage~=c.stage_id then self.seen={};self.seen_stage=c.stage_id end
  if self.attack_stage~=c.stage_id then self.attack_at={};self.attack_stage=c.stage_id end
  if self.trace_from and c.stage_frames>=self.trace_from and c.stage_frames<self.trace_from+90 then -- diagnostic: `synth trace <frame>` logs every frame of a window
   local t={};for p=1,2+#c.plan.foes do local v=g.player(p);if v then local fs=p<=2 and g.fly_state and g.fly_state(p) or {};t[#t+1]=('P%d(%.0f,%.0f %s%% hl=%s act=%s kb=%.1f/%.1f ph=%s)'):format(p,v.x or 0,v.y or 0,tostring(v.percent),tostring(v.hitlag),tostring(v.action),v.kb_vx or 0,v.kb_vy or 0,tostring(fs.phase)) end end
   g.log(('synth trace f=%d '):format(c.stage_frames)..table.concat(t,' '))
  end
  local foes={}
  for i=1,#c.plan.foes do
   local v=g.player(2+i)
   if v and (v.stocks or 0)>0 and type(v.x)=='number' then
    local st=self.seen[i] or {pct=v.percent or 0,stocks=v.stocks,since=c.stage_frames};self.seen[i]=st
    if (v.percent or 0)~=st.pct or v.stocks~=st.stocks then st.pct=v.percent or 0;st.stocks=v.stocks;st.since=c.stage_frames;st.skip_until=nil;st.engaged=nil end
    if c.stage_frames-st.since>240 and not st.skip_until then st.skip_until=c.stage_frames+300;st.since=c.stage_frames+300 end
    -- A foe that has not taken a point for 8 seconds while we attack it is in a hitlag/armour stalemate (found in the campaigns): both cursors back off
    -- and stop attacking for 1.5 seconds, so every hitlag ends and the foe moves, then attack again.
    if c.stage_frames-(st.engaged or st.since)>480 then self.disengage_until=c.stage_frames+90;st.engaged=c.stage_frames+90;self.disengages=(self.disengages or 0)+1 end
    if st.skip_until and c.stage_frames<st.skip_until then -- left alone for a while (no engine help: the burst attack's own knockback does the launching)
    else foes[#foes+1]=v end
   end
  end
  if #foes==0 then for i=1,#c.plan.foes do local v=g.player(2+i);if v and (v.stocks or 0)>0 and type(v.x)=='number' then foes[#foes+1]=v end end end
  local drops={}
  if c.host.holding_end and g.items then
   local want={};for _,r in pairs(c.mods.drives.drops.records) do want[r.handle]=true end
   for _,e in ipairs(g.items() or {}) do if want[e.handle] and type(e.x)=='number' then drops[#drops+1]=e end end
  end
  -- Two attack cursors on one foe keep it (and themselves) in hitlag for ever (found in the campaigns, traced: each cursor's hit lands every 5
  -- frames, the foe is re-hit before its hitlag ends and never moves): the seats split the opponents, and a seat with no foe of its own
  -- stands off the other seat's foe without attacking (S.tuning.share_attack=true restores both attacking).
  local claimed,drop_claimed={},{}
  -- Floor drives: each seat flies to the nearest drive nobody else has claimed; the seat that gets the first pick alternates by stage, so neither
  -- seat is the one that always touches first (the `first` drop rule would otherwise just reward seat 1's slot in this loop).
  local order={};for i,s in ipairs(c.seats) do order[i]=s end
  if (c.stage_id or 0)%2==1 and #order==2 then order[1],order[2]=order[2],order[1] end
  for _,s in ipairs(order) do
   if s.policy=='human' then
    local me=g.player(s.port)
    if me and (me.stocks or 0)>0 and type(me.x)=='number' then
     local tgt,attack=nil,true
     if #drops>0 then
      local free={};for _,d in ipairs(drops) do if not drop_claimed[d.handle] then free[#free+1]=d end end
      -- a drive nobody else has claimed, else this seat stays where it is (two seats chasing one drive would always give it to whoever is closer)
      tgt=#free>0 and nearest(free,me.x,me.y) or {x=me.x,y=me.y};attack=false;if tgt.handle then drop_claimed[tgt.handle]=true end
     else
      local free={};for _,f in ipairs(foes) do if not claimed[f.port or f] then free[#free+1]=f end end
      tgt=nearest(#free>0 and free or foes,me.x,me.y)
      if tgt then if claimed[tgt.port or tgt] and not S.tuning.share_attack then attack=false end;claimed[tgt.port or tgt]=true end
     end
     if tgt then
      -- a fighter that is dead, held or respawning cannot fly: skip the frame (an uncaught error disables the whole hook after a few)
      local dir=(tgt.x>0) and -1 or 1
      local side=(#drops>0) and 0 or dir*(S.tuning.stand_off+(s.index-1)*3)
      -- a seat that is not attacking keeps well clear: the other seat's capsule hits a teammate too (no damage, but hitlag on both of them, found
      -- in the campaigns: the pair then chained hitlag on a foe parked above the blast zone for ever)
      local disengaged=self.disengage_until and c.stage_frames<self.disengage_until and #drops==0
      if disengaged then attack=false end
      local lift=(#drops>0) and 0 or (disengaged and 250) or (attack and 4 or 70)
      local ok=pcall(g.fly_target,s.port,tgt.x+side,tgt.y+lift)
      -- fly_attack is called only when the on/off state changes: every call re-arms the burst cycle at phase 0, so calling it each frame is the old
      -- every-frame attack (a fresh hit the frame the hitlag ends, a foe that never leaves: the cause of the campaigns' stuck stages).
      if ok then local okf,fs=pcall(g.fly_state,s.port);if okf and type(fs)=='table' and (fs.attacking==true)~=attack then pcall(g.fly_attack,s.port,attack,self:firepower(c.loop)) end end
      if not ok then self.refused=(self.refused or 0)+1 end
     end
    end
   end
  end
 end
 -- ---- cross-player synergy experiments --------------------------------------------------------------------------------------------
 function S:gear(i,ids)
  local c=self.coop;local host=c.hosts[i];if not host then self.g.log('synth gear: no seat '..tostring(i));return false end
  local b=host:bag();local loot=host.mods.drives.loot;local ctx=c.mods.engine.context
  local snap=b:snapshot();snap.items={};snap.equipped={};snap.keystone=nil;snap.keystones={};local ok,why=b:publish(snap)
  if ok==false then self.g.log('synth gear: bag refused '..tostring(why));return false end
  local placed={};local keys={}
  local drives={};for _,id in ipairs(ids) do if id:sub(1,2)=='k_' then keys[#keys+1]=id:sub(3) else drives[#drives+1]=id end end
  if #keys>0 then -- keystones (the untagged appliers: the debug cursor's hits carry no element or move tag, so Kindling and Icebound cannot fire)
   local s2=b:snapshot();s2.keystones=keys;s2.keystone=keys[1];local okk,errk=b:publish(s2);if okk==false then self.g.log('synth gear: keystone refused '..tostring(errk)) else for _,k in ipairs(keys) do placed[#placed+1]='keystone '..k end end
  end
  ids=drives
  for slot,id in ipairs(ids) do
   if slot<=b:slots() then
    local rec;for seed=1,8000 do local r=loot:roll(seed+i*1000003,ctx,'common');if #r.affixes==1 and r.affixes[1].id==id then rec=r;break end end
    if rec then local ok2,err=b:place(slot,rec);if ok2~=false then placed[#placed+1]=id else self.g.log('synth gear: '..id..' refused '..tostring(err)) end else self.g.log('synth gear: no common drive rolls '..id) end
   end
  end
  host:touch()
  self.g.log(('synth gear: seat %d now holds %s (%d slots)'):format(i,table.concat(placed,', '),b:slots()));return true
 end
 -- Chain firings seen by the engine's own trace (what the synergy visual reads): how many, how many join a record only seat 1 holds with one
 -- only seat 2 holds (a cross-seat chain), and which port the engine attributed it to.
 function S:watch_chains()
  local c=self.coop;local e=c.mods and c.mods.engine;if not e then return end
  local gen=table.concat(e.trace or {},'|')..'#'..tostring(e.trace_port) -- the trace text itself (the display's generation counter is not advanced for a run)
  if gen==self.chain_gen then return end
  self.chain_gen=gen;self.chains=self.chains or {total=0,cross=0,by_port={},pairs={}}
  local fx=D.synergy_fx;if not (fx and fx.trace_ids) then return end
  local ids=fx.trace_ids(e);if #ids<2 then return end
  local own={}
  for _,id in ipairs(ids) do local o={};for p=1,2 do if (e.equipped[p] or {})[id] then o[#o+1]=p end end;own[id]=o end
  local a,b;for _,id in ipairs(ids) do local o=own[id];if #o==1 then if o[1]==1 then a=a or id else b=b or id end end end
  local ch=self.chains;ch.total=ch.total+1;local port=e.trace_port or 0;ch.by_port[port]=(ch.by_port[port] or 0)+1
  if a and b then ch.cross=ch.cross+1;local k=a..'>'..b;ch.pairs[k]=(ch.pairs[k] or 0)+1 end
 end
 function S:report_chains()
  local ch=self.chains or {total=0,cross=0,by_port={},pairs={}};local g=self.g
  local bp={};for p,n in pairs(ch.by_port) do bp[#bp+1]=('P%s=%d'):format(tostring(p),n) end;table.sort(bp)
  local pr={};for k,n in pairs(ch.pairs) do pr[#pr+1]=k..'='..n end;table.sort(pr)
  g.log(('synth chains: total=%d cross_seat=%d attributed %s pairs %s'):format(ch.total,ch.cross,table.concat(bp,','),table.concat(pr,',')))
  for i,h in ipairs(self.coop.hosts) do local f=h.synfx
   if f then local o={};for p,cn in pairs(f.counter or {}) do o[#o+1]=('P%d %s x%d'):format(p,cn.arch and cn.arch.id or '?',cn.n or 0) end;table.sort(o)
    local fl={};for _,x in ipairs(f.flashes or {}) do fl[#fl+1]=('%s:P%s>P%s%s'):format(x.arch and x.arch.id or '?',tostring(x.from),tostring(x.to),x.cross and '*' or '') end
    g.log(('synth chains: host %d counters [%s] last flashes [%s] cross-seat flashes drawn %d'):format(i,table.concat(o,'; '),table.concat(fl,' '),f.cross or 0))
   end
  end
 end
 -- ---- reward choices ------------------------------------------------------------------------------------------------------------
 local function pick(c,host,n,salt) local r=D.coop.seed_for(c.seed,c.stage,c.loop,host:port0()+500+salt);return (r%n)+1 end
 -- The strength a held build would have after taking offer i (merge into a held drive, or place into a free slot); unchanged otherwise.
 local function gain_value(host,i)
  local r=host.offers[i];local plan=host:plan_take(r)
  if plan.action~='merge' and plan.action~='equip' then return host:totals().strength end
  return host:totals(function(draft)
   if plan.action=='merge' then draft:replace(plan.loc.where,plan.loc.index,plan.merged) else draft:place(plan.slot,r) end
  end).strength
 end
 function S:decide(host)
  local c=self.coop;local n=#host.offers
  if #host.decide>0 and #host.offers==0 then host:leave_choice('synthetic: nothing to give up') end
  if n>0 then
   local i=1
   if self.policy=='seeded' then i=pick(c,host,n,1)
   elseif self.policy=='best' then local best=-1;for k=1,n do local v=gain_value(host,k);if v>best then best,i=v,k end end end
   local action,why=host:take_offer(i)
   if not action and why=='choose' then
    local slot=(pick(c,host,host:bag():slots(),2));local r=host.offers[i];host:replace_with({record=r,from='offer'},'equipped',slot)
   end
  end
  while #host.key_offers>0 do
   local before=#host.key_offers
   local id=host.key_offers[self.policy=='first' and 1 or pick(c,host,#host.key_offers,3)]
   local ok=host:take_keystone(id);if not ok then host.key_offers={};break end
   if #host.key_offers>=before then break end
  end
  while #host.decide>0 do host:leave_choice('synthetic') end
  host.screen:finish('done')
 end
 function S:tick()
  if not self.on then return end
  local c=self.coop;if not c.active then return end
  local h=c.screen_owner
  if h and h.screen.active and h.screen.mode=='reward' and (not self.cpu_only or h.seat.policy=='cpu') then
   self.waited[h]=(self.waited[h] or 0)+1
   if self.waited[h]>=S.tuning.decide_ticks then self.waited[h]=nil;self:decide(h) end
  end
 end
 return S
end
