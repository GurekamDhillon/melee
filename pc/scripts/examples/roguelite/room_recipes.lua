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

return RoomRecipes
