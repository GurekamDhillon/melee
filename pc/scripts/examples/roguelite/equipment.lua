-- Gate 7 pure equipment service.
--
-- A logical, versioned set of equipment with stable slots and a bounded owned
-- inventory. Items must be acquired before they can be equipped, so the
-- catalogue is not a free buff. Each equipped item contributes a bounded stat
-- delta to the player's equipped genes through the real Core modifier API.
--
-- apply() is a transactional reconciliation: it installs exactly the owned
-- equipment contributions for the record's host and removes any stale equipment
-- modifiers, all-or-nothing. If any Core add/remove fails or throws, the host's
-- modifier table is restored to its prior state and the call returns nil,reason.
-- Non-equipment contributions and other hosts are never touched. Because
-- Core.resolve rebuilds every stat from base + upgrades + active modifiers,
-- apply/revert round trips restore the exact source-derived value (no drift).
--
-- Ownership domain is checked before resolving/applying: the record must belong
-- to the run it is applied to and its host must exist. A foreign or malformed
-- record is refused; a valid record that contributes nothing returns 0.
--
-- This layer is purely logical. It never binds, deletes or replaces a fighter's
-- built-in weapon, shield, scabbard or other native article. Definitions that
-- request a native held item are refused ("native held items are not
-- supported"); claiming otherwise would be fake support.
local Equipment = {version = 1}

Equipment.max_slots = 8
Equipment.max_owned = 32
Equipment.max_owned_per_item = 4
Equipment.max_id = 64
Equipment.slots = {'weapon', 'offhand', 'core', 'plating', 'charm', 'boots', 'focus', 'sigil'}
Equipment.slot_set = {}
for _, slot in ipairs(Equipment.slots) do Equipment.slot_set[slot] = true end
Equipment.stats = {potency = true, capacity = true, gain = true, reach = true, cooldown = true}
Equipment.native_held_items_supported = false

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end
local function integer(x, lo, hi) return finite(x) and x % 1 == 0 and x >= lo and x <= hi end
local function valid_id(x) return type(x) == 'string' and #x > 0 and #x <= Equipment.max_id and not x:find('[%z\1-\31]') end

local function copy(t)
  if type(t) ~= 'table' then return t end
  local out = {}
  for k, v in pairs(t) do out[k] = copy(v) end
  return out
end

local function count(t) local n = 0 for _ in pairs(t) do n = n + 1 end return n end

local function sorted(t)
  local list = {}
  for k in pairs(t or {}) do list[#list + 1] = k end
  table.sort(list)
  return list
end

local function definition(id, slot, family, stat, delta, placements, extra)
  local def = {id = id, version = 1, name = extra and extra.name or id, slot = slot, family = family,
    stat = stat, delta = delta, placements = placements, implemented = true}
  if extra then
    def.two_handed = extra.two_handed or false
    def.occupies = extra.occupies or (extra.two_handed and {'offhand'} or {})
    for k, v in pairs(extra) do def[k] = v end
  end
  return def
end

-- At least eight supported definitions across stable slots plus two refusals
-- (an unimplemented item and a native held item).
Equipment.definitions = {
  ember_lens = definition('ember_lens', 'focus', 'fire', 'potency', 2, {'assault'}, {name = 'Ember Lens'}),
  rime_lens = definition('rime_lens', 'focus', 'frost', 'potency', 2, {'assault'}, {name = 'Rime Lens'}),
  storm_pin = definition('storm_pin', 'charm', 'any', 'gain', 0.5, {'assault', 'traversal'}, {name = 'Storm Pin'}),
  warding_charm = definition('warding_charm', 'charm', 'any', 'capacity', 1, {'assault', 'guard', 'traversal'}, {name = 'Warding Charm'}),
  bulwark_plate = definition('bulwark_plate', 'plating', 'any', 'capacity', 1, {'guard'}, {name = 'Bulwark Plate'}),
  swift_boots = definition('swift_boots', 'boots', 'any', 'cooldown', -10, {'traversal'}, {name = 'Swift Boots'}),
  long_reach_band = definition('long_reach_band', 'weapon', 'any', 'reach', 2, {'assault'}, {name = 'Long Reach Band', two_handed = true}),
  twin_daggers = definition('twin_daggers', 'weapon', 'any', 'potency', 1, {'assault'}, {name = 'Twin Daggers'}),
  buckler = definition('buckler', 'offhand', 'any', 'capacity', 1, {'guard'}, {name = 'Buckler'}),
  aegis_sigil = definition('aegis_sigil', 'sigil', 'any', 'capacity', 2, {'guard'}, {name = 'Aegis Sigil'}),
  null_core = definition('null_core', 'core', 'any', 'potency', 3, {'assault'}, {name = 'Null Core'}),
  -- Unimplemented mechanic (no visual/material pipeline yet).
  ghost_lens = {id = 'ghost_lens', version = 1, name = 'Ghost Lens', slot = 'focus', family = 'any',
    stat = 'potency', delta = 4, placements = {'assault'}, implemented = false},
  -- A definition requesting a native held item must be refused, not faked.
  ruin_blade = {id = 'ruin_blade', version = 1, name = 'Ruin Blade', slot = 'weapon', family = 'any',
    stat = 'potency', delta = 5, placements = {'assault'}, implemented = true, native_item = 'sword'},
}

function Equipment.def(id) return Equipment.definitions[id] end

function Equipment.classify(def)
  if type(def) ~= 'table' then return {supported = false, reason = 'invalid equipment definition'} end
  if not valid_id(def.id) then return {supported = false, reason = 'invalid equipment id'} end
  if def.version ~= 1 then return {supported = false, id = def.id, reason = 'unsupported equipment version'} end
  local out = {id = def.id, version = 1, name = def.name, slot = def.slot,
    family = def.family or 'any', stat = def.stat, delta = def.delta, supported = false}
  if def.native_item ~= nil or def.native ~= nil or def.held_item ~= nil or def.mesh ~= nil then
    out.reason = 'native held items are not supported'; return out
  end
  if def.implemented ~= true then out.reason = 'equipment is not implemented'; return out end
  if not Equipment.slot_set[def.slot] then out.reason = 'unknown equipment slot'; return out end
  if not (type(def.stat) == 'string' and Equipment.stats[def.stat]) then out.reason = 'unknown equipment stat'; return out end
  if not (finite(def.delta) and def.delta ~= 0 and math.abs(def.delta) <= 600) then out.reason = 'invalid equipment delta'; return out end
  if type(def.placements) ~= 'table' or #def.placements == 0 then out.reason = 'equipment has no placements'; return out end
  for _, p in ipairs(def.placements) do
    if p ~= 'assault' and p ~= 'guard' and p ~= 'traversal' then out.reason = 'unknown placement'; return out end
  end
  out.supported = true
  out.placements = copy(def.placements)
  out.occupies = copy(def.occupies or {})
  return out
end

-- Returns a conflict reason when `new_slot`/`new_cls` cannot coexist with the
-- current slot map, in either direction. The slot being explicitly replaced is
-- excluded so a deliberate swap is preserved.
local function occupancy_conflict(slots, new_slot, new_cls, exclude_slot)
  for _, occupied in ipairs(new_cls.occupies or {}) do
    if occupied ~= exclude_slot and slots[occupied] then return occupied end
  end
  for slot, other in pairs(slots) do
    if slot ~= new_slot and slot ~= exclude_slot then
      local other_def = Equipment.definitions[other]
      local other_cls = other_def and Equipment.classify(other_def)
      if other_cls and other_cls.supported then
        for _, occupied in ipairs(other_cls.occupies or {}) do
          if occupied == new_slot then return 'occupied by ' .. tostring(other) end
        end
      end
    end
  end
  return nil
end

function Equipment.new(Core, deps)
  assert(type(Core) == 'table' and type(Core.apply_modifier) == 'function'
    and type(Core.remove_modifier) == 'function' and type(Core.resolve) == 'function',
    'Core modifier API required')
  deps = deps or {}
  assert(deps.Codec == nil or type(deps.Codec.encode) == 'function', 'invalid Codec dependency')
  return setmetatable({Core = Core, Codec = deps.Codec}, {__index = Equipment})
end

function Equipment.create(opts)
  opts = opts or {}
  if not valid_id(opts.owner) then return nil, 'invalid equipment owner' end
  if not valid_id(opts.host or 'player') then return nil, 'invalid equipment host' end
  if not integer(opts.capacity, 0, Equipment.max_slots) then return nil, 'invalid equipment capacity' end
  return {version = 1, owner = opts.owner, host = opts.host or 'player',
    capacity = opts.capacity, slots = {}, owned = {}}
end

local FIELDS = {version = true, owner = true, host = true, capacity = true, slots = true, owned = true}
function Equipment.validate(record)
  if type(record) ~= 'table' or record.version ~= 1 then return false, 'unsupported equipment version' end
  for field in pairs(record) do if not FIELDS[field] then return false, 'unknown equipment field ' .. tostring(field) end end
  if not valid_id(record.owner) or not valid_id(record.host) then return false, 'invalid equipment owner/host' end
  if not integer(record.capacity, 0, Equipment.max_slots) then return false, 'invalid equipment capacity' end
  if type(record.slots) ~= 'table' or type(record.owned) ~= 'table' then return false, 'invalid equipment tables' end
  if count(record.owned) > Equipment.max_owned then return false, 'owned capacity exceeded' end
  for item, n in pairs(record.owned) do
    local def = Equipment.definitions[item]
    if not def then return false, 'unknown owned item ' .. tostring(item) end
    if not Equipment.classify(def).supported then return false, 'unsupported owned item ' .. tostring(item) end
    if not integer(n, 1, Equipment.max_owned_per_item) then return false, 'invalid owned count for ' .. tostring(item) end
  end
  local equipped = 0
  for slot, item in pairs(record.slots) do
    if not Equipment.slot_set[slot] then return false, 'unknown equipment slot ' .. tostring(slot) end
    local def = Equipment.definitions[item]
    if not def then return false, 'unknown equipment item ' .. tostring(item) end
    local cls = Equipment.classify(def)
    if not cls.supported then return false, cls.reason end
    if def.slot ~= slot then return false, 'equipment item is in the wrong slot' end
    if not (record.owned[item] and record.owned[item] >= 1) then return false, 'equipped item is not owned' end
    equipped = equipped + 1
  end
  if equipped > record.capacity then return false, 'equipment capacity exceeded' end
  -- Symmetric occupancy: reject a saved record where any item occupies a slot
  -- that already holds another item.
  for slot, item in pairs(record.slots) do
    local cls = Equipment.classify(Equipment.definitions[item])
    for _, occupied in ipairs(cls.occupies or {}) do
      if occupied ~= slot and record.slots[occupied] then
        return false, 'equipment occupancy conflict: ' .. item .. ' occupies ' .. occupied
      end
    end
  end
  return true
end

function Equipment.owns(record, item_id) return (record.owned and record.owned[item_id] or 0) > 0 end

-- Staged acquisition. Consumable-style equipment stack counts are bounded, and
-- the number of distinct owned items is bounded.
function Equipment.acquire(record, item_id, amount)
  local ok, why = Equipment.validate(record)
  if not ok then return nil, why end
  local def = Equipment.definitions[item_id]
  if not def then return nil, 'unknown equipment item' end
  local cls = Equipment.classify(def)
  if not cls.supported then return nil, cls.reason end
  amount = amount or 1
  if not integer(amount, 1, Equipment.max_owned_per_item) then return nil, 'invalid acquire amount' end
  local have = record.owned[item_id] or 0
  if have + amount > Equipment.max_owned_per_item then return nil, 'owned stack capacity' end
  if have == 0 and count(record.owned) >= Equipment.max_owned then return nil, 'owned capacity' end
  local out = copy(record)
  out.owned[item_id] = have + amount
  return out
end

-- Staged release. An item must be unequipped before it can be released, so a
-- release can never silently strip a contribution.
function Equipment.release(record, item_id, amount)
  local ok, why = Equipment.validate(record)
  if not ok then return nil, why end
  for _, equipped in pairs(record.slots) do
    if equipped == item_id then return nil, 'unequip before release' end
  end
  amount = amount or 1
  if not integer(amount, 1, Equipment.max_owned_per_item) then return nil, 'invalid release amount' end
  local have = record.owned[item_id] or 0
  if amount > have then return nil, 'not enough owned' end
  local out = copy(record)
  out.owned[item_id] = have - amount
  if out.owned[item_id] <= 0 then out.owned[item_id] = nil end
  return out
end

function Equipment.equip(record, item_id)
  local ok, why = Equipment.validate(record)
  if not ok then return nil, why end
  local def = Equipment.definitions[item_id]
  if not def then return nil, 'unknown equipment item' end
  local cls = Equipment.classify(def)
  if not cls.supported then return nil, cls.reason end
  if not Equipment.owns(record, item_id) then return nil, 'equipment not owned' end
  local out = copy(record)
  local conflict = occupancy_conflict(out.slots, def.slot, cls, def.slot)
  if conflict then return nil, 'occupancy conflict: ' .. tostring(conflict) end
  if out.slots[def.slot] == nil and count(out.slots) >= out.capacity then return nil, 'equipment capacity' end
  out.slots[def.slot] = item_id
  return out
end

function Equipment.unequip(record, slot)
  local ok, why = Equipment.validate(record)
  if not ok then return nil, why end
  if not Equipment.slot_set[slot] then return nil, 'unknown equipment slot' end
  if not record.slots[slot] then return nil, 'slot already empty' end
  local out = copy(record)
  out.slots[slot] = nil
  return out
end

-- Ownership domain: a record may only affect the run it belongs to.
function Equipment:_domain(record, run, require_active)
  if type(run) ~= 'table' or run.type ~= 'run' then return nil, 'run is required' end
  if require_active and run.status ~= 'active' then return nil, 'run is not active' end
  if record.owner ~= run.id then return nil, 'equipment belongs to another run' end
  if not (run.hosts and run.hosts[record.host]) then return nil, 'unknown equipment host' end
  return true
end

-- Resolve the desired modifier set from the owned, equipped record. Assumes the
-- record/domain were already validated.
function Equipment:_desired(record, run)
  local out = {}
  local h = run.hosts[record.host]
  for _, slot in ipairs(sorted(record.slots)) do
    local item = record.slots[slot]
    local def = Equipment.definitions[item]
    local cls = def and Equipment.classify(def)
    if cls and cls.supported then
      for _, gene_slot in ipairs(cls.placements) do
        local gene_id = h.slots[gene_slot]
        local gene = gene_id and run.genes and run.genes[gene_id]
        if gene then
          local core_def = self.Core.definitions and self.Core.definitions[gene.kind]
          local family = core_def and core_def.family
          if cls.family == 'any' or cls.family == family then
            out[#out + 1] = {id = 'eq:' .. record.host .. ':' .. item .. ':' .. gene_slot, host = record.host,
              slot = gene_slot, item = item, stat = cls.stat, add = cls.delta}
          end
        end
      end
    end
  end
  return out
end

-- Public read-only resolution. Refuses a malformed or foreign record rather
-- than returning an empty success.
function Equipment:modifiers(record, run)
  local ok, why = Equipment.validate(record)
  if not ok then return nil, why end
  local dom, dom_why = self:_domain(record, run, false)
  if not dom then return nil, dom_why end
  return self:_desired(record, run)
end

-- Transactional reconciliation. Installs exactly the owned equipment
-- contributions and removes stale equipment modifiers for this host. Any
-- failure (Core returns false, throws, or a capacity refusal) restores the
-- host's modifier table and returns nil,reason. Other hosts and non-equipment
-- (non `eq:<host>:`) contributions are untouched. Returns the applied count,
-- which is 0 for a valid record with no contributions.
function Equipment:apply(run, record)
  local ok, why = Equipment.validate(record)
  if not ok then return nil, why end
  local dom, dom_why = self:_domain(record, run, true)
  if not dom then return nil, dom_why end
  local desired = self:_desired(record, run)
  local desired_by_id = {}
  for _, m in ipairs(desired) do desired_by_id[m.id] = m end
  local host = record.host
  local h = run.hosts[host]
  h.modifiers = h.modifiers or {}
  local before = copy(h.modifiers)
  local prefix = 'eq:' .. host .. ':'
  local stale = {}
  for slot, mods in pairs(h.modifiers) do
    for id in pairs(mods) do
      if type(id) == 'string' and id:sub(1, #prefix) == prefix and not desired_by_id[id] then
        stale[#stale + 1] = {slot = slot, id = id}
      end
    end
  end
  local function attempt()
    for _, s in ipairs(stale) do
      local removed, err = self.Core.remove_modifier(run, host, s.slot, s.id)
      -- A false/nil return is a refusal even if the callback already mutated the
      -- table; the snapshot restore below undoes any such side effect.
      if not removed then error(err or 'remove failed', 0) end
    end
    for _, m in ipairs(desired) do
      local added, err = self.Core.apply_modifier(run, host, m.slot, {id = m.id, stat = m.stat, add = m.add})
      if not added then error(err or 'apply failed', 0) end
    end
  end
  local ran, err = pcall(attempt)
  if not ran then h.modifiers = copy(before); return nil, tostring(err) end
  for _, m in ipairs(desired) do
    local mods = h.modifiers[m.slot]
    local stored = mods and mods[m.id]
    if not (stored and stored.stat == m.stat and stored.add == m.add) then
      h.modifiers = copy(before); return nil, 'modifier reconciliation failed'
    end
  end
  for slot, mods in pairs(h.modifiers) do
    for id in pairs(mods) do
      if type(id) == 'string' and id:sub(1, #prefix) == prefix and not desired_by_id[id] then
        h.modifiers = copy(before); return nil, 'stale equipment modifier remained'
      end
    end
  end
  return #desired
end

-- Transactional removal of every equipment modifier owned by this record's host,
-- regardless of the current gene placement, so nothing stale can survive a swap
-- or unequip. Same all-or-nothing contract as apply: a Core remove that returns
-- false/nil or throws restores the host's modifier table and returns nil,reason;
-- a successful clear returns the removed count (0 is a real success, distinct
-- from a refusal). Only this host's `eq:<host>:` modifiers are ever touched, and
-- only this host's modifier table is replaced on rollback.
function Equipment:revert(run, record)
  local ok, why = Equipment.validate(record)
  if not ok then return nil, why end
  local dom, dom_why = self:_domain(record, run, false)
  if not dom then return nil, dom_why end
  local host = record.host
  local h = run.hosts[host]
  h.modifiers = h.modifiers or {}
  local before = copy(h.modifiers)
  local prefix = 'eq:' .. host .. ':'
  local owned = {}
  for slot, mods in pairs(h.modifiers) do
    for id in pairs(mods) do
      if type(id) == 'string' and id:sub(1, #prefix) == prefix then owned[#owned + 1] = {slot = slot, id = id} end
    end
  end
  local function attempt()
    for _, s in ipairs(owned) do
      local removed, err = self.Core.remove_modifier(run, host, s.slot, s.id)
      if not removed then error(err or 'remove failed', 0) end
    end
  end
  local ran, err = pcall(attempt)
  if not ran then h.modifiers = copy(before); return nil, tostring(err) end
  for slot, mods in pairs(h.modifiers) do
    for id in pairs(mods) do
      if type(id) == 'string' and id:sub(1, #prefix) == prefix then
        h.modifiers = copy(before); return nil, 'stale equipment modifier remained'
      end
    end
  end
  return #owned
end

function Equipment:encode(record)
  if not self.Codec then return nil, 'Codec dependency required' end
  local ok, why = Equipment.validate(record)
  if not ok then return nil, why end
  return self.Codec.encode(copy(record))
end

function Equipment:decode(text)
  if not self.Codec then return nil, 'Codec dependency required' end
  local value, why = self.Codec.decode(text)
  if not value then return nil, why end
  local ok, why2 = Equipment.validate(value)
  if not ok then return nil, why2 end
  return value
end

return Equipment
