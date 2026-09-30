-- Custom actors use the same Core hosts, instances, costs and authored abilities.
-- Call tick once AFTER Core.tick during unpaused gameplay. Never cache a run.
local E={}
local function count(t) local n=0 for _ in pairs(t) do n=n+1 end return n end
local function copy(t) if type(t)~='table' then return t end local o={} for k,v in pairs(t) do o[k]=copy(v) end return o end
function E.new(Core,gd,opts)
 assert(type(opts)=='table' and type(opts.get_run)=='function','get_run required')
 local self={records={}}
 local function view(s,r,q)
  local a=r and Core.ability(r,s.host,s.slot)
  return {handle=s.handle,host=s.host,family=s.family,slot=s.slot,x=q and q.x or s.x,y=q and q.y or s.y,
   facing=q and q.facing or s.facing,phase=s.phase,charge=a and a.charge or 0,cost=a and a.cost or 0,
   ability=a,windup_until=s.windup_until}
 end
 local function tell(s,r,q,event) if opts.on_tell then opts.on_tell(view(s,r,q),event) end end
 local function phase(s,r,q,p) if s.phase~=p then s.phase=p;tell(s,r,q,p) end end
 function self:attach(handle,o)
  o=o or {};local r=opts.get_run();local q=gd.enemy_state(handle)
  if not r or r.status~='active' or not q then return nil,'inactive actor/run' end
  if type(o.host)~='string' or #o.host<1 or #o.host>64 or o.host:find('[%z\1-\31]') or o.host=='player' then return nil,'invalid enemy host' end
  if o.family~='cinder' and o.family~='rime' then return nil,'unsupported family' end
  if o.slot and o.slot~='assault' then return nil,'custom controller supports assault only' end
  for h,s in pairs(self.records) do if s.host==o.host and h~=handle then return nil,'host already attached' end end
  if self.records[handle] then return view(self.records[handle],r,q) end
  local h=r.hosts[o.host];local id=h and h.slots.assault
  if id and r.genes[id].kind~=o.family then return nil,'host family mismatch' end
  if not id then
   if not h and count(r.hosts)>=32 then return nil,'host capacity' end
   local seed,next_id=r.seed,r.next_id
   id=Core.acquire(r,o.family);if not id then return nil,'gene capacity' end
   local ok,err=Core.equip(r,o.host,'assault',id)
   if not ok then r.genes[id]=nil;r.seed=seed;r.next_id=next_id;return nil,err end
  end
  -- Authored encounter traits: faster charging, stronger signature. Source stays immutable.
  assert(Core.apply_modifier(r,o.host,'assault',{id='encounter_cost',stat='capacity',add=-2}))
  assert(Core.apply_modifier(r,o.host,'assault',{id='encounter_power',stat='potency',add=1}))
  local s={handle=handle,host=o.host,slot='assault',family=o.family,hits=q.hits or 0,
   received=q.received or 0,phase='charging',retry_at=0,x=q.x,y=q.y,facing=q.facing}
  self.records[handle]=s;tell(s,r,q,'attached');return view(s,r,q)
 end
 function self:detach(handle)
  local s=self.records[handle];if not s then return false end
  s.phase='dead';tell(s,opts.get_run(),nil,'detached');self.records[handle]=nil;return true
 end
 function self:clear() local keys={} for h in pairs(self.records) do keys[#keys+1]=h end for _,h in ipairs(keys) do self:detach(h) end end
 function self:states()
  local a={};local r=opts.get_run();for h,s in pairs(self.records) do a[#a+1]=view(s,r,gd.enemy_state(h)) end
  table.sort(a,function(x,y)return x.handle<y.handle end);return a
 end
 function self:tick()
  local r=opts.get_run();if not r or r.status~='active' then self:clear();return end
  local handles={} for h in pairs(self.records) do handles[#handles+1]=h end table.sort(handles)
  for _,handle in ipairs(handles) do
   local s=self.records[handle];local q=gd.enemy_state(handle)
   if not q or not r.hosts[s.host] or not r.hosts[s.host].slots[s.slot] then self:detach(handle)
   elseif s.frame~=r.frame then
    s.frame=r.frame;s.x=q.x;s.y=q.y;s.facing=q.facing
    if (q.hits or 0)>s.hits then
     Core.on_event(r,{host=s.host,kind='direct_hit',move_id=s.host..':'..tostring(q.attack_id),lineage='direct'})
    end
    s.hits=q.hits or 0
    if opts.poll_player_hits and (q.received or 0)>s.received and (q.last_attacker or 0)>0 and opts.on_player_hit then
     opts.on_player_hit(handle,q.last_attacker,q.last_damage or 0)
    end
    s.received=q.received or 0
    local a=Core.ability(r,s.host,s.slot)
    local port=opts.target_port or 1;local p=gd.player(port)
    local in_range=a and p and math.abs(p.x-q.x)<=a.reach and math.abs(p.y-q.y)<=12 and (p.x-q.x)*(q.facing or 1)>=0
    if s.phase=='telegraph' and r.frame>=s.windup_until then
     local st=r.hosts[s.host].state[s.slot];local charge,ready=st.charge,st.ready_at
     local target=opts.target_host or 'player';local mark=copy(r.marks[target])
     local action=Core.activate(r,s.host,s.slot,{target=target})
     local ok=action and in_range and gd.enemy_strike(handle,port,{damage=math.floor(action.damage),
      angle=action.knockback.angle,kbg=action.knockback.kbg,bkb=action.knockback.bkb,reach=action.reach})
     s.windup_until=nil
     if ok then phase(s,r,q,'recovery');tell(s,r,q,'release')
     else st.charge=charge;st.ready_at=ready;r.marks[target]=mark;s.retry_at=r.frame+20;phase(s,r,q,'ready');tell(s,r,q,'refused') end
    elseif s.phase~='telegraph' then
     if a and a.ready then
      phase(s,r,q,'ready')
      if in_range and r.frame>=s.retry_at then s.windup_until=r.frame+24;phase(s,r,q,'telegraph') end
     elseif a and a.remaining>0 then phase(s,r,q,'recovery')
     else phase(s,r,q,'charging') end
    end
   end
  end
 end
 return self
end
return E
