-- Offline LAB companion for the Vanilla Striker (enable the separate vanilla-striker fighter folder and geno-lab).
-- Pad-driven: the keys press the Striker's inputs for P1, so each of its eight specials can be seen without a controller.
-- Host controls only; no fighter Lua simulation (that is slice 5).
local MOVES = {
  ['1'] = {'neutral special: hold B (charge), release strikes', {buttons = 'B'}, 45},
  ['2'] = {'side special: lunge', {buttons = 'B', x = 127}, 3},
  ['3'] = {'up special: steerable rise', {buttons = 'B', y = 127}, 3},
  ['4'] = {'down special: counter', {buttons = 'B', y = -127}, 3},
  ['5'] = {'jab chain', {buttons = 'A'}, 2},
  ['6'] = {'forward smash', {cx = 127}, 2},
  ['7'] = {'grab', {buttons = 'Z'}, 2},
}
local last = 'press 1-7 (specials 1-4, then jab, smash, grab); S save, L load; P pause, N step, R resume'
function on_tick()
  if not gd.match().active then return end
  for key, m in pairs(MOVES) do
    if gd.key_pressed(key) then
      gd.input(1, m[2], m[3])
      last = m[1]
      gd.log('striker demo: ' .. m[1])
    end
  end
  if gd.key_pressed('S') then gd.savestate(1) end
  if gd.key_pressed('L') then gd.loadstate(1) end
  if gd.key_pressed('P') then gd.pause() end
  if gd.key_pressed('N') then gd.step(1) end
  if gd.key_pressed('R') then gd.resume() end
end
function on_draw()
  local p = gd.player(1)
  gd.text(20, 20, 'Vanilla Striker demo: ' .. last)
  if p then gd.text(20, 42, string.format('motion %s (%s)', tostring(p.motion_name), tostring(p.motion))) end
end
