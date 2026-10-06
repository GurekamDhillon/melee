-- cd "$GW_MELEE" && lua pc/tests/atlas_ui_stub_test.lua   (it also runs from a directory that holds melee/)
local prefix = io.open('pc/tests/atlas_ui_stub.lua') and '' or 'melee/'
local Stub = dofile(prefix .. 'pc/tests/atlas_ui_stub.lua')
local fails, count = 0, 0
local function check(cond, msg) count = count + 1; if not cond then fails = fails + 1; print('FAIL: ' .. msg) end end
local function raises(f, part, msg) local ok, e = pcall(f); check(not ok and tostring(e):find(part, 1, true), msg .. ' (got ' .. tostring(e) .. ')') end

local L = Stub.limits
check(L.blocks == 6 and L.cells == 12 and L.items == 32 and L.keys == 6, 'limits are read from gw_ui_screen.h')
check(L['with'] == 4, 'the with limit is read from the header too')

local function cell(id, extra) local c = { id = id, name = id }; for k, v in pairs(extra or {}) do c[k] = v end; return c end
local function grid(id, bags)
  return { id = id, primary = { kind = 'grid', blocks = {
    { id = 'eq', title = 'EQUIPPED', cols = 6, cells = { cell('eq:1'), cell('eq:2'), cell('eq:3', { flags = { locked = true } }) } },
    { id = 'bag', title = 'BAG', cols = 4, cells = bags or { cell('bag:1'), cell('bag:2'), cell('bag:3') } } } },
    keys = { { 'A', 'Pick' }, { 'B', 'Close' } } }
end

local ui = Stub.new()
check(ui.available() == true, 'available')
raises(function() ui.screen({}) end, 'no id', 'an id is required')
raises(function() ui.screen({ id = 'x.a', primary = { kind = 'tiles' } }) end, 'not supported', 'only grid and list')
raises(function() ui.screen({ id = 'x.a', primary = { kind = 'grid', blocks = {} } }) end, 'blocks', 'a grid needs blocks')
raises(function() ui.screen({ id = 'x.a', primary = { kind = 'grid', blocks = { { id = 'b', cols = 2, cells = { cell('c'), cell('c') } } } } }) end, 'duplicate', 'cell ids are unique')
raises(function() ui.screen({ id = 'x.a', primary = { kind = 'grid', blocks = { { id = 'b', cols = 20, cells = {} } } } }) end, 'cols', 'cols is bounded')
local d = grid('x.a'); d.keys[#d.keys + 1] = { 'Q', 'No' }
raises(function() ui.screen(d) end, 'button', 'key buttons are checked')
local seven = { id = 'x.a', primary = { kind = 'grid', blocks = {} } }
for i = 1, 7 do seven.primary.blocks[i] = { id = 'b' .. i, cols = 1, cells = {} } end
raises(function() ui.screen(seven) end, 'blocks', 'at most six blocks')

-- the rules the C side gained in its review rounds: block ids are unique, ids fit their buffer, a slider has a range
local long = string.rep('i', 24)
raises(function() ui.screen({ id = 'x.a', primary = { kind = 'grid', blocks = { { id = 'b', cols = 1, cells = {} }, { id = 'b', cols = 1, cells = {} } } } }) end, 'duplicate block', 'block ids are unique')
raises(function() ui.screen({ id = 'x.a', primary = { kind = 'grid', blocks = { { id = long, cols = 1, cells = {} } } } }) end, 'too long', 'a block id fits its buffer')
raises(function() ui.screen({ id = 'x.a', primary = { kind = 'grid', blocks = { { id = 'b', cols = 1, cells = { cell(long) } } } } }) end, 'too long', 'a cell id fits its buffer')
raises(function() ui.screen({ id = 'x.a', primary = { kind = 'list', items = { { id = long, label = 'L' } } } }) end, 'too long', 'an item id fits its buffer')
raises(function() ui.screen({ id = 'x.a', primary = { kind = 'list', items = { { id = 's', label = 'S', value = { kind = 'slider', min = 5, max = 5 } } } } }) end, 'slider min', 'a slider needs min below max')
check(ui.screen({ id = 'x.sl', primary = { kind = 'list', items = { { id = 's', label = 'S', value = { kind = 'slider', min = 0, max = 10, value = 4 } } } } }), 'a slider with a range registers')
ui.screen({ id = 'x.sl2', primary = { kind = 'list', items = { { id = 's', label = 'S', value = { kind = 'slider' } } } } })
ui.close('x.sl'); ui.close('x.sl2')

check(ui.screen(grid('x.a')), 'a good grid registers')
local c, b = ui.focus('x.a'); check(c == 'eq:1' and b == 'eq', 'focus starts on the first cell')
ui.open('x.a'); check(ui.state().top == 'x.a' and ui.state().depth == 1, 'open pushes')

-- focus is kept by id across a re-registration; a gone cell clamps to the same block; a gone block falls to the first cell
ui.set_focus('x.a', 'bag', 'bag:3')
ui.screen(grid('x.a')); c = ui.focus('x.a'); check(c == 'bag:3', 'same id: same cell')
ui.screen(grid('x.a', { cell('bag:1'), cell('bag:2') })); c = ui.focus('x.a'); check(c == 'bag:2', 'gone: clamped to the same block, got ' .. tostring(c))
local g2 = grid('x.a'); table.remove(g2.primary.blocks, 2); ui.screen(g2); c = ui.focus('x.a'); check(c == 'eq:1', 'block gone: the first cell, got ' .. tostring(c))

-- the engine's part: on.focus, the provider and key/counter functions, dispatch, disabled cells
local log = {}
local e = grid('x.b')
e.explainer = { width = 'narrow', provide = function(cid, bid) return { title = cid .. '@' .. bid } end }
e.keys = { { 'A', function(cid) return cid == 'eq:1' and 'First' or 'Other' end }, { 'X', 'Hidden', when = function(cid) return cid ~= 'eq:1' end }, { 'B', 'Close' } }
e.counter = function(cid) return 'at ' .. cid end
e.primary.blocks[1].cells[2].flags = { disabled = true }
e.on = { focus = function(cid, bid) log[#log + 1] = 'focus ' .. cid .. ' ' .. bid end, accept = function(cid, bid) log[#log + 1] = 'accept ' .. cid; return nil end,
         back = function() log[#log + 1] = 'back' end, alt = { X = function(cid) log[#log + 1] = 'x ' .. cid end } }
ui.screen(e)
local v = ui.refresh('x.b')
check(v.explainer.title == 'eq:1@eq' and v.counter == 'at eq:1', 'provider and counter get the focused ids')
check(#v.keys == 2 and v.keys[1][2] == 'First' and v.keys[2][1] == 'B', 'a key whose when() is false is hidden')
ui.engine_focus('x.b', 'bag', 'bag:2')
check(log[1] == 'focus bag:2 bag' and ui.views['x.b'].keys[1][2] == 'Other' and #ui.views['x.b'].keys == 3, 'engine_focus runs on.focus then refreshes the view')
check(ui.engine_press('x.b', 'accept') and log[2] == 'accept bag:2', 'accept reaches the handler with the cell id')
ui.engine_focus('x.b', 'eq', 'eq:2')
check(not ui.engine_press('x.b', 'accept'), 'a disabled cell is focusable but never fires'); check(#log == 3, 'and nothing was logged for it')
check(ui.engine_press('x.b', 'x') and log[4] == 'x eq:2', 'alt X reaches its handler'); check(ui.engine_press('x.b', 'back'), 'back')
check(not ui.engine_press('x.b', 'y'), 'no handler: nothing')
ui.feed('x.b', 'down'); check(ui.fed[1][1] == 'x.b' and ui.fed[1][2] == 'down', 'feed is recorded')
ui.note({ text = 'hi' }); ui.dialog({ title = 'T' }); check(#ui.notes == 1 and #ui.dialogs == 1, 'notes and dialogs are recorded')
ui.close('x.a'); check(ui.state().depth == 0, 'close removes the screen')
local off = Stub.new({ available = false }); check(select(1, off.available()) == false, 'an unavailable stub says so')
print(('atlas ui stub: %d checks, %d failed'):format(count, fails))
os.exit(fails == 0 and 0 or 1)
