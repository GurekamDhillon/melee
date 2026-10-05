-- Synthetic players for the co-op run, for the lane that found out what the design needs: both players flown by the debug cursor
-- (gd.fly_target / gd.fly_attack, offline only), reward choices made by a seeded policy through the same host calls the screen's buttons end in.
-- Off unless started (`envoy synth start [policy]`); it is a test driver, never part of a normal run. Hurtboxes stay on while flying
-- (gd.fly_solid) so the players can take damage and lose stocks like anyone else.
return function(D)
 local S={};S.__index=S
 S.tuning={damage=6,radius=9,fly_speed=3.2,decide_ticks=24,collect_attack=false}
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
 function S:frame()
  if not self.on then return end
  local c=self.coop;if not c.active or c.state~='stage' then return end
  local g=self.g
  -- Target choice: the nearest living opponent, but one that has not taken damage for a while (invulnerable, out of reach, respawning) is
  -- left alone for a few seconds so both cursors do not sit on it for ever.
  self.seen=self.seen or {};if c.stage_frames==1 or self.seen_stage~=c.stage_id then self.seen={};self.seen_stage=c.stage_id end
  local foes={}
  for i=1,#c.plan.foes do
   local v=g.player(2+i)
   if v and (v.stocks or 0)>0 and type(v.x)=='number' then
    local st=self.seen[i] or {pct=v.percent or 0,stocks=v.stocks,since=c.stage_frames};self.seen[i]=st
    if (v.percent or 0)~=st.pct or v.stocks~=st.stocks then st.pct=v.percent or 0;st.stocks=v.stocks;st.since=c.stage_frames;st.skip_until=nil end
    if c.stage_frames-st.since>240 and not st.skip_until then st.skip_until=c.stage_frames+300;st.since=c.stage_frames+300 end
    if st.skip_until and c.stage_frames<st.skip_until then
     -- the debug attack carries no knockback, so a foe at 999% never leaves the stage: a real injected hit (gd.hit) launches it
     if c.stage_frames%15==0 and g.hit then pcall(g.hit,2+i,{damage=1,angle=45,kbg=250,bkb=250,from=((i%2)==0) and 2 or 1}) end
    else foes[#foes+1]=v end
   end
  end
  if #foes==0 then for i=1,#c.plan.foes do local v=g.player(2+i);if v and (v.stocks or 0)>0 and type(v.x)=='number' then foes[#foes+1]=v end end end
  local drops={}
  if c.host.holding_end and g.items then
   local want={};for _,r in pairs(c.mods.drives.drops.records) do want[r.handle]=true end
   for _,e in ipairs(g.items() or {}) do if want[e.handle] and type(e.x)=='number' then drops[#drops+1]=e end end
  end
  for _,s in ipairs(c.seats) do
   if s.policy=='human' then
    local me=g.player(s.port)
    if me and (me.stocks or 0)>0 and type(me.x)=='number' then
     local tgt=#drops>0 and nearest(drops,me.x,me.y) or nearest(foes,me.x,me.y)
     if tgt then
      -- a fighter that is dead, held or respawning cannot fly: skip the frame (an uncaught error disables the whole hook after a few)
      local ok,err=pcall(g.fly_target,s.port,tgt.x,tgt.y+((#drops>0) and 0 or 4))
      if ok then pcall(g.fly_attack,s.port,#drops==0,S.tuning.damage,S.tuning.radius) else self.refused=(self.refused or 0)+1 end
     end
    end
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
