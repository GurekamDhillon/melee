-- @name: Rust-port netplay pad (fixed input cycles, one role per client)
-- @gameplay: true
-- @rollback_safe: false
-- The pad_fixed.lua cycles for a scripted NETPLAY client: MELEE_NP_ROLE=host plays cycle1 (port 1,
-- Falcon), MELEE_NP_ROLE=guest plays cycle2 (the guest's controller is also gd.input port 1 on its
-- own machine). Input depends only on the match frame count; quits after FRAMES logic frames.
local FRAMES = tonumber(os.getenv and os.getenv("MELEE_NP_FRAMES") or "") or 1800
local role = (os.getenv and os.getenv("MELEE_NP_ROLE")) or "host"
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
    gd.input(1, at(role == "guest" and cycle2 or cycle1, n), 2)
    if n % 300 == 0 then gd.log("RUSTNP " .. role .. " frame " .. n) end
    if n == FRAMES + 20 then
        gd.log("RUSTNP quit at frame " .. n)
        gd.quit()
    end
end
