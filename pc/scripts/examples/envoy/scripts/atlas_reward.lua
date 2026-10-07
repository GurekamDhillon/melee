-- The Envoy reward screen as a gd.ui description (Atlas step 3; spec 13.3). The stage-clear reward moment: a row of OFFER CARDS (the drives on
-- offer, or, when there are none, the keystone offer), each one with its model well (a keystone is an arch stone with its letter), its name,
-- ONE short rule and a tag for what A would do. The explainer shows the focused card's name, its rule (a card with several pages them with
-- Z / L / R: "RULE 1 OF 2"), what the pick does (WITH) and what it is (FROM). The countdown sits at the trail's right end and is
-- re-registered once a second.
--
-- LOGIC IS NOT HERE. Like the bag (atlas_bag.lua) this attaches to a RunScreen: A, X and B run the screen's own press handlers, so every
-- outcome (merge, equip, bag, ask which to give up, skip, the timeout and the engine's own hold ending) stays the run's tested logic and the
-- host's log lines. Directions go to the engine; the pad is never masked from here (menu_input.lua hides D-pad and START and does not
-- mask online).
--
-- WHEN THE LAST OFFER IS RESOLVED the moment is over: the screen ends it (S:finish('done')) and says what happened as the pickup note.
-- A bag drive that needs a swap hands the screen to atlas_swap (layout 'swap'); the swap hands back.
--
-- ONE SCREEN PER SEAT (K.id); persist = true: the screen spans the ladder's VS-to-VS scenes (a scene exit does not close it: the owner does).
return function(D)
 local K=D.atlas_kit
 local R={}
 local BASE='envoy.reward';local KEY_BASE='envoy.keystone'
 local DIRS={up=true,down=true,left=true,right=true}
 local MAX_CARDS=4   -- gw_ui_screen.h AT_MAX_CARDS

 local function offer_block(S)
  for _,b in ipairs(S.blocks) do if b.id=='offer' or b.id=='koffer' then return b end end
 end
 function R.id_for(S)
  local b=offer_block(S)
  return K.id(b and b.id=='koffer' and KEY_BASE or BASE,S)
 end

 -- the rules a card shows, one at a time: a drive's detail lines after its name line up to the first blank line; a keystone's lines from the first
 function R.rules(cell,lines)
  local rules={}
  local from=(cell.ref and cell.ref.kind=='koffer') and 1 or 2
  for i=from,#lines do local l=lines[i];if l=='' then break end;rules[#rules+1]=l end
  return rules
 end

 local function lines_of(S,c)
  if not c.detail_done then c.lines=S:detail_lines(c);c.detail_done=true end
  return c.lines or {c.name}
 end

 -- the short word on a card for what A does with it, and the explainer's WITH tag
 local TAG={Merge={'+ MERGE','jade','Merges'},Equip={'EQUIP','plain','Equips'},Take={'TAKE','plain','Take'},['Replace which?']={'SWAP','sun','Swap'}}

 local function card_desc(S,self,block,i,c)
  local d={id=(block.id=='koffer' and 'key:' or 'offer:')..i,name=c.name or ''}
  local ic=c.icon
  if type(ic)=='table' then
   if ic.kind=='model' then d.model=ic.asset;d.ring=ic.ring
   elseif ic.kind=='letter' then d.letter=ic.letter;d.rgba=ic.colour end
  end
  local lines=lines_of(S,c);local rules=R.rules(c,lines)
  local k=K.rule_index(self,d.id,#rules)
  d.rule=rules[k] and K.fit_rule(S,c.name,rules[k],'atlas reward') or ''
  local label=c.actions and c.actions.A
  local t=label and TAG[label]
  if block.id=='koffer' then d.tag='KEYSTONE';d.tag_tone='sun'
  elseif t then d.tag=t[1];d.tag_tone=t[2] end
  return d
 end

 -- what the explainer shows for one card: ONE rule ("RULE k OF n" when there are several), what the pick does, what it is
 function R.explainer(self,cid)
  local S=self.S;local c=self.cells[cid]
  if not c then return nil end
  local block=self.block
  local lines=lines_of(S,c);local rules=R.rules(c,lines)
  local idx=cid:match(':(%d+)$')
  local ex={kicker=(block.id=='koffer' and 'KEYSTONE OFFER ' or 'OFFER ')..tostring(idx),title=c.name or ''}
  local k=K.rule_index(self,cid,#rules)
  if #rules>1 then ex.kicker=ex.kicker..' - RULE '..k..' OF '..#rules end
  ex.what=rules[k] and K.fit_rule(S,c.name,rules[k],'atlas reward') or ''
  if type(c.icon)=='table' and c.icon.kind=='model' then ex.media={model=c.icon.asset,ring=c.icon.ring} end
  local label=c.actions and c.actions.A;local t=label and TAG[label]
  if t then ex['with']={t[3]} end
  if block.id=='koffer' then ex.from={text=(D.keystones.family_names[D.keystones.family(c.ref.id)] or 'Wild')..' keystone, permanent this run'}
  elseif lines[1] then ex.from={text=lines[1]} end
  return ex
 end

 local function rule_count(self,cid)
  local c=self.cells[cid];if not c then return 0 end
  return #R.rules(c,lines_of(self.S,c))
 end

 function R.describe(S,self)
  local block=offer_block(S);local cells,cards={},{}
  self.block=block;self.id=R.id_for(S)
  if block then
   if #block.cells>MAX_CARDS then K.log_once(S,('block "%s" has %d offers; the Atlas screen shows %d'):format(block.id,#block.cells,MAX_CARDS),'atlas reward') end
   for i=1,math.min(#block.cells,MAX_CARDS) do
    local c=block.cells[i]
    if c then local d=card_desc(S,self,block,i,c);cards[#cards+1]=d;cells[d.id]=c end
   end
  end
  self.cells=cells
  local function action(btn) return function(cid) local c=self.cells[cid];local a=c and c.actions and c.actions[btn];if a then return a end;return nil end end
  local h=S.host
  local trail=K.parents();trail.title='STAGE CLEAR'
  local left=S.seconds_left and S:seconds_left()
  return {
   id=self.id,trail=trail,chapter=1,persist=true,
   primary={kind='cards',cards=cards},
   explainer={width='narrow',provide=function(cid) return R.explainer(self,cid) end},
   keys={{'A',action('A')},{'X',action('X')},
         {'Z',function(cid) if rule_count(self,cid)>1 then return 'More' end end},
         {'B',function() return ((#h.offers>0 or #h.key_offers>0) and 'Skip' or 'Continue') end}},
   counter=function() return h.milestone_line end,
   countdown=left and math.ceil(left) or nil,
   input='feed',port=K.seat_port(S),
   on=self.on}
 end

 function R.register(self)
  local S=self.S
  local ok,err=pcall(function() S.g.ui.screen(R.describe(S,self)) end)
  if not ok then
   K.log_once(S,'the engine refused the screen, the legacy reward screen stays: '..tostring(err),'atlas reward')
   R.detach(S);return false
  end
  self.blocks_ref=S.blocks;self.shown_left=S.seconds_left and S:seconds_left() and math.ceil(S:seconds_left()) or nil
  return true
 end

 -- The last offer is resolved: what happened is the pickup note, and the moment ends (the legacy screen stayed up for a "Continue" press).
 function R.complete(S)
  local h=S.host;local text=S.notice
  R.detach(S);if S.atlas_swap and D.atlas_swap then D.atlas_swap.detach(S) end
  if text and h.hud and h.hud.show_card then pcall(h.hud.show_card,h.hud,'Reward',{text}) end
  S:finish('done')
 end

 function R.more(self,dir)
  local g=self.S.g
  local ok,cid=pcall(g.ui.focus,self.id)
  if not ok or not cid then return false end
  local n=rule_count(self,cid)
  if n<2 then return false end
  K.page(self,cid,n,dir)
  R.register(self)
  return true
 end

 function R.attach(S)
  if S.atlas_reward or S.mode~='reward' or S.layout~='main' or not K.enabled(S.g) or not offer_block(S) then return false end
  local g=S.g
  local self={S=S,cells={}}
  S.atlas_reward=self
  local orig_press,orig_refresh,orig_notify,orig_tick=S.press,S.refresh,S.notify,S.tick
  self.on={
   accept=function() S:press('accept') end,
   back=function() S:press('back') end,
   alt={X=function() S:press('x') end,Z=function() R.more(self,1) end},
   page=function(dir) R.more(self,dir) end,
   focus=function() self.rule_cid=nil;R.register(self) end}
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
   if action=='more' then R.more(self,1);return end
   return orig_press(inst,action)
  end
  S.refresh=function(inst)
   orig_refresh(inst)
   if not inst.active or inst.mode~='reward' then return end
   if inst.layout~='main' then R.detach(inst);if D.atlas_swap then D.atlas_swap.attach(inst) end;return end   -- the swap layout: atlas_swap's screen
   local h=inst.host
   if #h.offers==0 and #h.key_offers==0 then R.complete(inst);return end   -- the last offer is resolved: the moment is over
   if inst.blocks~=self.blocks_ref then
    local id_before=self.id;local b=offer_block(inst)
    if b and R.id_for(inst)~=id_before then pcall(g.ui.close,id_before) end   -- drive offers done: the keystone offer is its own screen
    R.register(self);g.ui.open(self.id)
   end
  end
  S.notify=function(inst,text) orig_notify(inst,text);pcall(g.ui.note,{text=text,kind='info',seconds=4}) end
  S.enter_swap=function(inst,sw) return K.swap_entry(inst,self.id)(inst,sw) end
  -- the countdown is re-registered once a second (the screen costs O(1) per frame)
  S.tick=function(inst)
   orig_tick(inst)
   if S.atlas_reward~=self or not inst.active or inst.preview then return end
   local left=inst:seconds_left()
   if left and math.ceil(left)~=self.shown_left then R.register(self) end
  end
  -- the pad's Z: the legacy input does not read it, so its poll is wrapped for the life of the screen
  local inp=S.input
  if type(inp)=='table' and type(inp.poll)=='function' then
   self.input_poll=rawget(inp,'poll');local poll=inp.poll
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
  self.id=R.id_for(S)
  if not R.register(self) then return false end
  g.ui.open(self.id)
  return true
 end

 function R.detach(S)
  local self=S.atlas_reward
  if not self then return end
  S.atlas_reward=nil
  for _,k in ipairs({'draw','focused','sync','press','refresh','notify','tick','enter_swap'}) do S[k]=nil end
  if self.input then self.input.poll=self.input_poll end
  if S.g and S.g.ui then pcall(S.g.ui.close,K.id(BASE,S));pcall(S.g.ui.close,K.id(KEY_BASE,S)) end
 end

 return R
end
