-- state_overlay.lua - a frame-data readout for every fighter: action state, frames in it,
-- animation frame, percent, position, velocity, air/ground and hitlag. Read-only: it only draws,
-- so it is not a gameplay script and works online.
--
--   F2       show / hide
--   console  "state_overlay" toggles it too
--
-- @name: Frame-data overlay
-- @version: 1.0.0
-- @gameplay: false

local shown = true

gd.command("state_overlay", function() shown = not shown end, "show/hide the frame-data overlay")

function on_tick()
  if gd.key_pressed("F2") then shown = not shown end
end

local colors = { 0xFF5C5CFF, 0x5C9CFFFF, 0xFFD24DFF, 0x5CE07AFF, 0xC77DFFFF, 0xFFFFFFFF }

function on_draw()
  if not shown then return end
  local m = gd.match()
  if not m.active then
    gd.text(8, 8, string.format("%s  frame %d", gd.scene().name, gd.frame()), 0xFFFFFFB0, 0.9)
    return
  end
  local list = gd.players()
  local y = 8
  gd.fill(4, 4, 300, 14 + 34 * #list, 0x000000A0)
  gd.text(8, y, string.format("match frame %d%s", m.frame, gd.paused() and "  PAUSED" or ""), 0xFFFFFFFF, 0.9)
  y = y + 12
  for _, p in ipairs(list) do
    local c = colors[p.port] or 0xFFFFFFFF
    gd.text(8, y, string.format("P%d %s  %.0f%%  stocks %d", p.port, p.char_name, p.percent, p.stocks), c, 0.9)
    gd.text(8, y + 11, string.format("  action %3d  frame %3d  anim %5.1f  %s%s",
      p.action, p.action_frame + 1, p.anim_frame, p.airborne and "air" or "ground",
      p.hitlag > 0 and string.format("  hitlag %.0f", p.hitlag) or ""), 0xE0E0E0FF, 0.85)
    gd.text(8, y + 21, string.format("  pos %7.2f %7.2f  vel %6.2f %6.2f", p.x, p.y, p.vx, p.vy),
      0xB0B0B8FF, 0.85)
    y = y + 34
  end
end
