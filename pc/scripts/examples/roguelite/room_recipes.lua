-- Physical recipe contract for room templates (Gate 3 foundation). A recipe
-- resolves an authored template to concrete lane geometry and per-side socket
-- anchors that the runtime adapter can hand to the room planner.
--
-- Geometry stays inside the authored BF kit grid (floor [-65,65], 13-unit grid,
-- 26 bay/storey). Upper (top) doors use the reviewed stair/balcony/ramp/solid-landing
-- layout from the BF interior example, converted once to game units (UNIT 6.5):
--   stairs_4m_rise2m  x=-39 y=0   (floor -> balcony)
--   balcony_4m        x=-13 y=13  (2 m landing)
--   ramp_4m_rise2m    x=13  y=13  (balcony -> upper floor)
--   floor_4m          x=39  y=26  (solid 4 m upper landing)
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
local DROP_X, DROP_WIDTH = 0, 13
local BOTTOM = {x = DROP_X, y = -6, drop = true}

local ASCENT_MODULES = {
  {part = 'bf_stairs_4m_rise2m', x = -39, y = 0},
  {part = 'bf_balcony_4m', x = -13, y = 13},
  {part = 'bf_ramp_4m_rise2m', x = 13, y = 13},
  {part = 'bf_floor_4m', x = 39, y = 26},
  {part = 'bf_wall_doorway_4m', x = 39, y = 26, scale_x = -1},
}
RoomRecipes.ascent_modules = ASCENT_MODULES

local function platform(x, y, width)
  return {x = x, y = y, width = width, passthrough = true, ledges = true}
end

-- A one-way surface that runs continuously into a ramp or an adjoining bay:
-- BF placement rules disable ledges at interior seams, and the per-surface flag
-- cannot vary per edge, so a fully-seamed surface carries no ledge at all.
local function seam_platform(x, y, width)
  return {x = x, y = y, width = width, passthrough = true, ledges = false}
end

-- New-layout visual modules reuse only the ten BF models already cached by the
-- reviewed ascent (see `Rooms.max_assets`), so a run never grows its asset set.
-- A scaled floor bay is a standalone platform, not a structural wall bay.
local function floor_module(x, y, width)
  return {part = 'bf_floor_4m', x = x, y = y, scale_x = width / 26, scale_y = 1}
end

local function ramp_module(x, y, mirror)
  return {part = 'bf_ramp_4m_rise2m', x = x, y = y, scale_x = mirror and -1 or 1, scale_y = 1}
end

-- Landings match exporter sidecars, with no ledge flags at internal seams.
-- Slopes are explicit geometry.lines, rather than invisible flat step proxies.
-- The upper doorway requires solid floor: the opening variant has a gap
-- directly beneath x=39 and cannot supply a safe arrival there.
local function ascent()
  return {{x=-13,y=13,width=26,passthrough=false,ledges=false},
    {x=39,y=26,width=26,passthrough=false,ledges=false}}
end

local function geometry(platforms, anchors, openings)
  return {
    floor = {left = -65, right = 65, y = 0, openings = openings or {}},
    kit = KIT, platforms = platforms or {}, exit_anchors = anchors,
    spawn = {x = -42, y = 0}, enemy_spawns = {},
    camera = {left = -65, right = 65, bottom = -16, top = 60},
    arrivals = {left={x=-42,y=0,facing=1},right={x=42,y=0,facing=-1},
      top={x=39,y=26,facing=-1},bottom={x=20,y=0,facing=-1}},
    lines = {},
  }
end

local function recipe(id, platforms, anchors, opts)
  opts = opts or {}
  local r = {id = id, version = 2, certified = false, mobility_contract='bf-walk-jump-v2',
    geometry = geometry(platforms, anchors, opts.openings)}
  if opts.ascent then
    r.modules = ASCENT_MODULES
    -- Exporter sidecar slopes in game units. Flat balcony/upper landing are
    -- represented by geometry.platforms; never add a horizontal stair proxy.
    r.geometry.lines = {
      {x0=-52,y0=0,x1=-26,y1=13,kind='floor',passthrough=true,ledges=false,part='bf_stairs_4m_rise2m'},
      {x0=0,y0=13,x1=26,y1=26,kind='floor',passthrough=true,ledges=false,part='bf_ramp_4m_rise2m'},
    }
    -- The ascent transform (top doorway agrees with the top anchor) is the only
    -- layout that declares `upper_doorway`; validate keys the ascent check off
    -- it so new module layouts do not need a top socket.
    r.upper_doorway = ASCENT_MODULES[#ASCENT_MODULES]
  else
    if opts.modules then r.modules = opts.modules end
    if opts.lines then r.geometry.lines = opts.lines end
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

  -- Gate 3 bulk expansion. Each new recipe is a distinct movement demand, not a
  -- translated or recoloured copy of a reviewed one, and uses only the ten
  -- cached BF models. Platforms are one-way (drop-through) ledges, which is
  -- itself a different demand from the reviewed solid ascent landings.

  -- Zig-zag traversal: two storeys, alternating sides, no doorway.
  traverse_stagger = recipe('traverse_stagger',
    {platform(-26, 13, 20), platform(0, 26, 20), platform(26, 13, 20)},
    {left = LEFT, right = RIGHT},
    {modules = {floor_module(-26, 13, 20), floor_module(0, 26, 20), floor_module(26, 13, 20)}}),

  -- Catwalk traversal: two ground-connected side landings bridging a high centre.
  traverse_bridge = recipe('traverse_bridge',
    {platform(-39, 13, 26), platform(39, 13, 26), platform(0, 26, 26)},
    {left = LEFT, right = RIGHT},
    {modules = {floor_module(-39, 13, 26), floor_module(39, 13, 26), floor_module(0, 26, 26)}}),

  -- Flanking combat arena: a continuous one-storey walkway reached by two ramps.
  -- Every edge of the walkway is an interior seam (ramp at -26 and +26, the two
  -- bays meeting at 0), so the surfaces carry no ledges.
  combat_flank = recipe('combat_flank',
    {seam_platform(-13, 13, 26), seam_platform(13, 13, 26)},
    {left = LEFT, right = RIGHT},
    {modules = {ramp_module(-39, 0), floor_module(-13, 13, 26), floor_module(13, 13, 26), ramp_module(39, 0, true)},
     lines = {
       {x0 = -52, y0 = 0, x1 = -26, y1 = 13, kind = 'floor', passthrough = true, ledges = false, part = 'bf_ramp_4m_rise2m'},
       {x0 = 26, y0 = 13, x1 = 52, y1 = 0, kind = 'floor', passthrough = true, ledges = false, part = 'bf_ramp_4m_rise2m'},
     }}),

  -- Single raised combat dais: open floor around one one-way centre platform.
  combat_dais = recipe('combat_dais',
    {platform(0, 13, 26)},
    {left = LEFT, right = RIGHT},
    {modules = {floor_module(0, 13, 26)}}),

  -- Ring combat: a low centre with two high flanking ledges, distinct spacing.
  combat_ring = recipe('combat_ring',
    {platform(0, 13, 26), platform(-26, 26, 20), platform(26, 26, 20)},
    {left = LEFT, right = RIGHT},
    {modules = {floor_module(0, 13, 26), floor_module(-26, 26, 20), floor_module(26, 26, 20)}}),

  -- Boss-capable dais with two high side ledges, distinct from the flat boss arena.
  boss_dais = recipe('boss_dais',
    {platform(0, 13, 26), platform(-39, 26, 20), platform(39, 26, 20)},
    {left = LEFT, right = RIGHT},
    {modules = {floor_module(0, 13, 26), floor_module(-39, 26, 20), floor_module(39, 26, 20)}}),

  -- Rest landing: two quiet side ledges off the main floor.
  rest_platform = recipe('rest_platform',
    {platform(-26, 13, 20), platform(26, 13, 20)},
    {left = LEFT, right = RIGHT},
    {modules = {floor_module(-26, 13, 20), floor_module(26, 13, 20)}}),

  -- Reward ledge: one low side landing and one high centre cache.
  reward_ledge = recipe('reward_ledge',
    {platform(-39, 13, 26), platform(0, 26, 26)},
    {left = LEFT, right = RIGHT},
    {modules = {floor_module(-39, 13, 26), floor_module(0, 26, 26)}}),

  -- Crossing connector: a real drop-through floor with a one-way bridge above it,
  -- so a run can descend through the gap or cross above it (distinct from both
  -- the reviewed ascent-with-drop and the drop-only shortcut).
  crossing_door = recipe('crossing_door',
    {platform(0, 13, 20)},
    {left = LEFT, right = RIGHT, bottom = BOTTOM},
    {modules = {floor_module(0, 13, 20)}, openings = {{x = DROP_X, width = DROP_WIDTH}}}),
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

-- Segments represent physical floor spans; drop openings are never filled.
function RoomRecipes.floor_segments(floor)
  if type(floor)~='table' or not finite(floor.left) or not finite(floor.right)
    or floor.left>=floor.right or not finite(floor.y) then return nil,'invalid floor' end
  local holes={}
  for _,o in ipairs(floor.openings or {}) do
    if type(o)~='table' or not finite(o.x) or not finite(o.width) or o.width<=0 then return nil,'invalid opening' end
    holes[#holes+1]={left=o.x-o.width/2,right=o.x+o.width/2}
  end
  table.sort(holes,function(a,b) return a.left<b.left end)
  local result,cursor={},floor.left
  for _,o in ipairs(holes) do
    if o.left<cursor or o.right>floor.right then return nil,'overlapping or out-of-bounds opening' end
    if o.left>cursor then result[#result+1]={left=cursor,right=o.left,y=floor.y} end
    cursor=o.right
  end
  if cursor<floor.right then result[#result+1]={left=cursor,right=floor.right,y=floor.y} end
  return result
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
      if type(g.kit)~='table' then return false,'recipe '..r.id..' missing kit' end
      if g.kit.unit ~= 6.5 or g.kit.grid ~= 13 or g.kit.bay ~= 26 or g.kit.height ~= 26 then return false, 'recipe ' .. r.id .. ' kit mismatch' end
      local segments,segment_why=RoomRecipes.floor_segments(g.floor)
      if not segments then return false,'recipe '..r.id..': '..segment_why end
      for _,line in ipairs(g.lines or {}) do
        if line.kind~='floor' or not finite(line.x0) or not finite(line.x1) or not finite(line.y0) or not finite(line.y1)
          or line.x0>=line.x1 or line.x0 < -65 or line.x1 >65 or line.y0<0 or line.y1>60 then
          return false,'recipe '..r.id..' invalid collision line'
        end
      end
      for _, p in ipairs(g.platforms) do
        if not finite(p.x) or not finite(p.y) or not finite(p.width) or p.width <= 0 then return false, 'recipe ' .. r.id .. ' invalid platform' end
        if p.x - p.width / 2 < -65 or p.x + p.width / 2 > 65 or p.y < 0 or p.y > 60 then return false, 'recipe ' .. r.id .. ' platform out of bounds' end
        if type(p.passthrough) ~= 'boolean' or type(p.ledges) ~= 'boolean' then return false, 'recipe ' .. r.id .. ' platform flags' end
        if not p.passthrough and (not r.modules or (p.x~=-13 and p.x~=39)) then return false, 'recipe ' .. r.id .. ' solid platform needs clearance validation' end
      end
      -- Floor openings must be in bounds and non-overlapping.
      local openings = g.floor.openings or {}
      for i, o in ipairs(openings) do
        if type(o)~='table' or not finite(o.x) or not finite(o.width) or o.width <= 0 then return false, 'recipe ' .. r.id .. ' invalid opening' end
        if o.x~=DROP_X or o.width~=DROP_WIDTH then return false,'recipe '..r.id..' opening differs from authored kit' end
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
      -- Every destination arrival sits on an actual floor/landing, never over a gap.
      if type(g.arrivals)~='table' then return false,'recipe '..r.id..' missing arrivals' end
      for _,socket in ipairs(template.sockets) do
        local a=g.arrivals[socket.side]
        if type(a)~='table' or not finite(a.x) or not finite(a.y) or (a.facing~=1 and a.facing~=-1) then
          return false,'recipe '..r.id..' invalid arrival for '..socket.side
        end
        local supported=false
        for _,s in ipairs(segments) do if a.x>=s.left+2 and a.x<=s.right-2 and a.y==s.y then supported=true end end
        for _,p in ipairs(g.platforms) do if a.x>=p.x-p.width/2+2 and a.x<=p.x+p.width/2-2 and a.y==p.y then supported=true end end
        if not supported then return false,'recipe '..r.id..' unsafe arrival for '..socket.side end
      end
      -- Visual modules must be well-formed. Only the reviewed ascent declares an
      -- `upper_doorway`; key the top-anchor agreement off it so new module
      -- layouts (platforms and ramps) are not forced to own a top socket.
      if r.modules then
        for _, module in ipairs(r.modules) do
          if type(module.part) ~= 'string' or not finite(module.x) or not finite(module.y) then
            return false, 'recipe ' .. r.id .. ' invalid module'
          end
          if module.scale_x ~= nil and not finite(module.scale_x) then return false, 'recipe ' .. r.id .. ' invalid module scale' end
          if module.scale_y ~= nil and not finite(module.scale_y) then return false, 'recipe ' .. r.id .. ' invalid module scale' end
        end
      end
      if r.upper_doorway then
        local doorway, top = r.upper_doorway, g.exit_anchors.top
        if type(doorway.part) ~= 'string' or doorway.part ~= 'bf_wall_doorway_4m' then
          return false, 'recipe ' .. r.id .. ' ascent doorway malformed'
        end
        if not top then return false, 'recipe ' .. r.id .. ' ascent needs a top anchor and doorway' end
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
  local surfaces = {}
  local segments,why=RoomRecipes.floor_segments(recipe.geometry.floor)
  if not segments then return false,why end
  for _,s in ipairs(segments) do surfaces[#surfaces+1]={lo=s.left,hi=s.right,y=s.y} end
  if #surfaces==0 then return false,'no floor' end
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

-- Directed analytical mobility screening (Gate 3). These envelopes are
-- conservative screening parameters, NOT engine-measured fighter values: they
-- only reject layouts that cannot possibly work, and every admitted layout
-- still needs a normal-speed native replay before certification. `fall_gap`
-- separates a controlled drop from an upward jump, so a one-way descent is not
-- mistaken for a two-way route.
RoomRecipes.mobility_profiles = {
  standard    = {jump_height = 18, horizontal_gap = 30, fall_gap = 45},
  short_heavy = {jump_height = 13, horizontal_gap = 20, fall_gap = 30},
  floaty      = {jump_height = 26, horizontal_gap = 30, fall_gap = 45},
  fast_faller = {jump_height = 13, horizontal_gap = 26, fall_gap = 60},
  multi_jump  = {jump_height = 26, horizontal_gap = 36, fall_gap = 60},
}

local function profile_of(profile)
  if type(profile) == 'string' then return RoomRecipes.mobility_profiles[profile], profile end
  return profile, 'custom'
end

local function gap_between(a, b) return math.max(0, b.lo - a.hi, a.lo - b.hi) end

-- Directed edge test between two flat/near-flat surfaces. Upward motion needs a
-- jump; downward motion only needs the horizontal fall allowance. Never
-- symmetric. Non-finite coordinates (including NaN) are always refused.
function RoomRecipes.can_traverse(a, b, profile)
  local prof = profile_of(profile)
  if type(a) ~= 'table' or type(b) ~= 'table' or type(prof) ~= 'table' then return false end
  if not (finite(a.lo) and finite(a.hi) and finite(a.y) and finite(b.lo) and finite(b.hi) and finite(b.y)) then return false end
  local dy = b.y - a.y
  local gap = gap_between(a, b)
  if dy > 0 then return dy <= prof.jump_height and gap <= prof.horizontal_gap end
  if dy == 0 then return gap <= prof.horizontal_gap end
  return gap <= (prof.fall_gap or prof.horizontal_gap)
end

-- Reject non-finite movement geometry before any screen builds surfaces. Checks
-- NaN as well as infinity (finite() requires v == v and |v| < inf).
function RoomRecipes.geometry_finite(recipe)
  if type(recipe) ~= 'table' or type(recipe.geometry) ~= 'table' then return false, 'no geometry' end
  local g = recipe.geometry
  if type(g.floor) ~= 'table' or not (finite(g.floor.left) and finite(g.floor.right) and finite(g.floor.y)) then
    return false, 'nonfinite floor'
  end
  for _, o in ipairs(g.floor.openings or {}) do
    if type(o) ~= 'table' or not (finite(o.x) and finite(o.width)) then return false, 'nonfinite opening' end
  end
  for _, p in ipairs(g.platforms or {}) do
    if type(p) ~= 'table' or not (finite(p.x) and finite(p.y) and finite(p.width)) then return false, 'nonfinite platform' end
  end
  for _, l in ipairs(g.lines or {}) do
    if type(l) ~= 'table' or not (finite(l.x0) and finite(l.y0) and finite(l.x1) and finite(l.y1)) then return false, 'nonfinite line' end
  end
  for side, a in pairs(g.exit_anchors or {}) do
    if type(a) ~= 'table' or not (finite(a.x) and finite(a.y)) then return false, 'nonfinite anchor ' .. tostring(side) end
  end
  for side, a in pairs(g.arrivals or {}) do
    if type(a) ~= 'table' or not (finite(a.x) and finite(a.y)) then return false, 'nonfinite arrival ' .. tostring(side) end
  end
  return true
end

-- Flat surfaces plus the two endpoints of every authored slope, which are linked
-- in both directions because a ramp is walkable up and down.
local function traversal_surfaces(recipe)
  local nodes = {}
  local segments = RoomRecipes.floor_segments(recipe.geometry.floor)
  if not segments then return nil, 'invalid floor' end
  for _, s in ipairs(segments) do
    if not (finite(s.left) and finite(s.right) and finite(s.y)) then return nil, 'nonfinite floor segment' end
    nodes[#nodes + 1] = {lo = s.left, hi = s.right, y = s.y, kind = 'floor'}
  end
  for _, p in ipairs(recipe.geometry.platforms or {}) do
    if not (finite(p.x) and finite(p.y) and finite(p.width)) then return nil, 'nonfinite platform' end
    nodes[#nodes + 1] = {lo = p.x - p.width / 2, hi = p.x + p.width / 2, y = p.y, kind = 'platform'}
  end
  local first_slope = #nodes + 1
  for _, line in ipairs(recipe.geometry.lines or {}) do
    if not (finite(line.x0) and finite(line.y0) and finite(line.x1) and finite(line.y1)) then return nil, 'nonfinite line' end
    local ia, ib = #nodes + 1, #nodes + 2
    nodes[ia] = {lo = line.x0, hi = line.x0, y = line.y0, kind = 'slope', link = ib}
    nodes[ib] = {lo = line.x1, hi = line.x1, y = line.y1, kind = 'slope', link = ia}
  end
  return nodes, first_slope
end

-- Directed reachability from the actual arrival surface for one mobility
-- profile. It seeds only the surface at `opts.entry` (the recipe spawn by
-- default), never every floor, so disconnected floors must really be crossed.
-- Requires every required (non-slope) surface, every non-drop exit anchor and
-- every drop edge to be reachable, so a raised unreachable surface fails.
function RoomRecipes.screen(recipe, profile, opts)
  if type(recipe) ~= 'table' or not recipe.geometry then return false, 'not a supported recipe' end
  local ok, why = RoomRecipes.geometry_finite(recipe)
  if not ok then return false, why end
  local prof, name = profile_of(profile)
  if type(prof) ~= 'table' then return false, 'unknown mobility profile ' .. tostring(profile) end
  local nodes = traversal_surfaces(recipe)
  if not nodes then return false, 'invalid floor' end
  local entry = (opts and opts.entry) or recipe.geometry.spawn
  if type(entry) ~= 'table' or not (finite(entry.x) and finite(entry.y)) then return false, 'invalid entry point' end
  local entry_index
  for i, s in ipairs(nodes) do
    if s.kind ~= 'slope' and math.abs(entry.y - s.y) < 0.01 and entry.x >= s.lo and entry.x <= s.hi then entry_index = i break end
  end
  if not entry_index then return false, name .. ': entry is not on a surface' end
  local seen, queue = {[entry_index] = true}, {entry_index}
  local head = 1
  while queue[head] do
    local from = nodes[queue[head]]; head = head + 1
    for j, to in ipairs(nodes) do
      if not seen[j] then
        local linked = from.kind == 'slope' and from.link == j
        if linked or RoomRecipes.can_traverse(from, to, prof) then
          seen[j] = true; queue[#queue + 1] = j
        end
      end
    end
  end
  -- Every required surface (floor span or platform) must be reachable from the
  -- arrival. A raised platform no route reaches is a defect, not decoration.
  for i, s in ipairs(nodes) do
    if s.kind ~= 'slope' and not seen[i] then
      return false, name .. ': unreachable surface at y=' .. tostring(s.y) ..
        ' [' .. tostring(s.lo) .. ',' .. tostring(s.hi) .. ']'
    end
  end
  local floor_y = recipe.geometry.floor.y
  local function supported(anchor)
    for i, s in ipairs(nodes) do
      if seen[i] and s.kind ~= 'slope' and math.abs(anchor.y - s.y) < 0.01 and anchor.x >= s.lo and anchor.x <= s.hi then return true end
    end
    return false
  end
  local openings = recipe.geometry.floor.openings or {}
  for side, anchor in pairs(recipe.geometry.exit_anchors) do
    if anchor.drop then
      local opening
      for _, o in ipairs(openings) do
        if math.abs(anchor.x - o.x) <= o.width / 2 then opening = o end
      end
      if not opening then return false, name .. ': drop anchor ' .. side .. ' has no opening' end
      local edge = false
      for i, s in ipairs(nodes) do
        if seen[i] and s.kind == 'floor' and math.abs(s.y - floor_y) < 0.01
          and (math.abs(s.hi - (opening.x - opening.width / 2)) < 0.01 or math.abs(s.lo - (opening.x + opening.width / 2)) < 0.01) then
          edge = true
        end
      end
      if not edge then return false, name .. ': drop anchor ' .. side .. ' has no reachable floor edge' end
    elseif not supported(anchor) then
      return false, name .. ': anchor ' .. side .. ' is not on a reachable surface'
    end
  end
  return true
end

-- Honest, theme-free geometry signature: movement geometry only (floor span,
-- openings, platforms, slopes and module placements). Camera, spawn, arrivals,
-- socket/topology metadata and theme are ignored, and the structure is
-- translated so its minimum x/y sits at the origin, so a translated copy (or a
-- palette/shape relabel) is the same layout.
local function quant(v) return math.floor(v * 1000 + 0.5) end

function RoomRecipes.geometry_signature(recipe)
  if type(recipe) ~= 'table' or type(recipe.geometry) ~= 'table' then return tostring(recipe) end
  local g = recipe.geometry
  local pts = {}
  local function pt(x, y) pts[#pts + 1] = {x, y} end
  pt(g.floor.left, g.floor.y); pt(g.floor.right, g.floor.y)
  for _, o in ipairs(g.floor.openings or {}) do pt(o.x - o.width / 2, g.floor.y); pt(o.x + o.width / 2, g.floor.y) end
  for _, p in ipairs(g.platforms or {}) do pt(p.x - p.width / 2, p.y); pt(p.x + p.width / 2, p.y) end
  for _, l in ipairs(g.lines or {}) do pt(l.x0, l.y0); pt(l.x1, l.y1) end
  for _, m in ipairs(recipe.modules or {}) do pt(m.x, m.y) end
  local minx, miny = math.huge, math.huge
  for _, p in ipairs(pts) do minx = math.min(minx, p[1]); miny = math.min(miny, p[2]) end
  if minx == math.huge then minx, miny = 0, 0 end
  local function nx(x) return quant(x - minx) end
  local function ny(y) return quant(y - miny) end
  local out = {}
  local holes = {}
  for _, o in ipairs(g.floor.openings or {}) do holes[#holes + 1] = nx(o.x - o.width / 2) .. ':' .. nx(o.x + o.width / 2) end
  table.sort(holes); out[#out + 1] = 'h[' .. table.concat(holes, ',') .. ']'
  local plats = {}
  for _, p in ipairs(g.platforms or {}) do
    plats[#plats + 1] = string.format('%d@%d w%d p%d l%d', nx(p.x), ny(p.y), nx(p.width), p.passthrough and 1 or 0, p.ledges and 1 or 0)
  end
  table.sort(plats); out[#out + 1] = 'p[' .. table.concat(plats, ';') .. ']'
  local lines = {}
  for _, l in ipairs(g.lines or {}) do
    local a, b = {nx(l.x0), ny(l.y0)}, {nx(l.x1), ny(l.y1)}
    if a[1] > b[1] or (a[1] == b[1] and a[2] > b[2]) then a, b = b, a end
    lines[#lines + 1] = string.format('%d,%d-%d,%d %s', a[1], a[2], b[1], b[2], tostring(l.kind))
  end
  table.sort(lines); out[#out + 1] = 'l[' .. table.concat(lines, ';') .. ']'
  local mods = {}
  for _, m in ipairs(recipe.modules or {}) do
    mods[#mods + 1] = string.format('%d@%d %s %.3f,%.3f', nx(m.x), ny(m.y), tostring(m.part), m.scale_x or 1, m.scale_y or 1)
  end
  table.sort(mods); out[#out + 1] = 'm[' .. table.concat(mods, ';') .. ']'
  return table.concat(out, '|')
end

-- Backwards-compatible name; layout diversity is geometry diversity. The
-- template argument is accepted but ignored so a palette, shape or socket-name
-- change never inflates the count.
function RoomRecipes.signature(recipe, _template)
  return RoomRecipes.geometry_signature(recipe)
end

return RoomRecipes
