-- Presentation of earned states and crits (ONE meaning per output; see mod_skill.lua for the table):
--   earned afterimage : one emitter per fighter, created once, bound to the fighter's native timed-status channel 1
--                       (the host writes the earned status's remaining frames there), tinted by the status's CAUSE, so it
--                       is on exactly while an earned status lasts and off the moment it ends. Never continuous.
--   crit moment       : on `on_crit` one impact-frame post pass (the impact-drill sequence, toned by `strength`, deliberately
--                       light at low strength), centred on the contact point, one pass at a time, shader warmed at match start.
--   crit tracer       : a tracer on the attacker's active hitboxes for a short window: "this hit carries something extra".
--   toast             : only a strong crit.
-- Everything is presentation: no gameplay write is made here. Every gd call is guarded; a missing API disables that part
-- and logs once, it never throws into the host.
return function(D)
 local F={};F.__index=F
 F.tuning={slow=1,intensity=1,strong=.75,tracer_frames=24,tracer_width_low=3,tracer_width_high=11,burst_gap=.30,repeat_tear_amt=.5,repeat_tear_r=.45,min_s=.12,afterimage={copies=6,spacing=3,lifetime=18,fade=.6}}
 local function clamp(x,a,b) return x<a and a or (x>b and b or x) end
 -- The impact-drill sequence: one scalar s (0..1) drives every layer (demos/impact-drill, same shader).
 function F.look(s)
  -- A light crit is a crisp accent (a bright ring edge, a firmer ring push, a short contrast pop), never a flash: `accent` fades out as the layers above it take over.
  return {dur=.22+.28*s,tear_s=(s<.45) and (1/60) or (2/60),tear_amt=.30+.70*s,tear_inv=clamp((s-.55)/.3,0,1)*.9,tear_r=.30+1.60*s,tear_thr=.45,
   lines=clamp((s-.15)/.6,0,1)*.9,blur=.010+.070*s,ca=.003+.013*s,ring=.60+.40*s,contrast=.38+.62*s,accent=clamp(1.0-.9*s,.15,1)}
 end
 -- Crit strength (0..1, the engine's) to the sequence strength: a floor so the lightest crit still reads, then linear.
 function F.sequence_strength(strength,intensity)
  intensity=intensity or F.tuning.intensity
  return clamp((F.tuning.min_s+(1-F.tuning.min_s)*clamp(strength or 0,0,1))*intensity,0,1)
 end
 function F.new(host)
  return setmetatable({host=host,g=host.g,ai={},hold={},tracer={},failed={},stat={crits=0,passes=0,restarts=0,toasts=0,tracers=0},last_start=-10},F)
 end
 function F:note(key,text) if not self.failed[key] then self.failed[key]=true;if self.g.log then self.g.log('earned fx: '..text) end end end
 local function offline(g) local m=g.match();return m and m.active and not m.netplay end
 -- ---- earned afterimages ------------------------------------------------------------------------------------------
 function F:retire_afterimage(p)
  local a=self.ai[p];if not a then return end
  local g=self.g
  if a.warm and g.warm_release then pcall(g.warm_release,a.warm) end
  if a.handle and g.afterimage_remove then pcall(g.afterimage_remove,a.handle) end
  self.ai[p]=nil
 end
 function F:reset()
  for p=1,6 do self:retire_afterimage(p);self:retire_tracer(p) end
  self:drop_pass();self.ai={};self.hold={};self.tracer={};self.shader=nil;self.warmed=nil;self.failed={}
 end
 function F:wanted(engine,p) return engine:earned(p)~=nil or engine:earned_source(p) end
 function F:frame(players)
  local g,engine=self.g,self.host.engine
  if not g.afterimage_add or not g.afterimage_bind or not offline(g) then return end
  for p=1,6 do
   local want=players[p] and self:wanted(engine,p)
   local a=self.ai[p]
   if not want then if a then self:retire_afterimage(p) end
   else
    if not a and not (self.hold[p] and engine.frame<self.hold[p]) then -- a refused emitter is retried every 90 frames, not every frame (two builds with pictures on one fighter refuse each other)
     local t=F.tuning.afterimage
     local ok,h,why=pcall(g.afterimage_add,p,{copies=t.copies,spacing=t.spacing,lifetime=t.lifetime,fade=t.fade,blend='additive',trigger='flag',surface='silhouette',
      tint=D.mod_skill.cause.lcancel.tint,tail=D.mod_skill.cause.lcancel.tail,intensity=0})
     if ok and h then a={handle=h};self.ai[p]=a
      if g.warm then local w1,w2=pcall(g.warm,{fighters={p}});if w1 and type(w2)=='number' then a.warm=w2 else a.ready=true end else a.ready=true end
     else self:note('afterimage',tostring(ok and why or h));self.ai[p]=nil;self.hold[p]=engine.frame+90 end
    end
    a=self.ai[p]
    if a and not a.ready then
     if a.warm and g.warm_done then
      local ok,done,why=pcall(g.warm_done,a.warm)
      if not ok or why or done then pcall(g.warm_release,a.warm);a.warm=nil;a.ready=true end
     else a.ready=true end
     if a.ready then pcall(g.afterimage_set,a.handle,{intensity=1}) end
    end
    local e=engine:earned(p)
    if a and a.ready and e and a.bound~=e.cause then
     local c=D.mod_skill.cause[e.cause]
     local ok,err=pcall(g.afterimage_bind,a.handle,{status=1,tint=c.tint,tail=c.tail})
     if ok and err~=nil and err~=false then a.bound=e.cause;self.stat.bound=(self.stat.bound or 0)+1 else self:note('bind',tostring(err)) end
    end
   end
  end
 end
 -- ---- crit moment -------------------------------------------------------------------------------------------------
 function F:ensure_shader()
  if self.failed.shader then return false end
  if self.shader then return true end
  local g=self.g
  if not g.shader_load then self:note('shader','shader API unavailable');return false end
  local ok,h,err=pcall(g.shader_load,'shaders/crit.wgsl',{params={center={.5,.5,0,0},progress=0,elapsed=0,tear_s=0,tear_amt=0,tear_inv=0,tear_r=0,tear_thr=.45,lines=0,blur=0,ca=0,ring=0,contrast=0,accent=0}})
  if not ok or not h then self:note('shader','crit shader disabled: '..tostring(ok and err or h));self.failed.shader=true;return false end
  self.shader=h;return true
 end
 -- Pipeline warm-up: a first use compiles the shader (a measured 90-260 ms hitch). Draw it once, finished, at match start.
 function F:tick()
  local g=self.g
  if not offline(g) or not g.post_add then return end
  if not self.warmed and self.host.enabled and self:ensure_shader() then
   local ok,h=pcall(g.post_add,self.shader,{order=100,stage='world',half=false,duration_frames=8,params={progress=1}})
   if ok and h then self.warmed=true end
  end
 end
 function F:drop_pass()
  if self.pass then if self.g.post_remove then pcall(self.g.post_remove,self.pass) end;self.pass=nil end
 end
 local function centre(g,point)
  if point and g.project and g.safe_area then
   local ok,x,y,visible=pcall(g.project,point.x,point.y,point.z or 0)
   if ok and x and visible then local a=g.safe_area();return {x/a.w,y/a.h,0,0} end
  end
  return {.5,.5,0,0}
 end
 -- Play the sequence at sequence-strength s (0..1) around `point` ({x,y,z}); one pass at a time (a live one is removed first).
 function F:play(s,point)
  local g=self.g
  if not offline(g) or not g.post_add or not self:ensure_shader() then return false end
  local L=F.look(s);local restarted=self.pass~=nil;self:drop_pass()
  local dur=L.dur*F.tuning.slow;L.dur=nil -- `slow` stretches the sequence for inspection only
  local now=g.time and g.time() or 0
  if now-self.last_start<F.tuning.burst_gap then L.tear_inv=0;L.tear_amt=math.min(L.tear_amt,F.tuning.repeat_tear_amt);L.tear_r=math.min(L.tear_r,F.tuning.repeat_tear_r) end
  L.center=centre(g,point);L.progress=0;L.elapsed=0
  local ok,h,err=pcall(g.post_add,self.shader,{order=100,stage='world',half=false,duration_frames=math.ceil(dur*60),clock=true,params=L})
  if not ok or not h then self:note('pass','crit pass failed: '..tostring(ok and err or h));self.failed.shader=true;return false end
  self.pass=h;self.last_start=now;self.stat.passes=self.stat.passes+1;if restarted then self.stat.restarts=self.stat.restarts+1 end
  self.stat.last_s=s;self.pass_until=(self.host.frames or 0)+math.ceil(dur*60)
  return true
 end
 -- ---- crit tracer: this hit carries something ----------------------------------------------------------------------
 function F:retire_tracer(p)
  local t=self.tracer[p];if t and t.handle and self.g.tracer_remove then pcall(self.g.tracer_remove,t.handle) end;self.tracer[p]=nil
 end
 function F:mark_hit(attacker,strength)
  local g=self.g
  if not g.tracer_add or not g.tracer_window or self.failed.tracer or not attacker then return false end
  local c=D.mod_skill.cause.crit;local t=self.tracer[attacker]
  local width=F.tuning.tracer_width_low+(F.tuning.tracer_width_high-F.tuning.tracer_width_low)*clamp(strength,0,1)
  if not t then
   local ok,h,why=pcall(g.tracer_add,{port=attacker,anchor='active_hitboxes',width=width,taper=.4,length=22,smoothing=4,shader='glow',trigger='flag',tint=c.tint,tail=c.tail,intensity=1})
   if not ok or not h then self:note('tracer','tracer unavailable: '..tostring(ok and why or h));self.failed.tracer=true;return false end
   t={handle=h};self.tracer[attacker]=t
  else pcall(g.tracer_set,t.handle,{width=width}) end
  local ok,res=pcall(g.tracer_window,t.handle,F.tuning.tracer_frames,{tint=c.tint,tail=c.tail})
  if ok and res then self.stat.tracers=self.stat.tracers+1;return true end
  self:note('window','tracer window refused: '..tostring(res));return false
 end
 -- ---- on_crit -------------------------------------------------------------------------------------------------------
 function F:crit(e)
  self.stat.crits=self.stat.crits+1
  local strength=clamp(tonumber(e.strength) or 0,0,1)
  local point=(type(e.x)=='number' and type(e.y)=='number') and {x=e.x,y=e.y,z=e.z or 0} or nil
  self:play(F.sequence_strength(strength),point)
  self:mark_hit(e.attacker,strength)
  if strength>=F.tuning.strong and self.host.toast then
   self.stat.toasts=self.stat.toasts+1
   self.host.toast(('Critical hit x%.1f'):format(tonumber(e.multiplier) or 1))
  end
 end
 function F:command(arg)
  local w={};for x in (arg or ''):gmatch('%S+') do w[#w+1]=x end
  if w[1]=='preview' then local s=clamp(tonumber(w[2]) or .5,0,1);return self:play(F.sequence_strength(s),nil)
  elseif w[1]=='intensity' then F.tuning.intensity=clamp(tonumber(w[2]) or 1,0,1);return true
  elseif w[1]=='slow' then F.tuning.slow=clamp(tonumber(w[2]) or 1,1,8);return true
  elseif w[1]=='off' then F.tuning.intensity=0;return true end
  self.g.log(('crit fx: crits=%d passes=%d restarts=%d tracers=%d toasts=%d bound=%d intensity=%.2f'):format(self.stat.crits,self.stat.passes,self.stat.restarts,self.stat.tracers,self.stat.toasts,self.stat.bound or 0,F.tuning.intensity))
  return true
 end
 return F
end
