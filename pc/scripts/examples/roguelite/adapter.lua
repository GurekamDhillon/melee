-- Runtime adapter: maps a resolved v2 topology manifest to the node view the
-- existing rooms/main runtime understands, using the recipe module for
-- geometry. It is deliberately strict: if any room's recipe is not certified,
-- the whole manifest is refused, so uncertified geometry can never spawn actors.
--
-- This is the Gate 4 interface. The live runtime still runs v1; consuming this
-- adapter is a separate, native-verified step.
local Adapter = {version = 1}

local function copy(t)
  if type(t) ~= 'table' then return t end
  local out = {} for k,v in pairs(t) do out[k] = copy(v) end return out
end
local function equal(a,b)
  if type(a) ~= type(b) then return false end
  if type(a) ~= 'table' then return a == b end
  for k,v in pairs(a) do if not equal(v,b[k]) then return false end end
  for k in pairs(b) do if a[k] == nil then return false end end
  return true
end
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

-- Production generation may select only layouts admitted by the same contract
-- that checks saved manifests. Never changes recipe certification.
function Adapter:eligible_templates()
  local eligible = {}
  for id, template in pairs(self.catalogue.rooms) do
    if self.recipes.is_certified(template.recipe) and self.recipes.resolve(template) then
      eligible[id] = true
    end
  end
  return eligible
end

local function template_for(self, room)
  return self.catalogue.rooms[room.template_id]
end

function Adapter:check(v2)
  if type(v2) ~= 'table' or v2.schema_version ~= 2 then return nil, 'unsupported manifest schema' end
  if type(v2.order) ~= 'table' or type(v2.rooms_by_id) ~= 'table' or type(v2.edges_by_id) ~= 'table' or type(v2.edges_by_room) ~= 'table' then return nil, 'missing manifest graph' end
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
    local geometry, recipe = self.recipes.resolve(template)
    if not geometry then return nil, 'room ' .. id .. ': ' .. tostring(recipe) end
    if room.template_version ~= template.version then return nil, 'unsupported template version' end
    if room.geometry and (room.recipe_version ~= recipe.version or not equal(room.geometry,geometry)
      or not equal(room.recipe_modules or {},recipe.modules or {})) then
      return nil, 'saved geometry no longer has matching certification: ' .. id
    end
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
    local resolved, recipe = self.recipes.resolve(template)
    local geometry = copy(room.geometry or assert(resolved))
    local node = {
      id = id, kind = KIND[room.role] or room.role, role = room.role, v2_role = room.role,
      title = room.title, depth = room.depth, theme = room.theme, mandatory = room.mandatory,
      template_id = room.template_id, recipe = template.recipe, encounter = room.encounter,
      reward = room.reward, reward_spec = room.reward_spec, encounter_spec = room.encounter_spec,
      recipe_version = room.recipe_version or recipe.version, recipe_modules = copy(room.recipe_modules or recipe.modules or {}),
      lock = room.grants_key, room = geometry, exits = {},
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
        local arrival_socket = from_here and edge.to_socket or edge.from_socket
        local destination = nodes[other].room
        local other_side = v2.rooms_by_id[other].sockets_by_id[arrival_socket].side
        local arrival = (destination.arrivals or {})[arrival_socket] or (destination.arrivals or {})[other_side]
          or destination.exit_anchors[other_side]
        node.exits[#node.exits + 1] = {
          edge_id = edge_id, arrival_socket = arrival_socket, arrival = copy(arrival),
          anchor = copy(node.room.exit_anchors[socket_id] or node.room.exit_anchors[socket.side]),
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
