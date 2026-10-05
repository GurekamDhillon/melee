-- Console and hook glue. The generated main entry composes these factories.
return function(D)
  local C,S=D.companion,D.save
  local A={};A.__index=A
  function A.new(g,mission,rng)
    local a=setmetatable({g=g,mission=mission,store=S.open(g),last_effects=nil,visible=false},A)
    a.drives=D.drives.new(rng,g.log,g)
    a.run=D.run.new(g,mission,a.store.profile,function(p) return a.store:commit(p) end,
      function(n) return math.floor(D.genetics.random(rng or math.random)*n)+1 end)
    a.hub=D.hub and D.hub.new(g,mission);a.flashes={}
    a.menu=D.menu.new();a.input=D.menu_input.new(g)
    a.recolour=D.recolour.new(g,C.tuning);a.visual=D.visual.new(g,C.tuning)
    a.models=D.drive_models.new(g,g.log)
    g.command('envoy',function(arg) return a:command(arg or '') end,'menu | start | classic | adventure | campaign (parked) | retry | stop | status | menudump | dump | give <colour> <n> | reset-profile confirm')
    if a.store.error then g.log('envoy: '..a.store.error) end
    g.log('envoy: progression and garden loaded; Power cap +10% damage and knockback')
    return a
  end
  function A:companion()
    return self.run.companion or (self.run.profile and self.run.profile.companions[self.run.profile.active])
  end
  function A:effects()
    local c=self:companion();if not c then return end
    local e=C.effects(c)
    local summary=('power=%.3f speed=%.3f guard=%.3f shield=%.3f air=%.3f radius=%.2f drop=%.3f'):format(
      e.damage_dealt,e.speed,e.damage_taken,e.shield_max,e.air_speed,e.pickup_radius,e.drop_chance)
    if summary~=self.last_effects then
      self.last_effects=summary
      self.g.log('envoy: tuning '..summary)
    end
    if self.run.active then self.run.fighter:apply(c) end
  end
  function A:sample()
    local current=self.mission.current
    if not self.run.active or not current then return end
    for h,entry in pairs(current.run.state.tracked) do
      local at=self.g.enemy_state(h)
      if not at then at=current.doc.mission and current.doc.mission.enemies[entry.index] end
      self.drives:track(h,at)
    end
  end
  function A:defeated(e)
    if self.run.active then self.drives:defeat(e,self.run.companion) end
  end
  function A:growth(before)
    local c=self.run.companion;if not c then return end
    for _,k in ipairs(C.stats) do if c.stats[k].level>(before[k] or 0) then
      self.flashes[k]=C.tuning.levelup_frames
      self.g.log(('envoy: LEVEL UP %s %d -> %d'):format(C.tuning.stat_names[k],before[k] or 0,c.stats[k].level))
    end end
  end
  function A:levels()
    local before={};local c=self.run.companion
    if c then for _,k in ipairs(C.stats) do before[k]=c.stats[k].level end end;return before
  end
  function A:item_collect(e)
    if e.name~='drive' or not self.run.active then return end
    local before=self:levels();local p=self.drives:collect(e,self.run.companion)
    if p then
      self.run:pickup(p.colour);self:growth(before);self.visual:pickup(p.grade,p.colour);self:effects()
    end
  end
  function A:item_expire(e) self.drives:expire(e) end
  function A:mission_event(e)
    if e.type=='defeat' then self:defeated(e) end
  end
  function A:watch_waits()
    local r=self.run
    r:wait('staging',self.mission.staging~=nil or self.mission.pending~=nil or r.request~=nil)
    r:wait('retirement',self.mission.retiring~=nil)
    r:wait('recovery',self.mission.recovery~=nil)
    r:wait('native-cleanup',not r.active and (r.fighter.started or r.restore_damage or self.recolour.diagnostic=='cleanup pending' or next(self.drives.native)~=nil or #self.models.pending>0 or #self.models.releases>0) or false)
    r:wait('scene-ready',self.launching~=nil)
    r:wait('garden-ready',self.hub and self.hub.waiting or false)
    r:wait('mission-cleanup',self.mission.stopping~=nil)
  end
  function A:start_refused()
    if not self.run.last_start_error then return end
    self.notice=self.run.last_start_error;self.visible=true;self.menu:show('setup')
    self.drives:clear();self.models:clear();self.recolour:clear();self.visual:clear()
    if self.owns_pause then self.g.resume();self.owns_pause=nil end
    self:sync_pause()
  end
  function A:frame()
    self:watch_waits()
    if self.retire then self.retire() end
    if not self.run.active then
      if self.hub and self.hub.active then self.mission:frame();self.hub:frame(self:companion())
      elseif self.hub then self.hub:clear() end
      if not (self.hub and self.hub.active) and (self.mission.staging or self.mission.retiring or self.mission.stopping) then self.mission:frame() end
      self.run.fighter:clear();self.run:restore_start_damage();self.recolour:clear();self.visual:clear();self.models:clear();self.drives:clear();self:watch_waits();return
    end
    local before=self:levels();self.drives:set_depth(math.max(1,self.run.index))
    for k,n in pairs(self.flashes) do self.flashes[k]=n>1 and n-1 or nil end
    -- Feed before progression so collection on the terminal frame is retained.
    for _,pickup in ipairs(self.drives:tick(self.g.player(1),self.run.companion)) do
      self.run:pickup(pickup.colour);self.visual:pickup(pickup.grade,pickup.colour)
      self.models:collected(pickup.pickup)
    end
    self:growth(before);self.models:sync(self.drives.pickups)
    self:sample();self.mission:frame()
    local index=self.run.index;self.run:frame()
    if self.run.index~=index or not self.run.active then self.drives:clear();self.models:clear() end
    self:sample();self:effects()
    if self.run.interlude and self.menu.screen~='interlude' then self.menu:interlude(self.run.interlude);self:sync_pause() end
    if self.run.active then
      local _,why=self.recolour:tick(self.run.companion)
      if why and why~=self.tint_status then self.tint_status=why;self.g.log('envoy: recolour '..why) end
      self.visual:tick()
    else self.recolour:clear();self.visual:clear();if self.run.last_start_error then self:start_refused() else self.menu:results(self.run.results) end end
    self:watch_waits()
  end
  function A:draw()
    if self.mission.staging then self.mission:draw();return end
    if not self.visible and not self.run.active then return end
    if self.menu.screen=='garden' and self.hub and self.hub.active then self.hub:draw(self:companion(),self.notice);return end
    if self.g.kit and self.visible then D.menu_draw.draw(self.g,self.menu,self:context()) end
    if self.menu.screen~='playing' then return end
    self.mission:draw()
    local status=self.run.pending and 'SAVE PENDING: envoy stop' or self.run.active and
      (self.run.index==#self.run.levels and 'Boss: KO CPU, reach exit' or 'Collect drives, reach exit') or 'Ready: envoy start'
    if self.store.error then
      local a=self.g.safe_area();self.g.text(a.x+12,a.y+12,'Envoy profile refused; see console',0xF07474FF)
    end
    local glyphs={};for _,pickup in ipairs(self.drives.pickups) do
      if not self.models:visible(pickup) then glyphs[#glyphs+1]=pickup end
    end
    D.hud.draw(self.g,self:companion(),status,self.flashes,self.drives.juice.flash);D.hud.pickups(self.g,glyphs)
  end
  function A:stop(reason)
    local had=self.run.active or self.run.pending
    local ok,why=self.run:finish(reason or 'quit');self.drives:clear();self.models:clear()
    self.recolour:clear();self.visual:clear()
    if self.run.last_start_error then self:start_refused()
    elseif had then self.menu:results(self.run.results);if self.run.results and self.run.results.reincarnation then self.notice='Reincarnated: a new egg, inherited grades retained' end end
    self:watch_waits()
    return ok,why
  end
  function A:command(arg)
    local ok,result,detail=pcall(function()
      local w={};for v in arg:gmatch('%S+') do w[#w+1]=v end
      local op=w[1]
      if op=='menu' then
        assert(#w<=2,'usage: envoy menu [screen]')
        local screen=w[2] or (self.run.active and 'pause' or 'title')
        assert(({title=true,profile=true,hub=true,companion=true,records=true,pause=self.run.active})[screen],'invalid menu screen')
        local match=self.g.match();assert(not (match and match.netplay),'offline only')
        self.visible=true;self.menu:show(screen);return true
      end
      if op=='menudump' then
        assert(#w==1,'usage: envoy menudump')
        local m=self.menu;self.g.log(('envoy: menu screen=%s visible=%s focus=%d fighter=%s scroll=%d owns_pause=%s'):format(
          m.screen,tostring(self.visible),m.focus[m.screen] or 1,m.fighter,m.scroll,tostring(self.owns_pause==true)))
        for i,e in ipairs(m:entries(self:context())) do self.g.log(('envoy: entry=%d label=%s disabled=%s focused=%s'):format(
          i,e.label,tostring(e.disabled==true),tostring(i==(m.focus[m.screen] or 1)))) end
        return true
      end
      if op=='dump' then assert(#w==1,'usage: envoy dump');self:dump();return true end
      if op=='status' then
        assert(#w==1,'usage: envoy status')
        if self.store.error then self.g.log('envoy: '..self.store.error);return true end
        local c=self:companion();self.g.log(('envoy: status active=%s room=%d pending=%s next_run=%d white=%d'):format(
          tostring(self.run.active),self.run.active and self.run.index or 0,tostring(self.run.pending),self.run.profile.next_run,c.white_drives))
        for _,k in ipairs(C.stats) do local s=c.stats[k];self.g.log(('envoy: %s grade=%s points=%d level=%d'):format(C.tuning.stat_names[k],s.grade,s.points,s.level)) end
        self:effects();return true
      end
      local match=self.g.match();assert(not (match and match.netplay),'offline only')
      assert(not self.store.error,self.store.error)
      if op=='hub' then assert(#w==1,'usage: envoy hub');return self:enter_hub()
      elseif op=='retry' then assert(#w==1 and self.run.profile.last_seed~=nil,'no saved campaign to retry');return self:start_run(self.run.profile.last_seed)
      elseif op=='stop' then assert(#w==1,'usage: envoy stop');return self:stop('quit')
      elseif op=='start' then
        assert(#w==1,'usage: envoy start');assert(not self.mission.current,'mission already loaded; mission stop first')
        local started,why=self.run:start()
        if started then
          self.drives:clear();self.models:clear();self:sample();self:effects()
          self.menu:show('playing');self.visible=true;self:sync_pause()
          self.recolour:apply(self.run.companion)
        end
        return started,why
      elseif op=='give' then
        assert(#w==3 and w[3]:match('^%d+$'),'usage: envoy give <colour> <integer 0..1000>')
        local n=tonumber(w[3]);assert(n<=1000,'drive count 0..1000')
        assert(C.colours[w[2]] or w[2]=='white','unknown colour')
        assert(not self.run.pending,'settlement pending; envoy stop retries it')
        if self.run.active then C.feed(self.run.companion,w[2],w[2]=='white' and n or n*C.tuning.drive_points)
        else
          local p=S.decode(S.encode(self.run.profile));C.feed(p.companions[p.active],w[2],w[2]=='white' and n or n*C.tuning.drive_points)
          S.refresh_records(p);local wrote,why=self.store:commit(p);assert(wrote,why);self.run.profile=self.store.profile
        end
        self:effects();self.g.log('envoy: gave '..w[2]..' '..n);return true
      elseif op=='reset-profile' then
        assert(#w==2 and w[2]=='confirm','usage: envoy reset-profile confirm')
        assert(not self.run.active and not self.run.pending,'stop and settle the run first')
        local p=S.new_profile();local wrote,why=self.store:commit(p);assert(wrote,why)
        self.run.profile=self.store.profile;self.drives:clear();self.models:clear();self.last_effects=nil
        self.g.log('envoy: profile reset one-write');return true
      end
      error('usage: envoy menu|menudump|dump|start|stop|give <colour> <n>|status|reset-profile confirm')
    end)
    if not ok or not result then
      local why=ok and detail or result;self:start_refused();self.g.log('envoy: refused '..tostring(why));return false,why
    end
    return true
  end
  function A:context()
    local c=self:companion();local palette={}
    if c then for i,rgb in ipairs(self.recolour:preview(c)) do
      palette[i]={label=i==1 and 'Body' or 'Accent',rgba=rgb[1]*16777216+rgb[2]*65536+rgb[3]*256+255}
    end end
    local fighters={};for name in ('fox falco mario luigi drmario peach bowser donkey captain ganondorf link younglink zelda sheik samus yoshi kirby pikachu pichu jigglypuff mewtwo ness marth roy iceclimbers gamewatch'):gmatch('%S+') do fighters[#fighters+1]=name end
    local p=self.run.profile;local records={}
    if p then
      local history=p.records.unknown_runs>0 and (' (older '..p.records.unknown_runs..' runs unknown)') or ''
      local best=p.records.best_frames>0 and ('%.2f s'):format(p.records.best_frames/60) or 'None'
      records={'Runs started: '..p.records.runs,'Companions raised: '..p.records.companions_raised,'Highest grade: '..D.genetics.grades[p.records.highest_grade],'Settled runs: '..p.last_settled,'Wins: '..p.records.wins..history,'Best time: '..best..history,'Last result: '..p.last_result,'Next run: '..p.next_run,'Companions: '..#p.companions}
      if c then
        records[#records+1]='White drives found this life: '..c.white_drives
        for _,name in ipairs(C.stats) do local s=c.stats[name];records[#records+1]=('%s: %s ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â· level %d ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â· %d points'):format(C.tuning.stat_names[name],s.grade,s.level,s.points) end
      end
    end
    return {companion=c,palette=palette,fighters=fighters,error=self.store.error,records=records,notice=self.notice,hud='START: pause'}
  end
  function A:sync_pause()
    local open=not self.run.last_start_error and self.visible~=false and self.menu.screen~='playing' and self.menu.screen~='garden' and not self.launching and not self.mission.staging and not self.mission.stopping
    self.input:set_active(open)
    local match=self.g.match()
    if open and match and match.active and not match.netplay and self.g.paused and not self.g.paused() then
      self.g.pause();self.owns_pause=true
    elseif not open and self.owns_pause then self.g.resume();self.owns_pause=nil end
  end
  function A:start_run(seed)
    assert(not self.mission.current,'mission already loaded; leave garden first')
    local ok,why=self.run:start(seed)
    if ok then self.drives:clear();self.models:clear();self:sample();self:effects();self.menu:show('playing');self.visible=true;self:sync_pause();self.recolour:apply(self.run.companion) end
    if not ok then self:start_refused() end
    return ok,why
  end
  function A:enter_hub()
    if self.run.active or self.run.pending then return false,'settle run before garden' end
    if not self.hub then return false,'garden module unavailable' end
    local m=self.g.match()
    if not m or not m.active then return self:launch('hub',self.menu.fighter) end
    if self.mission.current and self.mission.current.doc.name~='hub' then return false,'mission cleanup pending' end
    local ok,why=self.hub:enter()
    if ok then self.run.last_start_error=nil;self.menu:show('garden');self.visible=true;self:sync_pause() end
    return ok,why
  end
  function A:launch(kind,fighter)
    if self.hub and self.hub.active then self.hub:clear();self.mission:stop('garden exit') end
    if self.mission.stopping or self.mission.staging or self.mission.recovery then
      self.g.log('envoy: launch waiting for mission cleanup');return false,'mission cleanup pending'
    end
    if not self.g.scene_launch then
      if kind=='hub' then return false,'engine request: scene_launch' end
      return self:command('start')
    end
    self.run:wait('scene-ready',true);self.launch_kind=kind;self.launching=fighter
    local ok,why=self.g.scene_launch{mode='lab',p1=fighter,p2='falco/cpu0',stage='fd'}
    if not ok then self.launching=nil;self.launch_kind=nil;self.menu:show('setup');self.g.log('envoy: launch refused '..tostring(why)) end
    self:sync_pause();return ok,why
  end
  function A:menu_effect(e)
    if not e then self:sync_pause();return end
    if e.type=='start' then self:launch('run',e.fighter)
    elseif e.type=='hub' then local ok,why=self:enter_hub();if not ok then self.g.log('envoy: garden refused '..tostring(why)) end
    elseif e.type=='continue' then local ok,why=self.run:continue();if ok then self.menu:show('playing') else self.g.log('envoy: continue refused '..tostring(why));self.menu:results(self.run.results) end
    elseif e.type=='abandon' then self:stop('quit')
    elseif e.type=='quit' then
      self.visible=false;if self.hub and self.hub.active then self.hub:clear();self.mission:stop('garden closed') end
      if self.run.active then self.menu:show('playing') end
    end
    self:sync_pause()
  end
  function A:tick()
    self:watch_waits()
    local match=self.g.match()
    if match and match.netplay then self.input:close();return end
    if self.launch_ready then
      self.launch_frames=(self.launch_frames or 0)+1
      local p1,p2=self.g.player(1),self.g.player(2)
      if D.run.ready(p1) and D.run.ready(p2) and p2.cpu then
        local kind=self.launch_kind;self.launch_kind=nil
        self.launching=nil;self.launch_ready=nil;self.launch_frames=nil
        if kind=='hub' then self:enter_hub() elseif not self:command('start') then self.menu:show('setup') end
      elseif self.launch_frames>=600 then
        self.launching=nil;self.launch_ready=nil;self.launch_frames=nil
        self.menu:show('setup');self.g.log('envoy: launch fighter readiness timeout')
      end
    end
    self:watch_waits();self:sync_pause()
    local actions=self.input:poll()
    if self.visible==false then return end
    if self.hub and self.hub.active and self.menu.screen=='garden' then
      local station=self.hub:activate((self.g.pad(1,true) or {}).A)
      if station then if station.closed then self.notice='Nest opens in slice 5' else self.menu.return_to='garden';self.menu:show(station.screen) end end
      for _,action in ipairs(actions) do if action=='start' then self.menu:show('hub') end end
      self:sync_pause();return
    end
    for _,action in ipairs(actions) do if not (action=='start' and self.results_up and self.menu.screen=='playing') then self:menu_effect(self.menu:input(action,self:context())) end end
  end
  function A:match_start()
    self.models:unload()
    if self.launching then self.launch_ready=true;self.launch_frames=0 end
  end
  function A:match_end()
    if self.hub then self.hub:clear() end
    self.mission:stop('match end') -- discard old scene ownership even during relaunch
    if not self.launching then self:stop('fail') end
    self.models:unload()
  end
  function A:unload()
    self:stop('quit');if self.hub then self.hub:clear() end
    -- No future frame can advance a rollback after the script unloads.
    local ok,why=pcall(self.mission.stop,self.mission,'unload')
    if not ok then self.g.log('envoy: unload cleanup refused '..tostring(why)) end
    self.run:restore_start_damage();self.models:unload();self.input:close()
    if self.detach_events then self.detach_events();self.detach_events=nil end
    self.run:clear_waits()
    if self.owns_pause then self.g.resume();self.owns_pause=nil end
  end
  function A:dump()
    local r=self.run;local c=self:companion();local f=r.fighter
    self.g.log(('envoy: run active=%s room=%d pending=%s boss=%s reserve_owned=%s calling=%s profile_error=%s'):format(
      tostring(r.active),r.active and r.index or 0,tostring(r.pending),tostring(f.boss),tostring(f.owned),tostring(r.calling~=nil),tostring(self.store.error)))
    self.g.log('envoy: modifiers applied='..tostring(f.modified)..' ratios='..tostring(f.key)..' tint='..tostring(self.recolour.diagnostic or 'inactive')..' tint_active='..tostring(self.recolour.active==true))
    if c then
      self.g.log(('envoy: companion id=%d age=%d type=%s colour=%s two_tone=%s shiny=%s white=%d'):format(c.id,c.age,c.type,c.colour,tostring(c.two_tone),tostring(c.shiny),c.white_drives))
      for _,name in ipairs(C.stats) do local s=c.stats[name];self.g.log(('envoy: %s grade=%s level=%d points=%d'):format(C.tuning.stat_names[name],s.grade,s.level,s.points)) end
      for _,name in ipairs(D.genetics.traits) do self.g.log('envoy: DNA '..name..'='..tostring(c.dna[name][1])..'/'..tostring(c.dna[name][2])) end
    end
    for i,v in ipairs(self.drives.pickups) do self.g.log(('envoy: drive=%d colour=%s x=%.2f y=%.2f left=%d'):format(i,v.colour,v.x,v.y,v.left)) end
  end
  if D.retail_app then D.retail_app.extend(A) end
  return A
end
