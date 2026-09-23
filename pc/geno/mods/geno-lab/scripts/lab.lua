-- Geno Lab - frame-steppable fighter inspection (private, Geno build). docs/geno.md "Geno Lab".
--
-- Offline only: pause, step and step-back need "gameplay": true, and every write the Lab makes
-- (debug drawing, history) is refused during a netplay/rollback session, where it only reads.
--
-- Keys (game window focused, console closed; none of them is a keyboard play key):
--   F3        help                      X         Lab on / off
--   P         pause / resume            TAB       focus the next fighter
--   N         step 1 frame (hold: play slowly; CTRL: 10 frames)
--   B         step BACK 1 frame (hold: rewind slowly; CTRL: 10 frames)
--   1 hit/hurtboxes   2 model   3 skeleton   4 joint numbers   5 ECB   6 stage collision
--   7 info panel      8 event log   9 hitbox labels   0 attributes
--   F5 / F6   save / load state 1
--   M         move timeline (the subaction script: hitboxes, IASA, GFX/SFX, body state...)
--   PAGEUP / PAGEDOWN   scrub the focused fighter's move one frame back / forward (replays it)
--   HOME      replay the move from frame 1        C   both fighters into this move, lock-step
--   R         mirror P1's controller onto P2 (P2 must be a human port)
-- The Lab runs in matches started from SOLO > LAB (or MELEE_LAB=1), or after X.
-- Console: "lab help".

if gd.lab_api == nil then
  gd.log("Geno Lab needs the Geno build (gd.lab_api missing) - not started")
  return
end

local cfg = {
  on = true, hit = true, model = true, skel = false, joints = false, ecb = false, stage = 0,
  info = true, log = true, labels = true, attrs = false, help = false, history = 20,
  timeline = true, always = false,
}
local NOT_SAVED = { help = true, on = true }
local SETTINGS = "settings.txt"
local focus = 1            -- the focused port (1-6)
local log_lines = {}       -- {frame, text, color}
local LOG_MAX = 12
local held = {}            -- key -> ticks held (auto-repeat)
local history_on = false   -- gd.history set for this match
local notice, notice_until = nil, 0
local attr_names = nil

local PORT_COLORS = { 0xFF5C5CFF, 0x5C9CFFFF, 0xFFD24DFF, 0x5CE07AFF, 0xC77DFFFF, 0xFFFFFFFF }
local HIT_COLORS = { [0] = 0xFF3030FF, 0xFF9020FF, 0xFFE020FF, 0xFF40C0FF, 0xFFFFFFFF }
local WHITE, GREY, DIM, YELLOW, GREEN, RED = 0xFFFFFFFF, 0xC8C8D0FF, 0x9090A0FF, 0xFFD166FF, 0x7BE495FF, 0xFF6B6BFF
local STAGE_MODES = { 0, gd.stage_draw.COLL, gd.stage_draw.COLL | gd.stage_draw.LEDGES,
                      gd.stage_draw.COLL | gd.stage_draw.TERRAIN }
local STAGE_NAMES = { [0] = "off", "lines+ECB", "ledges", "terrain" }

-- ---- settings -------------------------------------------------------------------------------
local function save_settings()
  local out = {}
  for k, v in pairs(cfg) do
    if not NOT_SAVED[k] then out[#out + 1] = k .. "=" .. tostring(v) end
  end
  table.sort(out)
  pcall(gd.data_write, SETTINGS, table.concat(out, "\n") .. "\n")
end

local function load_settings()
cfg.on = cfg.always or gd.lab_request()
  local ok, text = pcall(gd.data_read, SETTINGS)
  if not ok or text == nil then return end
  for k, v in text:gmatch("([%w_]+)=([^\r\n]+)") do
    if cfg[k] ~= nil then
      if v == "true" then cfg[k] = true
      elseif v == "false" then cfg[k] = false
      elseif tonumber(v) then cfg[k] = tonumber(v) end
    end
  end
end
load_settings()

local function say(text, color)
  notice, notice_until = { text, color or YELLOW }, gd.time() + 2.5
end

local function offline()
  return not gd.match().netplay
end

local function log(text, color)
  table.insert(log_lines, { gd.match().frame, text, color or GREY })
  while #log_lines > LOG_MAX do table.remove(log_lines, 1) end
end

-- ---- the game's own debug drawing (Fighter.x21FC_flag, the camera's collision display) -------
local function wanted_flags()
  if not cfg.on then return gd.draw.DEFAULT end
  local f = 0
  if cfg.model then f = f | gd.draw.MODEL end
  if cfg.hit then f = f | gd.draw.HIT | gd.draw.THROWN end
  return f
end

local function apply_draw()
  if not offline() then return end
  local want = wanted_flags()
  for _, p in ipairs(gd.players()) do
    if gd.debug_draw(p.port) ~= want then gd.debug_draw(p.port, want) end
  end
  local stage = cfg.on and STAGE_MODES[(cfg.stage % #STAGE_MODES) + 1] or 0
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
  if h.depth < cfg.history then say(string.format("history: %d frames (memory)", h.depth), RED) end
end

-- ---- stepping ---------------------------------------------------------------------------------
local function step(n)
  if not offline() then return end
  gd.step(n)
end

local function back(n)
  if not offline() then return end
  ensure_history()
  local ok, why = gd.step_back(n)
  if not ok then say("step back: " .. tostring(why), RED) end
end

-- pressed now, or held long enough to repeat (every 3rd tick after 18)
local function repeat_key(name)
  if gd.key(name) then
    held[name] = (held[name] or 0) + 1
  else
    held[name] = 0
    return false
  end
  local t = held[name]
  return t == 1 or (t > 18 and t % 3 == 0)
end

local function next_port()
  local ports = {}
  for _, p in ipairs(gd.players()) do ports[#ports + 1] = p.port end
  if #ports == 0 then return end
  for i, port in ipairs(ports) do
    if port == focus then focus = ports[(i % #ports) + 1] return end
  end
  focus = ports[1]
end

local function toggle(key, name)
  cfg[key] = not cfg[key]
  say(name .. (cfg[key] and " on" or " off"))
  save_settings()
end

-- ---- Stage 2: timelines, scrubbing, lock-step compare, mirrored input --------------------------
local tl_cache = {}      -- port -> {key, tl, windows, marks, len}
local scrub = {}         -- port -> {motion, frame}
local mirror = false
local lab_matched = false   -- a match ran with the LAB request since it was made

-- aerials (and other air states) need the fighter in the air, or it lands at once
local AIR_MOTIONS = { AttackAirN = true, AttackAirF = true, AttackAirB = true, AttackAirHi = true,
  AttackAirLw = true, EscapeAir = true, Fall = true, FallAerial = true }
local function lift_for(port, motion)
  local name = gd.motion_name(motion, port)
  if AIR_MOTIONS[name] or name:find("Air") and not name:find("Landing") then return 40 end
  return 0
end

-- hitbox windows etc. from a timeline's events (frames are 1-based, like frame-data sites)
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

-- replay a fighter's move up to `frame` (set_motion runs it from frame 1, then pauses)
local function scrub_to(port, frame)
  local p = gd.player(port)
  if p == nil or not offline() then return end
  local s0 = scrub[port]
  local motion = (s0 and s0.motion) or p.action
  if frame < 1 then frame = 1 end
  local ok, why = gd.set_motion(port, motion, frame, 1, lift_for(port, motion))
  if ok then
    scrub[port] = { motion = motion, frame = frame }
    say(string.format("%s frame %d", gd.motion_name(motion, port), frame))
  else
    say("scrub: " .. tostring(why), RED)
  end
end

-- every fighter into the focused fighter's move at the same frame, together (lock-step)
local function compare()
  local a = gd.player(focus)
  if a == nil or not offline() then return end
  local motion = (scrub[focus] and scrub[focus].motion) or a.action
  local frame = (scrub[focus] and scrub[focus].frame) or 1
  local n = 0
  for _, p in ipairs(gd.players()) do
    local ok = gd.set_motion(p.port, motion, frame, 1, lift_for(p.port, motion))
    if ok then scrub[p.port] = { motion = motion, frame = frame } n = n + 1 end
  end
  say(string.format("lock-step: %d fighters in %s, frame %d", n, gd.motion_name(motion, focus), frame))
end

function on_tick()
  if gd.key_pressed("X") then
    cfg.on = not cfg.on
    if not cfg.on then restore_draw() end
    say(cfg.on and "Geno Lab on" or "Geno Lab off")
    save_settings()
  end
  if not cfg.on then return end
  if gd.key_pressed("F3") then cfg.help = not cfg.help end
  if gd.key_pressed("TAB") then next_port() end
  if gd.key_pressed("1") then toggle("hit", "hit/hurtboxes") end
  if gd.key_pressed("2") then toggle("model", "model") end
  if gd.key_pressed("3") then toggle("skel", "skeleton") end
  if gd.key_pressed("4") then toggle("joints", "joint numbers") end
  if gd.key_pressed("5") then toggle("ecb", "ECB") end
  if gd.key_pressed("6") then
    cfg.stage = (cfg.stage + 1) % #STAGE_MODES
    say("stage collision: " .. STAGE_NAMES[cfg.stage])
    save_settings()
  end
  if gd.key_pressed("7") then toggle("info", "info panel") end
  if gd.key_pressed("8") then toggle("log", "event log") end
  if gd.key_pressed("9") then toggle("labels", "hitbox labels") end
  if gd.key_pressed("0") then toggle("attrs", "attributes") end
  if offline() then
    if gd.key_pressed("P") then
      if gd.paused() then gd.resume() else ensure_history() gd.pause() end
    end
    local big = gd.key("CTRL") and 10 or 1
    if repeat_key("N") then step(big) end
    if repeat_key("B") then back(big) end
    if gd.key_pressed("M") then toggle("timeline", "move timeline") end
    if repeat_key("PAGEDOWN") then
      local s0 = scrub[focus]
      scrub_to(focus, s0 and s0.frame + 1 or 1)
    end
    if repeat_key("PAGEUP") then
      local s0 = scrub[focus]
      scrub_to(focus, s0 and s0.frame - 1 or 1)
    end
    if gd.key_pressed("HOME") then scrub[focus] = nil scrub_to(focus, 1) end
    if gd.key_pressed("C") then compare() end
    if gd.key_pressed("R") then
      mirror = not mirror
      if mirror then gd.mirror_pad(1, 2) else gd.mirror_pad() end
      say(mirror and "P2 mirrors P1's controller (P2 must be a human port)" or "mirror off")
    end
    if gd.key_pressed("F5") then gd.savestate(1) end
    if gd.key_pressed("F6") then
      local ok, err = pcall(gd.loadstate, 1)
      if not ok then say((tostring(err):gsub("^.-: ", "")), RED) end
    end
  end
  if gd.match().active then
    ensure_history()
    apply_draw()
    if gd.player(focus) == nil then next_port() end
  end
end

-- ---- engine events ----------------------------------------------------------------------------
local function pname(port)
  return port and ("P" .. port) or "item"
end

function on_hit(attacker, victim, info)
  if not cfg.on then return end
  local text
  if info.angle then
    text = string.format("%s hit %s  #%s  %.1f%%  ang %d  kbg %d  bkb %d  wbk %d  %s",
      pname(attacker), pname(victim), tostring(info.hitbox), info.dealt, info.angle, info.kbg,
      info.bkb, info.wbk, info.element_name)
  else
    text = string.format("%s hit %s  %.1f%%%s", pname(attacker), pname(victim), info.dealt,
      info.item and "  (item)" or "")
  end
  log(text, PORT_COLORS[attacker or 6])
end

function on_hitlag(port, entering)
  if not cfg.on then return end
  local p = gd.player(port)
  log(string.format("%s hitlag %s%s", pname(port), entering and "on" or "off",
    (entering and p) and string.format(" (%.0f f)", p.hitlag) or ""), DIM)
end

function on_land(port, motion)
  if not cfg.on then return end
  log(string.format("%s land from %s", pname(port), gd.motion_name(motion, port)), DIM)
end

function on_action_change(port, old, new)
  if not cfg.on or port ~= focus then return end
  log(string.format("%s %s -> %s", pname(port), gd.motion_name(old, port), gd.motion_name(new, port)), DIM)
end

function on_match_start()
  history_on = false
  log_lines = {}
  focus = 1
  tl_cache, scrub = {}, {}
  cfg.on = cfg.always or gd.lab_request()
  if gd.lab_request() then lab_matched = true end
end

function on_scene(kind, name)
  -- back at the menus: a later plain TRAINING does not bring the Lab
  if lab_matched and (name == "GS_MENU" or name == "GS_TITLE") then
    gd.lab_request(true)
    lab_matched = false
  end
  if mirror and offline() then mirror = false gd.mirror_pad() end
end

-- the scrub position belongs to the move: when the fighter leaves it, forget it
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
  log(slot == 0 and "-- stepped back --" or ("-- loaded state " .. slot .. " --"), YELLOW)
end

-- ---- drawing ----------------------------------------------------------------------------------
local function draw_skeleton(p, color)
  local js = gd.joints(p.port)
  if js == nil then return end
  for _, j in ipairs(js) do
    if j.on then
      local par = j.parent >= 0 and js[j.parent + 1] or nil
      if cfg.skel and par and par.on then gd.line(par.sx, par.sy, j.sx, j.sy, color) end
      if cfg.skel then gd.fill(j.sx - 1, j.sy - 1, 3, 3, 0xFFFFFFE0) end
      if cfg.joints then gd.text(j.sx + 2, j.sy - 6, j.index, 0xFFFFA0FF, 0.55) end
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
    gd.line(a[1], a[2], b[1], b[2], 0xFF8C00FF)
  end
end

local function draw_hit_labels(p)
  for _, h in ipairs(p.hitboxes) do
    local sx, sy, vis = gd.project(h.x, h.y, h.z)
    if sx and vis then
      gd.text(sx + 4, sy - 4, string.format("#%d %.0f%% %d", h.id, h.damage, h.angle),
        HIT_COLORS[h.id] or WHITE, 0.6)
    end
  end
end

local function f1(v) return string.format("%.2f", v) end

-- one fighter's column of the info panel
local function info_lines(p)
  local t = {}
  local function add(s, c) t[#t + 1] = { s, c or GREY } end
  add(string.format("P%d %s  [kind %d%s]", p.port, p.char_name, p.kind, p.cpu and " cpu" or ""),
    PORT_COLORS[p.port])
  add(string.format("action %d %s  f%d", p.action, p.motion_name, p.action_frame + 1), WHITE)
  add(string.format("anim %s  %.2f  x%.2f", p.anim_name ~= "" and p.anim_name or "-", p.anim_frame_f,
    p.anim_rate))
  add(string.format("pos %s %s  %s", f1(p.x), f1(p.y), p.airborne and "air" or "ground"))
  add(string.format("vel self %s %s  kb %s %s", f1(p.vx), f1(p.vy), f1(p.kb_vx), f1(p.kb_vy)))
  add(string.format("jumps %d/%d  walljumps %d  gr vel %s", p.jumps_left, p.jumps_max,
    p.walljumps_used, f1(p.ground_vel)))
  add(string.format("hitlag %.0f  hitstun %.0f  kb %.1f", p.hitlag, p.hitstun, p.kb_applied),
    (p.in_hitlag or p.in_hitstun) and YELLOW or GREY)
  add(string.format("intang %d  invinc %d  body %s", p.intangible, p.invincible, p.body_state),
    (p.intangible > 0 or p.invincible > 0 or p.body_state ~= "normal") and GREEN or GREY)
  add(string.format("shield %.1f  %s  ledge cd %d", p.shield, p.iasa and "IASA" or "no IASA",
    p.ledge_cooldown), p.iasa and GREEN or GREY)
  add(string.format("ECB bottom %s  lock %d", f1(p.ecb.bottom.y), p.ecb_lock))
  if #p.hitboxes == 0 then
    add("hitboxes: none", DIM)
  else
    for _, h in ipairs(p.hitboxes) do
      add(string.format("#%d b%d %.1f%% a%d kbg%d bkb%d wbk%d r%.2f %s", h.id, h.bone, h.damage,
        h.angle, h.kbg, h.bkb, h.wbk, h.radius, h.element_name), HIT_COLORS[h.id] or WHITE)
    end
  end
  return t
end

local function draw_info(list)
  -- the focused fighter first, then the next one: side by side for comparisons
  local cols = {}
  for _, p in ipairs(list) do if p.port == focus then cols[1] = p end end
  for _, p in ipairs(list) do if p.port ~= focus and #cols < 2 then cols[#cols + 1] = p end end
  local x, w = 4, 312
  local h = 0
  local blocks = {}
  for i, p in ipairs(cols) do
    blocks[i] = info_lines(p)
    if #blocks[i] > h then h = #blocks[i] end
  end
  gd.fill(x, 22, w * #cols + 4, 10 * h + 8, 0x000000B0)
  for i, lines in ipairs(blocks) do
    for k, l in ipairs(lines) do
      gd.text(x + 4 + (i - 1) * w, 24 + (k - 1) * 10, l[1], l[2], 0.72)
    end
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
  for _, p in ipairs(list) do if #cols < 3 then cols[#cols + 1] = { p, gd.attrs(p.port) } end end
  local x, y = 330, 22
  gd.fill(x - 4, y - 2, 310, #attr_names * 9 + 16, 0x000000C0)
  for i, c in ipairs(cols) do
    gd.text(x + 150 + (i - 1) * 52, y, "P" .. c[1].port, PORT_COLORS[c[1].port], 0.65)
  end
  for k, name in ipairs(attr_names) do
    local yy = y + 10 + (k - 1) * 9
    local v1 = cols[1] and cols[1][2][name]
    local differs = false
    for _, c in ipairs(cols) do if c[2][name] ~= v1 then differs = true end end
    gd.text(x, yy, name, differs and YELLOW or DIM, 0.6)
    for i, c in ipairs(cols) do
      gd.text(x + 150 + (i - 1) * 52, yy, string.format("%.4g", c[2][name]), differs and WHITE or GREY, 0.6)
    end
  end
end

local function draw_timeline(p, y, color)
  local c = timeline_of(p)
  if c == nil then return y end
  local x0, w = 8, 624
  local len = math.max(c.len, 1)
  local sx = w / len
  local now = p.anim_frame_f + 1
  gd.fill(x0 - 4, y - 2, w + 8, 38, 0x000000B8)
  gd.text(x0, y, string.format("P%d %s  (%s)  %d frames  script %s", p.port, c.tl.motion_name,
    c.tl.anim_name, math.floor(len + 0.5), c.tl.stop or "-"), color, 0.68)
  local by = y + 11
  gd.fill(x0, by, w, 8, 0x303040FF)
  for f = 5, len, 5 do gd.line(x0 + (f - 1) * sx, by + 8, x0 + (f - 1) * sx, by + 10, DIM) end
  for _, hw in ipairs(c.windows) do
    local col = HIT_COLORS[hw.id] or WHITE
    gd.fill(x0 + (hw.from - 1) * sx, by + 1, math.max(2, (hw.to - hw.from + 1) * sx), 6, col)
  end
  for _, e in ipairs(c.marks) do
    local mx = x0 + (e.frame - 1) * sx
    if e.name == "iasa" then gd.fill(mx, by - 3, 2, 14, GREEN)
    elseif e.name == "gfx" then gd.fill(mx, by + 8, 2, 3, 0x40A0FFFF)
    elseif e.name:find("sfx") then gd.fill(mx, by - 3, 2, 3, 0xC77DFFFF)
    elseif e.name == "body_state" or e.name == "hurtbox_state" or e.name == "hurtboxes_state" then
      gd.fill(mx, by - 3, 2, 14, 0xFFFFFFFF)
    elseif e.name == "visibility" or e.name == "model_state" then gd.fill(mx, by + 8, 2, 3, YELLOW)
    end
  end
  gd.fill(x0 + (now - 1) * sx - 1, by - 4, 2, 16, 0xFF3030FF)
  -- the windows as text: frames, id, damage, angle, growth, base, weight-set, size
  local parts = {}
  for _, hw in ipairs(c.windows) do
    parts[#parts + 1] = string.format("f%d-%d #%d %d%% a%d g%d b%d w%d r%.1f", hw.from, hw.to, hw.id,
      hw.dmg, hw.angle, hw.kbg, hw.bkb, hw.wbk, hw.size)
  end
  local iasa
  for _, e in ipairs(c.marks) do if e.name == "iasa" then iasa = e.frame break end end
  local line = string.format("now %.1f  %s%s", now, iasa and ("IASA f" .. iasa .. "  ") or "",
    table.concat(parts, "  "))
  gd.text(x0, by + 12, line:sub(1, 150), GREY, 0.62)
  return y + 40
end

local HELP = {
  "Geno Lab - keys (game window focused, console closed)",
  "P pause/resume   N step (hold = slow play, CTRL = 10)   B step back (hold, CTRL = 10)",
  "TAB focus next fighter   X Lab on/off   F5/F6 save/load state 1   F3 this help",
  "1 hit/hurtboxes  2 model  3 skeleton  4 joint numbers  5 ECB  6 stage collision",
  "7 info panel  8 event log  9 hitbox labels  0 attributes (differences in yellow)",
  "M move timeline   PAGEUP/PAGEDOWN scrub the move   HOME replay from frame 1",
  "C both fighters into this move at this frame (lock-step)   R mirror P1's pad onto P2",
  "console: lab help | port N | history N | back N | dump [N] | move <id> [frame] | events",
}

local function draw_status(m)
  local h = gd.history()
  local s = string.format("GENO LAB  %s  frame %d  focus P%d  back %d/%d", gd.paused() and "PAUSED" or "running",
    m.frame, focus, h.back, h.depth)
  if m.netplay then s = "GENO LAB  (netplay: read-only)" end
  gd.fill(0, 0, 640, 14, 0x000000B0)
  gd.text(4, 1, s, gd.paused() and YELLOW or GREEN, 0.8)
  gd.text(560, 1, "F3 help", DIM, 0.75)
  if notice and gd.time() < notice_until then
    gd.fill(170, 440, 300, 16, 0x000000C0)
    gd.text(176, 442, notice[1], notice[2], 0.85)
  end
end

function on_draw()
  if not cfg.on then return end
  local m = gd.match()
  if not m.active then return end
  local list = gd.players()
  for _, p in ipairs(list) do
    if cfg.skel or cfg.joints then draw_skeleton(p, PORT_COLORS[p.port]) end
    if cfg.ecb then draw_ecb(p) end
    if cfg.labels then draw_hit_labels(p) end
  end
  draw_status(m)
  if cfg.info then draw_info(list) end
  if cfg.attrs then draw_attrs(list) end
  if cfg.timeline then
    local y = 300
    local a = gd.player(focus)
    if a then y = draw_timeline(a, y, PORT_COLORS[a.port]) end
    for _, p in ipairs(list) do
      if p.port ~= focus then draw_timeline(p, y, PORT_COLORS[p.port]) break end
    end
  end
  if cfg.log and #log_lines > 0 then
    local y0 = 470 - #log_lines * 10
    gd.fill(0, y0 - 2, 640, #log_lines * 10 + 4, 0x00000090)
    for i, l in ipairs(log_lines) do
      gd.text(4, y0 + (i - 1) * 10, string.format("%5d  %s", l[1], l[2]), l[3], 0.68)
    end
  end
  if cfg.help then
    gd.fill(40, 150, 560, #HELP * 14 + 12, 0x000000E0)
    for i, l in ipairs(HELP) do gd.text(50, 156 + (i - 1) * 14, l, i == 1 and YELLOW or WHITE, 0.85) end
  end
end

function on_unload()
  restore_draw()
end

-- ---- console ----------------------------------------------------------------------------------
local function dump(port)
  local p = gd.player(port)
  if p == nil then gd.log("no fighter on port " .. port) return end
  for _, l in ipairs(info_lines(p)) do gd.log(l[1]) end
  gd.log(string.format("anim_symbol %s  joints %d  hurtboxes %d  draw_flags 0x%02X", p.anim_symbol,
    p.joint_count, p.hurtbox_count, p.draw_flags))
end

gd.command("lab", function(arg)
  local cmd, rest = arg:match("^(%S*)%s*(.-)$")
  local n = tonumber(rest)
  if cmd == "" then
    cfg.on = not cfg.on
    if not cfg.on then restore_draw() end
    gd.log("Geno Lab " .. (cfg.on and "on" or "off"))
  elseif cmd == "help" then
    for _, l in ipairs(HELP) do gd.log(l) end
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
    local k, v = rest:match("^(%S+)%s+(%S+)$")
    if k and cfg[k] ~= nil then
      if v == "true" or v == "on" then cfg[k] = true
      elseif v == "false" or v == "off" then cfg[k] = false
      elseif tonumber(v) then cfg[k] = tonumber(v) end
      save_settings()
      gd.log(k .. " = " .. tostring(cfg[k]))
    else
      gd.log("lab set <key> <value>: keys on hit model skel joints ecb stage info log labels attrs history")
    end
  else
    gd.log("lab [help | port N | history N | back [N] | dump [N] | set <key> <value>]")
  end
end, "Geno Lab: help, port, history, back, dump, set")
