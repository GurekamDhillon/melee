-- The Envoy bag as a gd.ui description (Atlas step 1; spec section 13.1). It reads the cell data run_screen.lua already builds
-- (S.blocks: blocks of cells with name, icon, flags, actions, ref and detail lines) and describes it; the engine lays it out,
-- moves focus, reads mouse and keyboard and draws the key hints. The pad stays with run_screen's own input (menu_input.lua hides
-- the D-pad and START from the game and does not mask online), so the screen is input = 'feed': directions are fed to the
-- engine, A/B/X/Y run the legacy handlers. Off by default (A.set, console `uxatlas on`). Anything it cannot describe (the swap
-- layout, no gd.ui, no Atlas fonts) leaves the legacy grid screen in charge.
return function(D)
 local A={setting={on=false}}
 local ID='envoy.bag'
 local KICKER={eq='EQUIPPED SLOT',bag='BAG CELL',key='KEYSTONE',offer='OFFERED',koffer='KEYSTONE OFFER'}
 local DIRS={up=true,down=true,left=true,right=true}

 function A.set(on) A.setting.on=on and true or false end

 function A.enabled(g)
  if not A.setting.on or type(g.ui)~='table' or type(g.ui.available)~='function' then return false end
  return (g.ui.available()) and true or false
 end

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
  local ic=c.icon
  if type(ic)=='table' then
   if ic.kind=='model' then d.model=ic.asset;d.ring=ic.ring
   elseif ic.kind=='letter' then d.letter=ic.letter;d.color=ic.colour end
  end
  return d
 end

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

 -- what the explainer shows for one cell: the legacy detail lines, split into the drive's name line, its rule lines, and the rest
 function A.explainer(self,cid,bid)
  local S=self.S;local c=self.cells[cid]
  if not c then return nil end
  local lines=c.lines or S:detail_lines(c)
  local idx=cid:match(':(%d+)$')
  local ex={kicker=(KICKER[bid] or bid:upper())..(bid~='key' and idx and (' '..idx) or ''),title=c.name or ''}
  if c.empty or (c.ref and c.ref.kind=='locked') then ex.what=lines[1] or '';return ex end
  local rules,blank={},false
  for i=2,#lines do local l=lines[i];if l=='' then blank=true elseif not blank then rules[#rules+1]=l end end
  ex.what=table.concat(rules,'\n')
  if type(c.icon)=='table' and c.icon.kind=='model' then ex.media={model=c.icon.asset,ring=c.icon.ring} end
  if lines[1] then ex.from={text=lines[1]} end
  return ex
 end

 function A.describe(S,self)
  local cells,blocks={}, {}
  for _,b in ipairs(S.blocks) do
   local name,count=split_title(b.title or b.id)
   local bd={id=b.id,title=name,count=count,cols=math.max(1,math.min(b.cols or 1,12)),cells={}}
   if b.id=='key' or b.id=='koffer' then bd.kind='stones' end
   if b.id=='key' then bd.note=(#b.cells==1) and 'One held' or (#b.cells..' held') end
   for i=1,#b.cells do
    local c=b.cells[i]
    if c then local d=cell_desc(b.id,i,c);bd.cells[#bd.cells+1]=d;cells[d.id]=c end
   end
   blocks[#blocks+1]=bd
  end
  self.cells=cells
  local ok,fid=pcall(S.g.ui.focus,ID)
  local focus=ok and fid and cells[fid] or nil
  local function action(btn) return function(cid) local c=self.cells[cid];return c and c.actions and c.actions[btn] or nil end end
  return {
   id=ID,trail={'SOLO','ENVOY',title='YOUR DRIVES'},chapter=1,
   primary={kind='grid',blocks=blocks,footer=A.footer(S,focus)},
   explainer={width='narrow',provide=function(cid,bid) return A.explainer(self,cid,bid) end},
   keys={{'A',action('A')},{'X',action('X')},{'Y',action('Y')},{'B','Close'}},
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
   if S.host and S.host.log then S.host:log('atlas bag: '..tostring(err)) end
   A.detach(S);return false
  end
  self.blocks_ref=S.blocks
  return true
 end

 function A.attach(S)
  if S.mode~='bag' or S.layout~='main' or not A.enabled(S.g) then return false end
  local g=S.g
  local self={S=S,cells={}}
  S.atlas=self
  local orig_press,orig_refresh,orig_notify=S.press,S.refresh,S.notify
  local function focused_cell()
   local ok,cid=pcall(g.ui.focus,ID)
   return ok and cid and self.cells[cid] or nil
  end
  self.on={
   accept=function() S:press('accept') end,
   back=function() S:press('back') end,
   alt={X=function() S:press('x') end,Y=function() S:press('y') end},
   focus=function(cid)
    local c=self.cells[cid];S.confirm=nil
    if c then
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
   return orig_press(inst,action)
  end
  S.refresh=function(inst)
   orig_refresh(inst)
   if inst.layout~='main' or inst.mode~='bag' then A.detach(inst);return end
   if inst.blocks~=self.blocks_ref then A.register(self) end
  end
  S.notify=function(inst,text) orig_notify(inst,text);pcall(g.ui.note,{text=text,kind='info',seconds=4}) end
  if not A.register(self) then return false end
  g.ui.open(ID)
  return true
 end

 function A.detach(S)
  if not S.atlas then return end
  S.atlas=nil
  for _,k in ipairs({'draw','focused','sync','press','refresh','notify'}) do S[k]=nil end
  if S.g and S.g.ui then pcall(S.g.ui.close,ID) end
 end

 return A
end
