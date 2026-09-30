-- @name: Roguelite room certification probe
-- @version: 1.0.0
--
-- This file is NOT loaded as a standalone mod. tools/roguelite/certify_rooms.py
-- prepends the reviewed pure modules (RoomCatalogue, RoomRecipes, Rooms) exactly
-- as tools/roguelite/prepare.py does, then installs the bundle as an isolated
-- gameplay mod. The probe:
--
--   * resolves an UNCERTIFIED recipe directly through RoomRecipes.resolve,
--   * preloads the BF kit one model per tick, spawns the reviewed visual layout,
--     builds explicit floor segments / platforms / slope lines for collision,
--   * acquires Final Destination isolation so only the authored room remains,
--   * exposes fixture placement and bounded per-frame trace sampling.
--
-- It never sets recipe.certified and it never feeds the production adapter. A
-- teleport is a labelled fixture (certify_place); it is never traversal evidence.
-- Controller input is replayed by the Python driver through the real pad API at
-- normal 60 Hz speed. This script performs no input and holds no pad.

local VERSION = 1

local S = {
  version = VERSION,
  phase = 'idle',        -- idle | loading | visuals | collision | isolate | ready | error
  template = nil,
  node = nil,
  recipe = nil,
  recipe_version = nil,
  certified = nil,
  plan = nil,
  visuals = nil,
  handles = {},
  collision_count = 0,
  isolated = false,
  hud = nil,
  error = nil,
  arm = nil,
  sample_limit = 900,
  last_cleanup = nil,
}

local function say(fmt, ...)
  gd.log(string.format(fmt, ...))
end

local function fail(where, message)
  S.phase = 'error'
  S.error = tostring(message)
  say('certify_error where=%s message=%s', where, S.error)
  return false, S.error
end

local function flag(v)
  if v == true then return 'true' end
  if v == false then return 'false' end
  return 'nil'
end

-- The build path needs the current review API. Report precisely what is absent
-- instead of silently degrading to a partial room.
local BUILD_APIS = {
  'model_load', 'model_spawn', 'model_despawn', 'model_release',
  'stage_add_platform', 'stage_add_line', 'stage_isolate',
}
local function missing_apis()
  local missing = {}
  for _, name in ipairs(BUILD_APIS) do
    if gd[name] == nil then missing[#missing + 1] = name end
  end
  return missing
end

local function socket_for(node, id)
  if type(id) ~= 'string' then return nil end
  for _, socket in ipairs(node.exits or {}) do
    if socket.socket == id or socket.side == id then return socket end
  end
  return nil
end

local function arrival_for(node, socket)
  local arrivals = (node.room and node.room.arrivals) or {}
  return arrivals[socket.socket] or arrivals[socket.side]
end

local function anchor_for(node, socket)
  local anchors = (node.room and node.room.exit_anchors) or {}
  return anchors[socket.side]
end

-- Mirror the runtime node view but resolve the recipe directly: the Adapter's
-- certification gate is intentionally not consulted here.
local function make_node(template_id)
  local template = RoomCatalogue.get(template_id)
  if not template then return nil, 'unknown template ' .. tostring(template_id) end
  local geometry, recipe = RoomRecipes.resolve(template)
  if not geometry then return nil, tostring(recipe) end
  local exits = {}
  for _, socket in ipairs(template.sockets) do
    exits[#exits + 1] = {
      side = socket.side, socket = socket.id,
      anchor = geometry.exit_anchors[socket.side], to = socket.id,
    }
  end
  return {
    id = 'certify_' .. template_id, kind = 'traversal', template_id = template_id,
    recipe = template.recipe, recipe_version = recipe.version,
    recipe_modules = recipe.modules or {}, room = geometry, exits = exits, theme = 'cobalt',
  }, recipe
end

local function build_collision()
  local plan = S.plan
  local handles = {}
  S.handles = handles -- retain partial builds for cleanup on any refusal
  for _, seg in ipairs(plan.floor_segments) do
    local h, why = gd.stage_add_platform((seg.left + seg.right) / 2, seg.y, seg.right - seg.left,
      {passthrough = false, ledges = true, draw = true})
    if not h then return nil, 'floor segment at ' .. tostring(seg.left) .. ': ' .. tostring(why) end
    handles[#handles + 1] = h
  end
  for _, p in ipairs(plan.platforms) do
    local h, why = gd.stage_add_platform(p.x, p.y, p.width,
      {passthrough = p.passthrough, ledges = p.ledges, draw = true})
    if not h then return nil, 'platform at ' .. tostring(p.x) .. ': ' .. tostring(why) end
    handles[#handles + 1] = h
  end
  for _, line in ipairs(plan.lines) do
    local h, why = gd.stage_add_line(line.x0, line.y0, line.x1, line.y1, 'floor',
      {passthrough = line.passthrough, ledges = line.ledges, draw = true})
    if not h then return nil, 'line at ' .. tostring(line.x0) .. ': ' .. tostring(why) end
    handles[#handles + 1] = h
  end
  S.handles = handles
  return true, #handles
end

local function cleanup()
  local errors = {}
  local collider_count = #S.handles
  local pending = {}
  for _, h in ipairs(S.handles) do
    local ok, removed = pcall(gd.stage_remove, h)
    if not ok or removed ~= true then
      errors[#errors + 1] = 'stage_remove ' .. tostring(h)
      pending[#pending + 1] = h
    end
  end
  S.handles = pending
  if S.visuals then
    local ok, cleared, why = pcall(Rooms.clear, S.visuals)
    if not (ok and cleared) then
      -- Instances remain: keep ownership (and the assets they reference) for retry.
      errors[#errors + 1] = 'visuals ' .. tostring(ok and why or cleared)
    else
      local ok2, released, why2 = pcall(Rooms.release, S.visuals)
      if not (ok2 and released) then errors[#errors + 1] = 'release ' .. tostring(ok2 and why2 or released) end
      if ok2 and released then S.visuals = nil end
    end
  end
  if S.isolated and gd.stage_isolate then
    -- isolate(false) restores and returns false on success.
    local ok = pcall(gd.stage_isolate, false)
    if not ok then errors[#errors + 1] = 'stage_isolate release' end
  end
  S.isolated = false
  S.arm = nil
  S.last_cleanup = { colliders = collider_count, errors = errors }
  if S.hud ~= nil and gd.hud_visible then pcall(gd.hud_visible, true) end
  S.hud = nil
  return errors
end

local function advance()
  if S.phase == 'loading' then
    local ready, why = Rooms.preload_step(S.visuals, S.node)
    if ready == nil then fail('preload', why)
    elseif ready then S.phase = 'visuals' end
  elseif S.phase == 'visuals' then
    local ok, why = Rooms.enter(S.visuals, S.node)
    if not ok then fail('visuals', why) else S.phase = 'collision' end
  elseif S.phase == 'collision' then
    local ok, count = build_collision()
    if not ok then fail('collision', count)
    else S.collision_count = count; S.phase = 'isolate' end
  elseif S.phase == 'isolate' then
    local ok, result = pcall(gd.stage_isolate, true)
    if not ok or result ~= true then
      fail('isolate', ok and ('refused: ' .. tostring(result)) or result)
    else
      S.isolated = true; S.phase = 'ready'
      if gd.hud_visible then S.hud = gd.hud_visible(false) end
      say('certify_built template=%s phase=ready parts=%d colliders=%d isolated=true',
        tostring(S.template), #(S.plan and S.plan.parts or {}), S.collision_count)
    end
  end
end

function on_tick()
  if S.phase ~= 'idle' and S.phase ~= 'error' and S.phase ~= 'ready' then advance() end
end

function on_frame()
  if S.phase ~= 'ready' or not S.arm then return end
  local p = gd.player(1)
  if not p then return end
  local a = S.arm
  if a.count >= S.sample_limit then a.truncated = true; return end
  a.count = a.count + 1
  a.samples[a.count] = {
    frame = gd.match().frame, x = p.x, y = p.y, vx = p.vx, vy = p.vy,
    air = p.airborne and 1 or 0, act = p.action,
  }
end

-- Console commands -----------------------------------------------------------------

gd.command('certify_status', function()
  say('certify_status diag_version=%d api_version=%s phase=%s template=%s recipe=%s recipe_version=%s certified=%s isolated=%s colliders=%d samples=%s error=%s',
    S.version, tostring(gd.api_version), S.phase, tostring(S.template),
    S.recipe and 'true' or 'false', tostring(S.recipe_version), flag(S.certified),
    flag(gd.stage_isolate and gd.stage_isolate() or false), #S.handles, S.arm and tostring(S.arm.count) or '0', tostring(S.error))
end, 'certification probe status')

gd.command('certify_apis', function()
  for _, name in ipairs(BUILD_APIS) do
    say('certify_api name=%s present=%s', name, flag(gd[name] ~= nil))
  end
  for _, name in ipairs({'stage_bounds', 'camera_get', 'model_get'}) do
    say('certify_api name=%s present=%s', name, flag(gd[name] ~= nil))
  end
end, 'report current API availability')

gd.command('certify_recipe', function(arg)
  local id = (arg or ''):match('^(%S+)')
  local node, recipe = make_node(id)
  if not node then fail('recipe', recipe); return end
  local g = node.room
  say('certify_recipe template=%s recipe=%s version=%s certified=%s mobility=%s modules=%d platforms=%d lines=%d openings=%d floor_left=%s floor_right=%s',
    tostring(id), tostring(node.recipe), tostring(recipe.version), flag(recipe.certified),
    tostring(recipe.mobility_contract), #(recipe.modules or {}), #g.platforms, #g.lines,
    #(g.floor.openings or {}), tostring(g.floor.left), tostring(g.floor.right))
  for _, socket in ipairs(node.exits) do
    local a = anchor_for(node, socket) or {}
    local r = arrival_for(node, socket) or {}
    say('certify_socket template=%s socket=%s side=%s ax=%s ay=%s drop=%s rx=%s ry=%s facing=%s',
      tostring(id), socket.socket, socket.side, tostring(a.x), tostring(a.y), flag(a.drop),
      tostring(r.x), tostring(r.y), tostring(r.facing))
  end
  for i, o in ipairs(g.floor.openings or {}) do
    say('certify_opening template=%s index=%d x=%s width=%s', tostring(id), i, tostring(o.x), tostring(o.width))
  end
end, 'resolve an uncertified recipe directly (no certification gate)')

gd.command('certify_build', function(arg)
  local id = (arg or ''):match('^(%S+)')
  local missing = missing_apis()
  if #missing > 0 then
    for _, name in ipairs(missing) do say('certify_missing_api name=%s', name) end
    fail('build', 'missing native API: ' .. table.concat(missing, ',')); return
  end
  if not (gd.match and gd.match().active) then fail('build', 'no active offline match'); return end
  local node, recipe = make_node(id)
  if not node then fail('build', recipe); return end
  local plan, why = Rooms.plan(node)
  if not plan then fail('build', tostring(why)); return end
  if S.visuals then
    local errors = cleanup()
    if #errors > 0 then fail('build', 'previous cleanup refused: ' .. table.concat(errors, '; ')); return end
  end
  S.template, S.node, S.recipe, S.plan = id, node, recipe, plan
  S.recipe_version, S.certified = recipe.version, recipe.certified
  S.visuals, S.phase, S.error = Rooms.new(), 'loading', nil
  say('certify_build_start template=%s recipe=%s recipe_version=%s certified=%s parts=%d platforms=%d lines=%d floor_segments=%d',
    tostring(id), tostring(node.recipe), tostring(recipe.version), flag(recipe.certified),
    #plan.parts, #plan.platforms, #plan.lines, #plan.floor_segments)
end, 'load, build and isolate an uncertified recipe')

gd.command('certify_place', function(arg)
  local id = (arg or ''):match('^(%S+)')
  local socket = socket_for(S.node, id)
  if not socket then fail('place', 'no such socket: ' .. tostring(id)); return end
  local a = arrival_for(S.node, socket) or anchor_for(S.node, socket)
  if not a then fail('place', 'socket has no arrival or anchor: ' .. tostring(id)); return end
  local ok = pcall(gd.teleport, 1, a.x, a.y + 2)
  if not ok then fail('place', 'teleport refused'); return end
  say('certify_place socket=%s x=%s y=%s fixture=true note=placement-only-not-traversal', socket.socket, tostring(a.x), tostring(a.y))
end, 'FINISH fixture teleport to a socket arrival (inspection only)')

gd.command('certify_place_at', function(arg)
  local x, y = (arg or ''):match('^(%-?[%d%.]+)%s+(%-?[%d%.]+)')
  if not x then fail('place_at', 'usage: certify_place_at <x> <y>'); return end
  local ok = pcall(gd.teleport, 1, tonumber(x), tonumber(y) + 2)
  if not ok then fail('place_at', 'teleport refused'); return end
  say('certify_place x=%s y=%s fixture=true note=placement-only-not-traversal', x, y)
end, 'FINISH fixture teleport to a coordinate (inspection only)')

local function open_window(label, target, socket_name)
  local p = gd.player(1)
  S.arm = {socket = socket_name, label = label, target = target,
    floor_y = S.node and S.node.room.floor.y or 0, samples = {}, count = 0,
    start_x = p and p.x or nil, start_y = p and p.y or nil, truncated = false}
  say('certify_arm label=%s socket=%s target_x=%s target_y=%s floor_y=%s start_x=%s start_y=%s limit=%d',
    label, socket_name or 'window', target and tostring(target.x) or 'none',
    target and tostring(target.y) or 'none', tostring(S.arm.floor_y),
    tostring(S.arm.start_x), tostring(S.arm.start_y), S.sample_limit)
end

gd.command('certify_arm', function(arg)
  local id, label = (arg or ''):match('^(%S+)%s*(%S*)$')
  local socket = socket_for(S.node, id)
  if not socket then fail('arm', 'no such socket: ' .. tostring(id)); return end
  local a = arrival_for(S.node, socket) or anchor_for(S.node, socket)
  if not a then fail('arm', 'socket has no arrival: ' .. tostring(id)); return end
  open_window((label ~= '' and label) or socket.socket, {x = a.x, y = a.y}, socket.socket)
end, 'start a bounded per-frame trace window targeting a socket arrival')

-- A window without a socket target (drop/fall segments). x/y may be 'none'.
gd.command('certify_arm_coords', function(arg)
  local label, x, y = (arg or ''):match('^(%S+)%s+(%S+)%s+(%S+)$')
  if not label then fail('arm_coords', 'usage: certify_arm_coords <label> <x|none> <y|none>'); return end
  local target = nil
  if x ~= 'none' then target = {x = tonumber(x), y = tonumber(y)} end
  if x ~= 'none' and (not target.x or not target.y) then fail('arm_coords', 'invalid target'); return end
  open_window(label, target, 'window')
end, 'start a bounded trace window with or without a target region')

gd.command('certify_result', function()
  if not S.arm then fail('result', 'no armed window'); return end
  local a = S.arm
  local min_y, max_y, air = math.huge, -math.huge, 0
  for i = 1, a.count do
    local s = a.samples[i]
    if s.y < min_y then min_y = s.y end
    if s.y > max_y then max_y = s.y end
    air = air + s.air
  end
  say('certify_result label=%s socket=%s target_x=%s target_y=%s floor_y=%s samples=%d truncated=%s min_y=%s max_y=%s air_frames=%d start_x=%s start_y=%s',
    a.label, a.socket or 'window', a.target and tostring(a.target.x) or 'none',
    a.target and tostring(a.target.y) or 'none', tostring(a.floor_y), a.count,
    flag(a.truncated), tostring(a.count > 0 and min_y or nil), tostring(a.count > 0 and max_y or nil),
    air, tostring(a.start_x), tostring(a.start_y))
  local limit = math.min(a.count, 20)
  for i = 1, limit do
    local s = a.samples[i]
    say('certify_trace i=%d frame=%d x=%.3f y=%.3f vx=%.3f vy=%.3f airborne=%s action=%d',
      i, s.frame, s.x, s.y, s.vx, s.vy, flag(s.air == 1), s.act)
  end
  if a.count > limit then say('certify_trace_more remaining=%d use=certify_trace', a.count - limit) end
end, 'close and emit the bounded trace window')

gd.command('certify_trace', function(arg)
  if not S.arm then fail('trace', 'no armed window'); return end
  local offset, count = (arg or ''):match('^(%d+)%s+(%d+)$')
  offset, count = tonumber(offset) or 1, math.min(tonumber(count) or 20, 20)
  local a = S.arm
  for i = offset, math.min(a.count, offset + count - 1) do
    local s = a.samples[i]
    say('certify_trace i=%d frame=%d x=%.3f y=%.3f vx=%.3f vy=%.3f airborne=%s action=%d',
      i, s.frame, s.x, s.y, s.vx, s.vy, flag(s.air == 1), s.act)
  end
end, 'page the bounded trace window: certify_trace <offset> <count>')

gd.command('certify_sample', function()
  local p = gd.player(1)
  if not p then fail('sample', 'no fighter on port 1'); return end
  say('certify_sample port=1 x=%.3f y=%.3f vx=%.3f vy=%.3f airborne=%s action=%d facing=%d',
    p.x, p.y, p.vx, p.vy, flag(p.airborne), p.action, p.facing)
end, 'read one fighter observation')

gd.command('certify_bounds', function()
  if not gd.stage_bounds then say('certify_bounds available=false reason=gd.stage_bounds-missing'); return end
  local b = gd.stage_bounds()
  if not b then say('certify_bounds available=false reason=no-match'); return end
  local function rect(name, r) if r then say('certify_bounds kind=%s left=%s right=%s top=%s bottom=%s', name, tostring(r.left), tostring(r.right), tostring(r.top), tostring(r.bottom)) end end
  say('certify_bounds available=true surface_top=%s', tostring(b.surface_top))
  rect('blast', b.blast); rect('camera', b.camera); rect('main_floor', b.main_floor)
end, 'report stage blast/camera bounds')

gd.command('certify_cleanup', function()
  local errors = cleanup()
  S.phase, S.template, S.node, S.plan, S.recipe, S.recipe_version, S.certified = 'idle', nil, nil, nil, nil, nil, nil
  S.collision_count, S.error = 0, nil
  for _, e in ipairs(errors) do say('certify_cleanup_error message=%s', (tostring(e):gsub('%s', '_'))) end
  say('certify_cleanup errors=%d', #errors)
end, 'remove collision, visuals and isolation')

gd.command('certify_scene', function(arg)
  local fighter, costume = (arg or ''):match('^(%S+)%s*(%S*)$')
  if not fighter or fighter == '' then fail('scene', 'usage: certify_scene <fighter> [costume]'); return end
  local p1 = fighter .. '/c' .. (tonumber(costume) or 0)
  local ok, result = pcall(gd.scene_launch,
    {mode = 'vs', p1 = p1, p2 = 'fox/c0/cpu9', stage = 'fd', stocks = 99, items = 'off', time = 0})
  say('certify_scene p1=%s ok=%s result=%s', p1, flag(ok), tostring(result))
end, 'launch the isolated FD certification match for a fighter')

gd.command('certify_hud', function(arg)
  if not gd.hud_visible then fail('hud', 'gd.hud_visible missing'); return end
  local on = (arg or ''):match('^(%S+)')
  if on ~= 'on' and on ~= 'off' then fail('hud', 'usage: certify_hud <on|off>'); return end
  local value = on == 'off'
  local ok, result = pcall(gd.hud_visible, not value)
  S.hud = value
  say('certify_hud hidden=%s ok=%s result=%s', flag(value), flag(ok), tostring(result))
end, 'hide or restore the native status HUD')

function on_match_end()
  -- This notification follows native scene teardown: the old model/collision
  -- handles are already invalid. Forget them here only; live cleanup retains
  -- refused handles for retry.
  if S.visuals then Rooms.reset(S.visuals) end
  S.visuals, S.handles, S.arm, S.hud = nil, {}, nil, nil
  S.isolated, S.phase = false, 'idle'
end
function on_unload() cleanup() end

return { version = VERSION, state = S, cleanup = cleanup }
