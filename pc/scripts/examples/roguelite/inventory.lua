-- Gate 7 pure inventory/economy service.
--
-- A bounded, versioned record of run-owned consumables plus a bounded permanent
-- currency/breeding economy and preview adapters over the real Core profile
-- rules. It is deliberately independent of Core's run table and Progress's
-- legacy `supplies` integer: nothing here reads or writes those, so integrating
-- it cannot mutate a saved route or checkpoint. Serialization goes through the
-- injected sandbox-safe Codec; never load()/loadstring().
--
-- Transaction contract shared with equipment.lua/run_history.lua:
--   preview -> independent stage -> validate -> reversible native effects ->
--   atomic persist -> commit.
-- Every mutating operation returns a STAGED copy and leaves its input untouched.
-- The caller validates the staged record, applies the deferred effect intents
-- (native heal/charge, Core modifier, Progress key/supply), persists atomically,
-- and only then swaps the staged record in. Any failure simply discards the
-- staged record: rollback is "do not commit". An effect whose mechanic does not
-- exist is refused BEFORE any count or cooldown is spent.
--
-- Definitions declare an explicit `implemented` flag. Unimplemented effects are
-- never offered as working; classify()/plan_use() explain the refusal instead.
local Inventory = {version = 1}

Inventory.max_capacity = 64
Inventory.max_id = 64
Inventory.max_stack = 99
Inventory.max_frame = 1000000000
Inventory.max_currency = 1000000000
Inventory.scopes = {run = true, profile = true}
Inventory.contexts = {combat = true, rest = true, field = true, reward = true, menu = true}

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end
local function integer(x, lo, hi) return finite(x) and x % 1 == 0 and x >= lo and x <= hi end
local function valid_id(x) return type(x) == 'string' and #x > 0 and #x <= Inventory.max_id and not x:find('[%z\1-\31]') end

local function copy(t)
  if type(t) ~= 'table' then return t end
  local out = {}
  for k, v in pairs(t) do out[k] = copy(v) end
  return out
end

local function count(t) local n = 0 for _ in pairs(t) do n = n + 1 end return n end
local function contains(list, value)
  for _, v in ipairs(list or {}) do if v == value then return true end end
  return false
end

local function placement(run, id)
  for host, h in pairs(run.hosts or {}) do
    for slot, gene in pairs(h.slots or {}) do if gene == id then return host, slot end end
  end
end

local function same(a, b)
  if type(a) ~= type(b) then return false end
  if type(a) ~= 'table' then return a == b end
  for k, v in pairs(a) do if not same(v, b[k]) then return false end end
  for k in pairs(b) do if a[k] == nil then return false end end
  return true
end

-- Ownership domain: a run-scoped inventory must belong to the live run it is
-- used in; a profile-scoped inventory must belong to the supplied profile. This
-- refuses foreign heals/charges/supplies/modifiers before any spend.
local function domain(inv, ctx)
  if inv.scope == 'run' then
    local run = ctx.run
    if type(run) ~= 'table' or run.status ~= 'active' then return nil, 'run is not active' end
    if run.id ~= inv.owner then return nil, 'inventory belongs to another run' end
  elseif inv.scope == 'profile' then
    local profile = ctx.profile
    if type(profile) ~= 'table' or profile.type ~= 'profile' then return nil, 'profile is required' end
    if profile.id ~= inv.owner then return nil, 'inventory belongs to another profile' end
  end
  return true
end

-- Consumable catalogue. `effect.kind` names the deferred intent main applies;
-- `implemented` states whether that mechanic exists today. At least six are
-- supported (heal, charge, supply, modifier, key); the last three are honest
-- refusals for mechanics no module implements yet.
Inventory.definitions = {
  repair_kit = {id = 'repair_kit', version = 1, name = 'Repair Kit',
    contexts = {'combat', 'rest'}, cooldown = 600, stack_max = 3, implemented = true,
    effect = {kind = 'heal', amount = 30}},
  field_bandage = {id = 'field_bandage', version = 1, name = 'Field Bandage',
    contexts = {'combat', 'rest', 'field'}, cooldown = 240, stack_max = 5, implemented = true,
    effect = {kind = 'heal', amount = 15}},
  -- Bounded legacy "Restore" resource: an existing save's `progress.supplies`
  -- counter means a 30-point heal consumable, not a food-supply grant. It is a
  -- distinct definition so migration preserves the exact count and meaning and
  -- cannot double-source the supplies economy.
  legacy_restore = {id = 'legacy_restore', version = 1, name = 'Restore',
    contexts = {'combat', 'rest', 'field'}, cooldown = 0, stack_max = 9, implemented = true,
    effect = {kind = 'heal', amount = 30}},
  charge_flask = {id = 'charge_flask', version = 1, name = 'Charge Flask',
    contexts = {'combat', 'rest'}, cooldown = 300, stack_max = 3, implemented = true,
    effect = {kind = 'charge', amount = 3}},
  charge_cell = {id = 'charge_cell', version = 1, name = 'Charge Cell',
    contexts = {'combat', 'rest'}, cooldown = 180, stack_max = 4, implemented = true,
    effect = {kind = 'charge', amount = 1}},
  supply_crate = {id = 'supply_crate', version = 1, name = 'Supply Crate',
    contexts = {'reward', 'rest'}, cooldown = 0, stack_max = 9, implemented = true,
    effect = {kind = 'supply', amount = 1}},
  supply_pack = {id = 'supply_pack', version = 1, name = 'Supply Pack',
    contexts = {'reward'}, cooldown = 0, stack_max = 4, implemented = true,
    effect = {kind = 'supply', amount = 2}},
  ember_phial = {id = 'ember_phial', version = 1, name = 'Ember Phial',
    contexts = {'combat', 'rest'}, cooldown = 900, stack_max = 2, implemented = true,
    effect = {kind = 'modifier', stat = 'potency', add = 2, duration = 900}},
  rime_salve = {id = 'rime_salve', version = 1, name = 'Rime Salve',
    contexts = {'combat', 'rest'}, cooldown = 900, stack_max = 2, implemented = true,
    effect = {kind = 'modifier', stat = 'capacity', add = 1, duration = 600}},
  warp_shard = {id = 'warp_shard', version = 1, name = 'Warp Shard',
    contexts = {'field'}, cooldown = 0, stack_max = 1, implemented = true,
    effect = {kind = 'key', key = 'vault_charge'}},
  -- No status/cleanse system, no disengage system and no bounded blast system
  -- exist, so these are refused rather than sold as working items.
  frost_tonic = {id = 'frost_tonic', version = 1, name = 'Frost Tonic',
    contexts = {'rest'}, cooldown = 0, stack_max = 2, implemented = false,
    effect = {kind = 'cleanse'}},
  smoke_bomb = {id = 'smoke_bomb', version = 1, name = 'Smoke Bomb',
    contexts = {'combat'}, cooldown = 600, stack_max = 2, implemented = false,
    effect = {kind = 'flee'}},
  bomb_core = {id = 'bomb_core', version = 1, name = 'Bomb Core',
    contexts = {'combat'}, cooldown = 600, stack_max = 2, implemented = false,
    effect = {kind = 'blast', damage = 20}},
}

function Inventory.def(id) return Inventory.definitions[id] end

-- Structural classification. `supported` means the mechanic exists; it says
-- nothing about whether a particular use is contextually allowed.
function Inventory.classify(def)
  if type(def) ~= 'table' then return {supported = false, reason = 'invalid item definition'} end
  if not valid_id(def.id) then return {supported = false, reason = 'invalid item id'} end
  if def.version ~= 1 then return {supported = false, id = def.id, reason = 'unsupported item version'} end
  local out = {id = def.id, version = 1, name = def.name,
    kind = def.effect and def.effect.kind, supported = false}
  if not (integer(def.cooldown, 0, Inventory.max_frame) and integer(def.stack_max, 1, Inventory.max_stack)) then
    out.reason = 'invalid item bounds'; return out
  end
  if type(def.contexts) ~= 'table' or #def.contexts == 0 then out.reason = 'item has no contexts'; return out end
  for _, c in ipairs(def.contexts) do
    if not Inventory.contexts[c] then out.reason = 'unknown context ' .. tostring(c); return out end
  end
  if not (type(def.effect) == 'table' and type(def.effect.kind) == 'string') then
    out.reason = 'item has no effect'; return out
  end
  if def.implemented ~= true then
    out.reason = 'effect ' .. tostring(def.effect.kind) .. ' is not implemented'; return out
  end
  local kind = def.effect.kind
  if kind == 'heal' then
    if not integer(def.effect.amount, 1, 100) then out.reason = 'invalid heal amount'; return out end
  elseif kind == 'charge' then
    if not integer(def.effect.amount, 1, 12) then out.reason = 'invalid charge amount'; return out end
  elseif kind == 'supply' then
    if not integer(def.effect.amount, 1, 9) then out.reason = 'invalid supply amount'; return out end
  elseif kind == 'modifier' then
    if not (type(def.effect.stat) == 'string' and finite(def.effect.add) and def.effect.add ~= 0) then
      out.reason = 'invalid modifier'; return out
    end
    if not integer(def.effect.duration, 1, Inventory.max_frame) then out.reason = 'invalid modifier duration'; return out end
  elseif kind == 'key' then
    if not valid_id(def.effect.key) then out.reason = 'invalid key'; return out end
  else
    out.reason = 'unknown effect kind ' .. tostring(kind); return out
  end
  out.supported = true
  out.effect = copy(def.effect)
  out.contexts = copy(def.contexts)
  out.cooldown = def.cooldown
  out.stack_max = def.stack_max
  return out
end

-- Service constructor. Codec is needed for encode/decode; Core/Progress are
-- needed only for the effects that resolve against them (charge/modifier/supply).
function Inventory.new(deps)
  deps = deps or {}
  assert(deps.Codec == nil or type(deps.Codec.encode) == 'function', 'invalid Codec dependency')
  assert(deps.Core == nil or type(deps.Core.resolve) == 'function', 'invalid Core dependency')
  return setmetatable({Codec = deps.Codec, Core = deps.Core, Progress = deps.Progress}, {__index = Inventory})
end

function Inventory.create(opts)
  opts = opts or {}
  if not valid_id(opts.owner) then return nil, 'invalid inventory owner' end
  local scope = opts.scope or 'run'
  if not Inventory.scopes[scope] then return nil, 'invalid inventory scope' end
  if not integer(opts.capacity, 0, Inventory.max_capacity) then return nil, 'invalid inventory capacity' end
  return {version = 1, owner = opts.owner, scope = scope, capacity = opts.capacity, items = {}, ready_at = {}}
end

local FIELDS = {version = true, owner = true, scope = true, capacity = true, items = true, ready_at = true}
function Inventory.validate(inv)
  if type(inv) ~= 'table' or inv.version ~= 1 then return false, 'unsupported inventory version' end
  for field in pairs(inv) do if not FIELDS[field] then return false, 'unknown inventory field ' .. tostring(field) end end
  if not valid_id(inv.owner) or not Inventory.scopes[inv.scope] then return false, 'invalid inventory owner/scope' end
  if not integer(inv.capacity, 0, Inventory.max_capacity) then return false, 'invalid inventory capacity' end
  if type(inv.items) ~= 'table' or type(inv.ready_at) ~= 'table' then return false, 'invalid inventory tables' end
  local stacks = 0
  for id, c in pairs(inv.items) do
    stacks = stacks + 1
    local def = Inventory.definitions[id]
    if not def then return false, 'unknown inventory item ' .. tostring(id) end
    local cls = Inventory.classify(def)
    if not cls.supported then return false, 'unsupported inventory item ' .. tostring(id) end
    if not integer(c, 1, def.stack_max) then return false, 'invalid stack for ' .. tostring(id) end
  end
  if stacks > inv.capacity then return false, 'inventory capacity exceeded' end
  for id, frame in pairs(inv.ready_at) do
    if not inv.items[id] then return false, 'cooldown without a stack' end
    if not integer(frame, 0, Inventory.max_frame) then return false, 'invalid cooldown' end
  end
  return true
end

function Inventory.count(inv, id) return (inv.items and inv.items[id]) or 0 end

function Inventory.cooldown(inv, id, frame)
  if not inv.items[id] then return 0 end
  local ready = inv.ready_at and inv.ready_at[id]
  if not ready then return 0 end
  if not integer(frame, 0, Inventory.max_frame) then return nil, 'invalid frame' end
  return math.max(0, ready - frame)
end

function Inventory.available(inv, id, frame, context)
  local def = Inventory.definitions[id]
  if not def then return false, 'unknown item' end
  local cls = Inventory.classify(def)
  if not cls.supported then return false, cls.reason end
  if Inventory.count(inv, id) <= 0 then return false, 'not owned' end
  if context ~= nil and not contains(def.contexts, context) then return false, 'not usable in this context' end
  local remaining, why = Inventory.cooldown(inv, id, frame)
  if remaining == nil then return false, why end
  if remaining > 0 then return false, 'on cooldown' end
  return true
end

-- Staged acquisition. Refuses before touching the record so a full inventory
-- cannot silently drop a pickup.
function Inventory.acquire(inv, id, amount)
  local ok, why = Inventory.validate(inv)
  if not ok then return nil, why end
  local def = Inventory.definitions[id]
  if not def then return nil, 'unknown item' end
  local cls = Inventory.classify(def)
  if not cls.supported then return nil, cls.reason end
  amount = amount or 1
  if not integer(amount, 1, def.stack_max) then return nil, 'invalid acquire amount' end
  local have = inv.items[id] or 0
  local after = have + amount
  if after > def.stack_max then return nil, 'stack capacity' end
  if have == 0 and count(inv.items) >= inv.capacity then return nil, 'inventory capacity' end
  local out = copy(inv)
  out.items[id] = after
  return out
end

-- Resolve one use into a staged inventory plus deferred effect intents. Read
-- only: the source record is never mutated. `ctx` carries frame, context, the
-- live run/progress (for charge/modifier/supply) and the effect contract.
function Inventory:plan_use(inv, id, ctx)
  ctx = ctx or {}
  local ok, why = Inventory.validate(inv)
  if not ok then return nil, why end
  local def = Inventory.definitions[id]
  if not def then return nil, 'unknown item' end
  local cls = Inventory.classify(def)
  if not cls.supported then return nil, cls.reason end
  if (inv.items[id] or 0) <= 0 then return nil, 'not owned' end
  local frame = ctx.frame
  if not integer(frame, 0, Inventory.max_frame) then return nil, 'invalid frame' end
  local ready = inv.ready_at[id]
  if ready and frame < ready then return nil, 'on cooldown' end
  if ctx.context ~= nil and not contains(def.contexts, ctx.context) then return nil, 'not usable in this context' end
  -- Ownership domain: refuse a foreign scope before any spend.
  local dom, dom_why = domain(inv, ctx)
  if not dom then return nil, dom_why end

  local effects = {}
  local kind = def.effect.kind
  if kind == 'heal' then
    local contract = ctx.effects
    if type(contract) ~= 'table' or contract.version ~= 1 or contract.heal ~= true then
      return nil, 'heal requires a staged effect contract'
    end
    local cap = integer(contract.max_heal, 1, 100) and contract.max_heal or 100
    if def.effect.amount > cap then return nil, 'heal exceeds the effect contract bound' end
    effects[1] = {kind = 'heal', target = 'player', amount = def.effect.amount, contract_version = 1}
  elseif kind == 'charge' then
    local run = ctx.run
    if not (self.Core and type(self.Core.resolve) == 'function') then return nil, 'Core dependency required for charge' end
    local contract = ctx.effects
    if type(contract) ~= 'table' or contract.version ~= 1 or contract.charge ~= true then
      return nil, 'charge requires a staged effect contract'
    end
    if type(run) ~= 'table' or run.status ~= 'active' then return nil, 'run is not active' end
    local target = ctx.target
    if not (valid_id(target) and run.genes and run.genes[target]) then return nil, 'unknown charge target' end
    local host, slot = placement(run, target)
    if host ~= 'player' or not slot then return nil, 'charge target must be a player-equipped gene' end
    local stats = self.Core.resolve(run, 'player', slot)
    if not stats then return nil, 'charge target cannot be resolved' end
    local state = run.hosts.player.state[slot]
    -- Use the actual resolved float charge; never floor it, or a fractional
    -- state (2.5/3) would advertise and grant a free 0.5 charge.
    local current = (state and state.charge) or 0
    if not (finite(current) and current >= 0) then return nil, 'invalid charge state' end
    local room = stats.capacity - current
    if room <= 0 then return nil, 'charge is already full' end
    local granted = math.min(def.effect.amount, room)
    if granted <= 0 then return nil, 'charge is already full' end
    effects[1] = {kind = 'charge', host = 'player', slot = slot, gene = target, amount = granted,
      before = current, target = current + granted, capacity = stats.capacity, contract_version = 1}
  elseif kind == 'supply' then
    local max_supplies = (self.Progress and self.Progress.max_supplies) or 9
    if not integer(def.effect.amount, 1, max_supplies) then return nil, 'invalid supply amount' end
    if type(ctx.progress) ~= 'table' or ctx.progress.run_id ~= inv.owner then
      return nil, 'supply requires the matching progress record'
    end
    effects[1] = {kind = 'supply', amount = def.effect.amount, contract_version = 1}
  elseif kind == 'modifier' then
    local run = ctx.run
    if not (self.Core and type(self.Core.resolve) == 'function') then return nil, 'Core dependency required for modifier' end
    if type(run) ~= 'table' or run.status ~= 'active' then return nil, 'run is not active' end
    local target = ctx.target
    if not (valid_id(target) and run.genes and run.genes[target]) then return nil, 'unknown modifier target' end
    local host, slot = placement(run, target)
    if host ~= 'player' or not slot then return nil, 'modifier target must be a player-equipped gene' end
    if not (self.Core.definitions and self.Core.definitions[run.genes[target].kind]) then
      return nil, 'unknown gene kind'
    end
    effects[1] = {kind = 'modifier', host = 'player', slot = slot, gene = target,
      stat = def.effect.stat, add = def.effect.add, duration = def.effect.duration, contract_version = 1}
  elseif kind == 'key' then
    if type(ctx.progress) ~= 'table' or ctx.progress.run_id ~= inv.owner then
      return nil, 'key requires the matching progress record'
    end
    effects[1] = {kind = 'key', key = def.effect.key, contract_version = 1}
  else
    return nil, 'unknown effect kind'
  end

  local out = copy(inv)
  out.items[id] = out.items[id] - 1
  if out.items[id] <= 0 then out.items[id] = nil end
  out.ready_at[id] = def.cooldown > 0 and (frame + def.cooldown) or nil
  -- Bind the plan to the exact canonical source record, frame, context and
  -- effect semantics so a replayed, foreign or tampered plan cannot be spent.
  return {ok = true, item = id, scope = inv.scope, inventory = out, effects = effects,
    cost = {item = id, count = 1}, message = def.name, cooldown_until = out.ready_at[id],
    binding = {owner = inv.owner, scope = inv.scope, item = id, count = (inv.items[id] or 0),
      ready_at = inv.ready_at[id], frame = frame, context = ctx.context, record = copy(inv),
      effects = copy(effects)}}
end

-- Re-check a staged plan by RECONSTRUCTING the canonical action from the
-- authoritative current record and context, then requiring the caller's plan to
-- match it exactly. Nothing in the plan's own mutable fields (item, cost,
-- effects, staged inventory, binding) is trusted as authority: a forged amount,
-- an unstaged inventory, a swapped item/cost, an altered binding, a changed
-- charge state/capacity/target or a changed effect contract all re-derive a
-- different canonical plan and are refused before any effect or spend.
function Inventory:validate_plan(plan, ctx)
  ctx = ctx or {}
  if type(plan) ~= 'table' or plan.ok ~= true then return nil, 'invalid plan' end
  local current = ctx.inventory
  if type(current) ~= 'table' then return nil, 'current inventory required' end
  local ok, why = Inventory.validate(current)
  if not ok then return nil, why end
  if not valid_id(ctx.item) then return nil, 'canonical item required' end
  local canonical, calc_why = self:plan_use(current, ctx.item, {
    frame = ctx.frame, context = ctx.context, run = ctx.run, profile = ctx.profile,
    progress = ctx.progress, target = ctx.target, effects = ctx.effects})
  if not canonical then return nil, 'invalid or stale plan: ' .. tostring(calc_why) end
  if not same(plan, canonical) then return nil, 'plan does not match the canonical action' end
  return true
end

function Inventory:encode(inv)
  if not self.Codec then return nil, 'Codec dependency required' end
  local ok, why = Inventory.validate(inv)
  if not ok then return nil, why end
  return self.Codec.encode(copy(inv))
end

function Inventory:decode(text)
  if not self.Codec then return nil, 'Codec dependency required' end
  local value, why = self.Codec.decode(text)
  if not value then return nil, why end
  local ok, why2 = Inventory.validate(value)
  if not ok then return nil, why2 end
  return value
end

-- Legacy supplies migration boundary. A legacy save stores a plain integer
-- (`progress.supplies`) that means the old "Restore" heal consumable, NOT a food
-- supply grant. It maps to the bounded `legacy_restore` definition so the exact
-- count and meaning are preserved and the supplies economy is not double-sourced.
-- The count is never clamped (that would lose data); an out-of-range value is
-- refused. It reads only and never writes back to a saved route/progress.
function Inventory.from_legacy_supplies(value, opts)
  opts = opts or {}
  if not integer(value, 0, Inventory.max_stack) then return nil, 'invalid legacy supply count' end
  local inv, why = Inventory.create({owner = opts.owner, scope = opts.scope or 'run',
    capacity = opts.capacity or 8})
  if not inv then return nil, why end
  if value > 0 then
    if value > Inventory.definitions.legacy_restore.stack_max then
      return nil, 'legacy supplies exceed the bounded restore stack'
    end
    inv.items.legacy_restore = value
  end
  return inv
end

-- Permanent bounded economy: currency with an explicit cap and enumerated
-- sources, plus deterministic bounded breeding costs. Currency can never exceed
-- the cap and breeding always costs, so no loop produces free power.
Inventory.Economy = {version = 1, cap_default = 100000, max_currency = Inventory.max_currency,
  sources = {run_success = true, run_failure = true, reward = true, sell = true, discard = true, refund = true}}

function Inventory.Economy.new(opts)
  opts = opts or {}
  local cap = opts.cap or Inventory.Economy.cap_default
  if not integer(cap, 1, Inventory.Economy.max_currency) then return nil, 'invalid currency cap' end
  return {version = 1, currency = 0, cap = cap, earned = 0, spent = 0}
end

function Inventory.Economy.validate(ledger)
  if type(ledger) ~= 'table' or ledger.version ~= 1 then return false, 'unsupported economy version' end
  for field in pairs(ledger) do
    if field ~= 'version' and field ~= 'currency' and field ~= 'cap' and field ~= 'earned' and field ~= 'spent' then
      return false, 'unknown economy field ' .. tostring(field)
    end
  end
  if not integer(ledger.cap, 1, Inventory.Economy.max_currency) then return false, 'invalid currency cap' end
  if not integer(ledger.currency, 0, ledger.cap) then return false, 'invalid currency balance' end
  if not integer(ledger.earned, 0, Inventory.Economy.max_currency)
    or not integer(ledger.spent, 0, Inventory.Economy.max_currency) then return false, 'invalid economy counters' end
  return true
end

function Inventory.Economy.earn(ledger, amount, source)
  local ok, why = Inventory.Economy.validate(ledger)
  if not ok then return nil, why end
  if not integer(amount, 1, Inventory.Economy.max_currency) then return nil, 'invalid currency amount' end
  if not Inventory.Economy.sources[source] then return nil, 'unknown currency source' end
  local out = copy(ledger)
  out.currency = math.min(out.cap, out.currency + amount)
  out.earned = math.min(Inventory.Economy.max_currency, out.earned + amount)
  return out
end

function Inventory.Economy.spend(ledger, amount, reason)
  local ok, why = Inventory.Economy.validate(ledger)
  if not ok then return nil, why end
  if not integer(amount, 1, Inventory.Economy.max_currency) then return nil, 'invalid currency amount' end
  if type(reason) ~= 'string' or #reason == 0 then return nil, 'missing spend reason' end
  if ledger.currency < amount then return nil, 'insufficient currency' end
  local out = copy(ledger)
  out.currency = out.currency - amount
  out.spent = math.min(Inventory.Economy.max_currency, out.spent + amount)
  return out
end

function Inventory.Economy.reward_value(outcome, difficulty)
  if outcome == 'success' then return 40 + (integer(difficulty, 1, 5) and difficulty or 1) * 10 end
  if outcome == 'failure' then return 10 end
  return nil, 'invalid outcome'
end

-- Deterministic, bounded breeding cost. It depends only on the two parents, so
-- a repeated preview/commit always quotes the same price; there is no free
-- reroll. The cost is clamped to a finite range and never zero.
function Inventory.Economy.breed_cost(Core, profile, a, b)
  if type(profile) ~= 'table' or profile.type ~= 'profile' then return nil, 'invalid profile' end
  local ga, gb = profile.genes[a], profile.genes[b]
  if not ga or not gb then return nil, 'unknown parent' end
  if ga.kind ~= gb.kind then return nil, 'incompatible parents' end
  local cost = 20 + math.floor((ga.base.potency + gb.base.potency) * 2)
  return math.max(20, math.min(500, cost))
end

-- Collection preview adapters over the real Core rules. Every preview runs the
-- real Core operation on a snapshot/restore clone, so it reports exact Core
-- behaviour without mutating the live profile or run.
Inventory.Collection = {version = 1, max_genes = 128}

function Inventory.Collection.parent_references(profile, id)
  local n = 0
  for gid, gene in pairs(profile.genes or {}) do
    if gid ~= id and type(gene.parents) == 'table' then
      for _, parent in pairs(gene.parents) do if parent == id then n = n + 1 end end
    end
  end
  return n
end

function Inventory.Collection.breed_preview(Core, profile, a, b)
  if type(Core) ~= 'table' or type(Core.breed) ~= 'function' then return nil, 'Core required' end
  if type(profile) ~= 'table' or profile.type ~= 'profile' then return nil, 'invalid profile' end
  if a == b or not profile.genes[a] or not profile.genes[b] then return nil, 'invalid parents' end
  if profile.genes[a].kind ~= profile.genes[b].kind then return nil, 'incompatible parents' end
  local cost = Inventory.Economy.breed_cost(Core, profile, a, b)
  if count(profile.genes) >= Inventory.Collection.max_genes then
    return {ok = false, requires_discard = true, reason = 'collection full', cost = cost}
  end
  local clone = Core.restore(assert(Core.snapshot(profile)))
  local id, why = Core.breed(clone, a, b)
  if not id then return nil, tostring(why) end
  return {ok = true, child = id, child_seed = clone.genes[id].seed, cost = cost, requires_discard = false}
end

-- Committing charges first, then calls the real Core.breed. A failed breeding
-- leaves the caller's ledger untouched because the spend was staged.
function Inventory.Collection.commit_breed(Core, profile, ledger, a, b, opts)
  opts = opts or {}
  if opts.confirm ~= true then return nil, 'breeding requires confirmation' end
  local quote, why = Inventory.Collection.breed_preview(Core, profile, a, b)
  if not quote then return nil, why end
  if quote.requires_discard then return nil, 'collection full; discard or replace first' end
  local paid = ledger
  if ledger then
    paid, why = Inventory.Economy.spend(ledger, quote.cost, 'breeding')
    if not paid then return nil, why end
  end
  local id, err = Core.breed(profile, a, b)
  if not id then return nil, tostring(err) end
  return {gene = id, cost = quote.cost, ledger = paid}
end

function Inventory.Collection.export_preview(Core, profile, run, id)
  if type(Core) ~= 'table' or type(Core.finish) ~= 'function' then return nil, 'Core required' end
  if type(profile) ~= 'table' or profile.type ~= 'profile' then return nil, 'invalid profile' end
  if type(run) ~= 'table' then return nil, 'invalid run' end
  local clone_profile = Core.restore(assert(Core.snapshot(profile)))
  local clone_run = Core.restore(assert(Core.snapshot(run)))
  local result, why = Core.finish(clone_profile, clone_run, 'success', id)
  if not result then return nil, tostring(why) end
  return {ok = true, exported = result.export, requires_confirmation = true}
end

function Inventory.Collection.fusion_preview(Core, run, a, b)
  if type(Core) ~= 'table' or type(Core.fuse) ~= 'function' then return nil, 'Core required' end
  if type(run) ~= 'table' or run.status ~= 'active' then return nil, 'run is not active' end
  if a == b or not run.genes[a] or not run.genes[b] then return nil, 'invalid fusion parents' end
  local clone = Core.restore(assert(Core.snapshot(run)))
  local id, why = Core.fuse(clone, a, b)
  if not id then return nil, tostring(why) end
  return {ok = true, child = id, removed = {a, b}}
end

function Inventory.Collection.discard_preview(profile, id)
  if type(profile) ~= 'table' or profile.type ~= 'profile' then return nil, 'invalid profile' end
  if not profile.genes[id] then return nil, 'unknown gene' end
  local refs = Inventory.Collection.parent_references(profile, id)
  if refs > 0 then return nil, 'gene is inherited by ' .. refs .. ' child gene(s)' end
  return {ok = true, gene = id, requires_confirmation = true}
end

function Inventory.Collection.commit_discard(profile, id, opts)
  opts = opts or {}
  if opts.confirm ~= true then return nil, 'discard requires confirmation' end
  local quote, why = Inventory.Collection.discard_preview(profile, id)
  if not quote then return nil, why end
  profile.genes[id] = nil
  return {discarded = id}
end

return Inventory
