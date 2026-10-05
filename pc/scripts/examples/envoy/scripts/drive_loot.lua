-- Pure rolls. Park-Miller arithmetic stays exact on Lua doubles and integers.
return function(D)
 local L={};L.__index=L
 L.colours={'red','green','blue','yellow','purple','white'}
 L.max_merges=3
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
   local weights=self.config.rarity_weights or (self.config.ungated and {common=60,magic=28,rare=10,unique=2}) or D.mod_progression.rarity_weights(context);local total=0
   -- Natural rarity weights follow depth (mod_progression.rarity_weights: common only to depth 2, magic from 3,
   -- rare from 6, unique from 10); a forced rarity (a boss reward) is not gated, but its affix count still is.
   local function open(r) return true end
   for _,r in ipairs({'common','magic','rare','unique'}) do assert(type(weights[r])=='number' and weights[r]>=0 and weights[r]<math.huge,'invalid rarity weight');if open(r) then total=total+weights[r] end end
   assert(total>0);local pick=rand(total)
   for _,r in ipairs({'common','magic','rare','unique'}) do if open(r) then pick=pick-weights[r];if pick<0 then rarity=r;break end end end
  end
  assert(rarity=='common' or rarity=='magic' or rarity=='rare' or rarity=='unique','invalid rarity')
  local rec={seed=seed,depth=depth,loop=context.loop,colour=L.colours[math.floor(rand(6))+1],rarity=rarity,affixes={}}
  if rarity=='unique' then
   assert(#self.uniques>0,'no uniques');local total=0;for _,m in ipairs(self.uniques)do local w=(self.config.unique_weights or {})[m.id] or 1;assert(type(w)=='number' and w==w and w>0 and w<1e9,'invalid unique weight');total=total+w end
   local pick=rand(total);local m;for _,v in ipairs(self.uniques)do pick=pick-((self.config.unique_weights or {})[v.id] or 1);if pick<0 then m=v;break end end
   rec.unique=m.id;rec.colour=m.fixed_colour or 'white';rec.affixes={{id=m.id,tier=D.mod_progression.tier(context)}}
  else
   local redraw;local groups={};local floor=1+math.floor(D.mod_progression.effective(context)/(self.config.tier_depth or 5))
   local function affix(kind)
    local choices,total={},0
    for _,m in ipairs(self.normal) do if not groups[m.group] and (not kind or m.affix==kind) then choices[#choices+1]=m;local w=m.weight*((self.config.affix_weights or {})[m.id] or 1);assert(type(w)=='number' and w==w and w>0 and w<1e9,'invalid affix weight');total=total+w end end
    assert(total>0,'pool exhausted')
    -- A record with a minimum depth (the technique modifiers) is drawn from the same weighted list as every other. While it is
    -- still too early for it, the pick is redrawn from the records that are open, using a stream of its own (the seed's main
    -- stream is untouched), so a seed gives the same picks at every depth except where it picked a record not yet open.
    local effective=D.mod_progression.effective(context);local chosen
    do
     local pick=rand(total)
     for _,m in ipairs(choices) do pick=pick-m.weight*((self.config.affix_weights or {})[m.id] or 1);if pick<0 then chosen=m;break end end
     if chosen and chosen.min_depth and effective<chosen.min_depth then
      local open,sum={},0;for _,m in ipairs(choices) do if not m.min_depth or effective>=m.min_depth then open[#open+1]=m;sum=sum+m.weight*((self.config.affix_weights or {})[m.id] or 1) end end
      assert(sum>0,'pool exhausted');redraw=redraw or rng(seed+7919);local p2=redraw(sum);chosen=nil
      for _,m in ipairs(open) do p2=p2-m.weight*((self.config.affix_weights or {})[m.id] or 1);if p2<0 then chosen=m;break end end
      chosen=chosen or open[#open]
     end
    end
    assert(chosen,'pool exhausted');groups[chosen.group]=true;local tier=floor
    rec.affixes[#rec.affixes+1]={id=chosen.id,tier=tier}
   end
   -- Affix count is a function of rarity AND depth (mod_progression.affix_count); prefixes and suffixes alternate,
   -- starting from a seeded kind, so one affix is one plain effect and no chain.
   local base=D.mod_progression.affix_count(context,rarity,false)
   local first=rand(2)<1 and 'prefix' or 'suffix'
   for i=1,base do local kind=((i%2==1)==(first=='prefix')) and 'prefix' or 'suffix';affix(kind) end
   if rec.colour=='white' and D.mod_progression.affix_count(context,rarity,true)>base then affix() end
  end
  self:validate(rec);return rec
 end
 function L:validate(r)
  assert(type(r)=='table');keys(r,{seed=true,depth=true,colour=true,rarity=true,affixes=true,unique=true,loop=true,merged=true})
  local merged=r.merged or 0;assert(integer(merged) and merged<=L.max_merges,'invalid merge count')
  local context=D.mod_progression.context(r.depth,r.loop);assert(integer(r.seed),'invalid seed');assert(L.implicits[r.colour],'invalid colour')
  assert(type(r.affixes)=='table');local groups={};local prefix,suffix,n=0,0,0
  for k,a in pairs(r.affixes) do
   assert(type(k)=='number' and k==math.floor(k) and k>=1 and k<=#r.affixes,'invalid affix array')
   assert(type(a)=='table');keys(a,{id=true,tier=true});local m=assert(self.rules[a.id],'unknown affix')
   D.mod_schema.level(a.tier)
   if m.kind=='normal' then local base=r.loop==nil and math.min(#m.tiers,3,1+math.floor(r.depth/(self.config.tier_depth or 5))) or 1+math.floor(D.mod_progression.effective(context)/(self.config.tier_depth or 5))
    -- A merge (drive_merge.lua) may lift a modifier above its depth tier, at most one tier per merge.
    assert(a.tier>=base and a.tier<=base+merged,'tier does not match depth') end
   assert(m.kind=='normal' or (r.rarity=='unique' and m.id==r.unique and m.kind=='unique'),'non-loot affix')
   assert(not m.min_depth or D.mod_progression.effective(context)>=m.min_depth,'modifier is not available at this depth')
   local group=m.group or m.id;assert(not groups[group],'duplicate group');groups[group]=true;n=n+1
   if m.affix=='prefix' then prefix=prefix+1 elseif m.affix=='suffix' then suffix=suffix+1 end
  end
  assert(n==#r.affixes,'sparse affix array');for i=1,n do assert(r.affixes[i]~=nil,'sparse affix array') end
  if r.rarity=='unique' then
   local m=assert(self.rules[r.unique],'unknown unique');assert(m.kind=='unique' and n==1 and r.affixes[1].id==m.id and r.affixes[1].tier==(r.loop==nil and 1 or D.mod_progression.tier(context)) and r.colour==(m.fixed_colour or 'white'),'invalid fixed unique')
  else
   assert(r.unique==nil,'unexpected unique');local old=({common=0,magic=2,rare=4})[r.rarity];assert(old,'invalid rarity')
   -- The current curve: 1..affix_count for this rarity and depth (a merge adds no modifier, so a merged record
   -- never exceeds it). A pre-curve record (an old save) keeps its old fixed shape and is still accepted.
   local ceiling=D.mod_progression.affix_count(context,r.rarity,r.colour=='white')
   old=old+(r.colour=='white' and 1 or 0)
   assert((n>=1 and n<=ceiling) or (not self.config.strict_counts and n==old),'invalid affix counts')
  end
  return true
 end
 function L:name(r)
  self:validate(r);if r.unique then return self.rules[r.unique].label..' Drive' end
  local before,after={},{};for _,a in ipairs(r.affixes) do local m=self.rules[a.id];local t=m.affix=='prefix' and before or after;t[#t+1]=m.label end
  local base=r.colour:sub(1,1):upper()..r.colour:sub(2)..' Drive'
  -- One modifier reads as one short line, never a prefix and suffix chain.
  if #r.affixes==1 then return base..': '..self.rules[r.affixes[1].id].label end
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
