local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();local C=D.companion
T.test('yellow grows Jump and changes native air mobility without ground speed or collection changes',function()
 local c=C.new();assert(c.stats.jump and not c.stats.reach)
 local before=C.effects(c);C.feed(c,'yellow',20);local e=C.effects(c)
 assert(c.stats.jump.level==1 and e.air_speed>before.air_speed and e.speed==before.speed)
 assert(e.pickup_radius==before.pickup_radius and e.drop_chance==before.drop_chance)
 local fields;local g={fighter_mod=function(_,v) fields=v;return true end,log=function() end}
 local f=T.module('fighter',D).new(g);f:apply(c)
 assert(fields.air_speed==e.air_speed and fields.run_speed==e.speed)
 local engine=T.root:gsub('scripts/examples/envoy/scripts/$','platform/')..'gw_script_fighter_mod.inc'
 local file=assert(io.open(engine));local source=file:read('a');file:close()
 local registered=assert(source:match('keys%[%d+%]=%{([^}]+)%}'))
 for key in pairs(fields) do assert(registered:find('"'..key..'"',1,true),'unsupported modifier '..key) end
 assert(not fields.jump_height and not fields.air_jump_height)
end)
T.test('Jump zero and all growth levels keep fixed radius and drop probability',function()
 for _,level in ipairs({0,1,5,10,25,99}) do
  local c=C.new();for _,k in ipairs(C.stats) do local s=c.stats[k];s.points=C.threshold(level);s.life_gain=s.points;s.level=level end
  for _,kind in ipairs({'young','power','speed','guard','jump','balanced'}) do
   c.type=kind;local e=C.effects(c);assert(e.pickup_radius==17.5 and e.drop_chance==.8)
   assert(e.speed<=1.2 and e.air_speed<=1.28 and e.damage_dealt<=1.1 and e.damage_taken>=.85)
  end
 end
end)
T.test('Jump and balanced passives are felt in movement and stay capped',function()
 local c=C.new();local base=C.effects(c);c.type='jump';assert(C.passive(c).name=='Skybound')
 assert(C.effects(c).air_speed>base.air_speed)
 c.type='balanced';local e=C.effects(c);assert(C.passive(c).name=='Sure Footed' and e.speed>base.speed and e.air_speed>base.air_speed)
 for _,colour in ipairs({'red','green','blue','yellow'}) do C.feed(c,colour,100000) end
 c.type='jump';e=C.effects(c);assert(e.jump_air_bonus<=.08 and e.air_speed<=1.28)
end)
T.done()
