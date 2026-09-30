-- Physical recipe contract for room templates (Gate 3 foundation). A recipe
-- resolves an authored template to concrete lane geometry and per-side socket
-- anchors that the runtime adapter can hand to the room planner.
--
-- Geometry stays inside the certified BF kit grid (floor [-65,65], 13-unit grid,
-- 26 bay/storey). Upper (top) doors use the reviewed stair/balcony/ramp/opening
-- layout from the BF interior example, converted once to game units (UNIT 6.5):
--   stairs_4m_rise2m  x=-39 y=0   (floor -> balcony)
--   balcony_4m        x=-13 y=13  (2 m landing)
--   ramp_4m_rise2m    x=13  y=13  (balcony -> upper floor)
--   floor_opening_4m  x=39  y=26  (4 m upper floor)
--   wall_doorway_4m   x=39  y=26  (upper doorway; matches the top anchor)
-- A bottom socket is a real floor opening: geometry.floor.openings carries the
-- gap and the graph edge for that socket must be one-way (see topology/adapter).
--
-- `certified` is false everywhere. Bounds, anchor coverage and ascent
-- reachability are checked here, but Gate 3 requires an in-engine
-- traversal/combat clip before admission. The runtime must refuse uncertified
-- recipes.
local RoomRecipes = {version = 2, unit = 6.5, grid = 13, bay = 26, height = 26}

local KIT = {unit = 6.5, grid = 13, bay = 26, height = 26, depth = 0}
local LEFT, RIGHT = {x = -52, y = 0}, {x = 52, y = 0}
local TOP = {x = 39, y = 26}
local DROP_X, DROP_WIDTH = 0, 26
local BOTTOM = {x = DROP_X, y = 0, drop = true}

local ASCENT_MODULES = {
  {part = 'bf_stairs_4m_rise2m', x = -39, y = 0},
  {part = 'bf_balcony_4m', x = -13, y = 13},
  {part = 'bf_ramp_4m_rise2m', x = 13, y = 13},
  {part = 'bf_floor_opening_4m', x = 39, y = 26},
  {part = 'bf_wall_doorway_4m', x = 39, y = 26},
}
RoomRecipes.ascent_modules = ASCENT_MODULES

local function platform(x, y, width)
  return {x = x, y = y, width = width, passthrough = true, ledges = true}
end

-- Analytic surfaces aligned to the visual landings: the balcony at x=-13 y=13
-- and the upper floor at x=39 y=26. The visual ramp bridges the gap; the
-- analytic screen permits the 13-unit rise and 26-unit gap.
local function ascent()
  return {platform(-13, 13, 26), platform(39, 26, 26)}
end

local function geometry(platforms, anchors, openings)
  return {
    floor = {left = -65, right = 65, y = 0, openings = openings or {}},
    kit = KIT, platforms = platforms or {}, exit_anchors = anchors,
    spawn = {x = -42, y = 0}, enemy_spawns = {},
    camera = {left = -65, right = 65, bottom = 0, top = 60},
  }
end

local function recipe(id, platforms, anchors, opts)
  opts = opts or {}
  local r = {id = id, version = 1, certified = false,
    geometry = geometry(platforms, anchors, opts.openings)}
  if opts.ascent then
    r.modules = ASCENT_MODULES
    r.upper_doorway = ASCENT_MODULES[#ASCENT_MODULES]
  end
  return r
end

RoomRecipes.recipes = {
  entry_lane = recipe('entry_lane', {}, {right = RIGHT}),
  finish_lane = recipe('finish_lane', {}, {left = LEFT}),
  lane_open = recipe('lane_open', {}, {left = LEFT, right = RIGHT}),
  lane_two_level = recipe('lane_two_level', {platform(-22, 12, 22), platform(22, 12, 22), platform(0, 24, 24)}, {left = LEFT, right = RIGHT}),
  lane_pit = recipe('lane_pit', {platform(-33, 12, 20), platform(33, 12, 20)}, {left = LEFT, right = RIGHT}),
  lane_balcony = recipe('lane_balcony', ascent(), {left = LEFT, right = RIGHT, top = TOP}, {ascent = true}),
  lane_fork = recipe('lane_fork', ascent(), {left = LEFT, right = RIGHT, top = TOP}, {ascent = true}),
  arena_flat = recipe('arena_flat', {}, {left = LEFT, right = RIGHT}),
  arena_pillars = recipe('arena_pillars', {platform(-20, 10, 14), platform(20, 10, 14)}, {left = LEFT, right = RIGHT}),
  arena_tiered = recipe('arena_tiered', ascent(), {left = LEFT, right = RIGHT, top = TOP}, {ascent = true}),
  branch_y = recipe('branch_y', ascent(), {left = LEFT, right = RIGHT, top = TOP}, {ascent = true}),
  junction_cross = recipe('junction_cross', ascent(), {left = LEFT, right = RIGHT, top = TOP, bottom = BOTTOM}, {ascent = true, openings = {{x = DROP_X, width = DROP_WIDTH}}}),
  rejoin_merge = recipe('rejoin_merge', ascent(), {left = LEFT, top = TOP, right = RIGHT}, {ascent = true}),
  rest_alcove = recipe('rest_alcove', {}, {left = LEFT, right = RIGHT}),
  rest_balcony = recipe('rest_balcony', ascent(), {left = LEFT, right = RIGHT, top = TOP}, {ascent = true}),
  reward_vault = recipe('reward_vault', {}, {left = LEFT, right = RIGHT}),
  shortcut_door = recipe('shortcut_door', {}, {left = LEFT, right = RIGHT, bottom = BOTTOM}, {openings = {{x = DROP_X, width = DROP_WIDTH}}}),
  boss_arena = recipe('boss_arena', {}, {left = LEFT, right = RIGHT}),
}

function RoomRecipes.get(id) return RoomRecipes.recipes[id] end
function RoomRecipes.is_supported(id)
  local r = RoomRecipes.recipes[id]
  return r ~= nil and r.supported ~= false
end
function RoomRecipes.is_certified(id)
  local r = RoomRecipes.recipes[id]
  return r ~= nil and r.certified == true
end

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end

function RoomRecipes.resolve(template)
  if type(template) ~= 'table' or type(template.recipe) ~= 'string' then return nil, 'template needs a recipe' end
  local r = RoomRecipes.recipes[template.recipe]
  if not r then return nil, 'unknown recipe ' .. template.recipe end
  if r.supported == false then return nil, 'recipe ' .. template.recipe .. ' unsupported: ' .. r.reason end
  for _, socket in ipairs(template.sockets or {}) do
    if not r.geometry.exit_anchors[socket.side] then
      return nil, 'recipe ' .. template.recipe .. ' has no anchor for socket side ' .. tostring(socket.side)
    end
  end
  return r.geometry, r
end

local function opening_contains(opening, x)
  return x >= opening.x - opening.width / 2 and x <= opening.x + opening.width / 2
end

function RoomRecipes.validate(catalogue)
  assert(catalogue and catalogue.rooms, 'room catalogue required')
  for id, template in pairs(catalogue.rooms) do
    local r = RoomRecipes.recipes[template.recipe]
    if not r then return false, 'template ' .. id .. ' references unknown recipe ' .. tostring(template.recipe) end
    if r.supported == false then
      if type(r.reason) ~= 'string' or #r.reason == 0 then return false, 'unsupported recipe ' .. r.id .. ' lacks a reason' end
    else
      local g = r.geometry
      if type(g) ~= 'table' or type(g.floor) ~= 'table' or type(g.platforms) ~= 'table' or type(g.exit_anchors) ~= 'table' then
        return false, 'recipe ' .. r.id .. ' has malformed geometry'
      end
      if g.floor.left ~= -65 or g.floor.right ~= 65 or g.floor.y ~= 0 then return false, 'recipe ' .. r.id .. ' floor mismatch' end
      if g.kit.unit ~= 6.5 or g.kit.grid ~= 13 or g.kit.bay ~= 26 or g.kit.height ~= 26 then return false, 'recipe ' .. r.id .. ' kit mismatch' end
      for _, p in ipairs(g.platforms) do
        if not finite(p.x) or not finite(p.y) or not finite(p.width) or p.width <= 0 then return false, 'recipe ' .. r.id .. ' invalid platform' end
        if p.x - p.width / 2 < -65 or p.x + p.width / 2 > 65 or p.y < 0 or p.y > 60 then return false, 'recipe ' .. r.id .. ' platform out of bounds' end
        if type(p.passthrough) ~= 'boolean' or type(p.ledges) ~= 'boolean' then return false, 'recipe ' .. r.id .. ' platform flags' end
        if not p.passthrough then return false, 'recipe ' .. r.id .. ' solid platform needs clearance validation' end
      end
      -- Floor openings must be in bounds and non-overlapping.
      local openings = g.floor.openings or {}
      for i, o in ipairs(openings) do
        if not finite(o.x) or not finite(o.width) or o.width <= 0 then return false, 'recipe ' .. r.id .. ' invalid opening' end
        if o.x - o.width / 2 < -65 or o.x + o.width / 2 > 65 then return false, 'recipe ' .. r.id .. ' opening out of bounds' end
        for j = i + 1, #openings do
          local b = openings[j]
          if math.abs(o.x - b.x) < (o.width + b.width) / 2 then return false, 'recipe ' .. r.id .. ' overlapping openings' end
        end
      end
      for _, socket in ipairs(template.sockets) do
        if not g.exit_anchors[socket.side] then return false, 'recipe ' .. r.id .. ' missing anchor for ' .. socket.side end
      end
      -- A drop anchor needs a real opening, and every opening needs a drop anchor.
      for side, anchor in pairs(g.exit_anchors) do
        if anchor.drop then
          local covered = false
          for _, o in ipairs(openings) do if opening_contains(o, anchor.x) then covered = true end end
          if not covered then return false, 'recipe ' .. r.id .. ' drop anchor ' .. side .. ' has no floor opening' end
        end
      end
      for _, o in ipairs(openings) do
        local used = false
        for _, anchor in pairs(g.exit_anchors) do if anchor.drop and opening_contains(o, anchor.x) then used = true end end
        if not used then return false, 'recipe ' .. r.id .. ' floor opening has no drop anchor' end
      end
      -- Visual ascent modules and the upper doorway must agree with the top anchor.
      if r.modules then
        local doorway
        for _, module in ipairs(r.modules) do
          if type(module.part) ~= 'string' or not finite(module.x) or not finite(module.y) then return false, 'recipe ' .. r.id .. ' invalid ascent module' end
          if module.part == 'bf_wall_doorway_4m' then doorway = module end
        end
        local top = g.exit_anchors.top
        if not doorway or not top then return false, 'recipe ' .. r.id .. ' ascent needs a top anchor and doorway' end
        if doorway.x ~= top.x or doorway.y ~= top.y then return false, 'recipe ' .. r.id .. ' upper doorway disagrees with top anchor' end
      end
    end
  end
  return true
end

-- Analytic ascent check: every platform and the top anchor must be reachable
-- from the floor under the standard mobility profile. Fast pre-filter only.
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
