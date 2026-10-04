-- Flow through the shared runtime, including accepted-but-queued commands.
return function(D)
  local S=D.save
  local R={};R.__index=R
  function R.new(g,mission,profile,commit,rng)
    return setmetatable({g=g,mission=mission,profile=profile,commit=commit,
      fighter=D.fighter.new(g),rng=rng or math.random,levels={},index=0,active=false},R)
  end
  -- A name is armed once per wait; repeated polling must not renew it.
  function R:wait(name,active,frames)
    self.waits=self.waits or {};name='envoy/'..name
    if active then
      if self.waits[name] then return end
      self.waits[name]=true
      if type(self.g.deadline)=='function' then pcall(self.g.deadline,name,frames or 600) end
    elseif self.waits[name] then
      self.waits[name]=nil
      if type(self.g.deadline_done)=='function' then pcall(self.g.deadline_done,name) end
    end
  end
  function R:clear_waits()
    local names={};for name in pairs(self.waits or {}) do names[#names+1]=name:sub(7) end
    for _,name in ipairs(names) do self:wait(name,false) end
  end
  function R:request_room(index)
    self:wait('staging',true)
    local level=self.levels[index]
    self.request={index=index,name=level.name,frames=0}
    local ok,why=self.mission:command(level.command)
    if not ok then self.request=nil;self:wait("staging",false);return false,why end
    return true
  end
  function R:adopt()
    local req=self.request;local current=self.mission.current
    if not req then return true end
    if current and current.doc.name==req.name and not self.mission.pending and not self.mission.staging then
      if not self.accepted then
        if not self.resuming then
          if D.companion.start_run then D.companion.start_run(self.companion) end
          if S.record_start then S.record_start(self.working,self.seed) end
          self:wait('run-start-save',true)
          local ok,why=self.commit(self.working);self:wait('run-start-save',false)
          if not ok then return false,why end
          self.profile=S.decode(S.encode(self.working))
        end
        self.accepted=true;self.start_damage=nil
      end
      self.index=req.index;self.request=nil;self:wait("staging",false)
      if self.index==#self.levels then self.calling={frames=0};self:wait("boss-call",true) end
      self.g.log('envoy: room installed '..req.name);return true
    end
    req.frames=req.frames+1
    if not (self.mission.pending or self.mission.staging) or req.frames>600 then return false,self.mission.last_install_error or 'mission request refused or timed out' end
    return true,'waiting'
  end
  function R.ready(p)
    local a=p and p.action
    return p and (a==nil or (a>13 and a~=42 and a~=43 and not(a>=322 and a<=324)))
  end
  function R:start(seed)
    if self.active or self.pending then return false,'run active or settlement pending' end
    if not self:restore_start_damage() then return false,'start cleanup pending' end
    if self.mission.staging or self.mission.retiring then return false,'mission installation or cleanup pending' end
    if not self.profile then return false,'profile unavailable' end
    local match=self.g.match();local p2=self.g.player(2)
    if not match or not match.active or match.netplay then return false,'active offline match required' end
    if not self.g.player(1) or not p2 or not p2.cpu then return false,'P1 and match-start P2 CPU required' end
    if self.profile.next_run>=1000000 then return false,'run ledger full' end
    self.working=S.decode(S.encode(self.profile));self.companion=self.working.companions[self.working.active]
    local resuming=self.profile.pending_seed~=nil
    seed=resuming and self.profile.pending_seed or seed or self.rng(2147483647)-1
    local valid,levels=pcall(D.campaign.levels,seed,self.g)
    if not valid then self.working=nil;self.companion=nil;return false,levels end
    self.levels=levels;self.seed=seed;self.interlude=nil
    self:wait("bench-readiness",true)
    self.start_damage={self.g.player(1).percent,p2.percent}
    local ok,why=self.fighter:begin(self.companion)
    if not ok then return self:abort_start(why) end
    self.resuming=resuming;self.accepted=nil;self.last_start_error=nil
    self.elapsed=0;self.id=self.profile.next_run;self.index=0;self.active=true;self.starting=true
    self.initial={};self.gained={};self.results=nil
    for _,k in ipairs(D.companion.stats) do
      self.initial[k]={grade=self.companion.stats[k].grade,points=self.companion.stats[k].points,level=self.companion.stats[k].level};self.gained[k]=0
    end
    if self.fighter.ready then
      self:wait("bench-readiness",false)
      self.starting=nil;ok,why=self:request_room(1)
      if not ok then return self:abort_start(why) end
      ok,why=self:adopt();if not ok then return self:abort_start(why) end
    end
    self.g.log(('envoy: run %d started'):format(self.id));return true
  end
  function R:pickup(colour)
    local k=D.companion.colours[colour]
    if k then self.gained[k]=self.gained[k]+1 end
  end
  function R:restore_start_damage()
    if not self.restore_damage then return true end
    if self.mission.staging or self.mission.stopping or self.mission.recovery then return false end
    local set=self.g.set_damage or self.g.set_percent
    if type(set)=='function' then
      for port=1,2 do
        local value=(self.start_damage or {})[port]
        if type(value)=='number' then
          local ok,accepted,why=pcall(set,port,value)
          if not ok or accepted==false or (accepted==nil and why~=nil) then return false end
        end
      end
    end
    self.start_damage=nil;self.restore_damage=nil;return true
  end
  function R:abort_start(why)
    self.g.log('envoy: start refused '..tostring(why))
    self.last_start_error='Run could not start: first room unavailable or loading timed out. Try again or return to garden.'
    self:clear_waits();self.active=false;self.request=nil;self.starting=nil;self.index=0
    self.interlude=nil;self.calling=nil;self.working=nil;self.companion=nil;self.results=nil
    local ok,err=pcall(self.mission.stop,self.mission,'envoy start cancelled')
    if not ok then self.g.log('envoy: cleanup refused '..tostring(err)) end
    self.fighter:clear()
    self.restore_damage=true;self:restore_start_damage();self.g.log('envoy: '..self.last_start_error)
    return false,self.last_start_error
  end
  function R:finish(reason)
    if self.active and not self.accepted then return self:abort_start('the first room was not available') end
    assert(reason=='win' or reason=='fail' or reason=='quit','invalid settlement reason')
    if not self.active and not self.pending then self.fighter:clear();return true end
    if not self.pending then
      self:clear_waits();self:wait("settle",true)
      local result={frames=self.elapsed,drives=self.gained,levels={},points={},before={},after={},boss=reason=='win' and 'Defeated' or 'Not defeated',outcome=reason}
      for _,k in ipairs(D.companion.stats) do
        result.before[k]={grade=self.initial[k].grade,points=self.initial[k].points,level=self.initial[k].level}
        result.after[k]={grade=self.companion.stats[k].grade,points=self.companion.stats[k].points,level=self.companion.stats[k].level}
        result.points[k]=self.companion.stats[k].points-self.initial[k].points
        result.levels[k]=self.companion.stats[k].level-self.initial[k].level
      end
      S.refresh_records(self.working) -- retain rare-grade peaks before reincarnation
      local moments=D.companion.settle and D.companion.settle(self.companion,reason) or {}
      S.record_result(self.working,reason,reason=="win" and math.max(1,self.elapsed or 0) or (self.elapsed or 0))
      self.pending=reason;self.index=0;self.active=false;self.request=nil;self.starting=nil;self.calling=nil;self.interlude=nil
      self.working.last_settled=self.id;self.working.next_run=self.id+1;self.working.last_result=reason
      if moments.evolution and moments.evolution.stat then result.after[moments.evolution.stat].grade=moments.evolution.grade end
      result.evolution=moments.evolution;result.reincarnation=moments.reincarnation;result.seed=self.seed
      self.results=result
      local ok,why=pcall(self.mission.stop,self.mission,'envoy '..reason)
      if not ok then self.g.log('envoy: mission cleanup refused '..tostring(why)) end
      self.fighter:clear()
    end
    local ok,why=self.commit(self.working)
    if not ok then self.g.log('envoy: settlement pending '..tostring(why));return false,why end
    self:wait("settle",false)
    self.profile=S.decode(S.encode(self.working));self.pending=nil;self.companion=nil;self.working=nil
    self.g.log(('envoy: settled run=%d result=%s one-write'):format(self.id,self.profile.last_result));return true
  end
  function R:frame(continuation)
    if not self.active then self.fighter:clear();self:restore_start_damage();return end
    if self.interlude then return end
    if not continuation and self.accepted then self.elapsed=(self.elapsed or 0)+1 end
    local match=self.g.match()
    if not match or not match.active or match.netplay then self:finish('fail');return end
    local ok,why=self.fighter:tick(self.companion)
    if not ok then self.g.log('envoy: fighter refused '..tostring(why));self:finish('fail');return end
    if self.starting then
      if not self.fighter.ready then return end
      self:wait("bench-readiness",false)
      self.starting=nil;ok,why=self:request_room(1)
      if not ok then self:finish('fail');return end
    end
    if self.request then
      ok,why=self:adopt()
      if not ok then self.g.log('envoy: '..tostring(why));if not self.accepted then self:abort_start(why) else self:finish('fail') end;return end
      if self.request then return end
    end
    if self.mission.staging then return end -- shared runtime may reload in place
    local current=self.mission.current
    if not current or current.doc.name~=self.levels[self.index].name then self:finish('quit');return end
    if self.calling then
      self.calling.frames=self.calling.frames+1
      if (self.calling.frames-1)%6==0 then
        local called=self.fighter:activate(60,0)
        if called then
          self:wait("boss-call",false)
          self.calling=nil;self.boss_falls=self.g.player(2).falls;self.boss_p1_falls=self.g.player(1).falls
          self.boss_frames=0;self.boss_defeated=false
          self.g.log('envoy: room boss called falls-baseline='..self.boss_falls)
        end
      end
      if self.calling then if self.calling.frames>=600 then self:finish('fail') end;return end
    end
    local result=current.run.state.result and current.run.state.result.status
    if result=='failed' then self:finish('fail');return end
    if self.index==#self.levels then
      self.boss_frames=self.boss_frames+1
      local m=current.doc.mission;local p1=self.g.player(1)
      if (m.objective.time and self.boss_frames>=m.objective.time*60) or
         (p1 and m.objective.lives and p1.falls-self.boss_p1_falls>=m.objective.lives) then self:finish('fail');return end
      local p2=self.g.player(2);if not p2 then return end
      if p2.falls>self.boss_falls then self.boss_defeated=true end
      local z=m.goal;local at_exit=p1 and z and math.abs(p1.x-z.x)<=z.w/2 and math.abs(p1.y-z.y)<=z.h/2
      if result=='complete' and self.boss_defeated and at_exit then self:finish('win') end
    elseif result=='complete' then
      local levels={}
      for _,k in ipairs(D.companion.stats) do levels[k]=self.companion.stats[k].level-self.initial[k].level end
      self.interlude={next_theme=self.levels[self.index+1].theme,depth=self.index+1,drives=self.gained,levels=levels}
    end
  end
  function R:continue()
    if not self.active or not self.interlude then return false,'no interlude' end
    self.interlude=nil
    local ok,why=self:request_room(self.index+1)
    if not ok then self.g.log('envoy: transition refused '..tostring(why));self:finish('fail');return false,why end
    return self:adopt()
  end
  return R
end
