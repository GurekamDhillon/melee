-- Generated shared factory; source: pc/scripts/lib/pickup_juice.lua.
local J=assert(load("-- Reusable offline pickup presentation. Embed this factory in sandbox entries (no require).\n-- One gd.items() snapshot per tick; effects never decide rewards or item physics.\nlocal J={}\nJ.defaults={glow=true,pool=true,sparkles=true,highlight=true,pop_trail=true,collect_burst=true,\n sound=true,hud_flash=true,blink=true,streak_frames=90,streak_step=70,streak_cap=900,\n max_effect_drops=30,max_collect_bursts=12,core_height=5.6875,drop_sound=246,pickup_sound=170,white_sound=250,volume=90,flash_frames=24}\nJ.pitch={red=-100,green=200,blue=0,yellow=100,white=300}\nlocal function call(f,...) if type(f)~='function' then return nil end;local ok,r=pcall(f,...);if ok then return r end end\nfunction J.new(g,settings)\n local s={};for k,v in pairs(J.defaults) do s[k]=v end;for k,v in pairs(settings or {}) do s[k]=v end\n local v={g=g,s=s,drops={},transients={},flash={},frame=0,streak=0,last=nil,seed=0}\n local function finish(h) if h then call(g.fx_end,h,0) end end\n local function play(kind,c,x,y,z)\n  v.seed=(v.seed+1)%2147483647\n  local h=call(g.fx_world,'PickupJuice_'..kind..'_'..c,x,y,z or 0,1,v.seed)\n  if not (type(h)=='number' and h>0) and not v.warned then\n   v.warned=true;call(g.log,'pickup juice: FX unavailable; mount the owning mod at boot (effects disabled safely)')\n  end\n  return type(h)=='number' and h>0 and h or nil\n end\n local function kill(d) for _,h in pairs(d.fx) do finish(h) end;d.fx={} end\n function v:drop(id,colour,x,y,lifetime)\n  if self.drops[id] then self:expire(id) end\n  colour=J.pitch[colour] and colour or 'white'\n  local d={colour=colour,x=x,y=y,z=0,age=0,lifetime=lifetime or 900,fx={}}\n  self.drops[id]=d\n  if s.sound then call(g.play_sound,s.drop_sound,{volume=s.volume,pitch=J.pitch[colour]}) end\n  local active=0;for _,other in pairs(self.drops) do if next(other.fx) then active=active+1 end end\n  if active<s.max_effect_drops then\n  for _,kind in ipairs({'glow','pool','sparkles','highlight','pop_trail'}) do\n   if s[kind] then d.fx[kind]=play(kind,colour,x,y+(kind=='pool' and .15 or 6+s.core_height),0) end\n  end\n  end\n  return d\n end\n function v:expire(id) local d=self.drops[id];if d then kill(d);self.drops[id]=nil end end\n function v:collect(id,colour,collector)\n  local d=self.drops[id];if not d then return nil end\n  colour=d.colour;self:expire(id)\n  if not self.last or self.frame-self.last>s.streak_frames then self.streak=0 else self.streak=self.streak+1 end\n  self.last=self.frame\n  local pitch=math.min(1200,J.pitch[colour]+math.min(s.streak_cap,self.streak*s.streak_step))\n  if s.sound then call(g.play_sound,colour=='white' and s.white_sound or s.pickup_sound,{volume=s.volume,pitch=pitch}) end\n  if s.hud_flash then self.flash[colour]=s.flash_frames end\n  if s.collect_burst then\n   if #self.transients>=s.max_collect_bursts then finish(self.transients[1].h);table.remove(self.transients,1) end\n   local h=play('collect_burst',colour,d.x,(d.visual_y or d.y+6)+s.core_height,d.z)\n   if h then self.transients[#self.transients+1]={h=h,left=24,total=24,x=d.x,y=(d.visual_y or d.y+6)+s.core_height,z=d.z,target=collector} end\n  end\n  return pitch\n end\n function v:tick()\n  self.frame=self.frame+1\n  local by={};for _,r in ipairs(call(g.items) or {}) do if r.handle then by[r.handle]=r end end\n  for id,d in pairs(self.drops) do\n   d.age=d.age+1;local r=by[id]\n   if r then d.seen=true;d.age=r.age or d.age;d.x=r.x;d.y=r.y;d.z=r.z or 0;d.visual_y=r.visual_y or r.y+6 end\n   if (type(id)=='number' and not r and (d.seen or d.age>2)) or d.age>=d.lifetime then self:expire(id)\n   else\n    local visible=not r or r.visible~=false or not s.blink\n    local opacity=visible and 1 or 0\n    for kind,h in pairs(d.fx) do\n     local x,y,z=d.x,(d.visual_y or d.y+6)+s.core_height,d.z\n     if kind=='pool' then y=d.y+.15\n     elseif kind=='highlight' then local angle=r and math.rad(r.rotation or 0) or d.age*math.pi/30;x=x+math.cos(angle)*3;y=y+math.sin(angle)*1.2;z=z+math.sin(angle)*3 end\n     call(g.fx_move,h,x,y,z)\n     -- fx_control overwrites all controls: specify brightness to preserve pop emphasis.\n     local bright=(kind=='pop_trail' or (kind=='glow' and s.pop_trail and d.age<24)) and 2 or 1\n     if d.opacity~=opacity or (kind=='glow' and d.age==24) or (kind=='pop_trail' and d.age==1) then\n      call(g.fx_control,h,-1,{opacity=opacity,brightness=bright},0)\n     end\n     if kind=='pop_trail' and d.age>=24 then finish(h);d.fx[kind]=nil end\n    end\n    d.opacity=opacity\n   end\n  end\n  for c,n in pairs(self.flash) do self.flash[c]=n>1 and n-1 or nil end\n  for i=#self.transients,1,-1 do\n   local t=self.transients[i];t.left=t.left-1\n   if t.left<=0 then finish(t.h);table.remove(self.transients,i)\n   elseif t.target then\n    local f=1-t.left/t.total\n    call(g.fx_move,t.h,t.x+(t.target.x-t.x)*f,t.y+(t.target.y-t.y)*f,t.z)\n   end\n  end\n end\n function v:clear()\n  for id in pairs(self.drops) do self:expire(id) end\n  for _,t in ipairs(self.transients) do finish(t.h) end\n  self.transients={};self.flash={};self.streak=0;self.last=nil\n end\n return v\nend\nreturn J\n","@lib/pickup_juice.lua","t"))()
local juice=J.new(gd)
local owned={};local status='I: plain pickup; J: same pickup with juice; R: clear'
local requested={};local started=false
local function clear()
 juice:clear();for h in pairs(owned) do gd.item_despawn(h) end;owned={}
end
local function spawn(with_juice)
 local p=gd.player(1);if not p then return end
 local h,why=gd.item_spawn('demo_juice',p.x+20,p.y+12,{vy=2,payload={colour='blue',amount=1}})
 if not h then status=tostring(why);return end
 owned[h]=with_juice and 'juice' or 'plain'
 if with_juice then juice:drop(h,'blue',p.x+20,p.y+12,900) end
 status=with_juice and 'JUICE: halo, pool, sparkles, pop trail, sound' or 'PLAIN: identical physics, scale, hover and rotation'
end
gd.command('demo_key',function(k) requested[k]=true end,'I plain; J juice; R clear')
gd.command('demo_state',function(token) gd.log('tour state '..token..' '..status) end,'Report owner state')
function on_match_start() if not started then started=true;gd.item_define('items/demo_juice/item.json') end end
function on_frame()
 if not gd.match().active then return end;on_match_start();gd.cpu_mode(2,'stand');juice:tick()
end
function on_tick()
 if not gd.match().active then return end
 for _,k in ipairs({'I','J','R'}) do
  if requested[k] or gd.key_pressed(k) then requested[k]=nil;if k=='R' then clear() else spawn(k=='J') end end
 end
end
function on_item_collect(e)
 if e.name=='demo_juice' and e.port==1 and owned[e.item] then
  if owned[e.item]=='juice' then juice:collect(e.item,'blue',gd.player(1)) end
  status='Collected '..owned[e.item]..' pickup';owned[e.item]=nil
 end
end
function on_item_expire(e) juice:expire(e.item);owned[e.item]=nil end
function on_draw()
 local a=gd.safe_area();gd.fill(a.x+12,a.y+12,math.min(620,a.w-24),76,0x101827DD)
 gd.text(a.x+24,a.y+22,'Pickup with juice | I plain / J juice / R clear',0xFFD369FF,1.2)
 gd.text(a.x+24,a.y+48,status,0xFFFFFFFF)
 if juice.flash.blue then gd.fill(a.x+24,a.y+67,180,5,0xFFFFFFFF) end
end
function on_unload() clear() end
function on_match_end() juice:clear();owned={};started=false end
function on_loadstate() clear() end
