-- Retail integration wraps the parked mission app without extending its runtime.
return function(D)
 local C=D.companion;local M={}
 function M.extend(A)
  local old={};for _,k in ipairs({'new','companion','context','command','frame','tick','draw','stop','launch','menu_effect','sync_pause','match_start','match_end','unload','enter_hub','item_collect','item_expire'}) do old[k]=A[k] end
  function A.new(...)
   local a=old.new(...);a.retail=D.classic.new(a.g,a.run.profile,function(p) return a.store:commit(p) end)
   a.menu.run_type=C.tuning.retail.default_mode;a.menu.difficulty=C.tuning.retail.difficulty;a.menu.stocks=C.tuning.retail.stocks
   -- Atlas step 3: the legacy menu shows its screens by name; setup, pause and results open their Atlas descriptions when Atlas is on
   a.menu.on_show=function(_,screen) if D.atlas_kit then D.atlas_kit.menu_show(a,screen) end end
   a.g.log('envoy: default retail Classic / Adventure + NG+; campaigns parked');return a
  end
  -- ---- co-op (offline, two local players; coop.lua). Off unless asked for: `envoy coop [fighter1] [fighter2] [p1=cpu] [p2=cpu] [seed=N] [loops=N]`.
  function A:coop_ready() return self.coop~=nil and select(1,self.coop:available()) end
  function A:start_coop(opts)
   if not self.coop then return false,'co-op unavailable' end
   if self.retail.active or self.retail.pending or self.run.active or self.run.pending then return false,'finish the other run first' end
   if self.hub and self.hub.active then self.hub:clear();self.mission:stop('co-op run launch') end
   local ok,why=self.coop:start(opts)
   if ok then self.visible=false;self.input:close();self.menu:show('playing') else self.notice=why;self.visible=true;self.menu:show('setup');self.g.log('envoy: co-op refused '..tostring(why)) end
   return ok,why
  end
  function A:coop_command(arg)
   local w={};for x in arg:gmatch('%S+') do w[#w+1]=x end;table.remove(w,1)
   local C=D.coop
   if w[1]=='stop' then return self.coop and self.coop:stop() end
   if w[1]=='tuning' then
    if w[2]=='reset' then C.reset_tuning() end
    if w[2] and w[3] then local ok,err=pcall(C.set,w[2],w[3]);if not ok then self.g.log('envoy coop: '..tostring(err));return false,tostring(err) end end
    for _,l in ipairs(C.lines()) do self.g.log('envoy coop: '..l) end;return true
   end
   if w[1]=='ux' then for i,h in ipairs(self.coop.hosts) do self.g.log(('envoy coop ux: seat %d (port %d) screen active=%s mode=%s owner=%s (coop owner is %s, state %s) strips drawn while a screen is up: %s'):format(i,h:port0(),tostring(h.screen.active),tostring(h.screen.mode),tostring(self.coop.screen_owner==h),tostring(self.coop.screen_owner and self.coop.screen_owner:port0()),tostring(self.coop.state),tostring(self.coop.draws_other_during_screen or 0)));h:dump() end;return true end
   if w[1]=='rows' then -- one line per stage (the log may drop lines under load; this reads the run's own memory): `envoy coop rows`
    local c=self.coop;local fin=c.final
    for i,r in ipairs(c.results or {}) do
     local d=r.detail or {};local function list(t) local o={};for k,v in ipairs(t or {}) do o[k]=('%.2f'):format(v) end;return table.concat(o,',') end
     self.g.log(('coop row %d: loop=%d stage=%d result=%s frames=%d foes=%d team=%.3f name=%s builds=%s foestr=%s dealt=%.0f/%.0f taken=%.0f/%.0f gains=%s cost=%.3f/%.3f/%.3f'):format(i,r.loop,r.stage,r.result,r.frames,r.foes,r.team_strength or 0,tostring(r.stage_name),list(d.builds),list(d.foe),r.dealt[1],r.dealt[2],r.taken[1],r.taken[2],table.concat(d.gains or {},'/'),(d.cost or {})[1] or 0,(d.cost or {})[2] or 0,(d.cost or {})[3] or 0))
    end
    self.g.log(('coop final: active=%s reason=%s stages=%s loops=%s digest=%s'):format(tostring(c.active),fin and fin.reason or '-',fin and fin.stages or '-',fin and fin.loops or '-',fin and fin.digest or '-'));return true
   end
   if w[1]=='record' then local text,digest=self.coop:record();for l in text:gmatch('[^\n]+') do self.g.log('envoy coop record: '..l) end;self.g.log('envoy coop record: digest '..digest);return true end
   if w[1]=='status' then local c=self.coop;self.g.log(('envoy coop: active=%s state=%s stage=%s loop=%s seed=%s'):format(tostring(c.active),c.state,tostring(c.stage),tostring(c.loop),tostring(c.seed)));return true end
   local opts={};local fighters={}
   for _,tok in ipairs(w) do
    local k,v=tok:match('^(%a+%d?)=(.+)$')
    if k=='p1' or k=='p2' then if v~='cpu' and v~='human' then return false,k..' is cpu or human' end;opts[k]=v
    elseif k=='seed' then opts.seed=tonumber(v) elseif k=='loops' then opts.max_loops=tonumber(v)
    elseif k then return false,'unknown option '..k
    else local id,why=D.fighters.resolve(self.g,tok);if not id then self.notice=why;self.g.log('envoy: '..why);return false,why end;fighters[#fighters+1]=id end
   end
   opts.f1=fighters[1] or self.menu.fighter or 'fox';opts.f2=fighters[2] or (opts.f1=='marth' and 'fox' or 'marth')
   return self:start_coop(opts)
  end
  function A:companion()
   return self.retail and (self.retail.active or self.retail.pending) and self.retail.companion or old.companion(self)
  end
  function A:context()
   local c=old.context(self);c.reward=self.retail and self.retail.reward;c.retail_pending=self.retail and self.retail.pending
   c.retail_menu=self.menu.run_type~='campaign';c.coop_available=self:coop_ready()==true;c.garden_available=self.garden_available==true
   if self.retail and self.retail.active then c.hud=self.retail.mode..' / NG+'..self.retail.loop..' / START: pause' end
   return c
  end
  -- A run is launched by a scene reset. Issued in the opening movie / attract demo, or in the first frames of any scene, the reset can run while
  -- that scene still has DVD/stream reads outstanding and the game's reset spin never ends (the 2026-10-05 menu hang: a run started in the attract
  -- demo, whose own end the same START press caused). So a start waits until the scene is a settled one, says so, and goes ahead by itself.
  A.SCENE_SETTLE=90 -- logic frames a scene must have been up before a launch is issued from it
  A.TITLE_LAST=420 -- the title screen lasts about 620 frames before the demo starts
  A.SCENE_WAIT_MAX=18000 -- ticks to wait before giving up (the attract loop comes round to a usable title in a minute or two)
  function A:scene_watch()
   local g=self.g;local s=g.scene and g.scene();if type(s)~='table' then return end
   if s.epoch~=self.scene_epoch then self.scene_epoch=s.epoch;self.scene_since=g.frame and g.frame() or 0 end
  end
  function A:scene_safe()
   local g=self.g;local s=g.scene and g.scene();if type(s)~='table' then return true end
   self:scene_watch()
   -- the attract loop (opening movie, title, demo) ends its own scenes: a launch issued in the movie or the demo, or as the title is about to
   -- give way to the demo, lands in the first frame of the next scene
   if s.name=='GS_MOVIE_OPENING' or (s.name=='GS_VS' and s.mode_name=='GM_OPENING_MV') then return false,'the opening movie / demo is playing' end
   local f=g.frame and g.frame() or 0;local age=f-(self.scene_since or f)
   if age<A.SCENE_SETTLE then return false,'the screen has only just changed' end
   if s.name=='GS_TITLE' and age>A.TITLE_LAST then return false,'the title screen is about to give way to the demo' end
   return true
  end
  function A:start_retail(mode,fighter,difficulty,stocks,seed)
   if self.run.active or self.run.pending then return false,'settle parked campaign first' end
   do local safe,why=self:scene_safe()
    if not safe then
     if not self.retail_request then self.retail_request={mode=mode,fighter=fighter,difficulty=difficulty,stocks=stocks,seed=seed,ticks=0} end
     self.retail_request.scene_wait=true;self.notice='The run starts when the screen settles'
     self.g.log('envoy: run start deferred: '..tostring(why)..'; it starts once the screen has settled');return true
    end
   end
   local ok,why=self.retail:available()
   if not ok then self.notice=why;self.visible=true;self.menu:show('setup');self.g.log('envoy: '..why);return false,why end
   if self.hub and self.hub.active then self.hub:clear();self.mission:stop('retail run launch') end
   if self.mission.stopping or self.mission.staging or self.mission.retiring or self.mission.recovery then
    self.retail_request={mode=mode,fighter=fighter,difficulty=difficulty,stocks=stocks,seed=seed,ticks=0}
    self.notice='Waiting for garden cleanup';return true
   end
   self.retail.profile=self.run.profile
   if self.dev_spec and self.retail.host then self.retail.host:dev_start(self.dev_spec) end;self.dev_spec=nil
   ok,why=self.retail:start(mode or self.menu.run_type,fighter or self.menu.fighter,difficulty or self.menu.difficulty,stocks or self.menu.stocks,seed)
   if ok then self.run.profile=self.retail.profile;self.visible=true;self.notice=nil;self.menu:show('playing');if D.atlas_pause then D.atlas_pause.ensure(self) end   -- the pause screen the retail pause takeover may push
   else self.notice=why;self.visible=true;self.menu:show('setup');self.g.log('envoy: retail refused '..tostring(why)) end
   self:sync_pause();return ok,why
  end
  function A:command(arg)
   -- The rule host switch: Classic / Adventure runs install the pool, bag, slots, opponent rolls and looks
   -- instead of the companion-stat templates. Takes effect on the next run; the rule host is the DEFAULT (since the readability split);
   -- `envoy rules off` keeps the older companion-stat route reachable.
   if arg=='devui' or arg=='devui on' or arg=='devui off' then
    if arg~='devui' then D.mod_tuning.set_dev_ui(arg=='devui on');if self.retail.host then self.retail.host.hud.m=nil end end
    self.g.log('envoy: developer overlay is '..(D.mod_tuning.dev_ui() and 'ON' or 'off'));return true
   end
   if arg=='rules' or arg=='rules on' or arg=='rules off' then
    if arg~='rules' then
     if self.retail.active or self.retail.pending then return false,'finish the retail run before switching the rule host' end
     self.retail.rules=arg=='rules on'
    end
    self.g.log('envoy: rule host for retail runs is '..(self.retail.rules and 'ON' or 'off (companion stats)'));return true
   end
   -- `envoy grant <keystone id>`: take that keystone at once (a try-it command). In a rule-host run it replaces the held keystone when
   -- the allowance is one; with a bigger allowance it adds it (exclusions and drawback floors still refuse an illegal build).
   if arg:match('^grant ') then
    local id=arg:match('^grant%s+([a-z_]+)%s*$');if not id then return false,'usage: envoy grant <keystone id>' end
    local host=self.retail.host
    if self.retail.active and self.retail.rules and host and host.running then return host:grant(id) end
    return self.mods_command and self.mods_command('add '..id) or false,'start a rules-on run first (envoy rules on, envoy classic)'
   end
   -- `envoy tuning` lists the named tuning values (current column, proposed column); `envoy tuning proposed|current` switches the whole preset for the run;
   -- `envoy tuning <name> <value>` sets one; `envoy strength v1|v2` picks the strength formula. Values are rule data: they apply on every peer alike.
   if arg=='tuning' or arg:match('^tuning ') or arg:match('^strength ') then
    local T=D.mod_tuning;local w={};for x in arg:gmatch('%S+') do w[#w+1]=x end
    if w[1]=='strength' then local ok,err=pcall(T.set,'strength',w[2]);if not ok then return false,'usage: envoy strength v1|v2' end
    elseif w[2]=='proposed' or w[2]=='current' then T.preset(w[2])
    elseif w[2] and w[3] then local ok,err=pcall(T.set,w[2],w[3]);if not ok then self.g.log('envoy tuning: '..tostring(err));return false,tostring(err) end end
    self.g.log('envoy tuning: preset '..T.preset_name)
    for _,l in ipairs(T.lines()) do self.g.log('envoy tuning: '..l) end
    if self.retail.host then self.retail.host:touch_tuning() end
    return true
   end
   if arg=='coop' or arg:match('^coop%s') then return self:coop_command(arg) end
   -- `envoy start <classic|adventure> <fighter> [depth=<n>] [loop=<n>] [build=<seed|current|proposed>]`: a developer start at a chosen depth with a
   -- consistent build (rule host forced on). depth/loop set a floor on the run's progression context (slots, keystone allowance, tier, opponents);
   -- build=<seed> rolls the drives for every slot and the keystones up to the allowance at that depth (build=current|proposed also sets the tuning preset).
   if arg:match('^start%s') then
    local w={};for x in arg:gmatch('%S+') do w[#w+1]=x end
    local usage='usage: envoy start <classic|adventure> <fighter> [depth=<n>] [loop=<n>] [build=<seed|current|proposed>]'
    local mode,token=w[2],w[3];if mode~='classic' and mode~='adventure' then return false,usage end
    local spec={depth=0,loop=0}
    for i=4,#w do
     local k,v=w[i]:match('^(%a+)=(%S+)$');if not k then return false,usage end
     if k=='depth' or k=='loop' then local n=tonumber(v);if not n or n<0 or n%1~=0 then return false,k..' must be a nonnegative integer' end;spec[k]=n
     elseif k=='build' then
      if v=='current' or v=='proposed' then spec.preset=v;spec.build=true
      else local n=tonumber(v);if not n or n<1 or n%1~=0 or n>2147483646 then return false,'build must be a seed (integer >= 1), current or proposed' end;spec.build=true;spec.build_seed=n end
     else return false,usage end
    end
    if spec.depth>0 or spec.loop>0 then spec.build=spec.build or true end
    if self.retail.active or self.retail.pending or self.retail_request then return false,'finish the retail run first' end
    local id,why;if token then id,why=D.fighters.resolve(self.g,token);if not id then self.notice=why;self.g.log('envoy: '..tostring(why));return false,why end end
    self.retail.rules=true;self.dev_spec=spec;self.menu.run_type=mode;if id then self.menu.fighter=id end
    self.g.log(('envoy: developer start %s depth=%d loop=%d build=%s'):format(mode,spec.depth,spec.loop,spec.build and (spec.preset or spec.build_seed or 'run seed') or 'none'))
    local ok,err=self:start_retail(mode,id)
    if ok==false then self.dev_spec=nil end
    return ok,err
   end
   if arg=='start' then
    if self.menu.run_type=='campaign' then return old.command(self,'start') end
    if self.menu.run_type=='coop' then return self:start_coop({f1=self.menu.fighter}) end
    return self:start_retail()
   end
   do local mode,token=arg:match('^(%a+)%s+(%S+)$')
    if (mode=='classic' or mode=='adventure') and token then
     local id,why=D.fighters.resolve(self.g,token)
     if not id then self.notice=why;self.g.log('envoy: '..why);return false,why end
     self.menu.run_type=mode;self.menu.fighter=id;return self:start_retail(mode,id)
    end
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
   self.g.log(('envoy: retail event %s active=%s reward=%s results_up(before)=%s'):format(name,tostring(r.active),tostring(r.reward~=nil),tostring(self.results_up)))
   if name=='stage_start' or not r.active then self.results_up=nil end
   -- The retail results/clear screens belong to the game: while one is up (from the clear to the next stage's start) Envoy's own
   -- START menu does not open, so one START advances them. A stage start also needs a FRESH press: a START that was already held
   -- at the scene change (an Adventure intro skip) must not open the menu.
   if (name=='stage_clear' or name=='boss_defeated' or name=='complete' or name=='game_over') and r.active and not r.reward then self.results_up=true end
   if name=='stage_start' then self.start_ready=false;self.seen_foe=false end
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
   if e and e.type=='start' and e.mode=='coop' then return self:start_coop({f1=e.fighter,f2=(e.fighter=='marth') and 'fox' or 'marth'}) end
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
  -- The engine delivers no event when a retail stage is decided (on_1p_stage_clear runs when the clear is ACCEPTED, i.e. after the
  -- press that leaves the results screen, and on_match_end only at the next scene). So the results screen is read from game state:
  -- an opponent was seen this stage and none has a stock left, or P1 has none, or the engine holds the clear (mode_1p().held).
  function A:stage_decided()
   local g=self.g;if not g.player then return false end
   local p1=g.player(1);if p1 and p1.stocks==0 then return true end
   local alive=false
   for p=2,6 do local v=g.player(p);if v and type(v.stocks)=='number' and v.stocks>0 then alive=true;self.seen_foe=true end end
   if self.seen_foe and not alive then return true end
   local m=g.mode_1p and g.mode_1p();return type(m)=='table' and m.held==true
  end
  function A:frame()
   if self.coop and self.coop.active then return end   -- a co-op run is driven by coop.lua; the app's own menus and garden stay out of it
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
   if self.menu_held and not self.visible then   -- the legacy menu closed: the native menu under it takes input again
    if self.g.ui and self.g.ui.hold_menu then pcall(self.g.ui.hold_menu,false) end;self.menu_held=nil
   end
   self:scene_watch()
   if self.coop and self.coop.active then return end
   if self.retail_request then
    local q=self.retail_request;q.ticks=q.ticks+1
    if q.scene_wait then
     if self:scene_safe() then q.scene_wait=nil;if not (self.mission.stopping or self.mission.staging or self.mission.retiring or self.mission.recovery) then self.retail_request=nil;self:start_retail(q.mode,q.fighter,q.difficulty,q.stocks,q.seed) end
     elseif q.ticks>=A.SCENE_WAIT_MAX then self.retail_request=nil;self.notice='The run did not start: the screen never settled';self.g.log('envoy: run start abandoned: the screen never settled') end
    elseif not (self.mission.stopping or self.mission.staging or self.mission.retiring or self.mission.recovery) then
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
    if not self.results_up and self:stage_decided() then self.results_up=true end
    self:sync_pause();local actions=self.input:poll()
    do local pad=self.g.pad(1,true) or {};if not pad.START then self.start_ready=true end end
    if self.visible then for _,action in ipairs(actions) do
     if action=='start' then self.g.log(('envoy: START seen screen=%s results_up=%s start_ready=%s paused=%s'):format(tostring(self.menu.screen),tostring(self.results_up),tostring(self.start_ready),tostring(self.g.paused and self.g.paused()))) end
     if not (action=='start' and (self.results_up or self.start_ready==false) and self.menu.screen=='playing') then self:menu_effect(self.menu:input(action,self:context())) end end end
    return
   end
   self.retail:tick();return old.tick(self)
  end
  function A:draw()
   if self.coop and self.coop.active then return end
   if self.retail.active or self.retail.pending then
    local r=self.retail
    if r.host then r.host.menu_up=(self.visible and self.menu.screen~='playing') and true or false end   -- the strip and toasts stay out of the pause / app menu
    -- The rule-host route has no companion stats: its build strip (run_hud) is drawn by the host instead.
    if not (r.rules and r.host) then D.hud.draw(self.g,r.companion,r.mode..' NG+'..r.loop,self.flashes,self.drives.juice.flash,r.reward) end
    if self.visible and self.menu.screen~='playing' and self.g.kit then D.menu_draw.draw(self.g,self.menu,self:context()) end
    if r.tag_ticks and r.tag_ticks>0 and self.g.kit and not (r.rules and r.host) then   -- a rule-host run shows its own opponent plate and strip; the companion tag panel sat on top of the strip
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
   if self.coop and self.coop.active then self.coop:stop();self.drives:clear();self.models:clear() end
   self.retail_request=nil;self.results_up=nil
   if self.retail.active or self.retail.pending then
    local ok,why=self.retail:finish(reason or 'quit');self.run.profile=self.retail.profile
    self.drives:clear();self.models:clear();self.recolour:clear();self.visual:clear()
    self.menu:results(self.retail.results);self.visible=true;self.retail_return=true;self:sync_pause();return ok,why
   end
   return old.stop(self,reason)
  end
  function A:match_start() if self.coop and self.coop.active then self.coop:scene_started();self.models:unload();return end;if self.retail.active or self.retail.pending then self.models:unload();return end;return old.match_start(self) end
  -- From the end of a retail stage until the next one starts the game's own results screen is up and waits for START: Envoy's
  -- START-opens-the-pause-menu must stay out of the way, or it eats the press (and, with START hidden while a menu is open, the
  -- game never sees it at all).
  -- A scene change while a run start is waiting for the screen to settle is not a match end: the old handler stopped the app, which dropped the
  -- waiting request (the 2026-10-05 guard test: the start deferred in the opening movie never fired at the title). Only the old scene's
  -- ownership is released.
  function A:match_end() if self.retail_request and self.retail_request.scene_wait and not (self.retail.active or self.retail.pending) and not (self.coop and self.coop.active) then
    self.g.log('envoy: scene change while the run start is waiting: not a match end, the start stays queued');if self.hub then self.hub:clear() end;self.mission:stop('match end');self.models:unload();return end
   if self.coop and self.coop.active then self.coop:scene_ended();self.models:unload();self.recolour:clear();return end;if (self.retail.active or self.retail.pending) and not self.retail.stage_started then self.g.log('envoy: match end before the run reached a stage: another scene ended, ignored');return end;if self.retail.active or self.retail.pending then self.g.log('envoy: match end in a retail run: results screen up');self.results_up=true;self.start_ready=false;self.models:unload();self.recolour:clear();return end;return old.match_end(self) end
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
  -- ---- SOLO > ENVOY (Atlas step 2). mod.json "menus" names the entry; the host calls on_entry("envoy") (generated into main.lua by
  -- tools/port/envoy_bundle.py) when the player picks it. It opens a small Atlas screen in front of the native menu; Envoy's own
  -- screens become Atlas in step 3. "ENVOY MENU" is the legacy Envoy menu: while it is up the native menu is held (gd.ui.hold_menu).
  local ENTRY_ID='envoy.entry'
  local ENTRY_ROWS={
   {id='classic',label='START CLASSIC',what='A Classic run: the retail ladder with your drives, bag and keystones.'},
   {id='adventure',label='START ADVENTURE',what='An Adventure run: the retail side-scrolling stages with the same build.'},
   {id='menu',label='ENVOY MENU',what='Your companion, the garden, records and settings: the full Envoy menu.'},
   {id='back',label='BACK',what='Return to Solo.'}}
  function A:entry_available()
   local ui=self.g.ui
   if type(ui)~='table' or type(ui.available)~='function' or type(ui.screen)~='function' then return false end
   return (ui.available()) and true or false
  end
  function A:entry(id)
   if id~='envoy' then return nil end
   local m=self.g.match();if m and m.netplay then return nil end
   if not self:entry_available() then self.g.log('envoy: the Atlas menus are unavailable; open the Envoy menu with the console command: envoy menu');return nil end
   local app=self
   local items,what={},{}
   local rows={}
   for _,r in ipairs(ENTRY_ROWS) do
    rows[#rows+1]=r
    if r.id=='adventure' and D.atlas_kit and D.atlas_kit.enabled(self.g) and D.atlas_setup then   -- Atlas step 3: the run setup screen (mode, fighter, difficulty, stocks)
     rows[#rows+1]={id='setup',label='RUN SETUP',what='Pick the mode, fighter, difficulty and stocks, then begin.'}
    end
   end
   for _,r in ipairs(rows) do items[#items+1]={id=r.id,label=r.label};what[r.id]=r end
   local ok,err=pcall(function() self.g.ui.screen{
    id=ENTRY_ID,chapter=1,trail={'SOLO',title='ENVOY'},
    primary={kind='list',items=items},
    explainer={width='normal',provide=function(cell) local r=what[cell] or ENTRY_ROWS[1];return {kicker='ENVOY',title=r.label,what=r.what} end},
    keys={{'A','Open'},{'B','Back'}},
    on={accept=function(cell) return app:entry_accept(cell) end,back=function() return {pop=true} end}} end)
   if not ok then self.g.log('envoy: the entry screen was refused: '..tostring(err));return nil end
   return {push=ENTRY_ID}
  end
  function A:entry_accept(cell)
   if cell=='classic' or cell=='adventure' then
    self.menu.run_type=cell
    local ok,why=self:start_retail(cell)
    if not ok then
     if self.g.ui.note then pcall(self.g.ui.note,{text=tostring(why or 'The run did not start'),kind='warn'}) end
     return nil
    end
    return {pop=true}
   elseif cell=='setup' then
    self.visible=true;self.menu:show('setup')
    if self.menu.atlas and self.g.ui.hold_menu then pcall(self.g.ui.hold_menu,true);self.menu_held=true end
    return {pop=true}
   elseif cell=='menu' then
    self:command('menu')
    if self.g.ui.hold_menu then pcall(self.g.ui.hold_menu,true);self.menu_held=true end
    return {pop=true}
   end
   return {pop=true}
  end
 end
 return M
end
