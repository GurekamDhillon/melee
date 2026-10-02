-- Authored behavior specifications for the finite gene catalogue (Gate 5).
-- Pure data and validation only: no engine access and no Core mutation. Core's
-- own definitions remain authoritative for the offered cinder/rime actions;
-- from_core() cross-checks this table against them. Every other family is data
-- for admission review and is unoffered until a native action seam exists.
--
-- A behavior record is (gene, placement) -> spec. The spec names a mechanic
-- route (offense/control/movement/defense/conversion) and an effect kind the
-- action transaction in gene_actions.lua can resolve. This module does not
-- claim those prototype mechanics run natively; see docs/GENE-ACTION-CONTRACT.md.
local B = {version = 1}

B.offered_genes = {cinder = true, rime = true}

B.bounds = {
  startup = {0, 120}, active = {1, 240}, recovery = {0, 120},
  range = {0, 30}, height = {0, 24}, duration = {1, 600},
  amount = {0, 30}, ratio = {0, 2}, distance = {0, 10}, impulse = {0, 10}, links = {1, 4}, pierce = {0, 3},
  ['repeat'] = {1, 4}, window = {1, 60},
}

-- Family roles. `primary` is the mechanic a family must express at least once;
-- placements may differ so one gene can serve two roles without re-colouring.
B.families = {
  fire = {name = 'Ember', role = 'pressure/damage', primary = 'damage'},
  frost = {name = 'Rime', role = 'control/mark', primary = 'control'},
  kinetic = {name = 'Gale', role = 'movement', primary = 'movement'},
  aegis = {name = 'Aegis', role = 'defensive/impact', primary = 'defense'},
  flux = {name = 'Flux', role = 'resource conversion', primary = 'conversion'},
  sigil = {name = 'Sigil', role = 'mark/chain control', primary = 'chain'},
}

-- Mechanic -> transaction route and whether it selects an opponent. The action
-- transaction dispatches on the route, so these are real code paths, not labels.
B.mechanics = {
  damage = {route = 'offense', targeting = 'opponent'},
  control = {route = 'control', targeting = 'opponent'},
  movement = {route = 'movement', targeting = 'self'},
  defense = {route = 'defense', targeting = 'either'},
  conversion = {route = 'conversion', targeting = 'either'},
  chain = {route = 'offense', targeting = 'opponent'},
}

-- Effect kinds the transaction understands, each bound to one route.
B.kinds = {
  burst = {route = 'offense'}, line = {route = 'offense'},
  chain = {route = 'offense'}, push = {route = 'offense'},
  mark = {route = 'control'}, root = {route = 'control'},
  dash = {route = 'movement'}, hover = {route = 'movement'},
  guard = {route = 'defense'}, counter = {route = 'defense'},
  convert = {route = 'conversion'}, siphon = {route = 'conversion'},
}

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end
local function integer(x) return finite(x) and x % 1 == 0 end
local function name(x) return type(x) == 'string' and #x > 0 and #x <= 64 and not x:find('[%z\1-\31]') end

-- Default the transaction-policy fields. Callers may override any of them; the
-- validator is what actually enforces bounds and route compatibility.
local function p(t)
  t.targeting = t.targeting or (B.mechanics[t.mechanic].targeting == 'self' and 'self' or 'opponent')
  t.startup = t.startup or 0
  t.active = t.active or 1
  t.recovery = t.recovery or 0
  t.range = t.range or 0
  t.height = t.height or 0
  t.arc = t.arc or (t.targeting == 'opponent' and 'facing' or 'omni')
  t.occlusion = t.occlusion or (t.targeting == 'opponent' and 'line' or 'ignore')
  t.spend = t.spend or 'release'
  t.refund = t.refund or 'on_refuse'
  t.reversible = t.reversible or false
  t.earning = t.earning or 'per_move'
  t.native = t.native or false
  return t
end

-- 12 genes / 6 families. Placements use the Core slot names. action/trigger/
-- category for cinder and rime are copied verbatim from core.lua and re-checked
-- by from_core(); the remaining fields describe the transaction policy.
B.genes = {
  cinder = {id = 'cinder', family = 'fire', offered = true, placements = {
    assault = p{action = 'eruption', trigger = 'direct_hit', category = 'Magic', mechanic = 'damage',
      range = 9, height = 12, native = true, effect = {kind = 'burst', radius = 3, burn = 60}},
    traversal = p{action = 'step', trigger = 'move', category = 'Special', mechanic = 'movement',
      native = true, effect = {kind = 'dash', distance = 2.4, grounded = true}},
    guard = p{action = 'counter', trigger = 'defend', category = 'Special', mechanic = 'defense',
      targeting = 'opponent', range = 9, height = 12, active = 8, native = true,
      effect = {kind = 'counter', window = 8, power = 1.0}},
  }},
  emberline = {id = 'emberline', family = 'fire', offered = false, placements = {
    assault = p{action = 'tracer', trigger = 'direct_hit', category = 'Magic', mechanic = 'damage',
      range = 6, height = 12, effect = {kind = 'line', pierce = 1, length = 6}},
    traversal = p{action = 'trailblaze', trigger = 'move', category = 'Special', mechanic = 'movement',
      effect = {kind = 'hover', glide = true, duration = 30}},
  }},
  rime = {id = 'rime', family = 'frost', offered = true, placements = {
    assault = p{action = 'mark', trigger = 'direct_hit', category = 'Magic', mechanic = 'control',
      range = 9, height = 12, native = true, effect = {kind = 'mark', status = 'rime', duration = 180}},
    traversal = p{action = 'glide', trigger = 'move', category = 'Special', mechanic = 'movement',
      native = true, effect = {kind = 'hover', glide = true, duration = 24}},
    guard = p{action = 'mark', trigger = 'defend', category = 'Magic', mechanic = 'control',
      range = 9, height = 12, native = true, effect = {kind = 'mark', status = 'rime', duration = 180}},
  }},
  glacier = {id = 'glacier', family = 'frost', offered = false, placements = {
    assault = p{action = 'freeze', trigger = 'direct_hit', category = 'Magic', mechanic = 'control',
      startup = 6, recovery = 10, range = 12, height = 14, effect = {kind = 'root', duration = 45}},
    guard = p{action = 'walls', trigger = 'defend', category = 'Special', mechanic = 'defense',
      targeting = 'self', reversible = true, effect = {kind = 'guard', absorb = 20, duration = 120}},
  }},
  gale = {id = 'gale', family = 'kinetic', offered = false, placements = {
    traversal = p{action = 'gust', trigger = 'move', category = 'Special', mechanic = 'movement',
      effect = {kind = 'dash', distance = 3.2, grounded = false}},
    assault = p{action = 'windblade', trigger = 'direct_hit', category = 'Special', mechanic = 'damage',
      range = 4, height = 12, effect = {kind = 'push', impulse = 2.0}},
  }},
  dashstep = {id = 'dashstep', family = 'kinetic', offered = false, placements = {
    traversal = p{action = 'cadence', trigger = 'move', category = 'Special', mechanic = 'movement',
      effect = {kind = 'dash', distance = 1.6, ['repeat'] = 2}},
    assault = p{action = 'momentum', trigger = 'direct_hit', category = 'Magic', mechanic = 'damage',
      range = 5, height = 12, effect = {kind = 'burst', radius = 2, bonus_from_speed = true}},
  }},
  bulwark = {id = 'bulwark', family = 'aegis', offered = false, placements = {
    guard = p{action = 'bulwark', trigger = 'defend', category = 'Special', mechanic = 'defense',
      targeting = 'self', reversible = true, effect = {kind = 'guard', absorb = 12, duration = 120}},
    assault = p{action = 'bash', trigger = 'direct_hit', category = 'Special', mechanic = 'damage',
      range = 7, height = 12, effect = {kind = 'push', impulse = 1.4, shield_damage = 8}},
  }},
  retaliate = {id = 'retaliate', family = 'aegis', offered = false, placements = {
    guard = p{action = 'retaliate', trigger = 'defend', category = 'Special', mechanic = 'defense',
      targeting = 'opponent', range = 8, height = 12, reversible = true,
      effect = {kind = 'counter', window = 10, power = 1.5}},
    assault = p{action = 'riposte', trigger = 'direct_hit', category = 'Magic', mechanic = 'damage',
      range = 8, height = 12, effect = {kind = 'burst', radius = 2, after_block = true}},
  }},
  siphon = {id = 'siphon', family = 'flux', offered = false, placements = {
    assault = p{action = 'siphon', trigger = 'direct_hit', category = 'Magic', mechanic = 'conversion',
      reversible = true, range = 8, height = 12, effect = {kind = 'siphon', status = 'rime', grant = 1}},
    guard = p{action = 'draw', trigger = 'defend', category = 'Special', mechanic = 'conversion',
      targeting = 'self', reversible = true, effect = {kind = 'convert', from = 'charge', to = 'guard', ratio = 0.5}},
  }},
  convert = {id = 'convert', family = 'flux', offered = false, placements = {
    guard = p{action = 'lens', trigger = 'defend', category = 'Special', mechanic = 'conversion',
      targeting = 'self', reversible = true, effect = {kind = 'convert', from = 'charge', to = 'move', ratio = 0.5}},
    traversal = p{action = 'vent', trigger = 'move', category = 'Special', mechanic = 'movement',
      effect = {kind = 'dash', distance = 3.0, from = 'charge', cost = 2}},
  }},
  brand = {id = 'brand', family = 'sigil', offered = false, placements = {
    assault = p{action = 'brand', trigger = 'direct_hit', category = 'Magic', mechanic = 'chain',
      range = 14, height = 14, earning = 'per_target', effect = {kind = 'chain', status = 'brand', links = 1, duration = 240}},
    guard = p{action = 'sigilward', trigger = 'defend', category = 'Special', mechanic = 'defense',
      targeting = 'self', reversible = true, effect = {kind = 'guard', absorb = 10, duration = 120}},
  }},
  chain = {id = 'chain', family = 'sigil', offered = false, placements = {
    assault = p{action = 'chain', trigger = 'direct_hit', category = 'Magic', mechanic = 'chain',
      range = 18, height = 14, earning = 'per_target', effect = {kind = 'chain', links = 2, damage = 6}},
    traversal = p{action = 'linkstep', trigger = 'move', category = 'Special', mechanic = 'movement',
      effect = {kind = 'dash', distance = 2.0, toward_marked = true}},
  }},
}

-- Authored cross-family synergy table. Inputs are statuses a reaction consumes;
-- outputs are statuses it creates. A reaction may not consume another reaction's
-- output (non-recursive), which validate() enforces. Only thermal_shock is
-- modelled against existing Core behaviour; the rest are admission candidates.
local function reaction(families, precedence, implemented, input, output, mechanic)
  return {families = families, precedence = precedence, implemented = implemented,
    input = input, output = output, mechanic = mechanic}
end
B.reactions = {
  thermal_shock = reaction({'fire', 'frost'}, 1, true, {'rime'}, {}, 'consume_frost_mark'),
  plasma_surge = reaction({'fire', 'flux'}, 2, false, {}, {}, 'burst_on_conversion'),
  charged_crystals = reaction({'frost', 'sigil'}, 3, false, {}, {}, 'shatter_on_chain'),
  chain_lightning = reaction({'kinetic', 'sigil'}, 4, false, {}, {}, 'chain_on_move'),
  aegis_crush = reaction({'aegis', 'kinetic'}, 5, false, {}, {}, 'impact_on_dash'),
  flux_bloom = reaction({'flux', 'frost'}, 6, false, {}, {}, 'spread_on_freeze'),
}

local allowed_effect = {
  burst = {radius = 'amount', burn = 'duration', bonus_from_speed = 'bool', after_block = 'bool'},
  line = {pierce = 'pierce', length = 'amount'},
  chain = {links = 'links', damage = 'amount', status = 'string', duration = 'duration'},
  push = {impulse = 'impulse', shield_damage = 'amount'},
  mark = {status = 'string', duration = 'duration'},
  root = {duration = 'duration'},
  dash = {distance = 'distance', grounded = 'bool', ['repeat'] = 'repeat', from = 'string', cost = 'amount', toward_marked = 'bool'},
  hover = {glide = 'bool', duration = 'duration'},
  guard = {absorb = 'amount', duration = 'duration'},
  counter = {window = 'window', power = 'ratio'},
  convert = {from = 'string', to = 'string', ratio = 'ratio'},
  siphon = {status = 'string', grant = 'amount'},
}

local function check_effect(effect)
  assert(type(effect) == 'table' and B.kinds[effect.kind], 'invalid effect kind')
  local schema = allowed_effect[effect.kind]
  for field, value in pairs(effect) do
    if field ~= 'kind' then
      local rule = schema[field]
      assert(rule, 'unknown effect field ' .. tostring(field))
      if rule == 'string' then assert(name(value), 'effect string field')
      elseif rule == 'bool' then assert(type(value) == 'boolean', 'effect boolean field')
      else
        local bounds = B.bounds[rule]
        local ok = (rule == 'ratio' or rule == 'distance' or rule == 'impulse') and finite(value) or integer(value)
        assert(ok and value >= bounds[1] and value <= bounds[2], 'effect field out of range: ' .. field)
      end
    end
  end
end

local function check_placement(spec)
  for field in pairs(spec) do
    assert(({action = true, trigger = true, category = true, mechanic = true, targeting = true,
      startup = true, active = true, recovery = true, range = true, height = true, arc = true,
      occlusion = true, spend = true, refund = true, reversible = true, earning = true,
      native = true, effect = true})[field], 'unknown placement field ' .. tostring(field))
  end
  assert(name(spec.action) and name(spec.trigger) and name(spec.category), 'placement identity')
  local mech = B.mechanics[spec.mechanic]
  assert(mech, 'unknown mechanic')
  assert(integer(spec.startup) and spec.startup >= B.bounds.startup[1] and spec.startup <= B.bounds.startup[2], 'startup bound')
  assert(integer(spec.active) and spec.active >= B.bounds.active[1] and spec.active <= B.bounds.active[2], 'active bound')
  assert(integer(spec.recovery) and spec.recovery >= B.bounds.recovery[1] and spec.recovery <= B.bounds.recovery[2], 'recovery bound')
  assert(integer(spec.range) and spec.range >= B.bounds.range[1] and spec.range <= B.bounds.range[2], 'range bound')
  assert(integer(spec.height) and spec.height >= B.bounds.height[1] and spec.height <= B.bounds.height[2], 'height bound')
  assert(spec.arc == 'facing' or spec.arc == 'omni', 'invalid arc')
  assert(spec.occlusion == 'line' or spec.occlusion == 'ignore', 'invalid occlusion')
  assert(spec.spend == 'release' or spec.spend == 'start', 'invalid spend point')
  assert(spec.refund == 'on_refuse' or spec.refund == 'never', 'invalid refund policy')
  assert(type(spec.reversible) == 'boolean' and type(spec.native) == 'boolean', 'invalid booleans')
  assert(spec.earning == 'per_move' or spec.earning == 'per_target', 'invalid earning policy')
  -- The effect kind route must match the declared mechanic route; this is what
  -- makes "movement"/"defense"/"conversion"/"control" real code paths.
  assert(B.kinds[spec.effect.kind].route == mech.route, 'effect route does not match mechanic')
  local want_target = mech.targeting == 'opponent' and 'opponent' or (mech.targeting == 'self' and 'self' or nil)
  if want_target then assert(spec.targeting == want_target, 'targeting does not match mechanic') end
  assert(spec.targeting == 'opponent' or spec.targeting == 'self', 'invalid targeting')
  if spec.targeting == 'opponent' then assert(spec.range > 0, 'opponent placement needs range') end
  check_effect(spec.effect)
end

-- A placement signature includes its trigger so rime's direct-hit mark and
-- shield mark remain two authored behaviors rather than a duplicate action.
local function placement_signature(spec)
  return spec.action .. '|' .. spec.mechanic .. '|' .. spec.effect.kind .. '|' .. spec.trigger
end

local function signature(gene)
  local out = {}
  for _, slot in ipairs({'assault', 'traversal', 'guard'}) do
    if gene.placements[slot] then out[#out + 1] = placement_signature(gene.placements[slot]) end
  end
  table.sort(out)
  return table.concat(out, '+')
end

-- Field-level problems assert; count/consistency problems return false, reason.
function B.validate(self)
  self = self or B
  local families = 0
  for _, f in pairs(self.families) do
    families = families + 1
    assert(name(f.name) and name(f.role) and self.mechanics[f.primary], 'invalid family')
  end
  if families < 6 then return false, 'fewer than six families' end
  local genes, per_family, signatures = 0, {}, {}
  for id, gene in pairs(self.genes) do
    genes = genes + 1
    assert(gene.id == id and self.families[gene.family], 'invalid gene identity')
    assert(type(gene.offered) == 'boolean', 'gene offered flag')
    per_family[gene.family] = (per_family[gene.family] or 0) + 1
    local slots, primary_seen, placement_sigs = 0, false, {}
    for slot, spec in pairs(gene.placements) do
      assert(slot == 'assault' or slot == 'traversal' or slot == 'guard', 'invalid placement slot')
      slots = slots + 1
      check_placement(spec)
      if spec.mechanic == self.families[gene.family].primary then primary_seen = true end
      local psig = placement_signature(spec)
      assert(not placement_sigs[psig], 'duplicate placement behavior: ' .. id .. '.' .. slot)
      placement_sigs[psig] = true
    end
    assert(slots >= 2, 'gene needs two placements')
    assert(primary_seen, 'gene family primary mechanic missing: ' .. id)
    local sig = signature(gene)
    assert(not signatures[sig], 'duplicate behavior signature: ' .. id)
    signatures[sig] = true
  end
  if genes < 12 then return false, 'fewer than twelve genes' end
  for family, n in pairs(per_family) do if n < 2 then return false, 'family needs two genes: ' .. family end end
  for family in pairs(self.families) do
    if not per_family[family] then return false, 'family has no genes: ' .. family end
  end
  -- Offered genes are exactly the ones with Core definitions today.
  for id, offered in pairs(self.offered_genes) do
    if offered then assert(self.genes[id] and self.genes[id].offered, 'offered gene missing') end
  end
  for id, gene in pairs(self.genes) do
    assert(gene.offered == (self.offered_genes[id] == true), 'offered flag disagrees with offered_genes: ' .. id)
  end
  -- Reactions: six, pair-typed, non-recursive, outputs never feed inputs.
  local reactions, precedence = 0, {}
  local produced = {}
  for id, r in pairs(self.reactions) do
    reactions = reactions + 1
    assert(type(r.families) == 'table' and #r.families == 2, 'invalid reaction families')
    for _, f in ipairs(r.families) do assert(self.families[f], 'reaction references unknown family') end
    assert(finite(r.precedence) and integer(r.precedence), 'invalid reaction precedence')
    assert(not precedence[r.precedence], 'duplicate precedence')
    precedence[r.precedence] = true
    assert(type(r.implemented) == 'boolean' and name(r.mechanic), 'invalid reaction record')
    for _, s in ipairs(r.output) do produced[s] = true end
  end
  if reactions < 6 then return false, 'fewer than six reactions' end
  if reactions >= families * (families - 1) / 2 then return false, 'reaction table must not be exhaustive' end
  for id, r in pairs(self.reactions) do
    for _, s in ipairs(r.input) do
      assert(not produced[s], 'reaction chain consumes another reaction output: ' .. id)
    end
  end
  return true
end

function B.placement(gene_id, slot)
  local gene = B.genes[gene_id]
  return gene and gene.placements[slot], gene
end

function B.is_offered(gene_id)
  return B.offered_genes[gene_id] == true
end

function B.admitted_defaults()
  local out = {}
  for id, offered in pairs(B.offered_genes) do if offered then out[id] = true end end
  return out
end

-- Cross-check the offered behaviors against the authoritative Core definitions.
-- Returns true, or false, reason. This is the evidence that "Cinder/Rime exact
-- current Core" still holds after any edit.
function B.from_core(Core)
  if type(Core) ~= 'table' or type(Core.definitions) ~= 'table' then return false, 'Core definitions required' end
  for id, kind in pairs({cinder = 'cinder', rime = 'rime'}) do
    local def = Core.definitions[kind]
    local gene = B.genes[id]
    if not def or not gene then return false, 'missing core/behavior definition: ' .. id end
    if def.family ~= gene.family then return false, 'family mismatch: ' .. id end
    for slot, variant in pairs(def.variants) do
      local spec = gene.placements[slot]
      if not spec then return false, 'core placement absent from behavior: ' .. id .. '.' .. slot end
      if spec.action ~= variant.action or spec.trigger ~= variant.trigger or spec.category ~= variant.category then
        return false, 'core/behavior mismatch: ' .. id .. '.' .. slot
      end
    end
    for slot, spec in pairs(gene.placements) do
      if not def.variants[slot] then return false, 'behavior placement absent from core: ' .. id .. '.' .. slot end
    end
  end
  return true
end

return B
