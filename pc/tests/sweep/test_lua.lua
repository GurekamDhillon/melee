-- Host-only API stub: executes the shipped Lua state machine, never Melee.
local source = assert(arg[1], "pass path to sweep_all.lua")
local function exercise(wrong_stage, stalled)
    local logs, epoch, frame, kind, stage, task, quit = {}, 0, 0, 1, 12, nil, false
    local env = setmetatable({}, {__index = _G})
    local gd = {}
    env.gd = gd
    function gd.lab_env(key) if key == "SWEEP_FRAMES" then return "3" end end
    function gd.scene() return {epoch = epoch, mode = 2, name = "GS_VS"} end
    function gd.match() return {active = true, frame = frame, stage = stage} end
    function gd.player() return {kind = kind, x = 0, y = 0} end
    function gd.time() return 0 end
    function gd.scene_launch(text)
        epoch, frame = epoch + 1, 0
        kind = tonumber(text:match("p1=fk:(%d+)"))
        stage = ({[2] = 12, [31] = 36, [32] = 37})[tonumber(text:match("stage=ext:(%d+)"))]
        if wrong_stage then stage = 999 end
    end
    function gd.input() end
    function gd.log(line) logs[#logs + 1] = line end
    function gd.quit() quit = true end
    function gd.run(fn) task = coroutine.create(fn) end
    function gd.wait() coroutine.yield() end
    assert(loadfile(source, "t", env))()
    for _ = 1, 100 do
        if coroutine.status(task) == "dead" then break end
        if not stalled then frame = frame + 1 end
        local ok, err = coroutine.resume(task)
        assert(ok, err)
    end
    assert(quit, "fixture never called gd.quit")
    return logs
end

local logs = exercise(false, false)
assert(#logs == 18, "six cases must emit three checks each")
for n, log in ipairs(logs) do
    assert(log:find("TEST sweep-all section " .. n .. ": PASS", 1, true), log)
end
assert(exercise(true, false)[2]:find(": FAIL", 1, true), "wrong stage falsely passed")
assert(exercise(false, true)[3]:find(": FAIL", 1, true), "stalled logic falsely passed")
print("sweep-all Lua host checks: PASS (six-case flow, wrong-stage rejection, stalled-frame rejection)")
