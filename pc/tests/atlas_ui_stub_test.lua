-- cd "$GW_MELEE" && lua pc/tests/atlas_ui_stub_test.lua   (it also runs from a directory that holds melee/)
local prefix = io.open('pc/tests/atlas_ui_stub.lua') and '' or 'melee/'
local Stub = dofile(prefix .. 'pc/tests/atlas_ui_stub.lua')
local fails, count = 0, 0
local function check(cond, msg) count = count + 1; if not cond then fails = fails + 1; print('FAIL: ' .. msg) end end
local function raises(f, part, msg) local ok, e = pcall(f); check(not ok and tostring(e):find(part, 1, true), msg .. ' (got ' .. tostring(e) .. ')') end

local L = Stub.limits
check(L.blocks == 6 and L.cells == 12 and L.items == 64 and L.items_lua == 32 and L.keys == 6, 'limits are read from gw_ui_screen.h')
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
raises(function() ui.screen({ id = 'x.a', primary = { kind = 'wheel' } }) end, 'not supported', 'only grid, list and tiles')
check(ui.screen({ id = 'x.t', primary = { kind = 'tiles', cols = 2, items = { { id = 'a', label = 'A' } }, more = { { id = 'm', label = 'M' } } } }), 'a tiles screen is accepted')
raises(function() ui.screen({ id = 'x.t', primary = { kind = 'tiles', cols = 3, items = { { id = 'a' } } } }) end, 'tiles: cols must be 1 or 2', 'cols 3')
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
-- fix round 2: a handler's result is judged by slot owners
do
  local m = Stub.new({ owner_mod = 'envoy' })
  local function rs(id, caller, on) m.caller = caller; m.screen({ id = id, primary = { kind = 'list', items = { { id = 'a', label = 'A' } } }, on = on }) end
  rs('envoy.one', 'envoy/a', { accept = function() return { push = 'envoy.two' } end })
  rs('envoy.two', 'envoy/a', {})
  rs('envoy.theirs', 'envoy/b', {})
  rs('envoy.pop', 'envoy/a', { accept = function() return { pop = true } end })
  m.caller = 'envoy/a'; m.open('envoy.one'); m.engine_press('envoy.one', 'accept')
  check(m.state().top == 'envoy.two', 'a handler result pushes its own screen')
  rs('envoy.push_theirs', 'envoy/a', { accept = function() return { push = 'envoy.theirs' } end })
  m.caller = 'envoy/a'; m.open('envoy.push_theirs'); m.engine_press('envoy.push_theirs', 'accept')
  check(m.state().top == 'envoy.push_theirs', 'it cannot push another script screen')
  m.caller = 'envoy/a'; m.open('envoy.pop')
  m.caller = 'envoy/b'; m.open('envoy.theirs')
  m.caller = 'envoy/a'; m.engine_press('envoy.pop', 'accept')
  check(m.state().top == 'envoy.theirs', 'a pop returned by one script does not close another script top screen')
end
-- entries (Atlas step 2): register_entry validates like at_menus_parse / at_reg_add; gd.ui.entry is the owner's alone; the engine's part
do
  local m = Stub.new({ owner_mod = 'envoy', caller = 'envoy/a' })
  check(m.register_entry({ id = 'envoy', parent = 'solo', label = 'ENVOY', action = 'script' }), 'an entry registers')
  raises(function() m.register_entry({ id = 'envoy', parent = 'solo', label = 'X', action = 'script' }) end, 'already registered', 'a duplicate id')
  raises(function() m.register_entry({ id = 'lab', parent = 'solo', label = 'LAB', action = 'script' }) end, 'starts with "envoy."', 'the id namespace')
  raises(function() m.register_entry({ id = 'envoy.x', parent = 'nowhere', label = 'X', action = 'script' }) end, 'no menu shows parent', 'an unknown parent')
  raises(function() m.register_entry({ id = 'envoy.y', parent = 'solo', label = '', action = 'script' }) end, 'needs a label', 'a label')
  raises(function() m.register_entry({ id = 'envoy.z', parent = 'solo', label = 'Z' }) end, 'needs opens or action', 'an action')
  raises(function() m.register_entry({ id = 'envoy.s', parent = 'settings.video', label = 'S', opens = 'envoy.q' }) end, 'no menu shows parent', 'a parent nothing renders yet is refused')
  check(m.register_entry({ id = 'envoy.s', parent = 'settings', label = 'S', opens = 'envoy.q' }), 'the settings list is a parent')
  check(m.entry('envoy', { visible = false }) == true and #m.entries_under('solo') == 0, 'the owner hides its entry')
  check(m.entry('envoy', { visible = true, badge = 'NEW' }) == true and #m.entries_under('solo') == 1 and m.entries.envoy.badge == 'NEW', 'and shows it again with a badge')
  local other = Stub.new({ owner_mod = 'other', caller = 'other/a' }); other.entries = m.entries
  check(other.entry('envoy', { visible = false }) == false and #m.entries_under('solo') == 1, 'another mod cannot hide it')
  local con = Stub.new({}); con.entries = m.entries
  check(con.entry('envoy', { visible = false }) == false, 'the console owns no entry')
  -- the engine's part: a script entry runs on_entry as the mod and applies {push=}
  m.screen(list1({ id = 'envoy.entry' }))
  m.hooks.on_entry = function(id) m.hooks.seen = id; return { push = 'envoy.entry' } end
  check(m.engine_activate('envoy') and m.hooks.seen == 'envoy' and m.stack[#m.stack] == 'envoy.entry', 'a script entry runs on_entry and pushes the mod screen')
  m.close('envoy.entry')
  m.register_entry({ id = 'envoy.o', parent = 'solo', label = 'O', opens = 'envoy.entry' })
  check(m.engine_activate('envoy.o') and m.stack[#m.stack] == 'envoy.entry', 'an opens entry pushes the mod screen')
  m.close('envoy.entry')
  m.netplay = true
  check(m.engine_activate('envoy') == true and #m.entries_under('solo') == 2, 'netplay hides entries under versus and online only: solo is unaffected')
  m.close('envoy.entry')
  m.register_entry({ id = 'envoy.v', parent = 'versus', label = 'V', action = 'script' })
  check(#m.entries_under('versus') == 0 and m.engine_activate('envoy.v') == false, 'a versus entry without online is hidden in netplay')
  m.netplay = false
  check(#m.entries_under('versus') == 1, 'and is back offline')
end
do
  local a, b = Stub.new({ caller = 'a/main', owner_mod = 'a' }), Stub.new({ caller = 'b/main', owner_mod = 'b' })
  check(a.hold_menu(true) == true and a.held_by == 'a/main', 'a script holds the native menu')
  b.held_by = 'a/main'
  check(b.hold_menu(false) == true, 'another script cannot release it')
  check(a.hold_menu(false) == false, 'the holder releases it')
  local con = Stub.new({}); con.held_by = 'a/main'
  check(con.hold_menu(false) == false, 'the console may release it (console-only check)')
end
-- Atlas step 3, Task 1: sixteen slots and forget
do
  local u = Stub.new({ caller = 'a/main', owner_mod = 'a' })
  check(Stub.slots == 16, 'sixteen screen slots')
  local function one(id) return { id = id, primary = { kind = 'list', items = { { id = 'x', label = 'X' } } } } end
  for i = 1, 16 do u.screen(one('a.s' .. i)) end
  raises(function() u.screen(one('a.s17')) end, 'too many screens (16)', 'the 17th screen raises')
  check(u.screen(one('a.s3')), 'a re-registration of a held id still fits')
  u.open('a.s5'); check(u.forget('a.s5') == true and u.state().depth == 0 and u.screens['a.s5'] == nil, 'forget closes and frees')
  check(u.screen(one('a.s17')), 'the freed slot takes a new screen')
  raises(function() u.forget('a.nope') end, 'no screen', 'forget of an unknown id raises')
  local other = Stub.new({ caller = 'b/main' }); other.screens, other._owner = u.screens, u._owner
  raises(function() other.forget('a.s1') end, 'belongs to another script', "another script's screen is refused")
  local con = Stub.new({}); con.screens, con._owner, con._f, con.views, con._rows = u.screens, u._owner, u._f, u.views, u._rows
  check(con.forget('a.s1') == true, 'the console may forget any screen (console-only check)')
end
-- Atlas step 3, Task 7: the HUD layer and the retail takeover in the stand-in
do
  local u = Stub.new({ mod = 'envoy' })
  check(u.hud({ id = 'envoy.hud', zones = { top_left = { { kind = 'strip', pips = {}, keys = {} } }, top_center = { { kind = 'banner', text = 'Collect the drives', button = 'A' } } } }), 'a HUD registers')
  raises(function() u.hud({ id = 'other.hud', zones = {} }) end, 'must start with', 'the id starts with the mod')
  raises(function() u.hud({ id = 'envoy.hud', zones = { top_left = { { kind = 'banner' } }, top_center = { { kind = 'banner' } } } }) end, 'banner', 'one banner only, in top_center')
  raises(function() u.hud({ id = 'envoy.hud', zones = { top_right = { { kind = 'card' }, { kind = 'card' }, { kind = 'card' }, { kind = 'card' } } } }) end, 'three opponent cards', 'three cards at most')
  raises(function() u.hud({ id = 'envoy.hud', zones = { top_left = { { kind = 'note' }, {}, {}, {}, {} } } }) end, 'holds at most', 'four parts per zone')
  raises(function() u.hud({ id = 'envoy.hud', zones = { top_left = { { kind = 'wobble' } } } }) end, 'unknown kind', 'a known kind')
  raises(function() u.hud({ id = 'envoy.hud', zones = { top_left = { { kind = 'port_card', port = 9 } } } }) end, 'port 1 to 4', 'a port')
  check(u.huds['envoy/main'].zones.top_left[1].kind == 'strip', 'a refused description changed nothing')
  check(u.toast({ zone = 'top_right', title = 'ONE', text = 'a' }) and u.toast({ zone = 'top_right', title = 'TWO', text = 'b' }), 'toasts register')
  check(#u.huds['envoy/main'].zones.top_right == 1 and u.huds['envoy/main'].zones.top_right[1].title == 'TWO', 'a toast replaces the zone\'s toast, never queues')
  raises(function() u.toast({ zone = 'bottom_left', title = 'x' }) end, 'top_left', 'a toast lives in a top corner')
  u.hud({ id = 'envoy.hud', zones = { top_left = { { kind = 'note', text = 'Merged', seconds = 4 } } } })
  check(u.huds['envoy/main'].zones.top_right[1].title == 'TWO', 'a re-description keeps the live toast')
  local until_s = u.huds['envoy/main'].zones.top_left[1].until_s
  u.now = u.now + 0.5; u.hud({ id = 'envoy.hud', zones = { top_left = { { kind = 'note', text = 'Merged', seconds = 4 } } } })
  check(u.huds['envoy/main'].zones.top_left[1].until_s == until_s, 'the same note keeps its clock')
  u.now = 100; u.hud({ id = 'envoy.hud', zones = {} })
  check(#(u.huds['envoy/main'].zones.top_right or {}) == 0, 'an expired toast is not kept')
  check(u.hud_clear() == true and u.hud_clear() == false, 'hud_clear')
  -- retail_hide: the binding's checks in its order
  check(u.retail_hide({ 'hud.damage' }) and u.retail().hidden[1] == 'hud.damage', 'hide an element')
  raises(function() u.retail_hide({ 'hud.timer' }) end, 'hud.timer may not be hidden', 'never the clock')
  raises(function() u.retail_hide({ 'hud.bogus' }) end, 'unknown retail element', 'a known element')
  local b = Stub.new({ mod = 'envoy', caller = 'envoy/second' }); b.mask, b.mask_owner = u.mask, u.mask_owner
  raises(function() b.retail_hide({ 'hud.stock' }) end, 'belongs to another script', 'another script\'s mask')
  u.netplay = true
  raises(function() u.retail_hide({ 'hud.stock' }) end, 'online', 'refused online')
  check(#u.retail().hidden == 0, 'nothing is hidden online')
  u.netplay = false; u.match_active = false
  raises(function() u.retail_hide({ 'hud.stock' }) end, 'active match', 'needs a match')
  u.match_active = true; u.scene_changed()
  check(#u.retail().hidden == 0 and u.mask_owner == nil, 'a scene change releases the mask')
  check(u.retail_hide({ 'hud.stock' }) and u.retail_hide({}) and #u.retail().hidden == 0, '{} releases it')
  local con = Stub.new({})
  raises(function() con.hud({ id = 'envoy.hud', zones = {} }) end, 'mod script', 'the console has no HUD (console-only check)')
  raises(function() con.retail_hide({}) end, 'gameplay mod script', 'the console may not hide retail (console-only check)')
  local ng = Stub.new({ mod = 'envoy', gameplay = false })
  raises(function() ng.retail_hide({ 'hud.stock' }) end, 'gameplay mod script', 'a non-gameplay script may not hide retail')
end
-- Atlas step 3, Task 8: pause screens in the stand-in
do
  local u = Stub.new({ mod = 'envoy' })
  raises(function() u.screen({ id = 'envoy.p', kind = 'pause', primary = { kind = 'grid', blocks = { { id = 'b', cols = 1, cells = {} } } } }) end, 'a pause screen has a list primary', 'a pause screen is a list')
  u.screen({ id = 'envoy.pause', kind = 'pause', primary = { kind = 'list', items = { { id = 'resume', label = 'Resume' } } } })
  u.screen({ id = 'envoy.plain', primary = { kind = 'list', items = { { id = 'a', label = 'A' } } } })
  raises(function() u.pause_screen('envoy.plain') end, 'not a pause screen', 'only a pause screen is named')
  check(u.pause_screen('envoy.pause') and u.pause_slot == 'envoy.pause', 'the pause screen is named')
  check(u.unpause() == false, 'unpause needs a pause')
  u.retail_pause(1, true); check(u.state().top == nil, 'takeover off: nothing is pushed'); u.retail_pause(1, false)
  u.pause_wanted = true; u.retail_pause(1, true)
  check(u.state().top == 'envoy.pause' and u.screens['envoy.pause'].port == 2 and u.retail().takeover and u.retail().pauser == 1, 'takeover on: pushed, driven by the pauser')
  check(u.unpause() == true and u.unpause() == false and u.take_unpause() == 1 and u.take_unpause() == nil, 'unpause is one-shot and names the pauser')
  u.retail_pause(1, false); check(u.state().top == nil and u.take_unpause() == nil, 'popped when retail unpaused; no request leaks')
  u.netplay = true; u.retail_pause(0, true)
  check(u.state().top == nil and u.unpause() == false and u.take_unpause() == nil, 'never online')
  check(u.open('envoy.pause') == false, 'a pause screen does not open online')
  u.retail_pause(0, false)
end
-- scene exit and persist
do
  local u = Stub.new({ mod = 'envoy' })
  u.screen({ id = 'envoy.keep', persist = true, primary = { kind = 'list', items = { { id = 'a', label = 'A' } } } })
  local closed = 0
  u.screen({ id = 'envoy.drop', primary = { kind = 'list', items = { { id = 'a', label = 'A' } } }, on = { close = function() closed = closed + 1 end } })
  u.open('envoy.drop'); u.open('envoy.keep'); u.scene_exit()
  check(u.state().top == 'envoy.keep' and u.state().depth == 1 and closed == 1, 'a scene exit closes an ordinary screen and leaves a persist screen')
end
-- Atlas step 3, Task 9: the cards primary, grid links and the countdown in the stand-in
do
  local u = Stub.new({ mod = 'envoy' })
  local function card(i, extra) local c = { id = 'offer:' .. i, name = 'Drive', rule = 'One rule.' }; for k, v in pairs(extra or {}) do c[k] = v end; return c end
  check(u.screen({ id = 'envoy.reward', countdown = 8, primary = { kind = 'cards', cards = { card(1), card(2, { disabled = true }), card(3) } } }), 'a cards screen registers')
  local c, b = u.focus('envoy.reward'); check(c == 'offer:1' and b == 'cards', 'one block named cards, focus on the first card')
  raises(function() u.screen({ id = 'envoy.r5', primary = { kind = 'cards', cards = { card(1), card(2), card(3), card(4), card(5) } } }) end, 'at most 4 cards', 'a fifth card is refused')
  raises(function() u.screen({ id = 'envoy.r0', primary = { kind = 'cards', cards = {} } }) end, 'needs 1 to', 'no cards is refused')
  raises(function() u.screen({ id = 'envoy.rd', primary = { kind = 'cards', cards = { card(1), card(1) } } }) end, 'duplicate card id', 'card ids are unique')
  raises(function() u.screen({ id = 'envoy.rc', countdown = 'soon', primary = { kind = 'cards', cards = { card(1) } } }) end, 'countdown', 'the countdown is a number')
  u.open('envoy.reward'); u.set_focus('envoy.reward', 'cards', 'offer:2')
  local fired = 0
  u.screens['envoy.reward'].on = { accept = function() fired = fired + 1 end }
  check(u.engine_press('envoy.reward', 'accept') == false and fired == 0, 'a disabled card does not accept')
  u.set_focus('envoy.reward', 'cards', 'offer:3'); u.engine_press('envoy.reward', 'accept'); check(fired == 1, 'an enabled card accepts')
  local g = { id = 'envoy.swap', primary = { kind = 'grid', blocks = { { id = 'eq', cols = 2, cells = { { id = 'eq:1' }, { id = 'eq:2' } } }, { id = 'bag', cols = 1, cells = { { id = 'bag:1' } } } },
    links = { { a = 'eq:1', b = 'bag:1' }, { a = 'eq:1', b = 'nope' } } } }
  check(u.screen(g) and u.links_skipped['envoy.swap'] == 1, 'a link naming a missing cell is skipped and counted')
  g.primary.links = {}; for i = 1, 17 do g.primary.links[i] = { a = 'eq:1', b = 'eq:2' } end
  raises(function() u.screen(g) end, 'at most 16 links', 'at most 16 links')
end
-- Atlas step 3, Task 19: the two demo mods run against the stand-in (their own main.lua, loaded with a fake gd)
local function load_demo(path, mod, opts)
  opts = opts or {}
  local u = Stub.new({ mod = mod, netplay = opts.netplay, available = opts.available })
  local keys, logs = {}, {}
  local env = setmetatable({ gd = { ui = u, log = function(t) logs[#logs + 1] = t end, key_pressed = function(k) local v = keys[k]; keys[k] = nil; return v end,
    player = function(p) return opts.players and opts.players[p] end } }, { __index = _G })
  local chunk = assert(loadfile(prefix .. path, 't', env)); chunk()
  return env, u, keys, logs
end
do
  local env, u, keys, logs = load_demo('pc/scripts/examples/demos/atlas-hud/scripts/main.lua', 'demo_atlas_hud', { players = { { cpu = false }, { cpu = true }, nil } })
  keys.F7 = true; env.on_tick()
  local h = u.huds['demo_atlas_hud/main']
  check(h and h.id == 'demo_atlas_hud.hud', 'F7 shows the HUD')
  check(#h.zones.top_left == 1 and h.zones.top_left[1].kind == 'strip' and #h.zones.top_right == 2, 'a strip, a port card for the human port and the toast')
  check(h.zones.top_right[1].kind == 'toast' and h.zones.top_right[2].kind == 'port_card' and h.zones.top_right[2].port == 1, 'the toast on top of the port card')
  check(h.zones.top_center[1].kind == 'banner' and h.zones.top_center[1].button == 'A' and h.zones.bottom_left[1].kind == 'note', 'a banner with the A glyph, a note')
  check(u.hud_caps(h.zones), 'inside the quiet-HUD caps')
  local want = { '', 'hud.damage', 'hud.stock', 'hud.damage,hud.stock', '' }
  for i = 1, 4 do
    keys.F8 = true; env.on_tick()
    local got = table.concat(u.retail().hidden, ',')
    check(got == want[i + 1], 'F8 step ' .. i .. ' hides [' .. got .. ']')
  end
  keys.F8 = true; env.on_tick(); check(table.concat(u.retail().hidden, ',') == 'hud.damage', 'and round again')
  keys.F7 = true; env.on_tick(); check(u.huds['demo_atlas_hud/main'] == nil, 'F7 clears it')
  local env2, u2, keys2, logs2 = load_demo('pc/scripts/examples/demos/atlas-hud/scripts/main.lua', 'demo_atlas_hud', { netplay = true })
  keys2.F7 = true; env2.on_tick(); keys2.F8 = true; env2.on_tick(); keys2.F8 = true; env2.on_tick()
  check(u2.huds['demo_atlas_hud/main'] ~= nil and #u2.retail().hidden == 0, 'online the HUD draws and nothing is hidden')
  local refused = false; for _, l in ipairs(logs2) do if l:find('refused', 1, true) then refused = true end end
  check(refused, 'the refusal is logged, not raised')
end
do
  local env, u = load_demo('pc/scripts/examples/demos/atlas-pause/scripts/main.lua', 'demo_atlas_pause')
  check(env.on_load == nil, 'the engine has no on_load hook: registering from one never ran (Atlas proof D6)')
  env.on_tick()
  check(u.pause_slot == 'demo_atlas_pause.pause' and u.screens['demo_atlas_pause.pause'].kind == 'pause', 'the pause screen is registered and named')
  u.retail_pause(0, true); check(u.state().top == nil, 'takeover off: nothing changes')
  u.retail_pause(0, false)
  u.pause_wanted = true; u.retail_pause(0, true)
  check(u.state().top == 'demo_atlas_pause.pause', 'takeover on: the demo\'s list')
  u.engine_press('demo_atlas_pause.pause', 'accept')
  check(u.take_unpause() == 0, 'Resume asks the engine to unpause the pauser')
  u.retail_pause(0, false)
  env.on_unload(); check(u.pause_slot == nil, 'unloading clears the name')
  -- the roles load a frame after the scripts: unavailable first, named on the first tick it is available, and logged
  local env3, u3, _, logs3 = load_demo('pc/scripts/examples/demos/atlas-pause/scripts/main.lua', 'demo_atlas_pause', { available = false })
  env3.on_tick(); check(u3.pause_slot == nil, 'ui unavailable: not named yet')
  u3.available_ok = true; env3.on_tick()
  check(u3.pause_slot == 'demo_atlas_pause.pause', 'named on the first tick the Atlas screens are ready')
  local named = false; for _, l in ipairs(logs3) do if l:find('named demo_atlas_pause.pause', 1, true) then named = true end end
  check(named, 'and the "named" line is logged')
end
-- Atlas step 5, Task 3: gd.ui step, options, set_value and value. Run as a MOD caller, never only as the console.
do
  local S = Stub.new{ caller = 'demo.mod', owner_mod = 'demo', available = true }
  S.screen{ id = 'demo.set', trail = { title = 'T' }, primary = { kind = 'list', items = {
    { id = 'vol', label = 'Volume', value = { kind = 'slider', min = -1, max = 50, step = 1, value = 3 } },
    { id = 'fps', label = 'FPS', value = { kind = 'choice', options = { 'Off', 'FPS', 'Perf' }, value = 0 } },
    { id = 'on',  label = 'On',  value = { kind = 'toggle', on = false } },
    { id = 'who', label = 'Who', value = { kind = 'text', text = 'P1' } },
    { id = 'old', label = 'Old', value = { kind = 'choice', text = 'x' } } } },
    keys = { { 'B', 'Back' } }, on = { back = function() return { pop = true } end } }
  check(S.value('demo.set', 'vol') == 3, 'value reads the slider')
  check(S.set_value('demo.set', 'vol', 99) == true and S.value('demo.set', 'vol') == 50, 'set_value clamps a slider')
  check(S.set_value('demo.set', 'vol', 'x') == false, 'a wrong-typed value is refused, not coerced')
  check(S.set_value('demo.set', 'vol', 1.5) == false, 'a slider takes an integer')
  check(S.set_value('demo.set', 'fps', 2) == true and S.value('demo.set', 'fps') == 2, 'a choice takes an index')
  check(S.set_value('demo.set', 'fps', 3) == false and S.value('demo.set', 'fps') == 2, 'an index past the options is refused and changes nothing')
  check(S.set_value('demo.set', 'fps', -1) == false, 'a negative index is refused')
  check(S.set_value('demo.set', 'on', true) == true and S.value('demo.set', 'on') == true, 'toggle')
  check(S.set_value('demo.set', 'on', 1) == false, 'a toggle takes a boolean, not a number')
  check(S.set_value('demo.set', 'who', 'P2') == true and S.value('demo.set', 'who') == 'P2', 'text row')
  check(S.set_value('demo.set', 'who', 5) == false, 'a text row takes a string')
  check(S.set_value('demo.set', 'old', 'y') == true and S.value('demo.set', 'old') == 'y', 'a choice without options shows the string it is given')
  check(S.set_value('demo.set', 'nope', 1) == false and S.value('demo.set', 'nope') == nil, 'unknown item')
  raises(function() return S.set_value('no.such', 'vol', 1) end, 'no screen', 'an unknown screen raises')
  -- ownership: another script's screen is refused for a mod caller
  local T = Stub.new{ caller = 'other.mod', owner_mod = 'other', available = true, shared = S }
  raises(function() return T.set_value('demo.set', 'vol', 1) end, 'belongs to another script', 'set_value on another script\'s screen raises')
  raises(function() return T.value('demo.set', 'vol') end, 'belongs to another script', 'value on another script\'s screen raises')
  local C = Stub.new{ available = true, shared = S }
  check(C.set_value('demo.set', 'vol', 7) == true and C.value('demo.set', 'vol') == 7, 'the console may touch any script screen')
  -- the engine's rule through engine_press: step 1, not (max - min) // 20 = 2
  S.set_value('demo.set', 'vol', 50)
  S.engine_focus('demo.set', 'list', 'vol'); S.engine_press('demo.set', 'right')
  check(S.value('demo.set', 'vol') == 50, 'already at max 50: a step right clamps and says nothing')
  S.engine_press('demo.set', 'left')
  check(S.value('demo.set', 'vol') == 49, 'a slider with step 1 moves by 1, not by (max - min) // 20 = 2')
  -- an options choice: the engine wraps it and reports the index
  local got = {}
  S.screen{ id = 'demo.opts', primary = { kind = 'list', items = {
    { id = 'fps', label = 'FPS', value = { kind = 'choice', options = { 'Off', 'FPS', 'Perf' }, value = 2 } },
    { id = 'dz', label = 'DZ', value = { kind = 'slider', min = 0, max = 100, value = 50 } } } },
    on = { change = function(id, v) got[#got + 1] = id .. '=' .. tostring(v) end } }
  S.engine_focus('demo.opts', 'list', 'fps')
  S.engine_press('demo.opts', 'right'); S.engine_press('demo.opts', 'right'); S.engine_press('demo.opts', 'left')
  check(table.concat(got, ',') == 'fps=0,fps=1,fps=0', 'an options choice wraps (2 -> 0), steps and reports the index, never the direction')
  check(S.value('demo.opts', 'fps') == 0, 'and the value shows it')
  got = {}
  S.engine_focus('demo.opts', 'list', 'dz'); S.engine_press('demo.opts', 'right')
  check(table.concat(got, ',') == 'dz=55', 'a slider without step keeps the Lua rule (100 // 20 = 5)')
  -- set_value does not call on.change and does not move the focus
  got = {}
  S.set_value('demo.opts', 'dz', 10)
  local f1 = S.focus('demo.opts')
  check(#got == 0 and f1 == 'dz', 'set_value reports nothing and keeps the focus')
  -- a re-registration is data again: the value goes back to the description
  S.screen{ id = 'demo.opts', primary = { kind = 'list', items = { { id = 'dz', label = 'DZ', value = { kind = 'slider', min = 0, max = 100, value = 20 } } } } }
  check(S.value('demo.opts', 'dz') == 20, 'registering again replaces the live value')
  -- options validation
  raises(function() S.screen{ id = 'demo.bad1', primary = { kind = 'list', items = { { id = 'a', label = 'A', value = { kind = 'choice', options = {} } } } } } end, 'options', 'an empty options list is refused')
  local nine = {}; for i = 1, 9 do nine[i] = 'o' .. i end
  raises(function() S.screen{ id = 'demo.bad2', primary = { kind = 'list', items = { { id = 'a', label = 'A', value = { kind = 'choice', options = nine } } } } } end, 'options', 'nine options are refused')
  raises(function() S.screen{ id = 'demo.bad3', primary = { kind = 'list', items = { { id = 'a', label = 'A', value = { kind = 'slider', min = 0, max = 10, step = 0 } } } } } end, 'step', 'a step below 1 is refused')
  raises(function() S.screen{ id = 'demo.bad4', primary = { kind = 'list', items = { { id = 'a', label = 'A', value = { kind = 'choice', options = { 'a', 5 } } } } } } end, 'options', 'an option that is not a string is refused')
end

-- Atlas step 7: the stepper, tabs, the world backdrop, WITH tags, entries under lab.pause and mods.self, the readout, track and chip parts, tokens
do
  local s = Stub.new({ caller = 'geno-lab/main', owner_mod = 'geno-lab' })
  local one = { { id = 'a', label = 'A' } }
  local got = {}
  -- a stepper: left and right report their direction, A is an accept (never an on.change)
  s.screen({ id = 'geno-lab.step', primary = { kind = 'list', items = { { id = 'st', label = 'Focus', value = { kind = 'stepper', text = 'P1' } }, { id = 'tg', label = 'T', value = { kind = 'toggle', on = false } } } },
    on = { change = function(id, v) got[#got + 1] = id .. '=' .. tostring(v) end, accept = function(c) got[#got + 1] = 'accept ' .. c end } })
  s.open('geno-lab.step')
  s.engine_press('geno-lab.step', 'right'); s.engine_press('geno-lab.step', 'left'); s.engine_press('geno-lab.step', 'accept')
  check(table.concat(got, ',') == 'st=1,st=-1,accept st', 'a stepper reports left and right, and A is an accept (got ' .. table.concat(got, ',') .. ')')
  -- tabs and the world backdrop
  local TB = {}
  s.screen({ id = 'geno-lab.t', backdrop = 'world', tabs = { { name = 'ONE' }, { name = 'TWO' }, { name = 'THREE' } }, tab = 2,
    primary = { kind = 'list', items = one }, on = { tab = function(i) TB[#TB + 1] = i end, page = function() TB[#TB + 1] = 'page' end } })
  s.open('geno-lab.t')
  check(s.tab('geno-lab.t') == 2, 'the screen starts on the tab it asked for')
  TB = {}
  check(s.tab('geno-lab.t', 3) == 3 and s.tab('geno-lab.t') == 3 and #TB == 0 and s.tab('geno-lab.t', 9) == 3, 'the script can move the tab itself: no on.tab, and a tab out of range changes nothing')
  s.tab('geno-lab.t', 2)
  s.engine_press('geno-lab.t', 'r'); s.engine_press('geno-lab.t', 'r'); s.engine_press('geno-lab.t', 'l')
  check(table.concat(TB, ',') == '3,1,3', 'L and R move the tab and tell on.tab, wrapping; on.page is not called when there are tabs (got ' .. table.concat(TB, ',') .. ')')
  raises(function() s.screen({ id = 'geno-lab.t0', tabs = {}, primary = { kind = 'list', items = one } }) end, 'tabs', 'an empty tabs list is refused')
  local many = {}
  for i = 1, Stub.limits.tabs + 1 do many[i] = { name = 'T' .. i } end
  raises(function() s.screen({ id = 'geno-lab.t9', tabs = many, primary = { kind = 'list', items = one } }) end, 'tabs', 'more tabs than the record holds are refused')
  raises(function() s.screen({ id = 'geno-lab.bd', backdrop = 'sky', primary = { kind = 'list', items = one } }) end, 'backdrop', 'backdrop is ground or world')
  local NT = {}
  s.screen({ id = 'geno-lab.nt', primary = { kind = 'list', items = one }, on = { tab = function() NT[#NT + 1] = 'tab' end, page = function(d) NT[#NT + 1] = 'page' .. d end } })
  s.open('geno-lab.nt'); s.engine_press('geno-lab.nt', 'r')
  check(table.concat(NT, ',') == 'page1', 'a screen without tabs: R still reaches on.page (got ' .. table.concat(NT, ',') .. ')')
  raises(function() s.screen({ id = 'geno-lab.w', primary = { kind = 'list', items = one }, explainer = { with = { 'a', 'b', 'c', 'd', 'e' } } }) end, 'WITH', 'five WITH tags are refused')
  -- entries: lab.pause belongs to geno-lab, mods.self is filed under the mod's own id
  s.register_entry({ id = 'geno-lab.extra', parent = 'lab.pause', label = 'Extra', action = 'script' })
  local other = Stub.new({ caller = 'tools/main', owner_mod = 'tools' })
  other.register_entry({ id = 'tools.dummy', parent = 'lab.pause', label = 'Dummy', opens = 'tools.screen' })
  other.register_entry({ id = 'tools.settings', parent = 'mods.self', label = 'Settings', opens = 'tools.settings' })
  raises(function() other.register_entry({ id = 'tools.other', parent = 'mods.sora', label = 'X', opens = 'tools.x' }) end, 'mods.self', "another mod's detail screen is not yours to fill")
  check(other.entries['tools.settings'].parent == 'mods.tools', 'mods.self is filed under mods.<mod id>')
  raises(function() other.entries('lab.pause') end, 'not yours', 'a mod that does not own lab.pause cannot read it')
  raises(function() other.activate('tools.dummy') end, 'not yours', 'nor activate an entry under it')
  check(#s.entries('lab.pause') == 1 and s.entries('lab.pause')[1].id == 'geno-lab.extra' and s.entries('lab.pause')[1].blurb == '', 'entries(parent) lists what the registry shows under a parent the caller owns')
  raises(function() s.entries('solo') end, 'not yours', 'a parent the caller does not own')
  check(#other.entries('mods.tools') == 1, 'a mod may read its own detail screen entries')
  check(s.activate('geno-lab.extra') == false and s.activated == 'geno-lab.extra' and s.activate('nope') == false, 'activate goes through the registry; an unknown id is false')
  -- the new HUD parts and their limits
  check(s.hud({ id = 'geno-lab.hud', zones = { top_left = { { kind = 'readout', title = 'P1', rows = { { label = 'Motion', value = 'Wait' } } } },
                bottom_center = { { kind = 'track', len = 26, now = 5, spans = {}, marks = {} } }, bottom_left = { { kind = 'chips', items = { { text = 'FRAMES' } } } } } }), 'a readout, a track and a chip strip register')
  local rows = {}
  for i = 1, 17 do rows[i] = { label = 'r', value = 'v' } end
  raises(function() s.hud({ id = 'geno-lab.hud', zones = { top_left = { { kind = 'readout', rows = rows } } } }) end, 'at most 16 rows', 'a readout has at most 16 rows')
  raises(function() s.hud({ id = 'geno-lab.hud', zones = { top_left = { { kind = 'readout', rows = { 5 } } } } }) end, 'not a table', 'a readout row is a table')
  local marks = {}
  for i = 1, 25 do marks[i] = { frame = i, kind = 'gfx' } end
  raises(function() s.hud({ id = 'geno-lab.hud', zones = { bottom_center = { { kind = 'track', marks = marks } } } }) end, 'at most 16 spans and 24 marks', 'a track has at most 24 marks')
  -- tokens
  check(s.token('ember') == 0xFF7A3DFF and s.token('nope') == nil and s.token(5) == nil, 'the stub reads the tokens file (ember is ' .. tostring(s.token('ember')) .. ')')
end

local off = Stub.new({ available = false }); check(select(1, off.available()) == false, 'an unavailable stub says so')
print(('atlas ui stub: %d checks, %d failed'):format(count, fails))
os.exit(fails == 0 and 0 or 1)
