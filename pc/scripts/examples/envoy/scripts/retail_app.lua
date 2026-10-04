-- Retail integration wraps the parked mission app without extending its runtime.
return function(D)
 local C=D.companion;local M={}
 function M.extend(A)
  local old={};for _,k in ipairs({'new','companion','context','command','frame','tick','draw','stop','launch','menu_effect','sync_pause','match_start','match_end','unload','enter_hub','item_collect','item_expire'}) do old[k]=A[k] end
  function A.new(...)
   local a=old.new(...);a.retail=D.classic.new(a.g,a.run.profile,function(p) return a.store:commit(p) end)
   a.menu.run_type=C.tuning.retail.default_mode;a.menu.difficulty=C.tuning.retail.difficulty;a.menu.stocks=C.tuning.retail.stocks
   a.g.log('envoy: default retail Classic / Adventure + NG+; campaigns parked');return a
  end
  function A:companion()
   return self.retail and (self.retail.active or self.retail.pending) and self.retail.companion or old.companion(self)
  end
  function A:context()
   local c=old.context(self);c.reward=self.retail and self.retail.reward;c.retail_pending=self.retail and self.retail.pending
   c.retail_menu=self.menu.run_type~='campaign';c.garden_available=self.garden_available==true
   if self.retail and self.retail.active then c.hud=self.retail.mode..' / NG+'..self.retail.loop..' / START: pause' end
   return c
  end
  function A:start_retail(mode,fighter,difficulty,stocks,seed)
   if self.run.active or self.run.pending then return false,'settle parked campaign first' end
   local ok,why=self.retail:available()
   if not ok then self.notice=why;self.visible=true;self.menu:show('setup');self.g.log('envoy: '..why);return false,why end
   if self.hub and self.hub.active then self.hub:clear();self.mission:stop('retail run launch') end
   if self.mission.stopping or self.mission.staging or self.mission.retiring or self.mission.recovery then
    self.retail_request={mode=mode,fighter=fighter,difficulty=difficulty,stocks=stocks,seed=seed,ticks=0}
    self.notice='Waiting for garden cleanup';return true
   end
   self.retail.profile=self.run.profile
   ok,why=self.retail:start(mode or self.menu.run_type,fighter or self.menu.fighter,difficulty or self.menu.difficulty,stocks or self.menu.stocks,seed)
   if ok then self.run.profile=self.retail.profile;self.visible=true;self.notice=nil;self.menu:show('playing')
   else self.notice=why;self.visible=true;self.menu:show('setup');self.g.log('envoy: retail refused '..tostring(why)) end
   self:sync_pause();return ok,why
  end
  function A:command(arg)
   if arg=='start' then
    if self.menu.run_type=='campaign' then return old.command(self,'start') end
    return self:start_retail()
   end
   if arg=='classic' or arg=='adventure' then self.menu.run_type=arg;return self:start_retail(arg) end
   if arg=='retry' then return self:start_retail(nil,nil,nil,nil,self.run.profile and self.run.profile.last_seed) end
   if arg=='campaign' then
    if self.retail.active or self.retail.pending or self.retail_request then return false,'retail run active' end
    self.menu.run_type='campaign'
    return old.command(self,'start')
   end
   if arg=='stop' and (self.retail.active or self.retail.pending or self.retail_request) then return self:stop('quit') end
   if self.retail.active or self.retail.pending then
    if arg:match('^give ') or arg:match('^reset%-profile') then return false,'finish retail run before debug profile edits' end
    if arg=='status' then self.g.log('envoy: '..self.retail.mode..' NG+'..self.retail.loop..' seed='..self.retail.seed);return true end
    if arg=='menu' or arg=='menu pause' then self.visible=true;self.menu:show('pause');self:sync_pause();return true end
   end
   return old.command(self,arg)
  end
  function A:retail_event(name,e)
   local r=self.retail;if not r or not r.active then return end
   r[name](r,e or {})
   self.run.profile=r.profile
   if r.reward then self.menu:show('reward');self.menu.focus.reward=1;self.visible=true
   elseif not r.active and r.results then self.menu:results(r.results);self.visible=true;self.retail_return=true end
   if name=='stage_start' and r.active then self.recolour:apply(r.companion) end
   self:sync_pause()
  end
  function A:sync_pause()
   if self.retail and self.retail.reward then
    self.input:set_active(true)
    if self.owns_pause then self.g.resume();self.owns_pause=nil end
    return
   end
   if self.retail_request then self.input:set_active(false);if self.owns_pause then self.g.resume();self.owns_pause=nil end;return end
   return old.sync_pause(self)
  end
  function A:menu_effect(e)
   if e and e.type=='start' then
    if self.menu.run_type=='campaign' then return old.menu_effect(self,e) end
    return self:start_retail(e.mode,e.fighter,e.difficulty,e.stocks)
   end
   if e and e.type=='reward' then
    local ok,why=self.retail:pick(e.index)
    if not ok then self.notice=why else
     self.menu.focus.reward=1;local r=self.retail.reward;local choice=r.options[e.index]
     local stat=C.colours[choice.colour];local grade=stat and r.after[stat].grade or 'C'
     local p=self.g.player and self.g.player(1) or {};local juice=self.drives.juice
     -- Existing presentation only: this token never enters an item/reward ledger.
     local id={};juice.drops[id]={colour=choice.colour,x=p.x or 0,y=p.y or 0,z=0,fx={}}
     juice:collect(id,choice.colour,{x=p.x or 0,y=p.y or 0});self.visual:pickup(grade,choice.colour)
     for k,n in pairs(r.levelups) do
      self.flashes[k]=C.tuning.levelup_frames
      self.g.log(('envoy: LEVEL UP %s %d -> %d'):format(C.tuning.stat_names[k],r.before[k].level,r.after[k].level))
     end
    end;self.run.profile=self.retail.profile;return
   end
   if e and e.type=='reward_done' then
    if self.retail:acknowledge() then
     if not self.retail.active and self.retail.results then self.menu:results(self.retail.results);self.retail_return=true else self.menu:show('playing') end
     self.run.profile=self.retail.profile;self:sync_pause()
    end;return
   end
   if e and e.type=='retail_retry' then self.retail:retry_save();self.run.profile=self.retail.profile;self:sync_pause();return end
   if e and e.type=='abandon' and self.retail.active then return self:stop('quit') end
   if e and e.type=='quit' and self.retail.active then
    self.visible=false;self.menu:show('playing');self:sync_pause();return
   end
   return old.menu_effect(self,e)
  end
  function A:launch(kind,fighter)
   if kind=='run' and self.menu.run_type~='campaign' then return self:start_retail(nil,fighter) end
   return old.launch(self,kind,fighter)
  end
  function A:frame()
   if self.retail.active or self.retail.pending then
    self.retail:frame();if self.retail.active then self.recolour:tick(self.retail.companion) end;return
   end
   old.frame(self)
   if self.hub and self.hub.refused then
    self.hub:clear();self.mission:stop('garden unavailable');self.garden_available=false
    self.menu:show('hub');self.visible=true;self.notice=nil;self:sync_pause()
   end
  end
  function A:tick()
   if self.retail_request then
    local q=self.retail_request;q.ticks=q.ticks+1
    if not (self.mission.stopping or self.mission.staging or self.mission.retiring or self.mission.recovery) then
     self.retail_request=nil;self:start_retail(q.mode,q.fighter,q.difficulty,q.stocks,q.seed)
    elseif q.ticks>=600 then self.retail_request=nil;self.notice='Garden cleanup timed out';self.menu:show('setup') end
   end
   if self.retail.active or self.retail.pending then
    self.visual:tick();self.drives.juice:tick()
    for k,n in pairs(self.flashes) do self.flashes[k]=n>1 and n-1 or nil end
    local m=self.g.match();if m and m.netplay then self.input:close();return end
    self.retail:tick()
    self.run.profile=self.retail.profile
    if not self.retail.active and self.retail.results then self.menu:results(self.retail.results);self.retail_return=true
    elseif self.menu.screen=='reward' and not self.retail.reward then self.menu:show('playing') end
    self:sync_pause();local actions=self.input:poll()
    if self.visible then for _,action in ipairs(actions) do self:menu_effect(self.menu:input(action,self:context())) end end
    return
   end
   self.retail:tick();return old.tick(self)
  end
  function A:draw()
   if self.retail.active or self.retail.pending then
    local r=self.retail
    D.hud.draw(self.g,r.companion,r.mode..' NG+'..r.loop,self.flashes,self.drives.juice.flash,r.reward)
    if self.visible and self.menu.screen~='playing' and self.g.kit then D.menu_draw.draw(self.g,self.menu,self:context()) end
    if r.tag_ticks and r.tag_ticks>0 and self.g.kit then
     local a=self.g.safe_area();self.g.kit.panel(a.x+12,a.y+12,300,35)
     local labels={};for _,tag in ipairs(r.tags or {}) do labels[#labels+1]=tag.label end
     self.g.kit.text(a.x+22,a.y+35,#labels>0 and table.concat(labels,' / ') or 'BONUS / your companion stats are active','body','bone','left',{max_w=280})
     if self.g.project and self.g.player then
      for _,tag in ipairs(r.tags or {}) do
       local p=self.g.player(tag.port);if p then
        local x,y,visible=self.g.project(p.x,p.y+18,0)
        if x and visible then self.g.kit.text(x-24,y,tag.label,'body',tag.colour,'left',{max_w=160}) end
       end
      end
     end
    end
    if (r.guard_flash or 0)>0 and self.g.kit and self.g.project and self.g.player then
     local p=self.g.player(r.state and r.state.player_port or 1)
     if p then local x,y,visible=self.g.project(p.x,p.y+24,0)
      if x and visible then self.g.kit.panel(x-35,y-16,70,24,{fill=0x79AAF0C0});self.g.kit.text(x-30,y,'GUARD','body','bone','left',{max_w=60}) end
     end
    end
    return
   end
   return old.draw(self)
  end
  function A:stop(reason)
   self.retail_request=nil
   if self.retail.active or self.retail.pending then
    local ok,why=self.retail:finish(reason or 'quit');self.run.profile=self.retail.profile
    self.drives:clear();self.models:clear();self.recolour:clear();self.visual:clear()
    self.menu:results(self.retail.results);self.visible=true;self.retail_return=true;self:sync_pause();return ok,why
   end
   return old.stop(self,reason)
  end
  function A:match_start() if self.retail.active or self.retail.pending then self.models:unload();return end;return old.match_start(self) end
  function A:match_end() if self.retail.active or self.retail.pending then self.models:unload();self.recolour:clear();return end;return old.match_end(self) end
  function A:unload() self:stop('quit');return old.unload(self) end
  function A:enter_hub()
   if self.retail.active or self.retail.pending then return false,'settle retail run before garden' end
   self.garden_available=self.hub and self.hub:resolve() or false
   if self.retail_return or not self.garden_available then
    self.retail_return=nil;self.menu:show('hub');self.visible=true;self.notice=nil;self:sync_pause();return true
   end
   return old.enter_hub(self)
  end
  function A:item_collect(e)
   if self.retail.active then return self.retail:item_collect(e) end
   return old.item_collect(self,e)
  end
  function A:item_expire(e)
   if self.retail.items[e.item] then self.retail.items[e.item]=nil;return end
   if self.retail.active then return end
   return old.item_expire(self,e)
  end
 end
 return M
end
