-- Workbench/VS acceptance. The lead sets MELEE_SCRIPT_DATA_DIR=<wb-root>
-- and MELEE_WB_DIR=<wb-root>/pad_script so gd.data_read reads the returned file
-- through the existing per-script sandbox (io/os are deliberately unavailable).
local started, staged, requested, checked = false, false, false, false
local dair
local ticks = 0

local function array_count(json, key)
    local body = assert(json:match('"' .. key .. '":%[(.-)%]'), "missing " .. key)
    local _, n = body:gsub('"joint":', "")
    return n
end

function on_tick()
    if checked then return end
    ticks = ticks + 1
    assert(ticks < 3600, "workbench check timed out")
    if wb == nil then
        if gd.lab_mode() then error("wb absent in a LAB/workbench run") end
        if not gd.player(1) then return end
        assert(wb == nil)
        gd.print("WBCHECK wb=nil")
        checked = true
        gd.quit()
        return
    end
    if not gd.lab_mode() or not gd.player(1) then return end
    if not started then
        assert(gd.paused(), "workbench fighter must start paused")
        assert(select("#", wb.load("goku-xv2", 0)) == 1)
        assert(wb.load("goku-xv2", 0) == "ok")
        assert(not gd.player(2), "workbench seeded an unexpected second fighter")
        local fall
        for _, row in ipairs(gd.motion_list(1)) do
            if row.name == "AttackAirLw" then dair = row.id end
            if row.name == "Fall" then fall = row.id end
        end
        assert(dair and fall, "fighter needs AttackAirLw and Fall motions")
        -- wb.frame deliberately uses lift=0. Stage airborne first using the
        -- existing gd.set_motion lift path, then wait for its boundary.
        assert(gd.set_motion(1, fall, 1, 1.0, 40))
        started, staged = true, true
        return
    end
    if staged then
        assert(gd.player(1).airborne, "Fall/lift was not applied before the aerial request")
        assert(wb.row{motion=dair} == "ok")
        assert(wb.frame{motion=dair, frame=6} == "pending")
        staged, requested = false, true
        return
    end
    if requested then
        local reply = wb.frame_info()
        assert(reply:sub(1, 1) == "@", "frame_info must return a payload path")
        local file = assert(reply:match("[/\\]([^/\\]+)$"), "invalid payload path")
        assert(file == "frame.json")
        local json = assert(gd.data_read(file))
        local frame = tonumber(json:match('"frame":([%-0-9]+)'))
        if frame ~= 6 then return end
        assert(tonumber(json:match('"motion":([%-0-9]+)')) == dair, "snapshot has the wrong motion")
        local hits, bones = array_count(json, "hitboxes"), array_count(json, "bones")
        assert(bones > 0, "snapshot has no fighter joints")
        assert(hits > 0, "AttackAirLw frame 6 has no active hitboxes")
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
        gd.print("WBCHECK hitboxes=" .. hits .. " bones=" .. bones)
        requested, checked = false, true
        gd.quit()
    end
end
