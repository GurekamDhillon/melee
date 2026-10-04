-- Offline single-feature demo. Main fighter entities 1..6, secondary 7..12.
-- First active frame also covers loading into an existing match.
local started=false
local status='Waiting for offline match / P1'
local choices,index={0,1,3,8},3
local function apply() return gd.fighter_caps(1,{air_jumps=choices[index]}) end
local function controls()
  if gd.key_pressed('J') then index=index%#choices+1; status=apply() and 'Allowance updated' or 'Change refused' end
  if gd.key_pressed('R') then status=gd.fighter_caps(1,nil) and 'Retail restored' or 'Reset refused' end
end
local function detail()
  local c=gd.fighter_caps(1)
  return 'P1 air jumps: '..tostring(c and c.air_jumps or 'unavailable')..' (-1 means retail)'
end
local function cleanup() gd.fighter_caps(1,nil) end
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
  gd.text(a.x+24,a.y+22,"Air-jump allowance",0xffd369ff,1.2)
  local y=a.y+48
  for _,line in ipairs({"J: cycle 0, 1, 3, 8 extra jumps; R: retail defaults",status,started and gd.match().active and detail() or 'Waiting for P1'}) do
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
