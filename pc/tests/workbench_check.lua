-- Workbench/VS acceptance. The lead sets MELEE_SCRIPT_DATA_DIR=<wb-root>
-- and MELEE_WB_DIR=<wb-root>/pad_script so gd.data_read reads the returned file
-- through the existing per-script sandbox (io/os are deliberately unavailable).
local started, staged, requested, checked = false, false, false, false
local dair, wait_id, wait_req, fall_id
local step_frame
local want = 6
local ticks = 0

local function array_count(json, key)
    local body = assert(json:match('"' .. key .. '":%[(.-)%]'), "missing " .. key)
    local _, n = body:gsub('"joint":', "")
    return n
end

function on_tick()
    if checked then return end
    ticks = ticks + 1
    assert(ticks < 20000, "workbench check timed out")
    if wb == nil then
        if gd.lab_mode() then error("wb absent in a LAB/workbench run") end
        if not gd.player(1) then return end
        assert(wb == nil)
        gd.log("WBCHECK wb=nil")
        checked = true
        gd.quit()
        return
    end
    if not gd.lab_mode() or not gd.player(1) then return end
    if not started then
        if not gd.paused() then return end -- the fighter is set up and paused a few hundred frames after load
        assert(select("#", wb.load("goku-xv2", 0)) == 1)
        assert(wb.load("goku-xv2", 0) == "ok")
        assert(not gd.player(2), "workbench seeded an unexpected second fighter")
        local fall
        for _, row in ipairs(gd.motion_list(1)) do
            if row.name == "AttackAirLw" then dair = row.id end
            if row.name == "Fall" then fall = row.id end
            if row.name == "Wait" then wait_id = row.id end
        end
        assert(dair and fall and wait_id, "fighter needs AttackAirLw, Fall and Wait motions")
        -- wb.frame deliberately uses lift=0. Stage airborne first using the
        -- existing gd.set_motion lift path, then wait for its boundary.
        -- Plan T1 Step 4: Entry-squash regression on the wait row, frame 1, before anything else.
        assert(wb.row{motion=wait_id} == "ok")
        assert(wb.frame{motion=wait_id, frame=1} == "pending")
        fall_id = fall
        started, wait_req = true, true
        return
    end
    if wait_req then
        local reply = wb.frame_info()
        if reply:sub(1, 1) ~= "@" then return end
        local file = assert(reply:match("[/\\]([^/\\]+)$"), "invalid wait payload path")
        local json = assert(gd.data_read(file))
        if tonumber(json:match('"frame":([%-0-9]+)')) ~= 1 or tonumber(json:match('"motion":([%-0-9]+)')) ~= wait_id then return end
        local lo, hi
        for y in json:gmatch('"pos":{"x":[%-%.%de]+,"y":([%-%.%de]+),') do
            y = tonumber(y); lo = (lo and lo < y) and lo or y; hi = (hi and hi > y) and hi or y
        end
        assert(lo and hi and hi - lo > 5, "wait/1 bone span too small (Entry squash measured 0.15): " .. tostring(hi and lo and hi - lo))
        gd.log("WBCHECK wait_span=" .. (hi - lo))
        wait_req = false
        assert(gd.set_motion(1, fall_id, 1, 1.0, 40))
        staged = true
        return
    end
    if staged then
        assert(gd.player(1).airborne, "Fall/lift was not applied before the aerial request")
        assert(wb.row{motion=dair} == "ok")
        assert(wb.frame{motion=dair, frame=want} == "pending")
        staged, requested = false, true
        return
    end
    if step_frame then
        local reply = wb.frame_info()
        assert(reply:sub(1, 1) == "@", "step frame_info must return a payload path")
        local file = assert(reply:match("[/\\]([^/\\]+)$"), "invalid step payload path")
        local json = assert(gd.data_read(file))
        local frame = tonumber(json:match('"frame":([%-0-9]+)'))
        if frame == -1 then return end
        assert(frame == step_frame, "wb.step did not advance exactly one frame")
        gd.log("WBCHECK step frame=" .. frame)
        checked = true
        gd.quit()
        return
    end
    if requested then
        local reply = wb.frame_info()
        assert(reply:sub(1, 1) == "@", "frame_info must return a payload path")
        local file = assert(reply:match("[/\\]([^/\\]+)$"), "invalid payload path")
        assert(file == "frame.json")
        local json = assert(gd.data_read(file))
        local frame = tonumber(json:match('"frame":([%-0-9]+)'))
        if frame ~= want then return end
        assert(tonumber(json:match('"motion":([%-0-9]+)')) == dair, "snapshot has the wrong motion")
        local hits, bones = array_count(json, "hitboxes"), select(2, json:gsub('"parent":', ""))
        assert(bones > 0, "snapshot has no fighter joints")
        assert(json:match('"schema":2'), "frame_info must be schema 2")
        do -- Entry-squash regression (span was 0.15): bone heights must span a fighter
            local lo, hi
            for y in json:gmatch('"pos":{"x":[%-%.%de]+,"y":([%-%.%de]+),') do
                y = tonumber(y); lo = (lo and lo < y) and lo or y; hi = (hi and hi > y) and hi or y
            end
            assert(lo and hi and hi - lo > 5, "bone span too small (Entry squash?): " .. tostring(hi and lo and hi - lo))
            local mtx = select(2, json:gsub('"mtx":%[', ""))
            assert(mtx == bones + 1, "every bone and the root need an mtx: " .. mtx .. " vs " .. bones)
        end
        if hits == 0 then -- startup: probe later frames (the move's active window is data, not a constant)
            assert(want < 60, "AttackAirLw has no active hitboxes up to frame 60")
            want = want + 3
            assert(wb.frame{motion=dair, frame=want} == "pending")
            return
        end
        -- Verify live positions against the existing LAB hitbox-position getter.
        local live = gd.hitboxes(1)
        assert(hits == #live, "snapshot/live hitbox count differs")
        local body = json:match('"hitboxes":%[(.-)%]')
        local i = 0
        for slot, x, y, z in body:gmatch('"slot":(%d+).-\"world\":{\"x\":([^,]+),\"y\":([^,]+),\"z\":([^}]+)}') do
            i = i + 1
            local h = assert(live[i])
            assert(tonumber(slot) == h.id)
            assert(math.abs(tonumber(x) - h.x) < 0.02)
            assert(math.abs(tonumber(y) - h.y) < 0.02)
            assert(math.abs(tonumber(z) - h.z) < 0.02)
        end
        assert(i == hits)
        gd.log("WBCHECK hitboxes=" .. hits .. " bones=" .. bones)
        assert(wb.step{n=1} == "pending", "wb.step must defer its frame payload")
        step_frame = frame + 1
        requested = false
    end
end
