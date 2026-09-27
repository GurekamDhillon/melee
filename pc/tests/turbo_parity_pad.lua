-- turbo_parity.py's inputs and trace. Inputs are keyed to the MATCH frame, not to script frames: the
-- match-load hold (gmscene.c mnLoadScreen) runs script frames with the match clock stopped, and its
-- length depends on how fast pipelines compile, so a gd.wait()-timed script pressed its buttons two
-- match frames later in turbo than in realtime. Every match frame both fighters' gameplay state is
-- logged ("PARITY <frame> ..."). The rollback snapshot hash is not used: it covers all of MEM1 and
-- differs between two realtime runs from frame 0 (boot history).
-- Button masks are the PAD bits (0x0100 A, 0x0200 B, 0x0001 left, 0x0002 right).
local SEG = {
  {30, 0x0000, 0, 0}, {6, 0x0100, 0, 0}, {12, 0x0000, 0, 0}, {6, 0x0200, 0, 0}, {12, 0x0000, 0, 0},
  {6, 0x0002, 80, 0}, {12, 0x0000, 0, 0}, {6, 0x0001, -80, 0}, {120, 0x0000, 0, 0},
}
local START = 60 -- the first segment starts at this match frame
local END = START
for _, s in ipairs(SEG) do END = END + s[1] end

local function segment_at(f)
  local t = START
  for _, s in ipairs(SEG) do
    if f < t + s[1] then return s end
    t = t + s[1]
  end
  return nil
end

local function state(port)
  local p = gd.player(port)
  if not p then return "-" end
  return string.format("%.4f,%.4f,%.4f,%.4f,%.2f,%d,%d,%.2f,%s", p.x or 0, p.y or 0, p.vx or 0, p.vy or 0,
                       p.percent or 0, p.action or -1, p.facing or 0, p.anim_frame or 0, tostring(p.hitlag))
end

local last = -1
function on_frame()
  local m = gd.match()
  if not (m and m.active) or m.frame == last then return end
  last = m.frame
  gd.log(string.format("PARITY %d %s %s", m.frame, state(1), state(2)))
  -- the input for the NEXT match frame (it is read at that frame's PADRead)
  local s = segment_at(m.frame + 1)
  if m.frame + 1 >= START and s ~= nil and (s[2] ~= 0 or s[3] ~= 0) then
    gd.input(1, {buttons = s[2], x = s[3], y = s[4]}, 1)
  end
  if m.frame >= END then gd.quit() end
end
