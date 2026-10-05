-- LAB manual spawns branch native rewind history. No spawning from frame/replay.
return function(D)
 local R={};R.__index=R
 local element_colour={fire='red',electric='yellow',ice='blue',darkness='purple'}
 function R.core_colour(record)
  local present={};for _,a in ipairs(record.affixes) do present[a.id]=true end
  local colour=record.colour
  for _,m in ipairs(D.mod_pool) do if present[m.id] then for _,effect in ipairs(m.effects) do
   if effect.op=='convert' and effect.change and element_colour[effect.change.element] then colour=element_colour[effect.change.element] end
  end end end;return colour
 end
 function R.new(g)
  D.pickup_juice.pitch.purple=150
  local presentation=setmetatable({fx_world=function(name,...) if not g.fx_world then return nil end;return g.fx_world(name:gsub('PickupJuice_(.-)_purple','DriveLoot_%1_purple'),...) end},{__index=g})
  return setmetatable({g=g,next_id=1,records={},juice=D.pickup_juice.new(presentation)},R)
 end
 function R:count() local n=0;for _ in pairs(self.records) do n=n+1 end;return n end
 -- `at_x, at_y`: where the drop appears (a run drops where the player can reach it); default beside CPU 2 (the LAB).
 function R:spawn(record,at_x,at_y)
  assert(self:count()<12,'drop capacity exhausted')
  if not at_x then local p=self.g.player(2) or self.g.player(1);assert(p,'fighter absent');at_x,at_y=p.x+10,p.y+8 end
  local id=self.next_id;assert(id<=1000000,'drop id budget exhausted');local c=record.colour=='purple' and 'white' or record.colour
  local h,why=self.g.item_spawn('drive',at_x,at_y,{payload={colour=c,amount=id}})
  assert(h,why or 'drive spawn refused');self.next_id=id+1
  self.records[id]={handle=h,record=record};self:visual(h,record,at_x,at_y)
  return h
 end
 function R:visual(h,record,x,y)
  local d=self.juice:drop(h,R.core_colour(record),x,y,900)
  if record.rarity~="common" and self.g.fx_world then for _,kind in ipairs({"beam","sparkles"}) do d.fx["rarity_"..kind]=self.g.fx_world("DriveLoot_"..kind.."_"..record.rarity,x,y+12,0,1,1) end end
  if record.unique and self.g.fx_world then d.fx.signature=self.g.fx_world('DriveLoot_signature_'..record.unique,x,y+12,0,1,1) end
 end
 -- `hosted`: a run's host decides what happens to the drive (merge, bag, ask), so nothing is given to the bag here.
 function R:pickup(e,bag,hosted)
  if e.name~='drive' or e.port~=1 then return false end
  local id=e.payload and e.payload.amount;local d=id and self.records[id]
  if not d or d.handle~=(e.item or e.handle) then return false end
  if not hosted then assert(bag:give(d.record)) end;self.records[id]=nil;self.juice:collect(d.handle,nil,self.g.player(e.port or 1));return d.record
 end
 function R:expire(e)
  local id=e.payload and e.payload.amount;local d=id and self.records[id]
  if d and d.handle==(e.item or e.handle) then self.records[id]=nil;self.juice:expire(d.handle);return d.record end
 end
 function R:snapshot() return {next_id=self.next_id,records=self.records} end
 function R:validate(s)
  assert(type(s)=='table' and type(s.records)=='table' and type(s.next_id)=='number' and s.next_id%1==0 and s.next_id>=1 and s.next_id<=1000001,'invalid drop checkpoint')
  for k in pairs(s) do assert(k=='next_id' or k=='records','unknown drop checkpoint field') end
  local count=0;local handles={}
  for id,d in pairs(s.records) do
   assert(type(id)=='number' and id%1==0 and id>=1 and id<s.next_id and type(d)=='table' and type(d.handle)=='number' and d.handle%1==0 and d.handle>0 and d.handle<=2147483647,'invalid drop record')
   for k in pairs(d) do assert(k=='handle' or k=='record','unknown drop record field') end
   assert(not handles[d.handle],'duplicate drop handle');handles[d.handle]=true
   assert(type(d.record)=='table','missing loot record');count=count+1
  end
  assert(count<=12,'drop checkpoint capacity exceeded');return count
 end
 function R:restore(s)
  self:validate(s)
  self.juice:clear();self.next_id=s.next_id;self.records=s.records
  for _,d in pairs(self.records) do if self.retired then self.retired[d.handle]=nil end end
  local items={};for _,e in ipairs(self.g.items and self.g.items() or {}) do if type(e.handle)=='number' and e.handle>0 and e.handle%1==0 then items[e.handle]=e end end
  for _,d in pairs(self.records) do local e=items[d.handle];if e then self:visual(d.handle,d.record,e.x,e.y) end end
 end
 function R:clear()
  self.retired=self.retired or {}
  for _,d in pairs(self.records) do self.retired[d.handle]=true end
  self:retry_retired()
  self.juice:clear();self.records={};self.next_id=1
 end
 function R:retry_retired()
  local live={};if self.g.items then for _,e in ipairs(self.g.items()) do if type(e.handle)=='number' and e.handle>0 and e.handle%1==0 then live[e.handle]=true end end end
  for h in pairs(self.retired or {}) do
   local ok,result=false,false;if self.g.item_despawn then ok,result=pcall(self.g.item_despawn,h) end
   if ok and result~=false or self.g.items and not live[h] then self.retired[h]=nil end
  end
 end
 return R
end
