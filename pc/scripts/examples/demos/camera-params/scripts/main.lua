-- Retail camera parameters: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("G: zero yaw/pitch gains; R: stage baseline; walk away from origin"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Retail camera parameters", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

function on_tick()
  if not gd.match().active then return end
  if pressed('G') then gd.camera_params{yaw_gain=0,pitch_gain=0}; status='Origin skew removed' end
  if pressed('R') then gd.camera_params(nil); status='Stage baseline restored' end
end
function on_draw() caption() end
function on_unload() if gd.match().active then gd.camera_params(nil) end end

-- Owner-local state for the external behavior tour; never exposes another mod's handles.
gd.command('demo_state',function(token)
  gd.log('tour state '..token..' '..tostring(status)..' | '..status)
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
