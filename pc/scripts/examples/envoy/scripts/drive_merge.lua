-- Merging instead of hoarding: a gained drive that matches one the player already holds improves the held drive
-- and is consumed, so it never becomes another inventory item. Pure functions of two loot records and the loot
-- module; the run host decides when to call them (it is not wired in here).
--
-- WHAT "MATCHES" MEANS (all of these must hold):
--   1. neither is a Unique (a Unique is a fixed rule; two of the same already stack as copies);
--   2. COMPARABLE: they share a modifier id (exact), or one of the gained drive's modifiers touches a budget
--      family one of the held drive's modifiers touches (for example two of the "damage taken" modifiers);
--   3. a SHARED RULE merges across colours (a second copy of a rule you already hold is never inert, whatever its colour: the
--      readability split, since a duplicate rule on a second drive was ignored by the build); a merge on a shared budget FAMILY
--      alone still needs the same colour (the base type; White matches White).
--   And the held drive has merges left (Loot.max_merges = 3 per drive).
--
-- WHAT A MERGE DOES: exactly one of the held drive's modifiers goes up one tier (the exact-id one if there is one,
-- else the first comparable one), the held drive records one merge, and if the gained drive was rolled deeper the
-- held drive is re-based to that depth (its modifiers take that depth's tier, as if rolled there). It NEVER adds a
-- modifier, never changes colour, rarity, or affix count: an early one-modifier drive stays a one-modifier drive
-- however many drives are merged into it, so merging cannot skip the affix-count curve. (drive_loot validates the
-- result: a tier above its depth is legal only as far as the record's merge count allows.)
return function(D)
 local M={}
 local function copy(v) if type(v)~='table' then return v end;local t={};for k,x in pairs(v) do t[k]=copy(x) end;return t end
 local function effective(r) return D.mod_progression.effective(D.mod_progression.context(r.depth,r.loop)) end
 local function base_tier(loot,rule,r)
  local td=loot.config.tier_depth or 5
  if r.loop==nil then return math.min(#rule.tiers,3,1+math.floor(r.depth/td)) end
  return 1+math.floor(effective(r)/td)
 end
 local function families(rule) local set={};for _,f in ipairs(rule.families or {}) do set[f]=true end;return set end
 -- Which held affix a gained drive would lift, or nil. Returns the affix index and whether the id was exact.
 function M.target_affix(held,gained,loot)
  for i,a in ipairs(held.affixes) do for _,b in ipairs(gained.affixes) do if b.id==a.id then return i,true end end end
  for i,a in ipairs(held.affixes) do
   local fa=families(loot.rules[a.id])
   for _,b in ipairs(gained.affixes) do for _,f in ipairs(loot.rules[b.id].families or {}) do if fa[f] then return i,false end end end
  end
 end
 function M.can_merge(held,gained,loot)
  if type(held)~='table' or type(gained)~='table' then return false,'not drives' end
  if held.unique or gained.unique or held.rarity=='unique' or gained.rarity=='unique' then return false,'A unique drive does not merge.' end
  if (held.merged or 0)>=loot.max_merges then return false,'This drive is fully merged.' end
  local at,exact=M.target_affix(held,gained,loot)
  if not at then return false,held.colour~=gained.colour and 'Different colours do not merge.' or 'Nothing in common to improve.' end
  if held.colour~=gained.colour and not exact then return false,'Different colours do not merge.' end
  return true
 end
 -- Returns the improved record (a new table; `held` is not touched) and what changed, or nil and why.
 function M.merge(held,gained,loot)
  local ok,why=M.can_merge(held,gained,loot);if not ok then return nil,why end
  local at=M.target_affix(held,gained,loot)
  local out=copy(held)
  -- re-base to the deeper roll
  local rebased=false
  if effective(gained)>effective(held) then out.depth,out.loop=gained.depth,gained.loop;rebased=true end
   for i,a in ipairs(out.affixes) do
   local rule=loot.rules[a.id]
   local bonus=held.affixes[i].tier-base_tier(loot,rule,held)
   a.tier=base_tier(loot,rule,out)+math.max(0,bonus)
  end
  local from=out.affixes[at].tier;out.affixes[at].tier=from+1;out.merged=(held.merged or 0)+1
  local valid,err=pcall(function() loot:validate(out) end)
  if not valid then return nil,'This merge is not allowed: '..tostring(err) end
  return out,{affix=out.affixes[at].id,from=from,to=from+1,rebased=rebased,merges=out.merged}
 end
 -- The best held drive for a gained one: most shared modifiers first, then fewest merges, then list order. `list` is
 -- the player's drives in the order the host prefers (equipped first, then the bag). Returns the index or nil.
 function M.find_target(list,gained,loot)
  local pick,score
  for i,h in ipairs(list) do
   if M.can_merge(h,gained,loot) then
    local shared=0;for _,a in ipairs(h.affixes) do for _,b in ipairs(gained.affixes) do if a.id==b.id then shared=shared+1 end end end
    local s=shared*10-(h.merged or 0)
    if not score or s>score then pick,score=i,s end
   end
  end
  return pick
 end
 return M
end
