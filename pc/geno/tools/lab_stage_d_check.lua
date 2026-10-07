-- Stage D (TRAINING) logic check, off the game: a stub gd plays scripted frames through lab.lua and
-- checks D1 (frame advantage, tech feedback, the move card), D3 (the dummy: DI, SDI, tech, ledge,
-- out of shield, recording and playback), D4 (true combos) and D5 (drills, results).
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
    percent = 0, anim_id = 1, facing = 1, shield = 60, shield_on = false, kb_vy = 0, jumps_left = 1,
    body_state = "normal", invincible = 0, hitstun = 0, kb_last = 0 }
end
P[1], P[2] = mkp(1, 1), mkp(2, 2)
local now = 0
local pad = { buttons = 0, x = 0, y = 0, cx = 0, cy = 0, l = 0, r = 0 }
local logs = {}
local pad2 = { buttons = 0, x = 0, y = 0, cx = 0, cy = 0, l = 0, r = 0 }
local inputs = {}      -- inputs[port] = the last gd.input spec (per frame), or "released"
local input_log = {}   -- {frame, port, spec}
local mirror = nil
local floor_y = nil
local files = {}
local loads = {}

local K = setmetatable({ shear = 0.25, available = function() return true end, measure = function(s) return #s * 6 end,
  color = function() return 0xFFFFFFFF end, text = function(x, y, s) return #tostring(s) * 6 end,
  image = function(n, x, y, w) return w or 16 end }, { __index = function() return noop end })

local flying, fly_v = {}, 2.0 -- gd.fly / gd.fly_speed
local gd
gd = setmetatable({
  lab_api = 1, kit = K,
  draw = { DEFAULT = 1, MODEL = 2, HIT = 4, THROWN = 8 }, stage_draw = { COLL = 1, LEDGES = 4, TERRAIN = 2, POINTS = 8, ZONES = 16 },
  log = function(...) local t = {} for i = 1, select("#", ...) do t[#t + 1] = tostring(select(i, ...)) end logs[#logs + 1] = table.concat(t, " ") end,
  lab_request = function() return true end, data_read = function(n) return files[n] end,
  data_write = function(n, t) files[n] = t end,
  input = function(port, spec) inputs[port] = spec input_log[#input_log + 1] = { now, port, spec } end,
  release = function(port) inputs[port] = "released" end,
  floor_below = function() return floor_y end,
  kb_preview = function(v, t) return { kb = 10, level = 1, tumble = false, percent = 0, weight = 100, di = "none",
    vx = 1, vy = 1, angle = 45, angle_di = 45, hitstun = 20, points = { { 1, 1 } } } end,
  hurtboxes = function() return { { ax = 0, ay = 0, bx = 0, by = 5, radius = 2, state = "normal" } } end,
  set_shield = function(port, v) P[port].shield = v end, set_percent = function(port, v) P[port].percent = v end,
  fly = function(port, m) -- debug movement: nil reads; true / "on" / "toggle" / "place" / "off" / false
    if m == nil then return flying[port] == true end
    if m == "toggle" then flying[port] = not flying[port] elseif m == true or m == "on" then flying[port] = true else flying[port] = false end
    return flying[port] end,
  fly_speed = function(v) if v ~= nil then fly_v = v end return fly_v end,
  mirror_pad = function(a, b, take) mirror = a and { a, b, take } or nil end,
  savestate = noop, loadstate = function(s) loads[#loads + 1] = s end, state_load = function() return true end,
  lab_now = function() return "2026-09-24 12:00:00" end,
  buttons = { A = 0x100, B = 0x200, X = 0x400, Y = 0x800, START = 0x1000, UP = 8, DOWN = 4, LEFT = 1, RIGHT = 2, L = 0x40, R = 0x20, Z = 0x10 },
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
  pad = function(port) return port == 2 and pad2 or pad end, motion_name = function(id) return names[id] or ("M" .. id) end,
  history = function() return { busy = false, replaying = false, back = 0, fwd = 0, depth = 600, interval = 5, mb = 12.0 } end,
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
-- one game frame; `events` runs where the game dispatches its events (after the frame counter moves,
-- before on_frame), e.g. function() env.on_hit(1, 2, info) end
local function frame(events)
  inputs = {}
  now = now + 1
  for i = 1, 2 do
    P[i].action_frame = P[i].action_frame + 1
    P[i].anim_frame = P[i].anim_frame + 1
    if P[i].lr_age < 255 then P[i].lr_age = P[i].lr_age + 1 end
  end
  if events then events() end
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

-- 5. The move card, measured like the export: frame 1 = the frame the state starts (its change is seen).
--    Fox ftilt: hitboxes on 5-8, the state lasts 26, no IASA inside it (the export's reading, in game).
local HB = { { id = 0, x = 0, y = 0, z = 0, px = 0, py = 0, radius = 2, damage = 9, angle = 45, kbg = 100, bkb = 5, wbk = 0,
  bone = 1, element_name = "normal" } }
local function perform(name, len, from, to, iasa, after)
  set(1, name) -- frame 1 is the frame the change is seen
  for f = 1, len do
    P[1].hitboxes = (f >= from and f <= to) and HB or {}
    P[1].iasa = iasa ~= nil and f >= iasa
    frame()
  end
  P[1].hitboxes, P[1].iasa = {}, false
  set(1, after) frame()
end
perform("AttackS3S", 26, 5, 8, nil, "Wait")
out = lab("card")
print(out)
expect(out:find("startup 5 active 5-8 total 26 IASA nil", 1, true) ~= nil, "move card = the export's reading (ftilt 5-8, 26)")
-- nair, landed at 45 after its IASA at 42: the landing cuts it short, so no total from this run
perform("AttackAirN", 45, 4, 31, 42, "LandingAirN")
set(1, "Wait") frame()
P[1].hitboxes = {}
out = lab("card")
print(out)
expect(out:find("AttackAirN startup 4 active 4-31 total nil IASA 42 landing 15 L-cancel 7 autocancel 1-2 40-", 1, true) ~= nil,
  "move card (nair: landed, IASA 42, the lag and autocancel)")
-- the second time, the card has the numbers from the first frame on
set(1, "AttackS3S") frame() frame()
out = lab("card")
expect(out:find("startup 5 active 5-8 total 26", 1, true) ~= nil and not out:find("measuring", 1, true), "move card: kept for the next time")
set(1, "Wait") run(2)

-- 6. Inputs: X on f1, R on f4
pad.buttons = 0x0400 frame() pad.buttons = 0 run(2) pad.buttons = 0x0020 frame() pad.buttons = 0 run(2)
-- 8. D3: DI in on a hit launching up-right (the preview stub: vx = vy = 1), with 2 SDI flicks away
set(1, "Wait") set(2, "Wait") run(2)
lab("dummy port 2") lab("dummy on true") lab("dummy di in") lab("dummy sdi_n 2") lab("dummy sdi_dir up")
set(2, "DamageN1", { in_hitlag = true, in_hitstun = true, hitlag = 10 })
env.on_hit(1, 2, { dealt = 9, damage = 9, angle = 45, kbg = 100, bkb = 10, wbk = 0 })
frame()
local seen = {}
for _ = 1, 8 do seen[#seen + 1] = inputs[2] P[2].hitlag = P[2].hitlag - 1 frame() end
expect(seen[1] and seen[1].y == 80 and seen[1].x == 0, "SDI flick 1 up")
expect(seen[2] and (seen[2].x or 0) == 0 and (seen[2].y or 0) == 0, "SDI back to neutral")
expect(seen[3] and seen[3].y == 80, "SDI flick 2")
expect(seen[4] and seen[4].x == -25 and seen[4].y == 25, "clean DI step 1: each axis just past the 0.25 line (" ..
  tostring(seen[4] and seen[4].x) .. ", " .. tostring(seen[4] and seen[4].y) .. ")")
expect(seen[6] and seen[6].x == -25 and seen[7] and seen[7].x == -57 and seen[7].y == 57,
  "clean DI step 2 after the SDI window: the full perpendicular toward the attacker (" ..
  tostring(seen[7] and seen[7].x) .. ", " .. tostring(seen[7] and seen[7].y) .. ")")
P[2].in_hitlag = false
run(3)
lab("dummy sdi_n 0")
-- the full DI stick is written while 2 hitlag frames are left (it reaches the game a frame later,
-- on hitlag's last frame, when DI is read); with 1 left it is still the full stick
set(2, "DamageN1", { in_hitlag = true, in_hitstun = true, hitlag = 4 })
frame(function() env.on_hit(1, 2, { dealt = 9, damage = 9, angle = 45, kbg = 100, bkb = 10, wbk = 0 }) end)
P[2].hitlag = 3 frame()
P[2].hitlag = 2 frame()
local at2 = inputs[2]
P[2].hitlag = 1 frame()
local at1 = inputs[2]
expect(at2 and at2 ~= "released" and at2.x == -57 and at1 and at1.x == -57, "full DI out with 2 hitlag frames left ("
  .. tostring(at2 and at2.x) .. ", " .. tostring(at1 and at1.x) .. ")")
P[2].in_hitlag, P[2].hitlag = false, 0
run(3)
-- a reload replaying the same frame with another DI must not reuse the old one (it was cached by frame)
set(2, "DamageN1", { in_hitlag = true, in_hitstun = true, hitlag = 10 })
local f_hit = now + 1
frame(function() env.on_hit(1, 2, { dealt = 9, damage = 9, angle = 45, kbg = 100, bkb = 10, wbk = 0 }) end)
now = f_hit - 1
env.on_loadstate(4)
lab("dummy di out")
frame(function() env.on_hit(1, 2, { dealt = 9, damage = 9, angle = 45, kbg = 100, bkb = 10, wbk = 0 }) end)
for _ = 1, 4 do P[2].hitlag = P[2].hitlag - 1 frame() end
expect(inputs[2] and inputs[2] ~= "released" and inputs[2].x == 57 and inputs[2].y == -57,
  "after a reload the same frame's hit takes the new DI (out: " .. tostring(inputs[2] and inputs[2].x) .. ")")
P[2].in_hitlag, P[2].hitlag = false, 0
lab("dummy di in")
run(3)
-- the tech: falling in tumble at 2 units a frame, the floor at 0: the press comes 5 frames out
lab("dummy tech away")
set(2, "DamageFall", { airborne = true, in_hitstun = false, vy = -2, y = 30 })
floor_y = 0
local pressed_at, early_x = nil, 0
for k = 1, 20 do
  P[2].y = P[2].y - 2
  frame()
  if k <= 3 and inputs[2] and inputs[2] ~= "released" then early_x = early_x + math.abs(inputs[2].x or 0) end
  if inputs[2] and inputs[2] ~= "released" and ((inputs[2].buttons or 0) & 0x20) ~= 0 then pressed_at = pressed_at or P[2].y end
end
expect(pressed_at ~= nil and pressed_at <= 10 and pressed_at > 0, "tech press ~5 frames before the floor (y " .. tostring(pressed_at) .. ")")
expect(inputs[2] and inputs[2] ~= "released" and inputs[2].x == 80,
  "tech away: the full stick away at the press (" .. tostring(inputs[2] and inputs[2].x) .. ")")
expect(early_x == 0, "tech away: no drift early in the fall (x " .. tostring(early_x) .. ")")
set(2, "Passive", { airborne = false, vy = 0, y = 0 }) run(20) set(2, "Wait") run(3)
-- the tech near a floor's end (the floor ends at x = 68 here), drifting out, "away" pointing off
-- it: the stick stays under tumble_wiggle (no wiggle to Fall), steers back in, and the tech is in
-- place or the inward roll, never a roll off the edge
local keep_floor = gd.floor_below
gd.floor_below = function(x) if x < 68 then return floor_y end return nil end
set(2, "DamageFall", { airborne = true, in_hitstun = false, vy = -2, y = 30, x = 60, vx = 0.5 })
local wig = 56 -- tumble_wiggle 0.7 (the stub has none) * 80
local max_x, min_x, press_x = 0, 0, nil
for _ = 1, 20 do
  P[2].y = P[2].y - 2
  frame()
  local s = inputs[2]
  if s and s ~= "released" then
    max_x, min_x = math.max(max_x, s.x or 0), math.min(min_x, s.x or 0)
    if ((s.buttons or 0) & 0x20) ~= 0 and press_x == nil then press_x = s.x or 0 end
  end
end
expect(press_x ~= nil and press_x <= 0, "tech away at an edge: pressed, in place or inward (x " .. tostring(press_x) .. ")")
expect(max_x <= 0, "tech away at an edge: the stick never points off the edge (max x " .. max_x .. ")")
expect(min_x > -wig, "tech at an edge: steering under the wiggle threshold (min x " .. min_x .. ")")
set(2, "Passive", { airborne = false, vy = 0, y = 0, vx = 0 }) run(20) set(2, "Wait") run(3)
-- no floor under the drift at all: steer toward the stage, under the wiggle threshold
floor_y = nil
set(2, "DamageFall", { airborne = true, in_hitstun = false, vy = -2, y = 60, x = 80, vx = 0.5 })
local steer_seen
for _ = 1, 3 do P[2].y = P[2].y - 2 frame() local s = inputs[2] if s and s ~= "released" then steer_seen = s.x end end
expect(steer_seen and steer_seen < 0 and steer_seen > -wig, "no floor: steer toward the stage under the wiggle ("
  .. tostring(steer_seen) .. ")")
floor_y = 0
gd.floor_below = keep_floor
set(2, "Wait", { airborne = false, vy = 0, y = 0, vx = 0, x = 20 }) run(3)
-- the kb check's ASDI nudge: the stick as the game's float (x4BC = 3: about 3 units), not 80x that
local keep_common, keep_prev = gd.lab_common, gd.kb_preview
gd.lab_common = function() return { lcancel_window = 7, lcancel_div = 2, asdi_scale = 3 } end
local x_from
gd.kb_preview = function(v, t) if t.x then x_from = t.x end return keep_prev(v, t) end
lab("dummy di in")
set(2, "DamageN1", { in_hitlag = true, in_hitstun = true, hitlag = 4, x = 20 })
frame(function() env.on_hit(1, 2, { dealt = 9, damage = 9, angle = 45, kbg = 100, bkb = 10, wbk = 0 }) end)
expect(x_from and math.abs(x_from - (20 - 3 * 0.7071)) < 0.05, "kb check: ASDI nudge is stick float * 3 (x from "
  .. tostring(x_from) .. ", want 17.88)")
gd.lab_common, gd.kb_preview = keep_common, keep_prev
P[2].in_hitlag, P[2].hitlag = false, 0
set(2, "Wait", { in_hitstun = false }) run(3)
-- the ledge: jump off it after a 3-frame reaction
lab("dummy ledge jump") lab("dummy delay_min 3") lab("dummy delay_max 3")
set(2, "CliffWait") frame()
local jumped
for k = 1, 6 do
  if inputs[2] and inputs[2] ~= "released" and ((inputs[2].buttons or 0) & 0x400) ~= 0 then jumped = k end
  frame()
end
expect(jumped == 4, "ledge jump after the 3-frame reaction (frame " .. tostring(jumped) .. ")")
lab("dummy delay_min 0") lab("dummy delay_max 0")
set(2, "Wait") run(2)
-- out of shield: spotdodge the frame shieldstun ends
lab("dummy after_shield spotdodge")
set(2, "GuardSetOff") run(3) set(2, "Guard") frame()
local sd = inputs[2]
expect(sd and sd ~= "released" and ((sd.buttons or 0) & 0x20) ~= 0 and sd.y == -80, "after shieldstun: spotdodge (R + down)")
set(2, "Wait") run(3)
-- recording: P1's controller drives P2 (take), 3 frames recorded, then played back in order
env.gd.key = function() return false end
lab("dummy after_shield none")
local rec_on = function() for _, a in ipairs({ "dm_record" }) do end end
lab("port 1")
cmdfn("mode training")
-- the TRAINING mode's R action
local function action(k) env.gd.key_pressed = function(x) return x == k end env.on_tick() env.gd.key_pressed = function() return false end frame() end
action("R")
expect(mirror and mirror[1] == 1 and mirror[2] == 2 and mirror[3] == true, "recording: P1 drives P2, P1 left neutral")
pad2.buttons = 0x0100 frame() pad2.buttons = 0 pad2.x = 80 frame() pad2.x = 0 frame()
action("R")
expect(mirror == nil, "recording stopped")
local d = files["dummy_rec1.txt"] or ""
expect(select(2, d:gsub("\n", "")) >= 4, "slot 1 saved with its frames")
lab("dummy play in order")
local got = {}
for _ = 1, 6 do frame() got[#got + 1] = inputs[2] end
local okA = false
for _, g in ipairs(got) do if g ~= "released" and g and ((g.buttons or 0) & 0x100) ~= 0 then okA = true end end
expect(okA, "playback presses the recorded A")
lab("dummy play off")
run(2)

-- 9. D4: two hits while P2 is still in hitstun = TRUE; a third after P2 could act for 3 frames = escapable 3
lab("combo")
set(2, "DamageN1", { in_hitstun = true, in_hitlag = false })
local function hit5() frame(function() env.on_hit(1, 2, { dealt = 5 }) end) end
hit5() run(5)
hit5() run(5)
P[2].in_hitstun = false set(2, "Wait") run(3) -- actionable on these 3 frames
hit5()
out = lab("combo")
print(out)
expect(out:find("2 f%d+ .- TRUE") ~= nil or out:find("TRUE", 1, true) ~= nil, "combo: the second hit is true")
expect(out:find("escapable 3 f", 1, true) ~= nil, "combo: the third was escapable by 3 frames")
set(2, "Wait") run(40)

-- throw > uair: the throw is a hit when P2 is let go; a hit on P2 lying in DownWait is escapable
lab("combo")
set(2, "Wait") run(40)
set(1, "Catch") set(2, "CapturePulledHi") P[2].percent = 8 run(4)
set(2, "CaptureWaitHi") run(4)
set(1, "ThrowHi") set(2, "ThrownHi") P[2].percent = 10 run(8)
-- let go at 12%; the rest of the throw lands a frame later (17%): the throw is 17 - 8 = 9% from the grab
P[2].percent = 10 set(2, "DamageFlyHi", { in_hitstun = true }) set(1, "Wait") frame()
run(9) P[2].percent = 12 frame() P[2].percent = 14 frame() P[2].percent = 17 frame() -- the lasers, f10-12
run(16) -- quiet: the throw closes at 17 - 8 = 9%
run(10)
frame(function() env.on_hit(1, 2, { dealt = 13 }) end)
out = lab("combo")
print(out)
expect(out:find("1 f%d+ ThrowHi 9.0%% opener") ~= nil and out:find("2 f%d+ %S+ 13.0%% TRUE") ~= nil,
  "combo: grab > upthrow counted whole (9%, landing after the let-go too) > the next hit, true")
P[2].in_hitstun = false set(2, "DownWaitU") run(5)
frame(function() env.on_hit(1, 2, { dealt = 3 }) end)
out = lab("combo")
expect(out:find("escapable 5 f", 1, true) ~= nil, "combo: a hit on a downed P2 that could get up is escapable")
-- a lone throw is listed (before, "combo: none yet" followed a throw whose follow-up missed)
set(2, "Wait") run(40)
set(1, "Catch") set(2, "CapturePulledHi") P[2].percent = 20 run(4)
set(1, "ThrowHi") set(2, "ThrownHi") run(8)
P[2].percent = 27 set(2, "DamageFlyHi", { in_hitstun = true }) set(1, "Wait") frame()
run(20) P[2].in_hitstun = false set(2, "Wait") run(40)
out = lab("combo")
expect(out:find("1 hits 7.0%", 1, true) ~= nil, "combo: a lone throw is listed")
set(2, "Wait") run(40)

-- 10. D5: the L-cancel drill (streak), then its results line
lab("drill lcancel")
aerial_land(2) aerial_land(2) aerial_land(9) aerial_land(2)
out = lab("drill")
expect(out:find("running: L-cancel streak  3 / 4  streak 1", 1, true) ~= nil, "L-cancel drill: 3 of 4, streak back to 1")
lab("drill stop")
expect((files["drills/results.txt"] or ""):find("lcancel score 2 n 4 ok 3 streak 2", 1, true) ~= nil, "drill result saved (best streak 2)")
-- the tech chase: the dummy techs in place, P1 hits it 12 frames later
lab("drill techchase")
set(2, "Passive") run(11)
frame(function() env.on_hit(1, 2, { dealt = 5 }) end)
out = lab("drill")
expect(out:find("1 / 1", 1, true) ~= nil, "tech chase: a hit within the window counts")
lab("drill stop")
set(2, "Wait") run(3)
cmdfn("mode hitboxes")
P[1].hitboxes = { { id = 0, x = 5, y = 5, z = 0, px = 3, py = 5, radius = 2, damage = 9, angle = 45, kbg = 100, bkb = 5, wbk = 0, bone = 1, element_name = "normal" },
  { id = 1, x = 6, y = 6, z = 0, px = 6, py = 6, radius = 2, damage = 0, angle = 0, kbg = 0, bkb = 0, wbk = 0, bone = 1, element_name = "catch" } }
P[2].shield_on, P[2].shield_x, P[2].shield_y, P[2].shield_r = true, 20, 5, 10
run(2)

flying[1] = true -- the PLAY tab's fly rows read gd.fly / gd.fly_speed while it draws
-- the pause menu with a mouse: gd.mouse() -> x, y, buttons, wheel (four numbers, not a table)
env.gd.mouse = function() return 100, 150, 1, -1 end
env.gd.key_pressed = function(k) return k == "ESCAPE" end
local okm, errm = pcall(env.on_tick)
env.gd.key_pressed = function() return false end
for _ = 1, 3 do if okm then okm, errm = pcall(env.on_tick) end end
if okm then okm, errm = pcall(env.on_draw) end
expect(okm, "pause menu with gd.mouse's four numbers: tick and draw (" .. tostring(errm) .. ")")
-- Kit widescreen regression: exercise the actual LAB drawing at each canvas
-- width, without adding locals to lab.lua's already full top-level scope.
do
  local image, text = K.image, K.text
  local drawn, right_x
  K.image = function(name, x, y, w, h, ...)
    if name == "lab_solid" and x == 0 then drawn[y] = {w = w, h = h} end
    return image(name, x, y, w, h, ...)
  end
  K.text = function(x, y, s, ...)
    if s == "keys: arrows  ENTER  Q/E  ESC" then right_x = x end
    return text(x, y, s, ...)
  end
  for _, width in ipairs({640, 480 * 16 / 9, 480 * 21 / 9}) do
    env.gd.safe_area = function() return {x = 0, y = 0, w = width, h = 480, right = width, bottom = 480} end
    drawn, right_x = {}, nil
    local ok, err = pcall(env.on_draw)
    expect(ok, "wide LAB draw: " .. tostring(err))
    expect(drawn[0] and drawn[0].w == width and drawn[0].h == 480, "LAB dim fills " .. width)
    expect(drawn[446] and drawn[446].w == width and drawn[448] and drawn[448].w == width,
           "LAB footer fills " .. width)
    expect(right_x == width - 20, "LAB footer text anchors at right " .. width)
  end
  K.image, K.text = image, text
  env.gd.safe_area = nil
end
env.gd.key_pressed = function(k) return k == "ESCAPE" end
pcall(env.on_tick)
env.gd.key_pressed = function() return false end
env.gd.mouse = function() return -1000, -1000, 0, 0 end

do
  local image, drawn = K.image, nil
  env.gd.safe_area = function() return {w = 1120, right = 1120} end
  K.image = function(name, x, y, w, h, ...)
    if name == "lab_solid" and x == 0 and y == 452 then drawn = w end
    return image(name, x, y, w, h, ...)
  end
  local ok, err = pcall(env.on_draw)
  expect(ok and drawn == 1120, "LAB mode strip fills ultrawide: " .. tostring(err))
  K.image, env.gd.safe_area = image, nil
end

-- 7. Draw everything once (catches nil arithmetic in the drawing code)
local ok, err = pcall(env.on_draw)
expect(ok, "on_draw in TRAINING: " .. tostring(err))
local keep_now = now
now = 100 -- a load back to frame 100: the exchanges after it are from the abandoned timeline
env.on_loadstate(4)
out = lab("adv")
local later = 0
for f in out:gmatch("f(%d+) P") do if tonumber(f) > 100 then later = later + 1 end end
expect(later == 0, "a load drops the exchanges after its frame")
now = keep_now
env.on_loadstate(0)
ok, err = pcall(env.on_draw)
expect(ok, "on_draw after a step back: " .. tostring(err))
-- Exercise the real panel target selector and drawing with six fighters.
do
  local menu
  for i=1,100 do
    local name,value=debug.getupvalue(cmdfn,i)
    if not name then break end
    if name=="menu" then menu=value break end
  end
  local old_players=gd.players
  for i=3,6 do P[i]=mkp(i,1) end
  gd.players=function() local t={} for i=1,6 do if P[i] then t[#t+1]=P[i] end end return t end
  expect(menu and menu.next_fighter(4,1)==5 and menu.next_fighter(5,1)==6 and
         menu.next_fighter(6,1)==1 and menu.next_fighter(1,-1)==6,
         "LAB targets cycle all six fighter slots")
  P[5]=nil
  expect(menu and menu.next_fighter(4,1)==6, "LAB target selector skips empty fifth slot")
  P[5]=mkp(5,1)
  cmdfn("menu dummy")
  local ok,err=pcall(env.on_draw)
  expect(ok,"six-fighter LAB panel draws: "..tostring(err))
  cmdfn("menu close")
  expect(inputs[5]==0 and inputs[6]==0,"LAB close neutralizes virtual fighter inputs")
  gd.players=old_players
  for i=3,6 do P[i]=nil end
end

local u = {}
-- Geno events retain decoded operands in the console instead of printing only "geno".
do
  local old = gd.timeline
  gd.timeline = function()
    return { motion_name = "TutorialB", anim_name = "SpecialN", length = 9, stop = "end", conditional = true,
      events = { { frame = 2, name = "geno.PUT", sub = 9, detail = "value=3 operand=1073741824" },
        { frame = 5, name = "hitbox", id = 0, bone = 2, damage = 8, angle = 361, kbg = 100,
          bkb = 0, wbk = 0, size = 3, element_name = "normal" }, { frame = 9, name = "iasa" } } }
  end
  local out = lab("events")
  expect(out:find("geno.PUT value=3 operand=1073741824", 1, true) ~= nil,
    "geno_escape_events: decoded escape name and operands reach lab events")
  expect(out:find("hitbox", 1, true) and out:find("iasa", 1, true),
    "geno_overlay_events: hitbox and IASA events survive alongside escapes")
  expect(out:find("conditional script", 1, true), "geno_conditional_events: static path is labelled")
  local st -- LE is local: find it through the console closure below
  for i = 1, 100 do
    local name, value = debug.getupvalue(cmdfn, i)
    if name == "LE" then st = value.move_static(1, 44) break end
    if not name then break end
  end
  expect(type(st) == "table" and st.conditional and st.iasa == nil and st.ac == nil,
    "geno_conditional_static: conditional IASA/autocancel cannot masquerade as measured values")
  gd.timeline = old
end
-- ---- the Atlas pause menu (docs/superpowers/plans/2026-10-06-atlas-step7-mods-and-lab.md) --------------------------------------
-- Runs the LAB against the real gd.ui contract (atlas_ui_stub.lua) owned by the script geno-lab, never the console, which bypasses ownership.
do
  local here = (arg and arg[0] or ""):gsub("\\", "/"):gsub("[^/]*$", "")
  local Stub = dofile(here .. "../../tests/atlas_ui_stub.lua")
  local ui = Stub.new{ caller = "geno-lab", owner_mod = "geno-lab", available = true }
  local SCREEN = "geno-lab.pause"
  local function upv(fn, name)
    for i = 1, 250 do local n, v = debug.getupvalue(fn, i) if not n then return nil end if n == name then return v end end
  end
  gd.ui = ui
  chunk()                                   -- a fresh LAB, loaded after gd.ui exists (the mapper reads it once, at load)
  env.on_match_start()
  local menu, TABS = upv(cmdfn, "menu"), upv(cmdfn, "TABS")
  local resumed, left = 0, nil
  gd.resume = function() resumed = resumed + 1 end
  gd.lab_leave = function(where) left = where end
  local function press(kind) return ui.engine_press(SCREEN, kind) end
  local function desc() return ui.screens[SCREEN] end
  local function rows() return desc().primary.items end
  local function row(label) for _, r in ipairs(rows()) do if r.label == label then return r end end end
  local function focus_on(label) local r = row(label); assert(r, "no row " .. label); ui.engine_focus(SCREEN, "list", r.id) end
  local function closed() return ui.state().depth == 0 end

  -- 1. off by default: the legacy menu opens and no Atlas screen exists
  lab("menu")
  expect(menu.open and closed() and desc() == nil, "lab ui is off by default: the legacy menu opens and registers no screen")
  lab("menu close")
  -- 2. on: the pause menu is the Atlas screen, owned by the script, over the world, with every tab
  lab("ui on"); lab("menu")
  expect(ui.state().top == SCREEN and menu.open, "lab ui on: the pause menu is the Atlas screen")
  expect(ui._owner[SCREEN] == "geno-lab", "the screen is owned by the script geno-lab, not the console")
  local names, want = {}, {}
  for i, t in ipairs(desc().tabs) do names[i] = t.name end
  for i, t in ipairs(TABS) do want[i] = t.name end
  expect(table.concat(names, ",") == table.concat(want, ",") and want[1] == "PLAY" and want[#want] == "EXIT", "every tab is there: " .. table.concat(names, ","))
  expect(desc().backdrop == "world" and desc().chapter == 1 and desc().trail.title == "PLAY" and desc().trail[1] == "LAB", "over the world, chapter I, LAB > PAUSE > PLAY")
  expect(not pcall(ui.screen, { id = "lab.pause", primary = { kind = "list", items = { { id = "a", label = "A" } } } }), "a screen id without the mod's prefix is refused for the script")
  -- 3. every row of every tab maps (count, unique ids, no row over the record's cap)
  for i, t in ipairs(TABS) do
    cmdfn("menu " .. t.name:lower())
    local legacy = #(type(t.items) == "function" and t.items() or t.items)
    local seen, ok = {}, true
    for _, r in ipairs(rows()) do if seen[r.id] or #r.id > 23 then ok = false end seen[r.id] = true end
    expect(ok and #rows() == math.min(legacy, 32) and #rows() >= 1, t.name .. ": " .. #rows() .. " rows for " .. legacy)
    expect(ui.tab(SCREEN) == i, t.name .. ": the console command moved the engine's tab too")
  end
  -- 4. a toggle flips once (run and adjust both flip it: calling both would undo it)
  cmdfn("menu display")
  local tr
  for _, r in ipairs(rows()) do if r.value and r.value.kind == "toggle" then tr = r break end end
  expect(tr ~= nil, "the DISPLAY tab has toggle rows")
  local before = tr.value.on
  focus_on(tr.label); press("accept")
  expect(row(tr.label).value.on ~= before, "a toggle flips on A")
  press("accept")
  expect(row(tr.label).value.on == before, "and flips back on the next A: once each, not twice")
  ui.engine_row(SCREEN, "right")
  expect(row(tr.label).value.on ~= before, "left and right flip a toggle once as well")
  ui.engine_row(SCREEN, "left")
  -- 5. a stepper: left and right change it, A runs it
  cmdfn("menu play")
  focus_on("Focus")
  local function focus_now() return tonumber(lab("status"):match("focus=(%d+)")) end
  local f0 = focus_now()
  ui.engine_row(SCREEN, "right")
  local f1 = focus_now()
  press("accept")
  local f2 = focus_now()
  expect(f1 ~= f0 and f2 ~= f1, "a stepper: right changes it, A runs it (focus " .. f0 .. ", " .. f1 .. ", " .. f2 .. ")")
  expect(row("Focus").value.kind == "stepper", "the Focus row is a stepper")
  expect(row("Step +10").value == nil and row("Step +1").value.kind == "text", "a row with only a value is a text value, one with neither is plain")
  -- 6. the focus stays on the row after every re-registration, and a tab change restores the tab's last row
  focus_on("Step +10"); press("accept")
  expect(ui.focus(SCREEN) == row("Step +10").id, "the focus stays on the same row after the screen is re-registered")
  press("r"); expect(menu.tab == 2 and desc().trail.title == "DISPLAY", "R: the next tab, the trail follows")
  press("l"); expect(menu.tab == 1 and ui.focus(SCREEN) == row("Step +10").id, "L: back, and the tab's last row has the focus again")
  -- 7. closing: the game resumes once, every port's input is neutralised, the screen is gone
  resumed = 0
  focus_on("Resume")
  local okc, errc = pcall(press, "accept")
  expect(okc and closed() and not menu.open, "Resume closes the menu and the screen (" .. tostring(errc) .. ")")
  expect(resumed == 1 and inputs[5] == 0 and inputs[6] == 0, "the game resumes once and all six ports' inputs are neutralised")
  gd.paused = function() return true end
  lab("menu"); resumed = 0
  press("back")
  expect(closed() and not menu.open and resumed == 0, "opened over a paused game: B closes the menu and leaves the game paused")
  press("back")                                                    -- a stray second B on a closed screen is harmless
  gd.paused = function() return false end
  -- 8. leaving the match closes the screen too
  lab("menu"); cmdfn("menu exit"); focus_on("Quit"); left = nil
  press("accept")
  expect(closed() and not menu.open and left == "menu", "EXIT > Quit leaves the match and closes the screen")
  -- 9. the explainer: WHAT is the row's description, and a mode row carries its keys as tags
  lab("menu"); cmdfn("menu display"); focus_on("Display mode")
  local ex = desc().explainer.provide(row("Display mode").id)
  expect(ex and ex.title == "Display mode" and #(ex.with or {}) >= 1 and ex.from.text == "Geno LAB" and ex.well == false, "the mode row explains itself, with no picture well, and lists the mode's keys")
  lab("menu close")
  -- 10. the descriptions fit the explainer (the spec's one short rule: at most 110 characters; the engine cuts at 159)
  lab("menu")
  local long = {}
  for i, t in ipairs(TABS) do
    cmdfn("menu " .. t.name:lower())
    for _, r in ipairs(rows()) do
      local okp, e = pcall(desc().explainer.provide, r.id)
      if not okp then long[#long + 1] = t.name .. " / " .. r.label .. " (raised: " .. tostring(e) .. ")"
      elseif e and #e.what > 110 then long[#long + 1] = t.name .. " / " .. r.label .. " (" .. #e.what .. ")" end
    end
  end
  expect(#long == 0, "every row description is 110 characters or less; too long: " .. table.concat(long, "; "))
  lab("menu close"); lab("ui off")

  -- ---- Task 8: the library, other mods' tools, lifetime, ownership, online ----
  lab("ui on")
  menu = upv(cmdfn, "menu")

  -- 1. the saved-state library is paged: 60 states do not fit a 32-row record
  local gen, lib_rows = 1, {}
  for i = 1, 60 do lib_rows[i] = { name = "State " .. i, file = "s" .. i, saved = "today", what = "Fox v Falco", ok = true, frame = i } end
  gd.state_gen = function() return gen end
  gd.state_list = function() return lib_rows end
  gd.state_delete = function(file)
    for i, r in ipairs(lib_rows) do if r.file == file then table.remove(lib_rows, i) gen = gen + 1 return true end end
    return false, "no such state"
  end
  lab("menu"); cmdfn("menu states")
  expect(#rows() <= 32 and row("Library page") ~= nil and row("Library page").value.text == "1 / 3", "STATES with 60 saved states: at most 32 rows and a page row reading 1 / 3")
  focus_on("Library page"); ui.engine_row(SCREEN, "right")
  expect(row("Library page").value.text == "2 / 3" and row("State 26") ~= nil and row("State 1") == nil, "right turns the page: the second page starts at state 26")
  ui.engine_row(SCREEN, "left"); ui.engine_row(SCREEN, "left")
  expect(row("Library page").value.text == "3 / 3", "and wraps round to the last page")
  ui.engine_row(SCREEN, "right")
  -- 2. delete: Y asks in a dialog, B keeps, A deletes, and the focus lands on a neighbour (never on nothing)
  focus_on("State 3")
  ui.engine_press(SCREEN, "y")
  expect(#ui.dialogs == 1 and ui.dialogs[1].actions[1][2] == "Delete", "Y on a saved state asks first, in a dialog")
  ui.dialogs[1].on("B")
  expect(#lib_rows == 60 and row("State 3") ~= nil, "B (Keep) deletes nothing")
  ui.engine_press(SCREEN, "y")
  ui.dialogs[#ui.dialogs].on("A")
  expect(#lib_rows == 59 and row("State 3") == nil, "A (Delete) removes it from the library")
  expect(ui.focus(SCREEN) ~= nil and (ui.focus(SCREEN) == row("State 4").id or ui.focus(SCREEN) == row("State 2").id), "the focus is on a neighbour after the delete")
  expect(ui.state().top == SCREEN, "the screen is still open")
  focus_on("Quick save"); local nd = #ui.dialogs; ui.engine_press(SCREEN, "y")
  expect(#ui.dialogs == nd, "Y on a row with no delete asks nothing")
  lab("menu close")

  -- 2b. an empty library is just the fixed rows
  lib_rows = {}; gen = gen + 1
  lab("menu"); cmdfn("menu states")
  expect(#rows() >= 4 and row("Library page") == nil, "no saved states: no page row")
  lab("menu close")

  -- 3. other mods' tools: entries under lab.pause appear as a tab, activating one is the registry's act
  lab("menu"); local n_tabs0 = #desc().tabs; lab("menu close")
  ui.entries["tools.extra"] = { id = "tools.extra", parent = "lab.pause", label = "Extra tool", blurb = "Does a thing.", mod = "tools", opens = "tools.screen", visible = true, badge = "" }
  lab("menu")
  local has_mods = false
  for _, t in ipairs(desc().tabs) do if t.name == "MODS" then has_mods = true end end
  expect(has_mods and #desc().tabs == n_tabs0 + 1, "an entry under lab.pause adds a MODS tab")
  cmdfn("menu mods"); focus_on("Extra tool"); press("accept")
  expect(ui.activated == "tools.extra", "A on the entry activates it through the registry")
  local e = desc().explainer.provide(row("Extra tool").id)
  expect(e and e.from.text == "tools" and e.what == "Does a thing.", "the explainer's FROM names the mod that added it")
  lab("menu close")
  lab("menu"); lab("menu close")
  local mods_tabs = 0
  for _, t in ipairs(TABS) do if t.name == "MODS" then mods_tabs = mods_tabs + 1 end end
  expect(mods_tabs == 1, "the MODS tab is added once however often the menu opens")

  -- 4. ownership: the screen belongs to geno-lab; another script cannot touch it; the console bypass is not what the tests ran as
  lab("menu")
  expect(ui._owner[SCREEN] == "geno-lab" and ui.caller == "geno-lab", "these checks ran as the script geno-lab (not as the developer console)")
  lab("menu close")

  -- 5. the Atlas menu falls back when gd.ui is not available (fonts missing): the legacy menu opens instead
  ui.available_ok = false
  lab("menu")
  expect(menu.open and closed(), "gd.ui not available: the legacy menu opens and no screen is registered")
  lab("menu close"); ui.available_ok = true

  -- 6. offline only: online the menu does not open at all, and the Atlas path refuses even when called directly
  local real = gd.match
  gd.match = function() return { active = true, frame = now, netplay = true } end
  lab("menu")
  expect(not menu.open and closed(), "online: the LAB menu does not open")
  menu.ui.open(1)
  expect(closed(), "online: the Atlas path refuses to open")
  gd.match = real

  -- 7. a screen left open when the match ends is closed by on_match_start (a new match, a hot reload of the script)
  lab("menu")
  expect(ui.state().top == SCREEN, "open again")
  env.on_match_start()
  expect(closed() and not menu.open, "a new match closes a screen left open")
  -- 8. the script reloaded while a screen is registered: the new instance opens its own screen cleanly
  lab("menu"); chunk(); env.on_match_start()                        -- the reload re-registers the console command through gd.command
  lab("ui on"); lab("menu")
  expect(ui.state().top == SCREEN and #rows() >= 1, "after a reload the screen opens again from the new instance")
  lab("menu close"); lab("ui off")
-- (Task 9 adds its checks above this closing end: one block, one preamble, no new file-scope local)
end
for k in pairs(unknown) do u[#u + 1] = k end
print("gd functions stubbed as no-ops: " .. table.concat(u, " "))
if FAILED then os.exit(1) end
