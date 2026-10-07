-- @name: Zone rewind exactness fixture
-- @gameplay: true
-- Run in an ordinary offline match, without stage slots or a Lua director.
local started,finished=false,false
local enter,exit=0,0
function on_zone_enter() enter=enter+1 end
function on_zone_exit() exit=exit+1 end
function on_frame()
  if finished or not gd.match().active then return end
  if not started then
    local p=assert(gd.player(1),'P1 required')
    gd.zone_add{name='rewind-room',kind='room',x0=p.x-1000,y0=p.y-1000,x1=p.x+1000,y1=p.y+1000}
    gd.zone_add{name='rewind-door',kind='transition',x0=p.x-20,y0=p.y-1000,x1=p.x+20,y1=p.y+1000}
    assert(gd.history(300))
    assert(gd.rewind_test(60,true))
    started=true
  else
    local r=gd.rewind_test_result()
    if r.phase==0 and r.pass~=nil then
      finished=true
      assert(r.pass and r.diff_compared==0,'zone rewind differs: '..tostring(r.diff)..' '..r.text)
      assert(#gd.zones()==2,'zone definitions lost on restore')
      assert(#gd.zones_at(1)>=1,'membership lost on restore')
      gd.log('zones rewind PASS: 0 differing bytes',enter,exit)
    end
  end
end
