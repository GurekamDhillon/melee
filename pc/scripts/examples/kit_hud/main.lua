-- kit_hud: a panel in the port's own menu style over the match - the kit's 9-slice frame, its
-- fonts and text roles, its list rows and icons, its palette - all through gd.kit (see
-- docs/scripting.md, "Kit drawing"). It only draws, so it is not a gameplay script and works
-- online.
--
--   F3                show / hide
--   F4                move the highlight down the list (a kit list row in its selected state)
--   console           "kit_hud" toggles it; "kit_training [kit|native]" routes Training's
--                     character and stage select through the kit's screens (or back)
--
-- A mod that ships its own art puts <mod>/ui/*.gxtex next to its scripts/ (and, optionally, a
-- *_ui.json with sizes, tints and palette names): gd.kit.icon("x") then finds <mod>/ui/ico_x.gxtex
-- before the kit's, and gd.kit.panel(x, y, w, h, {prefix = "my_frame"}) draws its own 9-slice.

local shown = true
local sel = 1

gd.command("kit_hud", function() shown = not shown end, "show/hide the kit HUD panel")
gd.command("kit_training", function(arg)
  local now = gd.training_select(arg ~= "" and arg or "kit")
  gd.log("Training's character/stage select: " .. now)
end, "kit_training [kit|native]: Training's character/stage select on the kit or native")

function on_tick()
  if gd.key_pressed("F3") then shown = not shown end
  if gd.key_pressed("F4") then sel = sel + 1 end
end

function on_draw()
  if not shown or not gd.kit.available() then return end
  local players = gd.players()
  local m = gd.match()
  if not m.active then return end

  -- the panel: the kit's frame around a translucent Versus-blue fill
  local x, y, w = 16, 24, 236
  local rows = {}
  for _, p in ipairs(players) do
    rows[#rows + 1] = { label = string.format("P%d  %s", p.port, p.char_name),
                        value = string.format("%d%%", math.floor(p.percent + 0.5)) }
  end
  if sel > #rows then sel = 1 end
  local h = 64 + #rows * gd.kit.row.pitch
  gd.kit.panel(x, y, w, h, { piece = 24 }) -- the art pack's frame, its 64 px corners drawn at 24

  -- a title in the kit's 'label' role, an icon beside it, a caption under it
  gd.kit.icon("training", x + 14, y + 12, 0.25, "gold") -- ico_training: 128 px at 1x, drawn at 32
  gd.kit.text(x + 50, y + 34, "MATCH", "label", "bone")
  gd.kit.text(x + w - 14, y + 34, string.format("%d f", m.frame), "caption", "muted", "right")

  -- the fighters as the kit's list rows; the highlighted one in its selected state
  gd.kit.list(x + 12, y + 48, w - 24, rows, sel)
end
