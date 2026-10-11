-- @name: Screenshot regression
-- @gameplay: false
-- Run as MELEE_PAD_SCRIPT with MELEE_SCENE="mode=vs;at=css", or an offline
-- match, then check scripts-data/pad_script/scene.png with check_screenshot.py.
-- Requires rendered frames (run.sh without --test). For offscreen proof, use
-- MELEE_TURBO=1 MELEE_TURBO_RENDER=0 (no swapchain presentation).
gd.run(function()
    gd.wait(900)
    local ok, path = gd.screenshot("scene")
    assert(ok, "screenshot request refused")
    gd.log("TEST screenshot queued " .. path)
    -- PNG encoding is asynchronous. Use wall time so turbo cannot quit before
    -- the writer has completed (180 turbo frames can take under 100 ms).
    local deadline = gd.time() + 2
    repeat gd.wait(1) until gd.time() >= deadline
    assert(gd.data_exists("scene.png"), "screenshot file was not created")
    gd.quit()
end)
