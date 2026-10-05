-- @name: Turbo zero-to-death combo
-- @gameplay: true
-- One feature: a scripted, frame-exact 0-to-KO combo, written through the CPU virtual controller
-- (gd.cpu_mode "script" / gd.cpu_pad) on a match with the Turbo rule on.
-- Launch: MELEE_SCENE="mode=lab;stage=fd;turbo=on;p1=fox/cpu0;p2=falco/cpu0/idle"
-- P1 Fox is the attacker (CPU, script mode), P2 Falco is the victim (idle CPU: no DI, no tech).
-- Everything is an open-loop table of pad samples, one per logic frame, counted from the frame the
-- fighters have stood still for 25 frames after being placed. Found by an in-game search from
-- savestates (see README.md); each row is {stick x, stick y, c-stick x, c-stick y, buttons}.
local ATT, VIC = 1, 2
local SCENE = "mode=lab;stage=fd;turbo=on;p1=fox/cpu0;p2=falco/cpu0/idle"
-- Two recorded combos (see README). "waveshine": starts at the left edge and carries Falco across the
-- stage with shine / wavedash / shine ... into a down tilt kill (10 links). "classic": 6 links in the middle.
local VARIANTS = {
  waveshine = {ax = -72, vx = -63, pad = {
    [1] = {0, -127, 0, 0, 0x200},
    [6] = {0, 0, 0, 0, 0x100},
    [16] = {0, -127, 0, 0, 0x200},
    [20] = {0, 0, 0, 0, 0x400},
    [23] = {127, -46, 0, 0, 0x20},
    [33] = {0, -127, 0, 0, 0x200},
    [37] = {0, 0, 0, 0, 0x400},
    [40] = {127, -46, 0, 0, 0x20},
    [50] = {0, -127, 0, 0, 0x200},
    [54] = {0, 0, 0, 0, 0x400},
    [57] = {127, -46, 0, 0, 0x20},
    [67] = {0, -127, 0, 0, 0x200},
    [72] = {0, 0, 0, 0, 0x400},
    [75] = {127, -46, 0, 0, 0x20},
    [85] = {0, -127, 0, 0, 0x200},
    [90] = {0, 0, 0, 0, 0x400},
    [93] = {127, -46, 0, 0, 0x20},
    [103] = {0, -127, 0, 0, 0x200},
    [107] = {0, 0, 0, 0, 0x400},
    [110] = {127, -46, 0, 0, 0x20},
    [120] = {0, -127, 0, 0, 0x200},
    [122] = {0, -55, 0, 0, 0x0},
    [123] = {0, -55, 0, 0, 0x0},
    [124] = {0, -55, 0, 0, 0x100},
  }},
  classic = {ax = -9, vx = 0, pad = {
    [1] = {0, -127, 0, 0, 0x200},
    [5] = {0, 0, 0, 0, 0x100},
    [10] = {0, 0, 0, 127, 0x0},
    [35] = {0, 0, 127, 0, 0x0},
    [64] = {0, -55, 0, 0, 0x0},
    [65] = {0, -55, 0, 0, 0x0},
    [66] = {0, -55, 0, 0, 0x0},
    [67] = {0, -55, 0, 0, 0x0},
    [68] = {0, -55, 0, 0, 0x100},
    [82] = {0, 0, 0, 0, 0x400},
    [85] = {127, -46, 0, 0, 0x20},
    [102] = {0, 0, 0, 127, 0x0},
  }},
}
local variant = "waveshine"
local START_X_ATT, START_X_VIC, PAD, LAST
local function select_variant(name)
  local v = VARIANTS[name]
  if not v then return false end
  variant = name
  START_X_ATT, START_X_VIC, PAD = v.ax, v.vx, v.pad
  LAST = 0
  for k in pairs(PAD) do if k > LAST then LAST = k end end
  return true
end
select_variant(variant)

local mode, wf, tp, n = "off", 0, nil, 0
local stat = {hits = 0, broke = nil, ko = nil, p0 = 0, lastp = 0, frames = 0, log = {}}
local line = "waiting for a match"
local ready_to_replay = false

local function reset_stat()
  stat = {hits = 0, broke = nil, ko = nil, p0 = 0, lastp = 0, frames = 0, falls0 = 0, first = nil, free = {}}
end

local function begin()
  mode, wf, tp, n = "warm", 0, nil, 0
  reset_stat()
  gd.cpu_mode(ATT, "script")
  gd.cpu_mode(VIC, "stand")
  line = "Turbo combo: setting up"
end

function on_match_start() begin() end
gd.command("turbo_combo", function(args)
  local name = args and args:match("(%a+)")
  if name and not select_variant(name) then gd.log("turbo_combo: unknown variant " .. name .. " (waveshine, classic)"); return end
  gd.scene_launch(SCENE)
end, "turbo_combo [waveshine|classic]: restart the match and play the Turbo combo (again)")
gd.command("turbo_combo_here", function() begin() end,
  "turbo_combo_here: replay in this match (positions are reset; damage and stale moves are not, so the numbers differ)")

local function report()
  for _, f in ipairs(stat.free) do if f > stat.first and f < stat.last then stat.broke = stat.broke or f end end
  local ok = stat.hits > 0 and not stat.broke
  local msg = string.format("TURBO-COMBO RESULT variant=" .. variant .. " hits=%d victim=%.1f%% hitstun_unbroken=%s ko=%s frames=%d",
    stat.hits, stat.lastp, tostring(not stat.broke), tostring(stat.ko), stat.frames)
  if stat.broke then msg = msg .. " broke_at=" .. stat.broke end
  gd.log(msg)
  line = ok and (stat.ko and string.format("%d hits, 0 to %.0f%%, one unbroken combo, KO", stat.hits, stat.lastp)
                 or msg) or msg
end

function on_frame()
  if mode == "off" then return end
  local a, v = gd.player(ATT), gd.player(VIC)
  if not a or not v then return end
  if mode == "warm" then
    wf = wf + 1
    if wf > 2 and not tp and a.action == 14 and v.action == 14 and not a.airborne and not v.airborne then
      gd.teleport(ATT, START_X_ATT, 0); gd.teleport(VIC, START_X_VIC, 0); tp = wf
      gd.set_percent(VIC, 0)
    end
    if tp then
      local left = tp + 25 - wf
      line = string.format("Turbo combo in %.1f s", math.max(0, left) / 60)
      if wf == tp + 25 then
        mode, n = "play", 0
        stat.falls0 = v.falls; stat.p0 = v.percent; stat.lastp = v.percent
      end
    end
    return
  end
  if mode == "play" or mode == "tail" then
    n = n + 1
    stat.frames = n
    -- readback: count hits; the victim must be in hitstun on every frame between the first hit and the last
    if v.percent > stat.lastp + 0.001 then
      stat.hits = stat.hits + 1
      stat.first = stat.first or n
      stat.last = n
      gd.log(string.format("TURBO-COMBO[" .. variant .. "] hit %d frame %d fox_action %d victim %.1f%% hitstun %.0f  fox x=%.1f victim x=%.1f", stat.hits, n, a.action, v.percent, v.hitstun, a.x, v.x))
    end
    stat.lastp = v.percent
    if stat.first and not v.in_hitstun and v.falls == stat.falls0 then stat.free[#stat.free + 1] = n end
    if v.falls ~= stat.falls0 and not stat.ko then stat.ko = n; gd.log("TURBO-COMBO KO at frame " .. n) end
    if mode == "play" then
      local s = PAD[n]
      if s then gd.cpu_pad(ATT, {x = s[1], y = s[2], cx = s[3], cy = s[4], buttons = s[5]}, 1) end
      if n > LAST then mode = "tail" end
    end
    line = string.format("hits %d   victim %.0f%%   %s", stat.hits, v.percent, stat.ko and "KO" or (v.in_hitstun and "in hitstun" or "free"))
    if (stat.ko and n > stat.ko + 30) or n > LAST + 400 then mode = "done"; report(); ready_to_replay = true end
  end
end

function on_tick()
  if ready_to_replay and gd.key_pressed("R") then ready_to_replay = false; gd.scene_launch(SCENE) end
end

function on_draw()
  gd.text(20, 20, line)
  if mode == "done" then gd.text(20, 40, "press R (or console: turbo_combo) to play it again") end
end
