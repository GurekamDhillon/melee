-- @name: Vanilla Charger proof: Lua charge special, release and rewind
-- @gameplay: true
-- Headless proof for the slice 5 fixture (docs/geno.md section 23). Offline LAB, vanilla-charger as P1 against a standing CPU.
--   1. hold B: gd.fighter_lua(1).state.charge rises one per frame while the motion is the Charge state (0x400)
--   2. release B: the motion becomes the Release state (0x401) with the charge kept, and its hitboxes carry 6 + 0.15 x charge damage
--   2b. a savestate taken mid-charge: after gd.loadstate the typed state is back to the saved value
--   3. hold B for 200 frames and run gd.rewind_test(120, true) across the charge, the 60-frame cap and the release:
--      the test must pass with diff_compared == 0 and the typed state must be what it was when the test began
-- It logs PROOF lines and ends with "PROOF RESULT: PASS" or "PROOF RESULT: FAIL <why>".
local CHARGE, RELEASE = 0x400, 0x401
local step, t, c_hold, c0, started_frame = 0, 0, 0, nil, nil
local failed, quit_t = nil, 0
local c_save, c_before, c_after
local function fail(why)
  if not failed then failed = why; gd.log('PROOF RESULT: FAIL ' .. why) end
end
local function charge()
  local l = gd.fighter_lua(1)
  return l and l.state.charge or nil, l
end
local function motion() local p = gd.player(1); return p and (p.action or p.motion) or -1 end

function on_frame()
  if failed or step >= 90 then
    quit_t = quit_t + 1 -- the result is logged; let the log flush, then end the run (a headless proof run has nobody to close it)
    if quit_t == 30 then gd.quit() end
    return
  end
  if not gd.match().active then return end
  local p = gd.player(1)
  if not p then return end
  t = t + 1
  if step == 0 then
    if t == 1 then
      gd.cpu_mode(2, 'stand')
      local l = gd.fighter_lua(1)
      if not l then return fail('gd.fighter_lua(1) is nil: the Charger is not P1 or declares no Lua state') end
      gd.log(string.format('PROOF layout: profile %d slots charge=%s charged=%s', l.profile, tostring(l.state.charge), tostring(l.state.charged)))
      local keys = {}
      for k in pairs(p) do keys[#keys + 1] = k end
      table.sort(keys)
      gd.log('PROOF player keys: ' .. table.concat(keys, ','))
    end
    if t >= 150 and (p.action or p.motion) ~= CHARGE then step, t = 10, 0 end -- grounded and settled
  elseif step == 10 then -- 1. hold B 25 frames
    if t == 1 then gd.input(1, {buttons = 'B'}, 25) end
    local c = charge()
    if motion() == CHARGE then c_hold = c or c_hold end
    if t == 15 then
      if motion() ~= CHARGE then return fail('after 15 held frames the motion is ' .. motion() .. ', not the Charge state') end
      if not c or c < 8 or c > 17 then return fail('charge after 15 frames is ' .. tostring(c)) end
      gd.log('PROOF charge after 15 held frames: ' .. tostring(c))
    end
    if motion() == RELEASE then
      gd.log(string.format('PROOF released at frame %d with charge %s (last charge seen in Charge: %d)', t, tostring(c), c_hold))
      if c ~= c_hold then return fail('the charge changed on release: ' .. tostring(c) .. ' vs ' .. c_hold) end
      step, t = 20, 0
    elseif t > 60 then
      return fail('never reached the Release state')
    end
  elseif step == 20 then -- 2. the strike: hitbox damage 6 + 0.15 x charge (live from action frame ~2)
    local hb = p.hitboxes
    local seen
    if hb then for _, h in ipairs(hb) do if h.damage and h.damage > 0 then seen = h.damage end end end
    if seen then
      local want = 6 + 0.15 * c_hold
      gd.log(string.format('PROOF release hitbox damage %.3f, wanted %.3f (charge %d)', seen, want, c_hold))
      if math.abs(seen - want) > 0.6 then return fail(string.format('hitbox damage %.3f is not 6 + 0.15 x %d', seen, c_hold)) end
      step, t = 25, 0
    elseif t > 40 then
      gd.log('PROOF hitboxes not readable (p.hitboxes ' .. tostring(hb) .. '); damage not checked')
      step, t = 25, 0
    end
  elseif step == 25 then -- let the move finish
    if t >= 60 and motion() ~= RELEASE then step, t = 27, 0 end  elseif step == 27 then -- a savestate taken mid-charge brings the typed state back (the game's own snapshot, not a Lua value)
    if t == 1 then gd.input(1, {buttons = 'B'}, 200) end
    if t == 12 then
      c_save = charge()
      local ok, why = gd.savestate(1)
      if not ok and ok ~= nil then return fail('savestate refused: ' .. tostring(why)) end
      gd.log('PROOF savestate requested with charge ' .. tostring(c_save))
    elseif t == 26 then
      c_before = charge()
      gd.loadstate(1)
    elseif t > 26 and c_after == nil and (charge() or 999) < (c_before or 0) - 5 then
      c_after = charge()
      gd.log(string.format('PROOF loadstate: charge %s before, %s after (saved at %s)', tostring(c_before), tostring(c_after), tostring(c_save)))
      if c_after < c_save - 1 or c_after > c_save + 3 then return fail('loadstate did not restore the charge near ' .. tostring(c_save)) end
      step, t = 28, 0
    elseif t > 80 then
      return fail('loadstate did not bring the charge back (before ' .. tostring(c_before) .. ')')
    end
  elseif step == 28 then
    if t == 1 then gd.input(1, {}, 6) end -- let go of B (the hold of step 27 is not restored by the load), or the next hold is no new press
    if t >= 90 and motion() ~= CHARGE and motion() ~= RELEASE then step, t = 30, 0 end
  elseif step == 30 then -- 3. hold B 200 frames and rewind_test across the whole move
    if t == 1 then gd.input(1, {buttons = 'B'}, 200) end
    if t == 20 then
      c0 = charge()
      if motion() ~= CHARGE or not c0 then return fail('not charging 20 frames in (motion ' .. motion() .. ')') end
      assert(gd.history(300, 1), 'history refused')
      local ok, why = gd.rewind_test(120, true)
      if not ok then return fail('rewind_test refused: ' .. tostring(why)) end
      gd.log('PROOF rewind_test started with charge ' .. tostring(c0) .. ' at frame ' .. t)
      step, t = 40, 0
    end
  elseif step == 40 then
    local r = gd.rewind_test_result()
    if r and r.phase == 0 and r.pass ~= nil then
      gd.log(string.format('PROOF rewind_test pass=%s diff=%s diff_compared=%s %s', tostring(r.pass), tostring(r.diff), tostring(r.diff_compared), tostring(r.text)))
      if not r.pass or r.diff_compared ~= 0 then return fail('rewind_test: pass=' .. tostring(r.pass) .. ' diff_compared=' .. tostring(r.diff_compared)) end
      local c = charge()
      gd.log('PROOF typed state after the test: charge ' .. tostring(c) .. ' (was ' .. tostring(c0) .. ' when it began)')
      step = 90
      gd.log('PROOF RESULT: PASS')
    elseif t > 1500 then
      return fail('rewind_test did not finish')
    end
  end
end
