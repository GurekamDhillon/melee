-- Three-way command tree. Pure input policy; no engine calls or pad ownership.
-- The default tree is the hand-authored prototype. C.plan()/C.loadout() build an
-- equivalent tree from the run's installed loadout and declared inventory
-- capacities only -- never a catalogue dump -- so combat never browses the
-- whole collection. Left/Right/Down fork three children at every depth, Up
-- returns one level, and a fresh Up at the root is the native taunt.
local C = {}
local directions = {{'left',1}, {'right',2}, {'down',4}}
local default_tree
local function branch(label,icon,node) return {label=label,icon=icon,node=node} end
local function leaf(label,icon,action,family,slot)
 return {label=label,icon=icon,action=action,family=family,slot=slot}
end
local tree = {
 root={title='COMMAND',branches={branch('Magic','magic','magic'),branch('Item','item','item'),branch('Special','special','special')}},
 magic={title='MAGIC',parent='root',branches={branch('Fire','cinder','fire'),branch('Ice','rime','ice'),branch('Reaction','fusion','reaction')}},
 fire={title='FIRE',parent='magic',branches={leaf('Cinder Eruption','assault','gene','cinder','assault'),leaf('Cinder Step','traversal','gene','cinder','traversal'),leaf('Cinder Counter','guard','gene','cinder','guard')}},
 ice={title='ICE',parent='magic',branches={leaf('Rime Strike','assault','gene','rime','assault'),leaf('Rime Glide','traversal','gene','rime','traversal'),leaf('Rime Response','guard','gene','rime','guard')}},
 reaction={title='REACTION',parent='magic',branches={leaf('Thermal Shock','fusion','thermal_shock')}},
 item={title='ITEM',parent='root',branches={leaf('Restore','item','restore')}},
 special={title='SPECIAL',parent='root',branches={leaf('Loadout','gene','loadout'),leaf('Route','route','route'),leaf('Collection','focus','collection')}}
}
default_tree = tree
local function tree_of(s) return (type(s)=='table' and s.tree) or default_tree end
local function held(b,bit) return b % (bit*2) >= bit end
local function buttons(b)
 assert(type(b)=='number' and b==b and b>=0 and b%1==0 and b<=65535,'invalid buttons')
 return b%16
end
local function available(b,fn)
 if b.node then return true end
 if b.blocked then return false,b.blocked end
 if not fn then return false,'Unavailable' end
 local ok,reason=fn(b)
 return ok==true,reason or (ok and nil or 'Unavailable')
end
function C.new() return {node='root',previous=0,up_latched=false,chord=false} end
function C.reset(s,b)
 local raw=buttons(b or 0)
 s.node='root';s.previous=raw;s.up_latched=held(raw,8);s.chord=false
end
function C.view(s,fn)
 local t=tree_of(s)
 local n=assert(t[s.node],'invalid command node')
 local path,p={},s.node
 while p do table.insert(path,1,t[p].title);p=t[p].parent end
 local out={id=s.node,title=n.title,depth=#path-1,path=path,branches={},up=n.parent and 'back' or 'taunt'}
 for i,d in ipairs(directions) do
  local b=n.branches[i]
  if b then
   local v={} for k,x in pairs(b) do v[k]=x end
   v.direction=d[1];v.enabled,v.reason=available(b,fn);out.branches[i]=v
  end
 end
 return out
end
function C.update(s,b,fn)
 local raw=buttons(b);local previous=s.previous;s.previous=raw
 if not held(raw,8) then s.up_latched=false end
 local t=tree_of(s)
 local n=assert(t[s.node],'invalid command node')
 local count=0;for _,bit in ipairs({1,2,4,8}) do if held(raw,bit) then count=count+1 end end
 -- A diagonal/opposing chord performs no ambiguous action and must return to
 -- neutral before another selection. If Up occurs below root, consume it.
 if count>1 then
  s.chord=true;if n.parent and held(raw,8) then s.up_latched=true end
 end
 local e
 if raw==0 then s.chord=false
 elseif not s.chord and count==1 and previous==0 then
  if held(raw,8) then
   if n.parent then s.node=n.parent;s.up_latched=true;e={kind='back',node=s.node}
   elseif not s.up_latched then e={kind='taunt'} end
  else
   for i,d in ipairs(directions) do if held(raw,d[2]) then
    local b=n.branches[i]
    if b then
     local ok,reason=available(b,fn)
     if not ok then e={kind='blocked',reason=reason,action=b.action,family=b.family,slot=b.slot}
     elseif b.node then s.node=b.node;e={kind='navigate',node=s.node}
     else
      e={kind='execute'};for k,v in pairs(b) do e[k]=v end
      s.node='root'
     end
    end
   end end
  end
 end
 -- Left/right/down belong to the menu. Up stays masked until RELEASE even
 -- after popping to root, so the game cannot turn that same hold into taunt.
 local mask=7
 if t[s.node].parent or s.up_latched then mask=15 end
 return e,mask
end
-- Loadout-derived planning ---------------------------------------------------
-- C.plan(run, opts) returns a fresh tree whose Magic/abilities fork exposes only
-- the genes the run has actually placed, and whose Item fork reflects the
-- declared consumable supply. It never enumerates the gene catalogue.
--
-- opts fields (all optional):
--   order      stable slot order, default {'assault','guard','traversal'}
--   labels     {slot=..., [kind]=...} readable, stable family/placement labels
--   capacities {items={restore=n},supplies=n} declared inventory only
--   assign     {root={left=...,right=...,down=...},abilities={...}} prepared
--              branch assignments made between rooms
local slot_names={assault='ASSAULT',guard='GUARD',traversal='TRAVERSAL'}
local dir_names={'left','right','down'}
local function clean(s,n)
 if type(s)~='string' then return nil end
 s=s:sub(1,n);if #s==0 then return nil end;return s
end
local function kind_label(kind,opts)
 local l=opts.labels and opts.labels[kind]
 return clean(l,24) or ((kind or 'EMPTY'):upper())
end
local function slot_label(slot,opts)
 local l=opts.labels and opts.labels[slot]
 return clean(l,12) or slot_names[slot] or slot:upper()
end
-- Reconcile a prepared branch assignment into a permutation of `allowed`.
-- A partial assignment (e.g. only `left`) must not duplicate a node and drop
-- another: unassigned directions are filled from the remaining allowed values in
-- their stable order, so every installed action keeps exactly one path. Unknown
-- or repeated values, or unknown direction keys, are refused.
local function assignment(value,allowed)
 assert(type(allowed)=='table' and #allowed==3,'assignment needs three allowed values')
 local function allowed_index(v) for i,x in ipairs(allowed) do if x==v then return i end end end
 if value==nil then local out={} for i=1,3 do out[i]=allowed[i] end return out end
 if type(value)~='table' then return nil,'invalid assignment' end
 for k in pairs(value) do
  local known=false;for _,dir in ipairs(dir_names) do if k==dir then known=true end end
  if not known then return nil,'unknown assignment direction' end
 end
 local out,used={},{}
 for i,dir in ipairs(dir_names) do
  local v=value[dir]
  if v~=nil then
   local idx=allowed_index(v)
   if not idx then return nil,'unknown '..dir..' assignment' end
   if used[v] then return nil,'repeated '..dir..' assignment' end
   out[i]=v;used[v]=true
  end
 end
 for i=1,3 do
  if out[i]==nil then
   for _,v in ipairs(allowed) do if not used[v] then out[i]=v;used[v]=true;break end end
  end
 end
 return out
end
-- Build a leaf for an equipped ability (or an honest empty placeholder).
local function ability_leaf(slot,run,opts)
 local id=run and run.hosts and run.hosts.player and run.hosts.player.slots[slot]
 local gene=id and run.genes and run.genes[id]
 if not gene then
  local l=leaf('EMPTY '..slot_label(slot,opts),'guard','gene',nil,slot)
  l.blocked='No gene placed in '..slot;return l
 end
 return leaf(kind_label(gene.kind,opts)..' / '..slot_label(slot,opts),slot,
  'gene',gene.kind,slot)
end
function C.plan(run,opts)
 opts=opts or {}
 assert(type(run)=='table' and type(run.genes)=='table','plan needs a run')
 local order=opts.order or {'assault','guard','traversal'}
 assert(type(order)=='table' and #order==3,'plan order must permute exactly the three slots')
 local seen={}
 for _,slot in ipairs(order) do assert(slot_names[slot] and not seen[slot],'unknown or repeated slot '..tostring(slot));seen[slot]=true end
 -- The abilities fork shows one child per prepared direction. A partial
 -- assignment is reconciled into a permutation so no installed slot is dropped.
 local ab,awhy=assignment(opts.assign and opts.assign.abilities,order)
 assert(ab,awhy)
 local ra,rwhy=assignment(opts.assign and opts.assign.root,{'abilities','items','special'})
 assert(ra,rwhy)
 local t={}
 -- Root: three forks only.
 t.root={title='COMMAND',branches={}}
 local root_spec={abilities={node='abilities',title='ABILITIES',icon='magic',label='Abilities'},
  items={node='item',title='ITEM',icon='item',label='Item'},
  special={node='special',title='SPECIAL',icon='focus',label='Special'}}
 for i,key in ipairs(ra) do
  local spec=root_spec[key]
  t.root.branches[i]={label=spec.label,icon=spec.icon,node=spec.node,title=spec.title}
 end
 -- Abilities fork: one ability per direction, recursive grammar unchanged.
 t.abilities={title='ABILITIES',parent='root',branches={}}
 for i=1,3 do t.abilities.branches[i]=ability_leaf(ab[i],run,opts) end
 -- Items fork: declared supply only, or an honest blocked entry. A declared
 -- zero availability must never render as an enabled "Restore x0".
 local count
 if type(opts.capacities)=='table' then
  if type(opts.capacities.items)=='table' and opts.capacities.items.restore~=nil then count=opts.capacities.items.restore
  elseif opts.capacities.supplies~=nil then count=opts.capacities.supplies end
 end
 if count~=nil then
  assert(type(count)=='number' and count==count and count%1==0 and count>=0,'invalid supply count')
 end
 t.item={title='ITEM',parent='root',branches={}}
 if count~=nil and count<=0 then
  t.item.branches[1]=leaf('No supplies','item','restore')
  t.item.branches[1].blocked='No supplies in this run'
 else
  t.item.branches[1]=leaf(count and ('Restore x'..tostring(count)) or 'Restore','item','restore')
 end
 -- Special fork: between-room surfaces. Route opens the discovered map.
 t.special={title='SPECIAL',parent='root',branches={
  leaf('Loadout','gene','loadout'),leaf('Route','route','route'),leaf('Collection','focus','collection')}}
 return t
end
-- Convenience: build and install in one call. Returns the installed tree.
function C.loadout(s,run,opts,b)
 local t=C.plan(run,opts);C.install(s,t,b);return t
end
-- Install a planned tree. Passing nil restores the authored default tree.
-- The cursor returns to the valid root, but the held-button edge/latch is
-- preserved. Rebuilding while Up is held therefore cannot fabricate a fresh
-- root taunt: the Up stays masked until it is actually released. Pass the
-- current button word as `b` when the caller has it to sync exactly; C.reset is
-- the explicit way to clear held-button history.
function C.install(s,t,b)
 assert(type(s)=='table','install needs command state')
 if t~=nil then
  assert(type(t)=='table' and t.root,'planned tree needs a root')
  assert(t.root.branches and #t.root.branches<=3,'root may fork at most three ways')
  for _,b2 in ipairs(t.root.branches) do assert(b2.node and t[b2.node],'dangling root branch') end
 end
 s.tree=t
 s.node='root'
 if b~=nil then local raw=buttons(b);s.previous=raw;s.up_latched=held(raw,8) end
 s.chord=false
 return C.view(s)
end
-- Prepared branch assignments are data, not a second grammar. A between-room
-- screen can offer these directions for the next battle and read them back.
function C.assignments(t)
 t=t or default_tree
 local out={root={},abilities={}}
 for i,dir in ipairs(dir_names) do
  local b=t.root.branches[i]
  if b then out.root[dir]=(b.node=='abilities' and 'abilities') or (b.node=='item' and 'items') or b.node end
  local a=t.abilities and t.abilities.branches and t.abilities.branches[i]
  if a then out.abilities[dir]=a.slot end
 end
 return out
end
return C
