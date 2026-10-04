-- Text, kit and safe area: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("K: toggle kit; resize the window to see safe-area layout"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Text, kit and safe area", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local use_kit = true
function on_tick() if pressed('K') then use_kit=not use_kit end end
function on_draw()
  caption('The canvas widens with the window; text fits the available width.')
  local a=gd.safe_area(); local w=math.min(a.w-48,420)
  if use_kit and gd.kit.available() then
    gd.kit.panel(a.right-w-24,145,w,150,{piece=20})
    gd.kit.icon('training',a.right-w-8,157,0.25,'gold')
    gd.kit.text(a.right-w+24,184,'DEMO CARD','label','bone','left',{max_w=w-64})
    gd.kit.button(a.right-w-8,202,w-32,'Safe area',true,{value=('%.0f wide'):format(a.w)})
    gd.kit.paragraph(a.right-w-8,259,w-32,'Kit textures are looked up by name. No copied art.','body')
  else gd.box(24,145,w,100); gd.line(24,145,24+w,245) end
end
function on_unload() gd.label('') end

-- Owner-local state for the external behavior tour; never exposes another mod's handles.
gd.command('demo_state',function(token)
  gd.log('tour state '..token..' '..tostring(tostring(use_kit))..' | '..status)
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
