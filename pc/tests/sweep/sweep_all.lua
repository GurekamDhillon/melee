-- @name: sweep-all
-- @gameplay: true
-- Standalone: six VS cases (3 stages x 2 fighters). The runner selects one
-- case per process through MELEE_LAB_SWEEP_*; this is also the pad script.
local cases = {}
local scene = gd.lab_env("SWEEP_SCENE")
local frames = tonumber(gd.lab_env("SWEEP_FRAMES")) or 180
if scene then
    cases[1] = {scene = scene, mode = tonumber(gd.lab_env("SWEEP_MODE")),
        kind = tonumber(gd.lab_env("SWEEP_KIND")),
        stage = tonumber(gd.lab_env("SWEEP_STAGE"))}
else
    -- External StKind -> internal GrKind, from gr/stage.c::stage_id_map.
    for _, stage in ipairs({{2, 12}, {31, 36}, {32, 37}}) do
        for _, kind in ipairs({1, 22}) do -- Fox, Falco (FighterKind)
            cases[#cases + 1] = {
                scene = ("mode=vs;p1=fk:%d;p2=fox/cpu0;stage=ext:%d;time=0;items=off"):format(kind, stage[1]),
                mode = 2, kind = kind, stage = stage[2]}
        end
    end
end

local section, finished = 0, false
local function check(ok, detail)
    section = section + 1
    gd.log(("TEST sweep-all section %d: %s %s"):format(section, ok and "PASS" or "FAIL", detail))
    return ok
end
local function quit()
    finished = true
    gd.quit()
end
local started = gd.time()
function on_tick()
    -- Lua's deadline handles a stalled task if host ticks still run. The
    -- runner's process deadline also covers a blocked native game thread.
    if not finished and gd.time() - started > 60 * #cases then
        check(false, "Lua wall-clock deadline")
        quit()
    end
end

gd.run(function()
    for i, case in ipairs(cases) do
        local old_epoch = gd.scene().epoch
        -- Single-case processes boot directly into MELEE_SCENE. The standalone
        -- subset changes scene here and must not accept the previous fighters.
        if not scene then gd.scene_launch(case.scene) end
        gd.input(1, {}, 1)
        local entered = false
        for tick = 1, 3600 do
            local s, m, p = gd.scene(), gd.match(), gd.player(1)
            if (scene or s.epoch ~= old_epoch) and s.mode == case.mode
                and m.active and p then
                entered = true
                break
            end
            -- Skip memory-card prompts and 1P intros; never press Start in a match.
            if tick % 60 == 1 and not m.active then
                gd.input(1, s.name == "GS_MEMCARD" and "A" or "START", 2)
            end
            gd.wait(1)
        end
        if not check(entered, "case " .. i .. " entered " .. case.scene) then quit() return end
        local p, m = gd.player(1), gd.match()
        local identity = p.kind == case.kind and (not case.stage or m.stage == case.stage)
        if not check(identity, ("case %d kind=%d stage=%d"):format(i, p.kind, m.stage)) then quit() return end
        local epoch, first, stage = gd.scene().epoch, m.frame, m.stage
        local good, progressed = true, 0
        for n = 1, frames do
            -- Deterministic movement/jumps keep articles, physics and collision live.
            if n % 60 == 1 then gd.input(1, {x = n % 120 < 60 and 35 or -35}, 45) end
            if n % 60 == 46 then gd.input(1, "X", 2) end
            gd.wait(1)
            local now, who = gd.match(), gd.player(1)
            if gd.scene().epoch ~= epoch or not now.active or now.stage ~= stage
                or not who or who.kind ~= case.kind or who.x ~= who.x or who.y ~= who.y then
                good = false
                break
            end
            progressed = now.frame - first
        end
        if not check(good and progressed >= frames, ("case %d advanced=%d required=%d"):format(i, progressed, frames)) then
            quit() return
        end
        gd.input(1, {}, 1)
    end
    quit()
end)
