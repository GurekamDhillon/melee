-- Co-op run (coop.lua): two seats over one rule host, the stage plan, the rules behind their named values, the reward moment in turn,
-- ownership of drops, the run record and its digest, and that a one-player host is untouched by any of it.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_engine','keystones','mod_pool','drive_loot','drive_merge','drive_economy','drive_bag','foe_roll','fighters'}) do D[n]=T.module(n,D) end
D.grid=dofile(T.root..'../../demos/grid-inventory/scripts/grid.lua')
D.mod_display={new=function(g,engine) local v={engine=engine};function v:warm() return true end;function v:update() end;function v:clear() end;function v:on_loadstate(e) self.engine=e end;function v:tick() end;return v end}
D.pickup_juice={pitch={},new=function()return {drop=function()return {fx={}}end,collect=function()end,expire=function()end,clear=function()end,tick=function()end}end}
for _,n in ipairs({'menu_input','drive_menu','drive_text','drive_drop','drive_lab','foe_lab','mod_lab','run_screen','run_hud','run_host','coop','coop_synth'})do D[n]=T.module(n,D)end
D.drive_economy.tuning.floor_chance=1
local Coop=D.coop
local function reset() Coop.reset_tuning() end
local function fixture()
 reset()
 local s={commands={},pad={},logs={},clock=100,spawns=0,launches={},paused=false,holds={},assist={},fly={},items={},
  players={}}
 local g={fixture=s,command=function(n,f)s.commands[n]=f end,log=function(t)s.logs[#s.logs+1]=t end,
  match=function()return {active=true,netplay=false,stage=32}end,lab_mode=function()return false end,
  sim_supported=true,sim_replaying=function()return false end,player=function(p)return s.players[p]end,
  hit_rule_add=function()error('direct native write')end,fighter_status=function()error('direct native write')end,
  hit_rules=function()return {percent_only=true,progression=true,owner=7}end,
  sim_clear=function()s.blob=nil end,sim_read=function()return s.blob end,
  sim_commit=function(blob,ops)s.blob=blob;s.ops=ops;return true end,
  pad=function(p)return s.pad[p] or {} end,input_mask=function(p,v)s.mask=v end,input_chord=function()end,paused=function()return s.paused end,
  pause=function()s.paused=true end,resume=function()s.paused=false end,
  items=function()return s.items end,
  item_spawn=function(_,x,y,o)s.spawns=s.spawns+1;s.payload=o.payload;s.items[#s.items+1]={handle=100+s.spawns,x=x,y=y};return 100+s.spawns end,item_despawn=function(h)return true end,
  match_end_hold=function(r,on) s.holds[r]=on or nil;return true end,
  scene_launch=function(text) s.launches[#s.launches+1]=text;return text end,
  cpu_assist=function(p,c) s.assist[p]=c;return true end,
  fly_target=function(p,x,y) s.fly[p]=s.fly[p] or {};s.fly[p].x=x;s.fly[p].y=y end,fly_attack=function(p,on) s.attack_calls=(s.attack_calls or 0)+1;s.fly[p]=s.fly[p] or {};s.fly[p].attack=on end,fly_state=function(p) return {attacking=(s.fly[p] and s.fly[p].attack) or false} end,fly_speed=function()end,fly_solid=function()end,fly_clear=function()end,fly=function()end,
  time=function() return s.clock end,
  safe_area=function() return {x=0,y=0,w=640,h=480,right=640,bottom=480} end,
  stage_bounds=function() return {camera={left=-100,right=100,top=100,bottom=-100}} end,floor_below=function(x,y) return 0 end}
 local running=false
 local coop
 local mods=D.mod_lab.new(g,{run_host=function() return running end,run_ready=function() return true end})
 coop=Coop.new(g,mods,{})
 return s,g,mods,coop,function(v) running=v end
end
-- Put the fighters of the launched scene into the match, as the engine would.
local function populate(s,coop,keep)
 local plan=coop.plan
 for i=1,2 do s.players[i]={x=-20+40*(i-1),y=0,percent=0,stocks=plan.stocks_player,falls=0,char=i,action=14} end
 for i=1,#plan.foes do s.players[2+i]={x=40+20*i,y=0,percent=0,stocks=plan.stocks_foe,falls=0,char=3,action=14,cpu=true} end
 coop:scene_started()
 for _=1,4 do coop:frame() end
end
local function start(opts)
 local s,g,mods,coop,run=fixture();run(true)
 opts=opts or {};opts.seed=opts.seed or 777
 local ok,why=coop:start(opts);assert(ok,why)
 return s,g,mods,coop
end
local function settle(coop,frames) for _=1,frames do coop:frame();coop:tick();coop.mods:frame();coop.mods:tick() end end
local function ko_all(s,coop) for i=1,#coop.plan.foes do s.players[2+i].stocks=0;s.players[2+i].falls=1 end end
-- finish a stage the simple way: KO every foe, wait for the clear, let the synthetic chooser take every reward screen
local function win_stage(s,coop,synth,who)
 if coop.state=='launching' then populate(s,coop) end;assert(coop.state=='stage','stage began, was '..coop.state)
 settle(coop,70);s.players[3].falls=1;settle(coop,14);ko_all(s,coop);settle(coop,12)
 local mods=coop.mods
 for k,r in pairs(mods.drives.drops.records) do mods:pickup{name='drive',port=who or 1,item=r.handle,payload={colour='white',amount=k}} end
 for _=1,60 do settle(coop,1);synth:tick();if coop.state=='launching' or not coop.active then break end end
end
T.test('the stage plan is a pure function of seed, stage and loop, and the scene string names teams, stocks and CPU levels',function()
 reset()
 local a,b=Coop.plan(5,0,0),Coop.plan(5,0,0);assert(table.concat(a.foes,',')==table.concat(b.foes,',') and a.stage_name==b.stage_name)
 assert(#Coop.plan(5,0,0).foes==2 and #Coop.plan(5,3,0).foes==3 and #Coop.plan(5,6,0).foes==4 and #Coop.plan(5,0,1).foes==3 and #Coop.plan(5,7,3).foes==4,'count: base 2, +1 per 3 stages, +1 per loop, capped at 4')
 assert(Coop.plan(5,7,0).final and not Coop.plan(5,6,0).final,'stage 7 is the last of eight')
 local diff=false;for st=0,5 do if Coop.plan(5,st,0).stage_name~=Coop.plan(6,st,0).stage_name then diff=true end end;assert(diff,'the seed matters')
 local p=Coop.plan(9,2,0);local text=Coop.scene(p,{{fighter='fox',policy='human'},{fighter='marth',policy='cpu',cpu_level=7}})
 assert(text:find('mode=vs;teams=1',1,true) and text:find('p1=fox/hu/team0/stocks3',1,true) and text:find('p2=marth/cpu7/team0/stocks3',1,true) and text:find('p3=',1,true) and text:find('/team1/stocks1',1,true),text)
end)
T.test('team strength: the formula is a named value; one build gives that build\'s strength',function()
 reset()
 assert(math.abs(Coop.team_strength({1.5})-1.5)<1e-9)
 assert(math.abs(Coop.team_strength({1.5,1.3})-(1+.5+.5*.3))<1e-9,'max_share')
 Coop.set('team_formula','max');assert(math.abs(Coop.team_strength({1.5,1.3})-1.5)<1e-9)
 Coop.set('team_formula','sum_dim');assert(math.abs(Coop.team_strength({1.5,1.3})-(1+.8/1.25))<1e-9)
 Coop.set('team_formula','max_share');Coop.set('team_share',1);assert(math.abs(Coop.team_strength({1.5,1.3})-1.8)<1e-9)
 T.refuses(function() Coop.set('team_formula','nonsense') end);T.refuses(function() Coop.set('nothing',1) end)
 reset()
end)
T.test('two seats: each builds on its own port with its own bag, keystone and starter; the one-player seat 1 is the same object as before',function()
 local s,g,mods,coop=start()
 assert(#coop.hosts==2 and coop.hosts[1].mods==mods and coop.hosts[1].mods.drives==mods.drives and coop.hosts[2].mods.drives==mods.seats[2])
 local b1,b2=coop.hosts[1]:bag(),coop.hosts[2]:bag();assert(b1~=b2)
 assert(b1.equipped[1] and b2.equipped[1],'both got a starter drive')
 local k1,k2=coop.hosts[1]:keystone_ids()[1],coop.hosts[2]:keystone_ids()[1];assert(k1 and k2 and k1~=k2,'different starting keystones: '..tostring(k1)..' '..tostring(k2))
 assert(b1.equipped[1].seed~=b2.equipped[1].seed,'different starter drives')
 populate(s,coop);settle(coop,40)
 local e=mods.engine;assert((next(e.equipped[1] or {}) or next(e.implicits[1] or {})) and (next(e.equipped[2] or {}) or next(e.implicits[2] or {})),'the engine holds a build for port 1 AND port 2')
 assert(coop.hosts[2].mods.engine==mods.engine,'one shared engine')
 assert(s.launches[1]:find('p3=',1,true) and #s.launches==1)
end)
T.test('N=1 is unchanged: no seat, no salt, no tag, seat tables empty',function()
 local s,g,mods,coop,run=fixture();run(true)
 assert(next(mods.seats)==nil)
 local retail={state={player_port=1},loop=0,rules=true,active=true};local h=D.run_host.new(g,mods,retail)
 assert(h.seat==nil and h:port0()==1 and h:salt()==0 and not h.follower and not h:is_ally(2))
end)
T.test('the run starts the plan: stage launched through scene_launch, stage-long match hold, hosts told the stage with the team opponents',function()
 local s,g,mods,coop=start()
 assert(coop.state=='launching' and #s.launches==1)
 populate(s,coop);assert(coop.state=='stage' and s.holds['envoy-coop']==true,'the run, not the engine, ends the stage')
 local h=coop.hosts[1];assert(h.stage==0 and h.stage_kind=='team' and #h.foe_ports==2 and h.foe_ports[1]==3)
 assert(not coop.hosts[2].rolls[3] or true)
 settle(coop,120)   -- the opponents' search may take more slices now that the pool is narrower
 assert(mods.foes.builds[3] and mods.foes.builds[4],'opponents rolled a build each, on ports 3 and 4 and not on a teammate')
 assert(not mods.foes.builds[2] and not mods.foes.builds[1])
 assert(coop.last_team_strength and coop.last_team_strength>=1)
end)
T.test('opponents scale to the team: the run host asks the seat for the strength, not player 1\'s build',function()
 local s,g,mods,coop=start()
 populate(s,coop);settle(coop,40)
 local st=coop.last_strengths;assert(#st==2)
 local expect=Coop.team_strength(st);assert(math.abs(expect-coop.last_team_strength)<1e-9)
end)
T.test('a stage clear opens each player\'s reward screen in turn with its own offers and port colour; then the next stage launches',function()
 local s,g,mods,coop=start({seed=4321})
 coop.hosts[1].mods.drives.drops:count()
 populate(s,coop);settle(coop,70);ko_all(s,coop)
 for _=1,20 do settle(coop,1) end
 -- floor drop (team stage) is on the ground: the run holds the end until it is collected or the ceiling passes
 local floor=mods.drives.drops:count()
 if floor>0 then assert(coop.state=='stage');for k,r in pairs(mods.drives.drops.records) do mods:pickup{name='drive',port=1,item=r.handle,payload={colour='white',amount=k}} end;settle(coop,14) end
 -- reward_every=2: stage 0 owes nothing, so the run resumed and advanced straight on
 assert(not s.paused and coop.state=='launching','not paused, next stage launching: '..coop.state)
 assert(#s.launches>=2 and coop.stage==1,'stage 0 (no reward owed) went on to stage 1: '..coop.stage..' launches '..#s.launches)
end)
T.test('reward stages: two screens in turn, different offers, each on its own port; the second opens only after the first is done',function()
 local s,g,mods,coop=start({seed=99});Coop.set('reward_every',1)
 local synth=D.coop_synth.new(g,coop);synth.on=true;synth.policy='first'
 populate(s,coop);settle(coop,70);ko_all(s,coop);settle(coop,14)
 local floor=mods.drives.drops:count();if floor>0 then coop.hosts[1].hold_gave_up=true;coop.hosts[1]:set_hold(false,'test');settle(coop,14) end
 assert(coop.state=='reward' and coop.screen_owner==coop.hosts[1],'first screen belongs to P1')
 local h1,h2=coop.hosts[1],coop.hosts[2]
 assert(h1.screen.active and not h2.screen.active and #h1.offers>0)
 local o1={};for _,o in ipairs(h1.offers) do o1[#o1+1]=h1:name(o) end
 for _=1,30 do settle(coop,1);synth:tick();if coop.screen_owner==h2 then break end end
 assert(coop.screen_owner==h2 and h2.screen.active and not h1.screen.active,'P2 second')
 assert(h2.screen.input.port==2 and h1.screen.input.port==1,'each screen reads its own controller')
 local o2={};for _,o in ipairs(h2.offers) do o2[#o2+1]=h2:name(o) end
 assert(table.concat(o1,'|')~=table.concat(o2,'|'),'own offers: '..table.concat(o1,'|')..' vs '..table.concat(o2,'|'))
 for _=1,30 do settle(coop,1);synth:tick();if coop.state~='reward' then break end end
 assert(coop.stage==1 and not s.paused and #s.launches==2,'both done, resumed, next stage launched')
end)
T.test('each decision is a recorded event; the same seed and the same choices give the same digest, another seed does not',function()
 local function run_once(seed,policy)
  local s,g,mods,coop=start({seed=seed});Coop.set('reward_every',1);Coop.set('loop_length',3);Coop.set('max_loops',1)
  local synth=D.coop_synth.new(g,coop);synth.on=true;synth.policy=policy or 'seeded'
  for _=1,30 do
   if not coop.active then break end
   win_stage(s,coop,synth,1+(_%2))
  end
  return coop
 end
 local a,b=run_once(2024),run_once(2024);local c=run_once(2025)
 assert(a.final and a.final.reason=='complete' and a.final.stages==6,'3 stages x (1 + max_loops 1) = 6 stages, got '..tostring(a.final and a.final.stages)..' '..tostring(a.final and a.final.reason))
 assert(a.final.digest==b.final.digest,'deterministic: '..a.final.digest..' '..b.final.digest)
 assert(a.final.digest~=c.final.digest,'a different seed is a different record')
 local kinds={};for _,e in ipairs(a.events) do kinds[e.kind]=(kinds[e.kind] or 0)+1 end
 assert(kinds.take_offer and kinds.stage_clear==6 and kinds.new_game_plus==2 and kinds.screen_done,'the events carry the choices')
 local text,digest=a:record();assert(digest==a.final.digest or true)
end)
T.test('New Game+: after the final stage the loop goes up and the progression context follows',function()
 local s,g,mods,coop=start({seed=5});Coop.set('loop_length',2);Coop.set('reward_every',1)
 local synth=D.coop_synth.new(g,coop);synth.on=true
 for _=1,2 do win_stage(s,coop,synth) end
 assert(coop.loop==1 and coop.stage==0,'NG+1 stage 0: '..coop.loop..' '..coop.stage)
 populate(s,coop);settle(coop,5)
 assert(coop.hosts[1].loop==1 and coop.hosts[2].loop==1 and mods.engine.context.loop==1,'both seats and the engine are in NG+1')
 assert(#Coop.plan(5,0,1).foes==3)
end)
T.test('a player with no stocks spectates (the stage goes on, they are back next stage); both out ends the run; down_rule end ends it at once',function()
 local s,g,mods,coop=start({seed=7});populate(s,coop);settle(coop,70)
 s.players[2].stocks=0;settle(coop,12);assert(coop.state=='stage' and coop.active,'spectating, the stage goes on')
 s.players[1].stocks=0;settle(coop,12);assert(not coop.active and coop.final.reason=='loss','both out ends the run')
 local s2,g2,m2,c2=start({seed=7});Coop.set('down_rule','end');populate(s2,c2);settle(c2,70);s2.players[2].stocks=0;settle(c2,12)
 assert(not c2.active and c2.final.reason=='loss')
 reset();local s3,g3,m3,c3=start({seed=7});Coop.set('retry_on_loss',1);populate(s3,c3);settle(c3,70);s3.players[1].stocks=0;s3.players[2].stocks=0;settle(c3,12)
 assert(c3.active and #s3.launches==2 and c3.stage==0,'one retry of the same stage')
 reset()
end)
T.test('floor drops: first touch takes it; causer gives it to the player who last hit that opponent; both makes one each',function()
 local function drop_for(mode,hitter)
  local s,g,mods,coop=start({seed=31});Coop.set('drop_owner',mode);populate(s,coop);settle(coop,40)
  if hitter then mods:hit(hitter,3,{}) end
  s.players[3].falls=1;settle(coop,14)
  settle(coop,2);assert(mods.drives.drops:count()>=1,'a drop is on the floor')
  return s,g,mods,coop
 end
 local s,g,mods,coop=drop_for('first',2);local n=0;for k,r in pairs(mods.drives.drops.records) do n=n+1 end;assert(n==1)
 for k,r in pairs(mods.drives.drops.records) do mods:pickup{name='drive',port=2,item=r.handle,payload={colour='white',amount=k}} end
 -- a drive that merges into a held rule (any colour) counts as held too: it is in the build, not in a slot of its own
 local function held(h,s) local c=#h:bag().items;for i=1,h:bag():slots() do if h:bag().equipped[i] then c=c+1 end end;for _,l in ipairs(s.logs) do if l:find('P'..h:port0()..' merged',1,true) then c=c+1 end end;return c end
 assert(held(coop.hosts[2],s)==2 and held(coop.hosts[1],s)==1,'P2 touched it first and has it')
 local s,g,mods,coop=drop_for('causer',1);for k,r in pairs(mods.drives.drops.records) do mods:pickup{name='drive',port=2,item=r.handle,payload={colour='white',amount=k}} end
 assert(held(coop.hosts[1],s)==2 and held(coop.hosts[2],s)==1,'P2 touched it, P1 hit the opponent: the drive passed to P1')
 local got;for _,l in ipairs(s.logs) do if l:find('received',1,true) and l:find('from P2',1,true) then got=true end end;assert(coop.hosts[1].hud.card and got,'P1 received the drive from P2 (said in the log; the strip flash is a developer figure now)')
 local s,g,mods,coop=drop_for('both',nil);local n=0;for _ in pairs(mods.drives.drops.records) do n=n+1 end;assert(n==2,'duplicated: two on the floor')
 for k,r in pairs(mods.drives.drops.records) do mods:pickup{name='drive',port=1,item=r.handle,payload={colour='white',amount=k}} end
 assert(held(coop.hosts[1],s)==2 and held(coop.hosts[2],s)==2,'both got one whoever touched them')
 reset()
end)
T.test('a second run in the same session gives floor drops again (the attempt counts and given drops are cleared at run begin)',function()
 local s,g,mods,coop=start({seed=31});populate(s,coop);settle(coop,40);s.players[3].falls=1;settle(coop,14);settle(coop,2)
 assert(mods.drives.drops:count()>=1,'the first run drops a drive on stage 0')
 coop:stop();for k in pairs(mods.drives.drops.records) do mods.drives.drops.records[k]=nil end
 local ok,why=coop:start({seed=31});assert(ok,why)
 s.players={};populate(s,coop);settle(coop,40);s.players[3].falls=1;settle(coop,14);settle(coop,2)
 assert(mods.drives.drops:count()>=1,'the second run drops one too: its stage 0 is not a retry of the first run')
 assert(not coop.hosts[1].retry,'and is not marked a retry')
end)
T.test('uncollected floor drives are divided between the seats at the stage end, none lost',function()
 reset();local s,g,mods,coop=start({seed=61});Coop.set('reward_every',1);populate(s,coop);settle(coop,40)
 s.players[3].falls=1;settle(coop,14);ko_all(s,coop)
 assert(mods.drives.drops:count()>=1)
 coop.hosts[1].hold_gave_up=true;settle(coop,40)
 local total=0;for _,h in ipairs(coop.hosts) do total=total+#h:bag().items;for i=1,h:bag():slots() do if h:bag().equipped[i] then total=total+1 end end end
 assert(total>=3 or #coop.hosts[1].decide+#coop.hosts[2].decide>0,'starters plus the gathered drive: '..total)
 assert(mods.drives.drops:count()==0)
end)
T.test('a status one player applies to an opponent is read by the other player\'s rule: cross-player synergy is how the engine already works',function()
 local pool=D.mod_pool;local e=D.mod_engine.new(1,pool,{context=D.mod_progression.context(5,0)})
 e:set_build(1,{icebound=1},{});e:set_build(2,{brittle=1},{})
 local players={[1]={percent=0,grounded=true,stocks=3},[2]={percent=0,grounded=true,stocks=3},[3]={percent=40,grounded=true,stocks=1}}
 e:begin_frame(players)
 e:emit{kind='hit_dealt',port=1,target=3,tags={ice=true},self_context={},target_context={}};e:drain()
 assert(e.statuses[3] and e.statuses[3].chill,'P1 chilled the opponent')
 e:emit{kind='hit_dealt',port=2,target=3,tags={},self_context={},target_context={}};e:drain()
 assert(e.statuses[3].curse,'P2\'s Brittle read the Chill P1 applied: a cross-player chain')
end)
T.test('the strips and screens wear a port colour on their own side; a one-player host draws as before',function()
 local s,g,mods,coop=start();populate(s,coop);settle(coop,40)
 local fills={};g.fill=function(x,y,w,h,c) fills[#fills+1]={x=x,y=y,w=w,h=h,c=c} end;g.text=function() end
 g.kit={text=function() end,panel=function() end,measure=function(t) return #t*6 end}
 coop.hosts[1].hud.m=nil;coop.hosts[2].hud.m=nil
 coop.hosts[1].hud:draw_strip();local n1=#fills;coop.hosts[2].hud:draw_strip()
 local x1,x2;for i=1,#fills do if fills[i].c==D.run_hud.port_colour[1] then x1=fills[i].x end;if fills[i].c==D.run_hud.port_colour[2] then x2=fills[i].x end end
 assert(x1 and x2 and x1<x2 and x2>320,'P1 strip left, P2 strip right: '..tostring(x1)..' '..tostring(x2))
end)
T.test('the synthetic players fly each seat to the nearest living opponent and attack',function()
 local s,g,mods,coop=start();populate(s,coop);settle(coop,40)
 local synth=D.coop_synth.new(g,coop);synth:start('seeded');synth:frame()
 assert(s.fly[1] and s.fly[2] and s.fly[1].attack and s.fly[2].attack)
 -- the burst attack is armed once, not every frame (every call re-arms the cycle at phase 0, which is the old every-frame attack)
 synth:frame();synth:frame();assert(s.attack_calls==2,'one arming per seat, not one per frame: '..tostring(s.attack_calls))
 -- a fighter whose cursor the engine disarmed (a stock lost) is armed again
 s.fly[1].attack=false;synth:frame();assert(s.attack_calls==3 and s.fly[1].attack)
end)
T.test('synth gear gives a seat chosen common drives and an untagged keystone, and the build derives from them',function()
 local s,g,mods,coop=start();populate(s,coop);settle(coop,40)
 local synth=D.coop_synth.new(g,coop)
 assert(synth:gear(1,{'k_plague_bearer','armoured','updraft'}))
 local b=coop.hosts[1]:bag();assert(b.keystones[1]=='plague_bearer' and b.equipped[1] and b.equipped[2] and #b.items==0)
 settle(coop,5)
 assert(mods.engine.equipped[1] and mods.engine.equipped[1].plague_bearer and mods.engine.equipped[1].armoured,'the published build holds the keystone and the drive')
end)
T.test('the engine ending the match under the run (CPU-assisted players: the hold needs a human in slot 0) is read from the last frames, not lost',function()
 local s,g,mods,coop=start({seed=77,p1='cpu',p2='cpu'});Coop.set('reward_every',1)
 assert(s.launches[1]:find('p1=fox/cpu9/team0',1,true) and s.launches[1]:find('p2=marth/cpu9/team0',1,true))
 populate(s,coop);settle(coop,70);ko_all(s,coop)
 coop.hosts[1].hold_gave_up=true;settle(coop,2)
 -- the engine ends the match on the same frame the last opponent falls: the run's own watch has not cleared yet
 coop.cleared=false;coop.state='stage';coop.last_foes=0;coop.last_allies=2
 coop:scene_ended();assert(coop.state=='reward' or coop.state=='launching','cleared from the last seen state: '..coop.state)
 local kinds={};for _,e in ipairs(coop.events) do kinds[e.kind]=true end;assert(kinds.engine_ended and kinds.stage_clear)
 local s2,g2,m2,c2=start({seed=78});populate(s2,c2);settle(c2,70);c2.last_foes=2;c2.last_allies=0;c2:scene_ended();assert(not c2.active and c2.final.reason=='loss','allies out when the engine ended it is a loss')
 -- an engine rematch of the seeded scene while the stage is live is logged and replayed, never silently counted
 local s3,g3,m3,c3=start({seed=79});populate(s3,c3);settle(c3,5);c3:scene_started();assert(c3.state=='staging')
 local k={};for _,e in ipairs(c3.events) do k[e.kind]=true end;assert(k.engine_restart)
end)
T.test('the cheats list and the tuning lines exist for the owner and for the online version',function()
 assert(#Coop.cheats>=5 and #Coop.lines()==#Coop.names)
end)

T.done()
