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
--   TAB    next mode (SHIFT: previous)  1-5  a mode directly    F  focus the next fighter
--   H      hide / show the Lab UI  F3  help (this mode's keys)  ESC  the LAB pause menu
-- Mode keys:
--   CLEAN     (none)                      just the game and a tiny mode chip
--   HITBOXES  B boxes  L labels  E ECB  D hitbox data
--   FRAMES    T timeline  B boxes  Q / E scrub -1 / +1  HOME replay  C lock-step  R mirror pad
--   STAGE     C collision  L ledges  T terrain  P points  Z zones
--   INSPECT   M model  S skeleton  J joint numbers  I info  A attributes  L event log
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
    }, a = {} },
  { id = "frames", name = "FRAMES", icon = "lab_timeline",
    blurb = "The move as a timeline: windows, IASA, effects. Scrub it, replay it, line two up.",
    t = {
      { k = "T", id = "timeline", label = "Move timeline", icon = "lab_timeline", def = true,
        desc = "The subaction script as a track: hit windows, IASA, GFX, SFX, body state." },
      { k = "B", id = "boxes", label = "Hit / hurtboxes", icon = "lab_hitbox", def = true,
        desc = "Keep the boxes on while you scrub." },
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
}
local MODE_BY_ID = {}
for i, m in ipairs(MODES) do MODE_BY_ID[m.id] = i end

local GLOBAL_KEYS = {
  { "SPACE", "Pause / resume" }, { "RIGHT", "Step +1 (hold, CTRL x10)" },
  { "LEFT", "Step -1 (hold, CTRL x10)" }, { "F5", "Save state 1" }, { "F6", "Load state 1" },
  { "TAB", "Next mode (SHIFT back)" }, { "1-5", "Mode directly" }, { "F", "Focus next fighter" },
  { "H", "Hide / show the Lab UI" }, { "F3", "This help" }, { "ESC", "Pause menu" },
}

-- ---- settings (scripts-data/<mod>/settings.txt) ------------------------------------------------
local cfg = { on = true, always = false, history = 20, mode = 2, hidden = false, help = false }
local tog = {} -- tog[mode_id][toggle_id] = bool
for _, m in ipairs(MODES) do
  tog[m.id] = {}
  for _, t in ipairs(m.t) do tog[m.id][t.id] = t.def end
end
local SETTINGS = "settings.txt"

local function save_settings()
  local out = { "mode=" .. MODES[cfg.mode].id, "hidden=" .. tostring(cfg.hidden),
    "history=" .. cfg.history, "always=" .. tostring(cfg.always) }
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
      elseif k == "history" and tonumber(v) then cfg.history = tonumber(v) end
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
  if (in_mode("hitboxes") or in_mode("frames")) and T("boxes") then f = f | gd.draw.HIT | gd.draw.THROWN end
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
  local h = gd.history(cfg.history)
  history_on = true
  if h.depth < cfg.history then say(string.format("History: %d frames (memory)", h.depth), DANGER) end
end

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
local HISTORY_STEPS = { 10, 20, 40, 60 }

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

local TABS = {
  { name = "PLAY", icon = "lab_play", items = {
    { label = "Resume", icon = "lab_play", desc = "Unfreeze. Everything carries on from this exact frame.",
      run = function() menu_close() end },
    { label = "Step +1", icon = "lab_step", key = "RIGHT",
      desc = "One frame forward, then frozen again. The heart of frame study.",
      value = function() return "f " .. gd.match().frame end, run = function() gd.step(1) end },
    { label = "Step -1", icon = "lab_step_back", key = "LEFT",
      desc = "One frame back, out of the history ring. Undo, but for physics.",
      value = function() local h = gd.history() return string.format("%d / %d", h.back, h.depth) end,
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
  { name = "STATES", icon = "lab_save", items = {
    { label = "Save state", icon = "lab_save", key = "F5",
      desc = "Snapshot everything into a slot. Left / right picks the slot.",
      value = function() return "SLOT " .. menu.slot end,
      run = function() gd.savestate(menu.slot) say("State " .. menu.slot .. " saved") end,
      adjust = function(d) menu.slot = ((menu.slot - 1 + d) % 3) + 1 end },
    { label = "Load state", icon = "lab_load", key = "F6",
      desc = "Back to a snapshot, frame-exact.",
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
      desc = "How far step-back can rewind. Every frame kept costs about 27 MB.",
      value = function() return cfg.history .. " FRAMES" end,
      adjust = function(d)
        local k = 1
        for i, v in ipairs(HISTORY_STEPS) do if v == cfg.history then k = i end end
        cfg.history = HISTORY_STEPS[((k - 1 + d) % #HISTORY_STEPS) + 1]
        history_on = false
        ensure_history()
        save_settings()
      end },
  } },
  { name = "EXIT", icon = "lab_exit", items = {
    { label = "Change fighters", icon = "lab_focus", desc = "Back to LAB's character select.",
      run = function() menu_leave("css") end },
    { label = "Change stage", icon = "lab_stage", desc = "Same fighters, a different floor.",
      run = function() menu_leave("sss") end },
    { label = "Quit", icon = "lab_power", desc = "Leave the Lab for the menus. No contest, no results.",
      value = function() return "NO CONTEST" end, run = function() menu_leave("menu") end },
  } },
}

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

local PAD_EDGES = { "A", "B", "START", "UP", "DOWN", "LEFT", "RIGHT", "L", "R" }
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
  if e.A or gd.key_pressed("ENTER") or gd.key_pressed("SPACE") then
    menu_activate()
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
  for i, t in ipairs(TABS) do
    local tw = math.floor(measure(t.name, "label")) + 30
    local on = i == menu.tab
    quad(tx, ty, tw, 28, on and GOLD or alpha(GLASS_SOLID, 0xF0), SHEAR)
    if on then quad(tx + 2, ty + 28, tw - 4, 3, GOLD_DK, SHEAR) end
    stxt(tx + tw / 2, ty + 22, t.name, "label", on and INK or BONE, "center")
    menu.hits[#menu.hits + 1] = { kind = "tab", i = i, x = tx, y = ty, w = tw, h = 28 }
    tx = tx + tw + 8
  end
  img("glyph_r", tx + 2, ty + 6, 16, 16, MUTED)
  -- the frame ruler under the tabs
  for rx = 0, 639, 128 do img("lab_ruler", rx, 106, 128, 8, alpha(TICK, 0x90)) end

  -- rows: big, sheared; the selected one gold, pushed right, with a chevron
  local rx0, ry0, rw, rh, pitch = 44, 126, 276, 30, 36
  local wipe = menu.tab_t < 6
  for i, it in ipairs(items) do
    local ry = ry0 + (i - 1) * pitch
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
      or "A  do it"
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
  local status = string.format("f %d%s", gd.match().frame, h.back > 0 and ("  -" .. h.back) or "")
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
  local extra = string.format("history %d / %d", h.back, h.depth)
  local w = measure(s, "body") + measure(extra, "caption") + 44
  quad(8, 8, w, 22, GLASS, SHEAR)
  quad(6, 8, 4, 22, GOLD, SHEAR)
  txt(18, 24, s, "body", BONE)
  txt(8 + w - 10, 23, extra, "caption", MUTED, "right")
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
  if gd.key_pressed("H") then
    cfg.hidden = not cfg.hidden
    if cfg.hidden then restore_draw() end
    save_settings()
  end
  if gd.key_pressed("F3") then cfg.help = not cfg.help end
  if gd.key_pressed("TAB") then set_mode(cfg.mode + (gd.key("SHIFT") and -1 or 1)) end
  for i = 1, #MODES do if gd.key_pressed(tostring(i)) then set_mode(i) end end
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
  for port, s0 in pairs(scrub) do
    local p = gd.player(port)
    if p == nil or p.action ~= s0.motion then scrub[port] = nil end
  end
end

function on_loadstate(slot)
  local now = gd.match().frame
  for i = #log_lines, 1, -1 do
    if log_lines[i][1] > now then table.remove(log_lines, i) end
  end
  log(slot == 0 and "-- stepped back --" or ("-- loaded state " .. slot .. " --"), GOLD)
end

-- ---- drawing -------------------------------------------------------------------------------------
local warned_kit = false
function on_draw()
  if not kit_ok() then
    if not warned_kit then gd.log("Geno Lab: the kit is not available, the Lab UI is not drawn") warned_kit = true end
    return
  end
  if menu.open then lab_menu_draw() return end
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
  local out = { "Geno Lab - mode " .. mode().name .. " (TAB / 1-5 change it)" }
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
  out[#out + 1] = "           port N | history N | back N | dump [N] | move <id> [frame] | events"
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
    else gd.log("lab mode clean|hitboxes|frames|stage|inspect") end
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
    cfg.history = n
    history_on = false
    ensure_history()
    local h = gd.history()
    gd.log(string.format("history: %d frames (%.0f MB each)", h.depth, h.slot_mb))
    save_settings()
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
    gd.log("lab [help | status | mode <m> | set <m>.<t> on|off | hide | menu [tab|close] | port N | history N | back [N] | dump [N]]")
  end
end, "Geno Lab: help, status, mode, set, hide, menu, port, history, back, dump")
