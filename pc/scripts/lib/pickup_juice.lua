-- Reusable offline pickup presentation. Embed this factory in sandbox entries (no require).
-- One gd.items() snapshot per tick; effects never decide rewards or item physics.
local J={}
J.defaults={glow=true,pool=true,sparkles=true,highlight=true,pop_trail=true,collect_burst=true,
 sound=true,hud_flash=true,blink=true,streak_frames=90,streak_step=70,streak_cap=900,
 max_effect_drops=30,max_collect_bursts=12,core_height=5.6875,drop_sound=246,pickup_sound=170,white_sound=250,volume=90,flash_frames=24}
J.pitch={red=-100,green=200,blue=0,yellow=100,white=300}
local function call(f,...) if type(f)~='function' then return nil end;local ok,r=pcall(f,...);if ok then return r end end
function J.new(g,settings)
 local s={};for k,v in pairs(J.defaults) do s[k]=v end;for k,v in pairs(settings or {}) do s[k]=v end
 local v={g=g,s=s,drops={},transients={},flash={},frame=0,streak=0,last=nil,seed=0}
 local function finish(h) if h then call(g.fx_end,h,0) end end
 local function play(kind,c,x,y,z)
  v.seed=(v.seed+1)%2147483647
  local h=call(g.fx_world,'PickupJuice_'..kind..'_'..c,x,y,z or 0,1,v.seed)
  if not (type(h)=='number' and h>0) and not v.warned then
   v.warned=true;call(g.log,'pickup juice: FX unavailable; mount the owning mod at boot (effects disabled safely)')
  end
  return type(h)=='number' and h>0 and h or nil
 end
 local function kill(d) for _,h in pairs(d.fx) do finish(h) end;d.fx={} end
 function v:drop(id,colour,x,y,lifetime)
  if self.drops[id] then self:expire(id) end
  colour=J.pitch[colour] and colour or 'white'
  local d={colour=colour,x=x,y=y,z=0,age=0,lifetime=lifetime or 900,fx={}}
  self.drops[id]=d
  if s.sound then call(g.play_sound,s.drop_sound,{volume=s.volume,pitch=J.pitch[colour]}) end
  local active=0;for _,other in pairs(self.drops) do if next(other.fx) then active=active+1 end end
  if active<s.max_effect_drops then
  for _,kind in ipairs({'glow','pool','sparkles','highlight','pop_trail'}) do
   if s[kind] then d.fx[kind]=play(kind,colour,x,y+(kind=='pool' and .15 or 6+s.core_height),0) end
  end
  end
  return d
 end
 function v:expire(id) local d=self.drops[id];if d then kill(d);self.drops[id]=nil end end
 function v:collect(id,colour,collector)
  local d=self.drops[id];if not d then return nil end
  colour=d.colour;self:expire(id)
  if not self.last or self.frame-self.last>s.streak_frames then self.streak=0 else self.streak=self.streak+1 end
  self.last=self.frame
  local pitch=math.min(1200,J.pitch[colour]+math.min(s.streak_cap,self.streak*s.streak_step))
  if s.sound then call(g.play_sound,colour=='white' and s.white_sound or s.pickup_sound,{volume=s.volume,pitch=pitch}) end
  if s.hud_flash then self.flash[colour]=s.flash_frames end
  if s.collect_burst then
   if #self.transients>=s.max_collect_bursts then finish(self.transients[1].h);table.remove(self.transients,1) end
   local h=play('collect_burst',colour,d.x,(d.visual_y or d.y+6)+s.core_height,d.z)
   if h then self.transients[#self.transients+1]={h=h,left=24,total=24,x=d.x,y=(d.visual_y or d.y+6)+s.core_height,z=d.z,target=collector} end
  end
  return pitch
 end
 function v:tick()
  self.frame=self.frame+1
  local by={};for _,r in ipairs(call(g.items) or {}) do if r.handle then by[r.handle]=r end end
  for id,d in pairs(self.drops) do
   d.age=d.age+1;local r=by[id]
   if r then d.seen=true;d.age=r.age or d.age;d.x=r.x;d.y=r.y;d.z=r.z or 0;d.visual_y=r.visual_y or r.y+6 end
   if (type(id)=='number' and not r and (d.seen or d.age>2)) or d.age>=d.lifetime then self:expire(id)
   else
    local visible=not r or r.visible~=false or not s.blink
    local opacity=visible and 1 or 0
    for kind,h in pairs(d.fx) do
     local x,y,z=d.x,(d.visual_y or d.y+6)+s.core_height,d.z
     if kind=='pool' then y=d.y+.15
     elseif kind=='highlight' then local angle=r and math.rad(r.rotation or 0) or d.age*math.pi/30;x=x+math.cos(angle)*3;y=y+math.sin(angle)*1.2;z=z+math.sin(angle)*3 end
     call(g.fx_move,h,x,y,z)
     -- fx_control overwrites all controls: specify brightness to preserve pop emphasis.
     local bright=(kind=='pop_trail' or (kind=='glow' and s.pop_trail and d.age<24)) and 2 or 1
     if d.opacity~=opacity or (kind=='glow' and d.age==24) or (kind=='pop_trail' and d.age==1) then
      call(g.fx_control,h,-1,{opacity=opacity,brightness=bright},0)
     end
     if kind=='pop_trail' and d.age>=24 then finish(h);d.fx[kind]=nil end
    end
    d.opacity=opacity
   end
  end
  for c,n in pairs(self.flash) do self.flash[c]=n>1 and n-1 or nil end
  for i=#self.transients,1,-1 do
   local t=self.transients[i];t.left=t.left-1
   if t.left<=0 then finish(t.h);table.remove(self.transients,i)
   elseif t.target then
    local f=1-t.left/t.total
    call(g.fx_move,t.h,t.x+(t.target.x-t.x)*f,t.y+(t.target.y-t.y)*f,t.z)
   end
  end
 end
 function v:clear()
  for id in pairs(self.drops) do self:expire(id) end
  for _,t in ipairs(self.transients) do finish(t.h) end
  self.transients={};self.flash={};self.streak=0;self.last=nil
 end
 return v
end
return J
