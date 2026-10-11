-- throw_banks_rewind.lua: the lazy throw stand-in banks (geno.md 22.16) against savestate / rewind / SyncTest.
--
-- Scene (run_throw_banks_rewind.sh builds it): LAB, P1 Mario (the retail victim) and three template Geno throwers whose
-- "thrown_as" families are three different banks, here P2 geno-fox, P3 geno-falco, P4 geno-falcon. With MELEE_THROW_BANKS=1 a
-- second family evicts the first, so every throw after the first one in a different family is an eviction.
--
-- A "throw" parks the others, puts the thrower and the victim face to face, grabs, waits for the pummel window, throws
-- forward and records 70 frames of the victim: motion id, animation frame, position and the hash of its hurtbox capsules (the
-- pose, so a victim that kept a stale / zero animation key shows as a different hash). Same state in, same trace out.
--
-- Cases (each prints  TBR <case> PASS|FAIL ...; the last line is  TBR DONE n/m):
--   A  rewind across a FIRST load      : savestate before any bank is read, throw (loads), loadstate, throw again
--                                        (must load again, same trace: the statics and the arena went back with the snapshot)
--   B  rewind across an EVICTION       : savestate with Fox resident, throw Falco/Falcon/Fox in a row, loadstate
--                                        (Fox resident again), same three throws: same three traces
--   C  restore to a RESIDENT bank      : evict Fox with Falcon, loadstate the Fox snapshot and throw Fox first: the bank the
--                                        snapshot says is resident must be the bytes the snapshot restored (no reload needed)
--   D  free-running reference          : three further evicting rounds with no rewind; every throw must reach a Thrown state.
--                                        Hashes are logged for comparing one-arena and three-arena runs.
--
-- SyncTest proof lives in throw_banks_cpu_synctest.lua: no script input or teleports.
local V = 1
local T = {fox = 2, falco = 3, falcon = 4}
-- Keep idle fighters on FD's floor; parking beyond its edges makes them die and
-- leaves subsequent teleports refused during death/respawn.
local PARK = {-70, 70, -55, 55, -40, 40}
local pass, total = 0, 0

local function say(...) print(string.format(...)) end
local function ok(name, cond, detail)
  total = total + 1
  if cond then pass = pass + 1 end
  say("TBR %s %s %s", name, cond and "PASS" or "FAIL", detail or "")
end

local function tp(p, x, y)
  local last_error
  for _ = 1, 200 do
    local okk, err = pcall(gd.teleport, p, x, y)
    if okk then return true end
    last_error = err
    gd.wait(1)
  end
  error(string.format("TBR teleport failed port %d: %s", p, tostring(last_error)))
end

local function digest(h, s)
  for i = 1, #s do h = (h * 31 + s:byte(i)) % 2147483629 end
  return h
end

local function pose(v)
  local parts = {}
  for _, c in ipairs(gd.hurtboxes(v) or {}) do
    parts[#parts + 1] = string.format("%.3f,%.3f,%.3f,%.3f,%.3f,%.3f", c.ax, c.ay, c.az, c.bx, c.by, c.bz)
  end
  return table.concat(parts, ";")
end

-- one throw; returns {hash=, seen=, thrown=}
local function throw(t)
  for port = 1, 4 do
    if port ~= t and port ~= V then tp(port, PARK[port], 0) end
    gd.set_percent(port, 0)
  end
  tp(t, 0, 0)
  gd.wait(30)
  local pt = gd.player(t)
  local face = (pt and pt.facing and pt.facing < 0) and -1 or 1
  tp(V, 8 * face, 0)
  gd.wait(2)
  gd.input(t, {buttons = "Z"}, 3); gd.wait(3)
  gd.input(t, {}, 36); gd.wait(36)
  gd.input(t, {x = 127 * face}, 3); gd.wait(3)
  local h, seen, last, any = 7, {}, nil, false
  for _ = 1, 70 do
    gd.wait(1)
    local pv = gd.player(V)
    local m = pv and pv.motion_name or "?"
    if m ~= last then seen[#seen + 1] = m; last = m end
    if m:find("Thrown") then any = true end
    h = digest(h, string.format("%s|%s|%.3f|%.3f|%s", tostring(pv.action), tostring(pv.anim_frame), pv.x, pv.y, pose(V)))
  end
  return {hash = h, seen = table.concat(seen, ">"), thrown = any}
end

local function seqs(list)
  local out = {}
  for _, t in ipairs(list) do out[#out + 1] = throw(t) end
  return out
end

local function same(name, a, b, what)
  local good = #a == #b
  local d = {}
  for i = 1, #a do
    local g = b[i] ~= nil and a[i].hash == b[i].hash and a[i].thrown and b[i].thrown
    d[#d + 1] = string.format("[%d %s h=%d/%s]", i, g and "=" or "!=", a[i].hash, b[i] and tostring(b[i].hash) or "nil")
    good = good and g
  end
  ok(name, good, what .. " " .. table.concat(d, " ") .. " seen1=" .. (a[1] and a[1].seen or "?"))
end

gd.run(function()
  local okk, err = pcall(function()
    assert(gd.wait_until(function() return gd.match().active and gd.match().frame > 200 end, 9000), "no match")
    gd.wait(60)

    -- A: rewind across a first load
    gd.savestate(1); gd.wait(2)
    local a1 = seqs({T.fox})
    gd.loadstate(1); gd.wait(2)
    local a2 = seqs({T.fox})
    same("A rewind across first load", a1, a2, "fox")

    -- B: rewind across an eviction (Fox resident after A, so the snapshot holds Fox)
    gd.savestate(2); gd.wait(2)
    local order = {T.falco, T.falcon, T.fox}
    local b1 = seqs(order)
    gd.loadstate(2); gd.wait(2)
    local b2 = seqs(order)
    same("B rewind across evictions", b1, b2, "falco,falcon,fox")

    -- C: overwrite Fox with Falcon, then restore the Fox snapshot (no reload expected for the first Fox throw).
    seqs({T.falcon})
    gd.loadstate(2); gd.wait(2)
    local c1 = seqs({T.fox, T.falco, T.fox})
    seqs({T.falcon})
    gd.loadstate(2); gd.wait(2)
    local c2 = seqs({T.fox, T.falco, T.fox})
    same("C restore to a resident bank", c1, c2, "fox,falco,fox")

    -- D: free-running throws after repeated eviction; log pose hashes for cross-budget comparison.
    local d = {}
    for r = 1, 3 do d[r] = seqs({T.falcon, T.fox, T.falco}) end
    local allthrown = true
    for r = 1, 3 do for i = 1, 3 do allthrown = allthrown and d[r][i].thrown end end
    local hs = {}
    for r = 1, 3 do for i = 1, 3 do hs[#hs + 1] = d[r][i].hash end end
    ok("D free running evictions", allthrown, "9 throws all reached a Thrown state; hashes " .. table.concat(hs, ","))
  end)
  if not okk then say("TBR ERROR %s", tostring(err)) end
  say("TBR DONE %d/%d", pass, total)
  gd.quit()
end)
