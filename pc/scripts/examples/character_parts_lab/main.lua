-- Offline character-part analysis and live color/FX review. See tools/model_parts/README.md.
local config = {fighters={"falco"}, batch=false, autoscan=true, quick=false, assets={}}
local config_text = gd.data_read("config.lua")
if config_text then local fn = assert(load(config_text, "parts-config", "t")); config = fn() end
local function json(v)
  local t=type(v)
  if t=="nil" then return "null" end
  if t=="boolean" then return tostring(v) end
  if t=="number" then return (v==v and math.abs(v)<1e100) and string.format("%.9g",v) or "null" end
  if t=="string" then return '"'..v:gsub('[%z\1-\31\\"]',function(c)
    if c=='"' or c=='\\' then return '\\'..c end
    return string.format('\\u%04x',string.byte(c)) end)..'"' end
  local out={}
  if #v>0 then for i=1,#v do out[#out+1]=json(v[i]) end; return '['..table.concat(out,',')..']' end
  for k,x in pairs(v) do out[#out+1]=json(tostring(k))..':'..json(x) end
  table.sort(out); return '{'..table.concat(out,',')..'}'
end
local function hsv(h,s,v)
  local i=math.floor((h%1)*6); local f=(h%1)*6-i
  local p,q,t=v*(1-s),v*(1-f*s),v*(1-(1-f)*s)
  local a={{v,t,p},{q,v,p},{p,v,t},{p,q,v},{t,p,v},{v,p,q}}
  local c=a[i%6+1];return gd.rgb(math.floor(c[1]*255),math.floor(c[2]*255),math.floor(c[3]*255))
end
local labels={head="Head region",torso="Torso region",left_arm="Left arm",right_arm="Right arm",
  left_hand="Left hand region",right_hand="Right hand region",left_leg="Left leg",right_leg="Right leg",
  left_foot="Left foot",right_foot="Right foot",equipment="Owned equipment",unclassified="Unclassified surface"}
local function part_label(p)
  local best,weight="unclassified",0
  for r,w in pairs(p.regions) do if w>weight then best,weight=r,w end end
  local bounds=p.bounds;local ext={bounds[4]-bounds[1],bounds[5]-bounds[2],bounds[6]-bounds[3]};table.sort(ext)
  if p.source==0 and p.joint>0 and p.body_joint==0 then return "Attachment candidate",math.min(weight,0.6),true end
  local candidate=p.source==1 or ((best=="left_hand" or best=="right_hand") and ext[3]>4*math.max(ext[2],0.01))
  if candidate then return p.source==1 and ("Owned equipment "..p.item_kind) or "Equipment candidate",weight,true end
  if weight<0.7 then return "Mixed body regions",weight,false end
  return labels[best] or best,weight,false
end
local presets={"GoldEmbers","FrostDrift","ElectricSparks","CrimsonSpiral","EmeraldRings","HeatShimmer","ShadowPulse"}
local preset_names={"Gold embers","Frost drift","Electric sparks","Crimson spiral","Emerald rings","Heat shimmer","Shadow pulse"}
local tick,ready,index,requested=0,false,1,false
local launch_seen=false
local parts,controls,colors={}, {}, {}
local raw,selected,highlight,rainbow=false,1,false,false
local hue,saturation,brightness=0,0.95,1
local preset,rotate,fx_tick=0,false,0
local fx_captured=false
local ready_tick=0
local color_captured=false
local mouse_prev=0
local scan=nil
local status="Loading character"
local captures={}
local controls_text=nil
local function scan_name()
  local f=config.fighters[index]
  return (config.costume or 0)==0 and f or (f..'-c'..config.costume)
end
local function reset_fx() gd.fx_stop();preset=0;rotate=false end
local function selected_members()
  if raw then return {parts[selected] and parts[selected].index or 0} end
  return controls[selected] and controls[selected].members or {}
end
local function color_part(d,color)
  local p=parts[d+1]
  if not p or p.retired then return end
  if not gd.dobj_solid(1,d,color) then
    p.retired=true;colors[d]=nil
    status="Part expired; scan again to inspect new equipment"
  end
end
local function apply_colors()
  gd.parts_clear()
  for _,p in ipairs(parts) do
    local c=colors[p.index]
    if c then color_part(p.index,hsv(c[1],c[2],c[3])) end
  end
  if highlight then for _,d in ipairs(selected_members()) do color_part(d,gd.rgb(255,230,60)) end end
end
local function raw_controls()
  controls_text=nil
  controls={}
  for _,p in ipairs(parts) do
    local name,confidence,candidate=part_label(p)
    controls[#controls+1]={name=name.." #"..p.index,confidence=confidence,members={p.index},area=p.area,candidate=candidate}
  end
  table.sort(controls,function(a,b)return a.area>b.area end)
  selected=1
end
local function read_controls()
  local text=gd.data_read(scan_name().."-controls.tsv")
  if not text or text==controls_text then return end
  local lines={};for line in text:gmatch('[^\r\n]+') do lines[#lines+1]=line end
  local signature=lines[1] and lines[1]:match('^signature\t(.+)$')
  local asset=lines[2] and lines[2]:match('^asset_sha256\t(.+)$')
  local costume=lines[3] and lines[3]:match('^costume\t(%d+)$')
  if not signature or signature~=parts.geometry_signature or
    asset~=(config.assets and config.assets[config.fighters[index]]) or
    tonumber(costume)~=(config.costume or 0) then return end
  local out={}
  for i=4,#lines do
    local name,confidence,coverage,members=lines[i]:match('^([^\t]+)\t([^\t]+)\t([^\t]+)\t(.+)$')
    if name then
      local ids={};local valid=true
      for v in members:gmatch('%d+') do
        local d=tonumber(v);local p=parts[d+1]
        if not p or p.index~=d or p.source~=0 then valid=false;break end
        ids[#ids+1]=d
      end
      if valid and #ids>0 and tonumber(confidence) and tonumber(coverage) then
        out[#out+1]={name=name,confidence=tonumber(confidence),coverage=tonumber(coverage),members=ids}
      end
    end
  end
  if #out>0 then
    controls=out;controls_text=text
    if not raw then selected=math.min(selected,#out) end
    status="Measured controls loaded"
    if highlight then apply_colors() end
  end
end
local function select(n)
  selected=math.max(1,math.min(n,raw and #parts or #controls))
  local ids=selected_members();local c=colors[ids[1]] or {0,0.95,1}
  hue,saturation,brightness=c[1],c[2],c[3]
  if highlight then apply_colors() end
end
local function show_fx(n)
  gd.parts_clear();colors={};highlight=false;rainbow=false
  gd.fx_stop();gd.resume()
  preset=(n-1)%#presets+1
  assert(gd.fx_attach(presets[preset],1,0,0,9,0,1),"FX package missing: "..presets[preset])
  fx_tick=tick;fx_captured=false;status="FX: "..preset_names[preset]
  gd.log("PARTSLAB: showcase "..presets[preset])
end
local views={{"front",math.pi/2},{"back",-math.pi/2},{"left",math.pi},{"right",0},
  {"oblique_front",math.pi/4},{"oblique_back",math.pi*3/4}}
local function camera(view)
  local lo={1e9,1e9,1e9};local hi={-1e9,-1e9,-1e9}
  for _,p in ipairs(parts) do if p.hidden==0 and p.area>0 then
    for k=1,3 do lo[k]=math.min(lo[k],p.bounds[k]);hi[k]=math.max(hi[k],p.bounds[k+3]) end
  end end
  local player=gd.player(1);if lo[1]==1e9 then lo={player.x-10,player.y, -10};hi={player.x+10,player.y+25,10} end
  local cx,cy,cz=(lo[1]+hi[1])/2,(lo[2]+hi[2])/2,(lo[3]+hi[3])/2
  local radius=math.sqrt((hi[1]-lo[1])^2+(hi[2]-lo[2])^2+(hi[3]-lo[3])^2)/2
  local distance=math.max(55,radius/math.sin(math.rad(17.5))*1.3)
  local a=view[2]
  gd.camera_set{eye={x=cx+math.cos(a)*distance,y=cy+distance*0.08,z=cz+math.sin(a)*distance},
    interest={x=cx,y=cy,z=cz},fov=35}
end
local function scene_start()
  requested=true;ready=false;launch_seen=false;status="Loading "..config.fighters[index]
  gd.resume();gd.scene_launch{mode="training",p1=config.fighters[index]..'/c'..(config.costume or 0),p2="fox/cpu",stage="fd"}
end
local function scan_start()
  reset_fx();gd.parts_clear();colors={};rainbow=false;highlight=false;gd.teleport(1,0,2);gd.teleport(2,100,2)
  local lookup={};for _,m in ipairs(gd.motion_list(1)) do lookup[m.name]=m.id end
  local desired={{"Wait",15},{"Attack11",6},{"AttackS3S",12},{"AttackS4S",16},{"GuardOn",3}}
  -- Special state entry functions can initialize character-specific data. Do not force their motion
  -- directly: the scan uses actual controller B input below for a live equipment sample.
  local poses={};for _,p in ipairs(desired) do if lookup[p[1]] then poses[#poses+1]={name=p[1],id=lookup[p[1]],frame=p[2]} end end
  if config.quick then poses={poses[1]} end
  poses[#poses+1]={name="neutral_special_input",input=true,frame=12}
  if #poses==0 then error("No supported poses") end
  gd.camera_detach()
  scan={poses=poses,pose=1,view=1,phase="pose",since=tick,records={},failures={},started=tick}
  status="Scanning "..config.fighters[index]
end
local function cleanup_scan(cancelled)
  gd.parts_id_off();gd.parts_clear();gd.fx_stop();gd.resume();gd.camera_attach(0)
  if scan then
    gd.data_write(scan_name().."-manifest.json",json({schema=1,character=scan_name(),fighter=config.fighters[index],
      run_id=config.run_id,
      asset_sha256=config.assets and config.assets[config.fighters[index]],cancelled=cancelled or false,
      samples=scan.records,failures=scan.failures,coverage_mode="opaque_geometry_paired_ids",
      limitations={"Texture alpha cutouts are measured as opaque geometry", "Only currently fighter-owned item DObjs are included; detached and non-HSD effects are not classified"}}))
  end
  scan=nil;gd.set_motion(1,14,1);gd.resume();status=cancelled and "Scan cancelled; state restored" or "Scan complete; run report tool"
  parts=gd.parts(1,true);raw_controls();read_controls()
  if config.rainbow then rainbow=true;ready_tick=tick;color_captured=false end
  gd.log("PARTSLAB: "..status.." for "..config.fighters[index])
  if config.batch and not cancelled then
    if index<#config.fighters then index=index+1;requested=false;ready=false
    else gd.data_write("batch-done.json",json({complete=true,fighters=config.fighters,run_id=config.run_id}));gd.quit() end
  elseif config.showcase and not cancelled then rotate=true;show_fx(1)
  end
end
local function scan_tick()
  local s=scan;if not s then return end
  local pose=s.poses[s.pose]
  if tick-s.started>18000 then error("Scan exceeded five-minute tick deadline") end
  if s.phase=="pose" then
    gd.parts_id_off();gd.teleport(1,0,2)
    if pose.input then
      gd.set_motion(1,14,1);gd.resume();gd.input(1,{buttons="B"},1)
    else assert(gd.set_motion(1,pose.id,pose.frame)) end
    s.phase="settle";s.since=tick;return
  end
  if s.phase=="settle" then
    if tick-s.since<(pose.input and 18 or 12) then return end
    gd.pause();parts=gd.parts(1,true);s.view=1;s.phase="view";return
  end
  if s.phase=="view" then
    camera(views[s.view]);gd.parts_id(1,false)
    -- A new vertex layout can need a cold GPU pipeline compile. Warm ID rendering before
    -- the first measured image instead of accepting a missing silhouette as invisibility.
    s.wait=s.warmed and 8 or 120;s.warmed=true
    s.phase="a";s.since=tick;return
  end
  if s.phase=="a" and tick-s.since>=s.wait then
    s.stem=string.format('%s-p%02d-v%02d',scan_name(),s.pose,s.view)
    gd.screenshot(s.stem..'-a.png');s.phase="wait_a";s.since=tick;return
  end
  if s.phase=="wait_a" and tick-s.since>=6 then
    gd.parts_id(1,true);s.phase="b";s.since=tick;return
  end
  if s.phase=="b" and tick-s.since>=8 then
    gd.screenshot(s.stem..'-b.png');s.phase="wait_b";s.since=tick;return
  end
  if s.phase=="wait_b" and tick-s.since>=6 then
    local player=gd.player(1)
    local record={character=config.fighters[index],costume=player.costume,pose=pose.name,requested_frame=pose.frame,
      motion=player.action,actual_frame=player.anim_frame,view=views[s.view][1],
      camera=gd.camera_get(),
      pose_execution=pose.input and "controller_input" or "forced_common_motion",
      parts=parts,geometry_signature=parts.geometry_signature,coverage_mode=parts.coverage_mode,
      a=s.stem..'-a.png',b=s.stem..'-b.png'}
    -- JSON array encoding deliberately does not drop signature/measurement metadata: keep it above.
    gd.data_write(s.stem..'.json',json(record));s.records[#s.records+1]=s.stem..'.json'
    gd.parts_id_off();s.view=s.view+1
    if s.view>(config.quick and 2 or #views) then
      s.pose=s.pose+1
      if s.pose>#s.poses then cleanup_scan(false);return end
      s.phase="pose"
    else s.phase="view" end
  end
end
function on_frame()
  gd.input(4,{},1)
  if not requested then scene_start();return end
  local match=gd.match()
  if not match.active then launch_seen=true;return end
  if match.frame<=90 then launch_seen=true end
  -- A scene request is asynchronous; do not inspect or teleport the outgoing match.
  if not ready and launch_seen and match.frame>90 then
    gd.teleport(1,0,2);gd.teleport(2,100,2);parts=gd.parts(1,true);raw_controls();read_controls();ready=true;ready_tick=tick;color_captured=false
    if config.rainbow then rainbow=true end
    if config.autoscan then scan_start()
    elseif config.showcase then rotate=true;show_fx(1) end
  end
end
local function button(x,y,w,h,text,mx,my,pressed)
  if pressed and mx>=x and mx<=x+w and my>=y and my<=y+h then return true end
  return false
end
local function slider(y,value,low,high,mx,my,left)
  if left and mx>=28 and mx<=245 and my>=y-6 and my<=y+16 then
    return low+math.max(0,math.min(1,(mx-30)/210))*(high-low),true
  end
  return value,false
end
function on_tick()
  tick=tick+1
  local mx,my,buttons=gd.mouse();local left=buttons%2==1;local pressed=left and mouse_prev%2==0;mouse_prev=buttons
  if not ready then return end
  if scan then
    if gd.key_pressed("Escape") or button(20,390,230,28,"CANCEL",mx,my,pressed) then cleanup_scan(true);return end
    local ok,err=pcall(scan_tick)
    if not ok then scan.failures[#scan.failures+1]=tostring(err);cleanup_scan(true);gd.log("PARTSLAB scan error: "..tostring(err)) end
    return
  end
  if rotate and tick-fx_tick>300 then show_fx(preset+1) end
  if config.showcase and preset>0 and not fx_captured and tick-fx_tick>=150 then
    gd.screenshot('showcase-'..presets[preset]..'.png');fx_captured=true
  end
  if tick%180==0 then read_controls() end
  if tick%30==0 then local p=gd.player(1);gd.camera_detach();
    gd.camera_set{eye={x=p.x+13,y=p.y+19,z=100},interest={x=p.x,y=p.y+10,z=0},fov=29} end
  if rainbow and tick%4==0 then
    for _,p in ipairs(parts) do if not p.retired then colors[p.index]={(p.index/math.max(#parts,1)+tick/480)%1,0.95,1} end end
    apply_colors()
  end
  if config.rainbow and rainbow and not color_captured and tick-ready_tick>=150 then
    gd.screenshot('rainbow-'..scan_name()..'.png');color_captured=true
  end
  local v,changed=slider(145,selected,1,raw and #parts or #controls,mx,my,left)
  if changed then select(math.floor(v+0.5)) end
  local changed_color=false
  hue,changed=slider(190,hue,0,1,mx,my,left);changed_color=changed_color or changed
  saturation,changed=slider(235,saturation,0,1,mx,my,left);changed_color=changed_color or changed
  brightness,changed=slider(280,brightness,0.05,1,mx,my,left);changed_color=changed_color or changed
  if changed_color then
    rainbow=false;highlight=false;reset_fx()
    for _,d in ipairs(selected_members()) do colors[d]={hue,saturation,brightness} end;apply_colors()
  end
  if button(20,318,110,26,"RAW",mx,my,pressed) then raw=not raw;select(1)
  elseif button(138,318,110,26,"HIGHLIGHT",mx,my,pressed) then highlight=not highlight;reset_fx();apply_colors()
  elseif button(20,350,110,26,"RAINBOW",mx,my,pressed) then reset_fx();rainbow=not rainbow
  elseif button(138,350,110,26,"RESTORE",mx,my,pressed) then rainbow=false;highlight=false;colors={};gd.parts_clear();reset_fx()
  elseif button(20,382,228,26,"SCAN",mx,my,pressed) then scan_start()
  elseif button(280,392,95,28,"PREV",mx,my,pressed) then show_fx(preset-1)
  elseif button(385,392,95,28,"NEXT",mx,my,pressed) then show_fx(preset+1)
  elseif button(490,392,130,28,"ROTATE",mx,my,pressed) then rotate=not rotate;if rotate then show_fx(preset+1) end end
end
local function text(x,y,t,role,color)gd.kit.text(x,y,t,role or "caption",color or "bone")end
local function draw_slider(y,label,value,low,high)
  local t=high>low and (value-low)/(high-low) or 0
  text(24,y-5,label);text(200,y-5,string.format('%.0f',value))
  gd.fill(30,y+7,210,5,0x617084FF);gd.fill(30,y+7,math.floor(210*t),5,0x36C9D8FF)
  gd.fill(26+math.floor(210*t),y+3,8,13,0xF0B429FF)
end
local function draw_button(x,y,w,t)gd.fill(x,y,w,26,0x23384BFF);text(x+9,y+18,t)end
function on_draw()
  if not ready then return end
  gd.fill(12,65,250,351,0x080D19E8)
  text(24,87,string.upper(config.fighters[index]).." / PART LAB","label","gold")
  if scan then
    text(24,121,"MEASURING VISIBLE PARTS","label","gold")
    text(24,150,"Pose "..scan.pose.." / "..#scan.poses)
    text(24,177,"View "..scan.view.." / "..(config.quick and 2 or #views))
    text(24,205,"Two ID captures per view")
    text(24,236,"Equipment remains separate")
    text(24,267,"Texture cutouts: approximate")
    draw_button(20,390,230,"CANCEL / ESC");return
  end
  local entry=raw and parts[selected] or controls[selected]
  local name=raw and entry and part_label(entry) or entry and entry.name or "No entries"
  text(24,111,tostring(name):sub(1,31),"caption","gold")
  text(24,129,raw and ("Raw DObj "..(entry and entry.index or 0)..(entry and entry.retired and " (expired)" or "")) or string.format('Confidence %.0f%%',100*(entry and entry.confidence or 0)))
  draw_slider(145,raw and "Raw part" or "Ranked control",selected,1,raw and #parts or #controls)
  draw_slider(190,"Hue",hue*100,0,100);draw_slider(235,"Saturation",saturation*100,0,100)
  draw_slider(280,"Brightness",brightness*100,5,100)
  draw_button(20,318,110,raw and "GROUPS" or "RAW PARTS");draw_button(138,318,110,highlight and "PREVIEW OFF" or "HIGHLIGHT")
  draw_button(20,350,110,rainbow and "STOP COLORS" or "RAINBOW");draw_button(138,350,110,"RESTORE ALL")
  draw_button(20,382,228,"SCAN CHARACTER")
  gd.fill(278,354,344,70,0x080D19E8)
  text(290,377,preset>0 and preset_names[preset] or "SHADER SHOWCASE","label","gold")
  draw_button(280,392,95,"PREVIOUS");draw_button(385,392,95,"NEXT FX");draw_button(490,392,130,rotate and "STOP ROTATE" or "ROTATE FX")
  gd.fill(12,434,614,30,0x080D19E8);text(24,455,status.." | neutral pad P4")
end
function on_match_end()
  gd.parts_clear();gd.fx_stop();scan=nil;ready=false
end
function on_unload() gd.parts_clear();gd.fx_stop();gd.resume();gd.camera_attach(0) end
