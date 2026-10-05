-- Keystones: build-defining rules, each with a real drawback (about thirty, spread over the six drive colours).
-- The concept (a small pool of rule-changing "keystone" passives that trade a drawback for a new way to play) is
-- Path of Exile's: https://www.pathofexile.com/ ; Grinding Gear Games' design is the acknowledged model for the
-- idea, nothing here copies a keystone's rules, names or numbers from that game.
--
-- Pure data plus pure selection. A record is an ordinary modifier record (mod_schema validates it); the metadata
-- beside it (drive family, the two plain-word lines, exclusions, whether it can be a starting keystone) lives in
-- K.meta so the record format does not change. Every record here uses only what the modifier engine and the
-- run host apply TODAY: fighter values, native hit rules, statuses, heal/damage, one echo window, existing events.
-- K.waiting lists good keystones that still need engine work (an item route); they are data only, never in the pool,
-- so nothing is faked. The technique keystones (wavedash, L-cancel, perfect shield, combos, crits, armour, air jumps) moved
-- into the pool when the engine's skill events, crits and armour types landed (see the technique section below).
--
-- A one-trigger record can only carry effects of that trigger, so a triggered keystone pays its drawback with
-- an effect of the same trigger (self damage, a status on yourself, a status removed); an equip keystone pays
-- with a fighter value. Afterimage rule: nothing here draws an afterimage; the one echo keystone (Echo Weaver)
-- only echoes while Haste is on you, so it starts with Haste and ends with it.
return function(D)
 local K={}
 local S=D.mod_schema
 local function visual(look,hue) return {look=look,hue=hue,strength=.6,priority=35} end
 local function status(name,subject,duration,amount,max)
  return {op='status',status=name,subject=subject,duration=duration,amount=amount or 1,max=max or 1,refresh='refresh'}
 end
 local function drop(name,subject) return {op='remove_status',status=name,subject=subject or 'self'} end
 local function value(key,v) return {op='value',key=key,value=v} end
 local function round(n) return math.floor(n+.5) end
 local function pct(n) return ('%d%%'):format(round(n*100)) end
 local function secs(f) local s=f/60;return s==math.floor(s) and ('%d seconds'):format(s) or ('%.1f seconds'):format(s) end
 K.records_list={};K.meta={};K.order={}
 -- id, label, drive family (colour), trigger, conditions, effects, tier values, cost sentence, look, hue, tags, meta
 local function ks(id,label,family,trigger,conditions,effects,tier,cost,look,hue,tags,meta,interval)
  local r={id=id,label=label,kind='keystone',cost=cost,tags=tags,trigger=trigger,interval=interval,conditions=conditions,effects=effects,
   tiers={tier},stacking={max=1},text=label..': '..cost,visual=visual(look,hue)}
  K.records_list[#K.records_list+1]=r;K.order[#K.order+1]=id
  meta.family=family;K.meta[id]=meta
 end
 local function tags(...) local t={'keystone'};for _,x in ipairs({...}) do t[#t+1]=x end;return t end
 -- ---- red: damage dealt ---------------------------------------------------------------------------------------
 ks('smash_doctrine',"Smasher's Creed",'red','equip',{},
  {{op='convert',match={move='smash'},change={percent_damage=1.6}},value('run_speed',.85)},{},'Run speed -15%.','burn',.02,tags('smash','damage'),
  {effect=function(x) return 'Smash attacks deal '..x.up(1.6)..' more damage' end,drawback='Drawback: you run 15% slower',excludes={'aerial_doctrine'},uses='native hit rule + fighter value'})
 ks('aerial_doctrine','Skybreaker','red','equip',{},
  {{op='convert',match={move='aerial'},change={percent_damage=1.5}},value('jump_height',.8)},{},'Jump height -20%.','momentum',.08,tags('aerial','damage'),
  {effect=function(x) return 'Aerial attacks deal '..x.up(1.5)..' more damage' end,drawback='Drawback: you jump 20% lower',excludes={'smash_doctrine'},uses='native hit rule + fighter value'})
 ks('bloodlust','Bloodlust','red','ko_dealt',{},
  {{op='heal',amount='$heal',subject='self'},status('haste','self','$duration'),status('curse','self',180,.1)},{heal=20,duration=180},
  'Be launched 10% farther for 3 seconds.','curse',.0,tags('hasted','healing'),
  {effect=function(x) return 'Knock out a fighter: heal '..round(x.res('heal'))..'% and gain Haste for '..secs(x.res('duration')) end,
   drawback='Drawback: you are launched 10% farther for the same time',uses='ko_dealt event, heal, Haste, Curse on self'})
 ks('last_stand','Last Stand','red','interval',{{last_stock=true}},
  {status('guarded','self',90),status('haste','self',90),{op='damage',amount=2,subject='self'}},{},
  'On your last stock you lose 2 damage points every second.','guarded',.99,tags('guarded','hasted'),
  {effect=function() return 'On your last stock you are always Guarded and Hasted' end,drawback='Drawback: on your last stock you bleed 2 damage every second',starter=false,uses='interval + last_stock condition'},60)
 -- ---- green: speed -------------------------------------------------------------------------------------------
 ks('sprinter','Sprinter\'s Pact','green','equip',{},
  {value('run_speed',1.35),value('air_speed',1.2),value('knockback_taken',1.2)},{},'Launched 20% farther.','haste',.34,tags('hasted'),
  {effect=function() return 'Run speed +35% and air speed +20%' end,drawback='Drawback: you are launched 20% farther',excludes={'juggernaut'},uses='fighter values'})
 ks('hit_and_run','Hit and Run','green','hit_dealt',{},
  {status('haste','self','$duration'),status('momentum','self','$duration',1,5),status('curse','self',120,.05)},{duration=120},
  'Launched 5% farther for 2 seconds after each hit.','haste',.3,tags('hasted','momentum'),
  {effect=function(x) return 'Every hit you land gives Haste and Momentum for '..secs(x.res('duration')) end,drawback='Drawback: each hit also makes you launched 5% farther for the same time',uses='hit_dealt, Haste, Momentum, Curse on self'})
 ks('perpetual_motion','Perpetual Motion','green','landing',{},
  {status('haste','self','$duration'),{op='damage',amount=1,subject='self'}},{duration=240},
  'Every landing costs 1 damage point.','haste',.36,tags('hasted'),
  {effect=function(x) return 'Every landing gives Haste for '..secs(x.res('duration')) end,drawback='Drawback: every landing costs you 1 damage point',uses='landing event, Haste, self damage'})
 ks('banked_momentum','Banked Momentum','green','interval',{},
  {status('momentum','self',600,1,5),status('curse','self',90,.05)},{},'You are always launched 5% farther.','momentum',.16,tags('momentum'),
  {effect=function() return 'Every second you gain a Momentum stack (up to 5)' end,drawback='Drawback: you are always launched 5% farther',excludes={'iron_resolve'},uses='interval, Momentum, Curse on self'},60)
 -- ---- blue: defence -----------------------------------------------------------------------------------------
 ks('bulwark','Bulwark','blue','equip',{},
  {value('knockback_taken',.6),value('damage_taken',.8),value('run_speed',.75),value('jump_height',.85)},{},'Run speed -25% and jump height -15%.','guarded',.6,tags('guarded'),
  {effect=function() return 'Launch you take -40% and damage you take -20%' end,drawback='Drawback: you run 25% slower and jump 15% lower',uses='fighter values'})
 ks('iron_resolve','Iron Resolve','blue','interval',{},
  {status('guarded','self',60),drop('momentum')},{},'You cannot hold Momentum.','guarded',.63,tags('guarded','momentum'),
  {effect=function() return 'You are always Guarded: 25% less damage and 15% less launch taken' end,drawback='Drawback: you cannot hold Momentum',excludes={'banked_momentum'},uses='interval, Guarded, remove Momentum'},30)
 ks('retribution','Retribution','blue','hit_taken',{},
  {status('curse','target','$duration','$bonus'),status('chill','self',120)},{duration=180,bonus=.08},'You are Chilled for 2 seconds.','curse',.78,tags('cursed','chilled'),
  {effect=function(x) return 'When you are hit, the attacker is Cursed (later hits launch them '..pct(x.res('bonus'))..' farther) for '..secs(x.res('duration')) end,drawback='Drawback: you are Chilled (20% slower) for 2 seconds',uses='hit_taken, Curse on attacker, Chill on self'})
 ks('parry_master','Parry Master','blue','perfect_shield',{},
  {status('guarded','self','$duration'),{op='heal',amount='$heal',subject='self'},status('chill','self',90)},{duration=300,heal=10},'You are Chilled for 1.5 seconds.','guarded',.58,tags('guarded','healing'),
  {effect=function(x) return 'Perfect shield: Guarded for '..secs(x.res('duration'))..' and heal '..round(x.res('heal'))..'%' end,drawback='Drawback: you are Chilled (20% slower) for 1.5 seconds',uses='perfect_shield event, Guarded, heal, Chill on self'})
 -- ---- yellow: jump and air ------------------------------------------------------------------------------------
 ks('skyborne','Skyborne','yellow','equip',{},
  {value('jump_height',1.5),value('air_jump_height',1.5),value('knockback_taken',1.25)},{},'Launched 25% farther.','momentum',.14,tags('aerial'),
  {effect=function() return 'Jump height and air jump height +50%' end,drawback='Drawback: you are launched 25% farther',uses='fighter values'})
 ks('dive_bomber','Dive Bomber','yellow','equip',{},
  {{op='convert',match={move='aerial'},change={launch=1.3}},value('run_speed',.8)},{},'Run speed -20%.','momentum',.12,tags('aerial'),
  {effect=function() return 'Aerial attacks launch 30% farther' end,drawback='Drawback: you run 20% slower',uses='native hit rule + fighter value'})
 ks('skyfarer','Skyfarer','yellow','jump',{},
  {status('haste','self','$duration'),{op='damage',amount=1,subject='self'}},{duration=90},'Every jump costs 1 damage point.','haste',.33,tags('hasted','aerial'),
  {effect=function(x) return 'Every jump gives Haste for '..secs(x.res('duration')) end,drawback='Drawback: every jump costs you 1 damage point',uses='jump event, Haste, self damage'})
 ks('hang_time','Hang Time','yellow','air_jump',{},
  {status('guarded','self','$duration'),status('momentum','self',180,1,5),status('curse','self',120,.05)},{duration=120},'Launched 5% farther for 2 seconds.','guarded',.5,tags('guarded','momentum','aerial'),
  {effect=function(x) return 'Every air jump gives Guarded for '..secs(x.res('duration'))..' and Momentum' end,drawback='Drawback: each air jump also makes you launched 5% farther for 2 seconds',uses='air_jump event, Guarded, Momentum, Curse on self'})
 -- ---- purple: statuses --------------------------------------------------------------------------------------
 ks('pandemic','Pandemic','purple','equip',{},{value('status_duration',1.6),value('damage_dealt',.85)},{},'Damage you deal -15%.','curse',.7,tags('cursed'),
  {effect=function() return 'Statuses you cause last 60% longer' end,drawback='Drawback: damage you deal -15%',uses='fighter values'})
 ks('opportunist','Opportunist','purple','hit_dealt',{{target_status='any'}},
  {status('curse','target','$duration','$bonus'),{op='damage',amount=1,subject='self'}},{duration=150,bonus=.06},'Each such hit costs you 1 damage point.','curse',.74,tags('cursed'),
  {effect=function(x) return 'Hit a target with any status: Curse it (later hits launch it '..pct(x.res('bonus'))..' farther) for '..secs(x.res('duration')) end,drawback='Drawback: each such hit costs you 1 damage point',uses='hit_dealt + target_status condition, Curse, self damage'})
 ks('plague_bearer','Plague Bearer','purple','hit_dealt',{},
  {status('burn','target','$duration','$damage',3),status('burn','self',60,1)},{duration=120,damage=2},'Each hit also sets you Burning for 1 second.','burn',.04,tags('burning','fire'),
  {effect=function(x) return 'Every hit sets the target Burning for '..secs(x.res('duration'))..' ('..round(x.res('damage'))..' damage a second, stacking to 3)' end,drawback='Drawback: each hit also sets you Burning for 1 second',uses='hit_dealt, Burn on target and self'})
 ks('deep_freeze','Deep Freeze','purple','hit_dealt',{},
  {status('chill','target','$duration'),status('curse','target','$duration','$bonus'),status('chill','self',60)},{duration=240,bonus=.04},'Each hit also Chills you for 1 second.','chill',.53,tags('chilled','ice'),
  {effect=function(x) return 'Every hit Chills and Curses the target for '..secs(x.res('duration')) end,drawback='Drawback: each hit also Chills you (20% slower) for 1 second',uses='hit_dealt, Chill and Curse on target, Chill on self'})
 ks('everburn','Everburn','purple','equip',{},
  {{op='versus-status',status='burn',match={},change={percent_damage=1.5,launch=1.15}},value('status_duration',1.3),value('damage_taken',1.15)},{},'Damage you take +15%.','burn',.05,tags('burning','damage'),
  {effect=function(x) return 'Burning targets take '..x.up(1.5)..' more damage and launch 15% farther; your statuses last 30% longer' end,drawback='Drawback: you take 15% more damage',uses='native versus-status rule + fighter values'})
 -- ---- white: rule-bending -----------------------------------------------------------------------------------
 ks('clash_king','Clash King','white','clank',{},
  {{op='heal',amount='$heal',subject='self'},status('haste','self','$duration'),{op='damage',amount=2,subject='self'}},{heal=10,duration=150},'Each clank costs you 2 damage points.','shock',.57,tags('hasted','healing'),
  {effect=function(x) return 'Clank with a fighter: heal '..round(x.res('heal'))..'% and gain Haste for '..secs(x.res('duration')) end,drawback='Drawback: each clank costs you 2 damage points',starter=false,uses='clank event, heal, Haste, self damage'})
 ks('echo_weaver','Echo Weaver','white','equip',{},
  {{op='echo',copies=2,delay=8,damage=.5,knockback=1,match={move='any'},once_per_move=true,status='haste'},value('damage_dealt',.9)},{},'Damage you deal -10%.','haste',.72,tags('hasted','damage'),
  {effect=function() return 'While Hasted, your attacks repeat twice at half damage' end,drawback='Drawback: damage you deal -10%',excludes={'echo_oath'},starter=false,uses='echo effect gated on Haste (starts and ends with the status)'})
 ks('finishers_mark',"Finisher's Mark",'white','hit_dealt',{{target_damage_above=100}},
  {status('curse','target','$duration','$bonus'),status('chill','self',60)},{duration=150,bonus=.08},'Each such hit Chills you for 1 second.','curse',.8,tags('cursed'),
  {effect=function(x) return 'Hit a target above 100% damage: Curse it (later hits launch it '..pct(x.res('bonus'))..' farther) for '..secs(x.res('duration')) end,drawback='Drawback: each such hit Chills you (20% slower) for 1 second',uses='hit_dealt + target_damage_above condition, Curse, Chill on self'})
 ks('desperado','Desperado','white','interval',{{self_damage_above=80}},
  {status('haste','self',60),status('guarded','self',60),status('curse','self',60,.1)},{},'While you are above 80% damage you are launched 10% farther.','haste',.9,tags('hasted','guarded'),
  {effect=function() return 'While you are above 80% damage you are Hasted and Guarded' end,drawback='Drawback: in that state you are launched 10% farther',starter=false,uses='interval + self_damage_above condition'},30)
 ks('fury','Fury','white','hit_taken',{},
  {status('momentum','self',300,1,5),status('haste','self','$duration'),drop('guarded')},{duration=180},'Being hit ends your Guarded status.','momentum',.97,tags('hasted','momentum'),
  {effect=function(x) return 'When you are hit, gain Momentum and Haste for '..secs(x.res('duration')) end,drawback='Drawback: being hit ends your Guarded status',uses='hit_taken, Momentum, Haste, remove Guarded'})
 -- ---- technique keystones: the engine's skill events, armour types, caps and crits -----------------------------
 -- Triggered by a technique, so each earns its reward by playing Melee well, and pays on the same trigger. `technique` names
 -- the skill events they listen to (mod_skill: verified in the game or flagged) and `cpu` says whether a retail CPU opponent can
 -- ever fire them (an opponent's roll of a 'dead' one is inert: opponents are not scripted).
 ks('wavedasher','Wavedasher','green','wavedash',{},
  {{op='armor',type='super',frames=6},status('guarded','self',12),{op='damage',amount=2,subject='self'}},{},
  'Each wavedash costs 2 damage points.','haste',.12,tags('guarded','technique'),
  {effect=function() return 'Wavedash: super armour for 6 frames, then Guarded for 12 (gold afterimages while it lasts)' end,drawback='Drawback: each wavedash costs you 2 damage points',starter=false,technique={'wavedash'},uses='wavedash skill event, super armour (6 frames), Guarded, self damage'})
 ks('clean_lander','Clean Lander','green','lcancel_hit',{},
  {status('haste','self','$duration'),{op='damage',amount=1,subject='self'}},{duration=120},
  'Each such landing costs 1 damage point.','haste',.62,tags('hasted','aerial','technique'),
  {effect=function(x) return 'L-cancel a landing after the aerial hit: Haste for '..secs(x.res('duration'))..' (blue afterimages for exactly that long)' end,drawback='Drawback: each such landing costs you 1 damage point',starter=false,technique={'lcancel_hit'},uses='hit-confirmed L-cancel skill event, Haste (earned), self damage'})
 ks('powershield_oath','Powershield Oath','blue','perfect_shield',{},
  {{op='armor',type='hit_count',value=1,frames=90},status('guarded','self',90),status('chill','self',60)},{},
  'You are Chilled for 1 second.','guarded',.5,tags('guarded','technique'),
  {effect=function() return 'Perfect shield: absorb the next hit with armour and be Guarded for 1.5 seconds (teal afterimages)' end,drawback='Drawback: you are Chilled (20% slower) for 1 second',starter=false,excludes={'aerialist'},technique={'perfect_shield'},uses='perfect_shield, hit-count armour, Guarded (earned), Chill on self'})
 ks('juggernaut','Juggernaut','blue','equip',{},
  {{op='armor',type='damage_threshold',value=6},{op='restrict',forbid={'run'}}},{},'You cannot run.','guarded',.55,tags('guarded','technique'),
  {effect=function() return 'You do not flinch from hits that deal under 6 damage' end,drawback='Drawback: you cannot run',starter=false,excludes={'sprinter'},uses='permanent damage-threshold armour + run restriction (fighter caps)'})
 ks('executioner','Executioner','red','equip',{},
  {{op='crit',chance=1,multiplier=2,min_percent=100},value('damage_dealt',.9)},{},'Damage you deal -10%, and hits on a target under 100% never crit.','burn',.0,tags('damage','critical'),
  {effect=function() return 'Hits on a target above 100% damage always crit for double' end,drawback='Drawback: damage you deal -10%, and no hit below 100% can crit',starter=false,excludes={'gambler'},uses='crit chance with a percent floor, fighter value'})
 ks('critical_mass','Critical Mass','white','crit',{},
  {status('momentum','self','$duration',1,5),status('shock','target','$shock'),status('curse','self',90,.05)},{duration=300,shock=180},'A crit also makes you launched 5% farther for 1.5 seconds.','momentum',.1,tags('momentum','critical','shocked'),
  {effect=function(x) return 'Land a crit: gain Momentum (up to 5) for '..secs(x.res('duration'))..' and Shock the target (its next hit taken stuns longer)' end,drawback='Drawback: each crit also makes you launched 5% farther for 1.5 seconds',starter=false,uses='crit event, Momentum, Shock on the target, Curse on self'})
 ks('combo_conduit','Combo Conduit','red','combo',{{combo_at_least=3}},
  {{op='crit_next',count=1,multiplier=1.6},status('chill','self',60)},{},'Each such hit Chills you for 1 second.','curse',.98,tags('critical','technique'),
  {effect=function() return 'Land the third or later hit of a combo (the second on a Cursed target): your next hit crits for x1.6' end,drawback='Drawback: each such hit Chills you (20% slower) for 1 second',starter=false,technique={'combo'},uses='combo skill event with a count condition, forced crit, Chill on self'})
 K.records_list[#K.records_list].also={{trigger='combo',conditions={{combo_at_least=2},{target_status='curse'}}}}   -- also fires from the second hit on a Cursed target: a reader of Curse
 ks('aerialist','Aerialist','yellow','equip',{},
  {{op='air_jumps',count=5},{op='restrict',forbid={'shield'}}},{},'You cannot shield.','momentum',.14,tags('aerial','technique'),
  {effect=function() return 'You have five air jumps' end,drawback='Drawback: you cannot shield',starter=false,excludes={'powershield_oath'},uses='air-jump count and a shield restriction (fighter caps)'})
 ks('phase_dash','Phase Dash','green','air_dodge',{},
  {{op='intangible',frames=4},{op='damage',amount=1,subject='self'}},{},'Each air dodge costs 1 damage point.','haste',.5,tags('technique'),
  {effect=function() return 'Air dodge: you are intangible for 4 extra frames' end,drawback='Drawback: each air dodge costs you 1 damage point',starter=false,technique={'air_dodge'},uses='air_dodge skill event (flagged: not confirmed in play), intangibility, self damage'})
 ks('conductor','Conductor','purple','hit_dealt',{{tag='electric'}},
  {status('shock','target','$duration'),{op='chain_status',status='shock',duration='$duration',amount=1,max=1,refresh='refresh'},{op='damage',amount=1,subject='self'}},{duration=150},'Each electric hit costs you 1 damage point.','shock',.58,tags('electric','shocked'),
  {effect=function(x) return 'Electric hits Shock the target and chain Shock to the nearest other opponent for '..secs(x.res('duration')) end,drawback='Drawback: each electric hit costs you 1 damage point',starter=false,uses='hit_dealt with an electric condition, Shock on the target, chain_status to the nearest other opponent, self damage'})
 ks('gambler','Gambler','white','equip',{},
  {{op='crit',chance=.35,multiplier=2,multiplier_max=3},value('damage_dealt',.8)},{},'All your hits deal 20% less damage before any crit.','shock',.12,tags('damage','critical'),
  {effect=function() return 'Every hit has a 35% chance to deal double or triple damage' end,drawback='Drawback: all your hits deal 20% less damage first',starter=false,excludes={'executioner'},uses='crit chance with a multiplier range, fighter value'})
 ks('featherfall','Featherfall','yellow','equip',{},
  {value('fall_speed',.65),value('weight',.75),value('knockback_taken',1.3)},{},'Launched 30% farther.','momentum',.15,tags('aerial'),
  {effect=function() return 'You fall 35% slower and weigh 25% less' end,drawback='Drawback: you are launched 30% farther',uses='fall-speed and weight fighter values'})
 -- ---- keystones that already lived in mod_pool.lua (kept there; their metadata is here) ---------------------
 K.legacy={
  pyromancer={family='red',effect=function() return 'All your attacks become fire' end,drawback='Drawback: ice hits against you deal 60% more damage',excludes={'frozen_oath'},uses='native convert rule + versus-status (incoming)'},
  frozen_oath={family='blue',effect=function() return 'All your attacks become ice' end,drawback='Drawback: fire hits against you deal 60% more damage',excludes={'pyromancer'},uses='native convert rule + versus-status (incoming)'},
  still_heart={family='green',effect=function() return 'Moving fast turns into taking less damage instead' end,drawback='Drawback: you cannot get a speed boost',uses='status_applied + remove Haste, Guarded'},
  echo_oath={family='white',effect=function() return 'Every attack repeats a moment later at reduced damage' end,drawback='Drawback: all your attack damage is 50% lower',starter=false,excludes={'echo_weaver'},uses='echo effect (existing)'},
 }
 for id,m in pairs(K.legacy) do K.meta[id]=m;K.order[#K.order+1]=id end
 table.sort(K.order)
 K.families={'red','green','blue','yellow','purple','white'}
 K.family_names={red='Damage',green='Speed',blue='Defence',yellow='Air',purple='Status',white='Wild'}
 -- The records for the pool. Families are declared by mod_pool from the budget (it owns that vocabulary).
 function K.records() local out={};for i,r in ipairs(K.records_list) do out[i]=r end;return out end
 function K.ids() local out={};for i,id in ipairs(K.order) do out[i]=id end;return out end
 function K.family(id) return K.meta[id] and K.meta[id].family end
 -- Two plain lines for a keystone at a tier: its effect, then its drawback. No ids, no tiers, no budget numbers.
 function K.lines(m,tier)
  local meta=K.meta[m.id];if not meta then return nil end
  local x={m=m,tier=tier or 1}
  function x.res(key) return S.resolve('$'..key,m,x.tier) end
  function x.up(v) return pct(S.ratio(v,m,x.tier,'damage_dealt')-1) end
  local drawback=type(meta.drawback)=='function' and meta.drawback(x) or meta.drawback
  return {meta.effect(x),drawback}
 end
 -- ---- exclusion and drawback limits ---------------------------------------------------------------------------
 -- Mutually exclusive pairs are refused. Stacked drawbacks add (the budget families add them too) and are floored:
 -- no combination may take damage dealt below x0.6 or a speed/jump value below x0.55 overall, or raise damage taken
 -- above x1.8 or launch taken above x1.6, so a pile of keystones cannot build an unplayable character.
 K.limits={damage_dealt={-.4,nil},run_speed={-.45,nil},air_speed={-.45,nil},jump_height={-.45,nil},damage_taken={nil,.8},knockback_taken={nil,.6},fall_speed={-.6,nil},weight={-.5,nil}}
 local function by_id()
  if not K.by_id then K.by_id={};for _,r in ipairs(K.records_list) do K.by_id[r.id]=r end end
  return K.by_id
 end
 function K.check(ids,pool)
  local seen,count={},0
  for _,id in ipairs(ids) do
   if not K.meta[id] then return nil,'unknown keystone '..tostring(id) end
   if seen[id] then return nil,'duplicate keystone' end
   seen[id]=true;count=count+1
  end
  for id in pairs(seen) do for _,other in ipairs(K.meta[id].excludes or {}) do
   if seen[other] then return nil,'Those keystones exclude each other.' end
  end end
  local sums={}
  for id in pairs(seen) do local r=by_id()[id]
   for _,e in ipairs(r and r.effects or {}) do if e.op=='value' and K.limits[e.key] and type(e.value)=='number' then sums[e.key]=(sums[e.key] or 0)+e.value-1 end end
  end
  local forbid,seenf={},{}
  for id in pairs(seen) do local r=by_id()[id]
   for _,e in ipairs(r and r.effects or {}) do if e.op=='restrict' then for _,x in ipairs(e.forbid) do if not seenf[x] then seenf[x]=true;forbid[#forbid+1]=x end end end end
  end
  local okr,whyr=D.mod_budget and D.mod_budget.restrictions_ok(forbid) or true
  if not okr then return nil,whyr end
  for key,lim in pairs(K.limits) do local n=sums[key] or 0
   if (lim[1] and n<lim[1]-1e-9) or (lim[2] and n>lim[2]+1e-9) then return nil,'Those keystones stack too many drawbacks.' end
  end
  return true
 end
 function K.exclusive(a,b)
  for _,other in ipairs((K.meta[a] or {}).excludes or {}) do if other==b then return true end end
  return false
 end
 -- ---- selection (pure, seeded; the run host calls these) --------------------------------------------------------
 local function rng(seed)
  local state=(seed*104729)%2147483646+1
  return function(n) state=(state*16807)%2147483647;return (state-1)/2147483646*n end
 end
 local function integer(n) return type(n)=='number' and n==n and n%1==0 and n>=0 and n<=2147483646 end
 -- The keystone every run starts with: one id, rolled from the run seed over the pool that can be played from
 -- the first stage (a keystone that needs a rare trigger is not a starting keystone), equipped without a choice.
 function K.starting(seed)
  assert(integer(seed),'invalid seed');local rand=rng(seed);local pool={}
  for _,id in ipairs(K.order) do if K.meta[id].starter~=false then pool[#pool+1]=id end end
  return pool[math.floor(rand(#pool))+1]
 end
 -- A choice of `n` (default 3) keystones not held, compatible with the held ones, from different drive families
 -- where possible, deterministic in (seed, how many are held) so a retry shows the same choice.
 -- `connect` (optional {held=id set, pool=, graph=}): when none of the offered keystones connects to anything held (the derived graph, mod_graph),
 -- one offer (a seeded choice) is swapped for a candidate that does; nothing else about the offer changes.
 function K.offer(context,seed,n,held,connect)
  assert(integer(seed),'invalid seed');n=n or 3;held=held or {}
  local owed=D.mod_progression.allowance(context)-#held
  if owed<=0 then return {} end
  local rand=rng(seed+#held*7919);local candidates={}
  local isheld={};for _,id in ipairs(held) do isheld[id]=true end
  for _,id in ipairs(K.order) do if not isheld[id] then
   local trial={};for _,h in ipairs(held) do trial[#trial+1]=h end;trial[#trial+1]=id
   if K.check(trial) then candidates[#candidates+1]=id end
  end end
  for i=#candidates,2,-1 do local j=math.floor(rand(i))+1;candidates[i],candidates[j]=candidates[j],candidates[i] end
  local out,families={},{}
  for _,id in ipairs(candidates) do if #out<n and not families[K.meta[id].family] then out[#out+1]=id;families[K.meta[id].family]=true end end
  for _,id in ipairs(candidates) do if #out<n then local dup;for _,o in ipairs(out) do if o==id then dup=true end end;if not dup then out[#out+1]=id end end end
  if connect and #out>0 and next(connect.held or {}) then
   local G=connect.graph;local pool=connect.pool
   if not G.connects(pool,out,connect.held) then
    local partners={}
    for _,id in ipairs(candidates) do local dup;for _,o in ipairs(out) do if o==id then dup=true end end
     if not dup and G.connects(pool,{id},connect.held) then partners[#partners+1]=id end end
    if #partners>0 then local pick=partners[math.floor(rand(#partners))+1];out[math.floor(rand(#out))+1]=pick end
   end
  end
  return out
 end
 -- Allowance steps not yet used: how many keystones the player may still be offered.
 function K.owed(context,held) return math.max(0,D.mod_progression.allowance(context)-#(held or {})) end
 -- Good keystones that need engine work. Data only: never in the pool. `needs` names the missing piece.
 K.waiting={
  {id='hoarder',label='Hoarder',family='white',effect='Start every stage holding a random item.',drawback='You take 15% more damage from items.',needs='a journalled give_item effect (sim_commit has no give_item operation) and an item-damage hook for the drawback'},
 }
 return K
end
