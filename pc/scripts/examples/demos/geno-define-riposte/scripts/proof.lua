-- @name: Vanilla Riposte proof: Lua counter, follow-up, streak and rewind
-- @gameplay: true
-- Headless proof for the slice 5 second fixture (docs/geno.md section 23.7). Offline LAB, vanilla-riposte as P1 against a script-mode Mario CPU.
--   1. a full stance with a CPU jab inside the window: the hit is countered (P1's percent unmoved), P1 goes to the Riposte state (0x401),
--      the typed state holds streak 1 and power 9.0 (1.5 x a 2 percent jab, floored at 9), the answer damages P2
--   2. a second counter at once: streak 2, power 10.0 (9 + 1); A inside the answer chains into the Follow state (0x402) with follow = true
--   3. a stance with no hit: the streak is back to 0 once the window has passed, and the stance ends on time (about 30 frames)
--   4. a counter after the whiff starts a new streak (1, not 3)
--   5. B let go early (12 frames): the stance cancels (well before 30 frames) and the streak is 0
--   5b. a jab from behind: the answer turns the fighter around (ctx.turn), a hit from the front does not
--   6. gd.rewind_test across a stance, a counter and its answer: diff_compared == 0
--   7. no Lua fault anywhere (gd.fighter_lua(1).faults == 0)
-- It logs PROOF lines and ends with "PROOF RESULT: PASS" or "PROOF RESULT: FAIL <why>".
local PARRY, RIPOSTE, FOLLOW = 0x400, 0x401, 0x402
local failed, done, quit_t = nil, false, 0
local step, t = 1, 0
local S = {}   -- scratch shared by the steps

local function fail(why)
  if not failed then failed = why; gd.log('PROOF RESULT: FAIL ' .. why) end
end
local function P(n) return gd.player(n) end
local function lua() return gd.fighter_lua(1) end
local function log(fmt, ...) gd.log('PROOF ' .. string.format(fmt, ...)) end
local function approx(a, b, eps) return a ~= nil and math.abs(a - b) <= (eps or 0.05) end
local function next_step() step, t, S = step + 1, 0, {} end

-- P2 stands in jab range in front of P1, facing P1; true once both are actionable on the floor
local function ready()
  local p1, p2 = P(1), P(2)
  if not p1 or not p2 then return false end
  if p1.airborne or p2.airborne or p2.in_hitstun or p2.in_hitlag or p1.in_hitlag then return false end
  if p1.action == PARRY or p1.action == RIPOSTE or p1.action == FOLLOW then return false end
  return p1.action_frame >= 40 or (p1.action ~= PARRY and p1.action ~= RIPOSTE and p1.action ~= FOLLOW and t > 20)
end
local function place()
  local p1 = P(1)
  gd.teleport(2, p1.x + 9 * p1.facing, p1.y)
end

-- a stance + (optionally) a CPU jab at stance frame jab_at + (optionally) A in the answer at its frame a_at (gd.player's action_frame, which counts the
-- hitlag frames the move's own clock, ctx.self.action_frame, does not: the answer's follow-up window 6..16 is about 13..23 here); true when it is over
local function exchange(frames_held, jab_at, a_at)
  local p1, p2 = P(1), P(2)
  if t == 1 then
    S.p1_pct0, S.p2_pct0, S.face0 = p1.percent, p2.percent, p1.facing
    S.max_streak, S.seen = 0, {}
    gd.input(1, {buttons = 'B', y = -127}, frames_held)
  end
  local l = lua()
  S.seen[p1.action] = (S.seen[p1.action] or 0) + 1
  if p1.action == PARRY then S.parry_last = p1.action_frame end
  if jab_at and not S.jabbed and p1.action == PARRY and p1.action_frame >= jab_at then
    gd.cpu_pad(2, {buttons = 'A'}, 2); S.jabbed = true
  end
  if a_at and not S.pressed and p1.action == RIPOSTE and p1.action_frame >= a_at then
    gd.input(1, {buttons = 'A'}, 2); S.pressed = true
  end
  if p1.action == RIPOSTE and not S.rip then
    S.rip = {streak = l.state.streak, power = l.state.power, follow = l.state.follow, p1_pct = p1.percent, face = p1.facing}
  end
  if p1.action == FOLLOW and not S.fol then S.fol = {follow = l.state.follow, power = l.state.power} end
  if p1.hitboxes and (p1.action == RIPOSTE or p1.action == FOLLOW) then   -- the live hitbox damage, as the Lua set it (after staling)
    for _, h in ipairs(p1.hitboxes) do
      if h.damage and h.damage > 0 then
        local key = p1.action == FOLLOW and 'fol_dmg' or 'rip_dmg'
        S[key] = h.damage
      end
    end
  end
  if S.rip then S.p2_max = math.max(S.p2_max or 0, p2.percent) end
  if t > 12 and p1.action ~= PARRY and p1.action ~= RIPOSTE and p1.action ~= FOLLOW and (not S.rip or p1.action_frame > 3) then
    S.end_t = S.end_t or t
  end
  if t > 150 then S.end_t = S.end_t or t end
  return S.end_t ~= nil and t >= S.end_t + 40
end

local steps = {
  -- 1. settle, then a counter
  function()
    if t == 1 then gd.cpu_mode(2, 'stand') end
    if t == 150 then gd.cpu_mode(2, 'script'); local l = lua(); if not l then return fail('gd.fighter_lua(1) is nil: Riposte is not P1 or declares no Lua state') end
      log('layout: profile %d power=%s streak=%s follow=%s', l.profile, tostring(l.state.power), tostring(l.state.streak), tostring(l.state.follow)) end
    if t > 150 and ready() then place(); return true end
  end,
  function()   -- 1. the counter
    if t == 2 then place() end
    if exchange(30, 3, nil) then
      if not S.rip then return fail('1: the jab was not countered (never reached the Riposte state; saw ' .. (function() local r = {} for k, v in pairs(S.seen) do r[#r + 1] = k .. 'x' .. v end return table.concat(r, ',') end)() .. ')') end
      log('1 counter: streak %s power %.3f, P1 percent %.1f -> %.1f, P2 percent %.1f -> %.1f', tostring(S.rip.streak), S.rip.power, S.p1_pct0, S.rip.p1_pct, S.p2_pct0, S.p2_max or -1)
      if S.rip.p1_pct ~= S.p1_pct0 then return fail('1: P1 took damage in the window (countered hits are negated)') end
      if S.rip.streak ~= 1 or not approx(S.rip.power, 9.0) then return fail('1: want streak 1 power 9.0') end
      if S.rip.face ~= S.face0 then return fail('1: a hit from the front turned the fighter around') end
      if (S.p2_max or 0) - S.p2_pct0 < 5 then return fail('1: the answer did not damage P2') end
      return true
    end
  end,
  -- 2. a second counter in a row, chained into the follow-up
  function()
    if t == 1 and not ready() then t = 0 return end
    if t == 2 then place() end
    if t < 2 then return end
    local tt = t; t = t - 1
    local r = exchange(30, 3, 15)
    t = tt
    if r then
      if not S.rip then return fail('2: the second jab was not countered') end
      log('2 counter+follow: streak %s power %.3f follow-flag %s; Follow state seen %s frames; P2 percent %.1f -> %.1f', tostring(S.rip.streak), S.rip.power, tostring(S.fol and S.fol.follow), tostring(S.seen[FOLLOW]), S.p2_pct0, S.p2_max or -1)
      if S.rip.streak ~= 2 or not approx(S.rip.power, 10.0) then return fail('2: want streak 2 power 10.0') end
      if not S.seen[FOLLOW] or not S.fol or S.fol.follow ~= true then return fail('2: A in the answer did not chain into the Follow state') end
      log('2 hitbox damage: answer %s (want power 10.0), follow-up %s (want 0.6 x 10 = 6.0)', tostring(S.rip_dmg), tostring(S.fol_dmg))
      if S.rip_dmg and not approx(S.rip_dmg, 10.0, 0.1) then return fail('2: the answer hitbox damage is ' .. S.rip_dmg .. ', not the power 10.0') end
      if S.fol_dmg and not approx(S.fol_dmg, 6.0, 0.1) then return fail('2: the follow-up hitbox damage is ' .. S.fol_dmg .. ', not 0.6 x the power') end
      return true
    end
  end,
  -- 3. a whiff: nothing hits, the window passes, the streak breaks, the stance ends on time
  function()
    if t == 1 and not ready() then t = 0 return end
    if t < 2 then return end
    local tt = t; t = t - 1
    local r = exchange(30, nil, nil)
    t = tt
    if S.parry_last == 27 or (S.parry_last and S.parry_last >= 26 and not S.streak27) then
      if not S.streak27 then S.streak27 = lua().state.streak end
    end
    if r then
      log('3 whiff: stance lasted to action frame %s, streak %s, answer entered: %s', tostring(S.parry_last), tostring(lua().state.streak), tostring(S.rip ~= nil))
      if S.rip then return fail('3: a whiff reached the Riposte state') end
      if lua().state.streak ~= 0 then return fail('3: the whiff did not break the streak (streak ' .. tostring(lua().state.streak) .. ')') end
      if not S.parry_last or S.parry_last < 26 or S.parry_last > 32 then return fail('3: the stance ended at frame ' .. tostring(S.parry_last) .. ', not about 30') end
      return true
    end
  end,
  -- 4. a counter after the whiff starts a new streak
  function()
    if t == 1 and not ready() then t = 0 return end
    if t < 2 then return end
    if t == 3 then place() end
    local tt = t; t = t - 1
    local r = exchange(30, 3, nil)
    t = tt
    if r then
      if not S.rip then return fail('4: the jab was not countered') end
      log('4 counter after the whiff: streak %s power %.3f', tostring(S.rip.streak), S.rip.power)
      if S.rip.streak ~= 1 or not approx(S.rip.power, 9.0) then return fail('4: want streak 1 power 9.0 (the whiff broke the streak)') end
      return true
    end
  end,
  -- 5. let go of B early: the stance cancels
  function()
    if t == 1 and not ready() then t = 0 return end
    if t < 2 then return end
    local tt = t; t = t - 1
    local r = exchange(12, nil, nil)
    t = tt
    if r then
      log('5 early cancel: stance ended at action frame %s, streak %s', tostring(S.parry_last), tostring(lua().state.streak))
      if not S.parry_last or S.parry_last > 18 then return fail('5: B let go at 12 frames, yet the stance lasted to frame ' .. tostring(S.parry_last)) end
      if lua().state.streak ~= 0 then return fail('5: the cancel did not break the streak') end
      return true
    end
  end,
  -- 5b. a jab from BEHIND: the answer turns the fighter around (ctx.self.hit_from < 0 -> ctx.turn())
  function()
    if t == 1 and not ready() then t = 0 return end
    if t < 2 then return end
    local p1, p2 = P(1), P(2)
    if not S.turned then
      if t == 2 then gd.teleport(2, p1.x - 9 * p1.facing, p1.y) end
      if t == 4 then gd.cpu_pad(2, {x = 90 * p1.facing}, 2) end   -- a flick toward P1 turns P2 around
      if t > 4 and p2.facing == p1.facing and not p2.in_hitlag and p2.action == 14 then S.turned, S.t0 = true, t end
      if t > 120 then return fail('5b: P2 could not be turned to face P1 from behind') end
      return
    end
    if t == S.t0 + 1 then gd.teleport(2, p1.x - 9 * p1.facing, p1.y) end
    if t <= S.t0 + 3 then return end
    local tt, t0 = t, S.t0
    t = t - (S.t0 + 3)
    local turned = S.turned
    local r = exchange(30, 3, nil)
    S.turned, S.t0 = turned, t0
    t = tt
    if r then
      if not S.rip then return fail('5b: the jab from behind was not countered') end
      log('5b from behind: facing %s -> %s at the answer, streak %s', tostring(S.face0), tostring(S.rip.face), tostring(S.rip.streak))
      if S.rip.face ~= -S.face0 then return fail('5b: the answer to a hit from behind did not turn the fighter around') end
      if S.rip.streak ~= 1 then return fail('5b: want streak 1 (the early cancel broke it)') end
      return true
    end
  end,
  -- 6. rewind_test across a stance, a counter and its answer
  function()
    if t == 1 and not ready() then t = 0 return end
    if t < 2 then return end
    if t == 3 then
      place()
      assert(gd.history(300, 1), 'history refused')
      local ok, why = gd.rewind_test(120, true)
      if not ok then return fail('6: rewind_test refused: ' .. tostring(why)) end
      log('6 rewind_test started')
    end
    local tt = t; t = t - 1
    exchange(30, 3, 15)
    t = tt
    if t > 20 then
      local r = gd.rewind_test_result()
      if r and r.phase == 0 and r.pass ~= nil then
        log('6 rewind_test pass=%s diff=%s diff_compared=%s %s', tostring(r.pass), tostring(r.diff), tostring(r.diff_compared), tostring(r.text))
        if not r.pass or r.diff_compared ~= 0 then return fail('6: rewind_test pass=' .. tostring(r.pass) .. ' diff_compared=' .. tostring(r.diff_compared)) end
        return true
      end
      if t > 1500 then return fail('6: rewind_test did not finish') end
    end
  end,
  function()
    local l = lua()
    log('7 faults: %d (last %d)', l.faults, l.last_fault)
    if l.faults ~= 0 then return fail('7: the Lua faulted ' .. l.faults .. ' time(s), last code ' .. l.last_fault) end
    done = true
    gd.log('PROOF RESULT: PASS')
    return true
  end,
}

function on_frame()
  if failed or done then
    quit_t = quit_t + 1 -- the result is logged; let the log flush, then end the run (a headless proof run has nobody to close it)
    if quit_t == 30 then gd.quit() end
    return
  end
  if not gd.match().active then return end
  if not P(1) or not P(2) then return end
  t = t + 1
  local f = steps[step]
  if f and f() then next_step() end
end
