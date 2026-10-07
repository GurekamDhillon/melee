-- D1: the boss-hand check must know Crazy Hand (char 30) as well as Master Hand (char 26); D2: a console-sized foe roll
-- (one script call, 2M instructions) at depth 12 / loop 3 must end in a bounded number of tries with a best-effort record.
local T=dofile('melee/pc/tests/envoy_testlib.lua')
local D={};for _,n in ipairs({'mod_schema','mod_codec','mod_engine','mod_pool','drive_loot','drive_bag','mod_budget','mod_progression'}) do D[n]=D[n] or T.module(n,D) end
D.foe_roll=T.module('foe_roll',D)
T.test('boss hands: Master Hand 26 and Crazy Hand 30 (the engine kinds), not other fighters',function()
 local P=D.mod_progression
 assert(P.is_boss_hand,'one shared boss-hand helper required')
 assert(P.is_boss_hand(26) and P.is_boss_hand(30),'Master Hand (26) and Crazy Hand (30)')
 for _,c in ipairs({0,1,25,27,28,29,31,-1}) do assert(not P.is_boss_hand(c),'char '..c..' is not a boss hand') end
 assert(not P.is_boss_hand(nil))
end)
T.test('mod_lab does not hard-code boss ids',function()
 local f=assert(io.open(T.root..'mod_lab.lua'));local s=f:read('a');f:close()
 assert(not s:find('==26',1,true) and not s:find('==27',1,true),'mod_lab still tests raw char ids')
 assert(s:find('is_boss_hand',1,true))
end)
T.test('normal roll at seed 2144865533, depth 12 loop 3 ends within the call budget at every strength',function()
 local r=D.foe_roll.new(D.mod_pool);local lines={}
 r.log=function(s) lines[#lines+1]=s end
 local limit,worst,fell=2000000,0,0
 for _,s in ipairs({1.5,3,5,8,12,20,40,80,150,300}) do
  local n=0;debug.sethook(function() n=n+1000 end,'',1000)
  local rec=r:roll(s,2144865533,0,2,{depth=12,loop=3},'normal',nil,r.sync_attempts)
  debug.sethook();worst=math.max(worst,n)
  assert(n<limit,('strength %s took %d instructions'):format(s,n));r:validate(rec)
 end
 print('worst roll '..worst..' instructions; fallback lines '..#lines)
end)
T.done()
