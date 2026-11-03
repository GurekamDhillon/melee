-- @name: coverage geno_kirby_bhv
-- @gameplay: true
-- @rollback_safe: false
-- Geno coverage pad (hand-written in the gen_pads.py seq format). Kirby with geno-cov's v5.1 states: tornado, drill rush, cape, glide.
local FRAMES = 2400
local n = 0
local seq = {
  {150, 0, 0, 0},
  {3, 'B', 0, 0}, {120, 0, 0, 0},                              -- tornado (neutral B)
  {3, 'B', 100, 0}, {120, 0, 0, 0},                            -- drill rush (side B)
  {3, 'B', 0, -127}, {90, 0, 0, 0},                            -- cape (down B)
  {3, 'X', 0, 0}, {17, 0, 0, 0}, {45, 'X', 0, 0},              -- jump, midair jump held: glide
  {3, 'A', 0, 0}, {120, 0, 0, 0},                              -- glide attack
  {3, 'X', 0, 0}, {10, 0, 0, 0}, {3, 'B', 0, 0}, {6, 'B', 0, 0}, {6, 'B', 0, 0}, {100, 0, 0, 0},   -- air tornado, mashed
  {3, 'X', 0, 0}, {12, 0, 0, 0}, {3, 'B', -100, 0}, {120, 0, 0, 0},   -- air drill to the left
  {3, 'X', 0, 0}, {12, 0, 0, 0}, {3, 'B', 0, -127}, {100, 0, 0, 0},   -- air cape
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
