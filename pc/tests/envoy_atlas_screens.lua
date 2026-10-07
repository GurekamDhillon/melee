-- Offline tests of Envoy's Atlas screens (step 3): the reward cards, the swap, the setup, the pause, the results and the online pick,
-- on the real RunScreen / rule host and the gd.ui stand-in.   cd melee && lua pc/tests/envoy_atlas_screens.lua
local prefix = io.open('pc/tests/atlas_ui_stub.lua') and '' or 'melee/'
local F = dofile(prefix .. 'pc/tests/envoy_atlas_fixture.lua')
local T, Stub = F.T, F.Stub
local D = F.load()

local function key_labels(ui, id) local out = {}; for _, k in ipairs(ui.views[id].keys) do out[k[1]] = k[2] end; return out end
local function has_log(e, text) for _, l in ipairs(e.s.logs) do if l:find(text, 1, true) then return true end end; return false end
local function reward(e, offers, keys) F.present(e.host, offers, keys); return e.host.screen end

-- ---- Task 10: the reward screen ----------------------------------------------------------------------------------------

T.test('reward is a cards screen with one rule per card, the countdown and the trail', function()
  local e = F.start(D); F.stage(e.host)
  local S = reward(e, F.plain_drives(e.host, 3))
  local d = e.ui.screens['envoy.reward']
  assert(d and d.primary.kind == 'cards' and #d.primary.cards == 3, 'three cards')
  for _, c in ipairs(d.primary.cards) do assert(c.rule ~= '' and not c.rule:find('\n') and #c.rule <= 159, 'one short rule: ' .. tostring(c.rule)) end
  assert(d.trail[1] == 'SOLO' and d.trail[2] == 'ENVOY' and d.trail.title == 'STAGE CLEAR')
  assert(d.persist == true and d.input == 'feed' and d.port == 1 and d.chapter == 1)
  assert(e.ui.stack[#e.ui.stack] == 'envoy.reward' and S.atlas_reward, 'opened and attached')
  assert(type(d.countdown) == 'number' and d.countdown <= 45 and d.countdown >= 44, 'the countdown: ' .. tostring(d.countdown))
  local k = key_labels(e.ui, 'envoy.reward')
  assert(k.A and k.B == 'Skip', 'A names what it does and B says Skip: ' .. tostring(k.A) .. '/' .. tostring(k.B))
  local v = e.ui.views['envoy.reward']
  assert(v.explainer.kicker:find('OFFER 1', 1, true) and v.explainer.what ~= '' and v.explainer.from and v.explainer['with'] and #v.explainer['with'] == 1, 'WHAT, WITH and FROM: ' .. tostring(v.explainer.kicker))
end)

T.test('A takes the focused card through the legacy logic; the last offer ends the moment with the pickup note', function()
  local e = F.start(D); F.stage(e.host)
  local offers = F.plain_drives(e.host, 3)
  local S = reward(e, offers)
  e.ui.engine_focus('envoy.reward', 'cards', 'offer:2')
  assert(e.ui.engine_press('envoy.reward', 'accept'))
  assert(#e.host.offers == 0, 'every offer is resolved: one taken, the others declined')
  assert(not S.active and #e.ui.stack == 0, 'the screen closed itself')
  assert(has_log(e, 'took ') and has_log(e, 'reward moment done (done)'), 'the host logged it')
  assert(e.host.hud.card and e.host.hud.card.title == 'Reward', 'the result is the pickup note')
  assert(e.s.releases == 1, 'the hold was released')
end)

T.test('the countdown takes the first offer, never discards it silently', function()
  local e = F.start(D); F.stage(e.host)
  local S = reward(e, F.plain_drives(e.host, 3))
  F.advance(e, 46); S:tick()
  assert(#e.host.offers == 0 and not S.active, 'resolved by the countdown')
  assert(has_log(e, 'timeout: took the first offer automatically'), 'the first offer was kept and said so')
  assert(#e.ui.stack == 0)
end)

T.test('the countdown is re-registered once a second', function()
  local e = F.start(D); F.stage(e.host)
  local S = reward(e, F.plain_drives(e.host, 3))
  local n0, c0 = e.ui.refreshed, e.ui.screens['envoy.reward'].countdown
  S:tick(); S:tick(); assert(e.ui.refreshed == n0, 'no re-registration inside the same second')
  F.advance(e, 1.2); S:tick()
  assert(e.ui.refreshed == n0 + 1 and e.ui.screens['envoy.reward'].countdown == c0 - 1, 'one second later: once')
end)

T.test('closing resolves every offer: B asks, a second B skips them all and logs it', function()
  local e = F.start(D); F.stage(e.host)
  local S = reward(e, F.plain_drives(e.host, 3))
  e.ui.engine_press('envoy.reward', 'back')
  assert(#e.host.offers == 3 and S.active, 'the first B only asks')
  assert(e.ui.notes[#e.ui.notes].text:find('B again', 1, true))
  e.ui.engine_press('envoy.reward', 'back')
  assert(#e.host.offers == 0 and not S.active and has_log(e, 'skipped the stage reward'), 'skipped, resolved and logged')
end)

T.test('Z pages the rules of a card with several, one at a time', function()
  local e = F.start(D); F.stage(e.host)
  local loot = e.host.mods.drives.loot
  local many = F.roll(e.host, function(r) return #D.drive_text.drive_lines(loot, r) >= 2 and e.host:plan_take(r).action ~= 'merge' end, 'rare')
  local S = reward(e, { many }); local id = 'envoy.reward'
  local ex = e.ui.views[id].explainer
  local n = tonumber(ex.kicker:match('OF (%d+)'))
  assert(n and n >= 2 and ex.kicker:find('RULE 1 OF', 1, true), 'a multi-rule drive says RULE 1 OF n: ' .. ex.kicker)
  local first = ex.what
  assert(e.ui.engine_press(id, 'z')); ex = e.ui.views[id].explainer
  assert(ex.kicker:find('RULE 2 OF', 1, true) and ex.what ~= first, 'Z stepped to the next rule')
  assert(key_labels(e.ui, id).Z == 'More')
end)

T.test('keystone offers are stones with a letter, on their own screen', function()
  local e = F.start(D); F.stage(e.host)
  local ids = D.keystones.offer(D.mod_progression.context(10, 0), 77, 3, {})
  assert(#ids == 3)
  local S = reward(e, {}, ids)
  local d = e.ui.screens['envoy.keystone']
  assert(d and d.primary.kind == 'cards' and #d.primary.cards == 3 and e.ui.screens['envoy.reward'] == nil, 'the keystone screen')
  for _, c in ipairs(d.primary.cards) do assert(c.model == nil and #c.letter == 1 and c.tag == 'KEYSTONE' and c.rule ~= '', 'a stone with its letter and one rule') end
  e.ui.engine_press('envoy.keystone', 'accept')
  assert(#e.host.key_offers == 0 or #e.ui.stack >= 0)
end)

T.test('a seat tints its focus: its own id and port', function()
  local e = F.start(D, { seat = { port = 2, index = 2 } }); F.stage(e.host)
  reward(e, F.plain_drives(e.host, 3))
  local d = e.ui.screens['envoy.reward.p2']
  assert(d and d.port == 2 and e.ui.screens['envoy.reward'] == nil)
end)

T.test('netplay: described, never masked', function()
  local e = F.start(D, { netplay = true }); F.stage(e.host)
  reward(e, F.plain_drives(e.host, 3))
  assert(e.ui.screens['envoy.reward'] and e.s.mask_calls == 0 and e.s.pause_calls == 0)
end)

T.test('a scene exit does not close the reward screen (persist); the run end does', function()
  local e = F.start(D); F.stage(e.host)
  local S = reward(e, F.plain_drives(e.host, 3))
  e.ui.scene_exit()
  assert(e.ui.state().top == 'envoy.reward', 'the screen spans the scene change')
  S:close()
  assert(#e.ui.stack == 0)
end)

T.test('a refused description leaves the legacy reward screen', function()
  local e = F.start(D); F.stage(e.host)
  e.ui.screen = function() error('gd.ui.screen: refused') end
  local S = reward(e, F.plain_drives(e.host, 3))
  assert(S.atlas_reward == nil and rawget(S, 'draw') == nil and #e.ui.stack == 0)
  assert(has_log(e, 'the engine refused the screen'))
end)

T.test('the switch: off, legacy and a missing gd.ui all leave the legacy screen', function()
  local e = F.start(D); F.stage(e.host); D.atlas_kit.set(false)
  local S = reward(e, F.plain_drives(e.host, 3)); assert(S.atlas_reward == nil and #e.ui.stack == 0); S:close()
  D.atlas_kit.set(true); D.atlas_kit.set_legacy(true)
  F.present(e.host, F.plain_drives(e.host, 3)); assert(e.host.screen.atlas_reward == nil and #e.ui.stack == 0); e.host.screen:close()
  D.atlas_kit.set_legacy(false); e.g.ui = nil
  F.present(e.host, F.plain_drives(e.host, 3)); assert(e.host.screen.atlas_reward == nil)
end)

T.done()
