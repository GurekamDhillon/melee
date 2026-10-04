-- Transaction coordinator and lifecycle; module factories also load under plain Lua.
return function(D)
  local R={};R.__index=R
  function R.new(g)
    local r=setmetatable({g=g,current=nil,generation=0,poll=0,guests={fighters={}}},R)
    g.command('mission',function(arg) return r:command(arg) end,'Standalone mission folders and fly fixtures')
    return r
  end
  function R:attempt(arg)
    local op=arg:match('^%s*(%S+)')
    if op=='play' or op=='reload' or op=='restart' or op=='fly' or op=='drop' or ((op=='maze'or op=='world')and not arg:match('%s+map%s*$')) then
      if not D.fighters.ready(self.g.player(1)) or ((op=='play' or op=='reload' or op=='restart') and not D.fighters.stage_ready(self.g)) then return nil,'waiting for controllable P1',true end
    end
    local ok,why=pcall(D.commands.dispatch,self,arg)
    return ok,why,not ok and D.fighters.transient(why)
  end
  function R:command(arg)
    arg=arg or '';self.pending=nil;self.pending_error=nil
    if (self.staging or self.recovery or self.stopping or self.world_generation)and not arg:match('^%s*stop%s*$')then
      local why=self.stopping and 'mission cleanup pending; mission stop retries' or self.recovery and 'staging recovery required; mission stop retries cleanup' or 'mission installation already in progress'
      self.g.log('mission: refused '..why);return nil,why
    end
    local op=arg:match('^%s*(%S+)')
    local replacing=op=='play'or op=='reload'or op=='restart'or op=='stop'or((op=='maze'or op=='world')and not arg:match('%s+map%s*$'))
    if self.current and self.current.run.finish and (op=='fly'or op=='drop'or op=='tour'or op=='clear')then
      return nil,'mission ended; restart, reroll or stop'
    end
    local ok,why,retry=self:attempt(arg)
    if retry then
      if D.mission_finish and replacing then D.mission_finish.release(self,op~='stop')end
      self.pending={arg=arg,frames=0};self.g.log('mission: '..arg..' waiting for controllable P1 (600-frame limit)');return true
    end
    if not ok then self.g.log('mission: refused '..tostring(why));return nil,why end
    if D.mission_finish and replacing then D.mission_finish.release(self,op~='stop')end
    return true
  end
  function R:retry()
    local pending=self.pending;if not pending then return end
    pending.frames=pending.frames+1
    if pending.frames%6==0 then
      local ok,why,retry=self:attempt(pending.arg)
      if ok or not retry then
        self.pending=nil
        if not ok then self.pending_error=tostring(why);self.g.log('mission: refused '..tostring(why))end
        return
      end
    end
    if pending.frames>=600 then
      self.pending=nil;self.pending_error=pending.arg..' timed out waiting for controllable P1';self.g.log('mission: '..self.pending_error)
    end
  end
  function R:dispose(c)
    if not c then return end
    if c==self.current then D.commands.cancel(self)end
    D.glue.cleanup(self.g,c.run)
    D.chunks.stop(c.stream)
    if c.root then D.world.unload(self.g,c.root) end
    for _,asset in pairs(c.asset_pool or {}) do pcall(self.g.model_release,asset) end
  end
  function R:stop(reason)
    local g=self.g;self.pending=nil
    if D.mission_finish then D.mission_finish.release(self)end
    if D.world_commands then D.world_commands.cancel(self)end
    if self.stopping and self.stopping.blocked then self.stopping=nil end
    if self.recovery then self.staging=self.recovery;self.recovery=nil;self.staging.step_key=nil end
    if self.staging then
      local match=g.match()
      if not match or not match.active or match.netplay or reason=='unload' or reason=='state load unsupported' then D.install.abandon(self)
      else self.staging.stop_reason=reason;D.install.fail(self,'staging cancelled: '..tostring(reason));return end
    end
    local match=g.match()
    if self.current and match and match.active and not match.netplay then
      -- Settlement precedes a queued scene launch: retire the old floor only after
      -- leaving grounded callbacks, while its collision is still live.
      local ok,why=pcall(function()g.fly_attack(1,false);g.fly(1,true);g.fly(1,false)end)
      if not ok then
        if not self.stopping then self.stopping={reason=reason,frames=0};g.log('mission: cleanup waiting for airborne P1: '..tostring(why))end
        return
      end
    end
    self.stopping=nil
    D.install.dispose_retired(self)
    D.camera.stop(g)
    local guests=self.current or self.guests
    if self.current then self:dispose(self.current);self.current=nil end
    pcall(g.fly_attack,1,false);pcall(g.fly,1,false)
    pcall(g.stage_hide,false);pcall(g.stage_restore_bounds)
    D.fighters.stop(g,guests)
    self.guests={fighters={}};self.guarded=nil;self.host_bounds=nil;self.host_spawns=nil
    if reason=='match end'or reason=='unload'then self.autostart_checked=nil end
    if self.launch and D.mission_launch then D.mission_launch.cancel(self,reason)end
    g.log('mission: stopped '..tostring(reason))
  end
  function R:install(name,marker,reload) return D.install.begin(self,name,marker,reload) end
  function R:match_start()
    if self.staging then return end -- partial CPU reservations belong to this transaction
    local match=self.g.match()
    if match and match.active and not match.netplay then
      self.guarded=D.fighters.guard(self.g,self.guests)
      if D.mission_launch and not self.autostart_checked then self.autostart_checked=true;local ok,why=pcall(D.mission_launch.autostart,self)
        if not ok then self.g.log('mission: autostart refused '..tostring(why))end end
    end
  end
  function R:pre_frame()
    local match=self.g.match()
    if match and match.active and not match.netplay then
      if not self.guarded then self:match_start() end
      -- Retirement shares the active frame mutation budget below.
    end
  end
  function R:frame()
    local match=self.g.match()
    if not match or not match.active or match.netplay then
      if self.current or self.pending or self.staging or self.recovery or self.world_generation or next(self.guests.fighters) then self:stop('match end') end;return
    end
    if not self.guarded then self:match_start() end
    if self.stopping then
      local t=self.stopping;t.frames=t.frames+1
      if t.frames==120 then t.blocked=true;self.g.log('mission: cleanup refused after 120 frames; mission stop retries');return end
      if not t.blocked and t.frames%6==0 then self:stop(t.reason)end
      return
    end
    if self.launch and self.launch.engine then return end
    if self.staging then
      D.install.tick(self)
      if D.maze_commands then D.maze_commands.settle(self)end
      if D.world_commands then D.world_commands.settle(self)end
      if D.mission_finish then D.mission_finish.tick(self)end
      return
    end
    if self.recovery then return end
    D.camera.release(self.g) -- also release a rollback pose when no mission is active
    self:retry()
    if self.staging then return end
    if D.world_commands then D.world_commands.tick(self)end
    if self.staging then return end
    if not self.current then return end
    self.poll=self.poll+1
    if self.poll%15==0 and not self.pending and not self.world_generation and D.fighters.ready(self.g.player(1)) then
      local c=self.current
      if not D.loader.same(c.doc.stamps,D.loader.stamps(self.g,c.doc.folder)) then self:command('reload');return end
    end
    local c=self.current
    c.run.frames=c.run.frames+1
    local ok,why=pcall(function()
      local player=self.g.player(1) -- one post-physics observation for all bookkeeping
      local membership=D.zones.sample(self.g,c,player,c.run.frames)
      player=membership.point
      D.glue.step(self.g,c,player)
      local streamed,err=D.chunks.update(c.stream,player,membership)
      if not streamed then self.g.log('mission: chunk refused '..tostring(err))end
      if self.retiring and c.stream.load_frame~=membership.frame and c.stream.wave_frame~=membership.frame then
        c.stream.load_frame=membership.frame;D.install.retire(self)
      end
      if D.maze_rewards then D.maze_rewards.tick(self.g,c) end
      D.fighters.tick(self.g,c)
      D.camera.tick(self.g,c,player)
      D.commands.tick(self)
      if D.mission_finish then D.mission_finish.tick(self)end
    end)
    if not ok then self.g.log('mission: frame refused '..tostring(why)) end
  end
  function R:tick()
    if D.mission_launch then D.mission_launch.tick(self)end
  end
  function R:launch_start(q)
    if D.mission_launch then D.mission_launch.begin(self,q)end
  end
  function R:draw()
    if self.staging and D.mission_warm then D.mission_warm.draw(self.g,self.staging);return end
    D.hud.draw(self.g,self.current)
    if self.world_generation then self.g.text(16,88,'Generating world - please wait')end
  end
  return R
end
