-- LAB manual spawns branch native rewind history. No spawning from frame/replay.
return function(D)
 local R={};R.__index=R
 local element_colour={fire='red',electric='yellow',ice='blue',darkness='purple'}
 function R.core_colour(record)
  local present={};for _,a in ipairs(record.affixes) do present[a.id]=true end
  local colour=record.colour
  for _,m in ipairs(D.mod_pool) do if present[m.id] then for _,effect in ipairs(m.effects) do
   if effect.op=='convert' and effect.change and element_colour[effect.change.element] then colour=element_colour[effect.change.element] end
  end end end;return colour
 end
 function R.new(g)
  D.pickup_juice.pitch.purple=150
  local presentation=setmetatable({fx_world=function(name,...) if not g.fx_world then return nil end;return g.fx_world(name:gsub('PickupJuice_(.-)_purple','DriveLoot_%1_purple'),...) end},{__index=g})
  return setmetatable({g=g,next_id=1,records={},juice=D.pickup_juice.new(presentation)},R)
 end
 function R:count() local n=0;for _ in pairs(self.records) do n=n+1 end;return n end
 -- Which engine item a drop is. `press` (set by the run host for an Envoy run): collected by pressing A on it, as a retail item is
 -- (collection "press": geno_game_items.inc, ftpickupitem.c); `drive_payout*` is the stage-end payout's drive, which does not fade
 -- (a 10 minute lifetime, no blink: the hold waits for the player). Without `press` the old touch items, as the LAB and the missions use them.
 R.names={drive=true,drive_coop=true,drive_press=true,drive_press_coop=true,drive_payout=true,drive_payout_coop=true}
 function R:item_for(opts)
  local coop=(self.item_name or 'drive'):find('coop',1,true)~=nil
  if not self.press then return self.item_name or 'drive' end
  return (opts and opts.payout and 'drive_payout' or 'drive_press')..(coop and '_coop' or '')
 end
 -- `at_x, at_y`: where the drop appears (a run drops where the player can reach it); default beside CPU 2 (the LAB).
 -- `opts.payout`: the stage-end payout's drive (it does not fade, and its record is marked so a restored checkpoint keeps it).
 function R:spawn(record,at_x,at_y,opts)
  assert(self:count()<12,'drop capacity exhausted')
  if not at_x then local p=self.g.player(2) or self.g.player(1);assert(p,'fighter absent');at_x,at_y=p.x+10,p.y+8 end
  local id=self.next_id;assert(id<=1000000,'drop id budget exhausted');local c=record.colour=='purple' and 'white' or record.colour
  local h,why=self.g.item_spawn(self:item_for(opts),at_x,at_y,{payload={colour=c,amount=id}}) -- co-op spawns `drive_coop`: the same item either player may touch (ports mask 3)
  assert(h,why or 'drive spawn refused');self.next_id=id+1
  self.records[id]={handle=h,record=record,payout=opts and opts.payout or nil};self:visual(h,record,at_x,at_y,opts and opts.payout)
  return h
 end
 function R:visual(h,record,x,y,payout)
  local d=self.juice:drop(h,R.core_colour(record),x,y,payout and 2147483647 or 900)
  if record.rarity~="common" and self.g.fx_world then for _,kind in ipairs({"beam","sparkles"}) do d.fx["rarity_"..kind]=self.g.fx_world("DriveLoot_"..kind.."_"..record.rarity,x,y+12,0,1,1) end end
  if record.unique and self.g.fx_world then d.fx.signature=self.g.fx_world('DriveLoot_signature_'..record.unique,x,y+12,0,1,1) end
 end
 -- `hosted`: a run's host decides what happens to the drive (merge, bag, ask), so nothing is given to the bag here.
 function R:pickup(e,bag,hosted,port)
  if not R.names[e.name] or e.port~=(port or 1) then return false end
  local id=e.payload and e.payload.amount;local d=id and self.records[id]
  if not d or d.handle~=(e.item or e.handle) then return false end
  if not hosted then assert(bag:give(d.record)) end;self.records[id]=nil;self.juice:collect(d.handle,nil,self.g.player(e.port or 1));return d.record
 end
 function R:expire(e)
  local id=e.payload and e.payload.amount;local d=id and self.records[id]
  if d and d.handle==(e.item or e.handle) then self.records[id]=nil;self.juice:expire(d.handle);return d.record end
 end
 function R:snapshot() return {next_id=self.next_id,records=self.records} end
 function R:validate(s)
  assert(type(s)=='table' and type(s.records)=='table' and type(s.next_id)=='number' and s.next_id%1==0 and s.next_id>=1 and s.next_id<=1000001,'invalid drop checkpoint')
  for k in pairs(s) do assert(k=='next_id' or k=='records','unknown drop checkpoint field') end
  local count=0;local handles={}
  for id,d in pairs(s.records) do
   assert(type(id)=='number' and id%1==0 and id>=1 and id<s.next_id and type(d)=='table' and type(d.handle)=='number' and d.handle%1==0 and d.handle>0 and d.handle<=2147483647,'invalid drop record')
   for k in pairs(d) do assert(k=='handle' or k=='record' or k=='payout','unknown drop record field') end
   assert(not handles[d.handle],'duplicate drop handle');handles[d.handle]=true
   assert(type(d.record)=='table','missing loot record');count=count+1
  end
  assert(count<=12,'drop checkpoint capacity exceeded');return count
 end
 function R:restore(s)
  self:validate(s)
  self.juice:clear();self.next_id=s.next_id;self.records=s.records
  for _,d in pairs(self.records) do if self.retired then self.retired[d.handle]=nil end end
  local items={};for _,e in ipairs(self.g.items and self.g.items() or {}) do if type(e.handle)=='number' and e.handle>0 and e.handle%1==0 then items[e.handle]=e end end
  for _,d in pairs(self.records) do local e=items[d.handle];if e then self:visual(d.handle,d.record,e.x,e.y,d.payout) end end
 end
 function R:clear()
  self.retired=self.retired or {}
  for _,d in pairs(self.records) do self.retired[d.handle]=true end
  self:retry_retired()
  self.juice:clear();self.records={};self.next_id=1
 end
 function R:retry_retired()
  local live={};if self.g.items then for _,e in ipairs(self.g.items()) do if type(e.handle)=='number' and e.handle>0 and e.handle%1==0 then live[e.handle]=true end end end
  for h in pairs(self.retired or {}) do
   local ok,result=false,false;if self.g.item_despawn then ok,result=pcall(self.g.item_despawn,h) end
   if ok and result~=false or self.g.items and not live[h] then self.retired[h]=nil end
  end
 end
 -- ---- the stage-end payout: pure helpers (run_host.lua drives them) -----------------------------------------------------------
 -- The stage ends on the last opponent's defeat, and a fast clear can beat every mid-fight drop (a one-stock opponent that goes
 -- from 40% to KO in one hit never "passes 50%" and its stock loss is only seen at the next poll). So the end is always held
 -- (gd.match_end_hold, in run_host.lua), the drops the stage still OWES are decided then, and they arrive on the stage one after
 -- another (the sequence below) as ordinary collectable drives (A on them). Nothing here touches the engine except through `g`.
 local P={};R.payout=P
 -- arrive_height: how far above the floor a payout drive appears (it falls in under the item's own gravity, 0.12/frame^2).
 -- lead: frames from the clear to the first arrival; gaps[n]: frames between arrivals while n are still to come (about a
 -- second each, quicker when several). leave_frames: how long the leave chord must be held. idle_frames: a player who has not
 -- touched the pad for this long (2 minutes) is treated as having walked away. span_max: a main floor wider than this is a
 -- scrolling stage (the drives arrive around the player instead). near_span: half the width of that area.
 P.tuning={arrive_height=70,lead=40,gaps={[1]=60,[2]=42,[3]=30,[4]=24},ring_frames=45,ring_scale=2.4,burst_frames=24,
  leave_frames=60,idle_frames=7200,span_max=500,near_span=140,edge=24,spot_step=8,spot_reach=64}
 -- Which opponents' drop decisions were never made. `st`: {kos=decisions made so far (a roll that failed counts), dropped={[port]=true}
 -- (team stages: that opponent has had its one), given=true when this is a retry of a stage that already gave its drop}.
 -- Mirrors run_host.lua H:on_ko exactly (a battle stage has `max` slots in all, a team stage one per opponent up to `max`),
 -- so a decision made here has the same seed and the same chance as the one made mid-fight would have had.
 function P.owed_foes(kind,max,foes,st)
  local out={}
  if kind=='bonus' or kind=='boss' or (max or 0)<=0 or st.given then return out end
  local kos=st.kos or 0
  for _,p in ipairs(foes) do
   if kos>=max then break end
   if kind=='team' then if not (st.dropped and st.dropped[p]) then out[#out+1]=p;kos=kos+1 end
   else out[#out+1]=p;break end
  end
  return out
 end
 -- Frames to wait after an arrival while `remaining` more are to come.
 function P.gap(remaining)
  local g=P.tuning.gaps;return g[math.min(math.max(remaining,1),#g)]
 end
 -- Where `n` drives land: spread evenly across area.lo..area.hi, centre first, each on a floor `area.floor(x)` that exists and is
 -- within `area.tol` of `area.y` (the main floor: not over a pit, not on a platform that needs a jump), nudged sideways when
 -- the exact x has no floor. Returns up to n {x=,y=} (fewer when the stage offers no such floor).
 function P.spots(n,area)
  local out={};if n<=0 or not area or area.hi<=area.lo then return out end
  local t=P.tuning;local order={}
  for i=1,n do order[i]={frac=i/(n+1),i=i} end
  table.sort(order,function(a,b) local da,db=math.abs(a.frac-.5),math.abs(b.frac-.5);if da~=db then return da<db end;return a.frac<b.frac end)
  for _,o in ipairs(order) do
   local x0=area.lo+(area.hi-area.lo)*o.frac;local found
   for off=0,t.spot_reach,t.spot_step do
    for _,sgn in ipairs(off==0 and {1} or {1,-1}) do
     local x=x0+off*sgn
     if x>=area.lo and x<=area.hi and not found then
      local y=area.floor(x);if y and math.abs(y-area.y)<=area.tol then found={x=x,y=y} end
     end
    end
    if found then break end
   end
   if found then out[#out+1]=found end
  end
  return out
 end
 -- The area drives land in, from the stage's main floor (gd.stage_bounds) and the player: the whole main floor inside `edge`,
 -- or, on a scrolling stage (no main floor, or one wider than span_max), a window around the player. `floor(x)` is
 -- gd.floor_below from above the main floor's top. Returns nil when there is no player to anchor on.
 function P.area(g,player)
  local t=P.tuning;local b=g.stage_bounds and g.stage_bounds();local mf=b and b.main_floor
  local floor_below=g.floor_below
  if not player or type(player.x)~='number' or not floor_below then return nil end
  local lo,hi,y
  if mf and mf.right-mf.left<=t.span_max then lo,hi=mf.left+t.edge,mf.right-t.edge;y=mf.top
  else
   lo,hi=player.x-t.near_span,player.x+t.near_span;y=player.y
   if mf then lo,hi=math.max(lo,mf.left+t.edge),math.min(hi,mf.right-t.edge);y=mf.top end
  end
  local top=y+60
  local tol=mf and math.max(12,(mf.top-(mf.bottom or mf.top))*1+10) or 14
  return {lo=lo,hi=hi,y=y,tol=tol,floor=function(x) return floor_below(x,top) end}
 end
 -- The arrival sequence: entries {record=,port=,why=} queue up (the owed drops, decided when the last opponent falls) and come out one at a
 -- time. `step()` is called once per logic frame and returns the next entry when it is due (else nil).
 function P.seq()
  local s={queue={},wait=P.tuning.lead,fx={},seed=1,arrived=0}
  function s:add(e) self.queue[#self.queue+1]=e end
  function s:pending() return #self.queue end
  function s:step()
   if #self.queue==0 then return nil end
   if self.wait>0 then self.wait=self.wait-1;return nil end
   local e=table.remove(self.queue,1);self.arrived=self.arrived+1;self.wait=P.gap(#self.queue+1)
   return e
  end
  -- the arrival's own transient effects end by themselves after their frames (a pooled ring, a burst)
  function s:tick_fx(g)
   for i=#self.fx,1,-1 do local f=self.fx[i];f.left=f.left-1
    if f.left<=0 then if g.fx_end then pcall(g.fx_end,f.h,0) end;table.remove(self.fx,i) end end
  end
  -- Everything not yet arrived is skipped (the player left, or the stage was cleared another way); returns the skipped entries.
  function s:skip() local q=self.queue;self.queue={};return q end
  function s:clear(g) for _,f in ipairs(self.fx) do if g and g.fx_end then pcall(g.fx_end,f.h,0) end end;self.fx={};return self:skip() end
  return s
 end
 -- THE FLOURISH. Simple on purpose: it is tuned live with the owner. A pooled ring on the floor and a burst where the drive appears,
 -- in the drive's own colour (the packages the mid-fight drop already uses), and the drive itself falls in from `arrive_height`
 -- (the host spawns it there). Presentation only; it decides nothing.
 -- HOOK: `P.hooks.spire(g,x,floor_y,colour,record)` is called after the ring and the burst, for a richer effect (the spire the owner
 -- liked as a scratch). It may return a handle; give it to `seq.fx` as {h=,left=} to have it ended for you.
 P.hooks={spire=nil}
 function P.flourish(g,seq,x,floor_y,record)
  local t=P.tuning;local c=R.core_colour(record)
  local function pkg(kind) if c=='purple' then return 'DriveLoot_'..kind..'_purple' end;return 'PickupJuice_'..kind..'_'..c end
  local function fx(kind,y,scale,frames)
   if not g.fx_world then return end
   seq.seed=(seq.seed%2147483646)+1
   local ok,h=pcall(g.fx_world,pkg(kind),x,y,0,scale,seq.seed)
   if ok and type(h)=='number' and h>0 then seq.fx[#seq.fx+1]={h=h,left=frames} end
  end
  fx('pool',floor_y+.15,t.ring_scale,t.ring_frames)
  fx('collect_burst',floor_y+t.arrive_height*.5,1,t.burst_frames)
  if P.hooks.spire then pcall(P.hooks.spire,g,x,floor_y,c,record,seq) end
 end
 -- Leaving: hold Z and D-pad Down together for `leave_frames` (a second). START is the pause, so it is not used; the chord is held on
 -- purpose for a moment, so it cannot be hit by accident. `feed(pad,frames)` is given the raw pad (gd.pad(port,true)) and the frames
 -- since the last call; true once the chord has been held long enough (and it stays true until reset).
 function P.leave_watch()
  local w={held=0,done=false}
  function w:feed(pad,frames)
   if self.done then return true end
   if pad and pad.Z and pad.DOWN then self.held=self.held+(frames or 1) else self.held=0 end
   if self.held>=P.tuning.leave_frames then self.done=true end
   return self.done
  end
  function w:progress() return math.min(1,self.held/P.tuning.leave_frames) end
  return w
 end
 -- A player who left the controller: `feed(pad,frames)` true after `idle_frames` with no button and no stick movement.
 function P.idle_watch()
  local w={idle=0}
  function w:feed(pad,frames)
   local active=pad and ((pad.buttons or 0)~=0 or math.abs(pad.x or 0)>20 or math.abs(pad.y or 0)>20 or math.abs(pad.cx or 0)>20 or math.abs(pad.cy or 0)>20)
   if active then self.idle=0 else self.idle=self.idle+(frames or 1) end
   return self.idle>=P.tuning.idle_frames
  end
  return w
 end
 -- When does the hold end? One place for the rule. `s`: {floor=drives on the ground, queued=drives still to arrive, alive=a player
 -- of the team still has a stock, left=the leave chord completed, idle=the player walked away}. Returns nil to keep holding, else why.
 -- The order matters: a player who is out loses the stage normally (no payout); leaving skips what is left.
 function P.release_reason(s)
  if not s.alive then return 'the player is out' end
  if s.left then return 'the player left' end
  if s.idle then return 'the player is away' end
  if (s.floor or 0)+(s.queued or 0)==0 then return 'every drive collected' end
  return nil
 end
 return R
end
