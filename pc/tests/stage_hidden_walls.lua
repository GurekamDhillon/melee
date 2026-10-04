-- @name: stage_hidden_walls
-- @gameplay: true
-- @rollback_safe: false
local ticks,done=0,false
function on_tick()ticks=ticks+1;if not done and ticks>4000 then gd.log('TEST stage_hidden_walls FAIL watchdog');gd.quit()end end
gd.run(function()
 local ok,why=pcall(function()
  assert(gd.wait_until(function()return gd.match().active and gd.match().frame>90 end,3000),'match timeout')
  local handles={}
  for _,line in ipairs({{-10,0,-10,48,'left_wall'},{10,48,10,0,'right_wall'},{10,48,-10,48,'ceiling'}})do
   local h=assert(gd.stage_add_line(line[1],line[2],line[3],line[4],line[5],{draw=false,passthrough=false,ledges=false}));handles[#handles+1]=h
   assert(not pcall(gd.stage_add_line,line[1],line[2],line[3],line[4],line[5],{passthrough=true}),'wall accepted passthrough')
   assert(not pcall(gd.stage_add_line,line[1],line[2],line[3],line[4],line[5],{ledges=true}),'wall accepted ledges')
  end
  for _,h in ipairs(handles)do assert(gd.stage_remove(h)~=false,'release refused')end
 end)
 done=true;gd.log('TEST stage_hidden_walls '..(ok and 'PASS' or 'FAIL '..tostring(why)));gd.quit()
end)
