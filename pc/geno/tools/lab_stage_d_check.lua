-- Stage D (TRAINING) logic check, off the game: a stub gd plays scripted frames through lab.lua and
-- checks frame advantage, L-cancel / wavedash / waveland / ledgedash / hop feedback and the move card.
-- Run with any Lua 5.4:  lua pc/geno/tools/lab_stage_d_check.lua pc/geno/mods/geno-lab/scripts/lab.lua
-- It checks the Lab's reading of the game's states and counters, not the game: the in-game checks are
-- in docs/geno.md 14.12.
local LAB = arg[1]
local unknown = {}
local function noop() end

local names = { [14] = "Wait", [24] = "KneeBend", [25] = "JumpF", [35] = "EscapeAir", [43] = "LandingFallSpecial",
  [65] = "AttackAirN", [70] = "LandingAirN", [42] = "Landing", [44] = "AttackS3S", [178] = "Guard",
  [180] = "GuardSetOff", [75] = "DamageN1", [253] = "CliffWait", [29] = "Fall", [27] = "JumpAerialF", [40] = "Squat" }
local ids = {}
for k, v in pairs(names) do ids[v] = k end

local P = {}
local function mkp(port, char)
  return { port = port, char = char, char_name = char == 1 and "Fox" or "Falco", cpu = false, action = 14,
    motion_name = "Wait", action_frame = 0, anim_frame = 0, in_hitlag = false, in_hitstun = false, iasa = false,
    hitboxes = {}, x = port * 10, y = 0, vy = 0, lr_age = 255, jump_age = 255, intangible = 0, airborne = false,
    percent = 0, anim_id = 1, facing = 1 }
end
P[1], P[2] = mkp(1, 1), mkp(2, 2)
local now = 0
local pad = { buttons = 0, x = 0, y = 0, cx = 0, cy = 0, l = 0, r = 0 }
local logs = {}

local K = setmetatable({ shear = 0.25, available = function() return true end, measure = function(s) return #s * 6 end,
  color = function() return 0xFFFFFFFF end }, { __index = function() return noop end })

local gd
gd = setmetatable({
  lab_api = 1, kit = K,
  draw = { DEFAULT = 1, MODEL = 2, HIT = 4, THROWN = 8 }, stage_draw = { COLL = 1, LEDGES = 4, TERRAIN = 2, POINTS = 8, ZONES = 16 },
  log = function(...) local t = {} for i = 1, select("#", ...) do t[#t + 1] = tostring(select(i, ...)) end logs[#logs + 1] = table.concat(t, " ") end,
  lab_request = function() return true end, data_read = function() return nil end, data_write = noop,
  match = function() return { active = true, frame = now, netplay = false } end, time = function() return now / 60 end,
  players = function() local t = {} for i = 1, 2 do t[#t + 1] = P[i] end return t end,
  player = function(i) return P[i] end,
  attrs = function(i) return { normal_landing_lag = 4, landingairn_lag = 15, hop_v_initial_velocity = 2.1, jump_v_initial_velocity = 3.68 } end,
  lab_common = function() return { lcancel_window = 7, lcancel_div = 2 } end,
  timeline = function(port, id)
    local p = P[port]
    if (id or p.action) == 44 then
      return { length = 26, end_frame = 25, events = { { frame = 5, name = "hitbox", id = 0, damage = 9, angle = 45, kbg = 100, bkb = 5, wbk = 0, size = 3, bone = 1, element_name = "normal" },
        { frame = 9, name = "hitboxes_clear" }, { frame = 20, name = "iasa" } } }
    elseif (id or p.action) == 65 then
      return { length = 49, end_frame = 48, events = { { frame = 4, name = "hitbox", id = 0, damage = 12, angle = 361, kbg = 100, bkb = 0, wbk = 0, size = 3, bone = 1, element_name = "normal" },
        { frame = 32, name = "hitboxes_clear" }, { frame = 3, name = "cmd_var", index = 0, value = 1 }, { frame = 40, name = "cmd_var", index = 0, value = 0 } } }
    end
    return { length = 10, events = {} }
  end,
  pad = function() return pad end, motion_name = function(id) return names[id] or ("M" .. id) end,
  history = function() return { busy = false, replaying = false, back = 0, fwd = 0, depth = 600 } end,
  debug_draw = function() return 1 end, debug_stage = function() return 0 end,
  key = function() return false end, key_pressed = function() return false end, paused = function() return false end,
  command = noop, lab_mode = function() return true end, lab_env = function() return nil end,
  rollbacks = function() return { total = 0, list = {} } end, project = function(x, y) return x, y, true end,
  line = noop, fill = noop, hitboxes = function() return {} end, mouse = function() return nil end,
}, { __index = function(_, k) unknown[k] = true return noop end })

local env = setmetatable({ gd = gd }, { __index = function(_, k)
  local v = _G[k]
  if v == nil then error("unknown global read: " .. tostring(k), 2) end
  return v
end })
local chunk = assert(loadfile(LAB, "t", env))
chunk()

local function set(port, name, extra)
  local p = P[port]
  if p.motion_name ~= name then p.action_frame = 0 p.anim_frame = 0 end
  p.motion_name, p.action = name, ids[name] or 999
  for k, v in pairs(extra or {}) do p[k] = v end
end
local function frame()
  now = now + 1
  for i = 1, 2 do
    P[i].action_frame = P[i].action_frame + 1
    P[i].anim_frame = P[i].anim_frame + 1
    if P[i].lr_age < 255 then P[i].lr_age = P[i].lr_age + 1 end
  end
  env.on_frame()
end
local function run(n) for _ = 1, n do frame() end end

env.on_match_start()
env.gd.command = noop
-- the Lab's console command is registered through gd.command: capture it
local cmdfn
gd.command = function(_, fn) cmdfn = fn end
chunk() -- reload to capture the command (fresh state)
env.on_match_start()
local function lab(s) logs = {} cmdfn(s) return table.concat(logs, "\n") end

local function expect(cond, what) print((cond and "PASS " or "FAIL ") .. what) if not cond then FAILED = true end end

-- 1. Hit exchange: P1 ftilt hits P2 on frame 7 of the move; hitlag 4; P2 hitstun 10 after hitlag;
--    P1's move ends at f26 (IASA f20). Victim actionable at hit+4+10 (hitstun over), attacker at IASA.
lab("mode training")
set(1, "AttackS3S") run(6)
-- the hit
now = now + 0
set(1, "AttackS3S", { in_hitlag = true }) set(2, "DamageN1", { in_hitlag = true, in_hitstun = true })
env.on_hit(1, 2, { dealt = 9 })
frame()
local hit_f = now
run(3)
P[1].in_hitlag, P[2].in_hitlag = false, false
run(10) -- hitstun
P[2].in_hitstun = false
local v_act = now + 1
-- attacker's IASA at action frame 20
while P[1].action_frame + 1 < 20 do frame() end
P[1].iasa = true
frame()
local a_act = now
frame()
local out = lab("adv")
print(out)
expect(out:find(string.format("%+d on hit", (hit_f + 14) - a_act), 1, true) ~= nil, "hit exchange advantage " .. string.format("%+d", (hit_f + 14) - a_act))
P[1].iasa = false set(1, "Wait") set(2, "Wait") run(3)

-- 2. Shield: P1 ftilt on P2's shield; both enter hitlag the same frame; P2 GuardSetOff 8 frames then Guard;
--    P1 IASA at 20.
set(2, "Guard") run(2)
set(1, "AttackS3S") run(6)
P[1].in_hitlag = true set(2, "GuardSetOff", { in_hitlag = true })
frame()
local s0 = now
run(3)
P[1].in_hitlag, P[2].in_hitlag = false, false
run(7)
set(2, "Guard") frame()
local v2 = now
while P[1].action_frame + 1 < 20 do frame() end
P[1].iasa = true frame()
local a2 = now
frame()
out = lab("adv")
print(out)
expect(out:find(string.format("%+d on shield", v2 - a2), 1, true) ~= nil, "shield advantage " .. string.format("%+d", v2 - a2))
P[1].iasa = false set(1, "Wait") run(2)

-- 3. L-cancels: pressed 3 f before landing (ok), 10 f before (early by 4), after landing (late by 2), none.
local function aerial_land(press_before, press_after)
  set(1, "AttackAirN", { airborne = true }) run(20)
  if press_before then
    P[1].lr_age = -1 -- becomes 0 on the next frame: the press frame
    run(press_before + 1)
  end
  set(1, "LandingAirN") frame()
  if press_after then run(press_after - 1) P[1].lr_age = -1 frame() end
  run(8)
  set(1, "Wait") run(3)
end
aerial_land(2)
aerial_land(9) -- age 10 at landing: 4 f early
aerial_land(nil, 2)
aerial_land(nil, nil)
out = lab("tech")
print(out)
expect(out:find("L-cancel: 3 f before landing", 1, true) ~= nil, "L-cancel ok")
expect(out:find("L-cancel: 4 f early", 1, true) ~= nil, "L-cancel early")
expect(out:find("L-cancel: 2 f late", 1, true) ~= nil, "L-cancel late")
expect(out:find("L-cancel: no press", 1, true) ~= nil, "L-cancel no press")

-- 4. Wavedash: jumpsquat 3, airdodge on the first airborne frame; then one 2 frames late; short hop check.
local function wavedash(late)
  set(1, "KneeBend") run(3)
  if late > 0 then
    set(1, "JumpF", { vy = 2.0 }) frame()
    run(late - 1)
  end
  set(1, "EscapeAir") frame()
  run(1)
  set(1, "LandingFallSpecial") frame()
  run(10) set(1, "Wait") run(2)
end
wavedash(0)
wavedash(2)
-- Waveland: fall, airdodge, land 4 frames later
set(1, "Fall") run(10) set(1, "EscapeAir") frame() run(3) set(1, "LandingFallSpecial") frame() run(10) set(1, "Wait") run(2)
-- Ledgedash: CliffWait, drop to Fall, JumpAerialF, EscapeAir, land with 5 intangible frames left
set(1, "CliffWait") run(10) set(1, "Fall") frame() set(1, "JumpAerialF") run(2) set(1, "EscapeAir") run(4)
set(1, "LandingFallSpecial", { intangible = 5 }) frame() P[1].intangible = 0 run(10) set(1, "Wait") run(2)
out = lab("tech")
print(out)
expect(out:find("Wavedash: frame-perfect airdodge", 1, true) ~= nil, "wavedash perfect")
expect(out:find("Wavedash: airdodge 2 f late", 1, true) ~= nil, "wavedash late")
expect(out:find("Hops: short hop (jumpsquat 3 f)", 1, true) ~= nil, "short hop")
expect(out:find("Waveland: landed on airdodge f4", 1, true) ~= nil, "waveland")
expect(out:find("Ledgedash: GALINT 5", 1, true) ~= nil, "ledgedash GALINT")

-- 5. The move card
set(1, "AttackS3S") run(3)
out = lab("card")
print(out)
expect(out:find("startup 5 active 5-8 total 26 IASA 20", 1, true) ~= nil, "move card (ftilt)")
set(1, "AttackAirN") run(2)
out = lab("card")
print(out)
expect(out:find("landing 15 L-cancel 7 autocancel 1-2 40-", 1, true) ~= nil, "move card (nair lag, autocancel)")

-- 6. Inputs: X on f1, R on f4
pad.buttons = 0x0400 frame() pad.buttons = 0 run(2) pad.buttons = 0x0020 frame() pad.buttons = 0 run(2)
-- 7. Draw everything once (catches nil arithmetic in the drawing code)
local ok, err = pcall(env.on_draw)
expect(ok, "on_draw in TRAINING: " .. tostring(err))
env.on_loadstate(0)
ok, err = pcall(env.on_draw)
expect(ok, "on_draw after a step back: " .. tostring(err))
local u = {}
for k in pairs(unknown) do u[#u + 1] = k end
print("gd functions stubbed as no-ops: " .. table.concat(u, " "))
if FAILED then os.exit(1) end
