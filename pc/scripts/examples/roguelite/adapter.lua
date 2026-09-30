-- Runtime adapter: maps a resolved v2 topology manifest to the node view the
-- existing rooms/main runtime understands, using the recipe module for
-- geometry. It is deliberately strict: if any room's recipe is not certified,
-- the whole manifest is refused, so uncertified geometry can never spawn actors.
--
-- This is the Gate 4 interface. The live runtime still runs v1; consuming this
-- adapter is a separate, native-verified step.
local Adapter = {version = 1}

local KIND = {
  entry = 'entry', finish = 'exit', boss = 'boss', rest = 'rest', combat = 'arena',
  reward = 'reward', traversal = 'traversal', teach = 'traversal',
  branch = 'branch', connector = 'connector',
}

function Adapter.new(catalogue, recipes)
  assert(type(catalogue) == 'table' and catalogue.rooms, 'room catalogue required')
  assert(type(recipes) == 'table' and recipes.resolve and recipes.is_certified, 'recipe module required')
  return setmetatable({catalogue = catalogue, recipes = recipes}, {__index = Adapter})
end

local function template_for(self, room)
  return self.catalogue.rooms[room.template_id]
end

function Adapter:check(v2)
  if type(v2) ~= 'table' or v2.schema_version ~= 2 then return nil, 'unsupported manifest schema' end
  local seen = {}
  for _, id in ipairs(v2.order) do
    if seen[id] then return nil, 'duplicate room id in order' end
    seen[id] = true
    local room = v2.rooms_by_id[id]
    if not room then return nil, 'order references unknown room' end
    local template = template_for(self, room)
    if not template then return nil, 'room references unknown template' end
    if not self.recipes.is_certified(template.recipe) then
      return nil, 'room ' .. id .. ' recipe ' .. template.recipe .. ' is not certified'
    end
    local geometry, why = self.recipes.resolve(template)
    if not geometry then return nil, 'room ' .. id .. ': ' .. tostring(why) end
  end
  return true
end

-- Returns a legacy-shaped view: {nodes = {...}, order = {...}, start = id}.
function Adapter:manifest(v2)
  local ok, why = self:check(v2)
  if not ok then return nil, why end
  local nodes, order = {}, {}
  for _, id in ipairs(v2.order) do
    order[#order + 1] = id
    local room = v2.rooms_by_id[id]
    local template = template_for(self, room)
    local geometry = assert(self.recipes.resolve(template))
    local node = {
      id = id, kind = KIND[room.role] or room.role, role = room.role, v2_role = room.role,
      title = room.title, depth = room.depth, theme = room.theme, mandatory = room.mandatory,
      template_id = room.template_id, recipe = template.recipe, encounter = room.encounter,
      reward = room.reward, lock = room.grants_key, room = geometry, exits = {},
    }
    nodes[id] = node
  end
  -- Bind traversable exits; a socket's authored side yields the anchor.
  for _, id in ipairs(v2.order) do
    local room = v2.rooms_by_id[id]
    local node = nodes[id]
    local edge_ids = {}
    for _, edge_id in ipairs(v2.edges_by_room[id] or {}) do edge_ids[#edge_ids + 1] = edge_id end
    table.sort(edge_ids)
    for _, edge_id in ipairs(edge_ids) do
      local edge = v2.edges_by_id[edge_id]
      local from_here = edge.from_room == id
      local traversable = edge.direction == 'both'
        or (edge.direction == 'forward' and from_here)
        or (edge.direction == 'backward' and not from_here)
      if traversable then
        local other = from_here and edge.to_room or edge.from_room
        local socket_id = from_here and edge.from_socket or edge.to_socket
        local socket = room.sockets_by_id[socket_id]
        assert(socket, 'adapter: missing socket ' .. tostring(socket_id))
        node.exits[#node.exits + 1] = {
          to = other, side = socket.side, socket = socket_id,
          label = 'To ' .. (v2.rooms_by_id[other].title or other),
          kind = edge.kind, gate = edge.gate_rule, hidden = edge.discovery_rule == 'hidden',
        }
      end
    end
  end
  return {nodes = nodes, order = order, start = v2.start_room, final = v2.final_room,
    world_seed = v2.world_seed, topology_signature = v2.generation_report and v2.generation_report.topology_signature}
end

-- Versioned, read-only diagnostics for live checks.
function Adapter:diagnostics(v2)
  local ok, why = self:check(v2)
  return {
    diag_version = 1, schema_version = v2 and v2.schema_version, generator_version = v2 and v2.generator_version,
    world_seed = v2 and v2.world_seed, rooms = v2 and #v2.order, spine = v2 and #v2.spine,
    topology_signature = v2 and v2.generation_report and v2.generation_report.topology_signature,
    fallback_used = v2 and v2.generation_report and v2.generation_report.fallback_used,
    manifest_admissible = ok == true, refusal = ok and nil or why,
  }
end

return Adapter
