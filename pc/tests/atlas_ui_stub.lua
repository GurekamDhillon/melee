-- A gd.ui stand-in for offline tests (Atlas step 1). It enforces the contract of gw_ui_screen.c and gw_script_ui.inc (the
-- numeric limits are read from gw_ui_screen.h so the two cannot drift; the rules are copied from the C file and named by the
-- message the C side gives) and plays the engine's part where a test needs it: focus by cell id, on.focus, the explainer / key
-- label / counter refresh, accept / back / alt / page / start dispatch, value rows (on.change), {pop=}/{push=} results, a
-- held-button model of the pad, and the ownership rule (a screen belongs to the script that registered it).
--
-- What it does NOT model: layout, drawing, the quad budget, mouse and keyboard, the Lua registry, and the entry
-- registry's caps and ordering (6 per mod per parent, 12 visible, `after`): those are gw_ui_registry.c's, tested by atlas-registry.
-- Everything in the conversion arena IS modelled (node, entry and string-pool ceilings and the depth limit), because a stand-in
-- that accepts a description the engine refuses defeats its purpose.
local Stub = {}
Stub.slots = 16   -- GS_UI_SLOTS: one slot per registered screen id, freed only by forget (or the owner script unloading)

local function read_limits()
 local here = (arg and arg[0] or ''):gsub('\\', '/'):gsub('[^/]*$', '')
 for _, p in ipairs({ here .. '../platform/gw_ui_screen.h', 'pc/platform/gw_ui_screen.h', 'melee/pc/platform/gw_ui_screen.h' }) do
  local f = io.open(p)
  if f then
   local text = f:read('a'); f:close()
   local L = {}
   for name, v in text:gmatch('#define%s+AT_MAX_(%u[%u_]*)%s+(%d+)') do L[name:lower()] = tonumber(v) end   -- items = the native record (64); items_lua = the Lua door (32)
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
  available_ok = opts.available ~= false, caller = opts.caller or (opts.mod and (opts.mod .. '/main')) or 'console', owner_mod = opts.owner_mod or opts.mod,
  held = {}, _prev = {} }
 if opts.shared then   -- two callers over ONE engine: the screens, the stack and the views are the engine's, not the caller's
  for _, k in ipairs({ 'screens', 'stack', 'views', '_f', '_owner', '_rows', 'fed' }) do ui[k] = opts.shared[k] end
 end
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
  elseif p.kind == 'cards' then
   for i, c in ipairs(p.cards) do out[#out + 1] = { block = 'cards', id = c.id, cell = c, index = i } end
  elseif p.kind == 'tiles' then
   for i, it in ipairs(p.items) do out[#out + 1] = { block = 'tiles', id = it.id, cell = it, index = i } end
   for i, it in ipairs(p.more or {}) do out[#out + 1] = { block = 'more', id = it.id, cell = it, index = i } end
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
  if d.kind == 'pause' and p.kind ~= 'list' then fail('a pause screen has a list primary') end
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
  elseif p.kind == 'cards' then
   local n = #(p.cards or {})
   if n > L.cards then fail(('at most %d cards (%d given)'):format(L.cards, n)) end
   if n < 1 then fail(('a cards screen needs 1 to %d cards'):format(L.cards)) end
   for ci, c in ipairs(p.cards) do
    if type(c) ~= 'table' then fail(('card %d is not a table'):format(ci)) end
    check_id(('card %d'):format(ci), c.id)
    if seen[c.id] then fail('duplicate card id ' .. c.id) end
    seen[c.id] = true
   end
  elseif p.kind == 'list' or p.kind == 'tiles' then
   local n = #(p.items or {})
   if p.kind == 'tiles' then
    if p.cols ~= nil and p.cols ~= 0 and p.cols ~= 1 and p.cols ~= 2 then fail('tiles: cols must be 1 or 2') end
    if #(p.more or {}) > L.more then fail(('tiles: at most %d more items (%d given)'):format(L.more, #p.more)) end
    for mi, it in ipairs(p.more or {}) do
     if type(it) ~= 'table' then fail(('more item %d is not a table'):format(mi)) end
     check_id(('more item %d'):format(mi), it.id)
     if seen[it.id] then fail('duplicate item id "' .. it.id .. '"') end
     seen[it.id] = true
    end
   end
   if n < 1 or n > L.items_lua then fail(('a list needs 1 to %d items (it has %d)'):format(L.items_lua, n)) end
   for ii, it in ipairs(p.items) do
    if type(it) ~= 'table' then fail(('item %d is not a table'):format(ii)) end
    check_id(('item %d'):format(ii), it.id)
    if seen[it.id] then fail('duplicate item id "' .. it.id .. '"') end
    seen[it.id] = true
    if type(it.value) == 'table' then
     local k = it.value.kind
     if k == 'slider' then
      if not ((it.value.min or 0) < (it.value.max or 100)) then fail('item "' .. it.id .. '": slider min must be below max') end
      local st = it.value.step
      if st ~= nil and (math.type(st) ~= 'integer' or st < 1) then fail('item "' .. it.id .. '": slider step must be a whole number of at least 1') end
     elseif k == 'choice' and it.value.options ~= nil then
      local o = it.value.options
      if type(o) ~= 'table' or #o < 1 or #o > L.opts then fail(('item "%s": choice options are 1 to %d strings'):format(it.id, L.opts)) end
      for _, s in ipairs(o) do if type(s) ~= 'string' then fail('item "' .. it.id .. '": choice options are strings') end end
     elseif k ~= 'toggle' and k ~= 'choice' and k ~= 'text' and k ~= 'counter' then
      fail(('item "%s": unknown value kind "%s"'):format(it.id, tostring(k)))
     end
    end
   end
  else
   fail(('primary kind "%s" is not supported here (grid, list, tiles or cards)'):format(tostring(p.kind)))
  end
  ui.links_skipped = ui.links_skipped or {}
  if p.kind == 'grid' and p.links then
   if #p.links > L.links then fail(('at most %d links (%d given)'):format(L.links, #p.links)) end
   local skipped = 0
   for _, lk in ipairs(p.links) do
    if not (seen[lk.a] and seen[lk.b]) or lk.a == lk.b then skipped = skipped + 1 end
   end
   ui.links_skipped[d.id] = skipped
  end
  if d.countdown ~= nil and type(d.countdown) ~= 'number' then fail('countdown is a number of seconds') end
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
  if not ui.screens[d.id] then
   local n = 0
   for _ in pairs(ui.screens) do n = n + 1 end
   if n >= Stub.slots then fail(('too many screens (%d)'):format(Stub.slots)) end
  end
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
  if ui.screens[id].kind == 'pause' and ui.netplay then return false end        -- a pause screen is never opened online
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
 -- gd.ui.forget(id): closed if on the stack, its slot freed (the id is unknown afterwards)
 function ui.forget(id)
  own(id)
  ui.close(id)
  ui.screens[id], ui._f[id], ui._owner[id], ui.views[id], ui._rows[id] = nil, nil, nil, nil, nil
  return true
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

 -- the live value of a row (what the engine keeps; a registration starts it from the description, re-registering resets it)
 local function row_state(d, it)
  local v = it.value
  local st = ui._rows[d.id][it.id]
  if not st then
   local val = v.value or 0
   if v.kind == 'slider' then val = math.max(v.min or 0, math.min(v.max or 100, val)) end
   if v.kind == 'choice' and v.options then val = math.max(0, math.min(#v.options - 1, val)) end
   st = { on = v.on and true or false, val = val, text = v.text }
   ui._rows[d.id][it.id] = st
  end
  return st
 end
 local function item_of(id, item)
  for _, it in ipairs(ui.screens[id].primary.items or {}) do if it.id == item then return it end end
 end

 -- gd.ui.set_value(id, item, v): in place, no re-registration, no on.change, the focus stays; false for an unknown item or a wrong-typed value
 function ui.set_value(id, item, val)
  own(id)
  local it = item_of(id, item)
  if not it or type(it.value) ~= 'table' then return false end
  local v, st = it.value, row_state(ui.screens[id], it)
  if v.kind == 'toggle' then
   if type(val) ~= 'boolean' then return false end
   st.on = val
  elseif v.kind == 'slider' then
   if math.type(val) ~= 'integer' then return false end
   st.val = math.max(v.min or 0, math.min(v.max or 100, val))
  elseif v.kind == 'choice' and v.options then
   if math.type(val) ~= 'integer' or val < 0 or val >= #v.options then return false end
   st.val = val
  elseif v.kind == 'choice' or v.kind == 'text' or v.kind == 'counter' then
   if type(val) ~= 'string' then return false end
   st.text = (clip(val, L.str))
  else
   return false
  end
  return true
 end
 -- gd.ui.value(id, item): a boolean (toggle), an integer (slider, an options choice's index) or a string; nil for an unknown item
 function ui.value(id, item)
  own(id)
  local it = item_of(id, item)
  if not it or type(it.value) ~= 'table' then return nil end
  local v, st = it.value, row_state(ui.screens[id], it)
  if v.kind == 'toggle' then return st.on end
  if v.kind == 'slider' or (v.kind == 'choice' and v.options) then return st.val end
  return st.text or ''
 end

 -- A toggle flips, a slider steps by its step (default max(1, (max - min) // 20)) and clamps, a choice with options wraps and reports its index,
 -- a choice without options reports its direction; on.change(id, value).
 -- Returns true when the event was a value event (even at a slider's end, where nothing is reported).
 function ui.engine_row(id, how)
  local d, f = ui.screens[id], ui._f[id]
  local it = row_of(d, f)
  local v = it and type(it.value) == 'table' and it.value or nil
  if not v or disabled(it) or (v.kind ~= 'toggle' and v.kind ~= 'choice' and v.kind ~= 'slider') then return false end
  if how ~= 'accept' and how ~= 'left' and how ~= 'right' then return false end
  local st = row_state(d, it)
  local arg
  if v.kind == 'toggle' then st.on = not st.on; arg = st.on
  elseif v.kind == 'choice' and v.options then   -- the engine owns the choice: it wraps over the options and reports the INDEX
   local n = #v.options
   if how == 'accept' then return false end
   if n < 2 then return true end
   st.val = ((st.val + (how == 'left' and -1 or 1)) % n + n) % n; arg = st.val
  elseif v.kind == 'choice' then arg = (how == 'left') and -1 or 1
  else
   if how == 'accept' then return false end
   local lo, hi = v.min or 0, v.max or 100
   local step = v.step or math.max(1, (hi - lo) // 20)
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
  elseif kind == 'left' or kind == 'right' then return (ui.engine_row(id, kind))
  elseif kind == 'back' then fn = on.back
  elseif kind == 'x' or kind == 'y' or kind == 'z' then fn = on.alt and on.alt[kind:upper()]
  elseif kind == 'start' then fn = on.start
  elseif kind == 'l' or kind == 'r' then
   if not on.page then return false end
   local r = on.page(kind == 'l' and -1 or 1, f and f.cell, f and f.block)
   if ui.screens[id] and top() == id then ui.refresh(id) end   -- the engine refreshes the top screen only (gs_ui_tick), never one a handler closed
   apply(r, ui._owner[id])
   return true, r
  end
  if kind == 'accept' and f then
   for _, e in ipairs(flat(d)) do if e.block == f.block and e.id == f.cell and disabled(e.cell) then return false end end
  end
  if not fn then return false end
  local r = fn(f and f.cell, f and f.block)
  if ui.screens[id] and top() == id then ui.refresh(id) end
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

 -- gd.ui.hold_menu(on): a script holds the native menu (no input, no drawing); only the holder or the console releases it
 function ui.hold_menu(on)
  if on then if not is_console() then ui.held_by=ui.caller end
  elseif ui.held_by==ui.caller or is_console() then ui.held_by=nil end
  return ui.held_by~=nil
 end

 -- ---- entries: gd.ui.entry, on_entry, and the engine's part of choosing one ----------------------------------------------
 -- An entry as mod.json "menus" declares it (the host reads the manifest; a test registers it here). It validates what
 -- at_menus_parse and at_reg_add validate (a label, the id namespace, a built-in parent, opens or action = "script").
 local PARENTS = { main = true, solo = true, versus = true, settings = true }   -- the parents a menu draws today (online, mods, more, settings.<page> are refused)
 ui.entries, ui.hooks, ui.netplay = {}, {}, false
 function ui.register_entry(e)
  local function bad(msg) error('entry "' .. tostring(e.id) .. '": ' .. msg, 2) end
  if type(e.id) ~= 'string' or e.id == '' then error('an entry needs an id', 2) end
  if type(e.label) ~= 'string' or e.label == '' then bad('an entry needs a label') end
  if not PARENTS[e.parent] then bad('no menu shows parent "' .. tostring(e.parent) .. '" yet') end
  local mod = ui.owner_mod
  if mod and not (e.id == mod or e.id:sub(1, #mod + 1) == mod .. '.') then bad('the id of a mod entry is "' .. mod .. '" or starts with "' .. mod .. '."') end
  if not (e.action == 'script' or e.opens) then bad('an entry needs opens or action "script"') end
  if ui.entries[e.id] then bad('the id is already registered') end
  ui.entries[e.id] = { id = e.id, parent = e.parent, label = e.label:sub(1, 18), opens = e.opens, action = e.action, online = e.online and true or false,
                       visible = true, badge = '', mod = mod }
  return true
 end
 -- gd.ui.entry(id, { visible =, badge = }): true when the caller's mod owns that entry
 function ui.entry(id, t)
  local e = ui.entries[id]
  if not e or is_console() or not ui.owner_mod or e.mod ~= ui.owner_mod then return false end
  if type(t) == 'table' then
   if type(t.visible) == 'boolean' then e.visible = t.visible end
   if type(t.badge) == 'string' then e.badge = t.badge end
  end
  return true
 end
 -- what the player sees under `parent`: visible entries; a netplay session hides the ones without online under versus and online
 function ui.entries_under(parent)
  local out = {}
  for _, e in pairs(ui.entries) do
   if e.parent == parent and e.visible and not (ui.netplay and (parent == 'versus' or parent == 'online') and not e.online) then out[#out + 1] = e.id end
  end
  table.sort(out)
  return out
 end
 -- the engine's part: choosing an entry. "opens" pushes the mod's own screen; "script" runs the mod's hooks.on_entry(id) and applies {push=}.
 function ui.engine_activate(id)
  local e = ui.entries[id]
  if not e or not e.visible then return false end
  if ui.netplay and (e.parent == 'versus' or e.parent == 'online') and not e.online then return false end
  if e.action == 'script' then
   local fn = ui.hooks.on_entry
   if type(fn) ~= 'function' then return false end
   apply(fn(id), ui.hooks.owner or ui.caller)           -- the hook's owner: the mod's script (default: the stub's caller)
   return true
  end
  if ui.screens[e.opens] and tostring(ui._owner[e.opens]):match('^([^/]+)') == e.mod then
   local keep = ui.caller; ui.caller = 'console'; ui.open(e.opens); ui.caller = keep
   return true
  end
  return false
 end

 -- ---- the HUD layer and the retail takeover (Atlas step 3; gw_ui_hud.c, gw_script_ui.inc) ------------------------------------------------
 -- gd.ui.hud / hud_clear / toast / retail_hide / retail with the binding's refusals and the quiet-HUD caps. What it does NOT model: the
 -- layout, the keep-outs and the draw (atlas-hud tests those), and the scene change (ui.scene_changed() plays the binding's part).
 ui.now, ui.hud_calls, ui.retail_hide_calls, ui.toast_calls = 0, 0, 0, 0   -- ui.now: seconds, the UI clock; tests advance it
 ui.huds, ui.mask, ui.mask_owner = {}, {}, nil
 ui.match_active = opts.match ~= false
 ui.gameplay = opts.gameplay ~= false
 ui.netplay = opts.netplay and true or false
 local ZONES = { top_left = true, top_center = true, top_right = true, bottom_left = true, bottom_center = true, bottom_right = true }
 local ZONE_ORDER = { 'top_left', 'top_center', 'top_right', 'bottom_left', 'bottom_center', 'bottom_right' }
 local KINDS = { strip = true, banner = true, card = true, note = true, port_card = true, timer = true, toast = true }
 local ELEMENTS = { 'hud.damage', 'hud.stock', 'hud.timer', 'hud.nametag', 'hud.magnify', 'hud.coin', 'hud.prize', 'hud.hazard', 'pause.panel' }
 local ELEMENT = {}
 for _, e in ipairs(ELEMENTS) do ELEMENT[e] = true end
 local PER_ZONE = 4

 -- mirrors at_hud_cap_ok: one banner and only in top_center, one toast per top zone, three cards, one note
 function ui.hud_caps(zones)
  local banners, cards, notes = 0, 0, 0
  for _, z in ipairs(ZONE_ORDER) do
   local toasts = 0
   for _, p in ipairs(zones[z] or {}) do
    if p.kind == 'banner' then banners = banners + 1; if z ~= 'top_center' then return false, 'a banner lives in top_center only' end
    elseif p.kind == 'toast' then toasts = toasts + 1; if z:sub(1, 3) ~= 'top' then return false, 'a toast lives in a top zone' end
    elseif p.kind == 'card' then cards = cards + 1
    elseif p.kind == 'note' then notes = notes + 1 end
   end
   if toasts > 1 then return false, 'at most one toast per zone' end
  end
  if banners > 1 then return false, 'at most one banner in the whole HUD' end
  if cards > 3 then return false, 'at most three opponent cards' end
  if notes > 1 then return false, 'at most one pickup note' end
  return true
 end

 local function hud_err(msg) error('gd.ui.hud: ' .. msg, 3) end
 local function live(p) return not (p.until_s and ui.now >= p.until_s) end

 function ui.hud(d)
  ui.hud_calls = ui.hud_calls + 1
  if type(d) ~= 'table' then hud_err('a HUD description is a table') end
  if is_console() or not ui.owner_mod then hud_err('a HUD belongs to a mod script (the console has none)') end
  if type(d.id) ~= 'string' or d.id == '' then hud_err('the description has no id') end
  if d.id:sub(1, #ui.owner_mod + 1) ~= ui.owner_mod .. '.' then hud_err(('id "%s" must start with "%s."'):format(d.id, ui.owner_mod)) end
  check_arena(d, L)
  local zones = {}
  for z, list in pairs(d.zones or {}) do
   if not ZONES[z] then hud_err('zone ' .. tostring(z) .. ' is not a zone') end
   if #list > PER_ZONE then hud_err(('zone %s holds at most %d parts (%d given)'):format(z, PER_ZONE, #list)) end
   zones[z] = {}
   for i, p in ipairs(list) do
    if not KINDS[p.kind] or p.kind == 'toast' then hud_err(('part %d of %s has the unknown kind "%s" (strip, banner, card, note, port_card, timer)'):format(i, z, tostring(p.kind))) end
    if p.kind == 'port_card' and not (type(p.port) == 'number' and p.port >= 1 and p.port <= 4) then hud_err('a port_card needs port 1 to 4') end
    if p.kind == 'strip' and (#(p.pips or {}) > 8 or #(p.keys or {}) > 8) then hud_err('a strip shows at most 8 slot pips and 8 keystones') end
    if p.kind == 'card' and #(p.lines or {}) > 3 then hud_err('a card shows at most 3 lines') end
    local c = {}; for k, v in pairs(p) do c[k] = v end
    if c.kind == 'note' then c.until_s = ui.now + math.max(0.5, math.min(15, c.seconds or 4)) end
    zones[z][i] = c
   end
  end
  local old = ui.huds[ui.caller]
  if old then
   for z, list in pairs(old.zones) do
    for _, p in ipairs(list) do
     if p.kind == 'toast' and live(p) then
      zones[z] = zones[z] or {}
      local has = false; for _, q in ipairs(zones[z]) do if q.kind == 'toast' then has = true end end
      if not has and #zones[z] < PER_ZONE then table.insert(zones[z], 1, p) end
     elseif p.kind == 'note' and live(p) then
      for _, q in ipairs(zones[z] or {}) do if q.kind == 'note' and q.text == p.text then q.until_s = p.until_s end end
     end
    end
   end
  end
  local ok, why = ui.hud_caps(zones)
  if not ok then hud_err(why) end
  if not old then
   local n = 0; for _ in pairs(ui.huds) do n = n + 1 end
   if n >= 4 then hud_err('too many HUDs (4)') end
  end
  ui.huds[ui.caller] = { id = d.id, owner = ui.caller, zones = zones }
  return true
 end

 function ui.hud_clear(id)
  if is_console() or not ui.owner_mod then error('gd.ui.hud_clear: a HUD belongs to a mod script (the console has none)', 2) end
  local h = ui.huds[ui.caller]
  if h and id ~= nil and id ~= h.id then h = nil end
  if h then ui.huds[ui.caller] = nil end
  return h ~= nil
 end

 function ui.toast(t)
  ui.toast_calls = ui.toast_calls + 1
  if type(t) ~= 'table' then error('bad argument #1 to toast (table expected)', 2) end
  if is_console() or not ui.owner_mod then error('gd.ui.toast: a toast belongs to a mod script (the console has none)', 2) end
  local z = t.zone or 'top_right'
  if z ~= 'top_left' and z ~= 'top_right' then error('gd.ui.toast: zone is "top_left" or "top_right"', 2) end
  local h = ui.huds[ui.caller]
  if not h then h = { id = ui.owner_mod .. '.toast', owner = ui.caller, zones = {} }; ui.huds[ui.caller] = h end
  h.zones[z] = h.zones[z] or {}
  local list, at = h.zones[z], nil
  for i, p in ipairs(list) do if p.kind == 'toast' then at = i end end
  local secs = math.max(0.5, math.min(15, t.seconds or 4))
  local p = { kind = 'toast', title = tostring(t.title or ''), text = tostring(t.text or ''), rgba = t.rgba, from_s = ui.now, until_s = ui.now + secs }
  if at then list[at] = p
  else
   if #list >= PER_ZONE then error(('gd.ui.toast: zone %s is full (%d parts)'):format(z, PER_ZONE), 2) end
   table.insert(list, 1, p)
  end
  return true
 end

 -- the checks run in the binding's order: online, console or not gameplay, no match, another script's claim, an id a mod may not hide
 function ui.retail_hide(list)
  ui.retail_hide_calls = ui.retail_hide_calls + 1
  if type(list) ~= 'table' then error('bad argument #1 to retail_hide (table expected)', 2) end
  if ui.netplay then error('gd.ui.retail_hide: not available online (nothing retail is hidden online)', 2) end
  if is_console() or not ui.gameplay then error('gd.ui.retail_hide: needs a gameplay mod script', 2) end
  if not ui.match_active then error('gd.ui.retail_hide: needs an active match', 2) end
  if ui.mask_owner and ui.mask_owner ~= ui.caller then error('gd.ui.retail_hide: the mask belongs to another script', 2) end
  local m = {}
  for _, e in ipairs(list) do
   if type(e) ~= 'string' then error('gd.ui.retail_hide: element names are strings', 2) end
   if e == 'hud.timer' then error('gd.ui.retail_hide: hud.timer may not be hidden by a mod', 2) end
   if not ELEMENT[e] then error(('gd.ui.retail_hide: unknown retail element "%s"'):format(e), 2) end
   m[e] = true
  end
  ui.mask = m
  ui.mask_owner = next(m) and ui.caller or nil
  return true
 end

 function ui.retail()
  local hidden = {}
  if not ui.netplay then for _, e in ipairs(ELEMENTS) do if ui.mask[e] then hidden[#hidden + 1] = e end end end
  return { hidden = hidden, paused = ui.paused == true, pauser = ui.pauser, takeover = ui.takeover == true }
 end

 -- gw_Ui_SceneExit: a scene ends, every screen on the stack closes (its on.close runs) except one that says persist = true
 function ui.scene_exit()
  local ids = {}; for i, id in ipairs(ui.stack) do ids[i] = id end
  for i = #ids, 1, -1 do
   local d = ui.screens[ids[i]]
   if d and not d.persist then
    local keep = ui.caller; ui.caller = 'console'; ui.close(ids[i]); ui.caller = keep
    local on = d.on or {}; if on.close then on.close('', '') end
   end
  end
 end
 -- the binding's part at a scene change: the script's mask claim goes, every toast and note with it
 function ui.scene_changed()
  ui.mask, ui.mask_owner = {}, nil
  for _, h in pairs(ui.huds) do
   for z, list in pairs(h.zones) do
    local keep = {}
    for _, p in ipairs(list) do if p.kind ~= 'toast' and p.kind ~= 'note' then keep[#keep + 1] = p end end
    h.zones[z] = keep
   end
  end
 end

 -- ---- the pause screen and the retail pause takeover (gw_script_ui.inc; the takeover is off unless ui.pause_wanted) ----------------------------
 ui.paused, ui.pauser, ui.takeover, ui.pause_wanted, ui.pause_slot, ui.pause_pushed, ui.unpause_req = false, nil, false, false, nil, nil, false
 function ui.pause_screen(id)
  if id == nil then if ui.pause_slot and may_touch(ui.pause_slot) then ui.pause_slot = nil end; return ui.pause_slot == nil end
  own(id)
  if ui.screens[id].kind ~= 'pause' then error(('gd.ui.pause_screen: "%s" is not a pause screen (kind = "pause")'):format(id), 2) end
  ui.pause_slot = id; return true
 end
 function ui.unpause()
  if not ui.paused or ui.netplay or ui.unpause_req then return false end
  ui.unpause_req = true; return true
 end
 -- the game's part: the one-shot request, taken at the next unpause check (offline only)
 function ui.take_unpause()
  if ui.netplay or not ui.paused or not ui.unpause_req then return nil end
  ui.unpause_req = false; return ui.pauser
 end
 -- the retail pause began (on) or ended (not on): the tick pushes or pops the named pause screen
 function ui.retail_pause(port, on)
  if on then
   ui.paused, ui.pauser, ui.takeover, ui.unpause_req = true, port, ui.pause_wanted and not ui.netplay, false
   if ui.takeover and ui.pause_slot and ui.screens[ui.pause_slot] then
    ui.screens[ui.pause_slot].port = port + 1
    local keep = ui.caller; ui.caller = 'console'; if ui.open(ui.pause_slot) then ui.pause_pushed = ui.pause_slot end; ui.caller = keep
   end
  else
   ui.paused, ui.pauser, ui.takeover, ui.unpause_req = false, nil, false, false
   if ui.pause_pushed then local keep = ui.caller; ui.caller = 'console'; ui.close(ui.pause_pushed); ui.caller = keep; ui.pause_pushed = nil end
  end
 end

 return ui
end

return Stub
