-- The Atlas HUD layer (gd.ui.hud, gd.ui.toast, gd.ui.retail_hide, gd.ui.retail): one HUD with a port card for every human port, a match timer, a build strip,
-- a banner (with the A glyph) and a toast. F7 shows or hides the whole HUD and sends a toast; F8 steps the retail mask through
-- {} -> {hud.damage} -> {hud.stock} -> {hud.damage, hud.stock}, one step per press, logging each.
-- A HUD is presentation only: it works online. The retail mask is OFFLINE only (online nothing retail is ever hidden), needs a gameplay script and a match, and the
-- match clock (hud.timer) can never be hidden by a mod. Everything this script sets is released when it unloads or the scene changes.
-- The engine places the parts (title-safe box, clear of the retail percent plates and the timer); this script only names a zone for each.
local ui = gd.ui
local ID = 'demo_atlas_hud.hud'
local STEPS = { {}, { 'hud.damage' }, { 'hud.stock' }, { 'hud.damage', 'hud.stock' } }
local state = { shown = false, step = 1, toasts = 0 }

local function describe()
  local parts_left, parts_right = {}, {}
  local pips, keys = {}, {}
  for i = 1, 6 do pips[i] = { fill = (i <= 4) and 0xF07474FF or 0x161D23FF, ring = (i <= 4) and 0xF2C14EFF or 0x59656FFF } end
  keys[1] = { letter = 'P', rgba = 0xB872F0FF }
  parts_left[1] = { kind = 'strip', pips = pips, keys = keys, wait = '1 waiting' }
  for p = 1, 4 do
    local v = gd.player(p)
    if v and not v.cpu then parts_right[#parts_right + 1] = { kind = 'port_card', port = p } end   -- name, percent and stocks are read when it draws
  end
  return { id = ID, zones = {
    top_left = parts_left,
    top_right = parts_right,
    top_center = { { kind = 'banner', text = 'Collect the drives', button = 'A' } },
    bottom_left = { { kind = 'note', text = 'Merged: Lingering got stronger', seconds = 4 } } } }
end

local function show()
  ui.hud(describe())
  state.toasts = state.toasts + 1
  ui.toast{ zone = 'top_right', title = 'SKYWARD ASSEMBLED #' .. state.toasts, text = 'Your aerials gain Haste for 2 s.', rgba = 0xB872F0FF, seconds = 4 }
  state.shown = true
  gd.log('demo_atlas_hud: HUD shown')
end

function on_tick()
  if not ui.available() then return end
  if gd.key_pressed('F7') then
    if state.shown then ui.hud_clear(); state.shown = false; gd.log('demo_atlas_hud: HUD cleared') else show() end
  end
  if gd.key_pressed('F8') then
    state.step = state.step % #STEPS + 1
    local ok, err = pcall(ui.retail_hide, STEPS[state.step])
    local hidden = ok and ui.retail().hidden or {}
    gd.log(('demo_atlas_hud: retail mask step %d: [%s]%s'):format(state.step, table.concat(STEPS[state.step], ','), ok and (' -> hidden now: [' .. table.concat(hidden, ',') .. ']') or (' refused: ' .. tostring(err))))
  end
end
