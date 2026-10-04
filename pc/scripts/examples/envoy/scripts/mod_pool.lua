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
  rule('kindling','Kindling',{'fire','burning'},'hit_dealt',{{tag='fire'}},{status('burn','target','$duration','$damage')},{duration=180,damage=3},'Fire hits apply Burn for {duration} frames: {damage} damage each second.','burn',.03),
  rule('pyre','Pyre',{'burning','damage'},'equip',{},{{op='versus-status',status='burn',match={},change={knockback_taken='$ratio'}}},{ratio=1.25,bonus=.25},'Your connecting hits launch Burning targets +{bonus%} harder.','curse',.76),
  rule('burning','Burning',{'smash','fire'},'equip',{},{{op='convert',match={move='smash'},change={element='fire'}}},{},'Your ordinary smash hitboxes become Fire at creation.','burn',.08),
  rule('charged','Charged',{'aerial','electric'},'equip',{},{{op='convert',match={move='aerial'},change={element='electric'}}},{},'Your ordinary aerial hitboxes become Electric at creation.','shock',.57),
  rule('pyromancer','Pyromancer',{'keystone','fire','ice'},'equip',{},{{op='convert',match={move='any'},change={element='fire'}},{op='versus-status',match={incoming=true,element='ice'},change={damage=2}}},{},'Your ordinary owned attack hitboxes become Fire.','burn',.01,'keystone','Hits with an original Ice element deal double damage to you.'),
  rule('feasting','Feasting',{'healing'},'ko_dealt',{{target_status='any'}},{{op='heal',amount='$heal',subject='self'}},{heal=10},'KO a status-affected target: heal {heal} damage points.','guarded',.46),
  rule('updraft','Updraft',{'aerial','momentum'},'hit_dealt',{{tag='aerial'}},{status('momentum','self','$duration',1,5)},{duration=300},'Aerial hits grant Momentum (max 5) for {duration} frames.','momentum',.14),
  rule('crosswind','Crosswind',{'momentum','hasted'},'landing',{{self_status='momentum'}},{{op='remove_status',status='momentum',subject='self'},status('haste','self','$duration')},{duration=180},'Land with Momentum: spend it to gain Haste (+20% run/air speed) for {duration} frames.','haste',.34),
  rule('icebound','Icebound',{'ice','chilled'},'hit_dealt',{{tag='ice'}},{status('chill','target','$duration')},{duration=180},'Ice hits apply Chill (-20% run/air speed) for {duration} frames.','chill',.54),
  rule('brittle','Brittle',{'chilled','cursed'},'hit_dealt',{{target_status='chill'}},{status('curse','target','$duration','$bonus')},{duration=120,bonus=.25},'Hit a Chilled target: Curse adds {bonus%} launch impulse to later hits for {duration} frames.','curse',.73),
  rule('reprisal','Reprisal',{'guarded','momentum'},'perfect_shield',{}, {status('guarded','self','$duration'),status('momentum','self','$duration',1,5)},{duration=180},'Perfect shield: Guarded (-25% damage, -15% launch impulse) and +1 Momentum for {duration} frames.','guarded',.59),
  rule('glass_core','Glass Core',{'unique','damage'},'equip',{},{{op='value',key='damage_dealt',value=2},{op='value',key='damage_taken',value=2}},{},'Double attack damage dealt and taken.','shock',.89,'unique','You also take double attack damage.'),
  rule('still_heart','Still Heart',{'keystone','hasted','guarded'},'status_applied',{{status='haste'}},{{op='remove_status',status='haste',subject='self'},status('guarded','self','$duration')},{duration=180},'Your Haste becomes Guarded for {duration} frames.','guarded',.62,'keystone','You lose Haste movement bonuses.')
 }
 for _,m in ipairs(pool) do D.mod_schema.validate(m) end
 return pool
end
