-- @name: Vanilla Riposte cycle (SyncTest driver)
-- @gameplay: true
-- Both fighters are pad-driven (scene p2=mario/hu) so every input goes through gd.input, which the bench SyncTest records and replays; nothing here
-- writes game state directly (no teleport, no CPU script). Every 120 frames: P2 walks up to P1, P1 takes the down-B stance, P2 jabs inside the
-- window (a counter), and on every other cycle P1 presses A inside the answer (the follow-up); every fifth cycle P2 holds back (a whiff).
-- Pair it with MELEE_SYNCTEST_BENCH=1 MELEE_SYNCTEST_CURATED=1 MELEE_SYNCTEST=12 (docs/geno.md section 23.7). Ends the run at match frame 1500.
local done
function on_frame()
  local m = gd.match()
  if not m.active or done then return end
  local rel = m.frame - 150
  if rel >= 0 then
    local phase, cycle = rel % 120, math.floor(rel / 120)
    local p1, p2 = gd.player(1), gd.player(2)
    if p1 and p2 and phase < 48 then              -- walk P2 up to P1 (at most 48 frames)
      local d = p1.x - p2.x
      if math.abs(d) > 11 then gd.input(2, {x = d > 0 and 100 or -100}, 1) end
    end
    if phase == 50 then gd.input(1, {buttons = 'B', y = -127}, 30) end
    if phase == 55 and cycle % 5 ~= 4 then gd.input(2, {buttons = 'A'}, 2) end
    if phase == 78 and cycle % 2 == 1 then gd.input(1, {buttons = 'A'}, 2) end
  end
  if m.frame >= 1500 then
    done = true
    local l = gd.fighter_lua(1)
    gd.log(string.format('CYCLE END frame %d lua faults %s streak %s power %s', m.frame, l and tostring(l.faults) or 'n/a', l and tostring(l.state.streak) or 'n/a', l and tostring(l.state.power) or 'n/a'))
    gd.quit()
  end
end
