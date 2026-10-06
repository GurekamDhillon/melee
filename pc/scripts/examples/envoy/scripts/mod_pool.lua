-- Shared-tag rules; native creation/contact changes are composed before gameplay.
return function(D)
 local function visual(look,hue) return {look=look,hue=hue,strength=.35,priority=10} end
 local function status(name,subject,duration,amount,max)
  return {op='status',status=name,subject=subject,duration=duration,amount=amount or 1,max=max or 1,refresh='refresh'}
 end
 local function rule(id,label,tags,trigger,conditions,effects,tiers,text,look,hue,kind,cost)
  return {id=id,label=label,kind=kind or 'normal',cost=cost,tags=tags,trigger=trigger,conditions=conditions,effects=effects,
   tiers={tiers},stacking={max=1},text=text,visual=visual(look,hue)}
 end
 local pool={
  rule('kindling','Kindling',{'burning'},'hit_dealt',{},{status('burn','target','$duration','$damage')},{duration=180,damage=1},'Your hits set the target Burning for {duration} frames: {damage} damage each second.','burn',.03),
  rule('pyre','Pyre',{'burning','damage'},'equip',{},{{op='versus-status',status='burn',match={},change={launch='$ratio'}}},{ratio=1.08,bonus=.08},'Your hits launch Burning targets {bonus%} farther.','curse',.76),
  rule('burning','Burning',{'smash','fire'},'equip',{},{{op='convert',match={move='smash'},change={element='fire'}}},{},'Your ordinary smash hitboxes become Fire at creation.','burn',.08),
  rule('charged','Charged',{'aerial','electric'},'equip',{},{{op='convert',match={move='aerial'},change={element='electric'}}},{},'Your ordinary aerial hitboxes become Electric at creation.','shock',.57),
  rule('pyromancer','Pyromancer',{'keystone','fire','ice'},'equip',{},{{op='convert',match={move='any'},change={element='fire'}},{op='versus-status',match={incoming=true,element='ice'},change={percent_damage=1.6}}},{},'Your ordinary owned attack hitboxes become Fire.','burn',.01,'keystone','Hits with an original Ice element deal +60% damage to you.'),
  rule('feasting','Feasting',{'healing'},'ko_dealt',{{target_status='any'}},{{op='heal',amount='$heal',subject='self'}},{heal=10},'Knock out a target that has a status: heal {heal} damage points.','guarded',.46),
  rule('updraft','Updraft',{'aerial','momentum'},'hit_dealt',{{tag='aerial'}},{status('momentum','self','$duration',1,5)},{duration=300},'Aerial hits add 1 Momentum (up to 5) for {duration} frames.','momentum',.14),
  rule('crosswind','Crosswind',{'momentum','hasted'},'landing',{{self_status='momentum'}},{{op='remove_status',status='momentum',subject='self',count=1},status('haste','self','$duration')},{duration=180},'Land with Momentum: spend 1, gain Haste for {duration} frames.','haste',.34),
  rule('icebound','Icebound',{'chilled'},'hit_dealt',{},{status('chill','target','$duration')},{duration=120},'Your hits Chill the target for {duration} frames.','chill',.54),
  rule('brittle','Brittle',{'chilled','cursed'},'hit_dealt',{{target_status='chill'}},{status('curse','target','$duration','$bonus')},{duration=120,bonus=.025},'Hit a Chilled target: Mark it for {duration} frames (later hits launch it {bonus%} farther).','curse',.73),
  rule('reprisal','Reprisal',{'guarded'},'perfect_shield',{}, {status('guarded','self','$duration')},{duration=180},'Perfect shield: Guarded for {duration} frames.','guarded',.59),
  rule('glass_core','Glass Core',{'unique','damage'},'equip',{},{{op='value',key='damage_dealt',value=1.6},{op='value',key='damage_taken',value=1.6}},{},'Attack percent damage dealt and taken +60%; launch unchanged.','shock',.89,'unique','Take 60% more attack percent damage.')
 }
 local function add(...) pool[#pool+1]=rule(...) end
 add('frosted','Frosted',{'ice','chilled'},'equip',{},{{op='convert',match={move='smash'},change={element='ice'}}},{},'Your ordinary smash hitboxes become Ice.','chill',.51)
 add('lingering','Lingering',{'burning','chilled','hasted'},'equip',{},{{op='value',key='status_duration',value='$ratio'}},{ratio=1.2},'Statuses you cause last x{ratio} as long.','curse',.69)
 add('cinder','Cinder',{'burning','damage'},'equip',{},{{op='versus-status',status='burn',match={},change={percent_damage='$ratio'}}},{ratio=1.1},'Connecting hits percent damage Burning targets x{ratio}.','burn',.06)
 add('shatter','Shatter',{'chilled','damage'},'equip',{},{{op='versus-status',status='chill',match={},change={percent_damage='$ratio'}}},{ratio=1.1},'Connecting hits percent damage Chilled targets x{ratio}.','chill',.55)
 add('ledge','Ledge',{'hasted','momentum'},'ledge_grab',{}, {status('haste','self','$duration')},{duration=180},'Grab a ledge: Haste for {duration} frames.','haste',.32)
 add('bastion','Bastion',{'guarded','momentum'},'landing',{{self_status='momentum'}}, {{op='remove_status',status='momentum',subject='self',count=1},status('guarded','self','$duration')},{duration=120},'Land with Momentum: spend 1, gain Guarded for {duration} frames.','guarded',.6)
 add('renewal','Renewal',{'healing','guarded'},'status_applied',{{status='guarded'}}, {{op='heal',subject='self',amount='$heal'}},{heal=2},'Whenever you become Guarded: heal {heal} damage points.','guarded',.42)
 add('rush','Rush',{'hasted','momentum'},'hit_dealt',{{self_status='haste'}}, {status('momentum','self','$duration',1,5)},{duration=240},'Hit while you have Haste: add 1 Momentum for {duration} frames.','momentum',.19)
 add('shelter','Shelter',{'guarded','healing'},'ledge_grab',{}, {status('guarded','self','$duration')},{duration=120},'Grab a ledge: Guarded for {duration} frames.','guarded',.64)
 add('malice','Malice',{'cursed','burning'},'hit_dealt',{{target_status='burn'}}, {status('curse','target','$duration','$bonus')},{duration=120,bonus=.025},'Hit a Burning target: Mark it for {duration} frames (later hits launch it {bonus%} farther).','curse',.8)
 add('ember_crown','Ember Crown',{'unique','fire','burning'},'equip',{},{{op='convert',match={move='any'},change={element='fire'}},{op='value',key='damage_taken',value=1.3}},{},'Your ordinary hitboxes become Fire; take x1.3 damage.','burn',.025,'unique','Take 30% more attack damage.')
 add('winter_heart','Winter Heart',{'unique','ice','chilled'},'equip',{},{{op='convert',match={move='any'},change={element='ice'}},{op='value',key='run_speed',value=.8}},{},'Your ordinary hitboxes become Ice; run speed x0.8.','chill',.52,'unique','Run 20% slower.')
 add('storm_shell','Storm Shell',{'unique','electric'},'equip',{},{{op='convert',match={move='any'},change={element='electric'}},{op='value',key='damage_dealt',value=.8}},{},'Ordinary hitboxes become Electric; damage dealt x0.8.','shock',.58,'unique','Deal 20% less attack damage.')
 add('mirror_shard','Mirror Shard',{'unique','damage','guarded'},'clank',{},{{op='clank_damage',subject='target'}},{},'Clank with a fighter: that opponent takes damage equal to both clashing attacks combined.','curse',.82,'unique','Requires a fighter-to-fighter clank; no benefit against item clanks.')
 add('frozen_oath','Frozen Oath',{'keystone','ice','chilled','damage'},'equip',{},{{op='convert',match={move='any'},change={element='ice'}},{op='versus-status',match={incoming=true,element='fire'},change={percent_damage=1.6}}},{},'Your ordinary owned hitboxes become Ice.','chill',.5,'keystone','Original Fire hits against you deal +60% damage.')
 add('armoured','Damage resistant',{'guarded'},'equip',{},{{op='value',key='damage_taken',value='$ratio'}},{ratio=.92},'Attack percent damage taken x{ratio}; launch unchanged.','guarded',.61)
 add('cleansing','Cleansing',{'guarded'},'interval',{},{{op='remove_status',status='burn',subject='self'},{op='remove_status',status='chill',subject='self'},{op='remove_status',status='curse',subject='self'}},{},'Every second, Burning, Chilled and Marked on you wear off.','guarded',.44)
 pool[#pool].interval=60
 local colours={glass_core='white',ember_crown='red',winter_heart='blue',storm_shell='yellow',mirror_shard='purple'}
 for _,m in ipairs(pool) do
  if m.kind=='normal' then
   m.affix=m.trigger=='equip' and 'prefix' or 'suffix';m.group=m.id;m.weight=100
   local first=m.tiers[1];for tier=2,3 do local at={};for key,value in pairs(first) do
    if key=='duration' then at[key]=math.floor(value*(1+.25*(tier-1)))
    elseif key=='ratio' then at[key]=1+(value-1)*(1+.25*(tier-1))
    else at[key]=value*(1+.25*(tier-1)) end
   end;m.tiers[tier]=at end
  else m.visual.priority=35;m.visual.strength=.6;if m.kind=='unique' then m.fixed_colour=colours[m.id] end end
 end

 -- Payoffs (a rule that reads a status or a stack it does not give) open at effective depth 5: a one-rule early drive is never a payoff
 -- with nothing to pay off (readability split R10). Trailing, the Haste echo, is set in mod_echo.lua.
 local payoffs={'pyre','cinder','shatter','brittle','malice','feasting','renewal','rush','crosswind','bastion'}
 for _,id in ipairs(payoffs) do for _,m in ipairs(pool) do if m.id==id then m.min_depth=5 end end end

 local declared={
 kindling={'sustain'},pyre={'launch_dealt'},burning={'conversion'},charged={'conversion'},pyromancer={'conversion','damage_taken'},
 feasting={'sustain'},updraft={'momentum'},crosswind={'momentum','speed','cleanse'},icebound={'speed'},brittle={'launch_taken'},
 reprisal={'damage_taken','launch_taken'},glass_core={'damage_dealt','damage_taken'},
 frosted={'conversion'},lingering={'status_duration'},
 cinder={'damage_dealt'},shatter={'damage_dealt'},ledge={'speed'},bastion={'damage_taken','launch_taken','momentum','cleanse'},renewal={'sustain'},
 rush={'momentum'},shelter={'damage_taken','launch_taken'},malice={'launch_taken'},ember_crown={'conversion','damage_taken'},
 winter_heart={'conversion','speed'},storm_shell={'conversion','damage_dealt'},mirror_shard={'clank'},frozen_oath={'conversion','damage_taken'},armoured={'damage_taken'},cleansing={'sustain','speed','launch_taken','cleanse'}}
 for _,m in ipairs(pool) do m.families=assert(declared[m.id]) end
 -- The rest of the keystones (about thirty in all) are data in keystones.lua; their family declarations are
 -- derived from the budget so they cannot drift. A pool built without that module keeps the four above.
 if D.keystones then for _,m in ipairs(D.keystones.records()) do m.families=D.mod_budget.effect_families(m);m.visual.priority=35;pool[#pool+1]=m end end
 -- Technique and crit modifiers for ordinary drives (mod_techniques.lua): they carry min_depth, so they never roll in the first stages.
 if D.mod_techniques then for _,m in ipairs(D.mod_techniques.records()) do m.families=D.mod_budget.effect_families(m);pool[#pool+1]=m end end
 for _,m in ipairs(D.mod_echo.records())do pool[#pool+1]=m end
 for _,m in ipairs(pool) do D.mod_schema.validate(m) end
 if D.mod_budget then D.mod_budget.validate_pool(pool,{}) end
 return pool
end
