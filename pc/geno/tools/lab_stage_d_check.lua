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
  color = function() return 0xFFFFFFFF end }, { __index = function() return noop end })

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

-- 5. The move card, measured like the export: frame 1 = the first frame after the change of state.
--    Fox ftilt: hitboxes on 5-8, the state lasts 26, no IASA inside it (the export's reading, in game).
local HB = { { id = 0, x = 0, y = 0, z = 0, px = 0, py = 0, radius = 2, damage = 9, angle = 45, kbg = 100, bkb = 5, wbk = 0,
  bone = 1, element_name = "normal" } }
local function perform(name, len, from, to, iasa, after)
  set(1, name) frame() -- the frame the state is entered
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
set(2, "DamageN1", { in_hitlag = true, in_hitstun = true })
env.on_hit(1, 2, { dealt = 9, damage = 9, angle = 45, kbg = 100, bkb = 10, wbk = 0 })
frame()
local seen = {}
for _ = 1, 5 do seen[#seen + 1] = inputs[2] frame() end
expect(seen[1] and seen[1].y == 80 and seen[1].x == 0, "SDI flick 1 up")
expect(seen[2] and (seen[2].x or 0) == 0 and (seen[2].y or 0) == 0, "SDI back to neutral")
expect(seen[3] and seen[3].y == 80, "SDI flick 2")
expect(seen[5] and seen[5].x == -57 and seen[5].y == 57, "DI in: the perpendicular toward the attacker (" ..
  tostring(seen[5] and seen[5].x) .. ", " .. tostring(seen[5] and seen[5].y) .. ")")
P[2].in_hitlag = false
run(3)
lab("dummy sdi_n 0")
-- the tech: falling in tumble at 2 units a frame, the floor at 0: the press comes 5 frames out
lab("dummy tech away")
set(2, "DamageFall", { airborne = true, in_hitstun = false, vy = -2, y = 30 })
floor_y = 0
local pressed_at
for _ = 1, 20 do
  P[2].y = P[2].y - 2
  frame()
  if inputs[2] and inputs[2] ~= "released" and ((inputs[2].buttons or 0) & 0x20) ~= 0 then pressed_at = pressed_at or P[2].y end
end
expect(pressed_at ~= nil and pressed_at <= 10 and pressed_at > 0, "tech press ~5 frames before the floor (y " .. tostring(pressed_at) .. ")")
expect(inputs[2] and inputs[2] ~= "released" and inputs[2].x == 80, "tech away: the stick held away (P1 is left of P2)")
set(2, "Passive", { airborne = false, vy = 0, y = 0 }) run(20) set(2, "Wait") run(3)
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

-- 7. Draw everything once (catches nil arithmetic in the drawing code)
local ok, err = pcall(env.on_draw)
expect(ok, "on_draw in TRAINING: " .. tostring(err))
local keep_now = now
now = 100 -- a load back to frame 100: the exchanges after it are from the abandoned timeline
env.on_loadstate(4)
out = lab("adv")
local later = 0
for f in out:gmatch("f(%d+) P") do if tonumber(f) > 100 then later = later + 1 end end
expect(later == 0 and out:find("f6 P1", 1, true) ~= nil, "a load drops the exchanges after its frame, keeps f6")
now = keep_now
env.on_loadstate(0)
ok, err = pcall(env.on_draw)
expect(ok, "on_draw after a step back: " .. tostring(err))
local u = {}
for k in pairs(unknown) do u[#u + 1] = k end
print("gd functions stubbed as no-ops: " .. table.concat(u, " "))
if FAILED then os.exit(1) end
