-- Offline LAB companion for the Vanilla Riposte (enable the separate vanilla-riposte fighter folder and geno-lab).
-- Pad-driven: key 1 holds down+B for 30 frames (a full stance), 2 holds it for 12 (let go early: the stance cancels), 3 makes the CPU
-- (P2) jab once; 4 puts the CPU back in front of you. In the stance, a CPU jab inside frames 4-24 is countered and answered; press A in
-- the answer's frames 6-16 for the follow-up. The HUD reads the Lua state through gd.fighter_lua. The fighter's Lua is not a gd script.
local last = 'press 1 (stance), 2 (short stance), 3 (CPU jabs), 4 (CPU in front); S save, L load; P pause, N step, R resume'
local started
function on_tick()
  if not gd.match().active then return end
  if not started then started = true; gd.cpu_mode(2, 'script') end
  if gd.key_pressed('1') then gd.input(1, {buttons = 'B', y = -127}, 30); last = 'full stance' end
  if gd.key_pressed('2') then gd.input(1, {buttons = 'B', y = -127}, 12); last = 'short stance' end
  if gd.key_pressed('3') then gd.cpu_pad(2, {buttons = 'A'}, 2); last = 'CPU jab' end
  if gd.key_pressed('4') then
    local p = gd.player(1)
    if p then gd.teleport(2, p.x + 9 * p.facing, p.y) end
  end
  if gd.key_pressed('S') then gd.savestate(1) end
  if gd.key_pressed('L') then gd.loadstate(1) end
  if gd.key_pressed('P') then gd.pause() end
  if gd.key_pressed('N') then gd.step(1) end
  if gd.key_pressed('R') then gd.resume() end
end
function on_draw()
  local p, l = gd.player(1), gd.fighter_lua(1)
  gd.text(20, 20, 'Vanilla Riposte demo: ' .. last)
  if p then gd.text(20, 42, string.format('motion %s (%s) frame %s', tostring(p.motion_name), tostring(p.action), tostring(p.action_frame))) end
  if l then gd.text(20, 64, string.format('lua state: power %.2f, streak %s, follow %s, faults %d', l.state.power or 0, tostring(l.state.streak), tostring(l.state.follow), l.faults)) end
end
