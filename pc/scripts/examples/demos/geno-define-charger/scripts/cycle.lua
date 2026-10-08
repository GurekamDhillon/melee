-- @name: Vanilla Charger cycle (SyncTest driver)
-- @gameplay: true
-- Presses B for 40 frames every 100 frames, so the Lua charge, its release and the strike repeat for the whole run; pair it with
-- MELEE_SYNCTEST_BENCH=1 MELEE_SYNCTEST_CURATED=1 MELEE_SYNCTEST=12 (docs/geno.md section 23). Ends the run at match frame 2400.
local done
function on_frame()
  local m = gd.match()
  if not m.active or done then return end
  if m.frame == 1 then gd.cpu_mode(2, 'stand') end
  if m.frame > 150 and m.frame % 100 == 10 then gd.input(1, {buttons = 'B'}, 40) end
  if m.frame >= 2400 then
    done = true
    local l = gd.fighter_lua(1)
    gd.log(string.format('CYCLE END frame %d lua faults %s charge %s', m.frame, l and tostring(l.faults) or 'n/a', l and tostring(l.state.charge) or 'n/a'))
    gd.quit()
  end
end
