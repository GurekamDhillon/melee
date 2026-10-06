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
-- ---- fix round 1: parity with gw_ui_screen.c as it is now ---------------------------------------------------------------
local function list1(extra) local d = { id = 'x.l', primary = { kind = 'list', items = { { id = 'a', label = 'A' } } } }; for k, v in pairs(extra or {}) do d[k] = v end; return d end
raises(function() ui.screen(list1({ id = 'x.' .. string.rep('i', 46) })) end, 'too long', 'a screen id over 47 characters')
check(ui.screen(list1({ id = 'x.' .. string.rep('i', 45) })), 'a 47 character screen id registers')
raises(function() ui.screen(list1({ chapter = 6 })) end, 'chapter is 0 to 5', 'chapter above 5')
raises(function() ui.screen(list1({ chapter = -1 })) end, 'chapter is 0 to 5', 'chapter below 0')
check(ui.screen(list1({ chapter = 5 })), 'chapter 5')
raises(function() ui.screen(list1({ port = 0 })) end, 'port is 1 to 4', 'port 0')
raises(function() ui.screen(list1({ port = 5 })) end, 'port is 1 to 4', 'port 5')
check(ui.screen(list1({ port = 4 })), 'port 4')
raises(function() ui.screen(list1({ explainer = { width = 'huge' } })) end, 'narrow, normal or wide', 'a bad explainer width')
raises(function() ui.screen(list1({ explainer = 'some' })) end, 'table or "none"', 'a bad explainer string')
check(ui.screen(list1({ explainer = 'none' })) and ui.screen(list1({ explainer = { width = 'wide' } })), 'none and the three widths')
raises(function() ui.screen({ id = 'x.l', primary = { kind = 'grid', blocks = { 5 } } }) end, 'block 1 is not a table', 'a non-table block')
raises(function() ui.screen({ id = 'x.l', primary = { kind = 'grid', blocks = { { id = 'b', cols = 1, cells = { 'c' } } } } }) end, 'cell 1 is not a table', 'a non-table cell')
raises(function() ui.screen({ id = 'x.l', primary = { kind = 'list', items = { 'i' } } }) end, 'item 1 is not a table', 'a non-table item')
raises(function() ui.screen(list1({ keys = { 'A' } })) end, 'key hint 1 is not a table', 'a non-table key')
raises(function() ui.screen(list1({ keys = { { 'A', 'ok' }, { 'B', 'ok' }, { 'X', 'ok' }, { 'Y', 'ok' }, { 'Z', 'ok' }, { 'L', 'ok' }, { 'R', 'ok' } } })) end, 'at most 6', 'seven keys')
raises(function() ui.screen({ id = 'x.l', primary = { kind = 'list', items = { { id = 'a', value = { kind = 'dial' } } } } }) end, 'unknown value kind', 'an unknown value kind')
-- the conversion arena
local big = {}; for i = 1, 3000 do big[i] = i end
raises(function() ui.screen(list1({ extra = big })) end, 'the description is too large', 'an oversized array')
local keyed = {}; for i = 1, 3000 do keyed['k' .. i] = { i } end
raises(function() ui.screen(list1({ extra = keyed })) end, 'the description is too large', 'an oversized keyed table')
local deep = {}; do local c = deep; for _ = 1, 12 do c.x = {}; c = c.x end end
raises(function() ui.screen(list1({ extra = deep })) end, 'nested too deeply', 'nesting past 8 levels')
local cyc = {}; cyc.self = cyc
raises(function() ui.screen(list1({ extra = cyc })) end, 'nested too deeply', 'a cycle')
local words = {}; for i = 1, 200 do words[i] = string.rep('w', 300) end
raises(function() ui.screen(list1({ extra = words })) end, 'the description is too large', 'a string pool over the ceiling')
check(ui.screen(list1({ extra = { 1, 2, 3 } })), 'a small extra table is fine')
-- explainer fields are cut to their buffers (AT_STR 63, AT_TEXT 159) with a warning
local cutd = list1({ explainer = { provide = function() return { title = string.rep('t', 100), kicker = 'k', what = string.rep('word ', 60) } end } })
ui.screen(cutd); local cv = ui.refresh('x.l').explainer
check(#cv.title == 63 and #cv.what == 159 and cv.warn == true, 'title cut to 63 and what to 159, with a warning (got ' .. #cv.title .. ', ' .. #cv.what .. ')')
local okd = list1({ explainer = { provide = function() return { title = 'short', what = 'fits' } end } })
ui.screen(okd); check(ui.refresh('x.l').explainer.warn == false, 'nothing cut: no warning')
-- the owner prefix, and ownership of every call
local mod = Stub.new({ owner_mod = 'envoy', caller = 'envoy/a' })
raises(function() mod.screen(list1({ id = 'other.x' })) end, 'must start with "envoy."', 'a mod screen id carries the mod id')
check(mod.screen(list1({ id = 'envoy.x' })) and mod.open('envoy.x'), 'the owner registers and opens')
mod.caller = 'envoy/b'
raises(function() mod.open('envoy.x') end, 'belongs to another script', 'open of another script\'s screen')
raises(function() mod.close('envoy.x') end, 'belongs to another script', 'close of another script\'s screen')
raises(function() mod.feed('envoy.x', 'down') end, 'belongs to another script', 'feed')
raises(function() mod.focus('envoy.x') end, 'belongs to another script', 'focus')
raises(function() mod.set_focus('envoy.x', 'list', 'a') end, 'belongs to another script', 'set_focus')
raises(function() mod.screen(list1({ id = 'envoy.x' })) end, 'belongs to another script', 're-registering it')
check(mod.note({ text = 'x' }) == false and mod.dialog({ title = 'x' }) == false and mod.close() == false, 'note, dialog and close() act only on the caller\'s own top screen')
check(mod.state().top == 'envoy.x' and #mod.notes == 0 and #mod.dialogs == 0, 'and nothing changed')
mod.caller = 'console'; check(mod.feed('envoy.x', 'down') and mod.close('envoy.x'), 'the console may drive any screen')
raises(function() mod.feed('envoy.x', 'sideways') end, 'unknown intent', 'an unknown intent') -- (screen is closed but still registered)
-- page, start, value rows, results
local pl = {}
local pg = list1({ id = 'x.p', primary = { kind = 'list', items = {
  { id = 'tg', label = 'T', value = { kind = 'toggle', on = false } }, { id = 'sl', label = 'S', value = { kind = 'slider', min = 0, max = 100, value = 50 } },
  { id = 'ch', label = 'C', value = { kind = 'choice' } }, { id = 'off', label = 'D', disabled = true, value = { kind = 'toggle' } } } },
  on = { page = function(dir, c, b) pl[#pl + 1] = 'page ' .. dir .. ' ' .. c .. ' ' .. b end, start = function(c, b) pl[#pl + 1] = 'start ' .. c end,
         change = function(id, v) pl[#pl + 1] = id .. '=' .. tostring(v) end, accept = function(c) pl[#pl + 1] = 'accept ' .. c end } })
ui.screen(pg); ui.open('x.p')
ui.engine_press('x.p', 'l'); ui.engine_press('x.p', 'r'); ui.engine_press('x.p', 'start')
check(table.concat(pl, ',') == 'page -1 tg list,page 1 tg list,start tg', 'L and R reach on.page(dir, cell, block); START reaches on.start (got ' .. table.concat(pl, ',') .. ')')
pl = {}
ui.engine_press('x.p', 'accept'); ui.engine_press('x.p', 'accept')
check(table.concat(pl, ',') == 'tg=true,tg=false', 'a toggle flips on accept, and on.accept is not called for it (got ' .. table.concat(pl, ',') .. ')')
ui.set_focus('x.p', 'list', 'sl'); pl = {}
ui.engine_row('x.p', 'right'); ui.engine_row('x.p', 'left'); ui.engine_row('x.p', 'left')
check(table.concat(pl, ',') == 'sl=55,sl=50,sl=45', 'a slider steps by 5 on 0..100 (got ' .. table.concat(pl, ',') .. ')')
ui.set_focus('x.p', 'list', 'ch'); pl = {}
ui.engine_row('x.p', 'right'); ui.engine_row('x.p', 'left'); ui.engine_press('x.p', 'accept')
check(table.concat(pl, ',') == 'ch=1,ch=-1,ch=1', 'a choice reports its direction (got ' .. table.concat(pl, ',') .. ')')
ui.set_focus('x.p', 'list', 'off'); pl = {}
ui.engine_row('x.p', 'right'); ui.engine_press('x.p', 'accept')
check(#pl == 0, 'a disabled row changes nothing')
ui.close('x.p')
-- the pad: a held button is carried across a stack change
local hl = {}
local function mk(id, acc, back)
  return list1({ id = id, on = { accept = function() hl[#hl + 1] = id .. ':a'; return acc end, back = function() hl[#hl + 1] = id .. ':b'; return back end } })
end
ui.screen(mk('x.1', { push = 'x.2' }, { pop = true })); ui.screen(mk('x.2', nil, { pop = true })); ui.screen(mk('x.3', nil, { pop = true }))
ui.open('x.1'); ui.hold('A'); ui.tick()
check(table.concat(hl, ',') == 'x.1:a' and ui.state().top == 'x.2', 'the press fires on the first screen and pushes the second')
ui.tick(); ui.tick()
check(table.concat(hl, ',') == 'x.1:a', 'a still-held A does not fire on the new top')
ui.release('A'); ui.tick(); ui.hold('A'); ui.tick()
check(table.concat(hl, ',') == 'x.1:a,x.2:a', 'released and pressed again, it fires')
ui.release('A'); ui.tick(); hl = {}
ui.open('x.3'); ui.hold('B'); ui.tick(); ui.tick(); ui.tick()
check(table.concat(hl, ',') == 'x.3:b' and ui.state().top == 'x.2', 'one held B closes one level, not the stack (got ' .. table.concat(hl, ',') .. ' top ' .. tostring(ui.state().top) .. ')')
ui.release('B'); ui.tick(); ui.hold('B'); ui.tick()
check(table.concat(hl, ',') == 'x.3:b,x.2:b' and ui.state().top == 'x.1', 'after a release the next B closes the next level')
ui.release('B'); ui.close(); ui.held = {}
local off = Stub.new({ available = false }); check(select(1, off.available()) == false, 'an unavailable stub says so')
print(('atlas ui stub: %d checks, %d failed'):format(count, fails))
os.exit(fails == 0 and 0 or 1)
