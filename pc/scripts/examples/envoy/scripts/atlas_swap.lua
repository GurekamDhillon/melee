-- The Envoy swap screen as a gd.ui description (Atlas step 3; spec 13.3): "BAG FULL" (a drive arrived, nothing merges, nothing is free)
-- and "SWAP" (a bag drive wants an equipped slot). The six equipped cells (and, for a new drive, the four bag cells) are the targets;
-- the INCOMING drive is the screen's footer. The explainer says what A does to the focused target before you press it: what is given up,
-- and whether it goes to the bag or is gone for good. Link marks stay: the synergy links between held pieces are grid links, drawn under
-- the cells (the world-space chain flashes are not touched).
--
-- LOGIC IS NOT HERE. Like the bag and the reward screen this attaches to the RunScreen: A and B run its own press handlers (replace_with,
-- leave_choice, the "press B again to leave it behind" confirm), directions go to the engine, and every outcome keeps the host's log
-- lines. When the layout goes back to 'main' the screen hands over to the bag or the reward screen it came from, with the focus where it was
-- (the legacy back_focus rule).
return function(D)
 local K=D.atlas_kit
 local W={}
 local BASE='envoy.swap'
 local DIRS={up=true,down=true,left=true,right=true}

 function W.id_for(S) return K.id(BASE,S) end

 local function block_cells(self,S)
  local blocks,cells={},{}
  for _,b in ipairs(S.blocks) do
   if b.id~='in' then
    local name,count=D.atlas_bag.split_title(b.title or b.id)
    local bd={id=b.id,title=name,count=count,cols=math.max(1,math.min(b.cols or 1,12)),cells={}}
    for i=1,math.min(#b.cells,12) do
     local c=b.cells[i]
     if c then local d=D.atlas_bag.cell_desc(b.id,i,c);bd.cells[#bd.cells+1]=d;cells[d.id]=c end
    end
    blocks[#blocks+1]=bd
   end
  end
  return blocks,cells
 end

 -- the outcome sentence of a target: the line after the first blank line of its legacy detail text ("It goes to your bag." / "It is gone for good.")
 local function outcome(lines)
  local seen=false
  for i,l in ipairs(lines) do
   if l=='' then seen=true elseif seen then return l end
  end
  return nil
 end

 function W.explainer(self,cid,bid)
  local S=self.S;local c=self.cells[cid]
  if not c then return nil end
  local sw=S.swap;local lines=c.lines or S:detail_lines(c)
  local ex={kicker=(sw and sw.from=='bag' and 'SWAP IN' or 'GIVE UP')..(bid=='eq' and (' SLOT '..tostring(cid:match(':(%d+)$'))) or ' BAG '..tostring(cid:match(':(%d+)$'))),title=c.name or ''}
  if c.empty or (c.ref and c.ref.kind=='locked') then ex.what=lines[1] or '';return ex end
  local what=outcome(lines)
  if sw and sw.record and not what then what=lines[1] end
  ex.what=K.fit_rule(S,c.name,what or '','atlas swap')
  if type(c.icon)=='table' and c.icon.kind=='model' then ex.media={model=c.icon.asset,ring=c.icon.ring} end
  if lines[1] then ex.from={text=lines[1]} end
  return ex
 end

 -- the synergy links between held pieces (the swap layout has the eq and bag cells), as grid links with the archetype's colour
 local function links_of(S,known)
  local sf=S.host and S.host.synfx
  if not sf or not sf.grid_model then return nil end
  local ok,m=pcall(sf.grid_model,sf,S)
  if not ok or type(m)~='table' then return nil end
  local out={}
  for _,l in ipairs(m.links or {}) do
   if known[l.a] and known[l.b] and #out<16 then out[#out+1]={a=l.a,b=l.b,rgba=l.arch and l.arch.colour and ((l.arch.colour<<8)|255) or nil} end
  end
  return out
 end

 function W.describe(S,self)
  local blocks,cells=block_cells(self,S)
  self.cells=cells;self.id=W.id_for(S)
  local sw=S.swap
  local incoming
  for _,b in ipairs(S.blocks) do if b.id=='in' then incoming=b.cells[1] end end
  local footer
  if incoming then
   local ic=incoming.icon
   footer={label='INCOMING',text=incoming.name or '',a=(type(ic)=='table' and ic.kind=='model') and ic.asset or nil}
  end
  local known={};for id in pairs(cells) do known[id]=true end
  local function action(btn) return function(cid) local c=self.cells[cid];return c and c.actions and c.actions[btn] or nil end end
  local trail=K.parents();trail.title=(sw and sw.from=='bag') and 'SWAP' or 'BAG FULL'
  return {
   id=self.id,trail=trail,chapter=1,persist=true,
   primary={kind='grid',blocks=blocks,footer=footer,links=links_of(S,known)},
   explainer={width='narrow',provide=function(cid,bid) return W.explainer(self,cid,bid) end},
   keys={{'A',action('A')},{'B',action('B')}},
   counter=function(cid) local c=self.cells[cid];local r=c and c.ref;if r and r.where=='equipped' then return ('Slot %d / 6'):format(r.index) end end,
   input='feed',port=K.seat_port(S),
   on=self.on}
 end

 function W.register(self)
  local S=self.S
  local ok,err=pcall(function() S.g.ui.screen(W.describe(S,self)) end)
  if not ok then
   K.log_once(S,'the engine refused the screen, the legacy swap grid stays: '..tostring(err),'atlas swap')
   W.detach(S);return false
  end
  self.blocks_ref=S.blocks
  return true
 end

 -- where the focus goes when the swap hands back: the legacy back_focus {block id, index} as an Atlas cell
 local function restore_focus(S,id,mode)
  local bf=S.back_focus;S.back_focus=nil
  if not bf then return end
  local bid,idx=bf[1],bf[2]
  local ok
  if mode=='reward' then
   local cell=(bid=='koffer' and 'key:' or 'offer:')..tostring(idx)
   ok=pcall(S.g.ui.set_focus,id,'cards',cell)
  else ok=pcall(S.g.ui.set_focus,id,bid,bid..':'..tostring(idx)) end
  return ok
 end

 function W.attach(S)
  if S.atlas_swap or S.layout~='swap' or not S.swap or not K.enabled(S.g) then return false end
  local g=S.g
  local self={S=S,cells={}}
  S.atlas_swap=self
  local orig_press,orig_refresh,orig_notify=S.press,S.refresh,S.notify
  local back_focus=S.back_focus   -- remembered by enter_swap: the place to return to
  self.on={
   accept=function() S:press('accept') end,
   back=function() S:press('back') end,
   focus=function() end}
  S.draw=function() end
  S.sync=function() end
  S.focused=function()
   local ok,cid=pcall(g.ui.focus,self.id)
   local c=ok and cid and self.cells[cid] or nil
   return c,c and c.ref
  end
  S.press=function(inst,action)
   if not inst.active then return end
   if DIRS[action] then inst.confirm=nil;g.ui.feed(self.id,action);return end
   return orig_press(inst,action)
  end
  S.refresh=function(inst)
   orig_refresh(inst)
   if not inst.active then return end
   if inst.layout~='swap' then   -- the swap is over: back to the screen it came from, the focus where it was
    local mode=inst.mode;local bf=back_focus or inst.back_focus
    W.detach(inst)
    inst.back_focus=bf
    if mode=='reward' and D.atlas_reward then
     if #inst.host.offers==0 and #inst.host.key_offers==0 and #inst.host.decide==0 then D.atlas_reward.complete(inst)   -- the swap took the last offer
     elseif D.atlas_reward.attach(inst) then restore_focus(inst,D.atlas_reward.id_for(inst),'reward') end
    elseif mode=='bag' and D.atlas_bag and D.atlas_bag.enabled(inst.g) then
     if D.atlas_bag.attach(inst) then restore_focus(inst,D.atlas_bag.id_for(inst),'bag') end
    end
    inst.back_focus=nil
    return
   end
   if inst.blocks~=self.blocks_ref then W.register(self) end
  end
  S.notify=function(inst,text) orig_notify(inst,text);pcall(g.ui.note,{text=text,kind='info',seconds=4}) end
  self.id=W.id_for(S)
  if not W.register(self) then return false end
  g.ui.open(self.id)
  return true
 end

 function W.detach(S)
  local self=S.atlas_swap
  if not self then return end
  S.atlas_swap=nil
  for _,k in ipairs({'draw','focused','sync','press','refresh','notify'}) do S[k]=nil end
  if S.g and S.g.ui then pcall(S.g.ui.close,self.id or W.id_for(S)) end
 end

 return W
end
