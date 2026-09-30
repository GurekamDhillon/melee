-- Encounter, reward, lock and reaction catalogue. These are data contracts
-- resolved by the generator's independent streams and persisted in the manifest.
-- Native actor kinds are logical names; the runtime adapter maps them to spawn
-- APIs. Mechanical implementations are tracked by the relevant gates; the
-- reaction rows are explicitly `implemented = false` until Gate 5/6.
local EncounterCatalogue = {version = 1}

EncounterCatalogue.archetypes = {
  pressure = {name = 'Pressure', intent = 'close the distance'},
  guard = {name = 'Guard', intent = 'hold and punish'},
  zone = {name = 'Zone', intent = 'deny space'},
  aerial = {name = 'Aerial', intent = 'contest recovery'},
  elite = {name = 'Elite', intent = 'combine traits'},
  boss = {name = 'Champion', intent = 'phase and position'},
}

EncounterCatalogue.themes = {'cobalt', 'frost', 'fire'}

local function encounter(id, archetype, difficulty, enemies, themes)
  return {id = id, version = 1, archetype = archetype, difficulty = difficulty,
    enemies = enemies, themes = themes or {'cobalt', 'frost', 'fire'}}
end

EncounterCatalogue.encounters = {
  scout_pair = encounter('scout_pair', 'pressure', 1, {{kind = 'goomba', count = 2, level = 3}}),
  pincer_pair = encounter('pincer_pair', 'pressure', 2, {{kind = 'goomba', count = 2, level = 5}, {kind = 'redead', count = 1, level = 4}}),
  guard_post = encounter('guard_post', 'guard', 1, {{kind = 'redead', count = 1, level = 4}}),
  shield_wall = encounter('shield_wall', 'guard', 2, {{kind = 'redead', count = 2, level = 5}}),
  zoner_wall = encounter('zoner_wall', 'zone', 2, {{kind = 'goomba', count = 3, level = 5}}),
  skyline_denial = encounter('skyline_denial', 'zone', 3, {{kind = 'goomba', count = 2, level = 6}, {kind = 'fighter', count = 1, level = 6}}, {'cobalt', 'frost'}),
  aerial_duel = encounter('aerial_duel', 'aerial', 2, {{kind = 'fighter', count = 1, level = 6}}, {'frost', 'cobalt'}),
  recovery_hunt = encounter('recovery_hunt', 'aerial', 3, {{kind = 'fighter', count = 1, level = 7}, {kind = 'goomba', count = 1, level = 5}}, {'cobalt', 'fire'}),
  elite_mix = encounter('elite_mix', 'elite', 3, {{kind = 'redead', count = 1, level = 7}, {kind = 'goomba', count = 1, level = 5}}),
  elite_twin = encounter('elite_twin', 'elite', 4, {{kind = 'fighter', count = 1, level = 8}, {kind = 'redead', count = 1, level = 7}}),
  swarm_rush = encounter('swarm_rush', 'pressure', 3, {{kind = 'goomba', count = 4, level = 5}}),
  bastion_hold = encounter('bastion_hold', 'guard', 3, {{kind = 'redead', count = 2, level = 6}, {kind = 'goomba', count = 1, level = 4}}),
  vertigo_zone = encounter('vertigo_zone', 'zone', 4, {{kind = 'goomba', count = 2, level = 7}, {kind = 'fighter', count = 1, level = 7}}, {'frost', 'cobalt'}),
  hunter_pair = encounter('hunter_pair', 'aerial', 4, {{kind = 'fighter', count = 1, level = 8}, {kind = 'fighter', count = 1, level = 6}}, {'cobalt', 'fire'}),
  champ_cinder = encounter('champ_cinder', 'boss', 4, {{kind = 'champion', count = 1, level = 8, family = 'cinder'}}, {'fire'}),
  champ_rime = encounter('champ_rime', 'boss', 4, {{kind = 'champion', count = 1, level = 8, family = 'rime'}}, {'frost'}),
  champ_gale = encounter('champ_gale', 'boss', 4, {{kind = 'champion', count = 1, level = 8, family = 'cinder'}}, {'cobalt'}),
}

EncounterCatalogue.rewardKinds = {upgrade = true, tradeoff = true, mutation = true, consumable = true, equipment = true}
EncounterCatalogue.families = {any = true, fire = true, frost = true, cobalt = true}

local function reward(id, kind, family, field)
  field = field or {}
  field.id, field.version, field.kind, field.family = id, 1, kind, family or 'any'
  return field
end

EncounterCatalogue.rewards = {
  -- upgrades
  potency_small = reward('potency_small', 'upgrade', 'any', {stat = 'potency', delta = 2}),
  potency_large = reward('potency_large', 'upgrade', 'any', {stat = 'potency', delta = 5}),
  capacity_small = reward('capacity_small', 'upgrade', 'any', {stat = 'capacity', delta = 1}),
  capacity_large = reward('capacity_large', 'upgrade', 'any', {stat = 'capacity', delta = 2}),
  reach_small = reward('reach_small', 'upgrade', 'any', {stat = 'reach', delta = 3}),
  cooldown_trim = reward('cooldown_trim', 'upgrade', 'any', {stat = 'cooldown', delta = -15}),
  cooldown_major = reward('cooldown_major', 'upgrade', 'any', {stat = 'cooldown', delta = -30}),
  gain_small = reward('gain_small', 'upgrade', 'any', {stat = 'gain', delta = 0.5}),
  -- tradeoffs
  glass_cannon = reward('glass_cannon', 'tradeoff', 'fire', {add = {potency = 6}, remove = {capacity = 1}}),
  slow_burn = reward('slow_burn', 'tradeoff', 'fire', {add = {potency = 4}, remove = {gain = 0.5}}),
  brittle_guard = reward('brittle_guard', 'tradeoff', 'frost', {add = {capacity = 2}, remove = {potency = 3}}),
  short_fuse = reward('short_fuse', 'tradeoff', 'any', {add = {cooldown = -30}, remove = {gain = 0.5}}),
  overcharge = reward('overcharge', 'tradeoff', 'cobalt', {add = {gain = 1}, remove = {capacity = 1}}),
  heavy_step = reward('heavy_step', 'tradeoff', 'any', {add = {potency = 3}, remove = {reach = 3}}),
  narrow_focus = reward('narrow_focus', 'tradeoff', 'any', {add = {potency = 4}, remove = {reach = 5}}),
  volatile_core = reward('volatile_core', 'tradeoff', 'fire', {add = {potency = 8}, remove = {capacity = 2}}),
  -- mutations
  thermal_seed = reward('thermal_seed', 'mutation', 'fire', {reaction = 'thermal_shock'}),
  rime_bloom = reward('rime_bloom', 'mutation', 'frost', {reaction = 'glacial_shatter'}),
  chain_spark = reward('chain_spark', 'mutation', 'cobalt', {reaction = 'chain_lightning'}),
  echo_mark = reward('echo_mark', 'mutation', 'any', {reaction = 'echo_mark'}),
  -- consumables
  supply_crate = reward('supply_crate', 'consumable', 'any', {supply = 1}),
  supply_pack = reward('supply_pack', 'consumable', 'any', {supply = 2}),
  repair_kit = reward('repair_kit', 'consumable', 'any', {heal = 30}),
  charge_flask = reward('charge_flask', 'consumable', 'any', {charge = 3}),
  -- equipment
  warding_charm = reward('warding_charm', 'equipment', 'any', {slot = 'charm', stat = 'capacity', delta = 1}),
  ember_lens = reward('ember_lens', 'equipment', 'fire', {slot = 'focus', stat = 'potency', delta = 2}),
  frost_buckle = reward('frost_buckle', 'equipment', 'frost', {slot = 'charm', stat = 'capacity', delta = 1}),
  storm_pin = reward('storm_pin', 'equipment', 'cobalt', {slot = 'focus', stat = 'gain', delta = 0.5}),
}

-- Lock catalogue. Persistent keys are preferred on the mandatory path; a
-- consumable lock must be paired with a reachable grant for the search to pass.
EncounterCatalogue.locks = {
  frost_gate = {id = 'frost_gate', version = 1, kind = 'persistent_key', key = 'frost_key', theme = 'frost'},
  ember_gate = {id = 'ember_gate', version = 1, kind = 'persistent_key', key = 'ember_key', theme = 'fire'},
  gale_gate = {id = 'gale_gate', version = 1, kind = 'persistent_key', key = 'gale_key', theme = 'cobalt'},
  timed_vault = {id = 'timed_vault', version = 1, kind = 'consumable_key', key = 'vault_charge', theme = 'any'},
}

-- Authored interaction vocabulary. Data only until implemented by Gate 5/6.
local function reaction(id, components)
  return {id = id, version = 1, implemented = false, components = components}
end
EncounterCatalogue.reactions = {
  thermal_shock = reaction('thermal_shock', {'fire', 'frost'}),
  glacial_shatter = reaction('glacial_shatter', {'frost'}),
  plasma_surge = reaction('plasma_surge', {'fire', 'cobalt'}),
  charged_crystals = reaction('charged_crystals', {'frost', 'cobalt'}),
  fire_cyclone = reaction('fire_cyclone', {'fire'}),
  orbital_shard_storm = reaction('orbital_shard_storm', {'cobalt'}),
  chain_lightning = reaction('chain_lightning', {'cobalt'}),
  echo_mark = reaction('echo_mark', {'any'}),
}

function EncounterCatalogue.encounters_for_theme(theme)
  local out = {}
  local ids = {}
  for id in pairs(EncounterCatalogue.encounters) do ids[#ids + 1] = id end
  table.sort(ids)
  for _, id in ipairs(ids) do
    local encounter = EncounterCatalogue.encounters[id]
    for _, allowed in ipairs(encounter.themes) do
      if allowed == theme then out[#out + 1] = encounter break end
    end
  end
  return out
end

function EncounterCatalogue.validate(self)
  self = self or EncounterCatalogue
  local archetypes = 0
  for _ in pairs(self.archetypes) do archetypes = archetypes + 1 end
  if archetypes < 6 then return false, 'fewer than six archetypes' end
  local theme_set = {}
  for _, theme in ipairs(self.themes or {}) do theme_set[theme] = true end
  for id, encounter in pairs(self.encounters) do
    assert(type(id) == 'string' and encounter.id == id, 'invalid encounter id')
    assert(self.archetypes[encounter.archetype], 'invalid archetype')
    assert(type(encounter.difficulty) == 'number' and encounter.difficulty % 1 == 0 and encounter.difficulty >= 1 and encounter.difficulty <= 5, 'invalid difficulty')
    assert(type(encounter.enemies) == 'table' and #encounter.enemies >= 1, 'empty encounter')
    for _, enemy in ipairs(encounter.enemies) do
      assert(type(enemy.kind) == 'string' and enemy.count >= 1 and enemy.level >= 1 and enemy.level <= 9, 'invalid enemy row')
    end
    for _, theme in ipairs(encounter.themes) do assert(theme_set[theme], 'invalid encounter theme') end
  end
  for id, reward in pairs(self.rewards) do
    assert(type(id) == 'string' and reward.id == id and self.rewardKinds[reward.kind], 'invalid reward')
    assert(self.families[reward.family], 'invalid reward family')
  end
  for id, lock in pairs(self.locks) do
    assert(type(id) == 'string' and lock.id == id, 'invalid lock id')
    assert(lock.kind == 'persistent_key' or lock.kind == 'consumable_key', 'invalid lock kind')
    assert(type(lock.key) == 'string' and #lock.key > 0, 'invalid lock key')
  end
  for id, item in pairs(self.reactions) do
    assert(type(id) == 'string' and item.id == id and type(item.implemented) == 'boolean', 'invalid reaction')
    assert(type(item.components) == 'table' and #item.components >= 1, 'reaction needs components')
  end
  return true
end

return EncounterCatalogue
