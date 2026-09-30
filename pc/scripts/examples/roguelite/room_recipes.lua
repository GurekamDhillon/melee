-- Physical recipe contract for room templates (Gate 3 foundation). A recipe
-- resolves an authored template to concrete lane geometry and per-side socket
-- anchors that the runtime adapter can hand to the existing room planner.
--
-- The BF kit only authorises two lane ends (left/right at +/-52) plus the
-- current floor/platform pieces. Templates that declare a third socket (top or
-- bottom) or a split/merge/cross shape have no authored geometry yet, so they
-- are marked unsupported with an explicit reason: the runtime must fail before
-- spawning actors rather than drawing stairs the fighter cannot stand on.
--
-- `certified` is false everywhere: geometric consistency and bounds are checked
-- here, but in-engine traversal/combat certification is a separate native gate.
local RoomRecipes = {version = 1, unit = 6.5, grid = 13, bay = 26, height = 26}

local KIT = {unit = 6.5, grid = 13, bay = 26, height = 26, depth = 0}
local LEFT, RIGHT = {x = -52, y = 0}, {x = 52, y = 0}

local function geometry(platforms, anchors)
  return {floor = {left = -65, right = 65, y = 0}, kit = KIT, platforms = platforms or {}, exit_anchors = anchors}
end

local function platform(x, y, width)
  return {x = x, y = y, width = width, passthrough = true, ledges = true}
end

RoomRecipes.recipes = {
  entry_lane = {id = 'entry_lane', version = 1, certified = false, geometry = geometry({}, {right = RIGHT})},
  finish_lane = {id = 'finish_lane', version = 1, certified = false, geometry = geometry({}, {left = LEFT})},
  lane_open = {id = 'lane_open', version = 1, certified = false, geometry = geometry({}, {left = LEFT, right = RIGHT})},
  lane_two_level = {id = 'lane_two_level', version = 1, certified = false,
    geometry = geometry({platform(-22, 12, 22), platform(22, 12, 22), platform(0, 24, 24)}, {left = LEFT, right = RIGHT})},
  lane_pit = {id = 'lane_pit', version = 1, certified = false,
    geometry = geometry({platform(-33, 12, 20), platform(33, 12, 20)}, {left = LEFT, right = RIGHT})},
  arena_flat = {id = 'arena_flat', version = 1, certified = false, geometry = geometry({}, {left = LEFT, right = RIGHT})},
  arena_pillars = {id = 'arena_pillars', version = 1, certified = false,
    geometry = geometry({platform(-20, 10, 14), platform(20, 10, 14)}, {left = LEFT, right = RIGHT})},
  rest_alcove = {id = 'rest_alcove', version = 1, certified = false, geometry = geometry({}, {left = LEFT, right = RIGHT})},
  reward_vault = {id = 'reward_vault', version = 1, certified = false, geometry = geometry({}, {left = LEFT, right = RIGHT})},
  boss_arena = {id = 'boss_arena', version = 1, certified = false, geometry = geometry({}, {left = LEFT, right = RIGHT})},

  -- Awaiting authored geometry / native certification.
  lane_balcony = {id = 'lane_balcony', version = 1, supported = false, reason = 'top socket needs authored balcony/upper-door geometry'},
  lane_fork = {id = 'lane_fork', version = 1, supported = false, reason = 'top socket needs authored fork geometry'},
  arena_tiered = {id = 'arena_tiered', version = 1, supported = false, reason = 'top socket needs authored tier geometry'},
  rest_balcony = {id = 'rest_balcony', version = 1, supported = false, reason = 'top socket needs authored balcony geometry'},
  shortcut_door = {id = 'shortcut_door', version = 1, supported = false, reason = 'bottom socket needs authored drop-door geometry'},
  branch_y = {id = 'branch_y', version = 1, supported = false, reason = 'split needs authored up/down doorway geometry'},
  junction_cross = {id = 'junction_cross', version = 1, supported = false, reason = 'cross needs authored four-way geometry'},
  rejoin_merge = {id = 'rejoin_merge', version = 1, supported = false, reason = 'merge needs authored up/down doorway geometry'},
}

function RoomRecipes.get(id) return RoomRecipes.recipes[id] end
function RoomRecipes.is_supported(id)
  local recipe = RoomRecipes.recipes[id]
  return recipe ~= nil and recipe.supported ~= false
end

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end

-- Returns a `room` geometry table plus anchors, or nil, reason. A supported
-- recipe whose declared sockets have no anchor is refused rather than guessed.
function RoomRecipes.resolve(template)
  if type(template) ~= 'table' or type(template.recipe) ~= 'string' then return nil, 'template needs a recipe' end
  local recipe = RoomRecipes.recipes[template.recipe]
  if not recipe then return nil, 'unknown recipe ' .. template.recipe end
  if recipe.supported == false then return nil, 'recipe ' .. template.recipe .. ' unsupported: ' .. recipe.reason end
  for _, socket in ipairs(template.sockets or {}) do
    if not recipe.geometry.exit_anchors[socket.side] then
      return nil, 'recipe ' .. template.recipe .. ' has no anchor for socket side ' .. tostring(socket.side)
    end
  end
  return recipe.geometry, recipe
end

function RoomRecipes.validate(catalogue)
  assert(catalogue and catalogue.rooms, 'room catalogue required')
  for id, template in pairs(catalogue.rooms) do
    local recipe = RoomRecipes.recipes[template.recipe]
    if not recipe then return false, 'template ' .. id .. ' references unknown recipe ' .. tostring(template.recipe) end
    if recipe.supported == false then
      if type(recipe.reason) ~= 'string' or #recipe.reason == 0 then
        return false, 'unsupported recipe ' .. recipe.id .. ' lacks a reason'
      end
    else
      local g = recipe.geometry
      if type(g) ~= 'table' or type(g.floor) ~= 'table' or type(g.platforms) ~= 'table' or type(g.exit_anchors) ~= 'table' then
        return false, 'recipe ' .. recipe.id .. ' has malformed geometry'
      end
      if g.floor.left ~= -65 or g.floor.right ~= 65 or g.floor.y ~= 0 then return false, 'recipe ' .. recipe.id .. ' floor mismatch' end
      if g.kit.unit ~= 6.5 or g.kit.grid ~= 13 or g.kit.bay ~= 26 or g.kit.height ~= 26 then return false, 'recipe ' .. recipe.id .. ' kit mismatch' end
      for _, p in ipairs(g.platforms) do
        if not finite(p.x) or not finite(p.y) or not finite(p.width) or p.width <= 0 then return false, 'recipe ' .. recipe.id .. ' invalid platform' end
        if p.x - p.width / 2 < -65 or p.x + p.width / 2 > 65 or p.y < 0 or p.y > 60 then return false, 'recipe ' .. recipe.id .. ' platform out of bounds' end
        if type(p.passthrough) ~= 'boolean' or type(p.ledges) ~= 'boolean' then return false, 'recipe ' .. recipe.id .. ' platform flags' end
        if not p.passthrough then return false, 'recipe ' .. recipe.id .. ' solid platform needs clearance validation' end
      end
      for _, socket in ipairs(template.sockets) do
        if not g.exit_anchors[socket.side] then return false, 'recipe ' .. recipe.id .. ' missing anchor for ' .. socket.side end
      end
    end
  end
  return true
end

return RoomRecipes
