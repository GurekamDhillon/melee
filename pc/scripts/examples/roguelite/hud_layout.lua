-- Compact HUD geometry. Pure data: no engine calls, no drawing. Callers decide
-- whether the native stock/percent cluster is safe to replace.
--
-- Policy: the new compact HUD (lives, percent, three ability segments) is only
-- allowed to replace the vanilla stock/percentage cluster when it is actually
-- installed and available. Otherwise the fallback anchors stay inside the native
-- HUD region and keep life and damage readable without covering the fighters.
--
-- Responsive rule: every content scale is clamped so the scaled design box fits
-- the readable safe area. The rail therefore never grows past the screen while
-- its bars keep scaling beyond it. The opponent block prefers the space to the
-- right of the rail, otherwise stacks above it, so it can never cover an ability
-- segment. All returned boxes are pairwise disjoint inside the safe area when
-- an opponent is placed.
--
-- Coordinate space matches the port's fixed 640x480 design units.
local Hud = {version = 2, design_width = 640, design_height = 480}

local ASPECTS = {
  ['16:9'] = {ratio = 16 / 9, side = 20, top = 10, bottom = 12},
  ['4:3'] = {ratio = 4 / 3, side = 14, top = 10, bottom = 12},
  other = {ratio = nil, side = 16, top = 12, bottom = 14},
}
-- Minimum readable sizes for the fallback labels, in design units.
local MIN_LABEL_H, MIN_LABEL_W = 14, 40
local MIN_RAIL_H = 52
local CENTER_BAND = 0.34 -- the middle third stays clear of temporary panels
-- Design sizes of each component (at scale 1, 640x480 units).
local RAIL_W, RAIL_H = 258, 61
local NOTE_W, NOTE_H = 418, 46
local COMPACT_W, COMPACT_H = 588, 23
local OPP_W, OPP_H = 132, 39
local GAP = 12

local function finite(n) return type(n) == 'number' and n == n and math.abs(n) < math.huge end
local function clamp(n, lo, hi) if n < lo then return lo end; if n > hi then return hi end; return n end
local function round(n) return math.floor(n + 0.5) end
local function copy(t) if type(t) ~= 'table' then return t end local o = {} for k, v in pairs(t) do o[k] = copy(v) end return o end
local function rect(x, y, w, h) return {x = x, y = y, w = w, h = h} end
local function overlaps(a, b) return a.x < b.x + b.w and b.x < a.x + a.w and a.y < b.y + b.h and b.y < a.y + a.h end

-- Classify a viewport. Unknown ratios keep the conservative fallback margins.
function Hud.aspect(width, height)
  if not finite(width) or not finite(height) or width <= 0 or height <= 0 then return nil, 'invalid viewport' end
  local ratio = width / height
  if math.abs(ratio - 16 / 9) < 0.02 then return '16:9' end
  if math.abs(ratio - 4 / 3) < 0.02 then return '4:3' end
  return 'other'
end

-- Clamp a candidate rectangle inside the viewport and the readable safe area.
local function fit(box, safe, width, height)
  local x = clamp(box.x, safe.x, math.max(safe.x, safe.x + safe.w - box.w))
  local y = clamp(box.y, safe.y, math.max(safe.y, safe.y + safe.h - box.h))
  if x + box.w > width then box.w = math.max(0, width - x) end
  if y + box.h > height then box.h = math.max(0, height - y) end
  box.x, box.y = round(x), round(y)
  box.w, box.h = round(box.w), round(box.h)
  return box
end

-- layout(opts):
--   width,height  viewport (default 640x480)
--   dpi           UI scale, 1.0 = design units; clamped to [1,2]
--   new_hud       true only when the new compact HUD is actually installed
--   command       true while the command tree is expanded
--   compact_menu  true while a paused workbench owns the subtitle strip
function Hud.layout(opts)
  opts = opts or {}
  local width = opts.width or Hud.design_width
  local height = opts.height or Hud.design_height
  local dpi = clamp(finite(opts.dpi) and opts.dpi or 1, 1, 2)
  local aspect, why = Hud.aspect(width, height)
  if not aspect then return nil, why end
  local spec = ASPECTS[aspect]
  local side = round(spec.side * dpi)
  local top = round(spec.top * dpi)
  local bottom = round(spec.bottom * dpi)
  local safe = rect(side, top, width - side * 2, height - top - bottom)
  if safe.w < 1 or safe.h < 1 then return nil, 'viewport too small for safe area' end

  -- A component scale never exceeds dpi and never lets its scaled design box
  -- exceed the safe width. This is what keeps content inside the rail even when
  -- the rail itself must stop growing at the screen edge.
  local function fit_scale(design_w) return math.min(dpi, safe.w / design_w) end
  local rs = fit_scale(RAIL_W)      -- rail and fallback anchors
  local ns = fit_scale(NOTE_W)      -- normal notification
  local cs = fit_scale(COMPACT_W)   -- paused-workbench strip

  local out = {
    version = Hud.version, width = width, height = height, aspect = aspect,
    dpi = dpi, scale = dpi, content_scale = rs, note_scale = ns, compact_scale = cs,
    safe = safe, replace_vanilla = opts.new_hud == true,
    command_expanded = opts.command == true,
  }

  -- Bottom rail (player lives/percent/three abilities).
  local rail_h = math.max(MIN_RAIL_H, round(RAIL_H * rs))
  local rail_w = math.min(round(RAIL_W * rs), safe.w)
  local rail = rect(safe.x, safe.y + safe.h - rail_h, rail_w, rail_h)
  fit(rail, safe, width, height)
  out.rail = rail

  -- Normal notification box, tall enough for title + detail lines at ns.
  local note_h = math.max(round(NOTE_H * ns), MIN_LABEL_H + round(10 * ns))
  local note_w = math.min(round(NOTE_W * ns), safe.w)
  local note = rect(safe.x + safe.w - note_w, safe.y, note_w, note_h)
  fit(note, safe, width, height)
  out.notification = note

  -- Paused-workbench subtitle strip, always exposed so a caller can switch to a
  -- workbench without rebuilding the layout.
  local comp_h = math.max(MIN_LABEL_H, round(COMPACT_H * cs))
  local comp_w = math.min(round(COMPACT_W * cs), safe.w)
  local comp = rect(safe.x + round(14 * cs), safe.y + round(53 * cs), comp_w, comp_h)
  fit(comp, safe, width, height)
  out.compact_notification = comp
  if opts.compact_menu then out.notification = copy(comp) end

  -- Fallback life/damage anchors: tiny, bottom-left, scaled with the rail.
  local life_w = math.max(MIN_LABEL_W, round(56 * rs))
  local damage_w = math.max(MIN_LABEL_W + 30, round(90 * rs))
  local anchor_h = math.max(MIN_LABEL_H, round(20 * rs))
  local gap = round(6 * rs)
  local anchor_y = rail.y + rail.h - anchor_h - round(4 * rs)
  out.fallback = {
    lives = fit(rect(rail.x + round(6 * rs), anchor_y, life_w, anchor_h), safe, width, height),
    damage = fit(rect(rail.x + round(6 * rs) + life_w + gap, anchor_y, damage_w, anchor_h), safe, width, height),
  }

  -- Expanded command panel is temporary and lives on the side; it may grow with
  -- depth but must leave the center band (fighters, ledges, enemy tells) clear.
  local center_left = safe.x + math.floor(safe.w * (0.5 - CENTER_BAND / 2))
  local side_w = math.min(round(188 * dpi), math.max(0, center_left - safe.x - round(4 * dpi)))
  local cmd_h = math.max(round(96 * dpi), round(120 * dpi))
  out.command = fit(rect(safe.x, safe.y + round(54 * dpi), side_w, cmd_h), safe, width, height)
  out.command.max_width = math.max(0, center_left - safe.x)

  -- Opponent block. Prefer the space to the right of the rail; if it does not
  -- fit, stack it fully above the rail. It therefore never covers a charge bar.
  local os = dpi
  local ow = math.min(round(OPP_W * os), safe.w)
  local gap_o = round(GAP * os)
  local oh = math.min(round(OPP_H * os), safe.h)
  local ox, oy = rail.x + rail.w + gap_o, rail.y + rail.h - oh
  if ox + ow > safe.x + safe.w then
    ox = safe.x
    oy = rail.y - oh - gap_o
    if oy < safe.y then
      oh = math.max(1, rail.y - safe.y - gap_o)
      oy = rail.y - oh - gap_o
      if oy < safe.y then oy = safe.y end
    end
  end
  local opponent = rect(ox, oy, ow, oh)
  if overlaps(opponent, rail) then
    opponent.y = math.max(safe.y, rail.y - opponent.h - gap_o)
  end
  out.opponent = opponent
  out.opponent_scale = os

  return out
end

-- Temporary expansion while the player holds a direction: the panel grows
-- downward but never crosses into the center band or off the safe area.
function Hud.command_expanded(layout, depth)
  if type(layout) ~= 'table' or not layout.command then return nil, 'missing layout' end
  local steps = clamp(tonumber(depth) or 0, 0, 4)
  local base = layout.command
  local grown = copy(base)
  grown.h = math.min(base.h + steps * math.max(MIN_LABEL_H, layout.scale * 16),
    layout.safe.y + layout.safe.h - base.y)
  grown.w = math.min(base.w, layout.command.max_width or base.w)
  return grown
end

-- True when a rectangle stays clear of the center band of the safe area.
function Hud.avoids_center(layout, box)
  if type(layout) ~= 'table' or type(box) ~= 'table' then return false end
  local safe = layout.safe
  local center_left = safe.x + safe.w * (0.5 - CENTER_BAND / 2)
  local center_right = safe.x + safe.w * (0.5 + CENTER_BAND / 2)
  return (box.x + box.w) <= center_left or box.x >= center_right
end

function Hud.copy(layout) return copy(layout) end
return Hud
