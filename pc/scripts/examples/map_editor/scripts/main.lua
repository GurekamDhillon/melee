-- Offline editor and map loader. Models belong to this mod's models/ directory.
-- No require/io: layout files live in gd.data_read/write's script-local sandbox.
-- BEGIN GENERATED KIT
local U = 6.5
local PALETTE = {
  "bf_balcony_4m",
  "bf_balcony_4m_glass",
  "bf_beam_4m",
  "bf_corner_inside_4m",
  "bf_corner_outside_4m",
  "bf_door_leaf",
  "bf_floor_2m",
  "bf_floor_4m",
  "bf_floor_end_trim",
  "bf_floor_opening_4m",
  "bf_ramp_4m_rise2m",
  "bf_ramp_4m_rise2m_glass",
  "bf_rear_glass_rail_4m",
  "bf_rear_glass_rail_4m_glass",
  "bf_rear_post_4m",
  "bf_stairs_4m_rise2m",
  "bf_wall_doorway_4m",
  "bf_wall_side_return",
  "bf_wall_solid_4m",
  "bf_wall_window_4m",
  "bf_window_glass_insert_glass",
}
-- END GENERATED KIT
-- BEGIN GENERATED MISSION (scripts/mission.lua; regenerate with tools/port/map_mission_sync.py)
local Mission = (function()
-- Mission state machine for the kit map editor. Pure Lua: no gd calls, so it loads under plain lua
-- and the tests drive it with made-up observations. main.lua embeds this file verbatim between
-- "BEGIN/END GENERATED MISSION" (the engine loads one entry file per mod and has no require);
-- regenerate it with tools/port/map_mission_sync.py after editing.
local M = {}
-- Keep in step with gs_enemy_names in pc/platform/gw_script.c (topi is registered but not yet in the docs).
M.KINDS = { 'goomba', 'koopa', 'redead', 'like_like', 'octorok', 'polar_bear', 'topi' }
-- per_wave: gd.spawn_enemy allows 32 live script enemies; waves, checkpoints and the rest are authoring caps.
M.LIMITS = { enemies = 64, per_wave = 32, waves = 8, checkpoints = 16, zone = 2000, time = 3600, lives = 99,
             triggers = 16, text = 80, radius = 200 }
-- Trigger actions: wave (spawn a wave now), message (HUD line), collision (open or close the parts near
-- a point), complete / fail (end the mission).
M.ACTIONS = { wave = true, message = true, collision = true, complete = true, fail = true }
M.MESSAGE_FRAMES = 240 -- how long a message stays on the HUD
M.OBJECTIVES = { reach_goal = 'Reach the goal', defeat_all = 'Defeat all enemies',
                 defeat_then_goal = 'Defeat all enemies, then reach the goal' }
M.FPS = 60 -- logic frames per second; time limits count on_frame calls
-- P1 respawns through the engine's own sequence (platform and all) at stage spawn slot 4; the state
-- machine only says where that point is ('respawn_point' actions: the start, then each checkpoint).
M.RESPAWN_SLOT = 4

local known = {}
for _, k in ipairs(M.KINDS) do known[k] = true end

local function num(n) return type(n) == 'number' and n == n and math.abs(n) <= 100000 end
local function plain(t, what) assert(type(t) == 'table' and getmetatable(t) == nil, what .. ' must be a table') end
local function only(t, allowed, what)
  for k in pairs(t) do assert(allowed[k], 'unknown ' .. what .. ' field: ' .. tostring(k)) end
end
local function list(t, what, max)
  plain(t, what .. ' list')
  local n = 0
  for k in pairs(t) do
    assert(type(k) == 'number' and k % 1 == 0 and k >= 1, what .. ' must be a plain list')
    n = n + 1
  end
  assert(n == #t, what .. ' must be a plain list')
  assert(n <= max, 'too many ' .. what .. ' (max ' .. max .. ')')
  return t
end
local function point(t, what)
  plain(t, what) only(t, { x = true, y = true }, what)
  assert(num(t.x) and num(t.y), what .. ' needs finite x and y')
  return { x = t.x, y = t.y }
end
local function zone(t, what)
  plain(t, what) only(t, { x = true, y = true, w = true, h = true }, what)
  assert(num(t.x) and num(t.y), what .. ' needs finite x and y')
  assert(num(t.w) and num(t.h) and t.w > 0 and t.h > 0 and t.w <= M.LIMITS.zone and t.h <= M.LIMITS.zone,
         what .. ' needs w and h in (0, ' .. M.LIMITS.zone .. ']')
  return { x = t.x, y = t.y, w = t.w, h = t.h }
end

-- Strict structural check; returns a normalized copy (lists always present, wave defaulted). Whether
-- the mission is complete enough to play is check_playable's job, so authoring can go step by step.
function M.validate(m)
  plain(m, 'mission')
  only(m, { start = true, enemies = true, goal = true, checkpoints = true, objective = true,
            waves = true, triggers = true }, 'mission')
  local out = { enemies = {}, checkpoints = {}, waves = {}, triggers = {} }
  if m.start ~= nil then out.start = point(m.start, 'start') end
  if m.goal ~= nil then out.goal = zone(m.goal, 'goal') end
  local per = {}
  local es = m.enemies == nil and {} or list(m.enemies, 'enemies', M.LIMITS.enemies)
  for i, e in ipairs(es) do
    plain(e, 'enemy') only(e, { kind = true, x = true, y = true, wave = true }, 'enemy')
    assert(known[e.kind], 'unknown enemy kind: ' .. tostring(e.kind))
    assert(num(e.x) and num(e.y), 'enemy needs finite x and y')
    local w = e.wave == nil and 1 or e.wave
    assert(type(w) == 'number' and w % 1 == 0 and w >= 1 and w <= M.LIMITS.waves,
           'enemy wave must be an integer 1..' .. M.LIMITS.waves)
    per[w] = (per[w] or 0) + 1
    assert(per[w] <= M.LIMITS.per_wave, 'too many enemies in wave ' .. w .. ' (max ' .. M.LIMITS.per_wave .. ')')
    out.enemies[i] = { kind = e.kind, x = e.x, y = e.y, wave = w }
  end
  local cs = m.checkpoints == nil and {} or list(m.checkpoints, 'checkpoints', M.LIMITS.checkpoints)
  for i, c in ipairs(cs) do out.checkpoints[i] = zone(c, 'checkpoint') end
  -- Wave rules: a ruled wave appears on its own condition (seconds since the start, or P1 crossing x)
  -- instead of waiting for the earlier waves to be cleared.
  local ws = m.waves == nil and {} or list(m.waves, 'waves', M.LIMITS.waves)
  local ruled = {}
  for i, r in ipairs(ws) do
    plain(r, 'wave rule') only(r, { wave = true, time = true, x = true, dir = true }, 'wave rule')
    assert(type(r.wave) == 'number' and r.wave % 1 == 0 and r.wave >= 1 and r.wave <= M.LIMITS.waves,
           'wave rule needs an integer wave 1..' .. M.LIMITS.waves)
    assert(not ruled[r.wave], 'wave ' .. r.wave .. ' has two rules')
    ruled[r.wave] = true
    assert((r.time ~= nil) ~= (r.x ~= nil), 'wave rule needs exactly one of time or x')
    assert(r.time == nil or (num(r.time) and r.time >= 0 and r.time <= M.LIMITS.time),
           'wave rule time must be 0..' .. M.LIMITS.time .. ' seconds')
    assert(r.x == nil or num(r.x), 'wave rule x must be finite')
    assert(r.dir == nil or (r.x ~= nil and (r.dir == 1 or r.dir == -1)), 'wave rule dir is 1 or -1 and goes with x')
    out.waves[i] = { wave = r.wave, time = r.time, x = r.x, dir = r.x ~= nil and (r.dir or 1) or nil }
  end
  -- Trigger zones: entering one runs its action (once unless once = false).
  local ts = m.triggers == nil and {} or list(m.triggers, 'triggers', M.LIMITS.triggers)
  for i, t in ipairs(ts) do
    plain(t, 'trigger')
    only(t, { x = true, y = true, w = true, h = true, action = true, wave = true, text = true, at = true,
              r = true, open = true, once = true }, 'trigger')
    local z = zone({ x = t.x, y = t.y, w = t.w, h = t.h }, 'trigger')
    assert(M.ACTIONS[t.action], 'trigger action must be wave, message, collision, complete or fail')
    assert(t.once == nil or type(t.once) == 'boolean', 'trigger once must be true or false')
    local o = { x = z.x, y = z.y, w = z.w, h = z.h, action = t.action, once = t.once ~= false }
    local function only_for(field, action)
      assert(t[field] == nil or t.action == action,
             'trigger field ' .. field .. ' only goes with the ' .. action .. ' action')
    end
    only_for('wave', 'wave') only_for('text', 'message') only_for('at', 'collision')
    only_for('r', 'collision') only_for('open', 'collision')
    if t.action == 'wave' then
      assert(type(t.wave) == 'number' and t.wave % 1 == 0 and t.wave >= 1 and t.wave <= M.LIMITS.waves,
             'trigger wave must be an integer 1..' .. M.LIMITS.waves)
      o.wave = t.wave
    elseif t.action == 'message' then
      assert(type(t.text) == 'string' and #t.text >= 1 and #t.text <= M.LIMITS.text and not t.text:find('%c'),
             'trigger text must be 1..' .. M.LIMITS.text .. ' printable characters')
      o.text = t.text
    elseif t.action == 'collision' then
      o.at = point(t.at, 'trigger at')
      assert(type(t.open) == 'boolean', 'trigger collision needs open = true or false')
      o.open = t.open
      assert(t.r == nil or (num(t.r) and t.r > 0 and t.r <= M.LIMITS.radius),
             'trigger r must be in (0, ' .. M.LIMITS.radius .. ']')
      o.r = t.r or 6.5
    end
    out.triggers[i] = o
  end
  local o = m.objective
  if o ~= nil then
    plain(o, 'objective') only(o, { type = true, time = true, lives = true }, 'objective')
    assert(M.OBJECTIVES[o.type], 'objective must be reach_goal, defeat_all or defeat_then_goal')
    assert(o.time == nil or (num(o.time) and o.time > 0 and o.time <= M.LIMITS.time),
           'objective time must be 1..' .. M.LIMITS.time .. ' seconds')
    assert(o.lives == nil or (type(o.lives) == 'number' and o.lives % 1 == 0 and o.lives >= 1 and o.lives <= M.LIMITS.lives),
           'objective lives must be an integer 1..' .. M.LIMITS.lives)
    out.objective = { type = o.type, time = o.time, lives = o.lives }
  end
  return out
end

function M.empty(m)
  return m == nil or (not m.start and not m.goal and not m.objective and #m.enemies == 0 and #m.checkpoints == 0
                      and #m.waves == 0 and #m.triggers == 0)
end

-- nil when the mission can run, else what is missing.
function M.check_playable(m)
  if not m.start then return 'mission has no start (map mission start)' end
  if not m.objective then return 'mission has no objective (map mission objective <type>)' end
  local t = m.objective.type
  if (t == 'reach_goal' or t == 'defeat_then_goal') and not m.goal then
    return t .. ' needs a goal zone (map mission goal <w> <h>)'
  end
  if (t == 'defeat_all' or t == 'defeat_then_goal') and #m.enemies == 0 then
    return t .. ' needs at least one enemy (map mission enemy <kind>)'
  end
  local have = {}
  for _, e in ipairs(m.enemies) do have[e.wave] = true end
  for _, r in ipairs(m.waves) do
    if not have[r.wave] then return 'wave rule for wave ' .. r.wave .. ' but that wave has no enemies' end
  end
  for _, t in ipairs(m.triggers) do
    if t.action == 'wave' and not have[t.wave] then return 'trigger spawns wave ' .. t.wave .. ' but it has no enemies' end
  end
end

local function inside(z, p)
  return p ~= nil and math.abs(p.x - z.x) <= z.w / 2 and math.abs(p.y - z.y) <= z.h / 2
end

function M.new(m)
  local seen, waves = {}, {}
  for _, e in ipairs(m.enemies) do
    if not seen[e.wave] then seen[e.wave] = true waves[#waves + 1] = e.wave end
  end
  table.sort(waves)
  -- A wave is sequential (starts when everything before it is gone) unless a rule or a trigger owns it.
  local rule, owned = {}, {}
  for _, r in ipairs(m.waves or {}) do rule[r.wave] = r owned[r.wave] = true end
  for _, t in ipairs(m.triggers or {}) do if t.action == 'wave' then owned[t.wave] = true end end
  return { m = m, waves = waves, rule = rule, owned = owned, begun = {}, nbegun = 0, tracked = {}, pending = 0,
           frames = 0, deaths = 0, falls = nil, cp = nil, started = false, defeated = 0, vanished = 0,
           total = #m.enemies, cleared = #waves == 0, result = nil, fail = nil, fired = {}, inside_now = {} }
end

-- The glue reports each spawn action's outcome before the next step.
function M.spawned(st, index, handle)
  st.pending = st.pending - 1
  st.tracked[handle] = { index = index, frame = st.frames }
end
function M.spawn_failed(st, index, why)
  st.pending = st.pending - 1
  local e = st.m.enemies[index]
  st.fail = st.fail or { reason = 'spawn', detail = ('%s refused: %s'):format(e and e.kind or 'enemy', tostring(why)) }
end

local function sorted_handles(t)
  local out = {}
  for h in pairs(t) do out[#out + 1] = h end
  table.sort(out)
  return out
end

-- obs: {frames = logic frames since the mission began, player = {x,y} or nil, falls = P1's fall count,
-- alive = {[handle] = true} for the enemies still fighting, defeated = {[handle] = true} for the ones
-- the engine reports as stock-defeated (omitted: every enemy that is gone counts as defeated)}.
-- A tracked enemy that is neither alive nor defeated has vanished (blast zone, item destroyed): it
-- leaves the fight so the objective cannot stall, but it is not counted as a defeat.
-- Returns actions (spawn / respawn_point / cleanup, for the glue to carry out) and events (for logging).
function M.step(st, obs)
  local actions, events = {}, {}
  if st.result then return actions, events end
  st.frames = obs.frames
  local m, obj = st.m, st.m.objective

  local function finish(status, reason, detail)
    st.result = { status = status, reason = reason, detail = detail, frames = st.frames,
                  deaths = st.deaths, defeated = st.defeated, vanished = st.vanished }
    actions[#actions + 1] = { type = 'cleanup' }
    events[#events + 1] = { type = status, reason = reason, detail = detail, frames = st.frames,
                            deaths = st.deaths, defeated = st.defeated, vanished = st.vanished }
  end
  if st.fail then finish('failed', st.fail.reason, st.fail.detail) return actions, events end

  -- The start is the first respawn point.
  if not st.started then
    st.started = true
    if m.start then actions[#actions + 1] = { type = 'respawn_point', x = m.start.x, y = m.start.y } end
  end

  -- Defeats. A handle is not judged on the frame it spawned: the pool may not report it yet.
  for _, h in ipairs(sorted_handles(st.tracked)) do
    local t = st.tracked[h]
    if obs.frames > t.frame and not obs.alive[h] then
      st.tracked[h] = nil
      local kind = m.enemies[t.index].kind
      if obs.defeated == nil or obs.defeated[h] then
        st.defeated = st.defeated + 1
        events[#events + 1] = { type = 'defeat', handle = h, index = t.index, kind = kind }
      else
        st.vanished = st.vanished + 1
        events[#events + 1] = { type = 'vanish', handle = h, index = t.index, kind = kind }
      end
    end
  end

  -- Triggers fire on entry. They run before the waves so a triggered wave appears the same step.
  local function begin_wave(w)
    if st.begun[w] then return end
    st.begun[w] = true st.nbegun = st.nbegun + 1
    local px = obs.player and obs.player.x or (m.start and m.start.x) or 0
    for i, e in ipairs(m.enemies) do
      if e.wave == w then
        st.pending = st.pending + 1
        actions[#actions + 1] = { type = 'spawn', index = i, kind = e.kind, x = e.x, y = e.y,
                                  wave = w, facing = px >= e.x and 1 or -1 }
      end
    end
    events[#events + 1] = { type = 'wave', n = st.nbegun, of = #st.waves, wave = w }
  end
  for i, t in ipairs(m.triggers) do
    local now = inside(t, obs.player)
    if now and not st.inside_now[i] and not (t.once and st.fired[i]) then
      st.fired[i] = true
      events[#events + 1] = { type = 'trigger', index = i, action = t.action }
      if t.action == 'wave' then begin_wave(t.wave)
      elseif t.action == 'message' then events[#events + 1] = { type = 'message', text = t.text }
      elseif t.action == 'collision' then
        actions[#actions + 1] = { type = 'collision', x = t.at.x, y = t.at.y, r = t.r, open = t.open }
      elseif t.action == 'complete' then finish('complete') return actions, events
      elseif t.action == 'fail' then finish('failed', 'trigger', 'trigger zone ' .. i) return actions, events
      end
    end
    st.inside_now[i] = now
  end

  -- Waves. Ruled waves start on their own condition; the rest go in order once everything spawned is gone.
  for _, w in ipairs(st.waves) do
    local r = st.rule[w]
    if r and not st.begun[w] then
      local px = obs.player and obs.player.x
      if (r.time and obs.frames >= r.time * M.FPS) or
         (r.x and px and ((r.dir == 1 and px >= r.x) or (r.dir == -1 and px <= r.x))) then
        begin_wave(w)
      end
    end
  end
  if next(st.tracked) == nil and st.pending == 0 then
    for _, w in ipairs(st.waves) do
      if not st.owned[w] and not st.begun[w] then begin_wave(w) break end
    end
  end
  if not st.cleared and st.nbegun == #st.waves and next(st.tracked) == nil and st.pending == 0 then
    st.cleared = true
  end

  -- Checkpoints: touching one makes it the respawn point.
  local held = st.cp and inside(m.checkpoints[st.cp], obs.player) -- overlapping zones must not flip-flop
  for i, z in ipairs(m.checkpoints) do
    if not held and inside(z, obs.player) and st.cp ~= i then
      st.cp = i
      actions[#actions + 1] = { type = 'respawn_point', x = z.x, y = z.y }
      events[#events + 1] = { type = 'checkpoint', index = i }
      break
    end
  end

  -- Deaths: the fall count rising is the KO. Lives only fail the mission when the author set them.
  if st.falls == nil then st.falls = obs.falls end
  local lost = obs.falls - st.falls
  if lost > 0 then
    st.falls = obs.falls st.deaths = st.deaths + lost
    events[#events + 1] = { type = 'death', deaths = st.deaths,
                            remaining = obj.lives and math.max(0, obj.lives - st.deaths) or nil }
  end

  local goal_open = obj.type == 'reach_goal' or (obj.type == 'defeat_then_goal' and st.cleared)
  if (obj.type == 'defeat_all' and st.cleared) or (goal_open and inside(m.goal, obs.player)) then
    finish('complete') return actions, events
  end
  if obj.lives and st.deaths >= obj.lives then finish('failed', 'lives') return actions, events end
  if obj.time and obs.frames >= obj.time * M.FPS then finish('failed', 'time') return actions, events end
  return actions, events
end

local function clock(frames, up)
  local s = math.max(0, (up and math.ceil or math.floor)(frames / M.FPS))
  return ('%d:%02d'):format(s // 60, s % 60)
end

-- One HUD line: objective, enemies left, time (remaining when limited, else elapsed), lives left.
function M.hud(st)
  local o = st.m.objective
  local parts = { o.type == 'reach_goal' and 'Reach the goal' or o.type == 'defeat_all' and 'Defeat all'
                  or (st.cleared and 'Reach the goal' or 'Defeat all, then the goal') }
  if st.total > 0 then parts[#parts + 1] = 'enemies ' .. (st.total - st.defeated - st.vanished) end
  if o.time then
    parts[#parts + 1] = 'time ' .. clock(o.time * M.FPS - st.frames, true)
  else
    parts[#parts + 1] = 'time ' .. clock(st.frames)
  end
  if o.lives then parts[#parts + 1] = 'lives ' .. math.max(0, o.lives - st.deaths) end
  return table.concat(parts, ' | ')
end

local WHY = { time = 'time ran out', lives = 'out of lives' }
function M.result_text(st)
  local r = st.result
  if not r then return nil end
  if r.status == 'complete' then return 'MISSION COMPLETE  ' .. clock(r.frames) end
  return 'MISSION FAILED: ' .. (WHY[r.reason] or r.detail or r.reason)
end

return M
end)()
-- END GENERATED MISSION

local known = {}
for _, name in ipairs(PALETTE) do known[name] = true end
local parts, handles, assets, undo, redo = {}, {}, {}, {}, {}
local selected, next_id, palette = nil, 0, 1
local filter, filtering, recents, toasts = '', false, {}, {}
local editing, menu, action_index = false, false, 1
-- Authoring scale is separate from the exported base asset unit U.
-- Existing layout transforms remain literal; only new parts and reset use 2x.
local AUTHOR_SCALE = 2
local depth, rotation, grid = 0, 0, 2
local collision, floor_flags, overlay = true, 3, true
local previous, old_pad, saved_states = nil, {}, {}
local filename, dirty, status = 'layout.lua', false, 'F1 help | F6 edit | map play layout.lua: load'
local autoload, broken = nil, false
local modal, error_text, last_action = nil, nil, nil -- hybrid-transform state; status severities (bible §4.3, §5.6, §5.9)
local field_drag, typing, handles_ui = nil, nil, nil -- inspector scrub / typed entry / gizmo hit geometry
local ghost_on, ghost = true, nil -- placement preview instance (bible §5.8)
local action_log, undo_names, redo_names, log_open, log_rows = {}, {}, {}, false, nil -- named history (bible §5.6)
local search, search_rows = nil, nil -- action search overlay (bible §5.10)
local bounds = {camera=nil, blast=nil} -- stage bounds the map is authored against (bible §6.6, §8 P2)
local spawns = {} -- moved start/respawn points, slot -> {x=, y=} (bible §6.3)
local mission_doc = nil -- authored mission markers (see mission.lua); part of the document and its history
local run = nil -- the running mission: {state=, frames=, last_falls=}; kept after the result so the banner stays
local mission_stop -- forward: start() ends a running mission before editing resumes
local mission_pick -- forward: 'map mission pick' (defined with the marker tools)
local MK = { kind = 'enemy', pick = { enemy = 'goomba', wave = 1, action = 'message' } } -- marker tools (state: sel, kind, pick, drag)
local want_edit = nil -- {left=} frames to keep retrying start() after a test (the fighter may be mid-move)
local test_return = nil -- where P1 was in the editor when 'map mission test' began; editing resumes there
local bounds_drag = nil -- dragging a camera-bounds edge (bible §8 P2)
local group = {} -- extra selected part ids beside the anchor (bible §5.4)
local marquee = nil -- select-tool drag rectangle: {sx,sy,x,y,add} (bible §5.4)
local hover_gizmo = nil -- the handle or bounds edge under the pointer (bible §5.11)
local MAX_PARTS, HISTORY = 128, 64 -- ScriptGame_ModelSpawn's instance pool; shared with other mods.
local TOOLS = { 'place', 'select', 'move', 'rotate', 'scale', 'mission' }
local tool, axis_lock, snap_on, help_open = 'place', nil, true, false
local SCALE_STEP, SCALE_MIN, SCALE_MAX = 1.1, 0.25, 4.0
local hover, dragging, help_first = nil, false, 1
local PANEL_FILL = 0x0E1218F2 -- opaque editor panels: the game behind must not read through the text
local HINTS = {
  place='LMB/Ins: place; Up/Down: part; wheel: depth; hold G/E/C: transform',
  select='LMB / Tab: select the nearest part; F: frame; hold G/E/C: transform',
  move='hold G: drag to move; M: to cursor; Shift+C: constraint',
  rotate='hold E: drag to face the pointer; R / T: +-15 deg',
  scale='hold C: drag to scale; F7 / F8: -10% / +10%; Shift+X/Y: mirror',
  mission='LMB: place the marker / select / drag to move or resize; Up/Down: kind; Del: delete',
}
local mouse = { x = -1000, y = -1000, buttons = 0, prev = 0, over = false, used = 0 }
local map_hinv, map_key = nil, nil
local mouse_api = gd.mouse ~= nil and gd.camera_get ~= nil

local function say(s, kind)
  status = tostring(s)
  if kind == 'error' then error_text = status
  elseif kind == 'action' then last_action = status error_text = nil end
  gd.log('map_editor: ' .. (kind == 'error' and 'error: ' or '') .. status)
end
local function offline()
  local m = gd.match()
  return m.active and not m.netplay
end
local function clone(t)
  local out = {}
  for k,v in pairs(t) do out[k] = type(v)=='table' and clone(v) or v end
  return out
end
local function number(n)
  return type(n)=='number' and n==n and math.abs(n)<=100000
end
local function sign(n) return n < 0 and -1 or 1 end
-- Ctrl (hold) snaps hard to the grid, Shift (hold) takes a quarter step (bible §4.5).
local function snap(n)
  local s = U*grid
  if gd.key('SHIFT') then s = s * 0.25 end
  return math.floor(n/s+0.5)*s
end
local function sn(v) return snap_on and snap(v) or v end
local function fly_cursor()
  local p=assert(gd.player(1), 'P1 is required for the flight cursor')
  return snap(p.x), snap(p.y), depth
end
local function find(list, id)
  for i,p in ipairs(list) do if p.id==id then return p,i end end
end
local function scale_of(p, k) return p[k] or 1 end
local function in_group(id) return group[id]==true end
local function clear_group() group={} end
local function selected_ids()
  local out={}
  if selected then out[#out+1]=selected end
  for _,p in ipairs(parts) do if p.id~=selected and group[p.id] then out[#out+1]=p.id end end
  return out
end
local function palette_category(name)
  local n=name:gsub('^bf_','')
  if n:find('^floor') then return 'floor' end
  if n:find('^stair') then return 'stair' end
  if n:find('^ramp') then return 'ramp' end
  if n:find('^wall') then return 'wall' end
  if n:find('^corner') then return 'corner' end
  if n:find('^beam') or n:find('^post') or n:find('_trim') then return 'trim' end
  if n:find('door') or n:find('opening') then return 'door' end
  if n:find('glass') or n:find('window') then return 'glass' end
  if n:find('balcony') then return 'balcony' end
  return 'other'
end
local function palette_match(name)
  if filter=='' then return true end
  return name:lower():find(filter,1,true)~=nil or palette_category(name):find(filter,1,true)~=nil
end
local function palette_view()
  local out,pinned={},{}
  for _,name in ipairs(recents) do
    for i,n in ipairs(PALETTE) do
      if n==name and palette_match(name) then out[#out+1]=i pinned[i]=true end
    end
  end
  for i,name in ipairs(PALETTE) do
    if not pinned[i] and palette_match(name) then out[#out+1]=i end
  end
  return out
end
local function palette_part()
  local view=palette_view()
  return PALETTE[view[math.min(math.max(palette,1),#view)] or 1]
end
local function remember(name)
  for i,n in ipairs(recents) do if n==name then table.remove(recents,i) break end end
  table.insert(recents,1,name) if #recents>5 then table.remove(recents) end
end
local function toast(msg)
  toasts[#toasts+1]={msg=msg,t=100} if #toasts>3 then table.remove(toasts,1) end
end
local function canvas_w()
  local a=gd.safe_area and gd.safe_area()
  return a and tonumber(a.w) or 640
end

-- world/screen homography --------------------------------------------------------------------
-- The projection is opaque to Lua (fov and aspect live in the engine, and widescreen changes
-- the aspect), so a screen point maps to the depth plane through four projected sample points:
-- world->screen is a plane homography and gd.project supplies the samples; screen->world is its
-- inverse. No camera constants are assumed.
local function solve(a, b, n)
  for col = 1, n do
    local piv, best = col, math.abs(a[col][col])
    for r = col + 1, n do local v = math.abs(a[r][col]) if v > best then piv, best = r, v end end
    if best < 1e-12 then return nil end
    if piv ~= col then a[col], a[piv] = a[piv], a[col] b[col], b[piv] = b[piv], b[col] end
    for r = col + 1, n do
      local f = a[r][col] / a[col][col]
      if f ~= 0 then
        for c = col, n do a[r][c] = a[r][c] - f * a[col][c] end
        b[r] = b[r] - f * b[col]
      end
    end
  end
  local x = {}
  for r = n, 1, -1 do
    local sum = b[r]
    for c = r + 1, n do sum = sum - a[r][c] * x[c] end
    x[r] = sum / a[r][r]
  end
  return x
end
local function inv3(h)
  local a,b,c,d,e,f,g,i,j = h[1],h[2],h[3],h[4],h[5],h[6],h[7],h[8],h[9]
  local A,B,C =  (e*j - f*i), -(d*j - f*g),  (d*i - e*g)
  local D,E,F = -(b*j - c*i),  (a*j - c*g), -(a*i - b*g)
  local G,H,I =  (b*f - c*e), -(a*f - c*d),  (a*e - b*d)
  local det = a*A + b*B + c*C
  if math.abs(det) < 1e-12 then return nil end
  -- the inverse is the TRANSPOSED cofactor matrix over the determinant
  return { A/det,D/det,G/det,B/det,E/det,H/det,C/det,F/det,I/det }
end
local function build_map()
  if not mouse_api then return nil end
  local cam = gd.camera_get()
  if type(cam) ~= 'table' or type(cam.interest) ~= 'table' then return nil end
  local key = ('%g:%g:%g:%g:%g:%g:%g'):format(cam.interest.x, cam.interest.y, cam.interest.z, depth,
                                           cam.eye.x, cam.eye.y, cam.eye.z)
  if key == map_key then return map_hinv end
  local cx, cy = cam.interest.x, cam.interest.y
  local d = 150
  local corners = { {cx-d,cy-d}, {cx+d,cy-d}, {cx+d,cy+d}, {cx-d,cy+d} }
  local a, rhs = {}, {}
  for i = 1, 4 do
    local wx, wy = corners[i][1], corners[i][2]
    local sx, sy = gd.project(wx, wy, depth)
    if not number(sx) or not number(sy) then return nil end
    a[#a+1] = { wx, wy, 1, 0, 0, 0, -sx*wx, -sx*wy } rhs[#rhs+1] = sx
    a[#a+1] = { 0, 0, 0, wx, wy, 1, -sy*wx, -sy*wy } rhs[#rhs+1] = sy
  end
  local h = solve(a, rhs, 8)
  map_key = key
  map_hinv = h and inv3({ h[1],h[2],h[3],h[4],h[5],h[6],h[7],h[8],1 }) or nil
  return map_hinv
end
local function mouse_world()
  local hi = build_map()
  if not hi or not mouse.over then return nil end
  local w = hi[7]*mouse.x + hi[8]*mouse.y + hi[9]
  if math.abs(w) < 1e-9 then return nil end
  local wx = (hi[1]*mouse.x + hi[2]*mouse.y + hi[3]) / w
  local wy = (hi[4]*mouse.x + hi[5]*mouse.y + hi[6]) / w
  if not number(wx) or not number(wy) then return nil end
  return wx, wy
end
-- Display cursor: the mouse point when the pointer has been used recently, else the fly cursor.
local function shown_cursor()
  if mouse_api and mouse.used > 0 then
    local wx, wy = mouse_world()
    if wx then return sn(wx), sn(wy), depth end
  end
  return fly_cursor()
end

local function same(a,b)
  for _,k in ipairs({'part','x','y','z','rot','collision','floor_flags',
                     'scale','scale_x','scale_y','scale_z'}) do
    if a[k]~=b[k] then return false end
  end
  return true
end
local function options(p)
  return {x=p.x,y=p.y,z=p.z,rot=p.rot,collision=p.collision,floor_flags=p.floor_flags,
          scale=scale_of(p,'scale'),scale_x=scale_of(p,'scale_x'),
          scale_y=scale_of(p,'scale_y'),scale_z=scale_of(p,'scale_z')}
end
local function asset(name)
  if not assets[name] then assets[name]=assert(gd.model_load(name)) end
  return assets[name]
end
local function ghost_despawn()
  if ghost then pcall(gd.model_despawn,ghost.handle) ghost=nil end
end
-- The placement ghost is a real instance at 45% alpha with no collision: it must never add pool
-- pressure beyond one slot or collide with the map (bible §5.8).
local function ghost_sync()
  local view=palette_view()
  if not ghost_on or not editing or not offline() or not gd.player(1) or tool~='place' or
     modal or field_drag or typing or #view==0 then
    ghost_despawn()
    return
  end
  local name=PALETTE[view[math.min(math.max(palette,1),#view)]]
  local x,y,z=shown_cursor()
  local ok=#parts<MAX_PARTS
  if ghost and ghost.name~=name then ghost_despawn() end
  local opts={x=x,y=y,z=z,rot=rotation,scale=AUTHOR_SCALE,visible=true,alpha=true,collision=false,floor_flags=0,
              tint=ok and 0xFFFFFFAA or 0xDF4433AA}
  if not ghost then
    local h=gd.model_spawn(asset(name),opts)
    if h then ghost={handle=h,name=name} end
  else
    pcall(gd.model_set,ghost.handle,opts)
  end
end

-- Reconcile by stable document ID: a transform touches one instance, not the whole map.
-- On failure, the caller reconciles back to the last document before advancing history.
-- A sign change in scale_x/scale_y forces a despawn/respawn: the model API rejects effective
-- axis-sign changes on instances that own collision (docs/scripting.md, runtime models).
local function sync(target)
  for id,h in pairs(handles) do
    local p=find(target,id)
    if not p or h.part~=p.part or h.collision~=p.collision or h.floor_flags~=p.floor_flags or
       sign(scale_of(h,'scale_x'))~=sign(scale_of(p,'scale_x')) or
       sign(scale_of(h,'scale_y'))~=sign(scale_of(p,'scale_y')) then
      gd.model_despawn(h.handle) handles[id]=nil
    end
  end
  for _,p in ipairs(target) do
    local h=handles[p.id]
    if h then
      if not same(h,p) then assert(gd.model_set(h.handle,options(p)), 'model update refused') end
    else
      local handle,why=gd.model_spawn(asset(p.part),options(p))
      assert(handle,why or 'model spawn refused')
      h={handle=handle} handles[p.id]=h
    end
    for k,v in pairs(p) do h[k]=v end
  end
end
local function copy_rect(r)
  return r and {left=r.left,right=r.right,top=r.top,bottom=r.bottom}
end
local function doc_snapshot()
  return {parts=clone(parts), bounds={camera=copy_rect(bounds.camera), blast=copy_rect(bounds.blast)},
          spawns=clone(spawns), mission=mission_doc and clone(mission_doc)}
end
local function apply(target, record)
  assert(offline(), 'active offline match required')
  assert(not broken, 'model recovery failed; save the document and restart the match')
  assert(#target<=MAX_PARTS, 'model limit: 128 (shared with other mods)')
  local ok,why=pcall(sync,target)
  if not ok then
    local restored,err=pcall(sync,parts)
    if not restored then broken=true say('Recovery failed: '..tostring(err)) end
    error(why,0)
  end
  if record then
    undo[#undo+1]=doc_snapshot() if #undo>HISTORY then table.remove(undo,1) end
    redo={} redo_names={}
  end
  parts=clone(target) dirty=true
  if not find(parts,selected) then selected=parts[#parts] and parts[#parts].id end
  for id in pairs(group) do if not find(parts,id) then group[id]=nil end end
end
local function name_undo(label)
  if #undo>0 then undo_names[#undo]=label end
  action_log[#action_log+1]=label
  if #action_log>8 then table.remove(action_log,1) end
end
local function acted(target, label)
  apply(target, true)
  name_undo(label)
  say(label, 'action')
end
local function doc_commit(before, label)
  undo[#undo+1]=before if #undo>HISTORY then table.remove(undo,1) end
  redo={} redo_names={}
  name_undo(label) say(label,'action')
end
local function opt_scale(p,k)
  local v=p[k]
  if v==nil then return 1 end
  assert(number(v) and v~=0 and math.abs(v)>=0.001 and math.abs(v)<=100, 'invalid '..k)
  return v
end
local function validate(data)
  assert(type(data)=='table' and getmetatable(data)==nil and (data.version==1 or data.version==2),
         'layout version must be 1 or 2')
  assert(data.units==U, 'kit scale differs; re-export the kit or convert the layout')
  local lay_bounds={camera=nil,blast=nil}
  local lay_mission=nil
  if data.mission~=nil then
    assert(data.version==2, 'mission requires layout version 2')
    lay_mission=Mission.validate(data.mission)
    if Mission.empty(lay_mission) then lay_mission=nil end
  end
  if data.version==2 then
    for _,kind in ipairs({'camera','blast'}) do
      local b=data[kind]
      if b~=nil then
        assert(type(b)=='table' and getmetatable(b)==nil, kind..' bounds must be a table')
        for _,k in ipairs({'left','right','top','bottom'}) do assert(number(b[k]), 'invalid '..kind..' '..k) end
        assert(b.left<b.right and b.bottom<b.top, kind..' bounds require left < right and bottom < top')
        lay_bounds[kind]={left=b.left,right=b.right,top=b.top,bottom=b.bottom}
      end
    end
    local sp=data.spawn
    if sp~=nil then
      assert(type(sp)=='table' and getmetatable(sp)==nil, 'spawn must be a table')
      lay_bounds.spawn={}
      for slot,b in pairs(sp) do
        assert(type(slot)=='number' and slot%1==0 and ((slot>=0 and slot<=7) or (slot>=127 and slot<=146)),
               'spawn slots are 0-7 (starts/respawns) or 127-146 (item spawns)')
        assert(type(b)=='table' and getmetatable(b)==nil and number(b.x) and number(b.y),
               'spawn points need x and y')
        lay_bounds.spawn[slot]={x=b.x,y=b.y}
      end
    end
  end
  assert(type(data.parts)=='table' and getmetatable(data.parts)==nil, 'parts must be a list')
  local count=0
  for k in pairs(data.parts) do
    assert(type(k)=='number' and k%1==0 and k>=1 and k<=MAX_PARTS, 'invalid part index')
    count=count+1
  end
  assert(count==#data.parts and count<=MAX_PARTS, 'sparse or oversized part list')
  local out={}
  for _,p in ipairs(data.parts) do
    assert(type(p)=='table' and getmetatable(p)==nil and known[p.part], 'unknown kit part')
    for _,k in ipairs({'x','y','z','rot'}) do assert(number(p[k]), 'invalid '..k) end
    assert(math.abs(p.rot)<=360, 'rotation must be -360..360')
    assert(type(p.collision)=='boolean', 'collision must be boolean')
    assert(type(p.floor_flags)=='number' and p.floor_flags%1==0 and p.floor_flags>=0 and p.floor_flags<=3,
           'floor_flags must be 0..3')
    next_id=next_id+1
    out[#out+1]={id=next_id,part=p.part,x=p.x,y=p.y,z=p.z,rot=p.rot,
                collision=p.collision,floor_flags=p.floor_flags,
                scale=opt_scale(p,'scale'),scale_x=opt_scale(p,'scale_x'),
                scale_y=opt_scale(p,'scale_y'),scale_z=opt_scale(p,'scale_z')}
  end
  return out, lay_bounds, lay_mission
end
local function file_name(name)
  assert(type(name)=='string' and #name<=80 and name:match('^[%w_-]+%.lua$'),
         'use a plain filename such as layout.lua')
  return name
end
local function serialize()
  local version=((bounds.camera or bounds.blast) or next(spawns) or mission_doc) and 2 or 1
  local out={('-- Kit layout v%d; world units, Z is visual depth. Grid: %.17g units/metre.'):format(version,U)}
  out[#out+1]=('return {version=%d,units=%.17g,'):format(version,U)
  for _,kind in ipairs({'camera','blast'}) do
    local b=bounds[kind]
    if b then
      out[#out+1]=('%s={left=%.17g,right=%.17g,top=%.17g,bottom=%.17g},')
        :format(kind,b.left,b.right,b.top,b.bottom)
    end
  end
  if next(spawns) then
    out[#out+1]='spawn={'
    local slots={}
    for slot in pairs(spawns) do slots[#slots+1]=slot end
    table.sort(slots)
    for _,slot in ipairs(slots) do
      out[#out+1]=('[%d]={x=%.17g,y=%.17g},'):format(slot,spawns[slot].x,spawns[slot].y)
    end
    out[#out+1]='},'
  end
  if mission_doc then
    local m=mission_doc
    local function z(t) return ('{x=%.17g,y=%.17g,w=%.17g,h=%.17g}'):format(t.x,t.y,t.w,t.h) end
    out[#out+1]='mission={'
    if m.start then out[#out+1]=('start={x=%.17g,y=%.17g},'):format(m.start.x,m.start.y) end
    if #m.enemies>0 then
      out[#out+1]='enemies={'
      for _,e in ipairs(m.enemies) do
        out[#out+1]=('{kind=%q,x=%.17g,y=%.17g,wave=%d},'):format(e.kind,e.x,e.y,e.wave)
      end
      out[#out+1]='},'
    end
    if #m.waves>0 then
      out[#out+1]='waves={'
      for _,r in ipairs(m.waves) do
        out[#out+1]=r.time and ('{wave=%d,time=%.17g},'):format(r.wave,r.time) or
                    ('{wave=%d,x=%.17g,dir=%d},'):format(r.wave,r.x,r.dir)
      end
      out[#out+1]='},'
    end
    if #m.triggers>0 then
      out[#out+1]='triggers={'
      for _,t in ipairs(m.triggers) do
        local f=('{x=%.17g,y=%.17g,w=%.17g,h=%.17g,action=%q'):format(t.x,t.y,t.w,t.h,t.action)
        if t.wave then f=f..(',wave=%d'):format(t.wave) end
        if t.text then f=f..(',text=%q'):format(t.text) end
        if t.at then f=f..(',at={x=%.17g,y=%.17g},open=%s,r=%.17g'):format(t.at.x,t.at.y,tostring(t.open),t.r) end
        if not t.once then f=f..',once=false' end
        out[#out+1]=f..'},'
      end
      out[#out+1]='},'
    end
    if m.goal then out[#out+1]='goal='..z(m.goal)..',' end
    if #m.checkpoints>0 then
      out[#out+1]='checkpoints={'
      for _,c in ipairs(m.checkpoints) do out[#out+1]=z(c)..',' end
      out[#out+1]='},'
    end
    if m.objective then
      local o=m.objective
      out[#out+1]=('objective={type=%q%s%s},'):format(o.type,o.time and (',time=%.17g'):format(o.time) or '',
                                                        o.lives and (',lives=%d'):format(o.lives) or '')
    end
    out[#out+1]='},'
  end
  out[#out+1]='parts={'
  for _,p in ipairs(parts) do
    local fields=('{part=%q,x=%.17g,y=%.17g,z=%.17g,rot=%.17g,collision=%s,floor_flags=%d'):format(
      p.part,p.x,p.y,p.z,p.rot,tostring(p.collision),p.floor_flags)
    for _,k in ipairs({'scale','scale_x','scale_y','scale_z'}) do
      local v=scale_of(p,k)
      if v~=1 then fields=fields..(',%s=%.17g'):format(k,v) end
    end
    out[#out+1]=fields..'},'
  end
  out[#out+1]='}}\n' return table.concat(out,'\n')
end
local function save(name)
  name=file_name(name or filename)
  local text=serialize()
  -- Preserve the last file separately; read-back also catches short native writes.
  local old=gd.data_read(name)
  if old then gd.data_write(name..'.bak',old) end
  gd.data_write(name,text)
  assert(gd.data_read(name)==text, 'save read-back failed; previous file is in .bak')
  filename=name dirty=false say('Saved '..#parts..' parts to '..name) toast('Saved '..#parts..' parts')
end
local function apply_bounds()
  if bounds.camera then
    assert(gd.stage_set_camera_bounds(bounds.camera.left,bounds.camera.right,bounds.camera.top,bounds.camera.bottom))
  end
  if bounds.blast then
    assert(gd.stage_set_blast_bounds(bounds.blast.left,bounds.blast.right,bounds.blast.top,bounds.blast.bottom))
  end
end
local function doc_restore(d)
  local old_bounds,old_spawns,old_mission=bounds,spawns,mission_doc
  -- The mission's start is P1's slot-0 spawn and wins over a hand-set slot 0.
  local function effective(s,m)
    local out=clone(s)
    if m and m.start then out[0]={x=m.start.x,y=m.start.y} end
    return out
  end
  local function reconcile(b,s,m)
    -- Restore also clears every owned spawn. Omitted document fields mean defaults.
    assert(gd.stage_restore_bounds(), 'stage restore refused')
    bounds=b
    apply_bounds()
    for slot,p in pairs(effective(s,m)) do assert(gd.stage_set_spawn(slot,p.x,p.y), 'spawn refused: '..slot) end
  end
  local next_bounds={camera=copy_rect(d.bounds.camera),blast=copy_rect(d.bounds.blast)}
  local next_spawns=clone(d.spawns or {})
  local next_mission=d.mission and clone(d.mission) or nil
  local ok,why=pcall(reconcile,next_bounds,next_spawns,next_mission)
  if not ok then
    local recovered,err=pcall(reconcile,old_bounds,old_spawns,old_mission)
    bounds,spawns=old_bounds,old_spawns
    if not recovered then broken=true say('Arena recovery failed: '..tostring(err),'error') end
    error(why,0)
  end
  bounds,spawns,mission_doc=next_bounds,next_spawns,next_mission
end
local function restore_document(d)
  local old=doc_snapshot()
  apply(d.parts,false)
  local ok,why=pcall(doc_restore,d)
  if not ok then
    local recovered,err=pcall(apply,old.parts,false)
    if not recovered then broken=true say('Model recovery failed: '..tostring(err),'error') end
    error(why,0)
  end
  return old
end
local function outside_bounds(p)
  local b=bounds.blast
  local cam=bounds.camera
  if (not b or not cam) and gd.stage_bounds then
    local live=gd.stage_bounds() or {}
    b=b or live.blast
    cam=cam or live.camera
  end
  if b and (p.x<b.left or p.x>b.right or p.y<b.bottom or p.y>b.top) then return 'the blast zone' end
  if cam and (p.x<cam.left or p.x>cam.right or p.y<cam.bottom or p.y>cam.top) then return 'the camera bounds' end
end
local function load_map(name)
  assert(offline(), 'active offline match required')
  name=file_name(name or filename)
  local text=assert(gd.data_read(name), 'layout file not found: '..name)
  -- Text-only chunk with no globals: files cannot access gd, io or the script environment.
  local chunk,why=load(text,'@'..name,'t',{}) assert(chunk,why)
  local target,lay_bounds,lay_mission=validate(chunk())
  local before=restore_document({parts=target,bounds=lay_bounds,spawns=lay_bounds.spawn or {},mission=lay_mission})
  doc_commit(before,'Loaded '..name) filename=name dirty=false
  say('Loaded '..#parts..' parts from '..name) toast('Loaded '..#parts..' parts')
end
local function set_overlay()
  if previous then gd.stage_view(previous.geometry,overlay) end
end
local function stop(restore_fly)
  if previous then
    if restore_fly~=false and offline() then gd.fly(1,previous.fly) end
    gd.stage_view(previous.geometry,previous.overlay)
  end
  previous=nil editing=false menu=false help_open=false dragging=false
  ghost_despawn()
end
local function start()
  assert(offline(), 'active offline match required')
  assert(gd.player(1), 'P1 is required')
  assert(not broken, 'save layout and restart match before editing')
  mission_stop('editing')
  if editing then return end
  local back=test_return
  local geometry,lines=gd.stage_view()
  previous={fly=gd.fly(1),geometry=geometry,overlay=lines}
  local flew,on=pcall(gd.fly,1,true)
  assert(flew and on, flew and 'flight refused' or 'P1 cannot fly while dead, held or respawning; try again in a moment')
  editing=true autoload=nil set_overlay() say('Editing '..filename)
  test_return=nil
  if back then pcall(gd.teleport,1,back.x,back.y) end -- back to the cursor a mission test left from
end
local function edit()
  assert(editing and offline(), 'enable the editor in an offline match first')
  assert(not broken, 'save and restart: engine state needs recovery')
end
-- Missions (see mission.lua): the document holds the markers, the runtime below carries out the
-- state machine's actions with gd.spawn_enemy / gd.enemy_alive / gd.enemy_remove / gd.teleport.
local function mission_remove_enemies()
  if not run then return end
  local hs={}
  for h in pairs(run.state.tracked) do hs[#hs+1]=h end
  table.sort(hs)
  for _,h in ipairs(hs) do pcall(gd.enemy_remove,h) end
  run.state.tracked={}
end
mission_stop=function(reason)
  if not run then return end
  mission_remove_enemies()
  if not run.state.result then gd.log('mission: aborted ('..tostring(reason)..')') end
  local moved,opened=run.respawn_moved,next(run.coll)~=nil
  run=nil
  if opened then pcall(sync,parts) end -- trigger-opened or -closed parts go back to the document
  -- The respawn slot was borrowed: put the document's spawns back (no-op when the stage is gone).
  if moved then pcall(doc_restore,doc_snapshot()) end
end
local function mission_act(actions,events)
  for _,ev in ipairs(events) do
    local seconds=(ev.frames or 0)/Mission.FPS
    if ev.type=='wave' then gd.log(('mission: wave %d of %d'):format(ev.n,ev.of))
    elseif ev.type=='defeat' then gd.log('mission: defeated '..ev.kind)
    elseif ev.type=='trigger' then gd.log(('mission: trigger %d %s'):format(ev.index,ev.action))
    elseif ev.type=='message' then
      run.message={text=ev.text,left=Mission.MESSAGE_FRAMES}
      gd.log('mission: message '..ev.text)
    elseif ev.type=='vanish' then gd.log('mission: lost '..ev.kind..' (left the stage, not a defeat)')
    elseif ev.type=='checkpoint' then gd.log('mission: checkpoint '..ev.index)
    elseif ev.type=='death' then
      gd.log('mission: death '..ev.deaths..(ev.remaining and (' lives left '..ev.remaining) or ''))
    elseif ev.type=='complete' then
      gd.log(('mission: complete time=%.1fs deaths=%d defeated=%d vanished=%d'):format(seconds,ev.deaths,ev.defeated,ev.vanished))
    elseif ev.type=='failed' then
      gd.log(('mission: failed reason=%s%s time=%.1fs deaths=%d defeated=%d vanished=%d'):format(ev.reason,
        ev.detail and (' ('..ev.detail..')') or '',seconds,ev.deaths,ev.defeated,ev.vanished))
    end
  end
  for _,a in ipairs(actions) do
    if a.type=='spawn' then
      local ok,h,why=pcall(gd.spawn_enemy,a.kind,a.x,a.y,{facing=a.facing})
      if ok and h then Mission.spawned(run.state,a.index,h)
      else Mission.spawn_failed(run.state,a.index,ok and why or h) end
    elseif a.type=='collision' then
      -- Parts near the point change collision for this run only: the document is untouched and the
      -- override is cleared by mission_stop. sync respawns an instance whose collision differs.
      local n=0
      for _,p in ipairs(parts) do
        if math.sqrt((p.x-a.x)^2+(p.y-a.y)^2)<=a.r then run.coll[p.id]=not a.open n=n+1 end
      end
      local effective=clone(parts)
      for _,p in ipairs(effective) do if run.coll[p.id]~=nil then p.collision=run.coll[p.id] end end
      local ok,why=pcall(sync,effective)
      gd.log(('mission: collision %s on %d part(s) near %.1f,%.1f%s'):format(a.open and 'open' or 'closed',n,a.x,a.y,
                                                                          ok and '' or (' FAILED: '..tostring(why))))
    elseif a.type=='respawn_point' then
      -- P1's engine respawn (rebirth platform included) reads stage spawn slot 4.
      local ok,res=pcall(gd.stage_set_spawn,Mission.RESPAWN_SLOT,a.x,a.y)
      if ok and res then run.respawn_moved=true
      else say('Respawn point refused: '..tostring(res),'error') end
    elseif a.type=='cleanup' then mission_remove_enemies() end
  end
end
-- An enemy that ends without the engine's defeat event is either lost off the stage or was
-- transformed by a kill (a Koopa becomes a shell item: ScriptGame_EnemyStatus reports 0 either way).
-- Position tells them apart: last seen within EDGE of the blast zone means it fell out.
local EDGE=30
local function near_blast(x,y)
  local b=bounds.blast
  if not b and gd.stage_bounds then local live=gd.stage_bounds() b=live and live.blast end
  return b~=nil and (x<=b.left+EDGE or x>=b.right-EDGE or y<=b.bottom+EDGE or y>=b.top-EDGE)
end
local function mission_step()
  if not run or run.state.result then return end
  local p=gd.player(1)
  -- enemy_status: 1 fighting, 2 stock-defeated (may still be animating), 0 gone without a defeat.
  local alive,defeated={}, {}
  for h in pairs(run.state.tracked) do
    local ok,status=true,nil
    if gd.enemy_status then
      local code
      ok,code=pcall(gd.enemy_status,h)
      status=ok and (code=='alive' and 1 or code=='defeated' and 2 or 0) or nil
    else
      local live
      ok,live=pcall(gd.enemy_alive,h)
      status=ok and (live and 1 or 2) or nil -- no status API: any end counts as a defeat
    end
    if status==1 then
      alive[h]=true
      local got,e=pcall(gd.enemy_state,h)
      if got and e then run.last[h]={x=e.x,y=e.y} end
    elseif status==2 then defeated[h]=true
    else
      local at=run.last[h]
      if not (at and near_blast(at.x,at.y)) then defeated[h]=true end
    end
  end
  run.last_falls=p and p.falls or run.last_falls
  local actions,events=Mission.step(run.state,{frames=run.frames,player=p and {x=p.x,y=p.y} or nil,
                                               falls=run.last_falls,alive=alive,defeated=defeated})
  mission_act(actions,events)
end
-- opts.from = {x,y}: a test run that starts P1 there (the cursor) instead of at the authored start.
-- The document is not touched; the run works on a copy.
local function mission_begin(opts)
  assert(offline(), 'active offline match required')
  assert(gd.player(1), 'P1 is required')
  local m=clone(mission_doc or Mission.validate({}))
  if opts and opts.from then m.start={x=opts.from.x,y=opts.from.y} end
  local why=Mission.check_playable(m)
  assert(not why, why)
  mission_stop('restart')
  test_return=nil
  if opts and opts.test then
    local p1=gd.player(1)
    test_return=opts.back or {x=p1.x,y=p1.y}
  end
  if editing then stop() end
  gd.teleport(1,m.start.x,m.start.y)
  -- Lives are counted from P1's fall count (the LAB has no stocks at all). Where stocks do exist, one
  -- spare keeps the engine's game-over from ending the match before the mission's own failure.
  if m.objective.lives then pcall(gd.set_stocks,1,math.min(99,m.objective.lives+1)) end
  local p=gd.player(1)
  run={state=Mission.new(clone(m)),frames=0,last_falls=p and p.falls or 0,last={},coll={},
       test=opts and opts.test or false,from=opts and opts.from,back=test_return}
  gd.log('mission: start '..m.objective.type..(run.test and (' (test from the cursor %.1f,%.1f)'):format(m.start.x,m.start.y) or ''))
  mission_step() -- wave 1 appears with the mission, not a frame later
end
local function mission_autostart()
  if not mission_doc then return end
  local ok,why=pcall(mission_begin)
  if not ok then say('Mission not started: '..tostring(why),'error') end
end
local function mission_entries(m)
  local out={}
  if m.start then out[#out+1]={('start %.1f %.1f'):format(m.start.x,m.start.y),function(t) t.start=nil end} end
  if m.goal then
    out[#out+1]={('goal %.1f %.1f %gx%g'):format(m.goal.x,m.goal.y,m.goal.w,m.goal.h),function(t) t.goal=nil end}
  end
  for i,c in ipairs(m.checkpoints) do
    out[#out+1]={('checkpoint %.1f %.1f %gx%g'):format(c.x,c.y,c.w,c.h),function(t) table.remove(t.checkpoints,i) end}
  end
  for i,e in ipairs(m.enemies) do
    out[#out+1]={('enemy %s %.1f %.1f wave %d'):format(e.kind,e.x,e.y,e.wave),function(t) table.remove(t.enemies,i) end}
  end
  for i,r in ipairs(m.waves) do
    out[#out+1]={('wave %d %s'):format(r.wave,r.time and ('after %gs'):format(r.time) or
                                       ('when P1 passes x=%g going %s'):format(r.x,r.dir==1 and 'right' or 'left')),
                 function(t) table.remove(t.waves,i) end}
  end
  for i,z in ipairs(m.triggers) do
    local what=z.action=='wave' and (' wave '..z.wave) or z.action=='message' and (' "'..z.text..'"') or
               z.action=='collision' and (' %s near %.1f %.1f r%g'):format(z.open and 'open' or 'close',z.at.x,z.at.y,z.r) or ''
    out[#out+1]={('trigger %s%s %.1f %.1f %gx%g%s'):format(z.action,what,z.x,z.y,z.w,z.h,z.once and '' or ' repeat'),
                 function(t) table.remove(t.triggers,i) end}
  end
  return out
end
-- One undo step per edit; Mission.validate refuses before anything changes. Only a moved start
-- touches the engine (slot 0, through doc_restore's reconcile with its recovery path).
local function mission_edit(label,fn)
  edit()
  local before=doc_snapshot()
  local m=clone(mission_doc or Mission.validate({}))
  fn(m)
  m=Mission.validate(m)
  if Mission.empty(m) then m=nil end
  local function at(t) return t and t.start and (t.start.x..','..t.start.y) or '' end
  if at(m)~=at(mission_doc) then
    doc_restore({bounds=before.bounds,spawns=before.spawns,mission=m})
  else
    mission_doc=m
  end
  dirty=true doc_commit(before,label)
end
local function mission_xy(rest,usage)
  local x,y=rest:match('^(%S+)%s+(%S+)$')
  if x then
    x,y=tonumber(x),tonumber(y)
    assert(x and y and number(x) and number(y),'start needs a finite x y')
    return x,y
  end
  assert(rest=='',usage)
  local cx,cy=fly_cursor()
  return cx,cy
end
local function mission_command(arg)
  local sub,rest=(arg or ''):match('^(%S*)%s*(.-)%s*$')
  if sub=='start' then
    local x,y=mission_xy(rest,'map mission start [x y]')
    mission_edit('mission start',function(m) m.start={x=x,y=y} end)
  elseif sub=='enemy' then
    local kind,wave=rest:match('^(%S+)%s*(%S*)$')
    assert(kind,'map mission enemy <kind> [wave]: '..table.concat(Mission.KINDS,' '))
    if wave=='' then wave=nil else wave=assert(tonumber(wave),'wave must be a number') end
    local x,y=fly_cursor()
    mission_edit('mission enemy '..kind,function(m) m.enemies[#m.enemies+1]={kind=kind,x=x,y=y,wave=wave} end)
  elseif sub=='goal' or sub=='checkpoint' then
    local w,h=rest:match('^(%S+)%s+(%S+)$')
    w,h=tonumber(w),tonumber(h)
    assert(w and h,'map mission '..sub..' <w> <h> (world units, centred on the cursor)')
    local x,y=fly_cursor()
    mission_edit('mission '..sub,function(m)
      local z={x=x,y=y,w=w,h=h}
      if sub=='goal' then m.goal=z else m.checkpoints[#m.checkpoints+1]=z end
    end)
  elseif sub=='wave' then
    local n,how,arg=rest:match('^(%d+)%s+(%S+)%s*(.-)$')
    assert(n,'map mission wave <n> time <seconds> | x [<x>] [left|right] | clear')
    n=tonumber(n)
    mission_edit('mission wave '..n..' '..how,function(m)
      for i=#m.waves,1,-1 do if m.waves[i].wave==n then table.remove(m.waves,i) end end
      if how=='time' then
        m.waves[#m.waves+1]={wave=n,time=assert(tonumber(arg),'wave time needs a number of seconds')}
      elseif how=='x' then
        local xs,dir=arg:match('^(%S*)%s*(%S*)$')
        local x=tonumber(xs)
        if not x then dir=xs x=(fly_cursor()) end
        assert(dir=='' or dir=='left' or dir=='right','direction is left or right')
        m.waves[#m.waves+1]={wave=n,x=x,dir=dir=='left' and -1 or 1}
      else assert(how=='clear','wave rule is time, x or clear') end
    end)
  elseif sub=='trigger' then
    local action,w,h,args=rest:match('^(%S+)%s+(%S+)%s+(%S+)%s*(.-)$')
    w,h=tonumber(w),tonumber(h)
    assert(action and w and h,'map mission trigger wave <w> <h> <n> | message <w> <h> <text> | collision <w> <h> open|close [r] | complete <w> <h> | fail <w> <h>  (add "repeat" to fire on every entry)')
    local once=true
    local stripped=args:gsub('%s*repeat$','')
    if stripped~=args then once=false args=stripped end
    local x,y=fly_cursor()
    local z={x=x,y=y,w=w,h=h,action=action,once=once}
    if action=='wave' then z.wave=assert(tonumber(args),'trigger wave needs a wave number')
    elseif action=='message' then z.text=args
    elseif action=='collision' then
      local mode,r=args:match('^(%S+)%s*(%S*)$')
      assert(mode=='open' or mode=='close','trigger collision needs open or close')
      z.open=mode=='open' z.r=tonumber(r)
      local part=assert(find(parts,selected),'select a part first: the trigger acts on the parts near it')
      z.at={x=part.x,y=part.y}
    else assert(args=='','this trigger action takes no arguments') end
    mission_edit('mission trigger '..action,function(m) m.triggers[#m.triggers+1]=z end)
  elseif sub=='objective' then
    local kind,opts=rest:match('^(%S+)%s*(.-)$')
    assert(kind,'map mission objective <reach_goal|defeat_all|defeat_then_goal> [time=<s>] [lives=<n>]')
    local o={type=kind}
    for k,v in opts:gmatch('(%w+)=(%S+)') do
      assert(k=='time' or k=='lives','unknown option: '..k..' (time=, lives=)')
      o[k]=assert(tonumber(v),k..' must be a number')
    end
    mission_edit('mission objective',function(m) m.objective=o end)
  elseif sub=='list' then
    if Mission.empty(mission_doc) then say('no mission (map mission start|enemy|goal|checkpoint|objective)','action') return end
    local o=mission_doc.objective
    gd.log('map_editor: mission objective '..(o and (o.type..(o.time and (' time='..o.time) or '')..
           (o.lives and (' lives='..o.lives) or '')) or '(none)'))
    local entries=mission_entries(mission_doc)
    for i,e in ipairs(entries) do gd.log(('map_editor: mission %d %s'):format(i,e[1])) end
    say(('mission: %d items, objective %s'):format(#entries,o and o.type or 'none'),'action')
  elseif sub=='delete' then
    local n=tonumber(rest)
    assert(n and n%1==0,'map mission delete <index> (numbers from map mission list)')
    local entries=mission_entries(mission_doc or Mission.validate({}))
    assert(entries[n],'index '..n..' is out of range (see map mission list)')
    mission_edit('mission delete '..n,function(m) mission_entries(m)[n][2](m) end)
  elseif sub=='clear' then
    assert(not Mission.empty(mission_doc),'no mission to clear')
    mission_edit('mission clear',function(m)
      m.start,m.goal,m.objective=nil,nil,nil m.enemies,m.checkpoints,m.waves,m.triggers={},{},{},{}
    end)
  elseif sub=='pick' then mission_pick(rest)
  elseif sub=='test' then
    assert(editing,'map on first: a test starts P1 at the editing cursor')
    local p1=assert(gd.player(1),'P1 is required')
    mission_begin({from={x=p1.x,y=p1.y},test=true})
  elseif sub=='play' or sub=='restart' then
    if sub=='play' and rest~='' then load_map(rest) end
    if sub=='restart' and run and run.test then mission_begin({from=run.from,test=true,back=run.back}) else mission_begin() end
  elseif sub=='stop' then
    assert(run,'no mission is running')
    local was_test=run.test
    mission_stop('stopped')
    say('Mission stopped','action')
    -- A test hands the editor back, at the cursor it left from. The fighter can be in a state flight
    -- refuses for a few frames (the goal pose, a respawn), so on_frame keeps trying.
    if was_test then want_edit={left=180} pcall(start) end
  else
    error('map mission start [x y]|enemy <kind> [wave]|goal <w> <h>|checkpoint <w> <h>|objective <type> [time=s] [lives=n]|wave <n> time|x ...|trigger <action> <w> <h> ...|list|delete <index>|clear|test|play [file]|restart|stop')
  end
end
do
-- Mouse and inspector authoring of the mission markers (tool 'mission'). A click on empty space places
-- the picked kind, a click on a marker selects it, dragging moves it, dragging a selected zone's edge
-- resizes it. Every gesture is one undo step through mission_edit.
local MKINDS={'start','enemy','checkpoint','goal','trigger'}
local TRIGGER_ACTIONS={'message','wave','collision','complete','fail'}
local ZONE_W,ZONE_H,EDGE_PX=26,60,6
local function cycle(list,cur)
  for i,v in ipairs(list) do if v==cur then return list[i%#list+1] end end
  return list[1]
end
local function marker_at(m,sel)
  if not m or not sel then return nil end
  if sel.kind=='start' then return m.start
  elseif sel.kind=='goal' then return m.goal
  elseif sel.kind=='checkpoint' then return m.checkpoints[sel.index]
  elseif sel.kind=='enemy' then return m.enemies[sel.index]
  elseif sel.kind=='trigger' then return m.triggers[sel.index] end
end
local function is_zone(kind) return kind=='goal' or kind=='checkpoint' or kind=='trigger' end
local function marker_rect(t)
  local x1,y1=gd.project(t.x-t.w/2,t.y+t.h/2,0)
  local x2,y2=gd.project(t.x+t.w/2,t.y-t.h/2,0)
  if not (x1 and y1 and x2 and y2) then return nil end
  return math.min(x1,x2),math.min(y1,y2),math.abs(x2-x1),math.abs(y2-y1)
end
-- Under the pointer: the selected zone's edge (resize), else the top marker (move). Returns
-- sel, mode, ex, ey with ex/ey = -1 / 1 for the left or right and the top or bottom world edge.
local function marker_hit(mx,my)
  local m=mission_doc
  if not m then return nil end
  local cur=marker_at(m,MK.sel)
  if cur and is_zone(MK.sel.kind) then
    -- Compare against the projected world edges, so the test does not depend on which way the camera is
    -- flipped. ex = -1 / 1: left / right edge; ey = 1 / -1: top / bottom edge in world space.
    local sl,sy1=gd.project(cur.x-cur.w/2,cur.y+cur.h/2,0)
    local sr,sy2=gd.project(cur.x+cur.w/2,cur.y-cur.h/2,0)
    if sl and sr and sy1 and sy2 and mx>=math.min(sl,sr)-EDGE_PX and mx<=math.max(sl,sr)+EDGE_PX and
       my>=math.min(sy1,sy2)-EDGE_PX and my<=math.max(sy1,sy2)+EDGE_PX then
      local ex=math.abs(mx-sl)<=EDGE_PX and -1 or math.abs(mx-sr)<=EDGE_PX and 1 or 0
      local ey=math.abs(my-sy1)<=EDGE_PX and 1 or math.abs(my-sy2)<=EDGE_PX and -1 or 0
      if ex~=0 or ey~=0 then return MK.sel,'resize',ex,ey end
    end
  end
  local function near(t)
    local sx,sy=gd.project(t.x,t.y,0)
    return sx and math.abs(mx-sx)<=9 and math.abs(my-sy)<=9
  end
  for i=#m.enemies,1,-1 do if near(m.enemies[i]) then return {kind='enemy',index=i},'move' end end
  if m.start and near(m.start) then return {kind='start'},'move' end
  local function inzone(t)
    local l,tp,w,h=marker_rect(t)
    return l and mx>=l and mx<=l+w and my>=tp and my<=tp+h
  end
  for i=#m.triggers,1,-1 do if inzone(m.triggers[i]) then return {kind='trigger',index=i},'move' end end
  for i=#m.checkpoints,1,-1 do if inzone(m.checkpoints[i]) then return {kind='checkpoint',index=i},'move' end end
  if m.goal and inzone(m.goal) then return {kind='goal'},'move' end
  return nil
end
local function place_marker(x,y)
  local kind,idx=MK.kind,nil
  mission_edit('mission '..kind,function(m)
    if kind=='start' then m.start={x=x,y=y}
    elseif kind=='enemy' then
      m.enemies[#m.enemies+1]={kind=MK.pick.enemy,x=x,y=y,wave=MK.pick.wave} idx=#m.enemies
    elseif kind=='checkpoint' then
      m.checkpoints[#m.checkpoints+1]={x=x,y=y,w=ZONE_W,h=ZONE_H} idx=#m.checkpoints
    elseif kind=='goal' then m.goal={x=x,y=y,w=ZONE_W,h=ZONE_H}
    else
      local z={x=x,y=y,w=ZONE_W,h=ZONE_H,action=MK.pick.action,once=true}
      if MK.pick.action=='wave' then z.wave=MK.pick.wave
      elseif MK.pick.action=='message' then z.text='Message'
      elseif MK.pick.action=='collision' then
        local part=assert(find(parts,selected),'select a part first: the trigger acts on the parts near it')
        z.at={x=part.x,y=part.y} z.open=true
      end
      m.triggers[#m.triggers+1]=z idx=#m.triggers
    end
  end)
  MK.sel={kind=kind,index=idx}
end
local function marker_delete()
  local sel=assert(MK.sel,'select a marker first')
  mission_edit('mission delete '..sel.kind,function(m)
    assert(marker_at(m,sel),'that marker is gone')
    if sel.kind=='start' then m.start=nil elseif sel.kind=='goal' then m.goal=nil
    elseif sel.kind=='checkpoint' then table.remove(m.checkpoints,sel.index)
    elseif sel.kind=='enemy' then table.remove(m.enemies,sel.index)
    else table.remove(m.triggers,sel.index) end
  end)
  MK.sel=nil
end
local function marker_field_set(field,value)
  local n=tonumber(value)
  assert(n and number(n),'type a number')
  local sel=assert(MK.sel,'select a marker first')
  mission_edit('mission '..field,function(m)
    local t=assert(marker_at(m,sel),'that marker is gone')
    t[field]=n
  end)
end
local function mission_press()
  local wx,wy=mouse_world()
  if not wx then return end
  local sel,mode,ex,ey=marker_hit(mouse.x,mouse.y)
  if sel then
    MK.sel=sel
    local t=marker_at(mission_doc,sel)
    MK.drag={sel=sel,mode=mode,ex=ex,ey=ey,ox=t.x-wx,oy=t.y-wy,px=mouse.x,py=mouse.y,base=doc_snapshot(),used=false,
           l=t.w and t.x-t.w/2,r=t.w and t.x+t.w/2,top=t.h and t.y+t.h/2,bottom=t.h and t.y-t.h/2}
    return
  end
  place_marker(sn(wx),sn(wy))
end
-- Runs each frame while a marker is being dragged; the document is edited live and replaced by one
-- undoable edit when the button is released.
local function mission_drag_step()
  local d=MK.drag
  local t=marker_at(mission_doc,d.sel)
  if (mouse.buttons & 1)==1 then
    local wx,wy=mouse_world()
    if wx and t then
      if math.abs(mouse.x-d.px)+math.abs(mouse.y-d.py)>3 then d.used=true end
      if d.used then
        if d.mode=='move' then t.x,t.y=sn(wx+d.ox),sn(wy+d.oy)
        else
          local step=U*grid
          local l,r,top,bottom=d.l,d.r,d.top,d.bottom
          if d.ex<0 then l=math.min(sn(wx),r-step) elseif d.ex>0 then r=math.max(sn(wx),l+step) end
          if d.ey>0 then top=math.max(sn(wy),bottom+step) elseif d.ey<0 then bottom=math.min(sn(wy),top-step) end
          t.x,t.w=(l+r)/2,r-l
          t.y,t.h=(top+bottom)/2,top-bottom
        end
      end
    end
    return
  end
  MK.drag=nil
  if not d.used or not t then mission_doc=d.base.mission return end
  local final=clone(t)
  mission_doc=d.base.mission -- back to the pre-drag document; the edit below records one undo step
  local ok,why=pcall(mission_edit,'mission '..d.mode..' '..d.sel.kind,function(m)
    local u=assert(marker_at(m,d.sel),'that marker is gone')
    u.x,u.y,u.w,u.h=final.x,final.y,final.w,final.h
  end)
  if not ok then say('Error: '..tostring(why),'error') end
end

mission_pick=function(rest)
  local kind,what=rest:match('^(%S+)%s*(%S*)$')
  local ok=false
  for _,k in ipairs(MKINDS) do if k==kind then ok=true end end
  assert(ok,'map mission pick <'..table.concat(MKINDS,'|')..'> [enemy kind | trigger action]')
  if what~='' then
    local list=kind=='enemy' and Mission.KINDS or kind=='trigger' and TRIGGER_ACTIONS or nil
    assert(list,'only enemy and trigger take a second word')
    local known=false
    for _,k in ipairs(list) do if k==what then known=true end end
    assert(known,(kind=='enemy' and 'unknown enemy kind: ' or 'unknown trigger action: ')..what)
    if kind=='enemy' then MK.pick.enemy=what else MK.pick.action=what end
  end
  MK.kind=kind tool='mission'
  say('Click places: '..MK.kind,'action')
end
local function mission_rows()
  local rows={}
  local x,y=canvas_w()-240,70
  local function row(label,value,act)
    rows[#rows+1]={kind='m',label=label,value=value,act=act,x=x,y=y,w=224,h=18} y=y+20
  end
  row('click places',MK.kind,function() MK.kind=cycle(MKINDS,MK.kind) end)
  if MK.kind=='enemy' then
    row('enemy kind',MK.pick.enemy,function() MK.pick.enemy=cycle(Mission.KINDS,MK.pick.enemy) end)
    row('wave',tostring(MK.pick.wave),function() MK.pick.wave=MK.pick.wave%Mission.LIMITS.waves+1 end)
  elseif MK.kind=='trigger' then
    row('trigger action',MK.pick.action,function() MK.pick.action=cycle(TRIGGER_ACTIONS,MK.pick.action) end)
    if MK.pick.action=='wave' then row('wave',tostring(MK.pick.wave),function() MK.pick.wave=MK.pick.wave%Mission.LIMITS.waves+1 end) end
  end
  y=y+6
  local t=MK.sel and marker_at(mission_doc,MK.sel)
  if not t then
    rows[#rows+1]={kind='info',label='no marker selected',x=x,y=y,w=224,h=18}
    return rows
  end
  row(MK.sel.kind..(MK.sel.index and (' '..MK.sel.index) or ''),'',nil)
  for _,f in ipairs(is_zone(MK.sel.kind) and {'x','y','w','h'} or {'x','y'}) do
    local value=(typing and typing.field=='m:'..f) and (typing.text..'_') or ('%.2f'):format(t[f])
    row(f,value,function() typing={field='m:'..f,text=''} say('Type a value; Enter applies, ESC cancels') end)
  end
  local function change(fn)
    local sel=MK.sel
    mission_edit('mission '..sel.kind,function(m) fn(assert(marker_at(m,sel),'that marker is gone')) end)
  end
  if MK.sel.kind=='enemy' then
    row('kind',t.kind,function() change(function(u) u.kind=cycle(Mission.KINDS,u.kind) end) end)
    row('wave',tostring(t.wave),function() change(function(u) u.wave=u.wave%Mission.LIMITS.waves+1 end) end)
  elseif MK.sel.kind=='trigger' then
    row('action',t.action,nil)
    row('fires',t.once and 'once' or 'every entry',function() change(function(u) u.once=not u.once end) end)
  end
  row('delete','Del',function() marker_delete() end)
  return rows
end
-- Controller path (the Z menu): the same edits at the flight cursor instead of the pointer.
local function select_nearest_marker()
  local m=assert(mission_doc,'no mission yet')
  local cx,cy=fly_cursor()
  local best,bd=nil,math.huge
  local function consider(kind,index,t)
    local d=(t.x-cx)^2+(t.y-cy)^2
    if d<bd then best,bd={kind=kind,index=index},d end
  end
  if m.start then consider('start',nil,m.start) end
  if m.goal then consider('goal',nil,m.goal) end
  for i,c in ipairs(m.checkpoints) do consider('checkpoint',i,c) end
  for i,e in ipairs(m.enemies) do consider('enemy',i,e) end
  for i,t in ipairs(m.triggers) do consider('trigger',i,t) end
  assert(best,'no markers to select')
  MK.sel=best
  say(('Selected %s%s'):format(best.kind,best.index and (' '..best.index) or ''),'action')
end
local function move_marker_to_cursor()
  local sel=assert(MK.sel,'select a marker first (Mission: select nearest marker)')
  local cx,cy=fly_cursor()
  mission_edit('mission move '..sel.kind,function(m)
    local t=assert(marker_at(m,sel),'that marker is gone')
    t.x,t.y=cx,cy
  end)
end
MK.place_at_cursor=function() local x,y=fly_cursor() place_marker(x,y) end
MK.select_nearest=select_nearest_marker
MK.move_to_cursor=move_marker_to_cursor
MK.next_kind=function() MK.kind=cycle(MKINDS,MK.kind) tool='mission' say('Click places: '..MK.kind,'action') end
MK.rows=mission_rows MK.press=mission_press MK.drag_step=mission_drag_step
MK.delete=marker_delete MK.field_set=marker_field_set MK.cycle=cycle MK.KINDS=MKINDS
end
local function place(duplicate, wx, wy)
  edit()
  local p=duplicate and assert(find(parts,selected), 'select a part first') or
    {part=palette_part(),rot=rotation,collision=collision,floor_flags=floor_flags,
     scale=AUTHOR_SCALE,scale_x=1,scale_y=1,scale_z=1}
  p=clone(p)
  if wx then p.x,p.y,p.z=sn(wx),sn(wy),depth else p.x,p.y,p.z=fly_cursor() end
  next_id=next_id+1 p.id=next_id
  local target=clone(parts) target[#target+1]=p
  acted(target,(duplicate and 'Duplicated ' or 'Placed ')..p.part)
  selected=p.id
  local out=outside_bounds(p)
  if out then say(('Placed outside %s'):format(out)) toast('Outside '..out) end
  remember(p.part)
  if duplicate then toast('Duplicated '..p.part:gsub('^bf_','')) end
end
local function duplicate_many(n)
  edit()
  local src=assert(find(parts,selected),'select a part first')
  local target=clone(parts)
  local step=U*grid
  local made=0
  for i=1,n do
    if #target>=MAX_PARTS then break end
    local p=clone(src)
    next_id=next_id+1 p.id=next_id
    p.x=p.x+step*i
    target[#target+1]=p
    made=made+1
  end
  assert(made>0,'model limit reached')
  acted(target,('Duplicated x%d'):format(made))
  selected=target[#target].id
end
local function select_add()
  edit()
  local x,y,z=fly_cursor()
  local best,d=nil,math.huge
  for _,p in ipairs(parts) do
    if p.id~=selected and not group[p.id] then
      local distance=(p.x-x)^2+(p.y-y)^2+(p.z-z)^2
      if distance<d then best,d=p.id,distance end
    end
  end
  if best then group[best]=true say('Added '..find(parts,best).part,'action')
  else say('All parts selected') end
end
local function select_remove()
  edit()
  assert(#selected_ids()>1,'select more parts first')
  local x,y,z=fly_cursor()
  local worst,wd=nil,math.huge
  for _,p in ipairs(parts) do
    if group[p.id] then
      local distance=(p.x-x)^2+(p.y-y)^2+(p.z-z)^2
      if distance<wd then worst,wd=p.id,distance end
    end
  end
  if worst then
    group[worst]=nil
    say('Removed '..find(parts,worst).part,'action')
  else
    clear_group() say('Selection cleared','action')
  end
end
local function select_near(wx, wy)
  edit()
  local x,y,z
  if wx then x,y,z=wx,wy,depth else x,y,z=fly_cursor() end
  local best,d=nil,math.huge
  for _,p in ipairs(parts) do
    local distance=(p.x-x)^2+(p.y-y)^2+(p.z-z)^2
    if distance<d then best,d=p.id,distance end
  end
  selected=best say(best and ('Selected '..find(parts,best).part) or 'No parts to select','action')
end
local function transform(mode,delta,record,wx,wy)
  edit() local target=clone(parts) local p=assert(find(target,selected), 'select a part first')
  local ax,ay,az=p.x,p.y,p.z
  if mode=='move' then
    local x,y,z
    if wx then x,y,z=sn(wx),sn(wy),depth else x,y,z=fly_cursor() end
    if axis_lock=='x' then p.x=x elseif axis_lock=='y' then p.y=y else p.x,p.y,p.z=x,y,z end
  elseif mode=='rotate' then p.rot=((p.rot+delta+180)%360)-180
  elseif mode=='rotateto' then p.rot=((delta+180)%360)-180
  elseif mode=='scale' then
    p.scale=math.max(SCALE_MIN,math.min(SCALE_MAX,scale_of(p,'scale')*delta))
  elseif mode=='mirror' then p['scale_'..delta]=-scale_of(p,'scale_'..delta)
  elseif mode=='unscale' then p.scale,p.scale_x,p.scale_y,p.scale_z=AUTHOR_SCALE,1,1,1
  end
  if next(group) then
    local mx,my,mz=p.x-ax,p.y-ay,p.z-az
    for _,q in ipairs(target) do
      if q.id~=selected and group[q.id] then
        if mode=='move' then
          if not axis_lock or axis_lock=='x' then q.x=q.x+mx end
          if not axis_lock or axis_lock=='y' then q.y=q.y+my end
          if not axis_lock then q.z=q.z+mz end
        elseif mode=='rotate' then q.rot=((q.rot+delta+180)%360)-180
        elseif mode=='rotateto' then q.rot=((delta+180)%360)-180
        elseif mode=='scale' then q.scale=math.max(SCALE_MIN,math.min(SCALE_MAX,scale_of(q,'scale')*delta))
        elseif mode=='mirror' then q['scale_'..delta]=-scale_of(q,'scale_'..delta)
        elseif mode=='unscale' then q.scale,q.scale_x,q.scale_y,q.scale_z=AUTHOR_SCALE,1,1,1
        end
      end
    end
  end
  local label=mode..' '..p.part
  if next(group) then label=label..(' +%d'):format(#selected_ids()-1) end
  if record~=false then acted(target,label) else apply(target,false) end
end
local function remove()
  edit()
  local target=clone(parts)
  local ids=selected_ids()
  assert(#ids>0,'select a part first')
  for i=#target,1,-1 do
    if target[i].id==selected or group[target[i].id] then table.remove(target,i) end
  end
  clear_group()
  acted(target,#ids==1 and 'Deleted part' or ('Deleted '..#ids..' parts'))
end
local function history(back)
  edit() local from,to=back and undo or redo,back and redo or undo
  local target=from[#from] assert(target,back and 'Nothing to undo' or 'Nothing to redo')
  local old=restore_document(target)
  table.remove(from) to[#to+1]=old
  if back then
    local n=table.remove(undo_names)
    if n then redo_names[#redo_names+1]=n end
  else
    undo_names[#undo_names+1]='Redo'
    table.remove(redo_names)
  end
  action_log[#action_log+1]=back and 'Undo' or 'Redo'
  if #action_log>8 then table.remove(action_log,1) end
  say(back and 'Undo' or 'Redo','action')
end
local function history_jump(n)
  for _=1,n do if #undo==0 then break end history(true) end
end
local function redo_jump(n)
  for _=1,n do if #redo==0 then break end history(false) end
end
local function cycle_tool()
  local i=1
  for k,name in ipairs(TOOLS) do if name==tool then i=k end end
  tool=TOOLS[i%#TOOLS+1] axis_lock=nil say('Tool: '..tool)
end
local function cycle_axis()
  axis_lock = axis_lock==nil and 'x' or axis_lock=='x' and 'y' or nil
  say('Move constraint: '..(axis_lock or 'free'))
end
local function rotate_to_mouse()
  edit()
  local p=assert(find(parts,selected),'select a part first')
  local wx,wy=mouse_world() assert(wx,'no world point under the pointer')
  local deg=math.deg(math.atan(wy-p.y,wx-p.x))
  if snap_on then deg=math.floor(deg/15+0.5)*15 end
  transform('rotateto',deg)
end

local function frame_selection()
  local p=assert(find(parts,selected),'select a part first')
  depth=p.z
  if gd.player(1) then gd.teleport(1,sn(p.x),sn(p.y)) end
  say('Framed '..p.part,'action')
end

-- Hybrid transforms (bible §4.3-§4.5): holding G/E/C runs a modal transform that follows the
-- pointer, commits on release and cancels on ESC; a quick tap switches tool instead. Arrows lock
-- the axis during a modal. The base snapshot makes the commit idempotent.
local function modal_point(base)
  local target=clone(base.parts)
  local p=find(target,selected) if not p then return nil end
  local wx,wy=mouse_world()
  if modal.mode=='move' then
    if modal.ax=='x' then p.x=sn(wx or p.x)
    elseif modal.ax=='y' then p.y=sn(wy or p.y)
    else p.x,p.y,p.z=sn(wx or p.x),sn(wy or p.y),depth end
  elseif modal.mode=='rotate' then
    if wx then
      local deg=math.deg(math.atan(wy-p.y,wx-p.x))
      if snap_on then deg=math.floor(deg/15+0.5)*15 end
      p.rot=((deg+180)%360)-180
    end
  elseif wx then
    local d=math.max(0.5,math.sqrt((wx-p.x)^2+(wy-p.y)^2))
    p.scale=math.max(SCALE_MIN,math.min(SCALE_MAX,scale_of(p,'scale')*(d/modal.d0)))
  end
  return target
end
local function modal_begin(mode, ax, src)
  local p=find(parts,selected)
  if not p then return false end
  local wx,wy=mouse_world()
  modal={mode=mode, base=doc_snapshot(), mx=mouse.x, my=mouse.y, used=false, ax=ax, src=src or 'key',
         d0=wx and math.max(0.5,math.sqrt((wx-p.x)^2+(wy-p.y)^2)) or 1}
  return true
end
local function modal_commit()
  if not modal then return end
  local m=modal
  if m.used then
    local target=modal_point(m.base)
    local tp,bp=find(target or {},selected),find(m.base.parts,selected)
    if target and tp and bp and not same(tp,bp) then
      undo[#undo+1]=m.base if #undo>HISTORY then table.remove(undo,1) end
      redo={}
      apply(target,false)
      name_undo(m.mode..' '..tp.part)
      say(m.mode..' '..tp.part,'action')
    end
  end
  modal=nil
end
local function modal_key(name, mode)
  if gd.key_pressed(name) and not (modal and modal.mode==mode) then
    if not modal_begin(mode, axis_lock, 'key') then
      tool=mode axis_lock=nil say('Tool: '..mode,'action') return true
    end
  end
  if not (modal and modal.mode==mode and modal.src=='key') then return false end
  if gd.key_pressed('ESCAPE') then
    apply(modal.base.parts,false) doc_restore(modal.base)
    modal=nil say('Cancelled') return true
  end
  if gd.key_pressed('LEFT') or gd.key_pressed('RIGHT') then
    modal.ax=(modal.ax=='x') and nil or 'x' say('Axis: '..(modal.ax or 'free'))
  elseif gd.key_pressed('UP') or gd.key_pressed('DOWN') then
    modal.ax=(modal.ax=='y') and nil or 'y' say('Axis: '..(modal.ax or 'free'))
  end
  if not gd.key(name) then
    if not modal.used then tool=mode axis_lock=nil say('Tool: '..mode,'action') end
    modal_commit()
  elseif math.abs(mouse.x-modal.mx)+math.abs(mouse.y-modal.my)>3 then
    modal.used=true
    local target=modal_point(modal.base)
    if target then apply(target,false) end
  end
  return true
end
local function field_value(p, field)
  return field=='scale' and scale_of(p,'scale') or (p[field] or 0)
end
local function field_point(base, field, dx)
  local target=clone(base.parts)
  local p=find(target,selected) if not p then return nil end
  if field=='rot' then
    local v=field_value(p,'rot')+dx*1.5
    if snap_on then v=math.floor(v/15+0.5)*15 end
    p.rot=((v+180)%360)-180
  elseif field=='scale' then
    p.scale=math.max(SCALE_MIN,math.min(SCALE_MAX,scale_of(p,'scale')*(1+dx*0.01)))
  else
    local v=field_value(p,field)+dx*U*grid
    if snap_on then v=snap(v) end
    p[field]=v
  end
  return target
end
local function field_text(field)
  local p=find(parts,selected) if not p then return '' end
  return string.format(field=='rot' and '%.1f' or '%.2f', field_value(p,field))
end
local function field_set(field, value)
  edit()
  local v=tonumber(value) assert(v,'a number is required')
  assert(number(v),'out of range')
  local target=clone(parts)
  local p=find(target,selected) assert(p,'select a part first')
  if field=='rot' then p.rot=((v+180)%360)-180
  elseif field=='scale' then p.scale=math.max(SCALE_MIN,math.min(SCALE_MAX,v))
  elseif field=='x' or field=='y' or field=='z' then p[field]=v
  else error('field: x|y|z|rot|scale') end
  acted(target,field..' '..string.format('%.2f',field_value(p,field)))
end
local function bounds_handle_positions()
  local cam=bounds.camera
  if not cam then return nil end
  local cw=canvas_w()
  local x1,y1=gd.project(cam.left,cam.top,0)
  local x2,y2=gd.project(cam.right,cam.bottom,0)
  if not (x1 and y1 and x2 and y2) then return nil end
  local function clampv(v,lo,hi) return math.max(lo,math.min(hi,v)) end
  x1,y1,x2,y2=clampv(x1,0,cw),clampv(y1,0,480),clampv(x2,0,cw),clampv(y2,0,480)
  local mx,my=(x1+x2)/2,(y1+y2)/2
  local h={}
  local function inside(v,lo,hi) return v>=lo and v<=hi end
  if inside(x1,4,cw-4) then h.left={x=x1,y=my} end
  if inside(x2,4,cw-4) then h.right={x=x2,y=my} end
  if inside(y1,4,476) then h.top={x=mx,y=y1} end
  if inside(y2,4,476) then h.bottom={x=mx,y=y2} end
  return h
end
local function hit_bounds(mx,my)
  local h=bounds_handle_positions()
  if not h then return nil end
  for _,edge in ipairs({'left','right','top','bottom'}) do
    local p=h[edge]
    if p and math.abs(mx-p.x)<=6 and math.abs(my-p.y)<=6 then return edge end
  end
  return nil
end
local function seg_dist(x1,y1,x2,y2,mx,my)
  local dx,dy=x2-x1,y2-y1
  local l2=dx*dx+dy*dy
  local t=l2>0 and ((mx-x1)*dx+(my-y1)*dy)/l2 or 0
  t=math.max(0,math.min(1,t))
  local cx,cy=x1+t*dx,y1+t*dy
  return math.sqrt((mx-cx)^2+(my-cy)^2)
end
local function gizmo_handle_positions()
  local p=find(parts,selected)
  if not p then return nil end
  local px,py,visible=gd.project(p.x,p.y,p.z)
  if not (px and visible) then return nil end
  local ax,ay=gd.project(p.x+2,p.y,p.z)
  local bx,by=gd.project(p.x,p.y+2,p.z)
  local function unit(sx,sy)
    local dx,dy=sx-px,sy-py
    local d=math.sqrt(dx*dx+dy*dy)
    if d<0.001 then return 0,-1 end
    return dx/d,dy/d
  end
  local ux,uy=unit(ax or px+2,ay or py)
  local vx,vy=unit(bx or px,by or py-2)
  local L=34
  return {px=px, py=py, hx2=px+ux*L, hy2=py+uy*L, gx2=px+vx*L, gy2=py+vy*L,
          sx2=px-vx*L, sy2=py-vy*L, r=20}
end
local function hit_handle(mx,my)
  local h=gizmo_handle_positions()
  if not h then return nil end
  if seg_dist(h.px,h.py,h.hx2,h.hy2,mx,my)<=7 then return 'move','x' end
  if seg_dist(h.px,h.py,h.gx2,h.gy2,mx,my)<=7 then return 'move','y' end
  if math.abs(math.sqrt((mx-h.px)^2+(my-h.py)^2)-h.r)<=6 then return 'rotate' end
  if math.sqrt((mx-h.sx2)^2+(my-h.sy2)^2)<=8 then return 'scale' end
  return nil
end

-- The first ten entries keep their order: the contract tests and muscle memory rely on it.
local ACTIONS = {
  {'Place',function() place(false) end},   {'Select nearest',select_near},
  {'Frame selection',frame_selection},
  {'Move selected to cursor',function() transform('move') end},
  {'Rotate selected +15',function() transform('rotate',15) end},
  {'Rotate selected -15',function() transform('rotate',-15) end},
  {'Duplicate at cursor',function() place(true) end}, {'Delete selected',remove},
  {'Undo',function() history(true) end}, {'Redo',function() history(false) end},
  {'Save layout',function() save() end}, {'Load layout',function() load_map() end},
  {'Collision overlay',function() overlay=not overlay set_overlay() end},
  {'Grid 2 / 1 / 0.5 base metres',function() grid=grid==2 and 1 or grid==1 and 0.5 or 2 end},
  {'New parts: collision on/off',function() collision=not collision end},
  {'New floors: flags 0..3',function() floor_flags=(floor_flags+1)%4 end},
  {'New parts: rotation +15',function() rotation=((rotation+15+180)%360)-180 end},
  {'Clear map (undoable)',function() edit() acted({},'Cleared map') toast('Cleared map') end},
  {'Exit editor / play',function() stop() say('Editor closed; map remains live') end},
  {'Next tool',cycle_tool},
  {'Tool: place',function() tool='place' axis_lock=nil say('Tool: place') end},
  {'Tool: select',function() tool='select' axis_lock=nil say('Tool: select') end},
  {'Tool: move',function() tool='move' axis_lock=nil say('Tool: move') end},
  {'Tool: rotate',function() tool='rotate' axis_lock=nil say('Tool: rotate') end},
  {'Tool: scale',function() tool='scale' axis_lock=nil say('Tool: scale') end},
  {'Scale selected +10%',function() transform('scale',SCALE_STEP) end},
  {'Scale selected -10%',function() transform('scale',1/SCALE_STEP) end},
  {'Reset scale to 2x',function() transform('unscale') end},
  {'Mirror selected X',function() transform('mirror','x') end},
  {'Mirror selected Y',function() transform('mirror','y') end},
  {'Reset rotation',function() transform('rotateto',0) end},
  {'Move constraint: free / X / Y',cycle_axis},
  {'Snap on/off',function() snap_on=not snap_on say('Snap '..(snap_on and 'on' or 'off')) end},
  {'Help / keybinds',function() help_open=not help_open end},
  {'Duplicate x4 at cursor',function() duplicate_many(4) end},
  {'Mission: start at cursor',function() mission_command('start') end},
  {'Mission: play',function() mission_command('play') end},
  {'Mission: restart',function() mission_command('restart') end},
  {'Mission: test from cursor',function() mission_command('test') end},
  {'Mission: next marker kind',function() MK.next_kind() end},
  {'Mission: place marker at cursor',function() MK.place_at_cursor() end},
  {'Mission: select nearest marker',function() MK.select_nearest() end},
  {'Mission: move marker to cursor',function() MK.move_to_cursor() end},
  {'Mission: delete marker',function() MK.delete() end},
}
local function attempt(fn)
  local ok,why=pcall(fn) if not ok then say('Error: '..tostring(why),'error') end return ok
end
local function search_matches(text)
  local out={}
  text=(text or ''):lower()
  for i,a in ipairs(ACTIONS) do
    if text=='' or a[1]:lower():find(text,1,true) then out[#out+1]=i end
  end
  return out
end
local function search_execute(st, index)
  local m=search_matches(st and st.text)
  local i=m[math.min(math.max(index or (st and st.index) or 1,1),#m)]
  if i then attempt(ACTIONS[i][2]) else say('No matching action') end
end

gd.command('map',function(arg)
  attempt(function()
    local op,name=arg:match('^(%S+)%s*(.-)%s*$') op=op or 'toggle'
    if name=='' then name=nil end
    if op=='save' then save(name) return end -- Permit recovery of unsaved data outside a match.
    assert(offline(), 'active offline match required')
    if op=='on' then start()
    elseif op=='off' then mission_stop('map off') stop()
    elseif op=='toggle' then if editing then stop() else start() end
    elseif op=='load' then load_map(name) start()
    elseif op=='play' then load_map(name) stop() autoload=filename mission_autostart()
    elseif op=='mission' then mission_command(name)
    elseif op=='place' then place(false)
    elseif op=='duplicate' then
      local n=tonumber(name)
      if n and n>1 then duplicate_many(math.floor(n)) else place(true) end
    elseif op=='select' then
      if name=='add' then select_add()
      elseif name=='remove' then select_remove()
      elseif name=='all' then
        clear_group()
        if not selected and parts[1] then selected=parts[1].id end
        for _,p in ipairs(parts) do if p.id~=selected then group[p.id]=true end end
        say(('Selected %d parts'):format(#selected_ids()),'action')
      elseif name=='clear' then
        clear_group() say('Selection cleared','action')
      else select_near() end
    elseif op=='move' then transform('move')
    elseif op=='rotate' then transform('rotate',tonumber(name) or 15)
    elseif op=='scale' then transform('scale',tonumber(name) or SCALE_STEP)
    elseif op=='unscale' then transform('unscale')
    elseif op=='mirror' then
      assert(name=='x' or name=='y', 'mirror x|y')
      transform('mirror',name)
    elseif op=='tool' then
      local found=nil
      for _,t in ipairs(TOOLS) do if t==name then found=t end end
      assert(found,'map tool: place|select|move|rotate|scale')
      tool=found axis_lock=nil say('Tool: '..tool)
    elseif op=='snap' then
      assert(name=='on' or name=='off','snap on|off') snap_on=name=='on' say('Snap '..name)
    elseif op=='help' then
      assert(name=='on' or name=='off','help on|off') help_open=name=='on' say('Help '..name)
    elseif op=='delete' then remove()
    elseif op=='undo' then history(true)
    elseif op=='redo' then
      local n=tonumber(name)
      if n and n>1 then redo_jump(math.floor(n)) else history(false) end
    elseif op=='clear' then edit() acted({},'Cleared map') grid=2
    elseif op=='part' then
      local want=name and (name:find('^bf_') and name or 'bf_'..name)
      local found=nil
      for i,n in ipairs(PALETTE) do if n==want then found=i end end
      assert(found,'map part: unknown part '..tostring(name))
      filter='' filtering=false
      local view=palette_view() palette=1
      for pos,idx in ipairs(view) do if idx==found then palette=pos end end
      say('Part: '..want)
    elseif op=='set' then
      local field,value=(name or ''):match('^(%S+)%s*(.-)%s*$')
      assert(field and value~='','map set <x|y|z|rot|scale> <value>')
      field_set(field,value)
    elseif op=='spawn' then
      local slot,rest=(name or ''):match('^(%d*)%s*(.-)%s*$')
      local n=tonumber(slot)
      assert(n and ((n>=0 and n<=7) or (n>=127 and n<=146)),
             'map spawn <slot> [x y]: 0-3 starts, 4-7 respawns, 127-146 item spawns')
      local x,y=rest:match('^(%S+)%s+(%S+)$')
      if x then
        x,y=tonumber(x),tonumber(y)
        assert(x and y and number(x) and number(y),'spawn needs a finite x y')
        edit()
        local before=doc_snapshot()
        assert(gd.stage_set_spawn(n,x,y))
        spawns[n]={x=x,y=y} dirty=true
        doc_commit(before,('spawn %d'):format(n))
      else
        local sx,sy,sz=gd.stage_spawn(n)
        say(('spawn %d at %.1f %.1f %.1f'):format(n,sx or 0,sy or 0,sz or 0),'action')
      end
    elseif op=='bounds' then
      local kind,rest=(name or ''):match('^(%S*)%s*(.-)%s*$')
      if kind=='' or kind==nil then
        local b=gd.stage_bounds() or {}
        local function fmt(r) return r and ('%.1f %.1f %.1f %.1f'):format(r.left,r.right,r.top,r.bottom) or '-' end
        say('camera '..fmt(b.camera)..' | blast '..fmt(b.blast),'action')
      else
        edit()
        local before=doc_snapshot()
        if kind=='capture' then
          local b=assert(gd.stage_bounds(),'no stage bounds')
          assert(b.camera,'this stage has no camera bounds')
          bounds.camera=b.camera
          bounds.blast=b.blast or bounds.blast
          dirty=true doc_commit(before,'Bounds captured')
        elseif kind=='restore' then
          assert(gd.stage_restore_bounds())
          bounds={camera=nil,blast=nil} spawns={}
          if mission_doc and mission_doc.start then assert(gd.stage_set_spawn(0,mission_doc.start.x,mission_doc.start.y)) end
          dirty=true doc_commit(before,'Bounds restored')
        else
          local l,r,t,bt=rest:match('^(%S+)%s+(%S+)%s+(%S+)%s+(%S+)$')
          assert(l and (kind=='camera' or kind=='blast'),
                 'map bounds [capture|restore|camera l r t b|blast l r t b]')
          local rect={left=tonumber(l),right=tonumber(r),top=tonumber(t),bottom=tonumber(bt)}
          assert(rect.left and rect.right and rect.top and rect.bottom and rect.left<rect.right and rect.bottom<rect.top,
                 'bounds require left < right and bottom < top')
          if kind=='camera' then assert(gd.stage_set_camera_bounds(rect.left,rect.right,rect.top,rect.bottom))
          else assert(gd.stage_set_blast_bounds(rect.left,rect.right,rect.top,rect.bottom)) end
          bounds[kind]=rect dirty=true doc_commit(before,kind..' bounds set')
        end
      end
    elseif op=='log' then
      log_open=(name~='off')
      say(log_open and 'Action log on' or 'Action log off','action')
    elseif op=='history' then
      local n=math.max(1,math.floor(tonumber(name) or 1))
      history_jump(n)
    elseif op=='run' then
      assert(name and name~='','map run <text>')
      search_execute({text=name:lower(),index=1})
    elseif op=='search' then
      if name=='off' then
        search=nil
        say('Search off')
      else
        search={text=(name or ''):lower(),index=1}
        say('Search on')
      end
    elseif op=='ghost' then
      assert(name=='on' or name=='off','ghost on|off')
      ghost_on=name=='on'
      if not ghost_on then ghost_despawn() end
      say('Ghost '..name,'action')
    elseif op=='filter' then
      filter=(name or ''):lower() filtering=false palette=1
      say(filter=='' and 'Filter cleared' or ('Filter: '..filter))
    else error('map on|off|part <name>|tool <name>|filter [text]|ghost on|off|set <field> <value>|log [on|off]|history [n]|run <text>|scale <f>|mirror x|y|snap on|off|help on|off|place|select|move|rotate [deg]|duplicate|delete|undo|redo|clear|save|load|play [file.lua]|mission ...') end
  end)
end,'map on/off; part <name>; tool <place|select|move|rotate|scale>; scale <factor>; mirror x|y; snap on|off; place/select/move/rotate/duplicate/delete/undo/redo/clear; save/load/play [file.lua]; mission start|enemy|goal|checkpoint|objective|list|delete|clear|play|restart|stop')

-- Panel geometry is one function for drawing and hit-testing, so rows and clicks cannot drift.
local function panel_rows()
  local rows={}
  local y=70
  for i,t in ipairs(TOOLS) do
    rows[#rows+1]={kind='tool',index=i,label=(tool==t and '> ' or '  ')..t,x=16,y=y,w=230,h=19}
    y=y+20
  end
  y=204
  if tool=='place' then
    local view=palette_view()
    if #view==0 then
      rows[#rows+1]={kind='none',label='(no matches)',x=16,y=y,w=230,h=18}
    else
      local visible=5
      local first=math.max(1,math.min(palette-2,math.max(1,#view-visible+1)))
      local category=nil
      for i=first,math.min(first+visible-1,#view) do
        local name=PALETTE[view[i]]
        local cat=palette_category(name)
        if cat~=category then
          category=cat
          rows[#rows+1]={kind='header',label=cat,x=16,y=y,w=230,h=14}
          y=y+14
        end
        local pinned=false
        for _,r in ipairs(recents) do if r==name then pinned=true end end
        rows[#rows+1]={kind='part',index=i,label=(pinned and '* ' or '  ')..name:gsub('^bf_',''),
                       value=cat,x=16,y=y,w=230,h=18}
        y=y+18
      end
    end
  else
    local first=math.max(1,math.min(action_index-3,#ACTIONS-6))
    for i=first,math.min(first+6,#ACTIONS) do
      rows[#rows+1]={kind='action',index=i,label=ACTIONS[i][1],x=16,y=y,w=230,h=18}
      y=y+20
    end
  end
  rows[#rows+1]={kind='help',label='Help',value='F1',x=16,y=344,w=230,h=20}
  return rows
end
local function in_rect(mx,my,r)
  return r and mx>=r.x and mx<=r.x+r.w and my>=r.y and my<=r.y+r.h
end
local function inspector_rows()
  if tool=='mission' then return MK.rows() end
  local rows={}
  local p=find(parts,selected)
  local x=canvas_w()-240
  local y=70
  if not p then
    rows[#rows+1]={kind='info',label='no selection',x=x,y=y,w=224,h=20}
    return rows
  end
  rows[#rows+1]={kind='name',label=p.part:gsub('^bf_',''),x=x,y=y,w=224,h=20}
  y=y+24
  for _,f in ipairs({{'x','%.2f'},{'y','%.2f'},{'z','%.2f'},{'rot','%.1f'},{'scale','%.2f'}}) do
    local value
    if typing and typing.field==f[1] then value=typing.text..'_'
    else value=string.format(f[2], f[1]=='scale' and scale_of(p,'scale') or (p[f[1]] or 0)) end
    rows[#rows+1]={kind='field',field=f[1],label=f[1],value=value,x=x,y=y,w=224,h=18}
    y=y+20
  end
  rows[#rows+1]={kind='collision',label='collision',value=p.collision and 'on' or 'off',x=x,y=y,w=224,h=18}
  y=y+20
  rows[#rows+1]={kind='flags',label='floor flags',value=tostring(p.floor_flags or 0),x=x,y=y,w=224,h=18}
  return rows
end
local function click_inspector(mx,my)
  if help_open then return false end
  for _,r in ipairs(inspector_rows()) do
    if in_rect(mx,my,r) then
      if r.kind=='m' then
        if r.act then attempt(r.act) end
      elseif r.kind=='collision' then
        attempt(function()
          edit()
          local target=clone(parts) local p=find(target,selected)
          p.collision=not p.collision
          acted(target,'collision '..(p.collision and 'on' or 'off'))
        end)
      elseif r.kind=='field' then
        field_drag={field=r.field, base=doc_snapshot(), sx=mouse.x, sy=mouse.y, used=false}
      elseif r.kind=='flags' then
        attempt(function()
          edit()
          local target=clone(parts) local p=find(target,selected)
          p.floor_flags=((p.floor_flags or 0)+1)%4
          acted(target,'floor flags '..p.floor_flags)
        end)
      end
      return true
    end
  end
  return false
end
local function click_overlay(mx,my)
  if help_open then return false end
  if log_open and log_rows then
    for _,r in ipairs(log_rows) do
      if in_rect(mx,my,r) then
        if r.rdepth then redo_jump(r.rdepth) else history_jump(r.depth) end
        return true
      end
    end
  end
  if search and search_rows then
    for _,r in ipairs(search_rows) do
      if in_rect(mx,my,r) then local st=search search=nil search_execute(st,r.index) return true end
    end
  end
  return false
end
local function click_panel(mx,my)
  for _,r in ipairs(panel_rows()) do
    if in_rect(mx,my,r) then
      if r.kind=='tool' then tool=TOOLS[r.index] axis_lock=nil say('Tool: '..tool)
      elseif r.kind=='header' then return true
      elseif r.kind=='part' then palette=r.index say('Part: '..palette_part())
      elseif r.kind=='help' then help_open=not help_open
      else attempt(ACTIONS[r.index][2]) end
      return true
    end
  end
  return false
end

local function poll_mouse()
  if not mouse_api or not editing then return end
  local mx,my,buttons,wheel=gd.mouse()
  mouse.x,mouse.y,mouse.buttons=tonumber(mx) or -1000,tonumber(my) or -1000,tonumber(buttons) or 0
  mouse.over = mouse.x>=0 and mouse.x<canvas_w() and mouse.y>=0 and mouse.y<480
  if mouse.over and (mouse.x~=mouse.lastx or mouse.y~=mouse.lasty) then
    mouse.used,mouse.lastx,mouse.lasty=60,mouse.x,mouse.y
  elseif mouse.used>0 then
    mouse.used=mouse.used-1
  end
  if MK.drag then
    MK.drag_step()
    mouse.prev=mouse.buttons
    return
  end
  if modal then
    if modal.src=='mouse' then
      if (mouse.buttons & 1)==1 then
        local t=modal_point(modal.base)
        if t then apply(t,false) modal.used=true end
      else
        modal_commit()
      end
    end
    mouse.prev=mouse.buttons
    return
  end
  if field_drag then
    local dx=mouse.x-field_drag.sx
    if math.abs(dx)+math.abs(mouse.y-field_drag.sy)>3 then field_drag.used=true end
    if field_drag.used then
      local t=field_point(field_drag.base,field_drag.field,dx)
      if t then apply(t,false) end
    end
    if (mouse.buttons & 1)==0 then
      local f=field_drag
      field_drag=nil
      if f.used then
        local t=field_point(f.base,f.field,mouse.x-f.sx)
        local tp,bp=find(t or {},selected),find(f.base.parts,selected)
        if t and tp and bp and not same(tp,bp) then
          undo[#undo+1]=f.base if #undo>HISTORY then table.remove(undo,1) end
          redo={}
          apply(t,false)
          name_undo(f.field..' '..string.format('%.2f',field_value(tp,f.field)))
          say(f.field..' '..string.format('%.2f',field_value(tp,f.field)),'action')
        end
      else
        typing={field=f.field, text=''}
        say('Type a value; Enter applies, ESC cancels')
      end
    end
    mouse.prev=mouse.buttons
    return
  end
  if bounds_drag then
    if (mouse.buttons & 1)==1 then
      local wx,wy=mouse_world()
      local r=bounds.camera
      if wx and r then
        local v=(bounds_drag.edge=='left' or bounds_drag.edge=='right') and sn(wx) or sn(wy)
        if bounds_drag.edge=='left' then r.left=math.min(v,r.right-U*grid)
        elseif bounds_drag.edge=='right' then r.right=math.max(v,r.left+U*grid)
        elseif bounds_drag.edge=='bottom' then r.bottom=math.min(v,r.top-U*grid)
        else r.top=math.max(v,r.bottom+U*grid) end
        pcall(gd.stage_set_camera_bounds,r.left,r.right,r.top,r.bottom)
      end
    else
      local before=bounds_drag.base
      bounds_drag=nil
      doc_commit(before,'camera bounds')
    end
    mouse.prev=mouse.buttons
    return
  end
  if marquee then
    if (mouse.buttons & 1)==1 then
      marquee.x,marquee.y=mouse.x,mouse.y
    else
      local m=marquee marquee=nil
      if not m.add then clear_group() end
      if math.abs(m.x-m.sx)<4 and math.abs(m.y-m.sy)<4 then
        local wx,wy=mouse_world()
        attempt(function() select_near(wx,wy) end)
      else
        local x1,x2=math.min(m.sx,m.x),math.max(m.sx,m.x)
        local y1,y2=math.min(m.sy,m.y),math.max(m.sy,m.y)
        local hit=0
        for _,p in ipairs(parts) do
          local px,py,on=gd.project(p.x,p.y,p.z)
          if px and on and px>=x1 and px<=x2 and py>=y1 and py<=y2 then
            group[p.id]=true hit=hit+1
          end
        end
        if selected and in_group(selected) then group[selected]=nil end
        if hit==0 then say('Nothing in the box') else say(('Box-selected %d'):format(#selected_ids()),'action') end
      end
    end
    mouse.prev=mouse.buttons
    return
  end
  local lmb=(mouse.buttons & 1)==1
  local pressed=lmb and (mouse.prev & 1)==0
  local right=(mouse.buttons & 2)==2 and (mouse.prev & 2)==0
  if wheel and wheel~=0 then
    if help_open then
      help_first=math.max(1,math.min(math.max(1,#HELP-11),help_first+wheel))
    else
      depth=depth+wheel*U*grid say(('depth %.2f'):format(depth))
    end
  end
  if right then
    if help_open then help_open=false else menu=not menu end
  elseif pressed then
    if not click_panel(mouse.x,mouse.y) and not click_inspector(mouse.x,mouse.y) and
       not click_overlay(mouse.x,mouse.y) and mouse.over and not help_open then
      local bedge=tool~='mission' and hit_bounds(mouse.x,mouse.y)
      if tool=='mission' then
        attempt(MK.press)
      elseif bedge then
        bounds_drag={edge=bedge, base=doc_snapshot()}
      else
      local hmode,hax=hit_handle(mouse.x,mouse.y)
      if hmode then
        if modal_begin(hmode,hax,'mouse') then
          modal.used=true
          local t=modal_point(modal.base) if t then apply(t,false) end
        end
      else
      local wx,wy=mouse_world()
      if tool=='select' then
        marquee={sx=mouse.x, sy=mouse.y, x=mouse.x, y=mouse.y, add=gd.key('SHIFT')==true}
      elseif tool=='place' then attempt(function() place(false,wx,wy) end)
      elseif tool=='move' then dragging=true attempt(function() transform('move',nil,true,wx,wy) end)
      elseif tool=='rotate' then attempt(rotate_to_mouse)
      elseif tool=='scale' then attempt(function() transform('scale',SCALE_STEP) end)
      end
      end
      end
    end
  end
  if lmb and dragging then
    local wx,wy=mouse_world()
    if wx then attempt(function() transform('move',nil,false,wx,wy) end) end
  end
  if not lmb then dragging=false end
  mouse.prev=mouse.buttons
end

-- The part nearest the shown cursor, for the hover highlight and the readout only.
local function hover_part()
  local x,y=shown_cursor() local best,d=nil,20.0
  for _,p in ipairs(parts) do
    local distance=math.sqrt((p.x-x)^2+(p.y-y)^2)
    if distance<d then best,d=p.id,distance end
  end
  return best
end

function on_tick()
  local pad=gd.pad(1) or {}
  for _,t in ipairs(toasts) do t.t=t.t-1 end
  local function pressed(k) return pad[k] and not old_pad[k] end
  if not offline() then stop() old_pad=pad return end
  if gd.key_pressed('F6') then attempt(function() if editing then stop() else start() end end) end
  if not editing then
    if pad.Z and pressed('UP') then attempt(start) end
    old_pad=pad return
  end
  if gd.key_pressed('F1') or gd.key_pressed('H') then help_open=not help_open end
  if help_open then
    if gd.key_pressed('ESCAPE') and not menu then help_open=false end
    if gd.key_pressed('DOWN') then help_first=math.min(math.max(1,#HELP-11),help_first+1) end
    if gd.key_pressed('UP') then help_first=math.max(1,help_first-1) end
  end
  if gd.key_pressed('SPACE') and not search and not typing and not filtering and not menu and not help_open then
    search={text='',index=1}
    say('Search actions: type, Up/Down, Enter runs, ESC closes')
  end
  if search then
    local ch=''
    for c in ('ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'):gmatch('.') do
      if gd.key_pressed(c) then ch=c end
    end
    if ch~='' then search.text=(search.text..ch):lower() search.index=1 end
    if gd.key_pressed('BACKSPACE') then search.text=search.text:sub(1,-2) search.index=1 end
    if gd.key_pressed('UP') then search.index=math.max(1,(search.index or 1)-1) end
    if gd.key_pressed('DOWN') then search.index=(search.index or 1)+1 end
    if gd.key_pressed('ENTER') then local st=search search=nil search_execute(st) end
    if gd.key_pressed('ESCAPE') then search=nil say('Cancelled') end
    old_pad=pad return
  end
  if typing then
    local ch=''
    for _,c in ipairs({'0','1','2','3','4','5','6','7','8','9'}) do
      if gd.key_pressed(c) then ch=c end
    end
    if gd.key_pressed('PERIOD') then ch='.' end
    if gd.key_pressed('MINUS') then ch='-' end
    if ch~='' then typing.text=(typing.text..ch):sub(-12) end
    if gd.key_pressed('BACKSPACE') then typing.text=typing.text:sub(1,-2) end
    if gd.key_pressed('ENTER') then
      local f=typing typing=nil
      if f.text~='' then
        if f.field:sub(1,2)=='m:' then attempt(function() MK.field_set(f.field:sub(3),f.text) end)
        else attempt(function() field_set(f.field,f.text) end) end
      end
    elseif gd.key_pressed('ESCAPE') then
      typing=nil say('Cancelled')
    end
    old_pad=pad return
  end
  if gd.key_pressed('F4') then
    filtering=not filtering
    if not filtering then filter='' end
    palette=1
    say(filtering and 'Type a part name; Enter done, ESC clears' or 'Filter cleared')
  end
  if filtering then
    local ch=''
    for c in ('ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'):gmatch('.') do
      if gd.key_pressed(c) then ch=c end
    end
    if ch~='' then filter=filter..ch:lower() palette=1 end
    if gd.key_pressed('BACKSPACE') then filter=filter:sub(1,-2) palette=1 end
    if gd.key_pressed('ESCAPE') then filter='' filtering=false palette=1 end
    if gd.key_pressed('ENTER') then filtering=false end
    old_pad=pad return
  end
  if gd.key_pressed('F2') or pressed('Z') then menu=not menu end
  if menu then
    if gd.key_pressed('UP') or pressed('UP') then action_index=(action_index-2)%#ACTIONS+1 end
    if gd.key_pressed('DOWN') or pressed('DOWN') then action_index=action_index%#ACTIONS+1 end
    if gd.key_pressed('ENTER') or pressed('A') then attempt(ACTIONS[action_index][2]) end
    if gd.key_pressed('ESCAPE') or pressed('B') then menu=false end
  else
    local handled = false
    if not help_open then
      handled = (modal and ((modal.mode=='move' and modal_key('G','move')) or
                            (modal.mode=='rotate' and modal_key('E','rotate')) or
                            (modal.mode=='scale' and modal_key('C','scale')))) or
                (not modal and not gd.key('SHIFT') and
                 (modal_key('G','move') or modal_key('E','rotate') or modal_key('C','scale')))
    end
    if handled then old_pad=pad return end
    for i,t in ipairs(TOOLS) do
      if gd.key_pressed(tostring(i)) then tool=t axis_lock=nil say('Tool: '..t) end
    end
    if not help_open then
      local view=palette_view()
      if tool=='mission' then
        if gd.key_pressed('UP') or pressed('UP') or gd.key_pressed('DOWN') or pressed('DOWN') then
          MK.kind=MK.cycle(MK.KINDS,MK.kind) say('Click places: '..MK.kind)
        end
      elseif #view>0 then
        if gd.key_pressed('UP') or pressed('UP') then palette=(palette-2)%#view+1 end
        if gd.key_pressed('DOWN') or pressed('DOWN') then palette=(palette-1)%#view+1 end
      end
    end
    if gd.key_pressed('PAGEUP') or pressed('RIGHT') then depth=depth+U*grid end
    if gd.key_pressed('PAGEDOWN') or pressed('LEFT') then depth=depth-U*grid end
    if gd.key_pressed('INSERT') or pressed('A') then attempt(function() place(false) end) end
    if gd.key_pressed('TAB') or pressed('X') then
      if gd.key('SHIFT') then attempt(select_add)
      elseif gd.key('CTRL') then attempt(select_remove)
      else attempt(select_near) end
    end
    if gd.key_pressed('M') or pressed('Y') then attempt(function() transform('move') end) end
    if gd.key_pressed('R') or pressed('R') then attempt(function() transform('rotate',15) end) end
    if gd.key_pressed('T') or pressed('L') then attempt(function() transform('rotate',-15) end) end
    if gd.key_pressed('F7') then attempt(function() transform('scale',1/SCALE_STEP) end) end
    if gd.key_pressed('F8') then attempt(function() transform('scale',SCALE_STEP) end) end
    if gd.key('SHIFT') and gd.key_pressed('C') then cycle_axis() end
    if gd.key_pressed('Z') then snap_on=not snap_on say('Snap '..(snap_on and 'on' or 'off'),'action') end
    if gd.key_pressed('F') then attempt(frame_selection) end
    if gd.key_pressed('DELETE') then attempt(tool=='mission' and MK.delete or remove) end
    if gd.key_pressed('F3') then overlay=not overlay set_overlay() end
    if gd.key('CTRL') then
      if gd.key_pressed('D') then attempt(function() place(true) end) end
      if gd.key_pressed('Z') then attempt(function() history(true) end) end
      if gd.key_pressed('Y') then attempt(function() history(false) end) end
      if gd.key_pressed('S') then attempt(function() save() end) end
      if gd.key_pressed('O') then attempt(function() load_map() end) end
    end
    if gd.key('SHIFT') then
      if gd.key_pressed('X') then attempt(function() transform('mirror','x') end) end
      if gd.key_pressed('Y') then attempt(function() transform('mirror','y') end) end
    end
  end
  old_pad=pad
end
function on_frame_pre()
  if not editing or not offline() then return end
  if mouse_api then poll_mouse() end
  ghost_sync()
  local hg=nil
  if mouse.over and not modal and not help_open and not typing and not filtering and not search
     and not field_drag and not bounds_drag and not marquee then
    local edge=hit_bounds(mouse.x,mouse.y)
    if edge then
      hg='camera '..edge
    else
      local hmode,hax=hit_handle(mouse.x,mouse.y)
      if hmode then hg=hmode..(hax and (' '..hax) or '') end
    end
  end
  if hg~=hover_gizmo then
    hover_gizmo=hg
    if hg then say('drag: '..hg) end
  end
  hover=hover_part()
  if menu or help_open or modal or filtering or typing or field_drag or search or bounds_drag or marquee or gd.key('CTRL') then return end
  local p=gd.player(1) if not p then return end
  local dx=(gd.key('D') and 1 or 0)-(gd.key('A') and 1 or 0)
  local dy=(gd.key('W') and 1 or 0)-(gd.key('S') and 1 or 0)
  if dx~=0 or dy~=0 then
    local speed=gd.fly_speed()*(gd.key('SHIFT') and 4 or 1)
    gd.teleport(1,p.x+dx*speed,p.y+dy*speed)
  end
end

HELP = {
  {'F6', 'start / exit the editor'},
  {'F1 / H', 'this help (ESC or click closes)'},
  {'F4', 'palette filter (type; Enter done, ESC clears)'},
  {'F2 / Z', 'action menu (all commands)'},
  {'1..5', 'tool: place, select, move, rotate, scale'},
  {'G / E / C', 'hold: move / rotate / scale (tap: switch tool)'},
  {'F', 'frame selection (fly cursor + depth)'},
  {'Shift / Ctrl', 'fine step / hard snap while dragging'},
  {'Z', 'snap toggle'},
  {'WASD', 'fly (Shift fast); P1 stick on pad'},
  {'LMB', 'use the tool at the pointer'},
  {'drag LMB', 'move tool: drag the selection'},
  {'RMB', 'action menu'},
  {'grid', 'new parts / reset: 2x; grid cycles 2 / 1 / 0.5 base metres'},
  {'wheel', 'depth, one grid step per notch'},
  {'Tab / X', 'select nearest part'},
  {'Insert / A', 'place at the cursor'},
  {'M / Y', 'move selection to cursor'},
  {'R / T', 'rotate +15 / -15'},
  {'rotate drag', 'rotate tool: face the pointer'},
  {'F7 / F8', 'scale -10% / +10%'},
  {'Shift+X / Y', 'mirror selection X / Y'},
  {'Shift+C', 'move constraint free / X / Y'},
  {'gizmo', 'drag the handles (hover names them): red/green move X/Y, cyan scale, gold ring rotate'},
  {'ghost', 'translucent placement preview; map ghost on|off'},
  {'Space', 'search actions; Enter runs the top match'},
  {'action log', 'map log on: click a step to go back'},
  {'bounds', 'map bounds capture|restore|camera l r t b|blast l r t b (drag a green edge)'},
  {'spawns', 'map spawn <slot> [x y]: starts 0-3, respawns 4-7, item spawns 127-146'},
  {'out of bounds', 'placing outside the blast zone / camera bounds warns (toast + log)'},
  {'multi-select', 'select tool: drag a box (Shift-drag adds); Shift+Tab adds / Ctrl+Tab drops; select all|clear; transforms hit the selection'},
  {'inspector', 'drag a field to scrub; click a field to type; Enter applies'},
  {'PgUp/PgDn', 'depth +/- one grid step'},
  {'Ctrl+D', 'duplicate at cursor'},
  {'Delete', 'remove selection'},
  {'Ctrl+Z / Y', 'undo / redo'},
  {'Ctrl+S / O', 'save / load layout'},
  {'F3', 'collision overlay'},
  {'missions', 'map mission start|enemy|goal|checkpoint|objective|list|delete|clear|play|restart: markers drawn in the overlay'},
}
local function draw_help()
  local kit=gd.kit
  local visible=12
  local pages=math.max(1,math.ceil(#HELP/visible))
  local last=math.max(1,#HELP-visible+1)
  if help_first>last then help_first=last end
  local hx=(canvas_w()-560)/2
  gd.fill(hx,24,560,432,0x0E1218FF)
  kit.panel(hx,24,560,432,{piece=16,fill=PANEL_FILL})
  gd.box(hx,24,560,432,0x8A92A0FF)
  local track_y, track_h = 120, 300
  local kh = math.max(30, math.floor(track_h*visible/#HELP))
  local kf = (help_first-1)/math.max(1,#HELP-visible)
  gd.fill(hx+544,track_y,8,track_h,0x464F5EFF)
  gd.fill(hx+544,track_y+kf*(track_h-kh),8,kh,0xE8C878FF)
  kit.text(hx+20,54,'MAP EDITOR - KEYBINDS','label','gold')
  kit.text(hx+20,76,('page %d/%d - Up/Down scrolls - F1/H/ESC closes')
    :format((help_first-1)//visible+1,pages),'caption','muted',nil,{max_w=480})
  kit.text(hx+20,96,'pad: Z menu; Tab/X select; M/Y move; R/L rotate','caption','muted',nil,{max_w=480})
  for i=help_first,math.min(#HELP,help_first+visible-1) do
    local y=120+(i-help_first)*28
    kit.text(hx+20,y,HELP[i][1],'caption','bone')
    kit.paragraph(hx+130,y,410,HELP[i][2],'caption','muted')
  end
end
-- Mission markers use their own colours: start green, enemies red, checkpoints blue, goal gold.
local function draw_mission_markers()
  local m=mission_doc
  if not m then return end
  local kit=gd.kit
  local function label(x,y,text,color) if kit.available() then kit.text(x,y,text,'caption',color) end end
  local function chosen(kind,i) return tool=='mission' and MK.sel and MK.sel.kind==kind and MK.sel.index==i end
  local function rect(z,color,fill,text,name,on)
    local x1,y1=gd.project(z.x-z.w/2,z.y+z.h/2,0)
    local x2,y2=gd.project(z.x+z.w/2,z.y-z.h/2,0)
    if not (x1 and y1 and x2 and y2) then return end
    local l,t,w,h=math.min(x1,x2),math.min(y1,y2),math.abs(x2-x1),math.abs(y2-y1)
    gd.fill(l,t,w,h,fill) gd.box(l,t,w,h,on and 0xFFFFFFFF or color) label(l+3,t+12,text,name)
    if on then for _,c in ipairs({{l,t},{l+w,t},{l,t+h},{l+w,t+h},{l+w/2,t},{l+w/2,t+h},{l,t+h/2},{l+w,t+h/2}}) do
      gd.fill(c[1]-3,c[2]-3,6,6,0xFFFFFFFF)
    end end
  end
  local function pin(x,y,color,text,name,on)
    local sx,sy,vis=gd.project(x,y,0)
    if not (sx and vis) then return end
    gd.box(sx-6,sy-6,12,12,on and 0xFFFFFFFF or color) gd.line(sx-9,sy,sx+9,sy,color) gd.line(sx,sy-9,sx,sy+9,color)
    label(sx+9,sy-8,text,name)
  end
  if m.goal then rect(m.goal,0xFFD040FF,0xFFD04030,'GOAL','gold',chosen('goal',nil)) end
  for i,t in ipairs(m.triggers) do
    rect(t,0xC080FFFF,0xC080FF30,('T%d %s%s'):format(i,t.action,t.wave and (' '..t.wave) or ''),'bone',chosen('trigger',i))
    if t.at then pin(t.at.x,t.at.y,0xC080FFFF,'T'..i..' target','bone') end
  end
  local rule={}
  for _,r in ipairs(m.waves) do rule[r.wave]=r end
  for i,c in ipairs(m.checkpoints) do rect(c,0x40C0FFFF,0x40C0FF30,'CP'..i,'bone',chosen('checkpoint',i)) end
  if m.start then pin(m.start.x,m.start.y,0x60FF60FF,'START','ok',chosen('start',nil)) end
  for i,e in ipairs(m.enemies) do
    local r=rule[e.wave]
    pin(e.x,e.y,0xFF5050FF,('E%d %s w%d%s'):format(i,e.kind,e.wave,
        r and (r.time and (' @%gs'):format(r.time) or (' x%s%g'):format(r.dir==1 and '>' or '<',r.x)) or ''),'danger',
        chosen('enemy',i))
  end
end
-- HUD while a mission runs, banner once it has a result; both read the pure state.
local function draw_mission()
  if not run or not offline() then return end
  local kit=gd.kit
  local W=canvas_w()
  local result=run.state.result
  local function text(x,y,s,color,role)
    if kit.available() then kit.text(x,y,s,role or 'caption',color,nil,{max_w=W-40}) else gd.text(x,y-12,s) end
  end
  if result then
    local good=result.status=='complete'
    gd.fill(W/2-190,190,380,70,0x0E1218F0) gd.box(W/2-190,190,380,70,good and 0x60FF60FF or 0xFF5050FF)
    text(W/2-176,220,Mission.result_text(run.state),good and 'ok' or 'danger','label')
    text(W/2-176,246,'map mission restart to retry; F6 to edit','muted')
  else
    gd.fill(W/2-230,8,460,22,0x0E1218E0)
    text(W/2-220,24,Mission.hud(run.state),'gold')
  end
  if run.message then
    gd.fill(W/2-230,36,460,22,0x0E1218E0)
    text(W/2-220,52,run.message.text,'bone')
  end
end
local function draw_editor()
  if not editing or not offline() or not gd.player(1) then handles_ui=nil return end
  local x,y,z=shown_cursor()
  local sx,sy,on=gd.project(x,y,z)
  if sx and on then gd.line(sx-7,sy,sx+7,sy,0xFFE080FF) gd.line(sx,sy-7,sx,sy+7,0xFFE080FF) end
  local p=find(parts,selected)
  handles_ui=nil
  if p then
    local px,py,visible=gd.project(p.x,p.y,p.z)
    if px and visible then
      gd.box(px-10,py-10,20,20,0x60FFFFFF)
      -- Gizmo handles: screen-space directions from the projected +X / +Y, so they follow the camera.
      local ax,ay=gd.project(p.x+2,p.y,p.z)
      local bx,by=gd.project(p.x,p.y+2,p.z)
      local function unit(sx,sy)
        local dx,dy=sx-px,sy-py
        local d=math.sqrt(dx*dx+dy*dy)
        if d<0.001 then return 0,-1 end
        return dx/d,dy/d
      end
      local ux,uy=unit(ax or px+2,ay or py)
      local vx,vy=unit(bx or px,by or py-2)
      local L=34
      local hx2,hy2=px+ux*L,py+uy*L
      local gx2,gy2=px+vx*L,py+vy*L
      local sx2,sy2=px-vx*L,py-vy*L
      gd.line(px,py,hx2,hy2,0xE06060FF) gd.box(hx2-3,hy2-3,6,6,0xE06060FF)
      gd.line(px,py,gx2,gy2,0x60E060FF) gd.box(gx2-3,gy2-3,6,6,0x60E060FF)
      gd.line(px,py,sx2,sy2,0x40C0E0FF) gd.box(sx2-4,sy2-4,8,8,0x40C0E0FF)
      local r=20
      for i=0,11 do
        local a1=i/12*math.pi*2 local a2=(i+0.5)/12*math.pi*2
        gd.line(px+math.cos(a1)*r,py+math.sin(a1)*r,px+math.cos(a2)*r,py+math.sin(a2)*r,0xFFD060FF)
      end
      if tool=='rotate' then
        local t=math.rad(p.rot)
        gd.line(px,py,px+math.cos(t)*22,py-math.sin(t)*22,0xFFD060FF)
      end
      if hover_gizmo=='move x' then gd.box(hx2-6,hy2-6,12,12,0xFFFFFFFF)
      elseif hover_gizmo=='move y' then gd.box(gx2-6,gy2-6,12,12,0xFFFFFFFF)
      elseif hover_gizmo=='scale' then gd.box(sx2-7,sy2-7,14,14,0xFFFFFFFF)
      elseif hover_gizmo=='rotate' then
        gd.box(px-24,py-24,48,48,0x60FFFFFF)
      end
      handles_ui={movex={px,py,hx2,hy2}, movey={px,py,gx2,gy2}, scale={sx2,sy2}, rot={px,py,r}}
    end
  end
  if bounds.camera or bounds.blast then
    local cw=canvas_w()
    local function clampv(v,lo,hi) return math.max(lo,math.min(hi,v)) end
    local function draw_rect(b,color,kind)
      local x1,y1=gd.project(b.left,b.top,0)
      local x2,y2=gd.project(b.right,b.bottom,0)
      if not (x1 and y1 and x2 and y2) then return end
      x1,y1,x2,y2=clampv(x1,0,cw),clampv(y1,0,480),clampv(x2,0,cw),clampv(y2,0,480)
      gd.box(x1,y1,x2-x1,y2-y1,color)
      if kind=='camera' then
        local hp=bounds_handle_positions() or {}
        for _,h in pairs(hp) do gd.box(h.x-4,h.y-4,8,8,0x40E060FF) end
        if hover_gizmo and hover_gizmo:find('^camera') then
          local edge=hover_gizmo:match('^camera (%a+)$')
          if edge and hp[edge] then gd.box(hp[edge].x-7,hp[edge].y-7,14,14,0xFFFFFFFF) end
        end
      end
    end
    if bounds.camera then draw_rect(bounds.camera,0x40E060FF,'camera') end
    if bounds.blast then draw_rect(bounds.blast,0xE06060FF,'blast') end
  end
  draw_mission_markers()
  if marquee then
    local x1,x2=math.min(marquee.sx,marquee.x),math.max(marquee.sx,marquee.x)
    local y1,y2=math.min(marquee.sy,marquee.y),math.max(marquee.sy,marquee.y)
    gd.box(x1,y1,x2-x1,y2-y1,0x80FFFFFF)
  end
  local h=hover and find(parts,hover)
  if h and h.id~=selected then
    local hx,hy,hv=gd.project(h.x,h.y,h.z)
    if hx and hv then gd.box(hx-8,hy-8,16,16,0xC080FFFF) end
  end
  for _,q in ipairs(parts) do
    if q.id~=selected and group[q.id] then
      local qx,qy,qv=gd.project(q.x,q.y,q.z)
      if qx and qv then gd.box(qx-7,qy-7,14,14,0xFFA040FF) end
    end
  end
  local kit=gd.kit
  if not kit.available() then gd.text(12,12,'Map editor: menu kit assets missing; use map console commands') return end
  local W=canvas_w()
  gd.fill(8,10,W-16,30,0x0E1218FF)
  kit.panel(8,10,W-16,30,{piece=12,fill=PANEL_FILL})
  kit.text(20,30,'MAP EDITOR','label','gold')
  kit.text(150,30,('tool: %s%s'):format(tool,axis_lock and (' ['..axis_lock..' axis]') or ''),'caption','bone')
  kit.text(340,30,('part: %s'):format(palette_part():gsub('^bf_','')),'caption','bone',nil,{max_w=140})
  kit.text(W-12,30,('%s%s'):format(dirty and '* ' or '',filename),'caption','muted','right',{max_w=200})
  gd.fill(8,44,246,338,0x0E1218FF)
  kit.panel(8,44,246,338,{piece=16,fill=PANEL_FILL})
  kit.text(20,62,'TOOL','caption','gold')
  kit.text(20,198,tool=='place' and ('PART'..(filter~='' and (' /'..filter..(filtering and '_' or '')) or '')) or 'ACTIONS','caption','gold')
  for _,r in ipairs(panel_rows()) do
    if r.kind=='header' then
      kit.text(r.x+4,r.y+12,r.label,'caption','gold')
    else
    local state=(r.kind=='tool' and tool==TOOLS[r.index]) or
                (r.kind=='part' and palette==r.index) or
                (r.kind=='action' and action_index==r.index) or
                (r.kind=='help' and help_open) or false
    local value = r.kind=='tool' and tostring(r.index) or r.value
    kit.button(r.x,r.y,r.w,r.label,state and 'sel' or 'ng',{h=r.h,value=value})
    end
  end
  gd.fill(W-248,44,240,338,0x0E1218FF)
  kit.panel(W-248,44,240,338,{piece=16,fill=PANEL_FILL})
  kit.text(W-236,62,'SELECTION','caption','gold')
  for _,r in ipairs(inspector_rows()) do
    kit.button(r.x,r.y,r.w,r.label,'ng',{h=r.h,value=r.value})
  end
  gd.fill(8,376,W-16,54,0x0E1218FF)
  kit.panel(8,376,W-16,54,{piece=12,fill=PANEL_FILL})
  kit.text(20,392,HINTS[tool],'caption','bone',nil,{max_w=440})
  kit.text(W-12,392,('XYZ %.2f %.2f %.2f'):format(x,y,z),'caption','muted','right',{max_w=170})
  kit.text(20,408,(dirty and '* ' or '')..filename,'caption','muted',nil,{max_w=300})
  kit.text(W-12,408,('grid %.2g world units | snap %s | parts %d/%d'):format(U*grid,snap_on and 'on' or 'off',#parts,MAX_PARTS),
    'caption','muted','right',{max_w=300})
  local line=error_text and ('error: '..error_text) or (last_action or '')
  kit.text(20,424,line,'caption',error_text and 'danger' or 'gold',nil,{max_w=598})
  for i=#toasts,1,-1 do if toasts[i].t<=0 then table.remove(toasts,i) end end
  for i,t in ipairs(toasts) do
    local ty=356-(i-1)*22
    gd.fill(W-178,ty,166,18,0x0E1218E0)
    kit.text(W-170,ty+13,t.msg,'caption','gold',nil,{max_w=150})
  end
  log_rows=nil
  if log_open then
    local lw=300 local nu=math.min(#undo_names,6) local nr=math.min(#redo_names,3)
    local lh=44+(nu+nr)*18
    local lx=(W-lw)/2 local ly=150
    gd.fill(lx,ly,lw,lh,0x0E1218F0)
    kit.panel(lx,ly,lw,lh,{piece=12,fill=PANEL_FILL})
    kit.text(lx+12,ly+18,'ACTION LOG - click a step','caption','gold')
    log_rows={}
    for i=1,nu do
      local y=ly+26+(i-1)*18
      log_rows[#log_rows+1]={depth=i,x=lx+6,y=y,w=lw-12,h=16}
      kit.button(lx+6,y,lw-12,undo_names[#undo_names-i+1] or '','ng',{h=16})
    end
    for i=1,nr do
      local y=ly+26+(nu+i-1)*18
      log_rows[#log_rows+1]={rdepth=i,x=lx+6,y=y,w=lw-12,h=16}
      kit.button(lx+6,y,lw-12,'redo: '..(redo_names[#redo_names-i+1] or ''),'ng',{h=16})
    end
  end
  search_rows=nil
  if search then
    local m=search_matches(search.text)
    local sw=320 local rows=math.min(#m,7)
    local sh=44+rows*18
    local sx=(W-sw)/2 local sy=150
    gd.fill(sx,sy,sw,sh,0x0E1218F0)
    kit.panel(sx,sy,sw,sh,{piece=12,fill=PANEL_FILL})
    kit.text(sx+12,sy+18,'SEARCH: '..search.text..'_','caption','gold')
    search_rows={}
    for i=1,rows do
      local y=sy+26+(i-1)*18
      search_rows[i]={index=i,x=sx+6,y=y,w=sw-12,h=16}
      local sel=i==math.min(math.max(search.index or 1,1),math.max(#m,1))
      kit.button(sx+6,y,sw-12,ACTIONS[m[i]][1],sel and 'sel' or 'ng',{h=16})
    end
  end
  if help_open then gd.fill(0,0,W,480,0x000000A8) draw_help() end
end
function on_draw()
  draw_mission()
  draw_editor()
end
function on_frame()
  if want_edit then
    local w=want_edit
    if editing or not offline() then
      want_edit=nil
      if w.nudged then pcall(gd.release_pad,1) end
    else
      w.left=w.left-1
      -- Flight and teleport are both refused in states such as OttottoWait (teetering at a floor edge),
      -- which never ends by itself: a short stick-down nudge ends it. The pad claim is released after.
      if w.left%10==0 then w.nudged=pcall(gd.input,1,{y=-100},2) or w.nudged end
      if pcall(start) and editing then
        want_edit=nil
        if w.nudged then pcall(gd.release_pad,1) end
      elseif w.left<=0 then
        want_edit=nil
        if w.nudged then pcall(gd.release_pad,1) end
        say('Could not return to editing: press F6','error')
      end
    end
  end
  if not run or run.state.result then return end
  if not offline() then mission_stop('left the offline match') return end
  run.frames=run.frames+1
  if run.message then run.message.left=run.message.left-1 if run.message.left<=0 then run.message=nil end end
  mission_step()
end

-- Model slots are snapshotted, Lua is not. Pair manual savestates with document snapshots;
-- on other loads, refuse to mutate until a new match rather than deleting unknown instances.
function on_savestate(slot)
  saved_states[slot]={parts=clone(parts),handles=clone(handles),assets=clone(assets),
                      previous=previous and clone(previous),ghost=ghost and clone(ghost),selected=selected,broken=broken,
                      bounds={camera=copy_rect(bounds.camera),blast=copy_rect(bounds.blast)},spawns=clone(spawns),
                      mission=mission_doc and clone(mission_doc)}
end
function on_loadstate(slot)
  if not offline() then return end
  -- Host overlay is not snapshotted. Restore it from the current UI session, but do not
  -- overwrite restored native flight with the current timeline's saved fly setting.
  stop(false)
  mission_stop('state load') -- Lua state is not in the snapshot
  local saved=saved_states[slot]
  if saved then
    -- The native snapshot restored this ghost, including its asset reference.
    ghost=saved.ghost and clone(saved.ghost)
    ghost_despawn()
    if saved.previous then gd.fly(1,saved.previous.fly) end
    parts,handles,assets=clone(saved.parts),clone(saved.handles),clone(saved.assets)
    selected=saved.selected
    undo={} redo={} dirty=true broken=saved.broken
    if saved.bounds then bounds=saved.bounds end
    if saved.spawns then spawns=saved.spawns end
    mission_doc=saved.mission
    pcall(doc_restore,{bounds=bounds,spawns=spawns,mission=mission_doc})
    for _,h in pairs(handles) do if not gd.model_get(h.handle) then broken=true end end
    say(broken and 'Savestate handles missing; save and restart' or 'Restored document; undo history cleared')
  else
    assets={} -- Unknown native references: do not release another snapshot's asset tokens.
    broken=true say('Untracked state load: save layout and restart match before editing')
  end
end
function on_match_end()
  -- Engine already frees scene models. Retain unsaved document for save/reinstantiation.
  mission_stop('match end') test_return=nil stop() handles={} assets={} undo={} redo={} saved_states={} broken=false
end
function on_match_start()
  handles={} assets={} broken=false
  if not offline() then return end
  attempt(function()
    if autoload then load_map(autoload) mission_autostart() else sync(parts) doc_restore(doc_snapshot()) say('Restored layout on new stage') end
  end)
end
function on_unload()
  mission_stop('unload') test_return=nil
  stop()
  ghost_despawn()
  if offline() then
    for _,h in pairs(handles) do pcall(gd.model_despawn,h.handle) end
    for _,a in pairs(assets) do pcall(gd.model_release,a) end
  end
  handles={} assets={}
end

gd.log('map_editor: ready; F6 or Z+D-pad Up to edit; F1 help; map play layout.lua to load a map')
