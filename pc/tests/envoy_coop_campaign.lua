-- Fixes found by the co-op campaigns (2026-10-05, second lane): deep-loop crit configuration, the shared derivation memo, the checkpoint size
-- of a hosted run, ground drops on a full bag, a refused publication being retried and said, never silently dropped.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={};for _,name in ipairs({'mod_schema','mod_codec','mod_budget','keystones','mod_pool','mod_engine','mod_echo_lab'}) do D[name]=T.module(name,D) end
D.mod_display={new=function(g,engine) local v={engine=engine};function v:warm(e) self.engine=e;return true end;function v:update(e) self.engine=e;e.display.trace_key=table.concat(e.trace,',') end;function v:clear() end;function v:intensity() end;function v:on_loadstate(e) self.engine=e end;function v:tick() end;function v:draw() end;return v end}
for _,n in ipairs({'drive_loot','drive_bag','foe_roll'}) do D[n]=T.module(n,D) end
D.foe_lab=T.module('foe_lab',D)
D.pickup_juice={pitch={},new=function()return {clear=function()end,tick=function()end,drop=function()return {fx={}} end,collect=function()end,expire=function()end} end}
for _,n in ipairs({'menu_input','drive_menu','drive_drop','drive_lab'}) do D[n]=T.module(n,D) end
D.mod_lab=T.module('mod_lab',D)
local P=D.mod_progression
local function engine(build,ctx)
 local e=D.mod_engine.new(1,D.mod_pool,{context=ctx or P.context(40,0)});e:set_build(1,build,{});e:set_build(2,{},{});return e
end

T.test('a forced crit never leaves the crit multiplier above its maximum, at any depth',function()
 local seen_force=false
 for tier=1,8 do for _,ctx in ipairs({P.context(10,0),P.context(60,3),P.context(120,6)}) do
  local ok,e=pcall(engine,{keen=1,brutal=1,wave_edge=tier,retaliation=tier},ctx)
  if ok then
   local cfg=e:crit_config(1)
   if cfg then for tag,s in pairs(cfg.slots) do assert(s.multiplier<=s.multiplier_max+1e-9,'multiplier above multiplier_max in slot '..tag);assert(s.multiplier_max<=4+1e-9);if s.mean>4 then seen_force=true end end end
  end
 end end
 assert(seen_force,'the sweep must reach a forced crit above x4 (else it proves nothing)')
end)

T.test('the derivation memo is shared by engines of one pool and context, and cannot leak across contexts or builds',function()
 local a=engine({keen=1},P.context(40,0));local b=engine({keen=1},P.context(40,0));local c=engine({keen=1},P.context(40,1)),nil
 local ra=a:family_budget(1);local rb=b:family_budget(1)
 assert(ra==rb,'a second engine with the same build reuses the first one derivation')
 local ba,sa=a:family_budget(1);local bc,sc=c:family_budget(1);assert(sa==sc or ba~=bc) -- another loop never reads the first loop's table
 local d=engine({keen=1,brutal=1},P.context(40,0));local _,sd=d:family_budget(1);assert(sd~=sa,'another build is another answer')
 local r1=a:native_rules(1);local r2=b:native_rules(1);assert(r1==r2)
 -- the content memo of the budget: same inputs, same tables; a tuning revision is a new key
 local pool=D.mod_pool;local o1,s1=D.mod_budget.build(pool,{keen=1},{},{});local o2,s2=D.mod_budget.build(pool,{keen=1},{},{});assert(o1==o2 and s1==s2)
 local hits=D.mod_budget.cache_stats.hits;D.mod_budget.build(pool,{keen=1},{},{});assert(D.mod_budget.cache_stats.hits==hits+1)
 local v1=D.mod_budget.values(pool,{keen=1},{},{});v1.damage_dealt=99;local v2=D.mod_budget.values(pool,{keen=1},{},{});assert(v2.damage_dealt~=99,'values hands out a copy')
end)

local function fixture(hosted)
 local s={spawns=0,commands={},players={{x=0,y=0},{x=30,y=0}}}
 local g={command=function(n,f)s.commands[n]=f end,log=function()end,player=function(p)return s.players[p]end,
  paused=function()return false end,pause=function()end,resume=function()end,pad=function()return {}end,input_mask=function()end,
  item_spawn=function(_,_,_,o)s.spawns=s.spawns+1;s.payload=o.payload;return 100+s.spawns end,item_despawn=function()end,items=function()return {}end}
 local lab={engine=D.mod_engine.new(104729,D.mod_pool),allowed=function()return true end,replaying=function()return false end,display={warm=function()return true end},options={},hosted=function() return hosted end}
 return s,D.drive_lab.new(g,lab),lab
end
T.test('a run may have a drop on the ground with a full bag (the pickup asks which to give up); the LAB still reserves space',function()
 for _,hosted in ipairs({true,false}) do
  local s,a,lab=fixture(hosted);for i=1,11 do assert(a:command('give common '..i));a:apply() end
  assert(a:command('drop rare 40'))                             -- 11 in the bag + 1 on the ground = full by reservation
  assert(a.bag:give(a.loot:roll(7777,lab.engine.context)))      -- a run's pickup path puts a 12th drive in the bag while the drop still lies there
  local snap=D.mod_codec.decode(D.mod_codec.encode(a:snapshot()))
  local roots={engine=lab.engine,debug_equipped={},pending={},echo_manual={},check_echo_capacity=function()end}
  local ok=pcall(function() return a:validate(snap,roots) end) -- roots is what the publication check passes: a plain table
  assert(ok==hosted,'hosted '..tostring(hosted)..' validate '..tostring(ok))
 end
end)

T.test('a hosted run leaves opponent plates and records out of the 16 KiB checkpoint; the LAB keeps them',function()
 for _,hosted in ipairs({true,false}) do
  local g={command=function()end,log=function()end,player=function()return nil end,match=function()return {} end}
  local lab={hosted=function() return hosted end,engine=D.mod_engine.new(1,D.mod_pool),options={}}
  local f=D.foe_lab.new(g,lab);f.builds[3]={port=3};f.labels[3]={title='x',lines={'a'},left=100}
  local snap=f:snapshot(true)
  assert((next(snap.builds)==nil)==hosted and (next(snap.labels)==nil)==hosted)
 end
end)
local function lab_fixture()
 local s={ready=true,lab=true,net=false,replay=false,commits=0,clears=0,visual_clear=0,mods={},hit_rules={},players={
  [1]={percent=30,stocks=4,falls=0,char=1,action=14,airborne=false},
  [2]={percent=20,stocks=4,falls=0,char=2,action=14,airborne=false,cpu=true}}}
 local g={pad=function()return {}end,input_mask=function()end,items=function()return {}end,paused=function()return true end,fixture=s,sim_supported=true,hit_rule_add=function()error('direct hit-rule write is not replayable')end,fighter_status=function()error('direct status write is not replayable')end,command=function(_,fn) s.command=fn end,log=function() end,
  cpu_mode=function(p,m) if s.cpu_refuse then return false end;s.mode={port=p,mode=m};return true end,
  match=function() return {active=true,netplay=s.net,stage=32} end,lab_mode=function() return s.lab end,
  player=function(p) return s.players[p] end,sim_replaying=function() return s.replay end,
  hit_rules=function(p) return {owner=s.hit_rules[p] and 7 or 0,percent_only=s.capability~=false,progression=s.progression~=false} end,
  echoes=function()return {journal=s.echo_capability~=false}end,
  sim_read=function() return s.blob end,sim_clear=function() s.clears=s.clears+1;s.blob=nil;s.mods={};s.hit_rules={} end,
  sim_commit=function(blob,ops)
   if s.refuse then error("checkpoint budget exhausted") end
   s.commits=s.commits+1;s.blob=blob;s.ops=ops
   for _,e in ipairs(ops) do if e.op=='fighter_mod' then s.mods[e.port]=e.values elseif e.op=='hit_rules' then s.hit_rules[e.port]={rules=e.rules,bits=e.status_bits} elseif e.op=='echoes' then s.echoes=s.echoes or {};s.echoes[e.port]=e.rules elseif e.op=='damage' then s.players[e.port].percent=e.value end end   -- technique ops (crit, armour...) are not modelled by this fixture
   return true
  end}
 s.g=g;s.a=D.mod_lab.new(g);return s,s.a
end

T.test('a publication that runs out of script budget is deferred and retried, never dropped; a refused one is retried twice in a run and always said',function()
 local s,a=lab_fixture();local logs={};s.g.log=function(t) logs[#logs+1]=t end;a.g.log=s.g.log
 assert(a:command('add kindling'));assert(#a.pending==1)
 local real=a.prospective;a.prospective=function() error('envoy/mod_budget.lua:126: ran too long (limit: 2000000 instructions or 50 ms per call)') end
 for _=1,3 do assert(a:frame()==true);assert(#a.pending==1,'the staged edit is kept while the check is deferred') end
 local said=false;for _,l in ipairs(logs) do if l:find('deferred (script budget)',1,true) then said=true end end;assert(said,'the deferral is said in the log')
 a.prospective=real;a:frame();assert(#a.pending==0 and s.commits>=1,'the retry publishes it')
 -- a real refusal in a hosted run: two retries, then the drop is loud and the run keeps its bags (the host republishes from them)
 local s2,b=lab_fixture();local logs2={};s2.g.log=function(t) logs2[#logs2+1]=t end;b.g.log=s2.g.log;local toasts={};b.toast=function(t) toasts[#toasts+1]=t end
 b.options.run_host=function() return true end
 assert(b:command('add kindling'));b.prospective=function() error('envoy/x.lua:1: invalid thing') end
 b:frame();b:frame();local retries=0;for _,l in ipairs(logs2) do if l:find('retrying',1,true) then retries=retries+1 end end;assert(retries==2,'two retries first')
 b:frame();local dropped=false;for _,l in ipairs(logs2) do if l:find('disabled after pending publication refusal',1,true) and l:find('bags kept',1,true) then dropped=true end end
 assert(dropped,'the drop is said');assert(#toasts>=3,'and shown on screen: '..#toasts)
end)

T.test('a hosted run re-sends overlay and hit-rule operations only when they change, and commits a quiet frame only every tenth; the LAB commits every frame',function()
 local function count(hosted)
  local s,a=lab_fixture();if hosted then a.options.run_host=function() return true end end
  assert(a:command('add glass_core 2'));a:frame();a:frame()
  local real=s.g.sim_commit;local ops_sent,commits=0,0
  s.g.sim_commit=function(blob,ops) commits=commits+1;for _,o in ipairs(ops) do if o.op=='fighter_mod' or o.op=='hit_rules' then ops_sent=ops_sent+1 end end;return real(blob,ops) end
  for _=1,20 do a:frame();a.engine.frame=a.engine.frame end
  return ops_sent,commits
 end
 local o1,c1=count(true);local o2,c2=count(false)
 assert(o2>0 and c2==20,'the LAB keeps committing and sending: '..o2..' '..c2);assert(o1==0,'a run sends no overlay while nothing changed: '..o1);assert(c1<=3,'a quiet run commits about every tenth frame: '..c1)
end)

T.test('a run checkpoint trims each status origin list to six lines; a full export keeps them, and both import',function()
 local e=engine({keen=1},P.context(40,0))
 e.statuses[2]={burn={expires=e.frame+600,stacks=1,max=3,amount=1,next_tick=e.frame+60,origin={}}}
 for i=1,20 do e.statuses[2].burn.origin[i]='burn applied by Kindling '..i end
 local full,trimmed=e:export(),e:export(true)
 assert(#trimmed<#full,'trimmed is smaller: '..#trimmed..' < '..#full)
 local a=D.mod_engine.new(1,D.mod_pool,{context=P.context(40,0)});a:import(full);assert(#a.statuses[2].burn.origin==20)
 local b=D.mod_engine.new(1,D.mod_pool,{context=P.context(40,0)});b:import(trimmed);assert(#b.statuses[2].burn.origin==6)
 assert(#e.statuses[2].burn.origin==20,'the live status is untouched')
end)

-- ---- the synergy visual across seats --------------------------------------------------------------------------------------------------
D.mod_graph=T.module('mod_graph',D);D.synergy_fx=T.module('synergy_fx',D)
T.test('a chain whose pieces two teammates hold flashes from the applier seat to the payoff seat; a solo chain is unchanged',function()
 local g={calls={},cmds={},logs={},frame_n=0}
 for _,n in ipairs{'fill','box','line','text'} do g[n]=function() end end
 g.kit={text=function() return 1 end,measure=function() return 1 end,panel=function() end}
 g.safe_area=function() return {x=0,y=0,w=640,h=360} end;g.frame=function() return g.frame_n end
 g.player=function(p) if p<=3 then return {x=p*100,y=0} end end;g.project=function(x,y) return x,y,true end
 g.command=function(n,f) g.cmds[n]=f end;g.log=function() end
 local function chain(host)
  local f=D.synergy_fx.new(g,host)
  local e=D.mod_engine.new(1,D.mod_pool,{context=P.context(3,0)});e:set_build(1,{plague_bearer=1},{});e:set_build(2,{pyre=1,malice=1},{});e:set_build(3,{},{})
  local players={};for p=1,4 do players[p]={percent=30,grounded=true,stocks=2,x=p*10,y=0} end
  e:begin_frame(players);e:emit{kind='hit_dealt',port=1,target=3,tags={},depth=1,origin={}};e:drain()
  e:begin_frame(players);e:emit{kind='hit_dealt',port=2,target=3,tags={},depth=1,origin={}};e:drain()
  f:scan(e);return f
 end
 local co=chain({mods={},hud={},seat={allies={[1]=true,[2]=true}}})
 local fl=co.flashes[#co.flashes];assert(fl and fl.cross and fl.from==1 and fl.to==2,'the link runs from P1 (Plague Bearer) to P2 (Malice)');assert(co.counter[2] and co.counter[2].arch.id=='burn','counted on the payoff seat')
 local solo=chain({mods={},hud={}});local fs=solo.flashes[#solo.flashes];assert(fs and not fs.cross and fs.from==2,'no seat table: the old flash from the port that completed it')
end)

T.done()
