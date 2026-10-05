local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();D.save=T.module('save',D)
T.test('retail adapter available',function() local f=io.open(T.root..'classic.lua');assert(f,'retail adapter missing');f:close() end)
D.classic=T.module('classic',D)
local function fixture()
 local s={writes=0,mods={},held=false,loops=true,mode={mode='classic',stage_index=0,loop=0,player_port=1,opponents={{port=2},{port=3}}}}
 local g={log=function() end,match=function() return {netplay=false} end,
  mode_1p=function() return s.mode end,start_1p=function(v) s.launch=v;return true end,
  hold_1p=function() s.held=true;return true end,release_1p=function() s.held=false;return true end,
  loop_1p=function(v) s.loops=v;return true end,end_1p=function() s.ended=true;return true end,spawn_1p=function(p,v) s.mods[p]=v;return true end}
 s.g=g;s.r=D.classic.new(g,D.save.new_profile(),function(p) if s.refuse then return false,'disk refused' end;D.save.validate(p);s.writes=s.writes+1;s.saved=D.save.encode(p);return true end)
 return s,s.r
end
local function clear(r,s,stage,final)
 s.mode.stage_index=stage;s.mode.final=final;r:stage_start(s.mode);r:stage_clear(s.mode)
 assert(r.reward and s.held);assert(r:pick(1));assert(r.reward.preview);assert(r:acknowledge());assert(not s.held)
end
T.test('old engine unavailable and launch refusal never ages',function()
 local s,r=fixture();s.g.start_1p=nil;assert(not r:start('classic','mario',2,3,123));assert(s.writes==0 and r.profile.companions[1].age==0)
 s,r=fixture();s.g.start_1p=function() return false,'unavailable' end;assert(not r:start('classic','mario',2,3,123));assert(s.writes==0)
end)
T.test('full Classic and two NG loops carry stats age settings',function()
 local s,r=fixture();assert(r:start('classic','mario',2,3,123));assert(s.launch.loop and s.launch.stocks==3)
 for loop=0,2 do
  s.mode.loop=loop
  for stage=0,10 do clear(r,s,stage,stage==10) end
  r:complete(s.mode);assert(r.active and r.loop==loop+1 and s.loops)
  assert(r.companion.age==1 and r.profile.records.wins==loop+1)
 end
 assert(r.companion.stats.power.points+r.companion.stats.speed.points+r.companion.stats.guard.points+r.companion.stats.jump.points>0)
 assert(r.companion.type~='young');s.mode.loop=3;s.mode.stage_index=0;r:stage_start(s.mode)
 r:finish('fail');assert(not r.active and r.profile.last_result=='fail' and not s.loops and not s.held)
 assert(next(s.mods)==nil)
end)
T.test('Adventure ends at its own final boss and game over settles mid-run',function()
 local s,r=fixture();assert(r:start('adventure','mario',2,3,456));clear(r,s,0,false)
 local points=D.save.encode(r.working);r:game_over(s.mode);local writes=s.writes;r:game_over(s.mode)
 assert(s.writes==writes and r.profile.last_result=='fail' and r.profile.companions[1].age==1)
 assert(r.profile.companions[1].stats.power.points+r.profile.companions[1].stats.speed.points+r.profile.companions[1].stats.guard.points+r.profile.companions[1].stats.jump.points>0)
 assert(points and next(s.mods)==nil)
end)
T.test('seeded opponent spread deterministic normalized and bounded over 1000 rolls',function()
 local c=D.companion.new();for _,k in ipairs(D.companion.stats) do D.companion.feed(c,({power='red',speed='green',guard='blue',jump='yellow'})[k],650) end
 local sum=0;local expected=40
 for seed=1,1000 do
  local a=D.classic.roll(c,seed,4,0,2,1);local b=D.classic.roll(c,seed,4,0,2,1);local total=0
  for _,k in ipairs(D.companion.stats) do assert(a[k]==b[k] and a[k]>=0 and a[k]<=99);total=total+a[k] end
  sum=sum+total
 end
 assert(math.abs(sum/1000-expected)<=1,'normalization mean '..sum/1000)
 print(('opponent budget: mean %.3f / player %d / 1000 seeds / tolerance 1 level'):format(sum/1000,expected))
 local single=D.classic.roll(c,12,1,0,2,1);local team=D.classic.roll(c,12,1,0,2,3);local harder=D.classic.roll(c,12,1,2,2,1)
 local function total(t) local n=0;for _,k in ipairs(D.companion.stats) do n=n+t[k] end;return n end
 assert(total(team)<total(single) and total(harder)>total(single))
end)
T.test('starter and saturated budgets remain bounded without clipping bias',function()
 local c=D.companion.new();for _,k in ipairs(D.companion.stats) do c.stats[k].points=D.companion.threshold(99);c.stats[k].life_gain=c.stats[k].points;c.stats[k].level=99 end
 for seed=1,1000 do local a=D.classic.roll(c,seed,4,0,2,1);local sum=0;for _,k in ipairs(D.companion.stats) do sum=sum+a[k] end;assert(sum==396) end
 c=D.companion.new();for seed=1,1000 do local a=D.classic.roll(c,seed,4,0,2,1);local total=0;for _,k in ipairs(D.companion.stats) do total=total+a[k] end;assert(total>=3 and total<=5) end
end)
T.test('retry identical and spawn modifier tint templates clear at terminal',function()
 local s,r=fixture();assert(r:start('classic','mario',2,3,123));r:stage_start(s.mode)
 local a=s.mods[2];r:stage_start(s.mode);assert(s.mods[2].run_speed==a.run_speed and s.mods[2].tint==a.tint)
 r:spawn{port=2,entity=999};assert(s.mods[2].run_speed==a.run_speed);r:finish('quit');assert(next(s.mods)==nil)
end)
T.test('disk failure retries prepared reward no duplicates and timeout abandons hold',function()
 local s,r=fixture();assert(r:start('classic','mario',2,3,123));r:stage_clear(s.mode);s.refuse=true
 local original=D.save.encode(r.working);assert(not r:pick(1));local c=D.save.encode(r.reward.prepared);assert(not r:pick(1));assert(D.save.encode(r.working)==original and D.save.encode(r.reward.prepared)==c)
 s.refuse=false;assert(r:pick(1));assert(D.save.encode(r.working)==c);r:acknowledge()
 r:stage_clear{stage_index=1,loop=0};s.mode.held=false;r:tick();assert(not r.reward and not s.held)
end)
T.test('reward selection at most once with before after and physical drops off',function()
 local s,r=fixture();assert(r:start('classic','mario',2,3,123));r:stage_clear(s.mode)
 assert(#r.reward.options==3 and D.companion.tuning.retail.physical_drops==false)
 assert(r:pick(2));local points=D.save.encode(r.working);assert(not r:pick(3));assert(D.save.encode(r.working)==points)
 assert(r.reward.before and r.reward.after);assert(r:acknowledge());assert(not r:acknowledge())
end)
T.test('complete arriving with final reward defers evolution and cannot settle twice',function()
 local s,r=fixture();assert(r:start('classic','mario',2,3,123));local e={stage_index=10,loop=0,final=true}
 r:stage_clear(e);r:complete(e);assert(r.reward and r.active and r.profile.records.wins==0 and s.held)
 assert(r:pick(1));assert(r:acknowledge());assert(r.profile.records.wins==1 and r.loop==1)
 r:complete(e);assert(r.profile.records.wins==1 and r.loop==1)
end)
T.test('completion refusal has one winning ledger no phantom next loop and controller results',function()
 local s,r=fixture();assert(r:start('classic','mario',2,3,123));s.refuse=true;r:complete{loop=0,stage_index=10}
 assert(not r.active and r.pending and r.results and not s.loops)
 assert(r.working.pending_seed==nil and r.working.records.runs==1 and r.working.records.wins==1)
 local before=D.save.encode(r.working);s.refuse=false;assert(r:retry_save());assert(D.save.encode(r.working)==before and not r.pending)
end)
T.test('failed reward timeout discards uncommitted feed and rejected start cancels scene',function()
 local s,r=fixture();s.refuse=true;assert(not r:start('classic','mario',2,3,123));assert(s.ended)
 s,r=fixture();assert(r:start('classic','mario',2,3,123));r:stage_clear(s.mode)
 local before=D.save.encode(r.working);s.refuse=true;assert(not r:pick(1));s.mode.held=false;r:tick()
 assert(D.save.encode(r.working)==before and not r.reward)
end)
T.test('optional physical drives use native collection once and default drops nothing',function()
 local s,r=fixture();local player={falls=0,x=20,y=10};s.g.player=function(p) return p==2 and player or nil end
 local n=0;s.g.item_spawn=function(name,x,y,v) assert(name=='drive' and x==20 and y==10);n=n+1;s.payload=v.payload;return n end
 s.g.item_despawn=function() return true end
 assert(r:start('classic','mario',2,3,123));r:stage_start(s.mode);r:frame();player.falls=1;r:frame();assert(n==0)
 local t=D.companion.tuning.retail;t.physical_drops=true;r:frame();player.falls=2;r:frame();assert(n==1)
 assert(r:item_collect{name='drive',port=1,item=1,payload=s.payload});local saved=D.save.encode(r.working)
 assert(not r:item_collect{name='drive',port=1,item=1,payload=s.payload});assert(D.save.encode(r.working)==saved)
 r:finish('quit');t.physical_drops=false
end)
T.test('Adventure definitive completion offers larger reward after retail Giga decision (when the stage clear gave none)',function()
 local s,r=fixture();assert(r:start('adventure','mario',2,3,123));s.mode.stage_index=11
 r:complete{mode='adventure',stage_index=11,loop=0,final=true}
 assert(r.reward and r.reward.final and r.reward.options[1].points==D.companion.tuning.retail.reward_points[1]*D.companion.tuning.retail.final_multiplier and r.profile.records.wins==0)
 assert(r:pick(1));assert(r:acknowledge());assert(r.profile.records.wins==1)
end)
T.test('Adventure: a final that the stage clear already rewarded is not a second reward moment',function()
 local s,r=fixture();assert(r:start('adventure','mario',2,3,123));s.mode.stage_index=11
 r:stage_clear(s.mode);assert(r:pick(1));assert(r:acknowledge())
 r:complete{mode='adventure',stage_index=11,loop=0,final=true}
 assert(not r.reward,'a second reward moment');assert(r.profile.records.wins==1)
end)
T.test('physical drops require confirmed fall and despawn refusals retain cleanup ownership',function()
 local s,r=fixture();local p={falls=0,x=0,y=0};s.g.player=function(port) return port==2 and p or nil end
 local drops=0;s.g.item_spawn=function() drops=drops+1;return drops end
 local refused=true;s.g.item_despawn=function() return not refused end
 assert(r:start('classic','mario',2,3,123));r:stage_start(s.mode);local t=D.companion.tuning.retail;t.physical_drops=true
 r:frame();p=nil;r:frame();assert(drops==0,'disappearance is not a confirmed KO')
 p={falls=0,x=0,y=0};r:frame();p.falls=1;r:frame();assert(drops==1);p=nil;r:frame();assert(drops==1)
 r:finish('quit');assert(r.items[1] and r.items[1].retired);refused=false;r:tick();assert(next(r.items)==nil);t.physical_drops=false
end)
T.done()
