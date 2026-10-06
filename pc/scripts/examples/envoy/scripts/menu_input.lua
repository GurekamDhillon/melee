-- Reads the physical pre-mask pad. The engine can hide the D-pad and START from the game (gd.input_mask) and a
-- chord (gd.input_chord). While a menu is open START is hidden too, so the press that closes it cannot pause the match.
-- The app pauses gameplay for other menu buttons and owns pause/resume lifecycle.
return function(D)
 local I={};local S={};S.__index=S
 local START=0x1000
 -- Online (a netplay or rollback session) the engine refuses input masks and chords: they would change what is SENT. The Envoy lobby menus are local UI only, so
 -- they simply do not mask there (a masked D-pad is a convenience, never a rule).
 local function offline(g) local m=g.match and g.match();return not (m and m.netplay) end
 local shared={held_start=false} -- START still held after a menu closed: keep hiding it until it is let go
 local extra={} -- the same latch for another port (a co-op seat); port 1 keeps `shared`, so a one-player run is unchanged
 local function latch(port) if (port or 1)==1 then return shared end;local l=extra[port];if not l then l={held_start=false};extra[port]=l end;return l end
 -- Called every tick by the chord host: lets the hidden START go once the button is released.
 function I.settle(g,port)
  port=port or 1;local l=latch(port)
  if not l.held_start then return end
  local p=g.pad(port,true) or {}
  if not p.START then l.held_start=false;if g.input_mask and offline(g) then g.input_mask(port,0) end end
 end
 -- A scene change releases every mask in the engine: forget the latch with it, whichever host would have serviced it.
 function I.reset() shared.held_start=false;for _,l in pairs(extra) do l.held_start=false end end
 function I.new(g,port) return setmetatable({g=g,port=port or 1,active=false,previous={},held=0},S) end
 -- `hide_start`: this menu was opened by the Z+START chord (the bag), so its START must not reach the game. Every other menu (the
 -- app's pause menu, retail results) leaves START to the game: hiding it there ate the START the 1P results screen waits for.
 function S:set_active(active,hide_start) self.active=active;self.hide_start=active and hide_start or nil end
 function S:poll()
  I.settle(self.g,self.port)
  local p=self.g.pad(self.port,true) or {};local out={}
  local held=(p.LEFT and 1 or 0)+(p.RIGHT and 2 or 0)+(p.DOWN and 4 or 0)+(p.UP and 8 or 0)
  if self.g.input_mask and offline(self.g) then
   if self.active then self.g.input_mask(self.port,15|(self.hide_start and START or 0));self.masked=15;self.masked_start=self.hide_start
   elseif self.masked then local keep=self.masked_start and p.START and START or 0;self.masked=held;self.g.input_mask(self.port,held|keep);if held==0 and keep==0 then self.masked=nil;self.masked_start=nil end end
  end
  local actions={accept=p.A,back=p.B,start=p.START,x=p.X,y=p.Y}
  for _,name in ipairs({'accept','back','start','x','y'}) do if actions[name] and not self.previous[name] then out[#out+1]=name end end
  -- Up/down win over left/right when both are held (a stick on a diagonal): one direction event at a time.
  local direction=(p.UP or (p.y or 0)>60) and 'up' or ((p.DOWN or (p.y or 0)<-60) and 'down' or nil)
  if not direction then direction=(p.LEFT or (p.x or 0)<-60) and 'left' or ((p.RIGHT or (p.x or 0)>60) and 'right' or nil) end
  if direction~=self.direction then self.held=0;if direction then out[#out+1]=direction end
  elseif direction then self.held=self.held+1;if self.held>=18 and (self.held-18)%5==0 then out[#out+1]=direction end end
  self.direction=direction;self.previous=actions;return out
 end
 function S:close()
  local p=self.g.pad and self.g.pad(self.port,true) or {};local l=latch(self.port)
  l.held_start=(self.hide_start and p.START) and true or false
  if self.g.input_mask and offline(self.g) then self.g.input_mask(self.port,l.held_start and START or 0) end;self.active=false;self.hide_start=nil;self.masked_start=nil;self.masked=nil;self.previous={};self.direction=nil;self.held=0
 end
 return I
end
