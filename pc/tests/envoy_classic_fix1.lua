local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();D.save=T.module('save',D);D.classic=T.module('classic',D)
T.test('retail effects are noticeable capped and include real jumps and resistance',function()
 local C=D.companion;local c=C.new();local base=C.effects(c)
 C.feed(c,'green',120);local e=C.retail_effects(c)
 assert(e.speed>=1.10 and e.speed<=1.30 and base.speed==1)
 C.feed(c,'yellow',120);C.feed(c,'blue',120);e=C.retail_effects(c)
 assert(e.jump_height>1 and e.air_jump_height>1 and e.knockback_taken<1)
 for _,k in ipairs(C.stats) do C.feed(c,({power='red',speed='green',guard='blue',jump='yellow'})[k],1000000) end
 e=C.retail_effects(c);assert(e.speed<=1.30 and e.damage_dealt<=1.24 and e.damage_taken>=.76 and e.knockback_taken>=.80)
end)
T.test('zero level opponents have a bounded starter budget',function()
 local c=D.companion.new()
 for seed=1,100 do local v=D.classic.roll(c,seed,0,0,2,1);local n=0;for _,k in ipairs(D.companion.stats) do n=n+v[k] end;assert(n>=3 and n<=5) end
end)
T.test('every retail stage gets all enemy templates and concrete reward effects',function()
 local mods={};local g={log=function() end,match=function() return {} end,start_1p=function() return true end,
 mode_1p=function() return {held=true} end,hold_1p=function() return true end,release_1p=function() end,
 loop_1p=function() end,end_1p=function() end,spawn_1p=function(p,v) if v and v.tint then assert(type(v.tint)=='number','native tint rejects strings') end;mods[p]=v;return true end}
 local r=D.classic.new(g,D.save.new_profile(),function() return true end);assert(r:start('classic','mario',2,3,42))
 for stage,kind in ipairs({'battle','team','giant','metal','bonus'}) do
  local opponents=kind=='bonus' and {} or kind=='team' and {{port=2},{port=3},{port=4}} or {{port=2}}
  local e={stage_index=stage-1,stage_kind=kind,loop=0,player_port=1,opponents=opponents};r:stage_start(e)
  assert(mods[1].jump_height and #r.tags==#opponents)
  for _,p in ipairs(opponents) do assert(mods[p.port].tint and mods[p.port].knockback_taken and r.enemy_templates[p.port]) end
  r:stage_clear(e);assert(r.reward and r.reward.options[1].effect:find('%%') or r.reward.options[1].colour=='white')
  assert(r:pick(1));assert(r.reward.animation==0 and r.reward.animation_frames==48 and r.reward.levelups)
  r:tick();assert(r.reward.animation==1);assert(r:acknowledge())
 end
end)
T.test('Guard flash follows native percent increase not idle or respawn reset',function()
 local percent=0;local g={player=function() return {percent=percent} end}
 local r=D.classic.new(g,D.save.new_profile(),function() return true end)
 r.active=true;r.companion=r.profile.companions[1];D.companion.feed(r.companion,'blue',120)
 r:frame();assert(r.guard_flash==0);percent=8;r:frame();assert(r.guard_flash==12)
 percent=0;r:frame();assert(r.guard_flash==11)
 for _=1,11 do r:frame() end;assert(r.guard_flash==0)
end)
T.test('retail retains evolved passives within its caps',function()
 local C=D.companion;local c=C.new();c.type='speed';assert(C.retail_effects(c).speed>1)
 c.type='guard';assert(C.retail_effects(c).damage_taken<1)
 c.type='jump';assert(C.retail_effects(c).air_speed>1)
end)
T.done()
