-- Original kit HUD + explicitly bounded model bindings for the first slice.
-- Bindings come from multi-pose part-lab measurements, NOT guessed mesh names.
local V={}
local applied,dirty,diagnostics={},{},{}
local colors={cinder={0xffe7c8ff,0xffbd7aff,0xff7945ff},rime={0xd6f0ffff,0x9bddffff,0x6ec7ffff}}
local slots={'assault','traversal','guard'}
function V.clear() gd.parts_clear();applied={};dirty={};diagnostics={} end
function V.dirty(port) dirty[port]=true end
function V.status() return {[1]=diagnostics[1],[2]=diagnostics[2]} end
function V.update(run,hosts,force)
 if not gd.dobj_tint or not gd.dobjs then return end
 for port=1,2 do
  local host=hosts[port];local h=host and run.hosts[host]
  local phases={};local key=tostring(host)
  for _,slot in ipairs(slots) do
   local id=h and h.slots[slot];local a=id and Core.ability(run,host,slot)
   local kind=id and run.genes[id].kind
   local phase=a and (a.ready and 3 or a.charge>0 and 2 or 1) or 0
   phases[slot]={kind=kind,phase=phase};key=key..':'..tostring(id)..':'..phase
  end
  local prior=applied[port]
  if force or dirty[port] or not prior or prior.key~=key or run.frame%30==0 then
   local p=gd.player(port);local parts=p and gd.parts(port,true)
   local draws=p and gd.dobjs(port)
   local topology=Bindings.topology(parts,p)
   local map,why=Bindings.resolve(parts,p,draws);diagnostics[port]=why
   -- Render flags may change without geometry identity changing. Resolve fresh
   -- flags and include the eligible draw list, so overlays never inherit tint.
   local selection=''
   if map then for _,slot in ipairs(slots) do selection=selection..slot..':'..table.concat(map[slot],',') end end
   if force or not prior or prior.key~=key or prior.topology~=topology or prior.selection~=selection then
    local live={};for _,d in ipairs(draws or {}) do live[d.index]=true end
    if prior then for id in pairs(prior.ids) do if live[id] then gd.dobj_solid_off(port,id) end end end
    local ids={}
    if map then for _,slot in ipairs(slots) do
     local phase=phases[slot];local color=colors[phase.kind]
     if color and phase.phase>0 then for _,id in ipairs(map[slot]) do gd.dobj_tint(port,id,color[phase.phase]);ids[id]=true end end
    end end
    applied[port]={key=key,topology=topology,selection=selection,ids=ids}
   end
   dirty[port]=nil
  end
 end
end
local function text(x,y,s,role,color,w)
 return gd.kit.text(x,y,s,role or 'caption',color or 'bone','left',{max_w=w or 220,shear=0})
end
function V.tree(view)
 local x,w=450,178
 if view.depth==0 then
  -- A small hint rail at rest; the fork expands only while navigating.
  gd.fill(x,440,w,28,0x101a2cb8)
  for i,b in ipairs(view.branches) do
   local bx=x+5+(i-1)*58
   gd.kit.icon('rogue_'..b.direction,bx,449,.25,'gold')
   text(bx+13,458,b.label,'caption','bone',43)
  end
  return
 end
 local h=26+#view.branches*25;local y=468-h
 gd.fill(x+2,y+2,w,h,0x03071290);gd.fill(x,y,w,h,0x101a2ce0)
 gd.fill(x,y,w,1,0xf0b429ff)
 text(x+8,y+17,view.title,'caption','gold',102)
 text(x+w-64,y+17,'UP: BACK','caption','muted',58)
 for i,b in ipairs(view.branches) do
  local by=y+23+(i-1)*25
  gd.fill(x+5,by,w-10,22,b.enabled and 0x1e3a8ccc or 0x242e43aa)
  gd.kit.icon('rogue_'..b.direction,x+10,by+6,.28,b.enabled and 'gold' or 'muted')
  text(x+29,by+16,b.label,'caption',b.enabled and 'bone' or 'disabled',w-36)
 end
end

function V.toast(message)
 gd.fill(15,434,613,34,0x030712cc)
 gd.fill(12,431,613,34,0x19283fee)
 gd.fill(12,431,3,34,0xf0b429ff)
 gd.kit.icon('rogue_gene',22,440,.48,'gold')
 text(47,453,message,'caption','bone',565)
end
return V
