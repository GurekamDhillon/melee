-- @name: six_slot_recycle_visibility
-- @gameplay: true
-- @rollback_safe: false
-- Native fixture: launch an offline six-fighter LAB match before loading.
local finished,ticks=false,0
function on_tick()
  ticks=ticks+1
  if not finished and ticks>5000 then
    finished=true;gd.log('TEST six_slot_recycle_visibility FAIL watchdog');gd.quit()
  end
end
gd.run(function()
  local ok,why=pcall(function()
    assert(gd.wait_until(function() return gd.match().active and gd.match().frame>90 end,3000),'match timeout')
    local p=assert(gd.player(6),'requires six-fighter LAB match with CPU P6')
    local kind=p.char
    for cycle=1,3 do
      local falls=gd.player(6).falls
      gd.teleport(6,0,assert(gd.stage_bounds().blast).bottom-100)
      assert(gd.wait_until(function() return gd.player(6).falls>falls end,300),'KO timeout')
      local accepted,reason=false,nil
      assert(gd.wait_until(function()
        accepted,reason=gd.fighter_recycle(6,{x=35,y=0,facing=-1,intangible_frames=60})
        return accepted
      end,300),'recycle refused: '..tostring(reason))
      gd.wait(2)
      local benched,entities=gd.fighter_benched(6)
      local primary=assert(entities[1],'missing primary record')
      assert(primary.present and primary.entity_index==0,'missing primary fighter')
      assert(not benched and not primary.benched and not primary.dormant,'recycled fighter excluded')
      assert(not primary.invisible and not primary.frozen,'recycled fighter hidden/frozen')
      local q=assert(gd.player(6));assert(q.char==kind and q.percent==0,'character/damage reset failed')
      gd.log('TEST six_slot_recycle_visibility cycle',cycle,'PASS')
      gd.wait(15)
    end
  end)
  finished=true
  gd.log('TEST six_slot_recycle_visibility '..(ok and 'PASS' or 'FAIL '..tostring(why)))
  gd.quit()
end)
