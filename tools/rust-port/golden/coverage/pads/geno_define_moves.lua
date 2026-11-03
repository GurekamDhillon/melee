-- @name: coverage geno_define_moves
-- @gameplay: true
-- @rollback_safe: false
-- Geno coverage pad (hand-written in the gen_pads.py seq format) for a native define (pc/geno/mods/vanilla-*): each
-- special held and tapped, on the ground and in the air, jab strings, tilts, smashes and aerials.
local FRAMES = 3000
local n = 0
local seq = {
  {150, 0, 0, 0},
  {70, 'B', 0, 0}, {90, 0, 0, 0},                              -- neutral B held (a charge)
  {3, 'B', 0, 0}, {90, 0, 0, 0},                               -- neutral B tapped
  {3, 'B', 100, 0}, {90, 0, 0, 0},                             -- side B
  {3, 'B', 0, -127}, {90, 0, 0, 0},                            -- down B (a counter)
  {3, 'B', 0, 127}, {150, 0, 0, 0},                            -- up B
  {3, 'A', 0, 0}, {8, 0, 0, 0}, {3, 'A', 0, 0}, {8, 0, 0, 0}, {3, 'A', 0, 0}, {60, 0, 0, 0},   -- jab string
  {3, 'A', 80, 0}, {50, 0, 0, 0}, {3, 'A', 0, 80}, {50, 0, 0, 0}, {3, 'A', 0, -80}, {50, 0, 0, 0},   -- tilts
  {20, 0, 127, 0}, {3, 'A', 127, 0}, {60, 0, 0, 0},           -- dash attack
  {3, 'X', 0, 0}, {8, 0, 0, 0}, {3, 'A', 0, 0}, {60, 0, 0, 0}, -- nair
  {3, 'X', 0, 0}, {8, 0, 0, 0}, {3, 'A', 100, 0}, {60, 0, 0, 0}, -- fair
  {3, 'X', 0, 0}, {8, 0, 0, 0}, {3, 'A', 0, -100}, {60, 0, 0, 0}, -- dair
  {3, 'X', 0, 0}, {10, 0, 0, 0}, {50, 'B', 0, 0}, {90, 0, 0, 0},   -- air neutral B held
  {3, 'X', 0, 0}, {10, 0, 0, 0}, {3, 'B', 0, -127}, {90, 0, 0, 0}, -- air down B
  {3, 'Z', 0, 0}, {60, 0, 0, 0},                               -- grab
}
local total = 0
for _, p in ipairs(seq) do total = total + p[1] end
local function at(frame)
  frame = frame % total
  for _, p in ipairs(seq) do
    if frame < p[1] then return {buttons = p[2], x = p[3], y = p[4]} end
    frame = frame - p[1]
  end
end
function on_frame_pre()
    local m = gd.match()
    if not (m and m.active) then return end
    n = n + 1
    gd.input(1, at(n), 2)
    if n == FRAMES then
        gd.log("COVERAGE quit at frame " .. n)
        gd.quit()
    end
end
