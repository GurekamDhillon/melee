-- Atlas screens (gd.ui): a list with every value widget, and a grid with cells, an explainer and key hints.
-- F7 opens or closes it. A changes the row, X shows a corner note, Y a dialog, B closes, TAB switches screens.
-- Mouse: hover focuses, left click is A, right click is B, the wheel scrolls. Presentation only: it works online.
-- Valid under the validator: ids under 24 characters, none repeated, a slider with min below max.
local ui = gd.ui
local MODES = { 'Window', 'Borderless', 'Full' }
local state = { sync = true, mode = 1, vol = 40, shown = 'list' }
local LIST, GRID = 'demo_atlas_screen.list', 'demo_atlas_screen.grid'

local function change(cid)
  if cid == 'sync' then state.sync = not state.sync
  elseif cid == 'mode' then state.mode = state.mode % #MODES + 1
  elseif cid == 'vol' then state.vol = (state.vol + 20) % 120 end
end

local function list_desc()
  return { id = LIST, trail = { 'SOLO', title = 'ATLAS DEMO' }, chapter = 1,
    primary = { kind = 'list', items = {
      { id = 'sync', label = 'Sync', value = { kind = 'toggle', on = state.sync } },
      { id = 'mode', label = 'Mode', sub = 'A steps the choice', value = { kind = 'choice', text = MODES[state.mode] } },
      { id = 'vol', label = 'Volume', value = { kind = 'slider', min = 0, max = 100, value = math.min(state.vol, 100) } },
      { id = 'code', label = 'Code', value = { kind = 'text', text = 'ABC-123' } },
      { id = 'closed', label = 'Closed', disabled = true } } },
    explainer = { width = 'normal', provide = function(cid)
      return { kicker = 'ROW', title = cid:upper(), what = 'Press A to change it, X for a note, Y for a dialog.', from = { text = 'Atlas demo' } } end },
    keys = { { 'A', 'Change' }, { 'X', 'Note' }, { 'Y', 'Dialog' }, { 'B', 'Close' } },
    counter = function(cid) return cid end,
    on = {
      accept = function(cid) change(cid); ui.screen(list_desc()) end,
      back = function() return { pop = true } end,
      alt = {
        X = function() ui.note{ text = 'A note from the demo', kind = 'ok', seconds = 3 } end,
        Y = function() ui.dialog{ title = 'DIALOG', text = 'This is the dialog part. A confirms, B cancels.',
          actions = { { 'A', 'Fine' }, { 'B', 'Cancel' } }, on = function(b) gd.log('demo_atlas_screen: dialog ' .. b) end } end } } }
end

local NAMES = { 'Fox', 'Falco', 'Marth', 'Sheik', 'Peach', 'Jigglypuff' }
local function grid_desc()
  local a, b = {}, {}
  for i, n in ipairs(NAMES) do a[i] = { id = 'a:' .. i, name = n, index = i, origin = i == 6 and 'G' or nil, pips = i % 4, flags = { new = i == 2 } } end
  a[4].flags = { locked = true }
  for i = 1, 3 do b[i] = { id = 'b:' .. i, name = 'Cell ' .. i, flags = { empty = i == 3, merge = i == 1 } } end
  return { id = GRID, trail = { 'VERSUS', 'MELEE', title = 'FIGHTERS' }, chapter = 2,
    primary = { kind = 'grid', blocks = {
      { id = 'a', title = 'ROSTER', count = '6 / 6', cols = 6, cells = a },
      { id = 'b', title = 'SPARE', cols = 4, cells = b } } },
    explainer = { width = 'narrow', provide = function(cid, bid)
      return { kicker = bid:upper(), title = cid:upper(), what = 'A cell shows only its name or model. The rule shows here, for the focus.', from = { text = 'Demo data' } } end },
    keys = { { 'A', 'Pick' }, { 'B', 'Close' } },
    on = { accept = function(cid) ui.note{ text = 'Picked ' .. cid, kind = 'info' } end, back = function() return { pop = true } end } }
end

local function top() return ui.state().top end
function on_tick()
  if not ui.available() then return end
  if gd.key_pressed('F7') then
    if top() then ui.close() else ui.screen(list_desc()); ui.open(LIST); state.shown = 'list' end
  end
  if gd.key_pressed('TAB') and top() then
    ui.close()
    if state.shown == 'list' then ui.screen(grid_desc()); ui.open(GRID); state.shown = 'grid'
    else ui.screen(list_desc()); ui.open(LIST); state.shown = 'list' end
  end
end
