-- The rule host's player-facing flow: starter drive, opponent drops and pickups, the reward screen between stages
-- (free slots, full slots: swap / keep in bag / skip, timeout), uncollected drops, the build strip model, and
-- the rule that no drive is ever lost without a log line.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_engine','mod_pool','drive_loot','drive_bag','foe_roll'}) do D[n]=T.module(n,D) end
D.mod_display={new=function(g,engine)
 local v={engine=engine}
 function v:warm() return D.warm_ready~=false end
 function v:update() end
 function v:clear() end
 function v:on_loadstate(e) self.engine=e end
 function v:tick() end
 return v
end}
D.pickup_juice={pitch={},new=function()return {drop=function()return {fx={}}end,collect=function()end,expire=function()end,clear=function()end,tick=function()end}end}
for _,n in ipairs({'menu_input','drive_menu','drive_text','drive_drop','drive_lab','foe_lab','mod_lab','run_screen','run_hud','run_host'})do D[n]=T.module(n,D)end
local function fixture()
 local s={commands={},pad={},logs={},holds=0,releases=0,clock=100,spawns=0,despawns={},held=true,
  players={[1]={x=0,y=0,percent=0,stocks=3,falls=0,char=1,action=14},[2]={x=30,y=0,percent=0,stocks=1,falls=0,char=2,action=14,cpu=true}}}
 local running=false
 local g={fixture=s,command=function(n,f)s.commands[n]=f end,log=function(t)s.logs[#s.logs+1]=t end,
  match=function()return {active=true,netplay=false,stage=32}end,lab_mode=function()return false end,
  sim_supported=true,sim_replaying=function()return false end,player=function(p)return s.players[p]end,
  hit_rule_add=function()error('direct native write')end,fighter_status=function()error('direct native write')end,
  hit_rules=function()return {percent_only=true,progression=true,owner=7}end,
  sim_clear=function()s.blob=nil end,sim_read=function()return s.blob end,
  sim_commit=function(blob,ops)s.blob=blob;s.ops=ops;return true end,
  pad=function()return s.pad end,input_mask=function(_,v)s.mask=v end,paused=function()return s.paused end,
  pause=function()s.paused=true end,resume=function()s.paused=false end,items=function()return {}end,
  item_spawn=function(_,x,y,o)s.spawns=s.spawns+1;s.payload=o.payload;s.last_x,s.last_y=x,y;return 100+s.spawns end,item_despawn=function(h)s.despawns[#s.despawns+1]=h;return true end,
  hold_1p=function() s.holds=s.holds+1;s.held=true;return not s.no_hold end,release_1p=function() s.releases=s.releases+1;s.held=false;return true end,
  mode_1p=function() return {held=s.held,mode='classic'} end,time=function() return s.clock end,
  safe_area=function() return {x=0,y=0,w=640,h=480,right=640,bottom=480} end,
  stage_bounds=function() return {camera={left=-100,right=100,top=100,bottom=-100}} end,floor_below=function(x,y) return 0 end}
 local retail={state={player_port=1},loop=0,rules=true,active=true}
 local mods=D.mod_lab.new(g,{run_host=function() return running end,run_ready=function() return true end})
 local host=D.run_host.new(g,mods,retail);retail.host=host
 return s,g,mods,host,function(v) running=v end
end
local function has(s,text) for _,l in ipairs(s.logs) do if l:find(text,1,true) then return true end end;return false end
local function count_drives(host) local b=host:bag();local n=#b.items;for i=1,b:slots() do if b.equipped[i] then n=n+1 end end;return n end
local function start_run(seed)
 local s,g,mods,host,run=fixture();run(true);host:run_begin(seed or 4242);return s,g,mods,host
end
local function stage(host,e) e=e or {};host:stage_start({stage_index=e.stage or 0,loop=e.loop or 0,stage_kind=e.kind or 'battle',opponents=e.opponents or {{port=2}}});host.since=99 end
local function fill_slots(host)
 for i=1,host:bag():slots() do if not host:bag().equipped[i] then local r=host.mods.drives.loot:roll(900+i,host.mods.engine.context);local idx=host:acquire(r,'test');assert(host:equip(idx,i)) end end
end
T.test('a bag-only build (everything equipped, nothing in the bag) still reaches the engine once the looks are warm',function()
 local s,g,mods,host=start_run();stage(host);D.warm_ready=false
 for _=1,3 do mods:frame() end
 assert(mods.enabled,'the host stays enabled while the build waits for the looks')
 assert(not host.mods.drives.applied)
 D.warm_ready=true;mods:frame();assert(host.mods.drives.applied and next(mods.engine.equipped[1] or {}) or next(mods.engine.implicits[1] or {}),'build applied to the engine')
 assert(not host.mods.drives:stale())
end)
T.test('run begin puts the starter drive in slot 1 and says so honestly',function()
 local s,g,mods,host=start_run()
 local b=host:bag();assert(b.equipped[1] and #b.items==0 and count_drives(host)==1)
 assert(has(s,'equipped into slot 1:') and not has(s,'bag: equipped'))
 assert(host.hud.toasts[1] and host.hud.toasts[1].lines[1].text=='Your starter drive')
end)
T.test('an opponent KO drops a drive that is picked up into the bag with a card',function()
 local s,g,mods,host=start_run()
 stage(host);host.since=host.since-host.since%6
 host:frame();assert(not host.drop_queue or #host.drop_queue==0)
 s.players[2].falls=1;for _=1,6 do host:frame() end
 assert(host.drop_queue and #host.drop_queue==1,'KO queued a drop')
 host:tick();assert(s.spawns==1 and host.mods.drives.drops:count()==1,'drop spawned on the floor')
 assert(has(s,'opponent P2 dropped'))
 mods:pickup{name='drive',port=1,item=101,payload=s.payload}
 local b=host:bag();assert(#b.items==1 and host.mods.drives.drops:count()==0)
 assert(host:is_new(b.items[1]) and host.hud.card and host.hud.card.title:find('Picked up',1,true) and has(s,'picked up'))
end)
T.test('a one-stock opponent drops when it first passes 50 percent, so the drive is reachable before the final KO',function()
 local s,g,mods,host=start_run();stage(host);host:frame()
 s.players[2].percent=49;for _=1,6 do host:frame() end;assert(not host.drop_queue or #host.drop_queue==0)
 s.players[2].percent=55;for _=1,6 do host:frame() end;assert(#host.drop_queue==1 and host.drop_queue[1].why=='passed 50%')
 s.players[2].percent=80;for _=1,12 do host:frame() end;assert(#host.drop_queue==1,'only once per opponent')
 host:tick();assert(host.mods.drives.drops:count()==1 and has(s,'(passed 50%)'))
 -- the final KO (a second trigger) is a chance, not a guarantee
 s.players[2].falls=1;for _=1,6 do host:frame() end;assert(#host.drop_queue<=1)
end)
T.test('drop rules: team drops for every opponent, bonus and boss none, battle first guaranteed then a seeded chance, switch',function()
 local s,g,mods,host=start_run()
 stage(host,{kind='team',opponents={{port=2},{port=3},{port=4}}})
 for p=2,4 do s.players[p]={x=10*p,y=0,percent=0,stocks=1,falls=0,char=2,cpu=true} end
 host:frame();for p=2,4 do s.players[p].falls=1 end;for _=1,6 do host:frame() end
 assert(#host.drop_queue==3,'team: one per defeated opponent, got '..#host.drop_queue)
 local s2,g2,m2,h2=start_run();stage(h2,{kind='bonus',opponents={}});h2:on_ko(2);assert(not h2.drop_queue or #h2.drop_queue==0)
 stage(h2,{kind='boss'});h2:on_ko(2);assert(not h2.drop_queue or #h2.drop_queue==0)
 stage(h2,{kind='battle'});h2:on_ko(2);assert(#h2.drop_queue==1,'battle: first KO guaranteed')
 local drops=0;for i=1,40 do local s3,g3,m3,h3=start_run(i*977);stage(h3);h3:on_ko(2);h3:on_ko(2);if #h3.drop_queue==2 then drops=drops+1 end end
 assert(drops>0 and drops<40,'second drop is a chance, got '..drops..'/40')
 local s4,g4,m4,h4=start_run();D.run_host.tuning.drops=false;stage(h4);h4:on_ko(2);D.run_host.tuning.drops=true;assert(not h4.drop_queue or #h4.drop_queue==0,'drops switch off')
end)
T.test('reward moment with a free slot: one press equips; continue releases the hold; nothing is lost',function()
 local s,g,mods,host=start_run();stage(host)
 assert(host:stage_reward(0,0,false));assert(s.holds==1 and host.screen.active and #host.offers==2)
 local text=table.concat(host.screen:dump(),'\n');assert(text:find('STAGE REWARD: take one of 2',1,true) and text:find('A: take it and equip in slot 2',1,true),text)
 local before=count_drives(host)
 host.screen:press('down');host.screen:press('up');host.screen:press('accept')
 local b=host:bag();assert(b.equipped[2] and #host.offers==0 and count_drives(host)==before+1)
 assert(has(s,'equipped into slot 2:') and has(s,'declined'))
 host.screen:press('back')
 assert(host.screen.active)
 for _=1,40 do local m=host.screen:ensure();if m.rows[host.screen.focus].kind=='done' then break end;host.screen:press('down') end
 host.screen:press('accept');assert(not host.screen.active and s.releases==1 and has(s,'reward moment done (done)'))
end)
T.test('full slots: choosing an offer asks which drive to swap out, shows before and after, swap goes to the bag',function()
 local s,g,mods,host=start_run();stage(host);fill_slots(host);assert(host:equipped_count()==4)
 assert(host:stage_reward(0,0,false));local before=count_drives(host)
 host.screen:press('accept')
 assert(host.screen.view=='swap' and #host.offers==0 and #host:bag().items==1)
 local text=table.concat(host.screen:dump(),'\n');assert(text:find('SWAP OUT WHICH DRIVE',1,true) and text:find('IF YOU DO THIS',1,true) and text:find('Build strength',1,true),text)
 local old=host:bag().equipped[2];host.screen:press('down');assert(host.screen:ensure().rows[host.screen.focus].slot==2)
 host.screen:press('accept');assert(host.screen.view=='list' and count_drives(host)==before+1)
 assert(host:bag().equipped[2]~=old and #host:bag().items==1 and has(s,'swapped out'))
 assert(host:bag().items[1].seed==old.seed,'the swapped-out drive is in the bag')
end)
T.test('full slots: keep it in the bag, back out of the swap, skip asks twice, B cancels the skip',function()
 local s,g,mods,host=start_run();stage(host);fill_slots(host);host:stage_reward(0,0,false)
 host.screen:press('accept');assert(host.screen.view=='swap')
 host.screen:press('back');assert(host.screen.view=='list' and #host:bag().items==1,'back leaves the drive in the bag')
 host:finish_reward('done');host:stage_reward(1,0,false);host.screen:open('reward')
 host.screen:press('accept');local m=host.screen:ensure()
 for i,r in ipairs(m.rows) do if r.kind=='swap_keep' then host.screen.focus=i end end;host.screen:invalidate();host.screen:press('accept')
 assert(has(s,'bagged ') and #host:bag().items==2 and host:equipped_count()==4)
 host:finish_reward('done');host:stage_reward(2,0,false);host.screen:open('reward')
 local n=count_drives(host);local m2=host.screen:ensure()
 for i,r in ipairs(m2.rows) do if r.kind=='skip' then host.screen.focus=i end end;host.screen:invalidate()
 host.screen:press('accept');assert(#host.offers==2,'first A only asks');host.screen:press('back');assert(#host.offers==2)
 host.screen:press('accept');host.screen:press('accept');assert(#host.offers==0 and count_drives(host)==n and has(s,'skipped the stage reward: none kept'))
end)
T.test('the screen cannot be left with an unresolved offer except by explicit skip, timeout or release',function()
 local s,g,mods,host=start_run();stage(host);host:stage_reward(0,0,false)
 local m=host.screen:ensure();for i,r in ipairs(m.rows) do if r.kind=='done' then host.screen.focus=i end end;host.screen:invalidate()
 host.screen:press('accept');assert(host.screen.active and #host.offers==2,'Continue refuses while offers remain')
end)
T.test('timeout takes the first offer, fills free slots, keeps the rest in the bag, releases once',function()
 local s,g,mods,host=start_run();stage(host);fill_slots(host)
 host:bag().equipped[4]=nil;host:touch()
 local ground=host.mods.drives.loot:roll(31,host.mods.engine.context);host:acquire(ground,'test')
 host:stage_reward(0,0,false);local n=count_drives(host)
 s.clock=s.clock+D.run_screen.tuning.safe_seconds+1;host.screen:tick()
 assert(not host.screen.active and #host.offers==0 and s.releases==1)
 assert(count_drives(host)==n+1,'the offered drive was taken, not discarded')
 assert(host:equipped_count()==4 and #host:bag().items==1,'free slot filled, remainder bagged')
 assert(has(s,'timeout: took') and has(s,'reward moment done (timeout)'))
end)
T.test('the countdown is the shorter of the wall allowance and the engine hold at the measured tick rate',function()
 local s,g,mods,host=start_run();stage(host);host:stage_reward(0,0,false);local sc=host.screen
 s.clock=100.0;sc.opened=100.0;sc.ticks=0;assert(math.floor(sc:seconds_left())==14+12 or sc:seconds_left()>20,'fresh: wall allowance')
 sc.ticks=600;s.clock=105 -- 120 ticks a second
 assert(sc:seconds_left()<=(1800-90-600)/120+.01 and sc:seconds_left()>8)
 sc.ticks=1720;assert(sc:seconds_left()==0,'near the engine deadline the screen resolves itself')
end)
T.test('the engine releasing the hold first resolves the same way without releasing twice',function()
 local s,g,mods,host=start_run();stage(host);host:stage_reward(0,0,false);local n=count_drives(host)
 s.held=false;host.screen:tick();assert(not host.screen.active and count_drives(host)==n+1 and s.releases==0 and has(s,'released: took'))
end)
T.test('no hold available: the drives are sorted at once, logged, nothing stranded',function()
 local s,g,mods,host=start_run();stage(host);s.no_hold=true;host:stage_reward(0,0,false)
 assert(not host.screen.active and has(s,'reward hold unavailable') and #host.offers==0 and count_drives(host)==2)
end)
T.test('bonus stage and the final boss offer three; the final offers two rare and one unique',function()
 local s,g,mods,host=start_run();stage(host,{kind='bonus',opponents={}});host:stage_reward(0,0,false);assert(#host.offers==3);host.screen:close();host.offers={}
 stage(host,{kind='boss'});host:stage_reward(11,0,true);assert(#host.offers==3 and host.offers[1].rarity=='rare' and host.offers[2].rarity=='rare' and host.offers[3].rarity=='unique')
end)
T.test('uncollected drops are collected at stage end; with auto_collect off they are lost and logged',function()
 local s,g,mods,host=start_run();stage(host);host:on_ko(2);host:tick();assert(host.mods.drives.drops:count()==1)
 local n=count_drives(host);host:stage_reward(0,0,false);assert(count_drives(host)==n+1 and has(s,'collected an uncollected drop') and host.mods.drives.drops:count()==0)
 local s2,g2,m2,h2=start_run();D.run_host.tuning.auto_collect=false;stage(h2);h2:on_ko(2);h2:tick();h2:stage_reward(0,0,false);D.run_host.tuning.auto_collect=true
 assert(has(s2,'(uncollected, auto_collect off)') and count_drives(h2)==1)
 local s3,g3,m3,h3=start_run();stage(h3);h3:on_ko(2);h3:tick();m3:pickup_expire{name='drive',item=101,payload=s3.payload,reason='expired'}
 assert(#h3.faded==1 and has(s3,'faded from the floor'));local k=count_drives(h3);h3:stage_reward(0,0,false);assert(count_drives(h3)==k+1)
end)
T.test('discard needs two presses and is logged; keystones toggle with the allowance',function()
 local s,g,mods,host=start_run();stage(host);fill_slots(host);local r=host.mods.drives.loot:roll(77,host.mods.engine.context);host:acquire(r,'test')
 host.screen:open('bag');local m=host.screen:ensure();for i,row in ipairs(m.rows) do if row.kind=='bag' then host.screen.focus=i end end;host.screen:invalidate()
 host.screen:press('y');assert(#host:bag().items==1);host.screen:press('y');assert(#host:bag().items==0 and has(s,'discarded') and has(s,'player choice'))
 m=host.screen:ensure();for i,row in ipairs(m.rows) do if row.kind=='key' then host.screen.focus=i;break end end;host.screen:invalidate()
 host.screen:press('accept');assert(next(host:keystone_set()) and has(s,'keystone chosen'))
 host.screen:press('accept');assert(not next(host:keystone_set()) and has(s,'keystone removed'))
end)
T.test('bag screen from a fight pauses and resumes, B closes; swap works in the bag screen too',function()
 local s,g,mods,host=start_run();stage(host);fill_slots(host);host:acquire(host.mods.drives.loot:roll(88,host.mods.engine.context),'test')
 s.commands.bag();assert(host.screen.active and s.paused==true and host.screen.mode=='bag')
 local m=host.screen:ensure();for i,row in ipairs(m.rows) do if row.kind=='bag' then host.screen.focus=i end end;host.screen:invalidate()
 host.screen:press('accept');assert(host.screen.view=='swap');host.screen:press('back');host.screen:press('back')
 assert(not host.screen.active and s.paused==false)
end)
T.test('X keeps an offered drive in the bag; the build strip model shows pips, strength and flashes',function()
 local s,g,mods,host=start_run();stage(host);host:stage_reward(0,0,false)
 host.screen:press('x');assert(#host:bag().items==1 and host:equipped_count()==1 and has(s,'bagged '))
 local d=host.hud:dump()[1];assert(d:find('Strength',1,true) and d:find('Depth 0',1,true) and d:find('[empty]',1,true),d)
 host:equip(1,2);assert(host.hud.flash_left>0 and host.hud:dump()[1]:find('FLASH',1,true))
 host.hud:model();local m=host.hud.m;assert(#m.pips==4 and m.pips[1] and m.pips[2] and not m.pips[3] and m.strength>=1)
end)
T.test('milestones are announced once: fifth slot, keystone allowance, drive tier, New Game+',function()
 local s,g,mods,host=start_run();stage(host,{stage=4});assert(#host.hud.toasts==1,'only the starter toast before depth 5')
 host.hud.toasts={};stage(host,{stage=5});local titles={};for _,t in ipairs(host.hud.toasts) do titles[#titles+1]=t.lines[1].text end
 assert(table.concat(titles,'|')=='Fifth slot unlocked|Keystone allowance: 2|Drive tier 2',table.concat(titles,'|'))
 host.hud.toasts={};stage(host,{stage=6});assert(#host.hud.toasts==0,'said once')
 host.hud.toasts={};stage(host,{stage=0,loop=1});local found;for _,t in ipairs(host.hud.toasts) do if t.lines[1].text=='New Game+ 1' then found=true end end;assert(found,'New Game+ announced')
end)
T.test('opponent plate lists modifiers in plain words, not drive names',function()
 local s,g,mods,host=start_run();stage(host)
 local rec=mods.foes.roller:roll(2.0,17,0,2,mods.engine.context,'normal')
 mods.foes.pending[1]={op='roll',record=rec};mods.foes:apply()
 local l=mods.foes.labels[2];assert(l and l.title:find('opponent strength',1,true) and #l.lines>0)
 for _,line in ipairs(l.lines) do assert(not line:find(' Drive',1,true) and not line:find('T1',1,true),line) end
end)
T.test('screen and strip draw through the kit without errors (model only; the look is unverified)',function()
 local s,g,mods,host=start_run();stage(host);host:stage_reward(0,0,false)
 local calls=0;g.kit={panel=function()calls=calls+1 end,text=function()calls=calls+1;return 10 end,button=function()calls=calls+1 end,measure=function(t)return #t*7,18 end,paragraph=function()end}
 g.fill=function()calls=calls+1;end;g.text=function()end
 host:draw();assert(calls>20,'screen drew');host.screen:close();host.hud:announce{{text='x'},'y'};host.hud:show_card('t',{'a'});host:draw();assert(calls>40)
end)
T.test('run end with an open reward logs the unclaimed offers and clears everything',function()
 local s,g,mods,host=start_run();stage(host);host:stage_reward(0,0,false);host:run_end()
 assert(not host.screen.active and not host.running and has(s,'run ended during the stage reward: none kept'))
end)
T.done()
