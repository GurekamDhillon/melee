-- Common resolved ability transaction for genes (Gate 5).
--
-- Pure orchestration over the real Core charge/spend APIs. It never calls an
-- engine global and never invents a gd.* function: native application arrives
-- through an explicit `world` adapter. A required adapter callback that is
-- missing makes the transaction fail closed instead of pretending a hit.
--
--   preflight -> startup -> active -> recovery -> settled
--
-- Clock: run.frame only, bounded and deterministic. Charge is spent once, at
-- the startup->active release point, through Core.activate.
--
-- Concurrency/ownership contract (review 2026-09-30):
--   * The engine never replaces or snapshots the whole run. A refusal refunds
--     only the acting slot's cost (charge/ready_at) and the marks it touched,
--     so unrelated contact charge, modifiers and intervening ticks survive.
--   * Every phase re-validates the authoritative run, host, slot and gene
--     instance. Equipment/room/loadout changes cancel the pending action
--     instead of spending whatever gene is now equipped.
--   * Native callbacks are protected; false/nil/throw are refusals, never
--     silent successes. Owned contributions are released only through a
--     promised release seam and are retried until confirmed.
--   * Charging keys are per slot and honour each gene's authored earning
--     policy; Core.on_event dedup for other installed genes is preserved.
--
-- Prototype families stay unoffered: by default only cinder/rime are admitted.
local GA = {version = 3}
GA.caps = {records = 96, hosts = 32, lifetime = 600, provenance = 128,
  contributions = 32, contrib_retries = 3, default_contrib = 300}
GA.slots = {assault = true, traversal = true, guard = true}
GA.phase_order = {'preflight', 'startup', 'active', 'recovery', 'settled'}

-- Effect kinds that create an owned, reversible contribution handle, and the
-- adapter release callback each maps to. `convert`/`siphon` share the
-- conversion route, so they release through `release_conversion`.
local contribution_kinds = {guard = true, convert = true, siphon = true}
local release_kind = {guard = 'guard', convert = 'conversion', siphon = 'conversion'}

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end
local function integer(x) return finite(x) and x % 1 == 0 end
local function name(x) return type(x) == 'string' and #x > 0 and #x <= 64 and not x:find('[%z\1-\31]') end
local function copy(t)
  if type(t) ~= 'table' then return t end
  local o = {}
  for k, v in pairs(t) do o[k] = copy(v) end
  return o
end
local function count(t) local n = 0 for _ in pairs(t) do n = n + 1 end return n end
local function key(host, slot) return host .. '@' .. slot end

-- Effect kind -> native adapter callback. `mark` native is handled by
-- Core.activate itself; `counter` against a self target is a guard stance.
local seam_by_kind = {
  burst = 'apply_effect', line = 'apply_effect', chain = 'apply_effect', push = 'apply_effect',
  mark = 'apply_status', root = 'apply_status',
  dash = 'apply_movement', hover = 'apply_movement',
  guard = 'apply_guard', counter = 'apply_effect',
  convert = 'apply_conversion', siphon = 'apply_conversion',
}

local methods = {}

local function seam_for(spec)
  local kind = spec.effect.kind
  if kind == 'mark' and spec.native then return nil end
  if kind == 'counter' and spec.targeting == 'self' then return 'apply_guard' end
  return seam_by_kind[kind]
end

local function release_seam(world, kind)
  local mapped = release_kind[kind] or kind
  return world['release_' .. mapped] or world.release
end


-- Pure geometric target policy: horizontal reach, vertical band and facing arc.
local function in_policy(spec, ability, host_view, target)
  if type(host_view) ~= 'table' then return false, 'no host observation' end
  local reach = ability and ability.reach
  if not finite(reach) or reach <= 0 then reach = spec.range end
  local dx, dy = target.x - host_view.x, target.y - host_view.y
  if math.abs(dx) > reach then return false, 'out of range' end
  local band = spec.height > 0 and spec.height or 12
  if math.abs(dy) > band then return false, 'out of vertical band' end
  if spec.arc == 'facing' then
    local facing = host_view.facing or 1
    if dx * facing < 0 then return false, 'behind the attacker' end
  end
  return true
end

function GA.new(Core, Behaviors, world, opts)
  opts = opts or {}
  assert(type(Core) == 'table', 'Core required')
  for _, fn in ipairs({'ability', 'activate', 'on_event', 'tick', 'resolve', 'apply_modifier',
    'remove_modifier', 'mark_revision', 'restore_mark', 'spend_revision', 'restore_spend'}) do
    assert(type(Core[fn]) == 'function', 'Core.' .. fn .. ' required')
  end
  assert(type(Behaviors) == 'table' and type(Behaviors.validate) == 'function', 'Behaviors required')
  local ok, why = Behaviors.validate(Behaviors)
  assert(ok, why)
  if type(Behaviors.from_core) == 'function' then
    local match, err = Behaviors.from_core(Core)
    assert(match, err)
  end
  assert(type(world) == 'table', 'world adapter required')
  local self = setmetatable({}, {__index = methods})
  self.Core, self.B, self.world, self.opts = Core, Behaviors, world, opts
  self.admitted = {}
  local source = opts.admitted or Behaviors.admitted_defaults()
  for id, v in pairs(source) do if v then self.admitted[id] = true end end
  self.caps = copy(GA.caps)
  for k, v in pairs(opts.caps or {}) do
    assert(GA.caps[k] and integer(v) and v > 0, 'invalid cap ' .. tostring(k))
    self.caps[k] = v
  end
  self.records, self.contributions, self.settled, self.events = {}, {}, {}, {}
  self.serial = 0
  self.stats = {started = 0, settled = 0, interrupted = 0, refused = 0, refunded = 0,
    refund_refused = 0, released = 0, stuck = 0}
  return self
end

function methods:emit(event) self.events[#self.events + 1] = event end
function methods:get_run()
  return type(self.world.get_run) == 'function' and self.world.get_run() or nil
end
function methods:observe(host)
  if type(self.world.observe) ~= 'function' then return nil end
  local v = self.world.observe(host)
  return type(v) == 'table' and v or nil
end

-- Read-only preflight. Returns a plan or nil, reason. Never mutates.
function methods:preflight(host, slot, opts)
  if not name(host) or not GA.slots[slot] then return nil, 'invalid host or slot' end
  local run = self:get_run()
  if not run or run.status ~= 'active' then return nil, 'run inactive' end
  local host_rec = run.hosts and run.hosts[host]
  if not host_rec then return nil, 'unknown host' end
  local id = host_rec.slots[slot]
  if not id then return nil, 'empty slot' end
  local instance = run.genes[id]
  if not instance then return nil, 'gene instance missing' end
  local gene = self.B.genes[instance.kind]
  if not gene then return nil, 'no behavior definition' end
  if not self.admitted[gene.id] then return nil, 'definition not admitted' end
  if self.records[key(host, slot)] then return nil, 'action already in progress' end
  local spec = gene.placements[slot]
  if not spec then return nil, 'placement unsupported' end
  local ability, err = self.Core.ability(run, host, slot)
  if not ability then return nil, err or 'no ability' end
  if spec.native and ability.action ~= spec.action then return nil, 'core action drift' end
  if not ability.ready then return nil, 'not ready' end
  -- Free-state and occlusion are promised behaviour: fail closed if absent.
  if type(self.world.can_start) ~= 'function' then return nil, 'free-state seam missing' end
  local free, reason = self.world.can_start(run, host, slot)
  if free ~= true then return nil, reason or 'not free to act' end
  local seam = seam_for(spec)
  if seam and type(self.world[seam]) ~= 'function' then
    return nil, 'native ' .. seam .. ' seam missing'
  end
  local kind = spec.effect.kind
  if contribution_kinds[kind] then
    if type(release_seam(self.world, kind)) ~= 'function' then return nil, 'release seam missing' end
    -- Pending reservations count against the cap too, so simultaneous activations
    -- cannot each reserve the same free slot.
    if self:contribution_load() >= self.caps.contributions then return nil, 'contribution capacity' end
  end
  local target
  if spec.targeting == 'opponent' then
    if type(self.world.query) ~= 'function' then return nil, 'target query seam missing' end
    target = self.world.query(host, spec, ability)
    if type(target) ~= 'table' or not name(target.host) then return nil, 'no valid target' end
    if target.host == host then return nil, 'self target refused' end
    if target.reflecting then return nil, 'target reflects' end
    if target.shielding then return nil, 'target shielded' end
    local host_view = self:observe(host)
    local ok, why = in_policy(spec, ability, host_view, target)
    if not ok then return nil, why end
    if spec.occlusion == 'line' then
      if type(self.world.occluded) ~= 'function' then return nil, 'occlusion seam missing' end
      if self.world.occluded(host, target) then return nil, 'target occluded' end
    end
    -- Fail closed if this action would mutate a mark on a target Core can no
    -- longer track; Core also enforces this at the spend point.
    if (spec.action == 'mark' or gene.family == 'fire')
      and self.Core.mark_revision(run, target.host) == nil then
      return nil, 'mark revision capacity'
    end
    if opts and opts.reaction and target.host == host then return nil, 'self reaction refused' end
  end
  return {host = host, slot = slot, key = key(host, slot), gene = gene, id = id, spec = spec,
    ability = copy(ability), route = self.B.mechanics[spec.mechanic].route,
    run_id = run.id, target = target and copy(target) or nil}
end

function methods:begin(host, slot, opts)
  local plan, reason = self:preflight(host, slot, opts)
  if not plan then
    self.stats.refused = self.stats.refused + 1
    return nil, reason
  end
  if count(self.records) >= self.caps.records then return nil, 'action record cap' end
  local run = self:get_run()
  -- Optional generic intent hook: capture an opaque, immutable native identity
  -- token after target selection and before any spend. A present hook must
  -- return a token (nil refuses) and an explicit validate_intent must return
  -- exactly true at the spend point. Absent hooks leave injected worlds that do
  -- not implement them unchanged.
  local intent
  if type(self.world.capture_intent) == 'function' then
    local pok, token, iwhy = pcall(self.world.capture_intent, run, plan.host, plan.slot, plan.target, plan.spec)
    if not pok then
      self.stats.refused = self.stats.refused + 1
      return nil, 'intent capture threw: ' .. tostring(token)
    end
    if token == nil then
      self.stats.refused = self.stats.refused + 1
      return nil, type(iwhy) == 'string' and iwhy or 'intent unavailable'
    end
    intent = token
  end
  local host_rec = run.hosts[plan.host]
  self.serial = self.serial + 1
  local move_id = host .. ':' .. tostring(run.frame) .. ':' .. slot .. ':' .. tostring(self.serial)
  local rec = {host = plan.host, slot = plan.slot, key = plan.key, gene = plan.gene, gene_id = plan.gene.id,
    id = plan.id, run_id = plan.run_id, spec = plan.spec, route = plan.route, ability = plan.ability,
    run = run, instance = run.genes[plan.id], state = host_rec and host_rec.state[plan.slot] or nil,
    target = plan.target, phase = 'startup', start_frame = run.frame,
    release_frame = run.frame + plan.spec.startup, active_until = nil, recovery_until = nil,
    move_id = move_id, serial = self.serial, spent = false, applied = false, contrib = nil,
    reaction = nil, lineage = nil, intent = intent}
  self.records[plan.key] = rec
  self.stats.started = self.stats.started + 1
  self:emit({kind = 'startup', host = host, slot = slot, move_id = move_id, gene = plan.gene.id,
    route = plan.route, phase = 'startup', frame = run.frame})
  return self:view_record(rec)
end

function methods:request(rec, action)
  return {host = rec.host, slot = rec.slot, gene = rec.gene_id, id = rec.id, action = action,
    spec = copy(rec.spec), move_id = rec.move_id, route = rec.route, native = rec.spec.native,
    target = rec.target and rec.target.host or nil, target_view = rec.target and copy(rec.target) or nil,
    host_view = self:observe(rec.host), intent = rec.intent}
end

function methods:view_record(rec)
  return {host = rec.host, slot = rec.slot, gene = rec.gene_id, action = rec.spec.action,
    mechanic = rec.spec.mechanic, kind = rec.spec.effect.kind, route = rec.route, phase = rec.phase,
    move_id = rec.move_id, serial = rec.serial, run_id = rec.run_id, start_frame = rec.start_frame,
    release_frame = rec.release_frame, spent = rec.spent, applied = rec.applied,
    target = rec.target and rec.target.host or nil, reaction = rec.reaction, lineage = rec.lineage}
end

function methods:view()
  local keys = {}
  for k in pairs(self.records) do keys[#keys + 1] = k end
  table.sort(keys)
  local out = {}
  for _, k in ipairs(keys) do out[#out + 1] = self:view_record(self.records[k]) end
  return out
end

-- Stable identity: the exact run table, gene instance table, slot state table
-- and placement the action was planned against. A textual run id is not enough
-- (two profiles share `run1`). Any change cancels instead of spending a
-- replacement gene.
function methods:validate_identity(rec, run)
  if run ~= rec.run then return false, 'run instance changed' end
  if not run or run.status ~= 'active' then return false, 'run inactive' end
  if run.genes[rec.id] ~= rec.instance then return false, 'gene instance changed' end
  local host_rec = run.hosts[rec.host]
  if not host_rec or host_rec.slots[rec.slot] ~= rec.id then return false, 'equipment changed' end
  if rec.state and host_rec.state[rec.slot] ~= rec.state then return false, 'slot state changed' end
  local ability = self.Core.ability(run, rec.host, rec.slot)
  if not ability or ability.action ~= rec.spec.action then return false, 'action drift' end
  return true
end

-- Release one owned contribution through its promised seam. false/nil/throw are
-- refusals; the handle is retained by the caller for retry.
function methods:release_contrib(contrib, run)
  local rel = release_seam(self.world, contrib.kind)
  if type(rel) ~= 'function' then return false, 'release seam missing' end
  local pok, ok = pcall(rel, run, contrib)
  if not pok then return false, 'release threw: ' .. tostring(ok) end
  if ok ~= true then return false, 'release refused' end
  return true
end

function methods:interrupt(host, slot, reason)
  local k = key(host, slot)
  local rec = self.records[k]
  if not rec then return nil, 'no action' end
  local run = rec.run or self:get_run()
  if rec.contrib then
    local c = rec.contrib
    c.host, c.slot, c.serial = rec.host, rec.slot, rec.serial
    c.run = rec.run
    c.expires = (run and run.frame or 0) + 1
    local ok = self:release_contrib(c, run)
    if not ok then self.contributions[rec.serial] = c end
    rec.contrib = nil
  end
  self.records[k] = nil
  self.stats.interrupted = self.stats.interrupted + 1
  self:emit({kind = 'interrupted', host = host, slot = slot, move_id = rec.move_id,
    reason = reason, spent = rec.spent})
  return {status = 'interrupted', host = host, slot = slot, move_id = rec.move_id, reason = reason,
    spent = rec.spent}
end

-- Re-observe the chosen target. Used at release so a dodge or occlusion whiffs
-- before any charge is spent.
function methods:refresh_target(rec, run)
  local spec = rec.spec
  if spec.targeting ~= 'opponent' then return true end
  if type(self.world.observe) ~= 'function' then return true end
  local fresh = self.world.observe(rec.target.host)
  if type(fresh) ~= 'table' or fresh.alive == false then return false, 'target gone' end
  if fresh.reflecting then return false, 'target reflects' end
  if fresh.shielding then return false, 'target shielded' end
  local host_view = self:observe(rec.host)
  local ok, why = in_policy(spec, rec.ability, host_view, fresh)
  if not ok then return false, why end
  if spec.occlusion == 'line' and type(self.world.occluded) == 'function'
    and self.world.occluded(rec.host, fresh) then
    return false, 'target occluded'
  end
  rec.target = copy(fresh)
  return true
end

-- Release the contribution a refused apply created, so a failed native effect
-- cannot orphan an owned handle. Used by both refund policies.
function methods:cleanup_contrib(rec)
  if not rec.contrib then return end
  local c = rec.contrib
  c.host, c.slot, c.serial = rec.host, rec.slot, rec.serial
  c.run = rec.run
  c.expires = (rec.run and rec.run.frame or 0) + 1
  if self:release_contrib(c, rec.run) then rec.contrib = nil
  else self.contributions[rec.serial] = c; rec.contrib = nil end
end

-- Targeted refund: only the original gene's own state table, the mark
-- transaction this action owned, and its own contribution. A slot swapped
-- during the callback cannot receive the refund, and a later activation (even
-- in the same frame with an equal ready_at) owns the cooldown, so its spend is
-- preserved. Unrelated charge/modifiers/ticks are preserved.
function methods:refund(rec, cost, capacity, pre)
  -- Restore the original spend only if the exact runtime state object still
  -- holds the revision this action produced. A state moved to another slot or a
  -- newer spend on the same state (any slot/host, even an equal-value ABA) owns
  -- the cooldown, so the old refund is refused.
  local restored = false
  if rec.spend_owned then
    restored = self.Core.restore_spend(rec.state, rec.spend_rev_after,
      pre.ready_at, cost, capacity) == true
  end
  -- Conditional mark restore: Core refuses if any later mutation (another
  -- action's set/consume or an expiry tick) advanced the target's revision, or
  -- if the captured mark has since expired. This stops a nil->mark->nil
  -- interleaving or a callback tick from resurrecting an unrelated mark.
  if rec.mark_owned and rec.mark_target then
    self.Core.restore_mark(rec.run, rec.mark_target, rec.mark_before, rec.mark_rev_after)
  end
  self:cleanup_contrib(rec)
  -- Never claim a refund that the ownership check refused.
  if restored then self.stats.refunded = self.stats.refunded + 1
  else self.stats.refund_refused = self.stats.refund_refused + 1 end
end

-- Native application. Only exact `true` is success; false/nil/throw refuse.
function methods:apply(rec, run, action)
  local spec = rec.spec
  local kind = spec.effect.kind
  if kind == 'mark' and spec.native then return true, nil end
  local seam = seam_for(spec)
  local fn = seam and self.world[seam]
  if type(fn) ~= 'function' then return false, 'native ' .. tostring(seam) .. ' seam missing' end
  local pok, ok, detail = pcall(fn, run, self:request(rec, action))
  if not pok then return false, 'native ' .. seam .. ' threw: ' .. tostring(ok) end
  if ok ~= true then
    return false, type(detail) == 'string' and detail or 'effect refused'
  end
  if type(detail) == 'table' and detail.contrib then rec.contrib = detail.contrib end
  return true, detail
end

function methods:release(rec, run)
  local ident, iwhy = self:validate_identity(rec, run)
  if not ident then self:interrupt(rec.host, rec.slot, iwhy or 'identity changed') return end
  -- Free-state is promised behaviour and can change after begin (hitstun,
  -- landing, a menu). Re-check it at the spend point and fail closed if absent.
  if type(self.world.can_start) ~= 'function' then
    self:interrupt(rec.host, rec.slot, 'free-state seam missing')
    return
  end
  local free, freason = self.world.can_start(run, rec.host, rec.slot)
  if free ~= true then self:interrupt(rec.host, rec.slot, freason or 'not free to act') return end
  -- Validate the captured native intent before the spend point. A changed
  -- source/target/run/room identity or a now-missing capability cancels the
  -- action with no spend; a present hook must return exactly true.
  if rec.intent ~= nil then
    if type(self.world.validate_intent) ~= 'function' then
      self:interrupt(rec.host, rec.slot, 'intent validate seam missing')
      return
    end
    local pok, vok, iwhy = pcall(self.world.validate_intent, run, rec.intent)
    if not pok or vok ~= true then
      self:interrupt(rec.host, rec.slot, iwhy or 'intent changed')
      return
    end
  end
  local valid, twhy = self:refresh_target(rec, run)
  if not valid then self:interrupt(rec.host, rec.slot, twhy or 'target escaped') return end
  local st = rec.state
  local ability = self.Core.ability(run, rec.host, rec.slot)
  local cost = (ability and ability.cost) or rec.ability.cost
  -- Capacity is a Core.resolve stat, not part of Core.ability.
  local resolved = self.Core.resolve(run, rec.host, rec.slot)
  local capacity = resolved and resolved.capacity
  local mark_target = rec.target and rec.target.host or nil
  -- Read the target's mark revision before our mutation; Core.activate bumps it
  -- if this action sets or consumes a mark. Only a revision change means this
  -- action owns a mark transaction worth rolling back.
  local mark_rev_before = mark_target and self.Core.mark_revision(run, mark_target) or nil
  local mark_before = mark_target and copy(run.marks[mark_target]) or nil
  local pre = {ready_at = st and st.ready_at}
  -- Spend ownership follows the exact runtime state object, not the slot.
  local spend_rev_before = self.Core.spend_revision(st)
  local pok, action, aerr = pcall(self.Core.activate, run, rec.host, rec.slot, {target = mark_target})
  if not pok then
    self:interrupt(rec.host, rec.slot, 'spend threw: ' .. tostring(action))
    return
  end
  if type(action) ~= 'table' then
    self:interrupt(rec.host, rec.slot, aerr or 'spend refused')
    return
  end
  rec.spent = true
  rec.reaction = action.reaction
  rec.lineage = action.lineage
  rec.mark_target = mark_target
  rec.mark_before = mark_before
  rec.mark_rev_after = mark_target and self.Core.mark_revision(run, mark_target) or nil
  rec.mark_owned = mark_target ~= nil and mark_rev_before ~= nil
    and rec.mark_rev_after ~= nil and rec.mark_rev_after ~= mark_rev_before
  rec.spend_rev_after = self.Core.spend_revision(st)
  rec.spend_owned = spend_rev_before ~= nil and rec.spend_rev_after ~= nil
    and rec.spend_rev_after ~= spend_rev_before
  local applied, detail = self:apply(rec, run, action)
  -- Reentrant cleanup (e.g. a room-leave inside the native callback) removed the
  -- record: cancel the intent, apply the authored refund policy and release any
  -- handle the callback created after it returned.
  if self.records[rec.key] ~= rec then
    if rec.spec.refund == 'never' then self:cleanup_contrib(rec)
    else self:refund(rec, cost, capacity, pre) end
    return
  end
  if not applied then
    if rec.spec.refund == 'never' then self:cleanup_contrib(rec)
    else self:refund(rec, cost, capacity, pre) end
    self:interrupt(rec.host, rec.slot, detail or 'effect refused')
    return
  end
  rec.applied = true
  rec.phase = 'active'
  rec.active_until = run.frame + rec.spec.active
  self:emit({kind = 'active', host = rec.host, slot = rec.slot, move_id = rec.move_id,
    gene = rec.gene_id, action = action.action, reaction = rec.reaction, frame = run.frame})
end

function methods:settle(rec, run)
  local entry = {serial = rec.serial, source = rec.host, slot = rec.slot, gene = rec.gene_id,
    id = rec.id, action = rec.spec.action, mechanic = rec.spec.mechanic, kind = rec.spec.effect.kind,
    route = rec.route, move_id = rec.move_id, target = rec.target and rec.target.host or nil,
    reaction = rec.reaction, lineage = rec.lineage, spent = rec.spent, applied = rec.applied,
    frame = run and run.frame or nil}
  if rec.contrib then
    local c = rec.contrib
    c.host, c.slot, c.serial = rec.host, rec.slot, rec.serial
    c.run = rec.run or run
    local life = rec.spec.effect.duration or self.caps.default_contrib
    c.created = run and run.frame or 0
    c.expires = (run and run.frame or 0) + math.min(life, self.caps.lifetime)
    self.contributions[rec.serial] = c
    rec.contrib = nil
  end
  self.settled[#self.settled + 1] = entry
  if #self.settled > self.caps.provenance then table.remove(self.settled, 1) end
  self.records[rec.key] = nil
  self.stats.settled = self.stats.settled + 1
  self:emit({kind = 'settled', host = rec.host, slot = rec.slot, move_id = rec.move_id,
    provenance = copy(entry)})
end

function methods:expire_contributions(run)
  if not run then return end
  local serials = {}
  for serial in pairs(self.contributions) do serials[#serials + 1] = serial end
  table.sort(serials)
  for _, serial in ipairs(serials) do
    local c = self.contributions[serial]
    local target_run = (c and c.run) or run
    if c and target_run and c.expires and target_run.frame >= c.expires then
      local ok, err = self:release_contrib(c, target_run)
      if ok then
        self.contributions[serial] = nil
        self.stats.released = self.stats.released + 1
      else
        c.retry = (c.retry or 0) + 1
        if c.retry >= self.caps.contrib_retries and not c.stuck then
          c.stuck = true
          self.stats.stuck = self.stats.stuck + 1
          self:emit({kind = 'contribution_stuck', serial = serial, reason = err})
        end
      end
    end
  end
end

function methods:step(rec, run)
  if not run or run.status ~= 'active' then self:interrupt(rec.host, rec.slot, 'run inactive') return end
  local age = run.frame - rec.start_frame
  if not integer(age) or age < 0 or age > self.caps.lifetime then
    self:interrupt(rec.host, rec.slot, 'clock bounds')
    return
  end
  -- Cancel if identity changed: a different run instance, gene instance, slot or
  -- Core action. This covers the spend point and every later phase.
  local ident, why = self:validate_identity(rec, run)
  if not ident then self:interrupt(rec.host, rec.slot, why or 'identity changed') return end
  if rec.phase == 'startup' and run.frame >= rec.release_frame then
    self:release(rec, run)
  end
  if self.records[rec.key] and rec.phase == 'active' and run.frame >= rec.active_until then
    rec.phase = 'recovery'
    rec.recovery_until = run.frame + rec.spec.recovery
    self:emit({kind = 'recovery', host = rec.host, slot = rec.slot, move_id = rec.move_id, frame = run.frame})
  end
  if self.records[rec.key] and rec.phase == 'recovery' and run.frame >= rec.recovery_until then
    self:settle(rec, run)
  end
end

-- Advance every active transaction one observation. Called once after Core.tick.
-- The authoritative run is reacquired for every record so a change between
-- records cannot be missed.
function methods:advance()
  self.events = {}
  local run = self:get_run()
  if run then self:expire_contributions(run) end
  local keys = {}
  for k in pairs(self.records) do keys[#keys + 1] = k end
  table.sort(keys)
  for _, k in ipairs(keys) do
    local rec = self.records[k]
    if rec then self:step(rec, self:get_run()) end
  end
  local out = self.events
  self.events = {}
  return out
end

-- Charge from a real contact. Slot-specific keys honour each gene's authored
-- earning policy while preserving Core.on_event dedup for other genes. Reaction,
-- reflection and projectile provenance never self-charge.
function methods:_charge_slot(run, host, slot, kind, move_id)
  local h = run.hosts[host]
  if not h then return 0 end
  local saved = h.slots
  local only = {}
  only[slot] = saved[slot]
  h.slots = only
  local pok, gained = pcall(self.Core.on_event, run, {host = host, kind = kind, move_id = move_id, lineage = 'direct'})
  h.slots = saved
  if not pok or type(gained) ~= 'number' then return 0 end
  return gained
end

function methods:on_event(event)
  local run = self:get_run()
  if not run or run.status ~= 'active' then return nil, 'run inactive' end
  if type(event) ~= 'table' or not name(event.host) or not name(event.move_id) then return nil, 'invalid event' end
  if event.reaction or (event.lineage and event.lineage ~= 'direct') then return 0 end
  if event.reflected or event.projectile then return 0 end
  local host_rec = run.hosts[event.host]
  if not host_rec then return 0 end
  local gained = 0
  for _, slot in ipairs({'assault', 'traversal', 'guard'}) do
    local id = host_rec.slots[slot]
    if id then
      local instance = run.genes[id]
      local gene = instance and self.B.genes[instance.kind]
      local spec = gene and gene.placements[slot]
      local earning = spec and spec.earning or 'per_move'
      local move_id = event.move_id
      if earning == 'per_target' and event.target then move_id = event.move_id .. '@' .. event.target end
      gained = gained + self:_charge_slot(run, event.host, slot, event.kind, move_id)
    end
  end
  return gained
end

-- Room leave: drop pending transactions and release every owned contribution.
-- A refused release is retained (bounded) and reported, never forgotten.
function methods:clear(clear_evidence)
  local keys = {}
  for k in pairs(self.records) do keys[#keys + 1] = k end
  table.sort(keys)
  local n = 0
  for _, k in ipairs(keys) do
    local rec = self.records[k]
    if rec then
      self:interrupt(rec.host, rec.slot, 'room leave')
      n = n + 1
    end
  end
  local run = self:get_run()
  local released, refused = 0, 0
  local serials = {}
  for serial in pairs(self.contributions) do serials[#serials + 1] = serial end
  table.sort(serials)
  for _, serial in ipairs(serials) do
    local c = self.contributions[serial]
    if c then
      local ok, err = self:release_contrib(c, c.run or run)
      if ok then
        self.contributions[serial] = nil
        released = released + 1
        self.stats.released = self.stats.released + 1
      else
        refused = refused + 1
        c.retry = (c.retry or 0) + 1
        if not c.stuck then c.stuck = true; self.stats.stuck = self.stats.stuck + 1 end
        self:emit({kind = 'contribution_stuck', serial = serial, reason = err})
      end
    end
  end
  -- All pending transactions are gone and owned handles released, so it is safe
  -- to drop the target map to keep the registry near one room's needs. The
  -- monotonic clock is preserved by Core, so a refund that races this reset
  -- still fails its stale revision instead of resurrecting a mark.
  if run then
    local reset = self.Core.reset_mark_revisions or self.Core.forget_run
    if type(reset) == 'function' then reset(run) end
  end
  self.events = {}
  if clear_evidence then self.settled = {} end
  return released, refused
end

function methods:on_room_leave() return self:clear(true) end

function methods:provenance()
  local out = {}
  for i, e in ipairs(self.settled) do out[i] = copy(e) end
  return out
end

function methods:contribution_count() return count(self.contributions) end

-- Settled handles plus pending contribution-kind reservations.
function methods:contribution_load()
  local n = count(self.contributions)
  for _, rec in pairs(self.records) do
    if contribution_kinds[rec.spec.effect.kind] then n = n + 1 end
  end
  return n
end

return GA
