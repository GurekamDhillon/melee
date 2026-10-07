-- Offline tests of Envoy's app-level Atlas screens (step 3): the run setup, the pause (with its confirm and quit dialogs), the results.
-- The real app (app.lua / retail_app.lua) and the legacy menu state machine on the gd.ui stand-in.   cd melee && lua pc/tests/envoy_atlas_app.lua
local prefix = io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/'
local T = dofile(prefix .. 'pc/tests/envoy_testlib.lua')
local Stub = dofile(prefix .. 'pc/tests/atlas_ui_stub.lua')
local D = T.rules()
for _, k in ipairs({ 'save', 'drives', 'drive_models', 'fighter', 'campaign', 'hub', 'run', 'classic', 'hud', 'menu', 'menu_draw', 'menu_input', 'recolour', 'visual' }) do D[k] = T.module(k, D) end
D.retail_app = T.module('retail_app', D); D.app = T.module('app', D)
D.atlas_kit = T.module('atlas_kit', D)
for _, k in ipairs({ 'atlas_setup', 'atlas_pause', 'atlas_results' }) do D[k] = T.module(k, D) end
local K = D.atlas_kit

local function fixture(opts)
  opts = opts or {}
  local s = { text = D.save.encode(D.save.new_profile()), pad = {}, commands = {}, writes = 0, mods = {}, logs = {}, online = false, paused = false, mask_calls = 0,
    mode = { mode = 'classic', stage_index = 0, loop = 0, player_port = 1, opponents = { { port = 2 } } } }
  local ui = Stub.new({ caller = 'envoy/main', owner_mod = 'envoy', available = opts.available })
  local g = { command = function(n, f) s.commands[n] = f end, log = function(t) s.logs[#s.logs + 1] = t end, data_read = function() return s.text end,
    data_write_atomic = function(_, v) s.writes = s.writes + 1; s.text = v; return true end,
    match = function() return { active = opts.in_match or false, netplay = s.online } end, player = function() return {} end,
    pad = function() return s.pad end, input_mask = function() s.mask_calls = s.mask_calls + 1 end, paused = function() return s.paused end,
    pause = function() s.paused = true end, resume = function() s.paused = false end,
    fx_world = function() return 1 end, fx_move = function() end, fx_control = function() end, fx_end = function() end,
    mode_1p = function() return s.mode end, start_1p = function(v) s.launch = v; return true end, spawn_1p = function() return true end, loop_1p = function() return true end,
    end_1p = function() return true end, hold_1p = function() return true end, release_1p = function() return true end, ui = ui }
  local mission = { frame = function() end, draw = function() end, stop = function(m) m.current = nil end }
  K.set(true); K.set_legacy(false); K.logged = {}
  s.a = D.app.new(g, mission); s.ui = ui; s.g = g
  return s, s.a, ui
end
local function top(ui) return ui.state().top end
local function has_log(s, text) for _, l in ipairs(s.logs) do if l:find(text, 1, true) then return true end end; return false end
local function show(a, screen) a.visible = true; a.menu:show(screen) end

-- ---- Task 12: the setup screen ------------------------------------------------------------------------------------------------

T.test('setup lists the live choices only', function()
  local s, a, ui = fixture(); show(a, 'setup')
  local d = ui.screens['envoy.setup']
  assert(d and top(ui) == 'envoy.setup' and a.menu.atlas and a.menu.atlas.screen == 'setup', 'the Atlas setup is up in place of the legacy one')
  local ids = {}; for _, it in ipairs(d.primary.items) do ids[#ids + 1] = it.id end
  assert(table.concat(ids, ',') == 'begin,mode,fighter,difficulty,stocks', 'no Companion, Records or garden rows: ' .. table.concat(ids, ','))
  assert(d.persist == true and d.trail.title == 'RUN SETUP' and d.trail[1] == 'SOLO' and d.trail[2] == 'ENVOY')
  assert(d.primary.items[1].label:find('Begin Classic as', 1, true), 'Begin names the mode and the fighter: ' .. d.primary.items[1].label)
  ui.engine_focus('envoy.setup', 'list', 'mode')
  assert(ui.views['envoy.setup'].explainer.what:find('Classic', 1, true), 'one rule per row')
end)

T.test('changing a choice writes the menu field the legacy row writes', function()
  local s, a, ui = fixture(); show(a, 'setup')
  ui.engine_focus('envoy.setup', 'list', 'mode'); ui.engine_row('envoy.setup', 'right')
  assert(a.menu.run_type == 'adventure', 'Mode -> Adventure: ' .. tostring(a.menu.run_type))
  assert(ui.screens['envoy.setup'].primary.items[2].value.text == 'Adventure' and ui.screens['envoy.setup'].primary.items[1].label:find('Adventure', 1, true), 're-described')
  ui.engine_focus('envoy.setup', 'list', 'stocks'); local before = a.menu.stocks; ui.engine_row('envoy.setup', 'right')
  assert(a.menu.stocks == before % 5 + 1, 'Stocks steps')
  ui.engine_focus('envoy.setup', 'list', 'difficulty'); local d0 = a.menu.difficulty; ui.engine_row('envoy.setup', 'left')
  assert(a.menu.difficulty == (d0 - 1) % 5, 'Difficulty steps down and wraps')
  ui.engine_focus('envoy.setup', 'list', 'fighter'); local f0 = a.menu.fighter; ui.engine_row('envoy.setup', 'right')
  assert(a.menu.fighter ~= f0, 'Fighter steps through the app\'s list: ' .. tostring(a.menu.fighter))
end)

T.test('co-op appears only when it is available', function()
  local s, a, ui = fixture(); show(a, 'setup')
  ui.engine_focus('envoy.setup', 'list', 'mode')
  ui.engine_row('envoy.setup', 'right'); ui.engine_row('envoy.setup', 'right')
  assert(a.menu.run_type == 'classic', 'without co-op the mode wraps classic > adventure > classic')
  a.coop_ready = function() return true end
  ui.engine_row('envoy.setup', 'right'); ui.engine_row('envoy.setup', 'right')
  assert(a.menu.run_type == 'coop', 'with co-op: classic > adventure > co-op')
end)

T.test('Begin starts through the app\'s start path with the menu\'s choices', function()
  local s, a, ui = fixture(); show(a, 'setup')
  local got
  a.start_retail = function(self, mode, fighter, difficulty, stocks) got = { mode, fighter, difficulty, stocks }; self.menu:show('playing'); return true end
  a.menu.run_type = 'adventure'; a.menu.fighter = 'fox'; a.menu.difficulty = 3; a.menu.stocks = 2
  ui.engine_focus('envoy.setup', 'list', 'begin'); ui.engine_press('envoy.setup', 'accept')
  assert(got and got[1] == 'adventure' and got[2] == 'fox' and got[3] == 3 and got[4] == 2, 'the start path got the choices')
  assert(a.menu.screen == 'playing' and #ui.stack == 0 and a.menu.atlas == nil, 'the setup closed with the legacy screen')
end)

T.test('a refused start shows the reason as a note and the setup stays', function()
  local s, a, ui = fixture(); show(a, 'setup')
  a.start_retail = function(self) self.notice = 'Another run is active'; self.visible = true; self.menu:show('setup'); return false, 'Another run is active' end
  ui.engine_focus('envoy.setup', 'list', 'begin'); ui.engine_press('envoy.setup', 'accept')
  assert(top(ui) == 'envoy.setup' and a.menu.screen == 'setup', 'it stays')
  assert(ui.notes[#ui.notes] and ui.notes[#ui.notes].text == 'Another run is active', 'with the reason')
end)

T.test('the legacy pad events reach the Atlas setup: directions and A are fed, B leaves Envoy', function()
  local s, a, ui = fixture(); show(a, 'setup')
  local before = #ui.fed
  a.menu:input('down', a:context()); a.menu:input('accept', a:context())
  assert(#ui.fed == before + 2 and ui.fed[#ui.fed][2] == 'accept', 'fed to the engine')
  ui.engine_press('envoy.setup', 'back')
  assert(a.visible == false and #ui.stack == 0, 'B leaves Envoy')
end)

T.test('the switch: off and legacy leave the legacy setup; netplay opens nothing', function()
  local s, a, ui = fixture(); K.set(false); show(a, 'setup')
  assert(a.menu.atlas == nil and #ui.stack == 0 and a.menu.screen == 'setup', 'off: the legacy screen')
  K.set(true); K.set_legacy(true); a.menu:show('hub'); show(a, 'setup')
  assert(a.menu.atlas == nil and #ui.stack == 0, 'envoy ui legacy: the legacy screen')
  K.set_legacy(false); s.online = true; a.menu:show('hub'); show(a, 'setup')
  assert(a.menu.atlas == nil and #ui.stack == 0, 'online: nothing')
end)

T.test('the entry screen offers RUN SETUP only with Atlas on, and opens the setup', function()
  local s, a, ui = fixture()
  a:entry('envoy'); local n = #ui.screens['envoy.entry'].primary.items
  K.set(false); ui.screens['envoy.entry'] = nil; a:entry('envoy')
  assert(#ui.screens['envoy.entry'].primary.items == n - 1 and n == 5, 'five rows with Atlas on, four with it off: ' .. n)
  K.set(true)
  local r = a:entry_accept('setup')
  assert(r and r.pop == true and top(ui) == 'envoy.setup' and a.menu.atlas.screen == 'setup', 'the entry opens the setup')
end)

-- ---- Task 13: the pause -------------------------------------------------------------------------------------------------------

local function playing(opts)
  local s, a, ui = fixture(opts)
  a.retail.active = true; a.retail.rules = true; a.retail.host = {}; a.retail.mode = 'classic'; a.retail.loop = 0
  a.menu:show('playing'); a.visible = true
  return s, a, ui
end

T.test('pause lists Resume, Bag, Controls, Quit run; Bag is absent when rules are off', function()
  local s, a, ui = playing()
  a.menu:input('start', a:context())                                  -- START in play: the legacy route shows the pause
  local d = ui.screens['envoy.pause']
  assert(d and d.kind == 'pause' and top(ui) == 'envoy.pause' and a.menu.screen == 'pause', 'the pause screen')
  local ids = {}; for _, it in ipairs(d.primary.items) do ids[#ids + 1] = it.id end
  assert(table.concat(ids, ',') == 'resume,bag,controls,quit', table.concat(ids, ','))
  local k = {}; for _, kk in ipairs(d.keys) do k[kk[1]] = kk[2] end; assert(k.A == 'Select' and k.B == 'Resume')
  a.menu:show('playing'); a.retail.rules = false; a.menu:input('start', a:context())
  ids = {}; for _, it in ipairs(ui.screens['envoy.pause'].primary.items) do ids[#ids + 1] = it.id end
  assert(table.concat(ids, ',') == 'resume,controls,quit', 'rules off: no Bag: ' .. table.concat(ids, ','))
end)

T.test('B resumes through the app\'s resume path', function()
  local s, a, ui = playing({ in_match = true })
  a.menu:input('start', a:context()); a:sync_pause()
  assert(s.paused, 'the game is paused while the pause list is up')
  ui.engine_press('envoy.pause', 'back')
  assert(a.menu.screen == 'playing' and #ui.stack == 0 and a.menu.atlas == nil, 'resumed and closed')
  assert(not s.paused, 'the app resumed the game')
end)

T.test('Controls opens a dialog; Quit asks once: A abandons, B keeps playing', function()
  local s, a, ui = playing()
  a.menu:input('start', a:context())
  ui.engine_focus('envoy.pause', 'list', 'controls'); ui.engine_press('envoy.pause', 'accept')
  local dlg = ui.dialogs[#ui.dialogs]
  assert(dlg.title == 'CONTROLS' and dlg.text:find('Z + START', 1, true) and dlg.text:find('Hold Z + Down', 1, true), 'the controls dialog')
  ui.engine_focus('envoy.pause', 'list', 'quit'); ui.engine_press('envoy.pause', 'accept')
  dlg = ui.dialogs[#ui.dialogs]
  assert(dlg.title == 'Quit the run?' and #dlg.actions == 2 and dlg.actions[1][2] == 'Quit' and dlg.actions[2][2] == 'Keep playing', 'asks once')
  local abandoned = 0
  a.stop = function() abandoned = abandoned + 1 end
  dlg.on('B'); assert(abandoned == 0 and top(ui) == 'envoy.pause', 'B keeps playing')
  dlg.on('A'); assert(abandoned == 1, 'A runs the existing abandon effect')
end)

T.test('the pause screen is named as the takeover\'s screen, and the retail pause pushes it only when the takeover is on', function()
  local s, a, ui = playing()
  assert(D.atlas_pause.ensure(a) and ui.pause_slot == 'envoy.pause', 'named with gd.ui.pause_screen')
  ui.retail_pause(0, true); assert(top(ui) == nil, 'takeover off: a retail pause looks as today')
  ui.retail_pause(0, false)
  ui.pause_wanted = true; ui.retail_pause(0, true)
  assert(top(ui) == 'envoy.pause', 'takeover on: the Atlas pause list')
  ui.retail_pause(0, false)
end)

T.test('Resume during a retail pause (the takeover) also asks the engine to unpause', function()
  local s, a, ui = playing()
  D.atlas_pause.ensure(a); ui.pause_wanted = true; ui.retail_pause(0, true)
  a.menu:show('pause')
  ui.engine_focus('envoy.pause', 'list', 'resume'); ui.engine_press('envoy.pause', 'accept')
  assert(ui.take_unpause() == 0, 'the one-shot request names the pauser')
  ui.retail_pause(0, false)
end)

T.test('online: the pause screen is never opened and Envoy does not pause', function()
  local s, a, ui = playing(); s.online = true
  assert(D.atlas_pause.ensure(a) == false and ui.pause_slot == nil)
  a.menu:show('pause')
  assert(a.menu.atlas == nil and #ui.stack == 0 and ui.screens['envoy.pause'] == nil)
end)

-- ---- Task 14: the results -----------------------------------------------------------------------------------------------------

T.test('results show when the run is over: outcome, stages, time; A and B leave', function()
  local s, a, ui = fixture()
  a.retail.cleared = { 1, 2, 3 }; a.retail.loop = 1
  a.visible = true; a.menu:results({ outcome = 'win', boss = 'Defeated / NG+1', frames = 7260 })
  local d = ui.screens['envoy.results']
  assert(d and top(ui) == 'envoy.results' and a.menu.atlas.screen == 'results')
  local ids = {}; for _, it in ipairs(d.primary.items) do ids[#ids + 1] = it.id end
  assert(table.concat(ids, ',') == 'outcome,stages,time', table.concat(ids, ','))
  assert(d.primary.items[1].label == 'Run complete' and d.primary.items[2].value.text == '3' and d.primary.items[3].value.text == '2:01', 'the facts')
  assert(d.counter == 'NG+1', 'the counter says NG+ when looping')
  local k = {}; for _, kk in ipairs(d.keys) do k[kk[1]] = kk[2] end; assert(k.A == 'Done' and k.B == 'Leave')
  ui.engine_press('envoy.results', 'back')
  assert(a.visible == false and #ui.stack == 0, 'B leaves Envoy through the app\'s quit effect')
end)

T.test('a pending save keeps its row and A retries; B waits until it is written', function()
  local s, a, ui = fixture()
  a.retail.pending = 'win'; a.visible = true
  a.menu:results({ outcome = 'win', frames = 60 })
  local ids = {}; for _, it in ipairs(ui.screens['envoy.results'].primary.items) do ids[#ids + 1] = it.id end
  assert(ids[#ids] == 'retry', 'the retry row')
  local retried = 0
  a.retail.retry_save = function() retried = retried + 1; return false end
  ui.engine_press('envoy.results', 'accept')
  assert(retried == 1, 'A retried the save')
  ui.engine_press('envoy.results', 'back')
  assert(top(ui) == 'envoy.results' and a.visible, 'B does not leave while the save is pending')
end)

T.test('each final drive is one row with its one rule in the explainer', function()
  local F = dofile(prefix .. 'pc/tests/envoy_atlas_fixture.lua')
  local Dr = F.load()
  local e = F.start(Dr); F.stage(e.host)
  local b = e.host:bag(); local r1 = F.roll(e.host, function(r) return e.host:plan_take(r).action ~= 'merge' end); b:place(1, r1); e.host:touch()
  local ui = Stub.new({ mod = 'envoy' })
  local g = setmetatable({ ui = ui }, { __index = e.g })
  local app = { g = g, menu = Dr.menu and Dr.menu.new() or D.menu.new(), retail = { host = e.host, loop = 0, cleared = {} }, visible = true, notice = nil,
    context = function() return {} end, menu_effect = function() end }
  local KK = Dr.atlas_kit; KK.set(true)
  local V = Dr.atlas_results
  app.menu.result = { outcome = 'win', frames = 600 }
  assert(V.open(app), 'opens')
  local d = ui.screens['envoy.results']
  local drives = {}; for _, it in ipairs(d.primary.items) do if it.id:find('^drive:') then drives[#drives + 1] = it end end
  assert(#drives == 1 and drives[1].sub == 'Slot 1', 'one row for the equipped drive')
  ui.engine_focus('envoy.results', 'list', drives[1].id)
  local ex = ui.views['envoy.results'].explainer
  assert(ex.what ~= '' and not ex.what:find('\n') and ex.kicker:find('FINAL BUILD', 1, true), 'its one rule: ' .. tostring(ex.what))
end)

-- ---- Task 18 part A: the legacy switch ------------------------------------------------------------------------------------------

T.test('envoy ui legacy on|off forces the legacy screens back and forth; the default is Atlas off', function()
  local s, a, ui = fixture()
  assert(a:command('ui legacy on') == true and K.legacy.on == true and not K.enabled(a.g), 'legacy on: every Atlas screen is off')
  show(a, 'setup'); assert(a.menu.atlas == nil and #ui.stack == 0 and a.menu.screen == 'setup', 'the legacy setup')
  assert(a:command('ui legacy off') == true and K.legacy.on == false and K.enabled(a.g), 'legacy off: back to Atlas')
  local ok, why = a:command('ui sideways'); assert(ok == false and why:find('usage', 1, true))
  assert(a:command('ui') == true, 'status')
  local K2 = T.module('atlas_kit', {})
  assert(K2.setting.on == false and K2.legacy.on == false, 'off by default until the owner has seen the screens')
end)

T.done()
