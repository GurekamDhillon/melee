-- Offline tests of the Envoy bag as a gd.ui description:  cd melee && lua pc/tests/envoy_atlas_bag.lua
local prefix = io.open('pc/tests/atlas_ui_stub.lua') and '' or 'melee/'
local Stub = dofile(prefix .. 'pc/tests/atlas_ui_stub.lua')
local T = dofile(prefix .. 'pc/tests/envoy_testlib.lua')
local A = assert(loadfile(T.root .. 'atlas_bag.lua'))()({})
local ID = 'envoy.bag'

local function drive(i, extra)
  local c = { name = 'Drive ' .. i, colour = 'red', pips = 2, flags = {}, icon = { kind = 'model', asset = 100 + i, ring = 150 },
    lines = { 'Magic drive: Burning Red Drive ' .. i, 'Aerial hits set Burning for 3 s.', 'Burning targets take more damage.', '', 'Goes into slot 2.' },
    actions = { A = 'Equip', X = false, Y = 'Discard' }, ref = { kind = 'bag', where = 'bag', index = i, record = { colour = 'red' } } }
  for k, v in pairs(extra or {}) do c[k] = v end
  return c
end
local function blocks()
  local eq = {}
  for i = 1, 5 do eq[i] = drive(i, { ref = { kind = 'eq', where = 'equipped', index = i, record = {} }, actions = { A = 'To bag', X = 'To bag' } }) end
  eq[6] = { name = 'Locked slot', colour = 'grey', flags = { 'locked' }, lines = { 'Slot 6 unlocks at depth 5.' }, ref = { kind = 'locked', index = 6 }, actions = {} }
  local bag = { drive(1, { flags = { 'merge' } }), drive(2), drive(3),
    { empty = true, name = 'Empty slot', lines = { 'Empty bag place.' }, ref = { kind = 'bag', where = 'bag', index = 4, empty = true }, actions = {} } }
  local key = { { name = 'Pyromancer', colour = 'purple', pips = 0, icon = { kind = 'letter', letter = 'P', colour = 0xB872F0FF },
    lines = { 'Keystone rule line.', '', 'Purple keystone.' }, ref = { kind = 'key', id = 'pyro' }, actions = {} } }
  return { { id = 'eq', title = 'EQUIPPED 5/6', cols = 6, rows = 1, cells = eq }, { id = 'bag', title = 'BAG 3/4', cols = 4, rows = 1, cells = bag },
           { id = 'key', title = 'KEYSTONES 1/3', cols = 6, rows = 1, cells = key } }
end

-- a RunScreen look-alike: a class table, so that detaching can fall back to the "legacy" methods
local Class = {}
function Class:refresh() self.log.refreshes = self.log.refreshes + 1 end
function Class:press(a) self.log.pressed[#self.log.pressed + 1] = a; if a == 'accept' then local _, ref = self:focused(); self.log.accepted = ref end end
function Class:draw() self.log.drew = true end
function Class:focused() return nil end
function Class:sync() end
function Class:notify(t) self.log.notices[#self.log.notices + 1] = t end
function Class:mark_target(c) self.log.marked[#self.log.marked + 1] = c.name end
function Class:detail_lines(c) return c.lines or { c.name } end
function Class:model_desc() return { kind = 'model', asset = 900 } end
local function fake(opts)
  opts = opts or {}
  local ui = Stub.new({ available = opts.available })
  local g = { ui = ui, match = function() return { active = true, netplay = opts.netplay or false } end,
    input_mask = function() error('Atlas must never mask the pad') end, input_chord = function() error('Atlas must never chord the pad') end }
  local log = { pressed = {}, marked = {}, notices = {}, refreshes = 0, host = {} }
  local S = setmetatable({ g = g, mode = 'bag', layout = 'main', active = true, log = log, blocks = blocks(),
    host = { seat = opts.seat, log = function(_, t) log.host[#log.host + 1] = t end, bag = function() return { capacity = function() return 4 end } end, plan_take = function() return { action = 'equip' } end } },
    { __index = Class })
  return S, ui, log, g
end

T.test('the switch: off by default, and any missing piece leaves the legacy screen', function()
  local S = fake(); A.set(false); assert(not A.enabled(S.g))
  A.set(true); assert(A.enabled(S.g))
  local S2 = fake({ available = false }); assert(not A.enabled(S2.g))
  S2.g.ui = nil; assert(not A.enabled(S2.g))
  assert(A.attach(S2) == false, 'attach refuses without gd.ui')
  A.set(false)
end)

T.test('describe: three blocks, positional ids, counts split, stones, flags and models', function()
  local S, ui = fake(); A.set(true); assert(A.attach(S))
  local d = ui.screens[ID]
  assert(d and d.primary.kind == 'grid' and #d.primary.blocks == 3)
  local eq, bag, key = d.primary.blocks[1], d.primary.blocks[2], d.primary.blocks[3]
  assert(eq.title == 'EQUIPPED' and eq.count == '5 / 6' and eq.cols == 6 and #eq.cells == 6, 'equipped block')
  assert(bag.title == 'BAG' and bag.count == '3 / 4' and #bag.cells == 4, 'bag block')
  assert(key.kind == 'stones' and key.note == 'One held' and key.count == '1 / 3', 'keystone block')
  assert(eq.cells[1].id == 'eq:1' and eq.cells[1].index == 1 and eq.cells[1].model == 101 and eq.cells[1].ring == 150, 'a drive cell: id, slot, model, ring')
  assert(eq.cells[6].flags.locked and bag.cells[4].flags.empty and bag.cells[1].flags.merge, 'locked, empty and merge flags')
  assert(key.cells[1].letter == 'P' and key.cells[1].color == 0xB872F0FF and key.cells[1].index == nil, 'a keystone: letter and colour, no slot number')
  assert(d.trail[1] == 'SOLO' and d.trail[2] == 'ENVOY' and d.trail.title == 'YOUR DRIVES' and d.chapter == 1)
  assert(d.input == 'feed' and d.explainer.width == 'narrow' and d.port == 1)
  assert(ui.stack[#ui.stack] == ID, 'attach opened the screen')
  A.set(false)
end)

T.test('co-op: the seat port is the screen port', function()
  local S, ui = fake({ seat = { port = 2 } }); A.set(true); assert(A.attach(S)); assert(ui.screens['envoy.bag.p2'].port == 2 and ui.screens[ID] == nil, 'seat 2 has its own screen id'); A.set(false)
end)

T.test('focus: on.focus marks the merge target, re-registers, and the provider explains the real lines', function()
  local S, ui, log = fake(); A.set(true); A.attach(S)
  local before = ui.refreshed
  ui.engine_focus(ID, 'bag', 'bag:2')
  assert(log.marked[#log.marked] == 'Drive 2', 'the legacy mark_target ran for the focused cell')
  assert(ui.refreshed > before, 'the description was re-registered (merge flags may have changed)')
  local v = ui.views[ID]
  assert(v.explainer.kicker == 'BAG CELL 2 - RULE 1 OF 2' and v.explainer.title == 'Drive 2', v.explainer.kicker)
  assert(v.explainer.what == 'Aerial hits set Burning for 3 s.', v.explainer.what)   -- ONE rule; Z steps to the other
  assert(v.explainer.media.model == 102 and v.explainer.media.ring == 150 and v.explainer.from.text:find('Magic drive', 1, true))
  local btn = {}; for _, k in ipairs(v.keys) do btn[k[1]] = k[2] end
  assert(btn.A == 'Equip' and btn.Y == 'Discard' and btn.B == 'Close' and btn.X == nil, 'hints are the legacy actions plus Close; a false action is hidden')
  assert(btn.Z == 'More' and btn.START == 'Close', 'a drive with two rules offers More; START closes')
  assert(v.counter == 'Bag 2 / 4', tostring(v.counter))
  A.set(false)
end)

T.test('locked and empty cells have no drive actions and still explain themselves', function()
  local S, ui, log = fake(); A.set(true); A.attach(S)
  ui.engine_focus(ID, 'eq', 'eq:6')
  local v = ui.views[ID]
  assert(v.explainer.title == 'Locked slot' and v.explainer.what == 'Slot 6 unlocks at depth 5.' and v.explainer.media == nil and v.explainer.from == nil)
  assert(#v.keys == 2 and v.keys[1][1] == 'START' and v.keys[2][1] == 'B', 'only Close (START and B) is offered')
  assert(not ui.engine_press(ID, 'accept'), 'a locked cell is disabled: the engine never fires A on it')
  assert(log.accepted == nil, 'and the legacy accept was not called')
  ui.engine_focus(ID, 'bag', 'bag:4')
  local v2 = ui.views[ID]
  assert(v2.explainer.what == 'Empty bag place.' and #v2.keys == 2 and v2.counter == 'Bag 4 / 4')
  A.set(false)
end)

T.test('A, B, X and Y reach the legacy handlers; directions go to the engine and never to the legacy view', function()
  local S, ui, log = fake(); A.set(true); A.attach(S)
  ui.engine_focus(ID, 'bag', 'bag:1')
  S:press('down'); S:press('left')
  assert(#ui.fed == 2 and ui.fed[1][2] == 'down' and ui.fed[2][2] == 'left')
  assert(#log.pressed == 0, 'directions never reach the legacy press')
  ui.engine_press(ID, 'accept'); ui.engine_press(ID, 'back'); ui.engine_press(ID, 'x'); ui.engine_press(ID, 'y')
  assert(table.concat(log.pressed, ',') == 'accept,back,x,y', table.concat(log.pressed, ','))
  local c = S:focused(); assert(c and c.name == 'Drive 1', 'the focused cell is the engine focus')
  A.set(false)
end)

T.test('focus survives merge and discard: the same cell, else the same place, else the first cell', function()
  local S, ui = fake(); A.set(true); A.attach(S)
  ui.engine_focus(ID, 'bag', 'bag:3')
  S.blocks = blocks(); S.blocks[2].cells[3] = { empty = true, name = 'Empty slot', lines = { 'Empty bag place.' }, ref = { kind = 'bag', where = 'bag', index = 3, empty = true }, actions = {} }
  S:refresh()                                                   -- the legacy refresh rebuilt S.blocks: the wrapper re-registers
  local c, b = ui.focus(ID)
  assert(c == 'bag:3' and b == 'bag', 'positional ids keep the focus on the same place')
  S.blocks = blocks(); S.blocks[2].cells[4] = nil; S.blocks[2].cells[3] = nil
  S:refresh(); c = ui.focus(ID)
  assert(c == 'bag:2', 'the cell is gone: clamped to the last cell of the same block, got ' .. tostring(c))
  S.blocks = blocks(); table.remove(S.blocks, 2)
  S:refresh(); c = ui.focus(ID)
  assert(c == 'eq:1', 'the block is gone: the first cell, got ' .. tostring(c))
  A.set(false)
end)

T.test('a notice becomes a corner note', function()
  local S, ui, log = fake(); A.set(true); A.attach(S)
  S:notify('Merged! Drive got stronger.')
  assert(log.notices[1] == 'Merged! Drive got stronger.' and ui.notes[1].text == 'Merged! Drive got stronger.')
  A.set(false)
end)

T.test('netplay: described, never masked, and the legacy input is untouched', function()
  local S, ui = fake({ netplay = true }); A.set(true); assert(A.enabled(S.g))
  A.attach(S); ui.engine_focus(ID, 'bag', 'bag:1'); ui.engine_press(ID, 'accept'); S:press('down'); S:refresh()
  assert(ui.screens[ID], 'g.input_mask and g.input_chord raise when called: getting here proves they were not')
  A.set(false)
end)

T.test('the swap layout hands the screen back to the legacy grid; close detaches', function()
  local S, ui, log = fake(); A.set(true); A.attach(S)
  assert(rawget(S, 'draw') and rawget(S, 'press') and S.atlas)
  S.layout = 'swap'; S:refresh()
  assert(S.atlas == nil and rawget(S, 'draw') == nil and rawget(S, 'press') == nil and rawget(S, 'focused') == nil, 'legacy methods are back')
  assert(#ui.stack == 0, 'the Atlas screen is closed')
  S.layout = 'main'; assert(A.attach(S)); A.detach(S)
  assert(S.atlas == nil and #ui.stack == 0)
  S.mode = 'reward'; assert(A.attach(S) == false, 'only the bag is described in step 1')
  A.set(false)
end)

T.test('a description the engine refuses falls back to the legacy screen', function()
  local S, ui = fake(); A.set(true)
  S.blocks[1].cells[2].name = nil
  local real = ui.screen
  ui.screen = function() error('gd.ui.screen: refused') end
  assert(A.attach(S) == false and S.atlas == nil and rawget(S, 'draw') == nil, 'attach failed cleanly')
  ui.screen = real; A.set(false)
end)


-- ---- fix round 1 -----------------------------------------------------------------------------------------------------

T.test('one rule at a time: the first rule, "RULE 1 OF n", and Z (or L and R) steps to the next, wrapping', function()
  local S, ui, log = fake(); A.set(true); A.attach(S)
  S.blocks[2].cells[2].lines = { 'Magic drive: Triple', 'First rule.', 'Second rule.', 'Third rule.', '', 'Goes into slot 2.' }
  S.blocks[2].cells[2].detail_done = nil
  A.set(true); S:refresh(); S.blocks = blocks(); S.blocks[2].cells[2].lines = { 'Magic drive: Triple', 'First rule.', 'Second rule.', 'Third rule.', '', 'Goes into slot 2.' }
  S:refresh()
  ui.engine_focus(ID, 'bag', 'bag:2')
  local v = ui.views[ID]
  assert(v.explainer.what == 'First rule.' and v.explainer.kicker == 'BAG CELL 2 - RULE 1 OF 3', v.explainer.what .. ' / ' .. v.explainer.kicker)
  local btn = {}; for _, k in ipairs(v.keys) do btn[k[1]] = k[2] end
  assert(btn.Z == 'More' and btn.START == 'Close' and btn.B == 'Close', 'a More hint and a START hint')
  assert(ui.engine_press(ID, 'z'), 'Z reaches the step'); v = ui.views[ID]
  assert(v.explainer.what == 'Second rule.' and v.explainer.kicker == 'BAG CELL 2 - RULE 2 OF 3', v.explainer.what)
  ui.engine_press(ID, 'z'); assert(ui.views[ID].explainer.what == 'Third rule.')
  ui.engine_press(ID, 'z'); assert(ui.views[ID].explainer.what == 'First rule.', 'wraps')
  ui.engine_press(ID, 'l'); assert(ui.views[ID].explainer.what == 'Third rule.', 'L steps back')
  ui.engine_press(ID, 'r'); assert(ui.views[ID].explainer.what == 'First rule.', 'R steps forward')
  assert(not ui.views[ID].explainer.what:find('Second', 1, true), 'never more than one rule at a time')
  ui.engine_press(ID, 'z'); ui.engine_focus(ID, 'bag', 'bag:1'); ui.engine_focus(ID, 'bag', 'bag:2')
  assert(ui.views[ID].explainer.what == 'First rule.', 'moving the focus starts at the first rule again')
  A.set(false)
end)

T.test('a drive with one rule shows no count and no More hint', function()
  local S, ui = fake(); A.set(true); A.attach(S)
  S.blocks[2].cells[1].lines = { 'Magic drive: Single', 'Only rule.', '', 'Goes into slot 2.' }
  S.blocks[2].cells[1].detail_done = nil; S:refresh(); S.blocks = blocks(); S.blocks[2].cells[1].lines = { 'Magic drive: Single', 'Only rule.', '', 'Goes into slot 2.' }; S:refresh()
  ui.engine_focus(ID, 'bag', 'bag:1')
  local v = ui.views[ID]
  assert(v.explainer.what == 'Only rule.' and v.explainer.kicker == 'BAG CELL 1', v.explainer.kicker)
  local btn = {}; for _, k in ipairs(v.keys) do btn[k[1]] = k[2] end
  assert(btn.Z == nil, 'no More hint')
  assert(not ui.engine_press(ID, 'z') or ui.views[ID].explainer.what == 'Only rule.', 'Z changes nothing')
  A.set(false)
end)

T.test('a rule longer than the field is cut at a word, ends in "...", and is logged once as a content bug', function()
  local S, ui, log = fake(); A.set(true); A.attach(S)
  local long = {}; for i = 1, 60 do long[i] = 'word' .. i end
  local rule = table.concat(long, ' ')
  S.blocks = blocks(); S.blocks[2].cells[1].lines = { 'Magic drive: Long', rule, '', 'x' }; S:refresh()
  ui.engine_focus(ID, 'bag', 'bag:2'); ui.engine_focus(ID, 'bag', 'bag:1')
  local what = ui.views[ID].explainer.what
  assert(#what <= 159 and what:sub(-3) == '...', #what .. ' ' .. what:sub(-10))
  local body = what:sub(1, -4); local last = body:match('(%S+)$')
  assert(rule:find(body, 1, true) == 1 and rule:sub(#body + 1, #body + 1) == ' ', 'cut at a word boundary, last word "' .. tostring(last) .. '"')
  ui.engine_focus(ID, 'bag', 'bag:2'); ui.engine_focus(ID, 'bag', 'bag:1')
  local n = 0; for _, l in ipairs(log.host) do if l:find('content bug', 1, true) then n = n + 1 end end
  assert(n == 1, 'logged once, not every time it is shown (' .. n .. ')')
  A.set(false)
end)

T.test('two seats open at once are two screens; closing one does not close the other', function()
  local S1, ui, log1, g = fake(); A.set(true)
  local S2 = setmetatable({ g = g, mode = 'bag', layout = 'main', active = true, log = { pressed = {}, marked = {}, notices = {}, refreshes = 0 }, blocks = blocks(),
    host = { seat = { port = 2 }, log = function() end, bag = function() return { capacity = function() return 4 end } end, plan_take = function() return { action = 'equip' } end } }, { __index = Class })
  assert(A.attach(S1) and A.attach(S2))
  assert(ui.screens['envoy.bag'] and ui.screens['envoy.bag.p2'], 'two registrations')
  assert(ui.screens['envoy.bag'].port == 1 and ui.screens['envoy.bag.p2'].port == 2)
  assert(#ui.stack == 2 and ui.stack[2] == 'envoy.bag.p2')
  S2:press('down'); assert(ui.fed[#ui.fed][1] == 'envoy.bag.p2', 'seat 2 feeds its own screen')
  S1:press('down'); assert(ui.fed[#ui.fed][1] == 'envoy.bag', 'seat 1 feeds its own screen')
  A.detach(S1)
  assert(#ui.stack == 1 and ui.stack[1] == 'envoy.bag.p2' and S2.atlas, 'seat 1 closed; seat 2 is still open')
  assert(#'envoy.bag.p4' < 47, 'the id fits the engine limit')
  A.detach(S2); assert(#ui.stack == 0)
  A.set(false)
end)

T.test('a block over the engine\'s cell limit is shown cut with a note, so the screen is never refused', function()
  local S, ui = fake(); A.set(true)
  local key = {}; for i = 1, 14 do key[i] = { name = 'K' .. i, colour = 'purple', icon = { kind = 'letter', letter = 'K', colour = 1 }, lines = { 'k', '', 'x' }, ref = { kind = 'key', id = 'k' .. i }, actions = {} } end
  S.blocks[3].cells = key
  assert(A.attach(S), 'the screen was not refused')
  local kb = ui.screens[ID].primary.blocks[3]
  assert(#kb.cells == 12 and kb.note == '+2 more not shown', #kb.cells .. ' ' .. tostring(kb.note))
  A.set(false)
end)

T.test('when the engine refuses the screen it is logged once, with the reason, and the legacy bag stays', function()
  local S, ui, log = fake(); A.set(true)
  A.logged = {}
  ui.screen = function() error('gd.ui.screen: too many screens (8)') end
  assert(A.attach(S) == false and S.atlas == nil)
  local S2 = fake(); S2.g.ui = ui; S2.host.log = S.host.log
  assert(A.attach(S2) == false)
  local n = 0; for _, l in ipairs(log.host) do if l:find('too many screens', 1, true) then n = n + 1 end end
  assert(n == 1, 'the reason was logged once (' .. n .. ')')
  A.set(false)
end)

T.test('START has a hint, locked cells are disabled, and mark_target runs again after a re-registration', function()
  local S, ui, log = fake(); A.set(true); A.attach(S)
  local d = ui.screens[ID]
  assert(d.primary.blocks[1].cells[6].flags.disabled and d.primary.blocks[1].cells[6].flags.locked, 'a locked cell is disabled')
  assert(not d.primary.blocks[2].cells[1].flags.disabled)
  local has_start = false; for _, k in ipairs(d.keys) do if k[1] == 'START' then has_start = true end end
  assert(has_start, 'the START hint')
  ui.engine_focus(ID, 'bag', 'bag:2')
  local before = #log.marked
  S.blocks = blocks(); S:refresh()
  assert(#log.marked == before + 1 and log.marked[#log.marked] == 'Drive 2', 'the merge target was marked again for the cell the focus stayed on')
  A.set(false)
end)

T.test('the pad\'s Z becomes "more" through the wrapped input poll, once per press, and detach restores the poll', function()
  local S, ui = fake(); A.set(true)
  local pad = {}
  S.g.pad = function() return pad end
  local polls = 0
  local Input = {}; function Input.poll() polls = polls + 1; return {} end
  S.input = setmetatable({ port = 1 }, { __index = Input })
  assert(A.attach(S))
  S.blocks[2].cells[2].lines = { 'Magic drive: Triple', 'First rule.', 'Second rule.', '', 'x' }
  S.blocks = blocks(); S.blocks[2].cells[2].lines = { 'Magic drive: Triple', 'First rule.', 'Second rule.', '', 'x' }; S:refresh()
  ui.engine_focus(ID, 'bag', 'bag:2')
  assert(#S.input:poll() == 0, 'nothing pressed')
  pad.Z = true
  local out = S.input:poll(); assert(#out == 1 and out[1] == 'more', 'Z became more')
  assert(#S.input:poll() == 0, 'held: not again')
  pad.Z = nil; S.input:poll(); pad.Z = true; assert(#S.input:poll() == 1, 'pressed again')
  S:press('more'); assert(ui.views[ID].explainer.what ~= nil)
  A.detach(S)
  assert(rawget(S.input, 'poll') == nil, 'the legacy poll is back')
  A.set(false)
end)

T.test('attaching twice is refused', function()
  local S, ui = fake(); A.set(true)
  assert(A.attach(S) and A.attach(S) == false and #ui.stack == 1)
  A.set(false)
end)

T.done()
