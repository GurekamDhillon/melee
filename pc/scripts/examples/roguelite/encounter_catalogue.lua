-- Encounter, reward and lock catalogue. These are data contracts resolved by
-- the generator's independent streams and persisted in the manifest. Native
-- actor kinds are logical names; the runtime adapter maps them to spawn APIs.
local EncounterCatalogue = {version = 1}

EncounterCatalogue.archetypes = {
  pressure = {name = 'Pressure', intent = 'close the distance'},
  guard = {name = 'Guard', intent = 'hold and punish'},
  zone = {name = 'Zone', intent = 'deny space'},
  aerial = {name = 'Aerial', intent = 'contest recovery'},
  elite = {name = 'Elite', intent = 'combine traits'},
  boss = {name = 'Champion', intent = 'phase and position'},
}

local function encounter(id, archetype, difficulty, enemies, themes)
  return {id = id, version = 1, archetype = archetype, difficulty = difficulty,
    enemies = enemies, themes = themes or {'cobalt', 'frost', 'fire'}}
end

EncounterCatalogue.encounters = {
  scout_pair = encounter('scout_pair', 'pressure', 1, {{kind = 'goomba', count = 2, level = 3}}),
  guard_post = encounter('guard_post', 'guard', 1, {{kind = 'redead', count = 1, level = 4}}),
  zoner_wall = encounter('zoner_wall', 'zone', 2, {{kind = 'goomba', count = 3, level = 5}}),
  aerial_duel = encounter('aerial_duel', 'aerial', 2, {{kind = 'fighter', count = 1, level = 6}}, {'frost', 'cobalt'}),
  elite_mix = encounter('elite_mix', 'elite', 3, {{kind = 'redead', count = 1, level = 7}, {kind = 'goomba', count = 1, level = 5}}),
  champ_cinder = encounter('champ_cinder', 'boss', 4, {{kind = 'champion', count = 1, level = 8, family = 'cinder'}}, {'fire'}),
  champ_rime = encounter('champ_rime', 'boss', 4, {{kind = 'champion', count = 1, level = 8, family = 'rime'}}, {'frost'}),
}

EncounterCatalogue.rewardKinds = {upgrade = true, tradeoff = true, mutation = true, consumable = true, equipment = true}

local function reward(id, kind, field)
  field = field or {}
  field.id, field.version, field.kind = id, 1, kind
  return field
end

EncounterCatalogue.rewards = {
  potency_small = reward('potency_small', 'upgrade', {stat = 'potency', delta = 2, family = 'any'}),
  capacity_small = reward('capacity_small', 'upgrade', {stat = 'capacity', delta = 1, family = 'any'}),
  reach_small = reward('reach_small', 'upgrade', {stat = 'reach', delta = 3, family = 'any'}),
  cooldown_trim = reward('cooldown_trim', 'upgrade', {stat = 'cooldown', delta = -15, family = 'any'}),
  glass_cannon = reward('glass_cannon', 'tradeoff', {add = {potency = 6}, remove = {capacity = 1}, family = 'fire'}),
  slow_burn = reward('slow_burn', 'tradeoff', {add = {potency = 4}, remove = {gain = 0.5}, family = 'any'}),
  thermal_seed = reward('thermal_seed', 'mutation', {reaction = 'thermal_shock', family = 'fire'}),
  supply_crate = reward('supply_crate', 'consumable', {supply = 1}),
  warding_charm = reward('warding_charm', 'equipment', {slot = 'charm', stat = 'capacity', delta = 1}),
}

EncounterCatalogue.locks = {
  frost_gate = {id = 'frost_gate', version = 1, kind = 'persistent_key', key = 'frost_key', theme = 'frost'},
  ember_gate = {id = 'ember_gate', version = 1, kind = 'persistent_key', key = 'ember_key', theme = 'fire'},
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
  for id, encounter in pairs(self.encounters) do
    assert(type(id) == 'string' and encounter.id == id, 'invalid encounter id')
    assert(self.archetypes[encounter.archetype], 'invalid archetype')
    assert(type(encounter.difficulty) == 'number' and encounter.difficulty % 1 == 0 and encounter.difficulty >= 1 and encounter.difficulty <= 5, 'invalid difficulty')
    assert(type(encounter.enemies) == 'table' and #encounter.enemies >= 1, 'empty encounter')
    for _, enemy in ipairs(encounter.enemies) do
      assert(type(enemy.kind) == 'string' and enemy.count >= 1 and enemy.level >= 1 and enemy.level <= 9, 'invalid enemy row')
    end
  end
  for id, reward in pairs(self.rewards) do
    assert(type(id) == 'string' and reward.id == id and self.rewardKinds[reward.kind], 'invalid reward')
  end
  for id, lock in pairs(self.locks) do
    assert(type(id) == 'string' and lock.id == id, 'invalid lock id')
    assert(lock.kind == 'persistent_key' or lock.kind == 'consumable_key', 'invalid lock kind')
    assert(type(lock.key) == 'string' and #lock.key > 0, 'invalid lock key')
  end
  return true
end

return EncounterCatalogue
