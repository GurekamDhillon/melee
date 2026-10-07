-- The Envoy bag as a gd.ui description (Atlas step 1; spec section 13.1). It reads the cell data run_screen.lua already builds
-- (S.blocks: blocks of cells with name, icon, flags, actions, ref and detail lines) and describes it; the engine lays it out,
-- moves focus, reads mouse and keyboard and draws the key hints. The pad stays with run_screen's own input (menu_input.lua hides
-- the D-pad and START from the game and does not mask online), so the screen is input = 'feed': directions are fed to the
-- engine, A/B/X/Y run the legacy handlers. Off by default (A.set, console `uxatlas on`). Anything it cannot describe (the swap
-- layout, no gd.ui, no Atlas fonts) leaves the legacy grid screen in charge.
--
-- ONE RULE AT A TIME (the owner's ruling: one short rule per piece on screen). The explainer shows one rule of the focused
-- drive; a drive with more than one has "RULE 1 OF 2" in the explainer's kicker, and Z ("More"; L and R, Tab and Shift+Tab
-- through on.page) steps to the next. Y is the legacy Discard and X the legacy To-bag/Keep, so the step key is Z. The pad's Z
-- reaches the screen through a wrapper around the legacy input's poll (the legacy input does not read Z); the engine delivers
-- Z for the mouse (a click on the hint). A rule is never cut mid-word: a rule longer than the explainer's field is broken at
-- its last word that fits, ends in "...", and is logged once as a content bug.
--
-- ONE SCREEN PER SEAT: the id is "envoy.bag" for port 1 and "envoy.bag.pN" for port N, so two seats' bags on one machine are
-- two screens and closing one never closes the other.
return function(D)
 local K=D.atlas_kit   -- the shared helpers (the switch, the seat id, log once, the rules, paging), step 3
 local A={setting=K.setting}
 local BASE_ID='envoy.bag'
 local KICKER={eq='EQUIPPED SLOT',bag='BAG CELL',key='KEYSTONE',offer='OFFERED',koffer='KEYSTONE OFFER'}
 local DIRS={up=true,down=true,left=true,right=true}
 local MAX_CELLS=12   -- gw_ui_screen.h AT_MAX_CELLS: a block with more is refused whole, so a longer one is shown cut, with a note

 function A.set(on) K.set(on) end
 function A.enabled(g) return K.enabled(g) end
 function A.id_for(S) return K.id(BASE_ID,S) end
 local function log_once(S,text) K.log_once(S,text,'atlas bag') end

 -- 'EQUIPPED 5/6' -> 'EQUIPPED', '5 / 6'
 local function split_title(t)
  local name,a,b=tostring(t):match('^(.-)%s+(%d+)/(%d+)$')
  if name then return name,a..' / '..b end
  return tostring(t),nil
 end

 local function cell_desc(block_id,i,c)
  local d={id=block_id..':'..i,name=c.name or '',flags={},pips=c.pips or 0}
  if block_id=='eq' or block_id=='bag' then d.index=i end
  if c.empty then d.flags.empty=true end
  for _,f in ipairs(c.flags or {}) do if f=='locked' or f=='merge' or f=='new' then d.flags[f]=true end end
  if d.flags.locked then d.flags.disabled=true end   -- a locked slot takes focus and explains itself, and never fires a handler
  local ic=c.icon
  if type(ic)=='table' then
   if ic.kind=='model' then d.model=ic.asset;d.ring=ic.ring
   elseif ic.kind=='letter' then d.letter=ic.letter;d.color=ic.colour end
  end
  return d
 end

 A.cell_desc=cell_desc;A.split_title=split_title   -- shared with the swap screen (atlas_swap.lua)

 -- IF YOU MERGE: the focused drive, the drive it merges into and the result (only when the legacy plan says merge)
 function A.footer(S,c)
  local ref=c and c.ref
  if not ref or not ref.record or (ref.kind~='bag' and ref.kind~='offer') then return nil end
  local ok,res=pcall(function()
   local plan=S.host:plan_take(ref.record,ref.kind=='bag' and {where='bag',index=ref.index} or nil)
   if plan.action~='merge' then return nil end
   local target
   for _,b in ipairs(S.blocks) do for _,t in ipairs(b.cells) do
    if t.ref and t.ref.where==plan.loc.where and t.ref.index==plan.loc.index and not t.empty then target=t end
   end end
   local out=S:model_desc(plan.merged.colour,plan.merged.rarity)
   local name=S.host:merge_text(plan)
   return {label='IF YOU MERGE',a=c.icon and c.icon.asset,b=target and target.icon and target.icon.asset,out=out and out.asset,text=(name or 'It')..' gets stronger.'}
  end)
  return ok and res or nil
 end

 local function fit_rule(S,name,rule) return K.fit_rule(S,name,rule,'atlas bag') end
 function A.rules(lines) return K.rules(lines) end

 -- what the explainer shows for one cell: the drive's name line, ONE rule (and "RULE k OF n" when there are several)
 function A.explainer(self,cid,bid)
  local S=self.S;local c=self.cells[cid]
  if not c then return nil end
  local lines=c.lines or S:detail_lines(c)
  local idx=cid:match(':(%d+)$')
  local ex={kicker=(KICKER[bid] or bid:upper())..(bid~='key' and idx and (' '..idx) or ''),title=c.name or ''}
  if c.empty or (c.ref and c.ref.kind=='locked') then ex.what=lines[1] or '';return ex end
  local rules=A.rules(lines)
  local k=K.rule_index(self,cid,#rules)
  if #rules>1 then ex.kicker=ex.kicker..' - RULE '..k..' OF '..#rules end
  ex.what=rules[k] and fit_rule(S,c.name,rules[k]) or ''
  if type(c.icon)=='table' and c.icon.kind=='model' then ex.media={model=c.icon.asset,ring=c.icon.ring} end
  if lines[1] then ex.from={text=lines[1]} end
  return ex
 end

 -- the number of rules of a cell, for the "More" hint
 local function rule_count(self,cid)
  local c=self.cells[cid]
  if not c or c.empty or (c.ref and c.ref.kind=='locked') then return 0 end
  return #A.rules(c.lines or self.S:detail_lines(c))
 end

 function A.describe(S,self)
  local cells,blocks={}, {}
  self.id=self.id or A.id_for(S)
  for _,b in ipairs(S.blocks) do
   local name,count=split_title(b.title or b.id)
   local bd={id=b.id,title=name,count=count,cols=math.max(1,math.min(b.cols or 1,MAX_CELLS)),cells={}}
   if b.id=='key' or b.id=='koffer' then bd.kind='stones' end
   if b.id=='key' then bd.note=(#b.cells==1) and 'One held' or (#b.cells..' held') end
   if #b.cells>MAX_CELLS then   -- the engine refuses a block of more than MAX_CELLS cells and the player would drop to the legacy bag
    bd.note=('+%d more not shown'):format(#b.cells-MAX_CELLS)
    log_once(S,('block "%s" has %d cells; the Atlas screen shows %d'):format(b.id,#b.cells,MAX_CELLS))
   end
   for i=1,math.min(#b.cells,MAX_CELLS) do
    local c=b.cells[i]
    if c then local d=cell_desc(b.id,i,c);bd.cells[#bd.cells+1]=d;cells[d.id]=c end
   end
   blocks[#blocks+1]=bd
  end
  self.cells=cells
  local ok,fid=pcall(S.g.ui.focus,self.id)
  local focus=ok and fid and cells[fid] or nil
  local function action(btn) return function(cid) local c=self.cells[cid];return c and c.actions and c.actions[btn] or nil end end
  local trail=K.parents();trail.title='YOUR DRIVES'
  return {
   id=self.id,trail=trail,chapter=1,persist=true,
   primary={kind='grid',blocks=blocks,footer=A.footer(S,focus)},
   explainer={width='narrow',provide=function(cid,bid) return A.explainer(self,cid,bid) end},
   keys={{'A',action('A')},{'X',action('X')},{'Y',action('Y')},
         {'Z',function(cid) if rule_count(self,cid)>1 then return 'More' end end},
         {'START','Close'},{'B','Close'}},
   counter=function(cid)
    local c=self.cells[cid];local r=c and c.ref
    if r and r.where=='bag' then return ('Bag %d / %d'):format(r.index,S.host:bag():capacity()) end
    if r and r.where=='equipped' then return ('Slot %d / 6'):format(r.index) end
   end,
   input='feed',port=(S.host and S.host.seat and S.host.seat.port) or 1,
   on=self.on}
 end

 function A.register(self)
  local S=self.S
  local ok,err=pcall(function() S.g.ui.screen(A.describe(S,self)) end)
  if not ok then
   log_once(S,'the engine refused the screen, the legacy bag stays: '..tostring(err))
   A.detach(S);return false
  end
  self.blocks_ref=S.blocks
  -- a re-registration can move the focus (a merge, a discard): the legacy S:sync re-ran mark_target whenever the focused cell
  -- changed, so do the same for the cell the engine's focus now rests on
  local fok,fid=pcall(S.g.ui.focus,self.id)
  local c=fok and fid and self.cells[fid] or nil
  if c and c~=self.marked then self.marked=c;S:mark_target(c) end
  return true
 end

 -- step the shown rule of the focused cell (dir +1 or -1), wrapping; re-registers so the explainer refreshes
 function A.more(self,dir)
  local g=self.S.g
  local ok,cid=pcall(g.ui.focus,self.id)
  if not ok or not cid then return false end
  local n=rule_count(self,cid)
  if n<2 then return false end
  K.page(self,cid,n,dir)
  A.register(self)
  return true
 end

 function A.attach(S)
  if S.atlas or S.mode~='bag' or S.layout~='main' or not A.enabled(S.g) then return false end
  local g=S.g
  local self={S=S,cells={},id=A.id_for(S)}
  local ID=self.id
  S.atlas=self
  local orig_press,orig_refresh,orig_notify=S.press,S.refresh,S.notify
  local function focused_cell()
   local ok,cid=pcall(g.ui.focus,ID)
   return ok and cid and self.cells[cid] or nil
  end
  self.on={
   accept=function() S:press('accept') end,
   back=function() S:press('back') end,
   alt={X=function() S:press('x') end,Y=function() S:press('y') end,Z=function() A.more(self,1) end},
   page=function(dir) A.more(self,dir) end,
   start=function() S:press('start') end,
   focus=function(cid)
    local c=self.cells[cid];S.confirm=nil;self.rule_cid=nil
    if c then
     self.marked=c
     S:mark_target(c)
     if not c.detail_done then c.lines=S:detail_lines(c);c.detail_done=true end
    end
    A.register(self)
   end}
  S.draw=function() end
  S.sync=function() end
  S.focused=function() local c=focused_cell();return c,c and c.ref end
  S.press=function(inst,action)
   if not inst.active then return end
   if DIRS[action] then inst.confirm=nil;g.ui.feed(ID,action);inst:refresh();return end
   if action=='more' then A.more(self,1);return end
   return orig_press(inst,action)
  end
  S.refresh=function(inst)
   orig_refresh(inst)
   if inst.layout~='main' or inst.mode~='bag' then
    local swap=inst.layout=='swap'
    A.detach(inst)
    if swap and D.atlas_swap then D.atlas_swap.attach(inst) end   -- the swap layout is its own Atlas screen (step 3)
    return
   end
   if inst.blocks~=self.blocks_ref then A.register(self) end
  end
  S.notify=function(inst,text) orig_notify(inst,text);pcall(g.ui.note,{text=text,kind='info',seconds=4}) end
  S.enter_swap=function(inst,sw) return K.swap_entry(inst,ID)(inst,sw) end
  -- the pad's Z: the legacy input does not read it, so its poll is wrapped for the life of the screen
  local inp=S.input
  if type(inp)=='table' and type(inp.poll)=='function' then
   self.input_poll=rawget(inp,'poll');local poll=inp.poll
   -- the bag opens with the Z+START chord, so Z is normally still held now: start from the held state, as the legacy S:open does
   -- for the buttons it uses (previous.x, previous.y, ...), so only a fresh press of Z steps the rule
   local z_prev=false
   do local ok,p=pcall(g.pad,inp.port or 1,true);if ok and type(p)=='table' and p.Z then z_prev=true end end
   inp.poll=function(i,...)
    local out=poll(i,...)
    local ok,p=pcall(g.pad,i.port or 1,true)
    local z=ok and type(p)=='table' and p.Z and true or false
    if z and not z_prev then out[#out+1]='more' end
    z_prev=z
    return out
   end
   self.input=inp
  end
  if not A.register(self) then return false end
  g.ui.open(ID)
  return true
 end

 function A.detach(S)
  local self=S.atlas
  if not self then return end
  S.atlas=nil
  for _,k in ipairs({'draw','focused','sync','press','refresh','notify','enter_swap'}) do S[k]=nil end
  if self.input then self.input.poll=self.input_poll end   -- nil again: the legacy method on the class is back
  if S.g and S.g.ui then pcall(S.g.ui.close,self.id or BASE_ID) end
 end

 return A
end
