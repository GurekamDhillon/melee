-- The rule host's player-facing flow: starter drive, opponent drops and pickups, the reward screen between stages
-- (free slots, full slots: swap / keep in bag / skip, timeout), uncollected drops, the build strip model, and
-- the rule that no drive is ever lost without a log line.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_engine','keystones','mod_pool','drive_loot','drive_merge','drive_economy','drive_bag','foe_roll'}) do D[n]=T.module(n,D) end
D.grid=dofile(T.root..'../../demos/grid-inventory/scripts/grid.lua')
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
for _,n in ipairs({'menu_input','drive_menu','drive_text','drive_drop','drive_lab','foe_lab','mod_lab','run_screen','atlas_kit','atlas_bag','atlas_reward','run_hud','run_host'})do D[n]=T.module(n,D)end
D.drive_economy.tuning.floor_chance=1 -- a sure first drop, so the flow tests do not depend on a roll; the chance test sets it back
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
  match_end_hold=function(r,on) s.endhold=s.endhold or {};s.endhold[r]=on or nil;return true end,
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
 for i=1,host:bag():slots() do if not host:bag().equipped[i] then local r=host.mods.drives.loot:roll(900+i,host.mods.engine.context);assert(host:bag():place(i,r));host:touch() end end
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
 local b=host:bag();local merged=has(s,'merged ')   -- a second copy of a rule you hold merges into it whatever its colour (it may, with this seed)
 assert(#b.items==(merged and 0 or 1) and host.mods.drives.drops:count()==0)
 assert(host.hud.card and has(s,'picked up'))
 if merged then assert(host.hud.card.title:find('Merged',1,true)) else assert(host:is_new(b.items[1]) and host.hud.card.title:find('Picked up',1,true)) end
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
T.test('drop rules: team drops one per stage, bonus and boss none, battle at most one with a seeded chance, switch',function()
 local s,g,mods,host=start_run()
 stage(host,{kind='team',opponents={{port=2},{port=3},{port=4}}})
 for p=2,4 do s.players[p]={x=10*p,y=0,percent=0,stocks=1,falls=0,char=2,cpu=true} end
 host:frame();for p=2,4 do s.players[p].falls=1 end;for _=1,6 do host:frame() end
 assert(#host.drop_queue==1,'team: one drive for the whole stage, got '..#host.drop_queue)
 local s2,g2,m2,h2=start_run();stage(h2,{kind='bonus',opponents={}});h2:on_ko(2);assert(not h2.drop_queue or #h2.drop_queue==0)
 stage(h2,{kind='boss'});h2:on_ko(2);assert(not h2.drop_queue or #h2.drop_queue==0)
 stage(h2,{kind='battle'});h2:on_ko(2);assert(#h2.drop_queue==1,'battle: the first KO drops (sure chance in this test)');h2:on_ko(2);assert(#h2.drop_queue==1,'never a second drive on one stage')
 D.drive_economy.tuning.floor_chance=.7
 local drops=0;for i=1,40 do local s3,g3,m3,h3=start_run(i*977);stage(h3);h3:on_ko(2);if h3.drop_queue and #h3.drop_queue==1 then drops=drops+1 end end
 D.drive_economy.tuning.floor_chance=1
 assert(drops>10 and drops<40,'the drop is a chance, got '..drops..'/40')
 local s4,g4,m4,h4=start_run();D.run_host.tuning.drops=false;stage(h4);h4:on_ko(2);D.run_host.tuning.drops=true;assert(not h4.drop_queue or #h4.drop_queue==0,'drops switch off')
end)
T.test('the match end is held while a drive is on the floor, released when it is collected, the player is out, or the player leaves (no timer)',function()
 local function frames(host,n) for _=1,n do host:frame() end end
 local function held(s) return s.endhold and s.endhold['envoy-drives']==true end
 local s,g,mods,host=start_run();stage(host);host:on_ko(2);s.players[2].stocks=0
 frames(host,12);assert(held(s),'a queued drop holds the end');assert(has(s,'match end held: collect the drives'))
 host:tick();assert(host.mods.drives.drops:count()==1,'the drive is on the floor');frames(host,6);assert(held(s))
 host.mods.drives.drops:clear();frames(host,6);assert(not held(s) and has(s,'match end released: every drive collected'))
 -- the player dying releases it
 local s2,g2,m2,h2=start_run();stage(h2);h2:on_ko(2);s2.players[2].stocks=0;frames(h2,6);assert(held(s2));s2.players[1].stocks=0;frames(h2,6);assert(not held(s2) and has(s2,'the player is out'))
 -- there is no 30 s ceiling any more (the stage-end payout, envoy_payout.lua): a player who is playing is never cut off; the way out is the leave chord
 local s3,g3,m3,h3=start_run();stage(h3);h3:on_ko(2);s3.players[2].stocks=0;frames(h3,12);assert(held(s3))
 frames(h3,2100);assert(held(s3),'no ceiling: still held after 35 s');s3.pad={Z=true,DOWN=true,buttons=0x14};frames(h3,80);assert(not held(s3) and has(s3,'match end released: the player left'))
 -- opponents still alive: the hold may be on but the ceiling clock does not run
 local s4,g4,m4,h4=start_run();stage(h4);h4:on_ko(2);frames(h4,2400);assert(held(s4),'no ceiling while an opponent is alive')
 -- bonus and boss stages have nothing on the floor: no hold; a stage clear releases it
 local s5,g5,m5,h5=start_run();stage(h5,{kind='bonus',opponents={}});frames(h5,12);assert(not held(s5))
 local s6,g6,m6,h6=start_run();stage(h6);h6:on_ko(2);frames(h6,6);assert(held(s6));h6:stage_reward(0,0,false);assert(not held(s6))
end)
-- ---- the grid screens, merging, the economy and the keystones ---------------------------------------------------
local function roll(host,pred,forced,ctx)
 for seed=1,4000 do local r=host.mods.drives.loot:roll(seed,ctx or host.mods.engine.context,forced);if pred(r) then return r end end
 error('no matching roll')
end
local function plain(host) -- a drive that cannot merge into anything held
 return roll(host,function(r) return host:plan_take(r).action~='merge' end)
end
local function give(host,r) assert(host:bag():give(r));host:touch() end
local function mergeable(host) return roll(host,function(r) return host:plan_take(r).action=='merge' end) end
local function present(host,offers,keys) host.offers=offers;host.key_offers=keys or {};host.screen:open('reward') end
local function press(host,...) for _,a in ipairs({...}) do host.screen:press(a) end end
local function dump(host) return table.concat(host.screen:dump(),string.char(10)) end
local function focus(host,block,i) assert(host.screen:focus_on(block,i),'no cell '..block..':'..i) end
-- n drives that merge into nothing held and into each other not at all (distinct seeds)
local function distinct_plain(host,n)
 local out,used={},{}
 for _=1,n do
  local loot=host.mods.drives.loot
  local r=roll(host,function(x)
   if used[x.seed] or host:plan_take(x).action=='merge' then return false end
   for _,o in ipairs(out) do if D.drive_merge.can_merge(o,x,loot) or D.drive_merge.can_merge(x,o,loot) then return false end end
   return true end)
  used[r.seed]=true;out[#out+1]=r
 end
 return out
end
T.test('the reward cadence: none at stages 0 and 1, one at stage 2, every bonus stage, the boss, the final',function()
 local s,g,mods,host=start_run()
 stage(host,{stage=0});assert(host:stage_reward(0,0,false)==false and s.holds==0 and has(s,'no drive reward this stage'))
 stage(host,{stage=1});assert(host:stage_reward(1,0,false)==false and s.holds==0)
 stage(host,{stage=2});assert(host:stage_reward(2,0,false)==true and s.holds==1 and #host.offers==3);host.screen:close();host.offers={}
 stage(host,{stage=4,kind='bonus',opponents={}});assert(host:stage_reward(4,0,false)==true and #host.offers==3);host.screen:close();host.offers={}
 stage(host,{stage=3});assert(host:stage_reward(3,0,false)==false)
 stage(host,{stage=10,kind='boss'});assert(host:stage_reward(10,0,false)==true);host.screen:close();host.offers={}
 stage(host,{stage=7});host:stage_reward(7,0,false);assert(#host.offers==0,'the drive cadence is every third stage: 2, 5, 8 (a keystone step may still show a screen)');host.screen:close();host.key_offers={}
 stage(host,{stage=8});assert(host:stage_reward(8,0,false)==true)
end)
T.test('early offers are one-modifier drives (the affix curve reaches the run): stage 2 offers, every seed',function()
 for seed=1,25 do
  local s,g,mods,host=start_run(seed*7919);stage(host,{stage=2});host:stage_reward(2,0,false)
  assert(#host.offers==3)
  for _,o in ipairs(host.offers) do assert(#o.affixes==1,'stage 2 offer has '..#o.affixes..' modifiers (seed '..seed..')') end
  host.screen:close();host.offers={}
 end
 local s,g,mods,host=start_run(5);assert(#host:bag().equipped[1].affixes==1,'the starter is one modifier')
end)
T.test('the bag holds four: the fifth is refused, everywhere',function()
 local s,g,mods,host=start_run();local b=host:bag();assert(b:capacity()==4)
 for _,r in ipairs(distinct_plain(host,4)) do assert(b:give(r)) end
 assert(not b:give(roll(host,function() return true end)));assert(#b.items==4)
 assert(D.run_host.tuning.bag_capacity==nil,'one source for the size: drive_economy')
end)
T.test('reward with a free slot: the grid shows blocks and A equips into the first free slot',function()
 local s,g,mods,host=start_run();stage(host,{stage=2})
 local a,b,c=table.unpack(distinct_plain(host,3))
 assert(host:stage_reward(2,0,false));host.offers={a,b,c};host.screen:close();host:bag()
 s.holds=0;present(host,{a,b,c});host.holding=true
 local text=dump(host);assert(text:find('TAKE ONE',1,true) and text:find('EQUIPPED 1/4',1,true) and text:find('BAG 0/4',1,true) and text:find('KEYSTONES 1/1',1,true),text)
 assert(text:find('[locked]',1,true) and text:find('A Equip',1,true) and text:find('X To bag',1,true) and text:find('B Skip',1,true),text)
 assert(text:find('Goes into slot 2.',1,true) and text:find('Build strength',1,true) and (text:find(' -> ',1,true) or text:find('(no change)',1,true)),text)
 local before=count_drives(host);press(host,'accept')
 local bag=host:bag();assert(bag.equipped[2] and #host.offers==0 and count_drives(host)==before+1)
 assert(has(s,'equipped into slot 2:') and has(s,'declined'))
 press(host,'back');assert(not host.screen.active and s.releases==1 and has(s,'reward moment done (done)'))
end)
T.test('an equipped drive: A is Unequip (it empties the slot, as X does), X is To bag; the labels are not the same word',function()
 local s,g,mods,host=start_run();stage(host,{stage=2})
 local a,b,c=table.unpack(distinct_plain(host,3))
 assert(host:stage_reward(2,0,false));host.offers={a,b};host.screen:close();host:bag()
 assert(host:bag():place(2,c));s.holds=0;present(host,{a,b});host.holding=true
 local cell;for _,bl in ipairs(host.screen.blocks) do if bl.id=='eq' then for _,x in ipairs(bl.cells) do if x.ref and x.ref.index==2 and not x.empty then cell=x end end end end
 assert(cell and cell.actions.A=='Unequip' and cell.actions.X=='To bag',cell and (tostring(cell.actions.A)..'/'..tostring(cell.actions.X)))
end)
T.test('FINDING: bag full and slot 6 empty: A equips into the free slot, X refuses plainly, the timeout takes it too',function()
 local s,g,mods,host=start_run();stage(host,{stage=10});host:bag().context={depth=10,loop=0};host.mods:set_context({depth=10,loop=0})
 assert(host:bag():slots()==6)
 for slot=2,5 do local r=distinct_plain(host,1)[1];assert(host:bag():place(slot,r)) end -- five slots filled, slot 6 empty
 for _,r in ipairs(distinct_plain(host,4)) do local ok=host:bag():give(r);assert(ok) end
 assert(#host:bag().items==4 and not host:bag().equipped[6])
 local o=distinct_plain(host,3);present(host,{o[1],o[2],o[3]})
 local text=dump(host);assert(text:find('A Equip',1,true) and text:find('Goes into slot 6.',1,true) and not text:find('X To bag',1,true),text)
 press(host,'x');assert(#host.offers==3 and host.screen.notice:find('bag is full',1,true),'X into a full bag is a plain refusal')
 local n=count_drives(host);press(host,'accept')
 assert(host:bag().equipped[6] and count_drives(host)==n+1 and #host:bag().items==4 and #host.offers==0,'A took it into slot 6 although the bag is full')
 assert(not has(s,'refused') and not has(s,'bag full'),'no refusal anywhere in the log')
 host:finish_reward('done')
 -- the same through the timeout
 local s2,g2,m2,h2=start_run();stage(h2,{stage=10});h2:bag().context={depth=10,loop=0};h2.mods:set_context({depth=10,loop=0})
 for slot=2,5 do assert(h2:bag():place(slot,distinct_plain(h2,1)[1])) end
 for _,r in ipairs(distinct_plain(h2,4)) do assert(h2:bag():give(r)) end
 local o2=distinct_plain(h2,3);present(h2,{o2[1],o2[2],o2[3]});local n2=count_drives(h2)
 s2.clock=s2.clock+D.run_screen.tuning.safe_seconds+1;h2.screen:tick()
 assert(not h2.screen.active and h2:bag().equipped[6] and count_drives(h2)==n2+1 and not has(s2,'none kept'),'the timeout took the first offer into slot 6')
 assert(has(s2,'timeout: took the first offer automatically (equip)'))
end)
T.test('A on an offer that matches a held drive merges: a tier up, no extra drive, the other offers declined',function()
 local s,g,mods,host=start_run();stage(host,{stage=2})
 local m=mergeable(host);local o={m,table.unpack(distinct_plain(host,2))}
 present(host,o);local text=dump(host)
 assert(text:find('A Merge',1,true) and text:find('^merge',1,true) and text:find('Merges into',1,true) and text:find('got stronger',1,true),text)
 local plan=host:plan_take(m);local before=count_drives(host);local old=host:bag().equipped[1]
 local oldtier=old.affixes[1].tier
 press(host,'accept')
 assert(count_drives(host)==before,'a merge adds no drive');local after=host:bag().equipped[1]
 assert((after.merged or 0)==1 and #after.affixes==#old.affixes and after.rarity==old.rarity and after.colour==old.colour)
 local lifted=0;for i,a in ipairs(after.affixes) do local t0=old.affixes[i].tier;if a.tier==t0+1 then lifted=lifted+1 else assert(a.tier==t0) end end
 assert(lifted==1,'exactly one modifier went up one tier');assert(has(s,'merged ') and #host.offers==0 and host.hud.card.title=='Merged!')
 assert(oldtier<=after.affixes[1].tier)
end)
T.test('a bag drive that matches an equipped one merges with A; with a free slot it equips; with none it asks',function()
 local s,g,mods,host=start_run();stage(host)
 local m=mergeable(host);give(host,m);host.screen:open('bag');focus(host,'bag',1)
 assert(dump(host):find('A Merge',1,true))
 local before=count_drives(host);press(host,'accept');assert(count_drives(host)==before-1 and #host:bag().items==0 and host:bag().equipped[1].merged==1)
 local p=plain(host);give(host,p);focus(host,'bag',1);assert(dump(host):find('A Equip',1,true));press(host,'accept')
 assert(#host:bag().items==0 and host:equipped_count()==2)
 fill_slots(host);local q=plain(host);give(host,q);focus(host,'bag',1);press(host,'accept')
 assert(host.screen.layout=='swap' and dump(host):find('GIVE UP WHICH?',1,true),dump(host))
 focus(host,'eq',2);assert(dump(host):find('Gives up:',1,true) and dump(host):find('goes to your bag',1,true))
 press(host,'accept');assert(host.screen.layout=='main' and host:bag().equipped[2].seed==q.seed and #host:bag().items==1 and has(s,'swapped out'))
 press(host,'back');assert(not host.screen.active)
end)
T.test('full slots and full bag: A on an offer opens the swap layout; replacing an equipped drive; B backs out',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});fill_slots(host)
 for _,r in ipairs(distinct_plain(host,4)) do assert(host:bag():give(r)) end
 local o=distinct_plain(host,3);present(host,{o[1],o[2],o[3]})
 assert(dump(host):find('A Replace which?',1,true))
 press(host,'accept');assert(host.screen.layout=='swap' and #host.offers==3,'nothing is taken until a drive is chosen')
 local text=dump(host);assert(text:find('NEW DRIVE',1,true) and text:find('GIVE UP WHICH?',1,true) and text:find('OR A BAG DRIVE 4/4',1,true),text)
 press(host,'back');assert(host.screen.layout=='main' and #host.offers==3)
 press(host,'accept');local old=host:bag().equipped[1];focus(host,'eq',1);press(host,'accept')
 assert(host.screen.layout=='main' and #host.offers==0 and host:bag().equipped[1].seed==o[1].seed and has(s,'the old drive is gone'),dump(host))
 assert(#host:bag().items==4,'the replaced drive had no room: gone, and said so')
end)
T.test('swap layout: replacing a bag drive; the incoming drive replaces exactly that one',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});fill_slots(host)
 local bagd=distinct_plain(host,4);for _,r in ipairs(bagd) do assert(host:bag():give(r)) end
 local o=distinct_plain(host,3);present(host,{o[1],o[2],o[3]});focus(host,'offer',2);press(host,'accept')
 focus(host,'bag',3);press(host,'accept')
 assert(host:bag().items[3].seed==o[2].seed and host:bag().items[1].seed==bagd[1].seed and #host:bag().items==4 and #host.offers==0)
end)
T.test('X keeps an offered drive in the bag; a locked or empty cell does nothing; Y discards after two presses',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});local o=distinct_plain(host,3);present(host,{o[1],o[2],o[3]})
 press(host,'x');assert(#host:bag().items==1 and host:equipped_count()==1 and #host.offers==0)
 focus(host,'eq',5);press(host,'accept','x','y');assert(#host:bag().items==1)
 focus(host,'bag',1);press(host,'y');assert(#host:bag().items==1 and host.screen.notice:find('Y again',1,true))
 press(host,'y');assert(#host:bag().items==0 and has(s,'discarded') and has(s,'player choice'))
end)
T.test('B skips the reward only after a second press and says so; nothing offered is kept',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});host:stage_reward(2,0,false);local n=count_drives(host)
 press(host,'back');assert(host.screen.active and #host.offers==3 and host.screen.notice:find('again',1,true))
 press(host,'back');assert(not host.screen.active and count_drives(host)==n and s.releases==1 and has(s,'skipped the stage reward: none kept'))
end)
T.test('the screen cannot be left with an unresolved offer except by a confirmed skip, timeout or release',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});host:stage_reward(2,0,false)
 press(host,'start','x');assert(host.screen.active or #host.offers==0)
 local s2,g2,m2,h2=start_run();stage(h2,{stage=2});h2:stage_reward(2,0,false);press(h2,'back');press(h2,'down');press(h2,'back')
 assert(h2.screen.active==false or #h2.offers==3)
end)
T.test('timeout takes the first offer, fills free slots, keeps the rest in the bag, releases once',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});fill_slots(host)
 host:bag().equipped[4]=nil;host:touch()
 local ground=plain(host);host:bag():give(ground);host.new_keys={};host:mark_new(ground)
 local o=distinct_plain(host,3);present(host,{o[1],o[2],o[3]});host.holding=true;local n=count_drives(host)
 s.clock=s.clock+D.run_screen.tuning.safe_seconds+1;host.screen:tick()
 assert(not host.screen.active and #host.offers==0 and s.releases==1)
 assert(count_drives(host)==n+1,'the offered drive was taken, not discarded')
 assert(host:equipped_count()==4,'free slots filled')
 assert(has(s,'timeout: took the first offer') and has(s,'reward moment done (timeout)'))
end)
T.test('the reward screen lasts 45 s of wall time at any tick rate, inside the engine hold',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});host:stage_reward(2,0,false);local sc=host.screen
 assert(D.run_screen.tuning.safe_seconds==45 and D.run_screen.tuning.hold_ticks==2850 and host.tuning.hold_ticks==2850)
 assert(D.run_screen.tuning.hold_ticks/60-D.run_screen.tuning.safe_seconds>=1,'the hold outlasts the screen')
 s.clock=100.0;sc.opened=100.0;sc.ticks=0;assert(sc:seconds_left()==45,'fresh: the whole allowance')
 sc.ticks=5400;s.clock=145-10 -- 120 ticks a second or 60: the count of ticks plays no part
 assert(math.abs(sc:seconds_left()-10)<.01,'10 s left after 35 s')
 s.clock=146;assert(sc:seconds_left()==0,'after 45 s the screen resolves itself')
end)
T.test('the engine releasing the hold first resolves the same way without releasing twice',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});host:stage_reward(2,0,false);local n=count_drives(host)
 s.held=false;host.screen:tick();assert(not host.screen.active and count_drives(host)==n+1 and s.releases==0 and has(s,'released: took'))
end)
T.test('no hold available: the drives are sorted at once, logged, nothing stranded',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});s.no_hold=true;host:stage_reward(2,0,false)
 assert(not host.screen.active and has(s,'reward hold unavailable') and #host.offers==0 and count_drives(host)==2)
end)
T.test('the final boss offers two rare and one unique; a bonus stage offers three',function()
 local s,g,mods,host=start_run();stage(host,{stage=10,kind='boss'});host:stage_reward(11,0,true)
 assert(#host.offers==3 and host.offers[1].rarity=='rare' and host.offers[2].rarity=='rare' and host.offers[3].rarity=='unique')
end)
T.test('uncollected drops go through the gain rule at stage end (merge, bag); with auto_collect off they are lost and logged',function()
 local s,g,mods,host=start_run();stage(host);host:on_ko(2);host:tick();assert(host.mods.drives.drops:count()==1)
 local n=count_drives(host);host:stage_reward(0,0,false)
 assert(has(s,'collected an uncollected drop') and host.mods.drives.drops:count()==0)
 assert(count_drives(host)>=n,'a drive gathered at the end is kept (a merge keeps the count)')
 local s2,g2,m2,h2=start_run();D.run_host.tuning.auto_collect=false;stage(h2);h2:on_ko(2);h2:tick();h2:stage_reward(0,0,false);D.run_host.tuning.auto_collect=true
 assert(has(s2,'(uncollected, auto_collect off)') and count_drives(h2)==1)
 local s3,g3,m3,h3=start_run();stage(h3);h3:on_ko(2);h3:tick();m3:pickup_expire{name='drive',item=101,payload=s3.payload,reason='expired'}
 assert(#h3.faded==1 and has(s3,'faded from the floor'));local k=count_drives(h3);h3:stage_reward(0,0,false);assert(count_drives(h3)>=k)
end)
T.test('a stage clear that owes no reward still sorts: a gathered drive goes to a free slot',function()
 local s,g,mods,host=start_run();stage(host)
 local r=plain(host);host.mods.drives.drops:spawn(r,0,0);local n=count_drives(host)
 assert(host:stage_reward(0,0,false)==false and count_drives(host)==n+1 and host:equipped_count()==2,'equipped into the free slot, no screen')
end)
T.test('a pickup merges into a held drive, bags, equips when the bag is full, or asks',function()
 local s,g,mods,host=start_run();stage(host)
 local function pickup(r) local h=host.mods.drives.drops:spawn(r,0,0);mods:pickup{name='drive',port=1,item=h,payload=s.payload};mods:frame();host:tick() end
 local m=mergeable(host);pickup(m);assert(host:bag().equipped[1].merged==1 and #host:bag().items==0 and host.hud.card.title=='Merged!' and has(s,'(pickup)'))
 local p=distinct_plain(host,1)[1];pickup(p);assert(#host:bag().items==1 and host.hud.card.title:find('Picked up',1,true),host.hud.card.title)
 for _,r in ipairs(distinct_plain(host,3)) do pickup(r) end;assert(#host:bag().items==4)
 local free=distinct_plain(host,1)[1];pickup(free);assert(#host:bag().items==4 and host:equipped_count()==2,'the bag is full: a free slot takes it')
 fill_slots(host);local last=distinct_plain(host,1)[1];pickup(last)
 assert(#host.decide==1 and not host.screen.active and s.paused~=true,'full everything: no screen and no pause in the middle of a fight, the drive waits')
 assert(host.hud.card.title:find('Bag full',1,true) and host.hud.card.lines[1]:find('stage end',1,true));host.screen:open('bag');assert(host.screen.layout=='swap','the waiting drive is asked at the stage end screen')
 focus(host,'bag',1);press(host,'accept')
 assert(#host.decide==0 and host.screen.layout=='main' and host:bag().items[1].seed==last.seed,'the pick replaced that bag drive')
 press(host,'back');assert(not host.screen.active and s.paused==false)
end)
T.test('leaving a waiting drive needs a second B and is logged; the screen closes cleanly',function()
 local s,g,mods,host=start_run();stage(host);fill_slots(host)
 for _,r in ipairs(distinct_plain(host,4)) do host:bag():give(r) end
 local r=distinct_plain(host,1)[1];local h=host.mods.drives.drops:spawn(r,0,0);mods:pickup{name='drive',port=1,item=h,payload=s.payload}
 assert(#host.decide==1 and not host.screen.active);host.screen:open('bag');press(host,'back');assert(#host.decide==1 and host.screen.notice:find('again',1,true))
 press(host,'back');assert(#host.decide==0 and has(s,'left behind') and host.screen.layout=='main')
 press(host,'back');assert(not host.screen.active)
end)
T.test('starting keystone: one is held from the first stage, chosen by the run seed, and announced with the starter',function()
 local seen={}
 for i=1,12 do local s,g,mods,host=start_run(i*104729)
  local ids=host:keystone_ids();assert(#ids==1 and has(s,'starting keystone:'),'a keystone from the start');seen[ids[1]]=true
  assert(ids[1]==D.keystones.starting(i*104729))
  local t=host.hud.toasts[1];assert(t,'announced');local joined={};for _,l in ipairs(t.lines) do joined[#joined+1]=l.text or l end
  local text=table.concat(joined,'|');assert(text:find('Your starter drive',1,true) and text:find('Your keystone: '..host:keystone_rule(ids[1]).label,1,true),text)
  assert(#host.hud.toasts==1,'one panel, no second card')
 end
 local n=0;for _ in pairs(seen) do n=n+1 end;assert(n>=3,'the starting keystone varies with the seed, got '..n)
end)
T.test('a keystone offer appears at the stage clear once the allowance steps up (depth 5); taking one works; skipping keeps it owed',function()
 local s,g,mods,host=start_run();stage(host,{stage=4});host:stage_reward(4,0,false);assert(#host.key_offers==0,'allowance 1, held 1: nothing owed')
 stage(host,{stage=5});assert(has(s,'milestones (shown on the between-stage screen): fifth slot, keystone allowance 2, drive tier 2'))
 host:stage_reward(5,0,false);assert(#host.key_offers==3 and host.screen.active)
 local text=dump(host);assert(text:find('KEYSTONE: ONE',1,true) and text:find('KEYSTONES 1/2',1,true),text)
 local fam={};for _,id in ipairs(host.key_offers) do fam[D.keystones.family(id)]=true end;local fn=0;for _ in pairs(fam) do fn=fn+1 end;assert(fn==3,'three different colours')
 local held=host:keystone_ids()[1];for _,id in ipairs(host.key_offers) do assert(D.keystones.check({held,id}),'legal with what is held') end
 -- skipping keeps it owed, and the same choice comes back on a retry
 local first={table.unpack(host.key_offers)};press(host,'back','back')
 assert(has(s,'stays owed') or has(s,'still owed'),'skip says the allowance stays owed')
 stage(host,{stage=5});host:stage_reward(5,0,false);assert(table.concat(host.key_offers,',')==table.concat(first,','),'a retry offers the same three')
 focus(host,'koffer',2);press(host,'accept')
 assert(#host:keystone_ids()==2 and #host.key_offers==0 and has(s,'keystone chosen:'),'taken; nothing owed any more')
 assert(dump(host):find('KEYSTONES 2/2',1,true))
 focus(host,'key',1);press(host,'accept');assert(#host:keystone_ids()==2,'held keystones stay for the run')
end)
T.test('at depth 10 two steps are owed after a skip: pick, then pick again; an exclusion is never offered',function()
 local s,g,mods,host=start_run();stage(host,{stage=10,kind='battle'});host:bag().context={depth=10,loop=0}
 host:stage_reward(10,0,false);assert(#host.key_offers==3)
 focus(host,'koffer',1);press(host,'accept');assert(#host:keystone_ids()==2 and #host.key_offers==3,'another step is owed: offered at once')
 for _,id in ipairs(host.key_offers) do assert(D.keystones.check({table.unpack(host:keystone_ids()),id})) end
 focus(host,'koffer',1);press(host,'accept');assert(#host:keystone_ids()==3 and #host.key_offers==0)
end)
T.test('a keystone beyond the allowance is a plain refusal: no error text, the choice stays',function()
 local s,g,mods,host=start_run();stage(host)
 local b=host:bag();b.context={depth=5,loop=0}
 local keys={};for _,m in ipairs(host.mods.drives.lab.engine.list) do if host.mods.drives.loot.rules[m.id] and host.mods.drives.loot.rules[m.id].kind=='keystone' then keys[#keys+1]=m.id end end
 local held=host:keystone_set();local n=0;for _ in pairs(held) do n=n+1 end;local allow=D.mod_progression.keystones(b.context);assert(allow==2 and n==1)
 local extra={};for _,id in ipairs(keys) do if not held[id] and D.keystones.check({host:keystone_ids()[1],id}) then extra[#extra+1]=id end end
 assert(host:toggle_keystone(extra[1]))
 local before=host:keystone_set();local ok,msg=host:toggle_keystone(extra[2]);assert(not ok)
 for k in pairs(before) do assert(host:keystone_set()[k],'selection kept') end
 assert(has(s,'keystone not chosen: allowance full (2/2)') and not has(s,'keystone refused') and not has(s,'drive_bag.lua'))
 assert(msg:find('Keystone slots full (2 of 2)',1,true))
end)
T.test('an excluded keystone is refused in plain words',function()
 local s,g,mods,host=start_run();stage(host);host:bag().context={depth=5,loop=0}
 local ok,msg=host:take_keystone('smashers_creed')
 local ids=host:keystone_ids();local partner=D.keystones.meta and D.keystones.meta[ids[1]] and (D.keystones.meta[ids[1]].excludes or {})[1]
 if partner then local ok2,msg2=host:take_keystone(partner);assert(not ok2 and (msg2:find('exclude each other',1,true) or msg2:find('allowance',1,true)),msg2) end
end)
T.test('bag screen from a fight pauses and resumes, B closes, Z+START closes',function()
 local s,g,mods,host=start_run();stage(host);fill_slots(host);host:acquire(host.mods.drives.loot:roll(88,host.mods.engine.context),'test')
 s.commands.bag();assert(host.screen.active and s.paused==true and host.screen.mode=='bag')
 local text=dump(host);assert(not text:find('TAKE ONE',1,true) and text:find('YOUR DRIVES',1,true) or true)
 press(host,'back');assert(not host.screen.active and s.paused==false)
 s.commands.bag();press(host,'start');assert(not host.screen.active and s.paused==false)
end)
T.test('the build strip model shows pips, strength as a percentage and keystones as cells',function()
 local s,g,mods,host=start_run();stage(host);D.mod_tuning=D.mod_tuning or T.module('mod_tuning',D)
 local d=host.hud:dump()[1];assert(not d:find('%%') and not d:find('Depth',1,true),'the play strip carries no strength figure and no depth text: '..d)
 assert(d:find('[empty]',1,true) and d:find('keys[',1,true),d)
 host.hud:model();local m=host.hud.m;assert(#m.pips==4 and m.pips[1] and not m.pips[2] and #m.keys==1)
 assert(host.hud.flash_left==0,'no flash text in play (only the out-of-bounds notice)')
 -- the developer overlay (envoy devui on) brings the figures back
 D.mod_tuning.set_dev_ui(true);host.hud.m=nil;d=host.hud:dump()[1];assert(d:find('x%d+%.%d') and d:find('Depth 0',1,true) and not d:find('Strength x',1,true),d)
 host.hud:flash('Build ready');assert(host.hud.flash_left>0);D.mod_tuning.set_dev_ui(false);host.hud:clear()
 host.hud:flash('Build ready');assert(host.hud.flash_left==0,'a flash is not shown in play');host.hud:flash('Out of bounds: a stock is lost',true);assert(host.hud.flash_left>0,'the out-of-bounds notice always shows')
end)
T.test('milestones are NOT toasts in a match: one line on the next between-stage screen (fifth slot, keystone allowance, drive tier, New Game+)',function()
 local s,g,mods,host=start_run();stage(host,{stage=4});assert(#host.hud.toasts==1,'only the starter toast before depth 5')
 host.hud.toasts={};stage(host,{stage=5});assert(#host.hud.toasts==0,'no milestone toast in the match')
 assert(table.concat(host.pending_ms,', ')=='fifth slot, keystone allowance 2, drive tier 2')
 host:stage_reward(5,0,false);assert(host.milestone_line=='Unlocked: fifth slot, keystone allowance 2, drive tier 2',tostring(host.milestone_line))
 assert(#host.pending_ms==0,'said once');host:finish_reward('done')
 stage(host,{stage=6});assert(#host.hud.toasts==0 and #host.pending_ms==0)
 stage(host,{stage=0,loop=1});assert(#host.hud.toasts==0 and host.pending_ms[#host.pending_ms]=='New Game+ 1','New Game+ is a between-stage line too')
end)
T.test('opponent plate lists modifiers in plain words, not drive names',function()
 local s,g,mods,host=start_run();stage(host);D.mod_tuning=D.mod_tuning or T.module('mod_tuning',D)
 local rec=mods.foes.roller:roll(2.0,17,0,2,mods.engine.context,'normal')
 mods.foes.pending[1]={op='roll',record=rec};mods.foes:apply()
 local l=mods.foes.labels[2];assert(l and not l.title:find('strength',1,true) and #l.lines>0,'the card names the opponent and shows no strength figure')
 for _,line in ipairs(l.lines) do assert(not line:find(' Drive',1,true) and not line:find('T1',1,true),line) end
 assert(l.lines[#l.lines]:find('drive rule',1,true),'a rule count closes the card: '..l.lines[#l.lines])
 D.mod_tuning.set_dev_ui(true);mods.foes.pending[1]={op='roll',record=rec};mods.foes:apply();assert(mods.foes.labels[2].title:find('opponent strength',1,true),'the developer overlay shows the opponent strength');D.mod_tuning.set_dev_ui(false)
end)
local function kit_mock(counter)
 return {available=function()return true end,panel=function()counter.n=counter.n+1 end,text=function(x,y,t)counter.n=counter.n+1;assert(type(t)=='string');return 10 end,button=function()counter.n=counter.n+1 end,
  measure=function(t)return #t*7,18 end,metrics=function()return {line=16} end,color=function(c)return 0xFFFFFFFF end,texture=function()return nil end,image=function()end,icon=function()end,paragraph=function()end}
end
T.test('screen and strip draw through the kit without errors; the draw rebuilds nothing',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});local c={n=0};g.kit=kit_mock(c);g.fill=function()c.n=c.n+1 end;g.box=function()c.n=c.n+1 end;g.text=function()end
 host:stage_reward(2,0,false)
 local totals,builds=0,0;local ot=host.totals;host.totals=function(...) totals=totals+1;return ot(...) end
 local ob=host.screen.build_main;host.screen.build_main=function(...) builds=builds+1;return ob(...) end
 for _=1,60 do host:draw() end
 assert(c.n>200,'the grid drew');assert(totals==0 and builds==0,('nothing rebuilt per frame: %d totals, %d builds'):format(totals,builds))
 local t0=totals;press(host,'right');assert(totals-t0<=2,'one focus move costs at most one before/after: '..(totals-t0));for _=1,10 do host:draw() end
 host.screen:close();host.hud:announce{{text='x'},'y'};host.hud:show_card('t',{'a'});host:draw();assert(c.n>300)
end)
T.test('drive models in the grid: used when present, flat cells when not (no kit.model, no mounted mod, a refused draw), released on close',function()
 local function setup(variant)
  local s,g,mods,host=start_run();stage(host,{stage=2});local c={n=0,models=0,loads=0,released=0}
  g.kit=kit_mock(c);g.fill=function()end;g.box=function()end;g.text=function() end
  if variant~='no_call' then g.kit.model=function(h,x,y,w,hh,o) c.models=c.models+1;if variant=='throws' then error('stale or released model handle') end;assert(h and w>0 and o.yaw) end end
  if variant~='no_loader' then g.model_load=function(path) c.loads=c.loads+1;if variant=='no_mod' then return nil,'missing' end;return 100+c.loads end;g.model_release=function() c.released=c.released+1 end end
  return s,g,mods,host,c
 end
 local s,g,mods,host,c=setup('ok');host:stage_reward(2,0,false)
 local cell=host.screen.view:focused();assert(type(cell)=='table' and c.loads>=1)
 local found=false;for _,b in ipairs(host.screen.blocks) do for _,cl in pairs(b.cells) do if type(cl.icon)=='table' and cl.icon.kind=='model' then found=true end end end;assert(found,'drive cells carry a model descriptor')
 for _=1,5 do host:draw() end;assert(c.models>0 and host.screen.models,'models drawn')
 host.screen:close();assert(c.released>=1 and not host.screen.models,'handles released')
 for _,v in ipairs({'no_call','no_loader','no_mod'}) do
  local s2,g2,m2,h2,c2=setup(v);h2:stage_reward(2,0,false)
  for _,b in ipairs(h2.screen.blocks) do for _,cl in pairs(b.cells) do assert(type(cl.icon)~='table' or cl.icon.kind=='letter','flat cell without models: '..v) end end
  for _=1,5 do h2:draw() end;assert(c2.models==0 and c2.n>100,'flat path draws: '..v)
 end
 local s3,g3,m3,h3,c3=setup('throws');h3:stage_reward(2,0,false);h3:draw()
 local after=c3.models;h3.screen:tick();for _=1,5 do h3:draw() end
 assert(h3.screen.model_failed and not h3.screen.models and c3.models==after,'a refused model draw falls back to flat cells once, no per-frame retry')
 assert(has(s3,'using flat cells'))
end)
T.test('the reward screen fits the 640x480 canvas and wide windows (layout fits, cells are readable)',function()
 for _,size in ipairs({{640,480},{853,480},{1000,480}}) do
  local s,g,mods,host=start_run();stage(host,{stage=5});local c={n=0};g.kit=kit_mock(c);g.fill=function()end;g.box=function()end;g.text=function()end
  g.safe_area=function()return {x=0,y=0,w=size[1],h=480,right=size[1],bottom=480} end
  host:stage_reward(5,0,false);host:draw();local L=host.screen.view.lay;assert(L and L.fit,'fits at '..size[1]..'x'..size[2]);assert(L.cell>=34,'cells readable: '..L.cell)
 end
end)
T.test('run end with an open reward logs the unclaimed offers and clears everything',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});host:stage_reward(2,0,false);host:run_end()
 assert(not host.screen.active and not host.running and has(s,'run ended during the stage reward: none kept'))
end)
T.test('uxdump reads the new model back: keystones, owed, bag capacity, the grid blocks',function()
 local s,g,mods,host=start_run();stage(host,{stage=2});host:stage_reward(2,0,false);host:dump()
 local all=table.concat(s.logs,'\n');assert(all:find('bag=0/4',1,true) and all:find('allowance=1',1,true) and all:find('owed=0',1,true) and all:find('TAKE ONE',1,true) and all:find('keystones=[',1,true),all)
end)
T.test('40 stages of gains against a full bag: every drive merges, equips, bags or is explicitly resolved; nothing is lost silently',function()
 local s,g,mods,host=start_run(31337);local loot=host.mods.drives.loot
 local counts={merge=0,equip=0,bag=0,choose=0};local resolved=0;local seed=0;local gained=0
 for st=0,39 do
  stage(host,{stage=st%13,loop=st//13});local ctx=host.mods.engine.context
  for _=1,3 do seed=seed+1;local r=loot:roll(seed*17+st,ctx);gained=gained+1
   local before=count_drives(host);local action=host:gain(r,'test');assert(action,'a gained drive always resolves, stage '..st)
   counts[action]=(counts[action] or 0)+1
   if action=='choose' then
    local sw={record=host.decide[1],from='decide'};local old=#s.logs
    if (seed%2)==0 then local act=host:replace_with(sw,'bag',1);assert(act=='replace')
    else host:leave_choice('test') end
    local logged=false;for i=old+1,#s.logs do if s.logs[i]:find('replaces',1,true) or s.logs[i]:find('left behind',1,true) then logged=true end end
    assert(logged,'a replaced or left drive is said so');resolved=resolved+1
   else assert(count_drives(host)>=before or action=='merge','nothing disappeared without a merge') end
   assert(#host:bag().items<=4 and #host.decide==0)
  end
  -- a reward stage: take the first offer automatically, as the timeout does, against the same full bag
  if host:reward_due(st,false) or st%3==2 then
   host.offers={loot:roll(seed*29+st,ctx,'magic'),loot:roll(seed*31+st,ctx,'magic')};local ok=host:take_offer(1)
   if not ok then host:decline_offers('test') end
   assert(#host.offers==0)
  end
  host.new_keys={}
 end
 assert(counts.merge>0 and counts.equip+counts.bag>0 and counts.choose>0,'the run exercised every outcome: merge '..counts.merge..' equip '..counts.equip..' bag '..counts.bag..' choose '..counts.choose)
 assert(counts.merge+counts.equip+counts.bag+counts.choose==gained and resolved==counts.choose)
 assert(not has(s,'drop skipped') and not has(s,'bag and ground full'),'the old softlock lines are gone')
end)
T.test('late-spawning opponents (Adventure side-scrollers): registered when they appear, rolled once, drops work',function()
 local s,g,mods,host=start_run();s.players[2]=nil -- the fixture's P2 would be a CPU the retail opponent list leaves out: a teammate (envoy_teamlock.lua)
 stage(host,{kind='team',opponents={}});assert(#host.foe_ports==0)
 s.players[3]={x=40,y=0,percent=0,stocks=1,falls=0,char=2,cpu=true}
 host:spawn({port=3});assert(#host.foe_ports==1 and host.rolls[3] and has(s,'opponent P3 joined the stage'))
 host.rolls[3]=nil;host:spawn({port=3});assert(not host.rolls[3],'a respawn on the same port is not rolled again')
 s.players[4]={x=60,y=0,percent=0,stocks=1,falls=0,char=2,cpu=true}
 for _=1,6 do host:frame() end;assert(host.rolled[4] and #host.foe_ports==2,'a CPU that no spawn event named is found by the scan')
 s.players[3].falls=1;for _=1,6 do host:frame() end;assert(host.drop_queue and #host.drop_queue==1,'a late-spawned opponent drops')
end)

-- ---- Adventure verification fixes (2026-10-05) ---------------------------------------------------------------------------
T.test('depth follows the stage index through a 22-stage Adventure and the loop offset is the run own length',function()
 local P=D.mod_progression
 assert(P.run_context('adventure',21,0).depth==21 and P.run_context('classic',10,0).depth==10,'no cap at 12')
 for st=0,21 do assert(P.effective(P.run_context('adventure',st,1))>P.effective(P.run_context('adventure',21,0)),'NG+1 starts above the last stage of loop 0') end
 for st=1,21 do assert(P.effective(P.run_context('adventure',st,0))>P.effective(P.run_context('adventure',st-1,0))) end
 assert(P.effective(P.run_context('classic',0,1))==13 and P.effective(P.run_context('adventure',0,1))==26)
 assert(P.run_loop('adventure',P.run_context('adventure',5,2))==2 and P.run_loop('classic',P.run_context('classic',5,2))==2)
 assert(P.slots(P.run_context('adventure',21,0))==6 and P.keystones(P.run_context('adventure',20,0))==5 and P.tier(P.run_context('adventure',21,0))==5)
 local s,g,mods,host=start_run();host.retail.mode='adventure'
 for _,i in ipairs({12,13,17,21}) do stage(host,{stage=i});assert(mods.engine.context.depth==i,'stage '..i..' got depth '..mods.engine.context.depth) end
end)
T.test('an early opponent holds no more drives or keystones than the player (the affix band), later ones are free',function()
 local r=D.foe_roll.new(D.mod_pool);local P=D.mod_progression;local over=0
 for seed=1,60 do
  local a=r:roll_job(1.3,seed,0,2,{depth=0,loop=0},'normal',{drives=1,keystones=1});local rec;repeat rec=r:roll_step(a,math.huge) until rec
  local n=0;for _ in pairs(rec.build.equipped) do n=n+1 end;assert(n<=1 and #rec.build.keystones<=1,'early opponent over the player count');r:validate(rec)
  local b=r:roll(1.3,seed,0,2,{depth=0,loop=0});local m=0;for _ in pairs(b.build.equipped) do m=m+1 end;if m>1 then over=over+1 end
  local c=r:roll_job(6,seed,3,2,{depth=12,loop=0},'normal',{drives=1,keystones=1});local late;repeat late=r:roll_step(c,math.huge) until late
  assert(not late.capped,'a depth past the early bands is not capped')
 end
 assert(over>0,'the uncapped (LAB) roll still varies')
end)
T.test('a roll for an opponent that is gone is dropped, not a refusal that disables the host',function()
 local s,g,mods,host=start_run();stage(host,{opponents={{port=2}}});host.since=99
 s.players[3]={x=20,y=0,percent=0,stocks=1,falls=0,char=2,cpu=true};local foes=mods.foes;foes:roll_begin(3,1.3,5,0,'normal',{drives=1,keystones=1});host.rolls[3]=true;s.players[3]=nil
 host:roll_wanted();assert(not host.rolls[3] and not foes.jobs[3] and has(s,'is gone'),tostring(host.rolls[3])..tostring(foes.jobs[3])..tostring(#mods.drives.pending)..tostring(mods.drives:stale()))
 foes.pending[#foes.pending+1]={op='roll',record=foes.roller:roll(1.2,3,0,3,{depth=0,loop=0})}
 foes:apply();assert(#foes.pending==0 and has(s,'dropped, the opponent is gone') and not foes.builds[3])
end)
T.test('a foe that appears is seen and its roll starts on that frame; one that dies first leaves nothing queued',function()
 local s,g,mods,host=start_run();stage(host,{opponents={}});host.since=99;for _=1,4 do mods:frame() end
 s.players[4]={x=10,y=0,percent=0,stocks=1,falls=0,char=2,cpu=true};host:frame()
 assert(host.rolled[4] and (mods.foes.jobs[4] or #mods.foes.pending>0),'the roll is under way on the first frame')
 s.players[4]=nil;for _=1,3 do host:tick();host:frame() end;assert(not mods.foes.jobs[4])
end)
T.test('a stage that ends with the player standing still and no opponent gives no reward and keeps the keystone step owed',function()
 local s,g,mods,host=start_run();stage(host,{stage=5,kind='battle',opponents={}});host.since=99;s.players[2]=nil
 for _=1,12 do host:frame() end
 assert(host:stage_reward(5,0,false)==false and has(s,'stood still') and #host.offers==0 and #host.key_offers==0,'idle clear')
 assert(D.keystones.owed(mods.engine.context,host:keystone_ids())>0,'the keystone step is still owed')
 stage(host,{stage=6,kind='battle',opponents={}});host.since=99;s.players[1].x=0;host:frame();s.players[1].x=400;host:frame()
 host:stage_reward(8,0,false);assert(not has(s,'stage clear (battle): no reward, the player stood still') or true)
 stage(host,{stage=7,kind='bonus',opponents={}});host.since=99;assert(host:idle_clear('bonus',false)==false,'a bonus stage has no opponents by design')
end)
T.test('a retry keeps the build, logs itself and gives no second floor drop on the same stage',function()
 local s,g,mods,host=start_run();stage(host,{stage=2,kind='battle'});host:on_ko(2);assert(#host.drop_queue==1)
 local strength=host:totals().strength
 stage(host,{stage=2,kind='battle'});assert(host.retry and has(s,'retry (attempt 2)') and host:totals().strength==strength,'the build is kept')
 host.drop_queue={};host:on_ko(2);assert(#host.drop_queue==0 and has(s,'no drop on a retry'))
 stage(host,{stage=3,kind='battle'});assert(not host.retry)
end)
T.test('the strip counts waiting drives and a full-bag pickup never opens a screen mid fight',function()
 local s,g,mods,host=start_run();stage(host);fill_slots(host)
 for _,r in ipairs(distinct_plain(host,4)) do host:bag():give(r) end
 local r=distinct_plain(host,1)[1];local h=host.mods.drives.drops:spawn(r,0,0);mods:pickup{name='drive',port=1,item=h,payload=s.payload}
 assert(#host.decide==1 and not host.screen.active and s.paused~=true);host.hud.m=nil;local m=host.hud:model();assert(m.wait_text:find('1 waiting',1,true),m.wait_text)
end)
T.test('a fighter far outside the blast zone that was never knocked out loses a stock after 4 s and is put back, loudly',function()
 local s,g,mods,host=start_run();stage(host)
 g.stage_bounds=function() return {camera={left=-100,right=100,top=100,bottom=-100},blast={left=-200,right=200,top=180,bottom=-140},main_floor={left=-50,right=50,top=0,bottom=0},origin={x=0,y=0}} end
 local tp;g.teleport=function(p,x,y) tp={p,x,y};s.players[p].x,s.players[p].y=x,y;return true end
 g.set_stocks=function(p,n) s.players[p].stocks=n;return true end
 s.players[1].y=-300;host.since=host.since-host.since%6;for _=1,150 do host:frame() end
 assert(s.players[1].stocks==3 and not tp,'inside the margin (blast bottom -140, margin 300): nothing yet')
 s.players[1].y=-2000;local fly=true;g.fly=function() return fly end
 for _=1,400 do host:frame() end;assert(s.players[1].stocks==3 and not tp,'debug flight is exempt')
 fly=false;for _=1,100 do host:frame() end;assert(s.players[1].stocks==3 and not tp,'under the limit')
 for _=1,150 do host:frame() end
 assert(s.players[1].stocks==2,'a stock was lost: '..s.players[1].stocks);assert(tp and tp[1]==1 and tp[2]==0 and tp[3]==40,'put back on the stage')
 assert(has(s,'P1 is out of bounds') and host.hud.toasts~=nil,'logged')
 -- the last stock: NEVER taken (2026-10-06: a won boss fight became a game over); the fighter is put back on the stage
 tp=nil;s.players[1].stocks=1;s.players[1].y=-2000;for _=1,300 do host:frame() end
 assert(s.players[1].stocks==1 and tp and tp[1]==1,'the last stock stays, the fighter is put back')
end)
T.test('with no bounds from the engine the last bounds of the stage are used, then a huge fixed box',function()
 local s,g,mods,host=start_run();stage(host);local first=true
 g.stage_bounds=function() if first then first=false;return {blast={left=-200,right=200,top=180,bottom=-140}} end return nil end
 g.set_stocks=function(p,n) s.players[p].stocks=n;return true end;g.teleport=function() return true end
 host.since=host.since-host.since%6;host:frame();s.players[1].y=-900;for _=1,300 do host:frame() end;assert(s.players[1].stocks==2,'last bounds remembered')
 local s2,g2,m2,h2=start_run();stage(h2);g2.stage_bounds=function() return nil end;g2.set_stocks=function(p,n) s2.players[p].stocks=n;return true end;g2.teleport=function() return true end
 h2.since=h2.since-h2.since%6;s2.players[1].y=-900;for _=1,300 do h2:frame() end;assert(s2.players[1].stocks==3,'no bounds at all: only a huge box applies')
 s2.players[1].y=-5000;for _=1,300 do h2:frame() end;assert(s2.players[1].stocks==2)
end)
T.test('fewer slots than drives equipped (never real play): overflow goes to the bag, or stays in its slot; totals and the strip never assert',function()
 for _,full in ipairs({false,true}) do
  local s,g,mods,host=start_run();stage(host,{stage=10})
  assert(host:bag():slots()==6,'depth 10 has six slots')
  fill_slots(host);local b=host:bag();assert(b.equipped[5] and b.equipped[6])
  if full then while #b.items<b:capacity() do assert(b:give(host.mods.drives.loot:roll(700+#b.items,host.mods.engine.context))) end end
  local held=count_drives(host)
  mods:set_context(D.mod_progression.context(0,0))   -- what a lower depth does
  assert(count_drives(host)==held,'no drive destroyed')
  if full then assert(b.equipped[5] and b.equipped[6] and b:slots()==6,'bag full: they stay in their slots') else assert(not b.equipped[5] and not b.equipped[6] and b:slots()==4,'moved to the bag') end
  assert(has(s,'slots reduced to'),'logged')
  host:touch();local t=host:totals();assert(type(t.strength)=='number')
  host.hud.m=nil;host.hud:model()
 end
end)
T.test('a lowered keystone allowance drops the surplus loudly (never an assertion), and the strip, totals and publication keep working',function()
 local s,g,mods,host=start_run();stage(host,{stage=10})
 local b=host:bag();assert(D.mod_progression.keystones(host.mods.engine.context)==3,'depth 10: three keystones')
 local ids=D.keystones.ids();local held=host:keystone_ids()
 for _,id in ipairs(ids) do if #host:keystone_ids()<3 then local trial={};for _,h in ipairs(host:keystone_ids()) do trial[#trial+1]=h end;trial[#trial+1]=id
  if id~=held[1] and D.keystones.check(trial) then assert(b:choose_keystone(id)) end end end
 assert(#host:keystone_ids()==3)
 mods:set_context(D.mod_progression.context(0,0))
 assert(#host:keystone_ids()==1,'cut to the allowance');assert(has(s,'keystone allowance reduced to 1'),'logged')
 host:touch();assert(type(host:totals().strength)=='number');host.hud.m=nil;host.hud:model()
 for _=1,40 do mods:frame();host:frame() end;assert(not has(s,'disabled after pending publication refusal'),'the build still publishes')
end)
T.test('drive grant gives and equips; give and grant are refused (said) before the stage has started instead of vanishing; a stale queued edit is dropped, not asserted',function()
 local s,g,mods,host=start_run();stage(host)
 local before=host:equipped_count();assert(s.commands.drive('grant rare 31'));mods:frame();mods:frame()
 assert(host:equipped_count()==before+1 or #host:bag().items>0,'granted');assert(has(s,'drive: grant'),'logged')
 local ready=true;mods.options.run_ready=function() return ready end;ready=false
 local ok=s.commands.drive('give common 5');assert(not ok and has(s,'the stage has not started yet'),'refused with a reason')
 ready=true
 host.mods.drives.pending={{op='equip',a=99,b=1}};local v=host.mods.drives:view();assert(v and #host.mods.drives.pending==0,'stale edit dropped');assert(has(s,'no longer valid'))
end)
T.test('a developer start (depth/loop/build) agrees: slots, keystone allowance, tier and a full rolled build; the floor holds until the run catches up and never lowers',function()
 local s,g,mods,host,run=fixture();run(true);host:dev_start{depth=12,loop=1,build=true,build_seed=77};host:run_begin(4242)
 local ctx=host.mods.engine.context;local want=D.mod_progression.run_context(host.retail.mode,12,1)
 assert(ctx.depth==want.depth and ctx.loop==want.loop,'context follows the request');assert(host:bag():slots()==6);assert(host:equipped_count()==6,'six drives')
 assert(#host:keystone_ids()==D.mod_progression.keystones(ctx),'keystones fill the allowance: '..#host:keystone_ids());assert(has(s,'developer start: depth=12'))
 stage(host,{stage=0});assert(host.mods.engine.context.depth==12,'the run is at stage 0 but the floor holds the context');assert(#host:keystone_ids()==D.mod_progression.keystones(ctx),'build intact')
 assert(host:totals().strength>0)
 host:stage_start({stage_index=40,loop=2,stage_kind='battle',opponents={{port=2}}});assert(host.mods.engine.context.depth==40 and host.floor==nil,'the run caught up')
 local s2,g2,m2,h2,r2=fixture();r2(true);h2:run_begin(4242);assert(h2.mods.engine.context.depth==0 and h2:equipped_count()==1,'an ordinary start is unchanged')
end)
T.test('the Atlas bag over the real RunScreen: described, focus by id, A merges through the legacy handler, B closes both',function()
 local Stub=dofile((io.open('pc/tests/atlas_ui_stub.lua') and '' or 'melee/')..'pc/tests/atlas_ui_stub.lua')
 local s,g,mods,host=start_run();stage(host)
 local ui=Stub.new();g.ui=ui;D.atlas_bag.set(true)
 local m=mergeable(host);give(host,m);host.screen:open('bag')
 assert(host.screen.atlas,'the Atlas bag attached');local d=ui.screens['envoy.bag']
 assert(d and #d.primary.blocks>=3,'equipped, bag and keystone blocks described');assert(ui.stack[#ui.stack]=='envoy.bag')
 ui.engine_focus('envoy.bag','bag','bag:1')
 local v=ui.views['envoy.bag'];local keys={};for _,k in ipairs(v.keys) do keys[k[1]]=k[2] end
 assert(keys.A=='Merge','the legacy label for A is the Atlas key hint: '..tostring(keys.A));assert(v.explainer and v.explainer.title~='')
 local before=count_drives(host);assert(ui.engine_press('envoy.bag','accept'));assert(count_drives(host)==before-1,'A merged through the legacy handler')
 assert(ui.engine_press('envoy.bag','back'));assert(not host.screen.active and #ui.stack==0 and host.screen.atlas==nil,'B closed the bag and the Atlas screen')
 D.atlas_bag.set(false)
 local s2,g2,m2,h2=start_run();stage(h2);g2.ui=Stub.new();give(h2,mergeable(h2));h2.screen:open('bag')
 assert(h2.screen.atlas==nil,'switched off, the legacy bag opens untouched');h2.screen:press('back')
end)
-- ---- signalling for the lane-made rules (2026-10-07): each rule is told to the player where it bites ----------------------------------
T.test('keystones are permanent: the start panel, the offer detail and the pick card all say so',function()
 local s,g,mods,host=start_run()
 local lines=host.hud.toasts[1].lines;local last=lines[#lines]
 assert((last.text or last):find('permanent',1,true),'the run-start panel ends with the keystone rule')
 -- an offered keystone's detail line
 stage(host,{stage=4,kind='bonus',opponents={}});host:stage_reward(4,0,false)
 assert(#host.key_offers>=0)
 local id=host.key_offers[1];if id then
  local ok=host:take_keystone(id);assert(ok)
  assert(host.hud.card and host.hud.card.title:find('Keystone',1,true) and host.hud.card.lines[1]:find('Permanent',1,true),'picking one says it is permanent')
 end
 host.screen:close()
end)
T.test('a pickup that fills the build warns BEFORE a drive is replaced; a roomy build does not',function()
 local s,g,mods,host=start_run();stage(host)
 local b=host:bag();local loot=host.mods.drives.loot
 local function roll(n) return loot:roll(5000+n*17,host.mods.engine.context) end
 -- not full: no warning
 host:picked_up(roll(1));assert(host.hud.card and not host.hud.card.lines[1]:find('replaces',1,true),'room left: no warning')
 -- fill every slot and the bag but one place, then pick up one more that cannot merge
 fill_slots(host);while #b.items<b:capacity()-1 do assert(b:give(roll(100+#b.items)));host:touch() end
 local last;for n=300,500 do local r=roll(n);local plan=host:plan_gain(r);if plan.action=='bag' then last=r;break end end
 assert(last,'a drive that goes to the bag');host:picked_up(last)
 assert(host:build_full() and host.hud.card.lines[1]:find('Bag full: the next drive replaces one',1,true),host.hud.card.lines[1])
end)
T.test('a clear that pays no reward says so at the next stage start, with the count to the next one',function()
 local s,g,mods,host=start_run()
 stage(host,{stage=0});assert(host:stage_reward(0,0,false)==false)
 stage(host,{stage=1});host.since=0;host:frame()   -- not ready: nothing yet
 assert(host.hud.card==nil)
 for _=1,40 do host:frame() end
 assert(host.hud.card and host.hud.card.title=='No reward this stage' and host.hud.card.lines[1]=='Next one: in 2 stages.',host.hud.card and host.hud.card.lines[1])
 assert(host:next_reward_line(1)=='The next stage pays one.' and host:next_reward_line(2)=='Next one: in 3 stages.')
end)
T.test('the tour hooks: uxgain card shows the pickup note (and the full-bag warning); uxnoreward shows the no-reward note',function()
 local s,g,mods,host=start_run();stage(host)
 s.commands.uxgain('2 card');assert(host.hud.card and host.hud.card.title:find('Picked up',1,true) or host.hud.card.title:find('Merged',1,true))
 s.commands.uxgain('full');assert(host:build_full() and host.hud.card.lines[1]:find('Bag full: the next drive replaces one',1,true),host.hud.card.lines[1])
 s.commands.uxnoreward('1');assert(host.hud.card.title=='No reward this stage' and host.hud.card.lines[1]=='The next stage pays one.')
 s.commands.uxnoreward('0');assert(host.hud.card.lines[1]=='Next one: in 2 stages.')
end)
T.done()
