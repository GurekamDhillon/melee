-- Script-owned world pickups, not replacement native items. Only
-- native or mission defeat signals create them; both share one dedupe ledger.
return function(D)
  local C,G=D.companion,D.genetics
  local V={};V.__index=V
  V.kinds={goomba='red',koopa='blue',redead='blue',like_like='yellow',
           octorok='yellow',polar_bear='red',topi='green'}
  function V.new(rng,log,g)
    local engine,settings=g or {},{}
    for key,value in pairs(C.tuning.juice) do settings[key]=value end
    -- Partial FX APIs must not create handles that cannot move or be retired.
    for _,name in ipairs({'fx_world','fx_move','fx_control','fx_end'}) do
      if type(engine[name])~='function' then
        for _,key in ipairs({'glow','pool','sparkles','highlight','pop_trail','collect_burst'}) do settings[key]=false end
        break
      end
    end
    return setmetatable({rng=rng or math.random,log=log or function() end,tracked={},seen={},pickups={},native={},g=engine,juice=D.pickup_juice.new(engine,settings)},V)
  end
  function V:set_depth(depth) self.depth=math.max(1,math.min(4,depth or 1)) end
  function V:track(handle,at)
    if self.seen[handle] or not at or not V.kinds[at.kind] then return end
    if type(at.x)=='number' and type(at.y)=='number' then
      self.tracked[handle]={kind=at.kind,x=at.x,y=at.y}
    end
  end
  function V:defeat(event,c)
    local at=self.tracked[event.handle]
    if not at or at.kind~=event.kind or self.seen[event.handle] then return false end
    self.seen[event.handle]=true;self.tracked[event.handle]=nil
    local depth=self.depth or 1
    local chance=C.tuning.drop_base;local r=G.random(self.rng)
    if r>=chance then return true end
    local colour
    if r<C.tuning.white_chance then colour='white'
    elseif G.random(self.rng)<C.tuning.primary_chance then colour=V.kinds[at.kind]
    else
      local alternatives={};for _,k in ipairs({'red','green','blue','yellow'}) do
        if k~=V.kinds[at.kind] then alternatives[#alternatives+1]=k end
      end
      colour=alternatives[math.floor(G.random(self.rng)*#alternatives)+1]
    end
    local amount=colour=='white' and 1 or C.tuning.depth_drive_points[depth]
    if type(self.g.item_spawn)=='function' then
      local ok,h,why=pcall(self.g.item_spawn,'drive',at.x,at.y,{payload={colour=colour,amount=amount}})
      if ok and type(h)=='number' and h>0 then
        self.native[h]={colour=colour,amount=amount}
        self.juice:drop(h,colour,at.x,at.y,900)
      else
        -- An ambiguous native failure must never create a second script drop.
        self.log('envoy: native drive spawn refused '..tostring(ok and why or h))
      end
    else
      local pickup={colour=colour,amount=amount,x=at.x,y=at.y,left=C.tuning.drive_lifetime}
      self.pickups[#self.pickups+1]=pickup;self.juice:drop(pickup,colour,at.x,at.y,C.tuning.drive_lifetime)
    end
    self.log(('envoy: drop %s handle=%d x=%.2f y=%.2f position=last-observed'):format(colour,event.handle,at.x,at.y))
    return true
  end
  function V:tick(p,c)
    self.juice:tick()
    local radius=C.tuning.pickup_base
    local collected={}
    for i=#self.pickups,1,-1 do
      local v=self.pickups[i]
      if p and (p.x-v.x)^2+(p.y-v.y)^2<=radius^2 then
        self.juice:collect(v,v.colour,p)
        local gain=C.feed(c,v.colour,v.amount or (v.colour=='white' and 1 or C.tuning.drive_points))
        local k=C.colours[v.colour]
        collected[#collected+1]={pickup=v,colour=v.colour,gain=gain,grade=k and c.stats[k].grade or 'S'}
        self.log(('envoy: pickup %s gain=%d'):format(v.colour,gain))
        table.remove(self.pickups,i)
      else
        v.left=v.left-1
        if v.left<=0 then self.juice:expire(v);self.log('envoy: expired '..v.colour);table.remove(self.pickups,i) end
      end
    end
    return collected
  end
  function V:collect(event,c)
    if event.name~='drive' or event.port~=1 or not c then return nil end
    local v=self.native[event.item]
    if not v or v.retired then return nil end
    local p=event.payload
    if type(p)~='table' or p.colour~=v.colour or p.amount~=v.amount then return nil end
    -- Retire before awarding: duplicate engine events cannot credit twice.
    self.native[event.item]=nil
    self.juice:collect(event.item,v.colour,type(self.g.player)=='function' and self.g.player(1) or nil)
    local gain=C.feed(c,v.colour,v.amount);local k=C.colours[v.colour]
    self.log(('envoy: pickup %s gain=%d native'):format(v.colour,gain))
    return {colour=v.colour,gain=gain,grade=k and c.stats[k].grade or 'S'}
  end
  function V:expire(event)
    if event.name=='drive' then self.juice:expire(event.item);self.native[event.item]=nil end
  end
  function V:clear()
    self.juice:clear()
    self.tracked={};self.seen={};self.pickups={}
    for h,v in pairs(self.native) do
      v.retired=true -- Failed cleanup retries must never reward a later run.
      local ok,result=pcall(self.g.item_despawn,h)
      if ok and result~=false and result~=nil then self.native[h]=nil end
    end
  end
  return V
end
