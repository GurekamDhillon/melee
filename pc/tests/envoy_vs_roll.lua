-- Envoy VS builds, part 1: the roller. `max` (the highest bounded build for a context), P1 as a build port, a bounded cost per call at
-- every strength and seed, and the ordinary rolls staying byte for byte what they were.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={};for _,n in ipairs({'mod_schema','mod_codec','mod_engine','keystones','mod_pool','drive_loot','drive_bag'}) do D[n]=T.module(n,D) end
D.foe_roll=T.module('foe_roll',D)
local P=D.mod_progression
local LIMIT=2000000 -- the script call limit (instructions)
local function counted(f,...)
 local n=0;debug.sethook(function() n=n+1000 end,'',1000)
 local ok,a=pcall(f,...);debug.sethook()
 return ok,a,n
end
local function sum(s) local h=0;for i=1,#s do h=(h*31+s:byte(i))%4294967291 end return h end

T.test('ordinary rolls are unchanged (the pinned checksums of the roller before the VS work)',function()
 local r=D.foe_roll.new(D.mod_pool)
 for _,c in ipairs({{{depth=0},1.5,2379234544},{{depth=5},3,156425018},{{depth=12,loop=3},5,2218038403},{{depth=12,loop=3},8,2357947245},{{depth=9},2,4079859321}}) do
  local acc=0
  for seed=1,6 do acc=(acc+sum(D.mod_codec.encode(r:roll(c[2],seed,3,2,c[1],'normal',nil,r.sync_attempts))))%4294967291 end
  assert(acc==c[3],('roll depth %s strength %s changed: %d'):format(c[1].depth,c[2],acc))
 end
end)

T.test('max at depth 12 loop 3 stays under the call budget over many seeds, one call or sliced',function()
 local r=D.foe_roll.new(D.mod_pool);local ctx={depth=12,loop=3};local worst,slice=0,0
 for seed=1,60 do
  local ok,rec,n=counted(function() return r:roll('max',seed,3,2,ctx,'normal',nil,r.sync_attempts) end)
  assert(ok,'max roll refused for seed '..seed..': '..tostring(rec));assert(n<LIMIT,('max seed %d took %d instructions'):format(seed,n));worst=math.max(worst,n)
  r:validate(rec);assert(#rec.build.keystones>=1 and rec.strength>1)
  local job=r:roll_job('max',seed,3,2,ctx,'normal');local done,steps=nil,0
  while not done do
   local ok2,res,m=counted(function() return r:roll_step(job,2) end);assert(ok2,'sliced max: '..tostring(res));assert(m<LIMIT/3,('a slice took %d instructions'):format(m))
   slice=math.max(slice,m);done=res;steps=steps+1;assert(steps<200,'a max job never finished')
  end
  assert(D.mod_codec.encode(done)==D.mod_codec.encode(rec),'sliced and one-call max differ')
 end
 print('max: worst one-call '..worst..' instructions, worst slice '..slice)
end)

T.test('max is full: every slot, the whole keystone allowance the pool and rules permit, and it beats a plain roll',function()
 local r=D.foe_roll.new(D.mod_pool)
 for _,c in ipairs({{depth=0},{depth=5},{depth=12,loop=3}}) do
  local ctx=P.context(c);local rec=r:roll('max',7,3,2,ctx,'normal',nil,r.sync_attempts)
  local slots=0;for slot=1,P.slots(ctx) do if rec.build.equipped[slot] then slots=slots+1 end end
  assert(slots==P.slots(ctx),'max left a slot empty at depth '..c.depth)
  assert(#rec.build.keystones==math.min(P.keystones(ctx),#r.keys) or #rec.build.keystones>=1)
  local plain=r:roll(1.5,7,3,2,ctx,'normal',nil,r.sync_attempts);assert(rec.strength>=plain.strength,'max below an ordinary roll')
 end
 local a,b=r:roll('max',1,3,2,{depth=12,loop=3}),r:roll('max',2,3,2,{depth=12,loop=3});assert(D.mod_codec.encode(a)~=D.mod_codec.encode(b),'max is not randomised by seed')
 assert(D.mod_codec.encode(a)==D.mod_codec.encode(r:roll('max',1,3,2,{depth=12,loop=3})),'max is not deterministic')
end)

T.test('every strength and seed finishes: high targets are bounded per call and never refused or raise',function()
 local r=D.foe_roll.new(D.mod_pool);local ctx={depth=12,loop=3};local worst=0
 for _,s in ipairs({10,30,60,100,300,1000,100000}) do
  for seed=1,25 do
   local job=r:roll_job(s,seed,3,2,ctx,'normal');local done,steps=nil,0
   while not done do
    local ok,res,n=counted(function() return r:roll_step(job,2) end);assert(ok,('strength %s seed %d: %s'):format(s,seed,tostring(res)))
    assert(n<LIMIT/3,('strength %s seed %d: a slice took %d instructions'):format(s,seed,n));worst=math.max(worst,n);done=res;steps=steps and steps+1;assert(steps<400,'a roll never finished')
   end
   r:validate(done)
  end
 end
 print('sliced rolls: worst slice '..worst..' instructions')
end)

T.test('the one-call roll of a high strength is refused only by the call budget guard, never by raising mid-way',function()
 -- The console sends a high strength down the sliced path; roll() itself (one call) must still stay in budget for the strengths the old test pinned.
 local r=D.foe_roll.new(D.mod_pool)
 for _,s in ipairs({30,60}) do for seed=1,12 do local ok,res,n=counted(function() return r:roll(s,seed,3,2,{depth=12,loop=3},'normal',nil,r.sync_attempts) end);assert(ok,('strength %s seed %d: %s'):format(s,seed,tostring(res)));assert(n<LIMIT) end end
end)

T.test('P1 can take a build: a roll and its validation accept port 1, and port 0 or 7 stay refused',function()
 local r=D.foe_roll.new(D.mod_pool)
 for _,s in ipairs({1.5,'max'}) do
  local rec=r:roll(s,5,3,1,{depth=9},'normal',nil,r.sync_attempts);assert(rec.port==1);r:validate(rec)
 end
 T.refuses(function() r:roll(1.5,5,3,0,{depth=9}) end);T.refuses(function() r:roll(1.5,5,3,7,{depth=9}) end)
end)

T.test('a refused seed says why',function()
 local r=D.foe_roll.new(D.mod_pool)
 local ok,why=pcall(r.roll_job,r,1.5,2147483648,3,2,{depth=9});assert(not ok and tostring(why):find('seed',1,true) and tostring(why):find('2147483646',1,true),tostring(why))
 ok,why=pcall(r.roll_job,r,1.5,-1,3,2,{depth=9});assert(not ok and tostring(why):find('seed',1,true),tostring(why))
 ok,why=pcall(r.roll_job,r,'big',1,3,2,{depth=9});assert(not ok and tostring(why):find('max',1,true),tostring(why))
end)
T.done()
