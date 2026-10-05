-- Offline LAB companion for the Vanilla Caster (enable the separate vanilla-caster fighter folder and geno-lab).
-- Key 1 presses the Caster's neutral special for P1: it throws CasterBolt, an article of its own with its own effect and sound names.
-- Host controls only; no fighter Lua simulation (that is slice 5).
local last = 'press 1: neutral special (the Caster throws its own bolt); S save, L load; P pause, N step, R resume'
function on_tick()
  if not gd.match().active then return end
  if gd.key_pressed('1') then gd.input(1, {buttons = 'B'}, 3); last = 'neutral special' ; gd.log('caster demo: neutral special') end
  if gd.key_pressed('S') then gd.savestate(1) end
  if gd.key_pressed('L') then gd.loadstate(1) end
  if gd.key_pressed('P') then gd.pause() end
  if gd.key_pressed('N') then gd.step(1) end
  if gd.key_pressed('R') then gd.resume() end
end
function on_draw()
  local p = gd.player(1)
  gd.text(20, 20, 'Vanilla Caster demo: ' .. last)
  gd.text(20, 42, string.format('live articles: %d', #gd.items()))
  if p then gd.text(20, 64, string.format('motion %s', tostring(p.motion_name))) end
end
