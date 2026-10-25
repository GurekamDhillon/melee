-- @name: Rust-port golden corpus pad (CPU fights: no human input)
-- @gameplay: true
-- @rollback_safe: false
-- Sends no input; the CPUs fight. Quits after 4600 logic frames (past a one-minute time limit).
local FRAMES = 4600
local n = 0
function on_frame_pre()
    local m = gd.match()
    if not (m and m.active) then return end
    n = n + 1
    if n == FRAMES then
        gd.log("RUSTGOLD quit at frame " .. n)
        gd.quit()
    end
end
