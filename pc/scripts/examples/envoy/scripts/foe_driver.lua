-- Opponent technique driver. The retail AI cannot L-cancel, wavedash, perfect shield or tech on purpose, and the CPU
-- controller (gd.cpu_pad / cpu_script / cpu_macro) refuses a CPU in `fight` mode: it writes only in `script` mode,
-- where the retail AI writes nothing. So the only overlay the engine allows today is a brief SWITCH: fight -> script
-- (the macro or a pad hold for a few frames) -> fight. Measured (docs: _build/audit-20261003/envoy-foes/PROGRESS.md):
-- a switch replays identically from a savestate and the AI resumes within a few frames, but it re-runs the AI's init, so
-- its plan restarts at every switch. Hence: few, short switches, a cooldown, and only for techniques the opponent's build rewards.
--
-- Rules kept (docs/superpowers... netplay scoping "what to stop doing now"): decisions are a pure hash of the run seed, the
-- port, the logic frame and the opportunity (no stored generator); frames are logic frames; inputs come only from engine
-- state (fighter state, floor, attacker action); nothing here is wall time. Offline only (the engine refuses these calls online).
--
-- This switch-driver is the FALLBACK behind one switch (foe_lab `drive` flag, console `foe drive on|off`): an engine overlay (script adds
-- inputs on top of the retail AI in fight mode) would replace X:begin/X:finish and the cpu_macro/cpu_pad calls, nothing else.
--
-- Skill (0..1): the chance an opponent performs the technique at an opportunity. Grows with effective depth (depth+13*loop):
-- early opponents rarely do it, deep ones most of the time.
return function(D)
 local X={};X.__index=X
 -- trigger -> technique the driver performs for it
 X.drives={lcancel_hit='lcancel',lcancel='lcancel',wavedash='wavedash',air_dodge='wavedash',perfect_shield='ps',tech='tech'}
 X.tuning={base=.03,per_depth=.06,cap=.9,cooldown=45,switch_gap=20,max_hold=60,
  aerial_lo=65,aerial_hi=69,attack_lo=44,attack_hi=64,dash=20,run=21,
  lcancel_lead=4,tech_lead=5,wave_dist=30,ps_range_x=34,ps_range_y=26,
  -- first active frame by attack class (action_frame of the attacker one frame before the hit is first possible): jab, tilt, smash
  startup={jab=2,tilt=5,smash=9}}
 function X.skill(context)
  local t=X.tuning;local eff=D.mod_progression.effective(context or {depth=0,loop=0})
  return math.min(t.cap,t.base+t.per_depth*eff)
 end
 -- Deterministic unit-interval decision; no state.
 local function roll(seed,port,frame,salt)
  local s=(seed*7919+port*104729+frame*15485863+salt*32452843)%2147483646+1
  s=s*16807%2147483647;s=s*16807%2147483647
  return (s-1)/2147483646
 end
 X.roll=roll
 function X.new(g)
  return setmetatable({g=g,foes={},stats={},seed=1,attrs={}},X)
 end
 -- Which techniques a rolled build rewards (by trigger of any held modifier or keystone).
 function X.wanted(mods)
  local want={}
  local function scan(list) for _,m in ipairs(list or {}) do if mods[m.id] and X.drives[m.trigger or ''] then want[X.drives[m.trigger]]=true end end end
  scan(D.mod_pool)
  if D.keystones and D.keystones.records then scan(D.keystones.records()) end
  return want
 end
 function X:set(port,mods,context,seed)
  local want=X.wanted(mods);local any=next(want)~=nil
  if not any then self:clear(port);return false end
  self.foes[port]={want=want,skill=X.skill(context),seed=seed or 1,context=context,last_action=-1,cool=0,gap=0}
  self.stats[port]=self.stats[port] or {skill=X.skill(context),attempts={},events={},switches=0,opportunities={}}
  self.stats[port].skill=self.foes[port].skill
  return true
 end
 function X:clear(port)
  local st=self.foes[port];if st and st.active and self.g.cpu_mode then pcall(self.g.cpu_mode,port,'fight') end
  self.foes[port]=nil
 end
 function X:reset() for p in pairs(self.foes) do self:clear(p) end;self.foes={};self.stats={} end
 local function count(t,k) t[k]=(t[k] or 0)+1 end
 local function nearest(self,p,me)
  local best,bd
  for _,o in ipairs(self.g.players() or {}) do
   if o.port~=p and o.action and o.action>=14 then
    local d=math.abs(o.x-me.x)+math.abs(o.y-me.y);if not bd or d<bd then best,bd=o,d end
   end
  end
  return best
 end
 -- frames until the fighter reaches the floor under it, or nil when none within 40
 function X:landing(p,me)
  local g=self.g;if not g.floor_below then return nil end
  local a=self.attrs[p]
  if not a and g.cpu_attrs then local ok,v=pcall(g.cpu_attrs,p);if ok and type(v)=='table' then a=v;self.attrs[p]=v end end
  local grav=a and a.gravity or .095;local term=a and a.terminal_velocity or 1.7
  local fy=g.floor_below(me.x,me.y+1,120);if not fy then return nil end
  local y,vy=me.y,me.dy or me.vy or 0
  if vy>=0 and me.in_hitstun then return nil end -- still rising
  for n=1,40 do
   vy=math.max(-term,vy-grav);y=y+vy
   if y<=fy then return n end
  end
  return nil
 end
 local function actionable(a) return a==14 or (a>=15 and a<=23) end
 function X:begin(p,st,kind,frame,hold)
  local g=self.g;local ok=pcall(g.cpu_mode,p,'script');if not ok then return false end
  st.active={kind=kind,start=frame,release=frame+(hold or X.tuning.max_hold)}
  self.stats[p].switches=self.stats[p].switches+1;count(self.stats[p].attempts,kind)
  return true
 end
 function X:finish(p,st,frame)
  pcall(self.g.cpu_mode,p,'fight');st.active=nil;st.cool=frame+X.tuning.cooldown;st.gap=frame+X.tuning.switch_gap
 end
 -- One logic frame for every driven opponent.
 function X:frame(frame)
  self.nframes=(self.nframes or 0)+1
  local g=self.g;local t=X.tuning
  for p,st in pairs(self.foes) do
   local me=g.player(p)
   if me and me.cpu and me.action then
    me.dy=st.py and (me.y-st.py) or nil;st.py=me.y
    if st.active then
     local k=st.active.kind
     local done=frame>=st.active.release
     if k~='tech' and me.in_hitstun and not me.airborne then done=true end
     if k=='lcancel' and not me.airborne and (me.action_frame or 0)>=4 then done=true end
     if k=='lcancel' and me.airborne and me.action and (me.action<t.aerial_lo or me.action>t.aerial_hi) and frame>st.active.start+2 then done=true end
     if (k=='wavedash' or k=='ps') and frame>st.active.start+3 and g.cpu_script_done and g.cpu_script_done(p) and actionable(me.action) then done=true end
     if k=='tech' and not me.in_hitstun and not me.airborne and frame>st.active.start+3 and actionable(me.action) then done=true end
     if done then self:finish(p,st,frame) end
    elseif frame>=st.cool and frame>=st.gap and me.action>=14 and (me.hitlag or 0)<=0 then
     local foe=nearest(self,p,me)
     local fresh=me.action~=st.last_action
     -- 1. tech: tumbling toward the floor (one decision per tumble)
     if st.want.tech and me.airborne and me.in_hitstun then
      st.tech_seen=st.tech_seen or frame
      local n=self:landing(p,me)
      if n and n<=t.tech_lead and not st.tech_done then
       st.tech_done=true;count(self.stats[p].opportunities,'tech')
       if roll(st.seed,p,st.tech_seen,1)<st.skill and self:begin(p,st,'tech',frame,45) then
        pcall(g.cpu_macro,p,'tech',{at=math.max(0,n-2),dir='in'})
       end
      end
     elseif not me.in_hitstun then st.tech_seen=nil;st.tech_done=nil end
     -- 2. L-cancel: an aerial about to land
     if not st.active and st.want.lcancel and me.airborne and me.action>=t.aerial_lo and me.action<=t.aerial_hi then
      if fresh then st.lc_seen=frame;st.lc_done=false end
      local n=self:landing(p,me)
      if n and n<=t.lcancel_lead and not st.lc_done then
       st.lc_done=true;count(self.stats[p].opportunities,'lcancel')
       if roll(st.seed,p,st.lc_seen or frame,2)<st.skill and self:begin(p,st,'lcancel',frame,12) then
        pcall(g.cpu_pad,p,{buttons='L',l=255},3)
       end
      end
     end
     -- 3. perfect shield: a nearby attacker's ground attack about to connect
     if not st.active and st.want.ps and foe and not me.airborne and actionable(me.action) and foe.action>=t.attack_lo and foe.action<=t.attack_hi then
      local dx,dy=math.abs(foe.x-me.x),math.abs(foe.y-me.y)
      if dx<=t.ps_range_x and dy<=t.ps_range_y then
       local a=foe.action;local class=(a<=47 and 'jab') or (a>=53 and a<=57 and 'tilt') or 'smash'
       local want_at=t.startup[class]-1
       if (foe.action_frame or 0)==want_at and st.ps_for~=foe.action..':'..frame//30 then
        st.ps_for=foe.action..':'..frame//30;count(self.stats[p].opportunities,'ps')
        if roll(st.seed,p,frame,3)<st.skill and self:begin(p,st,'ps',frame,26) then
         pcall(g.cpu_macro,p,'perfect_shield',{at=0,hold=14})
        end
       end
      end
     end
     -- 4. wavedash: closing in on foot
     if not st.active and st.want.wavedash and fresh and (me.action==t.dash or me.action==t.run) and foe then
      local dx=foe.x-me.x;local toward=(dx>0 and me.facing>0) or (dx<0 and me.facing<0)
      if toward and math.abs(dx)>t.wave_dist then
       count(self.stats[p].opportunities,'wavedash')
       if roll(st.seed,p,frame,4)<st.skill and self:begin(p,st,'wavedash',frame,50) then
        pcall(g.cpu_macro,p,'wavedash',{angle=20})
       end
      end
     end
    end
    st.last_action=me.action
   end
  end
 end
 -- engine skill events for the stats (the driver never needs them to act)
 function X:on_skill(e)
  local s=self.stats[e.port];if not s then return end
  if X.drives[e.kind] or e.kind=='lcancel' or e.kind=='lcancel_miss' or e.kind=='wavedash' or e.kind=='perfect_shield' or e.kind=='tech' then count(s.events,e.kind) end
 end
 function X:report()
  local out={}
  for p,s in pairs(self.stats) do
   local a,o,e={},{},{}
   for k,v in pairs(s.attempts) do a[#a+1]=k..'='..v end;for k,v in pairs(s.opportunities) do o[#o+1]=k..'='..v end;for k,v in pairs(s.events) do e[#e+1]=k..'='..v end
   table.sort(a);table.sort(o);table.sort(e)
   out[#out+1]=('foe_driver frames=%s P%d skill=%.2f switches=%d opportunities[%s] attempts[%s] events[%s]'):format(tostring(self.nframes),p,s.skill,s.switches,table.concat(o,','),table.concat(a,','),table.concat(e,','))
  end
  table.sort(out);return out
 end
 return X
end
