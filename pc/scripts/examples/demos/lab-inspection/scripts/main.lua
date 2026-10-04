-- LAB hitboxes and frame timeline: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("J: request jab frame 1 (pauses); SPACE: resume; B: debug draw toggle"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "LAB hitboxes and frame timeline", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local debug,previous=false,nil
function on_match_start() previous=gd.debug_draw(1) end
function on_tick()
  if not gd.match().active then return end
  if pressed('J') then
    local moves=gd.motion_list(1) or {}; local jab
    for _,m in ipairs(moves) do if m.name=='Attack11' then jab=m.id; break end end
    if jab then local ok,why=gd.set_motion(1,jab,1); status=ok and 'Motion queued; paused at boundary' or tostring(why)
    else status='Attack11 not found in motion_list' end
  end
  if pressed('SPACE') then gd.resume() end
  if pressed('B') then debug=not debug; gd.debug_draw(1,debug and 3 or previous or 0) end
end
function on_draw()
  local p=gd.player(1); local t=gd.timeline(1)
  caption(p and ('%s: %d hitboxes, timeline %s'):format(gd.motion_name(p.action,1),#(gd.hitboxes(1) or {}),t and tostring(t.length) or '-') or 'No fighter')
end
function on_unload() if gd.match().active and previous then gd.debug_draw(1,previous); gd.resume() end end

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
