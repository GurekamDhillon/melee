-- Three-way command tree. Pure input policy; no engine calls or pad ownership.
local C = {}
local directions = {{'left',1}, {'right',2}, {'down',4}}
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
local function held(b,bit) return b % (bit*2) >= bit end
local function buttons(b)
 assert(type(b)=='number' and b==b and b>=0 and b%1==0 and b<=65535,'invalid buttons')
 return b%16
end
local function available(b,fn)
 if b.node then return true end
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
 local n=assert(tree[s.node],'invalid command node')
 local path,p={},s.node
 while p do table.insert(path,1,tree[p].title);p=tree[p].parent end
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
 local n=assert(tree[s.node],'invalid command node')
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
     if not ok then e={kind='blocked',reason=reason,action=b.action}
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
 if tree[s.node].parent or s.up_latched then mask=15 end
 return e,mask
end
return C
