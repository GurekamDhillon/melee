local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={};for _,n in ipairs({'mod_schema','mod_codec','mod_engine','mod_pool','drive_loot','drive_bag','menu_input','drive_menu'}) do D[n]=T.module(n,D) end
D.pickup_juice={pitch={},new=function() return {drop=function() return {fx={}} end,collect=function() end,expire=function() end,clear=function() end,tick=function() end} end}
D.drive_drop=T.module('drive_drop',D);D.drive_lab=T.module('drive_lab',D)
local function fixture()
 local s={paused=false,spawns=0,commands={},players={{x=0,y=0},{x=30,y=0}},resumes=0}
 local g={command=function(n,f)s.commands[n]=f end,log=function()end,player=function(p)return s.players[p]end,
  paused=function()return s.paused end,pause=function()s.paused=true end,resume=function()s.paused=false;s.resumes=s.resumes+1 end,
  pad=function()return {}end,input_mask=function(_,m)s.mask=m end,item_spawn=function(_,_,_,o)s.spawns=s.spawns+1;s.payload=o.payload;return 100+s.spawns end,
  item_despawn=function(h)s.despawn=h end,items=function()return {}end}
 local lab={engine=D.mod_engine.new(104729,D.mod_pool),allowed=function()return true end,replaying=function()return s.replay end,display={warm=function()return true end},options={}}
 return s,D.drive_lab.new(g,lab)
end
T.test('give queues until frame, equip derives rules and implicit',function()
 local s,a=fixture();assert(a:command('give rare 32'));assert(#a.bag.items==0);a:apply();assert(#a.bag.items==1)
 a:queue('equip',1,1);a:apply();assert(a.bag.equipped[1]);assert(a.lab.engine.display.drive_build[1][1].rarity=='rare')
 a:queue('unequip',1);a:apply();assert(not next(a.lab.engine.equipped[1] or {}));assert(not next(a.lab.engine.implicits[1] or {}))
end)
T.test('ground drops reserve space and pickup rewards exact record once',function()
 local s,a=fixture();for i=1,11 do assert(a:command('give common '..i));a:apply() end
 assert(a:command('drop rare 40'));assert(not a:command('give common 99'));assert(not a:command('drop common 98'))
 local event={name="drive",item=101,port=1,payload=s.payload};local record=a.drops.records[s.payload.amount].record
 a:pickup(event);assert(#a.bag.items==12);assert(a.loot:name(a.bag.items[12])==a.loot:name(record));a:pickup(event);assert(#a.bag.items==12)
 local saved=D.mod_codec.encode(a:snapshot());a.bag:discard(1);a:restore(D.mod_codec.decode(saved));assert(#a.bag.items==12)
end)
T.test('drop map checkpoint restore never duplicates native items',function()
 local s,a=fixture();assert(a:command('drop magic 12'));local saved=D.mod_codec.encode(a:snapshot())
 a.drops:expire{item=101,payload=s.payload};assert(a.drops:count()==0);a:restore(D.mod_codec.decode(saved));assert(a.drops:count()==1 and s.spawns==1)
 s.replay=true;assert(not a:command('drop rare 8'));assert(s.spawns==1)
end)
T.test('controller closes only its pause and releases input mask',function()
 local s,a=fixture();a.menu:close();assert(s.mask==nil and s.resumes==0);a.menu:open();assert(s.paused);a.menu:close();assert(not s.paused and s.mask==0 and s.resumes==1)
 s.paused=true;a.menu:open();a.menu:close();assert(s.paused and s.resumes==1)
end)
T.test('persistence switch retains build while default clear starts fresh',function()
 local s,a=fixture();assert(a:command('give rare 9'));a:apply();assert(a:queue('equip',1,1));a:apply();a:clear();assert(not a:has_build())
 D.drive_lab.tuning.persist=true;local _,b=fixture();assert(b:command('give rare 9'));b:apply();assert(b:queue('equip',1,1));b:apply();b:clear();assert(b:has_build());D.drive_lab.tuning.persist=false
end)
T.test('paused draft consecutive equips choose different records before commit',function()
 local s,a=fixture();assert(a:command('give rare 3'));assert(a:command('give rare 4'));a:apply()
 local first,second=a.bag.items[1].seed,a.bag.items[2].seed;a.menu:open()
 assert(a:queue('equip',1,1));assert(a:view().items[1].seed==second)
 assert(a:queue('equip',1,2));assert(#a.bag.items==2 and not a.bag.equipped[1])
 a:apply();assert(a.bag.equipped[1].seed==first and a.bag.equipped[2].seed==second)
end)
T.test('debug build survives bag edits and rejects combined second keystone atomically',function()
 local s,a=fixture();a.lab.debug_equipped={[1]={glass_core=1,pyromancer=1}}
 assert(a:command('give common 1'));a:apply();assert(a.lab.engine.equipped[1].glass_core)
 local key;for _,r in ipairs(a.lab.engine.list) do if r.kind=='keystone' and r.id~='pyromancer' then key=r.id;break end end
 assert(key);assert(not a:queue('choose_keystone',key));assert(not a.bag.keystone and #a.pending==0)
end)
T.test('detail pages stay within small panel and clear despawns owned handles',function()
 local s,a=fixture();assert(a:command('give rare 12'));a:apply();a.menu:open();a.menu.focus=5
 local max_y=0;a.g.safe_area=function()return{x=0,y=0,w=640,h=360}end
 a.g.kit={panel=function()end,list=function()end,text=function(_,y)max_y=math.max(max_y,y)end};a.menu:draw();assert(max_y<=336)
 a.menu:close();assert(a:command('drop common 20'));a.drops:clear();assert(s.despawn==101)
end)
T.test('live full bag cannot use pending equip capacity for physical drop',function()
 local s,a=fixture();for i=1,12 do assert(a:command('give common '..i));a:apply() end
 assert(a:queue('equip',1,1));assert(not a:command('drop rare 40'));assert(s.spawns==0)
 a:apply();assert(a:command('drop rare 40'));assert(not a:queue('give',a.loot:roll(4,1,'common')))
 a:pickup{name='drive',port=1,item=101,payload=s.payload};assert(#a.bag.items==12)
end)
T.test('thirteenth edit is graceful refusal and wrapped words are complete',function()
 local _,a=fixture();for i=1,12 do assert(a:queue('choose_keystone',nil)) end
 local ok,result=pcall(a.queue,a,'choose_keystone',nil);assert(ok and result==false and a.menu.notice)
 local text='Burning Red Drive of the Updraft: aerial hits apply a powerful burning status'
 local lines=D.drive_menu.wrap({measure=function(s)return #s*9 end},text,100)
 assert(table.concat(lines,' ')==text);for _,line in ipairs(lines) do assert(#line*9<=100) end
end)
T.test('strict checkpoint validation rejects malformed and overbooked drafts atomically',function()
 local _,a=fixture();assert(a:command('give common 1'));a:apply();local original=D.mod_codec.encode(a:snapshot())
 for _,mutate in ipairs({function(s)s.junk=true end,function(s)s.drops.junk=true end,function(s)s.pending={junk={op='discard',a=1}}end,function(s)s.pending={[2]={op='discard',a=1}}end,function(s)s.pending={{op='discard',a=1,junk=true}}end}) do
  local s=D.mod_codec.decode(original);mutate(s);assert(not pcall(a.restore,a,s));assert(D.mod_codec.encode(a:snapshot())==original)
 end
 local s=D.mod_codec.decode(original);s.drops={next_id=13,records={}};for i=1,12 do s.drops.records[i]={handle=100+i,record=a.loot:roll(i,1,'common')} end
 assert(not pcall(a.restore,a,s));assert(D.mod_codec.encode(a:snapshot())==original)
end)
T.test('seedless rolls advance checkpoint seed and restored drop escapes retired cleanup',function()
 local s,a=fixture();assert(a:command('give common'));local first=a.pending[1].a.seed;a:apply();local saved=a:snapshot()
 assert(a:command('give common'));assert(a.pending[1].a.seed~=first);a:restore(saved);assert(a:command('give common'));assert(a.pending[1].a.seed==saved.seed)
 a:apply();assert(a:command('drop rare 3'));local d=a.drops:snapshot();a.drops.retired={[101]=true};a.drops:restore(d);a.drops:retry_retired();assert(not s.despawn)
 a.menu:open();assert(not a:command('drop rare 3'))
end)
T.test('occupied swap cannot race native collection and ordinary item rows are harmless',function()
 local s,a=fixture();for i=1,3 do assert(a:command('give common '..i));a:apply() end
 assert(a:queue('equip',1,1));a:apply();assert(a:command('drop rare 40'))
 assert(not a:queue('equip',1,1));assert(not a:queue('equip',2,2));assert(not a:queue('discard',1))
 assert(a:queue('choose_keystone',nil));assert(a.bag.equipped[1].seed==1)
 local saved=a:snapshot();saved.pending={{op='equip',a=1,b=1}};assert(not pcall(a.restore,a,saved))
 a.g.items=function()return{{name='vanilla:1',id=55},{handle=101,name='drive'}}end
 a:restore(a:snapshot());a.drops:retry_retired()
 a:pickup{name='drive',port=1,item=101,payload=s.payload};a:apply()
 assert(a.bag.equipped[1].seed==1 and #a.bag.items==3 and a.bag.items[3].seed==40)
 a.drops:restore{next_id=1,records={}};a.drops:retry_retired()
end)
T.test('family budget details show all numeric totals safety strength and prospective deltas',function()
 local _,a=fixture();assert(a:command('give rare 12'));a:apply()
 local lines=a:budget_lines();local text=table.concat(lines,' ')
 for _,f in ipairs({'Damage dealt','Launch dealt','Damage taken','Launch taken','Speed'}) do assert(text:find(f,1,true),'missing '..f) end
 assert(text:find('Strength',1,true) and text:find('safety',1,true))
 assert(table.concat(a:delta(1,1),' '):find('Family',1,true))
 a.menu:open();a.menu.focus=1;a.menu.page=2;local shown={};a.g.safe_area=function()return{x=0,y=0,w=640,h=360}end
 a.g.kit={panel=function()end,list=function()end,text=function(_,y,t)assert(y<=336);shown[#shown+1]=t end};a.menu:draw();assert(#shown>0)
end)
T.test('empty player derivation retires neutral roots',function()
 local _,a=fixture();a:apply();assert(a.lab.engine.equipped[1]==nil and a.lab.engine.implicits[1]==nil)
end)
T.done()
