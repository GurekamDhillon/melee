-- MELEE_PAD_SCRIPT fixture; see boss-end-report.md. No boss hook or mod here:
-- also exercise the original state-7 finish, without the hook's state-8 wait.
-- Pass additionally requires the three native scene log milestones in the report.
local done = false
local function finish(ok, why)
    if done then return end
    done = true
    gd.log("BOSSEND " .. (ok and "PASS " or "FAIL ") .. why)
    gd.quit()
end

gd.run(function()
    local mh
    if not gd.wait_until(function()
        if gd.scene().name ~= "GS_VS" then return false end
        for p = 2, 4 do
            local q = gd.player(p)
            if q and q.kind == 0x1B then mh = p return true end
        end
        return false
    end, 6000) then
        finish(false, "Master Hand did not load")
        return
    end
    gd.wait(90)
    -- Matches the saved vanilla-disc, difficulty=0 reproduction (300 max HP).
    gd.set_damage(mh, 299)
    gd.wait(30)
    if not gd.hit(mh, {damage = 20, angle = 90, kbg = 50, bkb = 40}) then
        finish(false, "environment hit rejected")
        return
    end
    gd.log("BOSSEND environment KO submitted; waiting for native results log")
    -- Covers the death presentation and the example mod's 20-second timeout.
    -- GS_VS and match.active remain true on the results overlay: do not fail on them.
    gd.wait(3600)
    if gd.scene().name ~= "GS_VS" then
        finish(false, "left VS before confirming results")
        return
    end
    gd.log("BOSSEND confirm results with Start")
    gd.press(1, "START", 1)
    if not gd.wait_until(function()
        return gd.scene().name == "GS_REGEND_TOYFALL"
    end, 1200) then
        finish(false, "no Classic trophy fall after Start")
        return
    end
    finish(true, "Classic trophy fall reached after Start")
end)
