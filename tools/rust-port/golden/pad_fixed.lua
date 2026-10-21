-- @name: Rust-port golden corpus pad (fixed input cycles)
-- @gameplay: true
-- @rollback_safe: false
-- Drives ports 1 and 2 with fixed, frame-indexed input cycles for FRAMES logic frames, then quits.
-- Used by tools/rust-port/golden/record.sh. Input depends only on the match frame count.
local FRAMES = 1800
local n = 0
local cycle1 = {
    {40, 0, 100, 0}, {3, 'X', 100, 0}, {30, 0, 100, 0}, {4, 'A', 0, 0}, {25, 0, -100, 0},
    {6, 'B', 0, -127}, {50, 0, 0, 0}, {3, 'X', -100, 0}, {20, 'A', -60, 0}, {60, 0, 0, 0},
    {8, 'R', 0, 0}, {30, 0, 0, 127}, {5, 'A', 100, 0},
}
local cycle2 = {
    {30, 0, -100, 0}, {4, 'X', 0, 0}, {20, 'B', 0, 0}, {50, 0, 80, 0}, {5, 'A', 0, 0},
    {40, 0, 0, 0}, {6, 'Z', 0, 0}, {60, 0, -50, 0}, {12, 'L', 0, 0}, {7, 'B', 0, 127},
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
    -- frames=2, not 1: on_frame_pre runs after this frame's PADRead and the hold is consumed at the
    -- frame's end, so a 1-frame hold is never read in turbo (fighters stand still). With 2 each
    -- sample is read on the next frame: a fixed one-frame latency, still deterministic.
    gd.input(1, at(cycle1, n), 2)
    gd.input(2, at(cycle2, n), 2)
    if n % 300 == 0 then
        local p, q = gd.player(1), gd.player(2)
        gd.log(string.format("RUSTGOLD frame %d p1 x %.2f y %.2f %%%s p2 x %.2f y %.2f %%%s", n, p.x, p.y, tostring(p.percent), q.x, q.y, tostring(q.percent)))
    end
    if n == FRAMES then
        gd.log("RUSTGOLD quit at frame " .. n)
        gd.quit()
    end
end
