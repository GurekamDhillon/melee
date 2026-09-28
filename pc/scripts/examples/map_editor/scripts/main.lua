-- Offline editor and map loader. Models belong to this mod's models/ directory.
-- No require/io: layout files live in gd.data_read/write's script-local sandbox.
-- BEGIN GENERATED KIT
local U = 6.5
local PALETTE = {
  "bf_balcony_4m",
  "bf_balcony_4m_glass",
  "bf_beam_4m",
  "bf_corner_inside_4m",
  "bf_corner_outside_4m",
  "bf_door_leaf",
  "bf_floor_2m",
  "bf_floor_4m",
  "bf_floor_end_trim",
  "bf_floor_opening_4m",
  "bf_ramp_4m_rise2m",
  "bf_ramp_4m_rise2m_glass",
  "bf_rear_glass_rail_4m",
  "bf_rear_glass_rail_4m_glass",
  "bf_rear_post_4m",
  "bf_stairs_4m_rise2m",
  "bf_wall_doorway_4m",
  "bf_wall_side_return",
  "bf_wall_solid_4m",
  "bf_wall_window_4m",
  "bf_window_glass_insert_glass",
}
-- END GENERATED KIT

local known = {}
for _, name in ipairs(PALETTE) do known[name] = true end
local parts, handles, assets, undo, redo = {}, {}, {}, {}, {}
local selected, next_id, palette = nil, 0, 1
local editing, menu, action_index = false, false, 1
local depth, rotation, grid = 0, 0, 1
local collision, floor_flags, overlay = true, 3, true
local previous, old_pad, saved_states = nil, {}, {}
local filename, dirty, status = 'layout.lua', false, 'F6: edit | map play layout.lua: load for play'
local autoload, broken = nil, false
local MAX_PARTS, HISTORY = 128, 64 -- ScriptGame_ModelSpawn's instance pool; shared with other mods.

local function say(s) status = tostring(s) gd.log('map_editor: ' .. status) end
local function offline()
  local m = gd.match()
  return m.active and not m.netplay
end
local function clone(t)
  local out = {}
  for k,v in pairs(t) do out[k] = type(v)=='table' and clone(v) or v end
  return out
end
local function number(n)
  return type(n)=='number' and n==n and math.abs(n)<=100000
end
local function snap(n) local s=U*grid return math.floor(n/s+0.5)*s end
local function cursor()
  local p=assert(gd.player(1), 'P1 is required for the flight cursor')
  return snap(p.x), snap(p.y), depth
end
local function find(list, id)
  for i,p in ipairs(list) do if p.id==id then return p,i end end
end
local function same(a,b)
  for _,k in ipairs({'part','x','y','z','rot','collision','floor_flags'}) do
    if a[k]~=b[k] then return false end
  end
  return true
end
local function options(p)
  return {x=p.x,y=p.y,z=p.z,rot=p.rot,collision=p.collision,floor_flags=p.floor_flags}
end
local function asset(name)
  if not assets[name] then assets[name]=assert(gd.model_load(name)) end
  return assets[name]
end

-- Reconcile by stable document ID: a transform touches one instance, not the whole map.
-- On failure, the caller reconciles back to the last document before advancing history.
local function sync(target)
  for id,h in pairs(handles) do
    local p=find(target,id)
    if not p or h.part~=p.part or h.collision~=p.collision or h.floor_flags~=p.floor_flags then
      gd.model_despawn(h.handle) handles[id]=nil
    end
  end
  for _,p in ipairs(target) do
    local h=handles[p.id]
    if h then
      if not same(h,p) then assert(gd.model_set(h.handle,options(p)), 'model update refused') end
    else
      local handle,why=gd.model_spawn(asset(p.part),options(p))
      assert(handle,why or 'model spawn refused')
      h={handle=handle} handles[p.id]=h
    end
    for k,v in pairs(p) do h[k]=v end
  end
end
local function apply(target, record)
  assert(offline(), 'active offline match required')
  assert(not broken, 'model recovery failed; save the document and restart the match')
  assert(#target<=MAX_PARTS, 'model limit: 128 (shared with other mods)')
  local ok,why=pcall(sync,target)
  if not ok then
    local restored,err=pcall(sync,parts)
    if not restored then broken=true say('Recovery failed: '..tostring(err)) end
    error(why,0)
  end
  if record then
    undo[#undo+1]=clone(parts) if #undo>HISTORY then table.remove(undo,1) end
    redo={}
  end
  parts=clone(target) dirty=true
  if not find(parts,selected) then selected=parts[#parts] and parts[#parts].id end
end
local function validate(data)
  assert(type(data)=='table' and getmetatable(data)==nil and data.version==1, 'layout version must be 1')
  assert(data.units==U, 'kit scale differs; re-export the kit or convert the layout')
  assert(type(data.parts)=='table' and getmetatable(data.parts)==nil, 'parts must be a list')
  local count=0
  for k in pairs(data.parts) do
    assert(type(k)=='number' and k%1==0 and k>=1 and k<=MAX_PARTS, 'invalid part index')
    count=count+1
  end
  assert(count==#data.parts and count<=MAX_PARTS, 'sparse or oversized part list')
  local out={}
  for _,p in ipairs(data.parts) do
    assert(type(p)=='table' and getmetatable(p)==nil and known[p.part], 'unknown kit part')
    for _,k in ipairs({'x','y','z','rot'}) do assert(number(p[k]), 'invalid '..k) end
    assert(math.abs(p.rot)<=360, 'rotation must be -360..360')
    assert(type(p.collision)=='boolean', 'collision must be boolean')
    assert(type(p.floor_flags)=='number' and p.floor_flags%1==0 and p.floor_flags>=0 and p.floor_flags<=3,
           'floor_flags must be 0..3')
    next_id=next_id+1
    out[#out+1]={id=next_id,part=p.part,x=p.x,y=p.y,z=p.z,rot=p.rot,
                collision=p.collision,floor_flags=p.floor_flags}
  end
  return out
end
local function file_name(name)
  assert(type(name)=='string' and #name<=80 and name:match('^[%w_-]+%.lua$'),
         'use a plain filename such as layout.lua')
  return name
end
local function serialize()
  local out={('-- Kit layout v1; world units, Z is visual depth. Grid: %.17g units/metre.\nreturn {version=1,units=%.17g,parts={'):format(U,U)}
  for _,p in ipairs(parts) do
    out[#out+1]=('{part=%q,x=%.17g,y=%.17g,z=%.17g,rot=%.17g,collision=%s,floor_flags=%d},'):format(
      p.part,p.x,p.y,p.z,p.rot,tostring(p.collision),p.floor_flags)
  end
  out[#out+1]='}}\n' return table.concat(out,'\n')
end
local function save(name)
  name=file_name(name or filename)
  local text=serialize()
  -- Preserve the last file separately; read-back also catches short native writes.
  local old=gd.data_read(name)
  if old then gd.data_write(name..'.bak',old) end
  gd.data_write(name,text)
  assert(gd.data_read(name)==text, 'save read-back failed; previous file is in .bak')
  filename=name dirty=false say('Saved '..#parts..' parts to '..name)
end
local function load_map(name)
  assert(offline(), 'active offline match required')
  name=file_name(name or filename)
  local text=assert(gd.data_read(name), 'layout file not found: '..name)
  -- Text-only chunk with no globals: files cannot access gd, io or the script environment.
  local chunk,why=load(text,'@'..name,'t',{}) assert(chunk,why)
  local target=validate(chunk())
  apply(target,true) filename=name dirty=false
  say('Loaded '..#parts..' parts from '..name)
end
local function set_overlay()
  if previous then gd.stage_view(previous.geometry,overlay) end
end
local function stop(restore_fly)
  if previous then
    if restore_fly~=false and offline() then gd.fly(1,previous.fly) end
    gd.stage_view(previous.geometry,previous.overlay)
  end
  previous=nil editing=false menu=false
end
local function start()
  assert(offline(), 'active offline match required')
  assert(gd.player(1), 'P1 is required')
  assert(not broken, 'save layout and restart match before editing')
  if editing then return end
  local geometry,lines=gd.stage_view()
  previous={fly=gd.fly(1),geometry=geometry,overlay=lines}
  assert(gd.fly(1,true), 'flight refused')
  editing=true autoload=nil set_overlay() say('Editing '..filename)
end
local function edit()
  assert(editing and offline(), 'enable the editor in an offline match first')
  assert(not broken, 'save and restart: engine state needs recovery')
end
local function place(duplicate)
  edit()
  local p=duplicate and assert(find(parts,selected), 'select a part first') or
    {part=PALETTE[palette],rot=rotation,collision=collision,floor_flags=floor_flags}
  p=clone(p) p.x,p.y,p.z=cursor() next_id=next_id+1 p.id=next_id
  local target=clone(parts) target[#target+1]=p apply(target,true) selected=p.id
  say((duplicate and 'Duplicated ' or 'Placed ')..p.part)
end
local function select_near()
  edit() local x,y,z=cursor() local best,d=nil,math.huge
  for _,p in ipairs(parts) do
    local distance=(p.x-x)^2+(p.y-y)^2+(p.z-z)^2
    if distance<d then best,d=p.id,distance end
  end
  selected=best say(best and ('Selected '..find(parts,best).part) or 'No parts to select')
end
local function transform(mode,delta)
  edit() local target=clone(parts) local p=assert(find(target,selected), 'select a part first')
  if mode=='move' then p.x,p.y,p.z=cursor()
  else p.rot=((p.rot+delta+180)%360)-180 end
  apply(target,true) say(mode..' '..p.part)
end
local function remove()
  edit() local target=clone(parts) local _,i=find(target,selected)
  assert(i,'select a part first') table.remove(target,i) apply(target,true) say('Deleted part')
end
local function history(back)
  edit() local from,to=back and undo or redo,back and redo or undo
  local target=from[#from] assert(target,back and 'Nothing to undo' or 'Nothing to redo')
  local old=clone(parts) apply(target,false) table.remove(from) to[#to+1]=old
  say(back and 'Undo' or 'Redo')
end
local ACTIONS = {
  {'Place',function() place(false) end}, {'Select nearest',select_near},
  {'Move selected to cursor',function() transform('move') end},
  {'Rotate selected +15',function() transform('rotate',15) end},
  {'Rotate selected -15',function() transform('rotate',-15) end},
  {'Duplicate at cursor',function() place(true) end}, {'Delete selected',remove},
  {'Undo',function() history(true) end}, {'Redo',function() history(false) end},
  {'Save layout',function() save() end}, {'Load layout',function() load_map() end},
  {'Collision overlay',function() overlay=not overlay set_overlay() end},
  {'Grid 1 / 0.5 / 0.25 metre',function() grid=grid==1 and 0.5 or grid==0.5 and 0.25 or 1 end},
  {'New parts: collision on/off',function() collision=not collision end},
  {'New floors: flags 0..3',function() floor_flags=(floor_flags+1)%4 end},
  {'New parts: rotation +15',function() rotation=((rotation+15+180)%360)-180 end},
  {'Clear map (undoable)',function() edit() apply({},true) say('Cleared map') end},
  {'Exit editor / play',function() stop() say('Editor closed; map remains live') end},
}
local function attempt(fn)
  local ok,why=pcall(fn) if not ok then say('Error: '..tostring(why)) end return ok
end

gd.command('map',function(arg)
  attempt(function()
    local op,name=arg:match('^(%S+)%s*(.-)%s*$') op=op or 'toggle'
    if name=='' then name=nil end
    if op=='save' then save(name) return end -- Permit recovery of unsaved data outside a match.
    assert(offline(), 'active offline match required')
    if op=='on' then start()
    elseif op=='off' then stop()
    elseif op=='toggle' then if editing then stop() else start() end
    elseif op=='load' then load_map(name) start()
    elseif op=='play' then load_map(name) stop() autoload=filename
    elseif op=='place' then place(false)
    elseif op=='duplicate' then place(true)
    elseif op=='select' then select_near()
    elseif op=='move' then transform('move')
    elseif op=='rotate' then transform('rotate',tonumber(name) or 15)
    elseif op=='delete' then remove()
    elseif op=='undo' then history(true)
    elseif op=='redo' then history(false)
    elseif op=='clear' then edit() apply({},true)
    elseif op=='part' then
      local want=name and (name:find('^bf_') and name or 'bf_'..name)
      local found=nil
      for i,n in ipairs(PALETTE) do if n==want then found=i end end
      assert(found,'map part: unknown part '..tostring(name))
      palette=found say('Part: '..want)
    else error('map on|off|part <name>|place|select|move|rotate [degrees]|duplicate|delete|undo|redo|clear|save|load|play [file.lua]') end
  end)
end,'map on/off; part <name>; save/load/play [file.lua]; place/select/move/rotate/duplicate/delete/undo/redo/clear')

function on_tick()
  local pad=gd.pad(1) or {}
  local function pressed(k) return pad[k] and not old_pad[k] end
  if not offline() then stop() old_pad=pad return end
  if gd.key_pressed('F6') then attempt(function() if editing then stop() else start() end end) end
  if not editing then
    if pad.Z and pressed('UP') then attempt(start) end
    old_pad=pad return
  end
  if gd.key_pressed('F2') or pressed('Z') then menu=not menu end
  if menu then
    if gd.key_pressed('UP') or pressed('UP') then action_index=(action_index-2)%#ACTIONS+1 end
    if gd.key_pressed('DOWN') or pressed('DOWN') then action_index=action_index%#ACTIONS+1 end
    if gd.key_pressed('ENTER') or pressed('A') then attempt(ACTIONS[action_index][2]) end
    if gd.key_pressed('ESCAPE') or pressed('B') then menu=false end
  else
    if gd.key_pressed('UP') or pressed('UP') then palette=(palette-2)%#PALETTE+1 end
    if gd.key_pressed('DOWN') or pressed('DOWN') then palette=palette%#PALETTE+1 end
    if gd.key_pressed('PAGEUP') or pressed('RIGHT') then depth=depth+U*grid end
    if gd.key_pressed('PAGEDOWN') or pressed('LEFT') then depth=depth-U*grid end
    if gd.key_pressed('INSERT') or pressed('A') then attempt(function() place(false) end) end
    if gd.key_pressed('TAB') or pressed('X') then attempt(select_near) end
    if gd.key_pressed('M') or pressed('Y') then attempt(function() transform('move') end) end
    if gd.key_pressed('R') or pressed('R') then attempt(function() transform('rotate',15) end) end
    if gd.key_pressed('T') or pressed('L') then attempt(function() transform('rotate',-15) end) end
    if gd.key_pressed('DELETE') then attempt(remove) end
    if gd.key_pressed('F3') then overlay=not overlay set_overlay() end
    if gd.key('CTRL') then
      if gd.key_pressed('D') then attempt(function() place(true) end) end
      if gd.key_pressed('Z') then attempt(function() history(true) end) end
      if gd.key_pressed('Y') then attempt(function() history(false) end) end
      if gd.key_pressed('S') then attempt(function() save() end) end
      if gd.key_pressed('O') then attempt(function() load_map() end) end
    end
  end
  old_pad=pad
end
function on_frame_pre()
  if not editing or menu or not offline() or gd.key('CTRL') then return end
  local p=gd.player(1) if not p then return end
  local dx=(gd.key('D') and 1 or 0)-(gd.key('A') and 1 or 0)
  local dy=(gd.key('W') and 1 or 0)-(gd.key('S') and 1 or 0)
  if dx~=0 or dy~=0 then
    local speed=gd.fly_speed()*(gd.key('SHIFT') and 4 or 1)
    gd.teleport(1,p.x+dx*speed,p.y+dy*speed)
  end
end
function on_draw()
  if not editing or not offline() or not gd.player(1) then return end
  local x,y,z=cursor()
  local sx,sy,on=gd.project(x,y,z)
  if sx and on then gd.line(sx-7,sy,sx+7,sy,0xFFE080FF) gd.line(sx,sy-7,sx,sy+7,0xFFE080FF) end
  local p=find(parts,selected)
  if p then
    local px,py,visible=gd.project(p.x,p.y,p.z)
    if px and visible then gd.box(px-10,py-10,20,20,0x60FFFFFF) end
  end
  local kit=gd.kit
  if not kit.available() then gd.text(12,12,'Map editor: menu kit assets missing; use map console commands') return end
  kit.panel(8,12,246,302,{piece=16})
  kit.text(20,38,menu and 'MAP ACTIONS' or 'KIT PALETTE','label')
  local list,index=menu and ACTIONS or PALETTE,menu and action_index or palette
  local first=math.max(1,math.min(index-3,#list-6)) local rows={}
  for i=first,math.min(first+6,#list) do
    rows[#rows+1]={label=menu and list[i][1] or list[i]:gsub('^bf_',''),value=tostring(i)}
  end
  kit.list(20,50,222,rows,index-first+1,{pitch=26,h=24})
  kit.text(20,266,('%d/%d parts | grid %gm'):format(#parts,MAX_PARTS,grid),'caption','gold')
  kit.text(20,286,('XYZ %.2f %.2f %.2f'):format(x,y,z),'caption','bone',nil,{max_w=220})
  kit.text(20,306,('New: %d deg | collision %s | flags %d'):format(rotation,tostring(collision),floor_flags),
           'caption','muted',nil,{max_w=220})
  kit.panel(8,318,624,122,{piece=16}) -- ends above the FLY readout (y ~443)
  kit.text(20,342,(dirty and '* ' or '')..filename..' | '..status,'caption','gold',nil,{max_w=598})
  kit.text(20,362,'WASD / stick: fly | Up/Down: palette | PgUp/PgDn / D-left/right: depth','caption','bone',nil,{max_w=598})
  kit.text(20,382,'Insert/A: place | Tab/X: select nearest | M/Y: move | R,T / R,L: rotate','caption','bone',nil,{max_w=598})
  kit.text(20,402,'Ctrl D/Z/Y: duplicate/undo/redo | Delete: remove | Ctrl S/O: save/load','caption','bone',nil,{max_w=598})
  kit.text(20,422,'F2/Z: actions (all controls) | F3: collision | F6: exit | flags: 1 pass, 2 ledges','caption','muted',nil,{max_w=598})
end

-- Model slots are snapshotted, Lua is not. Pair manual savestates with document snapshots;
-- on other loads, refuse to mutate until a new match rather than deleting unknown instances.
function on_savestate(slot)
  saved_states[slot]={parts=clone(parts),handles=clone(handles),assets=clone(assets),
                      previous=previous and clone(previous),selected=selected,broken=broken}
end
function on_loadstate(slot)
  if not offline() then return end
  -- Host overlay is not snapshotted. Restore it from the current UI session, but do not
  -- overwrite restored native flight with the current timeline's saved fly setting.
  stop(false)
  local saved=saved_states[slot]
  if saved then
    if saved.previous then gd.fly(1,saved.previous.fly) end
    parts,handles,assets=clone(saved.parts),clone(saved.handles),clone(saved.assets)
    selected=saved.selected
    undo={} redo={} dirty=true broken=saved.broken
    for _,h in pairs(handles) do if not gd.model_get(h.handle) then broken=true end end
    say(broken and 'Savestate handles missing; save and restart' or 'Restored document; undo history cleared')
  else
    assets={} -- Unknown native references: do not release another snapshot's asset tokens.
    broken=true say('Untracked state load: save layout and restart match before editing')
  end
end
function on_match_end()
  -- Engine already frees scene models. Retain unsaved document for save/reinstantiation.
  stop() handles={} assets={} undo={} redo={} saved_states={} broken=false
end
function on_match_start()
  handles={} assets={} broken=false
  if not offline() then return end
  attempt(function()
    if autoload then load_map(autoload) elseif #parts>0 then sync(parts) say('Restored layout on new stage') end
  end)
end
function on_unload()
  stop()
  if offline() then
    for _,h in pairs(handles) do pcall(gd.model_despawn,h.handle) end
    for _,a in pairs(assets) do pcall(gd.model_release,a) end
  end
  handles={} assets={}
end

gd.log('map_editor: ready; F6 or Z+D-pad Up to edit; map play layout.lua to load a map')
