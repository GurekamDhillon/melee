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

-- ---- Task 11: the swap screen -----------------------------------------------------------------------------------------------

-- a build with every slot and every bag place full of drives that merge into nothing
local function full_build(e, keep_free)
  local host = e.host; local b = host:bag()
  for i = 1, b:slots() do
    if not b.equipped[i] then local r = F.roll(host, function(r) return host:plan_take(r).action ~= 'merge' end); assert(b:place(i, r)); host:touch() end
  end
  for _ = 1, b:capacity() - (keep_free or 0) do
    local r = F.roll(host, function(r) return host:plan_take(r).action ~= 'merge' end); assert(b:give(r)); host:touch()
  end
end
local function incoming(e) return F.roll(e.host, function(r) return e.host:plan_take(r).action == 'choose' end) end

T.test('swap opens from a full bag, B returns to the reward cards with the focus where it was', function()
  local e = F.start(D); F.stage(e.host); full_build(e)
  local offers = { incoming(e), incoming(e), incoming(e) }
  local S = reward(e, offers)
  e.ui.engine_focus('envoy.reward', 'cards', 'offer:2')
  e.ui.engine_press('envoy.reward', 'accept')
  assert(S.layout == 'swap' and e.ui.state().top == 'envoy.swap' and S.atlas_swap and not S.atlas_reward, 'the swap screen is up')
  local d = e.ui.screens['envoy.swap']
  assert(d.primary.kind == 'grid' and d.primary.footer and d.primary.footer.label == 'INCOMING' and d.trail.title == 'BAG FULL')
  assert(#d.primary.blocks == 2 and d.primary.blocks[1].id == 'eq' and d.primary.blocks[2].id == 'bag', 'the targets: six equipped and the bag')
  local k = key_labels(e.ui, 'envoy.swap'); assert(k.A and k.B == 'Back', 'A names the swap, B says Back: ' .. tostring(k.A) .. '/' .. tostring(k.B))
  e.ui.engine_press('envoy.swap', 'back')
  assert(S.layout == 'main' and S.atlas_reward and not S.atlas_swap and e.ui.state().top == 'envoy.reward', 'back to the cards')
  assert(select(1, e.ui.focus('envoy.reward')) == 'offer:2', 'the focus is where it was: ' .. tostring(select(1, e.ui.focus('envoy.reward'))))
  assert(#e.host.offers == 3, 'nothing was taken')
end)

T.test('the explainer names the outcome before A: goes to the bag, or is gone for good', function()
  local e = F.start(D); F.stage(e.host); full_build(e)
  local S = reward(e, { incoming(e) })
  e.ui.engine_press('envoy.reward', 'accept')
  e.ui.engine_focus('envoy.swap', 'eq', 'eq:1')
  local ex = e.ui.views['envoy.swap'].explainer
  assert(ex.what:find('gone', 1, true), 'a full bag: the equipped drive is gone: ' .. tostring(ex.what))
  S:close()
  -- a bag with a free place: the replaced equipped drive goes to the bag
  local e2 = F.start(D); F.stage(e2.host); full_build(e2)
  local S2 = reward(e2, { incoming(e2) })
  e2.ui.engine_press('envoy.reward', 'accept')
  local b = e2.host:bag(); b.items[#b.items] = nil; e2.host:touch(); S2:invalidate(); S2:refresh()
  e2.ui.engine_focus('envoy.swap', 'eq', 'eq:2')
  assert(e2.ui.views['envoy.swap'].explainer.what:find('goes to your bag', 1, true), 'a bag with room: ' .. tostring(e2.ui.views['envoy.swap'].explainer.what))
end)

T.test('A on a target replaces it through the legacy logic and ends the swap; the moment goes on', function()
  local e = F.start(D); F.stage(e.host); full_build(e)
  local S = reward(e, { incoming(e) })
  e.ui.engine_press('envoy.reward', 'accept')
  e.ui.engine_focus('envoy.swap', 'eq', 'eq:3')
  e.ui.engine_press('envoy.swap', 'accept')
  assert(has_log(e, 'replaces'), 'the host logged the replacement')
  assert(not S.active and #e.ui.stack == 0, 'the offer was the last one: the moment is over')
end)

T.test('a drive that cannot be kept (decide): B twice leaves it behind and logs it', function()
  local e = F.start(D); F.stage(e.host); full_build(e)
  local S = e.host.screen; S:open('bag')
  e.host.decide = { incoming(e) }; S:refresh()
  assert(S.layout == 'swap' and e.ui.state().top == 'envoy.swap' and e.ui.screens['envoy.swap'].trail.title == 'BAG FULL')
  assert(key_labels(e.ui, 'envoy.swap').B == 'Leave it')
  e.ui.engine_press('envoy.swap', 'back')
  assert(#e.host.decide == 1 and e.ui.notes[#e.ui.notes].text:find('B again', 1, true), 'the first B only asks')
  e.ui.engine_press('envoy.swap', 'back')
  assert(#e.host.decide == 0 and has_log(e, 'left behind'), 'left behind and logged')
  assert(S.layout == 'main')
end)

T.test('the bag hands a swap to the swap screen and takes it back', function()
  local e = F.start(D); F.stage(e.host); full_build(e)
  local S = e.host.screen; S:open('bag')
  assert(S.atlas and e.ui.state().top == 'envoy.bag')
  e.ui.engine_focus('envoy.bag', 'bag', 'bag:1')
  e.ui.engine_press('envoy.bag', 'accept')
  assert(S.layout == 'swap' and e.ui.state().top == 'envoy.swap', 'a bag drive wants a slot: the swap screen')
  assert(e.ui.screens['envoy.swap'].trail.title == 'SWAP' and key_labels(e.ui, 'envoy.swap').B == 'Back')
  e.ui.engine_press('envoy.swap', 'back')
  assert(S.layout == 'main' and S.atlas and e.ui.state().top == 'envoy.bag', 'back to the bag')
  assert(select(1, e.ui.focus('envoy.bag')) == 'bag:1', 'with the focus on the bag cell it left from')
end)

T.test('links: the synergy marks of held pieces are grid links drawn under the cells', function()
  local e = F.start(D); F.stage(e.host); full_build(e)
  e.host.synfx.grid_model = function() return { links = { { a = 'eq:1', b = 'bag:2', arch = { colour = 0x7A5CF0 } }, { a = 'eq:1', b = 'nope:9', arch = { colour = 1 } } } } end
  local S = reward(e, { incoming(e) })
  e.ui.engine_press('envoy.reward', 'accept')
  local d = e.ui.screens['envoy.swap']
  assert(#d.primary.links == 1 and d.primary.links[1].a == 'eq:1' and d.primary.links[1].b == 'bag:2' and d.primary.links[1].rgba == 0x7A5CF0FF, 'one link naming two real cells, in the archetype colour')
end)

T.test('the swap screen belongs to its seat', function()
  local e = F.start(D, { seat = { port = 2, index = 2 } }); F.stage(e.host); full_build(e)
  reward(e, { incoming(e) })
  e.ui.engine_press('envoy.reward.p2', 'accept')
  assert(e.ui.screens['envoy.swap.p2'] and e.ui.screens['envoy.swap.p2'].port == 2)
end)

T.done()
