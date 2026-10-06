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
 assert(drives(host)==before+1,'it went through the same gain rule as any pickup')
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
T.done()
