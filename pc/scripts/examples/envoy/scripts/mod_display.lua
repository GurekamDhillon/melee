-- Visual adapter: shader state is rebuilt from checkpointed Lua metadata.
return function(D)
 local M={};M.__index=M
 local names=D.mod_status.order
 local look_index={};for i,name in ipairs(names) do look_index[name]=i end
 local function clamp(x,a,b) return math.max(a,math.min(b,x)) end
 local function keys(t) local out={};for k in pairs(t or {}) do out[#out+1]=k end;table.sort(out);return out end
 local function state(e)
  e.display=e.display or {last_pulse=-30,pulse_start=-100,pulse_strength=0,trace_key='',intensity=.65}
  return e.display
 end
 local function call(g,name,...)
  if not g[name] then return nil,'missing gd.'..name end
  local ok,a,b=pcall(g[name],...);if not ok then return nil,a end;return a,b
 end
 local function cleanup(g,name,...) if g[name] then pcall(g[name],...) end end
 function M.new(g,engine) return setmetatable({g=g,engine=engine,selected={},ready=false,look_cache={}},M) end
 function M:intensity(value)
  assert(type(value)=='number' and value==value and value>=0 and value<=1,'intensity must be 0..1')
  state(self.engine).intensity=value
 end
 function M:clear(port)
  if port then
   if self.selected[port] then cleanup(self.g,'fighter_shader',port,nil);self.selected[port]=nil end
   if self.post then cleanup(self.g,'post_remove',self.post);self.post=nil end
   return
  else
   for p in pairs(self.selected) do cleanup(self.g,'fighter_shader',p,nil) end;self.selected={};self.look_cache={}
  end
  if self.post then cleanup(self.g,'post_remove',self.post);self.post=nil end
  if self.warm_post then cleanup(self.g,'post_remove',self.warm_post);self.warm_post=nil end
  if self.job then cleanup(self.g,'warm_release',self.job);self.job=nil end
  self.ready=false;self.error=nil;self.note=nil;self.restoring=false
 end
 local function fail(self,why) self:clear();self.error=why or "visual preparation failed";return false,self.error end
 -- `present` (optional) is the caller's own fresh port->state table from this frame; it saves six gd.player calls.
 function M:warm(engine,present)
  self.engine=engine or self.engine;self.note=nil
  if self.error then return false,self.error end
  local ports={};local changed=false
  for p=1,6 do if present and present[p] or not present and self.g.player(p) then ports[#ports+1]=p;if not self.selected[p] then changed=true end
   elseif self.selected[p] then call(self.g,'fighter_shader',p,nil);self.selected[p]=nil end end
  if #ports==0 then return false,'waiting for loaded fighters' end
  if self.ready and not changed then return true end
  if changed then
   self.ready=false
   if self.job then call(self.g,'warm_release',self.job);self.job=nil end
   for _,p in ipairs(ports) do if not self.selected[p] then
    local ok,why=call(self.g,'fighter_shader',p,'shaders/modifiers_surface.wgsl',{params={0,0}})
    if not ok then return fail(self,why) end;self.selected[p]=true
   end end
  end
  if not self.shader then
   local shader,why=call(self.g,'shader_load','shaders/modifiers_chain.wgsl',{params={progress=0,strength=0}})
   if not shader then return fail(self,why) end;self.shader=shader
  end
  if not self.job then
   local post,why=call(self.g,'post_add',self.shader,{stage='world',order=50,params={progress=0,strength=0}})
   if not post then return fail(self,why) end;self.warm_post=post
   local job,err=call(self.g,'warm',{fighters=ports})
   if not job then return fail(self,err) end;self.job=job
  end
  local done,why=self.g.warm_done(self.job)
  if why then return fail(self,why) end
  local status,err=self.g.shader_status(self.shader)
  if not status or not status.valid then return fail(self,err or (status and status.error) or 'chain shader invalid') end
  local prepared,err=call(self.g,'post_ready',self.warm_post)
  if err then return fail(self,err) end
  if not done or not prepared then return false,'preparing fighter and chain pipelines' end
  self.g.warm_release(self.job);self.job=nil
  self.g.post_remove(self.warm_post);self.warm_post=nil
  self.ready=true;self.previously_warmed=true;self.warmed_ports={}
  for _,p in ipairs(ports) do self.warmed_ports[p]=true end
  return true
 end
 -- The parameter array depends on the build, the drive looks, the statuses' stacks and the intensity; only its
 -- first slot (the clock) changes every frame. `look_key` is that content as one cheap string; the array for a
 -- port is rebuilt only when it changes, and otherwise the clock slot is rewritten in place.
 local function look_key(e,p,meta)
  local parts={tostring(meta.intensity)}
  for _,name in ipairs(names) do local v=(e.statuses[p] or {})[name];if v then parts[#parts+1]=name..(v.stacks or 1)..'/'..(v.max or 1) end end
  local eq=e.equipped[p];if eq and next(eq) then parts[#parts+1]=table.concat(keys(eq),',') end
  for i,drive in ipairs((meta.drive_build or {})[p] or {}) do parts[#parts+1]=i..tostring(drive.rarity)..tostring(drive.colour) end
  return table.concat(parts,'|')
 end
 local build_params
 local function params(self,e,p)
  local meta=state(e);local key=look_key(e,p,meta);local c=self.look_cache[p]
  if not c or c.key~=key or c.rules~=e.rules then c={key=key,rules=e.rules,out=build_params(e,p)};self.look_cache[p]=c end
  c.out[1]=e.frame/60;return c.out
 end
 function build_params(e,p)
  local meta=state(e);local out={e.frame/60,meta.intensity,0,0,0,0,0,0,0,0,0,0,0,0,0,0}
  for i,name in ipairs(names) do local v=(e.statuses[p] or {})[name]
   if v then out[i+2]=clamp((v.stacks or 1)/(v.max or 1),0,1) end
  end
  local mods={};for _,id in ipairs(keys(e.equipped[p])) do local rule=e.rules[id]
   if rule and rule.visual then mods[#mods+1]={id=id,v=rule.visual} end end
  local hues={red=.02,green=.33,blue=.57,yellow=.14,purple=.76,white=0};local looks={red='burn',green='haste',blue='chill',yellow='momentum',purple='curse',white='guarded'}
  local strengths={common=0,magic=.15,rare=.35,unique=.6}
  for i,drive in ipairs((meta.drive_build or {})[p] or {}) do local rarity=tostring(drive.rarity):lower();local strength=strengths[rarity] or 0
   if strength>0 then mods[#mods+1]={id='drive_'..i,v={look=looks[drive.colour] or 'guarded',hue=hues[drive.colour] or 0,strength=strength,priority=rarity=='unique' and 30 or 5}} end
  end
  table.sort(mods,function(a,b) if (a.v.priority or 0)==(b.v.priority or 0) then return a.id<b.id end;return (a.v.priority or 0)>(b.v.priority or 0) end)
  local cx,cy,total=0,0,0
  for i,m in ipairs(mods) do local hue=(m.v.hue or 0)%1;local strength=clamp(m.v.strength or .35,0,1)
   if i==1 then out[10]=(look_index[m.v.look] or 0)+hue;out[11]=strength
   elseif i==2 then out[12]=(look_index[m.v.look] or 0)+hue;out[13]=strength
   else cx=cx+math.cos(hue*math.pi*2)*strength;cy=cy+math.sin(hue*math.pi*2)*strength;total=total+strength end
  end
  if total>0 then out[15]=(math.atan(cy,cx)/(math.pi*2))%1;out[16]=clamp(total,0,1) end
  return out
 end
 function M:update(engine,present)
  local e=engine or self.engine;self.engine=e
  if not self.ready then return false,'visual pipelines not ready' end
  local meta=state(e);local key=tostring(meta.trace_generation or 0)..':'..table.concat(e.trace or {},' | ')
  if key~=meta.trace_key then
   meta.trace_key=key
   local count=0
   for _,rule in ipairs(e.list or {}) do
    local found=false
    for _,line in ipairs(e.trace or {}) do
     if line:sub(-#('by '..rule.label))=='by '..rule.label or line:sub(-#('from '..rule.label))=='from '..rule.label then found=true;break end
    end
    if found then count=count+1 end
   end
   if count>=2 and e.frame-meta.last_pulse>=30 then
    meta.last_pulse=e.frame;meta.pulse_start=e.frame;meta.pulse_strength=math.min(.06,.015*count)
   end
  end
  for p in pairs(self.selected) do
   if present and present[p] or not present and self.g.player(p) then
    local ok,why=self.g.fighter_shader_set(p,{params=params(self,e,p)})
    if not ok then self.error=why;self.ready=false;return false,why end
   end
  end
  local age=e.frame-meta.pulse_start
  if age>=0 and age<18 and meta.intensity>0 then
   if not self.post then local post,why=self.g.post_add(self.shader,{stage='world',order=50,params={progress=age/18,strength=meta.pulse_strength*meta.intensity}})
    if not post then return fail(self,why) end;self.post=post
   else self.g.post_set(self.post,{params={progress=age/18,strength=meta.pulse_strength*meta.intensity}}) end
  elseif self.post then self.g.post_remove(self.post);self.post=nil end
  return true
 end
 function M:on_loadstate(engine)
  self.engine=engine or self.engine
  if self.post then cleanup(self.g,'post_remove',self.post);self.post=nil end
  -- gd.warm traversals fork rewind history. A paused restore must only reuse
  -- this scene's previously prepared program/material pipelines, never warm.
  if not self.previously_warmed or not self.shader then
   self.ready=false;self.restoring=false
   self.note='visual cache unavailable for paused restore; resume to prepare'
   return false,self.note
  end
  for p=1,6 do
   local active=next(self.engine.equipped[p] or {}) or next(self.engine.statuses[p] or {}) or next((state(self.engine).drive_build or {})[p] or {})
   if active and self.g.player(p) and not self.selected[p] then
    if not (self.warmed_ports or {})[p] then
     self.ready=false;self.restoring=false
     self.note='fighter visual variant was not prepared before this checkpoint'
     return false,self.note
    end
    local ok,why=call(self.g,'fighter_shader',p,'shaders/modifiers_surface.wgsl',{params={0,0}})
    if not ok then return fail(self,why) end;self.selected[p]=true
   end
  end
  self.ready=true;self.restoring=false;self.note=nil;return self:update(self.engine)
 end
 function M:tick(engine)
  -- Host ticks do not advance timers or traverse gd.warm after restoration.
 end
 function M:scene_end()
  self:clear();self.shader=nil;self.restoring=false
  self.previously_warmed=false;self.warmed_ports=nil
 end
 function M:draw(engine)
  local e=engine or self.engine;local g=self.g;local area=g.safe_area();local width=math.min(420,area.w-24)
  local rows={'Modifier LAB | '..(self.error or self.note or (self.ready and 'ready' or 'warming'))}
  for p=1,6 do if g.player(p) then
   local mods={};for _,id in ipairs(keys(e.equipped[p])) do mods[#mods+1]=(e.rules[id] or {}).label or id end
   local statuses={};for _,name in ipairs(names) do local v=(e.statuses[p] or {})[name]
    if v then statuses[#statuses+1]=name..' x'..v.stacks..' ('..math.max(0,v.expires-e.frame)..'f)' end end
   rows[#rows+1]='P'..p..': '..(#mods>0 and table.concat(mods,', ') or 'no modifiers')
   if #statuses>0 then rows[#rows+1]='  '..table.concat(statuses,', ') end
  end end
  rows[#rows+1]='Last chain:'
  for _,line in ipairs(e.trace or {}) do rows[#rows+1]=line end
  local max_rows=math.floor((area.h-32)/15);while #rows>max_rows do table.remove(rows) end
  local x,y=area.x+12,area.y+12;g.fill(x,y,width,#rows*15+12,0x16202AE0)
  for i,text in ipairs(rows) do
   if g.kit and g.kit.available() then g.kit.text(x+8,y+8+i*15,text,'body',0xE8EEF4FF,'left',{max_w=width-16})
   else local limit=math.floor((width-16)/6);if #text>limit then text=text:sub(1,limit-3)..'...' end;g.text(x+8,y+4+(i-1)*15,text,0xE8EEF4FF,1) end
  end
 end
 return M
end
