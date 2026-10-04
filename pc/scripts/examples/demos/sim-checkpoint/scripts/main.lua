-- One feature: a Lua clock and absolute gameplay outputs survive LAB rewind.
local clock=0
function on_loadstate()
  clock=tonumber(gd.sim_read() or '0') or 0
end
function on_scene() clock=0 end
function on_frame()
  if gd.sim_replaying() or not gd.match().active then return end
  clock=clock+1
  local ops={}
  if gd.player(1) then
    ops[#ops+1]={op='fighter_mod',port=1,values={run_speed=1+(clock%120)/240}}
    ops[#ops+1]={op='damage',port=1,value=clock%100}
  end
  gd.sim_commit(tostring(clock),ops)
end
function on_draw()
  local a=gd.safe_area()
  gd.text(a.x+12,a.y+12,'Checkpoint clock '..clock..': P1 percent and speed follow this clock',0xffffffff)
end

gd.command('demo_state',function(token)
  local player,values=gd.player(1),gd.fighter_mod(1)
  local expected=1+(clock%120)/240
  local matches=tonumber(gd.sim_read() or '')==clock and player and values
    and math.abs(player.percent-clock%100)<.001 and math.abs(values.run_speed-expected)<.00001
  gd.log('tour state '..tostring(token)..' checkpoint '..clock..' | matches='..tostring(not not matches))
end,'demo_state <token>: compare checkpoint clock to real percent/speed')
