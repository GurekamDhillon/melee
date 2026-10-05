-- Bag capacity, unlocked slots and keystones share the progression context.
return function(D)
 local B={};B.__index=B
 local function copy(v) if type(v)~='table' then return v end;local t={};for k,x in pairs(v) do t[k]=copy(x) end;return t end
 local function index(i,n) return type(i)=='number' and i==math.floor(i) and i>=1 and i<=n end
 -- The bag's size comes from the economy's tuning when that module is loaded (4), else the old 12 (standalone tests).
 function B.default_capacity() return D.drive_economy and D.drive_economy.tuning.bag_capacity or 12 end
 function B:capacity() return self.config.capacity or B.default_capacity() end
 function B.new(loot,config)
  config=config or {};return setmetatable({loot=loot,config=config,context=D.mod_progression.context(config.context),items={},equipped={},keystone=nil,keystones={}},B)
 end
 function B:slots(context) return self.config.slots or D.mod_progression.slots(context or self.context) end
 function B:set_context(context)
  local s=self:snapshot();s.context=D.mod_progression.context(context);return self:publish(s)
 end
 function B:derive(state)
  state=state or self;local mods,implicits={},{}
  for slot=1,self:slots(state.context) do local r=state.equipped[slot];if r then
   self.loot:validate(r)
   for _,a in ipairs(r.affixes) do local m=self.loot.rules[a.id]
    if m.kind=='unique' then local old=mods[a.id];local tiers=old and D.mod_schema.instances(old) or {};tiers[#tiers+1]=a.tier;mods[a.id]={tier=math.max(old and D.mod_schema.level(old) or 0,a.tier),copies=#tiers,tiers=tiers}
    else mods[a.id]=math.max(mods[a.id] or 0,a.tier)end end
   for k,v in pairs(self.loot.implicits[r.colour]) do implicits[k]=(implicits[k] or 1)+(v-1) end
  end end
  local keys={};for _,id in ipairs(state.keystones or {})do assert(not keys[id],'duplicate keystone');keys[id]=true end
  if state.keystone then keys[state.keystone]=true end
  local count=0;for id in pairs(keys)do local m=assert(self.loot.rules[id],'unknown keystone');assert(m.kind=='keystone');mods[id]=D.mod_progression.tier(state.context or self.context);count=count+1 end
  assert(count<=D.mod_progression.keystones(state.context or self.context),'keystone allowance exceeded')
  D.mod_budget.build(self.loot.pool,mods,implicits,{})
  local rules=0;for id in pairs(mods) do for _,e in ipairs(self.loot.rules[id].effects) do if e.op=='convert' or e.op=='versus-status' then rules=rules+1 end end end
  assert(rules<=30,'native hit rule capacity exceeded (two budget slots reserved)');return mods,implicits
 end
 function B:validate(s)
  assert(type(s)=='table' and type(s.items)=='table' and type(s.equipped)=='table','invalid bag')
  for k in pairs(s) do assert(k=='items' or k=='equipped' or k=='keystone' or k=='keystones' or k=='context','unknown bag field') end
  D.mod_progression.context(s.context or self.context)
  if s.keystones then local n=0;for k,id in pairs(s.keystones)do assert(index(k,#s.keystones) and type(id)=='string','invalid keystone array');n=n+1 end;assert(n==#s.keystones,'sparse keystone array')end
  local count=0;for k,r in pairs(s.items) do assert(index(k,#s.items),'invalid bag index');self.loot:validate(r);count=count+1 end
  assert(count==#s.items and count<=self:capacity(),'bag full')
  for k,r in pairs(s.equipped) do assert(index(k,self:slots(s.context)),'invalid equipped slot');self.loot:validate(r) end
  return self:derive(s)
 end
 function B:snapshot() return copy({items=self.items,equipped=self.equipped,keystone=self.keystone,keystones=self.keystones,context=self.context}) end
 function B:publish(s)
  local ok,err=pcall(function()
   local mods,implicits=self:validate(s)
   if self.config.preflight then local accepted,why=self.config.preflight(mods,implicits,s.context or self.context);assert(accepted~=false,why or 'build rejected') end
  end)
  if not ok then return false,err end
  self.items,self.equipped,self.keystone=s.items,s.equipped,s.keystone;self.keystones=s.keystones or {};self.context=D.mod_progression.context(s.context or self.context);return true
 end
 function B:restore(s) return self:publish(copy(s)) end
 function B:give(r)
  local ok,err=pcall(function() self.loot:validate(r) end);if not ok then return false,err end
  if #self.items>=self:capacity() then return false,'bag full' end
  self.items[#self.items+1]=copy(r);return true
 end
 function B:equip(i,slot)
  if not index(i,#self.items) or not index(slot,self:slots()) then return false,'invalid index' end
  local s=self:snapshot();local r=table.remove(s.items,i);local old=s.equipped[slot];s.equipped[slot]=r;if old then s.items[#s.items+1]=old end
  return self:publish(s)
 end
 function B:unequip(slot)
  if not index(slot,self:slots()) or not self.equipped[slot] then return false,'empty slot' end
  if #self.items>=self:capacity() then return false,'bag full' end
  local s=self:snapshot();s.items[#s.items+1]=s.equipped[slot];s.equipped[slot]=nil;return self:publish(s)
 end
 -- Putting a drive into an EMPTY slot never needs bag space (the drive is not in the bag first).
 function B:place(slot,r)
  if not index(slot,self:slots()) then return false,'invalid index' end
  if self.equipped[slot] then return false,'slot occupied' end
  local ok,err=pcall(function() self.loot:validate(r) end);if not ok then return false,err end
  local s=self:snapshot();s.equipped[slot]=copy(r);return self:publish(s)
 end
 -- Replace a held drive by another record (a merge result, or a drive that replaces a given-up one). No space needed.
 function B:replace(where,i,r)
  local s=self:snapshot()
  if where=='equipped' then if not index(i,self:slots()) or not s.equipped[i] then return false,'invalid index' end;s.equipped[i]=copy(r)
  elseif where=='bag' then if not index(i,#s.items) then return false,'invalid index' end;s.items[i]=copy(r)
  else return false,'invalid index' end
  return self:publish(s)
 end
 -- Every drive the player holds, equipped first (slot order), then the bag: {record=,where='equipped'|'bag',index=}.
 function B:held()
  local out={}
  for slot=1,self:slots() do local r=self.equipped[slot];if r then out[#out+1]={record=r,where='equipped',index=slot} end end
  for i,r in ipairs(self.items) do out[#out+1]={record=r,where='bag',index=i} end
  return out
 end
 function B:discard(i) if not index(i,#self.items) then return false,'invalid index' end;table.remove(self.items,i);return true end
 function B:choose_keystone(id)
  local s=self:snapshot();s.keystones=s.keystones or {};s.keystone=nil
  if not id then s.keystones={}
  elseif D.mod_progression.keystones(self.context)==1 then s.keystones={id}
  else local found;for i,key in ipairs(s.keystones)do if key==id then table.remove(s.keystones,i);found=true;break end end;if not found then s.keystones[#s.keystones+1]=id end end
  if #s.keystones==1 then s.keystone=s.keystones[1]end
  return self:publish(s)
 end
 function B:new_run() if self.config.persist then return true end;return self:publish({items={},equipped={}}) end
 return B
end
