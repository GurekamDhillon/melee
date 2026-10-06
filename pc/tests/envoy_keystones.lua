-- The keystone pool: about thirty, every drive colour, each with a drawback, each running on today's engine.
local T=dofile('melee/pc/tests/envoy_testlib.lua');local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_budget','keystones','mod_pool','mod_engine','drive_loot','drive_bag','drive_text'}) do D[n]=T.module(n,D) end
local K,P=D.keystones,D.mod_progression;local loot=D.drive_loot.new(D.mod_pool)
local function pool_keys() local out={};for _,m in ipairs(D.mod_pool) do if m.kind=='keystone' then out[#out+1]=m end end;return out end
T.test('thirty keystones, six drive colours, unique ids, every one in the validated pool with a drawback',function()
 local keys=pool_keys();assert(#keys==30,#keys);local ids,fam={},{}
 for _,m in ipairs(keys) do assert(not ids[m.id]);ids[m.id]=true;assert(type(m.cost)=='string' and #m.cost>0,m.id);local f=K.family(m.id);assert(f,m.id);fam[f]=(fam[f] or 0)+1;assert(#m.families>=1) end
 for _,f in ipairs(K.families) do assert((fam[f] or 0)>=3,f..' has too few keystones') end
 assert(#K.ids()==30)
end)
T.test('every keystone reads as an effect line then a drawback line, in plain words',function()
 for _,m in ipairs(pool_keys()) do for _,tier in ipairs({1,3,6}) do
  local lines=D.drive_text.keystone_lines(m,tier);assert(#lines==2,m.id)
  assert(lines[2]:find('^Drawback: '),m.id..': '..lines[2])
  for _,l in ipairs(lines) do assert(#l>12 and not l:find('[{}$]') and not l:find('[Tt]ier') and not l:find(m.id,1,true) and not l:find('nan') and not l:find('inf'),m.id..': '..l) end
 end end
 local cell=D.drive_text.keystone_cell(D.mod_pool[1]);assert(cell.kind=='keystone')
end)
T.test('each keystone does something in the engine: equip rules change values or hit rules, triggers change statuses or damage',function()
 local ctx=P.context(60,0);local tier=P.tier(ctx)
 for _,m in ipairs(K.records()) do
  local e=D.mod_engine.new(1,D.mod_pool,{context=ctx});e:set_build(1,{[m.id]=tier},{});e:set_build(2,{},{})
  local players={[1]={percent=120,grounded=true,stocks=1},[2]={percent=150,grounded=false,stocks=2}}
  if m.trigger=='equip' then
   local rules=e:native_rules(1);local vals=e:values(1);local changed=#rules>0 or e:crit_config(1)~=nil;local ps=e:passive_state(1);if ps.armor or ps.air_jumps or #ps.forbid>0 then changed=true end;for _,v in pairs(vals) do if v~=1 then changed=true end end
   assert(changed,m.id..' has no equip effect')
  else
   if m.trigger=='interval' then e.frame=m.interval-1 end
   e:begin_frame(players)
   e.statuses[2]={chill={expires=e.frame+500,stacks=1,max=1,amount=1,next_tick=e.frame+60,origin={}}}
   e:emit{kind=m.trigger,port=1,target=2,tags={electric=true},depth=1,count=5,damage=40,hit=true}
   e:drain()
   local did=#e.fx>0 or next(e.statuses[1] or {})~=nil or (e.damage[1] or 0)~=0 or (e.statuses[2] and next(e.statuses[2],'chill'))~=nil
   for _,v in pairs(e.statuses[2] or {}) do did=did or v.amount~=1 end
   assert(did,m.id..' ('..m.trigger..') did nothing')
  end
 end
end)
T.test('exclusive keystones and stacked drawbacks are refused everywhere the budget is read',function()
 assert(K.check{'smash_doctrine','aerial_doctrine'}==nil and K.check{'pyromancer','frozen_oath'}==nil and K.check{'executioner','gambler'}==nil)
 assert(K.check{'smash_doctrine','sprinter','bloodlust'});assert(K.check({'bulwark','bulwark'})==nil)
 -- drawbacks add and floor: run speed -15% (Smasher) -25% (Bulwark) -20% (Dive Bomber) passes the -45% floor only two at a time
 assert(K.check{'smash_doctrine','bulwark'}==true and K.check{'smash_doctrine','bulwark','dive_bomber'}==nil)
 local ctx=P.context(60,0);local bag=D.drive_bag.new(loot,{context=ctx})
 assert(bag:choose_keystone('pyromancer'));assert(not bag:choose_keystone('frozen_oath'),'the bag must refuse an exclusive pair');assert(#bag.keystones==1)
 local e=D.mod_engine.new(1,D.mod_pool,{context=ctx});T.refuses(function() e:set_build(1,{smash_doctrine=1,aerial_doctrine=1},{}) end)
 assert(not select(1,K.check{'nonsense'}))
end)
T.test('the starting keystone is seeded, playable from stage one and varied',function()
 local seen,n={},0
 for seed=1,400 do local a=K.starting(seed);assert(a==K.starting(seed));assert(K.meta[a].starter~=false);if not seen[a] then seen[a]=true;n=n+1 end end
 assert(n>=18,'only '..n..' different starting keystones')
 T.refuses(function() K.starting(-1) end)
 -- it equips without a choice at depth 0 and the build is legal
 for seed=1,60 do local id=K.starting(seed);local bag=D.drive_bag.new(loot,{context=P.context(0,0)});assert(bag:choose_keystone(id),id);bag:derive() end
end)
T.test('offers: distinct, legal with what is held, from different colours, same offer on a retry, none when nothing is owed',function()
 local ctx=P.context(10,0) -- allowance 3
 assert(K.owed(ctx,{'bulwark'})==2 and K.owed(ctx,{'a','b','c'})==0)
 for seed=1,80 do local held={'smash_doctrine'};local o=K.offer(ctx,seed,3,held);assert(#o==3);local f,u={},{}
  for _,id in ipairs(o) do assert(not u[id] and id~='smash_doctrine');u[id]=true;f[K.family(id)]=true;local trial={'smash_doctrine',id};assert(K.check(trial)) end
  local nf=0;for _ in pairs(f) do nf=nf+1 end;assert(nf==3);assert(table.concat(o,',')==table.concat(K.offer(ctx,seed,3,held),','))
  assert(not u.aerial_doctrine,'an exclusive keystone must not be offered')
 end
 assert(#K.offer(P.context(0,0),1,3,{'bulwark'})==0,'allowance used up: nothing offered')
 -- a deep run can keep taking offers until the legal pool is exhausted, never past the allowance
 local held,ctx2={},P.context(0,20);while K.owed(ctx2,held)>0 do local o=K.offer(ctx2,#held+1,3,held);if #o==0 then break end;held[#held+1]=o[1] end
 assert(#held>=8 and K.check(held))
end)
T.test('waiting keystones are data only and never in the pool; no keystone keeps afterimages on',function()
 assert(#K.waiting>=1);local ids={};for _,m in ipairs(D.mod_pool) do ids[m.id]=true end
 for _,w in ipairs(K.waiting) do assert(not ids[w.id] and w.needs and w.effect and w.drawback and K.family_names[w.family],w.id) end
 for _,m in ipairs(K.records()) do for _,e in ipairs(m.effects) do if e.op=='echo' then assert(e.status,m.id..': an echo needs a status that starts and ends it') end end end
end)
T.done()
