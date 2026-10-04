-- Fighter state and control: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("T: teleport P1; P: P2 percent +25; C: stand/fight CPU"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Fighter state and control", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local saved, fight = {}, false
function on_match_start()
  saved={}; for _,p in ipairs(gd.players()) do saved[p.port]={x=p.x,y=p.y,percent=p.percent} end
  if gd.player(2) then gd.cpu_mode(2,'stand') end
end
function on_tick()
  if not gd.match().active then return end
  if pressed('T') then gd.teleport(1,0,45) end
  local p=gd.player(2)
  if p and pressed('P') then gd.set_percent(2,p.percent+25) end
  if p and pressed('C') then fight=not fight; gd.cpu_mode(2,fight and 'fight' or 'stand') end
end
function on_draw()
  local p=gd.player(1)
  caption(p and ('%s: action %d frame %.1f, x %.1f y %.1f; P2 %s'):format(
    p.char_name,p.action,p.action_frame,p.x,p.y,fight and 'fight' or 'stand') or 'No fighter')
end
function on_unload()
  if not gd.match().active then return end
  for port,p in pairs(saved) do gd.set_percent(port,p.percent); gd.teleport(port,p.x,p.y) end
  -- No CPU-mode query exists; restore fight, documented baseline for these demos.
  if gd.player(2) then gd.cpu_mode(2,'fight') end
end
function on_match_end() saved={} end

-- Owner-local state for the external behavior tour; never exposes another mod's handles.
gd.command('demo_state',function(token)
  gd.log('tour state '..token..' '..tostring(tostring(fight))..' | '..status)
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
