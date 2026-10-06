-- Loot adapter shares the modifier adapter's checkpoint and commit boundary.
return function(D)
 local V={tuning={persist=false}};V.__index=V
 local labels={damage_dealt='Damage dealt',launch_dealt='Launch dealt',damage_taken='Damage taken',launch_taken='Launch taken',speed='Speed'}
 function V.new(g,lab,opts)
  opts=opts or {}
  local self=setmetatable({g=g,lab=lab,pending={},seed=104729,port=opts.port or 1,shared_drops=opts.drops~=nil},V)
  self.loot=D.drive_loot.new(D.mod_pool)
  self.bag=D.drive_bag.new(self.loot,{context=lab.engine.context,persist=V.tuning.persist,preflight=function(mods,implicits,context)
   local probe=D.mod_engine.new(lab.engine.seed,D.mod_pool);probe:import(lab.engine:export());probe:set_context(context);probe:set_build(self.port,self:combined(mods),implicits);probe:native_rules(self.port);if lab.check_echo_capacity then lab:check_echo_capacity(probe)end;return true
  end})
  self.drops=opts.drops or D.drive_drop.new(g);self.menu=D.drive_menu.new(g,self)
  if self.port~=1 then return self end -- a co-op seat shares the console commands of seat 1
  g.command('drive',function(a) return self:command(a or '') end,'give|drop [rarity] [seed]')
  -- Balance harness hook (debug, LAB only): `simbag <file>` installs an encoded bag snapshot from the script data folder as the
  -- player's real build (the file holds what the offline run simulator produced with the real run rules), at the snapshot's own context.
  g.command('simbag',function(name)
   local ok,why=pcall(function()
    local allowed,reason=lab:allowed();assert(allowed,reason);assert(not lab:replaying(),'refused during rewind');assert(#lab.engine.queue==0,'wait for combat events to commit')
    local text=assert(g.data_read(tostring(name)),'no such data file');local snap=D.mod_codec.decode(text)
    local ctx=D.mod_progression.context(snap.context)
    lab:set_context(ctx);self.bag.config.context=ctx;assert(self.bag:restore(snap));self.pending={};self:apply();lab.enabled=true
    local _,strength=lab.engine:family_budget(1);g.log(('simbag: %s installed, depth %d loop %d, strength %.3f'):format(tostring(name),ctx.depth,ctx.loop,strength))
   end)
   if not ok then g.log('simbag: refused '..tostring(why));return false end;return true
  end,'debug: install an encoded bag snapshot (script data file) as the build')
  g.command('bag',function() local ok,why=lab:allowed();if not ok or lab:replaying() then g.log(why or 'bag edit refused during rewind');return false end;if self.opener and lab:hosted() then self.opener() else self.menu:open() end;return true end,'open drive bag')
  return self
 end
 function V:combined(mods,lab)
  lab=lab or self.lab
  local out=D.mod_codec.decode(D.mod_codec.encode(mods))
  local function merge(id,tier)
   local old=out[id];if not old then out[id]=D.mod_codec.decode(D.mod_codec.encode(tier));return end
   if type(old)~='table' and type(tier)~='table' then out[id]=math.max(old,tier);return end
   local levels,extra=D.mod_schema.instances(old),D.mod_schema.instances(tier);local order={}
   for i in ipairs(levels) do order[#order+1]=i end;table.sort(order,function(a,b)return levels[a]>levels[b]end);table.sort(extra,function(a,b)return a>b end)
   for i,n in ipairs(extra) do local at=order[i] or #levels+1;levels[at]=math.max(levels[at] or 0,n) end
   local highest=0;for _,n in ipairs(levels) do highest=math.max(highest,n) end
   out[id]={tier=highest,copies=#levels,tiers=levels}
  end
  for id,tier in pairs((lab.debug_equipped or {})[self.port] or {}) do merge(id,tier) end
  for _,e in ipairs(lab.pending or {}) do if e.port==self.port then merge(e.id,1) end end
  return out
 end
 function V:view()
  local draft=D.drive_bag.new(self.loot,self.bag.config);assert(draft:restore(self.bag:snapshot()))
  for _,e in ipairs(self.pending) do assert(draft[e.op](draft,e.a,e.b)) end
  return draft
 end
 -- `rev` changes whenever anything the bag screen shows changes (queued edits, applied edits, restore, clear, pickup).
 function V:bump() self.rev=(self.rev or 0)+1 end
 function V:reserved() return math.max(#self.bag.items,#self:view().items)+self.drops:count() end
 function V:queue(op,a,b)
  local allowed,why=self.lab:allowed();if not allowed or self.lab:replaying() then self.menu.notice=why or 'bag edit refused during rewind';return false,self.menu.notice end
  if #self.pending>=12 then self.menu.notice='drive edit queue full';return false,self.menu.notice end;local draft=self:view()
  if op~='choose_keystone' and self.drops:count()>0 then self.menu.notice='Collect ground drops before editing bag';return false,self.menu.notice end
  if (op=='give' or op=='unequip') and #draft.items+self.drops:count()>=self.bag:capacity() then return false,'bag full (ground drops reserve space)' end
  local ok,why=draft[op](draft,a,b);if not ok then self.menu.notice=tostring(why);return false,why end
  self.pending[#self.pending+1]={op=op,a=a,b=b};self:bump();self.lab.enabled=true;return true
 end
 function V:command(arg)
  local ok,why=pcall(function()
   local allowed,reason=self.lab:allowed();assert(allowed,reason);assert(not self.lab:replaying(),'drive edit refused during rewind')
   local w={};for v in arg:gmatch('%S+') do w[#w+1]=v end
   assert((w[1]=='give' or w[1]=='drop') and #w<=3,'usage: drive give|drop [rarity] [seed]')
   if w[1]=='drop' then assert(not (self.g.paused and self.g.paused()),'resume gameplay before dropping; wait one checkpoint before saving');assert(#self.pending==0,'wait for queued bag edits to commit before dropping') end
   local seed=tonumber(w[3] or self.seed);assert(seed and seed%1==0,'integer seed required')
   local record=self.loot:roll(seed,self.lab.engine.context,w[2] and w[2]:lower());assert(self:reserved()<self.bag:capacity(),'bag full (ground drops reserve space)')
   self.lab.display:warm(self.lab.engine);assert(not self.lab.display.error,'shader warmup unavailable')
   if w[1]=='give' then assert(self:queue('give',record)) else self.drops:spawn(record);self.lab.enabled=true end
   if self.lab.options.activate then self.lab.options.activate() end
   if not w[3] then self.seed=(self.seed*16807)%2147483647 end
   self.g.log('drive: '..w[1]..' '..self.loot:name(record))
  end)
  if not ok then self.g.log('drive: refused '..tostring(why));return false,why end;return true
 end
 function V:apply()
  for _,e in ipairs(self.pending) do
   local ok,why
   if e.op=='give' and #self.bag.items+self.drops:count()>=self.bag:capacity() then ok,why=false,'bag full (ground drops reserve space)'
   elseif e.op=="unequip" and #self.bag.items+self.drops:count()>=self.bag:capacity() then ok,why=false,"bag full (ground drops reserve space)"
   else ok,why=self.bag[e.op](self.bag,e.a,e.b) end
   if not ok then self.g.log('bag: refused '..tostring(why)) end
  end
  self.pending={};self:bump();self.applied=true;local b,e=self.bag,self.lab.engine;self.stamp={equipped=b.equipped,keystone=b.keystone,keystones=b.keystones,depth=b.context.depth,loop=b.context.loop,engine=e,edepth=e.context.depth,eloop=e.context.loop}
  local mods,implicits=self.bag:derive();mods=self:combined(mods);self.lab.engine:set_build(self.port,mods,implicits);if not next(mods) then self.lab.engine.equipped[self.port]=nil end;if not next(implicits) then self.lab.engine.implicits[self.port]=nil end
  self.lab.engine.display.drive_build=self.lab.engine.display.drive_build or {};local looks={};for i=1,self.bag:slots() do local r=self.bag.equipped[i];if r then looks[#looks+1]={colour=r.colour,rarity=r.rarity} end end;self.lab.engine.display.drive_build[self.port]=looks
 end
 function V:budget_lines()
  local mods,implicit=self:view():derive();local families,strength=D.mod_budget.build(D.mod_pool,self:combined(mods),implicit,self.lab.engine.statuses[1])
  local c=self.bag.context;local lines={('Strength %.2f / depth %d loop %d / %d slots, %d keystones'):format(strength,c.depth,c.loop,self.bag:slots(),D.mod_progression.keystones(c))}
  for _,f in ipairs(D.mod_budget.order) do if ({damage_dealt=true,launch_dealt=true,damage_taken=true,launch_taken=true,speed=true})[f] then
   local c=D.mod_budget.caps[f];lines[#lines+1]=('%s %.2fx / safety %.2f..%.2fx'):format(labels[f],families[f].value,1+c[1],1+c[2])
  end end;return lines
 end
 function V:delta(index,slot)
  local draft=self:view();local before,base=draft:derive();before=self:combined(before);local ok=draft:equip(index,slot)
  local out={};if ok then
   local after,implicit=draft:derive();after=self:combined(after);local old=D.mod_budget.build(D.mod_pool,before,base,{});local new,strength=D.mod_budget.build(D.mod_pool,after,implicit,{})
   out[#out+1]=('Family preview strength %.2f'):format(strength)
   for _,f in ipairs(D.mod_budget.order) do if ({damage_dealt=true,launch_dealt=true,damage_taken=true,launch_taken=true,speed=true})[f] then out[#out+1]=('%s %.2fx -> %.2fx'):format(labels[f],old[f].value,new[f].value) end end
   local ids={};for id in pairs(before) do ids[id]=true end;for id in pairs(after) do ids[id]=true end
   local ordered={};for id in pairs(ids) do ordered[#ordered+1]=id end;table.sort(ordered)
   local function label(t)return type(t)=='table' and ('T%d x%d [%s]'):format(D.mod_schema.level(t),D.mod_schema.copies(t),table.concat(D.mod_schema.instances(t),',')) or tostring(t or 0)end
   for _,id in ipairs(ordered) do if label(before[id])~=label(after[id]) then out[#out+1]=id..': '..label(before[id])..' -> '..label(after[id]) end end
   local values={};for id in pairs(base) do values[id]=true end;for id in pairs(implicit) do values[id]=true end
   local keys={};for id in pairs(values) do keys[#keys+1]=id end;table.sort(keys)
   for _,id in ipairs(keys) do if base[id]~=implicit[id] then out[#out+1]=('%s: %.2fx -> %.2fx'):format(id,base[id] or 1,implicit[id] or 1) end end
  end;if #out==0 then out={'No modifier tier changes'} end;return out
 end
 function V:pickup(e)
  if self.lab:replaying() then return end
  local hosted=self.on_pickup and self.lab:hosted()
  local r=self.drops:pickup(e,self.bag,hosted and true or false,self.port);self:bump()
  if r and hosted then self.on_pickup(r)  -- a run shows its own card and logs
  elseif r then self.menu.notice='Picked up '..self.loot:name(r);self.card=self.menu.notice;self.card_left=120;self.g.log(self.menu.notice) end
 end
 -- A drop's lifetime ran out: a run keeps the record (collected at stage end), the LAB forgets it.
 function V:expire(e)
  local r=self.drops:expire(e);if r and self.on_expire and self.lab:hosted() then self.on_expire(r,e.reason) end
 end
 function V:snapshot(raw)
  local b=self.bag;local bag=raw and {items=b.items,equipped=b.equipped,keystone=b.keystone,keystones=b.keystones,context=b.context} or b:snapshot()
  return {bag=bag,drops=self.drops:snapshot(),pending=self.pending,seed=self.seed}
 end
 function V:validate(s,lab)
  lab=lab or self.lab
  assert(type(s)=='table' and type(s.pending or {})=='table','invalid drive checkpoint')
  s=D.mod_codec.decode(D.mod_codec.encode(s))
  for k in pairs(s) do assert(({bag=true,drops=true,pending=true,seed=true})[k],'unknown drive checkpoint field') end
  assert(type(s.seed)=='number' and s.seed%1==0 and s.seed>=1 and s.seed<=2147483646,'invalid drive seed')
  local n=0;for k in pairs(s.pending or {}) do assert(type(k)=='number' and k%1==0 and k>=1 and k<=12,'invalid pending index');n=n+1 end;assert(n==#(s.pending or {}),'sparse pending array')
  local config={};for k,v in pairs(self.bag.config) do config[k]=v end
  local context=D.mod_progression.context(s.bag.context or lab.engine.context);assert(context.depth==lab.engine.context.depth and context.loop==lab.engine.context.loop,'bag/engine progression mismatch');s.bag.context=context;config.context=context
  config.preflight=function(mods,implicits,ctx)
   local engine=D.mod_engine.new(lab.engine.seed,D.mod_pool);engine:import(lab.engine:export());engine:set_context(ctx);engine:set_build(self.port,self:combined(mods,lab),implicits);engine:native_rules(self.port);if lab.check_echo_capacity then lab:check_echo_capacity(engine,lab.echo_manual)end;return true
  end
  local probe=D.drive_bag.new(self.loot,config);assert(probe:restore(s.bag))
  for _,d in pairs(s.drops.records or {}) do self.loot:validate(d.record) end
  local count=self.drops:validate(s.drops)
  -- Ground drops reserve bag space for LAB edits. In a run a drop may land on a full bag (the pickup then asks which drive to give up, `choose`),
  -- so there the bag is only checked against its own capacity (found in the co-op campaigns: the refusal disabled the mod for good).
  assert(#probe.items<=probe:capacity() and (#probe.items+count<=probe:capacity() or (self.lab and self.lab.hosted and self.lab:hosted())),'invalid reserved capacity')
  for _,e in ipairs(s.pending or {}) do assert(type(e)=='table','invalid pending edit');for k in pairs(e) do assert(({op=true,a=true,b=e.op=='equip'})[k],'unknown pending field') end;assert(({give=true,equip=true,unequip=true,discard=true,choose_keystone=true})[e.op],'invalid pending drive edit');if count>0 then assert(e.op=='choose_keystone','pending inventory edit races ground pickup') end;assert(probe[e.op](probe,e.a,e.b));assert(#probe.items+count<=probe:capacity(),'overbooked pending draft') end
  assert(#(s.pending or {})<=12,'invalid pending budget')
  return D.mod_codec.decode(D.mod_codec.encode(s))
 end
 function V:publish(s)
  self:bump();self.menu:close();self.bag.items=s.bag.items;self.bag.equipped=s.bag.equipped;self.bag.keystone=s.bag.keystone;self.bag.keystones=s.bag.keystones or {};self.bag.context=D.mod_progression.context(s.bag.context)
  self.drops:restore(s.drops);self.pending=s.pending or {};self.seed=s.seed
 end
 function V:restore(s) self:publish(self:validate(s))
 end
 function V:clear()
  self:bump();self.menu:close();self.drops:clear();self.bag:new_run();self.pending={};self.applied=false
  if not self.bag.config.persist then self.bag.context=D.mod_progression.context() end
 end
 -- Stage teardown inside a run: ground drops, queued edits and the open menu belong to the old scene; the
 -- bag, slots and equipped build persist.
 function V:soft_clear()
  self:bump();self.menu:close();self.drops:clear();self.pending={};self.applied=false
 end
 -- Run adapter: put one rolled drive in the bag and equip it into the first free slot (the bag menu can
 -- still swap it). Returns false with a reason when the bag or the checkpoint refuses it right now.
 function V:grant(record)
  local draft=self:view();local ok,why=draft:give(record);if not ok then return false,tostring(why) end
  if #self.bag.items+#self.pending+self.drops:count()>=self.bag:capacity() then return false,'bag full' end
  local queued,reason=self:queue('give',record);if not queued then return false,reason end
  local index=#draft.items;local free
  for slot=1,draft:slots() do if not draft.equipped[slot] then free=slot;break end end
  if free then local equipped=self:queue('equip',index,free);if not equipped then self.pending[#self.pending]=nil;return true,'bagged' end end
  return true,free and 'equipped' or 'bagged'
 end
 -- True when the engine's build for the player no longer matches what apply() last derived: the bag's slots,
 -- keystones or progression changed, or the rule engine was replaced. Every writer of those replaces a table or a
 -- number (equip/publish/set_context/reset/loadstate), so comparing them is the whole invalidation.
 function V:stale()
  if not self:has_build() then return false end
  local s,b,e=self.stamp,self.bag,self.lab.engine
  return not self.applied or not s or s.equipped~=b.equipped or s.keystone~=b.keystone or s.keystones~=b.keystones or s.depth~=b.context.depth or s.loop~=b.context.loop or s.engine~=e or s.edepth~=e.context.depth or s.eloop~=e.context.loop
 end
 function V:has_build() return next(self.bag.equipped)~=nil or self.bag.keystone~=nil or #(self.bag.keystones or {})>0 end
 function V:tick()
  self.drops:retry_retired()
  if self.menu.active and (not self.lab:allowed() or self.lab:replaying()) then self.menu:close();return end
  local p=self.g.pad(self.port,true) or {};local chord=p.Z and p.START
  D.menu_input.settle(self.g,self.port)
  -- Z+START is the bag: keep the whole chord from the game so the press that opens the bag does not also pause the
  -- match (and the START that closes it is hidden by the menu's own mask). Re-asserted now and then: a scene change
  -- clears the engine's masks.
  if self.g.input_chord then
   local want=self.lab:allowed() and not self.lab:replaying()
   self.chord_age=(self.chord_age or 0)+1
   if want~=self.chord_on or (want and self.chord_age>=120) then self.chord_on=want;self.chord_age=0;self.g.input_chord(self.port,want and 'Z+START' or nil) end
  end
  if chord and not self.chord and not self.menu.active and not self.lab:replaying() then local ok=self.lab:allowed();if ok then if self.opener and self.lab:hosted() then self.opener() else self.menu:open();if self.lab.options.activate then self.lab.options.activate() end end end end
  self.chord=chord;self.menu:tick()
 end
 function V:frame() if not self.shared_drops then self.drops.juice:tick() end;if self.card_left then self.card_left=self.card_left-1;if self.card_left<=0 then self.card=nil;self.card_left=nil end end end
 function V:draw()
  self.menu:draw()
  if self.card and not self.menu.active and self.g.kit then local a=self.g.safe_area();self.g.kit.panel(a.x+20,a.y+48,a.w-40,42);self.g.kit.text(a.x+32,a.y+74,self.card,'body','bone','left',{max_w=a.w-64}) end
 end
 return V
end
