-- Run progress record: everything that changes during a run, kept strictly
-- separate from the resolved manifest. Gate 4 persists this alongside the
-- manifest in the TBD3 checkpoint. Pure and additive; no engine access.
--
-- Stable identities only: room instance ids, edge ids, reward ids, logical
-- entity ids and key names. Native handles, pointers and closures never belong
-- here.
local Progress = {version = 2, max_rooms = 64, max_edges = 192, max_keys = 16,
  max_entities = 64, max_claimed = 64, max_objectives = 32, max_supplies = 9, max_lives = 99}

local function valid_id(x) return type(x) == 'string' and #x > 0 and #x <= 64 and not x:find('[%z\1-\31]') end
local function integer(x, lo, hi) return type(x) == 'number' and x % 1 == 0 and x >= lo and x <= hi end

local function count(t) local n = 0 for _ in pairs(t) do n = n + 1 end return n end

-- flags: a bounded boolean set keyed by stable id.
local function set_flag(t, id, max, what)
  if t[id] then return true end
  if count(t) >= max then return nil, what .. ' capacity' end
  t[id] = true
  return true
end

function Progress.new(run_id, start_room, opts)
  opts = opts or {}
  assert(valid_id(run_id), 'invalid run id')
  assert(valid_id(start_room), 'invalid start room')
  local record = {
    version = Progress.version, run_id = run_id,
    current_room = start_room, current_socket = nil, start_room = start_room,
    visited = {[start_room] = true}, discovered = {[start_room] = true},
    revealed = {}, claimed = {}, keys = {}, consumables = {}, defeated = {}, objectives = {},
    opened = {}, pickups = {}, encounter_kos = {},
    supplies = integer(opts.supplies, 0, Progress.max_supplies) and opts.supplies or 2,
    lives = integer(opts.lives, 0, Progress.max_lives) and opts.lives or 3,
    outcome = nil,
  }
  return record
end

function Progress.enter(record, room_id, socket)
  if record.version ~= Progress.version or record.outcome then return nil, 'run not active' end
  if not valid_id(room_id) or (socket ~= nil and not valid_id(socket)) then return nil, 'invalid destination' end
  local ok, why = set_flag(record.visited, room_id, Progress.max_rooms, 'visited')
  if not ok then return nil, why end
  record.current_room, record.current_socket = room_id, socket
  record.discovered[room_id] = true
  return true
end

function Progress.reveal(record, room_id, edge_id)
  if not valid_id(room_id) or not valid_id(edge_id) then return nil, 'invalid reveal' end
  if not record.discovered[room_id] and count(record.discovered) >= Progress.max_rooms then return nil, 'discovered capacity' end
  local ok,why=set_flag(record.revealed, edge_id, Progress.max_edges, 'revealed edge')
  if not ok then return nil,why end
  record.discovered[room_id] = true
  return true
end

function Progress.claim(record, reward_id)
  if not valid_id(reward_id) then return nil, 'invalid reward' end
  return set_flag(record.claimed, reward_id, Progress.max_claimed, 'claimed reward')
end

function Progress.unlock(record, key)
  if not valid_id(key) then return nil, 'invalid key' end
  return set_flag(record.keys, key, Progress.max_keys, 'key')
end

function Progress.grant_consumable(record, key, amount)
  if not valid_id(key) or not integer(amount, 1, Progress.max_supplies) then return nil, 'invalid consumable grant' end
  local next_total = (record.consumables[key] or 0) + amount
  if next_total > Progress.max_supplies then return nil, 'consumable capacity' end
  if not record.consumables[key] and count(record.consumables) >= Progress.max_keys then return nil, 'consumable key capacity' end
  record.consumables[key] = next_total
  return true
end

function Progress.spend_consumable(record, key)
  if (record.consumables[key] or 0) <= 0 then return nil, 'no such consumable' end
  record.consumables[key] = record.consumables[key] - 1
  if record.consumables[key] <= 0 then record.consumables[key] = nil end
  return true
end

function Progress.defeat(record, entity_id)
  if not valid_id(entity_id) then return nil, 'invalid entity' end
  return set_flag(record.defeated, entity_id, Progress.max_entities, 'defeated entity')
end

function Progress.complete_objective(record, objective_id, state)
  if not valid_id(objective_id) then return nil, 'invalid objective' end
  state = state or 'done'
  if state ~= 'done' and state ~= 'pending' then return nil, 'invalid objective state' end
  if not record.objectives[objective_id] and count(record.objectives) >= Progress.max_objectives then return nil, 'objective capacity' end
  record.objectives[objective_id] = state
  return true
end

function Progress.spend_supply(record)
  if record.supplies <= 0 then return nil, 'no supplies' end
  record.supplies = record.supplies - 1
  return true
end

function Progress.lose_life(record)
  if record.lives <= 0 then return nil, 'already out of lives' end
  record.lives = record.lives - 1
  return true
end

function Progress.finish(record, outcome)
  if outcome ~= 'success' and outcome ~= 'failure' then return nil, 'invalid outcome' end
  record.outcome = outcome
  return true
end

local FIELDS = {version = true, run_id = true, current_room = true, current_socket = true,
  start_room = true, visited = true, discovered = true, revealed = true, claimed = true,
  keys = true, consumables = true, defeated = true, objectives = true, supplies = true,
  opened = true, pickups = true, encounter_kos = true,
  lives = true, outcome = true}

function Progress.validate(record)
  if type(record) ~= 'table' or record.version ~= Progress.version then return false, 'unsupported progress version' end
  for field in pairs(record) do if not FIELDS[field] then return false, 'unknown progress field ' .. tostring(field) end end
  if not valid_id(record.run_id) or not valid_id(record.current_room) or not valid_id(record.start_room) then return false, 'invalid progress identity' end
  if record.current_socket ~= nil and not valid_id(record.current_socket) then return false, 'invalid current socket' end
  if record.outcome ~= nil and record.outcome ~= 'success' and record.outcome ~= 'failure' then return false, 'invalid outcome' end
  if not integer(record.supplies, 0, Progress.max_supplies) or not integer(record.lives, 0, Progress.max_lives) then return false, 'invalid supplies/lives' end
  local bounds = {{'visited', Progress.max_rooms}, {'discovered', Progress.max_rooms}, {'revealed', Progress.max_edges},
    {'opened', Progress.max_edges}, {'pickups', Progress.max_claimed}, {'claimed', Progress.max_claimed}, {'keys', Progress.max_keys}, {'defeated', Progress.max_entities}, {'objectives', Progress.max_objectives}}
  for _, pair in ipairs(bounds) do
    local t, max = record[pair[1]], pair[2]
    if type(t) ~= 'table' then return false, 'missing ' .. pair[1] end
    if count(t) > max then return false, pair[1] .. ' capacity exceeded' end
    for k, v in pairs(t) do
      if not valid_id(k) then return false, 'invalid id in ' .. pair[1] end
      if pair[1] == 'objectives' then
        if v ~= 'done' and v ~= 'pending' then return false, 'invalid objective state' end
      elseif v ~= true then
        return false, 'invalid flag in ' .. pair[1]
      end
    end
  end
  if type(record.consumables) ~= 'table' or count(record.consumables) > Progress.max_keys then return false, 'invalid consumables' end
  for k, v in pairs(record.consumables) do
    if not valid_id(k) or not integer(v, 1, Progress.max_supplies) then return false, 'invalid consumable entry' end
  end
  if type(record.encounter_kos) ~= 'table' or count(record.encounter_kos) > Progress.max_rooms then return false, 'invalid encounter progress' end
  for id,kos in pairs(record.encounter_kos) do if not valid_id(id) or not integer(kos,0,16) then return false, 'invalid encounter stocks' end end
  if not record.visited[record.current_room] then return false, 'current room not visited' end
  return true
end

return Progress
