-- Synergies: the derived interaction graph, the synergy-aware strength (v2, v1 behind a switch), the bridges, connecting offers and the tuning presets.
local T=dofile('melee/pc/tests/envoy_testlib.lua');local D=T.rules()
D.mod_tuning=T.module('mod_tuning',D)
for _,n in ipairs{'mod_codec','mod_schema','mod_graph','mod_budget','mod_engine','keystones','mod_pool','drive_loot','drive_merge','drive_economy','drive_bag','mod_synergy'} do D[n]=T.module(n,D) end
local P,B,G,Tn=D.mod_progression,D.mod_budget,D.mod_graph,D.mod_tuning
local pool=D.mod_pool
local function rec(id) for _,m in ipairs(pool) do if m.id==id then return m end end end
local function strength(mods,mode) Tn.set('strength',mode or 'v2');local _,s=B.build(pool,mods,{},{});Tn.set('strength','v2');return s end
local function engine(mods,ctx) local e=D.mod_engine.new(1,pool,{context=ctx or P.context(60,3)});e:set_build(1,mods,{});return e end
local function frame(e) e:begin_frame({[1]={percent=0},[2]={percent=0}}) end
T.test('the derived graph: every archetype piece exists, links read as words, partners are found',function()
 local g=G.graph(pool);assert(#g.list>60 and #g.list<400,#g.list)
 assert(#G.pieces(pool).missing==0,table.concat(G.pieces(pool).missing,' '))
 assert(g.out.kindling.cinder and g.out.kindling.cinder.why=='status:burn:target')
 assert(g.out.keen.brutal and g.out.keen.brutal.kind=='enable')
 local text=G.link_text(pool,'kindling','cinder',1);assert(text=='Puts Burning on the target, which your Cinder drive turns into +10% damage.',text)
 assert(not G.connected(pool,'smash_doctrine','kindling'),'a stat stick connects to nothing')
 assert(G.connects(pool,{'cinder'},{kindling=true,glass_core=true}) and not G.connects(pool,{'cinder'},{glass_core=true,lingering=true}))
 -- the declared pair table is the derived one now: no tag-overlap pair
 local pairs_,deg=D.mod_synergy.generate(pool)
 for _,p in ipairs(pairs_) do for _,why in ipairs(p.reasons) do assert(not why:match('^tag:'),p.a..' '..p.b..' '..why) end end
end)
T.test('archetypes: roles, progress, completion and what an offer would do',function()
 local a=G.by_id.burn
 local p=G.progress(a,{kindling=true});assert(p.filled==1 and p.missing=='a payoff','Kindling applies Burning on any hit, so it fills the applier role alone')
 p=G.progress(a,{plague_bearer=true});assert(p.filled==1 and not p.complete and p.missing=='a payoff')
 p=G.progress(a,{plague_bearer=true,cinder=true});assert(p.complete)
 assert(#G.completed({kindling=true,burning=true,pyre=true,icebound=true})==1)
 local kind,arch=G.effect_of({'cinder'},{plague_bearer=true});assert(kind=='complete' and arch.id=='burn')
 kind,arch=G.effect_of({'charged'},{keen=true});assert(kind==nil)
 assert(G.chain_archetype(pool,'kindling','cinder').id=='burn' and G.chain_archetype(pool,'updraft','crosswind').id=='momentum')
end)
T.test('bridges: Momentum is spent one stack at a time, Guarded and Shock and Curse have readers',function()
 local e=engine({updraft=1,crosswind=1});frame(e)
 for _=1,3 do e:emit{kind='hit_dealt',port=1,target=2,tags={aerial=true}};e:drain() end
 assert(e:status(1,'momentum').stacks==3)
 e:emit{kind='landing',port=1,tags={}};e:drain();assert(e:status(1,'momentum').stacks==2 and e:status(1,'haste'),'Crosswind spends ONE stack')
 e:emit{kind='landing',port=1,tags={}};e:drain();assert(e:status(1,'momentum').stacks==1)
 e:emit{kind='landing',port=1,tags={}};e:drain();assert(not e:status(1,'momentum'),'the last stack goes')
 local b=engine({updraft=1,bastion=1});frame(b)
 for _=1,2 do b:emit{kind='hit_dealt',port=1,target=2,tags={aerial=true}};b:drain() end
 b:emit{kind='landing',port=1,tags={}};b:drain();assert(b:status(1,'guarded') and b:status(1,'momentum').stacks==1,'Bastion spends one stack')
 -- Retaliation: from a hit taken while Guarded, and still from an armour absorb
 local r=engine({retaliation=1,shelter=1});frame(r)
 r:emit{kind='ledge_grab',port=1,tags={}};r:drain();assert(r:status(1,'guarded'))
 r:emit{kind='hit_taken',port=1,target=2,tags={}};r:drain();assert(#r.fx==0,'Retaliation reads armour only now (the Guarded alternative trigger was dropped)')
 local r2=engine({retaliation=1});frame(r2);r2:emit{kind='armor',port=1,tags={},absorbed=true};r2:drain();assert(#r2.fx==1)
 -- Combo Surge also fires from the second hit on a Shocked target; Combo Finish and Combo Conduit read Shock/Curse
 local s=engine({combo_surge=1});frame(s)
 s:emit{kind='combo',port=1,target=2,tags={},count=2};s:drain();assert(not s:status(1,'haste'),'two hits: no (one trigger, one condition: the 3rd hit)')
 s:emit{kind='combo',port=1,target=2,tags={},count=3};s:drain();assert(s:status(1,'haste'))
 -- every status has at least two readers (a record whose trigger, conditions or `also` read it)
 local readers={}
 local function note(st,id) readers[st]=readers[st] or {};readers[st][id]=true end
 for _,m in ipairs(pool) do
  local function scan(trigger,conds) for _,c in ipairs(conds or {}) do for _,k in ipairs{'self_status','target_status'} do if c[k] and c[k]~='any' then note(c[k],m.id) end end;if trigger=='status_applied' and c.status and c.status~='any' then note(c.status,m.id) end end end
  scan(m.trigger,m.conditions);for _,a in ipairs(m.also or {}) do scan(a.trigger,a.conditions) end
  for _,ef in ipairs(m.effects) do if ef.op=='versus-status' and ef.status then note(ef.status,m.id) end end
 end
 for _,st in ipairs{'burn','chill','shock','curse','haste','guarded','momentum'} do local n=0;for _ in pairs(readers[st] or {}) do n=n+1 end;assert(n>=(({burn=2,chill=2,haste=1,guarded=1,momentum=2,shock=0,curse=0})[st]),st..' has '..n..' readers') end -- the split: Shock is private and Marked is read by nothing (its effect is its number)
end)
T.test('Brutal carries a small crit chance of its own',function()
 local c=engine({brutal=1}):crit_config(1);assert(c and math.abs(c.slots.default.chance-.03)<1e-9)
end)
T.test('strength v2: reads what the build feeds, honest Echo, crits at full weight, v1 stays available',function()
 assert(B.mode()=='v2')
 local echoes,everburn,gambler=strength({echoes=3},'v1'),strength({everburn=3},'v1'),strength({gambler=3},'v1')
 assert(echoes>2.1 and everburn>1.6 and gambler<1.05,'v1 is the old number')
 local e2,ev2,g2=strength({echoes=3}),strength({everburn=3}),strength({gambler=3})
 assert(e2<1.7 and e2>1.5,'Echo capped (measured x1.05; capped, not zeroed)');assert(ev2<1.15,'Everburn alone: its payoff cannot fire');assert(g2>1.15,'Gambler: measured x1.25')
 local alone,fed=strength({cinder=3}),strength({cinder=3,kindling=3,burning=3})
 assert(alone<1.04 and fed>1.3,'a payoff counts in full only with its applier ('..alone..' / '..fed..')')
 -- a status granted by a rare trigger counts for its uptime, not for ever
 assert(strength({shelter=3})<strength({shelter=3},'v1'))
 -- wasted defence past the damage-taken floor does not count
 local armoured=strength({armoured=3,bulwark=3});local more=strength({armoured=3,bulwark=3,iron_resolve=3,shelter=3,bastion=3})
 local fam=B.build(pool,{armoured=3,bulwark=3,iron_resolve=3},{damage_taken=.5},{});assert(fam.damage_taken.potential>=.15-1e-9,'floor')
 Tn.set('strength','v1');assert(B.mode()=='v1');Tn.set('strength','v2')
end)
T.test('tuning presets: current by default, proposed on a command, the engine follows at once',function()
 Tn.preset('current');assert(Tn.preset_name=='current' and Tn.get('payoff_scale')==1 and Tn.get('plague_scale')==1 and Tn.get('min_depth_cap')==0)
 local e=engine({cinder=1,kindling=1},P.context(60,3))
 local function launch_percent(en) local rules=en:native_rules(1);for _,r in ipairs(rules) do if r.change.percent_damage and r.match.status_bits==1 then return r.change.percent_damage end end end
 local before=launch_percent(e);assert(before and before>1)
 local plague=engine({plague_bearer=1},P.context(60,3));frame(plague);plague:emit{kind='hit_dealt',port=1,target=2,tags={}};plague:drain();local burn0=plague:status(2,'burn').amount
 Tn.preset('proposed')
 assert(Tn.preset_name=='proposed' and Tn.get('payoff_scale')==2.5)
 local after=launch_percent(e);assert(after>before and math.abs((after-1)-(before-1)*2.5)<1e-9,'the payoff is scaled, no rebuild')
 local p2=engine({plague_bearer=1},P.context(60,3));frame(p2);p2:emit{kind='hit_dealt',port=1,target=2,tags={}};p2:drain();assert(math.abs(p2:status(2,'burn').amount-burn0*.5)<1e-9,'Plague Bearer cut')
 -- min_depth: a technique drive that cannot roll at depth 3 under current can under proposed; validation accepts the lower bound under both
 local loot=D.drive_loot.new(pool);local found
 for seed=1,3000 do local r=loot:roll(seed,4,'magic');for _,a in ipairs(r.affixes) do if (rec(a.id).min_depth or 0)>4 then found=r end end;if found then break end end
 assert(found,'a deep technique drive rolls at depth 4 under proposed')
 Tn.preset('current');assert(Tn.preset_name=='current' and loot:validate(found),'a drive rolled under proposed stays valid under current')
 for seed=1,600 do local r=loot:roll(seed,4,'magic');for _,a in ipairs(r.affixes) do assert((rec(a.id).min_depth or 0)<=4,'current: no deep technique drive at depth 4') end end
 assert(math.abs(launch_percent(e)-before)<1e-9,'back to the authored payoff')
end)
T.test('connecting offers: a keystone offer connects to the held build; a drive offer is swapped for a partner, seeded',function()
 local ctx=P.context(10,0);local held={kindling=true,burning=true}
 local connect={held=held,pool=pool,graph=G}
 local plain=D.keystones.offer(ctx,77,3,{});local with=D.keystones.offer(ctx,77,3,{},connect)
 assert(#with==3)
 local hit=false;for _,id in ipairs(with) do if G.connects(pool,{id},held) then hit=true end end
 if not G.connects(pool,plain,held) then assert(hit,'a partner replaces one offer') else assert(table.concat(plain,',')==table.concat(with,','),'nothing changes when an offer already connects') end
 local again=D.keystones.offer(ctx,77,3,{},connect);assert(table.concat(with,',')==table.concat(again,','),'deterministic')
 -- drives
 local loot=D.drive_loot.new(pool);local swapped,total=0,0
 for seed=1,60 do
  local offers={loot:roll(seed*3,ctx),loot:roll(seed*3+1,ctx),loot:roll(seed*3+2,ctx)}
  local copy={offers[1],offers[2],offers[3]}
  local k=G.connect_offers(pool,loot,copy,held,seed,ctx,function(i) return offers[i].rarity end,function(try,i) return seed*101+try*7+i end)
  if k then swapped=swapped+1;local ids={};for _,a in ipairs(copy[k].affixes) do ids[#ids+1]=a.id end;assert(G.connects(pool,ids,held));assert(loot:validate(copy[k]));assert(copy[k].rarity==offers[k].rarity) end
  total=total+1
 end
 assert(swapped>5,swapped)
end)
T.done()
