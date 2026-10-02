-- In-engine inherited library / run workbench. Original abstract body drawing.
-- Bundled after Core. Views and previews are read-only; main owns saves/lifecycle.
local M={}
local C=Core
local slots={'assault','guard','traversal'}
local traits={'potency','capacity','gain','reach','cooldown'}
local labels={potency='POWER',capacity='CAPACITY',gain='GAIN / EVENT',reach='REACH',cooldown='RECOVERY / F'}
local regions={assault='Hands / striking pieces',guard='Torso / defense pieces',traversal='Feet / movement pieces'}
local triggers={direct_hit='Clean fighter hits',move='Ground movement',defend='Native shieldstun'}
local function keys(t) local a={} for k in pairs(t or {}) do a[#a+1]=k end table.sort(a,function(x,y) local a,b=tonumber(x:match('%d+$')),tonumber(y:match('%d+$'));return a and b and a~=b and a<b or ((not a or not b or a==b) and x<y) end);return a end
local function clone(v) if type(v)~='table' then return v end local out={} for k,x in pairs(v) do out[k]=clone(x) end return out end
local function copy(v) return assert(C.restore(assert(C.snapshot(v)))) end
local function num(n) return string.format('%.2f',n):gsub('0+$',''):gsub('%.$','') end
local function used(r,id)
 for host,h in pairs(r.hosts) do for slot,x in pairs(h.slots) do if x==id then return host,slot end end end
end
local function inventory(ctx)
 local source=ctx.menu=='collection' and ctx.profile or ctx.run
 local ids={}
 for _,id in ipairs(keys(source and source.genes)) do
  local host=ctx.menu~='collection' and used(ctx.run,id)
  if not host or host=='player' then ids[#ids+1]=id end
 end
 return ids,source
end
local function resolve(r,id)
 local host,slot=used(r,id)
 if host then return C.resolve(r,host,slot),slot,C.ability(r,host,slot) end
 local preview=copy(r);assert(C.equip(preview,'preview','assault',id))
 return C.resolve(preview,'preview','assault'),'assault',C.ability(preview,'preview','assault')
end
local function detail(ctx,id)
 local _,source=inventory(ctx);local g=source and source.genes[id]
 if not g then return nil end
 local p,slot,a
 if source.type=='run' then p,slot,a=resolve(source,id)
 else p=clone(g.base);slot='assault';local d=C.definitions[g.kind];a=clone(d.variants.assault);a.family=d.family end
 local host,placed
 if source.type=='run' then host,placed=used(source,id) end
 return {id=id,gene=g,name=C.definitions[g.kind].name,kind=g.kind,stats=p,slot=slot,ability=a,placed=placed,host=host}
end
local function move(r,id,to)
 local host,from=used(r,id)
 if host and host~='player' then return nil,'Enemy-owned gene' end
 local h=r.hosts.player
 if h and h.slots[to] and h.slots[to]~=id then return nil,'Destination occupied; unequip it first' end
 if from and from~=to then assert(C.equip(r,'player',from,nil)) end
 return C.equip(r,'player',to,id)
end
local function fused(r,a,b)
 local result=copy(r);local placement
 for _,slot in ipairs(slots) do local id=result.hosts.player.slots[slot]
  if id==a or id==b then placement=placement or slot;assert(C.equip(result,'player',slot,nil)) end
 end
 local id,why=C.fuse(result,a,b)
 if not id then return nil,why end
 local to=placement or 'assault'
 if not result.hosts.player.slots[to] then assert(C.equip(result,'player',to,id)) end
 return result,id
end
local rewards={
 {title='FOCUSED POWER',desc='A stronger release.',changes={{'potency',2}}},
 {title='HEAVY RELEASE',desc='More power; slower charge gain.',changes={{'potency',4},{'gain',-.25}}},
 {title='QUICK RHYTHM',desc='Build charge faster.',changes={{'gain',.5}}},
 {title='NEW RIME',desc='Add an unplaced frost individual.',kind='rime'}
}
function M.new() return {focus=nil,selected=nil,page=1,section='main',pending=nil,gen=0} end
-- Reset invalidates any half-finished destructive confirmation: its bound
-- identity and source generation are no longer live.
function M.reset(s,which) s.menu=which;s.focus=nil;s.selected=nil;s.page=1;s.section='main';s.parent=nil;s.pending=nil;s.gen=(s.gen or 0)+1 end
local function sync(s,ctx)
 if s.menu~=ctx.menu then M.reset(s,ctx.menu) end
 local ids=inventory(ctx)
 local exists=false;for _,id in ipairs(ids) do if id==s.selected then exists=true end end
 if not exists then s.selected=ctx.menu=='collection' and ctx.profile and ctx.profile.genes[ctx.starter] and ctx.starter or ids[1] end
 -- Only the collection/rest/reward lists page over gene inventory; the map owns
 -- its own pagination and must not be clamped back to page 1 on every view.
 if ctx.menu~='map' then s.page=math.max(1,math.min(s.page,math.max(1,math.ceil(#ids/6)))) end
end
local function control(v,id,x,y,w,h,label,action,enabled,reason)
 local b={id=id,x=x,y=y,w=w,h=h,label=label,action=action,enabled=enabled~=false,reason=reason}
 v.controls[#v.controls+1]=b;return b
end
local function delta(before,after,stat) return num(before[stat])..' > '..num(after[stat]) end
-- Declared-data screens -------------------------------------------------------
-- These consume read-only context handed in by main. They never derive game
-- mechanics: an absent declaration shows an honest message, not a fabricated
-- option. Every control goes through the same focus/controller/mouse path as
-- the collection and rest screens.
local function view_map(s,v,ctx)
 v.title='DISCOVERED MAP'
 local map=ctx.map
 control(v,'map_close',484,410,130,30,'CLOSE',{kind='map_close'})
 if type(map)~='table' then v.error='No map available yet.';return v end
 v.map=map;v.map_current=map.current;v.map_counts=map.counts or {}
 v.map_exits=map.exits or {};v.map_locks=map.locks or {}
 v.map_rooms={}
 for id in pairs(map.rooms or {}) do v.map_rooms[#v.map_rooms+1]=id end
 table.sort(v.map_rooms,function(a,b)
  local ra,rb=map.rooms[a],map.rooms[b];local da,db=(ra and ra.depth) or 0,(rb and rb.depth) or 0
  if da~=db then return da<db end;return a<b end)
 -- Only revealed/visited rooms exist in the map record, so nothing hidden can
 -- leak through a label, title or id. Real runs hold ~12-18 rooms, so the list
 -- paginates rather than drawing past the safe area.
 local per=9
 v.map_pages=math.max(1,math.ceil(#v.map_rooms/per))
 s.page=math.max(1,math.min(s.page,v.map_pages));v.page=s.page;v.pages=v.map_pages
 for j=(s.page-1)*per+1,math.min(s.page*per,#v.map_rooms) do
  local id=v.map_rooms[j];local r=map.rooms[id]
  local label=(r.current and '> ' or r.visited and '* ' or '')..(r.title or id)
  control(v,'map_room:'..id,26,110+(j-(s.page-1)*per-1)*30,300,27,label,{kind='map_inspect',room=id})
 end
 if v.map_pages>1 then
  control(v,'previous',26,384,80,24,'< PAGE',{kind='page',delta=-1},s.page>1)
  control(v,'next',114,384,80,24,'PAGE >',{kind='page',delta=1},s.page<v.map_pages)
 end
 if s.map_selected and map.rooms[s.map_selected] then v.map_selected=s.map_selected
 elseif map.rooms[map.current] then v.map_selected=map.current
 elseif #v.map_rooms>0 then v.map_selected=v.map_rooms[1] end
 return v
end
local function view_onboarding(v,ctx)
 v.title='TUTORIAL'
 local o=ctx.onboarding
 control(v,'onboarding_close',484,410,130,30,'CLOSE',{kind='onboarding_close'})
 control(v,'onboarding_skip',347,410,130,30,o and o.skipped and 'ENABLE' or 'SKIP',{kind='onboarding_skip'})
 if type(o)~='table' then v.error='Tutorial state unavailable.';return v end
 v.onboarding=o
 -- Onboarding must never pause combat; assert the contract in the view itself.
 v.no_pause=o.pause==false
 if o.skipped or not o.step then v.scope='Tutorial complete.'
 else v.scope=o.text;v.detail=o.hint;v.step_index=o.index;v.step_total=o.total end
 return v
end
local function view_settings(v,ctx)
 v.title='SETTINGS'
 local cfg=ctx.settings
 control(v,'settings_close',484,410,130,30,'CLOSE',{kind='settings_close'})
 if type(cfg)~='table' or type(cfg.items)~='table' then
  v.error='No settings are declared in this build.';return v
 end
 v.settings=cfg.items
 for i,item in ipairs(cfg.items) do
  if type(item)=='table' and type(item.id)=='string' and type(item.label)=='string' then
   local value=item.value~=nil and (' / '..tostring(item.value)) or ''
   control(v,'setting:'..item.id,26,110+(i-1)*34,588,30,item.label..value,
    {kind='setting',id=item.id,value=item.value})
  end
 end
 return v
end
local function view_ending(v,ctx)
 local e=ctx.ending
 v.title=type(e)=='table' and (e.outcome=='success' and 'RUN COMPLETE' or 'RUN ENDED') or 'RESULT'
 if type(e)~='table' then v.error='No run result to show.';return v end
 v.ending=e;v.lines=e.lines
 control(v,'ending_continue',210,410,310,30,e.next or 'CONTINUE',{kind=e.next_kind or 'ending_continue'})
 return v
end
local function view_extra(s,v,ctx)
 if ctx.menu=='map' then return view_map(s,v,ctx) end
 if ctx.menu=='onboarding' then return view_onboarding(v,ctx) end
 if ctx.menu=='settings' then return view_settings(v,ctx) end
 if ctx.menu=='ending' then return view_ending(v,ctx) end
 return nil
end
function M.view(s,ctx)
 sync(s,ctx)
 local ids,source=inventory(ctx)
 local v={controls={},menu=ctx.menu,section=s.section,ids=ids,source=source,selected=detail(ctx,s.selected),page=s.page,pages=math.max(1,math.ceil(#ids/6)),starter=ctx.starter}
 local extra=view_extra(s,v,ctx);if extra then return extra end
 if ctx.menu~='collection' and ctx.menu~='error' and (not ctx.run or ctx.run.status~='active') then v.title='RUN UNAVAILABLE';v.error='There is no active run to modify.';return v end
 if ctx.menu=='error' then v.title='CHECKPOINT UNAVAILABLE';v.error=ctx.error or 'Saved files preserved. Repair the checkpoint to continue.';if ctx.retry then control(v,'retry',26,166,588,32,'RETRY SAVING RUN RESULT',{kind='retry_finish'}) end;return v end
 v.title=ctx.menu=='collection' and 'GENE COLLECTION' or ctx.menu=='reward' and 'ENCOUNTER REWARD' or 'REST / BUILD WORKBENCH'
 v.scope=ctx.menu=='collection' and 'INHERITED LIBRARY / KEPT BETWEEN RUNS' or 'RUN COPIES / UPGRADES END WITH THIS RUN'
 if ctx.menu=='collection' and ctx.fighter then v.scope=v.scope..' / '..tostring(type(ctx.fighter)=='table' and (ctx.fighter.name or (ctx.fighter.id..' / c'..ctx.fighter.costume)) or ctx.fighter) end
 if ctx.menu=='reward' then
  local h=ctx.run.hosts.player
  local slot=h.slots.assault and 'assault' or h.slots.traversal and 'traversal' or h.slots.guard and 'guard'
  local id=slot and h.slots[slot];v.reward_target=id and detail(ctx,id);v.rewards={}
  for i,rule in ipairs(rewards) do
   local r=copy(ctx.run);local target=id;local ok,why=true
   if rule.kind then target,why=C.acquire(r,rule.kind);ok=target~=nil
   elseif not id then ok=false;why='Place a gene before upgrading'
   else for _,change in ipairs(rule.changes) do ok,why=C.reward(r,id,change[1],change[2]);if not ok then break end end end
   local after=ok and detail({menu='rest',run=r},target)
   local card=control(v,'reward'..i,304,99+(i-1)*75,310,67,rule.title,{kind='reward',index=i,id=id},ok,why)
   card.desc=rule.desc;card.before=v.reward_target;card.after=after;card.changes=rule.changes;v.rewards[i]=card
   end
   if type(ctx.capacity)=='table' and ctx.capacity.full==true then
    v.capacity_note='Collection full / choose a replacement or discard before keeping a new gene'
   end
   return v
  end
 for j=(s.page-1)*6+1,math.min(s.page*6,#ids) do
  local id=ids[j];local g=source.genes[id];local d=C.definitions[g.kind]
  local b=control(v,'gene:'..id,26,113+(j-(s.page-1)*6-1)*35,168,30,id..' / '..(g.kind=='cinder' and 'CINDER' or 'RIME'),{kind='inspect',id=id})
  b.kind=g.kind;b.selected=id==s.selected;b.starter=id==ctx.starter
 end
 if #ids>6 then
  control(v,'previous',26,329,80,24,'< PAGE',{kind='page',delta=-1},s.page>1)
  control(v,'next',114,329,80,24,'PAGE >',{kind='page',delta=1},s.page<v.pages)
 end
 if s.section=='parents' then
  v.title=ctx.menu=='collection' and 'BREED / INHERITED CHILD' or 'FUSION / RUN CHILD'
  v.parent_a=detail(ctx,s.parent);v.parent_b=v.selected
  if s.parent and s.selected then
   if ctx.menu=='collection' then
    local p=copy(ctx.profile);local id,why=C.breed(p,s.parent,s.selected)
    if id then v.child=detail({menu='collection',profile=p},id) else v.preview_error=why end
   else
    local r,id=fused(ctx.run,s.parent,s.selected)
    if r then v.child=detail({menu='rest',run=r},id) else v.preview_error=id end
   end
  end
  control(v,'confirm',210,371,200,31,ctx.menu=='collection' and 'BREED CHILD' or 'FUSE PARENTS',{kind=ctx.menu=='collection' and 'breed' or 'fuse',a=s.parent,b=s.selected},v.child~=nil,v.preview_error)
  control(v,'back',422,371,192,31,'BACK TO BUILD',{kind='back'})
  return v
 end
 if ctx.menu=='collection' then
  local is_cinder=v.selected and v.selected.kind=='cinder'
  control(v,'starter',210,326,194,30,s.selected==ctx.starter and 'CURRENT STARTER' or 'SET AS STARTER',{kind='starter',id=s.selected},is_cinder and s.selected~=ctx.starter,is_cinder and 'Already your starter' or 'This slice starts with Cinder')
  control(v,'lock',416,326,198,30,v.selected and v.selected.gene.locks.potency and 'UNLOCK POWER' or 'LOCK INHERITED POWER',{kind='lock',id=s.selected,stat='potency'},v.selected~=nil)
  control(v,'parents',210,363,194,31,'CHOOSE BREEDING PAIR',{kind='parents'},v.selected~=nil)
  if ctx.fighters then control(v,'fighter',416,363,198,31,'CHANGE FIGHTER',{kind='choose_fighter'}) end
  local can_start=#keys(ctx.profile.finished)<512 and ctx.profile.next_run<1000000000
  control(v,'start',26,410,274,32,ctx.run and ctx.run.status=='active' and 'START FRESH RUN' or 'START A RUN',{kind='start'},can_start,'Run ledger is full')
  control(v,'resume',312,410,302,32,'RESUME SAVED RUN',{kind='resume'},ctx.run~=nil and ctx.run.status=='active','No saved active run')
  -- Declared capacity only. When the collection is full, discarding is offered
  -- as an explicit destructive action that requires a second confirmation; the
  -- actual owner change stays in main/Core.
  if type(ctx.capacity)=='table' then
   v.capacity=ctx.capacity
   v.capacity_note='Collection '..tostring(ctx.capacity.count or '?')..' / '..tostring(ctx.capacity.max or '?')
   if ctx.capacity.full==true and v.selected then
    control(v,'discard',26,363,168,30,'DISCARD SELECTED',
     {kind='discard',id=s.selected,destructive=true,
      prompt='Discard '..tostring(s.selected)..'? This cannot be undone.'})
   end
  end
 else
  v.placements={}
  for i,slot in ipairs(slots) do
   local id=ctx.run.hosts.player.slots[slot];local current=id and detail(ctx,id)
   local r=copy(ctx.run);local ok,why
   if s.selected then ok,why=move(r,s.selected,slot) else why='Select a gene first' end
   local after=ok and detail({menu='rest',run=r},s.selected)
   local b=control(v,'place:'..slot,210+(i-1)*137,326,130,30,'PLACE / '..slot:upper(),{kind='place',id=s.selected,slot=slot},ok==true,why)
   b.preview=after;b.current=current;v.placements[slot]=current
  end
  control(v,'unequip',210,364,130,31,'UNEQUIP GENE',{kind='unequip',id=s.selected},v.selected~=nil and v.selected.placed~=nil,'Selected gene is unplaced')
  control(v,'parents',347,364,130,31,'FUSION PAIR',{kind='parents'},v.selected~=nil)
  control(v,'leave',484,364,130,31,'SAVE / LEAVE',{kind='leave'})
  control(v,'continue',26,410,588,32,'CONTINUE EXPLORING',{kind='continue'})
 end
 return v
end
-- update receives fresh button edges, not held levels. No combat input is owned here.
function M.update(s,ctx,input)
 local v=M.view(s,ctx);local controls=v.controls
 if #controls==0 then return nil end
 local focus=1;for i,b in ipairs(controls) do if b.id==s.focus then focus=i end end
 local current=controls[focus]
 if input.back and s.section=='parents' then s.section='main';s.focus=nil;return nil end
 local dx=input.right and 1 or input.left and -1 or 0;local dy=input.down and 1 or input.up and -1 or 0
 if dx~=0 or dy~=0 then
  local cx,cy=current.x+current.w/2,current.y+current.h/2;local best,score
  for i,b in ipairs(controls) do if i~=focus then
   local x,y=b.x+b.w/2-cx,b.y+b.h/2-cy
   local along=dx~=0 and x*dx or y*dy;local cross=dx~=0 and math.abs(y) or math.abs(x)
   local value=along+cross*3
   if along>5 and (not score or value<score) then best,score=i,value end
  end end
  if best then focus=best else focus=dy<0 and #controls or dy>0 and 1 or focus end
 end
 local activate=input.confirm
 if input.mx and input.my then for i,b in ipairs(controls) do
  if input.mx>=b.x and input.mx<b.x+b.w and input.my>=b.y and input.my<b.y+b.h then
   if input.click then focus=i;activate=true end
  end
 end end
 current=controls[focus];s.focus=current.id
 if not activate then return nil end
 if not current.enabled then s.pending=nil;return {kind='blocked',message=current.reason or 'Unavailable'} end
 local a=current.action
 -- Destructive actions (discard, replace at full capacity) require two deliberate
 -- presses on the exact same action. The pending prompt is bound to the action
 -- identity, the menu/section context and a source generation, so resetting the
 -- menu or changing the selection invalidates it: a prompt raised for g1 can
 -- never be confirmed against g2 on the next click.
 if a.destructive then
  local sig=a.kind..'\0'..tostring(a.id)..'\0'..tostring(a.slot)..'\0'..tostring(s.menu)..'\0'..tostring(s.section)
  if not (s.pending and s.pending.sig==sig and s.pending.gen==(s.gen or 0)) then
   s.pending={sig=sig,gen=(s.gen or 0)}
   return {kind='blocked',message=a.prompt or 'Press again to confirm',confirm_pending=current.id}
  end
  s.pending=nil
 else s.pending=nil end
 if a.kind=='inspect' then s.selected=a.id;s.pending=nil;s.gen=(s.gen or 0)+1
 elseif a.kind=='page' then s.page=s.page+a.delta;s.focus=nil;s.pending=nil;s.gen=(s.gen or 0)+1
 elseif a.kind=='parents' then s.parent=s.selected;s.section='parents';s.focus=nil;s.pending=nil;s.gen=(s.gen or 0)+1
 elseif a.kind=='back' then s.section='main';s.focus=nil;s.pending=nil;s.gen=(s.gen or 0)+1
 elseif a.kind=='map_inspect' then s.map_selected=a.room;s.pending=nil;s.gen=(s.gen or 0)+1
 else return clone(a) end
end
function M.apply(ctx,a)
 local result={ok=false}
 if type(a)~='table' then return {ok=false,message='Invalid menu action'} end
 if a.kind=='place' or a.kind=='unequip' or a.kind=='fuse' or a.kind=='reward' then
  if not ctx.run or ctx.run.status~='active' then return {ok=false,message='No active run'} end
 end
 if a.kind=='starter' then
  if ctx.profile.genes[a.id] and ctx.profile.genes[a.id].kind=='cinder' then return {ok=true,starter=a.id,message='Starter '..a.id..' selected; collection originals retained'} end
  result.message='Unsupported starter';return result
 elseif a.kind=='lock' then
  local g=ctx.profile.genes[a.id];local ok,why=C.lock(ctx.profile,a.id,a.stat,g and not g.locks[a.stat] or false)
  return {ok=ok==true,message=ok and 'Inherited power lock changed for '..a.id or why}
 elseif a.kind=='breed' then
  local id,why=C.breed(ctx.profile,a.a,a.b)
  return {ok=id~=nil,id=id,message=id and ('Bred '..id..' from '..a.a..' + '..a.b..'; both parents retained') or why}
 elseif a.kind=='fuse' then
  local r,id=fused(ctx.run,a.a,a.b)
  return {ok=r~=nil,run=r,id=r and id or nil,message=r and ('Fused '..id..'; '..a.a..' + '..a.b..' consumed; base traits inherited') or id}
 elseif a.kind=='place' then
  local r=copy(ctx.run);local ok,why=move(r,a.id,a.slot)
  return {ok=ok==true,run=ok and r or nil,message=ok and (a.id..' / '..C.ability(r,'player',a.slot).name..' / '..triggers[C.ability(r,'player',a.slot).trigger]) or why}
 elseif a.kind=='unequip' then
  local host,slot=used(ctx.run,a.id)
  if host~='player' then return {ok=false,message='Selected gene is unplaced'} end
  local ok,why=C.equip(ctx.run,'player',slot,nil)
  return {ok=ok==true,message=ok and (a.id..' unplaced; its charge and recovery are retained') or why}
 elseif a.kind=='reward' then
  if ctx.node and ctx.run.progress.claimed[ctx.node.id] then return {ok=false,message='Reward already claimed'} end
  local rule=rewards[a.index];if not rule then return {ok=false,message='Invalid reward'} end
  local r=copy(ctx.run);local id=a.id;local ok,why=true
  if rule.kind then id,why=C.acquire(r,rule.kind);ok=id~=nil
  else for _,change in ipairs(rule.changes) do ok,why=C.reward(r,id,change[1],change[2]);if not ok then break end end end
  if not ok then return {ok=false,message=why} end
  if ctx.node then r.progress.claimed[ctx.node.id]=true end
  local d=detail({menu='rest',run=r},id);local message=id..' acquired / unplaced'
  if rule.changes then local before=resolve(ctx.run,id);local changes={}
   for _,change in ipairs(rule.changes) do changes[#changes+1]=labels[change[1]]..' '..delta(before,d.stats,change[1]) end
   message=table.concat(changes,' / ')
  end
  return {ok=true,run=r,id=id,reward_claimed=true,message=message}
 end
 return {ok=false,message='Lifecycle action must be handled by main'}
end
local function text(x,y,s,role,color,w)
 return gd.kit.text(x,y,tostring(s),role or 'caption',color or 'bone','left',{max_w=w or 220,shear=0})
end
local function line(x,y,w,color) gd.fill(x,y,w,1,color or 0x354664ff) end
local function panel(x,y,w,h) gd.fill(x+3,y+3,w,h,0x030712ff);gd.fill(x,y,w,h,0x101a2cff) end
local function kind_color(kind) return kind=='cinder' and 0xffa16bff or kind=='rime' and 0x75ccffff or 0x344761ff end
local function mannequin(x,y,placements)
 local function block(a,b,w,h,c) gd.fill(x+a*.65,y+b*.65,w*.65,h*.65,c) end
 -- Original rectilinear mannequin: head, torso, hands, legs and boots.
 local fire=placements.assault and kind_color(placements.assault.kind) or 0x344761ff
 local guard=placements.guard and kind_color(placements.guard.kind) or 0x344761ff
 local feet=placements.traversal and kind_color(placements.traversal.kind) or 0x344761ff
 for n=0,3 do line(x-40,y+16+n*23,108,0x22324cff) end
 block(0,0,24,24,0x7083a2ff);block(-7,31,38,49,guard)
 block(-23,35,12,34,0x536889ff);block(35,35,12,34,0x536889ff)
 block(-26,68,18,19,fire);block(33,68,18,19,fire)
 block(-5,86,13,34,0x536889ff);block(17,86,13,34,0x536889ff)
 block(-15,122,24,15,feet);block(17,122,24,15,feet)
end
local function metrics(d,x,y,w)
 if not d then text(x,y+15,'Select an individual gene','caption','muted',w);return end
 for i,stat in ipairs(traits) do
  local by=y+(i-1)*23
  text(x,by,labels[stat],'caption','muted',w-80)
  text(x+w-75,by,num(d.stats[stat]),'caption','bone',75)
 end
end
local function stat_panel(v,ctx,s)
 local d=v.selected
 panel(364,96,250,220)
 if not d then return end
 gd.kit.icon('rogue_'..d.kind,376,107,.6,d.kind=='cinder' and 'gold' or 'bone')
 text(402,122,d.name,'label','gold',199)
 text(377,145,d.id..' / '..(d.placed and d.placed:upper() or ctx.menu=='collection' and 'INHERITED BASE' or 'UNPLACED'),'caption','muted',224)
 text(377,163,d.ability.name,'caption','bone',224)
 metrics(d,377,187,218)
 local parent=d.gene.parents
 if ctx.menu=='collection' then text(377,307,parent[1] and ('Parents '..parent[1]..' + '..parent[2]) or ('Origin '..(d.gene.origin or 'founder')),'caption','muted',224)
 else text(377,307,'Base power '..num(d.gene.base.potency)..' / run '..num(d.gene.upgrades.potency or 0),'caption','muted',224) end
end
local function build_panel(v,ctx)
 panel(210,96,142,220)
 local placed={}
 if ctx.menu=='collection' then
  local starter=ctx.profile.genes[ctx.starter]
  if starter then placed.assault={kind=starter.kind,id=ctx.starter} end
  text(222,115,'STARTER PREVIEW','caption','muted',118)
 else
  placed=v.placements or {}
  if ctx.menu=='reward' then
   for _,slot in ipairs(slots) do local id=ctx.run.hosts.player.slots[slot];placed[slot]=id and detail(ctx,id) end
  end
  text(222,115,'CURRENT PLACEMENT','caption','muted',118)
 end
 mannequin(271,129,placed)
 for i,slot in ipairs(slots) do
  local d=placed[slot];local y=241+(i-1)*25
  text(221,y,slot:upper()..' / '..(d and d.id or 'EMPTY'),'caption',d and 'bone' or 'muted',121)
  text(221,y+12,slot=='assault' and 'Hands / held pieces' or slot=='guard' and 'Torso / defense' or 'Feet / movement','caption','muted',121)
 end

end
local function parent_panel(v,ctx)
 panel(210,96,404,260)
 local a,b=v.parent_a,v.parent_b
 text(223,119,(a and a.id or '?')..' + '..(b and b.id or '?'),'label','gold',378)
 text(223,142,ctx.menu=='collection' and 'Parents retained / base traits inherited' or 'Parents consumed / upgrades averaged','caption','bone',378)
 if not v.child then text(223,177,v.preview_error or 'Select the second parent from the library','caption','muted',378);return end
 text(223,169,'TRAIT','caption','muted',112);text(358,169,'PARENTS','caption','muted',106);text(477,169,'CHILD','caption','gold',117)
 for i,stat in ipairs(traits) do local y=192+(i-1)*25
  text(223,y,labels[stat],'caption','muted',128)
  text(358,y,num(a.stats[stat])..' / '..num(b.stats[stat]),'caption','bone',108)
  text(477,y,num(v.child.stats[stat])..(v.child.gene.locks[stat] and ' LOCK' or ''),'caption','gold',122)
 end
 text(223,332,'Child '..v.child.id..' / '..(v.child.placed or ctx.menu=='collection' and 'inherited' or 'unplaced')..' / base power '..num(v.child.gene.base.potency),'caption','muted',378)
end
local function focused(s,v,b) return s.focus==b.id or (not s.focus and b==v.controls[1]) end
local function draw_map(s,v)
 panel(26,96,300,300)
 text(36,108,'DISCOVERED '..tostring(v.map_counts.discovered_rooms or #v.map_rooms)..' / EXITS '..tostring(v.map_counts.known_exits or 0),'caption','muted',280)
 if (v.map_counts.undiscovered_reward_count or 0)>0 then
  text(36,126,tostring(v.map_counts.undiscovered_reward_count)..' reward(s) still hidden','caption','muted',280)
 end
 for _,b in ipairs(v.controls) do gd.kit.button(b.x,b.y,b.w,b.label,focused(s,v,b),{h=b.h}) end
 panel(336,96,278,300)
 text(348,108,'EXITS','caption','muted',254)
 local room=v.map_selected or v.map_current
 local r=room and v.map.rooms[room]
 text(348,128,r and (r.title or room) or 'No room selected','label','gold',254)
 local exits=room and v.map_exits[room] or {}
 if #exits==0 then text(348,156,'No known exits','caption','muted',254) end
 for i,e in ipairs(exits) do local y=156+(i-1)*28
  local dest=v.map.rooms[e.to]
  text(348,y,(e.side or '?')..' / '..(dest and dest.title or e.to),'caption',e.gate_open and 'bone' or 'muted',190)
  if not e.gate_open then text(536,y,'LOCKED','caption','gold',76) end
 end
end
local function draw_onboarding(s,v)
 panel(26,96,588,270)
 if v.onboarding and v.onboarding.step then
  text(40,120,'STEP '..tostring(v.onboarding.index or '?')..' / '..tostring(v.step_total or '?'),'caption','muted',560)
  text(40,146,v.scope or '','label','gold',560)
  text(40,178,v.detail or '','caption','bone',560)
  if v.onboarding.early then text(40,206,'This step is taught in the first rooms.','caption','muted',560) end
 else
  text(40,150,'Tutorial complete. Revisit any time from the pause menu.','caption','bone',560)
 end
 for _,b in ipairs(v.controls) do gd.kit.button(b.x,b.y,b.w,b.label,focused(s,v,b),{h=b.h}) end
end
local function draw_settings(s,v)
 panel(26,96,588,290)
 if v.settings then
  for i,item in ipairs(v.settings) do
   text(40,112+(i-1)*34,item.label..(item.value~=nil and (' / '..tostring(item.value)) or ''),'caption','bone',560)
  end
 else text(40,120,'No settings are declared in this build.','caption','bone',560) end
 for _,b in ipairs(v.controls) do gd.kit.button(b.x,b.y,b.w,b.label,focused(s,v,b),{h=b.h}) end
end
local function draw_ending(s,v)
 panel(26,96,588,290)
 local e=v.ending
 text(40,130,e.outcome=='success' and 'RUN COMPLETE' or 'RUN ENDED','label','gold',560)
 if type(e.lines)=='table' then for i,line_text in ipairs(e.lines) do text(40,162+(i-1)*24,line_text,'caption','bone',560) end end
 for _,b in ipairs(v.controls) do gd.kit.button(b.x,b.y,b.w,b.label,focused(s,v,b),{h=b.h}) end
end
function M.draw(s,ctx)
 local v=M.view(s,ctx)
 gd.fill(0,0,640,480,0x030712ea)
 gd.kit.panel(14,17,612,438,{piece=12})
 gd.kit.icon('rogue_'..(v.section=='parents' and 'fusion' or v.menu=='collection' and 'gene' or v.menu=='reward' and 'check' or 'guard'),27,30,.65,'gold')
 text(61,52,v.title,'label','gold',550)
 if not ctx.notice then text(27,78,v.scope or 'SAVE RECOVERY','caption','muted',587) end
 line(26,87,588,0xf0b429ff)
 if v.error then text(29,130,v.error,'caption','bone',576);for _,b in ipairs(v.controls) do gd.kit.button(b.x,b.y,b.w,b.label,true,{h=b.h}) end;return end
 if v.menu=='map' then draw_map(s,v) elseif v.menu=='onboarding' then draw_onboarding(s,v)
 elseif v.menu=='settings' then draw_settings(s,v) elseif v.menu=='ending' then draw_ending(s,v)
 elseif v.menu=='reward' then
  panel(26,99,266,292)
  local d=v.reward_target
  text(40,123,'CURRENT GENE','caption','muted',238)
  text(40,150,d and d.name or 'NO PLACED GENE','label','gold',238)
  text(40,174,d and d.id..' / '..d.slot:upper() or 'Acquire a gene instead','caption','bone',238)
  metrics(d,40,205,238)
  text(40,331,'UPGRADES APPLY TO THIS RUN','caption','muted',238)
  text(40,355,'New Rime starts unplaced.','caption','muted',238)
  for _,b in ipairs(v.rewards) do
   local active=s.focus==b.id or not s.focus and b==v.controls[1]
   gd.fill(b.x,b.y,b.w,b.h,active and 0x253d65ff or 0x19283fff)
   gd.fill(b.x,b.y,3,b.h,active and 0xf0b429ff or 0x3d557aff)
   gd.kit.icon('rogue_'..(b.after and b.after.kind or 'gene'),b.x+10,b.y+9,.42,b.enabled and 'gold' or 'muted')
   text(b.x+33,b.y+21,b.label,'caption',active and 'gold' or 'bone',b.w-43)
   text(b.x+12,b.y+40,b.desc,'caption','muted',b.w-24)
   local parts={}
   if b.changes and b.before and b.after then for _,c in ipairs(b.changes) do parts[#parts+1]=(c[1]=='potency' and 'Power ' or 'Gain ')..delta(b.before.stats,b.after.stats,c[1]) end
   elseif b.after then parts={'Power '..num(b.after.stats.potency)..' / cap '..num(b.after.stats.capacity)..' / gain '..num(b.after.stats.gain)} end
   text(b.x+12,b.y+59,b.enabled and table.concat(parts,' | ') or b.reason,'caption',active and 'gold' or 'bone',b.w-24)
  end
  text(27,421,'Pick one / exact resolved traits shown above','caption','bone',587)
 else
  panel(26,96,168,260)
  text(36,108,'INDIVIDUALS '..#v.ids..' / '..v.page..' OF '..v.pages,'caption','muted',148)
  if v.section=='parents' then parent_panel(v,ctx) else build_panel(v,ctx);stat_panel(v,ctx,s) end
  for _,b in ipairs(v.controls) do
   local focused=s.focus==b.id or not s.focus and b==v.controls[1]
   local selected=b.selected or focused
   gd.kit.button(b.x,b.y,b.w,b.label,selected,{h=b.h})
   if not b.enabled then gd.fill(b.x,b.y,b.w,b.h,0x03071265) end
   if b.selected then gd.fill(b.x,b.y,3,b.h,kind_color(b.kind)) end
   if b.starter then gd.kit.icon('rogue_check',b.x+b.w-22,b.y+8,.35,'gold') end
  end
 end
 text(27,474,'D-PAD: FOCUS   A: SELECT   B: BACK   /   MOUSE: CLICK','caption','muted',587)
 local active
 for _,b in ipairs(v.controls) do if b.id==s.focus then active=b end end
 if active and not active.enabled and active.reason then
  text(27,401,active.reason,'caption','gold',587)
 elseif ctx.menu=='rest' and v.section~='parents' then
  local preview=active and active.preview
  local caption=preview and (preview.ability.name..' / '..triggers[preview.ability.trigger]..' / '..regions[preview.slot]) or 'Body + held pieces share a slot; extra meshes add no charge capacity.'
  text(27,401,caption,'caption',preview and 'gold' or 'muted',587)
 elseif ctx.menu=='collection' and v.section~='parents' then
  text(27,401,'Run copies preserve originals. Failure loses run changes.','caption','muted',587)
 end
end
M.regions=regions
M.traits=traits
return M
