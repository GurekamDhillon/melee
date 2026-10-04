-- Reads the physical pre-mask pad. Only D-pad masking is supported by the engine.
-- The app pauses gameplay for other menu buttons and owns pause/resume lifecycle.
return function(D)
 local I={};local S={};S.__index=S
 function I.new(g) return setmetatable({g=g,active=false,previous={},held=0},S) end
 function S:set_active(active) self.active=active end
 function S:poll()
  local p=self.g.pad(1,true) or {};local out={}
  local held=(p.LEFT and 1 or 0)+(p.RIGHT and 2 or 0)+(p.DOWN and 4 or 0)+(p.UP and 8 or 0)
  if self.g.input_mask then
   if self.active then self.g.input_mask(1,15);self.masked=15
   elseif self.masked then self.masked=held;self.g.input_mask(1,held);if held==0 then self.masked=nil end end
  end
  local actions={accept=p.A,back=p.B,start=p.START}
  for _,name in ipairs({'accept','back','start'}) do if actions[name] and not self.previous[name] then out[#out+1]=name end end
  local direction=(p.UP or (p.y or 0)>60) and 'up' or ((p.DOWN or (p.y or 0)<-60) and 'down' or nil)
  if direction~=self.direction then self.held=0;if direction then out[#out+1]=direction end
  elseif direction then self.held=self.held+1;if self.held>=18 and (self.held-18)%5==0 then out[#out+1]=direction end end
  self.direction=direction;self.previous=actions;return out
 end
 function S:close() if self.g.input_mask then self.g.input_mask(1,0) end;self.active=false;self.masked=nil;self.previous={};self.direction=nil;self.held=0 end
 return I
end
