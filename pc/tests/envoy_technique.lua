-- Technique triggers, new effect kinds, crit configuration, earned statuses, loot gating and the host's journal ops.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={};for _,name in ipairs({'mod_schema','mod_codec','mod_budget','keystones','mod_pool','mod_engine','mod_echo_lab'}) do D[name]=T.module(name,D) end
D.mod_display={new=function(g,engine)
 local v={engine=engine}
 function v:warm(e) self.engine=e;return true end
 function v:update(e) self.engine=e;e.display.trace_key=table.concat(e.trace,',') end
 function v:clear() end
 function v:intensity() end
 function v:on_loadstate(e) self.engine=e end
 function v:tick() end
 function v:draw() end
 return v
end}
for _,n in ipairs({'drive_loot','drive_bag','foe_roll'}) do D[n]=T.module(n,D) end
D.foe_lab=T.module('foe_lab',D)
D.pickup_juice={pitch={},new=function()return {clear=function()end,tick=function()end}end}
for _,n in ipairs({'menu_input','drive_menu','drive_drop','drive_lab'}) do D[n]=T.module(n,D) end
D.mod_lab=T.module('mod_lab',D)
D.earned_fx=T.module('earned_fx',D)
local P=D.mod_progression
local function engine(build,ctx)
 local e=D.mod_engine.new(1,D.mod_pool,{context=ctx or P.context(40,0)});e:set_build(1,build,{});e:set_build(2,{},{});return e
end
local function frame(e,extra) e:begin_frame({[1]={percent=50,grounded=true,stocks=2,x=0,y=0},[2]={percent=120,grounded=true,stocks=2,x=10,y=0},[3]={percent=0,grounded=true,stocks=1,x=40,y=0},[4]={percent=0,grounded=true,stocks=1,x=15,y=0}}) end

T.test('skill triggers are in the vocabulary: verified ones plain, unverified ones flagged',function()
 local S,K=D.mod_schema,D.mod_skill
 for _,kind in ipairs({'lcancel','lcancel_hit','lcancel_miss','wavedash','perfect_shield','tech','tech_miss','short_hop','fast_fall','dash_dance','jump_cancel_grab','combo','combo_end','crit','armor'}) do assert(S.events[kind] and K.kinds[kind].verified,kind) end
 for _,kind in ipairs({'waveland','ledge_dash','sdi','shield_drop','auto_cancel'}) do assert(S.events[kind] and K.kinds[kind].verified==false,kind) end
 for kind,k in pairs(K.kinds) do assert(k.cpu=='live' or k.cpu=='maybe' or k.cpu=='dead' or k.cpu=='driven',kind);assert(K.cause[k.cause],kind) end
end)
T.test('an L-cancel after a hit earns Haste with its cause; a miss earns nothing; the status ends the earned state',function()
 local e=engine({clean_landing=1})
 frame(e);e:emit{kind='lcancel_miss',port=1,tags={},hit=false};e:drain();assert(not e:status(1,'haste') and not e:earned(1))
 e:emit{kind='lcancel_hit',port=1,tags={},hit=true};e:drain()
 local v=e:status(1,'haste');assert(v and v.cause=='lcancel');local earned=e:earned(1);assert(earned and earned.cause=='lcancel' and earned.frames>0)
 local left=earned.frames
 for _=1,left+1 do frame(e) end
 assert(not e:status(1,'haste') and not e:earned(1),'the earned state ends with the status')
 -- a status from a non-technique trigger is not earned
 local e2=engine({updraft=1,crosswind=1});frame(e2);e2:emit{kind='hit_dealt',port=1,target=2,tags={aerial=true}};e2:drain();assert(e2:status(1,'momentum') and not e2:earned(1))
end)
T.test('conditions: combo count, tech direction, hit flag, aerial, crit strength, armour result',function()
 local e=engine({combo_surge=1});frame(e)
 e:emit{kind='combo',port=1,target=2,tags={},count=2};e:drain();assert(not e:status(1,'haste'))
 e:emit{kind='combo',port=1,target=2,tags={},count=3};e:drain();assert(e:status(1,'haste'))
 local e2=engine({retaliation=1});frame(e2);e2:emit{kind='armor',port=1,tags={},absorbed=false,broke=true};e2:drain();assert(#e2.fx==0)
 e2:emit{kind='armor',port=1,tags={},absorbed=true};e2:drain();assert(#e2.fx==1 and e2.fx[1].op=='crit_next')
 local m=D.mod_pool;local rec
 for _,r in ipairs(m) do if r.id=='tech_guard' then rec=r end end
 local bad=D.mod_codec.decode(D.mod_codec.encode(rec));bad.conditions={{direction='sideways'}};T.refuses(function() D.mod_schema.validate(bad) end)
 bad=D.mod_codec.decode(D.mod_codec.encode(rec));bad.trigger='hit_dealt';bad.conditions={{combo_at_least=3}};T.refuses(function() D.mod_schema.validate(bad) end)
end)
T.test('timed effects: super armour, intangibility and an interrupt window are queued with their own expiry; keystone Wavedasher',function()
 local e=engine({wavedasher=P.tier(P.context(40,0))});frame(e);e:emit{kind='wavedash',port=1,tags={}};e:drain()
 local armor;for _,f in ipairs(e.fx) do if f.op=='armor' then armor=f end end
 assert(armor and armor.type=='super' and armor.frames==6,'wavedash gives 6 frames of super armour')
 assert(e:status(1,'guarded') and e:status(1,'guarded').cause=='wave')
 local e2=engine({phase_dash=P.tier(P.context(40,0))});frame(e2);e2:emit{kind='air_dodge',port=1,tags={}};e2:drain();assert(e2.fx[1].op=='intangible' and e2.fx[1].frames==4)
 local itr=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[1]))
 itr.id='probe_interrupt';itr.kind='normal';itr.cost=nil;itr.affix='suffix';itr.group='probe_interrupt';itr.weight=1;itr.trigger='hit_dealt';itr.conditions={};itr.tags={};itr.families={'interrupt'}
 itr.effects={{op='interrupt',frames=10,exits={'jab','jump'},guard=true}};itr.tiers={{x=1}};itr.text='probe'
 D.mod_schema.validate(itr);T.refuses(function() itr.effects[1].frames=999;D.mod_schema.validate(itr) end)
end)
T.test('safety floors: armour, intangibility and crit values are bounded by the schema and at run time',function()
 local function probe(effect,trigger)
  local r=D.mod_codec.decode(D.mod_codec.encode(D.mod_pool[1]));r.id='probe_x';r.kind='normal';r.cost=nil;r.affix='suffix';r.group='probe_x';r.weight=1
  r.trigger=trigger or 'wavedash';r.conditions={};r.tags={};r.effects={effect};r.tiers={{x=1}};r.text='probe';r.families={};return r
 end
 T.refuses(function() D.mod_schema.validate(probe({op='armor',type='super',frames=31})) end)
 T.refuses(function() D.mod_schema.validate(probe({op='armor',type='super'})) end) -- no permanent super armour
 T.refuses(function() D.mod_schema.validate(probe({op='armor',type='hit_count',value=4,frames=60})) end)
 T.refuses(function() D.mod_schema.validate(probe({op='intangible',frames=25})) end)
 T.refuses(function() D.mod_schema.validate(probe({op='crit',chance=.7,multiplier=2},'equip')) end) -- a certain crit needs a percent floor
 T.refuses(function() D.mod_schema.validate(probe({op='crit',chance=.2,multiplier=5},'equip')) end)
 D.mod_schema.validate(probe({op='crit',chance=1,multiplier=2,min_percent=100},'equip'))
 T.refuses(function() D.mod_schema.validate(probe({op='armor',type='super',frames=6},'equip')) end)
 -- a deep tier cannot push a native value past its bound: crit chance and multiplier are clamped in the derived configuration
 local e=engine({keen=1,brutal=1},P.context(60,3));local cfg=e:crit_config(1);assert(cfg.slots.default.chance<=.6 and cfg.slots.default.multiplier_max<=4)
 -- a deep tier cannot lift the restriction rules: two restrictions at most, never shield with air dodge
 assert(not D.mod_budget.restrictions_ok({'shield','air_dodge'}) and not D.mod_budget.restrictions_ok({'run','grab','shield'}) and D.mod_budget.restrictions_ok({'run'}))
 assert(not D.keystones.check({'aerialist','powershield_oath'}) and not D.keystones.check({'juggernaut','sprinter'}))
end)
T.test('crit: none by default; chance, multiplier, tag slot, percent floor and next-hit-crits derive one configuration',function()
 assert(engine({}):crit_config(1)==nil)
 local e=engine({keen=3,brutal=3,ruthless=1})
 local c=e:crit_config(1);assert(c and c.min_percent==0)
 assert(math.abs(c.slots.default.chance-.075)<1e-9 and math.abs(c.slots.default.mean-(1.5+.375))<1e-9 and c.slots.default.multiplier<c.slots.default.mean and c.slots.default.multiplier_max>c.slots.default.mean,c.slots.default.mean)
 assert(c.slots.aerial and c.slots.aerial.chance>c.slots.default.chance and c.slots.aerial.mean>=1.6)
 local f=engine({finishing=1});assert(f:crit_config(1).min_percent==100)
 local n=engine({wave_edge=3});local cn=n:crit_config(1);assert(cn and cn.slots.default.chance==0 and cn.slots.default.mean>=1.8,'a forced-crit rule needs a configuration to exist')
 local ex=engine({executioner=P.tier(P.context(40,0))});local cx=ex:crit_config(1);assert(cx.min_percent==100 and cx.slots.default.chance==1)
 local g=engine({gambler=P.tier(P.context(40,0))});local cg=g:crit_config(1);assert(cg.slots.default.multiplier_max>cg.slots.default.mean+.9)
 assert(not D.keystones.check({'executioner','gambler'}))
 -- crit as a trigger and a condition
 local flow=engine({critical_flow=1,keen=1});frame(flow);flow:emit{kind='crit',port=1,target=2,tags={critical=true},strength=.4};flow:drain();assert(flow:status(1,'haste'))
end)
T.test('passive state: air jumps, restrictions and permanent armour; Aerialist and Juggernaut',function()
 local tier=P.tier(P.context(40,0))
 local a=engine({aerialist=tier}):passive_state(1);assert(a.air_jumps==5 and a.forbid[1]=='shield')
 local j=engine({juggernaut=tier}):passive_state(1);assert(j.armor and j.armor.type=='damage_threshold' and j.armor.value==6 and j.forbid[1]=='run')
 T.refuses(function() engine({aerialist=tier,powershield_oath=tier}) end)
end)
T.test('technique modifiers sit in the depth curve: never before min_depth, present from the middle of a run',function()
 local loot=D.drive_loot.new(D.mod_pool);local early,mid=0,0
 for seed=1,400 do
  local r=loot:roll(seed,2,'magic');for _,a in ipairs(r.affixes) do if loot.rules[a.id].min_depth then early=early+1 end end
  local m=loot:roll(seed,12,'rare');for _,a in ipairs(m.affixes) do if loot.rules[a.id].min_depth then mid=mid+1 end end
 end
 assert(early==0,'no technique modifier before its depth');assert(mid>=40,'technique modifiers appear in a deep run: '..mid)
 for _,m in ipairs(D.mod_pool) do if m.min_depth then assert(m.min_depth>=4 and m.affix and m.notes,m.id) end end
 local r=loot:roll(5,12,'rare');local tech;for _,m in ipairs(D.mod_pool) do if m.min_depth then tech=m end end
 local forced=D.mod_codec.decode(D.mod_codec.encode(r));forced.depth=2;forced.affixes={{id=tech.id,tier=1}};forced.rarity='magic'
 T.refuses(function() loot:validate(forced) end)
end)
T.test('echo picture window: status-gated rules last as the status; others only a short window after a hit',function()
 local e=engine({trailing=1});frame(e);assert(e:echo_window(1)==0)
 e:emit{kind='status_applied',port=1,status='haste',tags={}};e.statuses[1]={haste={expires=e.frame+90,stacks=1,max=1,amount=1,next_tick=e.frame+60,origin={}}};assert(e:echo_window(1)==90)
 local e2=engine({echoes=1});frame(e2);assert(e2:echo_window(1)==0);e2:emit{kind='hit_dealt',port=1,target=2,tags={}};e2:drain();local w=e2:echo_window(1);assert(w>0 and w<=45,w)
 for _=1,60 do frame(e2) end;assert(e2:echo_window(1)==0,'never continuous')
end)
T.test('Shock status and Conductor chain to the nearest other opponent',function()
 local e=engine({conductor=P.tier(P.context(40,0))});frame(e)
 e:emit{kind='hit_dealt',port=1,target=2,tags={electric=true}};e:drain()
 assert(e:status(2,'shock') and e:status(4,'shock') and not e:status(3,'shock'),'port 4 is nearer to the victim than port 3')
 local plain=engine({conductor=P.tier(P.context(40,0))});frame(plain);plain:emit{kind='hit_dealt',port=1,target=2,tags={fire=true}};plain:drain();assert(not plain:status(2,'shock'))
 local cm=engine({critical_mass=P.tier(P.context(40,0))});frame(cm);cm:emit{kind='crit',port=1,target=2,tags={critical=true},strength=.5};cm:drain();assert(cm:status(1,'momentum') and cm:status(2,'shock'))
end)

-- ---- the host's journal operations -----------------------------------------------------------------------------------------
local function fixture()
 local s={commits=0,ops={},logs={},players={[1]={percent=30,stocks=4,falls=0,char=1,action=14,airborne=false,x=0,y=0},[2]={percent=20,stocks=4,falls=0,char=2,action=14,airborne=false,x=30,y=0}}}
 local g={pad=function()return {}end,input_mask=function()end,items=function()return {}end,paused=function()return true end,fixture=s,sim_supported=true,hit_rule_add=function() error('direct write') end,fighter_status=function() error('direct write') end,
  command=function() end,log=function(t) s.logs[#s.logs+1]=t end,match=function() return {active=true,netplay=false,stage=32} end,lab_mode=function() return true end,
  player=function(p) return s.players[p] end,sim_replaying=function() return false end,hit_rules=function() return {owner=0,percent_only=true,progression=true} end,
  echoes=function() return {journal=true} end,sim_read=function() return s.blob end,sim_clear=function() s.blob=nil end,
  skill_state=function() return {air_frames=12,aerial_hit=true,combo_count=0} end,
  fighter_interrupt=function(p,o) s.interrupt={p,o};return true end,
  sim_commit=function(blob,ops) s.commits=s.commits+1;s.blob=blob;s.ops=ops;return true end}
 s.g=g;s.a=D.mod_lab.new(g);return s,s.a
end
local function find(ops,kind,pred) for _,o in ipairs(ops) do if o.op==kind and (not pred or pred(o)) then return o end end end
T.test('host: skill event on_skill -> engine -> journal ops on the event frame; perfect_shield only through the legacy hook',function()
 local s,a=fixture();local tier=P.tier(P.context(40,0));a.engine.context=P.context(40,0)
 a.engine:set_build(1,{wavedasher=tier,juggernaut=tier},{});a.enabled=true
 assert(a:frame(),table.concat(s.logs,'|'));assert(find(s.ops,'fighter_caps',function(o) return o.values.run==true end),'permanent restriction written once')
 assert(find(s.ops,'fighter_armor',function(o) return o.type=='damage_threshold' and o.frames==0 end))
 assert(find(s.ops,'crit')==nil)
 a:skill{kind='wavedash',port=1,entity=1,frame=10,speed=2.6}
 assert(a:frame());local o=find(s.ops,'fighter_armor',function(o) return o.type=='super' end)
 assert(o and o.entity==1 and o.frames==6 and o.value==1,'super armour for exactly 6 frames on the event frame')
 assert(not find(s.ops,'fighter_caps'),'unchanged passive state is not rewritten')
 assert(a.engine:status(1,'guarded'))
 local ch=find(s.ops,'timed_status',function(o) return o.channel==1 end);assert(ch and ch.frames>0 and ch.value==D.mod_skill.cause.wave.index,'earned channel carries the cause')
 a:skill{kind='perfect_shield',port=1,entity=1};assert(#a.engine.queue==0,'perfect_shield skill copy is ignored')
end)
T.test('host: crit configuration written once, forced crit and interrupt, shock mirror, cleared on lost stock',function()
 local s,a=fixture();local tier=P.tier(P.context(40,0));a.engine.context=P.context(40,0)
 a.engine:set_build(1,{keen=2,wave_edge=2},{});a.enabled=true
 assert(a:frame());local begin=find(s.ops,'crit',function(o) return o.begin end);assert(begin and begin.entity==1)
 local slot=find(s.ops,'crit',function(o) return o.slot=='default' end);assert(slot and slot.chance>0.05)
 assert(a:frame());assert(not find(s.ops,'crit'),'an unchanged crit configuration is not rewritten each frame')
 a:skill{kind='wavedash',port=1,entity=1};assert(a:frame());local f=find(s.ops,'crit',function(o) return o.force end);assert(f and f.force==1,'wavedash: next hit crits')
 a:crit{attacker=1,victim=2,strength=.2,multiplier=1.5}
 -- shock mirror
 a.engine.statuses[2]={shock={expires=a.engine.frame+100,stacks=2,max=8,amount=1,next_tick=a.engine.frame+60,origin={}}}
 assert(a:frame());local sh=find(s.ops,'shock',function(o) return o.entity==2 end);assert(sh and sh.charges==2 and sh.hitstun==1.5)
 a:shock_end{port=2,reason='spent'};assert(not a.engine.statuses[2] or not a.engine.statuses[2].shock)
 assert(a:frame());assert(not find(s.ops,'shock'),'a natively spent Shock needs no clear')
 a.engine.statuses[2]={shock={expires=a.engine.frame+3,stacks=1,max=8,amount=1,next_tick=a.engine.frame+60,origin={}}};assert(a:frame());assert(find(s.ops,'shock',function(o) return o.entity==2 and not o.clear end))
 for _=1,5 do assert(a:frame()) end;assert(a.engine.statuses[2]==nil or not a.engine.statuses[2].shock)
 assert(a.tech.shock[2]==nil,'the mirror record is retired with the status')
 -- every op the host wrote passes the registry's native-parser mirror (the host drops, never sends, an invalid one)
 for _,l in ipairs(s.logs) do assert(not l:find('technique op refused',1,true),l) end
 -- checkpoint carries the written record
 local blob=D.mod_codec.decode(a:export());assert(blob.tech and blob.tech.crit[1])
end)
T.test('host: op budget (24) is respected and the journal blob decodes',function()
 local s,a=fixture();a.engine.context=P.context(40,0);a.engine:set_build(1,{keen=1},{});a.enabled=true;assert(a:frame());assert(#s.ops<=24)
end)
T.test('earned_fx: crit presentation scales with strength, is light at the bottom and one pass at a time',function()
 local F=D.earned_fx
 local lo,hi=F.sequence_strength(0),F.sequence_strength(1);assert(lo>0 and lo<=.15 and hi==1)
 local prev=-1;for i=0,20 do local s=F.sequence_strength(i/20);assert(s>=prev);prev=s end
 local l1,l2=F.look(lo),F.look(hi);assert(l1.dur<l2.dur and l1.tear_amt<l2.tear_amt and l1.tear_inv==0 and l1.lines==0 and l2.contrast>l1.contrast)
 local passes,removed,tracers={},0,{}
 local g={match=function() return {active=true,netplay=false} end,log=function() end,time=function() return 0 end,project=function() return 100,100,true end,safe_area=function() return {w=1000,h=500} end,
  shader_load=function() return 7 end,post_add=function(_,o) passes[#passes+1]=o;return #passes end,post_remove=function() removed=removed+1 end,
  tracer_add=function(o) tracers[#tracers+1]=o;return 55 end,tracer_window=function() return true end,tracer_set=function() return true end}
 local host;host={g=g,engine=nil,enabled=true,toast=function(t) host.toasted=t end}
 local fx=F.new(host);fx:crit{attacker=1,victim=2,strength=.2,multiplier=1.4,x=1,y=2,z=0};assert(#passes==1 and fx.pass==1 and not host.toasted)
 fx:crit{attacker=1,victim=2,strength=.9,multiplier=2.5,x=1,y=2,z=0};assert(#passes==2 and removed==1,'one pass at a time: the live one is removed first')
 assert(host.toasted and #tracers==1 and tracers[1].anchor=='active_hitboxes' and tracers[1].trigger=='flag','a strong crit toasts; the tracer is a window on the hit, not always on')
end)
T.done()
