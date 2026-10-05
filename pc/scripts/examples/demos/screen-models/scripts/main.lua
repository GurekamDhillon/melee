-- Screen models demo: gd.kit.model draws a script model into a rectangle of the 640x480 script canvas.
-- Read this file top to bottom. It only draws (and masks P1's D-pad while shown); it never writes game state, so run it
-- in the LAB with an idle CPU:  MELEE_SCENE="mode=lab;stage=fd;p1=fox/hu;p2=falco/cpu0;cpus=idle"
--
-- MODE 1 (cells): a 4x3 grid of cells, each drawing one of twelve original test models. The focused cell spins.
--   Every cell draws, in this order: gd.fill (background), gd.kit.model, gd.box (frame), a gd.fill badge and gd.text.
--   The badge overlaps the model's corner: if the model were drawn under the interface, the badge would hide
--   nothing, so the badge sitting ON TOP of the model and the frame on top of its edge is the ordering proof.
--   Cell 7 is locked (dim), cell 8 is a ghost (alpha), cell 12 draws with clip off to show the scissor option.
-- MODE 2 (grid): the real grid component (scripts/grid.lua, embedded below) with its icon_draw wired to gd.kit.model.
--
-- Keys: LEFT/RIGHT/UP/DOWN (or the D-pad) move the focus, TAB or pad L switches mode, F6 hides it.
-- Console: sm_mode <1|2>, sm_focus <n>, sm_press <dir>..., sm_bench [all|n [frames]], sm_cost, sm_shot <name>.

-- BEGIN GENERATED GRID (scripts/grid.lua; regenerate with scripts/embed.py)
local grid = (function()
-- Grid inventory component: blocks of cells you read at a glance, one detail panel for the focused cell.
-- Independent of any mod: it needs only a `gd`-shaped table (kit text/panel/image, fill, box, safe_area) for
-- drawing, and nothing at all for the logic, so it loads and tests under plain `lua`.
--
--   local view = grid.new{ title = "BAG", blocks = { {id="eq", title="EQUIPPED 3/4", cols=2, rows=2, cells={...}} } }
--   view:press("right")                      -- focus (returns "moved" or nil)
--   local btn, label, cell, block, index = view:press("A")   -- an action that the focused cell offers
--   view:draw()                              -- from on_draw; no table or string is built per frame
--
-- Cells are plain data (see G.new). Everything derived from them (colours, layout, wrapped text, the action
-- bar) is built when the data, the focus or the screen size changes, never per drawn frame.
-- Source of truth for the embedded copy in main.lua (scripts/embed.py).
local G = {}
G.VERSION = 1

local floor, ceil, max, min, abs = math.floor, math.ceil, math.max, math.min, math.abs

-- ---------------------------------------------------------------------------------------------- configuration
G.cfg = {
  max_cell = 56, min_cell = 24,       -- cell edge in px (640x480 canvas units); below min_cell the layout reports fit=false
  gap = 0.14, block_gap = 0.55,       -- between cells / blocks, as a fraction of the cell
  pad = 7, title_h = 20, band_gap = 8, -- block frame padding, block title row, gap between block rows
  mx = 32, my = 24, max_w = 760,      -- title-safe margins (the art brief's [32,24,608,456]) and widest composition
  head_h = 28, bar_h = 30, col_gap = 14,
  detail_min = 196, detail_max = 290, detail_frac = 0.40,
}

-- Family colours (the Envoy HUD's four stat colours plus white), rarity edges and chrome. Callers may add tokens.
G.palette = {
  red = 0xF07474FF, green = 0x77CD9CFF, blue = 0x79AAF0FF, yellow = 0xEBD175FF, white = 0xF2F2F2FF,
  gold = 0xEBD175FF, cyan = 0x5ED0E8FF, violet = 0xB58CFFFF, grey = 0x8A97A3FF,
}
G.rarity = {                          -- border colour + thickness + corner notches (shape, so it reads without colour)
  common   = { edge = 0x6E7B87FF, bt = 1, corners = 0 },
  uncommon = { edge = 0x5ED0E8FF, bt = 2, corners = 1 },
  rare     = { edge = 0xB58CFFFF, bt = 2, corners = 2 },
  unique   = { edge = 0xF2C14EFF, bt = 3, corners = 4 },
}
local CHROME = {
  dim = 0x05080DB4, cell_bg = 0x0E1319FF, empty_bg = 0x0E131990, empty_edge = 0x56626DFF, mark = 0xFFFFFFFF,
  ink = 0x0A0D12FF, shade = 0x000000B4, new = 0xF2C14EFF, merge = 0x5ED0E8FF, tab = 0xFFFFFFFF, lockdim = 0x000000A0,
  target = 0x5ED0E8FF, compare = 0x5ED0E8FF, gold = 0xEBD175FF, bar_back = 0x20303CFF, block_bg = 0x0A101AC8, block_edge = 0x5B7396FF,
}

-- ---------------------------------------------------------------------------------------------- pure helpers
local function shade(c, f)                                   -- scale rgb of 0xRRGGBBAA by f, alpha kept
  local r, g, b, a = (c >> 24) & 255, (c >> 16) & 255, (c >> 8) & 255, c & 255
  r, g, b = min(255, floor(r * f)), min(255, floor(g * f)), min(255, floor(b * f))
  return (r << 24) | (g << 16) | (b << 8) | a
end
G.shade = shade

local function resolve(c, kit)
  if type(c) == 'number' then return c end
  if type(c) == 'string' then
    local p = G.palette[c]
    if p then return p end
    if kit and kit.color then local k = kit.color(c); if k then return k end end
  elseif type(c) == 'table' and c.r then
    return ((c.r & 255) << 24) | ((c.g & 255) << 16) | ((c.b & 255) << 8) | ((c.a or 255) & 255)
  end
  return G.palette.grey
end
G.resolve = resolve

-- Word-wrap `text` to `width` using measure(text) -> width. A single overlong word is left to the kit's fit rule.
function G.wrap(text, width, measure)
  local out, line = {}, ''
  for word in tostring(text):gmatch('%S+') do
    local try = line == '' and word or (line .. ' ' .. word)
    if line ~= '' and measure(try) > width then out[#out + 1] = line; line = word else line = try end
  end
  if line ~= '' or #out == 0 then out[#out + 1] = line end
  return out
end

local function stat_text(s) if s == nil then return '-' end; if s.text ~= nil then return tostring(s.text) end; return tostring(s.value) end

-- before -> after lines for a swap. Cells carry optional `stats = {{key=, label=, value=, text=, better="high"|"low"}}`;
-- without stats the plain `lines` are paired by position. A nil cell is an empty slot. Returns {{text=, tone=}} where
-- tone is "ok" (better), "danger" (worse), "muted" (unchanged) or nil.
function G.compare_lines(before, after)
  local out = {}
  local bs, as = before and before.stats, after and after.stats
  if bs or as then
    local order, bymap_b, bymap_a = {}, {}, {}
    for _, s in ipairs(as or {}) do if not bymap_a[s.key] then bymap_a[s.key] = s; order[#order + 1] = s.key end end
    for _, s in ipairs(bs or {}) do bymap_b[s.key] = s; if not bymap_a[s.key] then order[#order + 1] = s.key end end
    for _, key in ipairs(order) do
      local b, a = bymap_b[key], bymap_a[key]
      local st = a or b
      local tone
      local bv, av = b and b.value, a and a.value
      if type(bv) == 'number' and type(av) == 'number' then
        if av ~= bv then
          local up = av > bv
          if st.better == 'low' then up = not up end
          tone = up and 'ok' or 'danger'
        else tone = 'muted' end
      elseif a and not b then tone = st.better == 'low' and 'danger' or 'ok'
      elseif b and not a then tone = st.better == 'low' and 'ok' or 'danger'
      elseif stat_text(a) == stat_text(b) then tone = 'muted' end
      out[#out + 1] = { text = (st.label or key) .. '  ' .. stat_text(b) .. ' -> ' .. stat_text(a), tone = tone }
    end
    return out
  end
  local bl, al = before and before.lines or {}, after and after.lines or {}
  for i = 1, max(#bl, #al) do
    local b, a = bl[i], al[i]
    if b == a then out[#out + 1] = { text = tostring(a), tone = 'muted' }
    else out[#out + 1] = { text = (b or '-') .. ' -> ' .. (a or '-') } end
  end
  return out
end

-- Cells that exist in a block, in row-major order: `cells[i]` is a table (a cell, or G.EMPTY for an empty slot)
-- or nil (no such slot: an uneven row). Returns the count of existing cells.
G.EMPTY = { empty = true }

-- ---------------------------------------------------------------------------------------------- the view
local V = {}; V.__index = V

-- spec: { g = gd-like table (default the global gd), title = "BAG", wrap = true, hint = "L: layout",
--         actions = { B = "Back" }          -- defaults offered when the focused cell does not list its own
--         blocks = { { id=, title=, cols=, rows=, band=1, focusable=true, cells = { [i] = cell|nil } } } }
-- cell: { colour = "red"|0xRRGGBBAA, rarity = "common"|"uncommon"|"rare"|"unique", pips = 0-4, flags = {"new","merge",
--         "equipped","locked"} (or {new=true}), icon = "bolt" (a kit icon) or {kind="model", asset=...} (handed to view.icon_draw), name = "", lines = {"plain", ...},
--         icon_draw = function(desc, x, y, w, h, focused, locked, cell) end   -- draws a table-valued cell.icon (e.g. a model)
--         actions = { A = "Equip", X = "Keep", Y = "Drop" } (false = unavailable), stats = { ... } (for compare),
--         target = true (a drop target, drawn as an outline with a plus), empty = true (empty slot),
--         nocompare = true (never show a compare for this cell) }
function G.new(spec)
  local g = spec.g or gd
  local self = setmetatable({
    g = g, title = spec.title or '', hint = spec.hint, wrap = spec.wrap ~= false, actions = spec.actions or {},
    blocks = {}, E = {}, nav = {}, bands = {}, vb = 0, ver = 0, dver = -1, skip = 0,   -- skip: profiling mask (1 panels, 2 text, 4 cells, 8 glyphs)
    o = { max_w = 0 }, st_block = { piece = 10 }, st_cursor = { piece = 8, fill = false, tint = 0 },
    st_detail = { piece = 14 }, st_bar = { piece = 8 },
  }, V)
  self.icon_draw = spec.icon_draw
  local k = g.kit
  self.kit = k and k.available and k.available() and k or nil
  self.measure_fn = spec.measure or (self.kit and function(t, role) return (self.kit.measure(t, role or 'body')) end)
    or function(t) return #t * 7 end
  self.line_h = spec.line_h or (self.kit and self.kit.metrics and self.kit.metrics('body').line) or 16
  self.gold = self.kit and self.kit.color and self.kit.color('gold') or CHROME.gold
  local lt = self.kit and self.kit.texture and self.kit.texture('ico_lock')
  if lt then self.lock_w, self.lock_h = lt.w1x, lt.h1x end
  self:set_blocks(spec.blocks or {})
  return self
end

-- Replace all blocks (rebuilds the derived model once). Focus is kept on the same block id and index if it still
-- exists, else it moves to the first focusable cell.
function V:set_blocks(blocks) self.blocks = blocks; self:rebuild() end
function V:set_block(id, cells, title, cols, rows)           -- change one block's cells (and optionally title / shape)
  for _, b in ipairs(self.blocks) do
    if b.id == id then
      b.cells = cells; if title then b.title = title end; if cols then b.cols = cols end; if rows then b.rows = rows end
      return self:rebuild()
    end
  end
end
function V:set_title(t) self.title = t end
function V:set_actions(a) self.actions = a or {}; self.ver = self.ver + 1 end
function V:set_hint(h) self.hint = h; self.ver = self.ver + 1 end

local RARITY_NONE = { edge = 0x6E7B87FF, bt = 1, corners = 0 }
local FLAGBIT = { new = 1, merge = 2, equipped = 4, locked = 8 }

local function flagbits(flags)
  local bits = 0
  if not flags then return 0 end
  for _, f in ipairs(flags) do bits = bits | (FLAGBIT[f] or 0) end
  for name, bit in pairs(FLAGBIT) do if flags[name] == true then bits = bits | bit end end
  return bits
end

function V:rebuild()
  local c = G.cfg
  local old = self.fe
  local old_id, old_i = old and old.b.id, old and old.i
  local cmp_id, cmp_i, cmp_as = self.cmp_id, self.cmp_i, self.cmp_as
  local E, nav, bands, bandof = {}, {}, {}, {}
  local ux, uy = 0, 0
  local bandmap = {}
  for _, b in ipairs(self.blocks) do
    local bi = b.band or 1
    if not bandmap[bi] then bandmap[bi] = { idx = bi }; bands[#bands + 1] = bandmap[bi] end
    local band = bandmap[bi]
    band[#band + 1] = b
  end
  table.sort(bands, function(p, q) return p.idx < q.idx end)
  local u = 1 + c.gap
  for _, band in ipairs(bands) do
    local x, rows = 0, 0
    for _, b in ipairs(band) do
      b.ux0, b.uy0 = x, uy
      rows = max(rows, b.rows)
      for i = 1, b.cols * b.rows do
        local cell = b.cells[i]
        if cell then
          local col, row = (i - 1) % b.cols, (i - 1) // b.cols
          local rc = self:resolve_cell(cell)
          local e = { b = b, i = i, cell = cell, rc = rc, ux = x + col * u, uy = uy + row * u, px = 0, py = 0,
                      nav = b.focusable ~= false }
          E[#E + 1] = e
          if e.nav then nav[#nav + 1] = e end
        end
      end
      x = x + b.cols * u + 0.7
    end
    uy = uy + rows * u + 0.9
  end
  self.E, self.nav, self.bands = E, nav, bands
  -- keep the focus / compare partner
  local function find(id, i) for _, e in ipairs(E) do if e.b.id == id and e.i == i then return e end end end
  local fe = old_id and find(old_id, old_i)
  if not (fe and fe.nav) then
    fe = nil
    if old_id then                                  -- same block, nearest existing cell
      local best, bd
      for _, e in ipairs(nav) do if e.b.id == old_id then local d = abs(e.i - old_i); if not bd or d < bd then best, bd = e, d end end end
      fe = best
    end
    fe = fe or nav[1]
  end
  self.fe = fe
  self.ce = cmp_id and find(cmp_id, cmp_i) or nil
  if not self.ce then self.cmp_id, self.cmp_i = nil, nil end
  self.cmp_as = cmp_as
  self.vb = self.vb + 1; self.ver = self.ver + 1
end

function V:resolve_cell(cell)
  local rc = {}
  if cell.empty then rc.empty = true; return rc end
  local kit = self.kit
  local r = G.rarity[cell.rarity or 'common'] or RARITY_NONE
  local main = resolve(cell.colour or 'grey', kit)
  rc.main, rc.dark, rc.lite = main, shade(main, 0.52), shade(main, 1.18)
  rc.edge, rc.bt, rc.corners = r.edge, r.bt, r.corners
  rc.pips = max(0, min(4, cell.pips or 0))
  rc.flags = flagbits(cell.flags)
  rc.target = cell.target and true or false
  if type(cell.icon) == 'table' then rc.desc = cell.icon else rc.icon = cell.icon end   -- a descriptor goes to view.icon_draw
  if rc.icon then
    local t = kit and kit.texture and kit.texture('ico_' .. rc.icon)
    if t then rc.icon_w, rc.icon_h = t.w1x, t.h1x else rc.icon = nil end
  end
  return rc
end

-- ------------------------------------------------------------------------------------------------ focus & input
local dirs = { left = true, right = true, up = true, down = true }

-- Move the focus. Rule: the nearest focusable cell strictly in that direction, scored by distance along the
-- direction plus twice the sideways offset (four times when wrapping) (holes and uneven rows are skipped, blocks are crossed in the same
-- row or column). With nothing further that way it WRAPS to the far side, keeping the row (left/right) or the
-- column (up/down) as nearly as it can; `wrap = false` on the view disables the wrap. Returns true if the focus changed.
function V:move(dir)
  local f = self.fe
  if not f or not dirs[dir] then return false end
  local best, bs
  for _, e in ipairs(self.nav) do
    if e ~= f then
      local dx, dy = e.ux - f.ux, e.uy - f.uy
      local prim, perp
      if dir == 'right' then prim, perp = dx, dy elseif dir == 'left' then prim, perp = -dx, dy
      elseif dir == 'down' then prim, perp = dy, dx else prim, perp = -dy, dx end
      if prim > 0.01 then
        local s = prim + 2 * abs(perp)
        if not bs or s < bs then best, bs = e, s end
      end
    end
  end
  if not best and self.wrap then
    for _, e in ipairs(self.nav) do
      if e ~= f then
        local dx, dy = e.ux - f.ux, e.uy - f.uy
        local s
        if dir == 'right' then s = e.ux + 4 * abs(dy) elseif dir == 'left' then s = -e.ux + 4 * abs(dy)
        elseif dir == 'down' then s = e.uy + 4 * abs(dx) else s = -e.uy + 4 * abs(dx) end
        if not bs or s < bs then best, bs = e, s end
      end
    end
  end
  if best then self.fe = best; self.ver = self.ver + 1; return true end
  return false
end

-- Pixel bounds of a cell as last laid out (call after a draw, or it lays out against gd.safe_area()):
-- cell_rect(block_id, index) -> x, y, w, h of the whole cell, then ix, iy, iw, ih of the reserved square icon
-- rectangle (inside the border; pips and flags are drawn over its lower corners). nil for a hole or unknown cell.
function V:cell_rect(block_id, index)
  local L = self.lay or self:layout(self.g.safe_area())
  for _, e in ipairs(self.E) do
    if e.b.id == block_id and e.i == index then
      local s, bt = L.cell, e.rc.bt or 1
      local ix, iw = e.px + bt + 1, s - 2 * bt - 2
      return e.px, e.py, s, s, ix, e.py + bt + 1, iw, iw
    end
  end
end
function V:focused() local f = self.fe; if f then return f.cell, f.b.id, f.i end end
function V:set_focus(block_id, index)
  for _, e in ipairs(self.nav) do if e.b.id == block_id and e.i == index then self.fe = e; self.ver = self.ver + 1; return true end end
  return false
end
-- Compare the focused cell with another cell: partner_is = "before" (the partner is what you have now, the focus is
-- what you would get: a reward) or "after" (the focus is what you have, the partner is incoming: a slot swap).
-- nil clears. The partner may sit in a block that cannot take focus.
function V:set_compare(block_id, index, partner_is)
  self.cmp_id, self.cmp_i, self.cmp_as = block_id, index, partner_is or 'before'
  self.ce = nil
  if block_id then for _, e in ipairs(self.E) do if e.b.id == block_id and e.i == index then self.ce = e end end end
  if not self.ce then self.cmp_id, self.cmp_i = nil, nil end
  self.ver = self.ver + 1
end
function V:set_countdown(seconds, total)
  if seconds == nil then self.cd, self.cd_txt, self.cd_n = nil, nil, nil; return end
  self.cd, self.cd_total = seconds, total or self.cd_total or seconds
  local n = ceil(max(0, seconds))
  if n ~= self.cd_n then self.cd_n = n; self.cd_txt = n .. 's' end
end

-- A button or direction: "left" "right" "up" "down" "A" "B" "X" "Y". Directions return "moved" or nil. A button
-- returns btn, label, cell, block_id, index when the focused cell (or the view's defaults) offers it, else nil.
function V:press(btn)
  if dirs[btn] then return self:move(btn) and 'moved' or nil end
  local f = self.fe
  local cell = f and f.cell
  local act
  if cell and cell.actions and cell.actions[btn] ~= nil then act = cell.actions[btn] else act = self.actions[btn] end
  if act then return btn, act, cell, f and f.b.id, f and f.i end
end

-- ------------------------------------------------------------------------------------------------ pad -> events
-- Edge-detects A B X Y L and turns D-pad/stick into repeated direction events (first after `first` frames, then every
-- `every`). poll(pad) takes a gd.pad-style table and returns a reused list {n=, [1..n] = "left"|"A"|...}.
function G.new_input(opts)
  opts = opts or {}
  local I = { prev = {}, held = 0, dir = nil, out = { n = 0 }, thresh = opts.stick or 60, first = opts.first or 18, every = opts.every or 5 }
  local names = { 'A', 'B', 'X', 'Y', 'L' }
  function I:poll(pad)
    local out = self.out; out.n = 0
    if not pad then self.dir = nil; self.held = 0; return out end
    local sx, sy = pad.x or 0, pad.y or 0
    local d
    local up, down = pad.UP or sy > self.thresh, pad.DOWN or sy < -self.thresh
    local left, right = pad.LEFT or sx < -self.thresh, pad.RIGHT or sx > self.thresh
    if (up or down) and (not (left or right) or abs(sy) >= abs(sx) or pad.UP or pad.DOWN) then d = up and 'up' or 'down'
    elseif left or right then d = left and 'left' or 'right' end
    if d ~= self.dir then self.held = 0; if d then out.n = out.n + 1; out[out.n] = d end
    elseif d then
      self.held = self.held + 1
      if self.held >= self.first and (self.held - self.first) % self.every == 0 then out.n = out.n + 1; out[out.n] = d end
    end
    self.dir = d
    for _, n in ipairs(names) do
      local now = pad[n] and true or false
      if now and not self.prev[n] then out.n = out.n + 1; out[out.n] = n end
      self.prev[n] = now
    end
    return out
  end
  return I
end

-- ------------------------------------------------------------------------------------------------ layout
-- Pure geometry for an area {x, y, w, h} (the safe area): returns the cached layout table
-- { fit, cell, gap, head, bar, grid, detail, blocks = {block -> {x,y,w,h}} } and sets e.px/e.py on every entry.
-- width a block's title needs (measured once per title; cached on the block until the title changes)
function V:title_w(b)
  local t = b.title or ''
  if b._tt ~= t then b._tt = t; b._tw = t == '' and 0 or (self.measure_fn(t, 'label') + 2 * G.cfg.pad + 6) end
  return b._tw
end

function V:layout(a)
  local c = G.cfg
  local L = self.lay
  local ay = a.y or 0
  if L and L.v == self.vb and L.ax == a.x and L.ay == ay and L.aw == a.w and L.ah == a.h then return L end
  L = { v = self.vb, ax = a.x, ay = ay, aw = a.w, ah = a.h, blocks = {}, fit = true }
  local totalw = min(a.w - 2 * c.mx, c.max_w)
  local x0 = a.x + floor((a.w - totalw) / 2)
  local y0, y1 = ay + c.my, ay + a.h - c.my
  L.head = { x = x0, y = y0, w = totalw, h = c.head_h }
  L.bar = { x = x0, y = y1 - c.bar_h, w = totalw, h = c.bar_h }
  local by0, by1 = y0 + c.head_h + 8, y1 - c.bar_h - 8
  local dw = max(c.detail_min, min(c.detail_max, floor(totalw * c.detail_frac)))
  local gw = totalw - dw - c.col_gap
  L.grid = { x = x0, y = by0, w = gw, h = by1 - by0 }
  L.detail = { x = x0 + gw + c.col_gap, y = by0, w = dw, h = by1 - by0 }
  local function need(cell)
    local g = max(2, floor(cell * c.gap)); local bg = max(8, floor(cell * c.block_gap))
    local wmax, h = 0, 0
    for bi, band in ipairs(self.bands) do
      local w, bh = 0, 0
      for k, b in ipairs(band) do
        w = w + max(b.cols * cell + (b.cols - 1) * g + 2 * c.pad, self:title_w(b)) + (k > 1 and bg or 0)
        bh = max(bh, c.title_h + b.rows * cell + (b.rows - 1) * g + 2 * c.pad)
      end
      wmax = max(wmax, w); h = h + bh + (bi > 1 and c.band_gap or 0)
    end
    return wmax, h, g, bg
  end
  local cell = c.max_cell
  local wn, hn, g, bg = need(cell)
  while cell > c.min_cell and (wn > L.grid.w or hn > L.grid.h) do cell = cell - 1; wn, hn, g, bg = need(cell) end
  if wn > L.grid.w or hn > L.grid.h then L.fit = false end
  L.cell, L.gap, L.block_gap, L.need_w, L.need_h = cell, g, bg, wn, hn
  local y = L.grid.y
  for _, band in ipairs(self.bands) do
    local bh = 0
    for _, b in ipairs(band) do bh = max(bh, c.title_h + b.rows * cell + (b.rows - 1) * g + 2 * c.pad) end
    local x = L.grid.x
    for _, b in ipairs(band) do
      local bw = max(b.cols * cell + (b.cols - 1) * g + 2 * c.pad, self:title_w(b))
      L.blocks[b] = { x = x, y = y, w = bw, h = bh }
      x = x + bw + bg
    end
    y = y + bh + c.band_gap
  end
  for _, e in ipairs(self.E) do
    local r = L.blocks[e.b]
    local col, row = (e.i - 1) % e.b.cols, (e.i - 1) // e.b.cols
    e.px = r.x + c.pad + col * (cell + g)
    e.py = r.y + c.title_h + c.pad + row * (cell + g)
  end
  self.lay = L
  self.dver = -1                                  -- the detail text depends on the panel width
  return L
end

-- ------------------------------------------------------------------------------------------------ detail model
local BAR_ORDER = { 'A', 'X', 'Y', 'B' }
local GLYPH = { A = 'glyph_a', B = 'glyph_b', X = 'glyph_x', Y = 'glyph_y' }

function V:model()
  if self.dver == self.ver and self.dlay == self.lay then return self.dm end
  local L = self.lay
  local dm = { lines = {}, tones = {}, n = 0, bar = {}, nbar = 0 }
  local f = self.fe
  local cell = f and f.cell
  local inner = (L and L.detail.w or 240) - 2 * 14
  local function add(text, tone)
    for _, part in ipairs(G.wrap(text, inner, function(t) return self.measure_fn(t, 'body') end)) do
      dm.n = dm.n + 1; dm.lines[dm.n] = part; dm.tones[dm.n] = tone
    end
  end
  local ce = self.ce
  if ce and f and not cell.nocompare then
    dm.cmp = true
    local before, after, ebefore, eafter
    if self.cmp_as == 'after' then before, after, ebefore, eafter = cell, ce.cell, f, ce else before, after, ebefore, eafter = ce.cell, cell, ce, f end
    dm.e_before, dm.e_after = ebefore, eafter
    dm.name_before = (before and not before.empty) and before.name or 'Empty'
    dm.name_after = (after and not after.empty) and after.name or 'Empty'
    for _, l in ipairs(G.compare_lines(not (before and before.empty) and before or nil, not (after and after.empty) and after or nil)) do add(l.text, l.tone) end
  else
    dm.name = cell and (cell.name or (cell.empty and 'Empty slot') or '') or ''
    if cell and cell.empty then dm.muted = true end
    for _, l in ipairs(cell and cell.lines or {}) do add(l, nil) end
  end
  -- action bar: glyph + label per offered action, offsets relative to the bar
  local x = 0
  for _, b in ipairs(BAR_ORDER) do
    local act
    if cell and cell.actions and cell.actions[b] ~= nil then act = cell.actions[b] else act = self.actions[b] end
    if act then
      dm.nbar = dm.nbar + 1
      local w = self.measure_fn(act, 'caption')
      dm.bar[dm.nbar] = { glyph = GLYPH[b], label = act, x = x, w = w }
      x = x + 16 + 4 + w + 18
    end
  end
  dm.bar_w = x
  self.dm, self.dver, self.dlay = dm, self.ver, self.lay
  return dm
end

-- ------------------------------------------------------------------------------------------------ drawing
local function text(self, x, y, s, role, col, align, maxw)
  local g = self.g
  if self.skip & 2 ~= 0 then return end
  if self.kit then
    local o
    if maxw then o = self.o; o.max_w = maxw end
    return g.kit.text(x, y, s, role, col, align, o)
  end
  g.text(x, y - 10, s, 0xF2F2F2FF, 1)
end

local function draw_cell(self, rc, x, y, s, focused, cell)
  local g = self.g
  if rc.empty then
    g.fill(x, y, s, s, CHROME.empty_bg); g.box(x, y, s, s, CHROME.empty_edge)
    local m, t = floor(s / 2), max(2, floor(s / 14)); local arm = floor(s / 6)
    g.fill(x + m - arm, y + m - floor(t / 2), arm * 2, t, CHROME.empty_edge)
    g.fill(x + m - floor(t / 2), y + m - arm, t, arm * 2, CHROME.empty_edge)
    return
  end
  if rc.target then
    g.fill(x, y, s, s, CHROME.empty_bg)
    local q = floor(s / 4)
    g.box(x, y, s, s, CHROME.target); g.box(x + 1, y + 1, s - 2, s - 2, CHROME.target)
    local m, t = floor(s / 2), max(2, floor(s / 12)); local arm = floor(s / 5)
    g.fill(x + m - arm, y + m - floor(t / 2), arm * 2, t, CHROME.target)
    g.fill(x + m - floor(t / 2), y + m - arm, t, arm * 2, CHROME.target)
    return
  end
  local bt = rc.bt
  g.fill(x, y, s, s, rc.edge)
  g.fill(x + bt, y + bt, s - 2 * bt, s - 2 * bt, CHROME.cell_bg)
  local ix, iy, iw = x + bt + 1, y + bt + 1, s - 2 * bt - 2
  g.fill(ix, iy, iw, iw, rc.dark)
  g.fill(ix, iy, iw, floor(iw * 0.6), rc.main)
  if rc.desc and self.icon_draw then
    self.icon_draw(rc.desc, ix, iy, iw, iw, focused, rc.flags & 8 ~= 0, cell)     -- the replaceable icon (see V:cell_rect)
  elseif rc.icon then
    local sc = (iw * 0.5) / max(rc.icon_w, rc.icon_h)
    g.kit.icon(rc.icon, ix + (iw - rc.icon_w * sc) / 2, iy + (iw * 0.6 - rc.icon_h * sc) / 2, sc, CHROME.ink)
  else
    local q = floor(iw * 0.3)
    g.fill(ix + floor((iw - q) / 2), iy + floor((iw * 0.6 - q) / 2), q, q, rc.lite)
  end
  local cs = max(4, floor(s / 8))                   -- rarity notches (shape as well as colour)
  local n = rc.corners
  if n >= 1 then g.fill(x - 1, y - 1, cs, cs, CHROME.mark) end
  if n >= 2 then g.fill(x + s - cs + 1, y + s - cs + 1, cs, cs, CHROME.mark) end
  if n >= 3 then g.fill(x + s - cs + 1, y - 1, cs, cs, CHROME.mark); g.fill(x - 1, y + s - cs + 1, cs, cs, CHROME.mark) end
  local p = rc.pips
  if p > 0 then
    local ps = max(3, floor(s / 11))
    local py = iy + iw - ps - 2
    g.fill(ix, py - 1, p * (ps + 1) + 2, ps + 3, CHROME.shade)
    for k = 0, p - 1 do g.fill(ix + 1 + k * (ps + 1), py, ps, ps, CHROME.mark) end
  end
  local fl = rc.flags
  if fl ~= 0 then
    if fl & 4 ~= 0 then g.fill(ix, iy, iw, max(3, floor(s / 14)), CHROME.tab) end
    if fl & 1 ~= 0 then
      local nw = max(7, floor(s / 5))
      g.fill(x + s - nw - bt, y + bt, nw, nw, CHROME.new)
      g.fill(x + s - nw - bt + 2, y + bt + 2, nw - 4, nw - 4, CHROME.ink)
      g.fill(x + s - nw - bt + 3, y + bt + 3, nw - 6, nw - 6, CHROME.new)
    end
    if fl & 2 ~= 0 then
      local arm, t = max(4, floor(s / 8)), max(2, floor(s / 18))
      local cx, cy = ix + iw - arm - 3, iy + iw - arm - 3
      g.fill(cx - arm - 2, cy - arm - 2, arm * 2 + 5, arm * 2 + 5, CHROME.shade)
      g.fill(cx - arm, cy - floor(t / 2), arm * 2 + 1, t, CHROME.merge)
      g.fill(cx - floor(t / 2), cy - arm, t, arm * 2 + 1, CHROME.merge)
    end
    if fl & 8 ~= 0 then
      g.fill(ix, iy, iw, iw, CHROME.lockdim)
      if self.lock_w then
        local sc = (s * 0.42) / self.lock_w
        g.kit.icon('lock', x + (s - self.lock_w * sc) / 2, y + (s - self.lock_h * sc) / 2, sc, 'bone')
      end
    end
  end
end
G.draw_cell = draw_cell

function V:draw(a)
  local g = self.g
  a = a or g.safe_area()
  local L = self:layout(a)
  local dm = self:model()
  local fr = g.frame and g.frame() or 0
  local pulse = 0xA0 + floor(0x5F * (0.5 + 0.5 * math.sin(fr * 0.12)))
  local kit = self.kit
  g.fill(a.x, a.y or 0, a.w, a.h, CHROME.dim)
  -- title and countdown
  local h = L.head
  text(self, h.x, h.y + 20, self.title, 'heading', 'bone', 'left', h.w - 90)
  if self.cd_txt then
    local urgent = self.cd <= 5
    text(self, h.x + h.w, h.y + 20, self.cd_txt, 'heading', urgent and 'danger' or 'gold', 'right')
    local frac = (self.cd_total and self.cd_total > 0) and max(0, min(1, self.cd / self.cd_total)) or 0
    g.fill(h.x, h.y + h.h - 3, h.w, 3, CHROME.bar_back)
    g.fill(h.x, h.y + h.h - 3, floor(h.w * frac), 3, urgent and 0xE5483BFF or self.gold)
  else
    g.fill(h.x, h.y + h.h - 2, h.w, 2, CHROME.bar_back)
  end
  -- blocks
  local fb = self.fe and self.fe.b
  for _, band in ipairs(self.bands) do
    for _, b in ipairs(band) do
      local r = L.blocks[b]
      if self.skip & 1 == 0 then
        g.fill(r.x, r.y, r.w, r.h, CHROME.block_bg); g.box(r.x, r.y, r.w, r.h, CHROME.block_edge)
        g.fill(r.x + 1, r.y + G.cfg.title_h - 2, r.w - 2, 1, CHROME.block_edge)
      end
      text(self, r.x + G.cfg.pad + 2, r.y + G.cfg.title_h - 4, b.title or '', 'label', b == fb and 'gold' or 'muted', 'left', r.w - 2 * G.cfg.pad)
    end
  end
  local s = L.cell
  if self.skip & 4 == 0 then local fe0 = self.fe; for _, e in ipairs(self.E) do draw_cell(self, e.rc, e.px, e.py, s, e == fe0, e.cell) end end
  -- compare partner and focus cursor
  local ce, fe = self.ce, self.fe
  if ce and ce ~= fe then
    g.box(ce.px - 3, ce.py - 3, s + 6, s + 6, CHROME.compare); g.box(ce.px - 4, ce.py - 4, s + 8, s + 8, CHROME.compare)
  end
  if fe then
    local tint = (self.gold & 0xFFFFFF00) | pulse
    g.box(fe.px - 3, fe.py - 3, s + 6, s + 6, tint); g.box(fe.px - 4, fe.py - 4, s + 8, s + 8, tint); g.box(fe.px - 5, fe.py - 5, s + 10, s + 10, tint)
  end
  -- detail panel
  local d = L.detail
  if self.skip & 1 ~= 0 then elseif kit then kit.panel(d.x, d.y, d.w, d.h, self.st_detail) else g.box(d.x, d.y, d.w, d.h, 0x8A97A3FF) end
  local pad = 14
  local ty
  if dm.cmp then
    local sw = 36
    local eb, ea = dm.e_before, dm.e_after
    draw_cell(self, eb.rc, d.x + pad, d.y + pad, sw, false, eb.cell)
    text(self, d.x + pad + sw + 8, d.y + pad + 25, '->', 'heading', 'gold', 'left')
    draw_cell(self, ea.rc, d.x + pad + sw + 40, d.y + pad, sw, false, ea.cell)
    text(self, d.x + pad, d.y + pad + sw + 16, dm.name_before, 'body', 'muted', 'left', d.w - 2 * pad)
    text(self, d.x + pad, d.y + pad + sw + 16 + self.line_h, dm.name_after, 'body', 'bone', 'left', d.w - 2 * pad)
    ty = d.y + pad + sw + 16 + self.line_h * 2 + 12
  else
    local sw = 44
    if fe then draw_cell(self, fe.rc, d.x + pad, d.y + pad, sw, true, fe.cell) end
    text(self, d.x + pad + sw + 10, d.y + pad + 30, dm.name, 'heading', dm.muted and 'muted' or 'bone', 'left', d.w - 2 * pad - sw - 10)
    ty = d.y + pad + sw + 24
  end
  local pitch = self.line_h + 3
  local room = floor((d.y + d.h - pad - ty) / pitch)
  local n = min(dm.n, max(0, room))
  for i = 1, n do text(self, d.x + pad, ty + (i - 1) * pitch, dm.lines[i], 'body', dm.tones[i] or 'bone', 'left', d.w - 2 * pad) end
  if n < dm.n then text(self, d.x + d.w - pad, d.y + d.h - 8, '...', 'caption', 'muted', 'right') end
  -- action bar
  local b = L.bar
  if self.skip & 1 == 0 then g.fill(b.x, b.y, b.w, b.h, CHROME.block_bg); g.box(b.x, b.y, b.w, b.h, CHROME.block_edge) end
  local x = b.x + 14
  for i = 1, dm.nbar do
    local it = dm.bar[i]
    if kit and self.skip & 8 == 0 then g.kit.image(it.glyph, x + it.x, b.y + (b.h - 16) / 2) end
    text(self, x + it.x + 20, b.y + b.h / 2 + 5, it.label, 'caption', 'bone', 'left')
  end
  if self.hint then text(self, b.x + b.w - 14, b.y + b.h / 2 + 5, self.hint, 'caption', 'muted', 'right', b.w - dm.bar_w - 40) end
end

return G
end)()
-- END GENERATED GRID

local NAMES = { 'gem_octa', 'gem_cube', 'gem_tprism', 'gem_hexbi', 'gem_icosa', 'gem_hexprism',
                'gem_pyramid', 'gem_tetra', 'gem_coin', 'gem_wide', 'gem_torus', 'gem_cone' }
local LABELS = { 'Octa', 'Cube', 'Prism', 'Hexa', 'Icosa', 'Hex bar', 'Pyramid', 'Tetra', 'Coin', 'Wide', 'Torus', 'Cone' }
local COLOURS = { 'red', 'green', 'blue', 'yellow', 'white', 'red', 'green', 'blue', 'yellow', 'white', 'cyan', 'violet' }

local assets, loaded, load_tick = {}, false, 0
local mode, focus, shown, masked = 1, 1, true, false
local COLS, ROWS = 4, 3
local view                                   -- the grid component (mode 2)
local pending = {}                           -- console-injected presses

local function load_models()
  for i, n in ipairs(NAMES) do
    local ok, h = pcall(gd.model_load, n)
    if not ok then gd.log('screen-models: ' .. tostring(h)); return false end
    assets[i] = h
  end
  loaded = true
  gd.log('screen-models: ' .. #NAMES .. ' models loaded')
  return true
end

local function set_mask(on)
  if gd.input_mask and on ~= masked then masked = (gd.input_mask(1, on and 15 or 0) ~= false) and on end
end

-- ------------------------------------------------------------------------------------------ mode 2: the grid
local function grid_cells()
  local cells, rar = {}, { 'common', 'uncommon', 'rare', 'unique' }
  for i = 1, #NAMES do
    cells[i] = {
      name = LABELS[i] .. ' drive', colour = COLOURS[i], rarity = rar[(i - 1) % 4 + 1], pips = (i - 1) % 4 + 1,
      flags = (i == 1 and { 'new' }) or (i == 7 and { 'locked' }) or (i == 3 and { 'merge' }) or nil,
      icon = { kind = 'model', asset = i },
      lines = { LABELS[i] .. ': a ' .. NAMES[i]:sub(5) .. ' test model', 'drawn by gd.kit.model inside the cell' },
      actions = { A = 'Equip', B = 'Close' },
    }
  end
  return cells
end

local function boot_grid()
  -- THE ADOPTION LINE: the grid hands icon_draw the cell's icon rectangle; this is all a mod needs to pass.
  local function icon_draw(desc, x, y, w, h, focused, locked)
    gd.kit.model(assets[desc.asset], x, y, w, h, { yaw = 20, pitch = 12, spin = focused and 120 or 0, dim = locked and 0.35 or 1 })
  end
  view = grid.new{ g = gd, title = 'SCREEN MODELS', hint = 'TAB: cells', icon_draw = icon_draw,
                   actions = { B = 'Close' },
                   blocks = { { id = 'bag', title = 'BAG 12/12', cols = COLS, rows = ROWS, cells = grid_cells() } } }
end

-- ------------------------------------------------------------------------------------------ mode 1: the cells
local function draw_cells()
  local sa = gd.safe_area()
  local gap = 12
  local cell = math.floor(math.min((sa.w - 96 - (COLS - 1) * gap) / COLS, (480 - 150 - (ROWS - 1) * gap) / ROWS))
  local gw, gh = COLS * cell + (COLS - 1) * gap, ROWS * cell + (ROWS - 1) * gap
  local x0, y0 = sa.x + (sa.w - gw) / 2, 74
  gd.text(x0, 40, 'SCREEN MODELS: mode 1, cells (arrows move, TAB: grid)', 0xF2EFE4FF, 1)
  for i = 1, #NAMES do
    local c, r = (i - 1) % COLS, (i - 1) // COLS
    local x, y = x0 + c * (cell + gap), y0 + r * (cell + gap)
    local f = i == focus
    gd.fill(x, y, cell, cell, f and 0x20324CFF or 0x101826F0)                        -- 1. the cell background
    local opts = { yaw = 25 + i * 20, pitch = 14, margin = 0.1 }
    if f then opts.yaw = 0; opts.spin = 90 end                                       -- the focused cell spins
    if i == 7 then opts.dim = 0.3 end                                                -- locked
    if i == 8 then opts.alpha = 0.45 end                                             -- ghost
    if i == 12 then opts.clip = false; opts.spin = 40 end                            -- clip off: for comparison
    gd.kit.model(assets[i], x + 3, y + 3, cell - 6, cell - 6, opts)                  -- 2. the model
    gd.box(x, y, cell, cell, f and 0xEBD175FF or 0x5B7396FF)                         -- 3. the frame, over the model's edge
    gd.fill(x, y, 22, 16, 0x000000C0)                                                -- 4. a badge, over the model's corner
    gd.text(x + 4, y + 2, tostring(i), 0xFFFFFFFF, 1)
    gd.text(x + 4, y + cell - 16, LABELS[i], 0xF2EFE4FF, 1)                          -- 5. the label, over the model's foot
  end
end

-- ------------------------------------------------------------------------------------------ input
local function move(dc, dr)
  local c, r = (focus - 1) % COLS, (focus - 1) // COLS
  c, r = (c + dc) % COLS, (r + dr) % ROWS
  focus = r * COLS + c + 1
end
local function handle(ev)
  if ev == 'L' then mode = mode == 1 and 2 or 1; return end
  if mode == 2 then view:press(ev); return end
  if ev == 'left' then move(-1, 0) elseif ev == 'right' then move(1, 0)
  elseif ev == 'up' then move(0, -1) elseif ev == 'down' then move(0, 1) end
end

local input = grid.new_input()
local KEYS = { { 'LEFT', 'left' }, { 'RIGHT', 'right' }, { 'UP', 'up' }, { 'DOWN', 'down' }, { 'ENTER', 'A' }, { 'BACKSPACE', 'B' }, { 'TAB', 'L' } }

-- ------------------------------------------------------------------------------------------ benchmark
-- sm_bench all: 0 (baseline), 16, 48, 0 models for 150 frames each; logs the script time of on_draw and the host's
-- frame times (gd.perf). The models are drawn in a grid over the whole canvas, cycling through the twelve assets.
local bench
local function bench_start(list, frames)
  gd.perf(1)                                -- sampling starts on the first read
  bench = { list = list, i = 1, frames = frames, n = 0, ms = {}, perf0 = nil, warm = 30, left = frames }
end
local function pct(t, p)
  local s = {}; for i = 1, #t do s[i] = t[i] end
  table.sort(s); return s[math.max(1, math.ceil(#s * p))] or 0
end
local function bench_draw()
  local count = bench.list[bench.i]
  gd.perf(1)                                -- keeps the host sampling
  local sa = gd.safe_area()
  local t0 = gd.time()
  if count > 0 then
    local cols = math.ceil(math.sqrt(count * sa.w / 480))
    local rows = math.ceil(count / cols)
    local cw, ch = sa.w / cols, 480 / rows
    for k = 0, count - 1 do
      local c, r = k % cols, k // cols
      gd.kit.model(assets[k % #NAMES + 1], c * cw, r * ch, cw, ch, { yaw = 30, pitch = 12, spin = 90 })
    end
  end
  local ms = (gd.time() - t0) * 1000
  if bench.warm > 0 then bench.warm = bench.warm - 1; return end
  bench.ms[#bench.ms + 1] = ms
  bench.left = bench.left - 1
  if bench.left <= 0 then
    local sum = 0; for i = 1, #bench.ms do sum = sum + bench.ms[i] end
    local mean = sum / #bench.ms
    local p = gd.perf(bench.frames)
    local total, logic, gx, n = 0, 0, 0, 0
    local worker = 0
    for _, fr in ipairs(p) do total = total + fr.total_ms; logic = logic + fr.logic_ms; gx = gx + fr.gx_ms; worker = worker + (fr.worker_ms or 0); n = n + 1 end
    if n == 0 then n = 1 end
    gd.log(('bench models=%d calls: script on_draw mean=%.4f ms p95=%.4f ms max=%.4f ms (%.2f us per call) | frame total=%.3f logic=%.3f gx=%.3f worker=%.3f ms over %d frames, fps=%s canvas=%.0fx480')
      :format(count, mean, pct(bench.ms, 0.95), pct(bench.ms, 1), count > 0 and mean * 1000 / count or 0,
              total / n, logic / n, gx / n, worker / n, n, tostring(p.fps), sa.w))
    bench.i = bench.i + 1; bench.ms = {}; bench.warm = 30; bench.left = bench.frames
    if bench.i > #bench.list then bench = nil; gd.log('bench done') end
  end
end

gd.command('sm_mode', function(a) mode = tonumber(a) == 2 and 2 or 1 end, 'sm_mode <1|2>')
gd.command('sm_focus', function(a) focus = math.max(1, math.min(#NAMES, tonumber(a) or 1)); if view then view.focus_to = nil end end, 'sm_focus <n>')
gd.command('sm_press', function(a) for w in a:gmatch('%S+') do pending[#pending + 1] = w end end, 'sm_press <left|right|up|down|A|B|L> ...')
gd.command('sm_bench', function(a)
  local frames = 150
  if a == 'all' or a == '' then bench_start({ 0, 16, 48, 0 }, frames)
  else local n, f = a:match('(%d+)%s*(%d*)'); bench_start({ tonumber(n) or 16 }, tonumber(f) or frames) end
end, 'sm_bench [all|n [frames]]')
gd.command('sm_shot', function(a) local ok, p = gd.screenshot(a ~= '' and a or 'sm') gd.log('shot ' .. tostring(ok)) end, 'sm_shot <name>')

function on_tick()
  if not loaded then
    load_tick = load_tick + 1
    if load_tick % 30 == 1 then load_models() end
    if not loaded then return end
  end
  if not view then boot_grid() end
  if gd.key_pressed('F6') then shown = not shown end
  if shown then
    set_mask(true)
    local ev = input:poll(gd.pad(1, true))
    for i = 1, ev.n do handle(ev[i]) end
    for _, k in ipairs(KEYS) do if gd.key_pressed(k[1]) then handle(k[2]) end end
  else
    set_mask(false)
  end
  for i = 1, #pending do handle(pending[i]); pending[i] = nil end
end

function on_draw()
  if not shown or not loaded or not view then return end
  if bench then return bench_draw() end
  if mode == 1 then draw_cells() else view:draw() end
end

function on_unload() set_mask(false) end
