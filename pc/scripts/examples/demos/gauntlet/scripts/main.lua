-- Gauntlet: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("Fight room 1; reach maze goal in room 2; KO boss in room 3; ENTER retries"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Gauntlet", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local room,frames,wave,enemies,drops,areas=0,0,0,{},{},{}
local boss,home,goal,maze,best,falls,buffed,started_at,finished
local pending_boss,camera_cell,setup_start,save_allowed=false,nil,0,false
local function read_table(path)
  local text,why=gd.mod_read(path); assert(text,why)
  local safe={assert=assert,type=type,math=math,ipairs=ipairs,pairs=pairs,next=next,
    string=string,table=table,tostring=tostring,tonumber=tonumber}
  return assert(load(text,'@'..path,'t',safe))()
end
local function wave_spawn()
  wave=wave+1; enemies={}
  for i=1,(wave==1 and 2 or 1) do
    local x=-30+i*30
    local h,why=gd.spawn_enemy(wave==1 and 'goomba' or 'redead',x,12)
    assert(h,why); enemies[h]={x=x,y=12}
  end
  status='Room 1, wave '..wave..': defeat the enemies'
end
local building,camera_pending,generation=false,false,0
local visible_lines={}
local function abandon()
  generation=generation+1; building=false; camera_pending=false
  for _,name in ipairs(areas) do gd.area_unload(name) end; areas={}; visible_lines={}
  for h in pairs(enemies) do gd.enemy_remove(h) end; enemies={}
  for h in pairs(drops) do gd.item_despawn(h) end; drops={}
  gd.stage_restore_bounds(); if not boss then gd.stage_hide(false) end
  if home and gd.fighter_benched(2) then gd.fighter_call(2,home.x,home.y) end
end
local function setup()
  if building then return false end
  assert(gd.match().stage~=nil,'LAB match required')
  local p=assert(gd.player(2),'P2 CPU boss required'); assert(p.cpu,'P2 must be CPU')
  local ok,why=gd.fighter_bench(2)
  if not ok then status='Waiting for safe boss reserve: '..tostring(why); return false end
  home={x=p.x,y=p.y}
  frames,wave,enemies,drops,areas=0,0,{},{},{}
  buffed,finished,pending_boss=0,false,false
  local bytes,err=gd.data_read('best.txt')
  best=bytes and tonumber(bytes) or nil
  save_allowed=(not bytes and err=='missing') or (best and best>0 and best%1==0 and best<1000000000)
  if not save_allowed then best=nil; gd.log('Prior best unavailable/invalid; saving disabled to preserve it') end
  if not bytes and err~='missing' then gd.log('Best read refused',err) end
  building=true
  generation=generation+1; local ticket=generation; local epoch=gd.scene().epoch
  status='Building one maze cell per frame; please wait'
  gd.run(function()
    local ok,err=pcall(function()
    assert(gd.item_define('items/gauntlet_drive/item.json'))
    local level=read_table('missions/gauntlet/level.lua')
    assert(gd.area_load('gauntlet_entry',function()
      for _,l in ipairs(level.lines) do assert(gd.stage_add_line(l[1],l[2],l[3],l[4],l[5])) end
    end)); areas[#areas+1]='gauntlet_entry'
    for _,l in ipairs(level.lines) do visible_lines[#visible_lines+1]={l[1],l[2],l[3],l[4]} end
    gd.wait(1)
    if ticket~=generation or gd.scene().epoch~=epoch then return end
    -- Reuse the project's pure generator, contained as text; no runtime require.
    local set=read_table('missions/gauntlet/maze_set.lua')()
    local factory=read_table('missions/gauntlet/maze.lua')
    local generator=factory({maze_set=set})
    maze=generator.generate(42,{size=8,length=4})
    gd.log('Gauntlet seed 42\n'..generator.ascii(maze))
    local lo,hi,bottom,top=0,200,-30,120
    for _,c in ipairs(maze.cells) do
      if ticket~=generation or gd.scene().epoch~=epoch then return end
      local geometry=generator.geometry(c)
      local dx=220
      local name='gauntlet_'..c.id
      assert(gd.area_load(name,function()
        for _,l in ipairs(geometry.lines) do
          assert(gd.stage_add_line(l.x1+dx,l.y1,l.x2+dx,l.y2,l.kind,{passthrough=l.passthrough}))
          visible_lines[#visible_lines+1]={l.x1+dx,l.y1,l.x2+dx,l.y2}
        end
      end)); areas[#areas+1]=name
      gd.wait(1)
      lo=math.min(lo,c.x*130+dx); hi=math.max(hi,(c.x+1)*130+dx)
      bottom=math.min(bottom,c.y*104); top=math.max(top,(c.y+1)*104)
      if c.id==maze.goal then goal={x=c.x*130+dx+100,y=c.y*104+8} end
    end
    if ticket~=generation or gd.scene().epoch~=epoch then return end
    gd.stage_set_camera_bounds(lo-100,hi+100,top+100,bottom-100)
    gd.stage_set_blast_bounds(lo-180,hi+180,top+180,bottom-180)
    assert(gd.stage_hide(true),'Gauntlet host must be FD')
    gd.teleport(1,-65,8); gd.stage_set_spawn(4,-65,8)
    falls=gd.player(1).falls or 0; wave_spawn(); started_at=gd.frame(); room=1; camera_pending=true
    end)
    if ticket~=generation then return end
    building=false
    if not ok then abandon(); room=0; finished=true; status='Setup failed: '..tostring(err)..'; ENTER retries'; gd.log(status) end
  end)
  return true
end
function on_match_start()
  finished=false; setup_start=gd.frame(); setup()
end
function on_enemy_defeated(e)
  local pos=enemies[e.handle]; if not pos then return end
  enemies[e.handle]=nil
  local h,why=gd.item_spawn('gauntlet_drive',pos.x,pos.y+10,{payload={colour='red',amount=1}})
  if h then drops[h]=true else gd.log('Drop refused',why) end
end
function on_item_collect(e)
  if e.port==1 and e.name=='gauntlet_drive' and drops[e.item] then
    drops[e.item]=nil; buffed=math.min(3,buffed+e.payload.amount)
    gd.fighter_mod(1,{damage_dealt=1+buffed*0.2,run_speed=1+buffed*0.1})
  end
end
function on_item_expire(e) drops[e.item]=nil end
function on_stage_switch(e)
  if e.phase=='after' and e.slot==boss and pending_boss then
    pending_boss=false; room=3
    -- Do not request a recursive stage switch from this callback.
    local ok,why=gd.fighter_call(2,30,gd.floor_below(30,100) or 0,{facing=-1,intangible_frames=60})
    assert(ok,why); gd.cpu_mode(2,'fight'); gd.set_percent(2,0)
    falls=gd.player(2).falls or 0; status='Room 3: KO the CPU boss'
  end
end
local function victory()
  finished=true; status=('Victory in %.2fs; best %s'):format(frames/60,best and ('%.2fs'):format(best/60) or '-')
  if not save_allowed then status=status..'; best save disabled'
  elseif not best or frames<best then
    local ok,why=gd.data_write_atomic('best.txt',tostring(frames))
    if ok then best=frames else status=status..'; save refused'; gd.log(why) end
  end
  assert(gd.post_add('shaders/victory.wgsl',{duration_frames=90,clock=true,
    params={elapsed=0,progress=0,tint={1,0.8,0.3,1}}}))
end
function on_frame()
  if room==0 and not finished then
    local age=gd.frame()-setup_start
    if building then return
    elseif age>600 then finished=true; status='Reserve setup timed out; ENTER retries'
    elseif age>0 and age%6==0 then setup() end
    return
  end
  if room==0 or finished then return end
  if camera_pending then camera_pending=false; gd.camera_bounds(true); gd.camera_follow(gd.player(1),{x=0,y=30,z=250}) end
  frames=gd.frame()-started_at
  local p=gd.player(1); if not p then return end
  if room<3 and (p.falls or 0)>falls then finished=true; status='Run failed: ENTER retries'; return end
  if room==1 then
    for h,pos in pairs(enemies) do local s=gd.enemy_state(h); if s then pos.x=s.x; pos.y=s.y end end
    if not next(enemies) then
      if wave<2 then wave_spawn()
      else room=2; gd.teleport(1,240,8); gd.stage_set_spawn(4,240,8)
        status='Room 2 route: right through two doors, drop through central floor gap, then right to GOAL' end
    end
  elseif room==2 and not pending_boss then
    -- Chunk mode: tween toward current cell; no camera tween is restarted every frame.
    local cx=math.floor((p.x-220)/130)*130+285; local cy=math.floor(p.y/104)*104+52
    local key=cx..','..cy
    if camera_cell~=key then
      camera_cell=key; gd.camera_move{to={eye={x=cx,y=cy,z=330},interest={x=cx,y=cy,z=0}},frames=45,ease='inout'}
    end
    if (p.x-goal.x)^2+(p.y-goal.y)^2<18^2 then
      -- Slots refuse any arena owner, even this script. Restore the supported host
      -- in one callback, with P1 safely inside FD, before requesting the finale.
      gd.teleport(1,0,12); assert(gd.stage_hide(false),'restore host before loading boss slot')
      gd.stage_restore_bounds(); gd.camera_bounds(false); gd.camera_attach(0)
      for _,name in ipairs(areas) do gd.area_unload(name) end; areas={}; visible_lines={}
      for h in pairs(drops) do gd.item_despawn(h) end; drops={}
      -- A loaded slot makes stage_hide(false) refuse, so the finale slot is loaded only now,
      -- after the arena is gone; P2 stays benched until the transition completes.
      local slot,slot_why=gd.stage_slot_load('battlefield'); boss=slot
      local ok,why=false,slot_why; if slot then ok,why=gd.stage_switch(boss,{transition='wipe',indicator=0,place='nearest'}) end
      if ok then pending_boss=true; status='Boss room transition'
      else finished=true; status='Transition refused: '..tostring(why)..'; ENTER retries' end
    end
  elseif room==3 then
    local b=gd.player(2)
    if b and (b.falls or 0)>falls then victory() end
  end
end
function on_tick()
  if pressed('ENTER') and finished then gd.scene_launch{mode='lab',stage='fd',p1='fox/hu',p2='marth/cpu0'} end
end
function on_draw()
  for _,l in ipairs(visible_lines) do
    local x,y=gd.project(l[1],l[2]); local u,v=gd.project(l[3],l[4])
    gd.line(x,y,u,v,0x60dfffff)
  end
  caption(status..('; buffs %d; %.2fs'):format(buffed or 0,frames/60))
  if room==2 and goal then
    local x,y,on=gd.project(goal.x,goal.y)
    if on then gd.box(x-10,y-25,20,25,0xffd369ff); gd.text(x-18,y-44,'GOAL') end
  end
  if finished and gd.kit.available() then
    local a=gd.safe_area(); gd.kit.panel(a.w/2-180,160,360,150)
    gd.kit.text(a.w/2,200,'GAUNTLET RESULT','label','gold','center')
    gd.kit.text(a.w/2,236,status,'body','bone','center',{max_w=330})
    gd.kit.text(a.w/2,270,'ENTER: RETRY','body','bone','center')
  end
end
function on_unload()
  abandon()
  gd.post_clear()
  if gd.match().active then
    for h in pairs(enemies) do gd.enemy_remove(h) end
    for h in pairs(drops) do gd.item_despawn(h) end
    if gd.fighter_benched(2) and home then gd.fighter_call(2,home.x,home.y) end
    gd.fighter_mod(1,nil); gd.camera_attach(0); gd.camera_bounds(false)
    for _,name in ipairs(areas) do gd.area_unload(name) end
    if not boss then gd.stage_hide(false) end; gd.stage_restore_bounds()
  end
  -- Engine owner cleanup restores an active destination slot and releases its DAT.
end
function on_match_end() generation=generation+1; building=false; camera_pending=false; visible_lines={}; room=0; enemies={}; drops={}; areas={}; boss=nil; pending_boss=false; camera_cell=nil; finished=false end
function on_loadstate() abandon(); finished=true; status='State load unsupported; retry with ENTER' end

-- Owner-local state for the external behavior tour; never exposes another mod's handles.
gd.command('demo_state',function(token)
  gd.log('tour state '..token..' '..tostring('room='..room..' building='..tostring(building))..' | '..status)
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
