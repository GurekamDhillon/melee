-- Echo gameplay publication is an absolute owner-scoped journal operation.
return function(D)
 local A={};A.__index=A
 local function clone(v)return D.mod_codec.decode(D.mod_codec.encode(v))end
 function A.new(host)return setmetatable({host=host,manual={},owned={},visual={},visual_keys={},visual_ready={},warm_jobs={},visual_notes={}},A)end
 function A:capable()
  local g=self.host.g;return g.echoes and g.echoes(1).journal==true
 end
 function A:note(p,why)
  local old=self.visual_notes[p];local frame=self.present_frame or 0
  if not old or old.text~=why or frame-old.frame>=120 then
   if self.host.g.log then self.host.g.log('echo picture P'..p..': '..tostring(why))end
   self.visual_notes[p]={text=why,frame=frame}
  end
 end
 function A:retire(p)
  local g=self.host.g
  if self.warm_jobs[p] and g.warm_release then pcall(g.warm_release,self.warm_jobs[p])end;self.warm_jobs[p]=nil
  if self.visual[p] and g.afterimage_remove then pcall(g.afterimage_remove,self.visual[p])end
  self.visual[p]=nil;self.visual_keys[p]=nil;self.visual_ready[p]=nil
 end
 function A:reset()
  for p=1,6 do self:retire(p)end
  self.manual={};self.owned={};self.visual_notes={}
 end
 function A:command(arg)
  local ok,err=pcall(function()
   local allowed,why=self.host:allowed();assert(allowed,why);assert(not self.host:replaying(),'echo edit refused during rewind');assert(self:capable(),'native echo journal unavailable; rebuild required')
   local w={};for x in arg:gmatch('%S+')do w[#w+1]=x end
   local manual=clone(self.manual)
   if w[1]=='clear' then assert(#w==1,'usage: echo clear');manual={}
   else
    assert(w[1]=='add' and #w>=2 and #w<=4,'usage: echo add <delay> [move] [scale]');assert(self.host.g.player(1),'fighter absent')
    local delay=tonumber(w[2]);assert(delay and delay%1==0 and delay>=1 and delay<=60,'delay 1..60 required')
    local move=w[3] or 'any';assert(D.mod_echo.moves[move],'unknown echo move');local scale=tonumber(w[4] or '.4');assert(scale and scale==scale and scale>=.1 and scale<=4,'scale .1..4 required')
    manual[1]=manual[1] or {};assert(#manual[1]<8,'eight echo rules per fighter maximum')
    manual[1][#manual[1]+1]={delay=delay,match={move=move},damage=scale,knockback=1,once_per_move=true}
   end
   if self.host.prospective then self.host:prospective(self.host.engine,self.host.debug_equipped,self.host.pending,self.host.foes,self.host.drives and self.host.drives:snapshot(),nil,manual) end
   self.manual=manual;self.host.enabled=true;if self.host.options.activate then self.host.options.activate()end
  end)
  if not ok then self.host.g.log('echo: refused '..tostring(err));return false,err end;return true
 end
 function A:snapshot(raw)if raw then return {manual=self.manual,owned=self.owned}end;return {manual=clone(self.manual),owned=clone(self.owned)}end
 function A:validate(s)
  assert(type(s)=='table' and type(s.manual)=='table' and type(s.owned)=='table','invalid echo checkpoint')
  for p,rules in pairs(s.manual)do assert(type(p)=='number' and p%1==0 and p>=1 and p<=6 and type(rules)=='table' and not getmetatable(rules) and #rules<=8,'invalid echo owner');local n=0;for i in pairs(rules)do assert(type(i)=='number' and i%1==0 and i>=1 and i<=#rules,'invalid checkpoint echo array');n=n+1 end;assert(n==#rules,'sparse checkpoint echoes')
   for _,r in ipairs(rules)do assert(type(r)=='table' and not getmetatable(r) and type(r.match)=='table','invalid checkpoint echo record');for k in pairs(r)do assert(({delay=true,match=true,damage=true,knockback=true,once_per_move=true})[k],'unknown checkpoint echo field')end;for k in pairs(r.match)do assert(k=='move','unknown checkpoint echo match')end;assert(type(r.delay)=='number' and r.delay%1==0 and r.delay>=1 and r.delay<=60 and D.mod_echo.moves[r.match.move] and type(r.damage)=='number' and r.damage>=.1 and r.damage<=4 and r.knockback==1 and r.once_per_move==true,'invalid checkpoint echo')end
  end
  for p,v in pairs(s.owned)do assert(type(p)=='number' and p%1==0 and p>=1 and p<=6 and v==true,'invalid checkpoint echo ownership')end
  return clone(s)
 end
 function A:restore(s)
  s=self:validate(s);self:reset();self.manual=s.manual;self.owned=s.owned
 end
 function A:ops(ops,players,lost)
  local next_owned={}
  for p=1,6 do local d=self.host.engine:echo_description(p);local rules={}
   if lost[p] or not players[p] then self.manual[p]=nil end
   if players[p] and not lost[p]then
    for _,r in ipairs(d.rules)do rules[#rules+1]=r end
    for _,r in ipairs(self.manual[p] or {})do rules[#rules+1]=clone(r)end
   end
   assert(#rules<=8,'eight echo rules per fighter maximum')
   if #rules>0 then assert(self:capable(),'native echo journal unavailable; rebuild required');ops[#ops+1]={op='echoes',port=p,rules=rules};next_owned[p]=true
   elseif self.owned[p]then ops[#ops+1]={op='echoes',port=p,rules={}}end
  end;self.owned=next_owned
 end
 -- An echo description is a memoised, never-mutated table (mod_engine): its identity is its content, so the
 -- encoded picture key is computed once per description rather than six times a frame.
 local keys=setmetatable({},{__mode='k'})
 local function key_of(d) local k=keys[d];if not k then k=D.mod_codec.encode(d);keys[d]=k end;return k end
 function A:present(players)
  local g=self.host.g;self.present_frame=(self.present_frame or 0)+1;local ready=true
  for p=1,6 do
   local d=self.host.engine:echo_description(p);local key=key_of(d)
   if not players[p] or #d.copies==0 then self:retire(p)
   else
    if self.visual_keys[p] and self.visual_keys[p]~=key then self:retire(p)end
    if not self.visual[p] then
     if not g.echo_afterimage then self:note(p,'joint afterimage helper unavailable; picture unprepared')
     else
      local description={copies=#d.copies,spacing=d.copies[1].age,presentation_only=true,echoes={},visual=d.visual}
      for _,r in ipairs(d.rules)do for i,c in ipairs(d.copies)do if c.age==r.delay then local rule=clone(r);rule.delay=nil;rule.copy=i;description.echoes[#description.echoes+1]=rule end end end
      local ok,h,why=pcall(g.echo_afterimage,p,description)
      if ok and type(h)=='number' and h%1==0 and h>0 then self.visual[p]=h;self.visual_keys[p]=key
       -- The picture is on only inside a window (a status the rule needs, or a short window after the fighter's own hit): channel 2.
       if g.afterimage_bind then pcall(g.afterimage_bind,h,{status=2,tint={.9,.8,1,.7},tail={.35,.2,.6,0}}) end
      else self:note(p,ok and (why or 'afterimage emitter unavailable; retrying') or h)end
     end
    end
    if self.visual[p] and not self.visual_ready[p] then
     if not (g.warm and g.warm_done and g.warm_release)then self:note(p,'afterimage warm lifecycle unavailable; picture unprepared')
     else
      if not self.warm_jobs[p] then
       local ok,h,why=pcall(g.warm,{fighters={p}})
       if ok and type(h)=='number' and h%1==0 and h>0 then self.warm_jobs[p]=h
       else self:note(p,ok and (why or 'afterimage warm ticket unavailable; retrying') or h)end
      end
      if self.warm_jobs[p] then local ok,done,why=pcall(g.warm_done,self.warm_jobs[p])
       if not ok or why or done then
        pcall(g.warm_release,self.warm_jobs[p]);self.warm_jobs[p]=nil
        if ok and done and not why then self.visual_ready[p]=true;self.visual_notes[p]=nil
        else self:note(p,ok and why or done)end
       end
      end
     end
    end
    if not self.visual_ready[p] then ready=false end
   end
  end;return ready
 end
 function A:active()return next(self.manual)~=nil or next(self.owned)~=nil end
 return A
end
