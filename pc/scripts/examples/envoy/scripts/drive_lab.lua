-- Loot adapter shares the modifier adapter's checkpoint and commit boundary.
return function(D)
 local V={tuning={persist=false}};V.__index=V
 function V.new(g,lab)
  local self=setmetatable({g=g,lab=lab,pending={},seed=104729},V)
  self.loot=D.drive_loot.new(D.mod_pool)
  self.bag=D.drive_bag.new(self.loot,{persist=V.tuning.persist,preflight=function(mods,implicits)
   local probe=D.mod_engine.new(lab.engine.seed,D.mod_pool);probe:import(lab.engine:export());probe:set_build(1,self:combined(mods),implicits);return true
  end})
  self.drops=D.drive_drop.new(g);self.menu=D.drive_menu.new(g,self)
  g.command('drive',function(a) return self:command(a or '') end,'give|drop [rarity] [seed]')
  g.command('bag',function() local ok,why=lab:allowed();if not ok then g.log(why);return false end;self.menu:open();return true end,'open drive bag')
  return self
 end
 function V:combined(mods)
  local out={};for id,tier in pairs(mods) do out[id]=tier end
  for id,tier in pairs((self.lab.debug_equipped or {})[1] or {}) do out[id]=math.max(out[id] or 0,tier) end
  for _,e in ipairs(self.lab.pending or {}) do if e.port==1 then out[e.id]=math.max(out[e.id] or 0,1) end end
  return out
 end
 function V:view()
  local draft=D.drive_bag.new(self.loot,self.bag.config);assert(draft:restore(self.bag:snapshot()))
  for _,e in ipairs(self.pending) do assert(draft[e.op](draft,e.a,e.b)) end
  return draft
 end
 function V:reserved() return math.max(#self.bag.items,#self:view().items)+self.drops:count() end
 function V:queue(op,a,b)
  if #self.pending>=12 then self.menu.notice='drive edit queue full';return false,self.menu.notice end;local draft=self:view()
  if op~='choose_keystone' and self.drops:count()>0 then self.menu.notice='Collect ground drops before editing bag';return false,self.menu.notice end
  if (op=='give' or op=='unequip') and #draft.items+self.drops:count()>=12 then return false,'bag full (ground drops reserve space)' end
  local ok,why=draft[op](draft,a,b);if not ok then self.menu.notice=tostring(why);return false,why end
  self.pending[#self.pending+1]={op=op,a=a,b=b};self.lab.enabled=true;return true
 end
 function V:command(arg)
  local ok,why=pcall(function()
   local allowed,reason=self.lab:allowed();assert(allowed,reason);assert(not self.lab:replaying(),'drive edit refused during rewind')
   local w={};for v in arg:gmatch('%S+') do w[#w+1]=v end
   assert((w[1]=='give' or w[1]=='drop') and #w<=3,'usage: drive give|drop [rarity] [seed]')
   if w[1]=='drop' then assert(not (self.g.paused and self.g.paused()),'resume gameplay before dropping; wait one checkpoint before saving');assert(#self.pending==0,'wait for queued bag edits to commit before dropping') end
   local seed=tonumber(w[3] or self.seed);assert(seed and seed%1==0,'integer seed required')
   local record=self.loot:roll(seed,1,w[2] and w[2]:lower());assert(self:reserved()<12,'bag full (ground drops reserve space)')
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
   if e.op=='give' and #self.bag.items+self.drops:count()>=12 then ok,why=false,'bag full (ground drops reserve space)'
   elseif e.op=="unequip" and #self.bag.items+self.drops:count()>=12 then ok,why=false,"bag full (ground drops reserve space)"
   else ok,why=self.bag[e.op](self.bag,e.a,e.b) end
   if not ok then self.g.log('bag: refused '..tostring(why)) end
  end
  self.pending={};local mods,implicits=self.bag:derive();self.lab.engine:set_build(1,self:combined(mods),implicits)
  self.lab.engine.display.drive_build=self.lab.engine.display.drive_build or {};local looks={};for i=1,4 do local r=self.bag.equipped[i];if r then looks[#looks+1]={colour=r.colour,rarity=r.rarity} end end;self.lab.engine.display.drive_build[1]=looks
 end
 function V:delta(index,slot)
  local draft=self:view();local before,base=draft:derive();before=self:combined(before);local ok=draft:equip(index,slot)
  local out={};if ok then
   local after,implicit=draft:derive();after=self:combined(after);local ids={};for id in pairs(before) do ids[id]=true end;for id in pairs(after) do ids[id]=true end
   local ordered={};for id in pairs(ids) do ordered[#ordered+1]=id end;table.sort(ordered)
   for _,id in ipairs(ordered) do if before[id]~=after[id] then out[#out+1]=id..': '..(before[id] or 0)..' -> '..(after[id] or 0) end end
   local values={};for id in pairs(base) do values[id]=true end;for id in pairs(implicit) do values[id]=true end
   local keys={};for id in pairs(values) do keys[#keys+1]=id end;table.sort(keys)
   for _,id in ipairs(keys) do if base[id]~=implicit[id] then out[#out+1]=('%s: %.2fx -> %.2fx'):format(id,base[id] or 1,implicit[id] or 1) end end
  end;if #out==0 then out={'No modifier tier changes'} end;return out
 end
 function V:pickup(e)
  if self.lab:replaying() then return end
  local r=self.drops:pickup(e,self.bag);if r then self.menu.notice='Picked up '..self.loot:name(r);self.card=self.menu.notice;self.card_left=120;self.g.log(self.menu.notice) end
 end
 function V:snapshot() return {bag=self.bag:snapshot(),drops=self.drops:snapshot(),pending=self.pending,seed=self.seed} end
 function V:restore(s)
  assert(type(s)=='table' and type(s.pending or {})=='table','invalid drive checkpoint')
  for k in pairs(s) do assert(({bag=true,drops=true,pending=true,seed=true})[k],'unknown drive checkpoint field') end
  assert(type(s.seed)=='number' and s.seed%1==0 and s.seed>=1 and s.seed<=2147483646,'invalid drive seed')
  local n=0;for k in pairs(s.pending or {}) do assert(type(k)=='number' and k%1==0 and k>=1 and k<=12,'invalid pending index');n=n+1 end;assert(n==#(s.pending or {}),'sparse pending array')
  local probe=D.drive_bag.new(self.loot,self.bag.config);assert(probe:restore(s.bag))
  for _,d in pairs(s.drops.records or {}) do self.loot:validate(d.record) end
  local count=self.drops:validate(s.drops)
  assert(#probe.items+count<=12,'invalid reserved capacity')
  for _,e in ipairs(s.pending or {}) do assert(type(e)=='table','invalid pending edit');for k in pairs(e) do assert(({op=true,a=true,b=e.op=='equip'})[k],'unknown pending field') end;assert(({give=true,equip=true,unequip=true,discard=true,choose_keystone=true})[e.op],'invalid pending drive edit');if count>0 then assert(e.op=='choose_keystone','pending inventory edit races ground pickup') end;assert(probe[e.op](probe,e.a,e.b));assert(#probe.items+count<=12,'overbooked pending draft') end
  assert(#(s.pending or {})<=12,'invalid pending budget')
  self.menu:close();assert(self.bag:restore(s.bag));self.drops:restore(s.drops);self.pending=s.pending or {};self.seed=s.seed
 end
 function V:clear() self.menu:close();self.drops:clear();self.bag:new_run();self.pending={} end
 function V:has_build() return next(self.bag.equipped)~=nil or self.bag.keystone~=nil end
 function V:tick()
  self.drops:retry_retired()
  local p=self.g.pad(1,true) or {};local chord=p.Z and p.START
  if chord and not self.chord and not self.menu.active and not self.lab:replaying() then local ok=self.lab:allowed();if ok then self.menu:open();if self.lab.options.activate then self.lab.options.activate() end end end
  self.chord=chord;self.menu:tick()
 end
 function V:frame() self.drops.juice:tick();if self.card_left then self.card_left=self.card_left-1;if self.card_left<=0 then self.card=nil;self.card_left=nil end end end
 function V:draw()
  self.menu:draw()
  if self.card and not self.menu.active and self.g.kit then local a=self.g.safe_area();self.g.kit.panel(a.x+20,a.y+48,a.w-40,42);self.g.kit.text(a.x+32,a.y+74,self.card,'body','bone','left',{max_w=a.w-64}) end
 end
 return V
end
