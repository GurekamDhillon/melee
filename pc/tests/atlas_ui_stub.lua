-- A gd.ui stand-in for offline tests (Atlas step 1). It enforces the contract of gw_ui_screen.c and gw_script_ui.inc (the
-- numeric limits are read from gw_ui_screen.h so the two cannot drift; the rules are copied from the C file and named by the
-- message the C side gives) and plays the engine's part where a test needs it: focus by cell id, on.focus, the explainer / key
-- label / counter refresh, accept / back / alt / page / start dispatch, value rows (on.change), {pop=}/{push=} results, a
-- held-button model of the pad, and the ownership rule (a screen belongs to the script that registered it).
--
-- What it does NOT model: layout, drawing, the quad budget, mouse and keyboard, the 8-slot table, and the Lua registry.
-- Everything in the conversion arena IS modelled (node, entry and string-pool ceilings and the depth limit), because a stand-in
-- that accepts a description the engine refuses defeats its purpose.
local Stub = {}

local function read_limits()
 local here = (arg and arg[0] or ''):gsub('\\', '/'):gsub('[^/]*$', '')
 for _, p in ipairs({ here .. '../platform/gw_ui_screen.h', 'pc/platform/gw_ui_screen.h', 'melee/pc/platform/gw_ui_screen.h' }) do
  local f = io.open(p)
  if f then
   local text = f:read('a'); f:close()
   local L = {}
   for name, v in text:gmatch('#define%s+AT_MAX_(%u+)%s+(%d+)') do L[name:lower()] = tonumber(v) end
   L.id = tonumber(text:match('#define%s+AT_ID%s+(%d+)'))     -- a block, cell or item id holds AT_ID - 1 characters; a screen id twice that
   L.str = tonumber(text:match('#define%s+AT_STR%s+(%d+)'))
   L.text = tonumber(text:match('#define%s+AT_TEXT%s+(%d+)'))
   local v = io.open((p:gsub('gw_ui_screen%.h$', 'gw_ui_val.h')))
   if v then
    local vt = v:read('a'); v:close()
    L.nodes = tonumber(vt:match('#define%s+ATV_MAX_NODES%s+(%d+)'))
    L.entries = tonumber(vt:match('#define%s+ATV_MAX_ENTRIES%s+(%d+)'))
    L.pool = tonumber(vt:match('#define%s+ATV_POOL%s+(%d+)'))
   end
   return L
  end
 end
 error('gw_ui_screen.h not found')
end
Stub.limits = read_limits()
local BUTTONS = { A = true, B = true, X = true, Y = true, Z = true, L = true, R = true, START = true }
local INTENTS = { up = true, down = true, left = true, right = true, accept = true, back = true, x = true, y = true, z = true, l = true, r = true, start = true }
local PAD = { A = 'accept', B = 'back', X = 'x', Y = 'y', Z = 'z', L = 'l', R = 'r', START = 'start' }
local PAD_ORDER = { 'A', 'B', 'X', 'Y', 'Z', 'L', 'R', 'START' }
local MAX_DEPTH = 8

local function disabled(c) return (c.flags and c.flags.disabled) or c.disabled end
local function clip(s, cap) s = tostring(s or ''); if #s > cap - 1 then return s:sub(1, cap - 1), true end; return s, false end

-- The conversion arena (gw_ui_val.h / gs_ui_copy): raises the message the binding raises when a description does not fit.
local function check_arena(d, L)
 local nodes, entries, pool = 0, 0, 0
 local function bust() error('gd.ui.screen: the description is too large or nested too deeply', 0) end
 local function node() nodes = nodes + 1; if nodes > L.nodes then bust() end end
 local function walk(v, depth)
  if depth > MAX_DEPTH then bust() end
  local t = type(v)
  if t == 'boolean' or t == 'number' or t == 'function' then node()
  elseif t == 'string' then node(); pool = pool + #v + 1; if pool > L.pool then bust() end
  elseif t == 'table' then
   node()
   for _, x in ipairs(v) do entries = entries + 1; if entries > L.entries then bust() end; walk(x, depth + 1) end
   for k, x in pairs(v) do
    if type(k) == 'string' then
     pool = pool + #k + 1; if pool > L.pool then bust() end
     entries = entries + 1; if entries > L.entries then bust() end
     walk(x, depth + 1)
    end
   end
  end
 end
 walk(d, 0)
end

function Stub.new(opts)
 opts = opts or {}
 local L = Stub.limits
 local ui = { screens = {}, stack = {}, views = {}, fed = {}, notes = {}, dialogs = {}, refreshed = 0, _f = {}, _owner = {}, _rows = {},
  available_ok = opts.available ~= false, caller = opts.caller or 'console', owner_mod = opts.owner_mod,
  held = {}, _prev = {} }
 local function fail(msg) error('gd.ui.screen: ' .. msg, 3) end
 local function check_id(what, id)
  if type(id) ~= 'string' or id == '' then fail(what .. ' has no id') end
  if L.id and #id >= L.id then fail(('%s: id is too long (%d characters at most)'):format(what, L.id - 1)) end
 end
 local function is_console() return ui.caller == 'console' end
 local function may_touch(id) return is_console() or ui._owner[id] == ui.caller end
 local function own(id)
  assert(ui.screens[id], 'gd.ui: no screen "' .. tostring(id) .. '"')
  if not may_touch(id) then error('gd.ui: screen "' .. id .. '" belongs to another script', 3) end
 end
 local function top() return ui.stack[#ui.stack] end

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
  check_arena(d, L)
  if type(d.id) ~= 'string' or d.id == '' then fail('the description has no id') end
  if ui.owner_mod and not is_console() then
   if d.id:sub(1, #ui.owner_mod + 1) ~= ui.owner_mod .. '.' then fail(('id "%s" must start with "%s."'):format(d.id, ui.owner_mod)) end
  end
  if #d.id >= L.id * 2 then fail(('id "%s" is too long (%d characters at most)'):format(d.id, L.id * 2 - 1)) end
  if ui._owner[d.id] and not may_touch(d.id) then fail('"' .. d.id .. '" belongs to another script') end
  local chapter = d.chapter or 0
  if type(chapter) ~= 'number' or chapter < 0 or chapter > 5 then fail('chapter is 0 to 5') end
  local p = d.primary
  if type(p) ~= 'table' then fail('"' .. d.id .. '" has no primary') end
  local seen = {}
  if p.kind == 'grid' then
   local n = #(p.blocks or {})
   if n < 1 or n > L.blocks then fail(('a grid needs 1 to %d blocks (it has %d)'):format(L.blocks, n)) end
   local blocks = {}
   for bi, b in ipairs(p.blocks) do
    if type(b) ~= 'table' then fail(('block %d is not a table'):format(bi)) end
    check_id(('block %d'):format(bi), b.id)
    if blocks[b.id] then fail('duplicate block id "' .. b.id .. '"') end
    blocks[b.id] = true
    if (b.cols or 1) < 1 or (b.cols or 1) > L.cells then fail('block "' .. b.id .. '": cols is 1 to ' .. L.cells) end
    if #(b.cells or {}) > L.cells then fail(('block "%s" has %d cells (%d at most)'):format(b.id, #b.cells, L.cells)) end
    for ci, c in ipairs(b.cells or {}) do
     if type(c) ~= 'table' then fail(('block "%s" cell %d is not a table'):format(b.id, ci)) end
     check_id(('block "%s" cell %d'):format(b.id, ci), c.id)
     if seen[c.id] then fail('duplicate cell id "' .. c.id .. '"') end
     seen[c.id] = true
    end
   end
  elseif p.kind == 'list' then
   local n = #(p.items or {})
   if n < 1 or n > L.items then fail(('a list needs 1 to %d items (it has %d)'):format(L.items, n)) end
   for ii, it in ipairs(p.items) do
    if type(it) ~= 'table' then fail(('item %d is not a table'):format(ii)) end
    check_id(('item %d'):format(ii), it.id)
    if seen[it.id] then fail('duplicate item id "' .. it.id .. '"') end
    seen[it.id] = true
    if type(it.value) == 'table' then
     local k = it.value.kind
     if k == 'slider' then
      if not ((it.value.min or 0) < (it.value.max or 100)) then fail('item "' .. it.id .. '": slider min must be below max') end
     elseif k ~= 'toggle' and k ~= 'choice' and k ~= 'text' and k ~= 'counter' then
      fail(('item "%s": unknown value kind "%s"'):format(it.id, tostring(k)))
     end
    end
   end
  else
   fail(('primary kind "%s" is not supported here (grid or list)'):format(tostring(p.kind)))
  end
  local ex = d.explainer
  if type(ex) == 'string' then
   if ex ~= 'none' then fail('explainer must be a table or "none"') end
  elseif type(ex) == 'table' then
   local w = ex.width or 'normal'
   if w ~= 'narrow' and w ~= 'normal' and w ~= 'wide' then fail(('explainer width "%s" is narrow, normal or wide'):format(tostring(w))) end
  end
  if d.keys and #d.keys > L.keys then fail(('at most %d key hints (%d given)'):format(L.keys, #d.keys)) end
  for i, k in ipairs(d.keys or {}) do
   if type(k) ~= 'table' then fail(('key hint %d is not a table'):format(i)) end
   local b = k[1] or k.btn
   if not BUTTONS[b] then fail(('key hint %d: unknown button "%s" (A B X Y Z L R START)'):format(i, tostring(b))) end
  end
  local port = d.port or 1
  if type(port) ~= 'number' or port < 1 or port > 4 then fail('port is 1 to 4') end
  local old = ui.screens[d.id] and ui._f[d.id]
  ui.screens[d.id] = d
  ui._owner[d.id] = ui._owner[d.id] or ui.caller
  ui._rows[d.id] = {}
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

 -- A screen that becomes the top one starts from what is held right now (gs_ui_prime): a press must be released first.
 local function prime()
  ui._prev = {}
  for b, v in pairs(ui.held) do ui._prev[b] = v end
 end

 function ui.open(id)
  own(id)
  if top() == id then return false end                                          -- already on top is a refusal (at_stack_push)
  ui.stack[#ui.stack + 1] = id; prime(); ui.refresh(id); return true
 end
 function ui.close(id)
  if id then own(id) else
   id = top()
   if id and not may_touch(id) then return false end
  end
  local was_top = top() == id
  for i = #ui.stack, 1, -1 do if ui.stack[i] == id then table.remove(ui.stack, i) end end
  if was_top and top() then prime() end
  return id ~= nil
 end
 function ui.feed(id, intent)
  own(id)
  if not INTENTS[intent] then error('gd.ui.feed: unknown intent "' .. tostring(intent) .. '" (up down left right accept back x y z l r start)', 2) end
  ui.fed[#ui.fed + 1] = { id, intent }; return true
 end
 function ui.focus(id)
  own(id)
  local f = ui._f[id]
  if not f then return nil end
  return f.cell, f.block
 end
 function ui.set_focus(id, block, cell)
  own(id)
  for _, e in ipairs(flat(ui.screens[id])) do
   if e.block == block and e.id == cell then ui._f[id] = { block = block, cell = cell, index = e.index }; return true end
  end
  return false
 end
 function ui.note(t)
  local id = top()
  if not id or not may_touch(id) then return false end
  ui.notes[#ui.notes + 1] = t; return true
 end
 function ui.dialog(t)
  local id = top()
  if not id or not may_touch(id) then return false end
  ui.dialogs[#ui.dialogs + 1] = t; return true
 end
 function ui.state() return { depth = #ui.stack, top = top(), roles_ok = ui.available_ok } end

 -- ---- the engine's part -------------------------------------------------------------------------------------
 -- the explainer table as the engine keeps it: fields cut to their buffers (AT_STR 63, AT_TEXT 159), a warning when cut
 local function explainer_view(t)
  if type(t) ~= 'table' then return nil end
  local e, cut, w = {}, false, false
  e.kicker, cut = clip(t.kicker, L.str); w = w or cut
  e.title, cut = clip(t.title, L.str); w = w or cut
  e.what, cut = clip(t.what, L.text); w = w or cut
  if type(t.media) == 'table' then e.media = { model = t.media.model, ring = t.media.ring } end
  if type(t.from) == 'table' then e.from = { text = (clip(t.from.text, L.str)) }; if #tostring(t.from.text or '') > L.str - 1 then w = true end end
  e.with = t.with
  e.warn = w
  return e
 end

 function ui.refresh(id)
  local d, f = ui.screens[id], ui._f[id]
  local cell, block = f and f.cell, f and f.block
  local v = { keys = {} }
  local ex = d.explainer
  if type(ex) == 'table' and type(ex.provide) == 'function' and cell then v.explainer = explainer_view(ex.provide(cell, block)) end
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

 -- {pop = true} / {push = "id"} from a handler, read raw like the binding
 -- judged by slot owners (gs_ui_apply_result): the handler's owner may pop only a top screen it owns, push only one it owns
 local function apply(r, owner)
  if type(r) ~= 'table' then return end
  if rawget(r, 'pop') and top() and ui._owner[top()] == owner then
   local keep = ui.caller; ui.caller = 'console'; ui.close(top()); ui.caller = keep
  end
  local p = rawget(r, 'push')
  if type(p) == 'string' and ui.screens[p] and ui._owner[p] == owner then
   local keep = ui.caller; ui.caller = 'console'; ui.open(p); ui.caller = keep
  end
 end

 local function row_of(d, f)
  if d.primary.kind ~= 'list' or not f then return nil end
  for _, it in ipairs(d.primary.items) do if it.id == f.cell then return it end end
 end

 -- A toggle flips, a slider steps by max(1, (max - min) // 20) and clamps, a choice reports its direction; on.change(id, value).
 -- Returns true when the event was a value event (even at a slider's end, where nothing is reported).
 function ui.engine_row(id, how)
  local d, f = ui.screens[id], ui._f[id]
  local it = row_of(d, f)
  local v = it and type(it.value) == 'table' and it.value or nil
  if not v or disabled(it) or (v.kind ~= 'toggle' and v.kind ~= 'choice' and v.kind ~= 'slider') then return false end
  if how ~= 'accept' and how ~= 'left' and how ~= 'right' then return false end
  local st = ui._rows[id][it.id]
  if not st then st = { on = v.on and true or false, val = v.value or 0 }; ui._rows[id][it.id] = st end
  local arg
  if v.kind == 'toggle' then st.on = not st.on; arg = st.on
  elseif v.kind == 'choice' then arg = (how == 'left') and -1 or 1
  else
   if how == 'accept' then return false end
   local lo, hi = v.min or 0, v.max or 100
   local step = math.max(1, (hi - lo) // 20)
   local nv = math.max(lo, math.min(hi, math.max(lo, math.min(hi, st.val)) + (how == 'left' and -step or step)))
   if nv == math.max(lo, math.min(hi, st.val)) then return true end
   st.val = nv; arg = nv
  end
  local on = d.on or {}
  if on.change then apply(on.change(it.id, arg), ui._owner[id]) end
  return true, arg
 end

 function ui.engine_press(id, kind)
  local d, f = ui.screens[id], ui._f[id]
  local on = d.on or {}
  local fn
  if kind == 'accept' then
   if row_of(d, f) and ui.engine_row(id, 'accept') then return true end
   fn = on.accept
  elseif kind == 'back' then fn = on.back
  elseif kind == 'x' or kind == 'y' or kind == 'z' then fn = on.alt and on.alt[kind:upper()]
  elseif kind == 'start' then fn = on.start
  elseif kind == 'l' or kind == 'r' then
   if not on.page then return false end
   local r = on.page(kind == 'l' and -1 or 1, f and f.cell, f and f.block)
   if ui.screens[id] then ui.refresh(id) end
   apply(r, ui._owner[id])
   return true, r
  end
  if kind == 'accept' and f then
   for _, e in ipairs(flat(d)) do if e.block == f.block and e.id == f.cell and disabled(e.cell) then return false end end
  end
  if not fn then return false end
  local r = fn(f and f.cell, f and f.block)
  if ui.screens[id] then ui.refresh(id) end
  apply(r, ui._owner[id])
  return true, r
 end

 -- ---- the pad: buttons held, and one tick of the top screen's polling (gs_ui_tick) ---------------------------------
 function ui.hold(button) assert(PAD[button], 'unknown button ' .. tostring(button)); ui.held[button] = true end
 function ui.release(button) ui.held[button] = nil end
 function ui.tick()
  local id = top()
  if not id then return end
  local edges = {}
  for _, b in ipairs(PAD_ORDER) do if ui.held[b] and not ui._prev[b] then edges[#edges + 1] = b end end
  for _, b in ipairs(PAD_ORDER) do ui._prev[b] = ui.held[b] end
  for _, b in ipairs(edges) do
   if top() ~= id then break end                -- the top screen changed: the rest of this tick's input is dropped
   ui.engine_press(id, PAD[b])
  end
 end

 return ui
end

return Stub
