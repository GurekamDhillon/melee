return function()
 local V={}
 local colours={red={1,.45,.4,1},green={.4,1,.55,1},blue={.4,.6,1,1},yellow={1,.9,.4,1},white={1,1,1,1}}
 local grades={E=1,D=2,C=3,B=4,A=5,S=6}
 local function call(f,...) if type(f)~='function' then return nil end;local ok,r=pcall(f,...);if ok then return r end end
 function V.new(g,tuning)
  local s=(tuning or {}).visuals or {};local v={};local handle,remaining,total,strength
  function v:clear() if handle then call(g.post_remove,handle) end;handle=nil;remaining=0 end
  function v:pickup(grade,colour)
   self:clear();if s.pulse==false then return false end
   total=math.floor(math.max(12,math.min(18,tonumber(s.pulse_frames) or 12)));remaining=total
   strength=.02+.01*(grades[grade] or 3)
   handle=call(g.post_add,'shaders/drive-pulse.wgsl',{stage='world',order=20,params={tint=colours[colour] or colours.white,strength=strength}})
   return handle~=nil and handle~=false
  end
  function v:tick()
   if not handle then return end;remaining=remaining-1
   if remaining<=0 then self:clear() else call(g.post_set,handle,{params={strength=strength*remaining/total}}) end
  end
  return v
 end
 return V
end
