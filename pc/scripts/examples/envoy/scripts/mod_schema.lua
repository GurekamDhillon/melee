-- Only a shared vocabulary can express rules. No move IDs or modifier references.
return function(D)
 local S={}
 local function set(words) local out={};for w in words:gmatch('%S+') do out[w]=true end;return out end
 S.tags=set('jab tilt smash aerial special grab throw projectile dash_attack grounded airborne normal fire electric ice darkness burning shocked chilled cursed hasted guarded momentum damage healing unique keystone')
 S.events=set('equip hit_dealt hit_taken ko_dealt stock_lost shield_hit perfect_shield clank jump air_jump landing ledge_grab grab throw taunt item_pickup stage_start interval status_applied status_removed stacks_changed')
 S.statuses=set('burn chill curse haste guarded momentum') -- Shock's hitstun effect awaits safe hit mutation.
 S.values=set('damage_dealt damage_taken run_speed air_speed shield_max jump_height air_jump_height knockback_taken fall_speed weight shield_regen status_duration')
 S.status_bits={burn=1,chill=4,curse=8,haste=16,guarded=32,momentum=64}
 S.hit_moves=set('any unknown jab dash_attack tilt smash aerial special grab throw projectile')
 S.elements=set('normal fire electric ice darkness')
 local record=set('id label kind cost tags tiers trigger interval conditions effects stacking text visual affix group weight fixed_colour families')
 local cond=set('tag self_status target_status status self_damage_above self_damage_below target_damage_above grounded airborne last_stock recently stage_kind')
 local fields={status=set('op status subject duration amount max refresh when'),value=set('op key value when'),heal=set('op amount subject when'),damage=set('op amount subject when'),stacks=set('op status subject duration amount max refresh when'),remove_status=set('op status subject when'),emit=set('op event subject tag when')}
 fields.clank_damage=set('op subject')
 fields.convert=set('op match change');fields['versus-status']=set('op status match change')
 local function keys(t,allowed) assert(type(t)=='table' and not getmetatable(t),'plain record required');for k in pairs(t) do assert(allowed[k],'unsupported record field '..tostring(k)) end end
 local function number(v,m)
  if type(v)=='string' then assert(v:match('^%$[a-z_]+$'),'only tier references allowed');for _,tier in ipairs(m.tiers) do assert(type(tier[v:sub(2)])=='number','missing tier value') end
  else assert(type(v)=='number' and v==v and math.abs(v)<=100000,'finite bounded value required') end
 end
 local function array(t,minimum,maximum)
  assert(type(t)=='table' and not getmetatable(t),'dense array required');local n=0
  for k in pairs(t) do assert(type(k)=='number' and k%1==0 and k>=1,'dense array required');n=n+1 end
  assert(n==#t and n>=minimum and n<=maximum,'array size out of bounds');for i=1,n do assert(t[i]~=nil,'dense array required') end
 end
 local function range(v,m,lo,hi,integer)
  number(v,m);for i=1,#m.tiers do local x=type(v)=='string' and m.tiers[i][v:sub(2)] or v
   assert(x>=lo and x<=hi and (not integer or x%1==0),'parameter out of bounds') end
 end
 function S.validate(m)
  keys(m,record);assert(type(m.id)=='string' and m.id:match('^[a-z][a-z_]+$') and #m.id<=40,'invalid modifier id')
  assert(type(m.label)=='string' and #m.label<=50 and type(m.text)=='string','label/text required')
  assert(m.kind=='normal' or m.kind=='unique' or m.kind=='keystone','invalid rarity')
  if m.kind~='normal' then assert(type(m.cost)=='string' and #m.cost>0,'rule-breaker needs explicit cost') end
  if m.affix then assert(m.kind=='normal' and (m.affix=='prefix' or m.affix=='suffix'),'invalid affix');assert(type(m.group)=='string' and #m.group>0 and type(m.weight)=='number' and m.weight>0 and m.weight<100000,'invalid loot metadata') end
  if m.fixed_colour then assert(m.kind=='unique' and set('red green blue yellow purple white')[m.fixed_colour],'invalid unique colour') end
  array(m.tags,0,8);for _,tag in ipairs(m.tags) do assert(S.tags[tag],'tag must be vocabulary, never a move/modifier name') end
  array(m.tiers,1,5)
  for _,tier in ipairs(m.tiers) do assert(type(tier)=='table');for k,v in pairs(tier) do assert(type(k)=='string' and k:match('^[a-z_]+$'));number(v,m);assert(type(v)=='number','numeric tier required') end end
  assert(S.events[m.trigger],'unsupported trigger')
  if m.trigger=='interval' then range(m.interval,m,1,3600,true) else assert(m.interval==nil,'interval requires interval trigger') end
  array(m.conditions or {},0,8)
  if m.trigger=='equip' then assert(#(m.conditions or {})==0,'passive equip conditions unsupported') end
  for _,c in ipairs(m.conditions or {}) do keys(c,cond);for k,v in pairs(c) do
   if k=='tag' then assert(S.tags[v],'condition tag must be vocabulary')
   elseif k=='self_status' or k=='target_status' or k=='status' then assert(v=='any' or S.statuses[v],'unsupported status condition')
   elseif k=='recently' then keys(v,set('event frames'));assert(S.events[v.event]);range(v.frames,m,1,3600,true)
   elseif k=='stage_kind' then assert(set('battle team giant metal bonus boss')[v],'unsupported stage kind')
   elseif k=='grounded' or k=='airborne' or k=='last_stock' then assert(type(v)=='boolean') else number(v,m) end
  end end
  array(m.effects,1,8)
  for _,e in ipairs(m.effects) do assert(fields[e.op],'unsupported effect (no connecting-hit mutation)');keys(e,fields[e.op])
   if e.subject then assert(e.subject=='self' or e.subject=='target','unsupported effect subject') end
   if e.when then assert(e.when==m.trigger,'effect event must match trigger') end
   if e.op=='convert' or e.op=='versus-status' then
    assert(m.trigger=='equip','native hit rules require unconditional equip')
    keys(e.match,set('move grounded element incoming'));assert(e.match.move==nil or S.hit_moves[e.match.move],'hit match must use vocabulary')
    assert(e.match.element==nil or S.elements[e.match.element],'only ordinary original elements supported')
    if e.match.grounded~=nil then assert(type(e.match.grounded)=='boolean') end
    if e.match.incoming~=nil then assert(type(e.match.incoming)=='boolean') end
    if e.op=='convert' then
     assert(not e.match.incoming,'incoming rules cannot modify creation')
     keys(e.change,set('element percent_damage launch damage knockback_growth knockback_base shield_damage hitstun'))
     if e.change.element then assert(S.elements[e.change.element],'special element conversion refused') end
    else
     assert(e.match.incoming or e.status==nil or S.status_bits[e.status],'unknown native status')
     if e.status then assert(S.status_bits[e.status],'unknown status') end
     keys(e.change,set('percent_damage launch damage knockback_taken'))
    end
    assert(next(e.change),'native change required')
    for k,v in pairs(e.change) do
     if k=='shield_damage' then range(v,m,0,50,true) elseif k=='hitstun' then range(v,m,0,30,true)
     elseif k=='percent_damage' then range(v,m,-1e9,1e9)
     elseif k=='launch' then range(v,m,-1e9,1e9)
     elseif k~='element' then range(v,m,.1,4) end
    end
   elseif e.op=='value' then assert(m.trigger=='equip' and S.values[e.key],'fighter values require unconditional equip');range(e.value,m,.1,4)
   elseif e.op=='status' or e.op=='stacks' then assert(m.trigger~='equip' and S.statuses[e.status],'unsupported status');range(e.duration,m,1,3600,true);range(e.amount or 1,m,0,100);assert(e.refresh=='refresh' or e.refresh=='extend' or e.refresh=='keep');assert(type(e.max)=='number' and e.max%1==0 and e.max>=1 and e.max<=8)
   elseif e.op=='clank_damage' then assert(m.trigger=='clank' and e.subject=='target','clank damage requires opposing fighter')
   elseif e.op=='remove_status' then assert(S.statuses[e.status])
   elseif e.op=='emit' then assert(S.events[e.event] and S.tags[e.tag]) else range(e.amount,m,0,100) end
  end
  keys(m.stacking,set('max'));assert(m.stacking.max==1,'step1 equips each rule once')
  keys(m.visual,set('look hue strength priority'));assert(set('burn shock chill curse haste guarded momentum')[m.visual.look]);assert(type(m.visual.hue)=='number' and m.visual.hue>=0 and m.visual.hue<=1 and type(m.visual.strength)=='number' and m.visual.strength>=0 and m.visual.strength<=1)
  if m.visual.priority then assert(type(m.visual.priority)=='number' and m.visual.priority%1==0 and m.visual.priority>=0 and m.visual.priority<=100) end
  for key in m.text:gmatch('{([a-z_]+)%%?}') do for _,tier in ipairs(m.tiers) do assert(type(tier[key])=='number','unknown tooltip tier') end end
  return true
 end
 function S.level(tier)
  if type(tier)=='table' then
   assert(not getmetatable(tier),'plain modifier stack required');for k in pairs(tier)do assert(k=='tier' or k=='copies' or k=='tiers','unknown modifier stack field')end
   if tier.tiers then
    array(tier.tiers,1,6);local highest=0
    for _,t in ipairs(tier.tiers)do D.mod_progression.growth(t);highest=math.max(highest,t)end
    assert(tier.tier==nil or tier.tier==highest,'stack display tier mismatch')
    assert(tier.copies==nil or tier.copies==#tier.tiers,'stack copy count mismatch');return highest
   end
   assert(type(tier.copies)=='number' and tier.copies%1==0 and tier.copies>=1 and tier.copies<=6,'invalid modifier copies');tier=tier.tier
  end
  tier=tier or 1;D.mod_progression.growth(tier);return tier
 end
 function S.copies(tier) S.level(tier);return type(tier)=='table' and (tier.tiers and #tier.tiers or tier.copies) or 1 end
 function S.instances(tier)
  local highest=S.level(tier);local out={}
  if type(tier)=='table' and tier.tiers then for _,t in ipairs(tier.tiers)do out[#out+1]=t end
  else for _=1,S.copies(tier)do out[#out+1]=highest end end
  return out
 end
 function S.resolve(v,m,tier)
  tier=S.level(tier)
  if type(v)~='string' then return v end
  local key=v:sub(2);if m.tiers[tier] then return m.tiers[tier][key]end
  local base=assert(m.tiers[1][key],'missing tier value');local growth=D.mod_progression.growth(tier)
  if key=='ratio' then return 1+(base-1)*growth end
  return base*(key=='bonus' and D.mod_progression.launch_growth(tier) or growth)
 end
 function S.ratio(v,m,tier,family)
  local delta=0
  for _,t in ipairs(S.instances(tier))do
   local n=S.resolve(v,m,t)
   -- Costs retain their authored size; each physical unique keeps its own growth.
   if type(v)=='number' and m.kind~='normal' and ((family=='damage_dealt' and n>1) or (family=='damage_taken' and n<1)) then n=1+(n-1)*D.mod_progression.growth(t)end
   if (family=='launch_taken' or family=='launch_dealt') and type(v)=='string' then n=1+(n-1)*D.mod_progression.launch_growth(t)/D.mod_progression.growth(t)end
   delta=delta+n-1
  end
  local n=1+delta;assert(n==n and math.abs(n)<=1e9,'native contribution exceeds finite safety');return n
 end
 function S.describe(m,tier)
  S.validate(m);local text=m.text:gsub('{([a-z_]+)(%%?)}',function(k,pct) local v=S.resolve('$'..k,m,tier);return pct=='%' and tostring(v*100)..'%' or tostring(v) end)
  if m.kind=='unique' then
   local lines={};local changed=false
   local labels={damage_dealt='Attack percent damage',damage_taken='Attack percent damage taken',run_speed='Run speed',air_speed='Air speed'}
   for _,e in ipairs(m.effects)do
    if e.op=='value' then
     changed=true;local n=S.ratio(e.value,m,tier,D.mod_budget.families[e.key])-1
     if e.key=='damage_taken' and n<0 then lines[#lines+1]='Resistance +'..string.format('%.1f',-n*100)..'%; combined damage taken has a 15% floor.'
     else lines[#lines+1]=(labels[e.key] or e.key)..' '..string.format('%+.1f%%',n*100)..'.'end
    elseif e.op=='convert' and e.change.element then local name=e.change.element;lines[#lines+1]='Ordinary owned hitboxes become '..name:sub(1,1):upper()..name:sub(2)..'.'end
   end
   if changed then text='Tier '..S.level(tier)..': '..table.concat(lines,' ')end
  end
  return text
 end
 function S.tooltip(m,tier) return m.label..': '..S.describe(m,tier)..(m.cost and ' Cost: '..m.cost or '')end

 return S
end
