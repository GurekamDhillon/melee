-- Offline tests of Envoy's match HUD on gd.ui.hud (step 3): the strip, the synergy toast, the opponent cards, the collect and payout banners, the
-- pickup note, co-op corners, the quiet-HUD caps.   cd melee && lua pc/tests/envoy_atlas_hud.lua
local prefix = io.open('pc/tests/atlas_ui_stub.lua') and '' or 'melee/'
local F = dofile(prefix .. 'pc/tests/envoy_atlas_fixture.lua')
local T, Stub = F.T, F.Stub
local D = F.load()

local function hud(e) return e.ui.huds['envoy/main'] end
local function parts(h, kind, zone)
  local out = {}
  for z, list in pairs(h and h.zones or {}) do
    if not zone or z == zone then for _, p in ipairs(list) do if not kind or p.kind == kind then out[#out + 1] = p end end end
  end
  return out
end
local function all_text(h)
  local out = {}
  for _, list in pairs(h and h.zones or {}) do
    for _, p in ipairs(list) do
      for _, k in ipairs({ 'text', 'title', 'rule', 'wait' }) do if type(p[k]) == 'string' then out[#out + 1] = p[k] end end
      for _, l in ipairs(p.lines or {}) do out[#out + 1] = l end
    end
  end
  return out
end
local function start(opts)   -- a run just begun: the starter's RUN START toast has been sent (one frame), so later toasts are the test's own
  local e = F.start(D, opts); D.atlas_hud.reset(); F.stage(e.host); e.host.hud:frame(); return e
end
local function tick(e, n) for _ = 1, n or 1 do e.host.hud:frame() end end
local function foes(e, n)   -- n opponent plates, as the foe lab keeps them
  e.host.mods.foes.labels = {}
  for p = 2, n + 1 do e.host.mods.foes.labels[p] = { title = 'MARTH  CPU ' .. p, lines = { 'Pyromancer: All your attacks become fire.', '12 drive rules' }, left = 240 } end
end
local ARCH = { id = 'skyward', name = 'Skyward', colour = 0xB872F0, blurb = 'Your aerials gain Haste for 2 s.' }

T.test('the strip: slot pips with their colour and ring, keystone stones with their letter, "n waiting"', function()
  local e = start(); tick(e)
  local h = hud(e); assert(h and h.id == 'envoy.hud', 'one HUD, id envoy.hud')
  local s = parts(h, 'strip', 'top_left')[1]
  assert(s and #s.pips == e.host:bag():slots() and s.pips[1].fill ~= nil and s.pips[1].ring ~= nil, 'a pip per slot')
  assert(#s.keys >= 1 and #s.keys[1].letter == 1, 'the starting keystone is a stone with a letter')
  assert(s.wait == nil, 'nothing waits')
  e.host.decide = { e.host.mods.drives.loot:roll(5, e.host.mods.engine.context) }; e.host:touch(); tick(e)
  assert(parts(hud(e), 'strip', 'top_left')[1].wait == '1 waiting', 'a waiting drive is said')
  local d = e.ui.huds['envoy/main']
  for _, p in pairs(parts(d)) do assert(p.kind ~= 'strip' or (p.strength == nil and p.depth == nil), 'no strength figure, no depth text') end
end)

T.test('quiet HUD caps hold at the busiest moment', function()
  local e = start(); foes(e, 5)
  e.host.holding_end = true; e.host.hold_banner = 'Collect the drives'
  e.host.hud:corner({ { text = 'Skyward assembled' }, { text = ARCH.blurb } }, ARCH)
  e.host.hud:show_card('Merged!', { 'Lingering got stronger' })
  tick(e, 1)
  local h = hud(e)
  local ok, why = e.ui.hud_caps(h.zones); assert(ok, tostring(why))
  assert(#parts(h, 'card') == 3, 'five opponents, three cards: ' .. #parts(h, 'card'))
  assert(#parts(h, 'banner') == 1 and #parts(h, 'note') == 1 and #parts(h, 'toast') == 1, 'one banner, one note, one toast')
  assert(e.ui.hud_calls < 6)
end)

T.test('the collect banner carries the A glyph; the payout banner its leave progress', function()
  local e = start()
  e.host.holding_end = true; e.host.hold_banner = 'Collect the drives'; tick(e)
  local b = parts(hud(e), 'banner', 'top_center')[1]
  assert(b and b.text == 'Collect the drives' and b.button == 'A', 'press A to collect')
  e.host.paying = true; e.host.hold_banner = 'The stage pays out'
  e.host.leave_w = { progress = function() return 0.4 end }; tick(e)
  b = parts(hud(e), 'banner')[1]
  assert(b.text:find('leave', 1, true) and b.button == 'Z' and math.abs(b.progress - 0.4) < 0.001, 'the payout hold keeps its banner and its progress')
  e.host.holding_end = false; e.host.hold_banner = nil; tick(e)
  assert(#parts(hud(e), 'banner') == 0, 'released: no banner')
end)

T.test('no crit text, ever', function()
  local e = start()
  e.host.mods.toast('Critical hit x1.5 (crit)')
  e.host.hud:flash('Crit!')
  tick(e, 30)
  for _, s in ipairs(all_text(hud(e))) do assert(not s:lower():find('crit'), s) end
  for _, t in ipairs(parts(hud(e), 'toast')) do assert(t.title == 'RUN START', 'the only toast is the run start: ' .. t.title) end
  assert(#parts(hud(e), 'note') == 0, 'no note for a crit')
end)

T.test('link flashes still fire: the world-space chain is not touched by the HUD', function()
  local e = start()
  local f = e.host.synfx; assert(f, 'the host has the synergy fx')
  local eng = { frame = 100 }
  f.flashes = {}
  f:chain_fired(eng, 1, ARCH, .5)
  assert(#f.flashes >= 1, 'a flash was queued')
  tick(e, 5)
  assert(#f.flashes >= 1, 'the HUD did not consume the flash')
  for _, t in ipairs(parts(hud(e), 'toast')) do assert(t.title == 'RUN START', 'no HUD text for a link flash: ' .. t.title) end
end)

T.test('the synergy notice is a small top-corner toast, never a banner', function()
  local e = start()
  e.host.hud:corner({ { text = 'Skyward assembled' }, { text = ARCH.blurb } }, ARCH); tick(e)
  local t = parts(hud(e), 'toast', 'top_right')[1]
  assert(t and t.title:find('ASSEMBLED', 1, true) and t.title == 'SKYWARD ASSEMBLED' and not t.text:find('\n') and t.rgba == 0xB872F0FF, 'a toast with the emblem colour: ' .. tostring(t and t.title))
  assert(#parts(hud(e), 'banner') == 0)
  local n = e.ui.toast_calls; tick(e, 60); assert(e.ui.toast_calls == n, 'sent once, not per frame')
end)

T.test('the run start is a toast in the seat\'s corner, not a centred panel', function()
  local e = F.start(D); D.atlas_hud.reset(); F.stage(e.host); tick(e)
  local t = parts(hud(e), 'toast', 'top_right')[1]
  assert(t and t.title == 'RUN START' and t.text ~= '' and not t.text:find('\n'), 'RUN START with the drive\'s one rule: ' .. tostring(t and t.text))
end)

T.test('the pickup note is one line; the out-of-bounds notice is a note too', function()
  local e = start(); tick(e)
  e.host.hud:show_card('Merged!', { 'Lingering got stronger' }); tick(e)
  local n = parts(hud(e), 'note', 'bottom_left')[1]
  assert(n and n.text == 'Merged!: Lingering got stronger' and not n.text:find('\n'), tostring(n and n.text))
  e.host.hud.card = nil; e.host.hud.card_left = 0
  e.host.hud:flash('Out of bounds: a stock is lost', true); tick(e)
  assert(parts(hud(e), 'note')[1].text:find('Out of bounds', 1, true), 'the OOB notice')
  e.host.hud.flash_left = 0; e.host.hud:flash('Slot 2 changed'); tick(e)   -- a developer flash is not shown in play
  for _, s in ipairs(all_text(hud(e))) do assert(not s:find('Slot 2 changed', 1, true), 'developer flashes stay developer-only') end
end)

T.test('co-op seats use their own corners', function()
  local e1 = start({ seat = { port = 1, index = 1 } })
  local h2 = F.new(D, { seat = { port = 2, index = 2 } }); h2.ui = e1.ui; h2.host.g = e1.g
  -- seat 2's host: the same gd (one script), its own hud
  h2.host.hud.host = h2.host; h2.host.g = e1.g
  h2.run(true); h2.host:run_begin(4243); F.stage(h2.host)
  tick(e1); h2.host.hud:frame()
  local h = hud(e1)
  assert(#parts(h, 'strip', 'top_left') == 1 and #parts(h, 'strip', 'top_right') == 1, 'each seat\'s strip in its own corner')
  e1.host.hud:corner({ { text = 'Skyward assembled' }, { text = ARCH.blurb } }, ARCH); tick(e1)
  assert(parts(hud(e1), 'toast', 'top_left')[1], 'seat 1\'s toast is in its own corner')
  local ok, why = e1.ui.hud_caps(hud(e1).zones); assert(ok, tostring(why))
end)

T.test('the HUD stays within the engine\'s zones: nothing in bottom_center, cards only top right', function()
  local e = start(); foes(e, 3); tick(e)
  for z in pairs(hud(e).zones) do assert(z ~= 'bottom_center' and z ~= 'bottom_right', 'zone ' .. z) end
  assert(#parts(hud(e), 'card', 'top_right') == 3)
end)

T.test('rebuilt only on change', function()
  local e = start(); tick(e)
  local n = e.ui.hud_calls; tick(e, 120)
  assert(e.ui.hud_calls == n, 'a quiet second describes nothing: ' .. e.ui.hud_calls - n)
  e.host.holding_end = true; e.host.hold_banner = 'Collect the drives'; tick(e)
  assert(e.ui.hud_calls == n + 1, 'a banner is one description')
end)

T.test('netplay: the HUD draws, nothing is hidden', function()
  local e = start({ netplay = true }); tick(e)
  assert(hud(e) ~= nil and e.ui.retail_hide_calls == 0, 'described online, never hid retail')
end)

T.test('the switch: off, legacy and the developer overlay keep the legacy HUD', function()
  local e = start(); tick(e); assert(hud(e))
  D.atlas_kit.set(false); tick(e)
  assert(hud(e) == nil, 'off: the Atlas HUD is taken down')
  D.atlas_kit.set(true); D.atlas_kit.set_legacy(true); tick(e); assert(hud(e) == nil, 'envoy ui legacy: none')
  D.atlas_kit.set_legacy(false); tick(e); assert(hud(e), 'back on')
  assert(D.atlas_hud.on(e.host) == true)
  D.mod_tuning = D.mod_tuning or nil
end)

T.test('the run end takes the HUD down', function()
  local e = start(); tick(e); assert(hud(e))
  e.host:run_end()
  assert(hud(e) == nil, 'cleared with the run')
end)

T.test('a sync error turns the Atlas HUD off for good: the legacy HUD stays', function()
  local e = start(); tick(e); assert(D.atlas_hud.on(e.host) and hud(e))
  e.host.hud.model = function() error('boom') end
  tick(e)
  assert(e.host.hud.atlas_failed == true, 'the failure was recorded')
  assert(D.atlas_hud.on(e.host) == false, 'on() is false now, so Hd:draw draws the legacy HUD')
  local logged = false; for _, l in ipairs(e.s.logs) do if l:find('atlas hud failed', 1, true) then logged = true end end
  assert(logged, 'and logged once')
end)

T.test('a scene change describes the HUD again, so a live note is re-sent', function()
  local e = start(); tick(e)
  e.host.hud:show_card('Merged!', { 'Lingering got stronger' }); tick(e)
  local n = e.ui.hud_calls
  tick(e, 30); assert(e.ui.hud_calls == n, 'quiet: nothing re-described')
  e.mods:scene()                                   -- the engine cleared the toasts and notes with the scene
  tick(e)
  assert(e.ui.hud_calls == n + 1, 'the next sync described it again')
  assert(parts(hud(e), 'note', 'bottom_left')[1], 'the note is back')
end)

T.done()
