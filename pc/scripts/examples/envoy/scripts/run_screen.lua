-- The run's one build screen, on the grid component (demos/grid-inventory, embedded as D.grid). It is used twice: as the
-- reward moment inside the hold after a stage clear (mode 'reward': a pick of drives and/or keystones, a countdown) and as
-- the bag screen opened with Z+START in a fight (mode 'bag'). When a drive arrives and nothing merges, nothing is free and
-- the bag is full, the same grid shows the swap layout ("which one do you give up?").
--   Blocks (cells, no row list): OFFERED drives / OFFERED keystones (reward only), EQUIPPED (always six cells, locks), BAG (four),
--   KEYSTONES (held only). One detail panel beside them describes the focused cell: one line per modifier, what A would do, and
--   build strength before -> after.
--   A the obvious thing (merge, else equip, else bag, else ask which to replace)   X to the bag   Y discard (asks twice)
--   B continue / close   D-pad or stick move   Z+START closes the bag screen
-- Controller only. Nothing is rebuilt per drawn frame: the blocks and the detail text are rebuilt when the bag, the offers or the
-- focus change (an event), and the draw replays the component's cached layout.
return function(D)
 local S={};S.__index=S
 S.tuning={hold_ticks=2850,tick_margin=90,safe_seconds=45}
 S.model_opts={yaw=18,pitch=12,margin=.02}   -- how a drive model sits in its cell (front-facing emblems, as wide as tall)
 local G=D.grid
 G.rarity.magic=G.rarity.magic or G.rarity.uncommon      -- Envoy's second rarity word
 G.palette.purple=G.palette.purple or 0xC79BFFFF
 local T=function() return D.drive_text end
 local function ceil(n) return math.ceil(n) end
 function S.new(g,host)
  return setmetatable({g=g,host=host,input=D.menu_input.new(g,host and host.seat and host.seat.port),active=false,mode='bag',layout='main',prev={},blocks={},descs={}},S)
 end
 function S:drives() return self.host.mods.drives end
 -- ---- cells ---------------------------------------------------------------------------------------------------
 local function slot_unlock_depth(slot)
  for depth=0,60 do if D.mod_progression.slots(D.mod_progression.context(depth,0))>=slot then return depth end end
 end
 -- ---- drive models (optional, local-only assets) ---------------------------------------------------------------
 -- When the model mod `envoy_drives` is mounted and this exe has gd.kit.model, a drive cell shows its colour's model in the cell's
 -- icon square (the component's icon_draw hook). Anything missing leaves the flat coloured cell: no error, no retry per frame.
 local MESH={red='drive_red',green='drive_green',yellow='drive_yellow',blue='drive_blue',white='drive_white',purple='drive_purple'}
 local RING={magic='drive_ring_magic',rare='drive_ring_rare',unique='drive_ring_unique'}   -- rarity overlays: a second model, same pose, drawn first
 local TINT={}   -- the shapes carry their own colours
 function S:load_models()
  self.models=nil;self.descs={};self.model_failed=nil
  local g=self.g
  if not (g.kit and type(g.kit.model)=='function' and type(g.model_load)=='function') then return end
  local mod=D.drive_models and D.drive_models.MOD or 'envoy_drives';local h,any={},false
  for _,name in pairs(MESH) do if not h[name] then
   local ok,r,why=pcall(g.model_load,mod..'/models/'..name);if ok and r then h[name]=r;any=true elseif not self.load_logged then self.load_logged=true;if self.host and self.host.log then self.host:log('drive models: model_load refused ('..tostring(ok and why or r)..'): flat cells for this screen') end end
  end end
  for _,name in pairs(RING) do if not h[name] then local ok,r=pcall(g.model_load,mod..'/models/'..name);if ok and r then h[name]=r end end end
  if any then self.models=h end
 end
 function S:release_models()
  if self.models and self.g.model_release then
   local seen={};for _,hd in pairs(self.models) do if not seen[hd] then seen[hd]=true;pcall(self.g.model_release,hd) end end
  end
  self.models=nil;self.descs={}
 end
 function S:model_desc(colour,rarity)
  if not self.models then return nil end
  local key=colour..'/'..tostring(rarity);local d=self.descs[key]
  if d==nil then
   local hd=self.models[MESH[colour] or 'drive_white']
   d=hd and {kind='model',asset=hd,tint=TINT[colour],ring=self.models[RING[tostring(rarity):lower()] or '']} or false;self.descs[key]=d
  end
  return d or nil
 end
 function S:drive_cell(r,ref,flags)
  local h=self.host;local loot=self:drives().loot;local c=T().cell(loot,r,flags)
  local fl={};if c.new then fl[#fl+1]='new' end;if c.can_merge then fl[#fl+1]='merge' end
  ref.record=r
  return {colour=c.colour_rgba,rarity=c.rarity,pips=c.affixes,flags=fl,icon=self:model_desc(r.colour,r.rarity) or (r.unique and 'crown' or nil),name=T().short(loot,r),ref=ref,actions={}}
 end
 function S:key_cell(id,kind)
  local h=self.host;local rule=h:keystone_rule(id);local fam=D.keystones.family(id)
  return {colour=T().base_colour[fam] or 'gold',rarity='unique',pips=0,icon={kind='letter',letter=(rule and rule.label or id):sub(1,1):upper(),colour=T().base_colour[fam] or 0xEBD175FF},
   name=rule and rule.label or id,ref={kind=kind,id=id},actions={}}
 end
 local function empty_cell(ref,text) return {empty=true,name='Empty slot',lines={text},ref=ref,actions={}} end
 -- A one-word label for what A does with a plan.
 local plan_label={merge='Merge',equip='Equip',bag='Take',choose='Replace which?'}
 -- ---- the blocks ----------------------------------------------------------------------------------------------
 function S:eq_cells(kind)
  local h=self.host;local b=h:bag();local d=self:drives();local cells={}
  for i=1,6 do
   local r=b.equipped[i]
   if i>b:slots() then cells[i]={colour='grey',flags={'locked'},name='Locked slot',lines={'Slot '..i..' unlocks at depth '..tostring(slot_unlock_depth(i))..'.'},ref={kind='locked',index=i},actions={}}
   elseif r then cells[i]=self:drive_cell(r,{kind=kind,where='equipped',index=i},{new=h:is_new(r)})
   else cells[i]=empty_cell({kind=kind,where='equipped',index=i,empty=true},'Empty. Pick a bag drive and press A to equip it here.') end
  end
  return cells
 end
 function S:bag_cells(kind)
  local h=self.host;local b=h:bag();local cells={}
  for i=1,b:capacity() do
   local r=b.items[i]
   if r then cells[i]=self:drive_cell(r,{kind=kind,where='bag',index=i},{new=h:is_new(r)})
   else cells[i]=empty_cell({kind=kind,where='bag',index=i,empty=true},'Empty bag place.') end
  end
  return cells
 end
 function S:build_main()
  local h=self.host;local b=h:bag();local blocks={};local room=#b.items<b:capacity()
  if self.mode=='reward' and #h.offers>0 then
   local cells={}
   for i,r in ipairs(h.offers) do
    local c=self:drive_cell(r,{kind='offer',index=i},{new=true});local plan=h:plan_take(r)
    c.actions={A=plan_label[plan.action],X=(room and plan.action~='bag') and 'To bag' or false}
    cells[i]=c
   end
   blocks[#blocks+1]={id='offer',title='TAKE ONE',cols=#h.offers,rows=1,band=1,cells=cells}
  end
  if self.mode=='reward' and #h.key_offers>0 then
   local cells={}
   for i,id in ipairs(h.key_offers) do local c=self:key_cell(id,'koffer');c.actions={A='Take',X=false,Y=false};cells[i]=c end
   blocks[#blocks+1]={id='koffer',title='KEYSTONE: ONE',cols=#h.key_offers,rows=1,band=1,cells=cells}
  end
  local eq=self:eq_cells('eq')
  for i,c in ipairs(eq) do if c.ref.where and not c.empty then c.actions={A=room and 'To bag' or false,X=room and 'To bag' or false} end end
  blocks[#blocks+1]={id='eq',title=('EQUIPPED %d/%d'):format(h:equipped_count(),b:slots()),cols=6,rows=1,band=2,cells=eq}
  local bag=self:bag_cells('bag')
  for i,c in ipairs(bag) do
   if c.ref.record then
    local plan=h:plan_take(c.ref.record,{where='bag',index=i})
    c.actions={A=plan_label[plan.action],X=false,Y='Discard'}
   end
  end
  blocks[#blocks+1]={id='bag',title=('BAG %d/%d'):format(#b.items,b:capacity()),cols=b:capacity(),rows=1,band=3,cells=bag}
  local ids=h:keystone_ids()
  if #ids>0 then
   local cells={};for i,id in ipairs(ids) do local c=self:key_cell(id,'key');cells[i]=c end
   blocks[#blocks+1]={id='key',title=('KEYSTONES %d/%d'):format(#ids,D.mod_progression.allowance(b.context)),cols=6,rows=math.ceil(#ids/6),band=4,cells=cells}
  end
  return blocks
 end
 function S:build_swap()
  local h=self.host;local b=h:bag();local sw=self.swap;local blocks={}
  local incoming=self:drive_cell(sw.record,{kind='swap_in'},{new=sw.from~='bag'})
  incoming.actions={}
  blocks[#blocks+1]={id='in',title=sw.from=='bag' and 'MOVING IN' or 'NEW DRIVE',cols=1,rows=1,band=1,focusable=false,cells={incoming}}
  local eq=self:eq_cells('swap_eq')
  for i,c in ipairs(eq) do
   if c.ref.where then c.actions={A=sw.from=='bag' and 'Swap in' or 'Replace',B=sw.from=='decide' and 'Leave it' or 'Back'} end
  end
  blocks[#blocks+1]={id='eq',title='GIVE UP WHICH?',cols=6,rows=1,band=2,cells=eq}
  if sw.from~='bag' then
   local bag=self:bag_cells('swap_bag')
   for i,c in ipairs(bag) do if c.ref.record then c.actions={A='Replace',B=sw.from=='decide' and 'Leave it' or 'Back'} end end
   blocks[#blocks+1]={id='bag',title=('OR A BAG DRIVE %d/%d'):format(#b.items,b:capacity()),cols=b:capacity(),rows=1,band=3,cells=bag}
  end
  return blocks
 end
 -- ---- detail text (for the focused cell only) -----------------------------------------------------------------
 function S:totals_before()
  local d=self:drives();if self.before_rev~=d.rev or not self.before then self.before=self.host:totals();self.before_rev=d.rev end
  return self.before
 end
 -- Any number only when it changes; strength alone says (no change) when nothing does.
 function S:compare(lines,edit)
  local h=self.host;local tx=T();local before=self:totals_before();local after=h:totals(edit)
  lines[#lines+1]=''
  -- Only the numbers that change are shown, strength included. "(no change)" appears alone, when nothing at all changes: beside
  -- a changed Speed line it read as a contradiction ("Build strength +358% (no change)" over "Speed x1.08 -> x1.16").
  local changed=0
  for _,e in ipairs(tx.total_rows) do
   if math.abs(before[e[1]]-after[e[1]])>=.005 then changed=changed+1;lines[#lines+1]=tx.total_line(e[1],e[2],before,after) end
  end
  if changed==0 then lines[#lines+1]=tx.total_line('strength','Build strength',before,after) end
 end
 local function plan_edit(plan,r,from)
  return function(d)
   if plan.action=='merge' then d:replace(plan.loc.where,plan.loc.index,plan.merged);if from then d:discard(from.index) end
   elseif plan.action=='equip' then if from then d:equip(from.index,plan.slot) else d:place(plan.slot,r) end
   elseif plan.action=='bag' then d:give(r) end
  end
 end
 function S:plan_line(plan)
  local h=self.host
  if plan.action=='merge' then local name,line=h:merge_text(plan);return 'Merges into '..name..': '..line..'.'
  elseif plan.action=='equip' then return 'Goes into slot '..plan.slot..'.'
  elseif plan.action=='bag' then return 'Goes into your bag.' end
  return 'Your bag is full: you will pick a drive to give up.'
 end
 function S:detail_lines_base(cell)
  local h=self.host;local d=self:drives();local loot=d.loot;local tx=T();local ref=cell.ref;local lines={}
  -- the long name lives in the detail panel (wrapped), the cell and the header carry the short one
  local function drive_lines(r) local full,short=loot:name(r),tx.short(loot,r);lines[#lines+1]=tx.rarity_label[r.rarity]..' drive'..(full~=short and (': '..full) or ''); for _,l in ipairs(tx.drive_lines(loot,r)) do lines[#lines+1]=l end end
  if ref.kind=='offer' then
   local r=ref.record;drive_lines(r);local plan=h:plan_take(r);lines[#lines+1]='';lines[#lines+1]=self:plan_line(plan)
   if plan.action~='choose' then self:compare(lines,plan_edit(plan,r)) end
  elseif ref.kind=='bag' and ref.record then
   local r=ref.record;drive_lines(r);local plan=h:plan_take(r,{where='bag',index=ref.index});lines[#lines+1]='';lines[#lines+1]=self:plan_line(plan)
   if plan.action=='choose' then lines[#lines]='No free slot: A asks which equipped drive to swap with.' else self:compare(lines,plan_edit(plan,r,{where='bag',index=ref.index})) end
  elseif ref.kind=='eq' and ref.record then
   drive_lines(ref.record);lines[#lines+1]='';lines[#lines+1]=('Slot %d.'):format(ref.index)
   local b=T().total_line('strength','Build strength',self:totals_before(),nil);lines[#lines+1]=b
  elseif ref.kind=='swap_eq' or ref.kind=='swap_bag' then
   local sw=self.swap;local old=ref.record
   if old then
    lines[#lines+1]='Gives up:';drive_lines(old);lines[#lines+1]=''
    if sw.from=='bag' then lines[#lines+1]=loot:name(old)..' goes to your bag.'
    elseif ref.where=='equipped' and #h:bag().items<h:bag():capacity() then lines[#lines+1]='It goes to your bag.'
    else lines[#lines+1]='It is gone for good.' end
    self:compare(lines,function(dr) if sw.from=='bag' then dr:equip(sw.index,ref.index) else dr:replace(ref.where,ref.index,sw.record) end end)
   else lines[#lines+1]='Empty.' end
  elseif ref.kind=='swap_in' then drive_lines(ref.record)
  elseif ref.kind=='key' or ref.kind=='koffer' then
   local rule=h:keystone_rule(ref.id);local fam=D.keystones.family(ref.id)
   if rule then for _,l in ipairs(tx.keystone_lines(rule,D.mod_progression.tier(h:bag().context))) do lines[#lines+1]=l end end
   lines[#lines+1]='';lines[#lines+1]=(D.keystones.family_names[fam] or 'Wild')..' keystone.'
   lines[#lines+1]=ref.kind=='key' and 'You keep it for the whole run.' or 'Pick one. If you skip, it stays owed.'
  elseif cell.lines then return cell.lines end
  return lines
 end
 -- The synergy lines (what a connection does, what an offer would complete) go after the description, before the comparison numbers.
 function S:detail_lines(cell)
  local lines=self:detail_lines_base(cell)
  local sf=self.host.synfx
  if sf and self.layout=='main' and cell.ref and cell.ref.kind~='locked' and not cell.empty then
   local ok,extra=pcall(sf.detail_lines,sf,self,cell)
   if ok and #extra>0 then
    local out,at={},nil;for i,l in ipairs(lines) do if l=='' and not at and i>1 then at=i end end
    for i,l in ipairs(lines) do if i==at then out[#out+1]='';for _,x in ipairs(extra) do out[#out+1]=x end end;out[#out+1]=l end
    if not at then out[#out+1]='';for _,x in ipairs(extra) do out[#out+1]=x end end
    return out
   end
  end
  return lines
 end
 -- ---- model ---------------------------------------------------------------------------------------------------
 function S:invalidate() self.dirty=true end
 local function signature(self)
  local h=self.host;local d=self:drives()
  return table.concat({d.rev or 0,#h.offers,#h.key_offers,#h.decide,self.layout,self.swap and self.swap.from or '',self.swap and self.swap.index or '',self.mode},'|')
 end
 -- Rebuild the blocks when the bag, the offers or the layout changed. The component keeps the focus on the same block/index.
 function S:refresh()
  local h=self.host
  if #h.decide>0 and not (self.swap and self.swap.from=='decide') then self.layout='swap';self.swap={record=h.decide[1],from='decide'};self.confirm=nil end
  if self.layout=='swap' and self.swap and self.swap.from=='decide' and h.decide[1]~=self.swap.record then
   if #h.decide>0 then self.swap.record=h.decide[1] else self.layout='main';self.swap=nil end
  end
  local key=signature(self)
  if not self.dirty and key==self.key and self.view then return end
  self.dirty=false;self.key=key;self.before=nil
  self.blocks=self.layout=='swap' and self.swap and self:build_swap() or self:build_main()
  if not self.view then self:new_view() end
  local old=self.view.fe
  self.view:set_blocks(self.blocks)
  self.view:set_title(self.layout=='swap' and (self.swap.from=='bag' and 'SWAP' or 'BAG FULL') or (self.mode=='reward' and ('STAGE CLEAR'..(self.host.milestone_line and ('  -  '..self.host.milestone_line) or '')) or 'YOUR DRIVES')..(self.host.seat and ('  -  PLAYER '..self.host.seat.port) or ''))
  local back=self.layout=='swap' and (self.swap.from=='decide' and 'Leave it' or 'Back') or (self.mode=='reward' and ((#h.offers>0 or #h.key_offers>0) and 'Skip' or 'Continue') or 'Close')
  self.view:set_actions({B=back})
  if self.layout=='main' and self.back_focus then self.view:set_focus(self.back_focus[1],self.back_focus[2]);self.back_focus=nil end
  self.sync_cell=nil;self:sync()
 end
 function S:new_view()
  local scr=self
  self.view=G.new{g=self.g,title='',blocks={},icon_draw=function(desc,x,y,w,h,focused,locked,cell) return scr:icon(desc,x,y,w,h,focused,locked,cell) end}
 end
 -- The icon hook. A keystone shows its initial; a model (or anything else) goes to `S.custom_icon` when one is set (an engine
 -- screen-space model draw can be adopted here without touching the screens).
 function S:icon(desc,x,y,w,h,focused,locked,cell)
  if desc.kind=='model' then
   local mo=S.model_opts;local opts={yaw=mo.yaw,pitch=mo.pitch,margin=mo.margin,spin=focused and 120 or 0,dim=locked and 0.35 or 1,tint=desc.tint}
   if desc.ring and not S.no_rings then pcall(self.g.kit.model,desc.ring,x,y,w,h,opts) end   -- rarity ring first, so it passes behind the body
   local ok,err=pcall(self.g.kit.model,desc.asset,x,y,w,h,opts)
   if not ok and not self.model_failed then   -- a stale handle or a refused draw: flat cells from now on, said once
    self.model_failed=true;self:release_models();self.dirty=true;self.host:log('drive models unavailable for the grid: using flat cells ('..tostring(err)..')')
   end
   return
  end
  if self.custom_icon and desc.kind~='letter' then return self.custom_icon(desc,x,y,w,h,focused,locked,cell) end
  local k=self.g.kit
  if desc.kind=='letter' and k then
   -- A keystone is a wedge (the keystone of an arch) in a lighter shade of its family colour, with its initial on it: no drive has this shape.
   local g=self.g;local geo=self.wedge;if not geo or geo.w~=w then geo={w=w,rows={}};self.wedge=geo
    local n=12;local top,bot,hh=w*.66,w*.42,w*.66;local y0=h*.1
    for i=0,n-1 do local f=(i+.5)/n;local ww=top+(bot-top)*f;geo.rows[#geo.rows+1]={math.floor(w/2-ww/2),math.floor(y0+hh*i/n),math.ceil(ww),math.ceil(hh/n)+1} end
   end
   local c=G.shade(desc.colour or 0xEBD175FF,1.28)
   for _,r in ipairs(geo.rows) do g.fill(x+r[1]-2,y+r[2],r[3]+4,r[4],0x0A0D12FF) end
   for _,r in ipairs(geo.rows) do g.fill(x+r[1],y+r[2],r[3],r[4],c) end
   if w>=58 then k.text(x+w/2,y+h*0.54,desc.letter,'heading','ink','center') else k.text(x+w/2,y+h*0.52,desc.letter,'body','ink','center') end
  end
 end
 -- Focus changed: fill that cell's detail text once (and the merge arrow on the cell the focused drive would merge into).
 function S:sync()
  local v=self.view;if not v then return end
  local fc=v:focused()
  if fc==self.sync_cell then return end
  self.sync_cell=fc
  if not fc then return end
  self:mark_target(fc)
  fc=v:focused()
  if fc and not fc.detail_done then fc.lines=self:detail_lines(fc);fc.detail_done=true;v.ver=v.ver+1 end
 end
 function S:mark_target(cell)
  local h=self.host;local ref=cell.ref;local loc
  if self.layout=='main' and ref and ref.record and (ref.kind=='offer' or ref.kind=='bag') then
   local plan=h:plan_take(ref.record,ref.kind=='bag' and {where='bag',index=ref.index} or nil);if plan.action=='merge' then loc=plan.loc end
  end
  local want=loc and (loc.where..':'..loc.index) or nil
  if want==self.marked then return end
  self.marked=want
  for _,b in ipairs(self.blocks) do
   if b.id=='eq' or b.id=='bag' then for _,c in pairs(b.cells) do
    local r=c.ref
    if r and r.where and not c.empty then
     local on=want~=nil and (r.where..':'..r.index)==want
     local fl,had={},false
     for _,f in ipairs(c.flags or {}) do if f=='merge' then had=true else fl[#fl+1]=f end end
     if on then fl[#fl+1]='merge' end
     if on~=had then c.flags=fl end
    end
   end end
  end
  self.view:rebuild()
 end
 -- Focus a cell by block and index (tests, the console).
 function S:focus_on(block,index) self:refresh();local ok=self.view:set_focus(block,index);self.sync_cell=nil;self:sync();return ok end
 function S:focused() self:refresh();local c=self.view:focused();return c,c and c.ref end
 -- ---- lifecycle -----------------------------------------------------------------------------------------------
 function S:open(mode)
  self.active=true;self.mode=mode;self.layout='main';self.swap=nil;self.confirm=nil;self.notice=nil;self.dirty=true;self.view=nil;self.key=nil;self.marked=nil
  self:load_models()
  self.input:set_active(true,true);self.input.previous.start=true;self.input.previous.accept=true;self.input.previous.back=true
  self.input.previous.x=true;self.input.previous.y=true
  self.deterministic=self.host and self.host.online or false -- an online run counts ticks: the reward countdown is a rule both peers share
  self.opened=self.g.time and self.g.time() or 0;self.ticks=0
  if mode=='bag' and self.g.paused and not self.g.paused() then self.g.pause();self.owns_pause=true end
  self:refresh()
  if mode=='bag' and D.atlas_bag and D.atlas_bag.enabled(self.g) then D.atlas_bag.attach(self) end
  if self.view then self.view:set_countdown(self:seconds_left(),S.tuning.safe_seconds) end
  self.host:log('screen open: '..mode)
 end
 function S:close()
  if self.atlas then D.atlas_bag.detach(self) end
  if self.active or self.input.masked then self.input:close() end
  self.preview=nil;self:release_models();self.active=false;self.swap=nil;self.confirm=nil;self.view=nil;self.layout='main'
  if self.owns_pause then self.g.resume();self.owns_pause=nil end
 end
 -- The engine's hold (gd.hold_1p) is counted in 60 Hz logic units of wall time, so it lasts hold_ticks/60 seconds at
 -- any display rate (2850 = 47.5 s). The countdown is the shorter of the allowance (safe_seconds, 45 s) and what is left
 -- of the hold less the margin, so the screen always resolves itself first.
 function S:seconds_left()
  if self.mode~='reward' then return nil end
  local wall=(not self.deterministic and self.g.time) and (self.g.time()-self.opened) or self.ticks/60 -- deterministic (online): counted ticks, never the wall clock
  local by_hold=(S.tuning.hold_ticks-S.tuning.tick_margin)/60-wall
  return math.max(0,math.min(S.tuning.safe_seconds-wall,by_hold))
 end
 -- Every way out of the reward moment goes through here with the reason, so nothing is dropped silently.
 function S:finish(reason)
  self:close();self.host:finish_reward(reason)
 end
 -- Open the swap layout, remembering where the focus was so that backing out puts it there again.
 function S:enter_swap(sw)
  local _,bid,idx=self.view:focused();self.back_focus=bid and {bid,idx} or nil
  self.layout='swap';self.swap=sw
 end
 -- ---- input ---------------------------------------------------------------------------------------------------
 function S:notify(text) self.notice=text;self.notice_until=(self.g.time and self.g.time() or 0)+4 end
 local dirs={up=true,down=true,left=true,right=true}
 -- What happened, in words, after a take.
 local function took_text(h,action,plan)
  if action=='merge' and plan then local name,line=h:merge_text(plan);return 'Merged into '..name..': '..line..'.' end
  if action=='equip' and plan then return 'Equipped in slot '..plan.slot..'.' end
  if action=='bag' then return 'In your bag.' end
  return 'Done.'
 end
 function S:accept()
  local c,ref=self:focused();local h=self.host;if not ref then return end
  if ref.kind~='skip' then self.confirm=nil end
  if self.layout=='swap' then
   if ref.kind=='swap_eq' or ref.kind=='swap_bag' then
    if ref.empty then self:notify('Pick a drive to give up.')
    else
     local act,msg=h:replace_with(self.swap,ref.where,ref.index)
     if act then self:notify(msg or 'Done.');self.layout='main';self.swap=nil else self:notify(msg or 'Not possible.') end
    end
   end
   self:invalidate();return
  end
  if ref.kind=='offer' then
   local r=ref.record;local act,plan=h:take_offer(ref.index)
   if act then self:notify(took_text(h,act,plan))
   elseif plan=='choose' then self:enter_swap({record=r,from='offer',index=ref.index})
   else self:notify(plan or 'Not possible.') end
  elseif ref.kind=='koffer' then
   local ok,msg=h:take_keystone(ref.id);self:notify(ok and ('Keystone: '..(h:keystone_rule(ref.id) or {label=ref.id}).label) or msg)
  elseif ref.kind=='bag' and ref.record then
   local act,plan=h:use_bag_drive(ref.index)
   if act then self:notify(took_text(h,act,plan))
   elseif plan=='choose' then self:enter_swap({record=ref.record,from='bag',index=ref.index})
   else self:notify(plan or 'Not possible.') end
  elseif ref.kind=='eq' and ref.record then local ok,msg=h:unequip(ref.index);self:notify(msg)
  elseif ref.kind=='eq' then self:notify('Pick a bag drive and press A to equip it here.')
  elseif ref.kind=='key' then self:notify('You keep keystones for the whole run.')
  end
  self:invalidate()
 end
 function S:keep()  -- X
  local c,ref=self:focused();local h=self.host;if not ref or self.layout=='swap' then return end
  if ref.kind=='offer' then
   local act,why=h:take_offer(ref.index,'bag');if act then self:notify('In your bag.') else self:notify(why) end
  elseif ref.kind=='eq' and ref.record then local ok,msg=h:unequip(ref.index);self:notify(msg)
  end
  self:invalidate()
 end
 function S:discard()  -- Y
  local c,ref=self:focused();local h=self.host
  if self.layout=='swap' or ref==nil or ref.kind~='bag' or not ref.record then return end
  if self.confirm=='discard'..ref.index then self.confirm=nil;local ok,msg=h:discard(ref.index);self:notify(msg)
  else self.confirm='discard'..ref.index;self:notify('Press Y again to discard this drive, B to cancel.') end
  self:invalidate()
 end
 function S:back()
  local h=self.host
  if self.confirm and self.confirm~='skip' and self.confirm~='leave' then self.confirm=nil;self:notify('Cancelled.');self:invalidate();return end
  if self.layout=='swap' then
   if self.swap.from=='decide' then
    if self.confirm=='leave' then self.confirm=nil;h:leave_choice('player choice');self.layout='main';self.swap=nil;self:notify('Left behind.')
    else self.confirm='leave';self:notify('Press B again to leave it behind.') end
   else self.layout='main';self.swap=nil end
   self:invalidate();return
  end
  if self.mode=='bag' then self:finish('closed');return end
  if #h.offers>0 or #h.key_offers>0 then
   if self.confirm=='skip' then self.confirm=nil;h:decline_offers('skipped');self:finish('done');return end
   self.confirm='skip';self:notify(#h.offers>0 and 'Press B again to skip the reward.' or 'Press B again to skip. The keystone stays owed.')
   self:invalidate();return
  end
  self:finish('done')
 end
 function S:press(action)
  if not self.active then return end
  if dirs[action] then
   self:refresh();if self.view:press(action) then self.confirm=nil end;self:sync()
  elseif action=='accept' then self:accept() elseif action=='back' then self:back()
  elseif action=='x' then self:keep() elseif action=='y' then self:discard()
  elseif action=='start' and self.mode=='bag' and self.layout~='swap' then self:finish('closed') end
  if self.active then self:refresh() end
 end
 function S:tick()
  if not self.active then return end
  self.ticks=self.ticks+1
  for _,a in ipairs(self.input:poll()) do self:press(a);if not self.active then return end end
  self:refresh()
  if self.notice and self.g.time and self.g.time()>self.notice_until then self.notice=nil end
  if self.mode=='reward' and not self.preview then
   local left=self:seconds_left()
   if self.view then self.view:set_countdown(left,S.tuning.safe_seconds) end
   local m=self.g.mode_1p and self.g.mode_1p()
   if left<=0 then self:finish('timeout') elseif m and m.held==false then self:finish('released') end
  end
 end
 -- ---- drawing -------------------------------------------------------------------------------------------------
 function S:draw()
  if not self.active or not self.view then return end
  local g=self.g
  self.view:draw()
  if self.host.seat then local a=g.safe_area();local c=D.run_hud.port_colour[self.host.seat.port] or 0xFFFFFFFF;g.fill(a.x,a.y,a.w,6,c);g.fill(a.x,a.y+a.h-6,a.w,6,c);g.fill(a.x,a.y,6,a.h,c);g.fill(a.x+a.w-6,a.y,6,a.h,c) end -- co-op: the screen wears its owner's port colour
  if self.host.synfx then local ok,err=pcall(self.host.synfx.draw_grid,self.host.synfx,self);if not ok and not self.synfx_failed then self.synfx_failed=true;self.host:log('synergy grid overlay failed: '..tostring(err)) end end
  local k=g.kit;local L=self.view.lay
  if self.notice and k and L then
   local h=L.head
   k.text(h.x+h.w-(self.mode=='reward' and 80 or 0),h.y+19,self.notice,'body','gold','right',{max_w=h.w-230})
  end
 end
 -- The screen as text: what a player would read (the model, no pixels).
 function S:dump()
  local out={};self:refresh()
  local fc=self.view:focused();local ref=fc and fc.ref
  out[#out+1]=('screen %s layout=%s focus=%s time_left=%s'):format(self.mode,self.layout,ref and (ref.kind..(ref.where and (':'..ref.where) or '')..':'..tostring(ref.index or ref.id or '')) or 'none',tostring(self:seconds_left() and math.floor(self:seconds_left())))
  local loot=self:drives().loot
  for _,b in ipairs(self.blocks) do
   local names={}
   for i=1,b.cols*b.rows do
    local c=b.cells[i]
    if not c then names[#names+1]='-' elseif c.empty then names[#names+1]='[empty]' elseif c.flags and (function() for _,f in ipairs(c.flags) do if f=='locked' then return true end end end)() then names[#names+1]='[locked]'
    else names[#names+1]=(c==fc and '>' or '')..c.name..(c.pips and c.pips>0 and ('*'..c.pips) or '')..((function() for _,f in ipairs(c.flags or {}) do if f=='merge' then return ' ^merge' end end;return '' end)()) end
   end
   out[#out+1]=b.title..': '..table.concat(names,' | ')
  end
  if fc then
   out[#out+1]='-- detail: '..tostring(fc.name)
   for _,l in ipairs(fc.lines or {}) do out[#out+1]='   '..l end
   local acts={};for _,b in ipairs({'A','X','Y','B'}) do local a=fc.actions and fc.actions[b];if a==nil then a=self.view.actions[b] end;if a then acts[#acts+1]=b..' '..a end end
   out[#out+1]='-- actions: '..table.concat(acts,'   ')
  end
  out[#out+1]='-- notice: '..tostring(self.notice)
  return out
 end
 return S
end
