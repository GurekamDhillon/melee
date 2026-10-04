local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={};for _,name in ipairs({'mod_schema','mod_codec','mod_engine','mod_pool'}) do D[name]=T.module(name,D) end
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
D.mod_lab=T.module('mod_lab',D)
local function fixture()
 local s={ready=true,lab=true,net=false,replay=false,commits=0,clears=0,visual_clear=0,mods={},hit_rules={},players={
  [1]={percent=30,stocks=4,falls=0,char=1,action=14,airborne=false},
  [2]={percent=20,stocks=4,falls=0,char=2,action=14,airborne=false,cpu=true}}}
 local g={fixture=s,sim_supported=true,hit_rule_add=function()error('direct hit-rule write is not replayable')end,fighter_status=function()error('direct status write is not replayable')end,command=function(_,fn) s.command=fn end,log=function() end,
  match=function() return {active=true,netplay=s.net} end,lab_mode=function() return s.lab end,
  player=function(p) return s.players[p] end,sim_replaying=function() return s.replay end,
  hit_rules=function(p) return {owner=s.hit_rules[p] and 7 or 0} end,
  sim_read=function() return s.blob end,sim_clear=function() s.clears=s.clears+1;s.blob=nil;s.mods={};s.hit_rules={} end,
  sim_commit=function(blob,ops)
   if s.refuse then error("checkpoint budget exhausted") end
   s.commits=s.commits+1;s.blob=blob;s.ops=ops
   for _,e in ipairs(ops) do if e.op=='fighter_mod' then s.mods[e.port]=e.values elseif e.op=='hit_rules' then s.hit_rules[e.port]={rules=e.rules,bits=e.status_bits} else s.players[e.port].percent=e.value end end
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
 s.ready=true;a:frame();assert(s.mods[2].damage_dealt==2 and s.mods[2].damage_taken==2)
 assert(a.engine.equipped[2].glass_core==1 and #a.pending==0)
end)
T.test('combat hooks queue captured contexts until one frame commit',function()
 local s,a=fixture();assert(a:command('add kindling'));assert(a:command('add pyre'));a:frame()
 local before=s.commits
 a:hit(1,2,{context_valid=true,element_tag='fire',move_tag='aerial',attacker_damage=7,victim_damage=9,attacker_grounded=false,victim_grounded=true})
 assert(s.commits==before and not a.engine:status(2,'burn'))
 local e=a.engine.queue[1];assert(e.tags.fire and e.tags.aerial and e.self_context.percent==7 and e.self_context.grounded==false and e.self_context.stocks==4)
 a:frame();assert(s.commits==before+1 and a.engine:status(2,'burn') and not a.engine:status(2,'curse'))
 assert(not s.mods[2] and s.hit_rules[1].rules[1].change.knockback_taken==1.25 and s.hit_rules[2].bits==1)
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
 assert(a.engine.frame==frame and a.owned[1] and a.engine.equipped[1].glass_core)
 local commits=s.commits;s.replay=true;a:hit(1,2,{context_valid=true,element_tag='fire'});a:frame()
 assert(s.commits==commits and #a.engine.queue==0)
end)
T.test('manual clear is immediate with no future frame and missing blob retires stale ownership',function()
 local s,a=fixture();assert(a:command('add glass_core'));a:frame();assert(s.mods[1])
 assert(a:command('clear'));assert(not s.mods[1] and not a.enabled and not next(a.engine.statuses))
 s.mods[1]={damage_dealt=2};s.blob=nil;a:loadstate();assert(not s.mods[1] and not a.enabled)
end)
T.test('respawn and character replacement retain build until scene ends',function()
 local s,a=fixture();assert(a:command('add glass_core'));a:frame()
 local clears=s.visual_clear;s.players[1].action=12;a:frame();assert(s.mods[1].damage_dealt==2 and a.engine.equipped[1].glass_core and a.enabled)
 s.players[1].action=14;assert(a:command('add glass_core'));a:frame();s.players[1]=nil;a:frame();assert(not s.mods[1])
 a:scene();assert(not a.enabled and not next(a.owned))
 s.lab=false;assert(not a:command('add kindling'));assert(not a:frame())
 s.lab=true;s.net=true;assert(not a:command('add kindling'))
end)
T.test('queued keystones validated and illegal ports refused',function()
 local s,a=fixture();assert(not a:command('add kindling 7'));assert(not a:command('add missing'))
 assert(a:command('intensity 0'));assert(a.engine.display.intensity==0)
 assert(not a:command('intensity 2'));assert(a:command('add still_heart'));assert(a:command('add still_heart'))
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
 local s,a=fixture();assert(a:command('add glass_core'));a:frame();assert(s.mods[1])
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
 a:stock_lost(1);a:frame();assert(a.enabled and s.mods[1].damage_dealt==2 and a.engine.equipped[1].glass_core)
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
T.done()
