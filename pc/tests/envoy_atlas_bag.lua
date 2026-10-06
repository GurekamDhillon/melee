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
  local log = { pressed = {}, marked = {}, notices = {}, refreshes = 0 }
  local S = setmetatable({ g = g, mode = 'bag', layout = 'main', active = true, log = log, blocks = blocks(),
    host = { seat = opts.seat, bag = function() return { capacity = function() return 4 end } end, plan_take = function() return { action = 'equip' } end } },
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
  local S, ui = fake({ seat = { port = 2 } }); A.set(true); assert(A.attach(S)); assert(ui.screens[ID].port == 2); A.set(false)
end)

T.test('focus: on.focus marks the merge target, re-registers, and the provider explains the real lines', function()
  local S, ui, log = fake(); A.set(true); A.attach(S)
  local before = ui.refreshed
  ui.engine_focus(ID, 'bag', 'bag:2')
  assert(log.marked[#log.marked] == 'Drive 2', 'the legacy mark_target ran for the focused cell')
  assert(ui.refreshed > before, 'the description was re-registered (merge flags may have changed)')
  local v = ui.views[ID]
  assert(v.explainer.kicker == 'BAG CELL 2' and v.explainer.title == 'Drive 2', v.explainer.kicker)
  assert(v.explainer.what == 'Aerial hits set Burning for 3 s.\nBurning targets take more damage.', v.explainer.what)
  assert(v.explainer.media.model == 102 and v.explainer.media.ring == 150 and v.explainer.from.text:find('Magic drive', 1, true))
  local btn = {}; for _, k in ipairs(v.keys) do btn[k[1]] = k[2] end
  assert(btn.A == 'Equip' and btn.Y == 'Discard' and btn.B == 'Close' and btn.X == nil, 'hints are the legacy actions plus Close; a false action is hidden')
  assert(v.counter == 'Bag 2 / 4', tostring(v.counter))
  A.set(false)
end)

T.test('locked and empty cells have no drive actions and still explain themselves', function()
  local S, ui, log = fake(); A.set(true); A.attach(S)
  ui.engine_focus(ID, 'eq', 'eq:6')
  local v = ui.views[ID]
  assert(v.explainer.title == 'Locked slot' and v.explainer.what == 'Slot 6 unlocks at depth 5.' and v.explainer.media == nil and v.explainer.from == nil)
  assert(#v.keys == 1 and v.keys[1][1] == 'B', 'only Close is offered')
  assert(ui.engine_press(ID, 'accept'), 'A reaches the handler ...')
  assert(log.accepted and log.accepted.kind == 'locked', '... on the locked ref, where the legacy accept does nothing')
  ui.engine_focus(ID, 'bag', 'bag:4')
  local v2 = ui.views[ID]
  assert(v2.explainer.what == 'Empty bag place.' and #v2.keys == 1 and v2.counter == 'Bag 4 / 4')
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

T.done()
