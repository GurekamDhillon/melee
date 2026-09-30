-- Physical recipe contract for room templates (Gate 3 foundation). A recipe
-- resolves an authored template to concrete lane geometry and per-side socket
-- anchors that the runtime adapter can hand to the room planner.
--
-- Geometry stays inside the certified BF kit grid (floor [-65,65], 13-unit
-- grid, 26 bay/storey). Upper (top) doors use a stair/balcony ascent whose rises
-- and gaps fit the standard mobility profile; the drop (bottom) socket is a
-- floor-level opening used for shortcuts.
--
-- `certified` is false everywhere. Bounds and anchor coverage are checked here,
-- but Gate 3 requires an in-engine traversal/combat clip before any template is
-- admitted for spawning. The runtime must refuse uncertified recipes.
local RoomRecipes = {version = 1, unit = 6.5, grid = 13, bay = 26, height = 26}

local KIT = {unit = 6.5, grid = 13, bay = 26, height = 26, depth = 0}
local LEFT, RIGHT = {x = -52, y = 0}, {x = 52, y = 0}
local TOP = {x = 10, y = 26}          -- upper doorway reached by the ascent below
local BOTTOM = {x = 0, y = 0, drop = true} -- floor-level drop-through opening

local function platform(x, y, width)
  return {x = x, y = y, width = width, passthrough = true, ledges = true}
end

-- Three passthrough steps rising to the y=26 balcony; every rise/gap is within
-- the standard jump profile from the v1 analytic screen.
local function ascent()
  return {platform(-22, 9, 16), platform(-6, 18, 16), platform(10, 26, 26)}
end

local function geometry(platforms, anchors)
  return {
    floor = {left = -65, right = 65, y = 0}, kit = KIT,
    platforms = platforms or {}, exit_anchors = anchors,
    spawn = {x = -42, y = 0}, enemy_spawns = {},
    camera = {left = -65, right = 65, bottom = 0, top = 60},
  }
end

local function recipe(id, platforms, anchors)
  return {id = id, version = 1, certified = false, geometry = geometry(platforms, anchors)}
end

RoomRecipes.recipes = {
  entry_lane = recipe('entry_lane', {}, {right = RIGHT}),
  finish_lane = recipe('finish_lane', {}, {left = LEFT}),
  lane_open = recipe('lane_open', {}, {left = LEFT, right = RIGHT}),
  lane_two_level = recipe('lane_two_level', {platform(-22, 12, 22), platform(22, 12, 22), platform(0, 24, 24)}, {left = LEFT, right = RIGHT}),
  lane_pit = recipe('lane_pit', {platform(-33, 12, 20), platform(33, 12, 20)}, {left = LEFT, right = RIGHT}),
  lane_balcony = recipe('lane_balcony', ascent(), {left = LEFT, right = RIGHT, top = TOP}),
  lane_fork = recipe('lane_fork', ascent(), {left = LEFT, right = RIGHT, top = TOP}),
  arena_flat = recipe('arena_flat', {}, {left = LEFT, right = RIGHT}),
  arena_pillars = recipe('arena_pillars', {platform(-20, 10, 14), platform(20, 10, 14)}, {left = LEFT, right = RIGHT}),
  arena_tiered = recipe('arena_tiered', ascent(), {left = LEFT, right = RIGHT, top = TOP}),
  branch_y = recipe('branch_y', ascent(), {left = LEFT, right = RIGHT, top = TOP}),
  junction_cross = recipe('junction_cross', ascent(), {left = LEFT, right = RIGHT, top = TOP, bottom = BOTTOM}),
  rejoin_merge = recipe('rejoin_merge', ascent(), {left = LEFT, top = TOP, right = RIGHT}),
  rest_alcove = recipe('rest_alcove', {}, {left = LEFT, right = RIGHT}),
  rest_balcony = recipe('rest_balcony', ascent(), {left = LEFT, right = RIGHT, top = TOP}),
  reward_vault = recipe('reward_vault', {}, {left = LEFT, right = RIGHT}),
  shortcut_door = recipe('shortcut_door', {}, {left = LEFT, right = RIGHT, bottom = BOTTOM}),
  boss_arena = recipe('boss_arena', {}, {left = LEFT, right = RIGHT}),
}

function RoomRecipes.get(id) return RoomRecipes.recipes[id] end
function RoomRecipes.is_supported(id)
  local recipe = RoomRecipes.recipes[id]
  return recipe ~= nil and recipe.supported ~= false
end
function RoomRecipes.is_certified(id)
  local recipe = RoomRecipes.recipes[id]
  return recipe ~= nil and recipe.certified == true
end

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end

-- Returns a `room` geometry table, or nil, reason. A supported recipe whose
-- declared sockets have no anchor is refused rather than guessed.
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

-- Analytic ascent check: every platform and the top anchor must be reachable
-- from the floor under the standard mobility profile. This mirrors the v1
-- screen and is a fast pre-filter, not an engine traversal proof.
function RoomRecipes.reachable(recipe, mobility)
  if type(recipe) ~= 'table' or not recipe.geometry then return false, 'not a supported recipe' end
  mobility = mobility or {}
  local height = mobility.jump_height or 18
  local lateral = mobility.horizontal_gap or 30
  local surfaces = {{lo = -65, hi = 65, y = 0}}
  for _, p in ipairs(recipe.geometry.platforms) do
    surfaces[#surfaces + 1] = {lo = p.x - p.width / 2 + 2, hi = p.x + p.width / 2 - 2, y = p.y}
  end
  local seen, changed = {[1] = true}, true
  while changed do
    changed = false
    for i, a in ipairs(surfaces) do if seen[i] then
      for j, b in ipairs(surfaces) do
        local gap = math.max(0, b.lo - a.hi, a.lo - b.hi)
        if not seen[j] and b.y - a.y <= height and gap <= lateral then seen[j] = true; changed = true end
      end
    end end
  end
  for i in ipairs(surfaces) do if not seen[i] then return false, 'unreachable platform surface' end end
  for side, anchor in pairs(recipe.geometry.exit_anchors) do
    if not anchor.drop then
      local found = false
      for i, s in ipairs(surfaces) do
        if seen[i] and anchor.x >= s.lo and anchor.x <= s.hi and math.abs(anchor.y - s.y) < 0.01 then found = true end
      end
      if not found then return false, 'anchor ' .. side .. ' is not on a reachable surface' end
    end
  end
  return true
end

return RoomRecipes
