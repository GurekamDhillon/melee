-- @name: Fighter capabilities rewind exactness fixture
-- @gameplay: true
-- Standalone offline LAB fixture; do not load alongside a gameplay director/demo.
-- All options coexist in the snapshot. Advance 120 frames past 30-frame expiry,
-- then require byte-exact restoration of that snapshot, including timer state.
-- Scope: existing LAB byte comparison plus explicit capability/timer readbacks.
-- Armour and capability values are checked through native getters after restore.
-- Does not establish jump reset, restrictions/CPU behavior, collision armour,
-- item safety or target filtering acceptance; those require actual gameplay.
-- This file has only been syntax checked, never executed in the game.
local started,finished=false,false
local expired=false
function on_frame()
  if finished or not gd.match().active then return end
  if not started then
    assert(gd.player(1),'P1 required')
    assert(gd.fighter_caps(1,{air_jumps=8,shield=true,air_dodge=true,
      run=true,grab=true,specials=true}),'capabilities refused')
    assert(gd.fighter_armour(1,{damage=12,knockback=60}),'armour refused')
    for _,kind in ipairs({'intangible','invincible','metal','size'}) do
      assert(gd.fighter_effect(1,kind,kind=='size' and 1.5 or 1,30),kind..' refused')
      local e=assert(gd.fighter_effect(1,kind),'effect readback unavailable')
      assert(e.frames>0,'effect timer missing')
    end
    for channel=1,4 do
      assert(gd.fighter_timed_status(1,channel,channel*7,30),'status refused')
      local s=assert(gd.fighter_timed_status(1,channel),'status readback unavailable')
      assert(s.frames>0 and s.value==channel*7,'status timer/value missing')
    end
    -- It_Kind_Bat = 0x0B, src/melee/it/forward.h; require loaded retail item data.
    assert(gd.give_item(1,11),'bat refused; begin with empty hands and enabled/loaded bat data')
    local opponent=assert(gd.nearest_opponent(1),'opponent required')
    local ids=assert(gd.opponents_in_radius(1,10000),'radius query unavailable')
    local found=false
    for _,id in ipairs(ids) do if id==opponent then found=true end end
    assert(found,'nearest opponent missing from broad radius')
    assert(gd.history(300,1),'history refused')
    assert(gd.rewind_test(120,true),'rewind test refused')
    started=true
  else
    local r=gd.rewind_test_result()
    if r and r.phase==2 then
      -- Observe native expiration on the forward run, before the test rewinds.
      local live=false
      for _,kind in ipairs({'intangible','invincible','metal','size'}) do
        local e=gd.fighter_effect(1,kind); if e and e.frames>0 then live=true end
      end
      for channel=1,4 do
        local v=gd.fighter_timed_status(1,channel); if v and v.frames>0 then live=true end
      end
      if not live then expired=true end
    end
    if r and r.phase==0 and r.pass~=nil then
      finished=true
      assert(expired,'forward run did not observe timer expiry')
      -- The native field name is diff (number of differing bytes).
      assert(r.pass and r.diff==0,'fighter capabilities rewind differs: '..tostring(r.diff)..' '..tostring(r.text))
      local c=assert(gd.fighter_caps(1),'restored capabilities unavailable')
      assert(c.air_jumps==8 and c.shield==true and c.air_dodge==true
        and c.run==true and c.grab==true and c.specials==true,'capabilities lost on restore')
      local a=assert(gd.fighter_armour(1),'restored armour unavailable')
      assert(a.damage==12 and a.knockback==60,'armour lost on restore')
      for _,kind in ipairs({'intangible','invincible','metal','size'}) do
        local e=assert(gd.fighter_effect(1,kind),'restored effect unavailable')
        assert(e.frames>0,'effect timer not restored: '..kind)
      end
      for channel=1,4 do
        local s=assert(gd.fighter_timed_status(1,channel),'restored status unavailable')
        assert(s.frames>0 and s.value==channel*7,'status not restored')
      end
      gd.log('fighter capabilities rewind PASS: 0 differing bytes')
    end
  end
end
