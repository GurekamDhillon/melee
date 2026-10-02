-- Enemy behaviour contract (Gate 6 planning spec). Maps encounter archetypes to
-- intended movement, targeting, tells, buffs and counterplay, and maps logical
-- enemy kinds to native hosts, ledge/fall rules and gene eligibility. Data only:
-- the AI and native wrappers are Gate 6 work and are not claimed here.
local EnemyCatalogue = {version = 1}

EnemyCatalogue.behaviors = {
  pressure = {approach = 'direct', tell = 'dash_windup', window = 12, buffs = {'haste'}, counterplay = 'shield or spot-dodge', genes = {'kinetic', 'fire'}},
  guard = {approach = 'hold', tell = 'guard_flare', window = 14, buffs = {'armor'}, counterplay = 'grab or shield pressure', genes = {'aegis', 'frost'}},
  zone = {approach = 'spacing', tell = 'zone_marker', window = 18, buffs = {'area'}, counterplay = 'close the gap', genes = {'sigil', 'fire'}},
  aerial = {approach = 'aerial', tell = 'dive_marker', window = 16, buffs = {'jumps'}, counterplay = 'contest the ledge', genes = {'kinetic'}},
  elite = {approach = 'mixed', tell = 'aura', window = 20, buffs = {'elite'}, counterplay = 'isolate and burst', genes = {'flux', 'aegis'}},
  boss = {approach = 'phased', tell = 'phase_roar', window = 24, buffs = {'phases'}, counterplay = 'punish recovery', genes = {'fire', 'frost', 'sigil'}},
}

EnemyCatalogue.enemies = {
  goomba = {id = 'goomba', host = 'adventure', motion = 'walker', ledge = 'turn', fall = 'despawn', genes = {'kinetic', 'fire'}},
  redead = {id = 'redead', host = 'adventure', motion = 'chaser', ledge = 'climb', fall = 'despawn', genes = {'frost', 'aegis'}},
  fighter = {id = 'fighter', host = 'fighter', motion = 'cpu', ledge = 'recover', fall = 'recover', genes = {'kinetic', 'sigil'}},
  champion = {id = 'champion', host = 'fighter', motion = 'boss', ledge = 'recover', fall = 'recover', genes = {'fire', 'frost', 'sigil'}, phases = 3},
}

function EnemyCatalogue.validate(self, encounter_catalogue)
  self = self or EnemyCatalogue
  local behaviors = 0
  for _, b in pairs(self.behaviors) do
    behaviors = behaviors + 1
    assert(type(b.tell) == 'string' and #b.tell > 0, 'behavior needs a tell')
    assert(type(b.window) == 'number' and b.window >= 8 and b.window <= 60, 'behavior window out of range')
    assert(type(b.buffs) == 'table', 'behavior needs buffs')
    assert(type(b.counterplay) == 'string' and #b.counterplay > 0, 'behavior needs counterplay')
    assert(type(b.genes) == 'table' and #b.genes >= 1, 'behavior needs gene eligibility')
  end
  if behaviors < 6 then return false, 'fewer than six behaviors' end
  assert(self.enemies.goomba and self.enemies.redead and self.enemies.fighter and self.enemies.champion, 'missing base enemy')
  for id, e in pairs(self.enemies) do
    assert(e.id == id, 'enemy id mismatch')
    assert(e.host == 'adventure' or e.host == 'fighter', 'invalid enemy host')
    assert(type(e.motion) == 'string' and type(e.ledge) == 'string' and type(e.fall) == 'string', 'enemy rules missing')
    assert(type(e.genes) == 'table' and #e.genes >= 1, 'enemy needs gene eligibility')
  end
  assert(type(self.enemies.champion.phases) == 'number' and self.enemies.champion.phases >= 2, 'champion needs phases')
  if encounter_catalogue then
    for id, encounter in pairs(encounter_catalogue.encounters) do
      for _, enemy in ipairs(encounter.enemies) do
        assert(self.enemies[enemy.kind], 'encounter ' .. id .. ' uses unknown enemy kind ' .. tostring(enemy.kind))
      end
      assert(self.behaviors[encounter.archetype], 'encounter ' .. id .. ' uses unknown archetype ' .. tostring(encounter.archetype))
    end
  end
  return true
end

return EnemyCatalogue
