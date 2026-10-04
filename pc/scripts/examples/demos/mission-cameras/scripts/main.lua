-- Mission camera modes: read this file top to bottom; offline LAB, vanilla Final Destination.
-- Host keys live in on_tick; simulation observations live in on_frame.
local status = "Ready"
local requested = {}
local function pressed(key)
  if requested[key] then requested[key]=nil; return true end
  return gd.key_pressed(key)
end
gd.command('demo_key',function(key) requested[key]=true end,'demo_key <uppercase key>: demo-owned tour control')
local function caption(detail)
  local a = gd.safe_area()
  local width=math.min(a.w-24,590)
  local function wrap(value)
    local rows,line={},''; local limit=math.max(12,math.floor((width-24)/7))
    for word in tostring(value):gmatch('%S+') do
      if #line+#word+1>limit and #line>0 then rows[#rows+1]=line; line='' end
      while #word>limit do rows[#rows+1]=word:sub(1,limit); word=word:sub(limit+1) end
      line=line=='' and word or line..' '..word
    end
    if line~='' then rows[#rows+1]=line end
    return rows
  end
  local controls,notes=wrap("F: follow; C: chunk; V: shaft; R: retail; jump along primitive route"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Mission camera modes", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local mode,handles,centre='follow',{},0
function on_match_start()
  handles={}; centre=0
  for i=0,3 do handles[#handles+1]=assert(gd.stage_add_platform(i*35-50,i*24+20,30,{passthrough=true})) end
  gd.camera_bounds(true)
end
function on_tick()
  if pressed('F') then mode='follow' end
  if pressed('C') then mode='chunk' end
  if pressed('V') then mode='shaft' end
  if pressed('R') then mode='retail'; gd.camera_attach(20) end
end
function on_frame()
  local p=gd.player(1); if not p then return end
  if mode=='follow' then gd.camera_follow(p,{x=15*p.facing,y=30,z=220})
  elseif mode=='shaft' then gd.camera_set{eye={x=0,y=p.y+30,z=350},interest={x=0,y=p.y+30,z=0},fov=30}
  elseif mode=='chunk' then
    local next_centre=p.x<0 and -45 or 45
    if next_centre~=centre then
      centre=next_centre
      gd.camera_move{to={eye={x=centre,y=55,z=260},interest={x=centre,y=55,z=0}},frames=45,ease='inout'}
    end
  end
end
function on_draw() caption('Mode '..mode..'; see missions for outer-clamp and live-aspect implementation') end
function on_unload()
  if gd.match().active then gd.camera_bounds(false); gd.camera_attach(0); for _,h in ipairs(handles) do gd.stage_remove(h) end end
end
function on_match_end() handles={}; centre=0 end

-- Owner-local state for the external behavior tour; never exposes another mod's handles.
gd.command('demo_state',function(token)
  gd.log('tour state '..token..' '..tostring(mode)..' | '..status)
end,'demo_state <token>: report this demo state')

-- Catch up when loaded into an existing match; never initialize twice.
local begin, frame, finish = on_match_start, on_frame, on_match_end
local started = false
function on_match_start()
  if not started then started=true; if begin then begin() end end
end
function on_frame()
  if gd.match().active then on_match_start(); if frame then frame() end end
end
function on_match_end()
  if finish then finish() end
  started=false
end
