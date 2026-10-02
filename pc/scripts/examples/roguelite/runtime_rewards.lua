-- Transaction-safe reward resolver for a resolved v2 node.reward_spec (an
-- EncounterCatalogue.rewards entry, not the fixed four-option slice in menus.lua).
--
-- A reward either modifies a run gene through Core.reward (stat upgrades and
-- all-or-nothing tradeoffs), grants bounded progress supplies, or stages a
-- deferred native effect (heal/charge). Definitions whose advertised mechanic
-- does not exist in Core (unimplemented mutations/reactions and equipment) are
-- refused explicitly; they are never surfaced as working. A tradeoff must pay
-- its ENTIRE advertised effective cost (measured on Core-resolved stats); a
-- partially clamped cost is refused rather than sold at a discount.
--
-- Everything is resolved on independent copies of the Core run and the schema-2
-- Progress record. Neither the caller's run/profile/progress nor any save is
-- touched. commit() returns the staged copies plus deferred effect intents for
-- main to persist and apply; a refusal returns nil,reason and preserves the
-- originals. preview() and commit() share the same resolver, so a preview can
-- never disagree with the committed stats.
--
-- Family bridge: reward families come from EncounterCatalogue.families
-- (any/fire/frost/cobalt). A gene is eligible only when Core.definitions[kind]
-- exists, GeneCatalogue.genes[kind] is marked implemented and both modules
-- agree on the family. The mapping is read from Core, never guessed: there is
-- no assumed cinder<->fire or rime<->frost equivalence beyond what Core states,
-- and `cobalt` currently names no implemented Core family.
local RuntimeRewards = {version = 1}

local STAT_ORDER = {'potency', 'capacity', 'gain', 'reach', 'cooldown'}
local STAT_LABELS = {potency = 'POWER', capacity = 'CAPACITY', gain = 'GAIN', reach = 'REACH', cooldown = 'RECOVERY'}
local SLOT_ORDER = {'assault', 'guard', 'traversal'}
local EPS = 1e-6

local function copy(t)
  if type(t) ~= 'table' then return t end
  local out = {}
  for k, v in pairs(t) do out[k] = copy(v) end
  return out
end

local function equal(a, b)
  if type(a) ~= type(b) then return false end
  if type(a) ~= 'table' then return a == b end
  for k, v in pairs(a) do if not equal(v, b[k]) then return false end end
  for k in pairs(b) do if a[k] == nil then return false end end
  return true
end

local function valid_id(x)
  return type(x) == 'string' and #x > 0 and #x <= 64 and not x:find('[%z\1-\31]')
end

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end
local function integer(x, lo, hi) return finite(x) and x % 1 == 0 and x >= lo and x <= hi end

local function sorted_keys(t)
  local list = {}
  for k in pairs(t or {}) do list[#list + 1] = k end
  table.sort(list)
  return list
end

local function fmt(n)
  if n == math.floor(n) then return tostring(n) end
  return (string.format('%.2f', n):gsub('0+$', ''):gsub('%.$', ''))
end

function RuntimeRewards.new(Core, deps)
  assert(type(Core) == 'table' and type(Core.definitions) == 'table' and type(Core.reward) == 'function'
    and type(Core.resolve) == 'function' and type(Core.snapshot) == 'function'
    and type(Core.restore) == 'function' and type(Core.equip) == 'function', 'Core module required')
  deps = deps or {}
  assert(type(deps.Progress) == 'table' and type(deps.Progress.validate) == 'function', 'Progress module required')
  assert(type(deps.EncounterCatalogue) == 'table' and type(deps.EncounterCatalogue.rewards) == 'table',
    'EncounterCatalogue required')
  local genes = deps.GeneCatalogue
  assert(genes == nil or (type(genes) == 'table' and type(genes.limits) == 'table' and type(genes.genes) == 'table'),
    'invalid GeneCatalogue')
  local self = setmetatable({
    Core = Core, progress_module = deps.Progress, encounters = deps.EncounterCatalogue, genes = genes,
    stat_names = genes and sorted_keys(genes.limits) or copy(STAT_ORDER),
    reward_kinds = deps.EncounterCatalogue.rewardKinds or
      {upgrade = true, tradeoff = true, mutation = true, consumable = true, equipment = true},
    reward_families = deps.EncounterCatalogue.families or {any = true, fire = true, frost = true, cobalt = true},
    max_supplies = deps.Progress.max_supplies or 9,
  }, {__index = RuntimeRewards})
  local stats = {}
  for _, stat in ipairs(self.stat_names) do stats[stat] = true end
  self.stat_set = stats
  self.kinds, self.by_family = self:_index_kinds()
  return self
end

-- Implemented gene kinds, keyed by kind, plus family -> kinds. A kind is only
-- implemented when Core has the definition and (if supplied) GeneCatalogue
-- marks it implemented and agrees on the family name.
function RuntimeRewards:_index_kinds()
  local kinds, by_family = {}, {}
  for _, kind in ipairs(sorted_keys(self.Core.definitions)) do
    local def = self.Core.definitions[kind]
    local gene = self.genes and self.genes.genes and self.genes.genes[kind] or nil
    local implemented = def.family ~= nil
    if gene then
      if gene.implemented == false then implemented = false end
      if gene.family ~= nil and gene.family ~= def.family then implemented = false end
    end
    if implemented then
      kinds[kind] = {kind = kind, family = def.family, version = def.version}
      by_family[def.family] = by_family[def.family] or {}
      by_family[def.family][#by_family[def.family] + 1] = kind
    end
  end
  for _, list in pairs(by_family) do table.sort(list) end
  return kinds, by_family
end

function RuntimeRewards:_family_kinds(family)
  if family == 'any' then return sorted_keys(self.kinds) end
  return self.by_family[family] or {}
end

function RuntimeRewards:_kind_family(kind)
  return self.kinds[kind] and self.kinds[kind].family or (self.Core.definitions[kind] and self.Core.definitions[kind].family)
end

function RuntimeRewards:_kind_eligible(kind, family)
  if not self.kinds[kind] then return false end
  return family == 'any' or self.kinds[kind].family == family
end

-- Public read-only views for generators/menus: the implemented family of a Core
-- kind and the implemented kinds of a reward family.
function RuntimeRewards:gene_family(kind) return self:_kind_family(kind) end
function RuntimeRewards:family_kinds(family) return self:_family_kinds(family) end

local function parse_ops(self, map, label, require_positive)
  if type(map) ~= 'table' then return nil, 'tradeoff is missing its ' .. label end
  local out, n = {}, 0
  for k, v in pairs(map) do
    if not self.stat_set[k] then return nil, 'unknown tradeoff stat ' .. tostring(k) end
    if not finite(v) or math.abs(v) > 600 or v == 0 then return nil, 'invalid tradeoff value for ' .. tostring(k) end
    if require_positive and v < 0 then return nil, 'tradeoff cost ' .. tostring(k) .. ' must be positive' end
    out[k] = v
    n = n + 1
  end
  if n == 0 then return nil, 'tradeoff ' .. label .. ' is empty' end
  return out
end

-- Structural classification of a reward definition. `supported` means the
-- mechanic exists in Core/Progress; it never means the definition is currently
-- applicable to a run (see the per-run resolver). `requires_effect` marks a
-- deferred native effect that needs an explicit staged gameplay contract.
function RuntimeRewards:classify(spec)
  if type(spec) ~= 'table' then return {supported = false, reason = 'invalid reward specification'} end
  if not valid_id(spec.id) then return {supported = false, reason = 'invalid reward id'} end
  if spec.version ~= 1 then return {supported = false, id = spec.id, reason = 'unsupported reward version'} end
  local out = {id = spec.id, version = spec.version, kind = spec.kind, family = spec.family or 'any', supported = false}
  if not self.reward_kinds[spec.kind] then out.reason = 'unknown reward kind'; return out end
  if not self.reward_families[out.family] then out.reason = 'unknown reward family'; return out end
  if #self:_family_kinds(out.family) == 0 then
    out.reason = 'no implemented gene family for ' .. out.family
    return out
  end
  if spec.kind == 'upgrade' then
    out.mechanic, out.requires_target = 'core_reward_stat', true
    if not self.stat_set[spec.stat] then
      out.reason = 'unknown stat ' .. tostring(spec.stat)
    elseif not finite(spec.delta) or math.abs(spec.delta) > 600 or spec.delta == 0 then
      out.reason = 'invalid upgrade delta'
    else
      out.supported, out.stat, out.delta = true, spec.stat, spec.delta
    end
  elseif spec.kind == 'tradeoff' then
    out.mechanic, out.requires_target = 'core_reward_tradeoff', true
    local adds, awhy = parse_ops(self, spec.add, 'gain', false)
    local removes, rwhy
    if adds then removes, rwhy = parse_ops(self, spec.remove, 'cost', true) end
    if not adds then
      out.reason = awhy
    elseif not removes then
      out.reason = rwhy
    else
      for stat in pairs(adds) do if removes[stat] then out.reason = 'tradeoff changes ' .. stat .. ' twice'; break end end
      if not out.reason then out.supported, out.adds, out.removes = true, adds, removes end
    end
  elseif spec.kind == 'mutation' then
    out.mechanic = 'none'
    out.reason = valid_id(spec.reaction) and ('mutation reaction ' .. spec.reaction .. ' is not implemented in Core')
      or 'invalid mutation reaction'
  elseif spec.kind == 'consumable' then
    local supplied = (spec.supply ~= nil) and 1 or 0
    supplied = supplied + ((spec.heal ~= nil) and 1 or 0) + ((spec.charge ~= nil) and 1 or 0)
    if supplied ~= 1 then
      out.reason = 'consumable needs exactly one effect'
    elseif spec.supply ~= nil then
      out.mechanic = 'progress_supply'
      if integer(spec.supply, 1, self.max_supplies) then out.supported, out.supply = true, spec.supply
      else out.reason = 'supply grant out of bounds' end
    elseif spec.heal ~= nil then
      out.mechanic, out.requires_effect = 'deferred_heal', 'heal'
      if integer(spec.heal, 1, 100) then out.supported, out.heal = true, spec.heal
      else out.reason = 'invalid heal amount' end
    else
      out.mechanic, out.requires_target, out.requires_effect = 'deferred_charge', true, 'charge'
      if integer(spec.charge, 1, 12) then out.supported, out.charge = true, spec.charge
      else out.reason = 'invalid charge amount' end
    end
  elseif spec.kind == 'equipment' then
    out.mechanic = 'none'
    out.reason = 'equipment definitions have no Core slot or ownership mechanic'
  end
  return out
end

-- Definitions a generator may admit, with the honest reason for the rest. This
-- is the supported-definition filter for generator admission; it does not
-- depend on a particular run.
function RuntimeRewards:supported_definitions(catalogue)
  catalogue = catalogue or self.encounters
  local out = {}
  for _, id in ipairs(sorted_keys(catalogue.rewards)) do
    local entry = self:classify(catalogue.rewards[id])
    entry.id = id
    out[#out + 1] = entry
  end
  return out
end

function RuntimeRewards:_placement(run, id)
  for host, h in pairs(run.hosts or {}) do
    for slot, gene in pairs(h.slots or {}) do if gene == id then return host, slot end end
  end
end

function RuntimeRewards:_enemy_owned(run, id)
  local host = self:_placement(run, id)
  return host ~= nil and host ~= 'player'
end

-- Player-available genes of an eligible family: unplaced genes and the player
-- host's genes. Enemy-owned genes never qualify.
function RuntimeRewards:_eligible_targets(run, spec)
  local cls = self:classify(spec)
  if not cls.supported then return {} end
  local out = {}
  for _, id in ipairs(sorted_keys(run.genes)) do
    if not self:_enemy_owned(run, id) and self:_kind_eligible(run.genes[id].kind, cls.family) then
      local host, slot = self:_placement(run, id)
      out[#out + 1] = {id = id, kind = run.genes[id].kind, family = self:_kind_family(run.genes[id].kind),
        host = host, slot = slot}
    end
  end
  return out
end

-- Charge can only land on a player-equipped gene; the placement is the target.
function RuntimeRewards:_effect_targets(run, spec)
  local cls = self:classify(spec)
  if not (cls.kind == 'consumable' and cls.charge) then return self:_eligible_targets(run, spec) end
  local out = {}
  local h = run.hosts and run.hosts.player
  if not h then return out end
  for _, slot in ipairs(SLOT_ORDER) do
    local id = h.slots and h.slots[slot]
    if id and run.genes[id] and self:_kind_eligible(run.genes[id].kind, cls.family) then
      out[#out + 1] = {id = id, kind = run.genes[id].kind, family = self:_kind_family(run.genes[id].kind),
        host = 'player', slot = slot}
    end
  end
  return out
end

-- Core-resolved stats for a gene, equipped in place or previewed unplaced.
function RuntimeRewards:_effective(run, id)
  local host, slot = self:_placement(run, id)
  if host then
    local stats = self.Core.resolve(run, host, slot)
    return stats and copy(stats)
  end
  local preview = self:_clone_run(run)
  if not preview or not self.Core.equip(preview, 'reward_preview', 'assault', id) then return nil end
  local stats = self.Core.resolve(preview, 'reward_preview', 'assault')
  return stats and copy(stats)
end

-- Public target enumeration: player-available genes for a stat reward, or
-- player-equipped genes for a charge reward.
function RuntimeRewards:targets(run, spec)
  if type(run) ~= 'table' or type(run.genes) ~= 'table' or type(spec) ~= 'table' then return {} end
  return self:_effect_targets(run, spec)
end

function RuntimeRewards:_clone_run(run)
  local text = self.Core.snapshot(run)
  if not text then return nil end
  return self.Core.restore(text)
end

function RuntimeRewards:_claim_header(run, progress, room_id)
  if not valid_id(room_id) then return nil, 'invalid room id' end
  if type(run) ~= 'table' or run.status ~= 'active' then return nil, 'run is not active' end
  local ok, why = self.progress_module.validate(progress)
  if not ok then return nil, 'invalid progress: ' .. tostring(why) end
  if progress.run_id ~= run.id then return nil, 'progress belongs to another run' end
  local claim_id = 'reward:' .. room_id
  if progress.claimed[claim_id] then return nil, 'reward already claimed' end
  if not progress.visited[room_id] then return nil, 'room not visited' end
  if progress.objectives[room_id] ~= 'done' then return nil, 'objective not fulfilled' end
  return claim_id
end

local function diff_stats(before, after, only)
  local list = {}
  for _, stat in ipairs(STAT_ORDER) do
    if (not only or only[stat]) and before[stat] ~= after[stat] then
      list[#list + 1] = {stat = stat, label = STAT_LABELS[stat], before = before[stat], after = after[stat]}
    end
  end
  return list
end

local function render(changes)
  local parts = {}
  for _, c in ipairs(changes) do parts[#parts + 1] = c.label .. ' ' .. fmt(c.before) .. ' > ' .. fmt(c.after) end
  return table.concat(parts, '; ')
end

function RuntimeRewards:_apply_stat(crun, target, cls)
  local before = self:_effective(crun, target)
  if not before then return nil, 'target gene cannot be resolved' end
  local ok, why
  if cls.kind == 'upgrade' then
    ok, why = self.Core.reward(crun, target, cls.stat, cls.delta)
  else
    for stat, delta in pairs(cls.adds) do
      ok, why = self.Core.reward(crun, target, stat, delta)
      if not ok then return nil, tostring(why) end
    end
    for stat, delta in pairs(cls.removes) do
      ok, why = self.Core.reward(crun, target, stat, -delta)
      if not ok then return nil, tostring(why) end
    end
    ok = true
  end
  if not ok then return nil, tostring(why) end
  local after = self:_effective(crun, target)
  if not after then return nil, 'target gene cannot be resolved after the change' end
  local changes = diff_stats(before, after)
  if cls.kind == 'upgrade' then
    if not changes[1] then return nil, 'no effect at the current cap' end
    return {before = before, after = after, changes = changes, benefit = changes, cost = {}}
  end
  -- All-or-nothing: every gain must move toward its add direction, and every
  -- cost must be paid IN FULL. A partially clamped cost (e.g. capacity 1.5 to 1
  -- against an advertised 1, or a modifier that clamps the effective gain) would
  -- sell the full benefit at a discount, so it voids the tradeoff. Measured on
  -- the Core-resolved effective stats, matching what the preview shows.
  for stat, delta in pairs(cls.adds) do
    local moved = delta > 0 and after[stat] > before[stat] or delta < 0 and after[stat] < before[stat]
    if not moved then return nil, 'tradeoff cannot apply its gain at the current cap' end
  end
  for stat, delta in pairs(cls.removes) do
    local paid = before[stat] - after[stat]
    if math.abs(paid - delta) > EPS then
      return nil, 'tradeoff cannot pay the full advertised cost for ' .. stat
    end
  end
  return {before = before, after = after, changes = changes,
    benefit = diff_stats(before, after, cls.adds), cost = diff_stats(before, after, cls.removes)}
end

-- The single resolver used by both preview() and commit(). It works only on
-- copies and returns nil,reason without side effects on any refusal.
function RuntimeRewards:_transaction(ctx)
  ctx = ctx or {}
  local spec = ctx.spec or (ctx.node and ctx.node.reward_spec)
  local room_id = ctx.room_id or (ctx.node and ctx.node.id)
  if type(spec) ~= 'table' then return nil, 'no reward specification' end
  local canonical = self.encounters.rewards[spec.id]
  if not canonical then return nil, 'unknown reward ' .. tostring(spec.id) end
  if not equal(spec, canonical) then return nil, 'reward specification does not match the catalogue' end
  local cls = self:classify(spec)
  if not cls.supported then return nil, cls.reason end
  local claim_id, why = self:_claim_header(ctx.run, ctx.progress, room_id)
  if not claim_id then return nil, why end
  local contract = ctx.effects
  if contract ~= nil and (type(contract) ~= 'table' or contract.version ~= 1) then
    return nil, 'unsupported effect contract version'
  end
  local crun = self:_clone_run(ctx.run)
  if not crun then return nil, 'run cannot be copied' end
  local cprog = copy(ctx.progress)
  local effects, benefit, cost, changes, target, message = {}, {}, {}, {}, nil, nil

  if cls.kind == 'upgrade' or cls.kind == 'tradeoff' then
    if ctx.target == nil then return nil, 'a target gene is required' end
    local gene = crun.genes[ctx.target]
    if not gene then return nil, 'unknown target gene' end
    if self:_enemy_owned(crun, ctx.target) then return nil, 'enemy-owned genes cannot be modified' end
    if not self:_kind_eligible(gene.kind, cls.family) then
      return nil, 'target family is incompatible with this reward'
    end
    local applied, awhy = self:_apply_stat(crun, ctx.target, cls)
    if not applied then return nil, awhy end
    benefit, cost, changes = applied.benefit, applied.cost, applied.changes
    target = {id = ctx.target, kind = gene.kind, family = self:_kind_family(gene.kind),
      slot = select(2, self:_placement(crun, ctx.target))}
    message = render(changes)
  elseif cls.kind == 'consumable' and cls.supply then
    local before = cprog.supplies
    local after = math.min(self.max_supplies, before + cls.supply)
    if after == before then return nil, 'supplies are already full' end
    cprog.supplies = after
    changes = {{stat = 'supply', label = 'SUPPLIES', before = before, after = after}}
    benefit = changes
    message = render(changes)
  elseif cls.kind == 'consumable' and cls.heal then
    if not (contract and contract.heal == true) then return nil, 'heal requires a staged effect contract' end
    local cap = integer(contract.max_heal, 1, 100) and contract.max_heal or 100
    if cls.heal > cap then return nil, 'heal exceeds the effect contract bound' end
    effects[#effects + 1] = {kind = 'heal', target = 'player', amount = cls.heal, contract_version = 1}
    changes = {{stat = 'heal', label = 'RESTORE', before = 0, after = cls.heal}}
    benefit = changes
    message = 'RESTORE ' .. cls.heal .. ' PERCENT'
  elseif cls.kind == 'consumable' and cls.charge then
    if not (contract and contract.charge == true) then return nil, 'charge requires a staged effect contract' end
    local gene = ctx.target and crun.genes[ctx.target]
    local host, slot
    if ctx.target then host, slot = self:_placement(crun, ctx.target) end
    if not gene or host ~= 'player' or not slot then return nil, 'charge target must be a player-equipped gene' end
    if not self:_kind_eligible(gene.kind, cls.family) then
      return nil, 'target family is incompatible with this reward'
    end
    local stats = self.Core.resolve(crun, 'player', slot)
    if not stats then return nil, 'charge target cannot be resolved' end
    local state = crun.hosts.player.state[slot]
    local current = state and math.floor(state.charge) or 0
    local room_left = stats.capacity - current
    if room_left <= 0 then return nil, 'charge is already full' end
    local granted = math.min(cls.charge, room_left)
    effects[#effects + 1] = {kind = 'charge', host = 'player', slot = slot, gene = ctx.target, amount = granted,
      before = current, target = current + granted, capacity = stats.capacity, contract_version = 1}
    changes = {{stat = 'charge', label = 'CHARGE', before = current, after = current + granted}}
    benefit = changes
    target = {id = ctx.target, kind = gene.kind, family = self:_kind_family(gene.kind), slot = slot}
    message = render(changes)
  else
    return nil, 'unsupported reward kind'
  end

  cprog.claimed[claim_id] = true
  -- The staged state must remain persistable exactly as main would save it.
  if not self.Core.snapshot(crun) then return nil, 'staged run is not serializable' end
  local pok, pwhy = self.progress_module.validate(cprog)
  if not pok then return nil, 'staged progress is invalid: ' .. tostring(pwhy) end
  return {kind = cls.kind, family = cls.family, spec = spec, claim_id = claim_id, room_id = room_id,
    run = crun, progress = cprog, target = target, benefit = benefit, cost = cost, changes = changes,
    effects = effects, message = message}
end

-- Read-only offer. With no target on a targeted reward it enumerates every
-- eligible target and its exact preview; otherwise it resolves the named target
-- (or the untargeted reward) with the same resolver commit() uses.
function RuntimeRewards:preview(ctx)
  ctx = ctx or {}
  local spec = ctx.spec or (ctx.node and ctx.node.reward_spec)
  if type(spec) ~= 'table' then return nil, 'no reward specification' end
  local canonical = self.encounters.rewards[spec.id]
  if not canonical or not equal(spec, canonical) then return nil, 'reward specification does not match the catalogue' end
  local cls = self:classify(spec)
  if not cls.supported then return nil, cls.reason end
  local room_id = ctx.room_id or (ctx.node and ctx.node.id)
  local claim_id, why = self:_claim_header(ctx.run, ctx.progress, room_id)
  if not claim_id then return nil, why end
  local view = {claim_id = claim_id, room_id = room_id, kind = cls.kind, family = cls.family, spec = spec,
    supported = true, requires_target = cls.requires_target == true, requires_effect = cls.requires_effect,
    targets = {}}
  if ctx.target ~= nil or not cls.requires_target then
    local result, rwhy = self:_transaction(ctx)
    if not result then return nil, rwhy end
    view.target, view.changes, view.benefit, view.cost = result.target, result.changes, result.benefit, result.cost
    view.effects, view.message = result.effects, result.message
    return view
  end
  for _, candidate in ipairs(self:_effect_targets(ctx.run, spec)) do
    local sub = copy(ctx)
    sub.target = candidate.id
    local result, rwhy = self:_transaction(sub)
    if result then
      view.targets[#view.targets + 1] = {id = candidate.id, kind = candidate.kind, family = candidate.family,
        slot = candidate.slot, supported = true, changes = result.changes, benefit = result.benefit,
        cost = result.cost, effects = result.effects, message = result.message}
    else
      view.targets[#view.targets + 1] = {id = candidate.id, kind = candidate.kind, family = candidate.family,
        slot = candidate.slot, supported = false, reason = rwhy}
    end
  end
  return view
end

-- Transactional commit. Returns staged run/progress copies, the stable claim id
-- and deferred effect intents. The caller persists run+progress and applies
-- effects; on any failure it simply keeps its originals (nothing here mutated
-- them).
function RuntimeRewards:commit(ctx)
  local result, why = self:_transaction(ctx)
  if not result then return nil, why end
  result.ok = true
  return result
end

return RuntimeRewards
