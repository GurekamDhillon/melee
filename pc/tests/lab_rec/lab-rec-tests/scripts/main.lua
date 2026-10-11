-- @name: LAB recorder acceptance
-- @gameplay: true
-- Offline LAB, P1 Goku (geno:goku-xv2) v P2 Mario, both human ports, FD. Records a scripted ~10 s
-- combo (gd.rec_start/stop), plays it back (gd.rec_play) and compares both fighters frame by frame
-- (action, x, y, vx, vy, percent, facing, action_frame); then the same for gd.rec_save_history, the
-- rewind + resume replay (R2) and a trim. Output: LABREC lines and "LABREC RESULT: PASS|FAIL ...".
local failures = {}
local function fail(msg) failures[#failures + 1] = msg; gd.log("LABREC FAIL " .. msg) end
local function check(c, msg) if not c then fail(msg) end end

local function sig()
  local t = {}
  for p = 1, 2 do
    local f = gd.player(p)
    t[#t + 1] = string.format("%d:%d:%d:%.5f:%.5f:%.5f:%.5f:%.3f:%d", p, f.action, f.action_frame, f.x, f.y, f.vx, f.vy, f.percent, f.facing)
  end
  return table.concat(t, "|")
end

local seed = 12345
local function rnd(n) seed = (seed * 1103515245 + 12345) % 2147483648; return (seed >> 8) % n end
local P1 = { { buttons = "A", x = 100 }, { buttons = "A" }, { buttons = "B" }, { buttons = "B", x = 100 }, { buttons = "A", y = -100 },
  { buttons = "X" }, { x = 127 }, { x = -127 }, { buttons = "A+B" }, { buttons = "Z" }, { buttons = "B", y = 100 }, { buttons = "A", x = -100 } }
local P2 = { {}, {}, { x = 100 }, { x = -100 }, { buttons = "X" }, { buttons = "A" } }

-- drive both fighters for n frames, recording sigs[match frame]
local function drive(n, into)
  local cur1, cur2
  for i = 0, n - 1 do
    if i % 7 == 0 then cur1 = P1[rnd(#P1) + 1]; cur2 = P2[rnd(#P2) + 1] end
    gd.input(1, cur1, 1)
    gd.input(2, cur2, 1)
    gd.wait(1)
    if into then into[gd.match().frame] = sig() end
  end
end

-- let a played recording run out, sampling into `into`
local function observe(into)
  local guard = 0
  while gd.rec_status().mode == "playing" and guard < 4000 do
    gd.wait(1)
    if gd.rec_status().mode == "playing" then into[gd.match().frame] = sig() end
    guard = guard + 1
  end
end

local function compare(label, a, b, lo, hi, min)
  local n, diff, first = 0, 0, nil
  for f = lo, hi do
    if a[f] ~= nil and b[f] ~= nil then
      n = n + 1
      if b[f] ~= a[f] then diff = diff + 1; first = first or f end
    end
  end
  gd.log(string.format("LABREC %s compared=%d diff=%d%s", label, n, diff, first and (" first=" .. first .. " rec " .. tostring(a[first]) .. " got " .. tostring(b[first])) or ""))
  check(n >= (min or 100), label .. ": too few frames compared (" .. n .. ")")
  check(diff == 0, label .. ": " .. diff .. " frames differ")
  return n, diff
end

local function wait_mode(mode, maxf, label)
  local ok = gd.wait_until(function() return gd.rec_status().mode == mode end, maxf or 600)
  check(ok, label .. ": recorder never reached " .. mode .. " (is " .. gd.rec_status().mode .. ")")
  return ok
end

gd.run(function()
  local ok, err = pcall(function()
    assert(gd.wait_until(function() return gd.match().active and gd.match().frame > 120 and gd.player(1) and gd.player(2) end, 3000), "match timeout")
    check(tostring(gd.player(1).char_name):lower():find("goku"), "P1 is " .. tostring(gd.player(1).char_name))
    assert(gd.history(1200, 30), "history")
    gd.wait(30)

    -- R3a: record 10 s, play back
    local sigs = {}
    local name, why = gd.rec_start("acc")
    assert(name, "rec_start: " .. tostring(why))
    wait_mode("recording", 600, "start")
    local base = gd.rec_status().base
    drive(600, sigs)
    local st, frames = gd.rec_stop()
    check(st and frames >= 599, "rec_stop frames=" .. tostring(frames))
    gd.log("LABREC recorded " .. tostring(frames) .. " frames from " .. base)
    local found
    for _, r in ipairs(gd.rec_list()) do if r.name == "acc" then found = r end end
    check(found and found.ok and found.frames == frames, "rec_list row")
    check(sigs[base + 599] ~= sigs[base], "the fighters never moved")
    gd.wait(60) -- some live time after

    local ok1, why1 = gd.rec_play("acc")
    assert(ok1, "rec_play: " .. tostring(why1))
    wait_mode("playing", 600, "play")
    local played = {}
    observe(played)
    compare("record->play", sigs, played, base, base + 599)

    -- R2: rewind, then resume REPLAYS the log, not live (the owner's trap). Frames the rewind covers are sampled first.
    local post = {}
    drive(120, post)
    local h0 = gd.history()
    assert(gd.rewind_to(h0.now - 100), "rewind_to")
    assert(gd.wait_until(function() return not gd.history().busy end, 600), "rewind never finished")
    local h1 = gd.history()
    check(h1.replaying and h1.fwd >= 95, "after rewind: replaying=" .. tostring(h1.replaying) .. " fwd=" .. tostring(h1.fwd))
    gd.resume()
    local rep, first_fwd = {}, nil
    for _ = 1, 400 do
      gd.wait(1)
      local h = gd.history()
      if first_fwd == nil then first_fwd = h.fwd end
      if not h.replaying then break end
      rep[gd.match().frame] = sig()
    end
    check(first_fwd ~= nil and first_fwd > 0 and first_fwd < h1.fwd, "after resume the first tick is still replaying: fwd " .. h1.fwd .. " -> " .. tostring(first_fwd))
    compare("rewind->resume replay", post, rep, h0.now - 100, h0.now, 80)
    -- go live on demand
    assert(gd.rewind_to(gd.history().now - 60), "rewind_to 2")
    assert(gd.wait_until(function() return not gd.history().busy end, 600), "rewind 2 never finished")
    gd.rewind_live()
    gd.resume() -- rewind_to leaves the game paused
    gd.wait(1)
    local h3 = gd.history()
    check(not h3.replaying and h3.fwd == 0, "rewind_live: replaying=" .. tostring(h3.replaying) .. " fwd=" .. tostring(h3.fwd))
    gd.wait(30)

    -- R3b: save-history of live play, then play it back
    local live = {}
    drive(240, live)
    local n2, why2 = gd.rec_save_history("hist", 3)
    assert(n2, "rec_save_history: " .. tostring(why2))
    wait_mode("idle", 3000, "save-history")
    local hsr
    for _, r in ipairs(gd.rec_list()) do if r.name == "hist" then hsr = r end end
    check(hsr and hsr.ok and hsr.frames >= 170, "history recording frames=" .. tostring(hsr and hsr.frames))
    local lo = gd.match().frame - 240
    local ok3, why3 = gd.rec_play("hist")
    assert(ok3, "rec_play hist: " .. tostring(why3))
    wait_mode("playing", 600, "play hist")
    local played2 = {}
    observe(played2)
    compare("history->play", live, played2, lo, lo + 400)

    -- trim and keep-last
    local cut, why4 = gd.rec_trim("acc", 300, 450, "acc-cut")
    assert(cut, "rec_trim: " .. tostring(why4))
    wait_mode("idle", 6000, "trim")
    local ok5, why5 = gd.rec_play("acc-cut")
    assert(ok5, "rec_play acc-cut: " .. tostring(why5))
    wait_mode("playing", 600, "play cut")
    local played3 = {}
    observe(played3)
    compare("trim->play", sigs, played3, base + 300, base + 449)
    local k, why6 = gd.rec_keep_last(2, "acc")
    assert(k, "rec_keep_last: " .. tostring(why6))
    wait_mode("idle", 6000, "keep-last")
    local ok7 = gd.rec_play("acc-last2")
    check(ok7, "rec_play acc-last2")
    wait_mode("playing", 600, "play last")
    local played4 = {}
    observe(played4)
    compare("keep-last->play", sigs, played4, base + 480, base + 599)

    local r1, w1 = gd.rec_play("no-such-take")
    check(r1 == false and w1, "a missing recording must be refused")
  end)
  if not ok then fail(tostring(err)) end
  gd.log("LABREC RESULT: " .. (#failures == 0 and "PASS" or ("FAIL " .. table.concat(failures, "; "))))
  gd.quit()
end)
