-- @name: Typed armour timer rewind fixture
-- @gameplay: true
-- Standalone offline LAB; no gameplay director or other capability demo.
-- Unexecuted native tester: byte comparison plus restored typed row readbacks.
-- Does not test collision formula, hit budgets or event correctness; native suite
-- and the interactive collision demo cover those separately when actually run.
local types={'knockback','damage_threshold','knockback_threshold','super','hit_count','damage_pool'}
local values={30,8,60,1,3,30}
local started,finished,expired=false,false,false
function on_frame()
  if finished or not gd.match().active then return end
  if not started then
    local p=assert(gd.player(1),'P1 required')
    if p.cpu then gd.cpu_mode(1,'stand') end
    if gd.player(2) then gd.cpu_mode(2,'stand') end
    assert(gd.fighter_armor(1,nil),'clear refused')
    for i,kind in ipairs(types) do
      assert(gd.fighter_armor(1,{type=kind,value=values[i],frames=30,direction='any'}),'typed armour refused: '..kind)
    end
    local rows=assert(gd.fighter_armor(1),'typed read unavailable')
    assert(#rows==6,'not all types coexist')
    assert(gd.history(300,1),'history refused')
    assert(gd.rewind_test(120,true),'rewind refused')
    started=true
  else
    local r=gd.rewind_test_result()
    if r and r.phase==2 then
      local rows=gd.fighter_armor(1) or {}
      if #rows==0 then expired=true end
    end
    if r and r.phase==0 and r.pass~=nil then
      finished=true
      assert(expired,'forward run did not observe window expiry')
      assert(r.pass and r.diff_compared==0,'typed armour rewind differs: '..tostring(r.diff)..' '..tostring(r.text))
      local rows=assert(gd.fighter_armor(1),'restored typed read unavailable')
      assert(#rows==6,'typed armour lost on restore')
      local found={}
      for _,a in ipairs(rows) do
        assert(a.frames>0,'timer not restored: '..tostring(a.type))
        found[a.type]=a
      end
      for i,kind in ipairs(types) do
        local a=assert(found[kind],'missing restored type: '..kind)
        assert(a.value==values[i],'value lost: '..kind)
        if kind=='hit_count' or kind=='damage_pool' then
          assert(a.remaining==values[i],'budget lost: '..kind)
        end
      end
      gd.log('typed armour rewind PASS: 0 differing bytes')
    end
  end
end
