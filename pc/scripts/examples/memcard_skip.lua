-- memcard_skip.lua - an INPUT SCRIPT: the Lua form of _build/pad_card.txt. Instead of pressing A on
-- fixed frame counts and hoping the prompt is there, it waits for the game's state: while the
-- memory-card screen is up it presses A, and it stops as soon as the scene changes.
--
--   MELEE_PAD_SCRIPT=memcard_skip.lua   (input scripts may press buttons: they count as gameplay)
--   or in the console: load examples/memcard_skip
--
-- @name: Memory-card prompt skipper
-- @version: 1.0.0
-- @gameplay: true

gd.run(function()
  -- the boot scene comes first; wait (at most 10 s) for the memory-card screen
  if not gd.wait_until(function() return gd.scene().name == "GS_MEMCARD" end, 600) then
    gd.log("memcard_skip: no memory-card prompt this boot")
    return
  end
  local presses = 0
  while gd.scene().name == "GS_MEMCARD" and presses < 20 do
    gd.press(1, "A", 4) -- hold A for 4 frames
    gd.wait(20)         -- then give the prompt time to react
    presses = presses + 1
  end
  gd.log(string.format("memcard_skip: through after %d press(es), now in %s", presses, gd.scene().name))
end)
