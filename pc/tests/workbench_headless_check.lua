-- Workbench data check that runs identically headless (MELEE_WB_HEADLESS=1) and windowed.
-- Logs one "WBJSON <name> <frame> <json>" line per sample so a headless and a windowed run can be
-- diffed (tools/workbench/frame_info_diff.py), then "HEADLESS_CHECK OK".
-- Env: MELEE_SCRIPT_DATA_DIR=<wb-root>, MELEE_WB_DIR=<wb-root>/pad_script (see workbench_check.lua).
local started, checked = false, false
local items, idx, fall, ticks = nil, 0, nil, 0
local cur, staged, requested

local function build_items()
    local ids = {}
    for _, row in ipairs(gd.motion_list(1)) do ids[row.name] = row.id end
    assert(ids.Wait and ids.Attack11 and ids.AttackAirLw and ids.Fall, "fighter needs Wait/Attack11/AttackAirLw/Fall")
    fall = ids.Fall
    items = {}
    for _, spec in ipairs{{"wait", ids.Wait, false}, {"jab", ids.Attack11, false}, {"dair", ids.AttackAirLw, true}} do
        for _, f in ipairs{1, 6, 12} do items[#items + 1] = {name = spec[1], motion = spec[2], aerial = spec[3], frame = f} end
    end
end

local function span(json)
    local lo, hi
    for y in json:gmatch('"pos":{"x":[%-%.%de]+,"y":([%-%.%de]+),') do
        y = tonumber(y); lo = (lo and lo < y) and lo or y; hi = (hi and hi > y) and hi or y
    end
    return hi - lo
end

function on_tick()
    if checked then return end
    ticks = ticks + 1
    assert(ticks < 40000, "headless check timed out")
    if wb == nil or not gd.lab_mode() or not gd.player(1) then return end
    if not started then
        if not gd.paused() then return end
        assert(wb.load("goku-xv2", 0) == "ok")
        build_items()
        local ok, err = pcall(wb.render, {})
        started = true
        gd.log("HEADLESS_CHECK render_pcall=" .. tostring(ok) .. " " .. tostring(err))
        return
    end
    if not cur then
        idx = idx + 1
        if idx > #items then
            gd.log("HEADLESS_CHECK OK")
            checked = true
            gd.quit()
            return
        end
        cur = items[idx]
        if cur.aerial then
            assert(gd.set_motion(1, fall, 1, 1.0, 40))
            staged = true
            return
        end
    end
    if staged then
        staged = false
        assert(wb.row{motion = cur.motion} == "ok")
        assert(wb.frame{motion = cur.motion, frame = cur.frame} == "pending")
        requested = true
        return
    end
    if not requested then
        assert(wb.row{motion = cur.motion} == "ok")
        assert(wb.frame{motion = cur.motion, frame = cur.frame} == "pending")
        requested = true
        return
    end
    local reply = wb.frame_info()
    if reply:sub(1, 1) ~= "@" then return end
    local file = assert(reply:match("[/\\]([^/\\]+)$"))
    local json = assert(gd.data_read(file))
    local frame = tonumber(json:match('"frame":([%-0-9]+)'))
    local motion = tonumber(json:match('"motion":([%-0-9]+)'))
    if frame ~= cur.frame or motion ~= cur.motion then return end
    assert(json:match('"schema":2'), "schema 2 expected")
    local bones = select(2, json:gsub('"parent":', ""))
    assert(bones > 20, "too few bones: " .. bones)
    assert(span(json) > 5, "bone span " .. span(json))
    gd.log("WBJSON " .. cur.name .. " " .. cur.frame .. " " .. json)
    cur, requested = nil, false
end
