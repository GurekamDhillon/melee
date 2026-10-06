-- A gd.ui stand-in for offline tests (Atlas step 1). It enforces the contract of gw_ui_screen.c (the limits are read from
-- that header so the two cannot drift) and plays the engine's part where a test needs it: focus by cell id, on.focus, the
-- explainer / key label / counter refresh, and accept / back / alt dispatch.
local Stub = {}

local function read_limits()
 local here = (arg and arg[0] or ''):gsub('\\', '/'):gsub('[^/]*$', '')
 for _, p in ipairs({ here .. '../platform/gw_ui_screen.h', 'pc/platform/gw_ui_screen.h', 'melee/pc/platform/gw_ui_screen.h' }) do
  local f = io.open(p)
  if f then
   local text = f:read('a'); f:close()
   local L = {}
   for name, v in text:gmatch('#define%s+AT_MAX_(%u+)%s+(%d+)') do L[name:lower()] = tonumber(v) end
   L.id = tonumber(text:match('#define%s+AT_ID%s+(%d+)'))   -- a block, cell or item id holds AT_ID - 1 characters
   return L
  end
 end
 error('gw_ui_screen.h not found')
end
Stub.limits = read_limits()
local BUTTONS = { A = true, B = true, X = true, Y = true, Z = true, L = true, R = true, START = true }

local function disabled(c) return (c.flags and c.flags.disabled) or c.disabled end

function Stub.new(opts)
 opts = opts or {}
 local L = Stub.limits
 local ui = { screens = {}, stack = {}, views = {}, fed = {}, notes = {}, dialogs = {}, refreshed = 0, _f = {}, available_ok = opts.available ~= false }
 local function fail(msg) error('gd.ui.screen: ' .. msg, 3) end
 local function check_id(what, id)
  if type(id) ~= 'string' or id == '' then fail(what .. ' has no id') end
  if L.id and #id >= L.id then fail(('%s: id is too long (%d characters at most)'):format(what, L.id - 1)) end
 end

 -- every focusable cell in focus order: { block =, id =, cell =, index = (1-based within its block) }
 local function flat(d)
  local out, p = {}, d.primary
  if p.kind == 'grid' then
   for _, b in ipairs(p.blocks) do for i, c in ipairs(b.cells or {}) do out[#out + 1] = { block = b.id, id = c.id, cell = c, index = i } end end
  else
   for i, it in ipairs(p.items) do out[#out + 1] = { block = 'list', id = it.id, cell = it, index = i } end
  end
  return out
 end

 function ui.available() return ui.available_ok, ui.available_ok and '' or 'stub: unavailable' end

 function ui.screen(d)
  if type(d) ~= 'table' then fail('a screen description is a table') end
  if type(d.id) ~= 'string' or d.id == '' then fail('the description has no id') end
  local p = d.primary
  if type(p) ~= 'table' then fail('"' .. d.id .. '" has no primary') end
  local seen = {}
  if p.kind == 'grid' then
   local n = #(p.blocks or {})
   if n < 1 or n > L.blocks then fail(('a grid needs 1 to %d blocks (it has %d)'):format(L.blocks, n)) end
   local blocks = {}
   for _, b in ipairs(p.blocks) do
    check_id('a block', b.id)
    if blocks[b.id] then fail('duplicate block id "' .. b.id .. '"') end
    blocks[b.id] = true
    if (b.cols or 1) < 1 or (b.cols or 1) > L.cells then fail('block "' .. b.id .. '": cols is 1 to ' .. L.cells) end
    if #(b.cells or {}) > L.cells then fail(('block "%s" has %d cells (%d at most)'):format(b.id, #b.cells, L.cells)) end
    for _, c in ipairs(b.cells or {}) do
     check_id('a cell', c.id)
     if seen[c.id] then fail('duplicate cell id "' .. c.id .. '"') end
     seen[c.id] = true
    end
   end
  elseif p.kind == 'list' then
   local n = #(p.items or {})
   if n < 1 or n > L.items then fail(('a list needs 1 to %d items (it has %d)'):format(L.items, n)) end
   for _, it in ipairs(p.items) do
    check_id('an item', it.id)
    if seen[it.id] then fail('duplicate item id "' .. it.id .. '"') end
    seen[it.id] = true
    if type(it.value) == 'table' and it.value.kind == 'slider' and not ((it.value.min or 0) < (it.value.max or 100)) then
     fail('item "' .. it.id .. '": slider min must be below max')
    end
   end
  else
   fail(('primary kind "%s" is not supported here (grid or list)'):format(tostring(p.kind)))
  end
  if d.keys and #d.keys > L.keys then fail(('at most %d key hints (%d given)'):format(L.keys, #d.keys)) end
  for i, k in ipairs(d.keys or {}) do
   local b = k[1] or k.btn
   if not BUTTONS[b] then fail(('key hint %d: unknown button "%s" (A B X Y Z L R START)'):format(i, tostring(b))) end
  end
  local old = ui.screens[d.id] and ui._f[d.id]
  ui.screens[d.id] = d
  ui.refreshed = ui.refreshed + 1
  local all, pick = flat(d), nil
  if old then
   for _, e in ipairs(all) do if e.block == old.block and e.id == old.cell then pick = e end end
   if not pick then   -- gone: the same block, the same place clamped (at_screen_refocus)
    local inblock = {}
    for _, e in ipairs(all) do if e.block == old.block then inblock[#inblock + 1] = e end end
    if #inblock > 0 then pick = inblock[math.min(old.index or 1, #inblock)] end
   end
  end
  pick = pick or all[1]
  ui._f[d.id] = pick and { block = pick.block, cell = pick.id, index = pick.index } or nil
  return true
 end

 function ui.open(id) assert(ui.screens[id], 'gd.ui: no screen "' .. tostring(id) .. '"'); ui.stack[#ui.stack + 1] = id; ui.refresh(id); return true end
 function ui.close(id)
  id = id or ui.stack[#ui.stack]
  for i = #ui.stack, 1, -1 do if ui.stack[i] == id then table.remove(ui.stack, i) end end
  return id ~= nil
 end
 function ui.feed(id, intent) assert(ui.screens[id], 'gd.ui: no screen "' .. tostring(id) .. '"'); ui.fed[#ui.fed + 1] = { id, intent }; return true end
 function ui.focus(id)
  assert(ui.screens[id], 'gd.ui: no screen "' .. tostring(id) .. '"')
  local f = ui._f[id]
  if not f then return nil end
  return f.cell, f.block
 end
 function ui.set_focus(id, block, cell)
  assert(ui.screens[id], 'gd.ui: no screen "' .. tostring(id) .. '"')
  for _, e in ipairs(flat(ui.screens[id])) do
   if e.block == block and e.id == cell then ui._f[id] = { block = block, cell = cell, index = e.index }; return true end
  end
  return false
 end
 function ui.note(t) ui.notes[#ui.notes + 1] = t; return #ui.stack > 0 end
 function ui.dialog(t) ui.dialogs[#ui.dialogs + 1] = t; return #ui.stack > 0 end
 function ui.state() return { depth = #ui.stack, top = ui.stack[#ui.stack], roles_ok = ui.available_ok } end

 -- ---- the engine's part -------------------------------------------------------------------------------------
 function ui.refresh(id)
  local d, f = ui.screens[id], ui._f[id]
  local cell, block = f and f.cell, f and f.block
  local v = { keys = {} }
  local ex = d.explainer
  if type(ex) == 'table' and type(ex.provide) == 'function' and cell then v.explainer = ex.provide(cell, block) end
  for _, k in ipairs(d.keys or {}) do
   local label = k[2] or k.label
   if type(label) == 'function' then label = label(cell, block) end
   local show = true
   if type(k.when) == 'function' then show = k.when(cell, block) and true or false end
   if show and type(label) == 'string' and label ~= '' then v.keys[#v.keys + 1] = { k[1] or k.btn, label } end
  end
  local c = d.counter
  if type(c) == 'function' then v.counter = c(cell, block) else v.counter = c end
  ui.views[id] = v
  return v
 end

 function ui.engine_focus(id, block, cell)
  local f = ui._f[id]
  if f and f.block == block and f.cell == cell then return false end
  assert(ui.set_focus(id, block, cell), 'no cell ' .. block .. ':' .. cell)
  local on = ui.screens[id].on or {}
  if on.focus then on.focus(cell, block) end      -- the handler may re-register the screen: the refresh below reads the newest one
  ui.refresh(id)
  return true
 end

 function ui.engine_press(id, kind)
  local d, f = ui.screens[id], ui._f[id]
  local on = d.on or {}
  local fn
  if kind == 'accept' then fn = on.accept elseif kind == 'back' then fn = on.back
  elseif kind == 'x' or kind == 'y' or kind == 'z' then fn = on.alt and on.alt[kind:upper()] end
  if kind == 'accept' and f then
   for _, e in ipairs(flat(d)) do if e.block == f.block and e.id == f.cell and disabled(e.cell) then return false end end
  end
  if not fn then return false end
  local r = fn(f and f.cell, f and f.block)
  if ui.screens[id] then ui.refresh(id) end
  return true, r
 end

 return ui
end

return Stub
