-- Optional model layer for Envoy drives. Drawing only: drives.lua still owns drop, pickup, expiry and
-- feeding. When the original asset mod `envoy_drives` is mounted, each pickup is shown as a bobbing, turning
-- model; otherwise `active()` stays false and the existing HUD glyph is drawn. Never a gameplay write.
return function(D)
  local V={};V.__index=V
  V.MOD='envoy_drives'
  V.MESH={red='drive_red',green='drive_green',yellow='drive_yellow',blue='drive_blue',white='drive_white',purple='drive_purple'}
  V.TINT={red=0xFFFFFFFF,green=0xFFFFFFFF,yellow=0xFFFFFFFF,blue=0xFFFFFFFF,white=0xFFFFFFFF,purple=0xFFFFFFFF}
  V.GLASS={red=0xFF9C9CFF,green=0x9CFFB4FF,yellow=0xFFEE9CFF,blue=0x9CBCFFFF,white=0xFFFFFFFF}
  V.SCALE=1.45   -- instance scale on top of the exported 4.4-unit height (about 40 px at 1.15; larger so a drop is found)
  V.LIFT=2.6      -- model half height (units) so the drive rests on the point the enemy left
  V.PULSE=14      -- frames of the collect pulse
  local function try(f,...) local ok,a,b=pcall(f,...);if ok then return a,b end;return nil,a end
  function V.new(g,log)
    return setmetatable({g=g,log=log or g.log or function() end,live={},dying={},pending={},releases={},frame=0,assets=nil,retry=0},V)
  end
  function V:load()
    if self.assets then return true end
    if self.frame<self.retry then return false end
    self.retry=self.frame+120
    for _,api in ipairs({'model_load','model_spawn','model_set','model_despawn','model_release'}) do
      if type(self.g[api])~='function' then return false end
    end
    local a={};self.warned=self.warned or {}
    for _,name in ipairs({'drive_red','drive_green','drive_yellow','drive_blue','drive_white','drive_glass'}) do
      local path=V.MOD..'/models/'..name
      local h,err=try(self.g.model_load,path)
      if h then a[name]=h
      elseif not self.warned[name] then
        try(self.log,'envoy: drive model unavailable: '..path..'.gxmesh ('..tostring(err)..'); using HUD markers for affected colours')
        self.warned[name]=true
      end
    end
    if not a.drive_glass or not (a.drive_red or a.drive_green or a.drive_yellow or a.drive_blue or a.drive_white) then
      for _,h in pairs(a) do self:release(h) end
      return false
    end
    self.assets=a;try(self.log,'envoy: drive models loaded');return true
  end
  function V:active() return self.assets~=nil and self.complete==true end
  -- HUD integration can retain glyphs only for refused instances.
  function V:visible(p) return self.live[p]~=nil end
  local function place(g,e,p,frame,extra)
    local t=(frame+e.phase)/60
    local spin=0.72+0.28*math.cos(t*2*math.pi*0.45)  -- a slow width wobble (rot is Z-only): never edge-on, the flat shapes stay readable
    local s=((extra and extra.scale) or 1)*V.SCALE
    local y=p.y+V.LIFT*s+0.7*math.sin(t*2*math.pi*0.9)+((extra and extra.lift) or 0)
    local o={x=p.x,y=y,z=0,scale=s,scale_x=spin,rot=6*math.sin(t*2*math.pi*0.45),visible=true}
    return o
  end
  function V:sync(pickups)
    self.frame=self.frame+1
    self:retry_cleanup()
    if not self:load() then self.complete=false;return false end
    self.complete=true
    local seen={}
    for _,p in ipairs(pickups) do
      seen[p]=true
      local e=self.live[p]
      if not e then
        local mesh=self.assets[V.MESH[p.colour] or 'drive_white']
        local o={x=p.x,y=p.y+V.LIFT*V.SCALE,z=0,scale=V.SCALE,tint=V.TINT[p.colour] or 0xFFFFFFFF,collision=false,scale_x=0.06}
        local solid=mesh and try(self.g.model_spawn,mesh,o)
        o.tint=V.GLASS[p.colour] or 0xFFFFFFFF
        local glass=solid and try(self.g.model_spawn,self.assets.drive_glass,o)
        if solid and glass then
          e={solid=solid,glass=glass,phase=(#pickups*7+self.frame)%60,left=p.left};self.live[p]=e
        else
          if solid then self:drop({solid=solid}) end
          self.complete=false
        end
      end
      if e then
        e.left=p.left
        local o=place(self.g,e,p,self.frame)
        if p.left<90 then o.visible=(self.frame%8)<5 end   -- blink for the last 1.5 s
        local solid=try(self.g.model_set,e.solid,o)
        local glass=try(self.g.model_set,e.glass,o)
        if not solid or not glass then
          self:drop(e);self.live[p]=nil;self.complete=false
        end
      end
    end
    for p,e in pairs(self.live) do
      if not seen[p] then
        self.live[p]=nil
        if e.left>1 then e.pos={x=p.x,y=p.y};e.age=0;self.dying[#self.dying+1]=e   -- collected: pulse out
        else self:drop(e) end                                                         -- expired: just go
      end
    end
    for i=#self.dying,1,-1 do
      local e=self.dying[i];e.age=e.age+1
      if e.age>=V.PULSE then self:drop(e);table.remove(self.dying,i)
      else
        local k=e.age/V.PULSE
        local o=place(self.g,e,e.pos,self.frame,{scale=1+0.9*k,lift=2.5*k})
        o.scale_x=math.max(0.06,1-k);o.tint=nil
        local solid=try(self.g.model_set,e.solid,o)
        local glass=try(self.g.model_set,e.glass,o)
        if not solid or not glass then self:drop(e);table.remove(self.dying,i) end
      end
    end
    return self.complete
  end
  V.tick=V.sync
  function V:collected(p)
    local e=self.live[p];if not e then return false end
    self.live[p]=nil;e.pos={x=p.x,y=p.y};e.age=0;self.dying[#self.dying+1]=e;return true
  end
  function V:drop(e)
    for _,key in ipairs({'solid','glass'}) do
      local h=e[key]
      if h then
        local ok=try(self.g.model_despawn,h)
        -- Scene resets invalidate instances. Never keep retrying an absent handle.
        local gone=false
        if not ok and type(self.g.model_get)=='function' then
          local queried,instance=pcall(self.g.model_get,h);gone=queried and instance==nil
        end
        if ok or gone then e[key]=nil end
      end
    end
    if (e.solid or e.glass) and not e.queued then e.queued=true;self.pending[#self.pending+1]=e end
  end
  function V:retry_cleanup()
    local pending=self.pending;self.pending={}
    for _,e in ipairs(pending) do e.queued=nil;self:drop(e) end
    local releases=self.releases;self.releases={}
    for _,h in ipairs(releases) do self:release(h) end
  end
  function V:release(h)
    local ok,err=pcall(self.g.model_release,h)
    -- After a scene reset the owned generation has already disappeared.
    if not ok and not tostring(err):find('stale or released model handle',1,true) then
      self.releases[#self.releases+1]=h
    end
  end
  function V:clear()
    self:retry_cleanup()
    for p,e in pairs(self.live) do self:drop(e) end
    for _,e in ipairs(self.dying) do self:drop(e) end
    self.live={};self.dying={};self.complete=false
  end
  function V:unload() self:clear();if self.assets then for _,v in pairs(self.assets) do self:release(v) end end;self.assets=nil;self.retry=0 end
  return V
end
