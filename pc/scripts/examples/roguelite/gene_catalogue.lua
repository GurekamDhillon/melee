-- Gene family contract (Gate 5 planning spec). Machine-readable targets for the
-- finite gene catalogue: families, per-family genes, placements, bounded base
-- stats and an authored compatibility/reaction table. Only cinder and rime have
-- mechanics today (core.lua); the rest are `implemented = false` until Gate 5
-- codes and tests them. This module never claims a mechanic that does not exist.
local GeneCatalogue = {version = 1, placements = {'assault', 'traversal', 'guard'}}

GeneCatalogue.limits = {potency = {1, 30}, capacity = {1, 12}, gain = {0.1, 4}, reach = {1, 30}, cooldown = {1, 600}}

GeneCatalogue.families = {
  fire = {name = 'Ember', role = 'pressure/damage'},
  frost = {name = 'Rime', role = 'control/mark'},
  kinetic = {name = 'Gale', role = 'movement'},
  aegis = {name = 'Aegis', role = 'defensive/impact'},
  flux = {name = 'Flux', role = 'resource conversion'},
  sigil = {name = 'Sigil', role = 'mark/chain control'},
}

local function gene(id, family, name, placements, base, recipe, implemented)
  assert(GeneCatalogue.families[family], 'unknown gene family')
  return {id = id, family = family, name = name, placements = placements, base = base,
    recipe = recipe, implemented = implemented or false}
end

GeneCatalogue.genes = {
  cinder = gene('cinder', 'fire', 'Cinder Drive', {'assault', 'traversal', 'guard'}, {potency = 8, capacity = 3, gain = 1, reach = 9, cooldown = 90}, 'SolarEruption', true),
  emberline = gene('emberline', 'fire', 'Emberline', {'assault', 'traversal'}, {potency = 10, capacity = 2, gain = 1.2, reach = 6, cooldown = 120}, 'CometCrescent', false),
  rime = gene('rime', 'frost', 'Rime Guard', {'assault', 'traversal', 'guard'}, {potency = 8, capacity = 3, gain = 1, reach = 9, cooldown = 90}, 'GlacialShatter', true),
  glacier = gene('glacier', 'frost', 'Glacier Brand', {'assault', 'guard'}, {potency = 6, capacity = 4, gain = 0.8, reach = 12, cooldown = 110}, 'FrostCrown', false),
  gale = gene('gale', 'kinetic', 'Gale Step', {'traversal', 'assault'}, {potency = 3, capacity = 5, gain = 1.4, reach = 4, cooldown = 60}, 'WindWake', false),
  dashstep = gene('dashstep', 'kinetic', 'Dash Cadence', {'traversal', 'assault'}, {potency = 4, capacity = 4, gain = 1.1, reach = 5, cooldown = 75}, 'TracerLine', false),
  bulwark = gene('bulwark', 'aegis', 'Bulwark', {'guard', 'assault'}, {potency = 5, capacity = 6, gain = 0.9, reach = 7, cooldown = 80}, 'AegisHalo', false),
  retaliate = gene('retaliate', 'aegis', 'Retaliation', {'guard', 'assault'}, {potency = 9, capacity = 2, gain = 1, reach = 8, cooldown = 100}, 'ImpactRing', false),
  siphon = gene('siphon', 'flux', 'Siphon', {'assault', 'guard'}, {potency = 7, capacity = 3, gain = 1.3, reach = 8, cooldown = 95}, 'FluxOrbit', false),
  convert = gene('convert', 'flux', 'Conversion', {'guard', 'traversal'}, {potency = 2, capacity = 8, gain = 1.6, reach = 4, cooldown = 70}, 'ChargeLens', false),
  brand = gene('brand', 'sigil', 'Brand', {'assault', 'guard'}, {potency = 6, capacity = 3, gain = 1.1, reach = 14, cooldown = 85}, 'SigilMark', false),
  chain = gene('chain', 'sigil', 'Chain', {'assault', 'traversal'}, {potency = 8, capacity = 2, gain = 1, reach = 18, cooldown = 115}, 'ChainSpark', false),
}

-- Authored reaction table: family pairs only, not all combinations. precedence
-- is a stable tie-breaker; exclusive pairs cannot both own a persistent surface.
local function reaction(id, families, precedence, implemented)
  return {id = id, families = families, precedence = precedence, implemented = implemented or false}
end
GeneCatalogue.reactions = {
  thermal_shock = reaction('thermal_shock', {'fire', 'frost'}, 1, false),
  plasma_surge = reaction('plasma_surge', {'fire', 'flux'}, 2, false),
  charged_crystals = reaction('charged_crystals', {'frost', 'sigil'}, 3, false),
  chain_lightning = reaction('chain_lightning', {'kinetic', 'sigil'}, 4, false),
  aegis_crush = reaction('aegis_crush', {'aegis', 'kinetic'}, 5, false),
  flux_bloom = reaction('flux_bloom', {'flux', 'frost'}, 6, false),
}
GeneCatalogue.exclusive = {{'fire', 'frost'}, {'kinetic', 'aegis'}}

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end

function GeneCatalogue.validate(self)
  self = self or GeneCatalogue
  local families = 0
  for _ in pairs(self.families) do families = families + 1 end
  if families < 6 then return false, 'fewer than six families' end
  local genes, per_family = 0, {}
  local placement_set = {}
  for _, p in ipairs(self.placements) do placement_set[p] = true end
  for id, g in pairs(self.genes) do
    genes = genes + 1
    assert(g.id == id and self.families[g.family], 'invalid gene identity')
    per_family[g.family] = (per_family[g.family] or 0) + 1
    assert(type(g.placements) == 'table' and #g.placements >= 2, 'gene needs two placements')
    for _, p in ipairs(g.placements) do assert(placement_set[p], 'invalid placement') end
    assert(type(g.recipe) == 'string' and #g.recipe > 0, 'gene needs a recipe')
    assert(type(g.implemented) == 'boolean', 'gene implemented flag')
    assert(type(g.base) == 'table', 'gene needs base stats')
    local n = 0
    for stat, value in pairs(g.base) do
      n = n + 1
      local range = self.limits[stat]
      assert(range, 'unknown stat ' .. tostring(stat))
      assert(finite(value) and value >= range[1] and value <= range[2], 'stat out of range: ' .. stat)
    end
    assert(n == 5, 'gene needs the five bounded stats')
  end
  if genes < 12 then return false, 'fewer than twelve genes' end
  for family in pairs(self.families) do
    if not per_family[family] or per_family[family] < 2 then return false, 'family needs at least two genes: ' .. family end
  end
  local reactions = 0
  for id, r in pairs(self.reactions) do
    reactions = reactions + 1
    assert(r.id == id and type(r.families) == 'table' and #r.families == 2, 'invalid reaction')
    for _, f in ipairs(r.families) do assert(self.families[f], 'reaction references unknown family') end
    assert(finite(r.precedence), 'reaction precedence')
  end
  if reactions < 6 then return false, 'fewer than six reactions' end
  if reactions >= families * (families - 1) / 2 then return false, 'reaction table must not be exhaustive' end
  for _, pair in ipairs(self.exclusive) do
    assert(self.families[pair[1]] and self.families[pair[2]], 'exclusive pair references unknown family')
  end
  return true
end

return GeneCatalogue
