-- Gate 6 boss controller layer. Three distinct champion controllers, each a
-- phase machine with mechanical phase transitions, attack selection, readable
-- tells, vulnerability/recovery windows and positioning demands. Built on the
-- pure encounter_behaviors decision boundary: a boss is an ordinary decision
-- agent with a custom brain, so it inherits vision/reaction fairness, the
-- simultaneous-tell arbiter, Core-legal ability requests and bounded cleanup.
--
-- It exports a separate, versioned progress addon for partially-fought bosses.
-- run progress (progress.lua) rejects unknown fields, so this addon is a
-- sibling record for the integration worker to persist; progress.lua is not
-- touched. The addon refuses to award a boss reward twice.
local BossBehaviors = {version = 1, max_phases = 8}

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end
local function integer(x, lo, hi) return finite(x) and x % 1 == 0 and x >= lo and x <= hi end
local function valid_id(x) return type(x) == 'string' and #x > 0 and #x <= 64 and not x:find('[%z\1-\31]') end
local function copy(t) if type(t) ~= 'table' then return t end local o = {} for k, v in pairs(t) do o[k] = copy(v) end return o end
local function count(t) local n = 0 for _ in pairs(t) do n = n + 1 end return n end

-- Phases are ordered. Every phase changes at least one of motion, spacing band,
-- attack slots, tell/recovery length, vulnerability window and positioning
-- anchor; a phase that only changed HP would not qualify.
BossBehaviors.controllers = {
  warden = {
    id = 'warden', name = 'Cinder Warden', family = 'cinder', host = 'fighter',
    signature = 'range -> rush -> overdrive; only its committed heavies are punishable',
    phases = {
      {id = 'measure', motion = 'spacing', band = {min = 28, max = 72}, tell = 20, recovery = 18,
        attack_slots = {'assault'}, attack_choice = 'single',
        positioning = {anchor = 'center', distance = {min = 28, max = 72}, demand = 'close the midlane'},
        vulnerability = {after_attack = 18}, advance = {kos = 0}},
      {id = 'advance', motion = 'advance', band = {min = 1, max = 40}, tell = 16, recovery = 16,
        attack_slots = {'traversal', 'assault'}, attack_choice = 'alternate',
        positioning = {anchor = 'center', distance = {min = 0, max = 40}, demand = 'keep a diagonal escape lane'},
        vulnerability = {after_attack = 14}, advance = {kos = 1}},
      {id = 'overdrive', motion = 'hold', band = {min = 0, max = 56}, tell = 24, recovery = 24,
        attack_slots = {'assault'}, attack_choice = 'range',
        positioning = {anchor = 'side_left', distance = {min = 0, max = 56}, demand = 'fight off the left wall'},
        vulnerability = {after_attack = 30, on_whiff = true}, advance = {kos = 2}},
    },
  },
  glacier = {
    id = 'glacier', name = 'Rime Bulwark', family = 'rime', host = 'fighter',
    signature = 'bulwark -> shatter -> undertow; baits a guard punish before it opens',
    phases = {
      {id = 'bulwark', motion = 'hold', band = {min = 0, max = 44}, tell = 22, recovery = 20,
        attack_slots = {'guard', 'assault'}, attack_choice = 'alternate',
        positioning = {anchor = 'center', distance = {min = 0, max = 44}, demand = 'bait the counter, then punish its recovery'},
        vulnerability = {on_recovery = 16}, advance = {hits = 0}},
      {id = 'shatter', motion = 'advance', band = {min = 1, max = 52}, tell = 18, recovery = 26,
        attack_slots = {'assault'}, attack_choice = 'single',
        positioning = {anchor = 'center', distance = {min = 0, max = 52}, demand = 'outrun the mark chain'},
        vulnerability = {after_attack = 22}, advance = {hits = 2}},
      {id = 'undertow', motion = 'spacing', band = {min = 20, max = 64}, tell = 28, recovery = 20,
        attack_slots = {'guard'}, attack_choice = 'range',
        positioning = {anchor = 'side_right', distance = {min = 20, max = 64}, demand = 'hold the right platform'},
        vulnerability = {after_attack = 26, on_whiff = true}, advance = {hits = 4}},
    },
  },
  tempest = {
    id = 'tempest', name = 'Rime Tempest', family = 'rime', host = 'fighter',
    signature = 'circle -> dive -> grounded; forces an aerial/lower-platform answer',
    phases = {
      {id = 'circle', motion = 'aerial', band = {min = 0, max = 160}, tell = 18, recovery = 14,
        attack_slots = {'assault'}, attack_choice = 'single',
        positioning = {anchor = 'air', distance = {min = 0, max = 160}, demand = 'do not idle airborne near it'},
        vulnerability = {after_attack = 12}, advance = {kos = 0}},
      {id = 'dive', motion = 'aerial', band = {min = 0, max = 200}, tell = 14, recovery = 22,
        attack_slots = {'assault', 'traversal'}, attack_choice = 'alternate',
        positioning = {anchor = 'above', distance = {min = 0, max = 80}, demand = 'stay off the centre line'},
        vulnerability = {after_attack = 26}, advance = {player_offstage = true}},
      {id = 'grounded', motion = 'hold', band = {min = 0, max = 48}, tell = 26, recovery = 30,
        attack_slots = {'guard'}, attack_choice = 'alternate',
        positioning = {anchor = 'center', distance = {min = 0, max = 48}, demand = 'pounce during the long landing window'},
        vulnerability = {after_attack = 34}, advance = {hits = 3}},
    },
  },
}

-- Boss rooms map the existing champion compositions to a controller. Family is
-- kept consistent with the encounter catalogue (gale keeps the cinder family).
BossBehaviors.compositions = {
  champ_cinder = {id = 'champ_cinder', controller = 'warden', family = 'cinder', level = 8, lane = 0, height = 0},
  champ_rime = {id = 'champ_rime', controller = 'glacier', family = 'rime', level = 8, lane = 0, height = 0},
  champ_gale = {id = 'champ_gale', controller = 'tempest', family = 'cinder', level = 8, lane = 0, height = 0},
}

local function anchor_x(obs, anchor)
  local b = obs and obs.bounds
  if not b then return nil end
  local center = finite(b.center_x) and b.center_x or ((b.min_x + b.max_x) / 2)
  local half = (b.max_x - b.min_x) / 2
  if anchor == 'side_left' then return center - half * 0.5 end
  if anchor == 'side_right' then return center + half * 0.5 end
  return center
end

-- Movement/attack/vulnerability policy for the current phase. Writes only the
-- per-agent override fields the archetype decision reads; the shared controller
-- data is never mutated.
local function apply_phase(agent, phase, obs)
  agent.motion = phase.motion
  agent.band = phase.band
  agent.tell_frames = phase.tell
  agent.recovery_frames = phase.recovery
  agent.phase_choices = phase.attack_slots
  agent.attack_choice = phase.attack_choice
  agent.vulnerable_after = phase.vulnerability and phase.vulnerability.after_attack or nil
  agent.vulnerable_on_whiff = phase.vulnerability and phase.vulnerability.on_whiff and 24 or nil
  local ax = anchor_x(obs, phase.positioning and phase.positioning.anchor)
  if ax then agent.spawn = {x = ax, y = obs and obs.bounds and obs.bounds.max_y or 0} end
  agent.positioning = phase.positioning
end

local function phase_ready(phase, agent, obs, frame)
  local a = phase.advance
  if not a then return false end
  if a.kos and (obs.self.kos or 0) >= a.kos then return true end
  if a.hits and (obs.self.hits_taken or 0) >= a.hits then return true end
  if a.time and (frame - (agent.phase_start or frame)) >= a.time then return true end
  for _, t in ipairs(obs.targets or {}) do
    if a.player_offstage and t.offstage == true then return true end
    if a.player_airborne and t.airborne == true then return true end
  end
  return false
end

function BossBehaviors.make_brain(controller)
  return function(agent, obs, frame, manager)
    local events = {}
    if not obs or not obs.self then return nil, events end
    agent.phase = agent.phase or 1
    if not agent.phase_start then agent.phase_start = frame end
    if agent.phase > #controller.phases then agent.phase = #controller.phases end
    local phase = controller.phases[agent.phase]
    -- advance forward only, and only on an observed condition
    for i = agent.phase + 1, #controller.phases do
      if phase_ready(controller.phases[i], agent, obs, frame) then
        agent.phase = i
        agent.phase_start = frame
        agent.phase_kos = obs.self.kos or agent.phase_kos or 0
        agent.phase_hits = obs.self.hits_taken or agent.phase_hits or 0
        agent.tell_until, agent.tell_ready = nil, false
        agent.vulnerable_until = nil
        manager.arbiter:release(agent.id, frame)
        events[#events + 1] = {kind = 'phase', actor = agent.id, controller = controller.id,
          phase = i, phase_id = controller.phases[i].id}
        phase = controller.phases[i]
        break
      end
    end
    apply_phase(agent, phase, obs)
    local req, evs = manager:archetype_decide(agent, obs, frame)
    for _, e in ipairs(evs or {}) do events[#events + 1] = e end
    for _, e in ipairs(evs or {}) do
      if e.kind == 'tell_abort' and phase.vulnerability and phase.vulnerability.on_recovery then
        agent.vulnerable_until = frame + phase.vulnerability.on_recovery
        agent.vulnerable_reason = 'recovery'
        events[#events + 1] = {kind = 'vulnerable', actor = agent.id, reason = 'recovery', until_frame = agent.vulnerable_until}
      end
    end
    return req, events
  end
end

-- Attach a boss to an existing encounter_behaviors manager. `actor.phase` and
-- the observed counters let a partially fought boss resume its phase.
function BossBehaviors.attach(manager, actor, controller_id)
  if type(manager) ~= 'table' or type(manager.add) ~= 'function' then return nil, 'manager required' end
  local c = BossBehaviors.controllers[controller_id]
  if not c then return nil, 'unknown boss controller ' .. tostring(controller_id) end
  if type(actor) ~= 'table' or not valid_id(actor.id) then return nil, 'invalid boss actor' end
  local added, why = manager:add({
    id = actor.id, handle = actor.handle, host = actor.host, slot = actor.slot or 'assault',
    family = actor.family or c.family, archetype = actor.archetype or 'boss', level = actor.level,
    seed = actor.seed, lane = actor.lane, height = actor.height,
  })
  if not added then return nil, why end
  local agent = manager.by_id[actor.id]
  agent.brain = BossBehaviors.make_brain(c)
  agent.controller = controller_id
  agent.phase = integer(actor.phase, 1, #c.phases) and actor.phase or 1
  agent.phase_start = nil
  agent.phase_kos = actor.phase_kos or actor.kos or 0
  agent.phase_hits = actor.phase_hits or actor.hits or 0
  agent.completed = false
  return manager:view(agent)
end

function BossBehaviors.resume(manager, actor, controller_id, entry)
  local c = BossBehaviors.controllers[controller_id]
  if not c then return nil, 'unknown boss controller' end
  local ok, why = BossBehaviors.progress_validate({version = 1, bosses = {[actor.id] = entry}})
  if not ok then return nil, why end
  local view, err = BossBehaviors.attach(manager, {
    id = actor.id, handle = actor.handle, host = actor.host, slot = actor.slot,
    family = entry.family or actor.family, level = actor.level, seed = actor.seed,
    lane = actor.lane, height = actor.height, phase = entry.phase,
    phase_kos = entry.kos, phase_hits = entry.hits,
  }, controller_id)
  return view, err
end

-- Read-only boss view for diagnostics and readability checks.
function BossBehaviors.state(manager, id, frame)
  local agent = manager.by_id and manager.by_id[id]
  if not agent or not agent.controller then return nil end
  local c = BossBehaviors.controllers[agent.controller]
  local phase = c.phases[agent.phase]
  return {
    id = id, controller = agent.controller, name = c.name, family = agent.family,
    phase = agent.phase, phases = #c.phases, phase_id = phase.id,
    phase_frames = (frame or manager.frame) - (agent.phase_start or frame or manager.frame),
    motion = phase.motion, positioning = phase.positioning,
    tell_frames = phase.tell, recovery_frames = phase.recovery,
    vulnerable = agent.vulnerable_until ~= nil and (frame or manager.frame) < agent.vulnerable_until,
    vulnerable_reason = agent.vulnerable_reason,
    completed = agent.completed == true,
  }
end

-- ------------------------------------------------------------ progress addon
-- Separate from progress.lua (which rejects unknown fields). Persist this
-- record in a checkpoint sidecar/next schema; progress.lua stays frozen.
BossBehaviors.progress_addon = {version = 1, max_bosses = 8}
local BOSS_FIELDS = {controller = true, family = true, phase = true, phase_frames = true,
  kos = true, hits = true, completed = true, rewarded = true}

function BossBehaviors.progress_new()
  return {version = 1, bosses = {}}
end

function BossBehaviors.progress_validate(state)
  if type(state) ~= 'table' or state.version ~= 1 or type(state.bosses) ~= 'table' then
    return false, 'invalid boss progress record'
  end
  for k in pairs(state) do
    if k ~= 'version' and k ~= 'bosses' then return false, 'unknown boss progress field ' .. tostring(k) end
  end
  local n = 0
  for room, entry in pairs(state.bosses) do
    n = n + 1
    if not valid_id(room) then return false, 'invalid boss room id' end
    if type(entry) ~= 'table' then return false, 'invalid boss entry' end
    for k in pairs(entry) do if not BOSS_FIELDS[k] then return false, 'unknown boss field ' .. tostring(k) end end
    local c = BossBehaviors.controllers[entry.controller]
    if not c then return false, 'unknown boss controller ' .. tostring(entry.controller) end
    if not integer(entry.phase, 1, #c.phases) then return false, 'invalid boss phase' end
    if entry.family ~= nil and (entry.family ~= 'cinder' and entry.family ~= 'rime') then return false, 'invalid boss family' end
    if not integer(entry.phase_frames, 0, 1000000) then return false, 'invalid phase frames' end
    if not integer(entry.kos, 0, 99) or not integer(entry.hits, 0, 999) then return false, 'invalid boss counters' end
    if type(entry.completed) ~= 'boolean' or type(entry.rewarded) ~= 'boolean' then return false, 'invalid boss flags' end
  end
  if n > BossBehaviors.progress_addon.max_bosses then return false, 'too many boss entries' end
  return true
end

function BossBehaviors.progress_record(state, room, entry)
  if type(state) ~= 'table' or state.version ~= 1 or type(state.bosses) ~= 'table' then
    return nil, 'invalid boss progress record'
  end
  if not valid_id(room) then return nil, 'invalid boss room' end
  local existing = state.bosses[room]
  if existing and existing.rewarded and not entry.rewarded then return nil, 'boss reward already claimed' end
  state.bosses[room] = entry
  local ok, why = BossBehaviors.progress_validate(state)
  if not ok then state.bosses[room] = existing; return nil, why end
  return true
end

-- Update the addon from a live boss agent. `completed` is only set from an
-- observed defeat, never from a timeout or a phase change.
function BossBehaviors.progress_sync(state, room, manager, id, frame)
  local agent = manager.by_id and manager.by_id[id]
  if not agent or not agent.controller then return nil, 'not a boss agent' end
  local entry = {
    controller = agent.controller, family = agent.family, phase = agent.phase or 1,
    phase_frames = (frame or manager.frame) - (agent.phase_start or frame or manager.frame),
    kos = agent.phase_kos or 0, hits = agent.phase_hits or 0,
    completed = agent.completed == true, rewarded = false,
  }
  local prev = state.bosses and state.bosses[room]
  if prev then entry.rewarded = prev.rewarded == true end
  return BossBehaviors.progress_record(state, room, entry)
end

function BossBehaviors.progress_phase(state, room)
  local e = state.bosses and state.bosses[room]
  return e and e.phase or nil
end

-- Duplicate-reward gate. Refuses until the boss is completed and refuses a
-- second claim; the caller persists before granting.
function BossBehaviors.progress_reward_eligible(state, room)
  local e = state.bosses and state.bosses[room]
  return e ~= nil and e.completed == true and e.rewarded ~= true
end
function BossBehaviors.progress_claim_reward(state, room)
  local e = state.bosses and state.bosses[room]
  if not e then return nil, 'no such boss' end
  if e.rewarded then return nil, 'boss reward already claimed' end
  if not e.completed then return nil, 'boss not completed' end
  local snapshot = copy(e)
  e.rewarded = true
  local ok, why = BossBehaviors.progress_validate(state)
  if not ok then state.bosses[room] = snapshot; return nil, why end
  return true
end

function BossBehaviors.progress_encode(state, Codec)
  if type(Codec) ~= 'table' or type(Codec.encode) ~= 'function' then return nil, 'Codec required' end
  local ok, why = BossBehaviors.progress_validate(state)
  if not ok then return nil, why end
  return Codec.encode(state)
end
function BossBehaviors.progress_decode(text, Codec)
  if type(Codec) ~= 'table' or type(Codec.decode) ~= 'function' then return nil, 'Codec required' end
  local decoded, why = Codec.decode(text)
  if not decoded then return nil, why end
  local ok, reason = BossBehaviors.progress_validate(decoded)
  if not ok then return nil, reason end
  return decoded
end

-- --------------------------------------------------------------- validation
function BossBehaviors.validate(Core, EncounterCatalogue)
  if type(Core) ~= 'table' or type(Core.definitions) ~= 'table' then return false, 'Core definitions required' end
  local n = 0
  for id, c in pairs(BossBehaviors.controllers) do
    n = n + 1
    if id ~= c.id or (c.host ~= 'fighter' and c.host ~= 'custom') then return false, 'invalid boss controller ' .. tostring(id) end
    if type(c.phases) ~= 'table' or #c.phases < 3 or #c.phases > BossBehaviors.max_phases then
      return false, 'boss needs at least three phases'
    end
    local signatures = {}
    for _, phase in ipairs(c.phases) do
      if not valid_id(phase.id) then return false, 'invalid phase id' end
      if not integer(phase.tell, 4, 60) or not integer(phase.recovery, 1, 60) then return false, 'invalid phase timing' end
      if type(phase.attack_slots) ~= 'table' or #phase.attack_slots < 1 then return false, 'phase needs attack slots' end
      if type(phase.vulnerability) ~= 'table' then return false, 'phase needs a vulnerability window' end
      if type(phase.positioning) ~= 'table' or not valid_id(phase.positioning.anchor) then return false, 'phase needs positioning' end
      if not phase.advance or count(phase.advance) < 1 then return false, 'phase needs an observed advance condition' end
      local sig = phase.motion .. '|' .. phase.tell .. '|' .. phase.recovery .. '|' .. table.concat(phase.attack_slots, ',')
      signatures[sig] = (signatures[sig] or 0) + 1
    end
    if count(signatures) < 2 then return false, 'boss phases are not mechanically distinct' end
  end
  if n ~= 3 then return false, 'exactly three boss controllers required' end
  local comps = 0
  for id, comp in pairs(BossBehaviors.compositions) do
    comps = comps + 1
    if comp.id ~= id or not BossBehaviors.controllers[comp.controller] then return false, 'invalid boss composition ' .. tostring(id) end
    if not Core.definitions[comp.family] then return false, 'unsupported boss family ' .. tostring(comp.family) end
    if EncounterCatalogue and EncounterCatalogue.encounters then
      local live = EncounterCatalogue.encounters[id]
      if not live then return false, 'boss composition has no encounter row ' .. id end
    end
  end
  if comps ~= 3 then return false, 'exactly three boss compositions required' end
  return true
end

return BossBehaviors
