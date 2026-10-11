-- throw_banks_cpu_synctest.lua: no input at all. Five CPU fighters (three Geno throwers of different thrown_as families, two retail
-- victims) fight on one stage under MELEE_SYNCTEST_BENCH; the retail AI grabs and throws on its own, so the throw banks load and
-- evict at frames nobody chose. Waits CFG.FRAMES match frames (default 6000) and quits; the verdict is in the "snap:" lines.
local C = CFG or {}
gd.run(function()
  assert(gd.wait_until(function() return gd.match().active and gd.match().frame > 200 end, 9000), "no match")
  local f0 = gd.match().frame
  assert(gd.wait_until(function() return gd.match().frame - f0 >= (C.FRAMES or 6000) end,
    (C.FRAMES or 6000) + 600), "CPU run did not finish")
  print("TBC DONE frames=" .. tostring(gd.match().frame - f0))
  gd.quit()
end)
