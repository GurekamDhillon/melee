local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_budget','keystones','mod_engine','mod_pool','drive_loot','drive_bag','foe_roll'}) do D[n]=T.module(n,D) end
D.mod_display={new=function(g,engine)
 local v={engine=engine}
 function v:warm() return g.fixture.ready end
 function v:update() end
 function v:clear() end
 function v:on_loadstate(e) self.engine=e end
 function v:tick() end
 return v
end}
D.pickup_juice={pitch={},new=function()return {drop=function()return {fx={}}end,collect=function()end,expire=function()end,clear=function()end,tick=function()end}end}
for _,n in ipairs({'menu_input','drive_menu','drive_drop','drive_lab','foe_lab','mod_lab'})do D[n]=T.module(n,D)end
local function fixture()
 local s={ready=true,lab=true,progression=true,percent=true,commands={},pad={},commits=0,clears=0,spawns=0,players={
  [1]={x=0,y=0,percent=0,stocks=4,falls=0,char=1,action=14},[2]={x=30,y=0,percent=0,stocks=4,falls=0,char=2,action=14,cpu=true}}}
 local g={fixture=s,command=function(n,f)s.commands[n]=f end,log=function(t)s.log=t end,
  match=function()return {active=true,netplay=s.net,stage=32}end,lab_mode=function()return s.lab end,
  sim_supported=true,sim_replaying=function()return s.replay end,player=function(p)return s.players[p]end,
  hit_rule_add=function()error('direct native write')end,fighter_status=function()error('direct native write')end,
  hit_rules=function()return {percent_only=s.percent,progression=s.progression,owner=7}end,
  sim_clear=function()s.clears=s.clears+1;s.blob=nil end,sim_read=function()return s.blob end,
  sim_commit=function(blob,ops)if s.refuse then return false end;s.blob=blob;s.ops=ops;s.commits=s.commits+1;return true end,
  pad=function()return s.pad end,input_mask=function(_,v)s.mask=v end,paused=function()return s.paused end,
  pause=function()s.paused=true end,resume=function()s.paused=false end,items=function()return {}end,
  item_spawn=function(_,_,_,o)s.spawns=s.spawns+1;s.payload=o.payload;return 100+s.spawns end,item_despawn=function(h)s.despawn=h end}
 local a=D.mod_lab.new(g);return s,a
end
local function depth(s,arg) assert(s.commands.depth,'depth command missing');return s.commands.depth(arg)end
local function context(a,d,l) assert(a.engine.context.depth==d and a.engine.context.loop==l);assert(a.drives.bag.context.depth==d and a.drives.bag.context.loop==l)end
local function tap(s,a,pad)s.pad=pad;a.drives:tick();s.pad={};a.drives:tick()end
T.test('depth command changes shared context and survives an empty checkpoint rewind',function()
 local s,a=fixture();assert(depth(s,'12 3'));context(a,12,3);assert(a.drives.bag:slots()==6)
 a:frame();local saved=s.blob;assert(saved);assert(depth(s,'0'));context(a,0,0);a:frame();s.blob=saved;a:loadstate();context(a,12,3)
 assert(a.drives:command('give unique 101'));local r=a.drives.pending[1].a;assert(r.depth==12 and r.loop==3 and r.affixes[1].tier>=D.mod_progression.tier(a.engine.context))
 a:frame();assert(a.drives:command('drop rare 103'));local ground=a.drives.drops.records[s.payload.amount].record;assert(ground.depth==12 and ground.loop==3)
 a:pickup{name='drive',port=1,item=101,payload=s.payload};assert(a.drives.bag.items[2].seed==103)
end)
T.test('depth input and offline replay capability gates leave exact roots untouched',function()
 local s,a=fixture();assert(depth(s,'5'));a:frame();local before=a:export()
 for _,arg in ipairs({'-1','nan','inf','1.5','2147483647','1 -1','1 1 1',''})do assert(not depth(s,arg));assert(a:export()==before)end
 for _,field in ipairs({'replay','net'})do s[field]=true;assert(not depth(s,'12 3'));assert(a:export()==before);s[field]=false end
 s.lab=false;assert(not depth(s,'12'));s.lab=true;s.progression=false;assert(not depth(s,'12'));assert(not a.drives:command('give'));assert(not a.foes:command('roll'));s.progression=true
 s.percent=false;assert(not depth(s,'12'));assert(a:export()==before)
end)
T.test('rewind before first checkpoint resets depth and releases bag input ownership',function()
 local s,a=fixture();assert(depth(s,'12 3'));a:frame();a.drives.menu:open();assert(s.paused)
 s.blob=nil;a:loadstate();context(a,0,0);assert(not a.enabled and not s.paused and s.mask==0)
end)
T.test('six real controller slots and all keystones remain reachable and paginated',function()
 local s,a=fixture();assert(depth(s,'12 3'))
 for i=1,6 do assert(a.drives:command('give unique '..i));a:frame() end
 a.drives.menu:open()
 for slot=1,6 do
  a.drives.menu.focus=7;tap(s,a,{A=true});a.drives.menu.focus=slot;tap(s,a,{A=true});a:frame()
 end
 assert(a.drives.bag.equipped[6] and #a.engine.display.drive_build[1]==6)
 local keys={};for _,m in ipairs(a.engine.list)do if m.kind=='keystone'then keys[#keys+1]=m.id end end
 local rows=a.drives.menu:entries();local selected=0
 for i,row in ipairs(rows)do if row.key and row.id then a.drives.menu.focus=i;tap(s,a,{A=true});selected=selected+1;if selected==3 then break end end end
 a:frame();assert(#a.drives.bag.keystones==3,'controller did not choose three keystones')
 rows=a.drives.menu:entries();local found={};for _,r in ipairs(rows)do if r.key and r.id then found[r.id]=true end end;for _,id in ipairs(keys)do assert(found[id],'missing keystone '..id)end
 local first;a.g.safe_area=function()return{x=0,y=0,w=640,h=360}end
 a.g.kit={panel=function(_,y,_,h)assert(y+h<=360)end,list=function(_,_,_,_,_,o)first=o.first end,text=function(_,y)assert(y<=336)end}
 a.drives.menu.focus=#rows-1;a.drives.menu:draw();assert(first>6);tap(s,a,{RIGHT=true});a.drives.menu:draw()
 a.drives.menu:close();assert(s.mask==0 and not s.paused)
end)
T.test('unsafe downshift refuses preserving roots and valid old tiers do not upgrade',function()
 local s,a=fixture();assert(a.drives:command('give unique 71'));a:frame();local old=D.mod_codec.encode(a.drives.bag.items[1]);assert(depth(s,'12 3'));assert(old==D.mod_codec.encode(a.drives.bag.items[1]))
 assert(a.drives:queue('equip',1,6));a:frame();local before=a:export();local engine,bag=a.engine,a.drives.bag
 assert(not depth(s,'0'));assert(before==a:export() and engine==a.engine and bag==a.drives.bag)
 assert(a.drives:queue('unequip',6));a:frame();assert(depth(s,'0'));context(a,0,0)
end)
T.test('downshift preflights pending sixth slot and multiple keystone choices atomically',function()
 local s,a=fixture();assert(depth(s,'12 3'));assert(a.drives:command('give unique 71'));a:frame()
 assert(a.drives:queue('equip',1,6));local before=a:export();assert(not depth(s,'0'));assert(before==a:export())
 a:frame();assert(a.drives:queue('unequip',6));a:frame()
 for _,id in ipairs({'pyromancer','bulwark','sprinter'})do assert(a.drives:queue('choose_keystone',id))end
 before=a:export();assert(not depth(s,'5'));assert(before==a:export());a:frame()
 before=a:export();assert(not depth(s,'5'));assert(before==a:export())
end)
T.test('legacy physical roll tiers restore without silent progression upgrades',function()
 local s,a=fixture();assert(a.drives:command('give common 1'));a:frame();local at=D.mod_codec.decode(a:export())
 local unique=a.drives.loot:roll(6,0,'unique');unique.depth=10;unique.loop=nil
 local rare=a.drives.loot:roll(7,10,'rare');rare.depth=20;rare.loop=nil
 at.drives.bag.items={unique,rare};at.drives.bag.context=nil;at.drives.bag.keystones=nil
 local e=D.mod_codec.decode(at.engine);e.context=nil;at.engine=D.mod_codec.encode(e);s.blob=D.mod_codec.encode(at)
 a:loadstate();assert(D.mod_codec.encode(a.drives.bag.items)==D.mod_codec.encode({unique,rare}));context(a,0,0)
 assert(depth(s,'12 3'));assert(D.mod_codec.encode(a.drives.bag.items)==D.mod_codec.encode({unique,rare}))
end)
T.test('maximum finite context rolls safely and impossible foe requests preserve adapter state',function()
 local s,a=fixture();assert(depth(s,'2147483646 2147483646'));context(a,2147483646,2147483646)
 for i=1,6 do assert(a.drives:command('give unique '..i));a:frame();assert(a.drives:queue('equip',1,i));a:frame()end
 for _,rule in ipairs(a.engine:native_rules(1))do for field,v in pairs(rule.change)do if field=='percent_damage' or field=='launch'then assert(v==v and math.abs(v)<=1e9)end end end
 local hit=a.engine:contact_ratios(1,2);assert(hit.outgoing>=.05 and hit.outgoing<=64 and hit.incoming>=.15 and hit.incoming<=64 and hit.launch_out>=.05 and hit.launch_out<=4 and hit.launch_in>=.05 and hit.launch_in<=4)
 local before=a:export();assert(not a.foes:command('roll 1000000000000 9 2 finalboss'));assert(before==a:export())
 assert(not a.foes:command('roll nan'));assert(before==a:export());assert(not a.foes:command('roll inf'));assert(before==a:export())
 local saved=s.blob;s.blob=saved;a:loadstate();assert(before==a:export())
end)
T.test('foes receive only scalar context role and labels include difficulty target',function()
 local s,a=fixture();assert(depth(s,'12 3'));a.engine:set_build(1,{glass_core=7},{})
 local _,power=a.engine:family_budget(1);local roll=a.foes.roller.roll;local seen
 a.foes.roller.roll=function(self,strength,seed,stage,p,ctx,role)
  assert(strength==power and ctx.depth==12 and ctx.loop==3 and role=='boss');for key in pairs(ctx)do assert(key=='depth' or key=='loop')end
  seen=true;return roll(self,strength,seed,stage,p,ctx,role)
 end
 assert(a.foes:command('roll - 31 2 boss'));assert(seen);a:frame();local r=a.foes.builds[2]
 assert(not r.reference and not r.mods);assert(math.abs(r.target-power*D.mod_progression.factor(a.engine.context,'boss'))<1e-7)
 assert(a.foes.labels[2].title:find('target',1,true) and a.foes.labels[2].title:find('boss',1,true))
end)
T.test('malformed contexts copies and keystones refuse loadstate atomically',function()
 local s,a=fixture();assert(depth(s,'12 3'));assert(a.drives:command('give unique 8'));a:frame();assert(a.drives:queue('equip',1,6));a:frame();local before=a:export()
 for _,mutate in ipairs({
  function(v)local e=D.mod_codec.decode(v.engine);e.context.loop=-1;v.engine=D.mod_codec.encode(e)end,
  function(v)v.drives.bag.context.depth=0;v.drives.bag.context.loop=0 end,
  function(v)v.drives.bag.keystones={'pyromancer','pyromancer'}end,
  function(v)v.debug_equipped[1]={glass_core={tier=1,copies=0}}end,
  function(v)v.pending={{port=1,id='missing'}}end
 })do local bad=D.mod_codec.decode(before);mutate(bad);s.blob=D.mod_codec.encode(bad);T.refuses(function()a:loadstate()end);assert(before==a:export())end
 s.blob=before;a:loadstate();assert(before==a:export());a:unload();assert(not next(a.foes.builds) and not a.drives:has_build())
end)
local function key_foe(a,ctx)
 local build={items={},equipped={},keystones={'pyromancer'},context=D.mod_progression.context(ctx)}
 if D.mod_progression.effective(ctx)>0 then build.equipped[1]={seed=1,depth=ctx.depth,loop=ctx.loop,colour='white',rarity='unique',unique='glass_core',affixes={{id='glass_core',tier=D.mod_progression.tier(ctx)}}}end
 local mods,implicit=D.drive_bag.new(a.drives.loot):validate(build);local _,strength=D.mod_budget.build(D.mod_pool,mods,implicit,{})
 local requested=strength/D.mod_progression.factor(ctx,'normal')
 local r={seed=1,stage=32,port=2,requested=requested,target=requested*D.mod_progression.factor(ctx,'normal'),strength=strength,build=build,context=D.mod_progression.context(ctx),role='normal'}
 a.foes.roller:validate(r);return r
end
T.test('joint incompatible pending foe and CPU modifier checkpoint refuses before publication',function()
 local s,a=fixture();assert(a.drives:command('give rare 79'));a:frame();a.drives.menu:open()
 local before=a:export();local engine,bag,foes=a.engine,a.drives.bag,a.foes.builds
 local bad=D.mod_codec.decode(before);bad.foes.pending={{op='roll',record=key_foe(a,a.engine.context)}};bad.pending={{port=2,id='frozen_oath'}};bad.enabled=true;s.blob=D.mod_codec.encode(bad)
 T.refuses(function()a:loadstate()end);assert(before==a:export(),'malformed joint pending checkpoint published')
 assert(engine==a.engine and bag==a.drives.bag and foes==a.foes.builds and a.drives.menu.active and s.paused and s.clears==0)
end)
T.test('valid late joint pending checkpoint stays queued then commits all three CPU keys',function()
 local s,a=fixture();assert(depth(s,'12 3'));assert(a.drives:command('give rare 79'));a:frame()
 local saved=D.mod_codec.decode(a:export());saved.foes.pending={{op='roll',record=key_foe(a,a.engine.context)}};saved.pending={{port=2,id='bulwark'},{port=2,id='sprinter'}};saved.enabled=true;s.blob=D.mod_codec.encode(saved)
 a:loadstate();assert(#a.pending==2 and #a.foes.pending==1 and not a.engine.equipped[2],'restore applied queued CPU builds')
 assert(a:export()==s.blob);a:frame();assert(a.enabled and a.engine.equipped[2].pyromancer and a.engine.equipped[2].bulwark and a.engine.equipped[2].sprinter)
 assert(#a.pending==0 and #a.foes.pending==0 and s.clears==0)
end)
T.test('joint pending roll clear and debug restore follows the actual retirement order',function()
 local s,a=fixture();assert(a:command('add pyromancer 2'));a:frame()
 local saved=D.mod_codec.decode(a:export());saved.foes.pending={{op='roll',record=key_foe(a,a.engine.context)},{op='clear'}};saved.pending={{port=2,id='frozen_oath'}};saved.enabled=true;s.blob=D.mod_codec.encode(saved)
 a:loadstate();assert(a:export()==s.blob and #a.foes.pending==2);a:frame()
 assert(a.enabled and a.engine.equipped[2].frozen_oath and not a.engine.equipped[2].pyromancer and not next(a.foes.builds))
 assert(a.debug_equipped[2].frozen_oath and not a.debug_equipped[2].pyromancer and s.clears==0)
end)
T.test('mixed unique copy tiers survive adapter merge checkpoint and same-count preview',function()
 local s,a=fixture();assert(depth(s,'12 3'))
 local function glass(seed,ctx)return {seed=seed,depth=ctx.depth,loop=ctx.loop,rarity='unique',colour='white',unique='glass_core',affixes={{id='glass_core',tier=D.mod_progression.tier(ctx)}}}end
 local early=glass(501,{depth=0});early.loop=nil;local late=glass(502,{depth=12,loop=3});local medium=glass(503,{depth=5,loop=0})
 assert(a.drives:queue('give',early));assert(a.drives:queue('give',late));a:frame();assert(a.drives:queue('equip',1,1));assert(a.drives:queue('equip',1,2));a:frame()
 a.debug_equipped[1]={glass_core=1};a.drives:apply();local stack=a.engine.equipped[1].glass_core
 assert(stack.tier==11 and stack.copies==2 and stack.tiers[1]==1 and stack.tiers[2]==11,'adapter homogenized owned unique tiers')
 local source=a.drives.bag:derive();local merged=a.drives:combined(source);merged.glass_core.tiers[1]=8;assert(source.glass_core.tiers[1]==1,'adapter aliases tier metadata')
 local _,expected=D.mod_budget.build(D.mod_pool,a.drives.bag:derive());local _,actual=a.engine:family_budget(1);assert(math.abs(expected-actual)<1e-8)
 a:frame();local before=a:export();s.blob=before;a:loadstate();assert(before==a:export() and a.engine.equipped[1].glass_core.tiers[1]==1)
 assert(a.drives:queue('give',medium));a:frame();local preview=table.concat(a.drives:delta(1,1),' ')
 assert(preview:find('glass_core:',1,true) and preview:find('1,11',1,true) and preview:find('2,11',1,true),'same-count mixed tier replacement hidden')
end)
T.done()
