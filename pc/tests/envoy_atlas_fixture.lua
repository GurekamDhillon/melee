-- The shared fixture of the Envoy Atlas screen and HUD tests (step 3): the real rule host, run screen and Atlas modules on a fake `gd`
-- whose `ui` is the Atlas stand-in. Built from what envoy_run_ux.lua constructs. It also builds co-op seats (see F.seat).
local prefix = io.open('pc/tests/atlas_ui_stub.lua') and '' or 'melee/'
local Stub = dofile(prefix .. 'pc/tests/atlas_ui_stub.lua')
local T = dofile(prefix .. 'pc/tests/envoy_testlib.lua')
local F = { Stub = Stub, T = T }

local MODULES = { 'mod_progression', 'mod_schema', 'mod_codec', 'mod_engine', 'keystones', 'mod_pool', 'drive_loot', 'drive_merge', 'drive_economy', 'drive_bag', 'foe_roll' }
local LATE = { 'menu_input', 'drive_menu', 'drive_text', 'drive_drop', 'drive_lab', 'foe_lab', 'mod_lab', 'run_screen', 'atlas_kit', 'atlas_bag', 'atlas_reward', 'atlas_swap',
  'atlas_setup', 'atlas_pause', 'atlas_results', 'atlas_netpick', 'run_hud', 'atlas_hud', 'run_host' }

-- the modules, loaded once per test file; a module that does not exist yet is skipped (the fixture serves the tasks in order)
function F.load()
  local D = {}
  for _, n in ipairs(MODULES) do D[n] = T.module(n, D) end
  D.grid = dofile(T.root .. '../../demos/grid-inventory/scripts/grid.lua')
  D.mod_display = { new = function(g, engine)
    local v = { engine = engine }
    function v:warm() return D.warm_ready ~= false end
    function v:update() end
    function v:clear() end
    function v:on_loadstate(e) self.engine = e end
    function v:tick() end
    return v
  end }
  D.pickup_juice = { pitch = {}, new = function() return { drop = function() return { fx = {} } end, collect = function() end, expire = function() end, clear = function() end, tick = function() end } end }
  for _, n in ipairs(LATE) do
    if io.open(T.root .. n .. '.lua') then D[n] = T.module(n, D) end
  end
  D.drive_economy.tuning.floor_chance = 1
  return D
end

-- one fake game, one host. opts: netplay, seat ({port=, index=}), gameplay
function F.new(D, opts)
  opts = opts or {}
  local s = { commands = {}, pad = {}, logs = {}, holds = 0, releases = 0, clock = 100, spawns = 0, despawns = {}, held = true, mask_calls = 0, pause_calls = 0,
    players = { [1] = { x = 0, y = 0, percent = 0, stocks = 3, falls = 0, char = 1, action = 14 }, [2] = { x = 30, y = 0, percent = 0, stocks = 1, falls = 0, char = 2, action = 14, cpu = true } } }
  local running = false
  local ui = Stub.new({ mod = 'envoy', netplay = opts.netplay })
  local g = { fixture = s, ui = ui, command = function(n, f) s.commands[n] = f end, log = function(t) s.logs[#s.logs + 1] = t end,
    match = function() return { active = true, netplay = opts.netplay or false, stage = 32 } end, lab_mode = function() return false end,
    sim_supported = true, sim_replaying = function() return false end, player = function(p) return s.players[p] end,
    hit_rule_add = function() error('direct native write') end, fighter_status = function() error('direct native write') end,
    hit_rules = function() return { percent_only = true, progression = true, owner = 7 } end,
    sim_clear = function() s.blob = nil end, sim_read = function() return s.blob end,
    sim_commit = function(blob, ops) s.blob = blob; s.ops = ops; return true end,
    pad = function() return s.pad end, input_mask = function(_, v) s.mask_calls = s.mask_calls + 1; s.mask = v end, input_chord = function() error('Atlas must never chord the pad') end,
    paused = function() return s.paused end, pause = function() s.pause_calls = s.pause_calls + 1; s.paused = true end, resume = function() s.paused = false end, items = function() return {} end,
    item_spawn = function(_, x, y, o) s.spawns = s.spawns + 1; s.payload = o.payload; s.last_x, s.last_y = x, y; return 100 + s.spawns end, item_despawn = function(h) s.despawns[#s.despawns + 1] = h; return true end,
    match_end_hold = function(r, on) s.endhold = s.endhold or {}; s.endhold[r] = on or nil; return true end,
    hold_1p = function() s.holds = s.holds + 1; s.held = true; return not s.no_hold end, release_1p = function() s.releases = s.releases + 1; s.held = false; return true end,
    mode_1p = function() return { held = s.held, mode = 'classic' } end, time = function() return s.clock end,
    safe_area = function() return { x = 0, y = 0, w = 640, h = 480, right = 640, bottom = 480 } end,
    stage_bounds = function() return { camera = { left = -100, right = 100, top = 100, bottom = -100 } } end, floor_below = function(x, y) return 0 end }
  local retail = { state = { player_port = 1 }, loop = 0, rules = true, active = true }
  local mods = D.mod_lab.new(g, { run_host = function() return running end, run_ready = function() return true end })
  local host = D.run_host.new(g, mods, retail); retail.host = host
  if opts.seat then host.seat = opts.seat end
  D.atlas_kit.set(true); D.atlas_kit.set_legacy(false); D.atlas_kit.logged = {}
  local env = { s = s, g = g, ui = ui, mods = mods, host = host, retail = retail, run = function(v) running = v end, D = D }
  return env
end

function F.start(D, opts, seed)
  local e = F.new(D, opts); e.run(true); e.host:run_begin(seed or 4242)
  return e
end

function F.stage(host, e)
  e = e or {}
  host:stage_start({ stage_index = e.stage or 0, loop = e.loop or 0, stage_kind = e.kind or 'battle', opponents = e.opponents or { { port = 2 } } }); host.since = 99
end

function F.roll(host, pred, forced, ctx)
  for seed = 1, 4000 do local r = host.mods.drives.loot:roll(seed, ctx or host.mods.engine.context, forced); if pred(r) then return r end end
  error('no matching roll')
end
-- n drives that merge into nothing held and into each other not at all
function F.plain_drives(host, n)
  local out, used = {}, {}
  for _ = 1, n do
    local r
    for seed = 1, 4000 do
      local c = host.mods.drives.loot:roll(seed, host.mods.engine.context)
      if host:plan_take(c).action ~= 'merge' and not used[c.colour .. tostring(c.rarity)] then r = c; used[c.colour .. tostring(c.rarity)] = true; break end
    end
    out[#out + 1] = assert(r, 'no plain roll')
  end
  return out
end
function F.present(host, offers, keys) host.offers = offers; host.key_offers = keys or {}; host.screen:open('reward') end
function F.advance(env, seconds) env.s.clock = env.s.clock + seconds end

return F
