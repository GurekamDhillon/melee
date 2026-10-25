-- @name: Rust-port audio probe pad (pause and resume)
-- @gameplay: true
-- @rollback_safe: false
-- Port 1 stands still; START pauses at logic frame 300 and resumes at 420; quits at 700.
local n = 0
function on_frame_pre()
    local m = gd.match()
    if not (m and m.active) then return end
    n = n + 1
    local b = 0
    if n >= 300 and n < 303 then b = 'START' end
    if n >= 420 and n < 423 then b = 'START' end
    gd.input(1, {buttons = b, x = 0, y = 0}, 2)
    gd.input(2, {buttons = 0, x = 0, y = 0}, 2)
    if n == 700 then
        gd.log("RUSTGOLD quit at frame " .. n)
        gd.quit()
    end
end
