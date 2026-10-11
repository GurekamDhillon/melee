-- @name: Whole-match .gdlp acceptance pad
-- @gameplay: true
-- @rollback_safe: false
-- Drives a retail Falcon v Fox VS match on FD with a fixed input cycle for FRAMES logic frames, then
-- quits the match (the scene change closes the .gdlp). Used by run.sh in this directory.
local FRAMES = 900
local n = 0
local cycle1 = {
    {40, 0, 100, 0}, {3, 'X', 100, 0}, {30, 0, 100, 0}, {4, 'A', 0, 0}, {25, 0, -100, 0},
    {6, 'B', 0, -127}, {50, 0, 0, 0}, {3, 'X', -100, 0}, {20, 'A', -60, 0}, {60, 0, 0, 0},
}
local cycle2 = {
    {30, 0, -100, 0}, {4, 'X', 0, 0}, {20, 'B', 0, 0}, {50, 0, 80, 0}, {5, 'A', 0, 0},
    {40, 0, 0, 0}, {6, 'Z', 0, 0}, {60, 0, -50, 0},
}
local function at(cycle, frame)
    local total = 0
    for _, p in ipairs(cycle) do total = total + p[1] end
    frame = frame % total
    for _, p in ipairs(cycle) do
        if frame < p[1] then return {buttons = p[2], x = p[3], y = p[4]} end
        frame = frame - p[1]
    end
end
function on_frame_pre()
    local m = gd.match()
    if not (m and m.active) then return end
    n = n + 1
    gd.input(1, at(cycle1, n), 1)
    gd.input(2, at(cycle2, n), 1)
    if n == FRAMES then
        gd.log("GDLPTEST quit at frame " .. n)
        gd.quit()
    end
end
