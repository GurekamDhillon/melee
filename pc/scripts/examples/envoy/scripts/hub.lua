-- Walk-up stations on a normal mission-folder level, with owned kit placeholder.
return function(D)
 local H={};H.__index=H
 H.stations={{x=-180,name='Run exit',screen='setup'},{x=-90,name='Fighter',screen='fighter'},
 {x=0,name='Companion',screen='companion'},{x=90,name='Records',screen='records'},
 {x=180,name='Nest - slice 5',closed=true}}
 local function call(fn,...) if type(fn)~='function' then return nil,'engine API absent' end;local ok,r,e=pcall(fn,...);if ok then return r,e end;return nil,r end
 function H.new(g,mission) return setmetatable({g=g,mission=mission,active=false},H) end
 function H:resolve()
  if not self.asset then self.asset=call(self.g.model_load,'models/bf_floor_4m.gxmesh') end
  return self.asset~=nil and self.asset~=false
 end
 function H:enter()
  if self.active then return true end
  if self.mission.staging or self.mission.stopping or self.mission.recovery then return false,'garden waiting for cleanup' end
  local ok,why=self.mission:command('play hub')
  if not ok then return false,why end
  self.active=true;self.waiting=true;self.previous=true;return true
 end
 function H:frame(c)
  if not self.active then return end
  local m=self.mission
  if m.staging or m.pending then return end
  if not m.current or m.current.doc.name~='hub' then self:clear();return end
  self.waiting=nil
  if not self.asset and not self.refused then
   self.asset=call(self.g.model_load,'models/bf_floor_4m.gxmesh')
   if not self.asset then self.refused=true;self.g.log('envoy: garden placeholder unavailable; prepare the room kit') end
  end
  if self.asset and not self.object and not self.refused then
   self.object=call(self.g.model_spawn,self.asset,{x=0,y=10,z=0,scale_x=.22,scale_y=.7,scale_z=.35,collision=false})
   if not self.object then self.refused=true;self.g.log('envoy: garden placeholder spawn refused') end
  end
  if self.object and c then
   local rgb=D.recolour and D.recolour.palette(c).primary or ({red={245,142,138},normal={222,232,245}})[c.colour] or {222,232,245}
   local tint=rgb[1]*16777216+rgb[2]*65536+rgb[3]*256+255
   local key=tostring(tint)..c.type..c.id
   if self.key~=key then
    call(self.g.model_set,self.object,{tint=tint});call(self.g.model_label,self.object,'Envoy '..c.id..' / '..c.type)
    self.key=key
   end
  end
 end
 function H:activate(down)
  local pressed=down and not self.previous;self.previous=down
  if not pressed or not self.active or self.waiting then return end
  local p=self.g.player(1);if not p then return end
  for _,s in ipairs(H.stations) do
   if math.abs(p.x-s.x)<=D.companion.tuning.hub_station_radius and math.abs(p.y-10)<=35 then return s end
  end
 end
 function H:draw(c,notice)
  if not self.active or self.waiting or type(self.g.project)~='function' then return end
  for _,s in ipairs(H.stations) do
   local x,y,v=self.g.project(s.x,28,0)
   if x and v then self.g.text(x-28,y,s.name,0xEBD175FF,10) end
  end
  local x,y,v=self.g.project(0,55,0)
  if x and v then self.g.text(x-65,y,('Envoy %d / %s%s'):format(c.id,c.type,notice and ' / '..notice or ''),0xFFFFFFFF,10) end
  local p=self.g.player(1)
  for _,s in ipairs(H.stations) do if p and math.abs(p.x-s.x)<=D.companion.tuning.hub_station_radius then
   local a=self.g.safe_area();self.g.text(a.x+20,a.y+a.h-30,'A: '..s.name..'    START: garden menu',0xFFFFFFFF,12);break
  end end
 end
 function H:clear()
  self.active=false;self.waiting=nil;self.previous=nil
  if self.object then local ok=call(self.g.model_despawn,self.object);if ok~=nil and ok~=false then self.object=nil end end
  if self.asset and not self.object then local ok,why=call(self.g.model_release,self.asset);if ok~=false and why==nil then self.asset=nil end end
  self.key=nil;self.refused=nil
 end
 return H
end
