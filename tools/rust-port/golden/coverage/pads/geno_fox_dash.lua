-- @name: coverage geno_fox_dash
-- @gameplay: true
-- @rollback_safe: false
-- Geno coverage pad (hand-written in the gen_pads.py seq format). Fox with geno-cov's multi-dash side special and 4 jumps.
local FRAMES = 2400
local n = 0
local seq = {
  {120, 0, 0, 0},
  {3, 'B', 100, 0}, {10, 0, 0, 0}, {3, 'B', 100, 0}, {10, 0, 0, 0}, {3, 'B', 0, 100}, {90, 0, 0, 0},  -- multi-dash chain
  {3, 'X', 0, 0}, {15, 0, 0, 0}, {3, 'X', 0, 0}, {10, 0, 0, 0}, {3, 'X', 0, 0}, {10, 0, 0, 0}, {3, 'X', 0, 0}, {60, 0, 0, 0},  -- 4 jumps
  {3, 'B', -100, 0}, {8, 0, 0, 0}, {3, 'B', -100, 0}, {120, 0, 0, 0},
  {20, 0, 60, 0}, {3, 'A', 0, 0}, {40, 0, 0, 0}
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
