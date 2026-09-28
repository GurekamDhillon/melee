-- Lua stage content check (pc/scripts/examples/fd_stage_content on FD). Run as MELEE_PAD_SCRIPT with the
-- example in scripts/, MELEE_SCENE=mode=vs;p1=fox;p2=marth;stage=fd;time=0 (P2 a human port left idle: a CPU
-- wanders into the targets). Logs STGT PASS/FAIL lines and "STGT DONE fails N"; quits at the end.
-- Lua stage content check on FD with the fd_stage_content example loaded. "STGT" lines; ends with gd.quit().
local fails = 0
local function log(fmt, ...) gd.log("STGT " .. string.format(fmt, ...)) end
local function expect(ok, what, ...) if not ok then fails = fails + 1 end log("%s %s", ok and "PASS" or "FAIL", string.format(what, ...)) end
local function P() return gd.player(1) end
local function mname() return gd.motion_name(P().action) or tostring(P().action) end
local function hold(spec, n) gd.input(1, spec, n) gd.wait(n) end
local function targets()
  local n = 0
  for _, it in ipairs(gd.items() or {}) do if it.kind == 0xD1 then n = n + 1 end end
  return n
end
local breaks, all_broken = 0, 0
function on_target_broken(h, remaining) breaks = breaks + 1 local a, b = gd.player(1), gd.player(2) log("event on_target_broken %d remaining %d (P1 %.1f %.1f %s, P2 %.1f %.1f %s)", h, remaining, a.x, a.y, gd.motion_name(a.action) or "?", b.x, b.y, gd.motion_name(b.action) or "?") end
function on_all_targets_broken() all_broken = all_broken + 1 log("event on_all_targets_broken") end

gd.run(function()
  gd.wait_until(function() return gd.match().active and gd.match().frame > 150 end, 3000)
  hold(0, 5)
  for _, it in ipairs(gd.items() or {}) do log("item kind 0x%X at %.1f %.1f", it.kind, it.x or 0, it.y or 0) end
  local t0 = targets()
  expect(t0 == 10, "10 targets on FD (%d)", t0)

  -- the example's three pass-through Battlefield platforms (w 47): (-48.5, 34), (48.5, 34), (0, 68)
  gd.teleport(1, 0, 85) gd.wait(60)
  expect(not P().airborne and math.abs(P().y - 68) < 0.2, "lands and stands on the top platform (y %.2f, %s)", P().y, mname())
  hold({ y = -127 }, 3) hold(0, 60)
  expect(not P().airborne and math.abs(P().y) < 0.2, "drops through the top platform to the floor (y %.2f)", P().y)
  gd.teleport(1, 48.5, 50) gd.wait(60)
  expect(not P().airborne and math.abs(P().y - 34) < 0.2, "lands on the right platform (y %.2f)", P().y)
  hold({ y = -127 }, 3) hold(0, 60)
  expect(not P().airborne and math.abs(P().y) < 0.2, "drops through the right platform (y %.2f)", P().y)
  -- the example's platforms have no ledges: falling past the left one's outer end grabs nothing
  gd.fly(1, true) hold({ x = 40 }, 1) gd.fly(1, false) gd.teleport(1, -76, 30) hold(0, 40)
  expect(mname():find("Cliff") == nil, "no ledge grab on the example's platforms: %s", mname())
  gd.wait(60)
  -- a solid platform with ledges made here (high over the right side): stands, down stays, both ends grab
  local sp = gd.stage_add_platform(60, 110, 30, { passthrough = false, ledges = true })
  expect(sp ~= nil, "solid platform with ledges (%s)", tostring(sp))
  gd.teleport(1, 60, 125) gd.wait(60)
  expect(not P().airborne and math.abs(P().y - 110) < 0.2, "lands on the solid platform (y %.2f)", P().y)
  hold({ y = -127 }, 3) hold(0, 40)
  expect(not P().airborne and math.abs(P().y - 110) < 0.2, "down on the solid platform: stays (y %.2f, %s)", P().y, mname())
  for _, spot in ipairs({ { 81, 108, -127, "right end" }, { 39, 108, 127, "left end" } }) do
    -- face the ledge: a 1-frame flight toward it turns the fighter, then drop into Fall
    gd.fly(1, true) gd.teleport(1, spot[1] - spot[3] / 127 * 1, spot[2]) hold({ x = spot[3] // 4 }, 1)
    gd.fly(1, false) gd.teleport(1, spot[1], spot[2]) hold(0, 40)
    local nm = mname()
    log("ledge %s: %s at %.2f %.2f", spot[4], nm, P().x, P().y)
    expect(nm:find("Cliff") ~= nil, "ledge grab at the %s (%s)", spot[4], nm)
    hold({ y = 127 }, 2) hold(0, 60) -- climb / jump off
  end
  gd.stage_remove(sp)
  gd.teleport(1, 0, 10) gd.wait(60)

  -- a moving platform created here, moved every frame: the fighter rides it
  local m = gd.stage_add_platform(-20, 90, 30)
  expect(m ~= nil, "stage_add_platform returns a handle (%s)", tostring(m))
  gd.teleport(1, -20, 100) gd.wait(60)
  expect(not P().airborne and math.abs(P().y - 90) < 0.2, "stands on the new platform (y %.2f)", P().y)
  gd.history(600, 30)
  local x0 = P().x
  for f = 1, 60 do gd.stage_move(m, -20 + f * 0.5, 90 + f * 0.1) gd.wait(1) end
  gd.wait(1)
  log("ride: dx %.3f dy %.3f airborne %s", P().x - x0, P().y - 90, tostring(P().airborne))
  expect(math.abs(P().x - x0 - 30) < 1.0 and math.abs(P().y - 96) < 0.5 and not P().airborne, "rides the moving platform (+30, +6)")
  gd.wait(10)
  expect(math.abs(P().x - x0 - 30) < 1.0, "no drift after it stops (dx %.3f)", P().x - x0)
  -- the rewind exactness test over a moving platform with a fighter on it
  gd.run(function() for f = 1, 200 do gd.stage_move(m, 10 - f * 0.25, 96) gd.wait(1) end end)
  local ok, why = gd.rewind_test(60, true)
  expect(ok, "rewind_test started (%s)", tostring(why))
  gd.wait_until(function() return gd.rewind_test_result().phase == 0 end, 600)
  local r = gd.rewind_test_result()
  expect(r.pass == true, "rewind exact over a moving platform (%s)", tostring(r.text))
  gd.wait(210)
  -- remove the platform while standing on it: falls
  gd.stage_remove(m) gd.wait(20)
  expect(P().airborne or P().y < 89, "removing the platform under it: falls (y %.2f)", P().y)
  gd.wait(120)

  -- savestate, break targets, load: they come back
  gd.teleport(1, 0, 0) gd.wait(30)
  gd.savestate(2) gd.wait(2)
  local positions = { { -82, 48 }, { -55, 58 }, { -28, 66 }, { 0, 74 }, { 28, 66 }, { 55, 58 }, { 82, 48 }, { -45, 12 }, { 0, 17 }, { 45, 12 } }
  for i, p in ipairs(positions) do
    -- up to 3 tries: the CPU wanders and can trade hits with P1; a target already broken is skipped
    for _ = 1, 3 do
      local here = false
      for _, it in ipairs(gd.items() or {}) do
        if it.kind == 0xD1 and math.abs(it.x - p[1]) < 1 and math.abs(it.y - p[2]) < 1 then here = true end
      end
      if not here then break end
      while not pcall(gd.teleport, 1, p[1], p[2] - 8) do gd.wait(10) end
      gd.wait(1) hold({ buttons = "A" }, 2) gd.wait(25)
    end
    log("target %d: breaks %d, targets left %d", i, breaks, targets())
  end
  gd.wait(30)
  expect(breaks == 10 and all_broken == 1 and targets() == 0, "10 hits: 10 on_target_broken, 1 on_all_targets_broken, none left (%d, %d, %d)", breaks, all_broken, targets())
  gd.loadstate(2) gd.wait(5)
  expect(targets() == 10, "loadstate: all 10 targets back (%d)", targets())
  -- and they still break after the load
  local b0 = breaks
  gd.teleport(1, 0, 9) gd.wait(1) hold({ buttons = "A" }, 2) gd.wait(30)
  expect(breaks == b0 + 1 and targets() == 9, "a restored target breaks (%d events, %d left)", breaks - b0, targets())
  -- a removed target raises no event
  local h = gd.spawn_target(-10, 30)
  expect(h ~= nil and targets() == 10, "spawn_target (%s)", tostring(h))
  local b1 = breaks
  expect(gd.stage_remove(h) == true, "stage_remove(target)")
  gd.wait(5)
  expect(breaks == b1 and targets() == 9, "removed target: no event (%d left)", targets())
  -- the camera and blast zones did not move: a fall below FD still KOs
  gd.teleport(1, 0, -300)
  local died = gd.wait_until(function() return P().action <= 10 end, 60)
  log("after the pit: action %s", mname())
  expect(died ~= false and P().action <= 10, "the bottom blast zone still KOs (%s)", mname())

  log("DONE fails %d", fails)
  gd.wait(120)
  gd.quit()
end)
