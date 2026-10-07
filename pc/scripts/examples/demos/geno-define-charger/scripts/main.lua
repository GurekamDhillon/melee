-- Offline LAB companion for the Vanilla Charger (enable the separate vanilla-charger fighter folder and geno-lab).
-- Pad-driven: key 1 holds B for 45 frames (charge, then release strikes), 2 holds it for 15 frames (a short charge);
-- the HUD reads the Lua state through gd.fighter_lua. The fighter's Lua is not a gd script: this file only presses buttons and reads.
local last = 'press 1 (hold B 45 frames) or 2 (hold B 15 frames); S save, L load; P pause, N step, R resume'
function on_tick()
  if not gd.match().active then return end
  if gd.key_pressed('1') then gd.input(1, {buttons = 'B'}, 45); last = 'long charge' end
  if gd.key_pressed('2') then gd.input(1, {buttons = 'B'}, 15); last = 'short charge' end
  if gd.key_pressed('S') then gd.savestate(1) end
  if gd.key_pressed('L') then gd.loadstate(1) end
  if gd.key_pressed('P') then gd.pause() end
  if gd.key_pressed('N') then gd.step(1) end
  if gd.key_pressed('R') then gd.resume() end
end
function on_draw()
  local p, l = gd.player(1), gd.fighter_lua(1)
  gd.text(20, 20, 'Vanilla Charger demo: ' .. last)
  if p then gd.text(20, 42, string.format('motion %s (%s)', tostring(p.motion_name), tostring(p.motion))) end
  if l then gd.text(20, 64, string.format('lua state: charge %s, charged %s, faults %d', tostring(l.state.charge), tostring(l.state.charged), l.faults)) end
end
