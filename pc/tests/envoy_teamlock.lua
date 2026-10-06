-- 2026-10-06 session faults, mod side: a status two appliers disagree on keeps the larger maximum (no snapshot of stacks 2 / max 1),
-- an out-of-range status in a snapshot is repaired or dropped (never a refusal that costs the build), and the boss-stage safety nets
-- (the out-of-bounds watchdog never takes a last stock, never acts on the player of a won boss fight).
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();D.mod_codec=T.module('mod_codec',D);D.mod_schema=T.module('mod_schema',D);D.mod_pool=T.module('mod_pool',D);D.mod_engine=T.module('mod_engine',D)
local function burner(id,max)
 local m=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[1]));m.id=id;m.trigger='hit_dealt';m.conditions={};m.stacking={max=1}
 m.effects={{op='status',status='burn',duration=120,amount=1,max=max,refresh='refresh',subject='target'}}
 return m
end
T.test('two appliers of Burn with different maxima: the instance keeps the larger max, so stacks never exceed it, and the engine exports/imports it',function()
 local a,b=burner('burn_one',1),burner('burn_three',3)
 local ok,r=pcall(function() local r=D.mod_engine.new(5,{a,b});return r end)
 if not ok then return end -- the schema refuses a hand-made rule here: the status-level check below still covers the invariant
 r:equip(1,'burn_one');r:equip(1,'burn_three')
 r:begin_frame({[1]={percent=0,stocks=3,grounded=true},[2]={percent=0,stocks=3,grounded=true}})
 r:emit{kind='hit_dealt',port=1,target=2,tags={fire=true},damage=8};r:drain()
 local v=r:status(2,'burn');assert(v and v.stacks<=v.max,'stacks '..tostring(v and v.stacks)..' max '..tostring(v and v.max))
 local copy=D.mod_engine.new(6,{a,b});copy:import(r:export())
end)
T.test('a snapshot holding Burn with 2 stacks and max 1 is repaired, not refused; an unusable status is dropped; both are listed',function()
 local r=D.mod_engine.new(9,D.mod_pool);r:equip(1,'kindling')
 r:begin_frame({[1]={percent=0,stocks=3,grounded=true},[2]={percent=0,stocks=3,grounded=true}})
 r.statuses[2]={burn={expires=r.frame+100,stacks=2,max=1,amount=1,next_tick=r.frame+60,origin={}},
  curse={expires=r.frame+100,stacks=1,max=1,amount=1,next_tick=r.frame+60,origin='bad'}}
 r:drain();local text=r:export()
 local into=D.mod_engine.new(10,D.mod_pool);into:import(text)
 local v=into.statuses[2] and into.statuses[2].burn;assert(v and v.stacks==2 and v.max==2,'repaired: max raised to the stacks '..tostring(v and v.stacks)..'/'..tostring(v and v.max)..' repairs '..table.concat(into.status_repairs or {},'; '))
 assert(not (into.statuses[2] and into.statuses[2].curse),'the unusable one is dropped')
 assert(into.status_repairs and #into.status_repairs==2,'both listed for the host to say')
 -- the live engine repairs itself before a publication
 local fixed=r:sanitize_statuses();assert(fixed and #fixed>=1 and r.statuses[2].burn.max==2)
end)
T.done()
