-- Bounded encounter orchestrator for a resolved v2 node.encounter_spec and the
-- schema-2 progress record. It owns exactly one room's actors:
--
--   * custom Adventure actors spawned through gd.spawn_enemy, with their
--     EnemyGenes gene host attached (goomba/redead today; any other kind the
--     catalogue marks host='adventure' and the native spawn list implements);
--   * ordinary fighter CPUs driven by the existing TechAI technical assist and,
--     unless buff_fighters=false, a real Core gene host with a supported
--     cinder/rime family and an encounter potency buff;
--   * champion "buffed gene host" fighters, a fighter CPU with a stronger Core
--     gene host buff plus an explicit remaining-stock counter fed only by
--     observed native stock transitions.
--
-- It is injected with Core, the engine table and the real modules; it never
-- requires modules, never reads or writes saves and never writes profile genes
-- or the collection. Root owns persistence and controls; this module returns
-- events/results for that transaction layer.
--
-- Admission is deliberately honest. Catalogue text that no native code
-- implements (archetype movement/tells/buffs, reactions, champion "phases" as
-- authored behaviour) is not surfaced as functional. Supported scheduling is:
-- custom actors of a run share a wave; each fighter-kind entity is a singleton
-- sequential wave, because this port has a single fighter CPU port. A
-- composition that cannot be represented inside the bounded budget is refused
-- whole; a fighter is never silently discarded. Stable entity ids are exactly
-- route.lua's `enemy_<room>_<serial>` so saved progress.defeated lines up and
-- duplicate/stale evidence is harmless.
--
-- API (main wiring):
--   local e = RuntimeEncounters.new(Core, gd,
--     {EnemyGenes=EnemyGenes, EnemyCatalogue=EnemyCatalogue, TechAI=TechAI, Progress=Progress},
--     {get_run=function() return run end, on_event=fn,
--      [fighter_family='cinder'|'rime', fighter_slot='assault'|'traversal'|'guard',
--       buff_fighters=true, fighter_potency=1, champion_potency=4,
--       bounds=, spawn_point=, max_*=]})
--   e:begin(node, progress) -> true,events | nil,reason   -- spawn the first live wave
--   e:update(progress)      -> true,events                 -- per frame; respawn/advance
--   e:hit{handle=,id=,from=,damage=} -> {handled,entity,host,...}  -- provenance gate
--   e:defeat(handle, progress) -> true,events | nil,reason -- confirmed native defeat
--   e:stock(port,before,after,progress) -> true,events      -- observed stock transition
--   e:clear() -> true | nil,reason,pending  e:reset()  e:states()
--   e:composition()  e:active_entity()  e:eligibility()  e:status()
-- Events: tell, defeat, stock, wave, clear, respawn, ignored, error.
local RuntimeEncounters = {version = 1}

-- The native spawn list mirrors gw_script.c gs_enemy_names. A catalogue entry
-- only becomes a custom actor when it names an adventure host and one of these.
local CUSTOM_NATIVE = {
  goomba = true, koopa = true, redead = true, like_like = true, octorok = true, polar_bear = true,
}
local FIGHTER_KINDS = {fighter = true, champion = true}

-- The only gene families whose Core definitions/variants actually exist today.
-- Planned kinetic/sigil families are catalogue intent, not supported mechanics.
RuntimeEncounters.families = {cinder = true, rime = true}
RuntimeEncounters.slots = {assault = true, traversal = true, guard = true}
RuntimeEncounters.default_fighter_family = 'cinder'
RuntimeEncounters.default_fighter_slot = 'assault'

local function count(t) local n = 0 for _ in pairs(t) do n = n + 1 end return n end

local function valid_id(x) return type(x) == 'string' and #x > 0 and #x <= 64 and not x:find('[%z\1-\31]') end

function RuntimeEncounters.new(Core, gd, deps, opts)
  assert(type(Core) == 'table' and Core.definitions and Core.acquire and Core.equip
    and Core.apply_modifier and Core.remove_modifier and Core.ability, 'Core module required')
  assert(type(gd) == 'table', 'engine table required')
  deps, opts = deps or {}, opts or {}
  assert(type(deps.EnemyGenes) == 'table' and type(deps.EnemyGenes.new) == 'function', 'EnemyGenes module required')
  assert(type(deps.EnemyCatalogue) == 'table' and type(deps.EnemyCatalogue.enemies) == 'table', 'EnemyCatalogue required')
  assert(type(deps.Progress) == 'table' and type(deps.Progress.validate) == 'function'
    and type(deps.Progress.defeat) == 'function', 'Progress module required')
  assert(type(opts.get_run) == 'function', 'get_run required')
  if deps.TechAI ~= nil then assert(type(deps.TechAI.configure) == 'function', 'invalid TechAI module') end
  -- Explicit supported fighter gene policy. An unsupported explicit family is a
  -- configuration error, never silently converted.
  local fighter_family = opts.fighter_family or RuntimeEncounters.default_fighter_family
  assert(RuntimeEncounters.families[fighter_family], 'unsupported fighter family: ' .. tostring(fighter_family))
  local fighter_slot = opts.fighter_slot or RuntimeEncounters.default_fighter_slot
  assert(RuntimeEncounters.slots[fighter_slot]
    and Core.definitions[fighter_family].variants[fighter_slot], 'unsupported fighter family/slot')
  local fighter_potency = opts.fighter_potency or 1
  local champion_potency = opts.champion_potency or 4
  assert(type(fighter_potency) == 'number' and type(champion_potency) == 'number'
    and math.abs(fighter_potency) <= 30 and math.abs(champion_potency) <= 30, 'invalid fighter potency buff')
  local self = setmetatable({
    Core = Core, gd = gd, catalogue = deps.EnemyCatalogue, progress_module = deps.Progress,
    tech = deps.TechAI, get_run = opts.get_run, on_event = opts.on_event,
    fighter_port = opts.fighter_port or 2, target_port = opts.target_port or 1,
    max_actors = opts.max_actors or 12, max_custom = opts.max_custom or 8,
    max_fighter_waves = opts.max_fighter_waves or 4,
    max_respawns = opts.max_respawns or 2, cleanup_attempts = opts.cleanup_attempts or 4,
    seed_salt = opts.seed_salt or 2654435761, opts_spawn = opts.spawn_point,
    opt_bounds = opts.bounds,
    fighter_family = fighter_family, fighter_slot = fighter_slot,
    buff_fighters = (opts.buff_fighters ~= false),
    fighter_potency = fighter_potency, champion_potency = champion_potency,
    entities = {}, owned = {},
  }, {__index = RuntimeEncounters})
  self.controller = deps.EnemyGenes.new(Core, gd, {
    get_run = opts.get_run, target_port = self.target_port,
    on_tell = function(state, event)
      if opts.on_event then
        opts.on_event({kind = 'tell', handle = state and state.handle, family = state and state.family,
          phase = state and state.phase, event = event})
      end
    end,
  })
  self:reset()
  return self
end

-- True while any native custom actor or Core gene host is still owned.
function RuntimeEncounters:has_live()
  for _, e in ipairs(self.entities or {}) do
    if e.role == 'custom' and e.spawned and e.handle then return true, e.id end
  end
  for host in pairs(self.owned or {}) do return true, host end
  return false
end

-- Drop per-room bookkeeping. It refuses while live owned actors/hosts remain so
-- a caller cannot silently forget native actors or gene hosts; clear() performs
-- the teardown first.
function RuntimeEncounters:reset()
  local live, owner = self:has_live()
  if live then return nil, 'live owned actor/host remains: ' .. tostring(owner) .. '; clear() it first' end
  self.active = false
  self.active_plan = false -- shadow the :plan method so an inactive plan is not callable
  self.node = nil
  self.progress = nil
  self.bounds = self.opt_bounds
  self.entities = {}
  self.by_id = {}
  self.by_handle = {}
  self.waves = {}
  self.wave = nil
  self.error = nil
  self.notice = nil
  self.cleared = false
  self.live_custom = 0
  self.active_fighter = nil
  self.owned = {}
  self.acquired = {}
  self.contacts = {}
  return true
end

-- Resolve one node's encounter_spec into stable entities and waves. Pure: no
-- engine or run access. Returns true,plan or nil,reason.
function RuntimeEncounters:plan(node)
  if type(node) ~= 'table' or not valid_id(node.id) then return nil, 'invalid node' end
  local spec = node.encounter_spec
  if type(spec) ~= 'table' then return nil, 'no encounter specification' end
  if spec.version ~= 1 then return nil, 'unsupported encounter specification version' end
  if not valid_id(spec.id) then return nil, 'invalid encounter id' end
  local enemies = spec.enemies
  if type(enemies) ~= 'table' or #enemies == 0 then return nil, 'empty encounter composition' end
  local cat = self.catalogue.enemies
  local entities, actors, fighters = {}, 0, 0
  for ri, row in ipairs(enemies) do
    if type(row) ~= 'table' then return nil, 'invalid enemy row' end
    local kind, level, countv = row.kind, row.level, row.count
    if not valid_id(kind) then return nil, 'enemy row missing kind' end
    if type(countv) ~= 'number' or countv % 1 ~= 0 or countv < 1 or countv > 8 then
      return nil, 'invalid enemy count for ' .. kind
    end
    if type(level) ~= 'number' or level % 1 ~= 0 or level < 1 or level > 9 then
      return nil, 'invalid enemy level for ' .. kind
    end
    local def = cat[kind]
    if not def then return nil, 'unsupported enemy kind ' .. kind end
    local role
    if def.host == 'adventure' and CUSTOM_NATIVE[kind] then role = 'custom'
    elseif def.host == 'fighter' and FIGHTER_KINDS[kind] then role = 'fighter'
    else return nil, 'enemy kind ' .. kind .. ' is described but has no implemented host' end
    -- An explicit family is respected only when its Core definition exists;
    -- otherwise the whole composition is refused, never quietly converted.
    if row.family ~= nil and not RuntimeEncounters.families[row.family] then
      return nil, 'unsupported explicit family ' .. tostring(row.family) .. ' for ' .. kind
    end
    local family, stocks, slot, buff
    if kind == 'champion' then
      family = row.family or self.fighter_family
      if countv ~= 1 then return nil, 'more than one champion is not a supported composition' end
    elseif role == 'custom' then
      -- EnemyGenes only implements cinder/rime placement; redead defaults rime.
      family = row.family or ((kind == 'redead') and 'rime' or 'cinder')
    else
      family = row.family or self.fighter_family
    end
    if role == 'fighter' then
      slot = row.slot or self.fighter_slot
      if not RuntimeEncounters.slots[slot] or not self.Core.definitions[family]
        or not self.Core.definitions[family].variants[slot] then
        return nil, 'unsupported fighter family/slot ' .. tostring(family) .. '/' .. tostring(slot)
      end
      buff = (kind == 'champion') and self.champion_potency or self.fighter_potency
      stocks = (kind == 'champion') and 2 or 1
      if type(row.stocks) == 'number' and row.stocks % 1 == 0 and row.stocks >= 1 and row.stocks <= 8 then
        stocks = row.stocks
      end
    else
      slot = 'assault'
    end
    actors = actors + countv
    for i = 1, countv do
      local serial = #entities + 1
      local id = 'enemy_' .. node.id .. '_' .. tostring(serial)
      entities[#entities + 1] = {
        id = id, serial = serial, kind = kind, role = role, family = family, level = level,
        slot = slot, buff = buff, row = ri, index = i, fall = def.fall, ledge = def.ledge,
        host = id, stocks = stocks, respawns = 0, spawned = false, handle = nil, error_reported = false,
      }
    end
    if role == 'fighter' then fighters = fighters + countv end
  end
  if #entities > self.max_actors then return nil, 'encounter exceeds the actor budget' end
  if fighters > self.max_fighter_waves then
    return nil, 'encounter needs more fighter waves than the single CPU port supports'
  end
  if fighters > 0 and not self.tech then return nil, 'fighter encounter requires the TechAI module' end
  -- Consecutive custom actors share a wave; each fighter is a singleton wave so
  -- only one ever needs the fighter CPU port.
  local waves, wi, prev = {}, 0, nil
  for _, e in ipairs(entities) do
    if e.role == 'fighter' or prev ~= 'custom' then wi = wi + 1; waves[wi] = {} end
    e.wave = wi
    waves[wi][#waves[wi] + 1] = e.id
    prev = e.role
  end
  return true, {spec = spec, room = node.id, entities = entities, waves = waves, fighters = fighters}
end

function RuntimeEncounters:spawn_point(e)
  if self.opts_spawn then
    local x, y = self.opts_spawn(e)
    if type(x) == 'number' and type(y) == 'number' then return x, y end
  end
  return 8 + ((e.serial - 1) % 4) * 10, 2
end

function RuntimeEncounters:skill(level)
  return math.max(1, math.min(3, math.floor((level + 2) / 3)))
end

function RuntimeEncounters:seed_for(e)
  local run = self.get_run()
  local base = (run and run.world_seed) or self.seed_salt
  return ((base + (e.serial or 1) * self.seed_salt) % 2147483646) + 1
end

function RuntimeEncounters:note(message)
  self.notice = message
end

-- Acquire or reuse the Core gene host for a fighter/champion entity and mark it
-- owned. The slot is the entity's resolved supported slot (assault by default).
function RuntimeEncounters:ensure_gene(e)
  local run = self.get_run()
  if not run then return nil, 'no active run' end
  local slot = e.slot or 'assault'
  local host = run.hosts[e.host]
  if host and host.slots[slot] then
    local gene = host.slots[slot]
    if run.genes[gene].kind ~= e.family then return nil, 'host family mismatch for ' .. e.id end
    e.gene = gene
    self.owned[e.host] = self.owned[e.host] or {gene = gene, slot = slot, modifiers = {}}
    self.acquired[gene] = true
    return true
  end
  local id, why = self.Core.acquire(run, e.family)
  if not id then return nil, 'gene acquisition refused: ' .. tostring(why) end
  self.acquired[id] = true
  local ok, equip_why = self.Core.equip(run, e.host, slot, id)
  if not ok then return nil, 'gene placement refused: ' .. tostring(equip_why) end
  e.gene = id
  self.owned[e.host] = {gene = id, slot = slot, modifiers = {}}
  return true
end

function RuntimeEncounters:apply_host_modifier(e, modifier)
  local run = self.get_run()
  if not run then return nil, 'no active run' end
  local slot = e.slot or 'assault'
  local ok, why = self.Core.apply_modifier(run, e.host, slot, modifier)
  if not ok then return nil, tostring(why) end
  local info = self.owned[e.host]
  if info then info.modifiers[modifier.id] = true end
  return true
end

function RuntimeEncounters:release_gene(e)
  local info = self.owned[e.host]
  if not info then return end
  local run = self.get_run()
  if not run then return end
  for id in pairs(info.modifiers or {}) do pcall(self.Core.remove_modifier, run, e.host, info.slot, id) end
  pcall(self.Core.equip, run, e.host, info.slot, nil)
end

-- Spawn/activate one entity transactionally. The caller rolls the whole begin
-- back on any refusal, so this leaves no half-owned actor or gene host behind.
function RuntimeEncounters:spawn_entity(e)
  if e.spawned then return true end
  if e.role == 'custom' then
    if self.live_custom >= self.max_custom then return nil, 'custom actor budget exhausted' end
    local x, y = self:spawn_point(e)
    local handle, why = self.gd.spawn_enemy(e.kind, x, y, {facing = -1})
    if not handle then return nil, 'spawn refused for ' .. e.id .. ': ' .. tostring(why) end
    -- Pre-register the host so a refusal after the adapter borrowed the slot is
    -- still torn down by the begin rollback.
    self.owned[e.host] = self.owned[e.host]
      or {slot = 'assault', modifiers = {encounter_cost = true, encounter_power = true}}
    local ok, view, reason = pcall(self.controller.attach, self.controller, handle,
      {host = e.host, family = e.family, slot = 'assault'})
    if not ok or not view then
      pcall(self.gd.enemy_remove, handle)
      return nil, 'gene host refused for ' .. e.id .. ': ' .. tostring(ok and (reason or 'attach failed') or view)
    end
    local run = self.get_run()
    if run and run.hosts[e.host] and run.hosts[e.host].slots.assault then
      local id = run.hosts[e.host].slots.assault
      self.acquired[id] = true
      self.owned[e.host].gene = id
    end
    e.handle = handle
    e.spawned = true
    self.live_custom = self.live_custom + 1
    self.by_handle[handle] = e
    return true
  end
  if self.active_fighter then return nil, 'only one fighter CPU can be active at a time' end
  -- Varied opponents use the gene system: ordinary fighters and champions both
  -- own a real Core host. buff_fighters=false is the only explicit opt-out.
  if e.kind == 'champion' or self.buff_fighters then
    local gok, gwhy = self:ensure_gene(e)
    if not gok then return nil, gwhy end
    local mok, mwhy = self:apply_host_modifier(e, {id = 'encounter', stat = 'potency', add = e.buff or 0})
    if not mok then self:release_gene(e); return nil, 'encounter modifier refused for ' .. e.id .. ': ' .. mwhy end
  end
  if self.tech then
    local ok, why = pcall(self.tech.configure, self.fighter_port, self:skill(e.level), self:seed_for(e))
    if not ok then
      self:release_gene(e)
      return nil, 'technical assist refused for ' .. e.id .. ': ' .. tostring(why)
    end
    if why == false then self:note('technical assist unavailable; native CPU baseline retained') end
  end
  if self.gd.cpu_mode then pcall(self.gd.cpu_mode, self.fighter_port, 'fight') end
  e.spawned = true
  self.active_fighter = e
  return true
end

function RuntimeEncounters:spawn_wave(index)
  for _, id in ipairs(self.active_plan.waves[index] or {}) do
    local e = self.by_id[id]
    if e and not self.progress.defeated[id] then
      local ok, why = self:spawn_entity(e)
      if not ok then return nil, why end
    end
  end
  return true
end

-- Begin the encounter. Resumes from saved progress.defeated: already-defeated
-- entities are not spawned and the first live wave is selected. The whole
-- transaction is rolled back (actors, gene hosts, modifiers) on any refusal.
function RuntimeEncounters:begin(node, progress)
  if self.active then return nil, 'an encounter is already active' end
  if type(progress) ~= 'table' then return nil, 'schema-2 progress record required' end
  local ok, planned = self:plan(node)
  if not ok then return nil, planned end
  local vok, vwhy = self.progress_module.validate(progress)
  if not vok then return nil, 'invalid progress: ' .. tostring(vwhy) end
  if not progress.visited[planned.room] then return nil, 'room is not visited in progress' end
  self.active_plan, self.node, self.progress = planned, node, progress
  for _, e in ipairs(planned.entities) do
    e.remaining = e.stocks
    self.entities[#self.entities + 1] = e
    self.by_id[e.id] = e
  end
  self.waves = planned.waves
  local floor = node.room and node.room.floor
  if not self.bounds and floor then
    self.bounds = {min_x = floor.left - 80, max_x = floor.right + 80,
      min_y = floor.y - 160, max_y = floor.y + 240}
  end
  -- A single fighter's saved encounter_kos lets a partially fought boss resume.
  if planned.fighters == 1 then
    for _, e in ipairs(planned.entities) do
      if e.role == 'fighter' and progress.encounter_kos[planned.room] then
        local killed = math.min(e.stocks or 1, progress.encounter_kos[planned.room])
        e.remaining = math.max(1, (e.stocks or 1) - killed)
      end
    end
  end
  local start
  for wi = 1, #planned.waves do
    for _, id in ipairs(planned.waves[wi]) do
      if not progress.defeated[id] then start = wi break end
    end
    if start then break end
  end
  if not start then
    self.active, self.cleared = true, true
    return true, {{kind = 'clear', room = planned.room, reason = 'encounter already complete'}}
  end
  self.wave = start
  local sok, serr = self:spawn_wave(start)
  if not sok then
    -- Roll the partial spawn back. If native removal keeps refusing, keep the
    -- pending ownership instead of erasing it, and report both.
    local tok, terr = self:teardown()
    if tok then
      self:reset()
      return nil, serr
    end
    self.error = terr
    return nil, serr .. '; cleanup pending: ' .. tostring(terr)
  end
  self.active = true
  return true, {}
end

-- Bounded recovery for an actor that vanished or left the camera. It never
-- grants a defeat or a clear; it respawns on the same stable entity id within
-- budget and then raises a hard error so a run cannot strand silently.
function RuntimeEncounters:recover_actor(e, reason)
  if e.respawns >= self.max_respawns then
    if not e.error_reported then
      e.error_reported = true
      self.error = 'required actor ' .. e.id .. ' ' .. reason .. ' and the respawn budget is exhausted'
      return {kind = 'error', room = self.active_plan and self.active_plan.room, entity = e.id,
        reason = 'actor ' .. reason .. '; respawn budget exhausted'}
    end
    return nil
  end
  pcall(self.controller.detach, self.controller, e.handle)
  if e.handle and self.gd.enemy_remove then pcall(self.gd.enemy_remove, e.handle) end
  self.by_handle[e.handle] = nil
  e.handle, e.spawned = nil, false
  self.live_custom = math.max(0, self.live_custom - 1)
  e.respawns = e.respawns + 1
  local ok, why = self:spawn_entity(e)
  if not ok then
    self.error = why
    return {kind = 'error', room = self.active_plan and self.active_plan.room, entity = e.id, reason = why}
  end
  return {kind = 'respawn', room = self.active_plan and self.active_plan.room, entity = e.id, attempt = e.respawns}
end

function RuntimeEncounters:escaped(q)
  local b = self.bounds
  if not b or type(q) ~= 'table' or type(q.x) ~= 'number' or type(q.y) ~= 'number' then return false end
  return q.x < b.min_x or q.x > b.max_x or q.y < b.min_y or q.y > b.max_y
end

function RuntimeEncounters:wave_done(index)
  for _, id in ipairs(self.waves[index] or {}) do
    if not self.progress.defeated[id] then return false end
  end
  return true
end

function RuntimeEncounters:advance()
  local wi = self.wave
  while true do
    wi = wi + 1
    if wi > #self.waves then
      if self.cleared then return nil end
      self.cleared = true
      return {kind = 'clear', room = self.active_plan.room}
    end
    local done = true
    for _, id in ipairs(self.waves[wi]) do
      if not self.progress.defeated[id] then done = false break end
    end
    if not done then break end
  end
  self.wave = wi
  local ok, why = self:spawn_wave(wi)
  if not ok then
    self.error = why
    return {kind = 'error', room = self.active_plan.room, reason = why}
  end
  return {kind = 'wave', room = self.active_plan.room, wave = wi}
end

-- Per-frame update: tick the gene controller, detect vanished/escaped custom
-- actors, and advance waves whose entities are all confirmed defeated.
function RuntimeEncounters:update(progress)
  if not self.active then return true, {} end
  if progress then self.progress = progress end
  local events = {}
  self.controller:tick()
  for _, e in ipairs(self.active_plan.entities) do
    if e.spawned and e.role == 'custom' and not self.progress.defeated[e.id] then
      local q = self.gd.enemy_state and self.gd.enemy_state(e.handle)
      local vanished = (q == nil) or (q.alive == false)
      local escaped = (not vanished) and self:escaped(q)
      if vanished or escaped then
        local ev = self:recover_actor(e, vanished and 'vanished' or 'escaped')
        if ev then events[#events + 1] = ev end
      end
    end
  end
  if not self.error then
    local done = true
    for _, id in ipairs(self.waves[self.wave] or {}) do
      if not self.progress.defeated[id] then done = false break end
    end
    if done then
      local ev = self:advance()
      if ev then events[#events + 1] = ev end
    end
  end
  return true, events
end

-- Provenance gate for an authoritative incoming contact. Maps a native handle
-- to its stable entity and rejects stale handles and duplicate contact ids.
function RuntimeEncounters:hit(event)
  if type(event) ~= 'table' then return {handled = false, reason = 'invalid contact'} end
  if not self.active or not self.progress then return {handled = false, reason = 'no active encounter'} end
  local e = event.handle ~= nil and self.by_handle[event.handle]
  if not e or not e.spawned then return {handled = false, reason = 'stale or unknown contact'} end
  if self.progress.defeated[e.id] then return {handled = false, reason = 'actor already defeated'} end
  if event.id ~= nil then
    if count(self.contacts) >= 256 then self.contacts = {} end
    local key = tostring(event.id)
    if self.contacts[key] then return {handled = false, reason = 'duplicate contact'} end
    self.contacts[key] = true
  end
  return {handled = true, entity = e.id, host = e.host, role = e.role, from = event.from, damage = event.damage}
end

-- Confirmed defeat for a custom actor. Idempotent; stale handles are harmless.
function RuntimeEncounters:defeat(input, progress)
  if type(input) == 'table' then input = input.handle end
  if progress then self.progress = progress end
  if not self.active or not self.progress then return nil, 'no active encounter' end
  local e = self.by_handle[input]
  if not e then return true, {{kind = 'ignored', reason = 'stale or unknown actor handle'}} end
  if self.progress.defeated[e.id] then return true, {{kind = 'ignored', reason = 'duplicate defeat'}} end
  if e.role ~= 'custom' then
    return true, {{kind = 'ignored', reason = 'defeat evidence is not valid for a fighter entity'}}
  end
  local ok, why = self.progress_module.defeat(self.progress, e.id)
  if not ok then return nil, why end
  pcall(self.controller.detach, self.controller, e.handle)
  if e.handle and self.gd.enemy_remove then pcall(self.gd.enemy_remove, e.handle) end
  self.by_handle[e.handle] = nil
  e.handle, e.spawned = nil, false
  self.live_custom = math.max(0, self.live_custom - 1)
  local events = {{kind = 'defeat', room = self.active_plan.room, entity = e.id}}
  if self:wave_done(self.wave) then
    local ev = self:advance()
    if ev then events[#events + 1] = ev end
  end
  return true, events
end

-- Observed native stock transition for the active fighter. Duplicate or
-- non-decreasing observations are ignored; the defeat is only confirmed when
-- the entity's explicit remaining counter reaches zero.
function RuntimeEncounters:stock(port, before, after, progress)
  if progress then self.progress = progress end
  if not self.active or not self.progress then return nil, 'no active encounter' end
  if type(port) ~= 'number' or port ~= self.fighter_port then
    return true, {{kind = 'ignored', reason = 'not the fighter port'}}
  end
  if type(before) ~= 'number' or type(after) ~= 'number' or after >= before then
    return true, {{kind = 'ignored', reason = 'no observed stock transition'}}
  end
  local e = self.active_fighter
  if not e or self.progress.defeated[e.id] then
    return true, {{kind = 'ignored', reason = 'stale fighter stock transition'}}
  end
  local losses = before - after
  if losses % 1 ~= 0 then return true, {{kind = 'ignored', reason = 'invalid stock transition'}} end
  local room = self.active_plan.room
  local kos = math.min(16, (self.progress.encounter_kos[room] or 0) + losses)
  self.progress.encounter_kos[room] = kos
  e.remaining = math.max(0, (e.remaining or e.stocks or 1) - losses)
  -- `phase` is only the observed stock phase (1-based KOs), not the catalogue's
  -- authored multi-phase mechanic, which this port does not implement.
  local events = {{kind = 'stock', room = room, entity = e.id, lost = losses,
    remaining = e.remaining, phase = kos + 1}}
  if e.remaining <= 0 then
    local ok, why = self.progress_module.defeat(self.progress, e.id)
    if not ok then return nil, why end
    self:release_gene(e)
    e.spawned = false
    self.active_fighter = nil
    events[#events + 1] = {kind = 'defeat', room = room, entity = e.id}
    if self:wave_done(self.wave) then
      local ev = self:advance()
      if ev then events[#events + 1] = ev end
    end
  end
  return true, events
end

function RuntimeEncounters:gene_unused(run, gene)
  for _, h in pairs(run.hosts) do
    for _, id in pairs(h.slots) do if id == gene then return false end end
  end
  return true
end

function RuntimeEncounters:remove_actor(e)
  for _ = 1, self.cleanup_attempts do
    local ok, result = pcall(self.gd.enemy_remove, e.handle)
    if ok and result then return true end
    if self.gd.enemy_alive then
      local aok, alive = pcall(self.gd.enemy_alive, e.handle)
      if aok and not alive then return true end
    end
  end
  return nil, 'could not remove native actor ' .. e.id
end

-- Native/controller/gene teardown. Removal failures are retried up to the
-- configured budget; whatever still cannot be released stays owned and is
-- returned as `pending` so a caller can retry without losing track of it.
function RuntimeEncounters:teardown()
  local pending = {}
  for _, e in ipairs(self.entities) do
    if e.role == 'custom' and e.spawned and e.handle then
      local ok, why = self:remove_actor(e)
      if ok then
        e.handle, e.spawned = nil, false
      else
        pending[#pending + 1] = {kind = 'actor', entity = e.id, handle = e.handle, reason = why}
      end
    end
  end
  if self.controller then self.controller:clear() end
  local run = self.get_run()
  for host, info in pairs(self.owned) do
    if run then
      for id in pairs(info.modifiers or {}) do pcall(self.Core.remove_modifier, run, host, info.slot, id) end
      pcall(self.Core.equip, run, host, info.slot, nil)
      local gene = info.gene
      if gene and self.acquired[gene] and self:gene_unused(run, gene) then
        run.genes[gene] = nil
        run.runtime[gene] = nil
      end
      run.hosts[host] = nil
      self.owned[host] = nil
    else
      -- No active run to release the Core host: keep ownership for a retry.
      pending[#pending + 1] = {kind = 'host', host = host, reason = 'no active run for gene host cleanup'}
    end
  end
  if #pending > 0 then
    self.error = tostring(#pending) .. ' owned actor/host cleanup(s) pending'
    return nil, self.error, pending
  end
  self.live_custom, self.active_fighter, self.by_handle = 0, nil, {}
  self.owned, self.acquired = {}, {}
  return true
end

-- Public clear: tear down every owned actor and gene host, then drop state.
-- On persistent refusal it keeps the pending ownership rather than erasing it,
-- returning nil, reason, pending. Retry clear() after the engine recovers.
function RuntimeEncounters:clear()
  local ok, why, pending = self:teardown()
  if not ok then return nil, why, pending end
  self:reset()
  return true
end

function RuntimeEncounters:states() return self.controller:states() end

-- Read-only stable-id view of the active composition, for diagnostics and the
-- runtime's own bookkeeping. Never returns native handles as a primary key.
function RuntimeEncounters:composition()
  local out = {}
  if self.active_plan then
    for _, e in ipairs(self.active_plan.entities) do
      out[#out + 1] = {id = e.id, kind = e.kind, role = e.role, family = e.family, slot = e.slot,
        wave = e.wave, host = e.host, buff = e.buff, stocks = e.stocks, remaining = e.remaining,
        spawned = e.spawned, handle = e.handle, respawns = e.respawns,
        defeated = (self.progress and self.progress.defeated[e.id] == true) or false}
    end
  end
  return out
end

-- The active fighter entity plus its real Core gene host, so main can drive
-- ability/activation/provenance exactly as it does for native fighter opponents.
function RuntimeEncounters:active_entity()
  local e = self.active_fighter
  if not e then return nil end
  return {id = e.id, kind = e.kind, role = e.role, remaining = e.remaining, host = e.host,
    family = e.family, slot = e.slot, buff = e.buff, defeated = false}
end

-- Explicit current eligibility: which gene families/slots the Core definitions
-- actually implement and the resolved policy this instance uses.
function RuntimeEncounters:eligibility()
  local families, slots = {}, {}
  for k in pairs(RuntimeEncounters.families) do families[#families + 1] = k end
  for k in pairs(RuntimeEncounters.slots) do slots[#slots + 1] = k end
  table.sort(families); table.sort(slots)
  return {families = families, slots = slots, default_fighter_family = RuntimeEncounters.default_fighter_family,
    fighter_family = self.fighter_family, fighter_slot = self.fighter_slot,
    buff_fighters = self.buff_fighters, fighter_potency = self.fighter_potency,
    champion_potency = self.champion_potency}
end

function RuntimeEncounters:status()
  local defeated, entities = 0, 0
  if self.active_plan then
    for _, e in ipairs(self.active_plan.entities) do
      entities = entities + 1
      if self.progress and self.progress.defeated[e.id] then defeated = defeated + 1 end
    end
  end
  return {active = self.active, room = self.active_plan and self.active_plan.room, wave = self.wave,
    waves = self.active_plan and #self.active_plan.waves or 0, entities = entities, defeated = defeated,
    cleared = self.cleared, error = self.error, notice = self.notice,
    fighter = self:active_entity(), eligibility = self:eligibility()}
end

return RuntimeEncounters
