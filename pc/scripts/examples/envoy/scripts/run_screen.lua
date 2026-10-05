-- The run's one build screen. It is used twice: as the reward moment inside the hold after a stage clear (mode
-- 'reward', offers + decisions + a timeout) and as the bag screen opened with Z+START in a fight (mode 'bag').
-- Controller only. Every number it shows is derived when something changes, never per drawn frame.
--   A  the focused row's main action (shown at the bottom)   X  keep in the bag / put back in the bag
--   Y  discard a bag drive (asks twice)   B  back / close   D-pad up/down move, left/right turn the text page
return function(D)
 local S={};S.__index=S
 S.tuning={hold_ticks=1800,tick_margin=90,safe_seconds=26,visible_rows=14}
 local T=function() return D.drive_text end
 local function ceil(n) return math.ceil(n) end
 function S.new(g,host)
  return setmetatable({g=g,host=host,input=D.menu_input.new(g),active=false,mode='bag',view='list',focus=1,page=0,prev={},rev=-1,first=1},S)
 end
 function S:drives() return self.host.mods.drives end
 -- ---- model -------------------------------------------------------------------------------------------------
 function S:rows()
  local h=self.host;local d=self:drives();local b=d.bag;local rows={}
  local function add(r) rows[#rows+1]=r;return r end
  if self.view=='swap' then
   add{kind='header',label='SWAP OUT WHICH DRIVE?',colour='gold'}
   for slot=1,b:slots() do local r=b.equipped[slot]
    add{kind='swap_slot',slot=slot,record=r,label=('Slot %d: %s'):format(slot,r and d.loot:name(r) or 'Empty')}
   end
   add{kind='swap_keep',label='Keep it in the bag instead'}
   return rows
  end
  if self.mode=='reward' and #h.offers>0 then
   add{kind='header',label=('STAGE REWARD: take one of %d'):format(#h.offers),colour='gold'}
   for i,r in ipairs(h.offers) do add{kind='offer',index=i,record=r,label=T().rarity_label[r.rarity]..' '..d.loot:name(r)} end
   add{kind='skip',label='Skip the reward (keep none)'}
  end
  add{kind='header',label=('EQUIPPED %d/%d'):format(self.host:equipped_count(),b:slots()),colour='gold'}
  for slot=1,b:slots() do local r=b.equipped[slot]
   add{kind='slot',slot=slot,record=r,label=('Slot %d: %s'):format(slot,r and d.loot:name(r) or 'Empty')}
  end
  add{kind='header',label=('BAG %d/%d'):format(#b.items,b.config.capacity or 12),colour='gold'}
  if #b.items==0 then add{kind='note',label='Empty'} end
  for i,r in ipairs(b.items) do
   add{kind='bag',index=i,record=r,new=h:is_new(r),label=(h:is_new(r) and 'NEW ' or '')..T().rarity_label[r.rarity]..' '..d.loot:name(r)}
  end
  local allow=D.mod_progression.keystones(b.context)
  local chosen=h:keystone_set()
  local n=0;for _ in pairs(chosen) do n=n+1 end
  add{kind='header',label=('KEYSTONES %d/%d'):format(n,allow),colour='gold'}
  for _,m in ipairs(d.lab.engine.list) do if m.kind=='keystone' then
   add{kind='key',id=m.id,rule=m,label=(chosen[m.id] and '[x] ' or '[ ] ')..m.label}
  end end
  add{kind='done',label=self.mode=='reward' and 'Continue to the next stage' or 'Close'}
  return rows
 end
 function S:focusable(r) return r.kind~='header' and r.kind~='note' end
 local function first_focus(rows) for i,r in ipairs(rows) do if r.kind~='header' and r.kind~='note' then return i end end;return 1 end
 -- The text block for the focused row, and the controls line.
 function S:detail(row)
  local h=self.host;local d=self:drives();local loot=d.loot;local tx=T();local lines,col={},nil
  local function head(r) lines[#lines+1]={tx.header(loot,r),tx.rarity_colour[r.rarity]};for _,l in ipairs(tx.drive_lines(loot,r)) do lines[#lines+1]={l} end end
  local controls='Up/Down: move'
  local before=h:totals()
  local after,swapout
  if row.record then head(row.record) end
  if row.kind=='offer' then
   local free=h:free_slot()
   if free then after=h:totals(function(bag) bag:give(row.record);bag:equip(#bag.items,free) end)
    controls='A: take it and equip in slot '..free..'   X: take it, keep in bag'
   else controls='A: take it, then pick a slot to swap   X: take it, keep in bag' end
   lines[#lines+1]={''}
  elseif row.kind=='bag' then
   local free=h:free_slot()
   if free then after=h:totals(function(bag) bag:equip(row.index,free) end);controls='A: equip in slot '..free..'   Y: discard'
   else controls='A: swap with an equipped drive   Y: discard' end
  elseif row.kind=='slot' then
   if row.record then controls='A or X: move it to the bag' else controls='Empty: pick a bag drive and press A' end
  elseif row.kind=='swap_slot' then
   local src=self.swap and self.swap.index
   if src and row.record then swapout=row.record;after=h:totals(function(bag) bag:equip(src,row.slot) end);controls='A: swap it in   B: back'
   elseif src then after=h:totals(function(bag) bag:equip(src,row.slot) end);controls='A: equip it here   B: back' end
   lines[#lines+1]={self.swap and self.swap.record and ('Moving in: '..loot:name(self.swap.record)) or '',tx.rarity_colour[(self.swap and self.swap.record or {rarity='common'}).rarity]}
  elseif row.kind=='swap_keep' then lines[#lines+1]={'The drive stays in your bag; nothing is lost.'};controls='A: keep it in the bag   B: back'
  elseif row.kind=='key' then
   lines[#lines+1]={row.rule.label..' (keystone)','gold'}
   for _,l in ipairs(tx.keystone_lines(row.rule)) do lines[#lines+1]={l} end
   local on=h:keystone_set()[row.id];controls=on and 'A: remove this keystone' or 'A: choose this keystone'
   lines[#lines+1]={''};lines[#lines+1]={'A keystone is one powerful rule with a built-in drawback.'}
  elseif row.kind=='skip' then
   lines[#lines+1]={'Skip the reward','gold'};lines[#lines+1]={('None of the %d offered drives is kept.'):format(#h.offers)}
   controls=self.confirm=='skip' and 'A: yes, skip them   B: no' or 'A: skip (asks again)'
  elseif row.kind=='done' then
   if self.mode=='reward' then
    if #h.offers>0 then lines[#lines+1]={'You still have a reward to choose or skip.'};controls='A: not yet'
    else lines[#lines+1]={'New drives in your bag are kept.','ok'};controls='A: continue' end
   else controls='A: close   B: close' end
  end
  if row.kind=='bag' or row.kind=='offer' then if #lines>0 then lines[#lines+1]={''} end end
  -- Totals: always shown; before -> after when a drive is highlighted.
  lines[#lines+1]={''}
  lines[#lines+1]={after and 'IF YOU DO THIS' or 'YOUR BUILD','gold'}
  for _,e in ipairs(tx.total_rows) do
   local text=tx.total_line(e[1],e[2],before,after)
   local better=after and tx.better(e[1],before[e[1]],after[e[1]])
   lines[#lines+1]={text,better==true and 'ok' or better==false and 'danger' or nil}
  end
  if swapout then lines[#lines+1]={'Swapped out: '..loot:name(swapout)..' (goes to the bag)'} end
  return lines,controls
 end
 function S:ensure()
  local d=self:drives();local key=table.concat({d.rev or 0,self.focus,self.view,self.mode,tostring(self.confirm),tostring(self.swap and self.swap.index),#self.host.offers,self.page},'|')
  if self.model and self.model.key==key and not self.dirty then return self.model end
  self.dirty=false
  local rows=self:rows();if self.focus>#rows then self.focus=#rows end
  while rows[self.focus] and not self:focusable(rows[self.focus]) do self.focus=self.focus%#rows+1 end
  local lines,controls=self:detail(rows[self.focus])
  -- wrap once per model
  local wrapped={};local k=self.g.kit;local width=self.detail_w or 280
  for _,l in ipairs(lines) do
   if l[1]=='' then wrapped[#wrapped+1]={''} else for _,part in ipairs(D.drive_menu.wrap(k or {},l[1],width)) do wrapped[#wrapped+1]={part,l[2]} end end
  end
  self.model={key=key,rows=rows,lines=wrapped,controls=controls}
  return self.model
 end
 function S:invalidate() self.dirty=true end
 -- ---- lifecycle -----------------------------------------------------------------------------------------------
 function S:open(mode)
  self.active=true;self.mode=mode;self.view='list';self.swap=nil;self.confirm=nil;self.page=0;self.notice=nil;self.focus=1;self.dirty=true
  self.input:set_active(true);self.input.previous.start=true;self.input.previous.accept=true;self.input.previous.back=true
  self.opened=self.g.time and self.g.time() or 0;self.ticks=0
  if mode=='bag' and self.g.paused and not self.g.paused() then self.g.pause();self.owns_pause=true end
  self.focus=first_focus(self:rows())
  self.host:log('screen open: '..mode)
 end
 function S:close()
  if self.active or self.input.masked then self.input:close() end
  self.active=false;self.swap=nil;self.confirm=nil;self.model=nil
  if self.owns_pause then self.g.resume();self.owns_pause=nil end
 end
 -- The engine's hold is counted in host ticks (at most 1800), and a host tick is a render tick: 60 or 120 a second.
 -- The countdown is the shorter of the wall-clock allowance and what is left of the hold at the measured tick rate.
 function S:seconds_left()
  if self.mode~='reward' then return nil end
  local wall=self.g.time and (self.g.time()-self.opened) or self.ticks/60
  local rate=(self.ticks>=20 and wall>0.05) and self.ticks/wall or 60
  local by_ticks=(S.tuning.hold_ticks-S.tuning.tick_margin-self.ticks)/rate
  return math.max(0,math.min(S.tuning.safe_seconds-wall,by_ticks))
 end
 -- Every way out of the reward moment goes through here with the reason, so nothing is dropped silently.
 function S:finish(reason)
  self:close();self.host:finish_reward(reason)
 end
 -- ---- input ---------------------------------------------------------------------------------------------------
 function S:notify(text) self.notice=text;self.notice_until=(self.g.time and self.g.time() or 0)+4;self.dirty=true end
 function S:move(dir)
  local m=self:ensure();local n=#m.rows
  for _=1,n do self.focus=(self.focus-1+dir)%n+1;if self:focusable(m.rows[self.focus]) then break end end
  self.page=0;self.confirm=nil;self.dirty=true
 end
 function S:accept()
  local m=self:ensure();local row=m.rows[self.focus];local h=self.host;if not row then return end
  if row.kind~='skip' then self.confirm=nil end
  if row.kind=='offer' then
   local free=h:free_slot();local idx,why=h:take_offer(row.index)
   if not idx then return self:notify(why) end
   if free then self:notify(select(2,h:equip(idx,free)) or '') else self.view='swap';self.swap={index=idx,record=self:drives().bag.items[idx]};self.focus=1;self.focus=first_focus(self:rows()) end
  elseif row.kind=='bag' then
   local free=h:free_slot()
   if free then local ok,msg=h:equip(row.index,free);self:notify(msg)
   else self.view='swap';self.swap={index=row.index,record=row.record};self.focus=first_focus(self:rows()) end
  elseif row.kind=='slot' then
   if row.record then local ok,msg=h:unequip(row.slot);self:notify(msg) else self:notify('Pick a bag drive to equip here.') end
  elseif row.kind=='swap_slot' then
   local ok,msg=h:equip(self.swap.index,row.slot);self:notify(msg);self.view='list';self.swap=nil;self.focus=first_focus(self:rows())
  elseif row.kind=='swap_keep' then
   h:log('bagged '..self:drives().loot:name(self.swap.record));self:notify('Kept in the bag.');self.view='list';self.swap=nil;self.focus=first_focus(self:rows())
  elseif row.kind=='key' then local ok,msg=h:toggle_keystone(row.id);self:notify(msg)
  elseif row.kind=='skip' then
   if self.confirm=='skip' then self.confirm=nil;h:decline_offers('skipped');self:notify('Reward skipped.');self.focus=first_focus(self:rows()) else self.confirm='skip';self:notify('Press A again to skip all '..#h.offers..' drives, or B to keep choosing.') end
  elseif row.kind=='done' then
   if self.mode=='reward' then
    if #h.offers>0 then self:notify('Choose a drive or skip the reward first.') else self:finish('done') end
   else self:finish('closed') end
  end
  self.dirty=true
 end
 function S:keep()  -- X
  local m=self:ensure();local row=m.rows[self.focus];local h=self.host
  if row.kind=='offer' then local idx,why=h:take_offer(row.index);if idx then h:log('bagged '..self:drives().loot:name(self:drives().bag.items[idx]));self:notify('In your bag.') else self:notify(why) end;self.focus=first_focus(self:rows())
  elseif row.kind=='slot' and row.record then local ok,msg=h:unequip(row.slot);self:notify(msg)
  end
  self.dirty=true
 end
 function S:discard()  -- Y
  local m=self:ensure();local row=m.rows[self.focus];local h=self.host
  if row.kind~='bag' then return end
  if self.confirm=='discard'..row.index then self.confirm=nil;local ok,msg=h:discard(row.index);self:notify(msg)
  else self.confirm='discard'..row.index;self:notify('Press Y again to discard this drive for good, or B to cancel.') end
  self.dirty=true
 end
 function S:back()
  if self.confirm then self.confirm=nil;self:notify('Cancelled.');return end
  if self.view=='swap' then self.view='list';self.swap=nil;self.focus=first_focus(self:rows());self.dirty=true;return end
  if self.mode=='bag' then self:finish('closed')
  else
   local m=self:ensure();for i,r in ipairs(m.rows) do if r.kind=='done' then self.focus=i end end
   self:notify(#self.host.offers>0 and 'Choose a drive or skip the reward before continuing.' or 'Press A on Continue to leave.');self.dirty=true
  end
 end
 function S:press(action)
  if action=='up' then self:move(-1) elseif action=='down' then self:move(1)
  elseif action=='accept' then self:accept() elseif action=='back' then self:back()
  elseif action=='x' then self:keep() elseif action=='y' then self:discard()
  elseif action=='left' then self.page=math.max(0,self.page-1);self.dirty=true
  elseif action=='right' then self.page=self.page+1;self.dirty=true
  elseif action=='start' and self.mode=='bag' then self:finish('closed') end
 end
 function S:tick()
  if not self.active then return end
  self.ticks=self.ticks+1
  local p=self.g.pad(1,true) or {}
  local acts=self.input:poll()
  for _,a in ipairs(acts) do self:press(a) end
  if self.active then
   for _,k in ipairs({'x','y','left','right'}) do
    local on=p[k:upper()]
    if on and not self.prev[k] then self:press(k) end
    self.prev[k]=on
   end
  end
  if self.active and self.notice and self.g.time and self.g.time()>self.notice_until then self.notice=nil;self.dirty=true end
  if self.active and self.mode=='reward' then
   local m=self.g.mode_1p and self.g.mode_1p()
   local left=self:seconds_left()
   if left<=0 then self:finish('timeout') elseif m and m.held==false then self:finish('released') end
  end
 end
 -- ---- drawing -------------------------------------------------------------------------------------------------
 function S:draw()
  if not self.active then return end
  local g=self.g;local k=g.kit;if not k then return end
  local a=g.safe_area();local W=math.min(800,a.w-24);local H=a.h-24;local x0=a.x+(a.w-W)/2;local y0=a.y+12
  local LW=math.floor(W*.46);local RX=x0+LW+32;local RW=W-LW-52
  self.detail_w=RW
  local m=self:ensure()
  k.panel(x0,y0,W,H)
  local title=self.mode=='reward' and 'STAGE CLEAR: your drives' or 'YOUR DRIVES'
  k.text(x0+18,y0+28,title,'body','bone','left',{max_w=LW})
  local left=self:seconds_left()
  if left then k.text(x0+W-18,y0+28,('Time left %ds, then new drives are sorted for you'):format(ceil(left)),'body',left<8 and 'danger' or 'muted','right',{max_w=W-LW-40}) end
  local pitch=24;local visible=math.min(S.tuning.visible_rows,math.floor((H-120)/pitch))
  local first=self.first or 1
  if self.focus<first then first=self.focus elseif self.focus>first+visible-1 then first=self.focus-visible+1 end
  self.first=math.max(1,first);first=self.first
  local ly=y0+44
  for i=first,math.min(#m.rows,first+visible-1) do
   local r=m.rows[i];local ry=ly+(i-first)*pitch
   if r.kind=='header' then k.text(x0+18,ry+16,r.label,'body',r.colour or 'gold','left',{max_w=LW})
   elseif r.kind=='note' then k.text(x0+28,ry+16,r.label,'body','muted','left',{max_w=LW})
   else
    local sel=i==self.focus
    if sel then k.button(x0+14,ry,LW,r.label,'sel',{h=pitch-2})
    else k.text(x0+28,ry+16,r.label,'body',r.record and T().rarity_colour[r.record.rarity] or 'bone','left',{max_w=LW-40}) end
    if r.record then g.fill(x0+14+LW-18,ry+5,12,12,T().base_colour[r.record.colour] or 0xFFFFFFFF) end
   end
  end
  if first>1 then k.text(x0+14+LW-6,ly-4,'^','body','muted','right') end
  if first+visible-1<#m.rows then k.text(x0+14+LW-6,ly+visible*pitch+10,'v','body','muted','right') end
  local per=math.max(1,math.floor((H-44-70)/19));local pages=math.max(1,math.ceil(#m.lines/per));self.page=math.min(self.page,pages-1)
  for i=1,per do local l=m.lines[self.page*per+i];if l and l[1]~='' then k.text(RX,y0+60+(i-1)*19,l[1],'body',l[2] or 'bone','left',{max_w=RW}) end end
  if pages>1 then k.text(x0+W-18,y0+H-70,('Text %d/%d: Left / Right'):format(self.page+1,pages),'body','muted','right',{max_w=RW}) end
  k.text(x0+18,y0+H-44,self.notice or m.controls,'body',self.notice and 'gold' or 'bone','left',{max_w=W-36})
  k.text(x0+18,y0+H-22,self.mode=='reward' and 'B: jump to Continue   Up/Down: move   Left/Right: more text' or 'B or Z+START: close   Up/Down: move   Left/Right: more text','body','muted','left',{max_w=W-36})
 end
 -- The screen as text: what a player would read (the model, no pixels).
 function S:dump()
  local out={};local m=self:ensure()
  out[#out+1]=('screen %s view=%s focus=%d time_left=%s'):format(self.mode,self.view,self.focus,tostring(self:seconds_left() and math.floor(self:seconds_left())))
  for i,r in ipairs(m.rows) do out[#out+1]=(i==self.focus and ' > ' or '   ')..r.label end
  out[#out+1]='-- detail'
  for _,l in ipairs(m.lines) do out[#out+1]='   '..l[1] end
  out[#out+1]='-- controls: '..tostring(self.notice or m.controls)
  return out
 end
 return S
end
