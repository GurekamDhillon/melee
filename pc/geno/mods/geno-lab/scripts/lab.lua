-- Geno Lab - frame-steppable fighter inspection (private, Geno build). docs/geno.md "Geno Lab".
--
-- Offline only: pause, step and step-back need "gameplay": true, and every write the Lab makes
-- (debug drawing, history) is refused during a netplay/rollback session, where it only reads.
--
-- Everything on screen is drawn with gd.kit and the Lab's own art (ui/): no plain gd.text.
-- The Lab has a DISPLAY MODE; each mode shows its own panels and overlays and has its own keys.
-- The keyboard does not play in a LAB match (the Lab owns every key), so any key is free.
--
-- Global keys (every mode):
--   SPACE  pause / resume          RIGHT  step 1 frame (hold: slow play; CTRL: 10)
--   LEFT   step back (hold; CTRL: 10)   F5 / F6  save / load state 1
--   F8     hot reload (geno.json, overlays, this script) and replay the last seconds
--   G      go live: stop replaying the logged input here (after a rewind)
--   TAB    next mode (SHIFT: previous)  1-9, 0  a mode directly    F  focus the next fighter
--   H      hide / show the Lab UI  F3  help (this mode's keys)  ESC  the LAB pause menu
-- Mode keys:
--   CLEAN     (none)                      just the game and a tiny mode chip
--   HITBOXES  B boxes  L labels  E ECB  D hitbox data
--   FRAMES    T timeline  B boxes  Q / E scrub -1 / +1  HOME replay  C lock-step  R mirror pad
--   STAGE     C collision  L ledges  T terrain  P points  Z zones
--   INSPECT   M model  S skeleton  J joint numbers  I info  A attributes  L event log
--   MOVES     UP / DOWN pick  ENTER play  V filter  Z / X speed  S stop  L loop  N from neutral
--   LAUNCH    Q / E hitbox  D DI  Z / X percent -/+10  P live percent  V victim  A arc  K check
--   A/B       R record A  B re-sim B on reloaded data  C re-sim B on the same data  M two fighters
--   TRAINING  A frame advantage  M move card  I input display  K tech feedback  B boxes  C clear
--             R record the dummy  P playback
--   COMBO     D DI fan  B boxes  C clear      (HITBOXES also: W swept  U hurtbox states  S shield  C grab)
--   (FRAMES also: N the rollback strip.)  The pause menu's TOOLS tab: frame-data export and diff.
-- Console: "lab help", "lab status".

if gd.lab_api == nil then
  gd.log("Geno Lab needs the Geno build (gd.lab_api missing) - not started")
  return
end

local K = gd.kit

-- ---- palette (lab_palette.py; kit tokens) ----------------------------------------------------
local INK, BONE, MUTED, DISABLED = 0x0A0E18FF, 0xF2EFE4FF, 0xB8C2DCFF, 0x7D88A6FF
local GOLD, GOLD_DK, OK, DANGER = 0xF0B429FF, 0xA9761AFF, 0x27B88AFF, 0xE5483BFF
local ACCENT = 0x38C9D9FF
local GLASS, GLASS_SOLID, TRACK, TICK = 0x111122DB, 0x111122FF, 0x232B40FF, 0x7D88A6FF
local HIT = { [0] = 0xE5483BFF, 0xF5902EFF, 0xE24FB7FF, 0x8E72FFFF, MUTED } -- no yellow
local MARK = { iasa = 0x27B88AFF, invinc = BONE, gfx = 0x4D8DFFFF, sfx = 0xE8A6FFFF, vis = 0xF7DF5EFF }
local PORT = { 0xE5483BFF, 0x2F7CF0FF, 0xF4D23AFF, 0x27B88AFF, 0xC77DFFFF, BONE }
local SHEAR = 0.25
if K then
  SHEAR = K.shear or SHEAR
  for i = 1, 4 do
    local ok, c = pcall(K.color, "p" .. i)
    if ok and c then PORT[i] = c end
  end
end
local function alpha(c, a) return (c & 0xFFFFFF00) | a end

-- ---- display modes ---------------------------------------------------------------------------
-- t = toggles (remembered per mode), a = actions (keys that do something rather than show it)
local MODES = {
  { id = "clean", name = "CLEAN", icon = "lab_clean",
    blurb = "Just the game. A tiny chip in the corner says where you are.", t = {}, a = {} },
  { id = "hitboxes", name = "HITBOXES", icon = "lab_hitbox",
    blurb = "Hitboxes, hurtboxes and the numbers on them. The reading-a-move mode.",
    t = {
      { k = "B", id = "boxes", label = "Hit / hurtboxes", icon = "lab_hitbox", def = true,
        desc = "The game's own hitbox and hurtbox draw, capsules and all." },
      { k = "L", id = "labels", label = "Hitbox labels", icon = "lab_hitlabels", def = true,
        desc = "A chip on every live hitbox: id, damage, angle." },
      { k = "E", id = "ecb", label = "ECB", icon = "lab_ecb", def = false,
        desc = "The environment collision diamond. Where the fighter meets the floor." },
      { k = "D", id = "data", label = "Hitbox data", icon = "lab_info", def = true,
        desc = "A panel with every live hitbox of the focused fighter, in full." },
      { k = "W", id = "swept", label = "Swept hitboxes", icon = "lab_hitlabels", def = true,
        desc = "Each hitbox as the capsule from last frame to this one (what Melee tests hits against), with fading ghosts." },
      { k = "U", id = "hurt", label = "Hurtbox states", icon = "lab_hurtbox", def = false,
        desc = "Hurtboxes coloured by state: normal yellow, intangible blue, invincible green, with the frames left." },
      { k = "S", id = "shield", label = "Shield bubble", icon = "lab_eye", def = true,
        desc = "The shield as the game has it this frame: its size, where it sits, its health." },
      { k = "C", id = "grab", label = "Grab boxes", icon = "lab_focus", def = true,
        desc = "Grab ranges, in purple." },
    }, a = {} },
  { id = "frames", name = "FRAMES", icon = "lab_timeline",
    blurb = "The move as a timeline: windows, IASA, effects. Scrub it, replay it, line two up.",
    t = {
      { k = "T", id = "timeline", label = "Move timeline", icon = "lab_timeline", def = true,
        desc = "The subaction script as a track: hit windows, IASA, GFX, SFX, body state." },
      { k = "B", id = "boxes", label = "Hit / hurtboxes", icon = "lab_hitbox", def = true,
        desc = "Keep the boxes on while you scrub." },
      { k = "N", id = "net", label = "Rollbacks", icon = "lab_rollback", def = true,
        desc = "Every rollback as a bar (netplay, SyncTest, the fake network): how far, why, what it cost, and the first mismatch." },
    },
    a = {
      { k = "Q", label = "Scrub -1", icon = "lab_step_back", rep = true, run = "scrub_back" },
      { k = "E", label = "Scrub +1", icon = "lab_step", rep = true, run = "scrub_fwd" },
      { k = "HOME", label = "Replay", icon = "lab_rewind", run = "replay" },
      { k = "C", label = "Lock-step", icon = "lab_lockstep", run = "compare" },
      { k = "R", label = "Mirror pad", icon = "lab_mirror", run = "mirror", state = "mirror" },
    } },
  { id = "stage", name = "STAGE", icon = "lab_stage",
    blurb = "The stage's bones: collision lines, ledges, spawn points, blast zones.",
    t = {
      { k = "C", id = "coll", label = "Collision", icon = "lab_stage", def = true,
        desc = "Floor, wall and ceiling lines, with every fighter's ECB." },
      { k = "L", id = "ledges", label = "Ledges", icon = "lab_ledge", def = true,
        desc = "The grabbable corners." },
      { k = "T", id = "terrain", label = "Terrain", icon = "lab_terrain", def = false,
        desc = "Surface kinds: what is ground, what is a platform." },
      { k = "P", id = "points", label = "Points", icon = "lab_points", def = false,
        desc = "Spawn, respawn and item points." },
      { k = "Z", id = "zones", label = "Zones", icon = "lab_zones", def = false,
        desc = "Camera limits and blast zones. Cross the line, lose the stock." },
    }, a = {} },
  { id = "inspect", name = "INSPECT", icon = "lab_inspect",
    blurb = "Under the hood: skeleton, joints, state, attributes, the event log.",
    t = {
      { k = "M", id = "model", label = "Model", icon = "lab_model", def = true,
        desc = "The fighter's model. Off leaves the skeleton on its own." },
      { k = "S", id = "skel", label = "Skeleton", icon = "lab_skeleton", def = true,
        desc = "Bones and joints, in the port's colour." },
      { k = "J", id = "joints", label = "Joint numbers", icon = "lab_joints", def = false,
        desc = "The joint index on every joint, for subaction and hitbox bone ids." },
      { k = "I", id = "info", label = "Info panel", icon = "lab_info", def = true,
        desc = "Action, frame, speeds, hitlag, hitstun, intangibility. Two fighters side by side." },
      { k = "A", id = "attrs", label = "Attributes", icon = "lab_attrs", def = false,
        desc = "Every attribute, fighters compared. Differences first, in gold." },
      { k = "L", id = "log", label = "Event log", icon = "lab_log", def = true,
        desc = "Hits, hitlag, landings and action changes, frame-stamped." },
    }, a = {} },
  { id = "moves", name = "MOVES", icon = "lab_moves",
    blurb = "Every action state the fighter has: common, specials, m-ex and Geno. Pick one and play it.",
    t = {
      { k = "L", id = "loop", label = "Loop", icon = "lab_rewind", def = true,
        desc = "Play the state again each time it ends." },
      { k = "N", id = "neutral", label = "From neutral", icon = "lab_focus", def = true,
        desc = "Start each play from the match's first frame (everyone standing), not from whatever was happening." },
      { k = "T", id = "timeline", label = "Timeline", icon = "lab_timeline", def = true,
        desc = "The playing state's track along the bottom, following the frame." },
      { k = "B", id = "boxes", label = "Hit / hurtboxes", icon = "lab_hitbox", def = true,
        desc = "The game's hitbox and hurtbox draw while it plays." },
    },
    a = {
      { k = "UP", label = "Up", icon = "lab_step_back", rep = true, run = "br_up" },
      { k = "DOWN", label = "Down", icon = "lab_step", rep = true, run = "br_down" },
      { k = "ENTER", label = "Play", icon = "lab_play", run = "br_play", state = "br_playing" },
      { k = "V", label = "Filter", icon = "lab_moves", run = "br_filter" },
      { k = "Z", label = "Slower", icon = "lab_slowmo", run = "br_slower" },
      { k = "X", label = "Faster", icon = "lab_forward", run = "br_faster" },
      { k = "S", label = "Stop", icon = "lab_pause", run = "br_stop" },
    } },
  { id = "launch", name = "LAUNCH", icon = "lab_launch",
    blurb = "Where a hit sends them: knockback, angle, hitstun, tumble and the arc to the blast zone, by the game's own formula.",
    t = {
      { k = "A", id = "arc", label = "Arc", icon = "lab_launch", def = true,
        desc = "The predicted flight, a line to the end of hitstun, a burst where it crosses a blast zone." },
      { k = "B", id = "boxes", label = "Hit / hurtboxes", icon = "lab_hitbox", def = true,
        desc = "The game's hitbox and hurtbox draw." },
      { k = "K", id = "check", label = "Check real hits", icon = "lab_ko", def = true,
        desc = "When a hit really lands, predict it from its own numbers, follow the real flight and show the error." },
    },
    a = {
      { k = "Q", label = "Hitbox -", icon = "lab_step_back", run = "kb_prev" },
      { k = "E", label = "Hitbox +", icon = "lab_step", run = "kb_next" },
      { k = "D", label = "DI", icon = "lab_mirror", run = "kb_di", state = "kb_di" },
      { k = "Z", label = "-10%", icon = "lab_percent", rep = true, run = "kb_pct_down" },
      { k = "X", label = "+10%", icon = "lab_percent", rep = true, run = "kb_pct_up" },
      { k = "P", label = "Live %", icon = "lab_percent", run = "kb_pct_live" },
      { k = "V", label = "Victim", icon = "lab_dummy", run = "kb_victim" },
    } },
  { id = "ab", name = "A/B", icon = "lab_ab",
    blurb = "Two variants on the same inputs: record A, re-simulate B from the same frame, see where they part.",
    t = {
      { k = "O", id = "ghost", label = "Ghosts", icon = "lab_eye", def = true,
        desc = "A's path and B's path over the stage, and where each is on the frame you look at." },
      { k = "T", id = "tracks", label = "Tracks", icon = "lab_timeline", def = true,
        desc = "A and B as two tracks coloured by action, the differing frames under them, the first divergence marked." },
    },
    a = {
      { k = "R", label = "Record A", icon = "lab_record", run = "ab_record", state = "ab_rec" },
      { k = "B", label = "B: reload", icon = "lab_reload", run = "ab_reload" },
      { k = "C", label = "B: same", icon = "lab_rewind", run = "ab_same" },
      { k = "M", label = "Two fighters", icon = "lab_mirror", run = "ab_mirror" },
      { k = "Q", label = "Frame -", icon = "lab_step_back", rep = true, run = "ab_back" },
      { k = "E", label = "Frame +", icon = "lab_step", rep = true, run = "ab_fwd" },
    } },
  { id = "training", name = "TRAINING", icon = "lab_focus",
    blurb = "For practice: frame advantage after every exchange, the move card, your inputs, and how your tech went.",
    t = {
      { k = "A", id = "adv", label = "Frame advantage", icon = "lab_ab", def = true,
        desc = "After every hit or shield hit: who acts first, and by how many frames. +3 on shield = the attacker is 3 frames ahead." },
      { k = "M", id = "card", label = "Move card", icon = "lab_timeline", def = true,
        desc = "The move being done now: startup, active frames, total, IASA; for aerials the landing lag, L-cancelled lag and autocancel." },
      { k = "I", id = "input", label = "Input display", icon = "lab_keys", def = true,
        desc = "The sticks, buttons and triggers as the game saw them, and a log of inputs with their frame: jump f1 > R f4." },
      { k = "K", id = "tech", label = "Tech feedback", icon = "lab_ledge", def = true,
        desc = "L-cancels (early / late by how many frames), wavedash and waveland timing, ledgedash GALINT, short or full hop, with success rates." },
      { k = "B", id = "boxes", label = "Hit / hurtboxes", icon = "lab_hitbox", def = false,
        desc = "The game's hitbox and hurtbox draw." },
    },
    a = {
      { k = "C", label = "Clear", icon = "lab_trash", run = "tr_clear" },
      { k = "R", label = "Record dummy", icon = "lab_record", run = "dm_record", state = "dm_rec" },
      { k = "P", label = "Playback", icon = "lab_play", run = "dm_play", state = "dm_play" },
    } },
  { id = "combo", name = "COMBO", icon = "lab_ko",
    blurb = "Was it true? Each hit of a combo, true or escapable (by how many frames, and how), why it dropped, and how DI moves the last hit.",
    t = {
      { k = "D", id = "fan", label = "DI fan", icon = "lab_launch", def = true,
        desc = "The last hit's flight for each DI: none, in, out, survival, to the end of hitstun." },
      { k = "B", id = "boxes", label = "Hit / hurtboxes", icon = "lab_hitbox", def = false,
        desc = "The game's hitbox and hurtbox draw, with the swept hitboxes." },
    },
    a = {
      { k = "C", label = "Clear", icon = "lab_trash", run = "cb_clear" },
    } },
}
local MODE_BY_ID = {}
for i, m in ipairs(MODES) do MODE_BY_ID[m.id] = i end

local GLOBAL_KEYS = {
  { "SPACE", "Pause / resume" }, { "RIGHT", "Step +1 (hold, CTRL x10)" },
  { "LEFT", "Step -1 (hold, CTRL x10)" }, { "F5", "Save state 1" }, { "F6", "Load state 1" },
  { "F8", "Hot reload + replay" }, { "G", "Go live (stop replaying)" },
  { "TAB", "Next mode (SHIFT back)" }, { "1-0", "Mode directly" }, { "F", "Focus next fighter" },
  { "H", "Hide / show the Lab UI" }, { "F3", "This help" }, { "ESC", "Pause menu" },
}

-- ---- settings (scripts-data/<mod>/settings.txt) ------------------------------------------------
local cfg = { on = true, always = false, history_s = 10, reload_s = 2, mode = 2, hidden = false, help = false }
local tog = {} -- tog[mode_id][toggle_id] = bool
for _, m in ipairs(MODES) do
  tog[m.id] = {}
  for _, t in ipairs(m.t) do tog[m.id][t.id] = t.def end
end
local SETTINGS = "settings.txt"

local function save_settings()
  local out = { "mode=" .. MODES[cfg.mode].id, "hidden=" .. tostring(cfg.hidden),
    "history_s=" .. cfg.history_s, "reload_s=" .. cfg.reload_s, "always=" .. tostring(cfg.always) }
  for _, m in ipairs(MODES) do
    for _, t in ipairs(m.t) do out[#out + 1] = m.id .. "." .. t.id .. "=" .. tostring(tog[m.id][t.id]) end
  end
  pcall(gd.data_write, SETTINGS, table.concat(out, "\n") .. "\n")
end

local function load_settings()
  local ok, text = pcall(gd.data_read, SETTINGS)
  if ok and text ~= nil then
    for k, v in text:gmatch("([%w_%.]+)=([^\r\n]+)") do
      local mid, tid = k:match("^(%w+)%.(%w+)$")
      if mid and tog[mid] and tog[mid][tid] ~= nil then
        tog[mid][tid] = v == "true"
      elseif k == "mode" and MODE_BY_ID[v] then cfg.mode = MODE_BY_ID[v]
      elseif k == "hidden" then cfg.hidden = v == "true"
      elseif k == "always" then cfg.always = v == "true"
      elseif k == "history_s" and tonumber(v) then cfg.history_s = tonumber(v)
      elseif k == "reload_s" and tonumber(v) then cfg.reload_s = tonumber(v) end
    end
  end
  cfg.on = cfg.always or gd.lab_request()
end
load_settings()

local function mode() return MODES[cfg.mode] end
local function T(id) return tog[mode().id][id] == true end
local function in_mode(id) return mode().id == id end

-- ---- state -------------------------------------------------------------------------------------
local focus = 1
local log_lines = {} -- {frame, text, color}
local LOG_MAX = 8
local held = {}
local history_on = false
local notice, notice_until = nil, 0
local attr_names = nil
local tl_cache, scrub = {}, {}
local mirror = false
local lab_matched = false

local function offline() return not gd.match().netplay end
local function kit_ok() return K ~= nil and K.available() end

local function say(text, color) notice, notice_until = { text, color or ACCENT }, gd.time() + 2.2 end

local function log(text, color)
  table.insert(log_lines, { gd.match().frame, text, color or MUTED })
  while #log_lines > LOG_MAX do table.remove(log_lines, 1) end
end

-- ---- the game's own debug drawing -------------------------------------------------------------
local function wanted_flags()
  if not cfg.on or cfg.hidden then return gd.draw.DEFAULT end
  local f = 0
  if not in_mode("inspect") or T("model") then f = f | gd.draw.MODEL end
  if (in_mode("hitboxes") or in_mode("frames") or in_mode("moves") or in_mode("launch") or in_mode("training") or in_mode("combo")) and T("boxes") then f = f | gd.draw.HIT | gd.draw.THROWN end
  return f
end

local function wanted_stage()
  if not cfg.on or cfg.hidden or not in_mode("stage") then return 0 end
  local S, f = gd.stage_draw, 0
  if T("coll") then f = f | S.COLL end
  if T("ledges") then f = f | S.COLL | S.LEDGES end
  if T("terrain") then f = f | S.COLL | S.TERRAIN end
  if T("points") then f = f | S.POINTS end
  if T("zones") then f = f | S.ZONES end
  return f
end

local function apply_draw()
  if not offline() then return end
  local want = wanted_flags()
  for _, p in ipairs(gd.players()) do
    if gd.debug_draw(p.port) ~= want then gd.debug_draw(p.port, want) end
  end
  local stage = wanted_stage()
  local now = gd.debug_stage()
  if now ~= nil and (now & 31) ~= stage then gd.debug_stage(stage) end
end

local function restore_draw()
  if not offline() then return end
  for _, p in ipairs(gd.players()) do gd.debug_draw(p.port, gd.draw.DEFAULT) end
  if gd.debug_stage() ~= nil then gd.debug_stage(0) end
end

local function ensure_history()
  if history_on or not offline() or not gd.match().active then return end
  local h = gd.history(math.floor(cfg.history_s * 60))
  history_on = true
  if h.depth < cfg.history_s * 60 then say(string.format("History: %.0f s (memory)", h.depth / 60), DANGER) end
end

-- seconds, for the history readouts
local function secs(frames) return string.format("%.1f", (frames or 0) / 60) end

-- ---- stepping, scrubbing, lock-step -----------------------------------------------------------
local function step(n) if offline() then gd.step(n) end end

local function back(n)
  if not offline() then return end
  ensure_history()
  local ok, why = gd.step_back(n)
  if not ok then say("Step back: " .. tostring(why), DANGER) end
end

local function repeat_key(name)
  if gd.key(name) then held[name] = (held[name] or 0) + 1 else held[name] = 0 return false end
  local t = held[name]
  return t == 1 or (t > 18 and t % 3 == 0)
end

local function next_port(d)
  local ports = {}
  for _, p in ipairs(gd.players()) do ports[#ports + 1] = p.port end
  if #ports == 0 then return end
  for i, port in ipairs(ports) do
    if port == focus then focus = ports[((i - 1 + (d or 1)) % #ports) + 1] return end
  end
  focus = ports[1]
end

local AIR_MOTIONS = { AttackAirN = true, AttackAirF = true, AttackAirB = true, AttackAirHi = true,
  AttackAirLw = true, EscapeAir = true, Fall = true, FallAerial = true }
local function lift_for(port, motion)
  local name = gd.motion_name(motion, port)
  if AIR_MOTIONS[name] or name:find("Air") and not name:find("Landing") then return 40 end
  return 0
end

local function analyse(tl)
  local len = math.max(tl.length or 1, (tl.end_frame or 0) + 1)
  local open, windows, marks = {}, {}, {}
  local function close(id, at)
    local w = open[id]
    if w then w.to = math.max(w.from, at - 1) windows[#windows + 1] = w open[id] = nil end
  end
  for _, e in ipairs(tl.events) do
    if e.name == "hitbox" then
      close(e.id, e.frame)
      open[e.id] = { id = e.id, from = e.frame, dmg = e.damage, angle = e.angle, kbg = e.kbg,
        bkb = e.bkb, wbk = e.wbk, size = e.size, bone = e.bone, element = e.element_name }
    elseif e.name == "hitbox_remove" then
      close(e.id, e.frame)
    elseif e.name == "hitboxes_clear" then
      local ids = {}
      for id in pairs(open) do ids[#ids + 1] = id end
      for _, id in ipairs(ids) do close(id, e.frame) end
    elseif e.name == "hitbox_damage" and open[e.id] then
      open[e.id].dmg = e.value
    else
      marks[#marks + 1] = e
    end
    if e.frame > len then len = e.frame end
  end
  local ids = {}
  for id in pairs(open) do ids[#ids + 1] = id end
  for _, id in ipairs(ids) do close(id, len + 1) end
  table.sort(windows, function(a, b) return a.from < b.from or (a.from == b.from and a.id < b.id) end)
  return windows, marks, len
end

local function timeline_of(p)
  local key = p.action .. ":" .. p.anim_id
  local c = tl_cache[p.port]
  if c and c.key == key then return c end
  local tl = gd.timeline(p.port)
  if tl == nil then return nil end
  local windows, marks, len = analyse(tl)
  c = { key = key, tl = tl, windows = windows, marks = marks, len = len }
  tl_cache[p.port] = c
  return c
end

local function scrub_to(port, frame)
  local p = gd.player(port)
  if p == nil or not offline() then return end
  local s0 = scrub[port]
  local motion = (s0 and s0.motion) or p.action
  if frame < 1 then frame = 1 end
  local ok, why = gd.set_motion(port, motion, frame, 1, lift_for(port, motion))
  if ok then
    scrub[port] = { motion = motion, frame = frame }
    say(string.format("%s  frame %d", gd.motion_name(motion, port), frame))
  else
    say("Scrub: " .. tostring(why), DANGER)
  end
end

local function compare()
  local a = gd.player(focus)
  if a == nil or not offline() then return end
  local motion = (scrub[focus] and scrub[focus].motion) or a.action
  local frame = (scrub[focus] and scrub[focus].frame) or 1
  local n = 0
  for _, p in ipairs(gd.players()) do
    if gd.set_motion(p.port, motion, frame, 1, lift_for(p.port, motion)) then
      scrub[p.port] = { motion = motion, frame = frame }
      n = n + 1
    end
  end
  say(string.format("Lock-step: %d fighters in %s, frame %d", n, gd.motion_name(motion, focus), frame))
end

local function set_mirror(on)
  mirror = on
  if mirror then gd.mirror_pad(1, 2) else gd.mirror_pad() end
  say(mirror and "P2 mirrors P1's controller (P2 must be a human port)" or "Mirror off")
end

local ACTIONS = {
  scrub_back = function() local s0 = scrub[focus] scrub_to(focus, s0 and s0.frame - 1 or 1) end,
  scrub_fwd = function() local s0 = scrub[focus] scrub_to(focus, s0 and s0.frame + 1 or 1) end,
  replay = function() scrub[focus] = nil scrub_to(focus, 1) end,
  compare = compare,
  mirror = function() set_mirror(not mirror) end,
}
local STATES = { mirror = function() return mirror end }

local function set_mode(i)
  cfg.mode = ((i - 1) % #MODES) + 1
  cfg.hidden = false
  say("Mode: " .. mode().name)
  save_settings()
end

local function flip(mid, tid)
  tog[mid][tid] = not tog[mid][tid]
  save_settings()
  return tog[mid][tid]
end

-- ---- kit drawing helpers ------------------------------------------------------------------------
local FLAT = { shear = 0 }
-- data text: never sheared
local function txt(x, y, s, role, col, align, max_w)
  return K.text(x, y, s, role or "caption", col or BONE, align or "left",
    max_w and { shear = 0, max_w = max_w } or FLAT)
end
-- display text: the kit's lean
local function stxt(x, y, s, role, col, align, max_w)
  return K.text(x, y, s, role or "row", col or BONE, align or "left", max_w and { max_w = max_w } or nil)
end
local function quad(x, y, w, h, col, shear)
  K.image("lab_solid", x, y, w, h, { tint = col, shear = shear or 0 })
end
local function img(name, x, y, w, h, col, opts)
  opts = opts or {}
  opts.tint = col
  return K.image(name, x, y, w, h, opts)
end
local function icon(name, x, y, size, col) K.icon(name, x, y, size / 32, col) end
local function measure(s, role) return (K.measure(s, role or "caption")) end

-- a toggle icon: ON = accent, OFF = gap + the icon in the off tint + the slash
local function toggle_icon(name, x, y, size, on, on_col, gap_col)
  if on then
    icon(name, x, y, size, on_col or ACCENT)
  else
    icon("lab_slash_gap", x, y, size, gap_col or GLASS_SOLID)
    icon(name, x, y, size, DISABLED)
    icon("lab_slash", x, y, size, DISABLED)
  end
end

local KEY_NAMES = { ESCAPE = "ESC" }
-- a keycap chip: 3-sliced body (muted) and top face (bone), the label in ink -> width
local function key_chip(x, y, label, held_now)
  label = KEY_NAMES[label] or label
  local w = math.max(16, math.floor(measure(label, "caption") + 9))
  local body = held_now and ACCENT or MUTED
  img("lab_key_l", x, y, 8, 16, body)
  img("lab_key_r", x + w - 8, y, 8, 16, body)
  if w > 16 then quad(x + 8, y, w - 16, 16, body) end
  local dy = held_now and 1 or 0
  img("lab_key_top_l", x, y + dy, 8, 16, BONE)
  img("lab_key_top_r", x + w - 8, y + dy, 8, 16, BONE)
  if w > 16 then quad(x + 8, y + 1 + dy, w - 16, 11, BONE) end
  txt(x + w / 2, y + 10 + dy, label, "caption", INK, "center")
  return w
end
local function key_w(label) return math.max(16, math.floor(measure(KEY_NAMES[label] or label, "caption") + 9)) end

local function panel(x, y, w, h, title, title_col)
  K.panel(x, y, w, h, { prefix = "lab_frame", fill = GLASS, piece = 16 })
  if title then
    local tw = measure(title, "caption") + 16
    quad(x + 8, y + 3, tw, 14, title_col or ACCENT, SHEAR)
    txt(x + 16, y + 14, title, "caption", INK)
  end
end

-- word-wrap for the kit's proportional font
local function wrap(s, role, w)
  local lines, cur = {}, ""
  for word in s:gmatch("%S+") do
    local try = cur == "" and word or (cur .. " " .. word)
    if measure(try, role) > w and cur ~= "" then
      lines[#lines + 1] = cur
      cur = word
    else
      cur = try
    end
  end
  if cur ~= "" then lines[#lines + 1] = cur end
  return lines
end

-- ---- LAB mode: the full-screen pause menu (START / ESC in a LAB match; LAB only) --------------
-- Tabs across the top (L / R), big sheared rows on the left, a detail panel on the right, a
-- controls strip along the bottom. The game is frozen (gd.pause) and dimmed behind it. Tabs and
-- rows are recorded as hit rects (menu.hits) so a mouse can drive it once the port has one.
local RESET_SLOT = 4
local menu = { open = false, tab = 1, sel = {}, port = 1, was_paused = false, prev = {}, rep = 0,
  reset_saved = false, t = 0, tab_t = 99, sel_t = 99, hits = {}, slot = 1, target = 2, pct = 0 }

local function in_lab_match()
  return gd.lab_mode ~= nil and gd.lab_mode() and gd.match().active and offline()
end

local function menu_close()
  menu.open = false
  if not menu.was_paused then gd.resume() end
  for port = 1, 4 do pcall(gd.input, port, 0, 10) end
end

local function menu_leave(where)
  menu.open = false
  for port = 1, 4 do pcall(gd.input, port, 0, 10) end
  gd.lab_leave(where)
end

local function fighter_name(port)
  local p = gd.player(port)
  if p == nil then return "P" .. port .. " -" end
  return string.format("P%d %s%s", port, (p.char_name or "?"):upper(), p.cpu and " CPU" or "")
end

local function onoff(v) return v and "ON" or "OFF" end
local HISTORY_STEPS = { 2, 5, 10, 20 } -- seconds
local RELOAD_STEPS = { 1, 2, 3, 5 }      -- seconds replayed after a hot reload

-- ---- the saved-state library (gd.state_*: files in scripts-data/geno-lab_lab/states) ----------
-- by GrKind (gd.match().stage; gr/forward.h)
local STAGE_NAMES = { [0x02] = "Castle", [0x03] = "Rainbow", [0x04] = "Kongo", [0x05] = "Japes",
  [0x06] = "Great Bay", [0x07] = "Temple", [0x08] = "Brinstar", [0x09] = "Depths", [0x0A] = "YS",
  [0x0B] = "Yoshi's Island", [0x0C] = "FoD", [0x0D] = "Green Greens", [0x0E] = "Corneria",
  [0x0F] = "Venom", [0x10] = "PS", [0x11] = "Poke Floats", [0x12] = "Mute City", [0x13] = "Big Blue",
  [0x14] = "Onett", [0x15] = "Fourside", [0x16] = "Icicle", [0x18] = "Mushroom I",
  [0x19] = "Mushroom II", [0x1B] = "Flat Zone", [0x1C] = "DL", [0x1D] = "YI64", [0x1E] = "Kongo64",
  [0x24] = "BF", [0x25] = "FD" }
local lib = { gen = -1, rows = {}, confirm = nil }

local function cap(s) return (s or "?"):gsub("^%l", string.upper) end

-- "Fox v Falco · FD · f1234": what the state is, as a name the list can show
local function auto_name()
  local names = {}
  for _, p in ipairs(gd.players()) do names[#names + 1] = cap(p.char_name) end
  local st = gd.match().stage
  return string.format("%s \u{2013} %s \u{2013} f%d", table.concat(names, " v "),
    STAGE_NAMES[st] or ("stage " .. tostring(st)), gd.match().frame)
end

local function what_now()
  local names = {}
  for _, p in ipairs(gd.players()) do names[#names + 1] = cap(p.char_name) end
  local st = gd.match().stage
  return table.concat(names, " v ") .. " on " .. (STAGE_NAMES[st] or ("stage " .. tostring(st)))
end

local function lib_refresh(force)
  if not gd.state_gen then return end
  local g = gd.state_gen()
  if force or g ~= lib.gen then
    lib.gen = g
    local ok, rows = pcall(gd.state_list)
    lib.rows = ok and rows or {}
  end
end

local function lib_save()
  local file, why = gd.state_save(auto_name(), what_now())
  if file then say("Saved to the library") else say("Not saved: " .. tostring(why), DANGER) end
end

local function lib_load(row)
  local ok, why = gd.state_load(row.file)
  if ok then say("Loading " .. row.name) else say("Refused: " .. tostring(why), DANGER) end
end

local function lib_delete(row)
  local ok, why = gd.state_delete(row.file)
  lib.confirm = nil
  if ok then say("Deleted " .. row.name) lib_refresh(true) else say(tostring(why), DANGER) end
end

local function hot_reload()
  if not offline() then return end
  local ok, why = gd.hot_reload(cfg.reload_s)
  if ok then say(string.format("Hot reload: rewinding %d s, then replaying", cfg.reload_s))
  else say("Hot reload: " .. tostring(why), DANGER) end
end

local function display_items()
  local items = {
    { label = "Display mode", icon = mode().icon, key = "TAB",
      desc = function() return mode().blurb .. " Each mode remembers its own toggles." end,
      value = function() return mode().name end,
      run = function() set_mode(cfg.mode + 1) end, adjust = function(d) set_mode(cfg.mode + d) end,
      preview = "mode" },
  }
  for _, t in ipairs(mode().t) do
    items[#items + 1] = { label = t.label, icon = t.icon, desc = t.desc, key = t.k,
      toggle = function() return T(t.id) end,
      value = function() return onoff(T(t.id)) end,
      run = function() flip(mode().id, t.id) end, adjust = function() flip(mode().id, t.id) end }
  end
  items[#items + 1] = { label = "Lab UI", icon = "lab_eye", key = "H",
    desc = "Hide every Lab panel and overlay: the game as the game draws it. H does the same.",
    toggle = function() return not cfg.hidden end,
    value = function() return cfg.hidden and "HIDDEN" or "SHOWN" end,
    run = function() cfg.hidden = not cfg.hidden save_settings() end,
    adjust = function() cfg.hidden = not cfg.hidden save_settings() end }
  return items
end

local states_items -- below: the rows are rebuilt from the library each time
local tools_items  -- below (stage E): the creator tools
local draw_rollbacks -- below (stage E)

local TABS = {
  { name = "PLAY", icon = "lab_play", items = {
    { label = "Resume", icon = "lab_play", desc = "Unfreeze. Everything carries on from this exact frame.",
      run = function() menu_close() end },
    { label = "Step +1", icon = "lab_step", key = "RIGHT",
      desc = "One frame forward, then frozen again. The heart of frame study.",
      value = function() return "f " .. gd.match().frame end, run = function() gd.step(1) end },
    { label = "Step -1", icon = "lab_step_back", key = "LEFT",
      desc = "One frame back through the history: the keyframe before it, then the logged input re-simulated. Undo, but for physics.",
      value = function() local h = gd.history() return string.format("-%s / %d s", secs(h.back), cfg.history_s) end,
      run = function() back(1) end },
    { label = "Step +10", icon = "lab_forward", desc = "Ten frames at once, for when one at a time is a chore.",
      run = function() gd.step(10) end },
    { label = "Focus", icon = "lab_focus", key = "F",
      desc = "Whose timeline, hitbox data and inspector you are looking at. Left / right to switch.",
      value = function() return fighter_name(focus) end,
      run = function() next_port(1) end, adjust = function(d) next_port(d) end },
  } },
  { name = "DISPLAY", icon = "lab_display", items = display_items },
  { name = "DUMMY", icon = "lab_dummy", items = {
    { label = "Target", icon = "lab_dummy", desc = "The fighter the dummy settings act on.",
      value = function() return fighter_name(menu.target) end,
      run = function() menu.target = menu.target % 4 + 1 end,
      adjust = function(d) menu.target = ((menu.target - 1 + d) % 4) + 1 end },
    { label = "Damage", icon = "lab_percent",
      desc = "Left / right in steps of 10, A applies it. Test the kill percent, not the vibes.",
      value = function() return menu.pct .. "%" end,
      run = function()
        if gd.player(menu.target) then
          gd.set_percent(menu.target, menu.pct)
          say(string.format("P%d at %d%%", menu.target, menu.pct))
        end
      end,
      adjust = function(d) menu.pct = math.max(0, math.min(999, menu.pct + d * 10)) end },
    { label = "Lock-step", icon = "lab_lockstep", key = "C",
      desc = "Every fighter into the focused fighter's move at the same frame. Compare side by side.",
      run = function() compare() menu_close() end },
    { label = "Replay move", icon = "lab_rewind", key = "HOME",
      desc = "The focused fighter's move again from frame 1.",
      run = function() scrub[focus] = nil scrub_to(focus, 1) menu_close() end },
    { label = "Mirror my pad", icon = "lab_mirror", key = "R",
      desc = "P2 copies P1's controller. P2 has to be a human port.",
      toggle = function() return mirror end, value = function() return onoff(mirror) end,
      run = function() set_mirror(not mirror) end },
  } },
  { name = "STATES", icon = "lab_save", items = function() return states_items() end },
  { name = "TOOLS", icon = "lab_export", items = function() return tools_items() end },
  { name = "EXIT", icon = "lab_exit", items = {
    { label = "Change fighters", icon = "lab_focus", desc = "Back to LAB's character select.",
      run = function() menu_leave("css") end },
    { label = "Change stage", icon = "lab_stage", desc = "Same fighters, a different floor.",
      run = function() menu_leave("sss") end },
    { label = "Quit", icon = "lab_power", desc = "Leave the Lab for the menus. No contest, no results.",
      value = function() return "NO CONTEST" end, run = function() menu_leave("menu") end },
  } },
}

states_items = function()
  lib_refresh(false)
  local items = {
    { label = "Save to library", icon = "lab_library",
      desc = function() return "Everything, to a file that survives restarts, named " .. auto_name() ..
        ". It loads only in this build, with this disc, these mods and this Geno data, in this match." end,
      value = function() return #lib.rows .. " SAVED" end,
      run = function() lib_save() end },
    { label = "Quick save", icon = "lab_save", key = "F5",
      desc = "Snapshot everything into a memory slot (gone when the game closes). Left / right picks the slot.",
      value = function() return "SLOT " .. menu.slot end,
      run = function() gd.savestate(menu.slot) say("State " .. menu.slot .. " saved") end,
      adjust = function(d) menu.slot = ((menu.slot - 1 + d) % 3) + 1 end },
    { label = "Quick load", icon = "lab_load", key = "F6",
      desc = "Back to a memory slot, frame-exact.",
      value = function() return "SLOT " .. menu.slot end,
      run = function()
        local ok, err = pcall(gd.loadstate, menu.slot)
        if ok then say("State " .. menu.slot .. " loaded") else say((tostring(err):gsub("^.-: ", "")), DANGER) end
      end,
      adjust = function(d) menu.slot = ((menu.slot - 1 + d) % 3) + 1 end },
    { label = "Reset positions", icon = "lab_rewind",
      desc = "Everyone back where the match started. Clean slate.",
      run = function()
        local ok, err = pcall(gd.loadstate, RESET_SLOT)
        if ok then menu_close() say("Reset to the match start") else say((tostring(err):gsub("^.-: ", "")), DANGER) end
      end },
    { label = "History", icon = "lab_history",
      desc = function()
        local h = gd.history()
        return string.format("How far step-back and the timeline reach: one keyframe every %d frames plus the input log. Now %.0f MB for %s s kept.",
          h.interval, h.mb, secs(h.back))
      end,
      value = function() return cfg.history_s .. " S" end,
      adjust = function(d)
        local k = 1
        for i, v in ipairs(HISTORY_STEPS) do if v == cfg.history_s then k = i end end
        cfg.history_s = HISTORY_STEPS[((k - 1 + d) % #HISTORY_STEPS) + 1]
        history_on = false
        ensure_history()
        save_settings()
      end },
    { label = "Hot reload", icon = "lab_reload", key = "F8",
      desc = function()
        local st = gd.hot_reload_status and gd.hot_reload_status()
        local last = (st and st.text ~= "") and ("  Last: " .. st.text) or ""
        return "Re-read the fighters' geno.json, their overlays and this script, rewind the chosen seconds and replay your input on the new data. Left / right: how far." .. last
      end,
      value = function() return "REPLAY " .. cfg.reload_s .. " S" end,
      run = function() menu_close() hot_reload() end,
      adjust = function(d)
        local k = 1
        for i, v in ipairs(RELOAD_STEPS) do if v == cfg.reload_s then k = i end end
        cfg.reload_s = RELOAD_STEPS[((k - 1 + d) % #RELOAD_STEPS) + 1]
        save_settings()
      end },
  }
  for _, r in ipairs(lib.rows) do
    local row = r
    items[#items + 1] = {
      label = row.name ~= "" and row.name or row.file, icon = row.ok and "lab_load" or "lab_slash",
      state_row = row,
      desc = function()
        local d = (row.what ~= "" and row.what or "a saved state") .. ", saved " .. row.saved .. "."
        if lib.confirm == row.file then return "Delete it? A deletes, B keeps it." end
        if not row.ok then return d .. "  Refused here: " .. row.why .. ".  Y / DELETE deletes it." end
        return d .. "  A loads it.  Y / DELETE deletes it."
      end,
      value = function()
        if lib.confirm == row.file then return "DELETE?" end
        return row.ok and ("f" .. row.frame) or "REFUSED"
      end,
      run = function()
        if lib.confirm == row.file then lib_delete(row) return end
        if row.ok then menu_close() lib_load(row) else say("Refused: " .. row.why, DANGER) end
      end,
      delete = function() lib.confirm = row.file end,
    }
  end
  return items
end

local function tab_items(i)
  local it = TABS[i or menu.tab].items
  if type(it) == "function" then return it() end
  return it
end

local function cur_sel()
  local n = #tab_items()
  local s = menu.sel[menu.tab] or 1
  if s > n then s = n end
  if s < 1 then s = 1 end
  menu.sel[menu.tab] = s
  return s
end

local function set_tab(i)
  local n = ((i - 1) % #TABS) + 1
  if n ~= menu.tab then menu.tab, menu.tab_t, menu.sel_t = n, 0, 0 end
end

local function set_sel(s)
  local n = #tab_items()
  menu.sel[menu.tab] = ((s - 1) % n) + 1
  menu.sel_t = 0
end

local function menu_open(port)
  menu.open, menu.port, menu.t, menu.tab_t, menu.sel_t = true, port, 0, 99, 99
  menu.was_paused = gd.paused()
  ensure_history()
  gd.pause()
  local others = {}
  for _, p in ipairs(gd.players()) do if p.port ~= focus then others[#others + 1] = p.port end end
  if gd.player(menu.target) == nil or menu.target == focus then menu.target = others[1] or focus end
  local tp = gd.player(menu.target)
  if tp then menu.pct = math.floor((tp.percent or 0) / 10 + 0.5) * 10 end
end

local PAD_EDGES = { "A", "B", "X", "Y", "START", "UP", "DOWN", "LEFT", "RIGHT", "L", "R" }
local function pad_edges(port)
  local ok, p = pcall(gd.pad, port)
  if not ok or p == nil then return {} end
  local prev = menu.prev[port] or 0
  menu.prev[port] = p.buttons
  local e = {}
  for _, name in ipairs(PAD_EDGES) do
    local bit = gd.buttons[name]
    e[name] = (p.buttons & bit) ~= 0 and (prev & bit) == 0
  end
  e.x, e.y = p.x, p.y
  return e
end

local function menu_activate()
  local it = tab_items()[cur_sel()]
  if it == nil then return end
  if it.run then it.run() elseif it.adjust then it.adjust(1) end
end

local function menu_adjust(d)
  local it = tab_items()[cur_sel()]
  if it and it.adjust then it.adjust(d) end
end

-- the rect under (x, y) in the last drawn menu: "tab", i | "row", i | nil (for a mouse)
local function menu_hit(x, y)
  for _, h in ipairs(menu.hits) do
    if x >= h.x and x < h.x + h.w and y >= h.y and y < h.y + h.h then return h.kind, h.i end
  end
end

-- returns true when the menu took this tick's input
local function lab_menu_tick()
  if not in_lab_match() then
    menu.open = false
    return false
  end
  if not menu.reset_saved and gd.match().frame >= 1 then
    gd.savestate(RESET_SLOT)
    menu.reset_saved = true
  end
  if not menu.open then
    for port = 1, 4 do
      if pad_edges(port).START then menu_open(port) return true end
    end
    if gd.key_pressed("ESCAPE") then menu_open(1) return true end
    return false
  end
  menu.t, menu.tab_t, menu.sel_t = menu.t + 1, menu.tab_t + 1, menu.sel_t + 1
  local e = pad_edges(menu.port)
  for port = 1, 4 do if port ~= menu.port then pad_edges(port) end end
  local up = e.UP or gd.key_pressed("UP")
  local down = e.DOWN or gd.key_pressed("DOWN")
  local left = e.LEFT or gd.key_pressed("LEFT")
  local right = e.RIGHT or gd.key_pressed("RIGHT")
  -- the stick, with a repeat
  local sx, sy = e.x or 0, e.y or 0
  if math.abs(sx) > 60 or math.abs(sy) > 60 then
    menu.rep = menu.rep + 1
    if menu.rep == 1 or (menu.rep > 16 and menu.rep % 4 == 0) then
      if math.abs(sy) >= math.abs(sx) then
        if sy > 0 then up = true else down = true end
      else
        if sx > 0 then right = true else left = true end
      end
    end
  else
    menu.rep = 0
  end
  if e.L or gd.key_pressed("Q") or gd.key_pressed("PAGEUP") then set_tab(menu.tab - 1) end
  if e.R or gd.key_pressed("E") or gd.key_pressed("PAGEDOWN") then set_tab(menu.tab + 1) end
  if up then set_sel(cur_sel() - 1) end
  if down then set_sel(cur_sel() + 1) end
  if left then menu_adjust(-1) end
  if right then menu_adjust(1) end
  local it = tab_items()[cur_sel()]
  if lib.confirm and (not it or not it.state_row or it.state_row.file ~= lib.confirm) then lib.confirm = nil end
  if (e.Y or gd.key_pressed("DELETE")) and it and it.delete then
    it.delete()
  elseif e.A or gd.key_pressed("ENTER") or gd.key_pressed("SPACE") then
    menu_activate()
  elseif lib.confirm and (e.B or gd.key_pressed("BACKSPACE") or gd.key_pressed("ESCAPE")) then
    lib.confirm = nil
  elseif e.B or e.START or gd.key_pressed("ESCAPE") or gd.key_pressed("BACKSPACE") then
    menu_close()
  end
  -- a mouse, once the port has one (gd.mouse is not in the API yet)
  if gd.mouse and menu.open then
    local ok, m = pcall(gd.mouse)
    if ok and m and m.pressed then
      local kind, i = menu_hit(m.x, m.y)
      if kind == "tab" then set_tab(i)
      elseif kind == "row" then
        if i == cur_sel() then menu_activate() else set_sel(i) end
      end
    end
  end
  return true
end

local function ease(t) t = math.max(0, math.min(1, t)) return 1 - (1 - t) ^ 3 end

local function lab_menu_draw()
  if not menu.open or not kit_ok() then return end
  menu.hits = {}
  local o = ease(menu.t / 7)
  local items = tab_items()
  local sel = cur_sel()
  local tab = TABS[menu.tab]
  local slide = (1 - o) * 200

  -- the game, dimmed and pushed back; a glass slab down the left with a cyan edge
  quad(0, 0, 640, 480, alpha(INK, math.floor(0xB0 * o)))
  quad(-80 - slide, 0, 420, 480, alpha(GLASS_SOLID, 0xEE), SHEAR)
  quad(330 - slide, 0, 8, 480, ACCENT, SHEAR)
  quad(346 - slide, 0, 3, 480, alpha(ACCENT, 0x80), SHEAR)
  -- flourishes: the hitbox burst, the flask, hazard stripes
  img("lab_burst", 452, 236, 256, 256, alpha(ACCENT, 0x1C))
  icon("lab", -34, 300, 200, alpha(ACCENT, 0x1A))
  img("lab_stripes", 540, -8, 112, 112, alpha(ACCENT, 0x30))

  -- the tab name, huge, bleeding off the left edge
  stxt(-10 - (1 - o) * 60, 58, tab.name, "display", alpha(ACCENT, 0x48))
  stxt(20, 22, "GENO LAB", "label", BONE)
  txt(620, 22, string.format("%s  f %d", gd.paused() and "PAUSED" or "RUNNING", gd.match().frame), "caption",
    GOLD, "right")

  -- tabs
  local ty = 70 - (1 - o) * 30
  local tx = 44
  img("glyph_l", 18, ty + 6, 16, 16, MUTED)
  local tpad, tgap, tsum = 30, 8, 0
  for _, t in ipairs(TABS) do tsum = tsum + math.floor(measure(t.name, "label")) + 38 end
  if tsum > 570 then tpad, tgap = 16, 4 end -- stage D's DRILLS tab: squeeze rather than run off the edge
  for i, t in ipairs(TABS) do
    local tw = math.floor(measure(t.name, "label")) + tpad
    local on = i == menu.tab
    quad(tx, ty, tw, 28, on and GOLD or alpha(GLASS_SOLID, 0xF0), SHEAR)
    if on then quad(tx + 2, ty + 28, tw - 4, 3, GOLD_DK, SHEAR) end
    stxt(tx + tw / 2, ty + 22, t.name, "label", on and INK or BONE, "center")
    menu.hits[#menu.hits + 1] = { kind = "tab", i = i, x = tx, y = ty, w = tw, h = 28 }
    tx = tx + tw + tgap
  end
  img("glyph_r", tx + 2, ty + 6, 16, 16, MUTED)
  -- the frame ruler under the tabs
  for rx = 0, 639, 128 do img("lab_ruler", rx, 106, 128, 8, alpha(TICK, 0x90)) end

  -- rows: big, sheared; the selected one gold, pushed right, with a chevron
  local rx0, ry0, rw, rh, pitch = 44, 126, 276, 30, 36
  local wipe = menu.tab_t < 6
  local VIS = 8
  menu.scroll = menu.scroll or {}
  local first = menu.scroll[menu.tab] or 1
  if sel < first then first = sel end
  if sel > first + VIS - 1 then first = sel - VIS + 1 end
  if first > math.max(1, #items - VIS + 1) then first = math.max(1, #items - VIS + 1) end
  menu.scroll[menu.tab] = first
  if first > 1 then
    txt(rx0 + rw / 2, ry0 - 6, string.format("\u{2191} %d more", first - 1), "caption", MUTED, "center")
  end
  if first + VIS - 1 < #items then
    txt(rx0 + rw / 2, ry0 + VIS * pitch + 8, string.format("\u{2193} %d more", #items - (first + VIS - 1)),
      "caption", MUTED, "center")
  end
  for i, it in ipairs(items) do
    if i < first or i > first + VIS - 1 then goto continue end
    local ry = ry0 + (i - first) * pitch
    local a = ease((menu.t - i * 0.8) / 6)
    if wipe then a = math.min(a, ease((menu.tab_t - i * 0.6) / 5)) end
    local dx = -(1 - a) * 90
    local on = i == sel
    if on then dx = dx + 14 * ease(menu.sel_t / 3) end
    local x = rx0 + dx
    local val = it.value and it.value() or nil
    quad(x, ry, rw, rh, on and GOLD or alpha(GLASS_SOLID, 0xF0), SHEAR)
    if on then
      quad(x - 8, ry, 4, rh, GOLD, SHEAR)
      img("lab_chev", 16 + dx * 0.3, ry + 7, 16, 16, GOLD)
    end
    local ic = on and INK or ACCENT
    if it.toggle then toggle_icon(it.icon, x + 10, ry + 6, 18, it.toggle(), ic, on and GOLD or GLASS_SOLID)
    else icon(it.icon, x + 10, ry + 6, 18, ic) end
    stxt(x + 36, ry + 21, it.label:upper(), "row", on and INK or BONE, "left", rw - 50 - (val and 96 or 0))
    if val then stxt(x + rw - 12, ry + 21, val, "row", on and INK or ACCENT, "right", 110) end
    menu.hits[#menu.hits + 1] = { kind = "row", i = i, x = x, y = ry, w = rw, h = rh }
    ::continue::
  end

  -- the detail panel
  local it = items[sel]
  local px, py, pw, ph = 366 + (1 - o) * 80, 126, 254, 300
  panel(px, py, pw, ph)
  for _, b in ipairs({ { px - 5, py - 5, false, false }, { px + pw - 11, py - 5, true, false },
                       { px - 5, py + ph - 11, false, true }, { px + pw - 11, py + ph - 11, true, true } }) do
    img("lab_bracket", b[1], b[2], 16, 16, ACCENT, { flip_x = b[3], flip_y = b[4] })
  end
  if it then
    local val = it.value and it.value() or nil
    if it.toggle then toggle_icon(it.icon, px + 14, py + 16, 44, it.toggle(), ACCENT)
    else icon(it.icon, px + 14, py + 16, 44, ACCENT) end
    stxt(px + 70, py + 36, it.label:upper(), "title", BONE, "left", pw - 84)
    if val then stxt(px + 70, py + 60, val, "label", GOLD, "left", pw - 84) end
    local desc = type(it.desc) == "function" and it.desc() or it.desc or ""
    local y = py + 90
    for _, l in ipairs(wrap(desc, "body", pw - 28)) do
      txt(px + 14, y, l, "body", MUTED)
      y = y + 17
    end
    y = y + 8
    if it.preview == "mode" then
      -- a preview of the mode: its keys, as they will be live
      local md = mode()
      if #md.t + #md.a == 0 then txt(px + 14, y + 10, "No mode keys: only the globals.", "caption", DISABLED) end
      for _, t in ipairs(md.t) do
        if y > py + ph - 52 then break end
        local w = key_chip(px + 14, y, t.k)
        toggle_icon(t.icon, px + 20 + w, y, 16, tog[md.id][t.id], ACCENT)
        txt(px + 42 + w, y + 12, t.label, "caption", BONE)
        y = y + 20
      end
      for _, a in ipairs(md.a) do
        if y > py + ph - 52 then break end
        local w = key_chip(px + 14, y, a.k)
        icon(a.icon, px + 20 + w, y, 16, ACCENT)
        txt(px + 42 + w, y + 12, a.label, "caption", BONE)
        y = y + 20
      end
    end
    -- what the buttons do here, and the match key for the same thing
    local hint = it.adjust and (it.run and "A  do it    LEFT / RIGHT  change" or "LEFT / RIGHT  change")
      or (it.delete and "A  load    Y  delete" or "A  do it")
    quad(px + 10, py + ph - 48, pw - 20, 1, alpha(TICK, 0x80))
    txt(px + 14, py + ph - 32, hint, "caption", MUTED)
    if it.key then
      local w = key_chip(px + 14, py + ph - 24, it.key)
      txt(px + 20 + w, py + ph - 12, "does this in a match", "caption", DISABLED)
    end
  end

  -- a quick cyan wipe across the rows when the tab changes
  if wipe then
    local wx = -60 + menu.tab_t * 80
    quad(wx, 120, 40, 320, alpha(ACCENT, 0xC0), SHEAR)
    img("lab_fade", wx - 60, 120, 60, 320, alpha(ACCENT, 0x60), { flip_x = true })
  end

  -- the controls strip
  quad(0, 446, 640, 2, ACCENT)
  quad(0, 448, 640, 32, alpha(INK, 0xF4))
  local cx = 20
  for _, g in ipairs({ { "glyph_a", "SELECT" }, { "glyph_b", "BACK" }, { "glyph_l", nil }, { "glyph_r", "TAB" },
                       { "glyph_stick", "CHANGE" }, { "glyph_start", "CLOSE" } }) do
    local w = img(g[1], cx, 456, nil, nil, BONE)
    cx = cx + (w or 16) + 5
    if g[2] then cx = cx + stxt(cx, 470, g[2], "row", BONE) + 18 end
  end
  txt(620, 469, "keys: arrows  ENTER  Q/E  ESC", "caption", DISABLED, "right")
end

-- ---- the HUD (menu closed): mode chip, key strip, panels ------------------------------------
local function draw_notice()
  if not (notice and gd.time() < notice_until) then return end
  local w = measure(notice[1], "body") + 28
  local x, y = 320 - w / 2, 420
  quad(x, y, w, 22, GLASS, SHEAR)
  quad(x - 2, y, 4, 22, notice[2], SHEAR)
  txt(x + 14, y + 16, notice[1], "body", BONE)
end

local function draw_strip()
  local md = mode()
  local y = 456
  local paused = gd.paused()
  if md.id == "clean" then
    quad(8, y, 24, 18, paused and GOLD or alpha(ACCENT, 0xC0), SHEAR)
    icon(md.icon, 12, y + 1, 16, INK)
    return
  end
  local nm_w = measure(md.name, "row")
  local x = 8
  quad(0, y - 4, 640, 26, alpha(GLASS_SOLID, 0xC8))
  quad(x, y - 2, nm_w + 38, 22, paused and GOLD or ACCENT, SHEAR)
  icon(md.icon, x + 6, y + 1, 16, INK)
  stxt(x + 26, y + 15, md.name, "row", INK)
  x = x + nm_w + 50
  for _, t in ipairs(md.t) do
    local w = key_chip(x, y, t.k, gd.key(t.k))
    toggle_icon(t.icon, x + w + 3, y, 16, tog[md.id][t.id], ACCENT)
    x = x + w + 26
  end
  for _, a in ipairs(md.a) do
    local w = key_chip(x, y, a.k, gd.key(a.k))
    if a.state then toggle_icon(a.icon, x + w + 3, y, 16, STATES[a.state](), ACCENT)
    else icon(a.icon, x + w + 3, y, 16, ACCENT) end
    x = x + w + 26
  end
  -- the right end: status, mode, help
  local h = gd.history()
  local status = string.format("f %d  %s", gd.match().frame,
    h.replaying and string.format("REPLAY +%s s", secs(h.fwd)) or string.format("rewind %s s", secs(h.back)))
  local rx = 632
  txt(rx, y + 12, "help", "caption", MUTED, "right")
  rx = rx - measure("help", "caption") - 4 - key_w("F3")
  key_chip(rx, y, "F3")
  rx = rx - 8
  txt(rx, y + 12, "mode", "caption", MUTED, "right")
  rx = rx - measure("mode", "caption") - 4 - key_w("TAB")
  key_chip(rx, y, "TAB")
  rx = rx - 10
  if paused then
    local pw = measure("PAUSED", "caption") + 12
    quad(rx - pw, y, pw, 16, GOLD, SHEAR)
    txt(rx - pw / 2, y + 12, "PAUSED", "caption", INK, "center")
    rx = rx - pw - 8
  end
  if rx - measure(status, "caption") > x then txt(rx, y + 12, status, "caption", MUTED, "right") end
end

local function draw_help()
  local md = mode()
  local rows = #md.t + #md.a
  local gl = #GLOBAL_KEYS
  local h = 60 + math.max(rows, 1) * 20 + 34 + math.ceil(gl / 2) * 20 + 8
  local w, x = 480, 80
  local y = math.floor((440 - h) / 2)
  panel(x, y, w, h)
  quad(x + 10, y + 10, measure(md.name .. "  KEYS", "title") + 40, 30, ACCENT, SHEAR)
  icon(md.icon, x + 18, y + 15, 20, INK)
  stxt(x + 44, y + 33, md.name .. "  KEYS", "title", INK)
  txt(x + w - 14, y + 30, "F3 closes", "caption", MUTED, "right")
  local yy = y + 52
  if rows == 0 then
    txt(x + 20, yy + 12, "No mode keys: CLEAN is just the game.", "body", DISABLED)
    yy = yy + 20
  end
  for _, t in ipairs(md.t) do
    local kw = key_chip(x + 20, yy, t.k)
    toggle_icon(t.icon, x + 28 + kw, yy, 16, tog[md.id][t.id], ACCENT)
    txt(x + 52 + kw, yy + 12, t.label, "body", BONE)
    txt(x + w - 20, yy + 12, onoff(tog[md.id][t.id]), "caption", tog[md.id][t.id] and ACCENT or DISABLED, "right")
    yy = yy + 20
  end
  for _, a in ipairs(md.a) do
    local kw = key_chip(x + 20, yy, a.k)
    icon(a.icon, x + 28 + kw, yy, 16, ACCENT)
    txt(x + 52 + kw, yy + 12, a.label .. (a.rep and "  (hold repeats)" or ""), "body", BONE)
    yy = yy + 20
  end
  yy = yy + 8
  quad(x + 14, yy, w - 28, 1, alpha(TICK, 0x80))
  txt(x + 20, yy + 18, "EVERY MODE", "caption", ACCENT)
  yy = yy + 26
  for i, g in ipairs(GLOBAL_KEYS) do
    local cx = x + 20 + ((i - 1) % 2) * 230
    local cy = yy + math.floor((i - 1) / 2) * 20
    local kw = key_chip(cx, cy, g[1])
    txt(cx + kw + 8, cy + 12, g[2], "caption", BONE)
  end
end

local function draw_skeleton(p, color)
  local js = gd.joints(p.port)
  if js == nil then return end
  for _, j in ipairs(js) do
    if j.on and T("skel") then
      local par = j.parent >= 0 and js[j.parent + 1] or nil
      if par and par.on then gd.line(par.sx, par.sy, j.sx, j.sy, color) end
      gd.fill(j.sx - 1, j.sy - 1, 3, 3, 0xFFFFFFE0)
    end
  end
  if T("joints") then
    for _, j in ipairs(js) do
      if j.on then
        local s = tostring(j.index)
        quad(j.sx + 2, j.sy - 11, measure(s, "caption") + 4, 12, alpha(GLASS_SOLID, 0xB0))
        txt(j.sx + 4, j.sy - 2, s, "caption", BONE)
      end
    end
  end
end

local function draw_ecb(p)
  local e = p.ecb
  local pts = {}
  for _, k in ipairs({ "top", "right", "bottom", "left" }) do
    local sx, sy = gd.project(e[k].x, e[k].y, 0)
    if sx == nil then return end
    pts[#pts + 1] = { sx, sy }
  end
  for i = 1, 4 do
    local a, b = pts[i], pts[i % 4 + 1]
    gd.line(a[1], a[2], b[1], b[2], 0xF5902EFF)
  end
end

local function draw_hit_labels(p)
  for _, h in ipairs(p.hitboxes) do
    local sx, sy, vis = gd.project(h.x, h.y, h.z)
    if sx and vis then
      local s = string.format("#%d %.0f%% a%d", h.id, h.damage, h.angle)
      local c = HIT[h.id] or BONE
      quad(sx + 4, sy - 16, measure(s, "caption") + 10, 14, alpha(GLASS_SOLID, 0xD0), SHEAR)
      quad(sx + 2, sy - 16, 3, 14, c, SHEAR)
      txt(sx + 9, sy - 5, s, "caption", c)
    end
  end
end

local function f2(v) return string.format("%.2f", v) end

local function info_lines(p)
  local t = {}
  local function add(s, c) t[#t + 1] = { s, c or MUTED } end
  add(string.format("%s  f%d", p.motion_name, p.action_frame + 1), BONE)
  add(string.format("action %d  anim %s %.1f x%.2f", p.action, p.anim_name ~= "" and p.anim_name or "-",
    p.anim_frame_f, p.anim_rate))
  add(string.format("pos %s %s  %s", f2(p.x), f2(p.y), p.airborne and "air" or "ground"))
  add(string.format("vel %s %s  kb %s %s", f2(p.vx), f2(p.vy), f2(p.kb_vx), f2(p.kb_vy)))
  add(string.format("jumps %d/%d  wj %d  gv %s", p.jumps_left, p.jumps_max, p.walljumps_used, f2(p.ground_vel)))
  add(string.format("hitlag %.0f  hitstun %.0f  kb %.1f", p.hitlag, p.hitstun, p.kb_applied),
    (p.in_hitlag or p.in_hitstun) and GOLD or MUTED)
  add(string.format("intang %d  invinc %d  %s", p.intangible, p.invincible, p.body_state),
    (p.intangible > 0 or p.invincible > 0 or p.body_state ~= "normal") and OK or MUTED)
  add(string.format("shield %.1f  %s  ledge cd %d", p.shield, p.iasa and "IASA" or "no IASA", p.ledge_cooldown),
    p.iasa and OK or MUTED)
  add(string.format("ECB bottom %s  lock %d", f2(p.ecb.bottom.y), p.ecb_lock))
  return t
end

local function focus_first(list, n)
  local cols = {}
  for _, p in ipairs(list) do if p.port == focus then cols[1] = p end end
  for _, p in ipairs(list) do if p.port ~= focus and #cols < n then cols[#cols + 1] = p end end
  return cols
end

local function draw_info(list)
  local cols = focus_first(list, 2)
  local cw, x, y = 196, 8, 8
  local blocks, n = {}, 0
  for i, p in ipairs(cols) do
    blocks[i] = info_lines(p)
    n = math.max(n, #blocks[i])
  end
  panel(x, y, cw * #cols + 12, 26 + n * 13 + 6, "INFO")
  for i, p in ipairs(cols) do
    local cx = x + 10 + (i - 1) * cw
    txt(cx + (i == 1 and 50 or 0), y + 14, fighter_name(p.port), "caption", i == 1 and GOLD or PORT[p.port] or BONE,
      "left", cw - 60)
    for k, l in ipairs(blocks[i]) do txt(cx, y + 28 + (k - 1) * 13, l[1], "caption", l[2], "left", cw - 8) end
  end
end

local function draw_attrs(list)
  if attr_names == nil then
    local a = list[1] and gd.attrs(list[1].port)
    if a == nil then return end
    attr_names = {}
    for k in pairs(a) do attr_names[#attr_names + 1] = k end
    table.sort(attr_names)
  end
  local cols = {}
  for _, p in ipairs(focus_first(list, 3)) do cols[#cols + 1] = { p, gd.attrs(p.port) } end
  local rows = {}
  for _, name in ipairs(attr_names) do
    local v1 = cols[1] and cols[1][2][name]
    local differs = false
    for _, c in ipairs(cols) do if c[2][name] ~= v1 then differs = true end end
    rows[#rows + 1] = { name, differs }
  end
  table.sort(rows, function(a, b) if a[2] ~= b[2] then return a[2] end return a[1] < b[1] end)
  local MAXR = 24
  local w = 150 + #cols * 48
  local x, y = 632 - w, 8
  local n = math.min(#rows, MAXR)
  panel(x, y, w, 30 + n * 12 + 8, "ATTRIBUTES")
  for i, c in ipairs(cols) do
    txt(x + 150 + (i - 1) * 48 + 40, y + 14, "P" .. c[1].port, "caption", i == 1 and GOLD or BONE, "right")
  end
  for k = 1, n do
    local name, differs = rows[k][1], rows[k][2]
    local yy = y + 30 + (k - 1) * 12
    txt(x + 10, yy, name, "caption", differs and GOLD or DISABLED, "left", 136)
    for i, c in ipairs(cols) do
      txt(x + 150 + (i - 1) * 48 + 40, yy, string.format("%.4g", c[2][name]), "caption",
        differs and BONE or MUTED, "right")
    end
  end
  if #rows > MAXR then
    txt(x + w - 10, y + 30 + n * 12 + 2, "+" .. (#rows - MAXR) .. " the same", "caption", DISABLED, "right")
  end
end

local function draw_log()
  if #log_lines == 0 then return end
  local h = 26 + #log_lines * 13
  local x, y, w = 8, 446 - h, 420
  panel(x, y, w, h, "EVENTS")
  for i, l in ipairs(log_lines) do
    local yy = y + 28 + (i - 1) * 13
    txt(x + 44, yy, tostring(l[1]), "caption", DISABLED, "right")
    txt(x + 52, yy, l[2], "caption", l[3], "left", w - 62)
  end
end

local function draw_hit_data()
  local p = gd.player(focus)
  if p == nil then return end
  local n = math.max(1, #p.hitboxes)
  local x, y, w = 8, 8, 300
  panel(x, y, w, 30 + n * 15, "HITBOXES")
  txt(x + 86, y + 14, string.format("%s  %s f%d", fighter_name(p.port), p.motion_name, p.action_frame + 1),
    "caption", GOLD, "left", w - 96)
  if #p.hitboxes == 0 then
    txt(x + 12, y + 34, "no live hitboxes", "caption", DISABLED)
    return
  end
  for i, h in ipairs(p.hitboxes) do
    local yy = y + 22 + (i - 1) * 15
    img("lab_mk_hitbox", x + 10, yy + 2, 12, 12, HIT[h.id] or BONE)
    txt(x + 28, yy + 12, string.format("#%d  %.1f%%  a%d  kbg %d  bkb %d  wbk %d  r%.2f  b%d  %s", h.id,
      h.damage, h.angle, h.kbg, h.bkb, h.wbk, h.radius, h.bone, h.element_name), "caption",
      HIT[h.id] or BONE, "left", w - 38)
  end
end

local MARK_TEX = { iasa = "iasa", body_state = "invinc", hurtbox_state = "invinc", hurtboxes_state = "invinc",
  gfx = "gfx", visibility = "vis", model_state = "vis" }
local MARK_ABOVE = { iasa = true, invinc = true }

local function draw_timeline(p, x0, y, w, is_focus)
  local c = timeline_of(p)
  if c == nil then return end
  local len = math.max(c.len, 1)
  local sx = w / len
  local now = p.anim_frame_f + 1
  txt(x0, y + 10, string.format("%s  %s", fighter_name(p.port), c.tl.motion_name), "caption",
    is_focus and GOLD or BONE, "left", w * 0.6)
  local iasa
  for _, e in ipairs(c.marks) do if e.name == "iasa" then iasa = e.frame break end end
  txt(x0 + w, y + 10, string.format("f %.0f / %d%s", now, math.floor(len + 0.5), iasa and ("   IASA " .. iasa) or ""),
    "caption", MUTED, "right")
  local by = y + 24
  img("lab_tl_cap_l", x0 - 4, by, 4, 8, TRACK)
  img("lab_tl_cap_r", x0 + w, by, 4, 8, TRACK)
  quad(x0, by, w, 8, TRACK)
  for f = 5, len, 5 do quad(x0 + (f - 1) * sx, by + 9, 1, f % 10 == 0 and 4 or 2, TICK) end
  for _, hw in ipairs(c.windows) do
    quad(x0 + (hw.from - 1) * sx, by + 1, math.max(2, (hw.to - hw.from + 1) * sx), 6, HIT[hw.id] or BONE)
  end
  for _, e in ipairs(c.marks) do
    local kind = MARK_TEX[e.name] or (e.name:find("sfx") and "sfx")
    if kind then
      local mx = x0 + (e.frame - 1) * sx - 4
      local my = MARK_ABOVE[kind] and by - 10 or by + 12
      img("lab_mk_" .. kind, mx + 1, my + 1, 8, 8, INK)
      img("lab_mk_" .. kind, mx, my, 8, 8, MARK[kind])
    end
  end
  local px = x0 + (now - 1) * sx
  img("lab_tl_playhead", px - 4 + 1, by - 6 + 1, 8, 16, INK)
  img("lab_tl_playhead", px - 4, by - 6, 8, 16, GOLD)
  local parts = {}
  for _, hw in ipairs(c.windows) do
    parts[#parts + 1] = string.format("f%d-%d #%d %d%% a%d", hw.from, hw.to, hw.id, hw.dmg, hw.angle)
  end
  if #parts > 0 then txt(x0, y + 44, table.concat(parts, "   "), "caption", MUTED, "left", w) end
end

local function draw_frames(list)
  local a = gd.player(focus)
  if a == nil then return end
  -- the action / frame chip, top-left
  local h = gd.history()
  local s = string.format("%s   %s  f%d", fighter_name(a.port), a.motion_name, a.action_frame + 1)
  local extra = h.replaying and string.format("replaying  +%s s  G live", secs(h.fwd))
    or string.format("rewind %s / %d s", secs(h.back), cfg.history_s)
  local w = measure(s, "body") + measure(extra, "caption") + 44
  quad(8, 8, w, 22, GLASS, SHEAR)
  quad(6, 8, 4, 22, GOLD, SHEAR)
  txt(18, 24, s, "body", BONE)
  txt(8 + w - 10, 23, extra, "caption", MUTED, "right")
  if T("net") then draw_rollbacks() end
  if not T("timeline") then return end
  local others = {}
  for _, p in ipairs(list) do if p.port ~= focus then others[#others + 1] = p end end
  local n = 1 + math.min(1, #others)
  local ph = 14 + n * 52
  local py = 444 - ph
  panel(8, py, 624, ph, "TIMELINE")
  draw_timeline(a, 22, py + 10, 596, true)
  if others[1] then draw_timeline(others[1], 22, py + 62, 596, false) end
end

local LE = {} -- stage E: the creator tools (a function of its own: the main chunk has a 200-local limit)
local function stage_e()
-- =================================================================================================
-- ---- stage E: the creator tools (docs/geno.md 14.11) -------------------------------------------
-- MOVES (the state browser), LAUNCH (the knockback preview), A/B (two variants on the same inputs),
-- the frame-data export + diff (TOOLS tab, `lab export`, MELEE_LAB_BATCH) and the rollback strip
-- (FRAMES, N).
-- =================================================================================================
local WAIT_MOTION = 14 -- ftCo_MS_Wait
local BATCH_SLOT = 3    -- the export's neutral state (the menu's quick slot 3)
local function hexid(id) return string.format("%03X", id) end
local function sorted_keys(t)
  local k = {}
  for key in pairs(t) do k[#k + 1] = key end
  table.sort(k)
  return k
end
local function windows_of(frames) -- {f = true} -> "2-3 9-12"
  local ks, out, a, b = sorted_keys(frames), {}, nil, nil
  for _, f in ipairs(ks) do
    if a and f == b + 1 then b = f
    else
      if a then out[#out + 1] = a == b and tostring(a) or (a .. "-" .. b) end
      a, b = f, f
    end
  end
  if a then out[#out + 1] = a == b and tostring(a) or (a .. "-" .. b) end
  return table.concat(out, " ")
end
local function air_name(name)
  return (name:find("Air") and not name:find("Landing")) or name:find("Fall") or name:find("Glide")
    or (name:find("Jump") and not name:find("Squat")) or false
end

-- ---- the specials, entered through their input ------------------------------------------------
-- A vanilla or m-ex special's entry function sets up more than its motion (Fox's blaster, a
-- charge, an article): a bare motion change into it can crash the game. So specials are played
-- (and exported) by pressing B on the pad from a neutral stance: the game enters them itself.
-- Geno states are entered through Geno's own entry (their behaviour's enter routine).
local INPUTS = {
  { key = "B", name = "Neutral B", x = 0, y = 0 }, { key = "B>", name = "Side B", x = 1, y = 0 },
  { key = "B^", name = "Up B", x = 0, y = 1 }, { key = "Bv", name = "Down B", x = 0, y = -1 },
}
local function input_entries()
  local out = {}
  for air = 0, 1 do
    for i, sp in ipairs(INPUTS) do
      out[#out + 1] = { id = -(i + air * 4), name = (air == 1 and "Air " or "") .. sp.name, group = "input",
        anim_id = 0, anim_name = "", input = sp, air = air == 1 }
    end
  end
  return out
end
-- where a special's sequence is over: back to standing, falling, landing (or helpless)
local END_STATES = { Wait = true, Fall = true, FallF = true, FallB = true, FallAerial = true, FallAerialF = true,
  FallAerialB = true, Landing = true, FallSpecial = true, FallSpecialF = true, FallSpecialB = true,
  LandingFallSpecial = true, Squat = true, SquatWait = true }
local function motion_by_name(port, name, def)
  for _, m in ipairs(gd.motion_list(port) or {}) do if m.name == name then return m.id end end
  return def
end
local function input_spec(port, sp)
  local p = gd.player(port)
  local face = p and p.facing or 1
  return { buttons = "B", x = sp.x * 127 * face, y = sp.y * 127 }
end
local function mname(m) return m.input and m.input.key or hexid(m.id) end

-- ---- MOVES: the state browser -------------------------------------------------------------------
local FILTERS = { "ALL", "ATTACKS", "COMMON", "SPECIAL", "M-EX", "GENO" }
local GROUP_COL = { common = MUTED, special = ACCENT, mex = 0xC77DFFFF, geno = GOLD, input = OK }
local SPEEDS = { 1, 0.5, 0.25, 0.1 }
local br = { key = nil, all = {}, filter = 1, sel = 1, first = 1, speed = 1, play = nil, acc = 0,
  pending = nil, text = "", hits = {}, tl = {} }

local function br_list()
  local p = gd.player(focus)
  if p == nil then return {} end
  local key = focus .. ":" .. tostring(p.char_name)
  if br.key ~= key then
    br.key = key
    br.all = input_entries()
    for _, m in ipairs(gd.motion_list(focus) or {}) do br.all[#br.all + 1] = m end
    br.sel, br.first, br.tl = 1, 1, {}
  end
  local f, out = FILTERS[br.filter], {}
  for _, m in ipairs(br.all) do
    local ok = f == "ALL" or (f == "COMMON" and m.group == "common")
      or (f == "SPECIAL" and (m.group == "special" or m.group == "input"))
      or (f == "M-EX" and m.group == "mex") or (f == "GENO" and m.group == "geno")
      or (f == "ATTACKS" and (m.name:find("^Attack") ~= nil or m.group ~= "common"))
    if ok and br.text ~= "" then ok = m.name:lower():find(br.text, 1, true) ~= nil end
    if ok then out[#out + 1] = m end
  end
  if br.sel > #out then br.sel = math.max(1, #out) end
  return out
end

local function br_static(m) -- the state's own script, analysed (cached per id)
  if m.input then return nil end
  local c = br.tl[m.id]
  if c == nil then
    local tl = gd.timeline(focus, m.id)
    if tl then
      local windows, marks, len = analyse(tl)
      local iasa
      for _, e in ipairs(marks) do if e.name == "iasa" then iasa = e.frame break end end
      c = { tl = tl, windows = windows, len = len, iasa = iasa }
    else
      c = false
    end
    br.tl[m.id] = c
  end
  return c or nil
end

local function br_start(m)
  if not offline() or gd.player(focus) == nil then return end
  if m.input == nil and (m.group == "special" or m.group == "mex") then
    say("Specials play through their input: the B rows (a bare entry skips the special's own setup)", DANGER)
    return
  end
  if m.input == nil and (m.anim_id < 0 or m.anim_name == "") then
    -- a common row this fighter never uses (Fox's Attack13): entering it leaves the game in a state
    -- it asserts on
    say(m.name .. ": this fighter has no animation for it", DANGER)
    br.play = nil
    return
  end
  if T("neutral") and menu.reset_saved then
    pcall(gd.loadstate, RESET_SLOT) -- everyone back to the match start: the same neutral every loop
    br.pending = { m = m, wait = 1 }
  elseif T("neutral") then
    gd.set_motion(focus, WAIT_MOTION, 1)
    br.pending = { m = m, wait = 1 }
  else
    br.pending = { m = m, wait = 0 }
  end
end

local function br_tick()
  local pd = br.pending
  if pd then
    if pd.wait > 0 then pd.wait = pd.wait - 1 return end
    if pd.m.input then
      if pd.m.air and not pd.lifted then
        pd.lifted, pd.wait = true, 1
        gd.set_motion(focus, motion_by_name(focus, "Fall", 29), 1, 1, 40)
        return
      end
      br.pending = nil
      local p = gd.player(focus)
      gd.input(focus, input_spec(focus, pd.m.input), 3)
      br.play = { m = pd.m, port = focus, seen = false, t = 0, base = p and p.action }
      if br.speed >= 1 then gd.resume() else gd.step(1) end
      return
    end
    br.pending = nil
    local ok, why = gd.set_motion(focus, pd.m.id, 1, 1, air_name(pd.m.name) and 40 or 0)
    if not ok then
      say("Play: " .. tostring(why), DANGER)
      br.play = nil
      return
    end
    br.play = { m = pd.m, port = focus, seen = false, t = 0 }
    if br.speed >= 1 then gd.resume() end
    return
  end
  if br.play and br.speed < 1 and not menu.open then
    br.acc = br.acc + br.speed
    if br.acc >= 1 then br.acc = br.acc - 1 gd.step(1) end
  end
end

local function br_frame()
  local pl = br.play
  if pl == nil or br.pending then return end
  local p = gd.player(pl.port)
  if p == nil then br.play = nil return end
  pl.t = pl.t + 1
  if pl.m.input then
    if not pl.seen then
      if p.action ~= pl.base then pl.seen = true elseif pl.t > 20 then br.play = nil say("No special came out", DANGER) end
      return
    end
    if not END_STATES[gd.motion_name(p.action, pl.port)] and pl.t < 600 then return end
    if tog.moves.loop then br_start(pl.m) else br.play = nil end
    return
  end
  if p.action == pl.m.id then
    pl.seen = true
  elseif pl.seen or pl.t > 30 then
    if tog.moves.loop then br_start(pl.m) else br.play = nil end
  end
end

ACTIONS.br_up = function() br.sel = math.max(1, br.sel - 1) end
ACTIONS.br_down = function() br.sel = math.min(#br_list(), br.sel + 1) end
ACTIONS.br_play = function()
  local m = br_list()[br.sel]
  if m then br_start(m) say(string.format("%s  %s", hexid(m.id), m.name)) end
end
ACTIONS.br_filter = function()
  br.filter = br.filter % #FILTERS + 1
  br.sel, br.first = 1, 1
  say("Filter: " .. FILTERS[br.filter])
end
ACTIONS.br_slower = function()
  for i, s in ipairs(SPEEDS) do if s == br.speed and i < #SPEEDS then br.speed = SPEEDS[i + 1] break end end
  say(string.format("Speed x%g", br.speed))
end
ACTIONS.br_faster = function()
  for i, s in ipairs(SPEEDS) do if s == br.speed and i > 1 then br.speed = SPEEDS[i - 1] break end end
  if br.speed >= 1 and br.play then gd.resume() end
  say(string.format("Speed x%g", br.speed))
end
ACTIONS.br_stop = function() br.play, br.pending = nil, nil gd.pause() say("Stopped") end
STATES.br_playing = function() return br.play ~= nil end

local function draw_moves()
  local list = br_list()
  local p = gd.player(focus)
  if p == nil then return end
  local ROWS = 18
  local x, y, w = 8, 8, 318
  local h = 44 + ROWS * 15
  panel(x, y, w, h, "MOVES")
  txt(x + 70, y + 14, string.format("%s   %s  %d / %d", fighter_name(focus), FILTERS[br.filter], #list, #br.all),
    "caption", GOLD, "left", w - 80)
  if br.text ~= "" then txt(x + w - 10, y + 14, "\u{201C}" .. br.text .. "\u{201D}", "caption", BONE, "right") end
  if br.sel < br.first then br.first = br.sel end
  if br.sel > br.first + ROWS - 1 then br.first = br.sel - ROWS + 1 end
  br.hits = {}
  for i = br.first, math.min(#list, br.first + ROWS - 1) do
    local m = list[i]
    local yy = y + 26 + (i - br.first) * 15
    local on = i == br.sel
    if on then quad(x + 6, yy, w - 12, 14, GOLD, SHEAR) end
    quad(x + 8, yy + 2, 3, 10, GROUP_COL[m.group] or MUTED)
    local col = on and INK or BONE
    txt(x + 16, yy + 11, mname(m), "caption", on and INK or DISABLED)
    txt(x + 46, yy + 11, m.name, "caption", col, "left", 150)
    txt(x + w - 12, yy + 11, m.anim_name ~= m.name and m.anim_name or "", "caption", on and INK or DISABLED, "right", 110)
    if br.play and br.play.m.id == m.id then icon("lab_play", x + w - 128, yy + 1, 12, on and INK or OK) end
    br.hits[#br.hits + 1] = { kind = "move", i = i, x = x + 6, y = yy, w = w - 12, h = 14 }
  end
  -- the legend: a colour per group
  local lx = x + 12
  for _, g in ipairs({ { "input", "INPUT" }, { "common", "COMMON" }, { "special", "SPECIAL" }, { "mex", "M-EX" },
                       { "geno", "GENO" } }) do
    quad(lx, y + h - 13, 3, 9, GROUP_COL[g[1]])
    lx = lx + 6 + txt(lx + 6, y + h - 5, g[2], "caption", MUTED) + 10
  end
  -- the detail panel: the selected state's own script
  local m = list[br.sel]
  if m == nil then return end
  local c = br_static(m)
  local dx, dw = 334, 298
  local dh = 150
  panel(dx, 8, dw, dh, "STATE")
  stxt(dx + 12, 44, m.name, "label", BONE, "left", dw - 24)
  if m.input then
    txt(dx + 12, 60, string.format("the pad: B%s%s, from %s", m.input.x ~= 0 and " + forward" or "",
      m.input.y > 0 and " + up" or m.input.y < 0 and " + down" or "", m.air and "the air" or "standing"), "caption",
      MUTED, "left", dw - 24)
    txt(dx + 12, 76, "the game enters the special itself, with its own setup", "caption", BONE, "left", dw - 24)
  else
    txt(dx + 12, 60, string.format("id %d (0x%s)   %s   anim %s", m.id, hexid(m.id), m.group:upper(),
      m.anim_id >= 0 and m.anim_id or "-"), "caption", MUTED, "left", dw - 24)
  end
  if m.input then
  elseif c then
    txt(dx + 12, 76, string.format("script %d frames%s%s", math.floor(c.len + 0.5),
      c.iasa and ("   IASA " .. c.iasa) or "", c.tl.stop ~= "end" and ("   (" .. c.tl.stop .. ")") or ""),
      "caption", BONE, "left", dw - 24)
    local yy = 92
    if #c.windows == 0 then txt(dx + 12, yy, "no hitboxes in its script", "caption", DISABLED) end
    for k, hw in ipairs(c.windows) do
      if k > 4 then txt(dx + 12, yy, string.format("+%d more", #c.windows - 4), "caption", DISABLED) break end
      img("lab_mk_hitbox", dx + 12, yy - 9, 10, 10, HIT[hw.id] or BONE)
      txt(dx + 26, yy, string.format("f%d-%d  #%d  %s%%  a%d  kbg %d  bkb %d  wbk %d", hw.from, hw.to, hw.id,
        tostring(hw.dmg), hw.angle, hw.kbg, hw.bkb, hw.wbk), "caption", HIT[hw.id] or BONE, "left", dw - 38)
      yy = yy + 13
    end
  else
    txt(dx + 12, 76, "no script for this state", "caption", DISABLED)
  end
  if m.input == nil and (m.group == "special" or m.group == "mex") then
    txt(dx + 12, 8 + dh - 22, "plays through its input (a B row)", "caption", DISABLED)
  end
  local st = string.format("x%g   %s%s", br.speed, T("loop") and "LOOP" or "ONCE", T("neutral") and "   FROM NEUTRAL" or "")
  txt(dx + dw - 12, 8 + dh - 8, st, "caption", br.play and OK or MUTED, "right")
  if T("timeline") then
    local ph = 66
    panel(8, 444 - ph, 624, ph, "TIMELINE")
    draw_timeline(p, 22, 444 - ph + 10, 596, true)
  end
end

-- ---- LAUNCH: the knockback preview --------------------------------------------------------------
local DIS = { "none", "in", "out", "survival" }
local kbv = { sel = 1, di = 1, pct = nil, victim = nil, check = nil, last = nil, cache = nil, key = nil }

local function kb_victim()
  if kbv.victim and kbv.victim ~= focus and gd.player(kbv.victim) then return kbv.victim end
  for _, p in ipairs(gd.players()) do if p.port ~= focus then return p.port end end
end

-- a live hitbox on the frame shown, else the ones the script opens next (the hit about to connect)
local function kb_hitboxes(p)
  local list = {}
  for _, h in ipairs(p.hitboxes) do
    list[#list + 1] = { live = true, id = h.id, damage = h.damage, angle = h.angle, kbg = h.kbg, bkb = h.bkb,
      wbk = h.wbk, element = h.element_name }
  end
  if #list == 0 then
    local c = timeline_of(p)
    local now = p.anim_frame_f + 1
    local nextf
    if c then
      for _, hw in ipairs(c.windows) do
        if hw.from >= now and (nextf == nil or hw.from == nextf) then
          nextf = hw.from
          list[#list + 1] = { live = false, from = hw.from, id = hw.id, damage = hw.dmg, angle = hw.angle,
            kbg = hw.kbg, bkb = hw.bkb, wbk = hw.wbk, element = hw.element }
        end
      end
    end
  end
  return list
end

local function kb_predict()
  local a, vport = gd.player(focus), kb_victim()
  local v = vport and gd.player(vport)
  if a == nil or v == nil then return nil end
  local hbs = kb_hitboxes(a)
  if #hbs == 0 then return nil, a, v, hbs end
  if kbv.sel > #hbs then kbv.sel = 1 end
  local h = hbs[kbv.sel]
  local key = table.concat({ gd.match().frame, focus, vport, kbv.sel, kbv.di, tostring(kbv.pct), h.id, h.damage,
    h.angle, v.x, v.y, v.percent }, ":")
  if kbv.key ~= key then
    kbv.key = key
    local ok, r = pcall(gd.kb_preview, vport, { attacker = focus, damage = h.damage, angle = h.angle, kbg = h.kbg,
      bkb = h.bkb, wbk = h.wbk, di = DIS[kbv.di], percent = kbv.pct })
    kbv.cache = ok and r or nil
  end
  return kbv.cache, a, v, hbs, h
end

-- the check: when a real hit lands, predict it from the hit's own numbers and the victim's state
-- before it, then follow the real flight to the end of hitstun and measure the error
local function kb_on_hit(attacker, victim, info)
  if not tog.launch.check or attacker == nil or info.angle == nil or not offline() then return end
  local v = gd.player(victim)
  if v == nil then return end
  local pre = math.max(0, v.percent - (info.dealt or 0))
  local ok, pred = pcall(gd.kb_preview, victim, { attacker = attacker, damage = info.dealt, angle = info.angle,
    kbg = info.kbg, bkb = info.bkb, wbk = info.wbk, percent = pre, x = v.x, y = v.y, di = "none", extra = 0 })
  if not ok or pred == nil then return end
  kbv.check = { victim = victim, pred = pred, real = {}, kb_real = v.kb_last, pre = pre, hit = info,
    frame = gd.match().frame, hitstun_real = nil }
end

local function kb_check_finish(c)
  local n, maxe, sum = math.min(#c.real, #c.pred.points), 0, 0
  for i = 1, n do
    local dx, dy = c.real[i][1] - c.pred.points[i][1], c.real[i][2] - c.pred.points[i][2]
    local d = math.sqrt(dx * dx + dy * dy)
    sum = sum + d
    if d > maxe then maxe = d end
  end
  kbv.last = { n = n, max = maxe, mean = n > 0 and sum / n or 0, kb_pred = c.pred.kb, kb_real = c.kb_real,
    hs_pred = c.pred.hitstun, hs_real = c.hitstun_real, frame = c.frame, victim = c.victim, dmg = c.hit.dealt,
    angle = c.hit.angle, pct = c.pre }
  kbv.check = nil
  gd.log(string.format("lab kbcheck: f%d P%d at %.0f%% hit %.1f%% a%d: kb pred %.3f real %.3f; hitstun pred %d real %s;"
    .. " flight %d frames, max err %.4f, mean err %.4f", kbv.last.frame, kbv.last.victim, kbv.last.pct, kbv.last.dmg,
    kbv.last.angle, kbv.last.kb_pred, kbv.last.kb_real or -1, kbv.last.hs_pred, tostring(kbv.last.hs_real), n, maxe,
    kbv.last.mean))
end

local function kb_frame()
  local c = kbv.check
  if c == nil then return end
  local v = gd.player(c.victim)
  if v == nil then kbv.check = nil return end
  if v.in_hitlag then return end
  if c.hitstun_real == nil then c.hitstun_real = math.floor(v.hitstun + 0.5) + 1 end
  if (c.kb_real or 0) == 0 then c.kb_real = v.kb_last end
  c.real[#c.real + 1] = { v.x, v.y }
  if #c.real >= c.pred.hitstun or (not v.airborne and #c.real > 1) or not v.in_hitstun then kb_check_finish(c) end
end

ACTIONS.kb_prev = function() kbv.sel = math.max(1, kbv.sel - 1) end
ACTIONS.kb_next = function() kbv.sel = kbv.sel + 1 end
ACTIONS.kb_di = function() kbv.di = kbv.di % #DIS + 1 say("DI: " .. DIS[kbv.di]:upper()) end
ACTIONS.kb_pct_down = function()
  local v = gd.player(kb_victim() or 2)
  kbv.pct = math.max(0, (kbv.pct or math.floor((v and v.percent or 0) / 10) * 10) - 10)
end
ACTIONS.kb_pct_up = function()
  local v = gd.player(kb_victim() or 2)
  kbv.pct = math.min(999, (kbv.pct or math.floor((v and v.percent or 0) / 10) * 10) + 10)
end
ACTIONS.kb_pct_live = function() kbv.pct = nil say("Percent: live") end
ACTIONS.kb_victim = function()
  local ports = {}
  for _, p in ipairs(gd.players()) do if p.port ~= focus then ports[#ports + 1] = p.port end end
  if #ports == 0 then return end
  local cur, k = kb_victim(), 1
  for i, pt in ipairs(ports) do if pt == cur then k = i end end
  kbv.victim = ports[k % #ports + 1]
  say("Victim: " .. fighter_name(kbv.victim))
end
STATES.kb_di = function() return kbv.di ~= 1 end

local function draw_launch()
  local r, a, v, hbs, h = kb_predict()
  local x, y, w = 8, 8, 312
  if a == nil or v == nil then
    panel(x, y, w, 40, "LAUNCH")
    txt(x + 12, y + 32, "needs two fighters: the focused one hits the other", "caption", DISABLED)
    return
  end
  if T("arc") and r and r.points then
    local c = HIT[h.id] or BONE
    local lx, ly = gd.project(v.x, v.y, 0)
    for i, pt in ipairs(r.points) do
      local sx, sy = gd.project(pt[1], pt[2], 0)
      if sx and lx then gd.line(lx, ly, sx, sy, c) end
      if sx and i % 10 == 0 then gd.fill(sx - 1, sy - 1, 3, 3, BONE) end
      lx, ly = sx, sy
    end
    if lx then
      icon("lab_step", lx - 6, ly - 6, 12, c)
      txt(lx + 8, ly + 4, string.format("f%d", #r.points), "caption", c)
    end
    if r.blast then
      local kx, ky = gd.project(r.blast.x, r.blast.y, 0)
      if kx then
        icon("lab_ko", kx - 12, ky - 12, 24, DANGER)
        txt(kx + 14, ky + 4, string.format("KO f%d", r.blast.frame), "caption", DANGER)
      end
    end
  end
  local lines = 11
  panel(x, y, w, 30 + lines * 13, "LAUNCH")
  txt(x + 76, y + 14, string.format("%s  >  %s", fighter_name(focus), fighter_name(v.port)), "caption", GOLD,
    "left", w - 86)
  local yy = y + 30
  local function line(s, c) txt(x + 12, yy, s, "caption", c or BONE, "left", w - 24) yy = yy + 13 end
  if h == nil then
    line("no live hitbox and none ahead in this move", DISABLED)
  else
    line(string.format("%s #%d  (%d of %d)  %s%%  a%d  kbg %d  bkb %d  wbk %d", h.live and "LIVE" or ("NEXT f" .. h.from),
      h.id, kbv.sel, #hbs, tostring(h.damage), h.angle, h.kbg, h.bkb, h.wbk), HIT[h.id] or BONE)
  end
  if r then
    line(string.format("victim %.0f%%%s  weight %g   DI %s", r.percent, kbv.pct and " (set)" or "", r.weight,
      DIS[kbv.di]:upper()), MUTED)
    line(string.format("knockback %.2f", r.kb), BONE)
    line(string.format("angle %.1f%s", r.angle, kbv.di ~= 1 and string.format("  ->  %.1f with DI", r.angle_di) or ""), BONE)
    line(string.format("hitstun %d frames   %s", r.hitstun, r.tumble and "TUMBLE" or ("no tumble (level " .. r.level .. ")")),
      r.tumble and GOLD or MUTED)
    if r.blast then
      line(string.format("KO: crosses the %s blast zone at f%d%s", r.blast.side, r.blast.frame,
        r.blast.after_hitstun and " (after hitstun)" or ""), DANGER)
    else
      line("survives (no blast zone while the launch lasts)", OK)
    end
  else
    yy = yy + 13 * 5
  end
  local L = kbv.last
  if kbv.check then
    line(string.format("checking a real hit: %d / %d frames", #kbv.check.real, kbv.check.pred.hitstun), ACCENT)
  elseif L then
    line(string.format("last real hit f%d: kb %.2f vs %.2f, hitstun %d vs %s", L.frame, L.kb_pred, L.kb_real or -1,
      L.hs_pred, tostring(L.hs_real)), ACCENT)
    line(string.format("flight error over %d f: max %.3f  mean %.3f units", L.n, L.max, L.mean), ACCENT)
  else
    line(tog.launch.check and "hit them for real (K) and the error shows here" or "check off (K)", DISABLED)
  end
end

-- ---- A/B: two variants on the same inputs -------------------------------------------------------
-- A is recorded live; B re-simulates the same logged input from the same frame (sequential,
-- exact: the rewind's keyframe + input log). B = "reload" (geno.json / overlays as they are on
-- disk now, through the hot reload), "same" (the same data: a determinism check, 0 differences
-- expected) or "mirror" (two fighters live: P2 gets P1's pad, compared from their own start).
local AB_FILE, AB_PENDING = "ab_a.txt", "ab_pending.txt"
local ab = { rec = nil, a = nil, b = nil, res = nil, view = nil }

local function ab_sample(tr)
  local row = {}
  for _, p in ipairs(gd.players()) do
    local mask = 0
    for _, h in ipairs(p.hitboxes) do mask = mask | (1 << h.id) end
    row[p.port] = { p.x, p.y, p.action, mask, p.facing or 1 }
  end
  tr[gd.match().frame] = row
end

local function ab_write(t)
  local out = { string.format("%d %d %s", t.from, t.to, t.kind or "live") }
  for _, f in ipairs(sorted_keys(t.tr)) do
    for port, s in pairs(t.tr[f]) do
      out[#out + 1] = string.format("%d %d %.6f %.6f %d %d", f, port, s[1], s[2], s[3], s[4])
    end
  end
  pcall(gd.data_write, AB_FILE, table.concat(out, "\n") .. "\n")
end

local function ab_read()
  local ok, text = pcall(gd.data_read, AB_FILE)
  if not ok or text == nil then return nil end
  local t = { tr = {} }
  local first = true
  for l in text:gmatch("[^\n]+") do
    if first then
      local a, b, k = l:match("^(%-?%d+) (%-?%d+) (%S+)")
      t.from, t.to, t.kind = tonumber(a), tonumber(b), k
      first = false
    else
      local f, port, x, y, act, mask = l:match("^(%-?%d+) (%d+) (%S+) (%S+) (%-?%d+) (%d+)")
      if f then
        f = tonumber(f)
        t.tr[f] = t.tr[f] or {}
        t.tr[f][tonumber(port)] = { tonumber(x), tonumber(y), tonumber(act), tonumber(mask) }
      end
    end
  end
  return t.from and t or nil
end

-- compare A's port pa with B's port pb frame by frame (rel: from each one's own start, x as faced)
local function ab_compare(A, B, pa, pb, rel)
  local frames, first, why, maxd, nact, nhb, npos = {}, nil, nil, 0, 0, 0, 0
  local a0, b0
  for _, f in ipairs(sorted_keys(A.tr)) do
    local ra, rb = A.tr[f][pa], B.tr[f] and B.tr[f][pb]
    if ra and rb then
      a0, b0 = a0 or ra, b0 or rb
      local ax, ay, bx, by = ra[1], ra[2], rb[1], rb[2]
      if rel then
        ax, ay = (ax - a0[1]) * (a0[5] or 1), ay - a0[2]
        bx, by = (bx - b0[1]) * (b0[5] or 1), by - b0[2]
      end
      local d = math.sqrt((ax - bx) ^ 2 + (ay - by) ^ 2)
      local act, hb, pos = ra[3] ~= rb[3], ra[4] ~= rb[4], d > 0.0001
      if d > maxd then maxd = d end
      if act then nact = nact + 1 end
      if hb then nhb = nhb + 1 end
      if pos then npos = npos + 1 end
      if first == nil and (act or hb or pos) then
        first = f
        why = (pos and "position" or "") .. (act and ((pos and ", " or "") .. "action") or "")
          .. (hb and (((pos or act) and ", " or "") .. "hitboxes") or "")
      end
      frames[#frames + 1] = { f = f, a = ra, b = rb, d = d, act = act, hb = hb, pos = pos }
    end
  end
  return { frames = frames, first = first, why = why, maxd = maxd, nact = nact, nhb = nhb, npos = npos,
    pa = pa, pb = pb }
end

local function ab_finish()
  local A, B = ab.a, ab.b
  if A == nil or B == nil then return end
  local res = { kind = B.kind, per = {} }
  if B.kind == "mirror" then
    res.per[1] = ab_compare(A, B, 1, 2, true)
  else
    for _, p in ipairs(gd.players()) do res.per[#res.per + 1] = ab_compare(A, B, p.port, p.port, false) end
  end
  ab.res, ab.view = res, nil
  for _, r in ipairs(res.per) do
    gd.log(string.format("lab ab: %s P%d vs P%d over %d frames: first divergence %s (%s); position differs on %d "
      .. "(max %.4f), action on %d, hitboxes on %d", res.kind, r.pa, r.pb, #r.frames, tostring(r.first or "none"),
      r.why or "-", r.npos, r.maxd, r.nact, r.nhb))
  end
  say(res.per[1] and res.per[1].first and ("A/B: they part at f" .. res.per[1].first) or "A/B: identical",
    res.per[1] and res.per[1].first and GOLD or OK)
end

local function ab_stop_record()
  local r = ab.rec
  ab.rec = nil
  gd.pause()
  r.to = gd.match().frame
  if r.kind == "mirror" then
    set_mirror(false)
    local A, B = { from = r.from, to = r.to, kind = "mirror", tr = {} }, { from = r.from, to = r.to, kind = "mirror", tr = {} }
    for f, row in pairs(r.tr) do A.tr[f] = { [1] = row[1] } B.tr[f] = { [2] = row[2] } end
    ab.a, ab.b = A, B
    ab_finish()
    return
  end
  ab.a, ab.b, ab.res = r, nil, nil
  ab_write(r)
  say(string.format("A recorded: f%d-%d. Now B: reload (B) or same data (C)", r.from, r.to))
end

ACTIONS.ab_record = function()
  if ab.rec then ab_stop_record() return end
  if not offline() then return end
  ensure_history()
  ab.rec = { from = gd.match().frame, tr = {}, kind = "live" }
  ab.res, ab.b = nil, nil
  gd.resume()
  say("Recording A: play, then R again")
end
ACTIONS.ab_mirror = function()
  if ab.rec then ab_stop_record() return end
  if not offline() then return end
  set_mirror(true)
  ab.rec = { from = gd.match().frame, tr = {}, kind = "mirror" }
  ab.res = nil
  gd.resume()
  say("Recording P1 and P2 on one pad: M again to stop")
end
local function ab_start_b(kind)
  local A = ab.a
  if A == nil or ab.rec then say("Record A first (R)", DANGER) return end
  local h = gd.history()
  if A.from < (h.oldest or 0) then say("A has left the rewind window: record again", DANGER) return end
  ab.b = { from = A.from, to = A.to, kind = kind, tr = {} }
  ab.res = nil
  if kind == "reload" then
    -- the script reloads: A and the plan wait in files
    ab_write(A)
    pcall(gd.data_write, AB_PENDING, string.format("reload %d %d\n", A.from, A.to))
    local ok, why = gd.hot_reload((gd.match().frame - A.from) / 60)
    if not ok then say("Hot reload: " .. tostring(why), DANGER) ab.b = nil pcall(gd.data_write, AB_PENDING, "") end
  else
    -- the rewind happens at the loop top while paused; play on once it has (LE.tick)
    local ok, why = gd.rewind_to(A.from)
    if ok then ab.b.resume = true else say("Rewind: " .. tostring(why), DANGER) ab.b = nil end
  end
end
ACTIONS.ab_reload = function() ab_start_b("reload") end
ACTIONS.ab_same = function() ab_start_b("same") end
ACTIONS.ab_back = function()
  local r = ab.res and ab.res.per[1]
  if r == nil or #r.frames == 0 then return end
  ab.view = math.max(1, (ab.view or (#r.frames + 1)) - 1)
end
ACTIONS.ab_fwd = function()
  local r = ab.res and ab.res.per[1]
  if r == nil or #r.frames == 0 then return end
  ab.view = math.min(#r.frames, (ab.view or 0) + 1)
end
STATES.ab_rec = function() return ab.rec ~= nil end

local function ab_frame()
  if ab.rec then
    ab_sample(ab.rec.tr)
    local h = gd.history()
    if ab.rec.kind ~= "mirror" and gd.match().frame - ab.rec.from >= math.max(60, (h.depth or 600) - 30) then
      ab_stop_record()
      say("A stopped at the edge of the rewind window", GOLD)
    end
    return
  end
  local B = ab.b
  if B and B.kind ~= "mirror" then
    local f = gd.match().frame
    -- the rewind lands at a frame boundary: wait until the frames are A's again
    if not B.armed then
      if f > B.from + 2 then return end
      B.armed = true
    end
    if f >= B.from and f <= B.to then ab_sample(B.tr) end
    if f >= B.to then
      gd.pause()
      ab_finish()
      if B.kind == "reload" then pcall(gd.data_write, AB_PENDING, "") end
    end
  end
end

-- a hot reload restarted the script: pick A and the plan back up
do
  local ok, text = pcall(gd.data_read, AB_PENDING)
  if ok and text and text:match("^reload") then
    local a = ab_read()
    if a then
      ab.a = a
      ab.b = { from = a.from, to = a.to, kind = "reload", tr = {} }
    end
  end
end

local function act_col(id) return PORT[(id % 5) + 1] end

local function draw_ab()
  local x, y, w = 8, 8, 330
  local lines = {}
  local function add(s, c) lines[#lines + 1] = { s, c or BONE } end
  if ab.rec then
    add(string.format("RECORDING %s  f%d  (%d frames)", ab.rec.kind == "mirror" and "P1 + P2" or "A", ab.rec.from,
      gd.match().frame - ab.rec.from), DANGER)
  elseif ab.a == nil then
    add("R records A (live). Then B re-simulates the same input from the same frame:", MUTED)
    add("B = reload the fighter data (edit geno.json first), C = the same data (a check).", MUTED)
    add("M records two fighters at once: P2 plays P1's pad (P2 must be a human port).", MUTED)
  else
    add(string.format("A: f%d-%d (%d frames)", ab.a.from, ab.a.to, ab.a.to - ab.a.from), ACCENT)
    if ab.b and not ab.res then add(string.format("B (%s): re-simulating ... f%d", ab.b.kind, gd.match().frame), GOLD) end
  end
  local res = ab.res
  if res then
    for _, r in ipairs(res.per) do
      add(string.format("P%d%s: %s", r.pa, r.pb ~= r.pa and (" vs P" .. r.pb) or "",
        r.first and string.format("part at f%d (%s)", r.first, r.why) or "identical"), r.first and GOLD or OK)
      add(string.format("   pos %d f (max %.3f)  action %d f  hitboxes %d f  of %d", r.npos, r.maxd, r.nact, r.nhb,
        #r.frames), MUTED)
    end
  end
  panel(x, y, w, 26 + #lines * 13 + 4, "A / B")
  if res then txt(x + 64, y + 14, res.kind:upper(), "caption", GOLD) end
  for i, l in ipairs(lines) do txt(x + 12, y + 28 + (i - 1) * 13, l[1], "caption", l[2], "left", w - 24) end
  if res == nil or res.per[1] == nil or #res.per[1].frames == 0 then return end
  -- the focused port's comparison (or the first)
  local r = res.per[1]
  for _, rr in ipairs(res.per) do if rr.pa == focus then r = rr end end
  local n = #r.frames
  local view = ab.view or n
  local cur = r.frames[view]
  -- ghosts: A's path and B's path, and where each is on the viewed frame
  if T("ghost") and res.kind ~= "mirror" then
    local la, lb
    for i, fr in ipairs(r.frames) do
      local ax, ay = gd.project(fr.a[1], fr.a[2], 0)
      local qx, qy = gd.project(fr.b[1], fr.b[2], 0)
      if la and ax then gd.line(la[1], la[2], ax, ay, alpha(ACCENT, 0xC0)) end
      if lb and qx then gd.line(lb[1], lb[2], qx, qy, alpha(GOLD, 0xC0)) end
      la, lb = ax and { ax, ay } or la, qx and { qx, qy } or lb
    end
    local ax, ay = gd.project(cur.a[1], cur.a[2], 0)
    local qx, qy = gd.project(cur.b[1], cur.b[2], 0)
    if ax then quad(ax - 4, ay - 4, 8, 8, ACCENT) txt(ax + 6, ay - 4, "A", "caption", ACCENT) end
    if qx then quad(qx - 4, qy - 4, 8, 8, GOLD) txt(qx + 6, qy + 8, "B", "caption", GOLD) end
  end
  if not T("tracks") then return end
  -- split timelines: A and B coloured by action, the differences, the first divergence
  local px, pw = 22, 596
  local ph = 96
  local py = 444 - ph
  panel(8, py, 624, ph, "A / B TRACKS")
  local sx = pw / n
  local ty = py + 26
  txt(px, ty - 4, "A", "caption", ACCENT)
  txt(px, ty + 16, "B", "caption", GOLD)
  for i, fr in ipairs(r.frames) do
    local fx = px + 12 + (i - 1) * sx * (pw - 12) / pw
    local fw = math.max(1, sx * (pw - 12) / pw)
    quad(fx, ty - 12, fw, 10, act_col(fr.a[3]))
    quad(fx, ty + 8, fw, 10, act_col(fr.b[3]))
    if fr.act or fr.pos or fr.hb then quad(fx, ty + 22, fw, 6, fr.act and DANGER or fr.hb and HIT[1] or GOLD) end
    if fr.a[4] ~= 0 then quad(fx, ty - 1, fw, 2, HIT[0]) end
    if fr.b[4] ~= 0 then quad(fx, ty + 19, fw, 2, HIT[0]) end
  end
  if r.first then
    for i, fr in ipairs(r.frames) do
      if fr.f == r.first then
        local fx = px + 12 + (i - 1) * sx * (pw - 12) / pw
        img("lab_tl_playhead", fx - 4, ty - 18, 8, 16, GOLD)
      end
    end
  end
  local vx = px + 12 + (view - 1) * sx * (pw - 12) / pw
  quad(vx, ty - 14, 1, 44, BONE)
  txt(px, py + ph - 10, string.format("f%d   A %s  B %s   d %.3f%s", cur.f, gd.motion_name(cur.a[3], r.pa),
    gd.motion_name(cur.b[3], r.pb), cur.d, cur.hb and "   hitboxes differ" or ""), "caption", BONE, "left", pw)
end

-- ---- the frame-data export (TOOLS, `lab export`, MELEE_LAB_BATCH) --------------------------------
local LANDING_ATTR = { AttackAirN = "landingairn_lag", AttackAirF = "landingairf_lag", AttackAirB = "landingairb_lag",
  AttackAirHi = "landingairhi_lag", AttackAirLw = "landingairlw_lag" }
local bx = nil
local fd_last = nil -- the last export: {fighter, version, dir, moves}

local function bx_moves(port)
  local out = {}
  for _, m in ipairs(gd.motion_list(port) or {}) do
    if m.anim_id >= 0 and m.anim_name ~= "" and ((m.group == "common" and m.name:find("^Attack") ~= nil)
        or m.group == "geno") then
      out[#out + 1] = m
    end
  end
  for i, m in ipairs(input_entries()) do table.insert(out, i, m) end
  return out
end

local function bx_start(port, version, quit)
  local p = gd.player(port)
  if p == nil or not offline() then return false, "no fighter there (or online)" end
  if p.airborne or gd.motion_name(p.action, port) ~= "Wait" then return false, "stand still on the ground first (Wait; now " .. gd.motion_name(p.action, port) .. (p.airborne and ", airborne" or "") .. ")" end
  if bx then return false, "an export is running" end
  gd.pause()
  gd.savestate(BATCH_SLOT)
  bx = { port = port, version = version or gd.lab_now(), moves = bx_moves(port), i = 0,
    phase = "save", wait = 2, out = {}, fighter = ((p.char_name or "fighter"):lower():gsub("[^%w_%-]+", "_")), quit = quit,
    fall = motion_by_name(port, "Fall", 29),
    t0 = gd.time(), attrs = gd.attrs(port), verbose = gd.lab_env("BATCH_VERBOSE") ~= nil }
  say(string.format("Frame data: %d states of %s", #bx.moves, fighter_name(port)))
  return true
end

local function bx_sample(cur, f)
  local p = gd.player(bx.port)
  if p == nil then return end
  local s = { action = p.action, iasa = p.iasa, air = p.airborne, hb = {} }
  for _, h in ipairs(gd.hitboxes(bx.port) or {}) do s.hb[#s.hb + 1] = h end
  cur.f[f] = s
end

local function bx_static(port, m)
  local tl = gd.timeline(port, m.id)
  local st = { len = 0, iasa = nil, ac = nil }
  if tl == nil then return st end
  local _, marks, len = analyse(tl)
  st.len = math.floor(len + 0.5)
  local lag_on, lag_off
  for _, e in ipairs(tl.events) do
    if e.name == "iasa" and st.iasa == nil then st.iasa = e.frame end
    if e.name == "cmd_var" and e.index == 0 then
      if e.value ~= 0 and lag_on == nil then lag_on = e.frame
      elseif e.value == 0 and lag_on and lag_off == nil then lag_off = e.frame end
    end
  end
  if lag_on then
    -- cmd_vars[0] set = a landing takes the move's landing lag (ftCo_LandingAir_EnterWithLag)
    st.ac = string.format("1-%d%s", lag_on - 1, lag_off and string.format(" %d-", lag_off) or "")
  end
  return st
end

local function bx_finish_move(cur)
  local m = cur.m
  local frames = sorted_keys(cur.f)
  local total, iasa, active = 0, nil, {}
  local hbs, order = {}, {}
  for _, f in ipairs(frames) do
    local s = cur.f[f]
    if cur.input or s.action == m.id then
      if f > total then total = f end
      if s.iasa and iasa == nil then iasa = f end
      if #s.hb > 0 then active[f] = true end
      for _, h in ipairs(s.hb) do
        local key = string.format("%d|%g|%d|%d|%d|%d|%.3f|%d|%s|%d", h.id, h.damage, h.angle, h.kbg, h.bkb, h.wbk,
          h.radius, h.bone, h.element_name, h.shield_damage)
        if hbs[key] == nil then
          hbs[key] = { id = h.id, damage = h.damage, angle = h.angle, kbg = h.kbg, bkb = h.bkb, wbk = h.wbk,
            radius = h.radius, bone = h.bone, element = h.element_name, shield_damage = h.shield_damage, frames = {} }
          order[#order + 1] = key
        end
        hbs[key].frames[f] = true
      end
    end
  end
  local st = m.input and { len = nil } or bx_static(bx.port, m)
  local startup
  for _, f in ipairs(sorted_keys(active)) do startup = f break end
  local lag_attr = LANDING_ATTR[m.name]
  local lag = lag_attr and bx.attrs and bx.attrs[lag_attr] or nil
  local row = { id = m.id, name = m.name, group = m.group, air = m.air or (air_name(m.name) and true or false),
    total = total, total_script = st.len, iasa = iasa or st.iasa, startup = startup, active = windows_of(active),
    landing_lag = lag and math.floor(lag + 0.5) or nil, lcancel_lag = lag and math.floor(lag / 2) or nil,
    autocancel = st.ac, ended = cur.ended or "", landed = cur.landed, hitboxes = {},
    chain = cur.chain and table.concat(cur.chain, ">") or nil }
  for _, key in ipairs(order) do
    local h = hbs[key]
    h.frames = windows_of(h.frames)
    row.hitboxes[#row.hitboxes + 1] = h
  end
  bx.out[#bx.out + 1] = row
end

local function csv(v)
  if v == nil then return "" end
  if type(v) == "boolean" then return v and "1" or "0" end
  local s = tostring(v)
  if s:find("[,\"]") then s = "\"" .. s:gsub("\"", "\"\"") .. "\"" end
  return s
end
local function jstr(v)
  if v == nil then return "null" end
  if type(v) == "boolean" or type(v) == "number" then return tostring(v) end
  return "\"" .. tostring(v):gsub("[\"%c]", function(c) return c == "\"" and "\\\"" or " " end) .. "\""
end

local MOVE_COLS = { "id", "name", "group", "air", "startup", "active", "total", "iasa", "total_script",
  "landing_lag", "lcancel_lag", "autocancel", "landed", "ended", "chain" }
local HB_COLS = { "id", "damage", "angle", "kbg", "bkb", "wbk", "radius", "bone", "element", "shield_damage", "frames" }

local function bx_write()
  local dir = string.format("framedata/%s/%s", bx.fighter, bx.version)
  local mc, hc, js = { "fighter," .. table.concat(MOVE_COLS, ",") .. ",hitboxes" },
    { "fighter,motion_id,motion,hb_" .. table.concat(HB_COLS, ",hb_") }, {}
  for _, r in ipairs(bx.out) do
    local cells = { csv(bx.fighter) }
    for _, c in ipairs(MOVE_COLS) do cells[#cells + 1] = csv(r[c]) end
    cells[#cells + 1] = tostring(#r.hitboxes)
    mc[#mc + 1] = table.concat(cells, ",")
    local jh = {}
    for _, h in ipairs(r.hitboxes) do
      local hcells = { csv(bx.fighter), tostring(r.id), csv(r.name) }
      local jf = {}
      for _, c in ipairs(HB_COLS) do
        hcells[#hcells + 1] = csv(h[c])
        jf[#jf + 1] = jstr(c) .. ":" .. jstr(h[c])
      end
      hc[#hc + 1] = table.concat(hcells, ",")
      jh[#jh + 1] = "{" .. table.concat(jf, ",") .. "}"
    end
    local jm = {}
    for _, c in ipairs(MOVE_COLS) do jm[#jm + 1] = jstr(c) .. ":" .. jstr(r[c]) end
    jm[#jm + 1] = "\"hitboxes\":[" .. table.concat(jh, ",") .. "]"
    js[#js + 1] = "  {" .. table.concat(jm, ",") .. "}"
  end
  local head = string.format("{\"fighter\":%s,\"version\":%s,\"date\":%s,\"frames\":\"the first frame in the state = frame 1\",\"moves\":[\n",
    jstr(bx.fighter), jstr(bx.version), jstr(gd.lab_now(true)))
  gd.data_write(dir .. "/moves.csv", table.concat(mc, "\n") .. "\n")
  gd.data_write(dir .. "/hitboxes.csv", table.concat(hc, "\n") .. "\n")
  gd.data_write(dir .. "/framedata.json", head .. table.concat(js, ",\n") .. "\n]}\n")
  -- the fighter's version list (the diff picks the last two)
  local idx = string.format("framedata/%s/versions.txt", bx.fighter)
  local ok, old = pcall(gd.data_read, idx)
  local text = (ok and old) or ""
  if not text:find("\n" .. bx.version .. "\n", 1, true) and not text:find("^" .. bx.version .. "\n") then
    text = text .. bx.version .. "\n"
  end
  gd.data_write(idx, text)
  return dir
end

local function bx_tick()
  if bx == nil then return end
  if not gd.match().active then bx = nil return end
  if bx.wait > 0 then bx.wait = bx.wait - 1 return end
  if bx.phase == "save" or bx.phase == "next" then
    bx.i = bx.i + 1
    if bx.i > #bx.moves then
      local okw, dir = pcall(bx_write)
      if not okw then
        gd.log("lab export: not written: " .. tostring(dir))
        say("Frame data: not written (see the log)", DANGER)
        local quit = bx.quit
        bx = nil
        if quit then gd.quit() end
        return
      end
      fd_last = { fighter = bx.fighter, version = bx.version, dir = dir, n = #bx.out }
      gd.log(string.format("lab export: %s %s: %d states in %.1f s -> scripts-data/geno-lab_lab/%s", bx.fighter,
        bx.version, #bx.out, gd.time() - bx.t0, dir))
      say(string.format("Frame data: %d states -> %s", #bx.out, dir), OK)
      local quit = bx.quit
      bx = nil
      pcall(gd.loadstate, BATCH_SLOT)
      if quit then gd.quit() end
      return
    end
    gd.resume()
    gd.pause()
    pcall(gd.loadstate, BATCH_SLOT)
    bx.phase, bx.wait = "enter", 1
  elseif bx.phase == "enter" then
    local m = bx.moves[bx.i]
    bx.cur = { m = m, f = {}, n = 0, entered = false, tries = 0 }
    if bx.verbose then gd.log(string.format("lab export: %d/%d %d %s anim %d", bx.i, #bx.moves, m.id, m.name, m.anim_id)) end
    if m.input then
      bx.cur.input = true
      if m.air then gd.set_motion(bx.port, bx.fall, 1, 1, 60) end -- low: an up special must not fly into the top blast zone
      bx.phase, bx.wait = "input", m.air and 1 or 0
      return
    end
    local ok = gd.set_motion(bx.port, m.id, 1, 1, air_name(m.name) and 160 or 0)
    if not ok then bx.phase = "next" return end
    bx.phase = "settle"
  elseif bx.phase == "input" then
    local p, cur = gd.player(bx.port), bx.cur
    cur.base, cur.pre, cur.chain = p and p.action, 0, {}
    -- the pad override reaches the game at the next tick's pad read: no frame runs until then
    gd.input(bx.port, input_spec(bx.port, cur.m.input), 3)
    bx.phase, bx.wait = "run", 1
  elseif bx.phase == "settle" then
    local p, cur = gd.player(bx.port), bx.cur
    cur.tries = cur.tries + 1
    if p and p.action == cur.m.id then
      -- the entry itself (the change at the frame boundary); the first frame the game runs in the
      -- state is frame 1, as frame-data sites count (Fox: jab 2-3, nair 4-31), so it overwrites this
      bx_sample(cur, 1)
      cur.entered = true
      bx.phase = "run"
      gd.step(30)
    elseif cur.tries > 4 then
      cur.ended = "never entered"
      bx_finish_move(cur)
      bx.phase = "next"
    end
  elseif bx.phase == "run" then
    local cur = bx.cur
    if cur.done then
      bx_finish_move(cur)
      bx.phase = "next"
      gd.resume()
      gd.pause()
    else
      gd.step(30)
    end
  end
end

local function bx_frame()
  if bx == nil or bx.phase ~= "run" then return end
  local cur = bx.cur
  if cur.done then return end
  local p = gd.player(bx.port)
  if p == nil then cur.done = true return end
  if cur.input then
    -- frame 1 = the first frame out of the neutral stance; the move ends back in an end state
    if cur.n == 0 then
      cur.pre = cur.pre + 1
      if p.action == cur.base then
        if cur.pre > 12 then cur.done = true cur.ended = "no special came out" end
        return
      end
    end
    local nm = gd.motion_name(p.action, bx.port)
    if cur.n > 0 and END_STATES[nm] then cur.done = true cur.ended = nm return end
    cur.n = cur.n + 1
    if cur.chain[#cur.chain] ~= nm then cur.chain[#cur.chain + 1] = nm end
    bx_sample(cur, cur.n)
    if cur.n >= 300 then cur.done = true cur.ended = "cap 300" end
    return
  end
  cur.n = cur.n + 1
  if p.action ~= cur.m.id then
    cur.done = true
    cur.ended = gd.motion_name(p.action, bx.port)
    return
  end
  bx_sample(cur, cur.n)
  if cur.n >= 299 then cur.done = true cur.ended = "cap 300" end
end

local function bx_land(port)
  if bx and bx.cur and port == bx.port and not bx.cur.landed and bx.phase == "run" then bx.cur.landed = bx.cur.n + 1 end
end

-- the diff of two exports (moves.csv + hitboxes.csv), by state name
local function read_csv(name)
  local ok, text = pcall(gd.data_read, name)
  if not ok or text == nil then return nil end
  local rows, head = {}, nil
  for l in text:gmatch("[^\n]+") do
    local cells, i = {}, 1
    while i <= #l do
      if l:sub(i, i) == "\"" then
        local j, s = i + 1, ""
        while j <= #l do
          local ch = l:sub(j, j)
          if ch == "\"" and l:sub(j + 1, j + 1) == "\"" then s = s .. "\"" j = j + 2
          elseif ch == "\"" then j = j + 1 break
          else s = s .. ch j = j + 1 end
        end
        cells[#cells + 1] = s
        i = j + 1
      else
        local j = l:find(",", i, true) or (#l + 1)
        cells[#cells + 1] = l:sub(i, j - 1)
        i = j + 1
      end
    end
    if l:sub(-1) == "," then cells[#cells + 1] = "" end
    if head == nil then head = cells else
      local r = {}
      for k, h in ipairs(head) do r[h] = cells[k] or "" end
      rows[#rows + 1] = r
    end
  end
  return rows
end

local function fd_diff(fighter, va, vb)
  local base = "framedata/" .. fighter .. "/"
  local ma, mb = read_csv(base .. va .. "/moves.csv"), read_csv(base .. vb .. "/moves.csv")
  if ma == nil or mb == nil then return nil, "no export " .. (ma == nil and va or vb) end
  local ha, hb = read_csv(base .. va .. "/hitboxes.csv") or {}, read_csv(base .. vb .. "/hitboxes.csv") or {}
  local function index(rows, hrows)
    local t = {}
    for _, r in ipairs(rows) do t[r.name] = { r = r, hb = {} } end
    for _, h in ipairs(hrows) do if t[h.motion] then table.insert(t[h.motion].hb, h) end end
    return t
  end
  local A, B = index(ma, ha), index(mb, hb)
  local out, changed = {}, 0
  local names = {}
  for n in pairs(A) do names[n] = true end
  for n in pairs(B) do names[n] = true end
  for _, n in ipairs(sorted_keys(names)) do
    local a, b = A[n], B[n]
    if a == nil then out[#out + 1] = "+ " .. n .. "  (new in " .. vb .. ")" changed = changed + 1
    elseif b == nil then out[#out + 1] = "- " .. n .. "  (gone in " .. vb .. ")" changed = changed + 1
    else
      local diffs = {}
      for _, c in ipairs(MOVE_COLS) do
        if c ~= "id" and (a.r[c] or "") ~= (b.r[c] or "") then
          diffs[#diffs + 1] = string.format("%s %s -> %s", c, a.r[c] ~= "" and a.r[c] or "-", b.r[c] ~= "" and b.r[c] or "-")
        end
      end
      local nh = math.max(#a.hb, #b.hb)
      for k = 1, nh do
        local x, y = a.hb[k], b.hb[k]
        if x == nil or y == nil then
          diffs[#diffs + 1] = string.format("hitbox %d %s", k, x == nil and "added" or "removed")
        else
          for _, c in ipairs(HB_COLS) do
            if x["hb_" .. c] ~= y["hb_" .. c] then
              diffs[#diffs + 1] = string.format("hb%d.%s %s -> %s", k, c, x["hb_" .. c], y["hb_" .. c])
            end
          end
        end
      end
      if #diffs > 0 then
        changed = changed + 1
        out[#out + 1] = "~ " .. n .. ": " .. table.concat(diffs, "; ")
      end
    end
  end
  table.insert(out, 1, string.format("framedata diff %s: %s -> %s: %d state(s) changed", fighter, va, vb, changed))
  pcall(gd.data_write, base .. "diff_" .. va .. "_" .. vb .. ".txt", table.concat(out, "\n") .. "\n")
  return out, changed
end

local function fd_versions(fighter)
  local ok, text = pcall(gd.data_read, "framedata/" .. fighter .. "/versions.txt")
  local v = {}
  if ok and text then for l in text:gmatch("[^\n]+") do v[#v + 1] = l end end
  return v
end

local fd_diff_text = nil
local function fd_diff_last(port)
  local p = gd.player(port or focus)
  local fighter = p and ((p.char_name or ""):lower():gsub("[^%w_%-]+", "_")) or (fd_last and fd_last.fighter)
  if fighter == nil then return nil, "no fighter" end
  local v = fd_versions(fighter)
  if #v < 2 then return nil, "export " .. fighter .. " twice first (" .. #v .. " export(s))" end
  local out, n = fd_diff(fighter, v[#v - 1], v[#v])
  if out == nil then return nil, n end
  fd_diff_text = out
  return out, n
end

-- ---- the rollback strip (FRAMES, N) ----------------------------------------------------------------
local RB_KIND_COL = { synctest = ACCENT, fake = 0x4D8DFFFF, netplay = GOLD }
draw_rollbacks = function()
  local r = gd.rollbacks(240)
  local x, y, w, h = 332, 36, 300, 104
  panel(x, y, w, h, "ROLLBACKS")
  if r.total == 0 and r.mismatch == nil then
    txt(x + 12, y + 34, "none yet: netplay, SyncTest or MELEE_RB_FAKE", "caption", DISABLED)
    txt(x + 12, y + 48, "record every rollback here, newest on the right", "caption", DISABLED)
    return
  end
  local newest = r.list[1] and r.list[1].frame or 0
  local span = 180
  local bx0, by0, bw, bh = x + 12, y + 70, w - 24, 44
  quad(bx0, by0, bw, 1, alpha(TICK, 0x90))
  local maxd, sum, nms, causes = 0, 0, 0, {}
  for _, e in ipairs(r.list) do
    local age = newest - e.frame
    if age < span then
      local fx = bx0 + bw - (age + 1) * bw / span
      local hh = math.min(bh, 5 * e.depth)
      quad(fx, by0 - hh, math.max(1, bw / span), hh, e.mismatch and DANGER or RB_KIND_COL[e.kind] or BONE)
    end
    if e.depth > maxd then maxd = e.depth end
    sum = sum + e.ms
    if e.mismatch then nms = nms + 1 end
    causes[e.cause] = (causes[e.cause] or 0) + 1
  end
  local cs = {}
  for c, n in pairs(causes) do cs[#cs + 1] = (c == 0 and "SyncTest" or ("P" .. c)) .. " " .. n end
  local last = r.list[1]
  txt(x + 70, y + 14, string.format("%d in all", r.total), "caption", GOLD)
  txt(x + 12, y + 30, last and string.format("last f%d: %d back (from f%d), %s, %.2f ms", last.frame, last.depth,
    last.first, last.kind, last.ms) or "", "caption", BONE, "left", w - 24)
  txt(x + 12, y + 43, string.format("deepest %d   mean %.2f ms   why: %s", maxd, #r.list > 0 and sum / #r.list or 0,
    table.concat(cs, ", ")), "caption", MUTED, "left", w - 24)
  if r.mismatch then
    panel(x, y + h + 6, w, 44, "MISMATCH")
    txt(x + 84, y + h + 20, string.format("f%d  x%d", r.mismatch.frame, r.mismatch.count), "caption", DANGER)
    txt(x + 12, y + h + 36, r.mismatch.where, "caption", BONE, "left", w - 24)
  end
end

tools_items = function()
  local function go(id)
    return function() set_mode(MODE_BY_ID[id]) menu_close() end
  end
  return {
    { label = "Frame data export", icon = "lab_export",
      desc = function()
        return "Every attack, special, m-ex and Geno state of the focused fighter, one after another from a neutral"
          .. " start, as fast as the game runs: startup, active frames, IASA, landing lag, autocancel and every"
          .. " hitbox, to CSV and JSON in scripts-data/geno-lab_lab/framedata/<fighter>/<version>. Uses quick slot "
          .. BATCH_SLOT .. "." .. (fd_last and ("  Last: " .. fd_last.dir .. ", " .. fd_last.n .. " states.") or "")
      end,
      value = function() return bx and "RUNNING" or (fd_last and fd_last.version or "") end,
      run = function()
        menu_close()
        local ok, why = bx_start(focus, nil, false)
        if not ok then say("Export: " .. tostring(why), DANGER) end
      end },
    { label = "Diff the last two", icon = "lab_diff",
      desc = function()
        local t = fd_diff_text
        if t == nil then return "Compare the focused fighter's last two exports: every changed state and field. The full list goes to framedata/<fighter>/diff_<a>_<b>.txt and the console." end
        local s = t[1]
        for k = 2, math.min(#t, 4) do s = s .. "  " .. t[k] end
        return s
      end,
      value = function() return fd_diff_text and (#fd_diff_text - 1) .. " CHANGED" or "" end,
      run = function()
        local out, n = fd_diff_last(focus)
        if out == nil then say("Diff: " .. tostring(n), DANGER) return end
        for _, l in ipairs(out) do gd.log(l) end
        say(out[1], n > 0 and GOLD or OK)
      end },
    { label = "State browser", icon = "lab_moves", key = "6",
      desc = "Every action state of the focused fighter, with a filter. Pick one and play it: from neutral, looped, slowed.",
      run = go("moves") },
    { label = "Launch preview", icon = "lab_launch", key = "7",
      desc = "The knockback, angle, hitstun and flight of the hitbox on screen (or the next one), for the victim's percent, weight and DI.",
      run = go("launch") },
    { label = "A/B compare", icon = "lab_ab", key = "8",
      desc = "Record your inputs once, re-simulate them on other fighter data from the same frame, see the first frame they part.",
      run = go("ab") },
    { label = "Rollbacks", icon = "lab_rollback",
      desc = function()
        local r = gd.rollbacks(1)
        return string.format("%d rollbacks recorded%s. A clears the record. FRAMES mode draws them (N).", r.total,
          r.mismatch and (", first mismatch f" .. r.mismatch.frame .. ": " .. r.mismatch.where) or "")
      end,
      value = function() return tostring(gd.rollbacks(0).total) end,
      run = function() gd.rollbacks_clear() say("Rollback record cleared") end },
  }
end

function LE.console(cmd, rest)
  local handled = true
  if cmd == "export" then
    local pt, ver = rest:match("^(%d*)%s*(%S*)$")
    local ok, why = bx_start(tonumber(pt) or focus, ver ~= "" and ver or nil, false)
    gd.log(ok and "export started" or ("export: " .. tostring(why)))
  elseif cmd == "fdiff" then
    local f, va, vb = rest:match("^(%S+)%s+(%S+)%s+(%S+)$")
    local out, n
    if f then out, n = fd_diff(f, va, vb) else out, n = fd_diff_last(focus) end
    if out == nil then gd.log("fdiff: " .. tostring(n)) else for _, l in ipairs(out) do gd.log(l) end end
  elseif cmd == "moves" then
    br.text = rest:lower()
    local list = br_list()
    gd.log(string.format("moves: %d of %d (filter %s, text \"%s\")", #list, #br.all, FILTERS[br.filter], br.text))
    for i, m in ipairs(list) do
      if i > 60 then gd.log(string.format("  ... %d more (lab moves <text> filters)", #list - 60)) break end
      gd.log(string.format("  %4d 0x%s %-8s %-24s %s", m.id, hexid(m.id), m.group, m.name, m.anim_name))
    end
  elseif cmd == "play" then
    local id = tonumber(rest)
    local m
    for _, x in ipairs(br_list() and br.all) do if x.id == id or x.name == rest then m = x end end
    if m then br_start(m) gd.log("playing " .. m.name) else gd.log("lab play <id | name>") end
  elseif cmd == "kb" then
    local r, a, v, hbs, h = kb_predict()
    if r == nil then gd.log("kb: no hitbox / no victim") else
      gd.log(string.format("kb: P%d #%d %s%% a%d kbg %d bkb %d wbk %d on P%d at %.0f%% (w %g, DI %s): kb %.3f angle %.2f -> %.2f"
        .. " hitstun %d level %d%s", focus, h.id, tostring(h.damage), h.angle, h.kbg, h.bkb, h.wbk, v.port, r.percent,
        r.weight, r.di, r.kb, r.angle, r.angle_di, r.hitstun, r.level,
        r.blast and string.format(" KO %s f%d", r.blast.side, r.blast.frame) or " survives"))
    end
    if kbv.last then
      local L = kbv.last
      gd.log(string.format("kb check: kb %.3f vs %.3f, hitstun %d vs %s, flight %d f max err %.4f mean %.4f", L.kb_pred,
        L.kb_real or -1, L.hs_pred, tostring(L.hs_real), L.n, L.max, L.mean))
    end
  elseif cmd == "di" then
    for i, d in ipairs(DIS) do if d == rest then kbv.di = i end end
    gd.log("DI " .. DIS[kbv.di])
  elseif cmd == "ab" then
    local sub = rest:match("^(%S*)")
    local map = { record = "ab_record", reload = "ab_reload", same = "ab_same", mirror = "ab_mirror" }
    if map[sub] then ACTIONS[map[sub]]() gd.log("ab " .. sub)
    else
      local res = ab.res
      gd.log(string.format("ab: A %s, B %s, rec %s", ab.a and (ab.a.from .. "-" .. ab.a.to) or "-",
        ab.b and ab.b.kind or "-", tostring(ab.rec ~= nil)))
      if res then for _, r in ipairs(res.per) do
        gd.log(string.format("  P%d/P%d first %s (%s) pos %d max %.4f act %d hb %d of %d", r.pa, r.pb, tostring(r.first),
          r.why or "-", r.npos, r.maxd, r.nact, r.nhb, #r.frames))
      end end
    end
  elseif cmd == "rollbacks" then
    local r = gd.rollbacks(tonumber(rest) or 20)
    gd.log(string.format("rollbacks: %d in all", r.total))
    for _, e in ipairs(r.list) do
      gd.log(string.format("  f%d back %d from f%d cause %s %s %.2f ms%s", e.frame, e.depth, e.first,
        e.cause == 0 and "synctest" or ("P" .. e.cause), e.kind, e.ms, e.mismatch and " MISMATCH" or ""))
    end
    if r.mismatch then gd.log(string.format("  first mismatch f%d (x%d): %s", r.mismatch.frame, r.mismatch.count,
      r.mismatch.where)) end
  else
    handled = false
  end
  return handled
end

-- ---- what the rest of the script uses ---------------------------------------------------------------
local batch_env_done = false
-- true while the export runs (it owns the tick)
function LE.tick()
  -- the headless export: MELEE_LAB_BATCH=<version> (MELEE_LAB_BATCH_PORT, MELEE_LAB_BATCH_QUIT=1)
  local p1 = gd.match().active and gd.player(tonumber(gd.lab_env and gd.lab_env("BATCH_PORT") or "1") or 1)
  if not batch_env_done and p1 and gd.match().frame >= 30 and not p1.airborne and gd.motion_name(p1.action, p1.port) == "Wait"
      and gd.lab_env("BATCH") then
    batch_env_done = true
    cfg.hidden = true
    local ok, why = bx_start(tonumber(gd.lab_env("BATCH_PORT") or "1") or 1, gd.lab_env("BATCH"),
      gd.lab_env("BATCH_QUIT") == "1")
    if not ok then gd.log("lab export: " .. tostring(why)) end
  end
  if bx then
    -- never leave the game stuck: an error ends the export, and so does a move that makes no
    -- progress for 5 s (a state that never ends is capped at 300 frames; this is the backstop)
    local key = bx.i .. ":" .. bx.phase .. ":" .. (bx.cur and bx.cur.n or -1)
    if key ~= bx.last_key then bx.last_key, bx.last_t = key, gd.time() end
    local ok, err = pcall(bx_tick)
    if not ok or (bx and gd.time() - (bx.last_t or gd.time()) > 5) then
      gd.log("lab export: stopped at state " .. tostring(bx and bx.i) .. ": " .. (ok and "no progress for 5 s" or tostring(err)))
      say("Frame data: stopped (see the log)", DANGER)
      local quit = bx and bx.quit
      bx = nil
      gd.resume()
      if quit then gd.quit() end
    end
    return true
  end
  if ab.b and ab.b.resume and not gd.history().busy then
    ab.b.resume = false
    gd.resume()
  end
  br_tick()
  return false
end
function LE.frame()
  bx_frame()
  br_frame()
  kb_frame()
  ab_frame()
end
LE.on_hit = kb_on_hit
LE.on_land = bx_land
function LE.reset()
  br.play, br.pending, br.key = nil, nil, nil
  kbv.check, kbv.cache, kbv.key = nil, nil, nil
  ab.rec = nil
  bx = nil
end
function LE.draw_batch()
  if not (bx and cfg.on and gd.match().active) then return end
  local s = string.format("FRAME DATA  %s  %d / %d  %s", bx.fighter:upper(), bx.i, #bx.moves,
    bx.moves[bx.i] and bx.moves[bx.i].name or "")
  local w = measure(s, "body") + 30
  quad(320 - w / 2, 12, w, 24, GLASS, SHEAR)
  quad(320 - w / 2 - 2, 12, 4, 24, GOLD, SHEAR)
  txt(320 - w / 2 + 14, 29, s, "body", BONE)
end
LE.draw_moves, LE.draw_launch, LE.draw_ab = draw_moves, draw_launch, draw_ab
-- for stage D's move card: the export's own reading of a state's script
function LE.move_static(port, id) return bx_static(port, { id = id }) end
LE.LANDING_ATTR, LE.windows_of = LANDING_ATTR, windows_of
end
stage_e()

local LD = {} -- stage D: the player-training half (a function of its own, as stage E)
local function stage_d()
-- =================================================================================================
-- ---- stage D: the player-training half (docs/geno.md 14.12) -------------------------------------
-- D1, TRAINING mode (9): frame advantage after every exchange, the move card, the input display
-- and tech-skill feedback (L-cancel, wavedash, waveland, ledgedash, short / full hop).
-- Everything here only reads the game; it works in any Lab match, CPUs and humans alike.
-- =================================================================================================
local HIST_N = 6          -- exchanges kept on screen
local EXCHANGE_MAX = 240  -- frames an exchange may run before it is dropped (someone never acts)
local SEQ_IDLE = 20       -- input log: a sequence ends after this many frames with no new input
local SEQ_N = 5           -- input sequences kept
local TECH_N = 6          -- tech results kept
local STICK_MAX = 80      -- a full stick tilt (the pad's usable range)
local FLICK = 0.8         -- the input log names a stick direction past this much of a full tilt
local LC_EARLY_MAX = 20   -- an L / R / Z press at most this many frames before the window is an early L-cancel

local attrs_cache = {} -- [port] = {char, a = gd.attrs(port)}
local common_cache = nil

local function attrs_of(p)
  local c = attrs_cache[p.port]
  if c == nil or c.char ~= p.char then
    c = { char = p.char, a = gd.attrs(p.port) or {} }
    attrs_cache[p.port] = c
  end
  return c.a
end

-- the PlCo constants (gd.lab_common, stage D); the retail values when the build has none
local function common()
  if common_cache == nil then
    common_cache = (gd.lab_common and gd.lab_common()) or {}
    if (common_cache.lcancel_window or 0) <= 0 then common_cache.lcancel_window = 7 end
    if (common_cache.lcancel_div or 0) <= 0 then common_cache.lcancel_div = 2 end
  end
  return common_cache
end

-- ---- "actionable": the first frame a fighter can act again -----------------------------------------
-- A state from which every normal option is open (FREE), the script's IASA flag, a normal landing
-- past its lag (ftCo_Landing_IASA: anim frame >= normal_landing_lag), or a damage state once
-- hitstun is over. Never during hitlag.
local FREE = {}
for _, n in ipairs({ "Wait", "WalkSlow", "WalkMiddle", "WalkFast", "Turn", "Dash", "Run", "RunDirect", "Squat",
  "SquatWait", "SquatRv", "Fall", "FallF", "FallB", "FallAerial", "FallAerialF", "FallAerialB", "JumpF", "JumpB",
  "JumpAerialF", "JumpAerialB", "GuardOn", "Guard", "CliffWait", "Ottotto", "OttottoWait" }) do
  FREE[n] = true
end
local GUARD = { GuardOn = true, Guard = true, GuardSetOff = true, GuardReflect = true }

local function is_damage(name)
  return name == "DamageFall" or name:find("^DamageFly") ~= nil or name:find("^Damage%a+%d$") ~= nil
end

local function actionable(p)
  local name = p.motion_name
  if p.in_hitlag then return false end
  if p.iasa or FREE[name] then return true end
  if name == "Landing" then return p.anim_frame >= (attrs_of(p).normal_landing_lag or 4) end
  if is_damage(name) then return not p.in_hitstun end
  return false
end
LD.actionable = actionable

-- ---- frame advantage -----------------------------------------------------------------------------------
-- An exchange starts on a hit (on_hit) or a shield hit (the victim enters hitlag while shielding).
-- From that frame on, each side's first actionable frame is taken; the advantage is the victim's
-- minus the attacker's, from the attacker's side: "+3 on shield" = the attacker acts 3 frames first.
-- Another hit by the same attacker restarts the exchange (multi-hit moves count from the last hit).
local fa = { cur = nil, hist = {} }

local function fa_push(e)
  table.insert(fa.hist, 1, e)
  while #fa.hist > HIST_N do table.remove(fa.hist) end
end

local function fa_start(a, v, kind)
  local pa = gd.player(a)
  fa.cur = { a = a, v = v, kind = kind, f0 = gd.match().frame, move = pa and pa.motion_name or "?",
    char = pa and pa.char_name or "?" }
end

local function fa_finish(c, why)
  fa.cur = nil
  local e = { a = c.a, v = c.v, kind = c.kind, move = c.move, char = c.char, frame = c.f0, why = why }
  if why == nil then e.adv = c.tv - c.ta end
  fa_push(e)
  if e.adv then
    log(string.format("P%d %s %+d on %s (P%d)", c.a, c.move, e.adv, c.kind, c.v), e.adv >= 0 and OK or DANGER)
  end
end

local function fa_frame(now)
  local c = fa.cur
  if c == nil then return end
  local pa, pv = gd.player(c.a), gd.player(c.v)
  if pa == nil or pv == nil or now - c.f0 > EXCHANGE_MAX then fa.cur = nil return end
  if c.tv == nil and pv.motion_name:find("^ShieldBreak") then fa_finish(c, "shield break") return end
  if c.ta == nil and actionable(pa) then c.ta = now end
  if c.tv == nil and actionable(pv) then c.tv = now end
  if c.ta and c.tv then fa_finish(c) end
end

-- the shield hit's attacker: whoever entered hitlag with the victim, else whoever has a live hitbox,
-- else the nearest other fighter
local function shield_attacker(v, rising)
  for port in pairs(rising) do if port ~= v then return port end end
  local pv = gd.player(v)
  local best, bd
  for _, p in ipairs(gd.players()) do
    if p.port ~= v then
      if #p.hitboxes > 0 then return p.port end
      local d = pv and math.abs(p.x - pv.x) + math.abs(p.y - pv.y) or 0
      if bd == nil or d < bd then best, bd = p.port, d end
    end
  end
  return best
end

function LD.on_hit(attacker, victim, info)
  if attacker == nil or victim == nil or attacker == victim or info.item then return end
  fa_start(attacker, victim, "hit")
  LD.dummy_hit(attacker, victim, info)
  LD.combo_victim_hit(attacker, victim)
  LD.combo_hit(attacker, victim, info)
  LD.drill_hit(attacker, victim)
end

-- ---- the move card ---------------------------------------------------------------------------------------
-- The focused fighter's move: its script's windows (the same analysis as the FRAMES timeline and
-- the state browser), IASA, the length, and for aerials the landing lag, the L-cancelled lag and the
-- autocancel windows (the frame-data export's own reading, LE.move_static). It stays up, dimmed,
-- after the move ends, until the next move with a hitbox or a landing lag.
local card = { port = nil, action = nil, info = nil, live = false }

local function card_info(p)
  local c = timeline_of(p)
  if c == nil then return nil end
  local lag_attr = LE.LANDING_ATTR[p.motion_name]
  if #c.windows == 0 and lag_attr == nil then return nil end
  local frames, iasa = {}, nil
  for _, w in ipairs(c.windows) do for f = w.from, w.to do frames[f] = true end end
  for _, e in ipairs(c.marks) do if e.name == "iasa" then iasa = e.frame break end end
  local st = LE.move_static(p.port, p.action)
  local lag = lag_attr and attrs_of(p)[lag_attr] or nil
  local info = { name = p.motion_name, char = p.char_name, len = math.floor(c.len + 0.5),
    startup = c.windows[1] and c.windows[1].from or nil, active = LE.windows_of(frames), iasa = iasa or st.iasa,
    windows = c.windows, ac = st.ac }
  if lag then
    info.lag = math.floor(lag + 0.5)
    info.lcl = math.max(1, math.floor(lag / common().lcancel_div))
  end
  return info
end

local function card_frame()
  local p = gd.player(focus)
  if p == nil then return end
  if p.port ~= card.port or p.action ~= card.action then
    card.port, card.action = p.port, p.action
    local info = card_info(p)
    card.live = info ~= nil
    if info then card.info = info end
  end
  card.frame = card.live and p.action_frame + 1 or nil
end

-- ---- the input display --------------------------------------------------------------------------------
-- What the game saw from the focused fighter's pad this frame (gd.pad), and a log of inputs with
-- their frame in the sequence: "jump f1 > R f4". A sequence ends after SEQ_IDLE quiet frames.
local inp = { port = nil, pad = nil, prev = 0, sdir = nil, cdir = nil, seqs = {}, last_f = -999 }
local BTN = { { 0x0400, "jump" }, { 0x0800, "jump" }, { 0x0100, "A" }, { 0x0200, "B" }, { 0x0010, "Z" },
  { 0x0040, "L" }, { 0x0020, "R" } }
local DIRS = { "right", "up-right", "up", "up-left", "left", "down-left", "down", "down-right" }

local function dir8(x, y)
  local m = math.sqrt(x * x + y * y) / STICK_MAX
  if m < FLICK then return nil end
  local a = math.atan(y, x)
  return DIRS[(math.floor(a / (math.pi / 4) + 0.5) % 8) + 1]
end

local function inp_add(now, label)
  local s = inp.seqs[1]
  if s == nil or now - inp.last_f > SEQ_IDLE then
    s = { f0 = now, items = {} }
    table.insert(inp.seqs, 1, s)
    while #inp.seqs > SEQ_N do table.remove(inp.seqs) end
  end
  inp.last_f = now
  local f = now - s.f0 + 1
  local last = s.items[#s.items]
  if last and last.f == f and last.label == label then return end
  s.items[#s.items + 1] = { f = f, label = label }
end

local function inp_frame(now)
  if focus > 4 then return end
  if inp.port ~= focus then inp.port, inp.prev, inp.sdir, inp.cdir, inp.seqs = focus, 0, nil, nil, {} end
  local ok, pad = pcall(gd.pad, focus)
  if not ok or pad == nil then return end
  inp.pad = pad
  local b = pad.buttons
  for _, e in ipairs(BTN) do
    if b & e[1] ~= 0 and inp.prev & e[1] == 0 then inp_add(now, e[2]) end
  end
  inp.prev = b
  local sd, cd = dir8(pad.x, pad.y), dir8(pad.cx, pad.cy)
  if sd and sd ~= inp.sdir then inp_add(now, "stick " .. sd) end
  if cd and cd ~= inp.cdir then inp_add(now, "C " .. cd) end
  inp.sdir, inp.cdir = sd, cd
end

-- ---- tech-skill feedback ---------------------------------------------------------------------------------
-- Per port, from the action changes and the game's own counters:
--   L-cancel   an aerial lands in its LandingAir state: the game's L / R / Z press age (player.lr_age,
--              fp->x67F) against its window (lab_common().lcancel_window, 7): under it = cancelled;
--              else "N f early" (up to LC_EARLY_MAX), or a press during the landing lag = "N f late".
--   wavedash   KneeBend > (Jump) > EscapeAir > LandingFallSpecial: how many frames after the jump's
--              first airborne frame the airdodge came (0 = frame-perfect: straight out of KneeBend).
--   waveland   EscapeAir > LandingFallSpecial with no jump just before: the airdodge frame it landed on.
--   ledgedash  off CliffWait (drop or jump) > EscapeAir > LandingFallSpecial: the ledge intangibility
--              left on landing (GALINT, player.intangible).
--   hop        KneeBend > JumpF / JumpB: short or full, by the take-off speed against the fighter's
--              hop and jump speeds; and the jumpsquat's length.
local tech = { st = {}, res = {}, stats = {} }
local TECHS = { "lcancel", "wavedash", "waveland", "ledgedash", "hop" }
local TECH_NAME = { lcancel = "L-cancel", wavedash = "Wavedash", waveland = "Waveland", ledgedash = "Ledgedash",
  hop = "Hops" }

local function stats_of(port)
  local s = tech.stats[port]
  if s == nil then
    s = {}
    for _, k in ipairs(TECHS) do s[k] = { ok = 0, n = 0, sum = 0 } end
    tech.stats[port] = s
  end
  return s
end

-- ok: true = hit, false = miss, nil = not scored (counted only)
local function tech_result(port, kind, ok, text, value)
  local s = stats_of(port)[kind]
  s.n = s.n + 1
  if ok then s.ok = s.ok + 1 end
  if value then s.sum = s.sum + value end
  table.insert(tech.res, 1, { port = port, kind = kind, ok = ok, text = text, frame = gd.match().frame })
  while #tech.res > TECH_N do table.remove(tech.res) end
  log(string.format("P%d %s: %s", port, TECH_NAME[kind], text), ok == true and OK or ok == false and DANGER or ACCENT)
  LD.drill_tech(port, kind, ok, text, value)
end
function LD.reset_tech() tech.st = {} end

local function lcancel_land(p, t, now)
  local age, win = p.lr_age or 255, common().lcancel_window
  if age < win then
    tech_result(p.port, "lcancel", true, age == 0 and "on the landing frame" or string.format("%d f before landing", age))
  elseif age < win + LC_EARLY_MAX then
    tech_result(p.port, "lcancel", false, string.format("%d f early", age - win + 1), age - win + 1)
  else
    t.lc = { f = now, name = p.motion_name } -- no press yet: a press in the landing lag is late
  end
end

local function tech_port(p, now)
  local t = tech.st[p.port]
  if t == nil then t = { prev = p.motion_name } tech.st[p.port] = t end
  local n, pn = p.motion_name, t.prev
  -- a late L-cancel press: the counter restarted after the landing
  if t.lc then
    if n ~= t.lc.name then
      tech_result(p.port, "lcancel", false, "no press")
      t.lc = nil
    elseif (p.lr_age or 255) < now - t.lc.f then
      tech_result(p.port, "lcancel", false, string.format("%d f late", now - (p.lr_age or 0) - t.lc.f))
      t.lc = nil
    end
  end
  if n == pn then return end
  t.prev = n
  if n == "KneeBend" then
    t.ks_f = now
  elseif pn == "KneeBend" and (n == "JumpF" or n == "JumpB") then
    t.jump_f = now
    local a = attrs_of(p)
    local hop, full = a.hop_v_initial_velocity or 0, a.jump_v_initial_velocity or 0
    local short = math.abs(p.vy - hop) < math.abs(p.vy - full)
    tech_result(p.port, "hop", nil, string.format("%s (jumpsquat %d f)", short and "short hop" or "full hop",
      t.ks_f and now - t.ks_f or 0), short and 1 or 0)
  elseif n == "EscapeAir" then
    -- the fighter procs run the animation before the input (fighter.c: proc 0, then proc 3), so the
    -- jump's first airborne frame can already be an airdodge: KneeBend > EscapeAir, JumpF never seen
    if pn == "KneeBend" then t.jump_f = now end
    t.ad_f = now
    t.ad_jump = t.jump_f and now - t.jump_f <= 12 and t.jump_f or nil
    t.ad_ledge = t.ledge_f and now - t.ledge_f <= 60 or false
    t.ad_ks = t.ks_f and t.jump_f and t.jump_f - t.ks_f or nil
  elseif n == "LandingFallSpecial" and pn == "EscapeAir" and t.ad_f then
    if t.ad_ledge then
      local galint = p.intangible or 0
      tech_result(p.port, "ledgedash", galint > 0, galint > 0 and string.format("GALINT %d", galint)
        or "no intangibility left", galint)
    elseif t.ad_jump then
      local late = t.ad_f - t.ad_jump
      tech_result(p.port, "wavedash", late == 0, late == 0 and "frame-perfect airdodge"
        or string.format("airdodge %d f late", late), late)
    else
      local d = now - t.ad_f
      tech_result(p.port, "waveland", nil, string.format("landed on airdodge f%d", d), d)
    end
    t.ad_f, t.ad_jump, t.ad_ledge = nil, nil, false
  elseif n:find("^LandingAir") and pn:find("^AttackAir") then
    lcancel_land(p, t, now)
  end
  if pn == "CliffWait" then
    t.ledge_f = (n == "Fall" or n == "FallAerial" or n:find("^JumpAerial")) and now or nil
  end
  if n == "Landing" and pn:find("^AttackAir") then
    table.insert(tech.res, 1, { port = p.port, kind = "lcancel", text = "autocancelled", frame = now })
    while #tech.res > TECH_N do table.remove(tech.res) end
  end
end

-- ---- per frame, reset ------------------------------------------------------------------------------------------
local prev_lag = {}
function LD.frame()
  if not (cfg.on and gd.match().active) then return end
  local now = gd.match().frame
  local rising = {}
  local list = gd.players()
  for _, p in ipairs(list) do
    if p.in_hitlag and not prev_lag[p.port] then rising[p.port] = true end
  end
  for _, p in ipairs(list) do
    -- a shield hit: hitlag begins while shielding (a hit on the body arrives through on_hit)
    local c = fa.cur
    if rising[p.port] and GUARD[p.motion_name] and not (c and c.v == p.port and c.f0 == now) then
      local a = shield_attacker(p.port, rising)
      if a then fa_start(a, p.port, p.motion_name == "GuardReflect" and "powershield" or "shield") end
    end
    prev_lag[p.port] = p.in_hitlag
    if p.port <= 4 then tech_port(p, now) end
  end
  fa_frame(now)
  card_frame()
  inp_frame(now)
  LD.dummy_frame(now)
  LD.d2_frame(now)
  LD.combo_frame(now)
  LD.drill_frame(now)
end

-- the timeline jumped (step back, a load, a rewind): drop everything in flight, keep the results
function LD.cut()
  fa.cur = nil
  prev_lag = {}
  tech.st = {}
  card.action = nil
  inp.prev, inp.sdir, inp.cdir, inp.last_f = 0, nil, nil, -999
  attrs_cache, common_cache = {}, nil
  LD.dummy_cut()
  LD.d2_cut()
  LD.combo_cut()
  LD.drill_cut()
end

function LD.reset()
  LD.cut()
  fa.hist, tech.res, tech.stats, inp.seqs, card.info = {}, {}, {}, {}, nil
  LD.combo_reset()
end

-- ---- drawing -------------------------------------------------------------------------------------------------
local KIND_WORD = { hit = "on hit", shield = "on shield", powershield = "on powershield" }
local function adv_col(v) return v > 0 and OK or v < 0 and DANGER or BONE end

local function draw_adv()
  local e = fa.hist[1]
  local cx, y = 320, 8
  if e == nil and fa.cur == nil then
    local s = "FRAME ADVANTAGE  hit something, or hit a shield"
    local w = measure(s, "caption") + 24
    quad(cx - w / 2, y, w, 20, GLASS, SHEAR)
    txt(cx, y + 14, s, "caption", DISABLED, "center")
    return
  end
  if e then
    local big = e.adv and string.format("%+d", e.adv) or "--"
    local word = e.adv and (KIND_WORD[e.kind] or e.kind) or e.why
    local sub = string.format("P%d %s > P%d", e.a, e.move, e.v)
    local bw = measure(big, "title")
    local w = math.max(bw + measure(word, "row") + 44, measure(sub, "caption") + 24)
    quad(cx - w / 2, y, w, 48, GLASS, SHEAR)
    quad(cx - w / 2 - 2, y, 4, 48, e.adv and adv_col(e.adv) or DISABLED, SHEAR)
    stxt(cx - w / 2 + 14, y + 28, big, "title", e.adv and adv_col(e.adv) or DISABLED)
    stxt(cx - w / 2 + 24 + bw, y + 26, word, "row", BONE)
    txt(cx - w / 2 + 14, y + 42, sub, "caption", MUTED, "left", w - 24)
    y = y + 52
  end
  -- the older exchanges, small
  local parts = {}
  for i = 2, #fa.hist do
    local h = fa.hist[i]
    parts[#parts + 1] = h.adv and string.format("%+d %s", h.adv, h.kind == "hit" and "hit" or "shd") or h.why
  end
  if fa.cur then parts[#parts + 1] = "..." end
  if #parts > 0 then
    local s = table.concat(parts, "   ")
    local w = measure(s, "caption") + 20
    quad(cx - w / 2, y, w, 16, alpha(GLASS_SOLID, 0xB0))
    txt(cx, y + 12, s, "caption", MUTED, "center")
  end
end

local function draw_card()
  local i = card.info
  local x, y, w = 8, 8, 236
  if i == nil then
    panel(x, y, w, 40, "MOVE")
    txt(x + 12, y + 34, "attack: startup, active, IASA, lag", "caption", DISABLED)
    return
  end
  local h = i.lag and 96 or 80
  panel(x, y, w, h, card.live and "MOVE" or "LAST MOVE", card.live and ACCENT or DISABLED)
  local head = string.format("%s  %s", i.char, i.name)
  txt(x + 82, y + 14, head, "caption", card.live and GOLD or MUTED, "left", w - 92)
  local function cell(cx, cy, label, value, col)
    txt(cx, cy, label, "caption", MUTED)
    txt(cx, cy + 13, value, "body", col or BONE)
  end
  cell(x + 12, y + 30, "startup", i.startup and tostring(i.startup) or "-")
  cell(x + 62, y + 30, "active", i.active ~= "" and i.active or "-", HIT[0])
  cell(x + 152, y + 30, "total", tostring(i.len))
  cell(x + 192, y + 30, "IASA", i.iasa and tostring(i.iasa) or "-", MARK.iasa)
  if i.lag then
    txt(x + 12, y + 70, string.format("landing %d   L-cancel %d   autocancel %s", i.lag, i.lcl, i.ac or "-"),
      "caption", BONE, "left", w - 24)
  end
  -- the move as a bar: active frames in the hitbox colours, IASA in green, the current frame
  local bx0, by0, bw = x + 12, y + h - 18, w - 24
  local len = math.max(i.len, 1)
  local sx = bw / len
  quad(bx0, by0, bw, 8, TRACK)
  for _, win in ipairs(i.windows) do
    quad(bx0 + (win.from - 1) * sx, by0, math.max(1, (win.to - win.from + 1) * sx), 8, HIT[win.id] or HIT[0])
  end
  if i.iasa then quad(bx0 + (i.iasa - 1) * sx, by0 - 2, 2, 12, MARK.iasa) end
  if card.frame then
    local fx = bx0 + math.min(card.frame - 1, len) * sx
    quad(fx, by0 - 3, 2, 14, BONE)
    txt(bx0 + bw, by0 - 4, "f" .. card.frame, "caption", BONE, "right")
  end
end

local function draw_input()
  local x, y, w, h = 8, 336, 316, 112
  panel(x, y, w, h, "INPUT", ACCENT)
  local pad = inp.pad
  txt(x + 70, y + 14, "P" .. focus, "caption", PORT[focus] or BONE)
  -- the sticks: the gate (an octagon), the position
  local function stick(cx, cy, r, sx, sy, col)
    local pts = {}
    for k = 0, 8 do
      local a = k * math.pi / 4
      pts[#pts + 1] = { cx + math.cos(a) * r, cy - math.sin(a) * r }
    end
    for k = 1, 8 do gd.line(pts[k][1], pts[k][2], pts[k + 1][1], pts[k + 1][2], alpha(MUTED, 0xA0)) end
    gd.line(cx - 2, cy, cx + 2, cy, alpha(MUTED, 0x80))
    local px = cx + math.max(-1, math.min(1, (sx or 0) / STICK_MAX)) * r
    local py = cy - math.max(-1, math.min(1, (sy or 0) / STICK_MAX)) * r
    gd.line(cx, cy, px, py, col)
    gd.fill(px - 2, py - 2, 5, 5, col)
  end
  stick(x + 38, y + 56, 24, pad and pad.x, pad and pad.y, BONE)
  stick(x + 88, y + 62, 15, pad and pad.cx, pad and pad.cy, GOLD)
  -- the buttons and the analog triggers
  local b = pad and pad.buttons or 0
  local function btn(bx, by, label, bit, col)
    local on = b & bit ~= 0
    quad(bx, by, 16, 14, on and col or alpha(TRACK, 0xE0))
    txt(bx + 8, by + 11, label, "caption", on and INK or DISABLED, "center")
  end
  btn(x + 14, y + 88, "A", 0x0100, OK)
  btn(x + 32, y + 88, "B", 0x0200, DANGER)
  btn(x + 50, y + 88, "X", 0x0400, BONE)
  btn(x + 68, y + 88, "Y", 0x0800, BONE)
  btn(x + 86, y + 88, "Z", 0x0010, 0x8E72FFFF)
  local function trig(tx, label, v, bit)
    local on = b & bit ~= 0
    quad(tx, y + 24, 8, 32, alpha(TRACK, 0xE0))
    local f = math.min(1, (v or 0) / 255)
    quad(tx, y + 24 + 32 * (1 - f), 8, 32 * f, on and ACCENT or MUTED)
    txt(tx + 4, y + 20, label, "caption", on and ACCENT or DISABLED, "center")
  end
  trig(x + 108, "L", pad and pad.l, 0x0040)
  trig(x + 120, "R", pad and pad.r, 0x0020)
  -- the log: newest sequence on top
  local lx, ly = x + 138, y + 36
  if #inp.seqs == 0 then txt(lx, ly, "inputs show here, with their frame", "caption", DISABLED, "left", w - 146) end
  for k, s in ipairs(inp.seqs) do
    local parts = {}
    for _, it in ipairs(s.items) do parts[#parts + 1] = it.label .. " f" .. it.f end
    txt(lx, ly + (k - 1) * 15, table.concat(parts, " > "), "caption", k == 1 and BONE or MUTED, "left", w - 146)
  end
end

local function draw_tech()
  local x, y, w = 404, 8, 228
  local rows = {}
  local s = stats_of(focus)
  for _, k in ipairs(TECHS) do
    local st = s[k]
    if st.n > 0 then
      local v
      if k == "hop" then v = string.format("SH %d  FH %d", st.sum, st.n - st.sum)
      elseif k == "waveland" then v = string.format("%d, mean f%.1f", st.n, st.sum / st.n)
      elseif k == "ledgedash" then v = string.format("%d / %d  (%d%%)  GALINT %.1f", st.ok, st.n,
        math.floor(100 * st.ok / st.n + 0.5), st.sum / st.n)
      else v = string.format("%d / %d  (%d%%)", st.ok, st.n, math.floor(100 * st.ok / st.n + 0.5)) end
      rows[#rows + 1] = { TECH_NAME[k], v }
    end
  end
  local n = 0
  for _, r in ipairs(tech.res) do if r.port == focus then n = n + 1 end end
  local h = 26 + math.max(1, #rows) * 14 + (n > 0 and 8 + n * 14 or 0)
  panel(x, y, w, h, "TECH")
  txt(x + 64, y + 14, "P" .. focus, "caption", PORT[focus] or BONE)
  local yy = y + 34
  if #rows == 0 then txt(x + 12, yy, "L-cancel, wavedash, waveland, ledgedash, hops", "caption", DISABLED, "left", w - 24) end
  for _, r in ipairs(rows) do
    txt(x + 12, yy, r[1], "caption", MUTED)
    txt(x + w - 12, yy, r[2], "caption", BONE, "right")
    yy = yy + 14
  end
  if n > 0 then
    yy = yy + 8
    quad(x + 12, yy - 12, w - 24, 1, alpha(TICK, 0x90))
    for _, r in ipairs(tech.res) do
      if r.port == focus then
        local col = r.ok == true and OK or r.ok == false and DANGER or ACCENT
        img("lab_mk_hitbox", x + 12, yy - 9, 10, 10, col)
        txt(x + 26, yy, TECH_NAME[r.kind] .. ": " .. r.text, "caption", col, "left", w - 38)
        yy = yy + 14
      end
    end
  end
end

function LD.draw()
  if T("boxes") then LD.draw_d2({ swept = true, shield = true, grab = true }) end
  if T("adv") then draw_adv() end
  if T("card") then draw_card() end
  if T("input") then draw_input() end
  if T("tech") then draw_tech() end
end

-- ---- console ---------------------------------------------------------------------------------------------------
function LD.console(cmd, rest)
  if cmd == "adv" then
    if #fa.hist == 0 then gd.log("adv: no exchange yet") end
    for _, e in ipairs(fa.hist) do
      gd.log(string.format("  f%d P%d %s > P%d: %s", e.frame, e.a, e.move, e.v,
        e.adv and string.format("%+d %s", e.adv, KIND_WORD[e.kind] or e.kind) or e.why))
    end
    if fa.cur then gd.log(string.format("  in flight: P%d > P%d %s since f%d", fa.cur.a, fa.cur.v, fa.cur.kind, fa.cur.f0)) end
  elseif cmd == "tech" then
    if rest == "clear" then tech.stats, tech.res = {}, {} gd.log("tech stats cleared") return true end
    for port, s in pairs(tech.stats) do
      for _, k in ipairs(TECHS) do
        local st = s[k]
        if st.n > 0 then gd.log(string.format("  P%d %-9s %d / %d ok, sum %g", port, TECH_NAME[k], st.ok, st.n, st.sum)) end
      end
    end
    for _, r in ipairs(tech.res) do gd.log(string.format("  f%d P%d %s: %s", r.frame, r.port, TECH_NAME[r.kind], r.text)) end
  elseif cmd == "card" then
    local i = card.info
    if i == nil then gd.log("card: no move yet") return true end
    gd.log(string.format("card: %s %s startup %s active %s total %d IASA %s landing %s L-cancel %s autocancel %s",
      i.char, i.name, tostring(i.startup), i.active, i.len, tostring(i.iasa), tostring(i.lag), tostring(i.lcl),
      tostring(i.ac)))
  elseif cmd == "actionable" then
    for _, p in ipairs(gd.players()) do
      gd.log(string.format("  P%d %s f%d actionable=%s iasa=%s hitlag=%s hitstun=%s lr_age=%s", p.port, p.motion_name,
        p.action_frame + 1, tostring(actionable(p)), tostring(p.iasa), tostring(p.in_hitlag), tostring(p.in_hitstun),
        tostring(p.lr_age)))
    end
  else
    return false
  end
  return true
end

ACTIONS.tr_clear = function()
  LD.reset()
  say("Training readouts cleared")
end
end
stage_d()

local function stage_d3()
-- =================================================================================================
-- ---- stage D3: the dummy (docs/geno.md 14.13) ---------------------------------------------------
-- The Lab plays a fighter on a HUMAN port (CPUs ignore pads) through gd.input, one frame at a time:
-- recordings, DI / ASDI / SDI, tech and getup options, ledge options, what to do after hitstun,
-- shieldstun and landings, a reaction delay, and holds. Offline only. It stands down while the
-- history replays (the replay feeds the logged pads) and while a recording is being made.
-- =================================================================================================
local DUMMY_FILE = "dummy.txt"
local REC_SLOTS, REC_MAX = 4, 600      -- 4 slots of up to 10 s
local STK = 80                         -- a full stick tilt
local B_A, B_B, B_X, B_Z, B_R = 0x0100, 0x0200, 0x0400, 0x0010, 0x0020
local TECH_PRESS_FRAMES = 5            -- press L/R when the floor is this many frames away (the window is 20)
local TECH_LOCKOUT = 41                -- the game ignores a tech press within 40 frames of the last one

local AFTER = { "none", "shield", "spotdodge", "roll away", "roll toward", "jump", "attack", "nair", "grab",
  "slot 1", "slot 2", "slot 3", "slot 4" }
local DI_OPTS = { "none", "in", "out", "survival", "angle", "random" }
local ASDI_OPTS = { "none", "away", "toward", "up", "down" }
local SDI_DIRS = { "away", "toward", "up", "down" }
local TECH_OPTS = { "in place", "away", "toward", "miss", "random" }
local GETUP_OPTS = { "stand", "attack", "away", "toward", "random" }
local LEDGE_OPTS = { "getup", "roll", "attack", "jump", "drop", "ledgedash", "random" }
local PLAY_OPTS = { "off", "in order", "random" }
local HOLD_OPTS = { "none", "shield", "crouch", "jump" }

local dm = {
  port = 2, on = false, play = "off", slot = 1, rec_state = false, -- off until asked: a human P2 keeps its pad
  w_slot = { 1, 1, 1, 1 },
  di = "none", di_angle = 90, asdi = "none", sdi_n = 0, sdi_dir = "away",
  tech = "in place", getup = "stand", ledge = "getup",
  w_tech = { ["in place"] = 1, away = 1, toward = 1, miss = 1 },
  w_getup = { stand = 1, attack = 1, away = 1, toward = 1 },
  w_ledge = { getup = 1, roll = 1, attack = 1, jump = 1, drop = 1, ledgedash = 1 },
  w_di = { none = 1, ["in"] = 1, out = 1, survival = 1 },
  after_hit = "none", after_shield = "none", after_land = "none",
  delay_min = 0, delay_max = 0, pct_lock = -1, inf_shield = false, tilt = "none", hold = "none",
}
local rec = {}          -- rec[slot] = {frames = {{b, x, y, cx, cy, l, r}, ...}, state = bool}
local st = {}           -- the dummy's live state (reset on a cut)
local recording = nil   -- {slot, frames, from}
local pb_keep = nil     -- a playback that loaded its start state survives that load's cut
local released = true

-- ---- settings ------------------------------------------------------------------------------------
local function wstr(w)
  local keys, out = {}, {}
  for k in pairs(w) do keys[#keys + 1] = k end
  table.sort(keys)
  for _, k in ipairs(keys) do out[#out + 1] = k .. ":" .. w[k] end
  return table.concat(out, ",")
end
local function wparse(s, w)
  for k, v in s:gmatch("([^,:]+):(%d+)") do if w[k] ~= nil then w[k] = tonumber(v) end end
end
local function dm_save()
  local out = {}
  for _, k in ipairs({ "port", "on", "play", "slot", "rec_state", "di", "di_angle", "asdi", "sdi_n", "sdi_dir", "tech",
    "getup", "ledge", "after_hit", "after_shield", "after_land", "delay_min", "delay_max", "pct_lock", "inf_shield",
    "tilt", "hold" }) do
    out[#out + 1] = k .. "=" .. tostring(dm[k])
  end
  for _, k in ipairs({ "w_tech", "w_getup", "w_ledge", "w_di" }) do out[#out + 1] = k .. "=" .. wstr(dm[k]) end
  out[#out + 1] = "w_slot=" .. table.concat(dm.w_slot, ",")
  pcall(gd.data_write, DUMMY_FILE, table.concat(out, "\n") .. "\n")
end
local function rec_file(slot) return "dummy_rec" .. slot .. ".txt" end
local function rec_save(slot)
  local r = rec[slot]
  if r == nil then pcall(gd.data_write, rec_file(slot), "") return end
  local out = { "state=" .. tostring(r.state) }
  for _, f in ipairs(r.frames) do out[#out + 1] = table.concat(f, " ") end
  pcall(gd.data_write, rec_file(slot), table.concat(out, "\n") .. "\n")
end
local function dm_load()
  local ok, text = pcall(gd.data_read, DUMMY_FILE)
  if ok and text then
    for k, v in text:gmatch("([%w_]+)=([^\r\n]*)") do
      if k:find("^w_") and k ~= "w_slot" and dm[k] then wparse(v, dm[k])
      elseif k == "w_slot" then
        local i = 0
        for n in v:gmatch("%d+") do i = i + 1 if i <= REC_SLOTS then dm.w_slot[i] = tonumber(n) end end
      elseif type(dm[k]) == "number" then dm[k] = tonumber(v) or dm[k]
      elseif type(dm[k]) == "boolean" then dm[k] = v == "true"
      elseif dm[k] ~= nil then dm[k] = v end
    end
  end
  for slot = 1, REC_SLOTS do
    local ok2, t = pcall(gd.data_read, rec_file(slot))
    if ok2 and t and t ~= "" then
      local r = { frames = {}, state = t:match("state=true") ~= nil }
      for line in t:gmatch("[^\r\n]+") do
        local f = {}
        for n in line:gmatch("%-?%d+") do f[#f + 1] = tonumber(n) end
        if #f == 7 then r.frames[#r.frames + 1] = f end
      end
      if #r.frames > 0 then rec[slot] = r end
    end
  end
end
dm_load()

-- ---- helpers -------------------------------------------------------------------------------------
local function cycle(list, v, d)
  local k = 1
  for i, x in ipairs(list) do if x == v then k = i end end
  return list[((k - 1 + d) % #list) + 1]
end
-- a weighted pick; the keys in sorted order so a seed gives the same choice
local function pick(w)
  local keys, total = {}, 0
  for k, v in pairs(w) do if v > 0 then keys[#keys + 1] = k total = total + v end end
  if total == 0 then return nil end
  table.sort(keys)
  local r = math.random() * total
  for _, k in ipairs(keys) do
    r = r - w[k]
    if r <= 0 then return k end
  end
  return keys[#keys]
end
local function opt(v, w) if v == "random" then return pick(w) end return v end
local function sgn(v) return v < 0 and -1 or 1 end
local function delay()
  local a, b = math.max(0, dm.delay_min), math.max(0, dm.delay_max)
  if b < a then b = a end
  return a + (b > a and math.random(0, b - a) or 0)
end
-- the player the dummy reacts to: the focused fighter, else the nearest other one
local function opponent(p)
  local f = gd.player(focus)
  if f and f.port ~= p.port then return f end
  local best, bd
  for _, q in ipairs(gd.players()) do
    if q.port ~= p.port then
      local d = math.abs(q.x - p.x)
      if bd == nil or d < bd then best, bd = q, d end
    end
  end
  return best
end
local function away_x(p)
  local o = opponent(p)
  return o and sgn(p.x - o.x) or -p.facing
end
local function dir_xy(p, d)
  if d == "away" then return away_x(p) * STK, 0 end
  if d == "toward" then return -away_x(p) * STK, 0 end
  if d == "up" then return 0, STK end
  if d == "down" then return 0, -STK end
  return 0, 0
end

-- an input script: a list of steps {b, x, y, cx, cy, n = frames}
local function run_script(steps, wait, why)
  st.run = { steps = steps, i = 1, left = steps[1] and steps[1].n or 1, at = gd.match().frame + (wait or 0), why = why }
end
local function slot_script(slot)
  local r = rec[slot]
  if r == nil then return nil end
  local steps = {}
  for _, f in ipairs(r.frames) do steps[#steps + 1] = { b = f[1], x = f[2], y = f[3], cx = f[4], cy = f[5], l = f[6], r = f[7], n = 1 } end
  return steps
end
local function after_script(p, what)
  local ax = away_x(p) * STK
  if what == "shield" then return { { b = B_R, r = 255, n = 20 } }
  elseif what == "spotdodge" then return { { b = B_R, r = 255, y = -STK, n = 2 } }
  elseif what == "roll away" then return { { b = B_R, r = 255, x = ax, n = 2 } }
  elseif what == "roll toward" then return { { b = B_R, r = 255, x = -ax, n = 2 } }
  elseif what == "jump" then return { { b = B_X, n = 1 } }
  elseif what == "attack" then return { { b = B_A, n = 1 } }
  elseif what == "nair" then return { { b = B_X, n = 1 }, { n = 2 }, { b = B_A, n = 1 } }
  elseif what == "grab" then return { { b = B_Z, n = 1 } }
  elseif what and what:find("^slot") then return slot_script(tonumber(what:match("%d")))
  end
  return nil
end

-- ---- DI: the same rule as the launch preview (gs_apply_di / kb_preview), from the launch -----------
local function di_stick(vx, vy, how)
  if how == "none" or how == nil then return 0, 0 end
  if how == "angle" then
    local a = math.rad(dm.di_angle)
    return math.cos(a), math.sin(a)
  end
  local t = math.atan(vy, vx)
  local ccx, ccy = -math.sin(t), math.cos(t)
  local dir = vx >= 0 and 1 or -1
  if how == "in" or how == "out" then
    local toward = (ccx * dir < 0) == (how == "in")
    if toward then return ccx, ccy end
    return -ccx, -ccy
  end
  -- survival: the perpendicular whose result is nearer the diagonal (45 / 135 degrees)
  local goal = dir > 0 and math.pi / 4 or 3 * math.pi / 4
  local function turned(sx, sy)
    local m2 = vx * vx + vy * vy
    local f3 = vy * sx - vx * sy
    local f30 = f3 * f3 / m2
    if vx * sy - vy * sx < 0 then f30 = -f30 end
    return t + math.rad(18) * f30
  end
  if math.abs(turned(ccx, ccy) - goal) <= math.abs(turned(-ccx, -ccy) - goal) then return ccx, ccy end
  return -ccx, -ccy
end

function LD.dummy_hit(attacker, victim, info)
  if victim ~= dm.port or not dm.on then return end
  local p, a = gd.player(victim), attacker and gd.player(attacker)
  if p == nil then return end
  local vx, vy = (a and sgn(p.x - a.x) or -p.facing), 1
  if info.angle and gd.kb_preview then
    local ok, r = pcall(gd.kb_preview, victim, { damage = info.damage or info.dealt, angle = info.angle, kbg = info.kbg,
      bkb = info.bkb, wbk = info.wbk, attacker = attacker, percent = math.max(0, p.percent - (info.dealt or 0)) })
    if ok and r then vx, vy = r.vx, r.vy end
  end
  local how = dm.di == "random" and pick(dm.w_di) or dm.di
  local sx, sy = di_stick(vx, vy, how)
  local ax, ay = dir_xy(p, dm.asdi)
  local dx, dy = dir_xy(p, dm.sdi_dir)
  st.hit = { f = gd.match().frame, sx = math.floor(sx * STK + 0.5), sy = math.floor(sy * STK + 0.5), ax = ax, ay = ay,
    sdi_left = dm.sdi_n, sdx = dx, sdy = dy, k = 0, how = how }
  st.run = nil
end

-- ---- the triggers ----------------------------------------------------------------------------------
local LANDING = { Landing = true, LandingFallSpecial = true, LandingAirN = true, LandingAirF = true, LandingAirB = true,
  LandingAirHi = true, LandingAirLw = true }
local DOWN_WAIT = { DownWaitU = true, DownWaitD = true }

local function ledge_script(p, what)
  local tx = -sgn(p.x) * STK -- toward the stage (the legal stages sit around x = 0)
  if what == "getup" then return { { x = tx, n = 2 } }
  elseif what == "roll" then return { { b = B_R, r = 255, n = 1 } }
  elseif what == "attack" then return { { b = B_A, n = 1 } }
  elseif what == "jump" then return { { b = B_X, n = 1 } }
  elseif what == "drop" then return { { y = -STK, n = 1 } }
  elseif what == "ledgedash" then
    -- drop (back), jump, then airdodge down toward the stage: approximate, the right frames vary by fighter
    return { { x = -tx, n = 1 }, { b = B_X, n = 1 }, { n = 2 }, { b = B_R, r = 255, x = math.floor(tx * 0.7), y = -56, n = 1 } }
  end
  return nil
end

local function triggers(p, now)
  local n, pn = p.motion_name, st.prev
  st.prev = n
  -- after hitstun: the first actionable frame once hitstun is over
  if p.in_hitstun then st.stunned = true
  elseif st.stunned and LD.actionable(p) then
    st.stunned = false
    local s = after_script(p, dm.after_hit)
    if s then run_script(s, delay(), "after hitstun") end
  end
  if n == pn then return end
  if pn == "GuardSetOff" and n == "Guard" then
    local s = after_script(p, dm.after_shield)
    if s then run_script(s, delay(), "after shieldstun") end
  end
  if LANDING[n] and not LANDING[pn] then st.landed = true end
  if st.landed and not LANDING[n] then
    st.landed = false
    local s = after_script(p, dm.after_land)
    if s and LD.actionable(p) then run_script(s, delay(), "after landing") end
  end
  if n == "CliffWait" then
    local what = opt(dm.ledge, dm.w_ledge)
    local s = what and ledge_script(p, what)
    if s then run_script(s, delay(), "ledge: " .. what) end
  end
  if DOWN_WAIT[n] then
    local what = opt(dm.getup, dm.w_getup)
    local s
    if what == "stand" then s = { { y = STK, n = 2 } }
    elseif what == "attack" then s = { { b = B_A, n = 1 } }
    elseif what == "away" or what == "toward" then local x = dir_xy(p, what) s = { { x = x, n = 2 } } end
    if s then run_script(s, delay(), "getup: " .. what) end
  end
  if n:find("^Passive") or n:find("^DownBound") then st.fall = nil end
end

-- the tech: while falling in a damage state, press L / R once the floor is close (the game takes a
-- press in the 20 frames before contact, one press per 40 frames); the stick picks in place or a roll
local function tech_spec(p, now)
  local n = p.motion_name
  -- tumble only: its interrupts have no airdodge (ftCo_DamageFall_IASA), so the press is only a tech;
  -- an air damage state out of hitstun would airdodge instead
  local falling = p.airborne and (n == "DamageFall" or n:find("^DamageFly") ~= nil)
  if not falling then
    if not p.airborne then st.fall = nil end
    return nil
  end
  if st.fall == nil then st.fall = { what = opt(dm.tech, dm.w_tech), pressed = nil } end
  local f = st.fall
  if f.what == nil or f.what == "miss" then return nil end
  local x = 0
  if f.what == "away" or f.what == "toward" then x = dir_xy(p, f.what) end
  if f.pressed then return { x = x } end -- hold the roll's direction to the landing
  local vy = (p.vy or 0) + (p.kb_vy or 0)
  if vy >= 0 or (st.last_press and now - st.last_press < TECH_LOCKOUT) then return nil end
  local floor = gd.floor_below(p.x, p.y + 4, 400)
  if floor == nil then return nil end
  if (p.y - floor) / -vy <= TECH_PRESS_FRAMES then
    f.pressed, st.last_press = now, now
    return { b = B_R, r = 255, x = x }
  end
  return nil
end

-- this frame's input for the dummy (applied to the next frame), or nil
local function dummy_spec(p, now)
  -- in hitlag from a hit: SDI (the stick flicks in and out, one flick per 2 frames), then DI + ASDI
  local h = st.hit
  if h and p.in_hitlag then
    h.k = h.k + 1
    if h.sdi_left > 0 then
      if h.k % 2 == 1 then
        h.sdi_left = h.sdi_left - 1
        return { x = h.sdx, y = h.sdy }
      end
      return {}
    end
    return { x = h.sx, y = h.sy, cx = h.ax, cy = h.ay }
  elseif h and now - h.f > 2 then
    st.hit = nil
  end
  local t = tech_spec(p, now)
  if t then return t end
  local r = st.run
  if r and now >= r.at then
    local s = r.steps[r.i]
    if s == nil then st.run = nil
    else
      r.left = r.left - 1
      if r.left <= 0 then
        r.i = r.i + 1
        r.left = r.steps[r.i] and (r.steps[r.i].n or 1) or 0
      end
      return s
    end
  end
  -- playback of the recordings
  if dm.play ~= "off" then
    local pb = st.pb
    if pb == nil or pb.i > #pb.steps then
      local slot
      if dm.play == "in order" then
        slot = st.pb_last or 0
        for _ = 1, REC_SLOTS do
          slot = slot % REC_SLOTS + 1
          if rec[slot] and dm.w_slot[slot] > 0 then break end
        end
      else
        local w = {}
        for k = 1, REC_SLOTS do if rec[k] then w[tostring(k)] = dm.w_slot[k] end end
        slot = tonumber(pick(w) or "")
      end
      if slot == nil or rec[slot] == nil then return nil end
      st.pb_last = slot
      st.pb = { steps = slot_script(slot), i = 1, slot = slot }
      if rec[slot].state and offline() and pcall(gd.loadstate, 2) then pb_keep = st.pb end
      pb = st.pb
    end
    local s = pb.steps[pb.i]
    pb.i = pb.i + 1
    return s
  end
  -- holds
  if dm.hold == "shield" then
    local x, y = 0, 0
    if dm.tilt ~= "none" then x, y = dir_xy(p, dm.tilt) end
    return { b = B_R, r = 255, x = x, y = y }
  elseif dm.hold == "crouch" then return { y = -STK }
  elseif dm.hold == "jump" then return { b = B_X } end
  return nil
end

local function apply(spec)
  if spec == nil then
    if not released then gd.release(dm.port) released = true end
    return
  end
  gd.input(dm.port, { buttons = spec.b or 0, x = spec.x or 0, y = spec.y or 0, cx = spec.cx or 0, cy = spec.cy or 0,
    l = spec.l or 0, r = spec.r or 0 }, 1)
  released = false
end

local function holds(p)
  if dm.pct_lock >= 0 and math.abs(p.percent - dm.pct_lock) >= 0.5 and not p.in_hitlag then
    gd.set_percent(dm.port, dm.pct_lock)
  end
  if dm.inf_shield and gd.set_shield and (p.shield or 60) < 50 then gd.set_shield(dm.port, 60) end
end

local function rec_frame()
  local ok, pad = pcall(gd.pad, dm.port)
  if not ok or pad == nil then return end
  local f = recording.frames
  f[#f + 1] = { pad.buttons, pad.x, pad.y, pad.cx, pad.cy, pad.l, pad.r }
  if #f >= REC_MAX then LD.dummy_record() end
end

function LD.dummy_frame(now)
  if not (offline() and dm.on) then return end
  if gd.history().replaying then st.run, st.pb = nil, nil return end
  local p = gd.player(dm.port)
  if p == nil then return end
  if recording then rec_frame() return end
  if p.cpu then return end
  triggers(p, now)
  holds(p)
  apply(dummy_spec(p, now))
end

function LD.dummy_cut()
  st = { pb = pb_keep, pb_last = pb_keep and pb_keep.slot or nil }
  pb_keep = nil
  if not released and offline() and gd.player(dm.port) then gd.release(dm.port) end
  released = true
end

function LD.dummy_record()
  if not offline() then say("The dummy is offline only", DANGER) return end
  if recording then
    gd.mirror_pad()
    local r = { frames = recording.frames, state = recording.state }
    if #r.frames > 0 then rec[recording.slot] = r end
    rec_save(recording.slot)
    say(string.format("Slot %d: %d frames recorded", recording.slot, #r.frames), OK)
    recording = nil
    return
  end
  if focus == dm.port or focus > 4 then say("Focus your own fighter (F): you drive the dummy from it", DANGER) return end
  local p = gd.player(dm.port)
  if p == nil or p.cpu then say("The dummy has to be a human port", DANGER) return end
  LD.dummy_cut()
  if dm.rec_state then gd.savestate(2) end
  gd.mirror_pad(focus, dm.port, true)
  recording = { slot = dm.slot, frames = {}, state = dm.rec_state }
  say(string.format("Recording slot %d: your controller drives P%d (R again stops)", dm.slot, dm.port), GOLD)
end
function LD.dummy_recording() return recording ~= nil end
LD.dm = dm
function LD.dummy_cfg(k, v) dm[k] = v dm_save() end
function LD.dummy_rec(slot) return rec[slot] end
LD.dummy_pick = pick

ACTIONS.dm_record = LD.dummy_record
ACTIONS.dm_play = function()
  dm.play = dm.play == "off" and "in order" or "off"
  st.pb = nil
  dm_save()
  say("Playback " .. dm.play, dm.play == "off" and DISABLED or ACCENT)
end
STATES.dm_rec = function() return recording ~= nil end
STATES.dm_play = function() return dm.play ~= "off" end

-- ---- the DUMMY tab of the pause menu ------------------------------------------------------------------
local function row(label, icon, desc, key, list)
  return { label = label, icon = icon, desc = desc,
    value = function() return tostring(dm[key]):upper() end,
    run = function() dm[key] = cycle(list, dm[key], 1) dm_save() end,
    adjust = function(d) dm[key] = cycle(list, dm[key], d) dm_save() end }
end
local function num(label, icon, desc, key, lo, hi, step, fmt)
  return { label = label, icon = icon, desc = desc,
    value = function() return fmt and fmt(dm[key]) or tostring(dm[key]) end,
    adjust = function(d) dm[key] = math.max(lo, math.min(hi, dm[key] + d * step)) dm_save() end,
    run = function() dm[key] = dm[key] + step > hi and lo or dm[key] + step dm_save() end }
end
local items = TABS[3].items
local extra = {
  { label = "Dummy", icon = "lab_dummy",
    desc = function()
      local p = gd.player(dm.port)
      local why = p == nil and "  Nobody on that port." or p.cpu and "  That port is a CPU: CPUs ignore pads. Pick a human port with no controller." or ""
      return "The port the Lab plays: every option below acts on it. Left / right: the port; A: on / off." .. why
    end,
    value = function() return "P" .. dm.port .. (dm.on and "" or " OFF") end,
    run = function() dm.on = not dm.on if not dm.on then LD.dummy_cut() end dm_save() end,
    adjust = function(d) LD.dummy_cut() dm.port = ((dm.port - 1 + d) % 4) + 1 dm_save() end },
  { label = "Record slot", icon = "lab_record", key = "R",
    desc = function()
      local r = rec[dm.slot]
      return "Your controller drives the dummy while it records (your fighter stands still). A starts, R in TRAINING too; again stops. "
        .. (r and string.format("Slot %d: %d frames%s.", dm.slot, #r.frames, r.state and ", from its saved state" or "") or "Empty.")
    end,
    value = function() return (recording and "REC " or "SLOT ") .. dm.slot end,
    run = function() menu_close() LD.dummy_record() end,
    adjust = function(d) if not recording then dm.slot = ((dm.slot - 1 + d) % REC_SLOTS) + 1 dm_save() end end },
  { label = "Record from a state", icon = "lab_save",
    desc = "On: starting a recording saves quick slot 2, and each playback of it loads that state first (a loop from the same spot).",
    toggle = function() return dm.rec_state end, value = function() return onoff(dm.rec_state) end,
    run = function() dm.rec_state = not dm.rec_state dm_save() end, adjust = function() dm.rec_state = not dm.rec_state dm_save() end },
  row("Playback", "lab_play", "Play the recorded slots: in order, or at random by each slot's weight (lab dummy w_slot 1,1,0,2).", "play", PLAY_OPTS),
  row("DI", "lab_launch", "Knockback DI on every hit: in, out, survival (toward the diagonal), a fixed stick angle, or random (weights: lab dummy w_di).", "di", DI_OPTS),
  num("DI angle", "lab_launch", "The stick angle for DI \"angle\", in degrees (0 = right, 90 = up).", "di_angle", 0, 345, 15, function(v) return v .. " DEG" end),
  row("ASDI", "lab_mirror", "The C-stick held through hitlag: the small nudge at its end.", "asdi", ASDI_OPTS),
  num("SDI count", "lab_mirror", "How many smash-DI flicks during hitlag (one every 2 frames).", "sdi_n", 0, 8, 1),
  row("SDI direction", "lab_mirror", "Which way the SDI flicks go.", "sdi_dir", SDI_DIRS),
  row("Tech", "lab_ko", "On landing in tumble: in place, roll away, roll toward, miss, or random (weights: lab dummy w_tech).", "tech", TECH_OPTS),
  row("Getup (missed tech)", "lab_ko", "From a missed tech: stand, attack, roll away / toward, or random (lab dummy w_getup).", "getup", GETUP_OPTS),
  row("Ledge", "lab_ledge", "From the ledge: getup, roll, attack, jump, drop, ledgedash (approximate timing), or random (lab dummy w_ledge).", "ledge", LEDGE_OPTS),
  row("After hitstun", "lab_dummy", "The first thing it does when hitstun ends.", "after_hit", AFTER),
  row("After shieldstun", "lab_dummy", "Out of shield, the frame shieldstun ends.", "after_shield", AFTER),
  row("After landing", "lab_dummy", "When a landing's lag ends.", "after_land", AFTER),
  num("Reaction min", "lab_slowmo", "Every response waits at least this many frames. Humans are not frame-perfect.", "delay_min", 0, 60, 1, function(v) return v .. " F" end),
  num("Reaction max", "lab_slowmo", "And at most this many: a random delay between the two.", "delay_max", 0, 60, 1, function(v) return v .. " F" end),
  num("Percent lock", "lab_percent", "Put back to this percent after every hit. OFF below 0.", "pct_lock", -10, 300, 10,
    function(v) return v < 0 and "OFF" or (v .. "%") end),
  { label = "Infinite shield", icon = "lab_hitbox", desc = "The shield is refilled whenever it gets low.",
    toggle = function() return dm.inf_shield end, value = function() return onoff(dm.inf_shield) end,
    run = function() dm.inf_shield = not dm.inf_shield dm_save() end, adjust = function() dm.inf_shield = not dm.inf_shield dm_save() end },
  row("Hold", "lab_pause", "What it does when nothing else is going on: shield, crouch, jump, or nothing.", "hold", HOLD_OPTS),
  row("Shield tilt", "lab_mirror", "While holding shield, the stick tilts it this way.", "tilt", { "none", "away", "toward", "up", "down" }),
}
for _, it in ipairs(extra) do items[#items + 1] = it end

-- ---- console: lab dummy [key value] -------------------------------------------------------------------
function LD.dummy_console(rest)
  local k, v = rest:match("^(%S+)%s+(.+)$")
  if k == nil then
    local keys = {}
    for key, val in pairs(dm) do
      keys[#keys + 1] = key .. "=" .. (type(val) == "table" and (key == "w_slot" and table.concat(val, ",") or wstr(val)) or tostring(val))
    end
    table.sort(keys)
    gd.log("dummy: " .. table.concat(keys, "  "))
    for slot = 1, REC_SLOTS do
      if rec[slot] then gd.log(string.format("  slot %d: %d frames%s", slot, #rec[slot].frames, rec[slot].state and " (state)" or "")) end
    end
    return
  end
  if k == "w_slot" then
    local i = 0
    for n in v:gmatch("%d+") do i = i + 1 if i <= REC_SLOTS then dm.w_slot[i] = tonumber(n) end end
  elseif k:find("^w_") and type(dm[k]) == "table" then wparse(v, dm[k])
  elseif type(dm[k]) == "number" then dm[k] = tonumber(v) or dm[k]
  elseif type(dm[k]) == "boolean" then dm[k] = v == "true" or v == "on"
  elseif dm[k] ~= nil then dm[k] = v
  else gd.log("dummy: no setting " .. k) return end
  dm_save()
  gd.log("dummy " .. k .. " = " .. v)
end
end
stage_d3()

local function stage_d2()
-- =================================================================================================
-- ---- stage D2: hitbox display upgrades (docs/geno.md 14.14) --------------------------------------
-- Swept hitboxes (the capsule from last frame's position to this frame's: the shape Melee tests),
-- with fading ghosts; hurtboxes by state; the shield bubble; grab boxes. Drawn by the Lab over the
-- game, in HITBOXES (W U S C) and, with B, in TRAINING and COMBO.
-- =================================================================================================
local GHOSTS = 4
local COL_HURT = { normal = 0xF7DF5EFF, invincible = 0x27B88AFF, intangible = 0x4D8DFFFF }
local ghosts = {} -- ghosts[port] = { {caps}, ... } newest first
local last_f = -1

local function circle(cx, cy, r, col, seg)
  seg = seg or 16
  local px, py = cx + r, cy
  for k = 1, seg do
    local a = k * 2 * math.pi / seg
    local x, y = cx + math.cos(a) * r, cy + math.sin(a) * r
    gd.line(px, py, x, y, col)
    px, py = x, y
  end
end

-- a capsule between two world points, radius r (world units), outlined in screen space
local function capsule(ax, ay, bx, by, r, col)
  local sax, say = gd.project(ax, ay, 0)
  local sbx, sby = gd.project(bx, by, 0)
  local rx = gd.project(ax + r, ay, 0)
  if sax == nil or sbx == nil or rx == nil then return end
  local sr = math.abs(rx - sax)
  circle(sax, say, sr, col)
  local dx, dy = sbx - sax, sby - say
  local len = math.sqrt(dx * dx + dy * dy)
  if len > 0.5 then
    circle(sbx, sby, sr, col)
    local nx, ny = -dy / len * sr, dx / len * sr
    gd.line(sax + nx, say + ny, sbx + nx, sby + ny, col)
    gd.line(sax - nx, say - ny, sbx - nx, sby - ny, col)
  end
end
LD.capsule, LD.circle = capsule, circle

-- once per game frame: remember each fighter's swept hitboxes for the ghosts
function LD.d2_frame(now)
  if now == last_f then return end
  last_f = now
  for _, p in ipairs(gd.players()) do
    local caps = {}
    for _, h in ipairs(p.hitboxes) do
      if h.element_name ~= "catch" then caps[#caps + 1] = { h.px, h.py, h.x, h.y, h.radius, h.id } end
    end
    local g = ghosts[p.port] or {}
    table.insert(g, 1, caps)
    while #g > GHOSTS + 1 do table.remove(g) end
    ghosts[p.port] = g
  end
end
function LD.d2_cut() ghosts, last_f = {}, -1 end

local function draw_swept(p)
  local g = ghosts[p.port]
  if g == nil then return end
  for age = #g, 1, -1 do
    local a = age == 1 and 0xFF or math.floor(0xB0 / age)
    for _, c in ipairs(g[age]) do capsule(c[1], c[2], c[3], c[4], c[5], alpha(HIT[c[6]] or HIT[0], a)) end
  end
end

local function draw_hurt(p)
  local whole = p.intangible > 0 and "intangible" or p.invincible > 0 and "invincible"
    or (p.body_state ~= "normal" and p.body_state) or nil
  local hb = gd.hurtboxes(p.port) or {}
  for _, h in ipairs(hb) do
    local s = whole or h.state
    capsule(h.ax, h.ay, h.bx, h.by, h.radius, alpha(COL_HURT[s] or COL_HURT.normal, s == "normal" and 0x70 or 0xE0))
  end
  if whole then
    local sx, sy = gd.project(p.x, p.y, 0)
    if sx then
      local left = p.intangible > 0 and p.intangible or p.invincible
      local s = whole:upper() .. (left > 0 and (" " .. left) or "")
      quad(sx - measure(s) / 2 - 4, sy + 4, measure(s) + 8, 13, alpha(GLASS_SOLID, 0xC0))
      txt(sx, sy + 14, s, "caption", COL_HURT[whole] or BONE, "center")
    end
  end
end

local function draw_shield(p)
  if not p.shield_on or (p.shield_r or 0) <= 0 then return end
  local sx, sy = gd.project(p.shield_x, p.shield_y, 0)
  local rx = gd.project(p.shield_x + p.shield_r, p.shield_y, 0)
  if sx == nil or rx == nil then return end
  local col = p.shield > 30 and ACCENT or p.shield > 15 and GOLD or DANGER
  circle(sx, sy, math.abs(rx - sx), col, 24)
  txt(sx, sy - math.abs(rx - sx) - 4, string.format("%.1f", p.shield), "caption", col, "center")
end

local function draw_grab(p)
  for _, h in ipairs(p.hitboxes) do
    if h.element_name == "catch" then
      capsule(h.px, h.py, h.x, h.y, h.radius, 0xC77DFFFF)
      local sx, sy = gd.project(h.x, h.y, 0)
      if sx then txt(sx, sy - 6, "GRAB", "caption", 0xC77DFFFF, "center") end
    end
  end
end

-- `which`: a table of the overlays to draw (swept, hurt, shield, grab)
function LD.draw_d2(which)
  for _, p in ipairs(gd.players()) do
    if which.hurt then draw_hurt(p) end
    if which.swept then draw_swept(p) end
    if which.grab then draw_grab(p) end
    if which.shield then draw_shield(p) end
  end
end
end
stage_d2()

local function stage_d4()
-- =================================================================================================
-- ---- stage D4: combo analysis (docs/geno.md 14.15) -----------------------------------------------
-- COMBO mode (0). For each hit on a victim, the victim's first actionable frame after it (the same
-- reading as frame advantage) against the next hit: TRUE when the next hit landed first, else the
-- escape window (how many frames the victim could act, and with what: jump, airdodge, shield...).
-- A combo is a run of hits by one attacker on one victim; it drops when the victim acts first, with
-- the reason. For the last hit, the launch for each DI (the knockback preview) shows how DI moves it.
-- The punish finder is later (decision 2).
-- =================================================================================================
local COMBO_N = 5
local cb = { cur = nil, hist = {}, di = nil }

local function options(p)
  local o = {}
  if p.airborne then
    if (p.jumps_left or 0) > 0 then o[#o + 1] = "jump" end
    o[#o + 1] = "airdodge"
    o[#o + 1] = "aerial"
  else
    o[#o + 1] = "shield"
    o[#o + 1] = "jump"
    o[#o + 1] = "spotdodge"
  end
  return table.concat(o, " / ")
end

local function end_combo(c, why)
  cb.cur = nil
  c.why = why
  if #c.hits >= 2 or c.escape then
    table.insert(cb.hist, 1, c)
    while #cb.hist > COMBO_N do table.remove(cb.hist) end
    log(string.format("P%d combo on P%d: %d hits %.1f%% - %s", c.a, c.v, #c.hits, c.dmg, why), GOLD)
  end
end

function LD.combo_hit(attacker, victim, info)
  if attacker == nil or victim == nil or attacker == victim or info.item then return end
  local now = gd.match().frame
  local c = cb.cur
  local pa = gd.player(attacker)
  local hit = { f = now, move = pa and pa.motion_name or "?", dmg = info.dealt or 0 }
  if c and c.a == attacker and c.v == victim then
    local prev = c.hits[#c.hits]
    if prev and prev.tv and prev.tv < now then
      -- the victim could act before this hit: not a true combo from the last hit
      hit.gap = now - prev.tv
      hit.opts = prev.opts
      c.escape = c.escape or hit
    else
      hit.gap = 0
    end
    c.hits[#c.hits + 1] = hit
    c.dmg = c.dmg + hit.dmg
  else
    if c then end_combo(c, "a new exchange") end
    c = { a = attacker, v = victim, f0 = now, hits = { hit }, dmg = hit.dmg }
    cb.cur = c
  end
  -- the DI fan for this hit: where each DI choice sends the victim
  cb.di = nil
  local pv = gd.player(victim)
  if pv and info.angle and gd.kb_preview then
    local fan = {}
    for _, how in ipairs({ "none", "in", "out", "survival" }) do
      local ok, r = pcall(gd.kb_preview, victim, { damage = info.damage or info.dealt, angle = info.angle, kbg = info.kbg,
        bkb = info.bkb, wbk = info.wbk, attacker = attacker, percent = math.max(0, pv.percent - (info.dealt or 0)), di = how })
      if ok and r then fan[#fan + 1] = { how = how, angle = r.angle_di, hitstun = r.hitstun, points = r.points } end
    end
    cb.di = { victim = victim, fan = fan, f = now }
  end
end

-- per frame: the victim's first actionable frame after the last hit; a combo drops when the victim
-- has been actionable long enough that nothing is following up (30 frames), or lands a hit of its own
function LD.combo_frame(now)
  local c = cb.cur
  if c == nil then return end
  local pv = gd.player(c.v)
  if pv == nil then cb.cur = nil return end
  local last = c.hits[#c.hits]
  if last.tv == nil and LD.actionable(pv) and now > last.f then
    last.tv = now
    last.opts = options(pv)
  end
  if last.tv and now - last.tv >= 30 then
    end_combo(c, string.format("dropped: P%d could act for %d f (%s)", c.v, now - last.tv, last.opts))
  end
end

function LD.combo_victim_hit(attacker, victim)
  local c = cb.cur
  if c and attacker == c.v then end_combo(c, "P" .. c.v .. " hit back") end
end

function LD.combo_cut() cb.cur = nil end
function LD.combo_reset() cb.cur, cb.hist, cb.di = nil, {}, nil end

local DI_COL = { none = BONE, ["in"] = 0x4D8DFFFF, out = DANGER, survival = OK }
function LD.draw_combo(show_fan)
  local c = cb.cur or cb.hist[1]
  local x, y, w = 8, 8, 300
  if c == nil then
    panel(x, y, w, 40, "COMBO")
    txt(x + 12, y + 34, "hit someone twice: true or escapable, per hit", "caption", DISABLED)
  else
    local n = math.min(#c.hits, 8)
    local h = 44 + n * 14 + 16
    panel(x, y, w, h, cb.cur and "COMBO" or "LAST COMBO", cb.cur and GOLD or DISABLED)
    txt(x + 104, y + 14, string.format("P%d > P%d  %d hits  %.1f%%", c.a, c.v, #c.hits, c.dmg), "caption", GOLD, "left", w - 114)
    local yy = y + 34
    for i = math.max(1, #c.hits - n + 1), #c.hits do
      local hit = c.hits[i]
      local verdict, col
      if i == 1 then verdict, col = "opener", MUTED
      elseif hit.gap == 0 then verdict, col = "TRUE", OK
      else verdict, col = string.format("escapable %d f: %s", hit.gap, hit.opts or "?"), DANGER end
      txt(x + 12, yy, string.format("%d  %s  %.1f%%", i, hit.move, hit.dmg), "caption", BONE, "left", 130)
      txt(x + 144, yy, verdict, "caption", col, "left", w - 154)
      yy = yy + 14
    end
    if c.why then txt(x + 12, yy + 4, c.why, "caption", MUTED, "left", w - 24) end
  end
  -- the older combos
  local parts = {}
  for i = cb.cur and 1 or 2, #cb.hist do
    local o = cb.hist[i]
    parts[#parts + 1] = string.format("%dh %.0f%%%s", #o.hits, o.dmg, o.escape and "" or " true")
  end
  if #parts > 0 then txt(x + 12, 470 - 30, "before: " .. table.concat(parts, "   "), "caption", MUTED) end
  -- the DI fan of the last hit: the flight for each DI to the end of hitstun
  if show_fan and cb.di and gd.match().frame - cb.di.f < 90 then
    local lx, ly = 404, 8
    panel(lx, ly, 228, 20 + #cb.di.fan * 14 + 8, "DI")
    for i, f in ipairs(cb.di.fan) do
      txt(lx + 12, ly + 20 + i * 14, string.format("%-8s %5.1f deg  hitstun %d", f.how, f.angle or 0, f.hitstun or 0), "caption",
        DI_COL[f.how] or BONE)
      local prev
      for k, pt in ipairs(f.points or {}) do
        local sx, sy = gd.project(pt[1] or pt.x, pt[2] or pt.y, 0)
        if sx and prev then gd.line(prev[1], prev[2], sx, sy, alpha(DI_COL[f.how] or BONE, 0xC0)) end
        if sx then prev = { sx, sy } end
        if k > 120 then break end
      end
    end
  end
end

function LD.combo_console()
  local list = {}
  if cb.cur then list[#list + 1] = cb.cur end
  for _, c in ipairs(cb.hist) do list[#list + 1] = c end
  if #list == 0 then gd.log("combo: none yet") end
  for _, c in ipairs(list) do
    gd.log(string.format("P%d > P%d from f%d: %d hits %.1f%% %s", c.a, c.v, c.f0, #c.hits, c.dmg, c.why or "(running)"))
    for i, h in ipairs(c.hits) do
      gd.log(string.format("  %d f%d %s %.1f%% %s", i, h.f, h.move, h.dmg,
        i == 1 and "opener" or h.gap == 0 and "TRUE" or string.format("escapable %d f (%s)", h.gap, h.opts or "?")))
    end
  end
end
end
stage_d4()

local function stage_d5()
-- =================================================================================================
-- ---- stage D5: scenarios and drills (docs/geno.md 14.16) -----------------------------------------
-- A scenario is data: a start (a library state, or the match as it is), dummy settings, a rule that
-- scores each attempt, a scoring kind (streak or rate) and a limit (attempts or seconds). The rules
-- are the only code; a new drill on an existing rule is a file in scripts-data/geno-lab_lab/drills/
-- (docs/geno.md has the format). Results go to drills/results.txt; the best per drill is kept.
-- =================================================================================================
local RESULTS = "drills/results.txt"

-- the built-in drills (the same format a drills/<id>.txt file uses)
local BUILTIN = {
  { id = "lcancel", name = "L-cancel streak", rule = "tech:lcancel", score = "streak", attempts = 0, seconds = 60,
    desc = "Land aerials and L-cancel them. The score is your longest streak; the hit rate is kept too. A plain landing (autocancel) does not count." },
  { id = "techchase", name = "Tech chase", rule = "techchase", score = "rate", attempts = 20, seconds = 0,
    window = 40, ["dummy.on"] = "true", ["dummy.tech"] = "random", ["dummy.getup"] = "random", ["dummy.w_tech"] = "in place:1,away:1,toward:1,miss:1",
    desc = "Knock the dummy down: it techs in place, away, toward or misses at random. Hit it within 40 frames of its tech or getup starting. The score is your rate; your reaction time is shown." },
  { id = "ledgedash", name = "Ledgedash consistency", rule = "tech:ledgedash", score = "rate", attempts = 20, seconds = 0,
    desc = "Ledgedash 20 times. A hit is a landing with ledge intangibility left (GALINT above 0); the mean GALINT is shown." },
  { id = "wavedash", name = "Wavedash timing", rule = "tech:wavedash", score = "rate", attempts = 20, seconds = 0,
    desc = "Wavedash 20 times. A hit is a frame-perfect airdodge." },
}

local drills = {}
local function parse_drill(text, id)
  local d = { id = id }
  for k, v in text:gmatch("([%w_%.]+)%s*=%s*([^\r\n]*)") do d[k] = tonumber(v) or v end
  if d.name and d.rule then return d end
  return nil
end
local function load_drills()
  drills = {}
  for _, d in ipairs(BUILTIN) do drills[#drills + 1] = d end
  local ok, idx = pcall(gd.data_read, "drills/index.txt")
  if ok and idx then
    for id in idx:gmatch("[%w_%-]+") do
      local ok2, t = pcall(gd.data_read, "drills/" .. id .. ".txt")
      local d = ok2 and t and parse_drill(t, id)
      if d then drills[#drills + 1] = d end
    end
  end
end
load_drills()

local best = {}  -- best[id] = score
local function load_best()
  local ok, t = pcall(gd.data_read, RESULTS)
  if not (ok and t) then return end
  for id, score in t:gmatch("[^\r\n]-%s(%S+)%s+score%s+([%d%.]+)") do
    local s = tonumber(score)
    if s and (best[id] == nil or s > best[id]) then best[id] = s end
  end
end
load_best()

local dr = nil      -- the running drill
local last = nil    -- the last finished drill's results (the results panel)

local function score_of(r)
  if r.d.score == "streak" then return r.best_streak end
  return r.n > 0 and math.floor(100 * r.ok / r.n + 0.5) or 0
end

local function finish(why)
  if dr == nil then return end
  local r = dr
  dr = nil
  local s = score_of(r)
  local id = r.d.id
  local was = best[id]
  if was == nil or s > was then best[id] = s end
  last = { d = r.d, score = s, was = was, n = r.n, ok = r.ok, streak = r.best_streak, log = r.log, why = why,
    mean = r.nv > 0 and r.sum / r.nv or nil, until_t = gd.time() + 10 }
  local ok, old = pcall(gd.data_read, RESULTS)
  local line = string.format("%s %s score %d n %d ok %d streak %d%s", gd.lab_now and gd.lab_now(true) or "?", id, s, r.n, r.ok,
    r.best_streak, last.mean and string.format(" mean %.2f", last.mean) or "")
  pcall(gd.data_write, RESULTS, ((ok and old) or "") .. line .. "\n")
  for _, k in ipairs(r.restore or {}) do LD.dummy_cfg(k[1], k[2]) end
  log("drill " .. r.d.name .. ": " .. s .. (r.d.score == "streak" and " streak" or "%"), GOLD)
  say(string.format("%s: %d%s%s", r.d.name, s, r.d.score == "streak" and " streak" or "%",
    (was == nil or s > was) and "  NEW BEST" or ""), GOLD)
end

local function attempt(ok, text, value)
  local r = dr
  r.n = r.n + 1
  if ok then
    r.ok = r.ok + 1
    r.streak = r.streak + 1
    if r.streak > r.best_streak then r.best_streak = r.streak end
  else
    r.streak = 0
  end
  if value then r.sum, r.nv = r.sum + value, r.nv + 1 end
  table.insert(r.log, 1, { ok = ok, text = text })
  while #r.log > 8 do table.remove(r.log) end
  if r.d.attempts and r.d.attempts > 0 and r.n >= r.d.attempts then finish("done") end
end

-- ---- the rules ------------------------------------------------------------------------------------------
-- tech:<kind>  one attempt per tech result of that kind (stage D1's reading), on the player's port
-- techchase    the dummy techs or gets up; the player hits it within `window` frames of that start
local TECH_STARTS = { PassiveStandF = "tech away/toward", PassiveStandB = "tech away/toward", Passive = "tech in place",
  DownStandU = "getup stand", DownStandD = "getup stand", DownAttackU = "getup attack", DownAttackD = "getup attack",
  DownFowardU = "getup roll", DownFowardD = "getup roll", DownBackU = "getup roll", DownBackD = "getup roll" }

function LD.drill_tech(port, kind, ok, text, value)
  if dr == nil or port ~= dr.port then return end
  local want = dr.d.rule:match("^tech:(%w+)$")
  if want and want == kind and ok ~= nil then attempt(ok, text, value) end
end

function LD.drill_hit(attacker, victim)
  if dr == nil or dr.d.rule ~= "techchase" then return end
  local o = dr.chase
  if o and attacker == dr.port and victim == LD.dm.port then
    local rt = gd.match().frame - o.f
    dr.chase = nil
    attempt(true, string.format("%s: hit %d f after", o.what, rt), rt)
  end
end

function LD.drill_frame(now)
  if dr == nil then return end
  if dr.d.seconds and dr.d.seconds > 0 and now - dr.f0 >= dr.d.seconds * 60 then finish("time") return end
  if dr.d.rule == "techchase" then
    local p = gd.player(LD.dm.port)
    if p == nil then return end
    local n = p.motion_name
    if n ~= dr.prev and TECH_STARTS[n] then
      if dr.chase then attempt(false, dr.chase.what .. ": missed") end
      dr.chase = { f = now, what = TECH_STARTS[n] }
    end
    dr.prev = n
    local w = dr.d.window or 40
    if dr and dr.chase and now - dr.chase.f > w then
      local what = dr.chase.what
      dr.chase = nil
      attempt(false, what .. ": too slow")
    end
  end
end

local function start(d)
  if not offline() then say("Drills are offline only", DANGER) return end
  if dr then finish("stopped") end
  -- the drill's dummy settings, put back when it ends
  local restore = {}
  for k, v in pairs(d) do
    local key = type(k) == "string" and k:match("^dummy%.(.+)$")
    if key and LD.dm[key] ~= nil then
      local cur = LD.dm[key]
      local keep = cur
      if type(cur) == "table" then keep = {} for kk, vv in pairs(cur) do keep[kk] = vv end end
      restore[#restore + 1] = { key, keep }
      if type(cur) == "table" then LD.dummy_console(key .. " " .. v)
      elseif type(cur) == "boolean" then LD.dummy_cfg(key, v == "true" or v == true)
      else LD.dummy_cfg(key, tonumber(v) or v) end
    end
  end
  if d.state and gd.state_load then
    local ok, why = gd.state_load(d.state)
    if not ok then say("Drill state: " .. tostring(why), DANGER) end
  end
  LD.reset_tech()
  dr = { d = d, port = focus, f0 = gd.match().frame, n = 0, ok = 0, streak = 0, best_streak = 0, sum = 0, nv = 0, log = {},
    restore = restore }
  last = nil
  say(d.name .. ": go", GOLD)
end
LD.drill_start = start
function LD.drill_running() return dr ~= nil end
function LD.drill_cut() if dr then dr.chase = nil end end

-- ---- the HUD and the results -----------------------------------------------------------------------
function LD.draw_drill()
  if dr then
    local r = dr
    local left = ""
    if r.d.attempts and r.d.attempts > 0 then left = string.format("%d / %d", r.n, r.d.attempts) end
    if r.d.seconds and r.d.seconds > 0 then
      left = left .. string.format("  %ds", math.max(0, r.d.seconds - math.floor((gd.match().frame - r.f0) / 60)))
    end
    local main = r.d.score == "streak" and string.format("streak %d  best %d", r.streak, r.best_streak)
      or string.format("%d / %d  %d%%", r.ok, r.n, r.n > 0 and math.floor(100 * r.ok / r.n + 0.5) or 0)
    local w = 300
    local x, y = 320 - w / 2, 84
    quad(x, y, w, 40, GLASS, SHEAR)
    quad(x - 2, y, 4, 40, GOLD, SHEAR)
    stxt(x + 14, y + 17, r.d.name:upper(), "row", GOLD)
    txt(x + w - 12, y + 15, left, "caption", MUTED, "right")
    txt(x + 14, y + 33, main, "body", BONE)
    local e = r.log[1]
    if e then txt(x + w - 12, y + 33, e.text, "caption", e.ok and OK or DANGER, "right", 150) end
  elseif last and gd.time() < last.until_t then
    local L = last
    local x, y, w = 170, 150, 300
    local h = 70 + math.min(#L.log, 6) * 14
    panel(x, y, w, h, "RESULTS", GOLD)
    stxt(x + 12, y + 40, L.d.name:upper(), "row", BONE)
    txt(x + w - 12, y + 40, string.format("%d%s", L.score, L.d.score == "streak" and " streak" or "%"), "body", GOLD, "right")
    txt(x + 12, y + 56, string.format("%d / %d hit%s   best before: %s%s", L.ok, L.n,
      L.mean and string.format("   mean %.1f", L.mean) or "", L.was and tostring(L.was) or "-",
      (L.was == nil or L.score > L.was) and "   NEW BEST" or ""), "caption", MUTED, "left", w - 24)
    for i = 1, math.min(#L.log, 6) do
      local e = L.log[i]
      txt(x + 12, y + 56 + i * 14, e.text, "caption", e.ok and OK or DANGER, "left", w - 24)
    end
  end
end
function LD.dismiss_results() last = nil end

-- ---- the DRILLS tab ------------------------------------------------------------------------------------
local function drill_items()
  local items = {}
  if dr then
    items[#items + 1] = { label = "Stop the drill", icon = "lab_pause",
      desc = "End it now and score what you have.", value = function() return dr and dr.d.name:upper() or "" end,
      run = function() finish("stopped") menu_close() end }
  end
  for _, d in ipairs(drills) do
    local dd = d
    items[#items + 1] = { label = dd.name, icon = "lab_record",
      desc = function()
        return dd.desc .. (best[dd.id] and string.format("  Best: %d%s.", best[dd.id], dd.score == "streak" and " streak" or "%") or "")
      end,
      value = function() return best[dd.id] and ("BEST " .. best[dd.id]) or "NEW" end,
      run = function() menu_close() start(dd) end }
  end
  items[#items + 1] = { label = "Reload drill files", icon = "lab_reload",
    desc = "Read scripts-data/geno-lab_lab/drills/index.txt again: one id per line, each drills/<id>.txt a drill (docs/geno.md 14.16).",
    value = function() return #drills .. " DRILLS" end,
    run = function() load_drills() say(#drills .. " drills") end }
  return items
end
-- before EXIT
table.insert(TABS, #TABS, { name = "DRILLS", icon = "lab_record", items = drill_items })

function LD.drill_console(rest)
  local id = rest:match("^(%S*)")
  if id == "" then
    for _, d in ipairs(drills) do gd.log(string.format("  %-10s %s (%s, %s)%s", d.id, d.name, d.rule, d.score,
      best[d.id] and ("  best " .. best[d.id]) or "")) end
    if dr then gd.log(string.format("running: %s  %d / %d  streak %d", dr.d.name, dr.ok, dr.n, dr.streak)) end
    return
  end
  if id == "stop" then finish("stopped") return end
  for _, d in ipairs(drills) do if d.id == id then start(d) gd.log("drill " .. d.name) return end end
  gd.log("lab drill [id | stop]  (lab drill lists them)")
end
end
stage_d5()

ACTIONS.cb_clear = function() LD.combo_reset() say("Combos cleared") end

-- ---- ticks ----------------------------------------------------------------------------------------
local function mode_keys()
  local md = mode()
  for _, t in ipairs(md.t) do
    if gd.key_pressed(t.k) then
      local v = flip(md.id, t.id)
      say(t.label .. (v and "  on" or "  off"), v and ACCENT or DISABLED)
    end
  end
  if not offline() then return end
  for _, a in ipairs(md.a) do
    if (a.rep and repeat_key(a.k)) or (not a.rep and gd.key_pressed(a.k)) then ACTIONS[a.run]() end
  end
end

function on_tick()
  if lab_menu_tick() then return end -- the menu owns every key while it is open
  if not cfg.on then return end
  if LE.tick() then return end -- the frame-data export owns the tick while it runs
  if gd.key_pressed("H") then
    cfg.hidden = not cfg.hidden
    if cfg.hidden then restore_draw() end
    save_settings()
  end
  if gd.key_pressed("F3") then cfg.help = not cfg.help end
  if gd.key_pressed("TAB") then set_mode(cfg.mode + (gd.key("SHIFT") and -1 or 1)) end
  for i = 1, #MODES do if gd.key_pressed(i == 10 and "0" or tostring(i)) then set_mode(i) end end
  if gd.key_pressed("F") then next_port(1) end
  mode_keys()
  if offline() then
    if gd.key_pressed("SPACE") then
      if gd.paused() then gd.resume() else ensure_history() gd.pause() end
    end
    local big = gd.key("CTRL") and 10 or 1
    if repeat_key("RIGHT") then step(big) end
    if repeat_key("LEFT") then back(big) end
    if gd.key_pressed("F5") then gd.savestate(1) say("State 1 saved") end
    if gd.key_pressed("F8") then hot_reload() end
    if gd.key_pressed("G") and gd.history().replaying then gd.rewind_live() say("Live from here") end
    if gd.key_pressed("F6") then
      local ok, err = pcall(gd.loadstate, 1)
      if not ok then say((tostring(err):gsub("^.-: ", "")), DANGER) end
    end
  end
  if gd.match().active then
    ensure_history()
    apply_draw()
    if gd.player(focus) == nil then next_port(1) end
  end
end

-- ---- engine events ---------------------------------------------------------------------------------
local function pname(port) return port and ("P" .. port) or "item" end

function on_hit(attacker, victim, info)
  if not cfg.on then return end
  LE.on_hit(attacker, victim, info)
  LD.on_hit(attacker, victim, info)
  local text
  if info.angle then
    text = string.format("%s hit %s  #%s  %.1f%%  a%d  kbg %d  bkb %d  wbk %d  %s", pname(attacker),
      pname(victim), tostring(info.hitbox), info.dealt, info.angle, info.kbg, info.bkb, info.wbk, info.element_name)
  else
    text = string.format("%s hit %s  %.1f%%%s", pname(attacker), pname(victim), info.dealt, info.item and "  (item)" or "")
  end
  log(text, PORT[attacker or 6])
end

function on_hitlag(port, entering)
  if not cfg.on then return end
  local p = gd.player(port)
  log(string.format("%s hitlag %s%s", pname(port), entering and "on" or "off",
    (entering and p) and string.format(" (%.0f f)", p.hitlag) or ""), DISABLED)
end

function on_land(port, motion)
  if not cfg.on then return end
  LE.on_land(port)
  log(string.format("%s land from %s", pname(port), gd.motion_name(motion, port)), DISABLED)
end

function on_action_change(port, old, new)
  if not cfg.on or port ~= focus then return end
  log(string.format("%s %s > %s", pname(port), gd.motion_name(old, port), gd.motion_name(new, port)), DISABLED)
end

function on_match_start()
  history_on = false
  log_lines = {}
  focus = 1
  tl_cache, scrub = {}, {}
  menu.open, menu.reset_saved, menu.prev = false, false, {}
  LE.reset()
  LD.reset()
  cfg.on = cfg.always or gd.lab_request()
  if gd.lab_request() then lab_matched = true end
end

function on_scene(kind, name)
  if lab_matched and (name == "GS_MENU" or name == "GS_TITLE") then
    gd.lab_request(true)
    lab_matched = false
  end
  if mirror and offline() then mirror = false gd.mirror_pad() end
end

function on_frame()
  LE.frame()
  LD.frame()
  for port, s0 in pairs(scrub) do
    local p = gd.player(port)
    if p == nil or p.action ~= s0.motion then scrub[port] = nil end
  end
end

function on_loadstate(slot)
  LD.cut()
  local now = gd.match().frame
  for i = #log_lines, 1, -1 do
    if log_lines[i][1] > now then table.remove(log_lines, i) end
  end
  log(slot == 0 and "-- stepped back --" or slot == 9 and "-- loaded from the library --"
    or ("-- loaded state " .. slot .. " --"), GOLD)
  if slot == 9 then lib_refresh(true) end
end

function on_hot_reload(ok)
  local st = gd.hot_reload_status()
  LD.cut()
  say((ok and "Reloaded: " or "Reload: ") .. st.text, ok and OK or DANGER)
  log("-- hot reload --", GOLD)
end

-- ---- drawing -------------------------------------------------------------------------------------
local warned_kit = false
function on_draw()
  if not kit_ok() then
    if not warned_kit then gd.log("Geno Lab: the kit is not available, the Lab UI is not drawn") warned_kit = true end
    return
  end
  if menu.open then lab_menu_draw() return end
  LE.draw_batch()
  if not cfg.on or cfg.hidden then return end
  if not gd.match().active then return end
  local list = gd.players()
  local id = mode().id
  for _, p in ipairs(list) do
    if id == "inspect" and (T("skel") or T("joints")) then draw_skeleton(p, PORT[p.port]) end
    if id == "hitboxes" and T("ecb") then draw_ecb(p) end
    if id == "hitboxes" and T("labels") then draw_hit_labels(p) end
  end
  if id == "hitboxes" and T("data") then draw_hit_data() end
  if id == "frames" then draw_frames(list) end
  if id == "moves" then LE.draw_moves() end
  if id == "launch" then LE.draw_launch() end
  if id == "ab" then LE.draw_ab() end
  if id == "training" then LD.draw() end
  if id == "combo" then
    if T("boxes") then LD.draw_d2({ swept = true, shield = true, grab = true }) end
    LD.draw_combo(T("fan"))
  end
  if id == "hitboxes" then LD.draw_d2({ swept = T("swept"), hurt = T("hurt"), shield = T("shield"), grab = T("grab") }) end
  LD.draw_drill()
  if id == "inspect" then
    if T("info") then draw_info(list) end
    if T("attrs") then draw_attrs(list) end
    if T("log") then draw_log() end
  end
  draw_strip()
  draw_notice()
  if cfg.help then draw_help() end
end

function on_unload()
  restore_draw()
end

-- ---- console ----------------------------------------------------------------------------------------
local function dump(port)
  local p = gd.player(port)
  if p == nil then gd.log("no fighter on port " .. port) return end
  for _, l in ipairs(info_lines(p)) do gd.log(l[1]) end
  gd.log(string.format("anim_symbol %s  joints %d  hurtboxes %d  draw_flags 0x%02X", p.anim_symbol,
    p.joint_count, p.hurtbox_count, p.draw_flags))
end

local function help_lines()
  local out = { "Geno Lab - mode " .. mode().name .. " (TAB / 1-9, 0 change it)" }
  for i, m in ipairs(MODES) do
    local ks = {}
    for _, t in ipairs(m.t) do ks[#ks + 1] = t.k .. " " .. t.label end
    for _, a in ipairs(m.a) do ks[#ks + 1] = a.k .. " " .. a.label end
    out[#out + 1] = string.format("  %d %-9s %s", i, m.name, #ks > 0 and table.concat(ks, ", ") or "(no keys)")
  end
  local g = {}
  for _, k in ipairs(GLOBAL_KEYS) do g[#g + 1] = k[1] .. " " .. k[2] end
  out[#out + 1] = "  global: " .. table.concat(g, ", ")
  out[#out + 1] = "  console: lab help | status | mode <name> | set <mode>.<toggle> on|off | hide | menu [tab]"
  out[#out + 1] = "           port N | history [seconds] | back N | dump [N] | move <id> [frame] | events"
  out[#out + 1] = "           states | save | load <file> | rename <file> <name> | delete <file> | reload [seconds]"
  out[#out + 1] = "  stage E: moves [text] | play <id|name> | kb | di none|in|out|survival | ab [record|reload|same|mirror]"
  out[#out + 1] = "           export [port] [version] | fdiff [fighter verA verB] | rollbacks [n]"
  out[#out + 1] = "  stage D: adv | card | tech [clear] | actionable | dummy [key value] | combo | drill [id | stop]"
  return out
end

gd.command("lab", function(arg)
  local cmd, rest = arg:match("^(%S*)%s*(.-)$")
  local n = tonumber(rest)
  if cmd == "" then
    cfg.on = not cfg.on
    if not cfg.on then restore_draw() end
    gd.log("Geno Lab " .. (cfg.on and "on" or "off"))
  elseif cmd == "help" then
    for _, l in ipairs(help_lines()) do gd.log(l) end
  elseif cmd == "status" then
    local ts = {}
    for _, t in ipairs(mode().t) do ts[#ts + 1] = t.id .. "=" .. onoff(T(t.id)) end
    gd.log(string.format("lab: on=%s mode=%s hidden=%s help=%s focus=%d toggles{%s} draw=0x%02X stage=0x%02X",
      tostring(cfg.on), mode().name, tostring(cfg.hidden), tostring(cfg.help), focus, table.concat(ts, " "),
      wanted_flags(), wanted_stage()))
    gd.log(string.format("lab: menu open=%s tab=%s sel=%d rows=%d hits=%d kit=%s lab_mode=%s",
      tostring(menu.open), TABS[menu.tab].name, cur_sel(), #tab_items(), #menu.hits, tostring(kit_ok()),
      tostring(gd.lab_mode and gd.lab_mode())))
  elseif cmd == "mode" then
    local want = rest:lower()
    local i = MODE_BY_ID[want] or tonumber(want)
    if i and MODES[i] then set_mode(i) gd.log("mode " .. mode().name)
    else gd.log("lab mode clean|hitboxes|frames|stage|inspect|moves|launch|ab|training|combo") end
  elseif cmd == "hide" then
    cfg.hidden = not cfg.hidden
    if cfg.hidden then restore_draw() end
    save_settings()
    gd.log("Lab UI " .. (cfg.hidden and "hidden" or "shown"))
  elseif cmd == "menu" then
    if not in_lab_match() then gd.log("the LAB menu is for LAB matches") return end
    if rest == "close" then if menu.open then menu_close() end gd.log("menu closed") return end
    if not menu.open then menu_open(1) end
    for i, t in ipairs(TABS) do if t.name:lower() == rest:lower() then set_tab(i) end end
    gd.log("menu open, tab " .. TABS[menu.tab].name)
  elseif cmd == "port" and n then
    focus = n
  elseif cmd == "history" and n then
    cfg.history_s = n
    history_on = false
    ensure_history()
    local h = gd.history()
    gd.log(string.format("history: %d s (%d frames), a keyframe every %d, %.1f MB now", cfg.history_s, h.depth,
      h.interval, h.mb))
    save_settings()
  elseif cmd == "history" then
    local h = gd.history()
    gd.log(string.format("history: %d s, back %d fwd %d, %d keyframes, %.1f MB (%.1f MB deltas), key %.2f ms, " ..
      "last rewind %d frames in %.2f ms (load %.2f ms)%s", cfg.history_s, h.back, h.fwd, h.keys, h.mb, h.delta_mb,
      h.key_ms, h.last_frames, h.last_ms, h.last_load_ms, h.replaying and ", replaying" or ""))
  elseif cmd == "states" then
    lib_refresh(true)
    for _, r in ipairs(lib.rows) do
      gd.log(string.format("  %s  %s  f%d  %s  %s", r.file, r.name, r.frame, r.saved, r.ok and "ok" or ("REFUSED: " .. r.why)))
    end
    gd.log(#lib.rows .. " saved state(s)")
  elseif cmd == "save" then
    lib_save()
  elseif cmd == "load" and rest ~= "" then
    local ok, why = gd.state_load(rest)
    gd.log(ok and ("loading " .. rest) or ("refused: " .. tostring(why)))
  elseif cmd == "rename" then
    local f, nm = rest:match("^(%S+)%s+(.+)$")
    if f then local ok, why = gd.state_rename(f, nm) gd.log(ok and "renamed" or tostring(why)) lib_refresh(true)
    else gd.log("lab rename <file> <name>") end
  elseif cmd == "delete" and rest ~= "" then
    local ok, why = gd.state_delete(rest)
    gd.log(ok and "deleted" or tostring(why))
    lib_refresh(true)
  elseif cmd == "reload" then
    if n then cfg.reload_s = n end
    hot_reload()
  elseif cmd == "back" then
    back(n or 1)
  elseif cmd == "dump" then
    dump(n or focus)
  elseif cmd == "move" then
    local id, fr = rest:match("^(%d+)%s*(%d*)$")
    if id then
      local ok, why = gd.set_motion(focus, tonumber(id), tonumber(fr) or 1, 1, lift_for(focus, tonumber(id)))
      if ok then scrub[focus] = { motion = tonumber(id), frame = tonumber(fr) or 1 } end
      gd.log(ok and ("P" .. focus .. " -> " .. gd.motion_name(tonumber(id), focus)) or tostring(why))
    else
      gd.log("lab move <motion id> [frame]")
    end
  elseif cmd == "events" then
    local p = gd.player(n or focus)
    local tl = p and gd.timeline(p.port)
    if tl then
      gd.log(string.format("%s (%s) %d events, length %.0f, %s", tl.motion_name, tl.anim_name,
        #tl.events, tl.length, tl.stop or "-"))
      for _, e in ipairs(tl.events) do
        local extra = e.name == "hitbox" and string.format(" #%d b%d %d%% a%d g%d b%d w%d r%.2f %s",
          e.id, e.bone, e.damage, e.angle, e.kbg, e.bkb, e.wbk, e.size, e.element_name) or
          (e.value and (" " .. e.value) or "")
        gd.log(string.format("  f%-3d %s%s", e.frame, e.name, extra))
      end
    end
  elseif LE.console(cmd, rest) then
    -- stage E (LE.console)
  elseif LD.console(cmd, rest) then
    -- stage D (LD.console)
  elseif cmd == "dummy" then
    LD.dummy_console(rest)
  elseif cmd == "combo" then
    LD.combo_console()
  elseif cmd == "drill" then
    LD.drill_console(rest)
  elseif cmd == "set" then
    local mid, tid, v = rest:match("^(%w+)%.(%w+)%s+(%S+)$")
    if mid and tog[mid] and tog[mid][tid] ~= nil then
      tog[mid][tid] = v == "true" or v == "on"
      save_settings()
      gd.log(mid .. "." .. tid .. " = " .. tostring(tog[mid][tid]))
    else
      gd.log("lab set <mode>.<toggle> on|off  (e.g. hitboxes.ecb on; lab help lists them)")
    end
  else
    gd.log("lab [help | status | mode <m> | set <m>.<t> on|off | hide | menu [tab|close] | port N | history [s] | back [N] | dump [N] | states | save | load | rename | delete | reload [s]]")
  end
end, "Geno Lab: help, status, mode, set, hide, menu, port, history, back, dump, states, save, load, reload")
