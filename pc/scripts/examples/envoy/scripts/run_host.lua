-- Run adapter for the rule host (mod_lab): Classic and Adventure install the same pool, bag, slots, opponent rolls
-- and looks the LAB does, at the director's own lifecycle boundaries (run begin, stage start, fighter spawn, stage
-- clear, run end). It exists only while the retail run's `rules` switch is on, so the old companion-stat route
-- stays playable beside it.
--
-- What the player sees (see MENUS.md): the run starts with one drive and one random keystone; opponents drop a drive
-- where they fall (at most one per stage); a drive that matches one you hold MERGES into it, otherwise it goes in the
-- bag (four places), a free slot, or asks which drive to give up; at a stage clear that owes a reward the hold shows
-- the grid screen (a pick of three drives, and a pick of three keystones when the allowance has stepped up); a strip
-- shows the build during fights; Z+START opens the same grid as a bag.
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
 --  auto_collect     drops still on the ground at stage end go through the same gain rule; false: they are lost, and the log says so
 --  drop_percent     an opponent also drops when it first passes this damage (a one-stock fight ends the instant it is KO'd,
 --                   so its drive has to be reachable before that)
 -- How many drives a run hands out (floor drops per stage, which stages give a reward, how many are offered, the bag's size)
 -- is `D.drive_economy.tuning` (floor_max, floor_chance, team_max, reward_every, offers, bonus_offers, bag_capacity).
 H.tuning={settle_frames=30,roll_attempts=2,drops=true,auto_collect=true,hold_ticks=2850,end_hold=true,end_hold_frames=1800,ko_poll=6,drop_percent=50,payout=true,oob_margin=300,oob_frames=240,oob_fallback=3000}
 local function econ() return D.drive_economy.tuning end
 -- `seat` (co-op only): {index=,port=,drives=,allies={[port]=true,...},strength=fn,gather=fn,route=fn,barrier={open=fn,release=fn},anchor=,colour=}.
 -- No seat: the one-player host, exactly as before. Seat 1 is the lead (it owns the stage's opponents, drops and match hold); the other
 -- seats are followers: own bag, strip and screens on their own port, a facade over the shared rule host that cannot reset it.
 function H.new(g,mods,retail,seat)
  if seat and seat.index>1 then
   mods=setmetatable({drives=seat.drives,foes=false,run_end=function() end,toast=false},{__index=mods})
  end
  local self=setmetatable({g=g,mods=mods,retail=retail,seat=seat,follower=seat~=nil and seat.index>1,running=false,fell={},rolls={},stage=0,loop=0,drops=0,since=0,
   offers={},key_offers={},decide={},deferred={},new_keys={},kos=0,hurt={},dropped={},foe_ports={},stage_kind='battle',faded={},seen_slots=nil,seen_keys=nil,seen_tier=nil,seen_loop=nil,rolled={}},H)
  self.screen=D.run_screen.new(g,self);self.hud=D.run_hud.new(g,self)
  self.synfx=D.synergy_fx and D.synergy_fx.new(g,self)
  if mods.drives then
   mods.drives.opener=function() if self.running then self.screen:open('bag') end end
   mods.drives.on_pickup=function(r) self:picked_up(r) end
   mods.drives.on_expire=function(r,why) self:expired(r,why) end
  end
  -- A technique or crit moment worth a line (first technique rule fired, a strong crit): the strip's toast, presentation only.
  -- Teaching and error messages from the mod (technique hint, "Build update refused ...") are LOGGED always and shown only with the developer overlay on.
  if not self.follower then mods.toast=function(text) if self.running then self:log('mod message: '..tostring(text));if D.run_hud.dev_ui() then self.hud:announce({{text='Technique',colour='gold'},text}) end end end end
  if mods.foes and not self.follower then mods.foes.defer=function() return self.running and ((self.hud and #self.hud.toasts>0) or self.menu_up==true) end end  -- the opponent plate waits while an announcement is up (they used to overlap)
  if self.follower then return self end -- the console commands belong to the lead seat (they take a port where it matters)
  g.command('uxdump',function() self:dump();return true end,'log the rule host state: slots, bag, keystones, offers, screen, hud')
  g.command('uxpress',function(a) self:press(a or '');return true end,'press a screen action: up down left right accept back x y start')
  g.command('uxcost',function(a) if a=='reset' then self.cost={} else for _,l in ipairs(self:cost_report()) do g.log(l) end end;return true end,'script cost of the run screens and strip: uxcost [reset]')
  -- Test hooks (they run the real rules, only the trigger is artificial): `uxgain` gains rolled drives through the pickup rule,
  -- `uxpreview` opens the reward grid with real offers outside a clear (A on it does what it does at a clear).
  g.command('uxgain',function(a)
   if not self.running then g.log('uxgain: no run');return false end
   local d=self.mods.drives;local ctx=self.mods.engine.context;self.gain_seed=(self.gain_seed or 1000)
   local n=1;local want_merge=false
   for w in (a or ''):gmatch('%S+') do if w=='merge' then want_merge=true else n=tonumber(w) or n end end
   for _=1,n do
    local r
    for _=1,4000 do self.gain_seed=self.gain_seed+1;r=d.loot:roll(self.gain_seed,ctx);if not want_merge or self:plan_take(r).action=='merge' then break end end
    local action=self:gain(r,'uxgain');g.log(('uxgain: %s -> %s'):format(self:name(r),tostring(action)))
    if action=='choose' and not self.screen.active then self.screen:open('bag') end
   end
   return true
  end,'test hook: gain rolled drives through the pickup rule: uxgain [n] [merge]')
  g.command('uxpreview',function(a)
   if not self.running or self.screen.active then return false end
   local d=self.mods.drives;local ctx=self.mods.engine.context;self.offers={};self.stage_kind='battle'
   for i=1,3 do self.offers[i]=d.loot:roll(7000+i*13,ctx,D.drive_economy.reward_rarity(ctx,i)) end
   self.stage=self.stage or 0;if (a or '')=='keys' then self.key_offers=D.keystones.offer(D.mod_progression.context(10,0),77,3,self:keystone_ids()) end
   self.screen:open('reward');self.screen.preview=true;return true
  end,'test hook: open the reward grid with real offers: uxpreview [keys]')
  -- Test hooks for synergy captures: `uxgive <id> [id...]` gains one single-modifier drive per record id through the pickup rule (the real rules, an artificial
  -- trigger); `uxoffer <id> [id...]` opens the reward grid with those single-modifier drives as the offer. A record that is not open at this depth is refused by the loot rules.
  local function made(self,id,n)
   local d=self.mods.drives;local ctx=self.mods.engine.context;local m=d.loot.rules[id];if not m or m.kind~='normal' then return nil,'unknown drive record '..tostring(id) end
   local r={seed=9000+n,depth=ctx.depth,loop=ctx.loop,colour=({'red','green','blue','yellow','purple'})[n%5+1],rarity='common',affixes={{id=id,tier=D.mod_progression.tier(ctx)}}}
   local ok,why=pcall(d.loot.validate,d.loot,r);if not ok then return nil,tostring(why) end
   return r
  end
  g.command('uxgive',function(a)
   if not self.running then g.log('uxgive: no run');return false end
   local n=0;for id in (a or ''):gmatch('%S+') do n=n+1;local r,why=made(self,id,n);if not r then g.log('uxgive: '..why);return false end
    local action=self:gain(r,'uxgive');g.log(('uxgive: %s -> %s'):format(self:name(r),tostring(action)));if action=='choose' and not self.screen.active then self.screen:open('bag') end end
   return true
  end,'test hook: gain single-modifier drives by record id: uxgive <id> [id...]')
  g.command('uxoffer',function(a)
   if not self.running or self.screen.active then return false end
   self.offers={};self.stage_kind='battle';local n=0
   for id in (a or ''):gmatch('%S+') do n=n+1;local r,why=made(self,id,n+40);if not r then g.log('uxoffer: '..why);return false end;self.offers[#self.offers+1]=r end
   self.stage=self.stage or 0;self.screen:open('reward');self.screen.preview=true;return true
  end,'test hook: open the reward grid with single-modifier drives as the offer: uxoffer <id> [id...]')
  g.command('uxmodel',function(a) local w={};for x in (a or ''):gmatch('%S+') do w[#w+1]=tonumber(x) end
   local mo=D.run_screen.model_opts;if w[1] then mo.yaw=w[1] end;if w[2] then mo.pitch=w[2] end;if w[3] then mo.margin=w[3] end
   g.log(('uxmodel: yaw %s pitch %s margin %s'):format(mo.yaw,mo.pitch,mo.margin));return true end,'look tuning: how a drive model sits in its cell: uxmodel [yaw] [pitch] [margin]')
  g.command('uxbag',function() if self.running then self.screen:open('bag') end;return true end,'open the run bag screen')
  return self
 end
 function H:log(text) self.g.log('envoy rules: '..(self.seat and ('P'..self.seat.port..' ') or '')..text) end
 -- The port this host's player plays on (1 in every one-player run), whether `p` is one of the player's allies, and a seed salt that
 -- differs per seat (0 for seat 1 and for a one-player run, so those seeds are unchanged).
 function H:port0() return self.seat and self.seat.port or (self.retail.state and self.retail.state.player_port or 1) end
 function H:is_ally(p) return self.seat~=nil and self.seat.allies~=nil and self.seat.allies[p]==true end
 -- Co-op: a decision worth recording (the run's event list, the same on both peers); a one-player host records nothing.
 function H:emit(kind,a,b) if self.seat and self.seat.emit then self.seat.emit(self,kind,a,b) end end
 function H:salt() return self.seat and (self.seat.index-1)*1000 or 0 end
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
 function H:keystone_ids() local b=self:bag();local ids={};for _,id in ipairs(b.keystones or {}) do ids[#ids+1]=id end;return ids end
 function H:keystone_rule(id) return self.mods.drives.loot.rules[id] end
 function H:name(r) return self.mods.drives.loot:name(r) end
 -- A tuning value changed: the derived numbers (budgets, native hit rules) are rebuilt from the same records at the next frame.
 function H:touch_tuning() if self.mods.drives then self:touch() end end
 function H:touch() local d=self.mods.drives;d:bump();self.mods.enabled=true;self.hud.m=nil;self.screen:invalidate() end
 -- Totals for the build as it is, or with `edit(draft_bag)` applied to a throwaway copy.
 function H:totals(edit)
  local d=self.mods.drives;local b=d.bag;local config={};for k,v in pairs(b.config) do config[k]=v end;config.preflight=nil
  local draft=D.drive_bag.new(d.loot,config);local s=b:snapshot()
  if not draft:restore(s) then self.totals_bad=true;self:log('build totals: the bag state is invalid, showing the last good totals');return self.last_totals or D.drive_text.totals({damage_dealt={value=1},launch_dealt={value=1},speed={value=1},damage_taken={value=1}},1) end
  if edit then pcall(edit,draft) end
  local ok,mods,implicits=pcall(draft.derive,draft)
  if not ok then draft=D.drive_bag.new(d.loot,config);draft:restore(s);mods,implicits=draft:derive() end
  local families,strength=D.mod_budget.build(D.mod_pool,d:combined(mods),implicits,{})
  local out=D.drive_text.totals(families,strength);if not edit then self.last_totals=out end;return out
 end
 local why_text={['bag full']='Your bag is full.',['invalid index']='That drive is not available.'}
 local function plain(why)
  why=tostring(why)
  if why:find('hit rule capacity',1,true) then return 'Too many special attack rules: unequip a drive first.' end
  if why:find('keystone allowance',1,true) then return 'You cannot hold another keystone yet: remove one first.' end
  if why:find('drained event boundary',1,true) then return 'Try again in a moment.' end
  if why:find('exclude each other',1,true) then return 'Those keystones exclude each other.' end
  if why:find('too many drawbacks',1,true) then return 'Those keystones stack too many drawbacks.' end
  return why_text[why] or ('Not possible: '..why)
 end
 H.plain=plain
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
 -- LAB-style keystone toggle (kept for the console and tests; a run's keystones are never removed by the grid).
 function H:toggle_keystone(id)
  local b=self:bag();local chosen=self:keystone_set();local allow=D.mod_progression.keystones(b.context);local label=id
  for _,m in ipairs(self.mods.drives.lab.engine.list) do if m.id==id then label=m.label end end
  -- A full allowance is an expected outcome, not an error: say so plainly and leave the choice where it was.
  if not chosen[id] and allow>1 then
   local n=0;for _ in pairs(chosen) do n=n+1 end
   if n>=allow then
    self:log(('keystone not chosen: allowance full (%d/%d)'):format(n,allow))
    return false,('Keystone slots full (%d of %d): remove one first'):format(n,allow)
   end
  end
  local ok,why
  if chosen[id] then if allow==1 then ok,why=b:choose_keystone(nil) else ok,why=b:choose_keystone(id) end
  else ok,why=b:choose_keystone(id) end
  if not ok then self:log('keystone refused: '..tostring(why));return false,plain(why) end
  self:touch()
  if chosen[id] then self:log('keystone removed: '..label);self.hud:flash('Keystone removed');return true,label..' removed.' end
  self:log('keystone chosen: '..label);self.hud:flash('Keystone: '..label);return true,label..' chosen.'
 end
 -- ---- gaining a drive: merge, bag, free slot, or ask ----------------------------------------------------------------
 -- What taking `r` would do. `exclude` ({where=,index=}) names r's own place when r is already a bag drive (it cannot merge
 -- into itself and needs no bag room). Order on a choice screen: merge, free slot, bag, else ask which to replace.
 -- A free slot never needs bag space: the drive goes straight into the slot (bag:place); it is not bagged first.
 function H:plan_take(r,exclude)
  local b=self:bag();local loot=self.mods.drives.loot;local held,locs={},{}
  for _,e in ipairs(b:held()) do if not (exclude and e.where==exclude.where and e.index==exclude.index) then held[#held+1]=e.record;locs[#locs+1]=e end end
  local at=D.drive_merge.find_target(held,r,loot)
  if at then local merged,info=D.drive_merge.merge(held[at],r,loot);if merged then return {action='merge',loc=locs[at],merged=merged,info=info,target=held[at],from=exclude} end end
  local free=self:free_slot()
  if exclude then if free then return {action='equip',slot=free,from=exclude} end;return {action='choose'} end
  if free then return {action='equip',slot=free} end
  if #b.items<b:capacity() then return {action='bag'} end
  return {action='choose'}
 end
 -- The plan for a drive arriving on the floor or uncollected: merge, bag, a free slot when the bag is full, else ask.
 function H:plan_gain(r)
  local b=self:bag();local loot=self.mods.drives.loot;local locs=b:held();local list={};for i,e in ipairs(locs) do list[i]=e.record end
  local plan=D.drive_economy.gain_plan(list,#b.items,r,loot,econ())
  if plan.action=='merge' then
   local merged,info=D.drive_merge.merge(list[plan.index],r,loot)
   if merged then return {action='merge',loc=locs[plan.index],merged=merged,info=info,target=list[plan.index]} end
   plan=#b.items<b:capacity() and {action='bag'} or {action='choose'}
  end
  if plan.action=='choose' then local free=self:free_slot();if free then return {action='equip',slot=free} end end
  return plan
 end
 function H:merge_text(plan)
  local loot=self.mods.drives.loot;local rule=loot.rules[plan.info.affix]
  return D.drive_text.short(loot,plan.target),(rule and rule.label or 'A modifier')..' got stronger'
 end
 -- Carry a plan out. Returns the action, or nil and why. All of it is logged; nothing is dropped silently.
 function H:apply_plan(plan,r,how)
  local b=self:bag();local ok,why
  if plan.action=='merge' then
   ok,why=b:replace(plan.loc.where,plan.loc.index,plan.merged)
   if plan.from and plan.from.where=='bag' and ok then
    -- the merged drive is a bag drive that went into another: find it again, it is the one that was `from`
    b:discard(plan.from.index)
   end
   if ok then local name,line=self:merge_text(plan)
    self:log(('merged %s into %s (%s): %s now tier %d, %d merge(s)'):format(self:name(r),name,how,plan.info.affix,plan.info.to,plan.info.merges))
    self.hud:show_card('Merged!',{name,line},D.drive_text.rarity_colour[plan.merged.rarity]);self.hud:flash('Merged') end
  elseif plan.action=='equip' then
   if plan.from then ok,why=b:equip(plan.from.index,plan.slot);if ok then self:log(('equipped into slot %d: %s'):format(plan.slot,self:name(r))) end
   else ok,why=b:place(plan.slot,r);if ok then self:mark_new(r);self:log(('equipped into slot %d: %s (%s)'):format(plan.slot,self:name(r),how)) end end
  elseif plan.action=='bag' then
   ok,why=b:give(r);if ok then self:mark_new(r);self:log(('bagged %s (%s)'):format(self:name(r),how)) end
  else return nil,'choose' end
  if not ok then
   -- The rule engine takes a build change only at a drained event boundary (a hit this frame leaves events queued).
   if tostring(why):find('drained event boundary',1,true) then return nil,'busy' end
   self:log(how..' refused: '..tostring(why));return nil,plain(why)
  end
  self:emit('gain',plan.action,self:name(r));self:touch();return plan.action
 end
 -- A drive arrives (a pickup, an uncollected drop): merge, bag or free slot, else it waits for the player's choice.
 function H:gain(r,how)
  local plan=self:plan_gain(r)
  if plan.action=='choose' then self.decide[#self.decide+1]=r;self:log(('%s: bag full and nothing merges: %s waits for your choice'):format(how,self:name(r)));self:touch();return 'choose' end
  local action,why=self:apply_plan(plan,r,how)
  if why=='busy' then self.deferred[#self.deferred+1]={record=r,how=how,tries=0};self:log(('%s: %s waits for a quiet frame'):format(how,self:name(r)));return 'wait' end
  return action,why
 end
 -- Deferred gains (a pickup while events were queued) are retried each tick; at a stage clear they are settled for good:
 -- into the bag when it has room, else they wait for the player's choice. Nothing is lost.
 function H:flush_deferred(force)
  if #self.deferred==0 then return end
  local list=self.deferred;self.deferred={}
  for _,it in ipairs(list) do
   it.tries=it.tries+1
   if not force and #self.mods.engine.queue>0 and it.tries<300 then self.deferred[#self.deferred+1]=it
   elseif force and #self.mods.engine.queue>0 then
    local b=self:bag()
    if #b.items<b:capacity() and b:give(it.record) then self:mark_new(it.record);self:touch();self:log('bagged '..self:name(it.record)..' (delayed, forced)')
    else self.decide[#self.decide+1]=it.record;self:touch() end
   else
    local action=self:gain(it.record,it.how..' (delayed)');if action=='wait' then self.deferred[#self.deferred]=nil;self.deferred[#self.deferred+1]=it end
   end
  end
 end
 -- The reward screen's A on an offered drive (and the timeout): merge, else free slot, else bag, else "choose".
 -- mode 'bag' (the X button) keeps it in the bag without merging or equipping.
 function H:take_offer(i,mode)
  local r=self.offers[i];if not r then return nil,'That drive is no longer on offer.' end
  local plan
  if mode=='bag' then
   local b=self:bag();if #b.items>=b:capacity() then return nil,'Your bag is full.' end;plan={action='bag'}
  else plan=self:plan_take(r) end
  if plan.action=='choose' then return nil,'choose' end
  local action,why=self:apply_plan(plan,r,'stage reward');if not action then return nil,why end
  self:emit('take_offer',i,self:name(r))
  local others={};for j,o in ipairs(self.offers) do if j~=i then others[#others+1]=self:name(o) end end
  self.offers={};self:touch()
  self:log(('took %s from the stage reward%s'):format(self:name(r),#others>0 and ('; declined '..table.concat(others,', ')) or ''))
  return action,plan
 end
 -- A bag drive's A (merge into a held drive, or equip into a free slot). Returns the action or nil and why / 'choose'.
 function H:use_bag_drive(i)
  local b=self:bag();local r=b.items[i];if not r then return nil,'That drive is not in the bag.' end
  local plan=self:plan_take(r,{where='bag',index=i})
  if plan.action=='choose' then return nil,'choose' end
  local action,why=self:apply_plan(plan,r,'bag');if not action then return nil,why end
  return action,plan
 end
 -- Replace a held drive with an incoming one (bag full, nothing merges). `swap` = {record=, from='offer'|'decide'|'bag', index=}.
 function H:replace_with(swap,where,index)
  local b=self:bag();local r=swap.record;local old=where=='equipped' and b.equipped[index] or b.items[index]
  if not old then return nil,'That drive is not available.' end
  local ok,why
  if swap.from=='bag' then
   if where~='equipped' then return nil,'Pick an equipped drive.' end
   ok,why=b:equip(swap.index,index)
   if not ok then self:log('swap refused: '..tostring(why));return nil,plain(why) end
   self:touch();self:log(('swapped out %s: %s now in slot %d, %s went to the bag'):format(self:name(old),self:name(r),index,self:name(old)))
   return 'swap',self:name(old)..' is in your bag.'
  end
  ok,why=b:replace(where,index,r)
  if not ok then self:log('replace refused: '..tostring(why));return nil,plain(why) end
  local lost=true
  if where=='equipped' and #b.items<b:capacity() then lost=not b:give(old) end
  self:emit('replace',self:name(r),self:name(old));self:mark_new(r);self:touch()
  self:log(('%s replaces %s in %s %d: %s'):format(self:name(r),self:name(old),where=='equipped' and 'slot' or 'bag place',index,lost and 'the old drive is gone' or 'the old drive went to the bag'))
  if swap.from=='offer' then
   local others={};for _,o in ipairs(self.offers) do if o~=r then others[#others+1]=self:name(o) end end
   self.offers={};self:log(('took %s from the stage reward%s'):format(self:name(r),#others>0 and ('; declined '..table.concat(others,', ')) or ''))
  elseif swap.from=='decide' then table.remove(self.decide,1) end
  return 'replace',lost and (self:name(old)..' is gone.') or (self:name(old)..' is in your bag.')
 end
 -- A drive that cannot be kept is left behind, said so.
 function H:leave_choice(why)
  local r=table.remove(self.decide,1);if r then self:emit('left_behind',self:name(r),why);self:log(('left behind %s (%s)'):format(self:name(r),why or 'player choice')) end;self:touch();return r
 end
 -- Keystones: the choice offered at a stage clear while the allowance has steps owed.
 function H:take_keystone(id)
  local ok,why=self:bag():choose_keystone(id)
  if not ok then self:log('keystone refused: '..tostring(why));return nil,plain(why) end
  self:emit('keystone',id);self:touch();local rule=self:keystone_rule(id);self:log('keystone chosen: '..(rule and rule.label or id));self.hud:flash('Keystone: '..(rule and rule.label or id))
  self:offer_keystones();return true
 end
 -- The try-it command (`envoy grant <id>`): take a keystone now, outside the offer. Same legality rules as a pick.
 function H:grant(id)
  local rule=self:keystone_rule(id)
  if not rule or rule.kind~='keystone' then self:log('grant refused: unknown keystone '..tostring(id));return false,'unknown keystone' end
  for _,held in ipairs(self:keystone_ids()) do if held==id then self:log('grant: already held '..id);return true end end
  local ok,why=self:bag():choose_keystone(id)
  if not ok then self:log('grant refused: '..tostring(why));return false,plain(why) end
  self:touch();self:log('grant: keystone '..rule.label);self.hud:flash('Keystone: '..rule.label);return true
 end
 -- Roll (or re-roll after a pick) the keystone choice when allowance steps are owed; the same stage always offers the same three.
 -- Every record id the build holds: equipped drives, bag drives and keystones (id -> true). Connecting offers and the grid's links read it.
 function H:held_ids()
  local b=self:bag();local out={}
  for slot=1,b:slots() do local r=b.equipped[slot];if r then for _,a in ipairs(r.affixes) do out[a.id]=true end end end
  for _,r in ipairs(b.items) do for _,a in ipairs(r.affixes) do out[a.id]=true end end
  for _,id in ipairs(self:keystone_ids()) do out[id]=true end
  return out
 end
 function H:connecting() return D.mod_tuning and D.mod_graph and D.mod_tuning.get('connect_offers')~=0 end
 -- When none of the offered drives connects to anything held, one is swapped for a roll pulled toward partners of a held piece (same rarity, seeded:
 -- the same stage always shows the same offers). Returns the index swapped, or nil.
 function H:connect_offers(base,ctx,rarity_of)
  if not self:connecting() then return nil end
  local k,r=D.mod_graph.connect_offers(D.mod_pool,self.mods.drives.loot,self.offers,self:held_ids(),base,ctx,rarity_of,function(try,k) return seed_for(self.seed,base,try,701+k+self:salt()) end)
  if k then self:log(('connecting offers: none of the offers connected to the build; offer %d became %s'):format(k,self:name(r))) end
  return k
 end
 function H:offer_keystones()
  local ctx=self.mods.engine.context;local held=self:keystone_ids()
  if D.keystones.owed(ctx,held)>0 then
   self.key_offers=D.keystones.offer(ctx,seed_for(self.seed,self.stage,self.loop,99+self:salt()),3,held,self:connecting() and {held=self:held_ids(),pool=D.mod_pool,graph=D.mod_graph} or nil)
   if #self.key_offers>0 then local names={};for _,id in ipairs(self.key_offers) do names[#names+1]=(self:keystone_rule(id) or {label=id}).label end;self:log('keystone offer: '..table.concat(names,' / ')) end
  else self.key_offers={} end
 end
 function H:decline_offers(reason)
  local names={};for _,o in ipairs(self.offers) do names[#names+1]=self:name(o) end
  if #names>0 then self:log(('%s the stage reward: none kept (%s)'):format(reason,table.concat(names,', '))) end
  if #self.key_offers>0 then self:log(('%s the keystone offer: it stays owed'):format(reason)) end
  self.offers={};self.key_offers={};self:touch()
 end
 -- ---- announcements ---------------------------------------------------------------------------------------------
 -- Milestones (a new slot, a keystone allowance, a drive tier, New Game+) are NOT announced in a match any more: when a stage starts they are
 -- remembered (self.pending_ms) and the NEXT between-stage screen carries them as ONE short line ("Unlocked: fifth slot, keystone allowance 2, drive tier 2").
 -- `milestone_parts` compares a context with what the run has already seen; `mark_milestones` records it when the stage starts.
 local function milestone_parts(self,ctx)
  local list={}
  local slots=D.mod_progression.slots(ctx);local keys=D.mod_progression.keystones(ctx);local tier=D.mod_progression.tier(ctx)
  if self.seen_slots and slots>self.seen_slots then list[#list+1]=({[5]='fifth slot',[6]='sixth slot'})[slots] or ('slot '..slots) end
  if self.seen_keys and keys>self.seen_keys then list[#list+1]='keystone allowance '..keys end
  if self.seen_tier and tier>self.seen_tier and not self.follower then list[#list+1]='drive tier '..tier end
  local ng=D.mod_progression.run_loop(self.retail.mode,ctx);if self.seen_loop and ng>self.seen_loop and not self.follower then list[#list+1]='New Game+ '..ng end
  return list,slots,keys,tier,ng
 end
 local function mark_milestones(self,ctx)
  local list,slots,keys,tier,ng=milestone_parts(self,ctx)
  self.seen_slots,self.seen_keys,self.seen_tier,self.seen_loop=slots,keys,tier,ng
  return list
 end
 -- ---- lifecycle -------------------------------------------------------------------------------------------------
 function H:run_begin(seed)
  -- Crits draw from the engine's generator; restart it from the run seed so a run is reproducible (never a default seed).
  if self.g.crit_seed and not self.follower then pcall(self.g.crit_seed,seed_for(seed,0,0,9)) end
  self.running=true;self.seed=seed;self.fell={};self.rolls={};self.stage=0;self.loop=0;self.drops=0
  -- a run's floor drives are collected by pressing A on them (collection "press"), not by walking over them (drive_drop.lua R:item_for)
  if self.mods.drives and self.mods.drives.drops and not self.follower then self.mods.drives.drops.press=true end
  -- A new run is not a retry of the last one's stages: the attempt counts and the stages' given drops were never cleared, so every run after the first
  -- in a game session found its stages 'already played' and gave no floor drive at all (found by the co-op campaigns; a solo run had it too).
  self.attempts={};self.drops_given={};self.retry=false;self.drop_queue=nil
  self.offers={};self.key_offers={};self.decide={};self.deferred={};self.new_keys={};self.kos=0;self.faded={};self.pending_ms={};self.milestone_line=nil;self.hud:clear();if self.synfx then self.synfx:reset() end;if self.screen.active then self.screen:close() end
  self.mods:run_end() -- a new run starts from an empty bag, whatever the last one left
  local dev=self.dev_pending;self.dev_pending=nil;self.floor=nil
  local ctx=D.mod_progression.context(0,0)
  if dev then
   ctx=D.mod_progression.run_context(self.retail.mode,dev.depth or 0,dev.loop or 0);self.floor=D.mod_progression.context(ctx)
   if dev.preset and D.mod_tuning and D.mod_tuning.preset then pcall(D.mod_tuning.preset,dev.preset) end
   self:log(('developer start: depth=%d loop=%d (effective %d, %d slots, keystone allowance %d)'):format(ctx.depth,ctx.loop,D.mod_progression.effective(ctx),D.mod_progression.slots(ctx),D.mod_progression.keystones(ctx)))
  end
  self.mods:set_context(ctx)
  self.seen_slots,self.seen_keys,self.seen_tier,self.seen_loop=D.mod_progression.slots(ctx),D.mod_progression.keystones(ctx),D.mod_progression.tier(ctx),D.mod_progression.run_loop(self.retail.mode,ctx)
  -- A starter drive and a starting keystone (one random one, from the run seed), so the first opponents already roll
  -- against a build: the drive goes straight into slot 1. One panel announces both.
  local record=self.mods.drives.loot:roll(seed_for(seed,0,0,5+self:salt()),ctx)
  self.starter=record
  local lines={}
  local idx=self:acquire(record,'starter')
  if idx then local ok,msg=self:equip(idx,1);self.starter_text=ok and msg or nil;self.new_keys={}
   lines[#lines+1]={text='Your starter drive',colour='gold'}
   lines[#lines+1]={text=self:name(record),colour=D.drive_text.rarity_colour[record.rarity]}
   lines[#lines+1]=D.drive_text.drive_lines(self.mods.drives.loot,record)[2] or ''
  end
  local kid=self.seat and self.seat.starting_keystone and self.seat.starting_keystone(self,seed) or D.keystones.starting(seed);local rule=self:keystone_rule(kid)
  local ok,why=self:bag():choose_keystone(kid)
  if ok and rule then
   self:touch();self.starting_keystone=kid;self:log('starting keystone: '..rule.label)
   local kl=D.drive_text.keystone_lines(rule,D.mod_progression.tier(ctx))
   local fam=D.keystones.family(kid)
   lines[#lines+1]={text='Your keystone: '..rule.label,colour=D.drive_text.base_colour[fam] or 'gold'}
   lines[#lines+1]=kl[1] or '';lines[#lines+1]=kl[2] or ''
  else self:log('starting keystone refused: '..tostring(why)) end
  if dev and dev.build then self:dev_fill(ctx,dev.build_seed or seed);self:log('developer build rolled at depth '..ctx.depth..': '..self:equipped_count()..' drives, '..#self:keystone_ids()..' keystones') end
  if #lines>0 then self.hud:announce(lines) end
  self:log('run begin seed='..seed)
 end
 -- A drive that arrives in the bag by the run itself (the starter): returns the bag index or nil + the reason.
 function H:acquire(r,how)
  local b=self:bag();local ok,why=b:give(r)
  if not ok then self:log(('discarded %s (%s): bag full'):format(self:name(r),how));return nil end
  self:mark_new(r);self:touch();return #b.items
 end
 -- The context of a stage: the run's own (stage, loop) context, except while a developer start (`envoy start ... depth=<n> loop=<n>`) holds a
 -- higher floor. The game's own stage counter cannot start past stage 1, so the floor is what makes the slots, keystone allowance and tier agree
 -- with the build that was rolled at that depth; it ends as soon as the run's own context catches up. The context never goes down (a lower
 -- one would take slots and keystones away).
 function H:context_for(stage,loop)
  local ctx=D.mod_progression.run_context(self.retail.mode,stage,loop);local f=self.floor
  if f then
   if D.mod_progression.effective(f)>D.mod_progression.effective(ctx) then return D.mod_progression.context(f) end
   self.floor=nil;self:log('developer depth floor reached by the run itself: the run counter takes over')
  end
  return ctx
 end
 -- A developer start: depth/loop floor and an optional seeded build (a full set of slots and keystones rolled at that depth).
 function H:dev_start(spec) self.dev_pending=spec end
 function H:dev_fill(ctx,seed)
  local b=self:bag();local drives=self.mods.drives
  for slot=1,b:slots() do if not b.equipped[slot] then
   local ok,why=b:place(slot,drives.loot:roll(seed_for(seed,0,slot,300+self:salt()),ctx))
   if not ok then self:log('dev build: slot '..slot..' refused: '..tostring(why)) end
  end end
  local held=self:keystone_ids()
  for i=1,D.mod_progression.keystones(ctx)-#held do
   local pick=D.keystones.offer(ctx,seed_for(seed,0,i,400+self:salt()),1,self:keystone_ids())[1]
   if not pick then break end
   local ok,why=b:choose_keystone(pick);if not ok then self:log('dev build: keystone '..tostring(pick)..' refused: '..tostring(why)) end
  end
  self:touch()
 end
 -- Progression follows the director's stage and NG+ loop; the bag's slots and keystones grow with it.
 function H:stage_start(e)
  if not self.running then return end
  self.stage=e.stage_index or 0;self.loop=e.loop or self.retail.loop or 0
  -- A retry (a continue after a game over keeps the ordinal) keeps the build; the stage's floor drop was already given once.
  self.attempts=self.attempts or {};local akey=self.loop..':'..self.stage;self.attempts[akey]=(self.attempts[akey] or 0)+1
  self.retry=self.attempts[akey]>1
  if self.retry then self:log(('stage %d retry (attempt %d): the build is kept as it was; a drop already given on this stage is not given twice'):format(self.stage,self.attempts[akey])) end
  self.fell={};self.hurt={};self.dropped={};self.since=0;self.kos=0;self.faded={}
  self.travel_min,self.travel_max=nil,nil;self.foe_seen=false
  self.stage_kind=e.stage_kind or 'battle';self.foe_ports={};self.rolled={};self.drop_queue={};self.hold_gave_up=false;self.holding_end=false;self.out_frames=0;self.last_blast=nil;self.oob={}
  do local P=D.drive_drop.payout;if self.seq then self:end_payout() end;self.seq=P.seq();self.leave_w=P.leave_watch();self.idle_w=P.idle_watch();self.paying=false;self.pay_idle=false;self.spots=nil;self.spot_i=0 end
  for _,o in ipairs(e.opponents or {}) do if o.port then self.foe_ports[#self.foe_ports+1]=o.port;self.rolled[o.port]=true;self.rolls[o.port]=true end end
  self.foe_seen=#self.foe_ports>0
  local ctx=self:context_for(self.stage,self.loop)
  self.mods:set_context(ctx)
  local passed=mark_milestones(self,ctx);self.pending_ms=self.pending_ms or {}
  if #passed>0 then for _,p in ipairs(passed) do self.pending_ms[#self.pending_ms+1]=p end;self:log('milestones (shown on the between-stage screen): '..table.concat(passed,', ')) end
  self.hud.m=nil;self.mods.drives:bump()
  self:log(('stage %d (%s) NG+%d context depth=%d slots=%d'):format(self.stage,self.stage_kind,self.loop,ctx.depth,D.mod_progression.slots(ctx)))
 end
 -- Opponents roll a build from the player's current strength, exactly as `foe roll` does.
 function H:spawn(e)
  if not self.running or not e or not e.port or e.port==self:port0() or self:is_ally(e.port) or self.follower then return end
  -- The spawn signal fires while the scene is still being built: only note the port here. The roll (which
  -- also warms the look shaders) is queued from the first logic frame, when the fighter is fully present.
  self:register_foe(e.port)
 end
 -- An opponent exists for this stage from the moment it appears (Adventure's side-scrollers and hordes spawn fighters after
 -- the stage starts): it is watched for drops, and its build is rolled ONCE per port per stage (a respawn on the same port
 -- keeps that build; the engine re-installs it).
 function H:register_foe(p)
  local known=false;for _,q in ipairs(self.foe_ports) do if q==p then known=true end end
  if not known then self.foe_ports[#self.foe_ports+1]=p;self:log('opponent P'..p..' joined the stage') end
  if not self.rolled[p] then self.rolled[p]=true;self.rolls[p]=true end
 end
 function H:roll_wanted()
  local foes=self.mods.foes;if not foes then return end
  local d=self.mods.drives
  for p in pairs(self.rolls) do if not self.g.player(p) then self.rolls[p]=nil;if foes.jobs then foes.jobs[p]=nil end;self:log('opponent roll dropped: P'..p..' is gone') end end
  -- Rolls scale to the player's published build: wait until the bag's build has reached the engine.
  if #d.pending>0 then return end
  if d:has_build() and (not d.applied or d:stale()) then return end
  for p in pairs(self.rolls) do
   local v=self.g.player(p)
   if v and v.cpu then
    local ok,why=pcall(function()
     if not (foes.jobs and foes.jobs[p]) then
      local strength=self.seat and self.seat.strength and self.seat.strength(self) or select(2,self.mods.engine:family_budget(1))
      foes:roll_begin(p,strength,seed_for(self.seed,self.stage,self.loop,p),self.stage,'normal',{drives=self:equipped_count(),keystones=#self:keystone_ids()})
     end
     return foes:roll_advance(p,H.tuning.roll_attempts)
    end)
    if not ok then self.rolls[p]=nil;self:log('opponent roll refused P'..p..': '..tostring(why))
    elseif why then self.rolls[p]=nil;self:log('queued opponent roll P'..p) end
   end
  end
 end
 -- ---- drops -----------------------------------------------------------------------------------------------------
 -- Rules (drive_economy.tuning): a battle, giant or metal stage drops at most `floor_max` (one) drive, with chance
 -- `floor_chance`, at its first trigger (an opponent passes drop_percent damage or loses a stock); a team stage drops one
 -- (`team_max`); a bonus stage has no opponents and drops none, the final boss none (their clears offer a pick of three).
 -- Drops are seeded by run seed, stage and defeat number. A drop never needs bag space: it merges, bags or asks.
 function H:drop_rule(kind)
  if kind=='bonus' or kind=='boss' then return 0,0 end
  if kind=='team' then return econ().team_max,1 end
  return econ().floor_max,econ().floor_chance
 end
 function H:on_ko(p,why,arrive)
  if self.stage_kind=='team' then if self.dropped[p] then return end;self.dropped[p]=true end -- one per opponent
  self.kos=self.kos+1
  self.drops_given=self.drops_given or {}
  if self.retry and self.drops_given[self.loop..':'..self.stage] then self:log(('opponent P%d: no drop on a retry (the stage already gave its drop)'):format(p));return end
  if not H.tuning.drops or not self.mods.drives then return end
  local max,chance=self:drop_rule(self.stage_kind)
  local d=self.mods.drives
  if self.kos>max then return end
  if chance<1 then
   local x=(seed_for(self.seed,self.stage,self.loop,p)+self.kos*48271)%2147483646+1
   for _=1,3 do x=(x*16807)%2147483647 end
   local roll=x/2147483647
   if roll>=chance then self:log(('opponent P%d dropped nothing'):format(p));return end
  end
  if d.drops:count()+(self.drop_queue and #self.drop_queue or 0)+(self.seq and self.seq:pending() or 0)>=12 then self:log('drop skipped: the ground is full');return end
  local record=d.loot:roll(seed_for(self.seed,self.stage,self.loop,p+self.kos*7),self.mods.engine.context)
  self.drop_queue=self.drop_queue or {}
  for _,rec in ipairs(self.seat and self.seat.drop_records and self.seat.drop_records(self,record,p) or {record}) do
   if arrive and self.seq then self.seq:add({record=rec,port=p,why=why}) -- the stage-end payout: it arrives with the flourish (H:arrive)
   else self.drop_queue[#self.drop_queue+1]={record=rec,port=p,tries=0,why=why or 'defeated'} end
  end
  self.drops_given[self.loop..':'..self.stage]=true
 end
 -- Where a drop appears: on the stage, on the floor a few steps from the player (the opponent may be off screen).
 function H:drop_position()
  local g=self.g;local port=self:port0()
  local v=g.player(port)
  if self.seat and self.seat.anchor_port then port=self.seat.anchor_port(self) or port;v=g.player(port) end
  if not v then return nil end
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
  elseif item.tries>=60 then table.remove(q,1);self:log(('drop failed for %s: %s'):format(self:name(item.record),tostring(why))) end
 end
 -- Called by the drive host after the player walked over a drive: merge, bag, free slot, or ask which to give up.
 function H:picked_up(r,from)
  self.hud.m=nil;if from then self:log('received '..self:name(r)..' from P'..from:port0()..' (the drop belongs to this player)');from.hud:flash('Passed it on') end
  if self.seat and self.seat.route then local to=self.seat.route(self,r);if to and to~=self then return to:picked_up(r,self) end end -- co-op: a drop that belongs to the other player is handed over
  local d=self.mods.drives;local action=self:gain(r,'pickup')
  if action=='merge' then -- apply_plan showed the merge card
  elseif action=='wait' then self.hud:show_card('Picked up: '..D.drive_text.short(d.loot,r),{'Being added to your build.'},D.drive_text.rarity_colour[r.rarity])
  elseif action=='choose' then
   -- Never a screen in the middle of a fight: the drive waits (shown on the strip) and is decided at the stage clear.
   self.hud:show_card('Bag full: '..D.drive_text.short(d.loot,r),{('Held for the stage end (%d waiting).'):format(#self.decide)},D.drive_text.rarity_colour[r.rarity])
   self.hud:flash('Drive waiting')
  elseif action then
   self.hud:show_card('Picked up: '..D.drive_text.short(d.loot,r),{action=='equip' and 'Equipped.' or 'In your bag (Z+START).'},D.drive_text.rarity_colour[r.rarity])
   self.hud:flash('+ drive')
  end
  self:log('picked up '..self:name(r)..(action and (' ('..action..')') or ''));self:update_hold()
 end
 function H:expired(r,why) self.faded[#self.faded+1]=r;self:log('a drive faded from the floor: '..self:name(r)..' (it is gathered at the stage end)');self:update_hold() end
 -- ---- the match does not end while a drive is still to be picked up ----------------------------------------------
 -- gd.match_end_hold (engine): while a drop is on the floor (or queued to appear) and the player is alive, a stock-based
 -- end of the stage is deferred, so the player keeps control until every drive is collected; then the stage ends as usual and
 -- the reward screen follows. Not on bonus/boss stages (nothing drops there). Ceiling: end_hold_frames (30 s of logic
 -- frames) after the last opponent is out, after which what is left is gathered as it always was. A drop that fell off
 -- the stage fades (H:expired) and is gathered at the stage's end, so nothing can trap the player.
 H.HOLD='envoy-drives'
 function H:set_hold(on,why)
  local g=self.g;if not g.match_end_hold then return end
  if (on and true or false)==(self.holding_end and true or false) then return end
  self.holding_end=on and true or false
  if not on then self.hold_banner=nil end
  if g.match_end_hold(H.HOLD,self.holding_end) then
   self:log(on and ('match end held: '..(self.hold_banner and 'collect the drives' or 'the stage end waits for the payout')) or ('match end released'..(why and (': '..why) or '')))
  end
  if not on then self.out_frames=0 end
 end
 function H:foes_out()
  local port0=self:port0()
  if #self.foe_ports==0 then return false end
  for _,p in ipairs(self.foe_ports) do
   if p~=port0 and not self:is_ally(p) then local v=self.g.player(p);if v and (v.stocks or 0)>0 then return false end end
  end
  return true
 end
 -- The "collect the drives" hold: a banner (the strip's small flash text was easy to miss) and a marker on the nearest floor drive:
 -- a bobbing arrow over it when it is on screen, an edge arrow pointing toward it when it is not. Drawing only.
 local function tri(g,cx,cy,dir,size,colour)
  for i=0,size-1 do
   local half=size-1-i   -- wide at the base, one pixel at the tip
   if dir=='down' then g.fill(cx-half,cy+i,half*2+1,1,colour)
   elseif dir=='left' then g.fill(cx-i,cy-half,1,half*2+1,colour)
   elseif dir=='right' then g.fill(cx+i,cy-half,1,half*2+1,colour) end
  end
 end
 function H:draw_hold()
  local g=self.g;local k=g.kit
  if not (self.holding_end and self.hold_banner and k and g.safe_area) then return end
  local d=self.mods.drives;local list={}
  if d and d.drops and d.drops.records and g.items then local want={};for _,r in pairs(d.drops.records) do want[r.handle]=true end
   for _,e in ipairs(g.items() or {}) do if want[e.handle] and type(e.x)=='number' then list[#list+1]=e end end end
  local a=g.safe_area();local frame=self.since or 0;local f=(self.g.frame and self.g.frame()) or 0
  local text=self.hold_banner
  local w=300;local x=a.x+(a.w-w)//2
  g.fill(x,a.y+150,w,34,0x3A3320E8);g.fill(x,a.y+150,w,2,0xEBD175FF)   -- well below the match timer; the banner is the one instruction
  k.text(x+w//2,a.y+174,text,'body','gold','center')
  if self.paying then -- the way out: hold Z + D-pad Down
   g.fill(x,a.y+184,w,18,0x3A3320E8);k.text(x+w//2,a.y+197,'Hold Z + D-pad Down to leave','caption','gold','center')
   local pr=self.leave_w and self.leave_w:progress() or 0;if pr>0 then g.fill(x,a.y+201,math.floor(w*pr),2,0xEBD175FF) end
  end
  local me=g.player(self:port0());local best,bd
  for _,p in ipairs(list) do if me then local dd=math.abs(p.x-me.x)+math.abs(p.y-me.y);if not bd or dd<bd then best,bd=p,dd end end end
  if not best or not g.project then return end
  local ok,sx,sy,visible=pcall(g.project,best.x,best.y+4,0)
  if not ok or not sx then return end
  local gold=0xEBD175FF;local bob=math.floor(3*math.sin(f/6))
  local margin=28
  local on=visible and sx>=a.x+margin and sx<=a.x+a.w-margin and sy>=a.y+margin and sy<=a.y+a.h-margin
  if on then tri(g,math.floor(sx),math.floor(sy)-34+bob,'down',14,gold)
  else
   local cx=math.max(a.x+margin,math.min(a.x+a.w-margin,sx));local cy=math.max(a.y+90,math.min(a.y+a.h-90,sy))
   local dir=(sx<a.x+margin) and 'left' or 'right'
   if dir=='left' then cx=a.x+margin else cx=a.x+a.w-margin end
   g.fill(cx-30,cy-22,60,44,0x1B1A12D8)
   tri(g,math.floor(cx+(dir=='left' and -2 or 2))+(dir=='left' and 0 or 0),math.floor(cy),dir,14,gold)
  end
 end
 -- ---- the stage-end payout --------------------------------------------------------------------------------------------------
 -- The end of a battle stage is ALWAYS held (gd.match_end_hold) while the player lives, so a stage cleared faster than any drop could
 -- appear still ends the same way: when the last opponent is out, the drops the stage still OWES (decisions the fight never made: see
 -- drive_drop.lua P.owed_foes) are decided with the same seeds as a mid-fight drop, and arrive on the stage one after another with a flourish
 -- (P.flourish) on the main floor, as ordinary drives the player collects with A. The hold ends when every drive is collected, or the player
 -- leaves (holds Z + D-pad Down for a second: what is left is skipped), or the player is out (the stage is lost the normal way: no payout), or
 -- the player is away from the pad for two minutes (what is left is gathered, as it always was). There is no timer to run out while the player plays.
 -- Not on bonus or boss stages (nothing is owed there, and Master Hand's own end must not be held: PROGRESS.md of 2026-10-05), nor in a run with
 -- `payout` off (the older hold, below, which only waits while a mid-fight drop is on the floor).
 function H:foes_list()
  local port0=self:port0();local out={}
  for _,p in ipairs(self.foe_ports) do if p~=port0 and not self:is_ally(p) then out[#out+1]=p end end
  return out
 end
 function H:begin_payout()
  local P=D.drive_drop.payout;self.paying=true
  local kind=self.stage_kind;local max=self:drop_rule(kind)
  local given=self.retry and self.drops_given~=nil and self.drops_given[self.loop..':'..self.stage]==true
  local owed=P.owed_foes(kind,max,self:foes_list(),{kos=self.kos,dropped=self.dropped,given=given})
  self:log(('the last opponent is out: %d drive decision(s) owed%s'):format(#owed,given and ' (a retry: the stage already gave its drop)' or ''))
  for _,p in ipairs(owed) do self:on_ko(p,'owed at the clear',true) end
  local n=self.seq:pending()
  if n>0 then
   local anchor=self.seat and self.seat.anchor_port and self.seat.anchor_port(self) or self:port0()
   self.spots=P.spots(n,P.area(self.g,self.g.player(anchor)));self.spot_i=0
   self.hud:flash('The stage pays out')
   self:log(('payout: %d drive(s) arrive, %d spot(s) planned on the main floor'):format(n,#self.spots))
  end
 end
 function H:arrive(e)
  local P=D.drive_drop.payout;local d=self.mods.drives
  local spot=self.spots and self.spots[(self.spot_i or 0)+1];local x,y
  if spot then x,y=spot.x,spot.y+12 else x,y=self:drop_position() end
  local ok,why=false,'no position'
  if x then ok,why=pcall(function() return d.drops:spawn(e.record,x,y+P.tuning.arrive_height,{payout=true}) end) end
  if ok then
   self.spot_i=(self.spot_i or 0)+1
   P.flourish(self.g,self.seq,x,y-12,e.record)
   self:log(('payout: %s arrives at x=%.0f (%s)'):format(self:name(e.record),x,e.why or 'owed'))
   self.hud:flash('A drive arrived')
  else
   e.tries=(e.tries or 0)+1
   if e.tries>=60 then self.faded[#self.faded+1]=e.record;self:log(('payout: %s could not arrive (%s): it is gathered at the stage end'):format(self:name(e.record),tostring(why)))
   else table.insert(self.seq.queue,1,e);self.seq.wait=10 end
  end
 end
 -- Called every logic frame by H:frame: the arrivals' pacing and effects, and the leave / away watches (raw pads of the player's ports).
 function H:payout_frame()
  if not self.paying or not self.seq then return end
  local g=self.g;local P=D.drive_drop.payout;self.seq:tick_fx(g)
  if g.pad then
   local ports={self:port0()};if self.seat and self.seat.allies then for p in pairs(self.seat.allies) do ports[#ports+1]=p end end
   local active,last
   for _,p in ipairs(ports) do local pad=g.pad(p,true);if pad then last=pad;self.leave_w:feed(pad,1)
     if (pad.buttons or 0)~=0 or math.abs(pad.x or 0)>20 or math.abs(pad.y or 0)>20 or math.abs(pad.cx or 0)>20 or math.abs(pad.cy or 0)>20 then active=pad end end end
   if self.idle_w:feed(active or last,1) then self.pay_idle=true end
  end
  local e=self.seq:step();if e then self:arrive(e) end
 end
 -- The payout ends (collected, left, lost, run end): its transient effects stop and what has not arrived is returned (and never lost silently).
 function H:end_payout()
  local left=self.seq and self.seq:clear(self.g) or {}
  self.paying=false;return left
 end
 function H:update_payout_hold()
  local P=D.drive_drop.payout;local k=self.stage_kind;local d=self.mods.drives
  if not self.running or self.screen.active or not d or k=='bonus' or k=='boss' or self.hold_gave_up or #self.foe_ports==0 then return self:set_hold(false) end
  local me=self.g.player(self:port0())
  if self.seat and self.seat.team_alive then me=self.seat.team_alive(self) end -- co-op: any living teammate keeps the hold
  local alive=me~=nil and (me.stocks or 0)>0
  if not alive then
   local lost=self:end_payout();if #lost>0 then self:log(('the player is out: %d owed drive(s) are not paid (the stage is lost)'):format(#lost)) end
   return self:set_hold(false,'the player is out')
  end
  if not self.paying and self:foes_out() then self:begin_payout() end
  local floor=d.drops:count();local queued=(self.drop_queue and #self.drop_queue or 0)+(self.seq and self.seq:pending() or 0)
  if floor+queued>0 then
   if not self.hold_banner then self.hud:flash('Collect the drives') end
   self.hold_banner=(self.paying and floor==0) and 'The stage pays out' or 'Collect the drives'
  else self.hold_banner=nil end
  self:set_hold(true) -- the stage's end is always held while the player lives (the log line says why)
  if not self.paying then return end
  local why=P.release_reason{floor=floor,queued=queued,alive=alive,left=self.leave_w.done,idle=self.pay_idle==true}
  if not why then return end
  if why=='the player left' then
   local skipped=self:end_payout()
   for _,rec in pairs(d.drops.records) do skipped[#skipped+1]={record=rec.record} end
   for _,it in ipairs(self.drop_queue or {}) do skipped[#skipped+1]=it end
   for _,r in ipairs(self.faded) do skipped[#skipped+1]={record=r} end
   for _,it in ipairs(skipped) do self:log('payout: skipped '..self:name(it.record)..' (the player left)') end
   self.drop_queue={};self.faded={};d.drops:clear();self.hold_gave_up=true
  elseif why=='the player is away' then
   for _,it in ipairs(self:end_payout()) do self.drop_queue[#self.drop_queue+1]={record=it.record,tries=0,why='away'} end
   self:log('payout: the player is away: what is left is gathered at the stage end');self.hold_gave_up=true
  else self:end_payout() end
  self:set_hold(false,why)
 end
 function H:update_hold()
  if not H.tuning.end_hold or not self.g.match_end_hold then return end
  if H.tuning.payout then return self:update_payout_hold() end -- the stage-end payout (above); `payout=false` keeps the older hold below
  local k=self.stage_kind
  local d=self.mods.drives
  if not self.running or self.screen.active or not d or k=='bonus' or k=='boss' or self.hold_gave_up then return self:set_hold(false) end
  local floor=d.drops:count()+(self.drop_queue and #self.drop_queue or 0)
  local port0=self:port0()
  local me=self.g.player(port0)
  if self.seat and self.seat.team_alive then me=self.seat.team_alive(self) end -- co-op: any living teammate keeps the hold
  if floor==0 then return self:set_hold(false,'every drive collected') end
  if not me or (me.stocks or 0)<=0 then return self:set_hold(false,'the player is out') end
  if not self.holding_end then self.hud:flash('Collect the drives') end
  self.hold_banner='Collect the drives'
  self:set_hold(true)
  if self:foes_out() then
   self.out_frames=(self.out_frames or 0)+H.tuning.ko_poll
   local left=math.max(0,H.tuning.end_hold_frames-self.out_frames)
   self.hold_banner=('Collect the drives  %d s'):format(math.ceil(left/60))
   if self.out_frames%30<H.tuning.ko_poll then self.hud:flash(('Collect the drives (%d s)'):format(math.ceil(left/60))) end
   if left<=0 then
    self.hold_gave_up=true
    self:log(('match end hold ceiling (%d s): %d drive(s) are gathered'):format(H.tuning.end_hold_frames//60,floor))
    self.hud:flash('Out of time: drives are gathered')
    self:set_hold(false,'ceiling')
   end
  end
 end
 -- Drops still on the ground when the stage clears: through the gain rule (merge, bag, free slot, else asked) or lost, said either way.
 function H:collect_ground()
  local d=self.mods.drives;local list={}
  for _,rec in pairs(d.drops.records) do list[#list+1]=rec.record end
  for _,r in ipairs(self.faded) do list[#list+1]=r end
  self.faded={};if self.drop_queue then
   for _,it in ipairs(self.drop_queue) do list[#list+1]=it.record end;self.drop_queue={}
  end
  if self.seat and self.seat.gather then list=self.seat.gather(self,list) end -- co-op: the shared ground is divided between the seats
  for _,r in ipairs(list) do
   if H.tuning.auto_collect then
    local action=self:gain(r,'uncollected')
    if action then self:log('collected an uncollected drop: '..self:name(r)..' ('..action..')') end
   else self:log('discarded '..self:name(r)..' (uncollected, auto_collect off)') end
  end
  d.drops:clear()
 end
 function H:frame()
  if not self.running then return end
  self.since=self.since+1;self.hud:frame();if not self:ready() then return end
  self.hud:watch(self.mods.engine)
  if self.synfx then local ok,err=pcall(self.synfx.frame,self.synfx,self.mods.engine,self:port0());if not ok and not self.synfx_failed then self.synfx_failed=true;self:log('synergy fx frame failed: '..tostring(err)) end end
  local port0=self:port0()
  -- any CPU fighter that has appeared since the stage began is an opponent too (a spawn event may not have named it). Seen on the
  -- frame it exists, and its roll starts at once (a fast kill must not beat the roll).
  if not self.follower then for p=1,6 do if p~=port0 and not self:is_ally(p) and not self.rolled[p] then local v=self.g.player(p);if v and v.cpu then self:register_foe(p);self:roll_wanted() end end end end
  local me=self.g.player(port0)
  if me and type(me.x)=='number' then self.travel_min=math.min(self.travel_min or me.x,me.x);self.travel_max=math.max(self.travel_max or me.x,me.x) end
  if #self.foe_ports>0 then self.foe_seen=true end
  if self.follower then return end
  self:payout_frame()
  if self.since%H.tuning.ko_poll~=0 then return end
  self:oob_watch()
  for _,p in ipairs(self.foe_ports) do
   if p~=port0 and not self:is_ally(p) then
    local v=self.g.player(p)
    if v then
     local falls=v.falls or 0;local before=self.fell[p] or 0 -- a stage's fighters start with no falls
     if falls>before then self:on_ko(p,'lost a stock') end
     self.fell[p]=falls
     if not self.hurt[p] and (v.percent or 0)>=H.tuning.drop_percent then self.hurt[p]=true;self:on_ko(p,'passed '..H.tuning.drop_percent..'%') end
    end
   end
  end
 self:update_hold()
 end
 -- ---- out-of-bounds watchdog ------------------------------------------------------------------------------------
 -- The retail rule is that a fighter that crosses a blast zone loses a stock at once. If one is still alive, far beyond the zone, for
 -- oob_frames, the game has stopped applying that rule (the 2026-10-05 boss-end state made the player immune to it while the fight
 -- was waiting on a boss that could not die). This applies the rule instead: lose a stock, and put a fighter that has stocks left
 -- back on the stage. Logged and flashed, never silent. A fighter in debug flight is exempt (it is meant to leave the stage), and so are the
 -- boss hands (their entrances and attacks leave the screen by design; a boss's stocks are the fight's own business).
 function H:oob_box()
  local g=self.g;local b=g.stage_bounds and g.stage_bounds();local z=b and b.blast
  if type(z)=='table' and type(z.left)=='number' and type(z.right)=='number' and type(z.bottom)=='number' and type(z.top)=='number' then
   self.last_blast={left=z.left,right=z.right,bottom=z.bottom,top=z.top,floor=b.main_floor,origin=b.origin}
  end
  return self.last_blast -- the last bounds seen this stage when the engine gives none
 end
 function H:oob_outside(v,box)
  local m=H.tuning.oob_margin;local f=H.tuning.oob_fallback
  if box then return v.x<box.left-m or v.x>box.right+m or v.y<box.bottom-m or v.y>box.top+m end
  return math.abs(v.x)>f or math.abs(v.y)>f
 end
 function H:oob_resolve(p,v,box)
  local g=self.g;local left=(v.stocks or 0)-1
  self:log(('P%d is out of bounds (x=%.0f y=%.0f) and was never knocked out: losing a stock (%d left)'):format(p,v.x,v.y,math.max(left,0)))
  self.hud:flash(p==self:port0() and 'Out of bounds: a stock is lost' or ('P'..p..' out of bounds: a stock is lost'),true)
  if g.set_stocks then g.set_stocks(p,math.max(left,0)) end
  if left>0 and g.teleport then
   local fl=box and box.floor;local x=fl and (fl.left+fl.right)/2 or (box and box.origin and box.origin.x) or 0
   local y=(fl and fl.top or (box and box.origin and box.origin.y) or 0)+40
   g.teleport(p,x,y)
  end
 end
 function H:oob_watch()
  local g=self.g;if not g.player then return end
  local box=self:oob_box();local step=H.tuning.ko_poll;self.oob=self.oob or {}
  local ports={self:port0()};for _,p in ipairs(self.foe_ports) do if p~=ports[1] and not self:is_ally(p) then ports[#ports+1]=p end end
  for _,p in ipairs(ports) do
   local v=g.player(p)
   if v and type(v.x)=='number' and type(v.y)=='number' and (v.stocks or 0)>0 and not v.hidden and not (g.fly and g.fly(p)==true) and v.char~=26 and v.char~=27 and self:oob_outside(v,box) then
    self.oob[p]=(self.oob[p] or 0)+step
    if self.oob[p]>=H.tuning.oob_frames then self.oob[p]=0;self:oob_resolve(p,v,box) end
   else self.oob[p]=0 end
  end
 end
 function H:ready() return self.running and self.since>=H.tuning.settle_frames end
 -- ---- reward moment ---------------------------------------------------------------------------------------------
 -- Does this clear owe a drive reward? Every bonus stage, the boss, the final, and every `reward_every`-th stage.
 H.tuning.idle_travel=120
 function H:idle_clear(kind,final)
  if final or kind=='bonus' or kind=='boss' or self.foe_seen then return false end
  return ((self.travel_max or 0)-(self.travel_min or 0))<H.tuning.idle_travel
 end

 function H:reward_due(stage,final)
  if final then return true end
  if self.seat and self.seat.reward_due then return self.seat.reward_due(self,stage,final) end
  local _,reward=D.drive_economy.stage(econ(),self.stage_kind,stage)
  return reward>0
 end
 -- A cleared stage: gather what is on the floor, roll the offers (drives when a reward is due, keystones when the
 -- allowance owes a step), hold the barrier and show the grid. Nothing to show: sort quietly and let the run go on.
 function H:stage_reward(stage,loop,final)
  if not self.running or not self.mods.drives then return false end
  self:set_hold(false,'stage clear');if self.seq then self:end_payout() end
  self:flush_deferred(true)
  self:collect_ground()
  local kind=self.stage_kind
  local d=self.mods.drives;local ctx=self.mods.engine.context
  self.offers={}
  -- A stage with no opponent that ended while the player stood still (Escape from Brinstar times out and retail moves on
  -- either way) is not earned: no drive and no keystone step (it stays owed for the next clear). Retail's own flow is untouched.
  if self:idle_clear(kind,final) then
   self:log(('stage clear (%s): no reward, the player stood still (%.0f units of travel, no opponent); the keystone step stays owed'):format(kind,(self.travel_max or 0)-(self.travel_min or 0)))
   self.key_offers={};self:settle();return false
  end
  if self:reward_due(stage,final) then
   local many=final or kind=='bonus' or kind=='boss'
   local n=many and econ().bonus_offers or econ().offers
   local forced=final and {'rare','rare','unique'} or nil
   for i=1,n do self.offers[i]=d.loot:roll(seed_for(self.seed,stage,loop,(final and 11 or 7)+i*13+self:salt()),ctx,forced and forced[i] or D.drive_economy.reward_rarity(ctx,i)) end
   self:connect_offers(seed_for(self.seed,stage,loop,3+self:salt()),ctx,function(i) return forced and forced[i] or D.drive_economy.reward_rarity(ctx,i) end)
   local names={};for _,o in ipairs(self.offers) do names[#names+1]=self:name(o) end
   self:log(('stage clear (%s%s): offers %s'):format(kind,final and ', final' or '',table.concat(names,' / ')))
  else self:log(('stage clear (%s): no drive reward this stage'):format(kind)) end
  self:offer_keystones()
  local newn=0;for _ in pairs(self.new_keys) do newn=newn+1 end
  self:log(('reward moment: %d offered, %d keystones offered, %d waiting, %d new'):format(#self.offers,#self.key_offers,#self.decide,newn))
  if #self.offers==0 and #self.key_offers==0 and #self.decide==0 then self:settle();return false end
  -- what unlocked since the last between-stage screen (a new slot, keystone allowance, drive tier or New Game+): one line on this screen
  self.milestone_line=(self.pending_ms and #self.pending_ms>0) and ('Unlocked: '..table.concat(self.pending_ms,', ')) or nil;self.pending_ms={}
  local ok=self:hold_open()
  if ok then self.screen:open('reward');self.holding=true
  else self:log('reward hold unavailable: sorting the drives automatically');self:finish_reward('no-hold') end
  return true
 end
 -- New drives fill free slots (the bag keeps the rest); said once at the end of a stage's moment.
 function H:settle()
  local b=self:bag();local i=1
  while i<=#b.items do
   local r=b.items[i];local free=self:free_slot()
   if self:is_new(r) and free then local ok=self:equip(i,free);if not ok then i=i+1 end
   else i=i+1 end
  end
  self.new_keys={}
 end
 -- Every end of the reward moment ends here. `reason`: done (the player continued), timeout, released (the
 -- engine's own hold ended first), no-hold, run-end. Nothing is dropped without a log line.
 function H:finish_reward(reason)
  local auto=reason~='done'
  if #self.offers>0 then
   if reason=='run-end' then self:decline_offers('run ended during')
   else
    local action=self:take_offer(1)
    if action then self:log(('%s: took the first offer automatically (%s)'):format(reason,action))
    else self:decline_offers(reason) end
   end
  end
  if #self.key_offers>0 then self:log(('%s: the keystone offer was not taken; it stays owed'):format(reason));self.key_offers={} end
  while #self.decide>0 do self:leave_choice(reason) end
  if auto and reason~='run-end' then self:settle() end
  self.new_keys={};self.holding=false
  self:log(('reward moment done (%s): %d slots filled, %d in the bag'):format(reason,self:equipped_count(),#self:bag().items))
  if reason~='released' then self:hold_release(reason) end
  self.hud:flash('Build ready')
 end
 function H:press(a) if self.screen.active then self.screen:press(a) end end
 -- The reward moment's barrier: a retail run claims the engine's interstage hold; a co-op run's coordinator owns its own (pause + sequence of screens).
 function H:hold_open() if self.seat and self.seat.barrier then return self.seat.barrier.open(self) end;return self.g.hold_1p and self.g.hold_1p(H.tuning.hold_ticks) end
 function H:hold_release(reason) if self.seat and self.seat.barrier then return self.seat.barrier.release(self,reason) end;if self.g.release_1p then self.g.release_1p() end end
 -- Rolling a build and validating a bag edit are each close to a script call's whole budget (2M instructions,
 -- 50 ms), and on_frame already carries the host's own work: staging happens in on_tick, a call of its own.
 -- Script cost samples (wall clock, a ring of the last 240 calls) of what the screens and the strip cost per frame / tick: `uxcost`.
 local RING=240
 local function sample(self,name,t0)
  local g=self.g;if not (g.time and t0) then return end
  local c=self.cost;if not c then c={};self.cost=c end
  local r=c[name];if not r then r={n=0,max=0};c[name]=r end
  local ms=(g.time()-t0)*1000;r.n=r.n+1;r[(r.n-1)%RING+1]=ms;if ms>r.max then r.max=ms end
 end
 function H:cost_report()
  local out={}
  for name,r in pairs(self.cost or {}) do
   local n=math.min(r.n,RING);local sum=0;local sorted={};for i=1,n do sum=sum+r[i];sorted[i]=r[i] end;table.sort(sorted)
   out[#out+1]=('uxcost: %s n=%d mean=%.3f ms p95=%.3f ms max=%.3f ms'):format(name,r.n,n>0 and sum/n or 0,sorted[math.max(1,math.ceil(n*.95))] or 0,r.max)
  end
  table.sort(out);return out
 end
 function H:tick()
  if self.screen.active then local t0=self.g.time and self.g.time();self.screen:tick();sample(self,'screen tick',t0) end
  if not self:ready() then return end
  local drives=self.mods.drives;if not drives then return end
  self:flush_deferred()
  if self.follower then return end
  self:spawn_queued()
  self:roll_wanted()
 end
 function H:draw()
  if not self.running then return end
  local t0=self.g.time and self.g.time()
  if self.screen.active then self.screen:draw();sample(self,'screen draw',t0);return end
  local m=self.g.match();if not (m and m.active) or not self:ready() or self.menu_up then return end
  self.hud:draw();sample(self,'strip draw',t0)
  if self.synfx then local t1=self.g.time and self.g.time();local ok,err=pcall(self.synfx.draw,self.synfx,self.mods.engine,self:port0());sample(self,'synergy draw',t1);if not ok and not self.synfx_failed then self.synfx_failed=true;self:log('synergy fx draw failed: '..tostring(err)) end end
  self:draw_hold()
 end
 function H:run_end()
  self.hold_banner=nil
  if not self.running then return end
  self:set_hold(false,'run end');if self.seq then self:end_payout() end
  if self.mods.drives and self.mods.drives.drops and not self.follower then self.mods.drives.drops.press=nil end
  if self.screen.active then self.screen:close() end
  if #self.offers>0 or #self.key_offers>0 or #self.decide>0 then self:finish_reward('run-end') end
  self.running=false;self.fell={};self.rolls={};self.mods:run_end();self.hud:clear();if self.synfx then self.synfx:reset() end;self:log('run end: bag and build cleared')
 end
 -- The model as text, for the console and the tests.
 function H:dump()
  local d=self.mods.drives;if not d then self.g.log('uxdump: no drive host');return end
  local b=d.bag;local loot=d.loot
  self.g.log(('uxdump: running=%s stage=%d (%s) loop=%d slots=%d bag=%d/%d offers=%d key_offers=%d waiting=%d pending=%d ground=%d new=%d'):format(tostring(self.running),self.stage,self.stage_kind,self.loop,b:slots(),#b.items,b:capacity(),#self.offers,#self.key_offers,#self.decide,#d.pending,d.drops:count(),(function() local n=0;for _ in pairs(self.new_keys) do n=n+1 end;return n end)()))
  local rolls={};for p in pairs(self.rolls) do rolls[#rolls+1]='P'..p end
  local disp=self.mods.display
  self.g.log(('uxdump: host ready=%s since=%d applied=%s stale=%s lab.enabled=%s rolls=[%s] display ready=%s error=%s note=%s'):format(tostring(self:ready()),self.since,tostring(d.applied),tostring(d:stale()),tostring(self.mods.enabled),table.concat(rolls,','),tostring(disp and disp.ready),tostring(disp and disp.error),tostring(disp and disp.note)))
  local ctx=b.context;local ids=self:keystone_ids()
  local kl={};for _,id in ipairs(ids) do kl[#kl+1]=id end
  self.g.log(('uxdump: depth=%d loop=%d allowance=%d keystones=[%s] owed=%d'):format(ctx.depth,ctx.loop,D.mod_progression.allowance(ctx),table.concat(kl,','),D.keystones.owed(ctx,ids)))
  for i=1,b:slots() do local r=b.equipped[i];self.g.log('uxdump: slot '..i..' = '..(r and (loot:name(r)..' ['..r.rarity..' '..r.colour..' mods='..#r.affixes..' merged='..(r.merged or 0)..']') or 'empty')) end
  for i,r in ipairs(b.items) do self.g.log('uxdump: bag '..i..' = '..loot:name(r)..' ['..r.rarity..' mods='..#r.affixes..' merged='..(r.merged or 0)..']'..(self:is_new(r) and ' (new)' or '')) end
  for i,r in ipairs(self.offers) do self.g.log('uxdump: offer '..i..' = '..loot:name(r)..' ['..r.rarity..' mods='..#r.affixes..']') end
  for i,id in ipairs(self.key_offers) do self.g.log('uxdump: keystone offer '..i..' = '..id) end
  for i,r in ipairs(self.decide) do self.g.log('uxdump: waiting choice '..i..' = '..loot:name(r)) end
  for _,l in ipairs(self.hud:dump()) do self.g.log('uxdump: '..l) end
  if self.screen.active then for _,l in ipairs(self.screen:dump()) do self.g.log('uxdump: '..l) end end
 end
 return H
end
