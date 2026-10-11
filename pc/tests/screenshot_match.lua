-- @name: Frozen match screenshot regression
-- @gameplay: true
-- MELEE_TEST_SEED=12345, MELEE_PAD_SCRIPT=<this file>, and:
-- MELEE_SCENE="mode=vs;at=match;p1=mario/human;p2=fox/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off"
-- Use realtime rendering with MELEE_PIPELINE_SKIP=0. Compare
-- scripts-data/pad_script/match.png from two builds with check_screenshot.py --same.
local ticks, frozen, deadline, done = 0, false, nil, false

function on_frame()
    local match = gd.match()
    if match and match.frame and match.frame >= 180 and not frozen then
        gd.pause(true)
        frozen = true
        gd.log("TEST screenshot frozen match frame=" .. match.frame)
    end
end

function on_tick()
    if not frozen or done then return end
    ticks = ticks + 1
    if ticks == 120 then
        local ok, path = gd.screenshot("match")
        assert(ok, "match screenshot request refused")
        deadline = gd.time() + 2
        gd.log("TEST screenshot frozen path=" .. path)
    end
    if deadline and gd.time() >= deadline then
        assert(gd.data_exists("match.png"), "match screenshot file was not created")
        done = true
        gd.quit()
    end
end
