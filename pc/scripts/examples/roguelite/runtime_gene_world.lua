-- Native gene-world adapter (Gate 5 world seam), review-2 correction.
--
-- Bind the real Core/gene_actions transaction to the documented gd.* native
-- APIs without inventing hit evidence. `GeneWorld.new(Core, engine, opts)`
-- returns a plain `world` table whose callbacks are the ones named in
-- docs/GENE-ACTION-CONTRACT.md plus two optional generic intent hooks:
--
--   get_run, observe, can_start, query, occluded,
--   apply_effect, apply_movement, apply_status,
--   apply_guard/release_guard, apply_conversion/release_conversion
--   capture_intent(run, host, slot, target, spec) -> opaque token
--   validate_intent(run, token) -> exact true, or false/reason
--
-- The runtime caller owns actor registration and native bindings. This module
-- never reads or writes profile genes, saves, rooms, main.lua or campaign
-- internals. It resolves a bound logical host to a fighter port or owned
-- Adventure handle and refuses anything the confirmed native build or an
-- explicit provider does not implement.
--
-- Identity: `capture_intent` freezes the run, source binding generation/ref/
-- port/handle/team/room/token and the selected target identity at begin.
-- `validate_intent` re-checks them (and target eligibility) before the spend
-- point, so a recycled host or port for a different encounter cancels the
-- action before Core.activate, including Core-owned Rime marks. A confirmed
-- capability that disappears also cancels. Absent hooks leave injected worlds
-- without native identity unchanged; a present hook fails closed.
--
-- Native acceptance contracts (read-only audit, native-hit-refusal-audit.md):
-- gd.hit and gd.enemy_strike route through ftColl_80076640, where native false
-- can already have chipped armor. Until root wires corrected native APIs, those
-- two paths require an explicit caller opt-in (`native_hit_contract =
-- 'accepted_damage'`); without it they refuse before the native call.
-- gd.enemy_hurt and gd.impulse have separate audited contracts and need no
-- opt-in. Even with the opt-in, `true` means "accepted native damage action"
-- (including armor absorption), never proof of a landed percent hit.
--
-- Eligibility: player immunity is read from the real gd.player body_state /
-- timed_state / invincible / intangible fields and shield_on; an unknown
-- eligibility state fails closed. Enemy vulnerability comes from
-- gd.enemy_state(...).vulnerable and a missing value fails closed. Armor flag
-- x221C_b4, armorHP x1834 and the opaque collision immunity flag x221D_b6 are
-- not exposed by the readers; that gap is documented, not guessed.
--
-- There is no native line-of-sight function. `occluded` fails closed until the
-- caller injects an explicit trustworthy `visibility` provider.
local GeneWorld = {version = 2, schema = 2}

-- Native limits copied from the confirmed contracts (docs/scripting.md and the
-- gw_script.c registration/validation). Refuse outside them before calling in.
GeneWorld.bounds = {
  hit = {damage = {0, 500}, angle = {0, 361}, kbg = {0, 1000}, bkb = {0, 1000}},
  enemy = {damage = {1, 30}, angle = {0, 361}, kbg = {0, 1000}, bkb = {0, 1000}, reach = {1, 30}},
  impulse = {x = 4, y = 3},
  default_targets = 32,
  default_bindings = 32,
  default_contributions = 64,
}

-- Offered placement capabilities only; this is not the behavior source of
-- truth (gene_behaviors.lua is). It lets the capability table report per
-- (gene, slot, direction) eligibility.
GeneWorld.offered = {
  cinder = {
    assault = {kind = 'burst', native = true, targeting = 'opponent'},
    traversal = {kind = 'dash', native = true, targeting = 'self'},
    guard = {kind = 'counter', native = true, targeting = 'opponent'},
  },
  rime = {
    assault = {kind = 'mark', native = true, targeting = 'opponent'},
    traversal = {kind = 'hover', native = true, targeting = 'self'},
    guard = {kind = 'mark', native = true, targeting = 'opponent'},
  },
}

local seam_by_kind = {
  burst = 'apply_effect', line = 'apply_effect', chain = 'apply_effect', push = 'apply_effect',
  mark = 'apply_status', root = 'apply_status',
  dash = 'apply_movement', hover = 'apply_movement',
  guard = 'apply_guard', counter = 'apply_effect',
  convert = 'apply_conversion', siphon = 'apply_conversion',
}

local function seam_for(spec)
  local kind = spec.effect.kind
  if kind == 'mark' and spec.native then return nil end
  if kind == 'counter' and spec.targeting == 'self' then return 'apply_guard' end
  return seam_by_kind[kind]
end

local function finite(x)
  return type(x) == 'number' and x == x and math.abs(x) < math.huge
end
local function integer(x)
  return finite(x) and x % 1 == 0
end
local function name(x)
  return type(x) == 'string' and #x > 0 and #x <= 64 and not x:find('[%z\1-\31]')
end
local function copy(t)
  if type(t) ~= 'table' then return t end
  local o = {}
  for k, v in pairs(t) do o[k] = copy(v) end
  return o
end
local function count(t)
  local n = 0
  for _ in pairs(t) do n = n + 1 end
  return n
end
local function active_run(run)
  return type(run) == 'table' and run.type == 'run' and run.version == 1 and run.status == 'active'
end

function GeneWorld.new(Core, engine, opts)
  assert(type(Core) == 'table', 'Core module required')
  assert(type(engine) == 'table', 'engine binding required')
  assert(type(engine.gd) == 'table', 'engine.gd native table required')
  assert(type(engine.get_run) == 'function', 'engine.get_run required')
  opts = opts or {}

  local hit_contract = engine.native_hit_contract or opts.native_hit_contract
  local state = {
    Core = Core, gd = engine.gd,
    get_run_provider = engine.get_run,
    current_room = engine.current_room or opts.current_room,
    visibility = engine.visibility or opts.visibility,
    reflecting = engine.reflecting or opts.reflecting,
    invulnerable = engine.invulnerable or opts.invulnerable,
    status = engine.status or opts.status,
    guard = engine.guard or opts.guard,
    conversion = engine.conversion or opts.conversion,
    hit_contract = (hit_contract == true or hit_contract == 'accepted_damage') and true or false,
    max_targets = opts.max_targets or GeneWorld.bounds.default_targets,
    max_bindings = opts.max_bindings or GeneWorld.bounds.default_bindings,
    max_contributions = opts.max_contributions or GeneWorld.bounds.default_contributions,
    binds = {}, contributions = {},
    bind_serial = 0, contrib_serial = 0, reserved = 0, run = nil,
  }
  assert(integer(state.max_targets) and state.max_targets >= 1, 'invalid max_targets')
  assert(integer(state.max_bindings) and state.max_bindings >= 1, 'invalid max_bindings')
  assert(integer(state.max_contributions) and state.max_contributions >= 1, 'invalid max_contributions')
  for _, fn in ipairs({state.current_room, state.visibility, state.reflecting,
    state.invulnerable, state.status}) do
    assert(fn == nil or type(fn) == 'function', 'invalid engine provider')
  end
  for _, prov in ipairs({state.guard, state.conversion}) do
    assert(prov == nil or (type(prov) == 'table'
      and (prov.apply == nil or type(prov.apply) == 'function')
      and (prov.release == nil or type(prov.release) == 'function')), 'invalid engine provider')
  end

  -- Contribution capacity is RESERVED before the provider is invoked, so a full
  -- adapter never reaches native at all. `reserved` counts provider calls that
  -- are in flight, so a provider callback that reentrantly applies another
  -- contribution cannot slip past the limit. A reservation is either released
  -- (no identifiable token was issued) or converted into the owned record (a
  -- token exists and keeps the capacity occupied until its release is
  -- confirmed). Records retained by a refused release stay counted, so a
  -- permanently refused cleanup can never be used to mint unbounded tokens.
  local function contribution_capacity_ok()
    return (count(state.contributions) + state.reserved) < state.max_contributions
  end

  local function reserve_contribution()
    if not contribution_capacity_ok() then return false, 'contribution capacity' end
    state.reserved = state.reserved + 1
    return true
  end

  local function drop_reservation() state.reserved = state.reserved - 1 end

  local world = {}
  local function current_room()
    if not state.current_room then return nil end
    local ok, room = pcall(state.current_room)
    if not ok then return nil end
    return room
  end
  local function same_room(a, b, current)
    if current ~= nil then return a.room == current and b.room == current end
    if a.room == nil or b.room == nil then return false end
    return a.room == b.room
  end
  local function provider_for(kind)
    if kind == 'guard' then return state.guard end
    if kind == 'conversion' then return state.conversion end
    return nil
  end

  -- Actual native/provider capability for one effect spec and direction. This
  -- is the capability proof: a callable adapter stub alone is not enough.
  local function native_capability(spec, source, target)
    if type(spec) ~= 'table' or type(spec.effect) ~= 'table' then return false, 'invalid effect spec' end
    local seam = seam_for(spec)
    if seam == nil then return true, 'core' end
    local gd = state.gd
    if seam == 'apply_effect' then
      if type(source) ~= 'table' then return false, 'no source binding' end
      if source.kind == 'player' then
        if target and target.kind == 'enemy' then
          if type(gd.enemy_hurt) ~= 'function' then return false, 'gd.enemy_hurt unavailable' end
          return true, 'native'
        end
        if type(gd.hit) ~= 'function' then return false, 'gd.hit unavailable' end
        if not state.hit_contract then return false, 'native hit acceptance contract not confirmed' end
        return true, 'native'
      elseif source.kind == 'enemy' then
        if target and target.kind == 'player' then
          if type(gd.enemy_strike) ~= 'function' then return false, 'gd.enemy_strike unavailable' end
          if not state.hit_contract then return false, 'native strike acceptance contract not confirmed' end
          return true, 'native'
        end
        return false, 'no native strike between custom actors'
      end
      return false, 'unknown source actor kind'
    end
    if seam == 'apply_movement' then
      -- gd.impulse is a fighter-only locomotion velocity; a custom actor
      -- direction is impossible and must refuse before any spend.
      if type(source) ~= 'table' or source.kind ~= 'player' then
        return false, 'gd.impulse requires a fighter port'
      end
      if type(gd.impulse) ~= 'function' then return false, 'gd.impulse unavailable' end
      return true, 'native'
    end
    if seam == 'apply_status' then
      if type(state.status) ~= 'function' then return false, 'no native status provider' end
      return true, 'provider'
    end
    if seam == 'apply_guard' then
      local p = state.guard
      if not (p and type(p.apply) == 'function' and type(p.release) == 'function') then
        return false, 'no native guard provider'
      end
      return true, 'provider'
    end
    if seam == 'apply_conversion' then
      local p = state.conversion
      if not (p and type(p.apply) == 'function' and type(p.release) == 'function') then
        return false, 'no native conversion provider'
      end
      return true, 'provider'
    end
    return false, 'unsupported effect kind'
  end

  -- Player eligibility requires a complete, explicitly known set of native
  -- state values. Absence or "?" is unknown, and unknown eligibility fails
  -- closed rather than granting damage.
  local known_state = {normal = true, invincible = true, intangible = true}
  local function player_eligibility(native)
    local body, timed = native.body_state, native.timed_state
    local inv, intan = native.invincible, native.intangible
    local shield = native.shield_on
    local known = known_state[body] == true and known_state[timed] == true
      and type(inv) == 'number' and type(intan) == 'number'
      and type(shield) == 'boolean'
    local immune = body == 'invincible' or body == 'intangible'
      or timed == 'invincible' or timed == 'intangible'
      or (type(inv) == 'number' and inv > 0)
      or (type(intan) == 'number' and intan > 0)
    return (known and not immune), known
  end

  -- Read one bound actor from the native engine into a finite snapshot. A
  -- missing/retired actor, non-finite position, zero-stock fighter or unknown
  -- eligibility fails closed with nil or an ineligible view.
  local function observe(host)
    local b = state.binds[host]
    if not b then return nil end
    local native
    if b.kind == 'player' then
      local ok, p = pcall(state.gd.player, b.port)
      if not ok or type(p) ~= 'table' then return nil end
      native = p
    else
      local ok, q = pcall(state.gd.enemy_state, b.handle)
      if not ok or type(q) ~= 'table' or q.alive == false then return nil end
      native = q
    end
    local x, y = native.x, native.y
    if not (finite(x) and finite(y)) then return nil end
    local view = {
      host = host, gen = b.gen, ref = b.ref, kind = b.kind, team = b.team, room = b.room,
      x = x, y = y, vx = native.vx or 0, vy = native.vy or 0,
      facing = (native.facing == -1) and -1 or 1,
      alive = true, vulnerable = true, shielding = false, reflecting = false,
      action = native.action, hitlag = native.hitlag or 0,
      in_hitlag = native.in_hitlag, in_hitstun = native.in_hitstun,
      airborne = native.airborne == true, stocks = native.stocks,
    }
    if b.kind == 'player' then
      view.shielding = native.shield_on == true
      view.body_state = native.body_state
      view.timed_state = native.timed_state
      view.invincible = native.invincible
      view.intangible = native.intangible
      view.stocks = native.stocks or 0
      if view.stocks <= 0 then return nil end
      local vuln, known = player_eligibility(native)
      if state.invulnerable then
        -- The provider contract is strict boolean: true = invulnerable,
        -- false = explicitly not invulnerable, nil/throw/other = unknown.
        local ok, ans = pcall(state.invulnerable, b, native)
        if not ok or type(ans) ~= 'boolean' then
          known = false
        elseif ans == true then
          vuln = false
        end
      end
      view.vulnerable = known and vuln
    else
      -- An enemy without an explicit boolean vulnerability is unknown.
      view.vulnerable = native.vulnerable == true
      view.damage = native.damage
    end
    if state.reflecting then
      local ok, ref = pcall(state.reflecting, b, native)
      if not ok or ref == true then view.reflecting = true end
    end
    if not view.vulnerable then view.alive = false end
    return view
  end

  local function binding_ref(spec)
    if spec.kind == 'player' then return 'p:' .. tostring(spec.port) end
    return 'h:' .. tostring(spec.handle)
  end

  local function binding_id(x)
    return {gen = x.gen, ref = x.ref, kind = x.kind, port = x.port, handle = x.handle,
      team = x.team, room = x.room, token = x.token}
  end
  local function same_binding(a, b)
    return type(a) == 'table' and type(b) == 'table' and a.gen == b.gen and a.ref == b.ref
      and a.kind == b.kind and a.port == b.port and a.handle == b.handle
      and a.team == b.team and a.room == b.room and a.token == b.token
  end

  local function is_free(view, binding, run, slot)
    if not view then return false, 'actor unavailable' end
    if binding.kind == 'player' then
      if view.in_hitlag == true or (view.hitlag or 0) > 0 then return false, 'hitlag' end
      if view.in_hitstun == true then return false, 'hitstun' end
      local action = view.action or 0
      if action < 14 or action > 34 or action == 24 then return false, 'not in a free state' end
      local ability = state.Core.ability(run, binding.host, slot)
      if ability and ability.action == 'step' and view.airborne then
        return false, 'grounded step while airborne'
      end
      return true
    end
    if not view.vulnerable then return false, 'actor not invulnerable-free' end
    return true
  end

  world.get_run = function()
    local ok, run = pcall(state.get_run_provider)
    if not ok or not active_run(run) then return nil end
    state.run = run
    return run
  end

  function world.bind(host, spec)
    if not name(host) then return nil, 'invalid host' end
    if type(spec) ~= 'table' then return nil, 'invalid binding' end
    if spec.kind ~= 'player' and spec.kind ~= 'enemy' then return nil, 'invalid actor kind' end
    if spec.kind == 'player' then
      if not (integer(spec.port) and spec.port >= 1 and spec.port <= 6) then return nil, 'invalid fighter port' end
    else
      if not (integer(spec.handle) and spec.handle > 0) then return nil, 'invalid enemy handle' end
    end
    if not name(spec.team) then return nil, 'team required' end
    if spec.room == nil then return nil, 'room required' end
    local existing = state.binds[host]
    if existing and existing.kind == spec.kind and existing.port == spec.port
      and existing.handle == spec.handle and existing.team == spec.team and existing.room == spec.room
      and existing.token == spec.token then
      return existing.gen
    end
    if not existing and count(state.binds) >= state.max_bindings then return nil, 'binding capacity' end
    state.bind_serial = state.bind_serial + 1
    state.binds[host] = {
      host = host, gen = state.bind_serial, kind = spec.kind, port = spec.port, handle = spec.handle,
      team = spec.team, room = spec.room, token = spec.token, ref = binding_ref(spec),
    }
    return state.binds[host].gen
  end

  function world.unbind(host)
    if not state.binds[host] then return false end
    state.binds[host] = nil
    return true
  end

  function world.bindings()
    local hosts = {}
    for h in pairs(state.binds) do hosts[#hosts + 1] = h end
    table.sort(hosts)
    local out = {}
    for _, h in ipairs(hosts) do out[#out + 1] = copy(state.binds[h]) end
    return out
  end

  function world.contributions()
    local keys = {}
    for k in pairs(state.contributions) do keys[#keys + 1] = k end
    table.sort(keys)
    local out = {}
    for _, k in ipairs(keys) do out[#out + 1] = copy(state.contributions[k]) end
    return out
  end

  world.observe = function(host) return observe(host) end

  -- Side-effect-free free-state gate. It must not record or overwrite any
  -- selected-identity expectation; identity is captured only by capture_intent.
  world.can_start = function(run, host, slot)
    if not active_run(run) then return false, 'run inactive' end
    if run ~= state.run then return false, 'run reference drift' end
    local b = state.binds[host]
    if not b then return false, 'actor unbound' end
    if not (type(run.hosts) == 'table' and run.hosts[host]) then return false, 'host not in run' end
    local view = observe(host)
    return is_free(view, b, run, slot)
  end

  -- Deterministic nearest eligible opponent. Candidate hosts are sorted before
  -- scoring so equal distances resolve by host name, never by table order.
  world.query = function(host, spec, ability)
    local src = state.binds[host]
    if not src then return nil end
    local src_view = observe(host)
    if not src_view then return nil end
    if src_view.in_hitlag == true or src_view.in_hitstun == true or (src_view.hitlag or 0) > 0 then return nil end
    if type(spec) ~= 'table' or type(spec.effect) ~= 'table' then return nil end
    local reach = ability and ability.reach
    if not finite(reach) or reach <= 0 then reach = spec.range end
    if not finite(reach) or reach <= 0 then return nil end
    local band = spec.height
    if not finite(band) or band <= 0 then band = 12 end
    local current = current_room()
    local hosts = {}
    for h in pairs(state.binds) do hosts[#hosts + 1] = h end
    table.sort(hosts)
    local candidates = {}
    for _, h in ipairs(hosts) do
      local b = state.binds[h]
      if h ~= host and b.team ~= src.team and same_room(b, src, current) then
        local tv = observe(h)
        if tv and tv.vulnerable and not (b.kind == 'player' and (tv.hitlag or 0) > 0) then
          local dx, dy = tv.x - src_view.x, tv.y - src_view.y
          if math.abs(dx) <= reach and math.abs(dy) <= band then
            if spec.arc ~= 'facing' or dx * (src_view.facing or 1) >= 0 then
              tv.distance = dx * dx + dy * dy
              candidates[#candidates + 1] = tv
            end
          end
        end
      end
    end
    if #candidates == 0 then return nil end
    table.sort(candidates, function(a, b)
      if a.distance ~= b.distance then return a.distance < b.distance end
      return a.host < b.host
    end)
    return candidates[1]
  end

  -- Fail closed: without an explicit trustworthy provider every line is
  -- treated as occluded, so an unknown LOS cannot grant a through-wall hit.
  world.occluded = function(from_host, target_view)
    local b = state.binds[from_host]
    if not b then return true end
    if type(target_view) ~= 'table' or not name(target_view.host) then return true end
    local tb = state.binds[target_view.host]
    if not tb then return true end
    if target_view.gen ~= nil and target_view.gen ~= tb.gen then return true end
    if type(state.visibility) ~= 'function' then return true end
    local src_view = observe(from_host)
    if not src_view then return true end
    local ok, visible = pcall(state.visibility, src_view, target_view)
    if not ok then return true end
    return visible ~= true
  end

  -- Freeze the immutable native identity at begin (after target selection).
  -- Refuses with nil when the source/target is unbound or the confirmed native
  -- capability for this effect/direction is missing, so no spend can follow.
  function world.capture_intent(run, host, slot, target, spec)
    if not active_run(run) then return nil, 'run inactive' end
    if type(spec) ~= 'table' or type(spec.effect) ~= 'table' then return nil, 'invalid spec' end
    local b = state.binds[host]
    if not b then return nil, 'source actor unbound' end
    if not (type(run.hosts) == 'table' and run.hosts[host]) then return nil, 'host not in run' end
    local tb
    if target ~= nil then
      if type(target) ~= 'table' or not name(target.host) then return nil, 'invalid target' end
      tb = state.binds[target.host]
      if not tb then return nil, 'target actor unbound' end
    end
    local cap, capwhy = native_capability(spec, b, tb)
    if not cap then return nil, capwhy end
    -- An exhausted adapter refuses here, before Core.activate, so the caller
    -- never spends only to interrupt and refund immediately.
    if not contribution_capacity_ok() then return nil, 'contribution capacity' end
    return {
      run = run, host = host, slot = slot, room = current_room(),
      spec = {effect = {kind = spec.effect.kind}, native = spec.native, targeting = spec.targeting},
      source = binding_id(b), target_host = target and target.host or nil,
      target = tb and binding_id(tb) or nil,
    }
  end

  -- Exact-true validation before the spend point. Any identity, room,
  -- capability or target-eligibility change cancels the action.
  function world.validate_intent(run, token)
    if type(token) ~= 'table' then return false, 'invalid intent token' end
    if not active_run(run) then return false, 'run inactive' end
    if token.run ~= run then return false, 'run changed' end
    local b = state.binds[token.host]
    if not b then return false, 'source actor unbound' end
    if not (type(run.hosts) == 'table' and run.hosts[token.host]) then return false, 'host not in run' end
    if not same_binding(b, token.source) then return false, 'source identity changed' end
    if token.room ~= current_room() then return false, 'room changed' end
    local tb
    if token.target_host ~= nil then
      tb = state.binds[token.target_host]
      if not tb then return false, 'target actor unbound' end
      if not same_binding(tb, token.target) then return false, 'target identity changed' end
    elseif token.target ~= nil then
      return false, 'invalid intent target'
    end
    local cap, capwhy = native_capability(token.spec, b, tb)
    if not cap then return false, capwhy end
    -- Authoritative gate at the spend point: capacity can be consumed between
    -- capture and here.
    if not contribution_capacity_ok() then return false, 'contribution capacity' end
    if not observe(token.host) then return false, 'source gone' end
    if tb then
      local tv = observe(token.target_host)
      if not tv then return false, 'target gone' end
      if not tv.vulnerable then return false, 'target invulnerable' end
      if tv.shielding then return false, 'target shielded' end
    end
    return true
  end

  -- Re-check binding identity, run, team, room, target eligibility, occlusion
  -- and native bounds at the mutation, not only at the query.
  local function revalidate(run, req, expect_target)
    if not active_run(run) then return false, 'run inactive' end
    if run ~= state.run then return false, 'run reference drift' end
    if type(req) ~= 'table' or not name(req.host) then return false, 'invalid request' end
    local b = state.binds[req.host]
    if not b then return false, 'source actor unbound' end
    if not (type(run.hosts) == 'table' and run.hosts[req.host]) then return false, 'host not in run' end
    if req.intent ~= nil then
      local pok, ok, why = pcall(world.validate_intent, run, req.intent)
      if not pok or ok ~= true then return false, why or 'intent changed' end
    end
    if expect_target then
      if not name(req.target) then return false, 'no target' end
      local tb = state.binds[req.target]
      if not tb then return false, 'target actor unbound' end
      if tb.team == b.team then return false, 'same team' end
      if not same_room(tb, b, current_room()) then return false, 'wrong room' end
      local tv = observe(req.target)
      if not tv then return false, 'target gone' end
      if not tv.vulnerable then return false, 'target invulnerable' end
      if req.target_view and req.target_view.gen ~= nil and req.target_view.gen ~= tb.gen then
        return false, 'target view drift'
      end
      if req.spec and req.spec.occlusion == 'line' and world.occluded(req.host, tv) then
        return false, 'target occluded'
      end
    end
    return true
  end

  local function core_hit_fields(action, spec)
    local dmg = math.floor(tonumber(action.damage) or -1)
    local kb = type(action.knockback) == 'table' and action.knockback or {}
    local angle = math.floor(tonumber(kb.angle) or -1)
    local kbg = math.floor(tonumber(kb.kbg) or -1)
    local bkb = math.floor(tonumber(kb.bkb) or -1)
    local reach = tonumber(action.reach)
    if reach == nil and type(spec) == 'table' then reach = tonumber(spec.range) end
    if not finite(reach) then reach = nil end
    return dmg, angle, kbg, bkb, reach
  end

  local B = GeneWorld.bounds
  local function in_hit_bounds(dmg, angle, kbg, bkb)
    return integer(dmg) and dmg >= B.hit.damage[1] and dmg <= B.hit.damage[2]
      and integer(angle) and angle >= B.hit.angle[1] and angle <= B.hit.angle[2]
      and integer(kbg) and kbg >= B.hit.kbg[1] and kbg <= B.hit.kbg[2]
      and integer(bkb) and bkb >= B.hit.bkb[1] and bkb <= B.hit.bkb[2]
  end
  local function in_enemy_bounds(dmg, angle, kbg, bkb, reach)
    return integer(dmg) and dmg >= B.enemy.damage[1] and dmg <= B.enemy.damage[2]
      and integer(angle) and angle >= B.enemy.angle[1] and angle <= B.enemy.angle[2]
      and integer(kbg) and kbg >= B.enemy.kbg[1] and kbg <= B.enemy.kbg[2]
      and integer(bkb) and bkb >= B.enemy.bkb[1] and bkb <= B.enemy.bkb[2]
      and finite(reach) and reach >= B.enemy.reach[1] and reach <= B.enemy.reach[2]
  end

  -- One bounded native contact. The return is the native call's own
  -- acceptance (with the opt-in contract, including armor absorption), never
  -- fabricated hit proof or a collision callback.
  world.apply_effect = function(run, req)
    local ok, why = revalidate(run, req, true)
    if not ok then return false, why end
    local src, tb = state.binds[req.host], state.binds[req.target]
    local cap, capwhy = native_capability(req.spec, src, tb)
    if not cap then return false, capwhy end
    local action = req.action
    if type(action) ~= 'table' then return false, 'missing resolved core action' end
    local dmg, angle, kbg, bkb, reach = core_hit_fields(action, req.spec)
    if not in_hit_bounds(dmg, angle, kbg, bkb) then return false, 'hit fields outside native bounds' end
    local meta = {seam = 'apply_effect', source = req.host, target = req.target,
      source_gen = src.gen, target_gen = tb.gen, damage = dmg, angle = angle, kbg = kbg, bkb = bkb}
    if src.kind == 'player' and tb.kind == 'player' then
      if not state.hit_contract then return false, 'native hit acceptance contract not confirmed' end
      local ok2, res = pcall(state.gd.hit, tb.port, {damage = dmg, angle = angle, kbg = kbg, bkb = bkb, from = src.port})
      if not ok2 then return false, 'native hit error' end
      if res ~= true then return false, 'native hit refused' end
      meta.contact = 'fighter'
      return true, meta
    elseif src.kind == 'player' and tb.kind == 'enemy' then
      if not in_enemy_bounds(dmg, angle, kbg, bkb, reach) then return false, 'enemy hit fields outside native bounds' end
      local ok2, res = pcall(state.gd.enemy_hurt, tb.handle,
        {from = src.port, damage = dmg, angle = angle, kbg = kbg, bkb = bkb, reach = reach})
      if not ok2 then return false, 'native enemy_hurt error' end
      if res ~= true then return false, 'native enemy_hurt refused' end
      meta.contact = 'owned_enemy'
      meta.reach = reach
      return true, meta
    elseif src.kind == 'enemy' and tb.kind == 'player' then
      if not state.hit_contract then return false, 'native strike acceptance contract not confirmed' end
      if not in_enemy_bounds(dmg, angle, kbg, bkb, reach) then return false, 'enemy strike fields outside native bounds' end
      local ok2, res = pcall(state.gd.enemy_strike, src.handle, tb.port,
        {damage = dmg, angle = angle, kbg = kbg, bkb = bkb, reach = reach})
      if not ok2 then return false, 'native enemy_strike error' end
      if res ~= true then return false, 'native enemy_strike refused' end
      meta.contact = 'owned_enemy_strike'
      meta.reach = reach
      return true, meta
    end
    return false, 'no native strike between custom actors'
  end

  -- gd.impulse is a fighter-only locomotion velocity; it is never used as a
  -- teleport or flight substitute and the native state refusals pass through.
  world.apply_movement = function(run, req)
    local ok, why = revalidate(run, req, false)
    if not ok then return false, why end
    local src = state.binds[req.host]
    local cap, capwhy = native_capability(req.spec, src, nil)
    if not cap then return false, capwhy end
    if src.kind ~= 'player' then return false, 'impulse requires a fighter port' end
    local ok2, p = pcall(state.gd.player, src.port)
    if not ok2 or type(p) ~= 'table' then return false, 'fighter unavailable' end
    local eff = (type(req.spec) == 'table' and type(req.spec.effect) == 'table') and req.spec.effect or {}
    local kind = eff.kind
    local facing = (p.facing == -1) and -1 or 1
    local dist = tonumber(eff.distance)
    if not finite(dist) or dist < 0 then dist = 0 end
    local x, y = 0, 0
    if kind == 'dash' then
      if eff.grounded and p.airborne then return false, 'grounded dash while airborne' end
      x = facing * dist
    elseif kind == 'hover' then
      x = facing * math.min(dist, 2.4)
      if p.airborne then y = 1.4 end
    else
      return false, 'unsupported movement kind'
    end
    if not (finite(x) and finite(y)) then return false, 'movement not finite' end
    if math.abs(x) > B.impulse.x or math.abs(y) > B.impulse.y then return false, 'movement outside native impulse bounds' end
    local ok3, res = pcall(state.gd.impulse, src.port, {x = x, y = y})
    if not ok3 then return false, 'native impulse error' end
    if res ~= true then return false, 'native impulse refused' end
    return true, {seam = 'apply_movement', source = req.host, source_gen = src.gen,
      kind = kind, x = x, y = y}
  end

  -- The logical Rime mark is applied by Core and skipped by the transaction
  -- before this seam. Prototype roots/statuses have no native implementation.
  world.apply_status = function(run, req)
    local ok, why = revalidate(run, req, req.spec and req.spec.targeting == 'opponent')
    if not ok then return false, why end
    local src = state.binds[req.host]
    local tb = req.target and state.binds[req.target] or nil
    local cap, capwhy = native_capability(req.spec, src, tb)
    if not cap then return false, capwhy end
    local ok2, res, detail = pcall(state.status, run, req)
    if not ok2 then return false, 'native status error' end
    if res == true then return true, (type(detail) == 'table' and detail or {seam = 'apply_status', source = req.host}) end
    if type(res) == 'table' and res.ok == true then
      res.seam = res.seam or 'apply_status'
      return true, res
    end
    return false, type(res) == 'string' and res or 'native status refused'
  end

  -- Release one adapter-owned token through the provider against the run the
  -- token was minted in (never a later run). Refusal retains the record.
  local function release_contribution(provider, kind, run, contrib)
    if type(contrib) ~= 'table' then return false, 'invalid contribution' end
    local owned_key, owned = nil, nil
    for k, v in pairs(state.contributions) do
      if v == contrib or (contrib.id ~= nil and v.id == contrib.id) then owned_key, owned = k, v end
    end
    if not owned then return false, 'contribution not owned' end
    if not (provider and type(provider.release) == 'function') then
      return false, 'no native ' .. kind .. ' release seam'
    end
    local ok, res = pcall(provider.release, owned.run or run, owned.native)
    if not ok or res ~= true then return false, 'native ' .. kind .. ' release refused' end
    state.contributions[owned_key] = nil
    return true
  end

  -- The adapter owns the handle. Acceptance requires the provider's exact
  -- `true`; a table `{ok=true,contrib=...}` is not acceptance. Any identifiable
  -- token is retained for cleanup regardless of acceptance, cleaned up now if
  -- possible, and kept for retry/reset if the release refuses. Cleanup always
  -- uses the record's owning run.
  local function apply_contribution(kind, provider, run, req)
    local ok, why = revalidate(run, req, req.spec and req.spec.targeting == 'opponent')
    if not ok then return false, why end
    local src = state.binds[req.host]
    local tb = req.target and state.binds[req.target] or nil
    local cap, capwhy = native_capability(req.spec, src, tb)
    if not cap then return false, capwhy end
    if not (provider and type(provider.apply) == 'function') then
      return false, 'no native ' .. kind .. ' contribution seam'
    end
    -- Refuse before the provider is touched when the adapter is already full.
    local reserved, reswhy = reserve_contribution()
    if not reserved then return false, reswhy end
    local ok2, res, detail = pcall(provider.apply, run, req)
    if not ok2 then drop_reservation(); return false, 'native ' .. kind .. ' error' end
    local token
    if type(detail) == 'table' and detail.contrib ~= nil then token = detail.contrib end
    if token == nil and type(res) == 'table' and res.contrib ~= nil then token = res.contrib end
    local accepted = (res == true)
    if token ~= nil then
      state.contrib_serial = state.contrib_serial + 1
      local record = {
        kind = kind, move_id = req.move_id, host = req.host, slot = req.slot,
        id = kind .. ':' .. tostring(state.contrib_serial), native = token, run = run,
      }
      -- Own the token first so cleanup is never lost. This record takes over the
      -- reservation, so the capacity it held stays consumed until the token's
      -- release is actually confirmed.
      state.contributions[record.id] = record
      drop_reservation()
      if accepted then
        return true, {seam = 'apply_' .. kind, contrib = record}
      end
      release_contribution(provider, kind, record.run, record) -- cleanup or retain
      return false, 'native ' .. kind .. ' refused'
    end
    -- No identifiable token: nothing is owned, so the reservation is freed.
    drop_reservation()
    if not accepted then return false, 'native ' .. kind .. ' refused' end
    -- Exact true with no identifiable token cannot be owned or cleaned up.
    return false, 'native ' .. kind .. ' token missing'
  end

  world.apply_guard = function(run, req) return apply_contribution('guard', state.guard, run, req) end
  world.release_guard = function(run, contrib) return release_contribution(state.guard, 'guard', run, contrib) end
  world.apply_conversion = function(run, req) return apply_contribution('conversion', state.conversion, run, req) end
  world.release_conversion = function(run, contrib) return release_contribution(state.conversion, 'conversion', run, contrib) end

  world.release = function(run, contrib)
    if type(contrib) ~= 'table' then return false, 'invalid contribution' end
    if contrib.kind == 'guard' then return world.release_guard(run, contrib) end
    if contrib.kind == 'conversion' then return world.release_conversion(run, contrib) end
    return false, 'unknown contribution kind'
  end

  -- Reset releases every owned contribution first. A refused release is
  -- retained and reported; a native token is never forgotten.
  function world.reset(run)
    run = run or state.run
    local ids = {}
    for id in pairs(state.contributions) do ids[#ids + 1] = id end
    table.sort(ids)
    local refused = {}
    for _, id in ipairs(ids) do
      local rec = state.contributions[id]
      if rec and not release_contribution(provider_for(rec.kind), rec.kind, run, rec) then
        refused[#refused + 1] = id
      end
    end
    state.binds = {}
    if #refused > 0 then return false, {refused = refused, reason = 'contribution release refused'} end
    return true
  end

  function world.capabilities()
    local gd = state.gd
    return {
      schema = 2,
      contact = {
        fighter_hit = type(gd.hit) == 'function',
        enemy_hurt = type(gd.enemy_hurt) == 'function',
        enemy_strike = type(gd.enemy_strike) == 'function',
        impulse = type(gd.impulse) == 'function',
      },
      providers = {
        visibility = state.visibility ~= nil,
        status = state.status ~= nil,
        guard = state.guard ~= nil,
        conversion = state.conversion ~= nil,
        invulnerable = state.invulnerable ~= nil,
        reflecting = state.reflecting ~= nil,
      },
      hit_contract = state.hit_contract,
      occluded_fail_closed = state.visibility == nil,
      eligibility = world.eligibility(),
    }
  end

  -- Exact per (gene, slot, direction) capability table for the offered genes.
  function world.eligibility()
    local genes = {}
    for gene in pairs(GeneWorld.offered) do genes[#genes + 1] = gene end
    table.sort(genes)
    local out = {}
    for _, gene in ipairs(genes) do
      local slots = {}
      local names = {}
      for slot in pairs(GeneWorld.offered[gene]) do names[#names + 1] = slot end
      table.sort(names)
      for _, slot in ipairs(names) do
        local p = GeneWorld.offered[gene][slot]
        local spec = {effect = {kind = p.kind}, native = p.native, targeting = p.targeting}
        local rows = {}
        local dirs = p.targeting == 'opponent' and {'fighter', 'enemy'} or {'self'}
        for _, d in ipairs(dirs) do
          local target = d == 'self' and nil or {kind = d}
          local ok, how = native_capability(spec, {kind = 'player'}, target)
          rows[#rows + 1] = {direction = d, eligible = ok == true, how = how}
        end
        slots[slot] = rows
      end
      out[gene] = slots
    end
    return out
  end

  function world.supports(gene, slot, direction)
    local p = GeneWorld.offered[gene] and GeneWorld.offered[gene][slot]
    if not p then return false, 'not offered' end
    local spec = {effect = {kind = p.kind}, native = p.native, targeting = p.targeting}
    local target = (direction and direction ~= 'self') and {kind = direction} or nil
    return native_capability(spec, {kind = 'player'}, target)
  end

  function world.diagnostics()
    return {
      schema = 2,
      run = state.run and state.run.id or nil,
      run_frame = state.run and state.run.frame or nil,
      bindings = count(state.binds),
      binding_capacity = state.max_bindings,
      contributions = count(state.contributions),
      contributions_reserved = state.reserved,
      contribution_capacity = state.max_contributions,
      capabilities = world.capabilities(),
    }
  end

  return world
end

return GeneWorld
