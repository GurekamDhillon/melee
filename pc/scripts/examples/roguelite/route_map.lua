-- Discovered-map view: the slice of a resolved v2 topology manifest the player
-- is allowed to see, derived from a Progress schema-2 record. Pure and additive:
-- it reads the manifest shapes topology.build() emits (rooms_by_id, edges_by_id,
-- locks, generation_report) and the flag sets progress.new() creates, and never
-- mutates either. Anything the run has not seen is omitted outright, so a reward
-- or encounter id cannot leak through a map the player has not earned.
local RouteMap = {version = 1}

local function is_table(x) return type(x) == 'table' end
local function is_id(x) return type(x) == 'string' and #x > 0 end

-- A gate with no lock entry, or one of an unknown kind, is closed: unknown data
-- must never read as traversable.
local function gate_open(edge, progress, locks)
  local gate = edge.gate_rule
  if gate == nil then return true end
  local lock = locks[gate]
  if not is_table(lock) then return false end
  if lock.kind == 'persistent_key' then
    return progress.keys[lock.key] == true
  elseif lock.kind == 'consumable_key' then
    return progress.opened[gate] == true or (progress.consumables[lock.key] or 0) > 0
  end
  return false
end

function RouteMap.build(manifest, progress)
  -- Shapes: topology.build() stamps schema_version 2 and the progress record
  -- from progress.lua carries version 2 (see its FIELDS list). Refuse malformed
  -- input with a reason instead of throwing, so a bad save cannot crash the HUD.
  if not is_table(manifest) or manifest.schema_version ~= 2 then return nil, 'unsupported manifest schema' end
  if not is_table(manifest.rooms_by_id) or not is_table(manifest.edges_by_id) then return nil, 'missing manifest graph' end
  if not is_table(progress) or progress.version ~= 2 then return nil, 'unsupported progress version' end
  if not is_id(progress.current_room) then return nil, 'invalid current room' end
  local visited, discovered, revealed = progress.visited, progress.discovered, progress.revealed
  local claimed, keys, consumables, opened = progress.claimed, progress.keys, progress.consumables, progress.opened
  if not is_table(visited) or not is_table(discovered) or not is_table(revealed)
    or not is_table(claimed) or not is_table(keys) or not is_table(consumables) or not is_table(opened) then
    return nil, 'malformed progress'
  end
  local locks_in = is_table(manifest.locks) and manifest.locks or {}

  -- Both endpoints of every edge must be known; an edge missing an endpoint is
  -- malformed, not a shortcut to nowhere.
  local incident = {}
  for eid, edge in pairs(manifest.edges_by_id) do
    if is_id(eid) and is_table(edge) and is_id(edge.from_room) and is_id(edge.to_room) then
      local a = incident[edge.from_room]
      if a then a[#a + 1] = eid else incident[edge.from_room] = {eid} end
      local b = incident[edge.to_room]
      if b then b[#b + 1] = eid else incident[edge.to_room] = {eid} end
    end
  end

  local room_ids = {}
  for id in pairs(manifest.rooms_by_id) do
    if is_id(id) and (discovered[id] == true or visited[id] == true) then room_ids[#room_ids + 1] = id end
  end
  table.sort(room_ids)

  -- Spoiler boundary: count (not name) rewards the run has not yet discovered.
  local undiscovered_rewards = 0
  for id, room in pairs(manifest.rooms_by_id) do
    if is_table(room) and room.reward ~= nil and discovered[id] ~= true and visited[id] ~= true then
      undiscovered_rewards = undiscovered_rewards + 1
    end
  end

  local rooms, exits, locks = {}, {}, {}
  local known_exits = 0
  for _, id in ipairs(room_ids) do
    local room = manifest.rooms_by_id[id]
    if not is_table(room) then return nil, 'malformed room ' .. id end
    rooms[id] = {
      id = id, title = room.title, role = room.role, theme = room.theme,
      depth = room.depth, mandatory = room.mandatory == true,
      visited = visited[id] == true, current = id == progress.current_room,
      -- Only whether the reward was taken crosses the boundary; its id stays
      -- hidden. route.lua records one-time claims under 'reward:<room_id>', so
      -- accept that key as well as the catalogue reward id.
      reward_claimed = room.reward ~= nil and (claimed[room.reward] == true or claimed['reward:' .. id] == true),
    }
    local edge_ids = incident[id] or {}
    table.sort(edge_ids)
    local list = {}
    for _, eid in ipairs(edge_ids) do
      local edge = manifest.edges_by_id[eid]
      local from_here = edge.from_room == id
      -- A one-way edge is an exit only from the room it points away from.
      local traversable = edge.direction == 'both'
        or (edge.direction == 'forward' and from_here)
        or (edge.direction == 'backward' and not from_here)
      local destination = from_here and edge.to_room or edge.from_room
      local edge_id = edge.id or eid
      -- Known once the route revealed the edge or already stands on its far end.
      if traversable and (revealed[edge_id] == true or discovered[destination] == true) then
        local socket_id = from_here and edge.from_socket or edge.to_socket
        local socket = room.sockets_by_id and room.sockets_by_id[socket_id]
        if not is_table(socket) then return nil, 'missing socket ' .. tostring(socket_id) end
        local gate = edge.gate_rule
        list[#list + 1] = {
          edge = edge_id, to = destination, socket = socket_id, side = socket.side,
          kind = edge.kind, hidden = edge.discovery_rule == 'hidden',
          destination_discovered = discovered[destination] == true,
          gate = gate, gate_open = gate_open(edge, progress, locks_in),
        }
        -- Only locks on a known exit are surfaced; an unrevealed hidden door is
        -- not a lock the map should name yet.
        if gate ~= nil and is_table(locks_in[gate]) then
          local lock = locks_in[gate]
          local has_key
          if lock.kind == 'persistent_key' then has_key = keys[lock.key] == true
          elseif lock.kind == 'consumable_key' then has_key = (consumables[lock.key] or 0) > 0
          else has_key = false end
          locks[gate] = {id = gate, kind = lock.kind, key = lock.key,
            opened = opened[gate] == true, has_key = has_key}
        end
      end
    end
    exits[id] = list
    known_exits = known_exits + #list
  end

  return {
    current = progress.current_room,
    world_seed = manifest.world_seed,
    topology_signature = is_table(manifest.generation_report) and manifest.generation_report.topology_signature or nil,
    rooms = rooms, exits = exits, locks = locks,
    counts = {discovered_rooms = #room_ids, known_exits = known_exits,
      undiscovered_reward_count = undiscovered_rewards},
  }
end

return RouteMap
