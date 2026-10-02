-- Room template catalogue: authored layout contracts, not geometry. A template
-- declares its role, socket count/types, theme eligibility, encounter/reward
-- eligibility and budgets. Physical module placement and collision recipes are
-- referenced by `recipe` and resolved by the runtime adapter (Gate 3/4).
--
-- Sockets are stable authored IDs with an abstract side. The generator binds
-- edges to sockets by ID; two destinations must never share one socket.
local RoomCatalogue = {version = 1, unit = 6.5, grid = 13, bay = 26, height = 26, depth = 0}

RoomCatalogue.roles = {
  entry = true, teach = true, traversal = true, combat = true, branch = true,
  rest = true, reward = true, connector = true, boss = true, finish = true,
}

RoomCatalogue.themes = {
  cobalt = {name = 'Cobalt Halls'},
  frost = {name = 'Rime Gallery'},
  fire = {name = 'Ember Court'},
}

-- role -> templates allowed. Sockets: array of {id, side}. side is an authored
-- abstract direction used by the physical adapter; it must be unique per room.
local function room(id, role, sockets, recipe, shape, theme)
  return {id = id, version = 1, role = role, sockets = sockets, recipe = recipe, shape = shape or 'line', theme = theme,
    mobility = 'standard', budget = {parts = 28, collisions = 16},
    encounters = role == 'combat' and {'pressure', 'guard', 'zone', 'elite'}
      or role == 'boss' and {'boss'} or {},
    rewards = (role == 'reward' or role == 'combat' or role == 'branch') and {'upgrade', 'tradeoff', 'consumable', 'equipment'} or {}}
end

RoomCatalogue.rooms = {
  entry_gate = room('entry_gate', 'entry', {{id = 'out', side = 'right'}}, 'entry_lane'),
  finish_gate = room('finish_gate', 'finish', {{id = 'in', side = 'left'}}, 'finish_lane'),
  teach_lane = room('teach_lane', 'teach', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'lane_open'),
  lane_straight = room('lane_straight', 'traversal', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'lane_open'),
  lane_vertical = room('lane_vertical', 'traversal', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'lane_two_level'),
  lane_pit = room('lane_pit', 'traversal', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'lane_pit'),
  lane_balcony = room('lane_balcony', 'traversal', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}, {id = 'up', side = 'top'}}, 'lane_balcony'),
  lane_fork = room('lane_fork', 'traversal', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}, {id = 'up', side = 'top'}}, 'lane_fork'),
  arena_flat = room('arena_flat', 'combat', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'arena_flat'),
  arena_pillars = room('arena_pillars', 'combat', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'arena_pillars'),
  arena_drop = room('arena_drop', 'combat', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'lane_pit'),
  arena_tiered = room('arena_tiered', 'combat', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}, {id = 'up', side = 'top'}}, 'arena_tiered'),
  branch_y = room('branch_y', 'branch', {{id = 'in', side = 'left'}, {id = 'branch_a', side = 'right'}, {id = 'branch_b', side = 'top'}}, 'branch_y', 'split'),
  junction_cross = room('junction_cross', 'branch', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}, {id = 'branch_a', side = 'top'}, {id = 'branch_b', side = 'bottom'}}, 'junction_cross', 'cross'),
  rejoin_merge = room('rejoin_merge', 'branch', {{id = 'in_a', side = 'left'}, {id = 'in_b', side = 'top'}, {id = 'out', side = 'right'}}, 'rejoin_merge', 'merge'),
  rest_alcove = room('rest_alcove', 'rest', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'rest_alcove'),
  rest_balcony = room('rest_balcony', 'rest', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}, {id = 'up', side = 'top'}}, 'rest_balcony'),
  reward_vault = room('reward_vault', 'reward', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'reward_vault'),
  shortcut_door = room('shortcut_door', 'connector', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}, {id = 'shortcut', side = 'bottom'}}, 'shortcut_door'),
  boss_arena = room('boss_arena', 'boss', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'boss_arena'),

  -- Gate 3 bulk expansion: distinct authored movement demands. Themes reuse the
  -- three existing compositions (cobalt/fire/frost); no new art or palette-only
  -- variants. All remain `certified = false` until a native replay lands.
  traverse_stagger = room('traverse_stagger', 'traversal', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'traverse_stagger', 'line', 'frost'),
  traverse_bridge = room('traverse_bridge', 'traversal', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'traverse_bridge', 'line', 'cobalt'),
  combat_flank = room('combat_flank', 'combat', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'combat_flank', 'line', 'fire'),
  combat_dais = room('combat_dais', 'combat', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'combat_dais', 'line', 'frost'),
  combat_ring = room('combat_ring', 'combat', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'combat_ring', 'line', 'cobalt'),
  boss_dais = room('boss_dais', 'boss', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'boss_dais', 'line', 'fire'),
  rest_platform = room('rest_platform', 'rest', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'rest_platform', 'line', 'frost'),
  reward_ledge = room('reward_ledge', 'reward', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}}, 'reward_ledge', 'line', 'cobalt'),
  crossing_door = room('crossing_door', 'connector', {{id = 'in', side = 'left'}, {id = 'out', side = 'right'}, {id = 'drop', side = 'bottom'}}, 'crossing_door', 'line', 'frost'),
}

RoomCatalogue.by_role = {}
for id, template in pairs(RoomCatalogue.rooms) do
  RoomCatalogue.by_role[template.role] = RoomCatalogue.by_role[template.role] or {}
  local list = RoomCatalogue.by_role[template.role]
  list[#list + 1] = id
end
for _, list in pairs(RoomCatalogue.by_role) do table.sort(list) end

function RoomCatalogue.templates_for_role(role)
  return RoomCatalogue.by_role[role]
end

function RoomCatalogue.get(id)
  return RoomCatalogue.rooms[id]
end

function RoomCatalogue.validate(self)
  self = self or RoomCatalogue
  assert(type(self.rooms) == 'table' and next(self.rooms), 'empty room catalogue')
  for id, template in pairs(self.rooms) do
    assert(type(id) == 'string' and template.id == id, 'invalid template id')
    assert(self.roles[template.role], 'invalid role ' .. tostring(template.role))
    assert(type(template.sockets) == 'table' and #template.sockets >= 1, 'template needs sockets')
    local ids, sides = {}, {}
    for _, socket in ipairs(template.sockets) do
      assert(type(socket.id) == 'string' and #socket.id > 0 and not ids[socket.id], 'duplicate socket id')
      assert(type(socket.side) == 'string' and not sides[socket.side], 'duplicate socket side')
      ids[socket.id], sides[socket.side] = true, true
    end
    assert(type(template.recipe) == 'string' and #template.recipe > 0, 'template needs a recipe')
    assert(template.shape == 'line' or template.shape == 'split' or template.shape == 'merge' or template.shape == 'cross', 'invalid shape')
    assert(self.themes[template.theme] or template.theme == nil, 'unknown theme')
    assert(type(template.budget) == 'table' and template.budget.parts >= 1 and template.budget.collisions >= 0, 'invalid budget')
  end
  for _, list in pairs(self.by_role) do
    assert(#list > 0, 'empty role list')
  end
  return true
end

-- Distinct-layout audit. Diversity is geometry diversity: a canonical, theme-
-- free signature over movement geometry and module placements only (camera,
-- spawn, arrivals, socket names, shape and translation are ignored). Aliases
-- are explicit, so a recolour, a relabel or a translated copy never inflates
-- the catalogue.
function RoomCatalogue.audit(self, recipes)
  self = self or RoomCatalogue
  local templates, by_role, aliases, recipe_users, signature_users = 0, {}, {}, {}, {}
  for id, template in pairs(self.rooms) do
    templates = templates + 1
    local group = by_role[template.role] or {templates = 0, distinct = {}}
    group.templates = group.templates + 1
    by_role[template.role] = group
    local signature
    if recipes and recipes.signature then
      local recipe = recipes.get(template.recipe)
      signature = recipe and recipes.signature(recipe, template)
        or (template.recipe .. ':' .. tostring(template.shape) .. ':' .. tostring(#template.sockets))
    else
      signature = template.recipe
    end
    group.distinct[signature] = true
    local users = signature_users[signature]
    if not users then users = {}; signature_users[signature] = users end
    users[#users + 1] = id
    local recipe_users_for = recipe_users[template.recipe]
    if not recipe_users_for then recipe_users_for = {}; recipe_users[template.recipe] = recipe_users_for end
    recipe_users_for[#recipe_users_for + 1] = id
  end
  for signature, users in pairs(signature_users) do
    if #users > 1 then table.sort(users); aliases[signature] = users end
  end
  local recipe_aliases = {}
  for recipe, users in pairs(recipe_users) do
    if #users > 1 then table.sort(users); recipe_aliases[recipe] = users end
  end
  local distinct = 0
  for _ in pairs(signature_users) do distinct = distinct + 1 end
  for _, group in pairs(by_role) do
    local count = 0
    for _ in pairs(group.distinct) do count = count + 1 end
    group.distinct_count = count
    group.distinct = nil
  end
  return {templates = templates, distinct = distinct, by_role = by_role, aliases = aliases, recipe_aliases = recipe_aliases}
end

return RoomCatalogue
