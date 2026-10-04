-- Debug fly cursor: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("F: fly toggle; T: target 30,40; A: native debug attack; R: clear"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Debug fly cursor", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local speed,solid
function on_match_start() speed=gd.fly_speed(); solid=gd.fly_solid() end
function on_tick()
  if not gd.match().active then return end
  -- Fly calls refuse while the fighter is dead, held, respawning or still entering: report, do not raise.
  local function try(...) local ok,err=pcall(...); if not ok then status='Refused: '..tostring(err):gsub('^.-: ','') end end
  if pressed('F') then try(gd.fly,1,'toggle') end
  if pressed('T') then try(gd.fly_target,1,30,40) end
  if pressed('A') then try(gd.fly_attack,1,true,3,6) end
  if pressed('R') then try(gd.fly_clear,1) end
end
function on_draw()
  local s=gd.fly_state(1)
  caption(s and ('Flying %s; attacking %s; pulses %d (attempts, not hits)'):format(tostring(s.flying),tostring(s.attacking),s.pulses) or 'No cursor')
end
function on_unload()
  if gd.match().active then gd.fly_clear(1); gd.fly(1,false) end
  if speed then gd.fly_speed(speed); gd.fly_solid(solid) end
end

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
