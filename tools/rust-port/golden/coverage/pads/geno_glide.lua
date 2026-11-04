-- @name: coverage geno_glide
-- @gameplay: true
-- @rollback_safe: false
-- Geno glide coverage (seq format): Kirby (geno-cov) jumps, midair-jumps and holds the jump into a glide, glides, attacks.
local FRAMES = 600
local n = 0
local seq = {
  {150, 0, 0, 0},
  {3, 'X', 0, 0}, {17, 0, 0, 0}, {80, 'X', 0, 0}, {60, 0, 0, 0},
  {3, 'X', 0, 0}, {17, 0, 0, 0}, {80, 'X', 0, 0}, {3, 'A', 0, 0}, {120, 0, 0, 0},
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
