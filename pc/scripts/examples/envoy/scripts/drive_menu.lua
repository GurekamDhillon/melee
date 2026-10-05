-- Controller-only kit screen. Its owner commits queued edits at a simulation boundary.
return function(D)
 local M={};M.__index=M
 local colours={common=0xD7D4CFFF,magic=0x8CABFFFF,rare=0xEBD175FF,unique=0xE8A269FF}
 function M.wrap(k,text,width)
  local function measure(s) return k.measure and k.measure(s,'body') or #s*9 end
  local out,line={},''
  for word in text:gmatch('%S+') do
   if line~='' and measure(line..' '..word)>width then out[#out+1]=line;line='' end
   while measure(word)>width and #word>1 do
    local n=1;while n<#word and measure(word:sub(1,n+1))<=width do n=n+1 end
    out[#out+1]=word:sub(1,n);word=word:sub(n+1)
   end
   line=line=='' and word or line..' '..word
  end
  if line~='' then out[#out+1]=line end;return out
 end
 function M.new(g,owner)
  return setmetatable({g=g,owner=owner,input=D.menu_input.new(g),focus=1,slot=1,active=false},M)
 end
 function M:open()
  self.active=true;self.input:set_active(true);self.input.previous.start=true
  if self.g.paused and not self.g.paused() then self.g.pause();self.owns_pause=true end
 end
 function M:close()
  if self.active or self.input.masked then self.input:close() end
  self.active=false;self.selected=nil
  if self.owns_pause then self.g.resume();self.owns_pause=nil end
 end
 function M:entries()
  local b=self.owner:view();local rows={}
  for i=1,b:slots() do rows[#rows+1]={label='Slot '..i..': '..(b.equipped[i] and self.owner.loot:name(b.equipped[i]) or 'Empty'),slot=i} end
  for i,r in ipairs(b.items) do rows[#rows+1]={label=self.owner.loot:name(r),bag=i,colour=colours[r.rarity]} end
  local selected={};for _,id in ipairs(b.keystones or {}) do selected[id]=true end;if b.keystone then selected[b.keystone]=true end
  for _,r in ipairs(self.owner.lab.engine.list) do if r.kind=='keystone' then rows[#rows+1]={label=(selected[r.id] and '[x] ' or '[ ] ')..r.label,key=true,id=r.id} end end
  rows[#rows+1]={label='Clear keystones',key=true}
  rows[#rows+1]={label='Discard selected bag drive',discard=true}
  rows[#rows+1]={label='Close',close=true};return rows
 end
 function M:tick()
  if not self.active then return end
  local pad=self.g.pad(1,true) or {};local right=pad.RIGHT;local left=pad.LEFT;if right and not self.right then self.page=(self.page or 0)+1 elseif left and not self.left then self.page=math.max(0,(self.page or 0)-1) end;self.right=right;self.left=left
  for _,action in ipairs(self.input:poll()) do
   local rows=self:entries();self.focus=math.min(self.focus,#rows)
   if action=='up' then self.page=0; self.focus=(self.focus-2)%#rows+1
   elseif action=='down' then self.page=0; self.focus=self.focus%#rows+1
   elseif action=='back' or action=='start' then self:close()
   elseif action=='accept' then
    local e=rows[self.focus]
    if e.close then self:close()
    elseif e.bag then self.selected=e.bag;self.notice='Choose an equipped slot to equip / swap'
    elseif e.slot then
     self.slot=e.slot
     if self.selected then self.owner:queue('equip',self.selected,e.slot);self.selected=nil
     elseif self.owner:view().equipped[e.slot] then self.owner:queue('unequip',e.slot) end
    elseif e.discard and self.selected then self.owner:queue('discard',self.selected);self.selected=nil
    elseif e.key then
     self.owner:queue('choose_keystone',e.id)
    end
   end
  end
 end
 function M:draw()
  if not self.active or not self.g.kit then return end
  local g,k=self.g,self.g.kit;local a=g.safe_area();local w=math.min(740,a.w-32);local x=a.x+(a.w-w)/2;local y=a.y+12
  k.panel(x,y,w,a.h-24);k.text(x+20,y+28,'Envoy / Drive bag','body','bone','left')
  -- Rows, the budget/tooltip/delta lines and their wrapping are derived from the bag (draft engines, budget
  -- builds, text measurement): done when the screen's inputs change, not on every drawn frame. The key is the
  -- owner's rev (every writer of the bag/queue bumps it) plus focus, selection, page, slot, width and the clock;
  -- an owner without a rev is never cached.
  local o=self.owner;local b0=o.bag
  local key=type(o.rev)=='number' and table.concat({o.rev,self.focus,tostring(self.selected),self.slot,w,#o.pending,b0 and #b0.items or 0,tostring(b0 and b0.equipped),b0 and b0.context.depth or 0,b0 and b0.context.loop or 0,o.lab.engine.frame,tostring(self.notice)},'|')
  local m=self.model
  if not (key and m and m.key==key) then
   local rows=self:entries();local first=math.max(1,self.focus-5+1)
   local e=rows[self.focus];local r=e and (e.bag and self.owner:view().items[e.bag] or e.slot and self.owner:view().equipped[e.slot])
   local lines=self.owner:budget_lines()
   if r then lines[#lines+1]=r.rarity..' / '..self.owner.loot:name(r)
   for _,line in ipairs(self.owner.loot:tooltip(r)) do lines[#lines+1]=line end
   if e.bag then for _,line in ipairs(self.owner:delta(e.bag,self.slot)) do lines[#lines+1]=line end end
   end
   local wrapped={};for _,line in ipairs(lines) do for _,part in ipairs(M.wrap(k,line,w-40)) do wrapped[#wrapped+1]=part end end
   m={key=key,rows=rows,first=first,r=r,lines=wrapped};self.model=m
  end
  local rows,first,r,lines=m.rows,m.first,m.r,m.lines;local visible=5
  k.list(x+20,y+48,w-40,rows,self.focus,{pitch=24,h=24,first=first,visible=math.min(visible,#rows)})
  do
   local dy=190
   local budget=math.max(1,math.floor((a.h-76-dy)/19));local pages=math.max(1,math.ceil(#lines/budget));self.page=math.min(self.page or 0,pages-1)
   for i=1,budget do local line=lines[self.page*budget+i];if line then k.text(x+20,y+dy+(i-1)*19,line,'body',r and i==1 and colours[r.rarity] or 'bone','left',{max_w=w-40}) end end
   k.text(x+20,y+a.h-76,('Details %d/%d: Left / Right'):format(self.page+1,pages),'body','bone','left',{max_w=w-40})
  end
  k.text(x+20,y+a.h-58,self.notice or 'A: select / equip   B: close   Up / Down: choose','body','bone','left',{max_w=w-40})
 end
 return M
end
