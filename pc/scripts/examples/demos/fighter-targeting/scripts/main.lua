-- Offline single-feature demo. Main fighter entities 1..6, secondary 7..12.
-- First active frame also covers loading into an existing match.
local started=false
local status='Waiting for offline match / P1'
local target
local marked={}
local function apply()
  target=gd.nearest_opponent(1)
  local ok=target and gd.fighter_timed_status(target,1,7,120) or false
  if ok then marked[target]=true end
  return ok
end
local function controls()
  if gd.key_pressed('T') then status=apply() and 'Nearest opponent marked' or 'No eligible opponent' end
end
local function detail()
  local near=gd.nearest_opponent(1)
  local ids=gd.opponents_in_radius(1,80) or {}
  local mark=target and gd.fighter_timed_status(target,1)
  return 'Nearest '..tostring(near or 'none')..'; in radius '..#ids..'; marker '..tostring(mark and mark.value or 'expired')..' / '..tostring(mark and mark.frames or 0)..'f'
end
local function cleanup() for entity in pairs(marked) do gd.fighter_timed_status(entity,1,0,0) end; marked={} end
function on_frame()
  if not gd.match().active or not gd.player(1) then return end
  if not started then
    started=true
    status=apply() and 'Initial setup applied' or 'Initial setup refused'
    gd.log(status)
  end
end
function on_tick()
  if started and gd.match().active and gd.player(1) then controls() end
end
function on_match_end() started=false; status='Waiting for match' end
function on_unload() if started and gd.match().active then cleanup() end end
function on_draw()
  local a=gd.safe_area(); local width=math.min(a.w-24,680)
  gd.fill(a.x+12,a.y+12,width,176,0x101827dd)
  gd.text(a.x+24,a.y+22,"Opponent queries and timed status",0xffd369ff,1.2)
  local y=a.y+48
  for _,line in ipairs({"T: mark nearest opponent for 120 frames; radius 80 units",status,started and gd.match().active and detail() or 'Waiting for P1'}) do
    local limit=math.max(12,math.floor((width-24)/7))
    while #line>limit do
      gd.text(a.x+24,y,line:sub(1,limit)); y=y+16; line=line:sub(limit+1)
    end
    gd.text(a.x+24,y,line); y=y+16
  end
end
gd.command('demo_state',function(token)
  gd.log('tour state '..tostring(token)..' '..status..' | '..(started and gd.match().active and detail() or 'waiting'))
end,'demo_state <token>: read this demo state')
