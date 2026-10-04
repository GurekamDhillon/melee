-- One feature: per-frame surface uniform updates retain one shader selection.
local selected,frame,updates=false,0,0
gd.command("demo_state",function(token) local perf=gd.perf();gd.log("tour state "..tostring(token).." selected="..tostring(selected).." updates="..updates.." loads="..tostring(perf.surface_load_calls)) end)
local note='Start an offline LAB match; P1 rim changes gently without shader reloads.'
local function clear() if selected then gd.fighter_shader(1,nil) end;selected=false end
function on_frame()
 if not gd.match().active or not gd.player(1) then clear();return end
 if not selected then local ok,why=gd.fighter_shader(1,'shaders/rim.wgsl',{params={.1,.6,1,.15,2}})
  if not ok then note=tostring(why);return end;selected=true
 end
 frame=frame+1
 local ok,why=gd.fighter_shader_set(1,{params={.1,.6,1,.1+.05*math.sin(frame/60),2}})
 if not ok then note=tostring(why) else updates=updates+1 end
end
function on_draw()
 local a=gd.safe_area();gd.fill(a.x+12,a.y+12,math.min(590,a.w-24),70,0x16202AE0)
 gd.text(a.x+22,a.y+22,'Surface parameter updates',0xEBD175FF,1.1)
 gd.text(a.x+22,a.y+42,note,0xE8EEF4FF,1)
 local perf=gd.perf();gd.text(a.x+22,a.y+60,'Surface loads: '..tostring(perf.surface_load_calls),0xE8EEF4FF,1)
end
function on_match_end() clear() end
function on_scene() clear() end
function on_unload() clear() end
