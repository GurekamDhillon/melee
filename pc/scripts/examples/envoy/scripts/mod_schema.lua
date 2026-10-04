-- Only a shared vocabulary can express rules. No move IDs or modifier references.
return function()
 local S={}
 local function set(words) local out={};for w in words:gmatch('%S+') do out[w]=true end;return out end
 S.tags=set('jab tilt smash aerial special grab throw projectile dash_attack grounded airborne normal fire electric ice darkness burning shocked chilled cursed hasted guarded momentum damage healing unique keystone')
 S.events=set('equip hit_dealt hit_taken ko_dealt stock_lost shield_hit perfect_shield clank jump air_jump landing ledge_grab grab throw taunt item_pickup stage_start interval status_applied status_removed stacks_changed')
 S.statuses=set('burn chill curse haste guarded momentum') -- Shock's hitstun effect awaits safe hit mutation.
 S.values=set('damage_dealt damage_taken run_speed air_speed shield_max jump_height air_jump_height knockback_taken fall_speed weight shield_regen status_duration')
 S.status_bits={burn=1,chill=4,curse=8,haste=16,guarded=32,momentum=64}
 S.hit_moves=set('any unknown jab dash_attack tilt smash aerial special grab throw projectile')
 S.elements=set('normal fire electric ice darkness')
 local record=set('id label kind cost tags tiers trigger interval conditions effects stacking text visual affix group weight fixed_colour')
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
     keys(e.change,set('element damage knockback_growth knockback_base shield_damage hitstun'))
     if e.change.element then assert(S.elements[e.change.element],'special element conversion refused') end
    else
     assert(e.match.incoming or S.status_bits[e.status],'versus-status requires a shared status or incoming match')
     if e.status then assert(S.status_bits[e.status],'unknown status') end
     keys(e.change,set('damage knockback_taken'))
    end
    assert(next(e.change),'native change required')
    for k,v in pairs(e.change) do
     if k=='shield_damage' then range(v,m,0,50,true) elseif k=='hitstun' then range(v,m,0,30,true)
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
 function S.resolve(v,m,tier) if type(v)=='string' then return m.tiers[tier or 1][v:sub(2)] end;return v end
 function S.tooltip(m,tier)
  S.validate(m);local text=m.text:gsub('{([a-z_]+)(%%?)}',function(k,pct) local v=assert(m.tiers[tier or 1][k],'unknown tooltip tier');return pct=='%' and tostring(v*100)..'%' or tostring(v) end)
  return m.label..': '..text..(m.cost and ' Cost: '..m.cost or '')
 end
 return S
end
