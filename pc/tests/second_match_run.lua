-- Regression for the "second match on a different stage" crash (2026-10-10): leave a LAB match to the stage select, pick another stage, start again.
-- Before the lbdvd.c fix the second load died with "Cannot find symbol ftDataMario." (lbarchive.c:87), because a preloaded, already initialised
-- archive had been moved by heap compaction and still pointed at its old copy. Passes when the second match is up (prints SECONDMATCH PASS).
-- Run (offscreen, any two retail fighters or a Geno define beside its base):
--   MELEE_SCENE="mode=lab;p1=mario/hu;p2=luigi/hu;stage=fd;items=off;time=0" MELEE_PAD_SCRIPT=<this file with CFG={dx=-127} prepended> tools/port/run.sh <name>
-- (workspace helper: ports/goku-xv2/tools/kame_run.py second_match SECONDMATCH --p1 mario/hu --vs luigi/hu --cfg dx=-127; the stage cursor moves with dx.)
local C = CFG or {}
gd.run(function()
  local ok, why = pcall(function()
    assert(gd.wait_until(function() return gd.match().active and gd.match().frame > 60 end, 6000), 'no first match')
    local r, reason = gd.lab_leave('sss'); gd.log('SECONDMATCH leave ' .. tostring(r) .. ' ' .. tostring(reason))
    gd.wait(90)
    if C.dx then gd.input(1, {x = tonumber(C.dx)}, 6); gd.wait(20) end
    for i = 1, 40 do gd.input(1, {buttons = 'A'}, 2); gd.wait(30); if gd.match().active then break end end
    assert(gd.wait_until(function() return gd.match().active and gd.match().frame > 60 end, 3000), 'no second match')
    gd.log('SECONDMATCH PASS second match up, frame ' .. tostring(gd.match().frame))
  end)
  if not ok then gd.log('SECONDMATCH FAIL ' .. tostring(why)) end
  gd.quit()
end)
