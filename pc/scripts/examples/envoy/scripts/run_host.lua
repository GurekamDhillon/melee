-- Run adapter for the rule host (mod_lab): Classic and Adventure install the same pool, bag, slots, opponent rolls
-- and looks the LAB does, at the director's own lifecycle boundaries (run begin, stage start, fighter spawn, stage
-- clear, run end). It exists only while the retail run's `rules` switch is on, so the old companion-stat route
-- stays playable beside it.
--
-- What the player sees (see MENUS.md): opponents drop a drive where they fall; walking over it puts it in the bag
-- with a name card; after a stage clear the hold shows a reward screen (choose one drive, equip or swap, keep in the
-- bag, skip); a strip of pips shows the build during fights; Z+START opens the same screen as a bag.
-- Bag edits made here are applied to the bag directly (a stage clear is a frozen barrier with no logic frames, and a
-- queued edit would be wiped by the next scene's reset); the engine picks the new build up at its next logic frame.
return function(D)
 local H={};H.__index=H
 local function seed_for(seed,stage,loop,port)
  return (seed+stage*104729+loop*15485863+port*32452843)%2147483646+1
 end
 -- settle_frames: the look warm-up draws the fighters' models, and a retail stage's fighters are not safe to draw
 -- until the entrance has played (a crash in the model draw 30 frames in was seen on the first try).
 -- Tuning switches:
 --  drops            opponents drop a drive where they fall (default ON on this route)
 --  auto_collect     drops still on the ground at stage end go to the bag; false: they are lost, and the log says so
 --  drop_percent     an opponent also drops when it first passes this damage (a one-stock fight ends the instant it is KO'd,
 --                   so its drive has to be reachable before that)
 --  drop_chance      chance of a further drop after the stage's first guaranteed one (battle/giant/metal stages)
 --  team_drops_max   a team stage drops one per defeated opponent, up to this many
 --  offer_count      drives offered after a normal stage clear; bonus_offer_count after a bonus stage and the final boss
 H.tuning={drop_chance=.25,settle_frames=30,roll_attempts=2,drops=true,auto_collect=true,team_drops_max=3,battle_drops_max=2,
  offer_count=2,bonus_offer_count=3,hold_ticks=1800,ko_poll=6,bag_capacity=12,drop_percent=50}
 function H.new(g,mods,retail)
  local self=setmetatable({g=g,mods=mods,retail=retail,running=false,fell={},rolls={},stage=0,loop=0,drops=0,since=0,
   offers={},new_keys={},kos=0,hurt={},dropped={},foe_ports={},stage_kind='battle',faded={},seen_slots=nil,seen_keys=nil,seen_tier=nil,seen_loop=nil},H)
  self.screen=D.run_screen.new(g,self);self.hud=D.run_hud.new(g,self)
  if mods.drives then
   mods.drives.opener=function() if self.running then self.screen:open('bag') end end
   mods.drives.on_pickup=function(r) self:picked_up(r) end
   mods.drives.on_expire=function(r,why) self:expired(r,why) end
  end
  g.command('uxdump',function() self:dump();return true end,'log the rule host state: slots, bag, offers, screen, hud')
  g.command('uxpress',function(a) self:press(a or '');return true end,'press a screen action: up down left right accept back x y start')
  g.command('uxbag',function() if self.running then self.screen:open('bag') end;return true end,'open the run bag screen')
  return self
 end
 function H:log(text) self.g.log('envoy rules: '..text) end
 -- ---- small accessors the screen and HUD share ------------------------------------------------------------------
 function H:bag() return self.mods.drives.bag end
 local function key(r) return table.concat({r.seed,r.depth,r.loop or 0,r.colour,r.rarity},':') end
 function H:is_new(r) return self.new_keys[key(r)]==true end
 function H:mark_new(r) self.new_keys[key(r)]=true end
 function H:free_slot() local b=self:bag();for s=1,b:slots() do if not b.equipped[s] then return s end end end
 function H:equipped_count() local b=self:bag();local n=0;for s=1,b:slots() do if b.equipped[s] then n=n+1 end end;return n end
 function H:keystone_set()
  local b=self:bag();local out={};for _,id in ipairs(b.keystones or {}) do out[id]=true end;if b.keystone then out[b.keystone]=true end;return out
 end
 function H:name(r) return self.mods.drives.loot:name(r) end
 function H:touch() local d=self.mods.drives;d:bump();self.mods.enabled=true;self.hud.m=nil;self.screen:invalidate() end
 -- Totals for the build as it is, or with `edit(draft_bag)` applied to a throwaway copy.
 function H:totals(edit)
  local d=self.mods.drives;local b=d.bag;local config={};for k,v in pairs(b.config) do config[k]=v end;config.preflight=nil
  local draft=D.drive_bag.new(d.loot,config);local s=b:snapshot()
  assert(draft:restore(s));if edit then pcall(edit,draft) end
  local ok,mods,implicits=pcall(draft.derive,draft)
  if not ok then draft=D.drive_bag.new(d.loot,config);draft:restore(s);mods,implicits=draft:derive() end
  local families,strength=D.mod_budget.build(D.mod_pool,d:combined(mods),implicits,{})
  return D.drive_text.totals(families,strength)
 end
 local why_text={['bag full']='Your bag is full: discard a drive first.',['invalid index']='That drive is not available.'}
 local function plain(why)
  why=tostring(why)
  if why:find('hit rule capacity',1,true) then return 'Too many special attack rules: unequip a drive first.' end
  if why:find('keystone allowance',1,true) then return 'You cannot hold another keystone yet: remove one first.' end
  return why_text[why] or ('Not possible: '..why)
 end
 -- ---- bag edits (all of them log what really happened) ----------------------------------------------------------
 function H:equip(i,slot)
  local b=self:bag();local r=b.items[i];local old=b.equipped[slot]
  if not r then return false,'That drive is not in the bag.' end
  local ok,why=b:equip(i,slot)
  if not ok then self:log('equip refused: '..tostring(why));return false,plain(why) end
  self:touch();self.hud:flash('Slot '..slot..' changed')
  if old then self:log(('swapped out %s: %s now in slot %d, %s went to the bag'):format(self:name(old),self:name(r),slot,self:name(old)));return true,('Slot %d: %s. %s is in your bag.'):format(slot,self:name(r),self:name(old)) end
  self:log(('equipped into slot %d: %s'):format(slot,self:name(r)));return true,('Equipped in slot %d.'):format(slot)
 end
 function H:unequip(slot)
  local b=self:bag();local r=b.equipped[slot]
  if not r then return false,'Slot is empty.' end
  local ok,why=b:unequip(slot)
  if not ok then self:log('unequip refused: '..tostring(why));return false,plain(why) end
  self:touch();self.hud:flash('Slot '..slot..' emptied');self:log(('unequipped slot %d: %s is in the bag'):format(slot,self:name(r)))
  return true,self:name(r)..' is in your bag.'
 end
 function H:discard(i)
  local b=self:bag();local r=b.items[i];if not r then return false,'That drive is not in the bag.' end
  local name=self:name(r);b:discard(i);self:touch();self:log('discarded '..name..' (player choice)');return true,'Discarded '..name..'.'
 end
 function H:toggle_keystone(id)
  local b=self:bag();local chosen=self:keystone_set();local allow=D.mod_progression.keystones(b.context);local label=id
  for _,m in ipairs(self.mods.drives.lab.engine.list) do if m.id==id then label=m.label end end
  local ok,why
  if chosen[id] then if allow==1 then ok,why=b:choose_keystone(nil) else ok,why=b:choose_keystone(id) end
  else ok,why=b:choose_keystone(id) end
  if not ok then self:log('keystone refused: '..tostring(why));return false,plain(why) end
  self:touch()
  if chosen[id] then self:log('keystone removed: '..label);self.hud:flash('Keystone removed');return true,label..' removed.' end
  self:log('keystone chosen: '..label);self.hud:flash('Keystone: '..label);return true,label..' chosen.'
 end
 function H:take_offer(i)
  local r=self.offers[i];if not r then return nil,'That drive is no longer on offer.' end
  local b=self:bag();local ok,why=b:give(r)
  if not ok then return nil,plain(why) end
  local others={};for j,o in ipairs(self.offers) do if j~=i then others[#others+1]=self:name(o) end end
  self.offers={};self:mark_new(r);self:touch()
  self:log(('took %s from the stage reward%s'):format(self:name(r),#others>0 and ('; declined '..table.concat(others,', ')) or ''))
  return #b.items
 end
 function H:decline_offers(reason)
  local names={};for _,o in ipairs(self.offers) do names[#names+1]=self:name(o) end
  if #names>0 then self:log(('%s the stage reward: none kept (%s)'):format(reason,table.concat(names,', '))) end
  self.offers={};self:touch()
 end
 -- A drive that arrives in the bag (pickup, auto-collect, starter): returns the bag index or nil + the reason.
 function H:acquire(r,how)
  local b=self:bag();local ok,why=b:give(r)
  if not ok then self:log(('discarded %s (%s): bag full'):format(self:name(r),how));return nil end
  self:mark_new(r);self:touch();return #b.items
 end
 -- ---- announcements ---------------------------------------------------------------------------------------------
 -- One announcement per change, said once: a new slot, a keystone allowance, a drive tier, New Game+.
 local function milestones(self,ctx)
  local list={}
  local slots=D.mod_progression.slots(ctx);local keys=D.mod_progression.keystones(ctx);local tier=D.mod_progression.tier(ctx)
  local function add(title,line) list[#list+1]={{text=title,colour='gold'},line} end
  if self.seen_slots and slots>self.seen_slots then add(({[5]='Fifth slot unlocked',[6]='Sixth slot unlocked'})[slots] or ('Slot '..slots..' unlocked'),'You can equip one more drive. Open the bag with Z+START.') end
  if self.seen_keys and keys>self.seen_keys then add('Keystone allowance: '..keys,'A keystone is one powerful rule with a drawback. Choose in the bag.') end
  if self.seen_tier and tier>self.seen_tier then add('Drive tier '..tier,'New drives roll stronger modifiers; opponents scale up too.') end
  if self.seen_loop and ctx.loop>self.seen_loop then add('New Game+ '..ctx.loop,'Your build carries over. Opponents start much stronger.') end
  self.seen_slots,self.seen_keys,self.seen_tier,self.seen_loop=slots,keys,tier,ctx.loop
  return list
 end
 -- ---- lifecycle -------------------------------------------------------------------------------------------------
 function H:run_begin(seed)
  self.running=true;self.seed=seed;self.fell={};self.rolls={};self.stage=0;self.loop=0;self.drops=0
  self.offers={};self.new_keys={};self.kos=0;self.faded={};self.hud:clear();if self.screen.active then self.screen:close() end
  self.mods:run_end() -- a new run starts from an empty bag, whatever the last one left
  local ctx=D.mod_progression.context(0,0);self.mods:set_context(ctx)
  self.seen_slots,self.seen_keys,self.seen_tier,self.seen_loop=D.mod_progression.slots(ctx),D.mod_progression.keystones(ctx),D.mod_progression.tier(ctx),0
  -- A starter drive, so the first opponents already roll against a build: put straight into slot 1.
  local record=self.mods.drives.loot:roll(seed_for(seed,0,0,5),ctx)
  self.starter=record
  local idx=self:acquire(record,'starter')
  if idx then local ok,msg=self:equip(idx,1);self.starter_text=ok and msg or nil;self.new_keys={}
   self.hud:announce({{text='Your starter drive',colour='gold'},{text=self:name(record),colour=D.drive_text.rarity_colour[record.rarity]},D.drive_text.drive_lines(self.mods.drives.loot,record)[2] or ''}) end
  self:log('run begin seed='..seed)
 end
 -- Progression follows the director's stage and NG+ loop; the bag's slots and keystones grow with it.
 function H:stage_start(e)
  if not self.running then return end
  self.stage=e.stage_index or 0;self.loop=e.loop or self.retail.loop or 0;self.fell={};self.hurt={};self.dropped={};self.since=0;self.kos=0;self.faded={}
  self.stage_kind=e.stage_kind or 'battle';self.foe_ports={};self.drop_queue={}
  for _,o in ipairs(e.opponents or {}) do if o.port then self.foe_ports[#self.foe_ports+1]=o.port end end
  local ctx=D.mod_progression.context(math.min(self.stage,12),self.loop)
  self.mods:set_context(ctx)
  for _,toast in ipairs(milestones(self,ctx)) do self.hud:announce(toast);self:log('announce: '..toast[1].text) end
  self.hud.m=nil;self.mods.drives:bump()
  self:log(('stage %d (%s) NG+%d context depth=%d slots=%d'):format(self.stage,self.stage_kind,self.loop,ctx.depth,D.mod_progression.slots(ctx)))
 end
 -- Opponents roll a build from the player's current strength, exactly as `foe roll` does.
 function H:spawn(e)
  if not self.running or not e or not e.port or e.port==(self.retail.state and self.retail.state.player_port or 1) then return end
  -- The spawn signal fires while the scene is still being built: only note the port here. The roll (which
  -- also warms the look shaders) is queued from the first logic frame, when the fighter is fully present.
  self.rolls[e.port]=true
 end
 function H:roll_wanted()
  local foes=self.mods.foes;if not foes then return end
  local d=self.mods.drives
  -- Rolls scale to the player's published build: wait until the bag's build has reached the engine.
  if #d.pending>0 then return end
  if d:has_build() and (not d.applied or d:stale()) then return end
  for p in pairs(self.rolls) do
   local v=self.g.player(p)
   if v and v.cpu then
    local ok,why=pcall(function()
     if not (foes.jobs and foes.jobs[p]) then
      local _,strength=self.mods.engine:family_budget(1)
      foes:roll_begin(p,strength,seed_for(self.seed,self.stage,self.loop,p),self.stage,'normal')
     end
     return foes:roll_advance(p,H.tuning.roll_attempts)
    end)
    if not ok then self.rolls[p]=nil;self:log('opponent roll refused P'..p..': '..tostring(why))
    elseif why then self.rolls[p]=nil;self:log('queued opponent roll P'..p) end
   end
  end
 end
 -- ---- drops -----------------------------------------------------------------------------------------------------
 -- Rules: a battle, giant or metal stage drops one drive at its first trigger (an opponent passes drop_percent damage or
 -- loses a stock) and may drop a second
 -- (drop_chance); a team stage drops one for every opponent's first trigger (up to team_drops_max); a bonus stage has no
 -- opponents and drops none (its clear offers three drives); the final boss drops nothing on the floor (its clear
 -- offers three: two rare and one unique). Drops are seeded by run seed, stage and defeat number.
 function H:drop_rule(kind)
  if kind=='bonus' or kind=='boss' then return 0,0 end
  if kind=='team' then return H.tuning.team_drops_max,1 end
  return H.tuning.battle_drops_max,H.tuning.drop_chance
 end
 function H:on_ko(p,why)
  if self.stage_kind=='team' then if self.dropped[p] then return end;self.dropped[p]=true end -- one per opponent
  self.kos=self.kos+1
  if not H.tuning.drops or not self.mods.drives then return end
  local max,chance=self:drop_rule(self.stage_kind)
  local d=self.mods.drives
  if self.kos>max then return end
  local guaranteed=self.kos==1 or self.stage_kind=='team'
  if not guaranteed then
   local x=(seed_for(self.seed,self.stage,self.loop,p)+self.kos*48271)%2147483646+1
   for _=1,3 do x=(x*16807)%2147483647 end
   local roll=x/2147483647
   if roll>=chance then self:log(('opponent P%d dropped nothing'):format(p));return end
  end
  if #d.bag.items+d.drops:count()+(self.drop_wait and 1 or 0)>=H.tuning.bag_capacity then self:log('drop skipped: bag and ground full');return end
  local record=d.loot:roll(seed_for(self.seed,self.stage,self.loop,p+self.kos*7),self.mods.engine.context)
  self.drop_queue=self.drop_queue or {};self.drop_queue[#self.drop_queue+1]={record=record,port=p,tries=0,why=why or 'defeated'}
 end
 -- Where a drop appears: on the stage, on the floor a few steps from the player (the opponent may be off screen).
 function H:drop_position()
  local g=self.g;local port=self.retail.state and self.retail.state.player_port or 1
  local v=g.player(port);if not v then return nil end
  local x,y=v.x+(v.x>0 and -40 or 40),v.y -- toward the middle of the stage, a few steps away, so it can be seen and walked to
  local b=g.stage_bounds and g.stage_bounds();local cam=b and b.camera
  if cam then local lo,hi=cam.left*.7,cam.right*.7;if lo<hi then x=math.max(lo,math.min(hi,x)) end end
  local fy=g.floor_below and g.floor_below(x,y+40);if fy then y=fy end
  return x,y+12
 end
 function H:spawn_queued()
  local q=self.drop_queue;if not q or #q==0 then return end
  local item=q[1];local x,y=self:drop_position();if not x then return end
  item.tries=item.tries+1
  local ok,why=pcall(function() return self.mods.drives.drops:spawn(item.record,x,y) end)
  if ok then table.remove(q,1);self:log(('opponent P%d dropped %s (%s)'):format(item.port,self:name(item.record),item.why))
   self.hud:show_card('A drive dropped!',{'Walk over it to pick it up.'},D.drive_text.rarity_colour[item.record.rarity])
  elseif item.tries>=60 then table.remove(q,1);self:log(('drop failed for %s: %s'):format(self:name(item.record),tostring(why))) end
 end
 -- Called by the drive host after a pickup made it into the bag.
 function H:picked_up(r)
  self:mark_new(r);self.hud.m=nil
  local d=self.mods.drives;local lines=D.drive_text.drive_lines(d.loot,r)
  self.hud:show_card('Picked up: '..self:name(r),{lines[2] or lines[1],'It is in your bag (Z+START).'},D.drive_text.rarity_colour[r.rarity])
  self.hud:flash('+ drive');self:log('picked up '..self:name(r)..' into the bag')
 end
 function H:expired(r,why) self.faded[#self.faded+1]=r;self:log('a drive faded from the floor: '..self:name(r)) end
 -- Drops still on the ground when the stage clears: into the bag (auto_collect) or lost, said either way.
 function H:collect_ground()
  local d=self.mods.drives;local list={}
  for _,rec in pairs(d.drops.records) do list[#list+1]=rec.record end
  for _,r in ipairs(self.faded) do list[#list+1]=r end
  self.faded={};if self.drop_queue then
   for _,it in ipairs(self.drop_queue) do list[#list+1]=it.record end;self.drop_queue={}
  end
  for _,r in ipairs(list) do
   if H.tuning.auto_collect then
    if self:acquire(r,'uncollected') then self:log('collected an uncollected drop: '..self:name(r)) end
   else self:log('discarded '..self:name(r)..' (uncollected, auto_collect off)') end
  end
  d.drops:clear()
 end
 function H:frame()
  if not self.running then return end
  self.since=self.since+1;self.hud:frame();if not self:ready() then return end
  self.hud:watch(self.mods.engine)
  if self.since%H.tuning.ko_poll~=0 then return end
  local port0=self.retail.state and self.retail.state.player_port or 1
  for _,p in ipairs(self.foe_ports) do
   if p~=port0 then
    local v=self.g.player(p)
    if v then
     local falls=v.falls or 0;local before=self.fell[p] or 0 -- a stage's fighters start with no falls
     if falls>before then self:on_ko(p,'lost a stock') end
     self.fell[p]=falls
     if not self.hurt[p] and (v.percent or 0)>=H.tuning.drop_percent then self.hurt[p]=true;self:on_ko(p,'passed '..H.tuning.drop_percent..'%') end
    end
   end
  end
 end
 function H:ready() return self.running and self.since>=H.tuning.settle_frames end
 -- ---- reward moment ---------------------------------------------------------------------------------------------
 -- A cleared stage: collect what is on the floor, roll the offered drives, hold the barrier and show the screen.
 function H:stage_reward(stage,loop,final)
  if not self.running or not self.mods.drives then return false end
  self:collect_ground()
  local kind=self.stage_kind
  local many=final or kind=='bonus' or kind=='boss'
  local n=many and H.tuning.bonus_offer_count or H.tuning.offer_count
  local d=self.mods.drives;local ctx=self.mods.engine.context
  self.offers={}
  local forced=final and {'rare','rare','unique'} or nil
  for i=1,n do self.offers[i]=d.loot:roll(seed_for(self.seed,stage,loop,(final and 11 or 7)+i*13),ctx,forced and forced[i] or nil) end
  local names={};for _,o in ipairs(self.offers) do names[#names+1]=self:name(o) end
  self:log(('stage clear (%s%s): offers %s'):format(kind,final and ', final' or '',table.concat(names,' / ')))
  local newn=0;for _ in pairs(self.new_keys) do newn=newn+1 end
  self:log(('reward moment: %d offered, %d new in the bag'):format(#self.offers,newn))
  local ok=self.g.hold_1p and self.g.hold_1p(H.tuning.hold_ticks)
  if ok then self.screen:open('reward');self.holding=true
  else self:log('reward hold unavailable: sorting the drives automatically');self:finish_reward('no-hold') end
  return true
 end
 -- Every end of the reward moment ends here. `reason`: done (the player continued), timeout, released (the
 -- engine's own hold ended first), no-hold, run-end. Nothing is dropped without a log line.
 function H:finish_reward(reason)
  local auto=reason~='done'
  if #self.offers>0 then
   if reason=='run-end' then self:decline_offers('run ended during')
   else
    local idx=self:take_offer(1)
    if idx then self:log(('%s: took %s automatically'):format(reason,self:name(self:bag().items[idx])))
    else self:decline_offers(reason) end
   end
  end
  if auto and reason~='run-end' then
   -- Sort the new drives: free slots first, the rest stay in the bag.
   local b=self:bag()
   local i=1
   while i<=#b.items do
    local r=b.items[i];local free=self:free_slot()
    if self:is_new(r) and free then
     local ok=self:equip(i,free);if not ok then i=i+1 end
    else i=i+1 end
   end
  end
  local kept=0;for _,r in ipairs(self:bag().items) do if self:is_new(r) then kept=kept+1 end end
  self.new_keys={};self.holding=false
  self:log(('reward moment done (%s): %d slots filled, %d in the bag'):format(reason,self:equipped_count(),#self:bag().items))
  if self.g.release_1p and reason~='released' then self.g.release_1p() end
  self.hud:flash('Build ready')
 end
 function H:press(a) if self.screen.active then self.screen:press(a) end end
 -- Rolling a build and validating a bag edit are each close to a script call's whole budget (2M instructions,
 -- 50 ms), and on_frame already carries the host's own work: staging happens in on_tick, a call of its own.
 function H:tick()
  if self.screen.active then self.screen:tick() end
  if not self:ready() then return end
  local drives=self.mods.drives;if not drives then return end
  self:spawn_queued()
  self:roll_wanted()
 end
 function H:draw()
  if not self.running then return end
  if self.screen.active then self.screen:draw();return end
  local m=self.g.match();if not (m and m.active) or not self:ready() then return end
  self.hud:draw()
 end
 function H:run_end()
  if not self.running then return end
  if self.screen.active then self.screen:close() end
  if #self.offers>0 then self:finish_reward('run-end') end
  self.running=false;self.fell={};self.rolls={};self.mods:run_end();self.hud:clear();self:log('run end: bag and build cleared')
 end
 -- The model as text, for the console and the tests.
 function H:dump()
  local d=self.mods.drives;if not d then self.g.log('uxdump: no drive host');return end
  local b=d.bag;local loot=d.loot
  self.g.log(('uxdump: running=%s stage=%d (%s) slots=%d bag=%d offers=%d pending=%d ground=%d new=%d'):format(tostring(self.running),self.stage,self.stage_kind,b:slots(),#b.items,#self.offers,#d.pending,d.drops:count(),(function() local n=0;for _ in pairs(self.new_keys) do n=n+1 end;return n end)()))
  local rolls={};for p in pairs(self.rolls) do rolls[#rolls+1]='P'..p end
  local disp=self.mods.display
  self.g.log(('uxdump: host ready=%s since=%d applied=%s stale=%s lab.enabled=%s rolls=[%s] display ready=%s error=%s note=%s'):format(tostring(self:ready()),self.since,tostring(d.applied),tostring(d:stale()),tostring(self.mods.enabled),table.concat(rolls,','),tostring(disp and disp.ready),tostring(disp and disp.error),tostring(disp and disp.note)))
  for i=1,b:slots() do self.g.log('uxdump: slot '..i..' = '..(b.equipped[i] and loot:name(b.equipped[i]) or 'empty')) end
  for i,r in ipairs(b.items) do self.g.log('uxdump: bag '..i..' = '..loot:name(r)..(self:is_new(r) and ' (new)' or '')) end
  for i,r in ipairs(self.offers) do self.g.log('uxdump: offer '..i..' = '..loot:name(r)) end
  for _,l in ipairs(self.hud:dump()) do self.g.log('uxdump: '..l) end
  if self.screen.active then for _,l in ipairs(self.screen:dump()) do self.g.log('uxdump: '..l) end end
 end
 return H
end
