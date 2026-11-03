#!/usr/bin/env python3
"""Make the C build's Lua driver for a menu proof run from the same inputs the Rust run uses.

  gen_lua.py PADSCRIPT.txt AGES OUT.lua [--start SCENE_NAME]

PADSCRIPT.txt: the `melee play` text pad script (`N BUTTONS_HEX X Y` per line, port 1).
AGES: comma list of scene ages (the Rust tick counter: the first tick in the scene is age 1; the Lua sees the scene one frame later, so its shot of age N is taken after N waits and labelled N).
The Lua first presses A through the memory-card prompt (not part of the timeline), waits for the
title scene (GS_TITLE), and from there plays the script one logic frame per script tick, taking
`<label>_<age>` screenshots. gd.screenshot files land in scripts-data/pad_script/.
"""
import sys
script, ages, out = sys.argv[1:4]
start = "GS_TITLE"
if "--start" in sys.argv:
    start = sys.argv[sys.argv.index("--start") + 1]
rows = []
for ln in open(script):
    ln = ln.split("#")[0].split()
    if len(ln) >= 4:
        rows.append((int(ln[0]), int(ln[1], 16), int(ln[2]), int(ln[3])))
want = sorted(int(a) for a in ages.split(","))
lua = ["-- @name: menu proof driver", "-- @gameplay: true", "local rows = {"]
lua += ["  {%d, %d, %d, %d}," % r for r in rows]
lua += ["}", "local want = {" + ",".join(map(str, want)) + "}",
"""gd.run(function()
  gd.wait(30)
  local n = 0
  while gd.scene().name ~= "@START@" and n < 3000 do
    -- A held 6 frames, released 6; the scene is polled every frame
    if n % 12 == 0 then gd.input(1, {buttons = 'A', x = 0, y = 0}, 6) end
    if n % 12 == 6 then gd.input(1, {buttons = 0, x = 0, y = 0}, 6) end
    gd.wait(1)
    n = n + 1
  end
  gd.log("MENUPROOF scene " .. gd.scene().name .. " at frame " .. gd.frame())
  local t0 = gd.frame()
  -- expand the script to one entry per tick; the row of tick k is set at Lua age k-LUA_LEAD (see the end of this file; the old note:
  -- with a START at the title: the menu's first frame lines up with the Rust tick when the input of tick k is
  -- set at Lua age k-2, measured on the title only)
  local ticks = {}
  for _, r in ipairs(rows) do
    for k = 1, r[1] do ticks[#ticks + 1] = r end
  end
  local last = want[#want] + 1
  local wi = 1
  for age = 1, last do
    local r = ticks[age + @LEAD@]
    if r then gd.input(1, {buttons = r[2], x = r[3], y = r[4]}, 1) else gd.input(1, {buttons = 0, x = 0, y = 0}, 1) end
    gd.wait(1)
    while wi <= #want and want[wi] == age + 1 do
      gd.screenshot(string.format("c_%04d", age + 1))
      gd.log("MENUPROOF shot " .. (age + 1) .. " scene " .. gd.scene().name)
      wi = wi + 1
    end
  end
  gd.wait(40)
  gd.quit()
end)""".replace("@START@", start)]
# LUA_LEAD: how many ticks ahead of the Lua age a script row is fed. 1 lines the C build's input up with melee play's ticks:
# the CSS cursor, the SSS cursor and every scene cut are frame-exact (measured 2026-11-03; the old 2 was a title-only fit).
import os
open(out, "w").write("\n".join(lua).replace("@LEAD@", os.environ.get("LUA_LEAD", "1")))
