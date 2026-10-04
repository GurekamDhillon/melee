-- Pure rolls. Park-Miller arithmetic stays exact on Lua doubles and integers.
return function(D)
 local L={};L.__index=L
 L.colours={'red','green','blue','yellow','purple','white'}
 L.implicit_families=D.mod_budget.implicit_families
 L.implicits={red={damage_dealt=1.08},green={run_speed=1.08,air_speed=1.08},blue={knockback_taken=.92},yellow={jump_height=1.08},purple={status_duration=1.2},white={}}
 local function integer(n) return type(n)=='number' and n==math.floor(n) and n>=0 and n<=2147483646 end
 local function keys(t,allowed) for k in pairs(t) do assert(allowed[k],'unknown record field '..tostring(k)) end end
 function L.new(pool,config)
  D.mod_budget.validate_pool(pool,L.implicits);local self=setmetatable({pool=pool,rules={},normal={},uniques={},config=config or {}},L)
  assert(integer(self.config.tier_depth or 5) and (self.config.tier_depth or 5)>0,'invalid tier depth')
  for _,m in ipairs(pool) do
   assert(not self.rules[m.id],'duplicate modifier');self.rules[m.id]=m
   if m.kind=='normal' then
    assert(m.affix=='prefix' or m.affix=='suffix','affix required');assert(type(m.group)=='string' and #m.group>0 and type(m.weight)=='number' and m.weight>0 and m.weight<math.huge,'pool metadata required')
    self.normal[#self.normal+1]=m
   elseif m.kind=='unique' then self.uniques[#self.uniques+1]=m end
  end
  return self
 end
 local function rng(seed)
  local state=(seed*104729)%2147483646+1
  return function(n) state=(state*16807)%2147483647;return (state-1)/2147483646*n end
 end
 function L:roll(seed,depth,forced,loop)
  local context=D.mod_progression.context(depth,loop);depth=context.depth
  assert(integer(seed) and integer(depth),'invalid seed/depth');local rand=rng(seed)
  local rarity=forced and string.lower(forced)
  if not rarity then
   local weights=self.config.rarity_weights or {common=60,magic=28,rare=10,unique=2};local total=0
   for _,r in ipairs({'common','magic','rare','unique'}) do assert(type(weights[r])=='number' and weights[r]>=0 and weights[r]<math.huge,'invalid rarity weight');total=total+weights[r] end
   assert(total>0);local pick=rand(total)
   for _,r in ipairs({'common','magic','rare','unique'}) do pick=pick-weights[r];if pick<0 then rarity=r;break end end
  end
  assert(rarity=='common' or rarity=='magic' or rarity=='rare' or rarity=='unique','invalid rarity')
  local rec={seed=seed,depth=depth,loop=context.loop,colour=L.colours[math.floor(rand(6))+1],rarity=rarity,affixes={}}
  if rarity=='unique' then
   assert(#self.uniques>0,'no uniques');local total=0;for _,m in ipairs(self.uniques)do local w=(self.config.unique_weights or {})[m.id] or 1;assert(type(w)=='number' and w==w and w>0 and w<1e9,'invalid unique weight');total=total+w end
   local pick=rand(total);local m;for _,v in ipairs(self.uniques)do pick=pick-((self.config.unique_weights or {})[v.id] or 1);if pick<0 then m=v;break end end
   rec.unique=m.id;rec.colour=m.fixed_colour or 'white';rec.affixes={{id=m.id,tier=D.mod_progression.tier(context)}}
  else
   local groups={};local floor=1+math.floor(D.mod_progression.effective(context)/(self.config.tier_depth or 5))
   local function affix(kind)
    local choices,total={},0
    for _,m in ipairs(self.normal) do if not groups[m.group] and (not kind or m.affix==kind) then choices[#choices+1]=m;local w=m.weight*((self.config.affix_weights or {})[m.id] or 1);assert(type(w)=='number' and w==w and w>0 and w<1e9,'invalid affix weight');total=total+w end end
    assert(total>0,'pool exhausted');local pick=rand(total);local chosen
    for _,m in ipairs(choices) do pick=pick-m.weight*((self.config.affix_weights or {})[m.id] or 1);if pick<0 then chosen=m;break end end
    groups[chosen.group]=true;local tier=floor
    rec.affixes[#rec.affixes+1]={id=chosen.id,tier=tier}
   end
   local count=rarity=='rare' and 2 or rarity=='magic' and 1 or 0
   for _=1,count do affix('prefix');affix('suffix') end
   if rec.colour=='white' then affix() end
  end
  self:validate(rec);return rec
 end
 function L:validate(r)
  assert(type(r)=='table');keys(r,{seed=true,depth=true,colour=true,rarity=true,affixes=true,unique=true,loop=true})
  local context=D.mod_progression.context(r.depth,r.loop);assert(integer(r.seed),'invalid seed');assert(L.implicits[r.colour],'invalid colour')
  assert(type(r.affixes)=='table');local groups={};local prefix,suffix,n=0,0,0
  for k,a in pairs(r.affixes) do
   assert(type(k)=='number' and k==math.floor(k) and k>=1 and k<=#r.affixes,'invalid affix array')
   assert(type(a)=='table');keys(a,{id=true,tier=true});local m=assert(self.rules[a.id],'unknown affix')
   D.mod_schema.level(a.tier)
   if m.kind=='normal' then assert(a.tier==(r.loop==nil and math.min(#m.tiers,3,1+math.floor(r.depth/(self.config.tier_depth or 5))) or 1+math.floor(D.mod_progression.effective(context)/(self.config.tier_depth or 5))),'tier does not match depth') end
   assert(m.kind=='normal' or (r.rarity=='unique' and m.id==r.unique and m.kind=='unique'),'non-loot affix')
   local group=m.group or m.id;assert(not groups[group],'duplicate group');groups[group]=true;n=n+1
   if m.affix=='prefix' then prefix=prefix+1 elseif m.affix=='suffix' then suffix=suffix+1 end
  end
  assert(n==#r.affixes,'sparse affix array');for i=1,n do assert(r.affixes[i]~=nil,'sparse affix array') end
  if r.rarity=='unique' then
   local m=assert(self.rules[r.unique],'unknown unique');assert(m.kind=='unique' and n==1 and r.affixes[1].id==m.id and r.affixes[1].tier==(r.loop==nil and 1 or D.mod_progression.tier(context)) and r.colour==(m.fixed_colour or 'white'),'invalid fixed unique')
  else
   assert(r.unique==nil,'unexpected unique');local count=({common=0,magic=1,rare=2})[r.rarity];assert(count,'invalid rarity')
   local extra=r.colour=='white' and 1 or 0;assert(n==count*2+extra and prefix>=count and suffix>=count,'invalid affix counts')
  end
  return true
 end
 function L:name(r)
  self:validate(r);if r.unique then return self.rules[r.unique].label..' Drive' end
  local before,after={},{};for _,a in ipairs(r.affixes) do local m=self.rules[a.id];local t=m.affix=='prefix' and before or after;t[#t+1]=m.label end
  local base=r.colour:sub(1,1):upper()..r.colour:sub(2)..' Drive'
  return (#before>0 and table.concat(before,' ')..' ' or '')..base..(#after>0 and ' of the '..table.concat(after,' and ') or '')
 end
 function L:tooltip(r)
  self:validate(r);local out={};local implicit={red='+8% attack percent damage; launch unchanged',green='+8% run and air speed',blue='-8% launch taken',yellow='+8% jump height',purple='+20% status duration',white='One extra modifier; no base implicit'}
  out[1]=r.unique and r.colour=='white' and 'No base implicit; fixed unique rules' or implicit[r.colour]
  for _,a in ipairs(r.affixes) do local m=self.rules[a.id];local tier=a.tier
   out[#out+1]=D.mod_schema.describe(m,tier)
   if m.cost then out[#out+1]=m.cost end
  end
  return out
 end
 return L
end
