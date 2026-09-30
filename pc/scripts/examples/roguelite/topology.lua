-- Variable connected topology generator (schema v2). Builds a mission graph:
-- a mandatory spine with a deliberate split/merge branch, optional returning
-- detours and forward shortcuts, resolved through explicit socket bindings.
--
-- Pure: every choice comes from named RNG streams derived from the seed and the
-- attempt number, so the same seed and versions reproduce the exact graph.
-- Dependencies (room catalogue, encounter catalogue, progression validator, rng)
-- are injected, never read from globals.
--
-- This module supersedes the fixed eight-node route in dungeon.lua. Until the
-- runtime adapter (Gate 4) consumes it, dungeon v1 remains the live path and
-- this is exercised by test_topology.py.
local Topology = {version = 2, schema_version = 2}
local MODEL = 2147483647

local function copy(t)
  if type(t) ~= 'table' then return t end
  local out = {}
  for k, v in pairs(t) do out[k] = copy(v) end
  return out
end

local ROLE_TITLE = {
  entry = 'Threshold', teach = 'First Steps', traversal = 'Passage', combat = 'Court',
  branch = 'Crossing', rest = 'Landing', reward = 'Vault', connector = 'Shortcut',
  boss = 'Champion Hall', finish = 'Run Complete',
}

function Topology.new(rooms, encounters, progression, rng)
  assert(type(rooms) == 'table' and rooms.validate, 'room catalogue required')
  assert(type(encounters) == 'table' and encounters.validate, 'encounter catalogue required')
  assert(type(rng) == 'table' and rng.new, 'rng module required')
  assert(rooms.validate() and encounters.validate())
  assert(progression == nil or progression.validate, 'invalid progression validator')
  return setmetatable({rooms = rooms, encounters = encounters, progression = progression, rng = rng}, {__index = Topology})
end

local function encounter_index(encounters)
  local index = {}
  for id, spec in pairs(encounters.encounters) do
    index[spec.archetype] = index[spec.archetype] or {}
    table.insert(index[spec.archetype], id)
  end
  for _, list in pairs(index) do table.sort(list) end
  return index
end

local function picker(usage)
  return function(self, role, min_sockets, rng, shape)
    local candidates, best
    for _, id in ipairs(self.rooms.by_role[role] or {}) do
      local template = self.rooms.rooms[id]
      if #template.sockets >= min_sockets and (shape == nil or template.shape == shape) then
        local u = usage[id] or 0
        if best == nil or u < best then best, candidates = u, {id}
        elseif u == best then candidates[#candidates + 1] = id end
      end
    end
    if not candidates then return nil end
    local id = candidates[rng:below(#candidates)]
    usage[id] = (usage[id] or 0) + 1
    return id
  end
end

local function new_id(state, prefix)
  state.serial = state.serial + 1
  return string.format('%s%03d', prefix, state.serial)
end

local function add_room(self, state, template_id, theme, mandatory)
  local template = self.rooms.rooms[template_id]
  local instance = new_id(state, 'r')
  local room = {
    id = instance, template_id = template_id, template_version = template.version,
    role = template.role, shape = template.shape, theme = theme, mandatory = mandatory or false,
    depth = 0, spine_index = nil,
    title = (self.rooms.themes[theme] and self.rooms.themes[theme].name or theme) .. ' ' .. (ROLE_TITLE[template.role] or template.role),
    sockets_by_id = {}, encounter = nil, reward = nil, grants_key = nil,
  }
  for _, socket in ipairs(template.sockets) do
    room.sockets_by_id[socket.id] = {id = socket.id, side = socket.side, edge = nil}
  end
  state.manifest.rooms_by_id[instance] = room
  state.manifest.order[#state.manifest.order + 1] = instance
  return instance
end

local function connect(self, state, from, from_socket, to, to_socket, kind, direction, gate)
  local fr, tr = state.manifest.rooms_by_id[from], state.manifest.rooms_by_id[to]
  local fs, ts = fr.sockets_by_id[from_socket], tr.sockets_by_id[to_socket]
  assert(fs and ts and not fs.edge and not ts.edge, 'socket reuse')
  state.edge_serial = state.edge_serial + 1
  local id = string.format('e%03d', state.edge_serial)
  state.manifest.edges_by_id[id] = {
    id = id, from_room = from, from_socket = from_socket, to_room = to, to_socket = to_socket,
    direction = direction or 'both', kind = kind or 'main', gate_rule = gate or nil, discovery_rule = 'always',
  }
  fs.edge, ts.edge = id, id
  state.manifest.edges_by_room[from] = state.manifest.edges_by_room[from] or {}
  state.manifest.edges_by_room[to] = state.manifest.edges_by_room[to] or {}
  table.insert(state.manifest.edges_by_room[from], id)
  table.insert(state.manifest.edges_by_room[to], id)
  return id
end

-- Build one candidate topology. Errors bubble to generate() for a documented,
-- bounded retry.
local function build(self, seed, attempt)
  local RNG = self.rng
  local base = RNG.derive(seed, 'attempt:' .. tostring(attempt))
  local topology = RNG.new(base, 'topology')
  local layout = RNG.new(base, 'layout')
  local encounter_rng = RNG.new(base, 'encounter')
  local reward_rng = RNG.new(base, 'reward')
  local lock_rng = RNG.new(base, 'lock')

  local m = {
    schema_version = Topology.schema_version, generator_version = Topology.version,
    catalogue_version = self.rooms.version, encounter_version = self.encounters.version,
    world_seed = seed, stream_versions = {topology = 1, layout = 1, encounter = 1, reward = 1, lock = 1, cosmetic = 1},
    rooms_by_id = {}, edges_by_id = {}, edges_by_room = {}, order = {}, spine = {}, locks = {},
    start_room = nil, final_room = nil,
    generation_report = {attempt_count = attempt, fallback_used = false, rejection_reasons = {}, topology_signature = ''},
  }
  local state = {manifest = m, serial = 0, edge_serial = 0}
  local usage = {}
  local pick = picker(usage)

  local themes = {'cobalt', 'frost', 'fire'}
  local primary = topology:pick(themes)
  local secondary = topology:pick(themes)
  while secondary == primary do secondary = topology:pick(themes) end
  local boss_theme = topology:pick({'frost', 'fire'})
  local function theme_for(role, index)
    if role == 'entry' or role == 'teach' or role == 'rest' or role == 'finish' then return 'cobalt' end
    if role == 'boss' or role == 'connector' then return boss_theme end
    return index % 2 == 0 and primary or secondary
  end

  local enc_index = encounter_index(self.encounters)
  local reward_ids = {}
  for id in pairs(self.encounters.rewards) do reward_ids[#reward_ids + 1] = id end
  table.sort(reward_ids)
  local lock_ids = {}
  for id in pairs(self.encounters.locks) do lock_ids[#lock_ids + 1] = id end
  table.sort(lock_ids)

  local mix = topology:range(2, 4)
  local segments = {{kind = 'single', role = 'entry'}, {kind = 'single', role = 'teach'}}
  for i = 1, mix do
    local role = (i % 2 == 1) and 'traversal' or 'combat'
    if topology:chance(1, 3) then role = role == 'combat' and 'traversal' or 'combat' end
    segments[#segments + 1] = {kind = 'single', role = role}
  end
  table.insert(segments, topology:range(2, #segments + 1), {kind = 'branch'})
  segments[#segments + 1] = {kind = 'single', role = 'rest'}
  segments[#segments + 1] = {kind = 'single', role = 'traversal'}
  segments[#segments + 1] = {kind = 'single', role = 'boss'}
  segments[#segments + 1] = {kind = 'single', role = 'finish'}

  local prev, spine_index = nil, 0
  local function template_sockets(room_id)
    return self.rooms.rooms[m.rooms_by_id[room_id].template_id].sockets
  end
  local function mark_spine(room_id)
    spine_index = spine_index + 1
    local room = m.rooms_by_id[room_id]
    room.mandatory, room.spine_index = true, spine_index
    m.spine[#m.spine + 1] = room_id
  end

  for index, segment in ipairs(segments) do
    local theme = theme_for(segment.role, index)
    if segment.kind == 'single' then
      local min = (segment.role == 'entry' or segment.role == 'finish') and 1 or 2
      local template_id = assert(pick(self, segment.role, min, layout), 'no template for role ' .. segment.role)
      local room_id = add_room(self, state, template_id, theme, true)
      local sockets = template_sockets(room_id)
      if segment.role == 'entry' then m.start_room = room_id
      elseif segment.role == 'finish' then m.final_room = room_id end
      if prev then connect(self, state, prev.room, prev.out_socket, room_id, sockets[1].id, 'main', 'both') end
      prev = {room = room_id, out_socket = (sockets[2] or sockets[1]).id}
      mark_spine(room_id)
    else
      local split_template = assert(pick(self, 'branch', 3, layout, 'split'), 'no split template')
      local merge_template = assert(pick(self, 'branch', 3, layout, 'merge'), 'no merge template')
      local split = add_room(self, state, split_template, secondary, true)
      local split_sockets = template_sockets(split)
      connect(self, state, prev.room, prev.out_socket, split, split_sockets[1].id, 'main', 'both')
      local role_a = layout:chance(1, 2) and 'combat' or 'traversal'
      local role_b = role_a == 'combat' and 'traversal' or 'combat'
      local branch_a = add_room(self, state, assert(pick(self, role_a, 2, layout)), secondary, false)
      local branch_b = add_room(self, state, assert(pick(self, role_b, 2, layout)), secondary, false)
      connect(self, state, split, split_sockets[2].id, branch_a, template_sockets(branch_a)[1].id, 'branch', 'both')
      connect(self, state, split, split_sockets[3].id, branch_b, template_sockets(branch_b)[1].id, 'branch', 'both')
      local merge = add_room(self, state, merge_template, secondary, true)
      local merge_sockets = template_sockets(merge)
      connect(self, state, branch_a, template_sockets(branch_a)[2].id, merge, merge_sockets[1].id, 'branch', 'both')
      connect(self, state, branch_b, template_sockets(branch_b)[2].id, merge, merge_sockets[2].id, 'branch', 'both')
      prev = {room = merge, out_socket = merge_sockets[3].id}
      mark_spine(split)
      mark_spine(merge)
    end
  end

  -- Optional detours: one bidirectional edge to a reward/rest room, returning
  -- along the same edge. Attached only where a spare mandatory socket exists.
  for _ = 1, layout:range(1, 3) do
    local anchors = {}
    for _, id in ipairs(m.spine) do
      for _, socket in ipairs(template_sockets(id)) do
        if not m.rooms_by_id[id].sockets_by_id[socket.id].edge then
          anchors[#anchors + 1] = {room = id, socket = socket.id}
        end
      end
    end
    if #anchors == 0 then break end
    local anchor = anchors[layout:below(#anchors)]
    local role = layout:chance(1, 2) and 'reward' or 'rest'
    local template_id = pick(self, role, 1, layout)
    if not template_id then break end
    local room_id = add_room(self, state, template_id, theme_for('connector', 0), false)
    local sockets = template_sockets(room_id)
    local edge = connect(self, state, anchor.room, anchor.socket, room_id, sockets[1].id, 'branch', 'both')
    if lock_rng:chance(1, 3) and #lock_ids > 0 then
      -- The key must sit on a mandatory room at or before the anchor, and no
      -- earlier lock may already claim that room, or its key would be lost.
      local anchor_index = m.rooms_by_id[anchor.room].spine_index
      local key_room
      for _, id in ipairs(m.spine) do
        local room = m.rooms_by_id[id]
        if room.spine_index <= anchor_index and room.grants_key == nil then key_room = id break end
      end
      if key_room then
        local lock_id = lock_ids[lock_rng:below(#lock_ids)]
        local lock = self.encounters.locks[lock_id]
        m.locks[lock_id] = copy(lock)
        m.edges_by_id[edge].gate_rule = lock_id
        m.edges_by_id[edge].discovery_rule = 'hidden'
        m.rooms_by_id[key_room].grants_key = lock.key
      end
    end
  end

  -- Forward shortcuts that cannot reach past the boss; they reduce backtracking.
  local boss_room
  for _, id in ipairs(m.spine) do if m.rooms_by_id[id].role == 'boss' then boss_room = id end end
  for _ = 1, layout:range(0, 2) do
    local spots = {}
    for _, id in ipairs(m.spine) do
      local room = m.rooms_by_id[id]
      if id ~= boss_room and room.role ~= 'finish' and room.role ~= 'entry' then
        for _, socket in ipairs(template_sockets(id)) do
          if not room.sockets_by_id[socket.id].edge then spots[#spots + 1] = {room = id, socket = socket.id} end
        end
      end
    end
    if #spots < 2 then break end
    local a = spots[layout:below(#spots)]
    local b = spots[layout:below(#spots)]
    if a.room ~= b.room then connect(self, state, a.room, a.socket, b.room, b.socket, 'shortcut', 'both') end
  end

  -- Resolve encounters and rewards from their own streams, after topology.
  for _, id in ipairs(m.order) do
    local room = m.rooms_by_id[id]
    if room.role == 'combat' then
      local pool = {}
      for _, archetype in ipairs({'pressure', 'guard', 'zone', 'elite'}) do
        for _, candidate in ipairs(enc_index[archetype] or {}) do
          for _, allowed in ipairs(self.encounters.encounters[candidate].themes) do
            if allowed == room.theme then pool[#pool + 1] = candidate break end
          end
        end
      end
      if #pool > 0 then room.encounter = pool[encounter_rng:below(#pool)] end
    elseif room.role == 'boss' then
      local pool = {}
      for _, candidate in ipairs(enc_index.boss or {}) do
        for _, allowed in ipairs(self.encounters.encounters[candidate].themes) do
          if allowed == room.theme then pool[#pool + 1] = candidate break end
        end
      end
      if #pool == 0 then pool = enc_index.boss or {} end
      if #pool > 0 then room.encounter = pool[encounter_rng:below(#pool)] end
    end
    local template = self.rooms.rooms[room.template_id]
    if #template.rewards > 0 and (room.role == 'reward' or room.role == 'combat') then
      local family = room.theme == 'fire' and 'fire' or room.theme == 'frost' and 'frost' or 'any'
      local pool = {}
      for _, reward_id in ipairs(reward_ids) do
        local spec = self.encounters.rewards[reward_id]
        if spec.family == nil or spec.family == 'any' or spec.family == family then pool[#pool + 1] = reward_id end
      end
      if #pool > 0 and reward_rng:chance(2, 3) then room.reward = pool[reward_rng:below(#pool)] end
    end
  end

  -- Depths by BFS from start, deterministic tie order.
  local queue, head, seen = {m.start_room}, 1, {[m.start_room] = true}
  m.rooms_by_id[m.start_room].depth = 0
  while queue[head] do
    local id = queue[head]; head = head + 1
    local edges = copy(m.edges_by_room[id] or {})
    table.sort(edges)
    for _, edge_id in ipairs(edges) do
      local edge = m.edges_by_id[edge_id]
      local other = edge.from_room == id and edge.to_room or edge.from_room
      if not seen[other] then
        seen[other] = true
        m.rooms_by_id[other].depth = m.rooms_by_id[id].depth + 1
        queue[#queue + 1] = other
      end
    end
  end

  m.generation_report.topology_signature = Topology.signature(m)
  return m
end

function Topology.signature(m)
  local parts = {}
  for _, id in ipairs(m.spine) do
    local room = m.rooms_by_id[id]
    local connected = 0
    for _, socket in pairs(room.sockets_by_id) do if socket.edge then connected = connected + 1 end end
    parts[#parts + 1] = room.role .. ':' .. connected
  end
  local optional = {}
  for _, id in ipairs(m.order) do
    if not m.rooms_by_id[id].mandatory then optional[#optional + 1] = m.rooms_by_id[id].role end
  end
  table.sort(optional)
  local shortcuts = 0
  for _, edge in pairs(m.edges_by_id) do if edge.kind == 'shortcut' then shortcuts = shortcuts + 1 end end
  return table.concat(parts, ',') .. '|opt=' .. table.concat(optional, ',') .. '|shortcut=' .. shortcuts
end

function Topology.validate(self, m)
  if type(m) ~= 'table' or m.schema_version ~= Topology.schema_version then return false, 'unsupported schema' end
  if m.catalogue_version ~= self.rooms.version or m.encounter_version ~= self.encounters.version then return false, 'version mismatch' end
  if type(m.world_seed) ~= 'number' or m.world_seed % 1 ~= 0 or m.world_seed < 1 or m.world_seed >= MODEL then return false, 'invalid world seed' end
  if not m.rooms_by_id[m.start_room] or not m.rooms_by_id[m.final_room] then return false, 'missing endpoints' end
  local count = 0
  for id, room in pairs(m.rooms_by_id) do
    count = count + 1
    if type(id) ~= 'string' or room.id ~= id then return false, 'room id mismatch' end
    if not self.rooms.rooms[room.template_id] then return false, 'unknown template' end
    if self.rooms.roles[room.role] ~= true then return false, 'invalid role' end
    if self.rooms.themes[room.theme] == nil then return false, 'invalid theme' end
    if type(room.sockets_by_id) ~= 'table' then return false, 'missing sockets' end
    local socket_count = 0
    for sid, socket in pairs(room.sockets_by_id) do
      socket_count = socket_count + 1
      if socket.id ~= sid then return false, 'socket id mismatch' end
      if socket.edge ~= nil and not m.edges_by_id[socket.edge] then return false, 'dangling socket edge' end
    end
    if socket_count < 1 then return false, 'room needs sockets' end
    if room.reward ~= nil and not self.encounters.rewards[room.reward] then return false, 'unknown reward' end
    if room.encounter ~= nil and not self.encounters.encounters[room.encounter] then return false, 'unknown encounter' end
    if room.mandatory ~= (room.spine_index ~= nil) then return false, 'mandatory/spine mismatch' end
  end
  if count < 12 or count > 18 then return false, 'room count out of band' end
  if #m.spine < 8 or #m.spine > 12 then return false, 'mandatory route out of band' end
  local used = {}
  for id, edge in pairs(m.edges_by_id) do
    if edge.id ~= id then return false, 'edge id mismatch' end
    local from, to = m.rooms_by_id[edge.from_room], m.rooms_by_id[edge.to_room]
    if not from or not to then return false, 'edge endpoint missing' end
    if edge.from_room == edge.to_room then return false, 'self edge' end
    if not from.sockets_by_id[edge.from_socket] or not to.sockets_by_id[edge.to_socket] then return false, 'edge socket missing' end
    local fs, ts = edge.from_room .. ':' .. edge.from_socket, edge.to_room .. ':' .. edge.to_socket
    if used[fs] or used[ts] then return false, 'socket bound twice' end
    used[fs], used[ts] = true, true
    if from.sockets_by_id[edge.from_socket].edge ~= id or to.sockets_by_id[edge.to_socket].edge ~= id then
      return false, 'edge/socket backreference mismatch'
    end
    if edge.gate_rule and not m.locks[edge.gate_rule] then return false, 'unknown gate lock' end
  end
  for _, id in ipairs(m.spine) do if not m.rooms_by_id[id] then return false, 'spine references unknown room' end end
  local adjacency = {}
  for _, edge in pairs(m.edges_by_id) do
    adjacency[edge.from_room] = adjacency[edge.from_room] or {}
    adjacency[edge.to_room] = adjacency[edge.to_room] or {}
    adjacency[edge.from_room][#adjacency[edge.from_room] + 1] = edge.to_room
    adjacency[edge.to_room][#adjacency[edge.to_room] + 1] = edge.from_room
  end
  local reach, q, i = {[m.start_room] = true}, {m.start_room}, 1
  while q[i] do
    for _, other in ipairs(adjacency[q[i]] or {}) do
      if not reach[other] then reach[other] = true; q[#q + 1] = other end
    end
    i = i + 1
  end
  for id in pairs(m.rooms_by_id) do if not reach[id] then return false, 'unreachable room ' .. id end end
  if self.progression then
    local ok, why = self.progression.validate(m)
    if not ok then return false, why end
  end
  return true
end

function Topology:generate(seed, opts)
  opts = opts or {}
  assert(type(seed) == 'number' and seed % 1 == 0 and seed >= 1 and seed <= MODEL - 1, 'invalid seed')
  local attempts = opts.attempts or 32
  local reasons = {}
  for attempt = 1, attempts do
    local ok, result = pcall(build, self, seed, attempt)
    if ok and result then
      local valid, why = Topology.validate(self, result)
      if valid then
        result.generation_report.attempt_count = attempt
        result.generation_report.fallback_used = false
        result.generation_report.rejection_reasons = reasons
        return result
      end
      reasons[#reasons + 1] = tostring(why)
    else
      reasons[#reasons + 1] = tostring(result)
    end
  end
  local fallback = build(self, seed, 'fallback')
  local valid, why = Topology.validate(self, fallback)
  assert(valid, 'fallback topology invalid: ' .. tostring(why))
  fallback.generation_report.attempt_count = attempts
  fallback.generation_report.fallback_used = true
  fallback.generation_report.rejection_reasons = reasons
  return fallback
end

return Topology
