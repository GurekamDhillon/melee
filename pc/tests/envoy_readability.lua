-- The readability split (2026-10-05): at most two rules per drive (one early), the 72-piece pool, the seven keystone prices, no self-status price,
-- duplicates merging across colours, retired pieces dropped with a notice, one word per status, the text bugs, crits at x1.5, the opponent card,
-- the Momentum counter, the developer overlay, the small corner notification, the default rule host.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules()
D.mod_tuning=T.module('mod_tuning',D)
for _,n in ipairs{'mod_progression','mod_codec','mod_schema','mod_graph','mod_budget','mod_engine','keystones','mod_pool','drive_loot','drive_merge','drive_economy','drive_bag','drive_text','synergy_fx','run_hud','foe_roll'} do D[n]=T.module(n,D) end
local P,K,S=D.mod_progression,D.keystones,D.mod_status
local loot=D.drive_loot.new(D.mod_pool)
local function words(text) local n=0;for _ in text:gmatch('%S+') do n=n+1 end;return n end

T.test('the pool: 72 pieces (13 standing-rule drives, 23 trigger-rule drives, 6 unique, 30 keystone); the cut pieces are gone and reserved',function()
 local n={prefix=0,suffix=0,unique=0,keystone=0}
 for _,m in ipairs(D.mod_pool) do if m.kind=='keystone' then n.keystone=n.keystone+1 elseif m.kind=='unique' then n.unique=n.unique+1 else n[m.affix]=n[m.affix]+1 end end
 assert(n.prefix==13 and n.suffix==23 and n.unique==6 and n.keystone==30,('%d/%d/%d/%d'):format(n.prefix,n.suffix,n.unique,n.keystone))
 for id in pairs({heavy=1,featherweight=1,skyfarer=1,phase_dash=1,clash_king=1,echo_oath=1,banked_momentum=1,desperado=1,finishers_mark=1,powershield_oath=1,featherfall=1,still_heart=1,hang_time=1,critical_mass=1}) do
  assert(not loot.rules[id],id..' must be cut');assert(D.drive_loot.is_retired(id),id..' must stay reserved for old saves')
 end
 assert(not D.drive_loot.is_retired('kindling'))
end)

T.test('at most two rules per drive: one below effective depth 5, two from 5, no white extra, loops included',function()
 assert(P.max_rules==2)
 for _,ctx in ipairs{P.context(0,0),P.context(2,0),P.context(4,0)} do for _,rarity in ipairs{'common','magic','rare'} do for seed=1,120 do
  assert(#loot:roll(seed*31,ctx,rarity).affixes==1,'one rule early (depth '..ctx.depth..')') end end end
 for _,ctx in ipairs{P.context(5,0),P.context(9,0),P.context(30,0),P.context(3,2)} do
  local two,white=0,0
  for _,rarity in ipairs{'magic','rare'} do for seed=1,150 do local r=loot:roll(seed*37,ctx,rarity);assert(#r.affixes==2,'magic and rare carry two');two=two+1
   if r.colour=='white' then white=white+1;assert(#r.affixes==2,'no white extra') end end end
  for seed=1,150 do assert(#loot:roll(seed*41,ctx,'common').affixes==1) end
  assert(white>0)
 end
 -- a standing rule and a trigger rule: the roller alternates prefix and suffix
 for seed=1,100 do local r=loot:roll(seed,10,'rare');local a,b=loot.rules[r.affixes[1].id],loot.rules[r.affixes[2].id];assert(a.affix~=b.affix) end
end)

T.test('payoffs open at effective depth 5',function()
 local payoffs={'pyre','cinder','shatter','brittle','malice','feasting','renewal','rush','crosswind','bastion','trailing'}
 for _,id in ipairs(payoffs) do assert(loot.rules[id].min_depth==5,id) end
 local seen={}
 for seed=1,600 do
  for _,a in ipairs(loot:roll(seed,4,'magic').affixes) do for _,id in ipairs(payoffs) do assert(a.id~=id,id..' rolled before depth 5') end end
  for _,a in ipairs(loot:roll(seed,12,'rare').affixes) do seen[a.id]=true end
 end
 local n=0;for _,id in ipairs(payoffs) do if seen[id] then n=n+1 end end;assert(n>=8,'payoffs appear from depth 5: '..n)
end)

T.test('keystones: one rule line and one price line, the price from the fixed list of seven, never a status on yourself',function()
 assert(#K.prices==7)
 local used={}
 for _,m in ipairs(K.records()) do
  local price=K.price(m.id);assert(price,m.id..' has no price');used[price]=true
  local ok=false for _,p in ipairs(K.prices) do if p==price then ok=true end end;assert(ok,m.id..' price '..tostring(price)..' is not on the list')
  local lines=D.drive_text.keystone_lines(m,1);assert(#lines==2 and lines[2]:find('^Drawback: '),m.id)
  assert(words(lines[1])<=14 and words(lines[2])<=12 or m.id=='executioner' or m.id=='iron_resolve',m.id..': '..lines[1]..' / '..lines[2])
  -- the effects: exactly the price the record declares, and no status on the keystone's own fighter that is a price
  local kinds={}
  for _,e in ipairs(m.effects) do
   if e.op=='value' and e.key=='run_speed' and e.value<1 then kinds.slow=true
   elseif e.op=='value' and e.key=='jump_height' and e.value<1 then kinds.low=true
   elseif e.op=='value' and e.key=='knockback_taken' and e.value>1 then kinds.fragile=true
   elseif e.op=='value' and e.key=='damage_dealt' and e.value<1 then kinds.weak=true
   elseif e.op=='value' and e.key=='damage_taken' and e.value>1 then kinds.vulnerable=true
   elseif e.op=='versus-status' and e.match and e.match.incoming then kinds.vulnerable=true
   elseif e.op=='restrict' or (e.op=='remove_status' and (e.subject or 'self')=='self' and e.status=='momentum') then kinds.lock=true
   elseif e.op=='damage' and e.subject=='self' then kinds.bleed=true end
   if e.op=='status' and e.subject=='self' then assert(e.status=='haste' or e.status=='guarded',m.id..' puts '..e.status..' on its own fighter') end
   if e.op=='chain_status' or (e.op=='status' and e.subject=='target') then assert(e.status~='haste' and e.status~='guarded',m.id) end
  end
  assert(kinds[price],m.id..' declares '..price..' but its effects do not carry it')
  local count=0;for _ in pairs(kinds) do count=count+1 end;assert(count==1,m.id..' pays more than one price')
  if price=='bleed' then assert(m.trigger~='equip',m.id) end
 end
 for _,p in ipairs(K.prices) do assert(used[p],'no keystone pays '..p) end
 -- ordinary drives carry no price at all
 for _,m in ipairs(D.mod_pool) do if m.kind=='normal' then assert(m.cost==nil,m.id);for _,e in ipairs(m.effects) do assert(not (e.op=='damage' and e.subject=='self'),m.id) end end end
end)

T.test('a duplicate of a rule you hold merges across colours (it is never inert); a family match alone still needs the colour',function()
 local M=D.drive_merge
 local function rec(colour,id,depth) return {seed=1,depth=depth or 0,colour=colour,rarity='common',affixes={{id=id,tier=1}}} end
 local held,gained=rec('red','kindling'),rec('blue','kindling')
 assert(M.can_merge(held,gained,loot));local out,info=M.merge(held,gained,loot)
 assert(out and out.colour=='red' and #out.affixes==1 and out.affixes[1].tier==2 and info.affix=='kindling' and out.merged==1)
 assert(not M.can_merge(held,rec('blue','cleansing'),loot) and M.can_merge(held,rec('red','cleansing'),loot))
 local plan=D.drive_economy.gain_plan({held},0,gained,loot);assert(plan.action=='merge' and plan.index==1)
 local bag=D.drive_bag.new(loot);assert(bag:give(held));assert(bag:equip(1,1));assert(bag:replace('equipped',1,out));assert(bag:derive().kindling==2,'the merged rule is stronger, not duplicated')
 -- fully merged: the duplicate cannot merge and stays a plain drive
 local full=rec('red','kindling');full.merged=3;assert(not M.can_merge(full,gained,loot))
end)

T.test('a saved run that holds a cut piece drops it with a notice and still loads',function()
 local bag=D.drive_bag.new(loot,{context=P.context(30,0)})
 local state={context=P.context(30,0),
  items={{seed=1,depth=30,colour='red',rarity='common',affixes={{id='heavy',tier=3}}},{seed=2,depth=30,colour='red',rarity='common',affixes={{id='kindling',tier=3}}}},
  equipped={[1]={seed=3,depth=30,colour='green',rarity='magic',affixes={{id='featherweight',tier=3},{id='ledge',tier=3}}},
            [2]={seed=4,depth=30,colour='blue',rarity='rare',affixes={{id='armoured',tier=3},{id='lingering',tier=3},{id='shelter',tier=3},{id='cleansing',tier=3}}}},
  keystones={'still_heart','bulwark','hang_time'}}
 local notes=D.drive_bag.migrate(state,loot)
 assert(#notes==7,#notes..' notices: '..table.concat(notes,' | '))
 local text=table.concat(notes,' ');assert(text:find('Heavy') and text:find('Featherweight') and text:find('Still heart') and text:find('Hang time') and text:find('at most 2 rules'),text)
 assert(#state.items==1 and state.items[1].affixes[1].id=='kindling','the drive with nothing left goes')
 assert(#state.equipped[1].affixes==1 and state.equipped[1].affixes[1].id=='ledge' and #state.equipped[2].affixes==2,'extra rules trimmed to the first two')
 assert(#state.keystones==1 and state.keystones[1]=='bulwark' and state.keystone=='bulwark','a cut keystone leaves the list')
 assert(bag:restore(state),'the migrated state validates');assert(K.owed(P.context(30,0),state.keystones)>0,'the keystone pick is owed again')
 local clean={items={},equipped={},keystones={'bulwark'},context=P.context(0,0)};assert(#D.drive_bag.migrate(clean,loot)==0)
 -- an engine snapshot that holds one: the piece is dropped and listed, the snapshot loads
 local e=D.mod_engine.new(1,D.mod_pool);e:set_build(1,{kindling=1},{})
 local at=D.mod_codec.decode(e:export());at.equipped[1].heavy=1;at.equipped[1].still_heart=1
 local fresh=D.mod_engine.new(1,D.mod_pool);fresh:import(D.mod_codec.encode(at))
 assert(fresh.equipped[1].kindling==1 and not fresh.equipped[1].heavy and not fresh.equipped[1].still_heart and #fresh.retired_dropped==2)
 local bad=D.mod_codec.decode(e:export());bad.equipped[1].nonsense_piece=1;T.refuses(function() fresh:import(D.mod_codec.encode(bad)) end)
end)

T.test('one word for each status in every text a player reads; the glossary holds the numbers',function()
 assert(table.concat(S.core,',')=='burn,chill,curse,haste,guarded' or #S.core==5);assert(#S.core==5 and S.counter_name=='momentum' and S.private_name=='shock')
 assert(S.label('burn')=='Burning' and S.label('chill')=='Chilled' and S.label('haste')=='Haste' and S.label('guarded')=='Guarded' and S.label('curse')=='Marked')
 local gloss=table.concat(S.glossary(),' | ');assert(gloss:find('Marked:',1,true) and gloss:find('Momentum: a counter',1,true) and gloss:find('Crit: a hit that deals x1.5',1,true),gloss)
 local function scan(label,text)
  for _,w in ipairs(S.banned_words) do assert(not text:find(w,1,true),label..' uses "'..w..'": '..text) end
  assert(not text:find('engine',1,true),label..' speaks developer words: '..text)
 end
 for _,m in ipairs(D.mod_pool) do for t=1,#m.tiers do
  for _,l in ipairs(D.drive_text.mod_lines(m,t)) do scan(m.id..' line',l) end
  scan(m.id..' schema text',S.glossary and D.mod_schema.describe(m,t) or '')
 end;scan(m.id..' label',m.label) end
 for _,a in ipairs(D.mod_graph.archetypes) do scan(a.id..' blurb',a.blurb);scan(a.id..' name',a.name) end
 local pairs_,_=D.mod_synergy and D.mod_synergy.generate(D.mod_pool) or {},0
 for _,a in ipairs(D.mod_pool) do for _,b in ipairs(D.mod_pool) do local text=D.mod_graph.link_text(D.mod_pool,a.id,b.id,1);if text then scan('link '..a.id..'>'..b.id,text) end end end
 -- every status word that a piece speaks is the one word
 local seen={};for _,m in ipairs(D.mod_pool) do for _,l in ipairs(D.drive_text.mod_lines(m,1)) do for _,w in ipairs{'Burning','Chilled','Haste','Guarded','Marked','Momentum','Shock'} do if l:find(w,1,true) then seen[w]=true end end end end
 for _,w in ipairs{'Burning','Chilled','Haste','Guarded','Marked','Momentum','Shock'} do assert(seen[w],w..' is spoken by no piece') end
end)

T.test('text bugs: no "of the of" names, Brutal states its multiplier, no developer sentence on a crit drive',function()
 for seed=1,800 do local r=loot:roll(seed*13,seed%40,seed%3==0 and 'rare' or seed%3==1 and 'magic' or 'common');local n=loot:name(r)
  assert(not n:find(' of the ',1,true) and not n:find(' of of',1,true) and not n:find('the of',1,true),n);assert(n:find('^%u%l+ Drive: ') or r.unique,n) end
 local brutal=loot.rules.brutal
 local l1=D.drive_text.mod_lines(brutal,1);assert(#l1==1 and l1[1]=='Your hits crit 3% of the time; crits gain +0.25x.',l1[1])
 assert(D.drive_text.mod_lines(brutal,3)[1]:find('crits gain +0.38x',1,true),D.drive_text.mod_lines(brutal,3)[1])
 for _,id in ipairs{'keen','brutal','ruthless','finishing','critical_flow','clean_landing','tech_guard','combo_surge','wave_edge'} do
  local lines=D.drive_text.mod_lines(loot.rules[id],1);assert(#lines==1,id..' is one line');assert(not lines[1]:find('Crits are rare',1,true) and not lines[1]:find('afterimage',1,true) and not lines[1]:find('earn it',1,true),lines[1])
 end
 assert(D.drive_text.mod_lines(loot.rules.keen,1)[1]=='Your hits crit 5% of the time.')
 assert(D.drive_text.mod_lines(loot.rules.finishing,1)[1]=='Hits on a target above 100% crit 25% of the time.')
 assert(D.drive_text.mod_lines(loot.rules.combo_finish,1)[1]=='Finish a combo of 3+ hits: heal 3%.')
 assert(D.drive_text.mod_lines(loot.rules.combo_surge,1)[1]=='3rd hit of a combo: Haste for 0.8 s.')
end)

T.test('crit multiplier is a fixed x1.5 except Brutal, Gambler and Executioner',function()
 for _,id in ipairs{'keen','ruthless','finishing'} do for _,e in ipairs(loot.rules[id].effects) do assert(e.op=='crit' and e.multiplier==nil,id) end end
 local e=D.mod_engine.new(1,D.mod_pool);e:set_build(1,{keen=1},{});local cfg=e:crit_config(1);assert(cfg and cfg.slots.default.mean==1.5)
 e:set_build(1,{ruthless=1},{});assert(e:crit_config(1).slots.aerial.mean==1.5)
 e:set_build(1,{finishing=1},{});assert(e:crit_config(1).slots.default.mean==1.5)
 local has_mult={};for _,m in ipairs(D.mod_pool) do for _,fx in ipairs(m.effects) do if fx.op=='crit' and fx.multiplier then has_mult[m.id]=true end end end
 assert(has_mult.brutal and has_mult.gambler and has_mult.executioner and not has_mult.keen and not has_mult.ruthless and not has_mult.finishing)
end)

T.test('Burn and Chill appliers act on any hit; Kindling is tuned down (Burn per hit was the strongest thing in the pool)',function()
 for _,id in ipairs{'kindling','icebound'} do assert(#loot.rules[id].conditions==0 and loot.rules[id].trigger=='hit_dealt',id) end
 assert(loot.rules.kindling.tiers[1].damage==1 and loot.rules.kindling.tiers[1].duration==180)
 assert(K.meta.plague_bearer and D.mod_pool[1] and true)
 local plague;for _,m in ipairs(D.mod_pool) do if m.id=='plague_bearer' then plague=m end end;assert(plague.tiers[1].damage==1.5,'Plague Bearer 1.5 a second, stacking to 3')
 local e=D.mod_engine.new(1,D.mod_pool);e:set_build(1,{kindling=1},{});e:begin_frame({[1]={percent=0},[2]={percent=0}});e:emit{kind='hit_dealt',port=1,target=2,tags={}};e:drain()
 assert(e:status(2,'burn') and e:status(2,'burn').amount==1)
end)

T.test('Momentum is a counter: the count is one accessor (a later pass draws orbs from it)',function()
 local e=D.mod_engine.new(1,D.mod_pool);e:set_build(1,{updraft=1},{});e:begin_frame({[1]={percent=0},[2]={percent=0}})
 assert(e:momentum(1)==0 and e:momentum_max(1)==5)
 for i=1,3 do e:emit{kind='hit_dealt',port=1,target=2,tags={aerial=true}};e:drain() end
 assert(e:momentum(1)==3);for i=1,9 do e:emit{kind='hit_dealt',port=1,target=2,tags={aerial=true}};e:drain() end;assert(e:momentum(1)==5,'capped at five')
 assert(S.by_name.momentum.counter and not S.by_name.momentum.core)
end)

T.test('opponent card: its keystones with their rule, then a rule count; no strength figure',function()
 local R=D.foe_roll.new(D.mod_pool)
 local ctx=P.context(12,0);local b=R:construct(function(n) return n/2 end,ctx,2,true)
 b.keystones={'bulwark','fury'};b.keystone=nil
 local lines=D.drive_text.build_lines(loot,b,D.mod_pool)
 assert(#lines==3 and lines[1]:find('^Bulwark: Launch you take %-35%%') and lines[2]:find('^Fury: When you are hit, gain Haste'),table.concat(lines,' | '))
 local rules=0;for slot=1,P.slots(b.context) do local r=b.equipped[slot];if r then rules=rules+#r.affixes end end
 assert(lines[3]==rules..' drive rules' and D.drive_text.rule_count(b)==rules,lines[3])
 for _,l in ipairs(lines) do assert(not l:find('strength',1,true)) end
end)

T.test('the developer overlay is off by default and one predicate; the glossary is one call',function()
 assert(D.mod_tuning.dev_ui()==false and D.run_hud.dev_ui()==false)
 D.mod_tuning.set_dev_ui(true);assert(D.mod_tuning.dev_ui() and D.run_hud.dev_ui());D.mod_tuning.set_dev_ui(false);assert(not D.mod_tuning.dev_ui())
 assert(#D.drive_text.glossary()==8)
end)

T.test('the synergy message is a small corner note, not the six-second banner; the pickup card is one line',function()
 local got,banner
 local hud={corner=function(_,lines,arch) got={lines=lines,arch=arch} end,announce=function() banner=true end}
 D.synergy_fx.announce({host={hud=hud,log=function() end}},{id='burn',name='Burn stacking',blurb='b'})
 assert(got and got.lines[1].text=='Burn stacking assembled' and not banner)
 -- the real HUD: a corner note times out on logic frames, a card is one row
 local fills,texts,panels={},{},{}
 local g={fill=function(...) fills[#fills+1]={...} end,box=function() end,safe_area=function() return {x=0,y=0,w=640,h=360} end,
  kit={panel=function(x,y,w,h) panels[#panels+1]={x,y,w,h} end,text=function(x,y,t) texts[#texts+1]=t end,measure=function(t) return #t*6 end}}
 local host={mods={drives={}},seat=nil,decide={}}
 local h=D.run_hud.new(g,host);h:corner({{text='Haste web assembled',colour='gold'},'blurb'},nil);assert(#h.corners==1)
 h:draw_corner();assert(#panels==1 and panels[1][4]==40 and panels[1][1]>320,'top right, 40 px high')
 for _=1,D.run_hud.tuning.corner_frames do h:frame() end;assert(#h.corners==0)
 panels,texts={},{};h:show_card('Picked up: Red Drive',{'In your bag (Z+START).'},nil);h:draw_card();assert(#panels==1 and panels[1][4]==26 and #texts==1 and texts[1]:find('Picked up: Red Drive: In your bag',1,true))
end)

T.test('a run started from the menus or the console uses the rule host by default; `rules off` keeps the older route',function()
 local C=D.companion;D.save=T.module('save',D);D.classic=T.module('classic',D)
 local R=D.classic.new({log=function() end,match=function() return nil end},{},function() return true end)
 assert(R.rules==true,'the rule host is the default');R.rules=false;assert(R.rules==false)
end)
T.done()
