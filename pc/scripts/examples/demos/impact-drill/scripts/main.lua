-- Crit impact frames on Fox's drill (down air). Offline LAB, vanilla Final Destination.
-- Treat every hit of the drill as a crit and play a short, strength-scaled impact sequence.
-- Visual only: no gameplay changes unless `impact freeze` (extra freeze frames) is set above 0.
-- Read top to bottom. Everything tunable is in CFG / PRESETS / look().

local ACTION_DAIR = 69          -- ftCo_MS_AttackAirLw (verified in game: see README)
local RAMP_LOW    = 0.30        -- ramp mode: the first hit of a drill plays at 30% of the strength...
local RAMP_HITS   = 3           -- ...rising linearly to 100% by this hit; later hits stay at 100%
local BURST_GAP   = 0.30        -- a sequence starting within this many seconds of the last one is a repeat
local REPEAT_TEAR_AMT, REPEAT_TEAR_R = 0.50, 0.45   -- repeats keep a small two-tone bubble, never a full-screen flip
local DRILL_GAP   = 24          -- logic frames without a drill hit that end the drill (ramp reset)

local PRESETS = { light = 0.25, medium = 0.55, heavy = 0.85 }   -- `clank` is the original look
local ORDER   = { "light", "medium", "heavy", "clank" }

local CFG = {
  on = true, preset = "light", strength = 0.25, ramp = false, chance = 100,
  who = "p1drill", hud = true, freeze = 0, log = false, seed = 1,
}

local crit_shader, clank_shader      -- loaded once per match, never recompiled per hit
local last_start = -10             -- gd.time() of the previous sequence start
local pass, pass_kind, started, dur  -- the single live pass of this mod (nil when none)
local world                          -- world contact point of the live sequence
local STAT = { fired = 0, skipped = 0, hits = 0, restarts = 0, last = "-" }
local drill = {}                     -- [attacker port] = {n=, frame=}
local counter = 0                    -- crit-chance draw counter (deterministic)
local failed

local function offline() local m = gd.match(); return m and m.active and not m.netplay end
local function clamp(x, a, b) return x < a and a or (x > b and b or x) end

-- ---------------------------------------------------------------- the look
-- One scalar s (0..1) drives every layer. Durations are wall-clock seconds.
local function look(s)
  return {
    dur      = 0.16 + 0.34 * s,                         -- 0.245 s at light, 0.5 s at 1.0
    tear_s   = (s < 0.45) and (1 / 60) or (2 / 60),     -- the hard frame: 1 logic frame, 2 above 0.45
    tear_amt = 0.30 + 0.70 * s,                         -- how hard the two-tone cut is
    tear_inv = clamp((s - 0.55) / 0.3, 0, 1) * 0.9,     -- inversion only from medium up
    tear_r   = 0.30 + 1.60 * s,                         -- radius of the cut (height units); >1.4 = whole screen
    tear_thr = 0.45,
    lines    = clamp((s - 0.15) / 0.6, 0, 1) * 0.9,     -- speed lines come in after light
    blur     = 0.010 + 0.070 * s,
    ca       = 0.002 + 0.014 * s,
    ring     = 0.35 + 0.65 * s,
    contrast = 0.20 + 0.80 * s,
  }
end

local function ensure_crit()
  if failed then return false end
  if crit_shader then return true end
  local h, err = gd.shader_load("shaders/crit.wgsl", { params = {
    center = {0.5, 0.5, 0, 0}, progress = 0, elapsed = 0, tear_s = 0, tear_amt = 0, tear_inv = 0,
    tear_r = 0, tear_thr = 0.45, lines = 0, blur = 0, ca = 0, ring = 0, contrast = 0 } })
  if not h then failed = true; gd.log("impact-drill: crit shader disabled: " .. tostring(err)); return false end
  crit_shader = h
  return true
end
local function ensure_clank()
  if clank_shader then return true end
  local h, err = gd.shader_load("shaders/clank.wgsl", { params = {
    center = {0.5, 0.5, 0, 0}, progress = 0, elapsed = 0, intensity = 1, flash_limit = 0, limiter_strength = 0 } })
  if not h then gd.log("impact-drill: clank shader unavailable: " .. tostring(err)); return false end
  clank_shader = h
  return true
end

local function center_uv()
  if world then
    local x, y, visible = gd.project(world.x, world.y, world.z or 0)
    if x and visible then local a = gd.safe_area(); return { x / a.w, y / a.h, 0, 0 } end
  end
  return { 0.5, 0.5, 0, 0 }
end

local function drop()
  if pass then gd.post_remove(pass); pass = nil end
  started = nil
end

-- Pipeline warm-up: a first use compiles the shader (measured 90-260 ms hitch). Draw each shader once,
-- finished (progress=1 returns the untouched scene), for 8 presentation ticks, at match start / preset change.
local warmed = {}
local function warm(kind)
  if warmed[kind] or not offline() then return end
  if kind == "clank" then if not ensure_clank() then return end
  elseif not ensure_crit() then return end
  local h = gd.post_add(kind == "clank" and clank_shader or crit_shader, { order = 100, stage = "world", half = false,
    duration_frames = 8, params = kind == "clank" and { progress = 1, intensity = 1 } or { progress = 1 } })
  if h then warmed[kind] = true end
end

-- Start (or restart) the sequence. A live pass is removed first, so at most one exists.
local function play(s, point, kind)
  if kind == "clank" then if not ensure_clank() then return false end
  elseif not ensure_crit() then return false end
  local restarted = pass ~= nil
  drop()
  world = point
  local err
  if kind == "clank" then
    dur = 1.18
    pass, err = gd.post_add(clank_shader, { order = 100, stage = "world", half = false,
      duration_frames = 71, clock = true,
      params = { center = center_uv(), progress = 0, elapsed = 0, intensity = 1, flash_limit = 0, limiter_strength = 0 } })
  else
    local L = look(s)
    dur = L.dur
    L.dur = nil
    if gd.time() - last_start < BURST_GAP then   -- rapid repeat (the next hit of a drill): photosensitivity guard
      L.tear_inv = 0
      L.tear_amt = math.min(L.tear_amt, REPEAT_TEAR_AMT)
      L.tear_r = math.min(L.tear_r, REPEAT_TEAR_R)
    end
    L.center = center_uv(); L.progress = 0; L.elapsed = 0
    pass, err = gd.post_add(crit_shader, { order = 100, stage = "world", half = false,
      duration_frames = math.ceil(dur * 60), clock = true, params = L })
  end
  if not pass then failed = true; gd.log("impact-drill: pass failed: " .. tostring(err)); return false end
  pass_kind = kind; started = gd.time(); last_start = started
  STAT.fired = STAT.fired + 1; if restarted then STAT.restarts = STAT.restarts + 1 end
  local frames = math.floor(CFG.freeze * s + 0.5)
  if frames > 0 and offline() and gd.hitstop then gd.hitstop(frames) end   -- optional, default off
  return true
end

-- deterministic crit-chance draw: a fixed hash of (seed, draw number), never the game's RNG
local function roll(percent)
  counter = counter + 1
  local x = (counter * 0x9E3779B97F4A7C15 + CFG.seed * 0xBF58476D1CE4E5B9) & 0x7fffffffffffffff
  x = ((x ~ (x >> 29)) * 0x94D049BB133111EB) & 0x7fffffffffffffff
  x = x ~ (x >> 32)
  return (x % 10000) / 100 < percent
end

local function hit_strength(n)
  local s = CFG.strength
  if CFG.ramp then s = s * (RAMP_LOW + (1 - RAMP_LOW) * clamp((n - 1) / (RAMP_HITS - 1), 0, 1)) end
  return s
end

local function qualifies(attacker, info)
  if not attacker then return false end
  if CFG.who == "p1all" then return attacker == 1 end
  if CFG.who == "alldrill" then return info.attacker_action == ACTION_DAIR end
  return attacker == 1 and info.attacker_action == ACTION_DAIR
end

local function contact_point(victim, info)
  if type(info.x) == "number" and type(info.y) == "number" then return { x = info.x, y = info.y, z = info.z or 0 } end
  local v = gd.player(victim)
  if v then return { x = v.x, y = v.y + 8, z = v.z or 0 } end
end

-- Every hit event is its own call: each of the drill's hits fires (or restarts) the sequence.
function on_hit(attacker, victim, info)
  if not CFG.on or not qualifies(attacker, info) then return end
  STAT.hits = STAT.hits + 1
  local m = gd.match(); local frame = m and m.frame or 0
  local d = drill[attacker]
  if not d or frame - d.frame > DRILL_GAP then d = { n = 0 }; drill[attacker] = d end
  d.n = d.n + 1; d.frame = frame
  local crit = CFG.chance >= 100 or roll(CFG.chance)
  local s = hit_strength(d.n)
  if CFG.log then
    gd.log(("impact-drill hit atk=%s victim=%s action=%s tag=%s n=%d frame=%d crit=%s s=%.2f")
      :format(tostring(attacker), tostring(victim), tostring(info.attacker_action), tostring(info.move_tag), d.n, frame, tostring(crit), s))
  end
  if not crit then STAT.skipped = STAT.skipped + 1; return end
  STAT.last = ("hit %d  s=%.2f"):format(d.n, s)
  play(s, contact_point(victim, info), CFG.preset == "clank" and "clank" or "crit")
end

-- ---------------------------------------------------------------- per tick
local pad_prev = {}
local function edge(name, down) local was = pad_prev[name]; pad_prev[name] = down; return down and not was end

local function set_preset(name)
  CFG.preset = name
  if PRESETS[name] then CFG.strength = PRESETS[name] end
end
local function cycle(dir)
  local i = 1
  for k, v in ipairs(ORDER) do if v == CFG.preset then i = k end end
  i = (i - 1 + dir) % #ORDER + 1
  set_preset(ORDER[i])
end

function on_tick()
  if not offline() then if pass then drop() end; return end
  warm(CFG.preset == "clank" and "clank" or "crit")
  -- Shortcuts. Hold L+R (shield, which cannot taunt) and tap the D-pad; the C-stick is never read.
  local p = gd.pad(1)
  if p then
    local chord = p.L and p.R
    if edge("left", chord and p.LEFT) then cycle(-1) end
    if edge("right", chord and p.RIGHT) then cycle(1) end
    if edge("up", chord and p.UP) then CFG.strength = clamp(CFG.strength + 0.1, 0, 1); CFG.preset = "custom" end
    if edge("down", chord and p.DOWN) then CFG.strength = clamp(CFG.strength - 0.1, 0, 1); CFG.preset = "custom" end
  end
  if gd.key_pressed("F6") then cycle(1) end
  if gd.key_pressed("F7") then CFG.on = not CFG.on; if not CFG.on then drop() end end
  if gd.key_pressed("F8") then CFG.hud = not CFG.hud end
  if pass then
    local age = gd.time() - started
    if age >= dur + 0.05 then drop()
    elseif not gd.post_set(pass, { params = { center = center_uv() } }) then pass = nil; started = nil end
  end
end

function on_draw()
  if not CFG.hud or not offline() then return end
  local a = gd.safe_area()
  local txt = ("IMPACT %s  %s  s=%.2f  ramp %s  chance %d%%  who %s  freeze %d   [%s]")
    :format(CFG.on and "on" or "OFF", CFG.preset, CFG.strength, CFG.ramp and "on" or "off", CFG.chance, CFG.who, CFG.freeze, STAT.last)
  gd.fill(a.x + 8, a.y + 8, math.min(a.w - 16, #txt * 7 + 16), 20, 0x101827cc)
  gd.text(a.x + 16, a.y + 11, txt, 0xffd369ff, 1.0)
end

local function reset_all() drop(); warmed = {}; world = nil; drill = {}; crit_shader = nil; clank_shader = nil end
function on_match_end() reset_all() end
function on_scene() reset_all() end
function on_unload() drop(); gd.hitstop_cancel() end

-- ---------------------------------------------------------------- console
local function status()
  return ("impact %s preset=%s strength=%.2f ramp=%s chance=%d who=%s hud=%s freeze=%d | fired=%d skipped=%d hits=%d restarts=%d live=%s")
    :format(CFG.on and "on" or "off", CFG.preset, CFG.strength, tostring(CFG.ramp), CFG.chance, CFG.who,
      tostring(CFG.hud), CFG.freeze, STAT.fired, STAT.skipped, STAT.hits, STAT.restarts, tostring(pass ~= nil))
end
gd.command("impact", function(arg)
  local c, v = (arg or ""):match("^(%S*)%s*(.-)%s*$")
  if c == "preset" and (PRESETS[v] or v == "clank") then set_preset(v)
  elseif c == "strength" and tonumber(v) then CFG.strength = clamp(tonumber(v), 0, 1); if CFG.preset ~= "clank" then CFG.preset = "custom" end
  elseif c == "ramp" and (v == "on" or v == "off") then CFG.ramp = v == "on"
  elseif c == "chance" and tonumber(v) then CFG.chance = clamp(math.floor(tonumber(v)), 0, 100); counter = 0
  elseif c == "who" and (v == "p1drill" or v == "p1all" or v == "alldrill") then CFG.who = v
  elseif c == "freeze" and tonumber(v) then CFG.freeze = clamp(math.floor(tonumber(v)), 0, 12)
  elseif c == "seed" and tonumber(v) then CFG.seed = math.floor(tonumber(v)); counter = 0
  elseif c == "hud" and (v == "on" or v == "off") then CFG.hud = v == "on"
  elseif c == "log" and (v == "on" or v == "off") then CFG.log = v == "on"
  elseif c == "on" then CFG.on = true
  elseif c == "off" then CFG.on = false; drop()
  elseif c == "test" then
    if not offline() then gd.log("impact: no active offline match"); return end
    play(CFG.strength, nil, CFG.preset == "clank" and "clank" or "crit")
  elseif c == "state" then gd.log(status())
  else
    gd.log("impact preset light|medium|heavy|clank | strength 0..1 | ramp on|off | chance 0..100 | who p1drill|p1all|alldrill"
      .. " | test | on | off | hud on|off | freeze 0..12 | seed N | log on|off | state")
    return
  end
  gd.log(status())
end, "impact ...: crit impact frames on Fox's drill (try `impact`)")
