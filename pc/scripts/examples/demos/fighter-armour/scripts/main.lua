-- Offline single-feature demo. Main fighter entities 1..6, secondary 7..12.
-- First active frame also covers loading into an existing match.
local started=false
local status='Waiting for offline match / P1'
local damage=true
local requested='none'
local function apply()
  local ok=gd.fighter_armour(1,damage and {damage=12} or {knockback=60})
  if ok then requested=damage and 'damage < 12' or 'knockback < 60' end
  return ok
end
local function controls()
  if gd.key_pressed('A') then damage=not damage; status=apply() and 'Armour updated' or 'Change refused' end
  if gd.key_pressed('R') then local ok=gd.fighter_armour(1,nil); if ok then requested='none' end; status=ok and 'Armour cleared' or 'Reset refused' end
end
local function detail()
  local a=gd.fighter_armour(1)
  return 'P1 damage threshold '..tostring(a and a.damage or 0)..'; KB threshold '..tostring(a and a.knockback or 0)..'; percent still rises'
end
local function cleanup() gd.fighter_armour(1,nil) end
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
  gd.text(a.x+24,a.y+22,"Flinch resistance",0xffd369ff,1.2)
  local y=a.y+48
  for _,line in ipairs({"A: damage threshold / knockback threshold; R: clear armour",status,started and gd.match().active and detail() or 'Waiting for P1'}) do
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
