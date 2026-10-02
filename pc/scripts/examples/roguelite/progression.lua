-- Progression validator. A connected graph is not enough: every mandatory
-- objective must be reachable under one-way/edge/gate rules. The search walks
-- bounded states (room, persistent unlocks, consumable inventory, opened locks,
-- claimed one-shot pickups, visited-objective bitmask) so a lock cannot strand
-- the player, a shortcut cannot skip a required boss, and a finite pickup cannot
-- be harvested twice by revisiting its room.
--
-- Manifest contract (see topology.lua):
--   start_room, final_room, rooms_by_id, edges_by_id, locks (by id), spine
--   (ordered required room ids; start/finish are always required).
-- A room may carry:
--   grants_key         persistent key granted once on entry
--   grants_consumable  map key -> amount, a one-shot pickup per (room,key)
--   pickups            array of {id, key, amount, repeatable?} explicit pickups
-- A lock's `repeat` policy declares whether a consumable gate charges on every
-- traversal (true) or opens permanently after the first payment (false default).
local Progression = {version = 2, max_states = 200000, max_keys = 64,
  max_pickups = 64, max_opened = 128, max_consumable_total = 64}

local function copy_map(t)
  local out = {}
  for k, v in pairs(t) do out[k] = v end
  return out
end

-- Objective bitmask without bitwise operators (LuaJIT-safe). Setting a bit must
-- be idempotent: a plain addition would carry into the next objective when a
-- room is revisited and falsely mark an unvisited objective complete.
local function bit_value(index) return 2 ^ (index - 1) end
local function has_bit(mask, value) return mask % (value * 2) >= value end
local function with_bit(mask, value)
  if has_bit(mask, value) then return mask end
  return mask + value
end

local function signature(parts)
  local list = {}
  for k in pairs(parts) do list[#list + 1] = k end
  table.sort(list)
  return table.concat(list, '+')
end
local function count_signature(counts)
  local list = {}
  for k in pairs(counts) do list[#list + 1] = k .. '=' .. counts[k] end
  table.sort(list)
  return table.concat(list, '+')
end

local function bounded_amount(x) return type(x) == 'number' and x % 1 == 0 and x >= 1 and x <= 64 end

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
  for _, id in ipairs(order) do full = full + bit_value(bit[id]) end

  -- Stable pickup identity. Explicit pickups take precedence; grants_consumable
  -- derives one named pickup per (room,key) so revisiting cannot replenish it.
  local pickups, pickups_by_room = {}, {}
  local function add_pickup(pid, room, key, amount, repeatable)
    if type(pid) ~= 'string' or #pid == 0 or #pid > 96 then return false, 'invalid pickup id' end
    if type(key) ~= 'string' or #key == 0 or #key > 64 then return false, 'invalid pickup key' end
    if not bounded_amount(amount) then return false, 'invalid pickup amount' end
    if pickups[pid] then return false, 'duplicate pickup id ' .. pid end
    if #pickups >= Progression.max_pickups then return false, 'pickup catalogue too large' end
    pickups[pid] = {id = pid, room = room, key = key, amount = amount, repeatable = repeatable == true}
    pickups_by_room[room] = pickups_by_room[room] or {}
    pickups_by_room[room][#pickups_by_room[room] + 1] = pid
    return true
  end
  for id, room in pairs(manifest.rooms_by_id) do
    if room.grants_consumable ~= nil then
      if type(room.grants_consumable) ~= 'table' then return false, 'invalid grants_consumable' end
      for key, amount in pairs(room.grants_consumable) do
        local ok, why = add_pickup(id .. '/' .. key, id, key, amount, false)
        if not ok then return false, why end
      end
    end
    if room.pickups ~= nil then
      if type(room.pickups) ~= 'table' then return false, 'invalid room pickups' end
      for _, pickup in ipairs(room.pickups) do
        local ok, why = add_pickup(pickup.id, id, pickup.key, pickup.amount or 1, pickup.repeatable)
        if not ok then return false, why end
      end
    end
  end
  for _, list in pairs(pickups_by_room) do table.sort(list) end

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
  local queue = {{room = start, keys = {}, consumables = {}, opened = {}, claimed = {},
    mask = bit[start] and bit_value(bit[start]) or 0}}
  local head, states = 1, 0
  while head <= #queue do
    local state = queue[head]
    head = head + 1
    states = states + 1
    if states > max_states then return false, 'progression state space exceeded' end
    local room = manifest.rooms_by_id[state.room]
    local keys, consumables, opened, claimed = state.keys, state.consumables, state.opened, state.claimed

    -- Persistent keys: one grant, immutable afterwards.
    if room.grants_key and not keys[room.grants_key] then
      keys = copy_map(keys)
      keys[room.grants_key] = true
    end
    -- One-shot pickups: claim once; a repeatable pickup (explicit) re-grants.
    for _, pid in ipairs(pickups_by_room[state.room] or {}) do
      local pickup = pickups[pid]
      if pickup.repeatable or not claimed[pid] then
        claimed = copy_map(claimed)
        consumables = copy_map(consumables)
        claimed[pid] = true
        consumables[pickup.key] = (consumables[pickup.key] or 0) + pickup.amount
        if consumables[pickup.key] > Progression.max_consumable_total then
          return false, 'consumable inventory exceeded'
        end
      end
    end
    if state.mask == full then return true, {states = states} end

    for _, link in ipairs(adjacency[state.room] or {}) do
      local allowed, edge_consumables, edge_opened = true, consumables, opened
      if link.edge.gate_rule then
        local lock = locks[link.edge.gate_rule]
        if not lock then return false, 'edge references unknown lock' end
        if lock.kind == 'persistent_key' then
          allowed = keys[lock.key] == true
        elseif lock.kind == 'consumable_key' then
          if opened[link.edge.gate_rule] and not lock['repeat'] then
            allowed = true
          else
            allowed = (consumables[lock.key] or 0) > 0
            if allowed then
              edge_consumables = copy_map(consumables)
              edge_consumables[lock.key] = edge_consumables[lock.key] - 1
              if edge_consumables[lock.key] <= 0 then edge_consumables[lock.key] = nil end
              if not lock['repeat'] then
                edge_opened = copy_map(opened)
                edge_opened[link.edge.gate_rule] = true
              end
            end
          end
        else
          return false, 'invalid lock kind'
        end
      end
      if allowed then
        local mask = state.mask
        if bit[link.to] then mask = with_bit(mask, bit_value(bit[link.to])) end
        local sig = table.concat({link.to, signature(keys), count_signature(edge_consumables),
          signature(edge_opened), signature(claimed), mask}, '|')
        if not visited[sig] then
          visited[sig] = true
          queue[#queue + 1] = {room = link.to, keys = keys, consumables = edge_consumables,
            opened = edge_opened, claimed = claimed, mask = mask}
        end
      end
    end
  end
  return false, 'mandatory objectives unreachable with available keys'
end

return Progression
