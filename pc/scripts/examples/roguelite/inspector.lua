-- Deterministic text renderer for a resolved topology manifest. This is the
-- Gate 2 "graph inspector": it makes the actual route shape, branch, detours,
-- shortcuts and locks visible for evidence and human review. It consumes a
-- manifest and never mutates it.
local Inspector = {version = 1}

local function sorted(ids)
  local out = {}
  for _, id in ipairs(ids) do out[#out + 1] = id end
  table.sort(out)
  return out
end

local function room_lookup(manifest)
  return manifest.rooms_by_id
end

function Inspector.render(manifest)
  assert(type(manifest) == 'table' and manifest.rooms_by_id and manifest.edges_by_id, 'invalid manifest')
  local report = manifest.generation_report or {}
  local lines = {}
  local function emit(s) lines[#lines + 1] = s end
  emit(string.format('seed=%d schema=%s gen=%s catalogue=%s encounter=%s',
    manifest.world_seed, tostring(manifest.schema_version), tostring(manifest.generator_version),
    tostring(manifest.catalogue_version), tostring(manifest.encounter_version)))
  emit(string.format('rooms=%d spine=%d attempt=%s fallback=%s',
    (function() local n = 0 for _ in pairs(manifest.rooms_by_id) do n = n + 1 end return n end)(),
    #manifest.spine, tostring(report.attempt_count), tostring(report.fallback_used)))
  emit('signature=' .. tostring(report.topology_signature))
  emit('')
  emit('MANDATORY SPINE')
  local rooms = room_lookup(manifest)
  for _, id in ipairs(manifest.spine) do
    local room = assert(rooms[id], 'spine room missing')
    local notes = {}
    if room.encounter then notes[#notes + 1] = 'encounter=' .. room.encounter end
    if room.reward then notes[#notes + 1] = 'reward=' .. room.reward end
    if room.grants_key then notes[#notes + 1] = 'grants=' .. room.grants_key end
    emit(string.format('  %2d. %-6s %-9s %-7s %s%s', room.spine_index or 0, id, room.role, room.theme, room.title,
      #notes > 0 and ('  [' .. table.concat(notes, ', ') .. ']') or ''))
  end
  emit('')
  emit('EDGES')
  local ids = {}
  for id in pairs(manifest.edges_by_id) do ids[#ids + 1] = id end
  ids = sorted(ids)
  for _, id in ipairs(ids) do
    local edge = manifest.edges_by_id[id]
    local gate = edge.gate_rule and (' gate=' .. edge.gate_rule) or ''
    emit(string.format('  %s %s.%s -> %s.%s  (%s/%s)%s', id,
      edge.from_room, edge.from_socket, edge.to_room, edge.to_socket, edge.kind, edge.direction, gate))
  end
  emit('')
  emit('OPTIONAL ROOMS')
  local optional = {}
  for id in pairs(manifest.rooms_by_id) do
    if not manifest.rooms_by_id[id].mandatory then optional[#optional + 1] = id end
  end
  optional = sorted(optional)
  for _, id in ipairs(optional) do
    local room = manifest.rooms_by_id[id]
    emit(string.format('  %-6s %-9s %-7s depth=%d %s', id, room.role, room.theme, room.depth, room.title))
  end
  if #optional == 0 then emit('  (none)') end
  emit('')
  emit('LOCKS')
  local lock_ids = {}
  for id in pairs(manifest.locks or {}) do lock_ids[#lock_ids + 1] = id end
  lock_ids = sorted(lock_ids)
  for _, id in ipairs(lock_ids) do
    local lock = manifest.locks[id]
    emit(string.format('  %s kind=%s key=%s', id, lock.kind, lock.key))
  end
  if #lock_ids == 0 then emit('  (none)') end
  return table.concat(lines, '\n') .. '\n'
end

return Inspector
