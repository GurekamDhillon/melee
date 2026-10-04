-- Rectangles use half-open containment: the shared border belongs to the next chunk.
return function(D)
  local C={}
  -- Predict one neighbouring window before a doorway, from committed membership.
  function C.prefetch(s,p,c)
    if not c or not p then return nil end
    local old=s.last_point;s.last_point={x=p.x,y=p.y}
    if not old then return nil end
    local dx,dy=p.x-old.x,p.y-old.y;local b=c.rect
    local w,h=b.right-b.left,b.top-b.bottom
    local x,y=p.x,p.y
    if math.abs(dx)>=math.abs(dy) and dx~=0 then
      if dx>0 and b.right-p.x<=w*.5 then x=b.right+w*.5
      elseif dx<0 and p.x-b.left<=w*.5 then x=b.left-w*.5 else return nil end
    elseif dy~=0 then
      if dy>0 and b.top-p.y<=h*.5 then y=b.top+h*.5
      elseif dy<0 and p.y-b.bottom<=h*.5 then y=b.bottom-h*.5 else return nil end
    else return nil end
    return C.containing(s.doc.chunks,{x=x,y=y})
  end
  function C.containing(chunks,p)
    if not p then return nil end
    for _,c in ipairs(chunks) do
      local b=c.rect
      if p.x>=b.left and p.x<b.right and p.y>=b.bottom and p.y<b.top then return c end
    end
  end
  function C.new(g,doc,prefix) return {g=g,doc=doc,prefix=prefix,loaded={},current=nil} end
  function C.respawn(s,force)
    local c=s.current
    if c and (force or s.spawn~=c) then
      assert(s.g.stage_set_spawn(D.mission.RESPAWN_SLOT,c.spawn.x,c.spawn.y))
      s.spawn=c
    end
  end
  function C.preload(s,target,player)
    local previous=s.destination;s.destination=C.containing(s.doc.chunks,target)
    local ok,why=C.update(s,player,s.membership)
    if not ok then s.destination=previous end
    return ok,why
  end
  function C.plan(doc,target,selected,zoned)
    local centre;if zoned then centre=selected else centre=C.containing(doc.chunks,target)end;local out={}
    if not centre then return out,nil end
    local b=centre.rect;local w,h=b.right-b.left,b.top-b.bottom
    for _,c in ipairs(doc.chunks) do
      local r=c.rect
      if r.left<b.right+w and r.right>b.left-w and r.bottom<b.top+h and r.top>b.bottom-h then out[#out+1]=c end
    end
    local function distance(c)
      local r=c.rect;return ((r.left+r.right)/2-target.x)^2+((r.bottom+r.top)/2-target.y)^2
    end
    table.sort(out,function(a,b)local x,y=distance(a),distance(b);if x==y then return a.id<b.id end;return x<y end)
    return out,centre
  end
  function C.update(s,p,...)
    local membership=select(1,...)
    local c=membership and membership.committed or s.current
    if not membership then
      local holder={doc=s.doc,stream=s,zone_state=s.zone_state,membership=s.membership}
      membership=D.zones.sample(s.g,holder,p,(s.zone_frame or 0)+1)
      s.zone_state=holder.zone_state;s.zone_frame=membership.frame;c=membership.committed
    end
    if not c and not s.destination then return true end
    local arrived=c and s.destination==c
    local wanted={}
    local function window(center)
      if not center then return end
      local b=center.rect;local w,h=b.right-b.left,b.top-b.bottom
      for _,v in ipairs(s.doc.chunks) do
      local r=v.rect
      if r.left<b.right+w and r.right>b.left-w and r.bottom<b.top+h and r.top>b.bottom-h then wanted[v.id]=v end
      end
    end
    window(c);if not arrived then window(s.destination) end
    local ahead=C.prefetch(s,p,c);if not s.destination then window(ahead)end
    -- Build one missing area per observation tick. Keep the old window and spawn
    -- while the requested window fills. Commit only a room with installed collision.
    if c and s.current~=c then
      local ok,why=pcall(function()assert(s.g.stage_set_spawn(D.mission.RESPAWN_SLOT,c.spawn.x,c.spawn.y))end)
      if not ok then D.zones.reject(s,membership);return nil,why end
    end
    local missing,best
    for _,v in pairs(wanted)do if not s.loaded[v.id]then
      local b=v.rect;local d=((b.left+b.right)/2-p.x)^2+((b.bottom+b.top)/2-p.y)^2
      if not best or d<best or (d==best and v.id<missing.id)then missing=v;best=d end
    end end
    if s.destination and not s.loaded[s.destination.id]then missing=s.destination end
    if c and not s.loaded[c.id]then missing=c end
    if missing then
      local frame=membership.frame
      if frame and (s.load_frame==frame or s.wave_frame==frame) then s.pending=true;D.zones.reject(s,membership);return true end
      s.load_frame=frame
      local t=s.g.time()
      local ok,area=pcall(D.world.load,s.g,s.prefix..missing.serial,missing.level)
      if not ok then
        if s.current then s.g.stage_set_spawn(D.mission.RESPAWN_SLOT,s.current.spawn.x,s.current.spawn.y)end
        D.zones.reject(s,membership);return nil,area
      end
      s.loaded[missing.id]=area
      s.g.log(('mission: chunk load %s seconds=%.6f'):format(missing.id,s.g.time()-t))

    end
    if c and not s.loaded[c.id]then
      s.pending=true
      if s.current then s.g.stage_set_spawn(D.mission.RESPAWN_SLOT,s.current.spawn.x,s.current.spawn.y)end
      D.zones.reject(s,membership);return true
    end
    local pending=false
    for id in pairs(wanted)do if not s.loaded[id]then pending=true end end
    s.pending=pending
    if not pending and not missing and s.load_frame~=membership.frame and s.wave_frame~=membership.frame then
      local id;for key in pairs(s.loaded)do if not wanted[key]and(not id or key<id)then id=key end end
      if id then s.load_frame=membership.frame;D.world.unload(s.g,s.loaded[id]);s.loaded[id]=nil;s.g.log('mission: chunk unload '..id)end
    end
    if c and s.current~=c then
      s.current=c;s.spawn=c
      s.g.log(('mission: chunk spawn %s %.1f %.1f player=%.1f,%.1f'):format(c.id,c.spawn.x,c.spawn.y,p.x,p.y))
    end
    if arrived then s.destination=nil end
    return true
  end
  function C.stop(s)
    for id,a in pairs(s.loaded) do D.world.unload(s.g,a);s.loaded[id]=nil end
  end
  return C
end
