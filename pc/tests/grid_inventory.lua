-- Pure-logic tests for the grid inventory component (demos/grid-inventory). Plain Lua, no game:
--   lua melee/pc/tests/grid_inventory.lua          (from the workspace root)
local root = (arg and arg[0] or ''):gsub('\\', '/'):gsub('[^/]*$', '')
local base = root .. '../scripts/examples/demos/grid-inventory/scripts/'
local G = dofile(base .. 'grid.lua')
local LAY = dofile(base .. 'layouts.lua')

local fails, count = 0, 0
local function check(cond, msg) count = count + 1; if not cond then fails = fails + 1; print('FAIL: ' .. msg) end end
local function eq(a, b, msg) check(a == b, ('%s (got %s, want %s)'):format(msg, tostring(a), tostring(b))) end

-- a gd stand-in without a kit: counts draw calls
local function stub_g(w)
  local calls = 0
  local g = {}
  for _, n in ipairs({ 'fill', 'box', 'text', 'line' }) do g[n] = function() calls = calls + 1 end end
  g.safe_area = function() return { x = 0, y = 0, w = w or 640, h = 480, right = w or 640, bottom = 480 } end
  g.frame = function() return 0 end
  g.calls = function() return calls end
  g.reset = function() calls = 0 end
  return g
end
local function cell(name, extra) local c = { name = name, colour = 'red', rarity = 'common', lines = { name } }; for k, v in pairs(extra or {}) do c[k] = v end; return c end
local function where(view) local c, b, i = view:focused(); return b .. ':' .. i end

-- ---------------------------------------------------------------- focus movement
do
  -- block A: 3x2 with a hole at 5 (uneven second row); block B: 2x2 to its right; block K below: 1x3
  local A = { id = 'A', cols = 3, rows = 2, band = 1, cells = { cell('a1'), cell('a2'), cell('a3'), cell('a4'), nil, cell('a6') } }
  local B = { id = 'B', cols = 2, rows = 2, band = 1, cells = { cell('b1'), cell('b2'), G.EMPTY, cell('b4') } }
  local K = { id = 'K', cols = 3, rows = 1, band = 2, cells = { cell('k1'), cell('k2') } }   -- third slot does not exist
  local v = G.new{ g = stub_g(), blocks = { A, B, K } }
  eq(where(v), 'A:1', 'starts on the first focusable cell')
  v:press('right'); eq(where(v), 'A:2', 'right within a block')
  v:press('right'); v:press('right'); eq(where(v), 'B:1', 'right crosses into the next block in the same row')
  v:press('right'); eq(where(v), 'B:2', 'right inside B')
  v:press('right'); eq(where(v), 'A:1', 'right at the end of the band wraps to the first cell of the first block, same row')
  v:press('right'); v:press('right'); eq(where(v), 'A:3', 'right again')
  v:press('down'); eq(where(v), 'A:6', 'down inside a block')
  v:press('down'); eq(where(v), 'K:2', 'down leaves the block into the band below, nearest column (k2 is under a3)')
  v:press('left'); eq(where(v), 'K:1', 'left within K')
  v:press('left'); eq(where(v), 'K:2', 'left at the start of a one-row band wraps to its last existing cell (a hole is not a cell)')
  v:press('down'); eq(where(v), 'A:2', 'down at the bottom wraps to the top, same column')
  v:press('up'); eq(where(v), 'K:2', 'up at the top wraps to the bottom band, same column')

  -- uneven rows: from a3 (row 1, col 3) down lands on a6 (row 2, col 3); from a5-hole's neighbours
  v:set_focus('A', 4); v:press('right'); eq(where(v), 'A:6', 'right skips the hole at A:5')
  v:press('left'); eq(where(v), 'A:4', 'left skips the hole at A:5')
  v:set_focus('A', 6); v:press('up'); eq(where(v), 'A:3', 'up in the third column')
  v:set_focus('A', 4); v:press('down'); eq(where(v), 'K:1', 'down from the last row crosses to the keystones')

  -- empty slots are focusable; holes are not
  v:set_focus('B', 3); eq(where(v), 'B:3', 'an empty slot can hold focus')
  eq(v:set_focus('A', 5), false, 'a hole cannot be focused')
  eq(v:set_focus('B', 3), true, 'empty slot focus')

  -- wrap = false stops at the edges
  local w = G.new{ g = stub_g(), wrap = false, blocks = { { id = 'A', cols = 3, rows = 1, cells = { cell('x'), cell('y'), cell('z') } } } }
  eq(w:press('left'), nil, 'no wrap: left at the start does nothing')
  w:press('right'); w:press('right'); eq(w:press('right'), nil, 'no wrap: right at the end does nothing')
  eq(where(w), 'A:3', 'no wrap: stays on the last cell')

  -- a single focusable cell never loops onto itself
  local one = G.new{ g = stub_g(), blocks = { { id = 'S', cols = 1, rows = 1, cells = { cell('only') } } } }
  eq(one:press('right'), nil, 'one cell: right does nothing'); eq(one:press('down'), nil, 'one cell: down does nothing')

  -- unfocusable blocks are skipped for focus but still exist for compare
  local u = G.new{ g = stub_g(), blocks = {
    { id = 'in', cols = 1, rows = 1, band = 1, focusable = false, cells = { cell('incoming') } },
    { id = 'eq', cols = 2, rows = 1, band = 2, cells = { cell('e1'), cell('e2') } } } }
  eq(where(u), 'eq:1', 'focus starts on a focusable block'); u:press('up'); eq(where(u), 'eq:2', 'up cannot reach an unfocusable block (wraps within the focusable ones)')
  u:set_compare('in', 1, 'after'); check(u.ce ~= nil, 'compare partner may sit in an unfocusable block')

  -- rebuild keeps focus on the same block and index; falls back to the nearest cell when the slot is gone
  v:set_focus('A', 6)
  v:set_block('A', { cell('a1'), cell('a2'), cell('a3'), cell('a4'), cell('a5'), cell('a6') })
  eq(where(v), 'A:6', 'focus survives a block update')
  v:set_block('A', { cell('a1'), cell('a2'), cell('a3') }, nil, 3, 1)
  eq(where(v), 'A:3', 'focus falls back to the nearest remaining cell')

  -- press(): actions
  local f = G.new{ g = stub_g(), actions = { B = 'Back' }, blocks = { { id = 'x', cols = 2, rows = 1, cells = {
    cell('c1', { actions = { A = 'Equip', Y = 'Drop', X = false } }), cell('c2') } } } }
  local b, label, c, id, i = f:press('A'); eq(b, 'A', 'A offered'); eq(label, 'Equip', 'A label'); eq(id, 'x', 'block id'); eq(i, 1, 'index'); eq(c.name, 'c1', 'cell')
  eq(f:press('X'), nil, 'a false action is unavailable'); eq(f:press('B'), 'B', 'view default B'); f:press('right'); eq(f:press('A'), nil, 'no A on c2')
end

-- ---------------------------------------------------------------- compare lines
do
  local a = { stats = { { key = 'dmg', label = 'Damage', value = 18, text = '+18%' }, { key = 'hit', label = 'Taken', value = 6, text = '+6%', better = 'low' } } }
  local b = { stats = { { key = 'dmg', label = 'Damage', value = 26, text = '+26%' }, { key = 'hit', label = 'Taken', value = 6, text = '+6%', better = 'low' },
                        { key = 'spd', label = 'Speed', value = 4, text = '+4%' } } }
  local l = G.compare_lines(a, b)
  eq(#l, 3, 'union of stat keys')
  eq(l[1].text, 'Damage  +18% -> +26%', 'before -> after text'); eq(l[1].tone, 'ok', 'higher damage is better')
  eq(l[2].tone, 'muted', 'unchanged is muted')
  eq(l[3].text, 'Speed  - -> +4%', 'a stat only the new drive has'); eq(l[3].tone, 'ok', 'gained stat is ok')
  local worse = G.compare_lines({ stats = { { key = 'hit', label = 'Taken', value = 5, better = 'low' } } }, { stats = { { key = 'hit', label = 'Taken', value = 9, better = 'low' } } })
  eq(worse[1].tone, 'danger', 'more damage taken is worse (better = low)')
  local lost = G.compare_lines(b, a); eq(lost[3].tone, 'danger', 'a stat that goes away is worse')
  local empty = G.compare_lines(nil, a); eq(empty[1].text, 'Damage  - -> +18%', 'empty slot before'); eq(empty[1].tone, 'ok', 'filling an empty slot is better')
  local plain = G.compare_lines({ lines = { 'one', 'same' } }, { lines = { 'two', 'same', 'extra' } })
  eq(plain[1].text, 'one -> two', 'plain lines pair by position'); eq(plain[2].text, 'same', 'identical plain lines collapse'); eq(plain[3].text, '- -> extra', 'missing plain line')

  -- the view shows compare for a reward: focus = incoming (after), partner = equipped (before)
  local v = G.new{ g = stub_g(), blocks = { { id = 'o', cols = 1, rows = 1, band = 1, cells = { b } }, { id = 'e', cols = 1, rows = 1, band = 2, focusable = false, cells = { a } } } }
  v:set_compare('e', 1, 'before'); v:layout(v.g.safe_area()); local dm = v:model()
  check(dm.cmp, 'compare model'); eq(dm.e_before.cell, a, 'partner is the before'); eq(dm.e_after.cell, b, 'focus is the after')
  v:set_compare('e', 1, 'after'); dm = v:model(); eq(dm.e_before.cell, b, 'swap: focus is the before'); eq(dm.e_after.cell, a, 'swap: partner is the after')
  b.nocompare = true; v.ver = v.ver + 1; dm = v:model(); check(not dm.cmp, 'nocompare cells never compare')
end

-- ---------------------------------------------------------------- word wrap
do
  local m = function(t) return #t * 7 end
  local lines = G.wrap('one two three four five six', 70, m)
  for _, l in ipairs(lines) do check(m(l) <= 70, 'wrapped line fits: ' .. l) end
  eq(table.concat(lines, ' '), 'one two three four five six', 'wrap keeps every word')
  eq(#G.wrap('', 70, m), 1, 'empty text is one empty line')
  eq(#G.wrap('supercalifragilistic', 70, m), 1, 'an overlong word is one line (the kit fits it)')
end

-- ---------------------------------------------------------------- layout maths
local function overlap(a, b) return a.x < b.x + b.w and b.x < a.x + a.w and a.y < b.y + b.h and b.y < a.y + a.h end
local function inside(a, o) return a.x >= o.x and a.y >= o.y and a.x + a.w <= o.x + o.w and a.y + a.h <= o.y + o.h end
local sizes = { { 640, 480 }, { 853, 480 }, { 960, 480 }, { 1138, 480 }, { 560, 480 } }
local minimums = {}
for _, name in ipairs(LAY.names) do
  for _, sz in ipairs(sizes) do
    local g = stub_g(sz[1])
    local v = G.new{ g = g }
    LAY.apply(v, name)
    local area = g.safe_area()
    local L = v:layout(area)
    local tag = ('%s @%dx%d'):format(name, sz[1], sz[2])
    local canvas = { x = 0, y = 0, w = sz[1], h = sz[2] }
    check(inside(L.head, canvas) and inside(L.bar, canvas) and inside(L.grid, canvas) and inside(L.detail, canvas), tag .. ': regions inside the canvas')
    check(not overlap(L.head, L.grid) and not overlap(L.head, L.detail) and not overlap(L.grid, L.detail), tag .. ': head, grid, detail do not overlap')
    check(not overlap(L.bar, L.grid) and not overlap(L.bar, L.detail) and not overlap(L.bar, L.head), tag .. ': action bar clear of the rest')
    -- title-safe margins on the canvas used
    check(L.head.x >= 32 and L.head.x + L.head.w <= sz[1] - 32 and L.head.y >= 24 and L.bar.y + L.bar.h <= sz[2] - 24, tag .. ': inside the title-safe margins')
    if sz[1] >= 640 then check(L.fit, tag .. ': fits') end
    local rects = {}
    for b, r in pairs(L.blocks) do
      check(inside(r, L.grid), tag .. ': block ' .. b.id .. ' inside the grid area')
      for _, o in ipairs(rects) do check(not overlap(r, o), tag .. ': blocks do not overlap') end
      rects[#rects + 1] = r
    end
    for _, e in ipairs(v.E) do
      check(inside({ x = e.px, y = e.py, w = L.cell, h = L.cell }, L.blocks[e.b]), tag .. ': cell inside its block')
    end
    -- the detail panel is wide enough for its text and the action bar items fit the bar
    local dm = v:model()
    check(dm.bar_w + 28 <= L.bar.w + 18, tag .. ': action bar items fit')
    if sz[1] == 640 then minimums[name] = L.cell end
    -- a wider canvas never makes the cells smaller
    if sz[1] == 853 then check(L.cell >= minimums[name], tag .. ': wider screen, cells no smaller') end
  end
end
do
  -- the layout is cached: same area returns the same table; a new size recomputes
  local g = stub_g(); local v = G.new{ g = g }; LAY.apply(v, 'bag6')
  local L1 = v:layout(g.safe_area()); eq(v:layout(g.safe_area()), L1, 'layout cached')
  local L2 = v:layout({ x = 0, y = 0, w = 853, h = 480 }); check(L2 ~= L1, 'layout recomputed for a new size')
  -- stated minimum: the largest layout (bag6: 3x2 + 2x2 + keystones) keeps cells >= min_cell at 640x480
  local c640 = minimums.bag6
  check(c640 >= G.cfg.min_cell, 'bag6 at 640x480 keeps cells at or above min_cell')
  print(('cell edge at 640x480: %s'):format(table.concat((function() local t = {} for _, n in ipairs(LAY.names) do t[#t + 1] = n .. '=' .. minimums[n] end return t end)(), ' ')))
  -- below the minimum canvas the layout says so instead of overlapping silently
  local tiny = G.new{ g = g }; LAY.apply(tiny, 'bag6'); local Lt = tiny:layout({ x = 0, y = 0, w = 420, h = 480 })
  check(Lt.fit == false or Lt.cell >= G.cfg.min_cell, 'a canvas that is too small reports fit = false')
end

-- ---------------------------------------------------------------- countdown, input helper
do
  local v = G.new{ g = stub_g(), blocks = { { id = 'a', cols = 1, rows = 1, cells = { cell('x') } } } }
  v:set_countdown(12.4, 30); eq(v.cd_txt, '13s', 'countdown shows whole seconds, rounded up')
  local s = v.cd_txt; v:set_countdown(12.1, 30); eq(v.cd_txt, s, 'same second keeps the same string (no new string)')
  v:set_countdown(nil); eq(v.cd_txt, nil, 'countdown can be cleared')
  local I = G.new_input{ first = 3, every = 2 }
  local o = I:poll({ RIGHT = true }); eq(o.n, 1, 'direction press'); eq(o[1], 'right', 'right')
  o = I:poll({ RIGHT = true }); eq(o.n, 0, 'held: no repeat yet'); o = I:poll({ RIGHT = true }); eq(o.n, 0, 'held 2')
  o = I:poll({ RIGHT = true }); eq(o.n, 1, 'repeat after the delay'); o = I:poll({ RIGHT = true }); eq(o.n, 0, 'repeat spacing')
  o = I:poll({ RIGHT = true }); eq(o.n, 1, 'repeats again')
  o = I:poll({ y = 100, x = 0 }); eq(o[1], 'up', 'stick up past the threshold')
  o = I:poll({ y = 20, x = 20 }); eq(o.n, 0, 'inside the dead zone')
  o = I:poll({ A = true }); eq(o[1], 'A', 'A edge'); o = I:poll({ A = true }); eq(o.n, 0, 'A held is not a new press')
  o = I:poll({ A = true, B = true }); eq(o[1], 'B', 'B edge while A held')
end

-- ---------------------------------------------------------------- draw cost shape (stub gd)
do
  local g = stub_g(853); local v = G.new{ g = g }; LAY.apply(v, 'bag6')
  v:draw(); g.reset(); v:draw()
  local per = g.calls()
  check(per < 500, 'bag6 draws in under 500 calls (' .. per .. ')')
  collectgarbage(); collectgarbage()
  local before = collectgarbage('count')
  for _ = 1, 300 do v:draw() end
  collectgarbage(); collectgarbage()
  local grown = collectgarbage('count') - before
  check(grown < 8, ('300 draws leave the heap alone (%.2f KB)'):format(grown))
  print(('draw calls per frame (stub, no kit): %d'):format(per))
end

-- ---------------------------------------------------------------- replaceable icon + cell_rect
do
  local g = stub_g(853); local got = {}
  local v = G.new{ g = g, icon_draw = function(desc, x, y, w, h, focused, locked) got[#got + 1] = { desc = desc, x = x, y = y, w = w, h = h, focused = focused, locked = locked } end,
    blocks = { { id = 'b', cols = 2, rows = 1, cells = { cell('m', { icon = { kind = 'model', asset = 'x' } }), cell('n', { icon = { kind = 'model', asset = 'y' }, flags = { 'locked' } }) } } } }
  v:draw()
  eq(#got, 3, 'icon_draw runs for each descriptor cell and once more for the detail swatch')
  local x, y, w, h, ix, iy, iw, ih = v:cell_rect('b', 1)
  eq(got[1].x, ix, 'icon rect x matches cell_rect'); eq(got[1].y, iy, 'icon rect y'); eq(got[1].w, iw, 'icon rect w'); eq(got[1].h, ih, 'icon rect h')
  check(ix > x and iy > y and ix + iw < x + w and iy + ih < y + h, 'icon rect sits inside the cell border')
  eq(got[1].focused, true, 'focused cell flagged'); eq(got[2].focused, false, 'other cell not focused'); eq(got[2].locked, true, 'locked flag passed')
  eq(v:cell_rect('b', 5), nil, 'unknown cell has no rect')
end

-- ---------------------------------------------------------------- embedded copies are current
do
  local function read(p) local f = assert(io.open(p, 'rb')); local s = f:read('a'); f:close(); return s end
  local main = read(base .. 'main.lua')
  local function embedded(tag) return main:match('%-%- BEGIN GENERATED ' .. tag .. '[^\n]*\n(.-)%-%- END GENERATED ' .. tag .. '\n') end
  for _, p in ipairs({ { 'GRID', 'grid.lua' }, { 'LAYOUTS', 'layouts.lua' } }) do
    local e = embedded(p[1])
    check(e ~= nil, 'main.lua embeds ' .. p[2])
    check(e and e:find(read(base .. p[2]), 1, true) ~= nil, 'main.lua embeds the current ' .. p[2] .. ' (run scripts/embed.py)')
  end
end

print(('%d checks, %d failed'):format(count, fails))
os.exit(fails == 0 and 0 or 1)
