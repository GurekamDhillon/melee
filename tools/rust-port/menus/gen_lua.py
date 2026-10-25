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
  local age = 0
  local wi = 1
  for _, r in ipairs(rows) do
    for k = 1, r[1] do
      age = age + 1
      gd.input(1, {buttons = r[2], x = r[3], y = r[4]}, 1)
      gd.wait(1)
      while wi <= #want and want[wi] == age + 1 do
        gd.screenshot(string.format("c_%04d", age + 1))
        gd.log("MENUPROOF shot " .. (age + 1) .. " scene " .. gd.scene().name)
        wi = wi + 1
      end
      if wi > #want then break end
    end
    if wi > #want then break end
  end
  gd.wait(40)
  gd.quit()
end)""".replace("@START@", start)]
open(out, "w").write("\n".join(lua))
