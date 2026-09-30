-- Transactional physical-room runtime (Gate 5 seam). An injected service that
-- consumes a resolved adapter node and the room module's pure planner, loads its
-- assets incrementally, constructs its collision (floor segments, platforms and
-- real ascent slopes through gd.stage_add_line) and spawns its visuals. It owns
-- the native handles, the per-room Rooms state and the transactional fighter
-- placements; main owns the route/progress transaction around it.
--
-- Correct transition order (gameplay paused throughout):
--   1. begin(dest,{from=active,exit=canonical_exit}); step until 'ready'
--   2. place fighters at :arrival(tx); stage and validate Route/Core/progress
--   3. persist successfully (save)
--   4. only then commit(tx): this destructively retires the source room
-- If save is refused, rollback(tx) destroys the destination and restores every
-- moved fighter, leaving the source untouched. commit can never restore the
-- source, so it must not run before a durable save. If the engine refuses to
-- retire the source after a durable save, commit reports released=false and
-- keeps the source handles in :pending_source; gameplay must stay paused until
-- :flush()/cleanup() actually retires those colliders.
--
-- Construction is destination-first: the source stays resident until commit, so
-- a refused allocation, a lost isolation, a placement refusal or an interrupted
-- step can be rolled back without losing the current room. The only physical
-- limitation is that the engine has no per-room grouping, so while a destination
-- is built both rooms' colliders are live in the same isolated stage; the overlap
-- is bounded (max_total_lines) and lasts until commit. This module never
-- certifies a recipe: the adapter's certification gate stays upstream.
--
-- Engine and room modules are injected, never read from globals:
--   engine: stage_isolate, stage_add_platform, stage_add_line, stage_link, stage_remove,
--           player, teleport (player/teleport are required for :place)
--   rooms:  new, plan, collision, anchor, arrival, preload_step, enter, clear,
--           release, view
local RuntimeRooms={version=1,max_total_lines=32}
local function copy(t)
 if type(t)~='table' then return t end
 local out={} for k,v in pairs(t) do out[k]=copy(v) end return out
end
local function finite(v) return type(v)=='number' and v==v and math.abs(v)<math.huge end
local function handle(h) return finite(h) and h%1==0 and h>0 end

-- Only the caller's newly allocated room floors participate. Sharing a native
-- script owner with a resident source room must never weld the two rooms. Check
-- the entire endpoint graph before linking, so a fork cannot pick an arbitrary
-- neighbour. Interiors, crossings and same-side endpoints are not seams.
function RuntimeRooms.link_floor_seams(engine,entries)
 local seams,candidates,seen={},{},{}
 local epsilon2=0.05*0.05
 for i,e in ipairs(entries) do
  if not handle(e.handle) or seen[e.handle] or not finite(e.x0) or not finite(e.y0)
   or not finite(e.x1) or not finite(e.y1) or e.x0>=e.x1 then return false,'invalid room floor endpoint record' end
  seen[e.handle]=true;candidates[i]={left=0,right=0}
 end
 for i,a in ipairs(entries) do
  for j,b in ipairs(entries) do
   if i~=j then
    local dx,dy=a.x1-b.x0,a.y1-b.y0
    if dx*dx+dy*dy<=epsilon2 then
     candidates[i].right=candidates[i].right+1;candidates[j].left=candidates[j].left+1
     seams[#seams+1]={a.handle,b.handle}
    end
   end
  end
 end
 for _,c in ipairs(candidates) do
  if c.left>1 or c.right>1 then return false,'ambiguous room floor seam' end
 end
 if #seams==0 then return true,0 end
 if type(engine.stage_link)~='function' then return false,'stage_link API unavailable for room floor seams' end
 for _,s in ipairs(seams) do
  local ok,linked,why=pcall(engine.stage_link,s[1],s[2])
  if not ok or linked~=true then
   return false,'room floor seam link refused: '..tostring(not ok and linked or why or linked)
  end
 end
 return true,#seams
end

function RuntimeRooms.new(engine,rooms,opts)
 assert(type(engine)=='table' and engine.stage_add_platform and engine.stage_add_line and engine.stage_remove,'stage engine required')
 assert(type(rooms)=='table' and rooms.new and rooms.plan and rooms.collision and rooms.anchor
  and rooms.arrival and rooms.preload_step and rooms.enter and rooms.clear and rooms.release and rooms.view,'rooms module required')
 opts=opts or {}
 local self={engine=engine,rooms=rooms,max_lines=opts.max_lines or 16,
  max_total_lines=opts.max_total_lines or RuntimeRooms.max_total_lines,
  drop_margin=opts.drop_margin or 4,drop_speed=opts.drop_speed or 0.1,
  door_reach=opts.door_reach or 13,vertical_reach=opts.vertical_reach or 10,
  active=nil,busy=nil,pending_source=nil,serial=0,total_lines=0}
 return setmetatable(self,{__index=RuntimeRooms})
end

-- Isolation is a caller-owned precondition, never toggled here. It is checked
-- before begin and again before every construction step, so main keeps its own
-- stage_isolate(true/false) symmetry.
function RuntimeRooms:isolated()
 local ok,value=false,false
 if self.engine.stage_isolate then ok,value=pcall(self.engine.stage_isolate) end
 return ok and value==true
end

-- A caller-supplied exit is accepted only if it is one of the source node's
-- canonical adapter exits (by identity or by every canonical socket/target
-- field). A forged exit cannot change the target or arrival, and the trigger is
-- resolved from the canonical exit, not the caller's copy.
function RuntimeRooms:_canonical_exit(source_node,exit)
 if type(source_node)~='table' or type(source_node.exits)~='table' then return nil end
 for _,c in ipairs(source_node.exits) do
  if c==exit then return c end
  if type(c)=='table' and exit.edge_id~=nil and c.edge_id==exit.edge_id and c.socket==exit.socket
   and c.to==exit.to and c.arrival_socket==exit.arrival_socket then return c end
 end
 return nil
end

-- Pure per-exit resolution against the canonical adapter exit. The source socket
-- supplies the trigger (explicit anchor first) and the destination socket the
-- arrival, so a fork, merge entry, return, upper socket or one-way drop keeps
-- its own identity instead of a fixed room/side/two-door whitelist.
function RuntimeRooms:resolve_exit(source_node,dest_node,exit)
 if type(exit)~='table' then return nil,'missing route exit' end
 local trigger=self.rooms.anchor(source_node,exit)
 if not trigger or not finite(trigger.x) or not finite(trigger.y) then return nil,'unresolved source trigger' end
 local info={trigger=trigger,side=exit.side,socket=exit.socket,arrival_socket=exit.arrival_socket,
  to=exit.to,edge_id=exit.edge_id,kind=exit.kind,label=exit.label,drop=trigger.drop==true}
 if info.drop then
  local f=source_node and source_node.room and source_node.room.floor
  local opening=nil
  for _,o in ipairs(f and f.openings or {}) do
   if finite(o.x) and finite(o.width) and math.abs(o.x-trigger.x)<=o.width/2 then
    opening={x=o.x,width=o.width};break end
  end
  if not opening then return nil,'drop trigger has no authored floor opening' end
  info.opening=opening
 end
 if dest_node then
  local arrival=self.rooms.arrival(dest_node,exit.arrival_socket)
  if not arrival and exit.arrival_side then arrival=self.rooms.arrival(dest_node,exit.arrival_side) end
  if not arrival then arrival=copy(exit.arrival) end
  if not arrival or not finite(arrival.x) or not finite(arrival.y) then return nil,'unresolved destination arrival' end
  info.arrival=arrival
 end
 return info
end

-- Physical activation test for the current fighter. A bottom drop needs real
-- downward passage below the authored opening, never proximity to the standing
-- anchor and never motion carrying a fighter up from below. Descent comes from
-- the native player.vy when present, else from a previous position sample.
function RuntimeRooms:triggered(info,player,floor,previous)
 if type(info)~='table' or type(player)~='table' or not finite(player.x) or not finite(player.y) then return false end
 if info.drop then
  local fy=floor and finite(floor.y) and floor.y or 0
  if player.y>fy-self.drop_margin then return false end
  local opening=info.opening
  if not (opening and math.abs(player.x-opening.x)<=opening.width/2) then return false end
  local descent
  if finite(player.vy) then descent=player.vy
  elseif type(previous)=='table' and finite(previous.y) then descent=player.y-previous.y
  else return false end
  return descent<=-self.drop_speed
 end
 local t=info.trigger
 return math.abs(player.x-t.x)<=self.door_reach and math.abs(player.y-t.y)<=self.vertical_reach
end

local function new_room(self,node)
 local plan,why=self.rooms.plan(node)
 if not plan then return nil,why end
 local collision,why2=self.rooms.collision(node)
 if not collision then return nil,why2 end
 local count=#collision.floor_segments+#collision.platforms+#collision.lines
 if count>self.max_lines then return nil,'room collision line budget' end
 return {node=node,plan=plan,collision=collision,state=self.rooms.new(),handles={},line_count=count}
end

-- Remove every owned collision handle. A refused removal stays owned in the room
-- record so a later retry can finish it; nothing is silently dropped.
function RuntimeRooms:_remove_handles(room)
 local remaining,why={},nil
 for _,entry in ipairs(room.handles) do
  local ok,result=pcall(self.engine.stage_remove,entry.handle)
  if ok and result==true then self.total_lines=self.total_lines-1
  else remaining[#remaining+1]=entry;why=why or 'collision handle removal refused' end
 end
 room.handles=remaining
 return remaining,why
end

-- Build one destination: floor segments (drop gaps stay open), real ascent
-- slopes through stage_add_line, then platforms, then visuals. Recipe geometry
-- is already in game units, translated from the authored BF kit exactly once by
-- RoomRecipes; this module neither rescales nor re-offsets it.
function RuntimeRooms:_construct(room)
 local engine=self.engine
 local function platform(x,y,width,opts)
  local ok,h,why=pcall(engine.stage_add_platform,x,y,width,opts)
  if not ok or not handle(h) then return nil,tostring(not ok and h or why or 'platform allocation refused') end
  room.handles[#room.handles+1]={kind='platform',handle=h,x0=x-width/2,y0=y,x1=x+width/2,y1=y};self.total_lines=self.total_lines+1
  return h
 end
 local function line(x0,y0,x1,y1,opts)
  local ok,h,why=pcall(engine.stage_add_line,x0,y0,x1,y1,'floor',opts)
  if not ok or not handle(h) then return nil,tostring(not ok and h or why or 'line allocation refused') end
  room.handles[#room.handles+1]={kind='line',handle=h,x0=x0,y0=y0,x1=x1,y1=y1};self.total_lines=self.total_lines+1
  return h
 end
 local _,why
 for _,s in ipairs(room.collision.floor_segments) do
  _,why=platform((s.left+s.right)/2,s.y,s.right-s.left,{passthrough=false,ledges=true,draw=false})
  if why then return false,why end
 end
 for _,l in ipairs(room.collision.lines) do
  _,why=line(l.x0,l.y0,l.x1,l.y1,{passthrough=l.passthrough,ledges=l.ledges,draw=false})
  if why then return false,why end
 end
 for _,p in ipairs(room.collision.platforms) do
  _,why=platform(p.x,p.y,p.width,{passthrough=p.passthrough,ledges=p.ledges,draw=false})
  if why then return false,why end
 end
 local linked,link_why=RuntimeRooms.link_floor_seams(engine,room.handles)
 if not linked then return false,link_why end
 local ok,enter_why=self.rooms.enter(room.state,room.node)
 if not ok then return false,enter_why end
 return true
end

function RuntimeRooms:_fail(tx,why)
 local cleaned,clean_why=self:rollback(tx)
 if not cleaned then return nil,tostring(why)..'; cleanup refused: '..tostring(clean_why) end
 return nil,tostring(why)
end

-- Return every fighter moved by :place to its captured position. A refused
-- restore stays owned in tx.placement.pending so a later retry can finish it;
-- positions are only forgotten once the engine accepts them.
function RuntimeRooms:_restore_placement(tx)
 local pl=tx and tx.placement
 if not pl then return true end
 local keep,why={},nil
 local function attempt(entry)
  local ok,result=pcall(self.engine.teleport,entry.port,entry.x,entry.y)
  if ok and result~=false then return end
  keep[#keep+1]=entry;why=why or tostring(not ok and result or 'restore refused')
 end
 for _,entry in ipairs(pl.pending) do attempt(entry) end
 for _,entry in ipairs(pl.moved) do attempt(entry) end
 pl.moved={};pl.pending=keep
 if #keep>0 then return false,why end
 return true
end

-- Open a transaction for dest_node. args.from must be the module's own active
-- room token (or nil only before the first room) and args.exit one of that
-- room's canonical adapter exits whose target is dest_node; a foreign token, an
-- inactive room or a forged exit is refused. Nothing is allocated until step;
-- a refused plan, collision, budget, exit or isolation leaves the source intact.
function RuntimeRooms:begin(dest_node,args)
 args=args or {}
 if self.busy then return nil,'transition already open' end
 if self.pending_source then
  local flushed,why=self:flush()
  if not flushed then return nil,'prior room cleanup outstanding: '..tostring(why) end
 end
 if not self:isolated() then return nil,'empty-stage isolation required' end
 if type(dest_node)~='table' or type(dest_node.id)~='string' then return nil,'invalid destination node' end
 local source,exit=args.from,args.exit
 if self.active then
  if source~=self.active then return nil,'source token is not the active room' end
 elseif source~=nil then return nil,'source token is foreign or inactive' end
 local info=nil
 if source then
  if type(exit)~='table' then return nil,'missing exit for transition' end
  local canonical=self:_canonical_exit(source.node,exit)
  if not canonical then return nil,'exit does not belong to the source room' end
  if canonical.to~=dest_node.id then return nil,'destination does not match the exit target' end
  local why
  info,why=self:resolve_exit(source.node,dest_node,canonical)
  if not info then return nil,why end
 end
 local dest,why=new_room(self,dest_node)
 if not dest then return nil,why end
 if self.total_lines+dest.line_count>self.max_total_lines then return nil,'total collision line budget' end
 self.serial=self.serial+1
 local tx={id=self.serial,phase='load',source=source,exit=exit,info=info,dest=dest,done=false,placed=false}
 self.busy=tx
 return tx
end

-- One unit of work. Returns 'loading' (one asset loaded, call again), 'ready'
-- (destination constructed; place fighters, stage/validate, save, then commit),
-- or nil + reason (destination rolled back, source intact).
function RuntimeRooms:step(tx)
 if type(tx)~='table' or tx.done or self.busy~=tx then return nil,'stale or closed transition' end
 if not self:isolated() then return self:_fail(tx,'empty-stage isolation lost') end
 if tx.phase=='load' then
  local ready,why=self.rooms.preload_step(tx.dest.state,tx.dest.node)
  if ready==nil then return self:_fail(tx,why) end
  if ready==false then return 'loading' end
  tx.phase='build'
 end
 if tx.phase=='build' then
  local ok,why=self:_construct(tx.dest)
  if not ok then return self:_fail(tx,why) end
  tx.phase='ready'
 end
 return 'ready'
end

-- Transactional fighter placement. Each fighter's current position is verified
-- and captured through gd.player before any fighter moves. A throw or an
-- explicit false from the injected teleport is a failure, and every already
-- moved fighter is restored; if a restore is refused, ownership is kept in
-- tx.placement for a later rollback retry. placements is a list of {port=,x=,y=}.
function RuntimeRooms:place(tx,placements)
 if type(tx)~='table' or tx.done or self.busy~=tx then return nil,'stale or closed transition' end
 if tx.phase~='ready' then return nil,'destination not constructed' end
 if tx.placed then return nil,'fighters already placed' end
 if type(self.engine.player)~='function' then return nil,'fighter position API unavailable' end
 if type(self.engine.teleport)~='function' then return nil,'placement API unavailable' end
 if type(placements)~='table' or #placements==0 then return nil,'no placements' end
 local captured={}
 for _,p in ipairs(placements) do
  local port=p.port or p[1]
  if not handle(port) or not finite(p.x) or not finite(p.y) then return nil,'invalid placement' end
  local ok,state=pcall(self.engine.player,port)
  if not ok or type(state)~='table' or not finite(state.x) or not finite(state.y) then
   return nil,'fighter position unavailable for port '..tostring(port) end
  captured[#captured+1]={port=port,x=state.x,y=state.y}
 end
 tx.placement={moved={},pending={}}
 for i,p in ipairs(placements) do
  local port=p.port or p[1]
  local ok,result=pcall(self.engine.teleport,port,p.x,p.y)
  if not ok or result==false then
   local why=tostring(not ok and result or 'placement refused')
   local restored,rwhy=self:_restore_placement(tx)
   if not restored then return nil,why..'; restore refused: '..tostring(rwhy) end
   return nil,why
  end
  tx.placement.moved[#tx.placement.moved+1]=captured[i]
 end
 tx.placed=true
 return true
end

-- Destination arrival for spawning: the exit's destination socket arrival on a
-- transition, else the room's authored spawn. Distinguishes trigger (source)
-- from arrival (destination).
function RuntimeRooms:arrival(tx)
 if type(tx)~='table' or tx.done then return nil end
 return copy(tx.info and tx.info.arrival or self.rooms.arrival(tx.dest.node))
end

-- Finish a partially removed source on a later frame. Keeps ownership until the
-- engine actually accepts each removal.
function RuntimeRooms:flush()
 local pending=self.pending_source
 if not pending then return true end
 local remaining,why=self:_remove_handles(pending.room)
 local cleared=self.rooms.clear(pending.room.state)
 local released=self.rooms.release(pending.room.state)
 if #remaining>0 or cleared==false or released==false then
  self.pending_source={room=pending.room,handles=remaining,why=why or 'source cleanup refused'}
  return false,self.pending_source.why
 end
 self.pending_source=nil
 return true
end

-- Destructively retire the source and activate the destination. Call this only
-- after placements are correct and the logical route has been persisted: commit
-- cannot restore the source. If the engine refuses a source removal, the
-- destination is still active but the retained handles are reported through
-- released=false and :pending(), so the caller keeps gameplay paused and retries
-- :flush() until the colliders actually retire.
function RuntimeRooms:commit(tx)
 if type(tx)~='table' or tx.done or self.busy~=tx then return nil,'stale or closed transition' end
 if tx.phase~='ready' then return nil,'destination not constructed' end
 if tx.placement and #tx.placement.pending>0 then return nil,'fighters were not restored' end
 local released,reason=true,nil
 if tx.source then
  local pending,why=self:_remove_handles(tx.source)
  local cleared=self.rooms.clear(tx.source.state)
  local r=self.rooms.release(tx.source.state)
  if #pending>0 or cleared==false or r==false then
   released=false;reason=why or 'source cleanup refused'
   self.pending_source={room=tx.source,handles=pending,why=reason}
  end
 end
 local outcome={room_id=tx.dest.node.id,room=self:view(tx.dest),
  arrival=copy(tx.info and tx.info.arrival or self.rooms.arrival(tx.dest.node)),
  spawn=copy(self.rooms.arrival(tx.dest.node)),released=released,room_ref=tx.dest}
 self.active=tx.dest
 tx.done=true
 self.busy=nil
 return outcome,reason
end

-- Abandon a transaction: restore every moved fighter, then destroy only the
-- destination. The source is never touched. Restore or removal refusals stay
-- owned and the transaction stays open so the caller can retry after the engine
-- recovers; nothing is freed twice.
function RuntimeRooms:rollback(tx)
 if type(tx)~='table' or self.busy~=tx then return false,'stale transition' end
 local restored,rwhy=self:_restore_placement(tx)
 if not restored then return false,'placement restore refused: '..tostring(rwhy) end
 if tx.phase=='ready' then tx.phase='rollback' end
 local pending,why=self:_remove_handles(tx.dest)
 local cleared=self.rooms.clear(tx.dest.state)
 local released=self.rooms.release(tx.dest.state)
 local clean=#pending==0 and cleared~=false and released~=false
 if clean then tx.done=true;tx.phase='closed';self.busy=nil end
 return clean,clean and nil or (why or 'destination cleanup refused')
end

-- Release everything the module owns: an in-flight destination (restoring its
-- placements), a pending source, then the active room. Refused collision or
-- model handles stay owned for a later cleanup/flush retry; no handle is freed
-- twice. Symmetric with begin/commit and safe to call repeatedly.
function RuntimeRooms:cleanup()
 local failures={}
 if self.busy and not self.busy.done then
  local cleaned,why=self:rollback(self.busy)
  if not cleaned then failures[#failures+1]=why end
 end
 if self.pending_source then
  local flushed,why=self:flush()
  if not flushed then failures[#failures+1]=why end
 end
 if self.active then
  local remaining,why=self:_remove_handles(self.active)
  local cleared=self.rooms.clear(self.active.state)
  local released=self.rooms.release(self.active.state)
  if #remaining>0 or cleared==false or released==false then
   failures[#failures+1]=why or 'active room cleanup refused'
  else self.active=nil end
 end
 if #failures>0 then return false,table.concat(failures,'; ') end
 return true
end
function RuntimeRooms:release() return self:cleanup() end

-- Drop all bookkeeping after real native scene teardown. Makes no native calls;
-- while the module still believes it owns live handles it refuses unless the
-- caller explicitly confirms the teardown, so live handles are never lost
-- silently.
function RuntimeRooms:reset(confirmed)
 local live=self.total_lines>0 or self.active~=nil or self.busy~=nil or self.pending_source~=nil
 if live and confirmed~=true then return false,'reset requires confirmed native scene teardown' end
 self.active=nil;self.busy=nil;self.pending_source=nil;self.total_lines=0;self.serial=0
 return true
end

function RuntimeRooms:view(room)
 if not room then return nil end
 local v=self.rooms.view(room.state)
 return {room_id=room.node.id,theme=v.theme,instances=v.instances,assets=v.assets,
  line_count=#room.handles,lines=#room.handles,error=v.error}
end

function RuntimeRooms:active_room() return self.active end
function RuntimeRooms:view_active() return self:view(self.active) end
function RuntimeRooms:pending() return self.pending_source and {handles=copy(self.pending_source.handles),why=self.pending_source.why} or nil end
function RuntimeRooms:total() return self.total_lines end

return RuntimeRooms
