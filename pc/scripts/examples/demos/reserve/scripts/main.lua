-- Bench and call a reserve: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("B: bench P2; C: call P2 onto floor; refused states shown"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Bench and call a reserve", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local owned, home = false,nil
function on_match_start() local p=gd.player(2); home=p and {x=p.x,y=p.y} end
function on_tick()
  if not gd.match().active or not gd.player(2) then return end
  if pressed('B') then
    local ok,why=gd.fighter_bench(2); owned=ok or owned
    status=ok and 'Reserve hidden, frozen and camera-excluded' or tostring(why)
  elseif pressed('C') and owned then
    local y=gd.floor_below(25,100) or 0
    local ok,why=gd.fighter_call(2,25,y,{facing=-1,intangible_frames=60})
    if ok then owned=false end; status=ok and 'Called with 60 intangible frames' or tostring(why)
  end
end
function on_draw() caption(status..'; benched='..tostring(gd.fighter_benched(2))) end
function on_unload()
  if owned and home and gd.match().active then gd.fighter_call(2,home.x,home.y) end
end
function on_match_end() owned=false; home=nil end

-- Owner-local state for the external behavior tour; never exposes another mod's handles.
gd.command('demo_state',function(token)
  gd.log('tour state '..token..' '..tostring(tostring(owned))..' | '..status)
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
