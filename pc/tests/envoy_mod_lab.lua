local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={};for _,name in ipairs({'mod_schema','mod_codec','mod_engine','mod_pool','mod_echo_lab'}) do D[name]=T.module(name,D) end
D.mod_display={new=function(g,engine)
 local v={engine=engine}
 function v:warm(e) self.engine=e;return g.fixture.ready,'warming' end
 function v:update(e) self.engine=e;e.display.trace_key=table.concat(e.trace,',') end
 function v:clear() g.fixture.visual_clear=g.fixture.visual_clear+1 end
 function v:intensity(n) self.engine.display.intensity=n end
 function v:on_loadstate(e) self.engine=e;self:update(e) end
 function v:tick(e) g.fixture.visual_ticks=(g.fixture.visual_ticks or 0)+1;self:warm(e) end
 function v:draw() end
 return v
end}
for _,n in ipairs({'drive_loot','drive_bag','foe_roll'}) do D[n]=T.module(n,D) end
T.test('foe LAB adapter exists',function()assert(io.open(T.root..'foe_lab.lua'),'foe LAB missing')end)
D.foe_lab=T.module('foe_lab',D)
D.pickup_juice={pitch={},new=function()return {clear=function()end,tick=function()end}end}
for _,n in ipairs({'menu_input','drive_menu','drive_drop','drive_lab'}) do D[n]=T.module(n,D) end
D.mod_lab=T.module('mod_lab',D)
local function fixture()
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
T.test('LAB modifiers dormant until explicit command',function()
 local s,a=fixture();assert(not a:frame());assert(s.commits==0)
 a:hit(1,2,{context_valid=true,element_tag='fire'});assert(#a.engine.queue==0)
 s.replay=true;assert(a:frame() and s.commits==0);s.replay=false
 assert(a:command('list'));assert(a:command('trace'));assert(s.commits==0)
end)
T.test('warm gate queues equip and no overlay before ready',function()
 local s,a=fixture();s.ready=false;assert(a:command('add glass_core 2'))
 a:frame();assert(not s.mods[2] and #a.pending==1)
 s.ready=true;a:frame();assert(s.mods[2]==nil and a.engine:values(2).damage_dealt==1.6)
 assert(a.engine.equipped[2].glass_core==1 and #a.pending==0)
end)
T.test('combat hooks queue captured contexts until one frame commit',function()
 local s,a=fixture();assert(a:command('add kindling'));assert(a:command('add pyre'));a:frame()
 local before=s.commits
 a:hit(1,2,{context_valid=true,element_tag='fire',move_tag='aerial',attacker_damage=7,victim_damage=9,attacker_grounded=false,victim_grounded=true})
 assert(s.commits==before and not a.engine:status(2,'burn'))
 local e=a.engine.queue[1];assert(e.tags.fire and e.tags.aerial and e.self_context.percent==7 and e.self_context.grounded==false and e.self_context.stocks==4)
 a:frame();assert(s.commits==before+1 and a.engine:status(2,'burn') and not a.engine:status(2,'curse'))
 assert(not s.mods[2] and s.hit_rules[1].rules[1].change.launch==1.08 and s.hit_rules[2].bits==1)
end)
T.test('KO consumes target status before stock cleanup then heals',function()
 local s,a=fixture();for _,id in ipairs({'kindling','pyre','feasting'}) do assert(a:command('add '..id)) end
 a:frame();a:hit(1,2,{context_valid=true,element_tag='fire'});a:frame()
 a:ko(1,2);a:stock_lost(2);a:frame()
 assert(s.players[1].percent==20 and not a.engine.statuses[2] and not s.mods[2])
 assert(table.concat(a.engine.trace,' '):find('Feasting',1,true))
end)
T.test('checkpoint restores adapter lifecycle and engine roots',function()
 local s,a=fixture();assert(a:command('add glass_core'));a:frame()
 local blob=s.blob;local frame=a.engine.frame
 a:frame();assert(a.engine.frame>frame);s.blob=blob;a:loadstate()
 assert(a.engine.frame==frame and a.hit_owned[1] and a.engine.equipped[1].glass_core)
 local commits=s.commits;s.replay=true;a:hit(1,2,{context_valid=true,element_tag='fire'});a:frame()
 assert(s.commits==commits and #a.engine.queue==0)
end)
T.test('manual clear is immediate with no future frame and missing blob retires stale ownership',function()
 local s,a=fixture();assert(a:command('add glass_core'));a:frame();assert(s.hit_rules[1])
 assert(a:command('clear'));assert(not s.mods[1] and not a.enabled and not next(a.engine.statuses))
 s.mods[1]={damage_dealt=2};s.blob=nil;a:loadstate();assert(not s.mods[1] and not a.enabled)
end)
T.test('respawn and character replacement retain build until scene ends',function()
 local s,a=fixture();assert(a:command('add glass_core'));a:frame()
 local clears=s.visual_clear;s.players[1].action=12;a:frame();assert(a.engine:values(1).damage_dealt==1.6 and a.engine.equipped[1].glass_core and a.enabled)
 s.players[1].action=14;assert(a:command('add glass_core'));a:frame();s.players[1]=nil;a:frame();assert(not s.mods[1])
 a:scene();assert(not a.enabled and not next(a.owned))
 s.lab=false;assert(not a:command('add kindling'));assert(not a:frame())
 s.lab=true;s.net=true;assert(not a:command('add kindling'))
end)
T.test('queued keystones validated and illegal ports refused',function()
 local s,a=fixture();assert(not a:command('add kindling 7'));assert(not a:command('add missing'))
 assert(a:command('intensity 0'));assert(a.engine.display.intensity==0)
 assert(not a:command('intensity 2'));assert(a:command('add pyromancer'));assert(a:command('add pyromancer'))
end)
T.test('invalid hit context stays unclassified and nil attacker still emits taken',function()
 local s,a=fixture();assert(a:command('add kindling'));a:frame()
 a:hit(nil,2,{item=true,element_tag='fire',move_tag='aerial',attacker_damage=4,victim_damage=9})
 assert(#a.engine.queue==1);local e=a.engine.queue[1]
 assert(e.kind=='hit_taken' and e.target==nil and next(e.tags)==nil and e.self_context.percent==nil)
 a:frame();assert(not a.engine:status(2,'burn'))
 assert(not a:command('clear 2'));assert(a.enabled)
end)
T.test('checkpoint refusal retires old gameplay and mutated Lua state',function()
 local s,a=fixture();assert(a:command('add glass_core'));a:frame();assert(s.hit_rules[1])
 s.refuse=true;a:frame();assert(not a.enabled and not s.mods[1] and not next(a.engine.equipped))
 assert(s.clears==1)
end)
T.test('paused host tick polls display without advancing gameplay timers',function()
 local s,a=fixture();assert(a:command('add glass_core'));a:frame()
 local frame,commits=a.engine.frame,s.commits;a:tick();a:tick()
 assert(a.engine.frame==frame and s.commits==commits and s.visual_ticks==2)
end)
T.test('last stock and final status expiry retire all visual resources immediately',function()
 local s,a=fixture();assert(a:command('add glass_core'));a:frame();local cleared=s.visual_clear
 a:stock_lost(1);a:frame();assert(a.enabled and a.engine:values(1).damage_dealt==1.6 and a.engine.equipped[1].glass_core)
 local blob=s.blob;s.visual_clear=0;a:loadstate();assert(a.enabled and a.engine.equipped[1].glass_core)
 s.blob=blob;a:command("clear");a.enabled=true;a.engine.statuses[2]={haste={expires=a.engine.frame+1,next_tick=100,stacks=1,max=1,amount=1,origin={}}}
 a:frame();assert(not a.enabled and not next(a.engine.statuses))
end)
T.test('native conversions and statuses are journaled without direct setters',function()
 local s,a=fixture();assert(a:command('add burning'));assert(a:command('add kindling'));assert(a:command('add pyre'));a:frame()
 local rules=s.hit_rules[1].rules;assert(#rules==2 and rules[1].change.element=='fire')
 a:hit(1,2,{context_valid=true,element_tag='fire',move_tag='smash',hit_rule_ids={rules[1].id},hit_rule_owners={7}});a:frame()
 assert(s.hit_rules[2].bits==1 and table.concat(a.engine.trace,' '):find('Burning',1,true))
 local saved=s.blob;a:command('clear');assert(not next(s.hit_rules));s.blob=saved;a:loadstate()
 assert(a.hit_owned[1] and a.hit_owned[2] and a.engine.equipped[1].burning)
 a:stock_lost(2);a:frame();assert(s.hit_rules[2].bits==0 and #s.hit_rules[2].rules==0)
end)
T.test('native traces refuse another script with the same rule ID',function()
 local s,a=fixture();assert(a:command('add burning'));a:frame()
 local id=s.hit_rules[1].rules[1].id
 a:hit(1,2,{context_valid=true,hit_rule_ids={id},hit_rule_owners={8}})
 assert(#a.engine.queue[1].origin==0)
end)
T.test('clank captures native pair damage and opposite fighter without inventing item target',function()
 local s,a=fixture();assert(a:command('add glass_core'));a:frame()
 a:clank{port_a=1,port_b=2,damage_a=8,damage_b=12}
 local x,y=a.engine.queue[1],a.engine.queue[2]
 assert(x.port==1 and x.target==2 and y.port==2 and y.target==1)
 assert(x.damage_a==8 and x.damage_b==12 and y.damage_a==8 and y.damage_b==12)
 a.engine.queue={};a:clank{port_a=1,damage_a=8,damage_b=12};assert(#a.engine.queue==0)
 a:clank{port_a=1,port_b=0,damage_a=8,damage_b=12};assert(#a.engine.queue==0)
 a:clank{port_a=1,port_b=2,damage_a=0/0,damage_b=math.huge}
 assert(a.engine.queue[1].damage_a==nil and a.engine.queue[1].damage_b==nil)
end)
T.test('stale native refuses before queuing any edit',function()
 local s,a=fixture();s.capability=false;assert(not a:command('add glass_core'));assert(#a.pending==0 and not a.enabled)
end)
T.test('percent and launch values never enter fighter overlays',function()
 local s,a=fixture();assert(a:command('add glass_core'));a:frame()
 for _,op in ipairs(s.ops) do if op.op=='fighter_mod' and op.values then assert(not op.values.damage_dealt and not op.values.damage_taken and not op.values.knockback_taken and not op.values.status_duration) end end
 assert(s.hit_rules[1] and #s.hit_rules[1].rules>0)
end)
T.test('foes queue warmup and clear cancels pending without touching P1',function()
 local s,a=fixture();assert(a:command('add kindling'));a:frame();s.ready=false
 assert(a.foes:command('roll 1.4 25 2'));assert(not a.engine.equipped[2]);a:frame();assert(#a.foes.pending==1)
 assert(a.foes:command('clear'));s.ready=true;a:frame();assert(not next(a.engine.equipped[2] or {}) and a.engine.equipped[1].kindling)
 assert(a.foes:command('roll 1.4 25 2'));a:frame();assert(a.foes.builds[2] and a.foes.labels[2].left>0)
 local saved=s.blob;a.foes:command('clear');a:frame();s.blob=saved;a:loadstate();assert(a.foes.builds[2] and a.foes.labels[2])
end)
T.test('foe guards and CPU mode refusal do not fake acceptance',function()
 local s,a=fixture();assert(not a.foes:command('roll 1.4 1 1'));s.players[2].cpu=false;assert(not a.foes:command('roll'));s.players[2].cpu=true
 s.replay=true;assert(not a.foes:command('roll'));s.replay=false;s.net=true;assert(not a.foes:command('roll'));s.net=false
 s.cpu_refuse=true;assert(not a.foes:command('fight'));s.cpu_refuse=false;assert(a.foes:command('fight'));assert(s.mode.mode=='fight')
 assert(a.foes:command('stand'));assert(s.mode.mode=='stand')
end)
T.test('CPU engine statuses and native percent rules affect P1 and retire on clear',function()
 local s,a=fixture();assert(a.foes:command('roll 1.4 32 2'));a:frame();a.engine:set_build(2,{burning=1,kindling=1,glass_core=1},{})
 a:hit(2,1,{context_valid=true,element_tag='fire'});a:frame();assert(a.engine:status(1,'burn'));assert(s.hit_rules[2])
 assert(a.foes:command('clear'));a:frame();assert(not a.engine.statuses[1] and not a.engine.equipped[2]);assert(#s.hit_rules[2].rules==0)
end)
T.test('high valid player strength defaults to independent scalar targeted roll',function()
 local s,a=fixture();local mods={};for _,m in ipairs(a.engine.list) do if m.kind=='normal' and not m.min_depth then local native=false;for _,e in ipairs(m.effects) do if e.op=='convert' or e.op=='versus-status' then native=true end end;if not native then mods[m.id]=#m.tiers end end end
 a.engine:set_build(1,mods,{});local _,strength=a.engine:family_budget(1);assert(strength>1.8)
 assert(a.foes:command('roll'));a:frame();local r=a.foes.builds[2];assert(r.strength>=strength*.8 and r.strength<=strength*1.2 and not r.reference and not r.mods)
end)
T.test('corrupt foe checkpoint leaves all live roots untouched',function()
 local s,a=fixture();assert(a.foes:command('roll 1.4 32 2'));a:frame();local before=a:export();local bad=D.mod_codec.decode(s.blob);bad.foes.builds[2].strength=99;s.blob=D.mod_codec.encode(bad)
 T.refuses(function()a:loadstate()end);assert(a:export()==before)
end)
T.test('full bag and CPU pending label lifecycle checkpoint restore exactly',function()
 local s,a=fixture();for i=1,12 do assert(a.drives:command('give rare '..i));a:frame() end
 assert(a.drives:queue('equip',1,1));a:frame();assert(a.drives:command('give common 44'));a:frame()
 assert(a.foes:command('roll 1.4 41 2'));a:frame();assert(a.foes:command('roll 1.4 43 2'));s.ready=false;a:frame()
 assert(#a.drives.bag.items==12 and #a.foes.pending==1);local blob=a:export();s.blob=blob
 a.foes:command('clear');s.ready=true;a:frame();s.blob=blob;a:loadstate();assert(a:export()==blob)
 s.refuse=true;a:frame();assert(not a.enabled and not next(a.foes.builds) and #a.foes.pending==0)
end)
T.test('every accepted pending CPU equip fits native rule capacity before commit',function()
 local _,a=fixture()
 for _,m in ipairs(a.engine.list) do if m.kind=='normal' then
  local native=false;for _,e in ipairs(m.effects) do if e.op=='convert' or e.op=='versus-status' then native=true end end
  if native and a:command('add '..m.id..' 2') then local probe=D.mod_engine.new(1,D.mod_pool);for _,e in ipairs(a.pending) do probe:equip(e.port,e.id) end;assert(pcall(probe.native_rules,probe,2),'accepted pending build overflow') end
 end end
end)
T.test('five CPU nameplates paginate inside 640 by 360 viewport',function()
 local s,a=fixture();for p=2,6 do s.players[p]={cpu=true,percent=0,stocks=4,falls=0,char=p,action=14};assert(a.foes:command('roll 1.4 '..p..' '..p)) end;a:frame()
 a.g.safe_area=function()return{x=0,y=0,w=640,h=360}end;a.g.kit={panel=function(_,y,_,h)assert(y+h<=336,'nameplate overflow')end,text=function(_,y)assert(y<=336)end};a.foes:draw()
end)
T.test('foe list exposes all rolled records without mutating builds',function()
 local _,a=fixture();assert(a.foes:command('roll 1.4 99'));a:frame();local before=a:export();assert(a.foes:command('list'));assert(before==a:export())
end)
T.test('invalid pending checkpoint cannot publish any adapter state',function()
 local s,a=fixture();assert(a:command('add kindling'));a:frame();local before=a:export();local bad=D.mod_codec.decode(before);bad.pending={{port=2,id='missing'}};s.blob=D.mod_codec.encode(bad)
 T.refuses(function()a:loadstate()end);assert(before==a:export())
end)
T.test('CPU disappearing during shader warmup disables pending build safely',function()
 local s,a=fixture();s.ready=false;assert(a.foes:command('roll 1.4 32 2'));s.players[2]=nil;s.ready=true;assert(a:frame());assert(not a.enabled and #a.foes.pending==0 and not next(a.foes.builds))
end)
T.test('invalid drive checkpoint cannot replace prospective debug or pending roots',function()
 local s,a=fixture();assert(a:command('add kindling'));a:frame();local before=a:export();local bad=D.mod_codec.decode(before)
 bad.pending={{port=2,id='burning'}};bad.debug_equipped[1]={kindling=1,charged=1};bad.drives.seed=0;s.blob=D.mod_codec.encode(bad)
 T.refuses(function()a:loadstate()end);assert(before==a:export(),'invalid drive replaced live state')
end)
T.test('foe and mod CPU queues refuse conflicts in either command order preserving inventory',function()
 for _,id in ipairs({'frozen_oath','burning'}) do for _,first in ipairs({'foe','mod'}) do
  local _,a=fixture();assert(a.drives:command('give rare 77'));a:frame();local bag=D.mod_codec.encode(a.drives:snapshot())
  if first=='foe' then assert(a.foes:command('roll 1.8 1 2'));assert(not a:command('add '..id..' 2'),'accepted mod alongside foe pending')
  else assert(a:command('add '..id..' 2'));assert(not a.foes:command('roll 1.8 1 2'),'accepted foe alongside mod pending') end
  assert(a:frame());assert(bag==D.mod_codec.encode(a.drives:snapshot()))
 end end
end)
T.test('unexpected pending publication failure does not escape frame or replace P1 inventory',function()
 local s,a=fixture();assert(a.drives:command('give rare 78'));a:frame();local bag=D.mod_codec.encode(a.drives:snapshot());assert(a:command('add kindling 2'))
 a.engine.equip=function()error('unexpected publication failure')end
 local ok=pcall(a.frame,a);assert(ok,'pending error escaped frame');assert(not a.enabled and #a.pending==0);assert(bag==D.mod_codec.encode(a.drives:snapshot()))
end)
T.test('combined actual HUD and CPU plates never paint over each other',function()
 local s,a=fixture();D.mod_tuning=D.mod_tuning or T.module('mod_tuning',D);D.mod_tuning.set_dev_ui(true)   -- the Modifier LAB box is a developer overlay
 s.players[3]={cpu=true,percent=0,stocks=4,falls=0,char=3,action=14};assert(a.foes:command('roll 1.4 32 2'));assert(a.foes:command('roll 1.4 33 3'));a:frame()
 a.engine.statuses[1]={burn={stacks=1,expires=200}};local fills,panels=0,0;a.g.safe_area=function()return{x=0,y=0,w=640,h=360}end
 a.g.kit={available=function()return true end,panel=function(_,y,_,h)assert(y+h<=336);panels=panels+1 end,text=function(_,y)assert(y<=336)end};a.g.fill=function()fills=fills+1 end
 local actual=T.module('mod_display',D);a.display=actual.new(a.g,a.engine);a:draw();assert(panels>0 and fills==0,'debug HUD overlaps foe plate')
 a.foes.labels={};a:draw();assert(fills==1,'debug HUD missing after timed plates retire');D.mod_tuning.set_dev_ui(false)
end)
T.test('clear remains available at full foe queue capacity',function()
 local _,a=fixture();for i=1,12 do assert(a.foes:command('roll 1.4 '..i..' 2')) end;assert(a.foes:command('clear'));assert(#a.foes.pending==1 and a.foes.pending[1].op=='clear')
end)
T.test('legacy jointly queued incompatible CPU keystones refuse before publishing while retaining bag',function()
 local _,a=fixture();assert(a.drives:command('give rare 79'));a:frame();local bag=D.mod_codec.encode(a.drives:snapshot())
 local build={items={},equipped={},keystones={'pyromancer'},context=D.mod_progression.context()};local mods,implicit=D.drive_bag.new(a.drives.loot):validate(build);local _,strength=D.mod_budget.build(D.mod_pool,mods,implicit,{});local r={seed=1,stage=32,port=2,requested=strength,target=strength,strength=strength,build=build,context=D.mod_progression.context(),role='normal'}
 a.foes.roller:validate(r);a.foes.pending={{op='roll',record=r}};a.pending={{port=2,id='frozen_oath'}};a.enabled=true
 assert(pcall(a.frame,a));assert(not a.enabled and not next(a.foes.builds));assert(bag==D.mod_codec.encode(a.drives:snapshot()))
end)
T.test('drive restore validates against prospective debug context not previous live keystone',function()
 local s,a=fixture();assert(a:command('add pyromancer'));a:frame();local bad=D.mod_codec.decode(a:export());bad.debug_equipped={[1]={frozen_oath=1}};bad.pending={};bad.drives.bag.keystone='frozen_oath'
 local engine=D.mod_codec.decode(bad.engine);engine.equipped[1]={frozen_oath=1};bad.engine=D.mod_codec.encode(engine);s.blob=D.mod_codec.encode(bad)
 a:loadstate();assert(a.debug_equipped[1].frozen_oath and a.drives.bag.keystone=='frozen_oath')
end)
T.test('finite raw native coefficients clamp once at contact and malformed implicit refuses atomically',function()
 for _,case in ipairs({
  {mods={storm_shell=1},implicit={damage_dealt=.01}},
  {mods={glass_core=1},implicit={damage_dealt=65}},
  {mods={},implicit={knockback_taken=.01}},
  {mods={},implicit={knockback_taken=5}},
  {mods={glass_core=100},implicit={},outgoing=64}
 }) do
  local s,a=fixture();assert(a:command('add kindling'));a:frame();local before=a:export();local bad=D.mod_codec.decode(before);local engine=D.mod_codec.decode(bad.engine)
  engine.equipped[2]=case.mods;engine.implicits[2]=case.implicit;bad.engine=D.mod_codec.encode(engine);s.blob=D.mod_codec.encode(bad)
  a:loadstate();local rules=a.engine:native_rules(2);for _,r in ipairs(rules)do for key,v in pairs(r.change)do if key=='percent_damage' or key=='launch' then assert(v==v and math.abs(v)<=1e9)end end end
  local out=a.engine:contact_ratios(2,1);local incoming=a.engine:contact_ratios(1,2)
  assert(out.outgoing>=.05 and out.outgoing<=64 and out.launch_out>=.05 and out.launch_out<=4)
  assert(incoming.incoming>=.15 and incoming.incoming<=64 and incoming.launch_in>=.05 and incoming.launch_in<=4)
  if case.outgoing then assert(out.outgoing==case.outgoing)end
  local stable=a:export();bad=D.mod_codec.decode(stable);engine=D.mod_codec.decode(bad.engine);engine.implicits[2]={damage_dealt=1000001};bad.engine=D.mod_codec.encode(engine);s.blob=D.mod_codec.encode(bad)
  T.refuses(function()a:loadstate()end);assert(stable==a:export(),'malformed implicit replaced live state')
 end
end)
T.test('echo LAB commands journal and checkpoint cleanup preserve live builds',function()
 local s,a=fixture();assert(a.echoes:command('add 24 nair .4'));a:frame();assert(s.echoes[1][1].delay==24)
 local blob=s.blob;assert(a.echoes:command('clear'));a:frame();assert(#s.echoes[1]==0)
 s.blob=blob;a:loadstate();assert(a.echoes.manual[1][1].match.move=='nair')
 a:stock_lost(1);a:frame();assert(#s.echoes[1]==0 and not a.echoes.manual[1])
 assert(a:command('add echoes'));a:frame();assert(#s.echoes[1]==1)
 s.players[1]=nil;a:frame();assert(#s.echoes[1]==0)
 a:unload();assert(not a.echoes:active())
end)
T.test('old echo executable refuses explicit capability',function()
 local s,a=fixture();s.echo_capability=false;assert(not a.echoes:command('add 24'))
 assert(not a.echoes:active() and s.commits==0)
end)
T.test('rewind stages checkpoint echo roots independent of future manual rules',function()
 local s,a=fixture();a.engine:set_build(1,{echoes=3},{});s.blob=a:export()
 for i=1,8 do a.echoes.manual[1]=a.echoes.manual[1] or {};a.echoes.manual[1][i]={delay=i,match={move='any'},damage=.4,knockback=1,once_per_move=true} end
 a:loadstate();assert(a.engine.equipped[1].echoes==3 and not a.echoes.manual[1])
 local before=a:export();local bad=D.mod_codec.decode(before);bad.echoes.manual[1]={}
 for i=1,8 do bad.echoes.manual[1][i]={delay=i,match={move='any'},damage=.4,knockback=1,once_per_move=true}end
 s.blob=D.mod_codec.encode(bad);T.refuses(function()a:loadstate()end);assert(a:export()==before)
end)
T.test('manual and queued build commands refuse combined overflow atomically',function()
 local s,a=fixture();for i=1,8 do assert(a.echoes:command('add '..i))end
 local before=a:export();assert(not a:command('add echoes'));assert(a:export()==before and #a.pending==0 and s.clears==0)
 local s2,b=fixture();s2.ready=false;assert(b:command('add echoes'))
 for i=1,7 do assert(b.echoes:command('add '..i))end
 before=b:export();assert(not b.echoes:command('add 8'));assert(b:export()==before)
end)
T.test('publication abandonment retires echo roots and visuals',function()
 local s,a=fixture();assert(a.echoes:command('add 8'));a.echoes.visual[1]=42
 local removed=0;s.g.afterimage_remove=function(h)assert(h==42);removed=removed+1 end
 a.pending={{port=1,id='missing_rule'}};a:frame()
 assert(not a.enabled and not a.echoes:active() and not next(a.echoes.visual) and removed==1)
end)
T.test('drive equip and pending drive roots share manual echo capacity checks',function()
 local function record(a)
  for seed=1,100 do local r=a.drives.loot:roll(seed,a.engine.context,'unique');if r.unique=='echo_heart' then return r end end;error('echo unique witness missing')
 end
 local s,a=fixture();a.drives.bag.items={record(a)};for i=1,6 do assert(a.echoes:command('add '..i))end
 local before=a:export();assert(not a.drives:queue('equip',1,1));assert(a:export()==before and #a.drives.pending==0)
 local s2,b=fixture();b.drives.bag.items={record(b)};assert(b.drives:queue('equip',1,1))
 for i=1,5 do assert(b.echoes:command('add '..i))end
 before=b:export();assert(not b.echoes:command('add 6'));assert(b:export()==before)
end)
T.test('depth command rejects incompatible staged echoes before context publication',function()
 local s,a=fixture();for i=1,8 do assert(a.echoes:command('add '..i))end
 a.pending={{port=1,id='echoes'}};local before=a:export();assert(not a:depth_command('5'));assert(a:export()==before and a.engine.context.depth==0)
end)
T.test('echo command exact native move vocabulary refuses unsupported projectile atomically',function()
 local s,a=fixture();local before=a:export();assert(not a.echoes:command('add 8 projectile'));assert(a:export()==before and s.commits==0)
 for _,move in ipairs{'nair','fair','bair','uair','dair'}do assert(a.echoes:command('add 8 '..move))end
end)
T.test('scene resets echo warm ticket and emitter ownership',function()
 local s,a=fixture();local releases,removed=0,0
 s.g.warm_release=function(h)assert(h==91);releases=releases+1 end;s.g.afterimage_remove=function(h)assert(h==17);removed=removed+1 end
 a.echoes.warm_jobs[1]=91;a.echoes.visual[1]=17;a:scene()
 assert(releases==1 and removed==1 and not next(a.echoes.warm_jobs) and not next(a.echoes.visual))
end)
T.test('canonical fighter bindings checkpoint and clear same-kind replacement transients',function()
 local s,a=fixture();local old='e1:1:1:0:100:1';s.players[1].entity_ref=old
 s.g.entity_valid=function(r)return r==s.players[1].entity_ref end
 s.g.entity_resolve=function(r)if s.g.entity_valid(r)then return{kind='fighter',port=1,sub=false}end end
 assert(a:command('add glass_core'));a:frame();local saved=s.blob
 assert(D.mod_codec.decode(saved).observed[1].entity_ref==old)
 a.engine.statuses[1]={haste={stacks=1,max=8,expires=100,next_tick=100,amount=1,origin={'fixture'}}}
 s.players[1].entity_ref='e1:1:1:0:101:1';a:frame();assert(not a.engine.statuses[1])
 local before=a:export();s.blob=saved;assert(not pcall(a.loadstate,a));assert(a:export()==before,'stale checkpoint changed roots')
 s.players[1].entity_ref=old;assert(pcall(a.loadstate,a));assert(a.observed[1].entity_ref==old)
end)
T.test('a burn tick raises an ordinary fighter damage but never a boss damage (only a hit may end a boss fight)',function()
 local function burned(char)
  local s,a=fixture();s.players[2].char=char;assert(a:command('add kindling'));assert(a:command('add pyre'));a:frame()
  a:hit(1,2,{context_valid=true,element_tag='fire',move_tag='aerial',attacker_damage=7,victim_damage=9,attacker_grounded=false,victim_grounded=true})
  a:frame();assert(a.engine:status(2,'burn'),'burning')
  local before=s.players[2].percent;for _=1,200 do a:frame() end
  return before,s.players[2].percent,s
 end
 local b0,b1=burned(2);assert(b1>b0,'an ordinary fighter burns: '..b0..' -> '..b1)
 -- the engine kinds: CKind_MasterH 0x1A = 26, CKind_CrezyH 0x1E = 30 (27 is the Male Wireframe, a plain fighter)
 local g0,g1=burned(29);assert(g1>g0,'Giga Bowser (29) is an ordinary percent fighter and burns')
 for _,boss in ipairs({26,30}) do local c0,c1,s=burned(boss);assert(c1==c0,'boss char '..boss..' must not burn: '..c0..' -> '..c1) end
end)
T.done()
