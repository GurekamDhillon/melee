-- @name: gameplay codes: UCF dashback probe
-- @gameplay: true
-- @rollback_safe: false
-- Port 1 dashes right, then reverses through one frame of a tilt-zone x (-45) before full left: vanilla 1.02 reads the
-- tilt frame as a turn and the reversal is a smash turn; UCF 0.84 Dashback (ftCo_Turn_IASA) makes it a dash.
local FRAMES = 1200
local n = 0
local seq = {
  {40, 0, 0, 0},
  {14, 0, 127, 0}, {1, 0, -45, 0}, {10, 0, -127, 0}, {30, 0, 0, 0},
  {14, 0, -127, 0}, {1, 0, 45, 0}, {10, 0, 127, 0}, {30, 0, 0, 0},
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
