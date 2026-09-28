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
local filter, filtering, recents, toasts = '', false, {}, {}
local editing, menu, action_index = false, false, 1
local depth, rotation, grid = 0, 0, 1
local collision, floor_flags, overlay = true, 3, true
local previous, old_pad, saved_states = nil, {}, {}
local filename, dirty, status = 'layout.lua', false, 'F1 help | F6 edit | map play layout.lua: load'
local autoload, broken = nil, false
local modal, error_text, last_action = nil, nil, nil -- hybrid-transform state; status severities (bible §4.3, §5.6, §5.9)
local MAX_PARTS, HISTORY = 128, 64 -- ScriptGame_ModelSpawn's instance pool; shared with other mods.
local TOOLS = { 'place', 'select', 'move', 'rotate', 'scale' }
local tool, axis_lock, snap_on, help_open = 'place', nil, true, false
local SCALE_STEP, SCALE_MIN, SCALE_MAX = 1.1, 0.25, 4.0
local hover, dragging, help_first = nil, false, 1
local PANEL_FILL = 0x0E1218F2 -- opaque editor panels: the game behind must not read through the text
local HINTS = {
  place='LMB/Ins: place; Up/Down: part; wheel: depth; hold G/E/C: transform',
  select='LMB / Tab: select the nearest part; F: frame; hold G/E/C: transform',
  move='hold G: drag to move; M: to cursor; Shift+C: constraint',
  rotate='hold E: drag to face the pointer; R / T: +-15 deg',
  scale='hold C: drag to scale; F7 / F8: -10% / +10%; Shift+X/Y: mirror',
}
local mouse = { x = -1000, y = -1000, buttons = 0, prev = 0, over = false, used = 0 }
local map_hinv, map_key = nil, nil
local mouse_api = gd.mouse ~= nil and gd.camera_get ~= nil

local function say(s, kind)
  status = tostring(s)
  if kind == 'error' then error_text = status
  elseif kind == 'action' then last_action = status error_text = nil end
  gd.log('map_editor: ' .. (kind == 'error' and 'error: ' or '') .. status)
end
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
local function sign(n) return n < 0 and -1 or 1 end
-- Ctrl (hold) snaps hard to the grid, Shift (hold) takes a quarter step (bible §4.5).
local function snap(n)
  local s = U*grid
  if gd.key('SHIFT') then s = s * 0.25 end
  return math.floor(n/s+0.5)*s
end
local function sn(v) return snap_on and snap(v) or v end
local function fly_cursor()
  local p=assert(gd.player(1), 'P1 is required for the flight cursor')
  return snap(p.x), snap(p.y), depth
end
local function find(list, id)
  for i,p in ipairs(list) do if p.id==id then return p,i end end
end
local function scale_of(p, k) return p[k] or 1 end
local function palette_match(name)
  return filter=='' or name:lower():find(filter,1,true)~=nil
end
local function palette_view()
  local out,pinned={},{}
  for _,name in ipairs(recents) do
    for i,n in ipairs(PALETTE) do
      if n==name and palette_match(name) then out[#out+1]=i pinned[i]=true end
    end
  end
  for i,name in ipairs(PALETTE) do
    if not pinned[i] and palette_match(name) then out[#out+1]=i end
  end
  return out
end
local function palette_part()
  local view=palette_view()
  return PALETTE[view[math.min(math.max(palette,1),#view)] or 1]
end
local function remember(name)
  for i,n in ipairs(recents) do if n==name then table.remove(recents,i) break end end
  table.insert(recents,1,name) if #recents>5 then table.remove(recents) end
end
local function toast(msg)
  toasts[#toasts+1]={msg=msg,t=100} if #toasts>3 then table.remove(toasts,1) end
end
local function canvas_w()
  local a=gd.safe_area and gd.safe_area()
  return a and tonumber(a.w) or 640
end

-- world/screen homography --------------------------------------------------------------------
-- The projection is opaque to Lua (fov and aspect live in the engine, and widescreen changes
-- the aspect), so a screen point maps to the depth plane through four projected sample points:
-- world->screen is a plane homography and gd.project supplies the samples; screen->world is its
-- inverse. No camera constants are assumed.
local function solve(a, b, n)
  for col = 1, n do
    local piv, best = col, math.abs(a[col][col])
    for r = col + 1, n do local v = math.abs(a[r][col]) if v > best then piv, best = r, v end end
    if best < 1e-12 then return nil end
    if piv ~= col then a[col], a[piv] = a[piv], a[col] b[col], b[piv] = b[piv], b[col] end
    for r = col + 1, n do
      local f = a[r][col] / a[col][col]
      if f ~= 0 then
        for c = col, n do a[r][c] = a[r][c] - f * a[col][c] end
        b[r] = b[r] - f * b[col]
      end
    end
  end
  local x = {}
  for r = n, 1, -1 do
    local sum = b[r]
    for c = r + 1, n do sum = sum - a[r][c] * x[c] end
    x[r] = sum / a[r][r]
  end
  return x
end
local function inv3(h)
  local a,b,c,d,e,f,g,i,j = h[1],h[2],h[3],h[4],h[5],h[6],h[7],h[8],h[9]
  local A,B,C =  (e*j - f*i), -(d*j - f*g),  (d*i - e*g)
  local D,E,F = -(b*j - c*i),  (a*j - c*g), -(a*i - b*g)
  local G,H,I =  (b*f - c*e), -(a*f - c*d),  (a*e - b*d)
  local det = a*A + b*B + c*C
  if math.abs(det) < 1e-12 then return nil end
  return { A/det,B/det,C/det,D/det,E/det,F/det,G/det,H/det,I/det }
end
local function build_map()
  if not mouse_api then return nil end
  local cam = gd.camera_get()
  if type(cam) ~= 'table' or type(cam.interest) ~= 'table' then return nil end
  local key = ('%g:%g:%g:%g:%g:%g:%g'):format(cam.interest.x, cam.interest.y, cam.interest.z, depth,
                                           cam.eye.x, cam.eye.y, cam.eye.z)
  if key == map_key then return map_hinv end
  local cx, cy = cam.interest.x, cam.interest.y
  local d = 150
  local corners = { {cx-d,cy-d}, {cx+d,cy-d}, {cx+d,cy+d}, {cx-d,cy+d} }
  local a, rhs = {}, {}
  for i = 1, 4 do
    local wx, wy = corners[i][1], corners[i][2]
    local sx, sy = gd.project(wx, wy, depth)
    if not number(sx) or not number(sy) then return nil end
    a[#a+1] = { wx, wy, 1, 0, 0, 0, -sx*wx, -sx*wy } rhs[#rhs+1] = sx
    a[#a+1] = { 0, 0, 0, wx, wy, 1, -sy*wx, -sy*wy } rhs[#rhs+1] = sy
  end
  local h = solve(a, rhs, 8)
  map_key = key
  map_hinv = h and inv3({ h[1],h[2],h[3],h[4],h[5],h[6],h[7],h[8],1 }) or nil
  return map_hinv
end
local function mouse_world()
  local hi = build_map()
  if not hi or not mouse.over then return nil end
  local w = hi[7]*mouse.x + hi[8]*mouse.y + hi[9]
  if math.abs(w) < 1e-9 then return nil end
  local wx = (hi[1]*mouse.x + hi[2]*mouse.y + hi[3]) / w
  local wy = (hi[4]*mouse.x + hi[5]*mouse.y + hi[6]) / w
  if not number(wx) or not number(wy) then return nil end
  return wx, wy
end
-- Display cursor: the mouse point when the pointer has been used recently, else the fly cursor.
local function shown_cursor()
  if mouse_api and mouse.used > 0 then
    local wx, wy = mouse_world()
    if wx then return sn(wx), sn(wy), depth end
  end
  return fly_cursor()
end

local function same(a,b)
  for _,k in ipairs({'part','x','y','z','rot','collision','floor_flags',
                     'scale','scale_x','scale_y','scale_z'}) do
    if a[k]~=b[k] then return false end
  end
  return true
end
local function options(p)
  return {x=p.x,y=p.y,z=p.z,rot=p.rot,collision=p.collision,floor_flags=p.floor_flags,
          scale=scale_of(p,'scale'),scale_x=scale_of(p,'scale_x'),
          scale_y=scale_of(p,'scale_y'),scale_z=scale_of(p,'scale_z')}
end
local function asset(name)
  if not assets[name] then assets[name]=assert(gd.model_load(name)) end
  return assets[name]
end

-- Reconcile by stable document ID: a transform touches one instance, not the whole map.
-- On failure, the caller reconciles back to the last document before advancing history.
-- A sign change in scale_x/scale_y forces a despawn/respawn: the model API rejects effective
-- axis-sign changes on instances that own collision (docs/scripting.md, runtime models).
local function sync(target)
  for id,h in pairs(handles) do
    local p=find(target,id)
    if not p or h.part~=p.part or h.collision~=p.collision or h.floor_flags~=p.floor_flags or
       sign(scale_of(h,'scale_x'))~=sign(scale_of(p,'scale_x')) or
       sign(scale_of(h,'scale_y'))~=sign(scale_of(p,'scale_y')) then
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
local function opt_scale(p,k)
  local v=p[k]
  if v==nil then return 1 end
  assert(number(v) and v~=0 and math.abs(v)>=0.001 and math.abs(v)<=100, 'invalid '..k)
  return v
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
                collision=p.collision,floor_flags=p.floor_flags,
                scale=opt_scale(p,'scale'),scale_x=opt_scale(p,'scale_x'),
                scale_y=opt_scale(p,'scale_y'),scale_z=opt_scale(p,'scale_z')}
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
    local fields=('{part=%q,x=%.17g,y=%.17g,z=%.17g,rot=%.17g,collision=%s,floor_flags=%d'):format(
      p.part,p.x,p.y,p.z,p.rot,tostring(p.collision),p.floor_flags)
    for _,k in ipairs({'scale','scale_x','scale_y','scale_z'}) do
      local v=scale_of(p,k)
      if v~=1 then fields=fields..(',%s=%.17g'):format(k,v) end
    end
    out[#out+1]=fields..'},'
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
  filename=name dirty=false say('Saved '..#parts..' parts to '..name) toast('Saved '..#parts..' parts')
end
local function load_map(name)
  assert(offline(), 'active offline match required')
  name=file_name(name or filename)
  local text=assert(gd.data_read(name), 'layout file not found: '..name)
  -- Text-only chunk with no globals: files cannot access gd, io or the script environment.
  local chunk,why=load(text,'@'..name,'t',{}) assert(chunk,why)
  local target=validate(chunk())
  apply(target,true) filename=name dirty=false
  say('Loaded '..#parts..' parts from '..name) toast('Loaded '..#parts..' parts')
end
local function set_overlay()
  if previous then gd.stage_view(previous.geometry,overlay) end
end
local function stop(restore_fly)
  if previous then
    if restore_fly~=false and offline() then gd.fly(1,previous.fly) end
    gd.stage_view(previous.geometry,previous.overlay)
  end
  previous=nil editing=false menu=false help_open=false dragging=false
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
local function place(duplicate, wx, wy)
  edit()
  local p=duplicate and assert(find(parts,selected), 'select a part first') or
    {part=palette_part(),rot=rotation,collision=collision,floor_flags=floor_flags,
     scale=1,scale_x=1,scale_y=1,scale_z=1}
  p=clone(p)
  if wx then p.x,p.y,p.z=sn(wx),sn(wy),depth else p.x,p.y,p.z=fly_cursor() end
  next_id=next_id+1 p.id=next_id
  local target=clone(parts) target[#target+1]=p apply(target,true) selected=p.id
  say((duplicate and 'Duplicated ' or 'Placed ')..p.part,'action')
  remember(p.part)
  if duplicate then toast('Duplicated '..p.part:gsub('^bf_','')) end
end
local function select_near(wx, wy)
  edit()
  local x,y,z
  if wx then x,y,z=wx,wy,depth else x,y,z=fly_cursor() end
  local best,d=nil,math.huge
  for _,p in ipairs(parts) do
    local distance=(p.x-x)^2+(p.y-y)^2+(p.z-z)^2
    if distance<d then best,d=p.id,distance end
  end
  selected=best say(best and ('Selected '..find(parts,best).part) or 'No parts to select','action')
end
local function transform(mode,delta,record,wx,wy)
  edit() local target=clone(parts) local p=assert(find(target,selected), 'select a part first')
  if mode=='move' then
    local x,y,z
    if wx then x,y,z=sn(wx),sn(wy),depth else x,y,z=fly_cursor() end
    if axis_lock=='x' then p.x=x elseif axis_lock=='y' then p.y=y else p.x,p.y,p.z=x,y,z end
  elseif mode=='rotate' then p.rot=((p.rot+delta+180)%360)-180
  elseif mode=='rotateto' then p.rot=((delta+180)%360)-180
  elseif mode=='scale' then
    p.scale=math.max(SCALE_MIN,math.min(SCALE_MAX,scale_of(p,'scale')*delta))
  elseif mode=='mirror' then p['scale_'..delta]=-scale_of(p,'scale_'..delta)
  elseif mode=='unscale' then p.scale,p.scale_x,p.scale_y,p.scale_z=1,1,1,1
  end
  apply(target,record ~= false) say(mode..' '..p.part,'action')
end
local function remove()
  edit() local target=clone(parts) local _,i=find(target,selected)
  assert(i,'select a part first') table.remove(target,i) apply(target,true) say('Deleted part','action')
end
local function history(back)
  edit() local from,to=back and undo or redo,back and redo or undo
  local target=from[#from] assert(target,back and 'Nothing to undo' or 'Nothing to redo')
  local old=clone(parts) apply(target,false) table.remove(from) to[#to+1]=old
  say(back and 'Undo' or 'Redo','action')
end
local function cycle_tool()
  local i=1
  for k,name in ipairs(TOOLS) do if name==tool then i=k end end
  tool=TOOLS[i%#TOOLS+1] axis_lock=nil say('Tool: '..tool)
end
local function cycle_axis()
  axis_lock = axis_lock==nil and 'x' or axis_lock=='x' and 'y' or nil
  say('Move constraint: '..(axis_lock or 'free'))
end
local function rotate_to_mouse()
  edit()
  local p=assert(find(parts,selected),'select a part first')
  local wx,wy=mouse_world() assert(wx,'no world point under the pointer')
  local deg=math.deg(math.atan(wy-p.y,wx-p.x))
  if snap_on then deg=math.floor(deg/15+0.5)*15 end
  transform('rotateto',deg)
end

local function frame_selection()
  local p=assert(find(parts,selected),'select a part first')
  depth=p.z
  if gd.player(1) then gd.teleport(1,sn(p.x),sn(p.y)) end
  say('Framed '..p.part,'action')
end

-- Hybrid transforms (bible §4.3-§4.5): holding G/E/C runs a modal transform that follows the
-- pointer, commits on release and cancels on ESC; a quick tap switches tool instead. Arrows lock
-- the axis during a modal. The base snapshot makes the commit idempotent.
local function modal_point(base)
  local target=clone(base)
  local p=find(target,selected) if not p then return nil end
  local wx,wy=mouse_world()
  if modal.mode=='move' then
    if modal.ax=='x' then p.x=sn(wx or p.x)
    elseif modal.ax=='y' then p.y=sn(wy or p.y)
    else p.x,p.y,p.z=sn(wx or p.x),sn(wy or p.y),depth end
  elseif modal.mode=='rotate' then
    if wx then
      local deg=math.deg(math.atan(wy-p.y,wx-p.x))
      if snap_on then deg=math.floor(deg/15+0.5)*15 end
      p.rot=((deg+180)%360)-180
    end
  elseif wx then
    local d=math.max(0.5,math.sqrt((wx-p.x)^2+(wy-p.y)^2))
    p.scale=math.max(SCALE_MIN,math.min(SCALE_MAX,scale_of(p,'scale')*(d/modal.d0)))
  end
  return target
end
local function modal_key(name, mode)
  if gd.key_pressed(name) and not (modal and modal.mode==mode) then
    local p=find(parts,selected)
    if not p then tool=mode axis_lock=nil say('Tool: '..mode,'action') return true end
    local wx,wy=mouse_world()
    modal={mode=mode, base=clone(parts), mx=mouse.x, my=mouse.y, used=false, ax=axis_lock,
           d0=wx and math.max(0.5,math.sqrt((wx-p.x)^2+(wy-p.y)^2)) or 1}
  end
  if not (modal and modal.mode==mode) then return false end
  if gd.key_pressed('ESCAPE') then apply(modal.base,false) modal=nil say('Cancelled') return true end
  if gd.key_pressed('LEFT') or gd.key_pressed('RIGHT') then
    modal.ax=(modal.ax=='x') and nil or 'x' say('Axis: '..(modal.ax or 'free'))
  elseif gd.key_pressed('UP') or gd.key_pressed('DOWN') then
    modal.ax=(modal.ax=='y') and nil or 'y' say('Axis: '..(modal.ax or 'free'))
  end
  if not gd.key(name) then
    if modal.used then
      local target=modal_point(modal.base)
      local tp,bp=find(target or {},selected),find(modal.base,selected)
      if target and tp and bp and not same(tp,bp) then
        undo[#undo+1]=modal.base if #undo>HISTORY then table.remove(undo,1) end
        redo={}
        apply(target,false)
        say(mode..' '..tp.part,'action')
      end
    else
      tool=mode axis_lock=nil say('Tool: '..mode,'action')
    end
    modal=nil
  elseif math.abs(mouse.x-modal.mx)+math.abs(mouse.y-modal.my)>3 then
    modal.used=true
    local target=modal_point(modal.base)
    if target then apply(target,false) end
  end
  return true
end

-- The first ten entries keep their order: the contract tests and muscle memory rely on it.
local ACTIONS = {
  {'Place',function() place(false) end},   {'Select nearest',select_near},
  {'Frame selection',frame_selection},
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
  {'Clear map (undoable)',function() edit() apply({},true) say('Cleared map') toast('Cleared map') end},
  {'Exit editor / play',function() stop() say('Editor closed; map remains live') end},
  {'Next tool',cycle_tool},
  {'Tool: place',function() tool='place' axis_lock=nil say('Tool: place') end},
  {'Tool: select',function() tool='select' axis_lock=nil say('Tool: select') end},
  {'Tool: move',function() tool='move' axis_lock=nil say('Tool: move') end},
  {'Tool: rotate',function() tool='rotate' axis_lock=nil say('Tool: rotate') end},
  {'Tool: scale',function() tool='scale' axis_lock=nil say('Tool: scale') end},
  {'Scale selected +10%',function() transform('scale',SCALE_STEP) end},
  {'Scale selected -10%',function() transform('scale',1/SCALE_STEP) end},
  {'Reset scale',function() transform('unscale') end},
  {'Mirror selected X',function() transform('mirror','x') end},
  {'Mirror selected Y',function() transform('mirror','y') end},
  {'Reset rotation',function() transform('rotateto',0) end},
  {'Move constraint: free / X / Y',cycle_axis},
  {'Snap on/off',function() snap_on=not snap_on say('Snap '..(snap_on and 'on' or 'off')) end},
  {'Help / keybinds',function() help_open=not help_open end},
}
local function attempt(fn)
  local ok,why=pcall(fn) if not ok then say('Error: '..tostring(why),'error') end return ok
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
    elseif op=='scale' then transform('scale',tonumber(name) or SCALE_STEP)
    elseif op=='unscale' then transform('unscale')
    elseif op=='mirror' then
      assert(name=='x' or name=='y', 'mirror x|y')
      transform('mirror',name)
    elseif op=='tool' then
      local found=nil
      for _,t in ipairs(TOOLS) do if t==name then found=t end end
      assert(found,'map tool: place|select|move|rotate|scale')
      tool=found axis_lock=nil say('Tool: '..tool)
    elseif op=='snap' then
      assert(name=='on' or name=='off','snap on|off') snap_on=name=='on' say('Snap '..name)
    elseif op=='help' then
      assert(name=='on' or name=='off','help on|off') help_open=name=='on' say('Help '..name)
    elseif op=='delete' then remove()
    elseif op=='undo' then history(true)
    elseif op=='redo' then history(false)
    elseif op=='clear' then edit() apply({},true)
    elseif op=='part' then
      local want=name and (name:find('^bf_') and name or 'bf_'..name)
      local found=nil
      for i,n in ipairs(PALETTE) do if n==want then found=i end end
      assert(found,'map part: unknown part '..tostring(name))
      filter='' filtering=false
      local view=palette_view() palette=1
      for pos,idx in ipairs(view) do if idx==found then palette=pos end end
      say('Part: '..want)
    elseif op=='filter' then
      filter=(name or ''):lower() filtering=false palette=1
      say(filter=='' and 'Filter cleared' or ('Filter: '..filter))
    else error('map on|off|part <name>|tool <name>|filter [text]|scale <f>|mirror x|y|snap on|off|help on|off|place|select|move|rotate [deg]|duplicate|delete|undo|redo|clear|save|load|play [file.lua]') end
  end)
end,'map on/off; part <name>; tool <place|select|move|rotate|scale>; scale <factor>; mirror x|y; snap on|off; place/select/move/rotate/duplicate/delete/undo/redo/clear; save/load/play [file.lua]')

-- Panel geometry is one function for drawing and hit-testing, so rows and clicks cannot drift.
local function panel_rows()
  local rows={}
  local y=70
  for i,t in ipairs(TOOLS) do
    rows[#rows+1]={kind='tool',index=i,label=(tool==t and '> ' or '  ')..t,x=16,y=y,w=230,h=20}
    y=y+21
  end
  y=192
  if tool=='place' then
    local view=palette_view()
    if #view==0 then
      rows[#rows+1]={kind='none',label='(no matches)',x=16,y=y,w=230,h=18}
    else
      local first=math.max(1,math.min(palette-3,math.max(1,#view-6)))
      for i=first,math.min(first+6,#view) do
        local name=PALETTE[view[i]]
        local pinned=false
        for _,r in ipairs(recents) do if r==name then pinned=true end end
        rows[#rows+1]={kind='part',index=i,label=(pinned and '* ' or '  ')..name:gsub('^bf_',''),x=16,y=y,w=230,h=18}
        y=y+20
      end
    end
  else
    local first=math.max(1,math.min(action_index-3,#ACTIONS-6))
    for i=first,math.min(first+6,#ACTIONS) do
      rows[#rows+1]={kind='action',index=i,label=ACTIONS[i][1],x=16,y=y,w=230,h=18}
      y=y+20
    end
  end
  rows[#rows+1]={kind='help',label='Help',value='F1',x=16,y=344,w=230,h=20}
  return rows
end
local function in_rect(mx,my,r)
  return r and mx>=r.x and mx<=r.x+r.w and my>=r.y and my<=r.y+r.h
end
local function inspector_rows()
  local rows={}
  local p=find(parts,selected)
  local x=canvas_w()-240
  local y=70
  if not p then
    rows[#rows+1]={kind='info',label='no selection',x=x,y=y,w=224,h=20}
    return rows
  end
  rows[#rows+1]={kind='name',label=p.part:gsub('^bf_',''),x=x,y=y,w=224,h=20}
  y=y+24
  for _,f in ipairs({{'x','%.2f'},{'y','%.2f'},{'z','%.2f'},{'rot','%.1f'},{'scale','%.2f'}}) do
    local value=f[1]=='scale' and scale_of(p,'scale') or (p[f[1]] or 0)
    rows[#rows+1]={kind='field',field=f[1],label=f[1],value=string.format(f[2],value),x=x,y=y,w=224,h=18}
    y=y+20
  end
  rows[#rows+1]={kind='collision',label='collision',value=p.collision and 'on' or 'off',x=x,y=y,w=224,h=18}
  y=y+20
  rows[#rows+1]={kind='flags',label='floor flags',value=tostring(p.floor_flags or 0),x=x,y=y,w=224,h=18}
  return rows
end
local function click_inspector(mx,my)
  if help_open then return false end
  for _,r in ipairs(inspector_rows()) do
    if in_rect(mx,my,r) then
      if r.kind=='collision' then
        attempt(function()
          edit()
          local target=clone(parts) local p=find(target,selected)
          p.collision=not p.collision
          apply(target,true) say('collision '..(p.collision and 'on' or 'off'),'action')
        end)
      elseif r.kind=='flags' then
        attempt(function()
          edit()
          local target=clone(parts) local p=find(target,selected)
          p.floor_flags=((p.floor_flags or 0)+1)%4
          apply(target,true) say('floor flags '..p.floor_flags,'action')
        end)
      end
      return true
    end
  end
  return false
end
local function click_panel(mx,my)
  for _,r in ipairs(panel_rows()) do
    if in_rect(mx,my,r) then
      if r.kind=='tool' then tool=TOOLS[r.index] axis_lock=nil say('Tool: '..tool)
      elseif r.kind=='part' then palette=r.index say('Part: '..palette_part())
      elseif r.kind=='help' then help_open=not help_open
      else attempt(ACTIONS[r.index][2]) end
      return true
    end
  end
  return false
end

local function poll_mouse()
  if not mouse_api or not editing then return end
  local mx,my,buttons,wheel=gd.mouse()
  mouse.x,mouse.y,mouse.buttons=tonumber(mx) or -1000,tonumber(my) or -1000,tonumber(buttons) or 0
  mouse.over = mouse.x>=0 and mouse.x<canvas_w() and mouse.y>=0 and mouse.y<480
  if mouse.over and (mouse.x~=mouse.lastx or mouse.y~=mouse.lasty) then
    mouse.used,mouse.lastx,mouse.lasty=60,mouse.x,mouse.y
  elseif mouse.used>0 then
    mouse.used=mouse.used-1
  end
  if modal then mouse.prev=mouse.buttons return end -- a modal owns the pointer; no clicks, no depth
  local lmb=(mouse.buttons & 1)==1
  local pressed=lmb and (mouse.prev & 1)==0
  local right=(mouse.buttons & 2)==2 and (mouse.prev & 2)==0
  if wheel and wheel~=0 then
    if help_open then
      help_first=math.max(1,math.min(math.max(1,#HELP-11),help_first+wheel))
    else
      depth=depth+wheel*U*grid say(('depth %.2f'):format(depth))
    end
  end
  if right then
    if help_open then help_open=false else menu=not menu end
  elseif pressed then
    if not click_panel(mouse.x,mouse.y) and not click_inspector(mouse.x,mouse.y) and mouse.over and not help_open then
      local wx,wy=mouse_world()
      if tool=='select' then attempt(function() select_near(wx,wy) end)
      elseif tool=='place' then attempt(function() place(false,wx,wy) end)
      elseif tool=='move' then dragging=true attempt(function() transform('move',nil,true,wx,wy) end)
      elseif tool=='rotate' then attempt(rotate_to_mouse)
      elseif tool=='scale' then attempt(function() transform('scale',SCALE_STEP) end)
      end
    end
  end
  if lmb and dragging then
    local wx,wy=mouse_world()
    if wx then attempt(function() transform('move',nil,false,wx,wy) end) end
  end
  if not lmb then dragging=false end
  mouse.prev=mouse.buttons
end

-- The part nearest the shown cursor, for the hover highlight and the readout only.
local function hover_part()
  local x,y=shown_cursor() local best,d=nil,20.0
  for _,p in ipairs(parts) do
    local distance=math.sqrt((p.x-x)^2+(p.y-y)^2)
    if distance<d then best,d=p.id,distance end
  end
  return best
end

function on_tick()
  local pad=gd.pad(1) or {}
  for _,t in ipairs(toasts) do t.t=t.t-1 end
  local function pressed(k) return pad[k] and not old_pad[k] end
  if not offline() then stop() old_pad=pad return end
  if gd.key_pressed('F6') then attempt(function() if editing then stop() else start() end end) end
  if not editing then
    if pad.Z and pressed('UP') then attempt(start) end
    old_pad=pad return
  end
  if gd.key_pressed('F1') or gd.key_pressed('H') then help_open=not help_open end
  if help_open then
    if gd.key_pressed('ESCAPE') and not menu then help_open=false end
    if gd.key_pressed('DOWN') then help_first=math.min(math.max(1,#HELP-11),help_first+1) end
    if gd.key_pressed('UP') then help_first=math.max(1,help_first-1) end
  end
  if gd.key_pressed('F4') then
    filtering=not filtering
    if not filtering then filter='' end
    palette=1
    say(filtering and 'Type a part name; Enter done, ESC clears' or 'Filter cleared')
  end
  if filtering then
    local ch=''
    for c in ('ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'):gmatch('.') do
      if gd.key_pressed(c) then ch=c end
    end
    if ch~='' then filter=filter..ch:lower() palette=1 end
    if gd.key_pressed('BACKSPACE') then filter=filter:sub(1,-2) palette=1 end
    if gd.key_pressed('ESCAPE') then filter='' filtering=false palette=1 end
    if gd.key_pressed('ENTER') then filtering=false end
    old_pad=pad return
  end
  if gd.key_pressed('F2') or pressed('Z') then menu=not menu end
  if menu then
    if gd.key_pressed('UP') or pressed('UP') then action_index=(action_index-2)%#ACTIONS+1 end
    if gd.key_pressed('DOWN') or pressed('DOWN') then action_index=action_index%#ACTIONS+1 end
    if gd.key_pressed('ENTER') or pressed('A') then attempt(ACTIONS[action_index][2]) end
    if gd.key_pressed('ESCAPE') or pressed('B') then menu=false end
  else
    local handled = false
    if not help_open then
      handled = (modal and ((modal.mode=='move' and modal_key('G','move')) or
                            (modal.mode=='rotate' and modal_key('E','rotate')) or
                            (modal.mode=='scale' and modal_key('C','scale')))) or
                (not modal and not gd.key('SHIFT') and
                 (modal_key('G','move') or modal_key('E','rotate') or modal_key('C','scale')))
    end
    if handled then old_pad=pad return end
    for i,t in ipairs(TOOLS) do
      if gd.key_pressed(tostring(i)) then tool=t axis_lock=nil say('Tool: '..t) end
    end
    if not help_open then
      local view=palette_view()
      if #view>0 then
        if gd.key_pressed('UP') or pressed('UP') then palette=(palette-2)%#view+1 end
        if gd.key_pressed('DOWN') or pressed('DOWN') then palette=(palette-1)%#view+1 end
      end
    end
    if gd.key_pressed('PAGEUP') or pressed('RIGHT') then depth=depth+U*grid end
    if gd.key_pressed('PAGEDOWN') or pressed('LEFT') then depth=depth-U*grid end
    if gd.key_pressed('INSERT') or pressed('A') then attempt(function() place(false) end) end
    if gd.key_pressed('TAB') or pressed('X') then attempt(select_near) end
    if gd.key_pressed('M') or pressed('Y') then attempt(function() transform('move') end) end
    if gd.key_pressed('R') or pressed('R') then attempt(function() transform('rotate',15) end) end
    if gd.key_pressed('T') or pressed('L') then attempt(function() transform('rotate',-15) end) end
    if gd.key_pressed('F7') then attempt(function() transform('scale',1/SCALE_STEP) end) end
    if gd.key_pressed('F8') then attempt(function() transform('scale',SCALE_STEP) end) end
    if gd.key('SHIFT') and gd.key_pressed('C') then cycle_axis() end
    if gd.key_pressed('Z') then snap_on=not snap_on say('Snap '..(snap_on and 'on' or 'off'),'action') end
    if gd.key_pressed('F') then attempt(frame_selection) end
    if gd.key_pressed('DELETE') then attempt(remove) end
    if gd.key_pressed('F3') then overlay=not overlay set_overlay() end
    if gd.key('CTRL') then
      if gd.key_pressed('D') then attempt(function() place(true) end) end
      if gd.key_pressed('Z') then attempt(function() history(true) end) end
      if gd.key_pressed('Y') then attempt(function() history(false) end) end
      if gd.key_pressed('S') then attempt(function() save() end) end
      if gd.key_pressed('O') then attempt(function() load_map() end) end
    end
    if gd.key('SHIFT') then
      if gd.key_pressed('X') then attempt(function() transform('mirror','x') end) end
      if gd.key_pressed('Y') then attempt(function() transform('mirror','y') end) end
    end
  end
  old_pad=pad
end
function on_frame_pre()
  if not editing or not offline() then return end
  if mouse_api then poll_mouse() end
  hover=hover_part()
  if menu or help_open or modal or filtering or gd.key('CTRL') then return end
  local p=gd.player(1) if not p then return end
  local dx=(gd.key('D') and 1 or 0)-(gd.key('A') and 1 or 0)
  local dy=(gd.key('W') and 1 or 0)-(gd.key('S') and 1 or 0)
  if dx~=0 or dy~=0 then
    local speed=gd.fly_speed()*(gd.key('SHIFT') and 4 or 1)
    gd.teleport(1,p.x+dx*speed,p.y+dy*speed)
  end
end

HELP = {
  {'F6', 'start / exit the editor'},
  {'F1 / H', 'this help (ESC or click closes)'},
  {'F4', 'palette filter (type; Enter done, ESC clears)'},
  {'F2 / Z', 'action menu (all commands)'},
  {'1..5', 'tool: place, select, move, rotate, scale'},
  {'G / E / C', 'hold: move / rotate / scale (tap: switch tool)'},
  {'F', 'frame selection (fly cursor + depth)'},
  {'Shift / Ctrl', 'fine step / hard snap while dragging'},
  {'Z', 'snap toggle'},
  {'WASD', 'fly (Shift fast); P1 stick on pad'},
  {'LMB', 'use the tool at the pointer'},
  {'drag LMB', 'move tool: drag the selection'},
  {'RMB', 'action menu'},
  {'wheel', 'depth, one grid step per notch'},
  {'Tab / X', 'select nearest part'},
  {'Insert / A', 'place at the cursor'},
  {'M / Y', 'move selection to cursor'},
  {'R / T', 'rotate +15 / -15'},
  {'rotate drag', 'rotate tool: face the pointer'},
  {'F7 / F8', 'scale -10% / +10%'},
  {'Shift+X / Y', 'mirror selection X / Y'},
  {'Shift+C', 'move constraint free / X / Y'},
  {'PgUp/PgDn', 'depth +/- one grid step'},
  {'Ctrl+D', 'duplicate at cursor'},
  {'Delete', 'remove selection'},
  {'Ctrl+Z / Y', 'undo / redo'},
  {'Ctrl+S / O', 'save / load layout'},
  {'F3', 'collision overlay'},
}
local function draw_help()
  local kit=gd.kit
  local visible=12
  local pages=math.max(1,math.ceil(#HELP/visible))
  local last=math.max(1,#HELP-visible+1)
  if help_first>last then help_first=last end
  local hx=(canvas_w()-560)/2
  gd.fill(hx,24,560,432,0x0E1218FF)
  kit.panel(hx,24,560,432,{piece=16,fill=PANEL_FILL})
  gd.box(hx,24,560,432,0x8A92A0FF)
  local track_y, track_h = 120, 300
  local kh = math.max(30, math.floor(track_h*visible/#HELP))
  local kf = (help_first-1)/math.max(1,#HELP-visible)
  gd.fill(hx+544,track_y,8,track_h,0x464F5EFF)
  gd.fill(hx+544,track_y+kf*(track_h-kh),8,kh,0xE8C878FF)
  kit.text(hx+20,54,'MAP EDITOR - KEYBINDS','label','gold')
  kit.text(hx+20,76,('page %d/%d - Up/Down scrolls - F1/H/ESC closes')
    :format((help_first-1)//visible+1,pages),'caption','muted',nil,{max_w=480})
  kit.text(hx+20,96,'pad: Z menu; Tab/X select; M/Y move; R/L rotate','caption','muted',nil,{max_w=480})
  for i=help_first,math.min(#HELP,help_first+visible-1) do
    local y=120+(i-help_first)*28
    kit.text(hx+20,y,HELP[i][1],'caption','bone')
    kit.paragraph(hx+130,y,410,HELP[i][2],'caption','muted')
  end
end
function on_draw()
  if not editing or not offline() or not gd.player(1) then return end
  local x,y,z=shown_cursor()
  local sx,sy,on=gd.project(x,y,z)
  if sx and on then gd.line(sx-7,sy,sx+7,sy,0xFFE080FF) gd.line(sx,sy-7,sx,sy+7,0xFFE080FF) end
  local p=find(parts,selected)
  if p then
    local px,py,visible=gd.project(p.x,p.y,p.z)
    if px and visible then
      gd.box(px-10,py-10,20,20,0x60FFFFFF)
      -- Axis handles: 2 world units along +X / +Y, projected, so the gizmo follows the camera.
      local ax,ay=gd.project(p.x+2,p.y,p.z)
      local bx,by=gd.project(p.x,p.y+2,p.z)
      if ax then gd.line(px,py,ax,ay,0xE06060FF) end
      if bx then gd.line(px,py,bx,by,0x60E060FF) end
      if tool=='rotate' then
        local t=math.rad(p.rot)
        gd.line(px,py,px+math.cos(t)*22,py-math.sin(t)*22,0xFFD060FF)
      end
    end
  end
  local h=hover and find(parts,hover)
  if h and h.id~=selected then
    local hx,hy,hv=gd.project(h.x,h.y,h.z)
    if hx and hv then gd.box(hx-8,hy-8,16,16,0xC080FFFF) end
  end
  local kit=gd.kit
  if not kit.available() then gd.text(12,12,'Map editor: menu kit assets missing; use map console commands') return end
  local W=canvas_w()
  gd.fill(8,10,W-16,30,0x0E1218FF)
  kit.panel(8,10,W-16,30,{piece=12,fill=PANEL_FILL})
  kit.text(20,30,'MAP EDITOR','label','gold')
  kit.text(150,30,('tool: %s%s'):format(tool,axis_lock and (' ['..axis_lock..' axis]') or ''),'caption','bone')
  kit.text(340,30,('part: %s'):format(palette_part():gsub('^bf_','')),'caption','bone',nil,{max_w=140})
  kit.text(W-12,30,('%s%s'):format(dirty and '* ' or '',filename),'caption','muted','right',{max_w=200})
  gd.fill(8,44,246,338,0x0E1218FF)
  kit.panel(8,44,246,338,{piece=16,fill=PANEL_FILL})
  kit.text(20,62,'TOOL','caption','gold')
  kit.text(20,186,tool=='place' and ('PART'..(filter~='' and (' /'..filter..(filtering and '_' or '')) or '')) or 'ACTIONS','caption','gold')
  for _,r in ipairs(panel_rows()) do
    local state=(r.kind=='tool' and tool==TOOLS[r.index]) or
                (r.kind=='part' and palette==r.index) or
                (r.kind=='action' and action_index==r.index) or
                (r.kind=='help' and help_open) or false
    local value = r.kind=='tool' and tostring(r.index) or r.value
    kit.button(r.x,r.y,r.w,r.label,state and 'sel' or 'ng',{h=r.h,value=value})
  end
  gd.fill(W-248,44,240,338,0x0E1218FF)
  kit.panel(W-248,44,240,338,{piece=16,fill=PANEL_FILL})
  kit.text(W-236,62,'SELECTION','caption','gold')
  for _,r in ipairs(inspector_rows()) do
    kit.button(r.x,r.y,r.w,r.label,'ng',{h=r.h,value=r.value})
  end
  gd.fill(8,376,W-16,54,0x0E1218FF)
  kit.panel(8,376,W-16,54,{piece=12,fill=PANEL_FILL})
  kit.text(20,392,HINTS[tool],'caption','bone',nil,{max_w=440})
  kit.text(W-12,392,('XYZ %.2f %.2f %.2f'):format(x,y,z),'caption','muted','right',{max_w=170})
  kit.text(20,408,(dirty and '* ' or '')..filename,'caption','muted',nil,{max_w=300})
  kit.text(W-12,408,('grid %.2g m | snap %s | parts %d/%d'):format(grid,snap_on and 'on' or 'off',#parts,MAX_PARTS),
    'caption','muted','right',{max_w=300})
  local line=error_text and ('error: '..error_text) or (last_action or '')
  kit.text(20,424,line,'caption',error_text and 'danger' or 'gold',nil,{max_w=598})
  for i=#toasts,1,-1 do if toasts[i].t<=0 then table.remove(toasts,i) end end
  for i,t in ipairs(toasts) do
    local ty=356-(i-1)*22
    gd.fill(W-178,ty,166,18,0x0E1218E0)
    kit.text(W-170,ty+13,t.msg,'caption','gold',nil,{max_w=150})
  end
  if help_open then gd.fill(0,0,W,480,0x000000A8) draw_help() end
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

gd.log('map_editor: ready; F6 or Z+D-pad Up to edit; F1 help; map play layout.lua to load a map')
