local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();local C=D.companion
T.test('full companion and four stats',function()
  local c=C.new();C.validate(c);assert(c.age==0 and c.type=='young')
  for _,k in ipairs(C.stats) do assert(c.stats[k].points==0 and c.stats[k].level==0 and c.stats[k].grade=='C') end
end)
T.test('every grade gains according to table and level follows diminishing curve',function()
  for grade,mul in pairs(C.tuning.grade_gain) do
    local c=C.new();c.stats.power.grade=grade;c.stats.power.base_grade=grade;c.dna.power={grade,grade}
    assert(C.feed(c,'red',200)==math.floor(200*mul));assert(c.stats.power.level==C.level(math.floor(200*mul)))
    C.validate(c)
  end
end)
T.test('colours caps and effects are bounded in one table',function()
  local c=C.new();for _,colour in ipairs({'red','green','blue','yellow'}) do C.feed(c,colour,100000) end
  for _,k in ipairs(C.stats) do assert(c.stats[k].points==C.tuning.point_cap and c.stats[k].level==99) end
  local e=C.effects(c);assert(e.damage_dealt<=1.2 and e.speed<=1.20 and e.damage_taken>=.8)
  assert(e.pickup_radius<=35 and e.drop_chance<=1)
  assert(C.feed(c,'red',20)==0)
end)
T.test('white upgrades this life without touching DNA',function()
  local c=C.new();C.feed(c,'white',1);assert(c.white_drives==1 and c.stats.power.grade=='B' and c.dna.power[1]=='C')
end)
T.test('bad input cannot partially feed',function()
  local c=C.new();for _,n in ipairs({-1,.5,0/0,math.huge}) do T.refuses(function() C.feed(c,'red',n) end) end
  T.refuses(function() C.feed(c,'purple',1) end);assert(c.stats.power.points==0)
  c.stats.power.level=2;T.refuses(function() C.validate(c) end)
end)
T.done()
