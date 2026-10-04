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
