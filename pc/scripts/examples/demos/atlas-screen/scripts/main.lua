-- Atlas screens (gd.ui): a list with every value widget, and a grid with cells, an explainer and key hints. Each on.* handler is used once.
-- F7 opens or closes it. A or left/right change a row (on.change), X a corner note, Y a dialog, START a note (on.start),
-- L or R (Tab on the keyboard) switch list and grid (on.page), B closes. Mouse: hover focuses, click is A, right click is B.
-- Presentation only: it works online. Valid under the validator: ids under 24 characters, none repeated, slider min below max.
local ui = gd.ui
local MODES = { 'Window', 'Borderless', 'Full' }
local state = { sync = true, mode = 1, vol = 40, shown = 'list' }
local LIST, GRID = 'demo_atlas_screen.list', 'demo_atlas_screen.grid'
local list_desc, grid_desc

local function show(name)               -- swap the top screen for the other one
  ui.close()
  state.shown = name
  if name == 'list' then ui.screen(list_desc()); ui.open(LIST) else ui.screen(grid_desc()); ui.open(GRID) end
end

local function on_change(id, value)      -- a toggle gives true/false, a slider its number, a choice -1 or +1
  if id == 'sync' then state.sync = value
  elseif id == 'vol' then state.vol = value
  elseif id == 'mode' then state.mode = (state.mode - 1 + value) % #MODES + 1 end
  ui.screen(list_desc())
end

function list_desc()
  return { id = LIST, trail = { 'SOLO', title = 'ATLAS DEMO' }, chapter = 1,
    primary = { kind = 'list', items = {
      { id = 'sync', label = 'Sync', value = { kind = 'toggle', on = state.sync } },
      { id = 'mode', label = 'Mode', sub = 'A or left/right steps the choice', value = { kind = 'choice', text = MODES[state.mode] } },
      { id = 'vol', label = 'Volume', value = { kind = 'slider', min = 0, max = 100, value = state.vol } },
      { id = 'code', label = 'Code', value = { kind = 'text', text = 'ABC-123' } },
      { id = 'closed', label = 'Closed', disabled = true } } },
    explainer = { width = 'normal', provide = function(cid)
      return { kicker = 'ROW', title = cid:upper(), what = 'A or left/right changes it, X for a note, Y for a dialog, R for the grid.', from = { text = 'Atlas demo' } } end },
    keys = { { 'A', 'Change' }, { 'X', 'Note' }, { 'Y', 'Dialog' }, { 'R', 'Grid' }, { 'START', 'Note' }, { 'B', 'Close' } },
    counter = function(cid) return cid end,
    on = {
      change = on_change,
      accept = function(cid) ui.note{ text = 'Pressed A on ' .. cid, kind = 'info' } end,
      back = function() return { pop = true } end,
      page = function() show('grid') end,
      start = function() ui.note{ text = 'START reached on.start', kind = 'warn' } end,
      focus = function(cid) gd.log('demo_atlas_screen: focus ' .. cid) end,
      open = function() gd.log('demo_atlas_screen: list open') end,
      close = function() gd.log('demo_atlas_screen: list closed') end,
      alt = {
        X = function() ui.note{ text = 'A note from the demo', kind = 'ok', seconds = 3 } end,
        Y = function() ui.dialog{ title = 'DIALOG', text = 'This is the dialog part. A confirms, B cancels.',
          actions = { { 'A', 'Fine' }, { 'B', 'Cancel' } }, on = function(b) gd.log('demo_atlas_screen: dialog ' .. b) end } end } } }
end

local NAMES = { 'Fox', 'Falco', 'Marth', 'Sheik', 'Peach', 'Jigglypuff' }
function grid_desc()
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
    keys = { { 'A', 'Pick' }, { 'L', 'List' }, { 'B', 'Close' } },
    on = { accept = function(cid) ui.note{ text = 'Picked ' .. cid, kind = 'info' } end, back = function() return { pop = true } end,
      page = function() show('list') end } }
end

local function top() return ui.state().top end
function on_tick()
  if not ui.available() then return end
  if gd.key_pressed('F7') then
    if top() then ui.close() else show('list') end
  end
end
