-- Original procedural effects lab. All timing and tweens use game frames.
local names={"Solar Eruption","Glacial Shatter"}
local roles={"All layers","Charge","Core / shell","Flames / shards","Shock ring","Fragments","Smoke / mist","Distortion"}
local keys={"palette","energy","turbulence","cohesion","rhythm","persistence","structure"}
-- These prototype controls are narrower than the semantic traits in EFFECTS-LAB-PLAN.md.
local control_labels={"Hot/cool light","Energy","Distortion","Particle size","Density","Lifetime","Layer balance"}
local values={.5,.5,.5,.5,.5,.5,.5}
local a,b,blend,layer=1,2,0,0
local handle,ready,requested,mouse_prev=0,false,false,0
local launch_seen=false
local active_shock=false
local was_alive=false
local loop,close,seed=false,true,17029
local shock_at=-1000
local frame_ms=0
local status="Loading offline lab"
local config={}
local c=gd.data_read("config.lua");if c then config=assert(load(c,"effects-config","t"))() end
loop=config.loop==true
local function expression(i)
  local t=blend*blend*(3-2*blend)
  if a==b then t=a==1 and 0 or 1 elseif a==2 then t=1-t end
  local midpoint=1+.6*4*t*(1-t)
  local role=(i-1)%7+1
  local weight=i<=7 and (1-t) or t
  if layer~=0 and layer~=role then weight=0 end
  local structure=(role==2 or role==4) and (.35+1.3*values[7]) or
    ((role==3 or role==5 or role==6) and (1.65-1.3*values[7]) or 1)
  local light=.75+.5*(i<=7 and values[1] or 1-values[1])
  return {opacity=weight,rate=weight*structure*((role==1 or role==3 or role==5 or role==6) and (.6+.4*values[5]) or 1),speed=.55+.9*values[2],
    life=.6+.8*values[6],size=.75+.5*values[4],brightness=(.65+.7*values[2])*light*midpoint,turbulence=.3+1.4*values[3]}
end
local play
local function update(frames)
  if handle==0 then return end
  -- Edits after a finite preview ends must remain visible. A reaction uses a
  -- different emitter layout, so return to the ordinary preview before editing.
  if frames~=0 and (active_shock or not gd.fx_instance(handle).alive) then play(false);return end
  for i=1,14 do gd.fx_control(handle,i-1,expression(i),frames or 12) end
end
play=function(shock)
  gd.resume()
  active_shock=shock or false
  if handle>0 then gd.fx_end(handle,shock and 10 or 0) end
  handle=gd.fx_play(shock and "ThermalShock" or "ThermalBlend",1,0,0,10,0,1,seed)
  if handle==0 then status="Package unavailable or emitter capacity reached";return end
  was_alive=true
  if not shock then update(0) end
  status=shock and "Thermal Shock: fracture > steam > hot fragments" or "Preview running; edits tween over 12 game frames"
end
local function camera()
  if close then local p=gd.player(1);gd.camera_detach()
    -- Reserve the left side for sliders and the bottom for playback controls.
    gd.camera_set{eye={x=p.x+2,y=p.y+16,z=105},interest={x=p.x-10,y=p.y+6,z=0},fov=30}
  else gd.camera_attach(0) end
end
function on_frame()
  gd.input(4,{},1)
  if not requested then requested=true;gd.scene_launch{mode="training",p1="falco",p2="fox/cpu",stage="fd"};return end
  local match=gd.match()
  if not match.active then launch_seen=true;return end
  if match.frame<=90 then launch_seen=true end
  if not ready and launch_seen and match.frame>90 then
    ready=true;gd.teleport(1,0,2);gd.teleport(2,100,2);camera();play(false)
  end
  if ready then
    local q=gd.fx_instance(handle)
    if not q.alive then
      if loop then play(false)
      elseif was_alive then was_alive=false;status="Preview finished; edit a slider or press Replay" end
    end
  end
end
local function inside(mx,my,x,y,w,h) return mx>=x and mx<=x+w and my>=y and my<=y+h end
local function save()
  local fields={"return {schema=1,recipe=2,seed="..seed..",a="..a..",b="..b..",blend="..blend..",layer="..layer..",values={"}
  for i,v in ipairs(values) do fields[#fields+1]=string.format("%.9g,",v) end
  fields[#fields+1]="}}\n";gd.data_write("favourite.lua",table.concat(fields));status="Saved favourite with seed and recipe version 2"
end
local function load_saved()
  local raw=gd.data_read("favourite.lua");if not raw then status="No favourite saved";return end
  local ok,v=pcall(function()return assert(load(raw,"favourite","t",{}))()end)
  if not ok or type(v)~="table" or v.recipe~=2 or v.schema~=1 or type(v.values)~="table" or #v.values~=7 then status="Unsupported saved recipe";return end
  for _,n in ipairs(v.values) do if type(n)~="number" or n~=n or n<0 or n>1 then status="Invalid saved trait";return end end
  if (v.a~=1 and v.a~=2) or (v.b~=1 and v.b~=2) or type(v.blend)~="number" or v.blend~=v.blend or v.blend<0 or v.blend>1 or type(v.layer)~="number" or v.layer%1~=0 or v.layer<0 or v.layer>7 or type(v.seed)~="number" or v.seed%1~=0 or v.seed<0 or v.seed>2147483647 then status="Invalid saved recipe";return end
  a,b,blend,layer,seed,values=v.a,v.b,v.blend,v.layer,v.seed,v.values;play(false)
end
local smoke_case,smoke_frame,smoke_peak=0,0,0
local captured={}
local smoke_phases={18,44,90}
local smoke_cost,smoke_samples=0,0
local smoke_refused,smoke_refused_emitters,smoke_failed=0,0,false
local cases={{0,false,true},{.25,false,true},{.5,false,true},{.75,false,true},{1,false,true},{.5,true,true},{0,false,false},{1,false,false},{.5,false,false}}
local function smoke(q)
  if not config.validate then return end
  smoke_frame=smoke_frame+1
  if smoke_frame==1 then local v=cases[smoke_case+1];blend=v[1];close=v[3];camera();play(v[2]);captured={};smoke_phases=v[2] and {4,18,60} or {18,44,90};q=gd.fx_instance(handle) end
  smoke_peak=math.max(smoke_peak,q.particles)
  smoke_refused=math.max(smoke_refused,q.refused)
  smoke_refused_emitters=math.max(smoke_refused_emitters,q.refused_emitters)
  for _,phase in ipairs(smoke_phases) do if q.alive and q.age>=phase and not captured[phase] then
    gd.screenshot(string.format("effects-%02d-f%03d.png",smoke_case,phase));captured[phase]=true
  end end
  local perf=gd.perf(1);if #perf.frames>0 then smoke_cost=smoke_cost+perf.frames[1].total_ms;smoke_samples=smoke_samples+1 end
  if smoke_frame>=150 then
    local passed=smoke_peak>0 and not q.alive and q.particles==0 and q.emitters==0 and smoke_refused==0 and smoke_refused_emitters==0
    for _,phase in ipairs(smoke_phases) do passed=passed and captured[phase]==true end
    smoke_failed=smoke_failed or not passed
    gd.data_write(string.format("effects-%02d.tsv",smoke_case),string.format("case\tpeak\tlive\temitters\trefused\trefused_emitters\tpassed\n%d\t%d\t%d\t%d\t%d\t%d\t%s\n",smoke_case,smoke_peak,q.particles,q.emitters,smoke_refused,smoke_refused_emitters,tostring(passed)))
    gd.data_write(string.format("effects-%02d-cost.txt",smoke_case),string.format("total frame mean_ms=%.6f samples=%d (includes gameplay and overlay)\n",smoke_cost/math.max(smoke_samples,1),smoke_samples))
    smoke_case=smoke_case+1;smoke_frame=0;smoke_peak=0;smoke_cost=0;smoke_samples=0;smoke_refused=0;smoke_refused_emitters=0
    if smoke_case>=#cases then gd.data_write("validation-done.txt",(smoke_failed and "FAILED" or "complete").." recipe=2 run="..tostring(config.run_id).."\n");gd.quit() end
  end
end
function on_tick()
  local mx,my,buttons=gd.mouse();local left=buttons%2==1;local pressed=left and mouse_prev%2==0;mouse_prev=buttons
  if not ready then return end
  local q=gd.fx_instance(handle)
  local perf=gd.perf(1);if #perf.frames>0 then frame_ms=perf.frames[1].total_ms end
  -- Validation advances only when the game frame advances, including pause/step behavior.
  if config.validate then return end
  local changed=false
  for i=0,7 do local y=155+i*27
    if left and inside(mx,my,20,y-6,225,20) then local v=math.max(0,math.min(1,(mx-24)/215))
      if i==0 then blend=v else values[i]=v end;changed=true
    end
  end
  if changed then update(12) end
  if pressed then
    if inside(mx,my,20,78,225,25) then a=3-a;update()
    elseif inside(mx,my,20,109,225,25) then b=3-b;update()
    elseif inside(mx,my,20,376,225,25) then layer=(layer+1)%8;update()
    elseif inside(mx,my,282,345,100,28) then play(false)
    elseif inside(mx,my,390,345,100,28) then loop=not loop
    elseif inside(mx,my,498,345,120,28) then gd.fx_end(handle,12);loop=false;status="Fading to stop"
    elseif inside(mx,my,282,379,100,28) then if gd.paused() then gd.resume() else gd.pause() end
    elseif inside(mx,my,390,379,100,28) then gd.step(1)
    elseif inside(mx,my,498,379,120,28) then
      if gd.match().frame-shock_at>=90 then shock_at=gd.match().frame;play(true);loop=false else status="Thermal Shock cooling down (90 game frames)" end
    elseif inside(mx,my,282,413,100,28) then save()
    elseif inside(mx,my,390,413,100,28) then load_saved()
    elseif inside(mx,my,498,413,120,28) then close=not close;camera()
    end
  end
end
local previous_frame=-1
local original_frame=on_frame
function on_frame()
  original_frame()
  if ready and config.validate and gd.match().frame~=previous_frame then previous_frame=gd.match().frame;smoke(gd.fx_instance(handle)) end
end
local function text(x,y,t,role,color)gd.kit.text(x,y,t,role or "caption",color or "bone")end
local function button(x,y,w,t)gd.fill(x,y,w,27,0x23384BFF);text(x+7,y+19,t)end
function on_draw()
  if not ready then return end
  gd.fill(12,42,246,372,0x080D19EA);text(22,64,"EFFECTS PROTOTYPE","label","gold")
  button(20,78,225,"A: "..names[a]);button(20,109,225,"B: "..names[b])
  for i=0,7 do local y=155+i*27;local v=i==0 and blend or values[i]
    text(22,y-3,(i==0 and "Founder blend" or control_labels[i])..string.format("  %.0f%%",v*100))
    gd.fill(24,y+5,215,4,0x617084FF);gd.fill(24,y+5,math.floor(v*215),4,0x36C9D8FF);gd.fill(21+math.floor(v*215),y+1,6,12,0xF0B429FF)
  end
  button(20,376,225,roles[layer+1])
  gd.fill(274,42,352,90,0x080D19EA)
  gd.fill(274,337,352,111,0x080D19EA)
  local q=gd.fx_instance(handle);local phase=not q.alive and "FINISHED" or q.age<(active_shock and 8 or 36) and "CHARGE" or q.age<(active_shock and 50 or 78) and "RELEASE" or "DECAY"
  text(284,64,phase..(q.alive and (" / "..q.age.." frames") or ""),"label","gold")
  text(284,87,string.format("Particles %d  Emitters %d  Seed %d",q.particles,q.emitters,seed))
  text(284,110,string.format("Refused %d / emitters %d | %.2f ms",q.refused,q.refused_emitters,frame_ms))
  button(282,345,100,"REPLAY");button(390,345,100,loop and "LOOP ON" or "LOOP OFF");button(498,345,120,"STOP")
  button(282,379,100,gd.paused() and "RESUME" or "PAUSE");button(390,379,100,"STEP");button(498,379,120,"THERMAL SHOCK")
  button(282,413,100,"SAVE");button(390,413,100,"LOAD");button(498,413,120,close and "GAME CAMERA" or "CLOSE CAMERA")
  gd.fill(12,453,614,25,0x080D19EA);text(22,471,status)
end
function on_match_end()gd.fx_stop();ready=false end
function on_unload()gd.fx_stop();gd.resume();gd.camera_attach(0)end
