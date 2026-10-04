-- Offline collision-injection demo; this does not animate P2's attack move.
local types={'knockback','damage_threshold','knockback_threshold','super','hit_count','damage_pool'}
local values={30,8,60,1,3,30}
local index,started,clock,barrage=1,false,0,false
local status='Waiting for P1 and P2 in offline match'
local event='No armour reaction observed yet'
local hits={{damage=3,angle=45,kbg=30,bkb=10,from=2},
  {damage=10,angle=45,kbg=70,bkb=30,from=2},
  {damage=25,angle=45,kbg=110,bkb=70,from=2}}
local labels={'weak','medium','strong'}
local function apply()
  if not gd.fighter_armor(1,nil) then status='Clear refused'; return false end
  local ok=gd.fighter_armor(1,{type=types[index],value=values[index],frames=600,direction='any'})
  status=ok and 'Applied '..types[index] or 'Armour refused'
  event='Awaiting collision; percent continues accumulating'
  return ok
end
local function strike(which)
  local ok=gd.hit(1,hits[which])
  status=ok and ('Queued '..labels[which]..' P2 collision hit') or 'Hit refused'
end
function on_frame()
  if not gd.match().active or not gd.player(1) or not gd.player(2) then return end
  if not started then
    started=true; clock=0
    if gd.player(1).cpu then gd.cpu_mode(1,'stand') end
    gd.cpu_mode(2,'stand'); apply()
  end
  clock=clock+1
  if barrage and clock%90==0 then strike((math.floor(clock/90)-1)%3+1) end
end
function on_tick()
  if not started or not gd.match().active or not gd.player(1) or not gd.player(2) then return end
  if gd.key_pressed('A') then index=index%#types+1; apply() end
  if gd.key_pressed('R') then apply() end
  if gd.key_pressed('W') then strike(1) end
  if gd.key_pressed('M') then strike(2) end
  if gd.key_pressed('S') then strike(3) end
  if gd.key_pressed('SPACE') then barrage=not barrage; status='Auto barrage '..tostring(barrage) end
end
function on_armor(e)
  if not e or e.port~=1 then return end
  event=tostring(e.type)..' absorbed='..tostring(e.absorbed)..' broke='..tostring(e.broke)
    ..' damage='..tostring(e.damage)..' KB='..tostring(e.knockback)
  gd.log('armor demo '..event)
end
local function detail()
  local rows=gd.fighter_armor(1) or {}; local texts={}
  for _,entry in ipairs(rows) do
    texts[#texts+1]=tostring(entry.type)..' value='..tostring(entry.value)
      ..' remaining='..tostring(entry.remaining)..' frames='..tostring(entry.frames)
      ..' enabled='..tostring(entry.enabled)
  end
  return #texts>0 and table.concat(texts,'; ') or 'No active typed armour'
end
function on_draw()
  local a=gd.safe_area(); local width=math.min(a.w-24,720); local lines={}
  local function wrap(text)
    local limit=math.max(12,math.floor((width-24)/7))
    while #text>limit do lines[#lines+1]=text:sub(1,limit); text=text:sub(limit+1) end
    lines[#lines+1]=text
  end
  wrap('A: cycle six types; R: refresh; W/M/S: weak/medium/strong; SPACE: barrage')
  wrap(status); wrap(started and gd.match().active and detail() or 'Waiting for match')
  wrap(event); wrap('Scripted collision hits from P2; real hit reactions and percent. No direct percent writes.')
  gd.fill(a.x+12,a.y+12,width,48+#lines*16,0x101827dd)
  gd.text(a.x+24,a.y+22,'Armour types',0xffd369ff,1.2)
  for i,line in ipairs(lines) do gd.text(a.x+24,a.y+46+(i-1)*16,line) end
end
function on_match_end() started=false; barrage=false end
function on_unload() if started and gd.match().active then gd.fighter_armor(1,nil) end end
gd.command('demo_state',function(token)
  gd.log('tour state '..tostring(token)..' '..status..' | '..(started and gd.match().active and detail() or 'waiting')..' | '..event)
end,'demo_state <token>: report typed armour demo')
