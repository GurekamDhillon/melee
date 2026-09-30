-- Progression validator. A connected graph is not enough: every mandatory
-- objective must be reachable under one-way/edge/gate rules. The search walks
-- bounded states (room, persistent unlocks, remaining consumable keys, visited
-- required-objective bitmask) so a lock cannot strand the player and a shortcut
-- cannot skip a required boss.
--
-- Manifest contract (see topology.lua):
--   start_room, final_room, rooms_by_id, edges_by_id, locks (by id), spine
--   (ordered required room ids, start/finish included or added here).
-- A room may carry `grants_key` (persistent key granted on entry).
local Progression = {version = 1, max_states = 200000, max_keys = 64}

local function copy_map(t)
  local out = {}
  for k, v in pairs(t) do out[k] = v end
  return out
end

local function key_signature(keys)
  local list = {}
  for k in pairs(keys) do list[#list + 1] = k end
  table.sort(list)
  return table.concat(list, '+')
end

local function count_signature(counts)
  local list = {}
  for k in pairs(counts) do list[#list + 1] = k .. '=' .. counts[k] end
  table.sort(list)
  return table.concat(list, '+')
end

function Progression.validate(manifest, opts)
  opts = opts or {}
  if type(manifest) ~= 'table' or type(manifest.rooms_by_id) ~= 'table'
    or type(manifest.edges_by_id) ~= 'table' then return false, 'invalid manifest shape' end
  local start, finish = manifest.start_room, manifest.final_room
  if not manifest.rooms_by_id[start] then return false, 'missing start room' end
  if not manifest.rooms_by_id[finish] then return false, 'missing final room' end

  -- Deterministic bit assignment for required rooms: spine order first, then any
  -- remaining required ids sorted, so the bitmask is stable across runs.
  local order, seen = {}, {}
  for _, id in ipairs(manifest.spine or {}) do
    if not seen[id] then seen[id] = true; order[#order + 1] = id end
  end
  local extras = {}
  for id in pairs(manifest.rooms_by_id) do
    if not seen[id] and manifest.rooms_by_id[id].mandatory then extras[#extras + 1] = id end
  end
  table.sort(extras)
  for _, id in ipairs(extras) do seen[id] = true; order[#order + 1] = id end
  local bit = {}
  for i, id in ipairs(order) do bit[id] = i end
  if #order > 40 then return false, 'too many mandatory rooms for bounded search' end
  local full = 0
  for _, id in ipairs(order) do full = full + 2 ^ (bit[id] - 1) end

  local adjacency = {}
  for id, edge in pairs(manifest.edges_by_id) do
    if not manifest.rooms_by_id[edge.from_room] or not manifest.rooms_by_id[edge.to_room] then
      return false, 'edge references unknown room'
    end
    if edge.from_room == edge.to_room then return false, 'self edge ' .. tostring(id) end
    local direction = edge.direction or 'both'
    if direction ~= 'both' and direction ~= 'forward' and direction ~= 'backward' then
      return false, 'invalid edge direction'
    end
    if direction == 'both' or direction == 'forward' then
      adjacency[edge.from_room] = adjacency[edge.from_room] or {}
      table.insert(adjacency[edge.from_room], {to = edge.to_room, edge = edge})
    end
    if direction == 'both' or direction == 'backward' then
      adjacency[edge.to_room] = adjacency[edge.to_room] or {}
      table.insert(adjacency[edge.to_room], {to = edge.from_room, edge = edge})
    end
  end
  for _, list in pairs(adjacency) do
    table.sort(list, function(a, b) return tostring(a.edge.id) < tostring(b.edge.id) end)
  end

  local locks = manifest.locks or {}
  local max_states = math.min(opts.max_states or Progression.max_states, Progression.max_states)
  local visited = {}
  local queue = {{room = start, keys = {}, consumables = {}, mask = bit[start] and 2 ^ (bit[start] - 1) or 0}}
  local head, states = 1, 0
  while head <= #queue do
    local state = queue[head]
    head = head + 1
    states = states + 1
    if states > max_states then return false, 'progression state space exceeded' end
    local room = manifest.rooms_by_id[state.room]
    local keys = state.keys
    if room.grants_key and not keys[room.grants_key] then
      keys = copy_map(keys)
      keys[room.grants_key] = true
    end
    if state.mask == full then return true, {states = states} end
    for _, link in ipairs(adjacency[state.room] or {}) do
      local consumables = state.consumables
      local allowed = true
      if link.edge.gate_rule then
        local lock = locks[link.edge.gate_rule]
        if not lock then return false, 'edge references unknown lock' end
        if lock.kind == 'persistent_key' then
          allowed = keys[lock.key] == true
        elseif lock.kind == 'consumable_key' then
          allowed = (consumables[lock.key] or 0) > 0
          if allowed then
            consumables = copy_map(consumables)
            consumables[lock.key] = consumables[lock.key] - 1
            if consumables[lock.key] <= 0 then consumables[lock.key] = nil end
          end
        else
          return false, 'invalid lock kind'
        end
      end
      if allowed then
        local mask = state.mask
        if bit[link.to] then mask = mask + 2 ^ (bit[link.to] - 1) end
        local sig = link.to .. '|' .. key_signature(keys) .. '|' .. count_signature(consumables) .. '|' .. mask
        if not visited[sig] then
          visited[sig] = true
          queue[#queue + 1] = {room = link.to, keys = keys, consumables = consumables, mask = mask}
        end
      end
    end
  end
  return false, 'mandatory objectives unreachable with available keys'
end

return Progression
