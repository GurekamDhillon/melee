-- The stage-end payout and the press-A drive (drive_drop.lua R.payout, run_host.lua H:update_payout_hold). Pure accounting
-- (what a stage owes, a fast clear, a retry, a loss), the spots and the pacing, the item definitions and who may take a drive,
-- and the hold's release conditions (collected, leave, away, player out) against a fake `gd`. Nothing here needs the game; what does
-- (the A press itself, the retail pickup states, the flourish on screen) is on the proofs-owed list in the payout PROGRESS.md.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_engine','keystones','mod_pool','drive_loot','drive_merge','drive_economy','drive_bag','foe_roll'}) do D[n]=T.module(n,D) end
D.grid=dofile(T.root..'../../demos/grid-inventory/scripts/grid.lua')
D.mod_display={new=function(g,engine) local v={engine=engine};function v:warm() return true end;function v:update() end;function v:clear() end;function v:on_loadstate(e) self.engine=e end;function v:tick() end;return v end}
D.pickup_juice={pitch={},new=function()return {drop=function()return {fx={}}end,collect=function()end,expire=function()end,clear=function()end,tick=function()end}end}
for _,n in ipairs({'menu_input','drive_menu','drive_text','drive_drop','drive_lab','foe_lab','mod_lab','run_screen','run_hud','run_host'})do D[n]=T.module(n,D)end
D.drive_economy.tuning.floor_chance=1
local P=D.drive_drop.payout
-- ---- the accounting ---------------------------------------------------------------------------------------------------------
T.test('what a stage owes: a battle stage one decision until one is made, a team stage one per opponent up to the cap, none on bonus/boss/retry',function()
 local function names(l) return table.concat(l,',') end
 assert(names(P.owed_foes('battle',1,{2,3},{kos=0}))=='2','a fast clear: nothing was decided, the stage owes its one decision')
 assert(#P.owed_foes('battle',1,{2},{kos=1})==0,'a decision was made mid-fight (even one that rolled no drive): nothing is owed')
 assert(names(P.owed_foes('team',3,{2,3,4},{kos=1,dropped={[2]=true}}))=='3,4','team: the opponents without their drop, up to the cap')
 assert(names(P.owed_foes('team',1,{2,3,4},{kos=0,dropped={}}))=='2','team cap 1: one in all')
 assert(#P.owed_foes('team',1,{2,3},{kos=1,dropped={[2]=true}})==0,'team cap reached')
 assert(#P.owed_foes('bonus',1,{},{kos=0})==0 and #P.owed_foes('boss',1,{2},{kos=0})==0,'bonus and boss owe nothing')
 assert(#P.owed_foes('battle',1,{2},{kos=0,given=true})==0,'a retry: the drop already given is not given twice')
 assert(#P.owed_foes('battle',0,{2},{kos=0})==0,'a stage whose cap is 0 owes nothing')
end)
T.test('pacing: about a second for the last drive, quicker the more are still to come; the sequence skips what has not arrived',function()
 assert(P.gap(1)==60 and P.gap(2)<P.gap(1) and P.gap(3)<P.gap(2) and P.gap(9)<=P.gap(4))
 local s=P.seq();s:add({record=1});s:add({record=2});s:add({record=3})
 local out,at={},{};for f=1,400 do local e=s:step();if e then out[#out+1]=e.record;at[#at+1]=f end end
 assert(#out==3 and out[1]==1 and out[3]==3,'arrive in order')
 assert(at[1]==P.tuning.lead+1,'the first arrives after the lead '..at[1])
 assert(at[2]-at[1]==P.gap(3)+1 and at[3]-at[2]==P.gap(2)+1,'gaps shrink with the count to come: '..(at[2]-at[1])..' '..(at[3]-at[2]))
 local s2=P.seq();s2:add({record=1});s2:add({record=2});assert(s2:pending()==2);for _=1,P.tuning.lead+1 do s2:step() end
 local left=s2:skip();assert(#left==1 and left[1].record==2 and s2:pending()==0,'what has not arrived is returned, not lost')
end)
T.test('spots: spread across the main floor, centre first, never over a pit or on a platform, nudged off an edge',function()
 local area={lo=-100,hi=100,y=0,tol=10,floor=function(x) return 0 end}
 local s=P.spots(3,area);assert(#s==3 and s[1].x==0,'centre first, got '..tostring(s[1] and s[1].x))
 local xs={s[1].x,s[2].x,s[3].x};table.sort(xs);assert(xs[1]==-50 and xs[3]==50,'evenly spread')
 assert(#P.spots(1,area)==1 and P.spots(1,area)[1].x==0)
 -- a pit in the middle: the spot moves to the nearest floor
 local pit={lo=-100,hi=100,y=0,tol=10,floor=function(x) if x>-20 and x<20 then return nil end;return 0 end}
 local p=P.spots(1,pit)[1];assert(p and math.abs(p.x)>=20,'not over the pit, got '..tostring(p and p.x))
 -- a high platform is not the main floor
 local plat={lo=-100,hi=100,y=0,tol=10,floor=function(x) if x>-30 and x<30 then return 60 end;return 0 end}
 p=P.spots(1,plat)[1];assert(p and p.y==0 and math.abs(p.x)>=30,'not on the platform')
 -- no floor anywhere: no spots (the host falls back to beside the player)
 assert(#P.spots(2,{lo=0,hi=50,y=0,tol=10,floor=function() return nil end})==0)
 assert(#P.spots(0,area)==0 and #P.spots(2,nil)==0)
end)
T.test('area: the main floor inside its edges; a scrolling stage (a floor wider than span_max or none) is a window around the player',function()
 local calls={}
 local g={stage_bounds=function() return {main_floor={left=-200,right=200,top=5,bottom=5},camera={left=-1,right=1}} end,floor_below=function(x,y) calls[#calls+1]={x,y};return 5 end}
 local a=P.area(g,{x=0,y=5});assert(a.lo==-200+P.tuning.edge and a.hi==200-P.tuning.edge and a.y==5 and a.floor(10)==5)
 g.stage_bounds=function() return {main_floor={left=-3000,right=3000,top=5,bottom=5}} end
 a=P.area(g,{x=1000,y=5});assert(a.lo>800 and a.hi<1200,'a window around the player on a long stage')
 g.stage_bounds=function() return nil end;a=P.area(g,{x=40,y=2});assert(a and a.lo==40-P.tuning.near_span and a.y==2)
 assert(P.area(g,nil)==nil)
end)
-- ---- who may take a drive, and the definitions ----------------------------------------------------------------------------
local function read(path) local f=assert(io.open(T.root..'../items/'..path,'rb'));local s=f:read('a');f:close();return (s:gsub('%s','')) end
T.test('item definitions: press drives are press, payout drives do not fade, the touch drives are unchanged, all differ only in the named fields',function()
 local base=read('drive/item.json');assert(base:find('"collection":"touch"',1,true),'the old drive is still touch (the missions and the LAB use it)')
 assert(read('drive_coop/item.json'):find('"collection":"touch"',1,true))
 local function variant(name,ports,life,blink)
  local s=base:gsub('"name":"drive"','"name":"'..name..'"'):gsub('"collection":"touch"','"collection":"press"'):gsub('"ports":1','"ports":'..ports):gsub('"lifetime":900','"lifetime":'..life):gsub('"blink":120','"blink":'..blink)
  return s
 end
 assert(read('drive_press/item.json')==variant('drive_press',1,900,120),'drive_press drifted from drive')
 assert(read('drive_press_coop/item.json')==variant('drive_press_coop',3,900,120),'drive_press_coop drifted from drive')
 assert(read('drive_payout/item.json')==variant('drive_payout',1,36000,0),'drive_payout drifted from drive')
 assert(read('drive_payout_coop/item.json')==variant('drive_payout_coop',3,36000,0))
end)
T.test('which engine item a drop is: touch unless the run host asked for press; co-op keeps its own masks; every name collects',function()
 local R=D.drive_drop;local made=0;local last
 local g={item_spawn=function(name,x,y) made=made+1;last=name;return 10+made end,player=function() return {x=0,y=0} end}
 local d=R.new(g)
 d:spawn({colour='red',rarity='common',affixes={}},0,0);assert(last=='drive','the LAB and missions: touch')
 d.press=true;d:spawn({colour='red',rarity='common',affixes={}},0,0);assert(last=='drive_press')
 d:spawn({colour='red',rarity='common',affixes={}},0,0,{payout=true});assert(last=='drive_payout')
 d.item_name='drive_coop';d:spawn({colour='red',rarity='common',affixes={}},0,0);assert(last=='drive_press_coop')
 d:spawn({colour='red',rarity='common',affixes={}},0,0,{payout=true});assert(last=='drive_payout_coop')
 for name in pairs({drive_press=1,drive_press_coop=1,drive_payout=1,drive_payout_coop=1}) do assert(R.names[name]) end
 -- ownership at the Lua side: only the port the host asked for, and only a drive the host still knows
 local d2=R.new({item_spawn=function() return 77 end,player=function() return {x=0,y=0} end});d2.press=true
 local rec={colour='red',rarity='common',affixes={}};d2:spawn(rec,0,0)
 assert(d2:pickup({name='drive_press',port=2,item=77,payload={amount=1}},nil,true,1)==false,'another port cannot take it')
 assert(d2:pickup({name='drive_press',port=1,item=999,payload={amount=1}},nil,true,1)==false,'an unknown item is not a drive')
 assert(d2:pickup({name='drive_press',port=1,item=77,payload={amount=1}},nil,true,1)==rec,'the owner takes it')
 local d3=R.new({item_spawn=function() return 78 end,player=function() return {x=0,y=0} end});d3.press=true;d3:spawn(rec,0,0,{payout=true})
 assert(d3:validate(d3:snapshot())==1 and d3.records[1].payout==true,'a payout drive survives a checkpoint')
end)
-- ---- the host: the hold and its release ---------------------------------------------------------------------------------------
local function fixture()
 local s={logs={},pad={},spawns={},held={},players={[1]={x=0,y=0,percent=0,stocks=3,falls=0,char=1,action=14},[2]={x=30,y=0,percent=0,stocks=1,falls=0,char=2,action=14,cpu=true}},fx={},clock=100}
 local n=0
 local g={fixture=s,command=function() end,log=function(t)s.logs[#s.logs+1]=t end,
  match=function()return {active=true,netplay=false,stage=32}end,lab_mode=function()return false end,
  sim_supported=true,sim_replaying=function()return false end,player=function(p)return s.players[p]end,
  hit_rule_add=function()error('direct native write')end,fighter_status=function()error('direct native write')end,
  hit_rules=function()return {percent_only=true,progression=true,owner=7}end,
  sim_clear=function()s.blob=nil end,sim_read=function()return s.blob end,sim_commit=function(b,o)s.blob=b;return true end,
  pad=function()return s.pad end,input_mask=function() end,paused=function()return false end,pause=function()end,resume=function()end,items=function()return {}end,
  item_spawn=function(name,x,y,o) n=n+1;s.spawns[#s.spawns+1]={name=name,x=x,y=y,handle=100+n};s.payload=o.payload;return 100+n end,item_despawn=function() return true end,
  match_end_hold=function(r,on) s.held[r]=on or nil;return true end,
  fx_world=function(name,x,y) s.fx[#s.fx+1]={name=name,x=x,y=y};return 500+#s.fx end,fx_end=function(h) s.fx_ended=(s.fx_ended or 0)+1 end,
  hold_1p=function() return true end,release_1p=function() return true end,mode_1p=function() return {held=true,mode='classic'} end,time=function() return s.clock end,
  safe_area=function() return {x=0,y=0,w=640,h=480,right=640,bottom=480} end,
  stage_bounds=function() return {main_floor={left=-120,right=120,top=0,bottom=0},camera={left=-200,right=200,top=100,bottom=-100}} end,
  floor_below=function(x,y) if x>-15 and x<15 then return nil end;return 0 end}
 local retail={state={player_port=1},loop=0,rules=true,active=true}
 local mods=D.mod_lab.new(g,{run_host=function() return true end,run_ready=function() return true end})
 local host=D.run_host.new(g,mods,retail);retail.host=host
 return s,g,mods,host
end
local function start(kind,opp,seed)
 local s,g,mods,host=fixture();host:run_begin(seed or 4242)
 host:stage_start({stage_index=0,loop=0,stage_kind=kind or 'battle',opponents=opp or {{port=2}}});host.since=99
 return s,g,mods,host
end
local function frames(host,n) for _=1,n do host:frame() end end
local function held(s) return s.held and s.held['envoy-drives']==true end
local function has(s,text) for _,l in ipairs(s.logs) do if l:find(text,1,true) then return true end end;return false end
local function drives(host) local b=host:bag();local n=#b.items;for i=1,b:slots() do if b.equipped[i] then n=n+1 end end;return n end
T.test('a fast clear: the stage end is held from the start, the last opponent falls with no KO seen, and the owed drive arrives with its flourish and is collected with A',function()
 local s,g,mods,host=start();local before=drives(host)
 frames(host,12);assert(held(s),'the end is held on every battle stage from the start, with no drive on the floor')
 assert(host.drops==0 and not host.hold_banner,'nothing to say yet')
 s.players[2].stocks=0;frames(host,6) -- the stock went to 0 with no stock loss ever polled: today this stage owed a drive and lost it
 assert(host.paying and has(s,'1 drive decision(s) owed'),'the clear is noticed and the debt computed')
 assert(#s.spawns==0,'it waits its lead before the first arrival')
 frames(host,60);assert(#s.spawns==1,'the owed drive arrived once')
 local sp=s.spawns[1];assert(sp.name=='drive_payout','a payout drive collected with A that does not fade (press was set by the run)')
 assert(sp.y>=D.drive_drop.payout.tuning.arrive_height and math.abs(sp.x)>=15 and math.abs(sp.x)<=120,'it comes in from above, on the main floor, not over the pit: '..sp.x..','..sp.y)
 assert(#s.fx>=2 and s.fx[1].name:find('pool',1,true) and s.fx[2].name:find('collect_burst',1,true),'the flourish: a ring and a burst in the drive colour')
 assert(held(s) and host.hold_banner=='Collect the drives','held while the drive waits')
 mods:pickup{name='drive_payout',port=1,item=sp.handle,payload=s.payload}
 frames(host,6);assert(not held(s) and has(s,'match end released: every drive collected'),'released when it is taken')
 assert(drives(host)==before+1 or has(s,'(merge)'),'it went through the same gain rule as any pickup: a new drive, or a merge into a held rule')
 for _=1,60 do host:frame() end;assert((s.fx_ended or 0)>=2,'the transient effects ended themselves')
end)
T.test('nothing owed: a decision already made mid-fight (or a retry) releases the held end at once',function()
 local s,g,mods,host=start();host:on_ko(2,'passed 50%');host.drop_queue={} -- decided mid-fight, its drive already taken
 frames(host,12);assert(held(s));s.players[2].stocks=0;frames(host,6);assert(not held(s) and #s.spawns==0 and has(s,'0 drive decision(s) owed'),'no payout, no wait')
 local s2,g2,m2,h2=start();h2.drops_given={['0:0']=true};h2.retry=true;s2.players[2].stocks=0;frames(h2,12)
 assert(not held(s2) and #s2.spawns==0 and has(s2,'a retry'),'a retry gives nothing twice')
end)
T.test('a stage that rolled no drive (chance) owes nothing more: the roll was the decision',function()
 D.drive_economy.tuning.floor_chance=0.0001
 local s,g,mods,host=start();frames(host,12);s.players[2].stocks=0;frames(host,12)
 D.drive_economy.tuning.floor_chance=1
 assert(not held(s) and #s.spawns==0,'no drive, no hold')
end)
T.test('leaving: the chord held for a second skips what is left (nothing is gained), and a tap does not',function()
 local s,g,mods,host=start('battle',nil,31);local before=drives(host)
 frames(host,12);s.players[2].stocks=0;frames(host,60);assert(#s.spawns==1)
 s.pad={Z=true,DOWN=true,buttons=0x14};frames(host,30);assert(held(s),'half a second is not leaving')
 s.pad={buttons=0};frames(host,6);s.pad={Z=true,DOWN=true,buttons=0x14};frames(host,30);assert(held(s),'a released chord starts again')
 frames(host,40);assert(not held(s) and has(s,'match end released: the player left') and has(s,'payout: skipped'))
 assert(drives(host)==before and host.mods.drives.drops:count()==0,'skipped: not gained, not left on the floor for the clear to gather')
 host:stage_reward(0,0,false);assert(drives(host)==before,'the clear does not hand back what was skipped')
end)
T.test('leaving before a drive has arrived skips it too, and a second drive on the way does not arrive after',function()
 local s,g,mods,host=start('team',{{port=2},{port=3}},5);s.players[3]={x=40,y=0,stocks=1,falls=0,percent=0,char=2,cpu=true}
 D.drive_economy.tuning.team_max=2
 frames(host,12);s.players[2].stocks=0;s.players[3].stocks=0;frames(host,6);assert(host.seq:pending()==2)
 s.pad={Z=true,DOWN=true,buttons=0x14};frames(host,66);assert(not held(s) and #s.spawns<=1 and host.seq:pending()==0)
 frames(host,200);assert(#s.spawns<=1,'nothing arrives after leaving')
 D.drive_economy.tuning.team_max=1
end)
T.test('the player walking away: after the idle limit what is left is gathered, as it always was; there is no ceiling for a player who plays',function()
 local s,g,mods,host=start('battle',nil,32);local before=drives(host)
 frames(host,12);s.players[2].stocks=0;frames(host,60);assert(#s.spawns==1)
 s.pad={buttons=0};frames(host,7000);assert(held(s),'an idle pad for under the limit is still holding')
 s.pad={buttons=0,x=60};frames(host,200) ;assert(held(s));s.pad={buttons=0};frames(host,7300)
 assert(not held(s) and has(s,'the player is away'),'released once away')
 host:stage_reward(0,0,false);assert(drives(host)==before+1,'the gathered drive is gained at the clear')
end)
T.test('the player is out during the hold: no payout, the stage is lost the normal way; nothing is gained',function()
 local s,g,mods,host=start();local before=drives(host);frames(host,12);assert(held(s))
 s.players[2].stocks=0;frames(host,6);assert(host.seq:pending()==1)
 s.players[1].stocks=0;frames(host,6);assert(not held(s) and host.seq:pending()==0 and has(s,'the player is out') and #s.spawns==0)
 assert(drives(host)==before)
end)
T.test('boss and bonus stages are never held (the boss end is the game\'s own); the clear releases the hold',function()
 for _,k in ipairs({'boss','bonus'}) do local s,g,mods,host=start(k,k=='bonus' and {} or {{port=2}});frames(host,30);assert(not held(s),k..' is not held') end
 local s,g,mods,host=start();frames(host,12);assert(held(s));host:stage_reward(0,0,false);assert(not held(s))
 local s2,g2,m2,h2=start();h2:run_end();assert(not h2.paying and h2.mods.drives.drops.press==nil,'the run end clears the press flag')
end)
T.test('a stage with opponents the host never registered is not held (a retail stage the game ends its own way)',function()
 local s,g,mods,host=start('battle',{});s.players[2]=nil;frames(host,30);assert(not held(s))
end)
T.test('a payout with the ground full or a refused spawn retries, then gathers; arrivals do not strand the hold',function()
 local s,g,mods,host=start();frames(host,12);s.players[2].stocks=0;frames(host,6)
 local refuse=true;local real=g.item_spawn;g.item_spawn=function(...) if refuse then return nil,'refused' end;return real(...) end
 frames(host,900);assert(host.seq:pending()==0 and #host.faded==1 and has(s,'could not arrive'),'after 60 refusals it is gathered at the clear')
 assert(not held(s),'and the hold ends')
end)
T.test('co-op: the shared drive is the coop press item and is claimed by whichever player presses A first',function()
 local s,g,mods,host=start();host.mods.drives.drops.item_name='drive_coop'
 frames(host,12);s.players[2].stocks=0;frames(host,66);assert(s.spawns[1].name=='drive_payout_coop')
end)
T.test('the older hold (payout=false) is untouched: only while a drive is on the floor',function()
 D.run_host.tuning.payout=false
 local s,g,mods,host=start();frames(host,12);assert(not held(s),'no drive, no hold in the old mode')
 host:on_ko(2);frames(host,12);assert(held(s));D.run_host.tuning.payout=true
end)
-- ---- teammates, the engine's end signal, a payout that runs once, the stale-hold watchdog (2026-10-06 team-stage softlock) ----------
-- A retail Classic TEAM stage gives the human a CPU teammate; the giant stage two. They are allies: never a foe, never rolled, never owed.
local function cpu(team,stocks) return {x=40,y=0,percent=0,stocks=stocks or 1,falls=0,char=2,action=14,cpu=true,team=team} end
local function count(s,text) local n=0;for _,l in ipairs(s.logs) do if l:find(text,1,true) then n=n+1 end end;return n end
-- P1 (team 0) with teammate(s) on team 0 and opponents on team 1, the engine reporting teams; `opp` is the retail opponent list the engine builds.
local function team_stage(kind,mates,foes,withteam)
 local opp={};for _,p in ipairs(foes) do opp[#opp+1]={port=p} end
 local s,g,mods,host=start(kind,opp,77)
 s.players[1].team=withteam and 0 or nil;s.players[2]=nil
 for _,p in ipairs(mates) do s.players[p]=cpu(withteam and 0 or nil) end
 for _,p in ipairs(foes) do s.players[p]=cpu(withteam and 1 or nil) end
 return s,g,mods,host,opp
end
local function restage(host,kind,opp) host:stage_start({stage_index=1,loop=0,stage_kind=kind,opponents=opp});host.since=99 end
T.test('a retail teammate (the engine reports teams): never registered, rolled, owed or watched; the stage end counts only the opponents',function()
 local s,g,mods,host,opp=team_stage('team',{2},{3,4},true)
 restage(host,'team',opp)
 assert(has(s,'ally rule for this team stage: engine team') and has(s,"P2 is the player's teammate (engine team 0"),'the rule is logged')
 host:spawn({port=2});host:spawn({port=3});host:spawn({port=4})
 frames(host,12)
 assert(#host.foe_ports==2 and not host.rolls[2] and not host.rolled[2],'the teammate is not an opponent: no registration, no roll')
 assert(not has(s,'opponent P2 joined'),'and is not logged as one')
 assert(#host:foes_list()==2 and host:is_ally(2) and not host:is_ally(3))
 assert(not host:foes_out());s.players[3].stocks=0;assert(not host:foes_out())
 s.players[4].stocks=0;assert(host:foes_out(),'both opponents out while the teammate lives: the stage is over by the foes')
 -- the debt covers the two opponents, never the teammate
 host:begin_payout();assert(count(s,'drive decision(s) owed')==1)
 -- the out-of-bounds watchdog leaves the teammate alone (it only watches the player and the opponents)
 s.players[2].x=99999;for _=1,600 do host:oob_watch() end;assert(not has(s,'P2 is out of bounds'),'allies are not policed')
end)
T.test('a retail teammate on an engine with no team read: retail\'s opponent list for the stage leaves it out, the fallback is logged',function()
 local s,g,mods,host,opp=team_stage('team',{2},{3,4},false)
 restage(host,'team',opp)
 assert(has(s,"fallback rule: the engine reports no team") and has(s,'ally rule for this team stage: retail opponent list'))
 host:spawn({port=2});host:spawn({port=3});frames(host,12)
 assert(host:is_ally(2) and not host:is_ally(3) and #host.foe_ports==2 and not host.rolls[2])
 s.players[3].stocks=0;s.players[4].stocks=0;assert(host:foes_out(),'the stock count ignores the teammate')
end)
T.test('every stage kind with a non-opponent fighter: giant (two allies), metal, battle and team: allies never count, the end begins by the engine signal',function()
 for _,k in ipairs({'giant','metal','battle','team'}) do
  local mates=(k=='giant') and {2,3} or (k=='team') and {2} or {}
  local foes=(k=='giant') and {4} or (k=='team') and {3,4} or {3}
  local s,g,mods,host,opp=team_stage(k,mates,foes,true);restage(host,k,opp)
  s.pending=nil;g.match_end_pending=function() return s.pending end
  for _,p in ipairs(mates) do host:spawn({port=p}) end;for _,p in ipairs(foes) do host:spawn({port=p}) end
  frames(host,12);assert(held(s),k..': held');assert(#host.foe_ports==#foes,k..': only the real opponents are foes ('..#host.foe_ports..')')
  for _,p in ipairs(mates) do assert(host:is_ally(p) and not host.rolled[p],k..': P'..p..' is an ally') end
  for _,p in ipairs(foes) do s.players[p].stocks=0 end
  frames(host,12);assert(host.pay_state=='idle' and not has(s,'drive decision(s) owed'),k..': an engine that reports no end yet means retail has not ended: no payout')
  s.pending=3;frames(host,6);assert(host.pay_state~='idle' and has(s,'retail has decided the stage is over: engine outcome 3'),k..': the payout begins on the engine signal')
 end
end)
T.test('the engine signal begins the payout even while the stock count says an opponent lives (retail decides, not the mod)',function()
 local s,g,mods,host=start();s.pending=nil;g.match_end_pending=function() return s.pending end
 frames(host,12);assert(held(s) and host.pay_state=='idle' and s.players[2].stocks==1)
 s.pending=3;frames(host,6);assert(host.pay_state=='arriving' or host.pay_state=='waiting',tostring(host.pay_state))
 assert(has(s,'the last opponent is out: 1 drive decision(s) owed [retail has decided'),'logged with the rule that began it')
end)
T.test('without the engine read the stock count still begins the payout (an older engine), and says so',function()
 local s,g,mods,host=start();assert(g.match_end_pending==nil);frames(host,12);s.players[2].stocks=0;frames(host,6)
 assert(host.paying and has(s,'stock count, this engine has no end signal'))
end)
T.test('a payout runs once per stage: 600 frames after a clear that owes nothing hold exactly one hold and one release (the 2026-10-06 loop)',function()
 local s,g,mods,host=start();host:on_ko(2,'passed 50%');host.drop_queue={}
 frames(host,12);s.players[2].stocks=0;frames(host,600)
 assert(host.pay_state=='done');assert(count(s,'match end held')==1,'one hold, got '..count(s,'match end held'))
 assert(count(s,'match end released')==1,'one release, got '..count(s,'match end released'))
 assert(count(s,'drive decision(s) owed')==1,'one payout, got '..count(s,'drive decision(s) owed'))
 assert(not held(s))
 -- and with a drive paid out and collected: still once
 local s2,g2,m2,h2=start();frames(h2,12);s2.players[2].stocks=0;frames(h2,70)
 m2:pickup{name='drive_payout',port=1,item=s2.spawns[1].handle,payload=s2.payload};frames(h2,600)
 assert(count(s2,'match end held')==1 and count(s2,'match end released')==1 and count(s2,'drive decision(s) owed')==1,'collected: still one of each')
 -- a new stage attempt starts the machine again
 h2:stage_start({stage_index=1,loop=0,stage_kind='battle',opponents={{port=2}}});assert(h2.pay_state=='idle')
end)
T.test('the stale-hold watchdog: the end is held, nothing owed or on the floor, nobody acting for stale_hold_frames: released loudly, never retaken',function()
 local s,g,mods,host=start();frames(host,12);s.players[2].stocks=0
 host.begin_payout=function() end -- a payout that can never begin: the strand this watchdog exists for
 local N=D.run_host.tuning.stale_hold_frames
 frames(host,N-60);assert(held(s),'not yet: '..N..' frames of nothing')
 frames(host,200);assert(not held(s) and has(s,'WATCHDOG'),'released and said so')
 local released=count(s,'match end released');frames(host,600);assert(not held(s) and count(s,'match end held')==1 and count(s,'match end released')==released,'never retaken')
 -- a fighter acting resets the clock
 local s2,g2,m2,h2=start();frames(h2,12);s2.players[2].stocks=0;h2.begin_payout=function() end
 for i=1,3 do frames(h2,N-200);s2.players[2].action=14+i end
 assert(held(s2) and not has(s2,'WATCHDOG'),'a fighter that acts keeps the clock from running out')
end)
T.test('a respawn on a port re-reads its team (Adventure reuses slots)',function()
 local s,g,mods,host,opp=team_stage('battle',{},{3},true);restage(host,'battle',opp)
 host:spawn({port=3});assert(not host:is_ally(3))
 s.players[3].team=0;assert(not host:is_ally(3),'cached for the fighter that was there')
 host:spawn({port=3});assert(host:is_ally(3),'a spawn event reads the port again')
end)
T.test('the out-of-bounds net never takes the last stock (put back instead) and leaves the player of a won boss fight alone; an opponent still loses one',function()
 local s,g,mods,host=start('battle');local set,tp={},{}
 g.set_stocks=function(p,n) set[#set+1]={p,n};s.players[p].stocks=n end;g.teleport=function(p,x,y) tp[#tp+1]={p,x,y};s.players[p].x=x;s.players[p].y=y end
 s.players[1].stocks=1;s.players[1].x=-88;s.players[1].y=-9000;frames(host,600)
 assert(#set==0,'the last stock is not taken');assert(#tp>=1 and has(s,'on the LAST stock: put back on the stage'),'the fighter is put back, said so')
 s.players[1].stocks=3;s.players[1].x=-88;s.players[1].y=-9000;frames(host,600);assert(#set==1 and set[1][2]==2,'a spare stock is still taken')
 local sb,gb,mb,hb=start('boss');local set2={};gb.set_stocks=function(p,n) set2[#set2+1]=n end;gb.teleport=function() set2[#set2+1]='tp' end
 hb.retail.boss=true;sb.players[1].stocks=3;sb.players[1].x=0;sb.players[1].y=-9000;frames(hb,900);assert(#set2==0,'after boss_defeated the player is not policed')
 local s3,g3,m3,h3=start('battle');local set3={};g3.set_stocks=function(p,n) set3[#set3+1]={p,n};s3.players[p].stocks=n end;g3.teleport=function() end
 s3.players[2].stocks=2;s3.players[2].x=0;s3.players[2].y=-9000;frames(h3,600);assert(#set3>=1 and set3[1][1]==2,'an opponent that never fell loses a stock')
end)
T.done()
