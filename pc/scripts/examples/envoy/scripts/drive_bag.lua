-- Bag holds unequipped drives; four independent slots and one companion keystone.
return function(D)
 local B={};B.__index=B
 local function copy(v) if type(v)~='table' then return v end;local t={};for k,x in pairs(v) do t[k]=copy(x) end;return t end
 local function index(i,n) return type(i)=='number' and i==math.floor(i) and i>=1 and i<=n end
 function B.new(loot,config) return setmetatable({loot=loot,config=config or {},items={},equipped={},keystone=nil},B) end
 function B:derive(state)
  state=state or self;local mods,implicits={},{}
  for slot=1,(self.config.slots or 4) do local r=state.equipped[slot];if r then
   self.loot:validate(r)
   for _,a in ipairs(r.affixes) do mods[a.id]=math.max(mods[a.id] or 0,a.tier) end
   for k,v in pairs(self.loot.implicits[r.colour]) do implicits[k]=(implicits[k] or 1)*v end
  end end
  if state.keystone then local m=assert(self.loot.rules[state.keystone],'unknown keystone');assert(m.kind=='keystone');mods[m.id]=1 end
  for k,v in pairs(implicits) do implicits[k]=math.max(.25,math.min(4,v)) end
  local rules=0;for id in pairs(mods) do for _,e in ipairs(self.loot.rules[id].effects) do if e.op=='convert' or e.op=='versus-status' then rules=rules+1 end end end
  assert(rules<=8,'native hit rule capacity exceeded');return mods,implicits
 end
 function B:validate(s)
  assert(type(s)=='table' and type(s.items)=='table' and type(s.equipped)=='table','invalid bag')
  for k in pairs(s) do assert(k=='items' or k=='equipped' or k=='keystone','unknown bag field') end
  local count=0;for k,r in pairs(s.items) do assert(index(k,#s.items),'invalid bag index');self.loot:validate(r);count=count+1 end
  assert(count==#s.items and count<=(self.config.capacity or 12),'bag full')
  for k,r in pairs(s.equipped) do assert(index(k,self.config.slots or 4),'invalid equipped slot');self.loot:validate(r) end
  return self:derive(s)
 end
 function B:snapshot() return copy({items=self.items,equipped=self.equipped,keystone=self.keystone}) end
 function B:publish(s)
  local ok,err=pcall(function()
   local mods,implicits=self:validate(s)
   if self.config.preflight then local accepted,why=self.config.preflight(mods,implicits);assert(accepted~=false,why or 'build rejected') end
  end)
  if not ok then return false,err end
  self.items,self.equipped,self.keystone=s.items,s.equipped,s.keystone;return true
 end
 function B:restore(s) return self:publish(copy(s)) end
 function B:give(r)
  local ok,err=pcall(function() self.loot:validate(r) end);if not ok then return false,err end
  if #self.items>=(self.config.capacity or 12) then return false,'bag full' end
  self.items[#self.items+1]=copy(r);return true
 end
 function B:equip(i,slot)
  if not index(i,#self.items) or not index(slot,self.config.slots or 4) then return false,'invalid index' end
  local s=self:snapshot();local r=table.remove(s.items,i);local old=s.equipped[slot];s.equipped[slot]=r;if old then s.items[#s.items+1]=old end
  return self:publish(s)
 end
 function B:unequip(slot)
  if not index(slot,self.config.slots or 4) or not self.equipped[slot] then return false,'empty slot' end
  if #self.items>=(self.config.capacity or 12) then return false,'bag full' end
  local s=self:snapshot();s.items[#s.items+1]=s.equipped[slot];s.equipped[slot]=nil;return self:publish(s)
 end
 function B:discard(i) if not index(i,#self.items) then return false,'invalid index' end;table.remove(self.items,i);return true end
 function B:choose_keystone(id) local s=self:snapshot();s.keystone=id;return self:publish(s) end
 function B:new_run() if self.config.persist then return true end;return self:publish({items={},equipped={}}) end
 return B
end
