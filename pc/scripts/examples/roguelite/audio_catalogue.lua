-- Audio event map (Gate 10 planning spec). One row per player-facing audio
-- event with its bus, variation and concurrency budget, and an explicit
-- provenance field. Assets are 'unassigned' until original or appropriately
-- licensed sounds are sourced and recorded; this module claims no audio exists.
local AudioCatalogue = {version = 1}

AudioCatalogue.buses = {ui = true, sfx = true, music = true}

local function event(id, bus, opts)
  opts = opts or {}
  opts.id, opts.bus = id, bus
  opts.asset = opts.asset or nil
  opts.provenance = opts.provenance or 'unassigned'
  opts.variation = opts.variation or 1
  opts.concurrency = opts.concurrency or 1
  opts.critical = opts.critical or false
  return opts
end

AudioCatalogue.events = {
  menu_focus = event('menu_focus', 'ui', {concurrency = 2, variation = 2}),
  menu_confirm = event('menu_confirm', 'ui'),
  menu_back = event('menu_back', 'ui'),
  menu_refusal = event('menu_refusal', 'ui', {critical = true}),
  door_use = event('door_use', 'sfx', {concurrency = 2}),
  pickup = event('pickup', 'sfx', {variation = 2}),
  charge_ready = event('charge_ready', 'sfx', {variation = 2, critical = true}),
  gene_startup = event('gene_startup', 'sfx'),
  gene_release = event('gene_release', 'sfx', {variation = 2}),
  gene_recovery = event('gene_recovery', 'sfx'),
  reaction = event('reaction', 'sfx', {critical = true}),
  enemy_tell = event('enemy_tell', 'sfx', {variation = 2, critical = true}),
  reward = event('reward', 'sfx'),
  fusion = event('fusion', 'ui'),
  boss_phase = event('boss_phase', 'music'),
  death = event('death', 'sfx'),
  run_complete = event('run_complete', 'music'),
}

-- Events that must stay audible over music/spectacle.
AudioCatalogue.critical = {'menu_refusal', 'charge_ready', 'enemy_tell', 'reaction'}

function AudioCatalogue.validate(self)
  self = self or AudioCatalogue
  local count = 0
  for id, e in pairs(self.events) do
    count = count + 1
    assert(e.id == id, 'audio id mismatch')
    assert(self.buses[e.bus], 'invalid audio bus: ' .. tostring(e.bus))
    assert(type(e.provenance) == 'string' and #e.provenance > 0, 'audio provenance required')
    assert(type(e.variation) == 'number' and e.variation >= 1 and e.variation <= 8, 'audio variation range')
    assert(type(e.concurrency) == 'number' and e.concurrency >= 1 and e.concurrency <= 8, 'audio concurrency range')
    assert(type(e.critical) == 'boolean', 'audio critical flag')
  end
  if count < 15 then return false, 'fewer than fifteen audio events' end
  for _, id in ipairs(self.critical) do
    assert(self.events[id], 'critical event missing: ' .. id)
    assert(self.events[id].critical == true, 'critical event not flagged: ' .. id)
  end
  -- No asset is claimed before it is sourced.
  for _, e in pairs(self.events) do
    if e.asset then assert(e.provenance ~= 'unassigned', 'asset without provenance: ' .. e.id) end
  end
  return true
end

return AudioCatalogue
