-- Common resolved ability transaction for genes (Gate 5).
--
-- Pure orchestration over the real Core charge/spend APIs. It never calls an
-- engine global and never invents a gd.* function: native application arrives
-- through an explicit `world` adapter. When a required adapter callback is
-- absent, the transaction refuses instead of pretending the hit happened.
--
--   preflight -> startup -> active -> recovery -> settled
--
-- The clock is run.frame only (bounded, deterministic). Charge is spent once at
-- the release/spend point; a refusal before release costs nothing and a refusal
-- at/after release restores the pre-spend Core snapshot when the spec allows it.
--
-- This module deliberately does not make prototype families shippable: by
-- default only Behaviors.admitted_defaults() (cinder/rime) can run. Tests and
-- future admission pass an explicit opts.admitted override.
local GA = {version = 1}
GA.caps = {records = 96, hosts = 32, lifetime = 600, provenance = 128}
GA.slots = {assault = true, traversal = true, guard = true}
GA.phase_order = {'preflight', 'startup', 'active', 'recovery', 'settled'}

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

local function observe(self, host)
  if type(self.world.observe) ~= 'function' then return nil end
  local v = self.world.observe(host)
  return type(v) == 'table' and v or nil
end

-- Pure geometric target policy: horizontal reach, vertical band and facing arc.
local function in_policy(spec, ability, host_view, target)
  if type(host_view) ~= 'table' then return false, 'no host observation' end
  local reach = ability.reach
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
    'remove_modifier', 'snapshot', 'restore'}) do
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
  self.stats = {started = 0, settled = 0, interrupted = 0, refused = 0, restored = 0}
  return self
end

function methods:emit(event)
  self.events[#self.events + 1] = event
end

function methods:get_run()
  return type(self.world.get_run) == 'function' and self.world.get_run() or nil
end

function methods:observe(host) return observe(self, host) end

-- Read-only preflight. Returns a plan table or nil, reason. Never mutates.
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
  if type(self.world.can_start) == 'function' then
    local ok, reason = self.world.can_start(run, host, slot)
    if ok ~= true then return nil, reason or 'not free to act' end
  end
  local seam = seam_for(spec)
  if seam and type(self.world[seam]) ~= 'function' then
    return nil, 'native ' .. seam .. ' seam missing'
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
    local ok, reason = in_policy(spec, ability, host_view, target)
    if not ok then return nil, reason end
    if spec.occlusion == 'line' and type(self.world.occluded) == 'function' and self.world.occluded(host, target) then
      return nil, 'target occluded'
    end
    -- A reaction against the source's own status is refused before spending.
    if opts and opts.reaction and target.host == host then return nil, 'self reaction refused' end
  end
  return {host = host, slot = slot, key = key(host, slot), gene = gene, id = id, spec = spec,
    ability = copy(ability), route = self.B.mechanics[spec.mechanic].route,
    target = target and copy(target) or nil}
end

function methods:begin(host, slot, opts)
  local plan, reason = self:preflight(host, slot, opts)
  if not plan then
    self.stats.refused = self.stats.refused + 1
    return nil, reason
  end
  if count(self.records) >= self.caps.records then return nil, 'action record cap' end
  local run = self:get_run()
  self.serial = self.serial + 1
  local move_id = host .. ':' .. tostring(run.frame) .. ':' .. slot .. ':' .. tostring(self.serial)
  local rec = {host = plan.host, slot = plan.slot, key = plan.key, gene = plan.gene, gene_id = plan.gene.id,
    id = plan.id, spec = plan.spec, route = plan.route, ability = plan.ability,
    target = plan.target, phase = 'startup', start_frame = run.frame,
    release_frame = run.frame + plan.spec.startup, active_until = nil, recovery_until = nil,
    move_id = move_id, serial = self.serial, spent = false, applied = false, contrib = nil,
    reaction = nil, lineage = nil, refunded = false}
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
    host_view = self:observe(rec.host)}
end

function methods:view_record(rec)
  return {host = rec.host, slot = rec.slot, gene = rec.gene_id, action = rec.spec.action,
    mechanic = rec.spec.mechanic, kind = rec.spec.effect.kind, route = rec.route, phase = rec.phase,
    move_id = rec.move_id, serial = rec.serial, start_frame = rec.start_frame,
    release_frame = rec.release_frame, spent = rec.spent, applied = rec.applied,
    refunded = rec.refunded, target = rec.target and rec.target.host or nil,
    reaction = rec.reaction, lineage = rec.lineage}
end

function methods:view()
  local keys = {}
  for k in pairs(self.records) do keys[#keys + 1] = k end
  table.sort(keys)
  local out = {}
  for _, k in ipairs(keys) do out[#out + 1] = self:view_record(self.records[k]) end
  return out
end

-- Restore the caller's run table in place so the pre-spend state is real for
-- every holder of that reference. A world.replace_run adapter may swap instead.
function methods:restore_run(run, text)
  local restored = self.Core.restore(text)
  if not restored then return false end
  if type(self.world.replace_run) == 'function' then self.world.replace_run(restored) return true end
  for k in pairs(run) do run[k] = nil end
  for k, v in pairs(restored) do run[k] = v end
  return true
end

function methods:release_contrib(rec, run)
  if not rec.contrib then return end
  local kind = rec.contrib.kind
  local release = self.world['release_' .. kind] or self.world.release
  if type(release) == 'function' then release(run, rec.contrib) end
  rec.contrib = nil
end

function methods:interrupt(host, slot, reason)
  local k = key(host, slot)
  local rec = self.records[k]
  if not rec then return nil, 'no action' end
  self:release_contrib(rec, self:get_run())
  self.records[k] = nil
  self.stats.interrupted = self.stats.interrupted + 1
  self:emit({kind = 'interrupted', host = host, slot = slot, move_id = rec.move_id,
    reason = reason, refunded = rec.refunded, spent = rec.spent})
  return {status = 'interrupted', host = host, slot = slot, move_id = rec.move_id,
    reason = reason, refunded = rec.refunded, spent = rec.spent}
end

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
  if spec.occlusion == 'line' and type(self.world.occluded) == 'function' and self.world.occluded(rec.host, fresh) then
    return false, 'target occluded'
  end
  rec.target = copy(fresh)
  return true
end

function methods:apply(rec, run, action)
  local spec = rec.spec
  local kind = spec.effect.kind
  if kind == 'mark' and spec.native then return true, nil end
  local seam = seam_for(spec)
  local fn = seam and self.world[seam]
  if type(fn) ~= 'function' then return false, 'native ' .. tostring(seam) .. ' seam missing' end
  local ok, detail = fn(run, self:request(rec, action))
  if not ok then return false, type(detail) == 'string' and detail or 'effect refused' end
  if type(detail) == 'table' and detail.contrib then rec.contrib = detail.contrib end
  return true, detail
end

function methods:release(rec, run)
  local spec = rec.spec
  local valid, why = self:refresh_target(rec, run)
  if not valid then self:interrupt(rec.host, rec.slot, why or 'target escaped') return end
  local snapshot = spec.refund == 'on_refuse' and self.Core.snapshot(run) or nil
  local action, kind = self.Core.activate(run, rec.host, rec.slot, {target = rec.target and rec.target.host})
  if not action then
    self:interrupt(rec.host, rec.slot, kind or 'spend refused')
    return
  end
  rec.spent = true
  rec.reaction = action.reaction
  rec.lineage = action.lineage
  local applied, detail = self:apply(rec, run, action)
  if not applied then
    if snapshot then
      if self:restore_run(run, snapshot) then
        rec.refunded = true
        self.stats.restored = self.stats.restored + 1
      end
    end
    self:interrupt(rec.host, rec.slot, detail or 'effect refused')
    return
  end
  rec.applied = true
  rec.phase = 'active'
  rec.active_until = run.frame + spec.active
  self:emit({kind = 'active', host = rec.host, slot = rec.slot, move_id = rec.move_id,
    gene = rec.gene_id, action = action.action, reaction = rec.reaction, frame = run.frame})
end

function methods:settle(rec, run)
  local entry = {serial = rec.serial, source = rec.host, slot = rec.slot, gene = rec.gene_id,
    id = rec.id, action = rec.spec.action, mechanic = rec.spec.mechanic, kind = rec.spec.effect.kind,
    route = rec.route, move_id = rec.move_id, target = rec.target and rec.target.host or nil,
    reaction = rec.reaction, lineage = rec.lineage, spent = rec.spent, applied = rec.applied,
    frame = run and run.frame or nil}
  if rec.contrib then self.contributions[rec.serial] = rec.contrib end
  self.settled[#self.settled + 1] = entry
  if #self.settled > self.caps.provenance then table.remove(self.settled, 1) end
  self.records[rec.key] = nil
  self.stats.settled = self.stats.settled + 1
  self:emit({kind = 'settled', host = rec.host, slot = rec.slot, move_id = rec.move_id,
    provenance = copy(entry)})
end

function methods:step(rec, run)
  if not run or run.status ~= 'active' then self:interrupt(rec.host, rec.slot, 'run inactive') return end
  local age = run.frame - rec.start_frame
  if not integer(age) or age < 0 or age > self.caps.lifetime then
    self:interrupt(rec.host, rec.slot, 'clock bounds')
    return
  end
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

-- Advance every active transaction one observation. Must be called once after
-- Core.tick on each unpaused logic frame.
function methods:advance()
  self.events = {}
  local run = self:get_run()
  local keys = {}
  for k in pairs(self.records) do keys[#keys + 1] = k end
  table.sort(keys)
  for _, k in ipairs(keys) do
    local rec = self.records[k]
    if rec then self:step(rec, run) end
  end
  local out = self.events
  self.events = {}
  return out
end

-- Charge from a real contact. Reaction lineage never charges (no self-trigger),
-- and per_target genes key the move by target using Core's existing move_id.
function methods:on_event(event)
  local run = self:get_run()
  if not run or run.status ~= 'active' then return nil, 'run inactive' end
  if type(event) ~= 'table' or not name(event.host) or not name(event.move_id) then return nil, 'invalid event' end
  if event.reaction or (event.lineage and event.lineage ~= 'direct') then return 0 end
  local forwarded = copy(event)
  local host_rec = run.hosts[event.host]
  if event.target and host_rec then
    for _, slot in ipairs({'assault', 'traversal', 'guard'}) do
      local id = host_rec.slots[slot]
      if id and run.genes[id] then
        local gene = self.B.genes[run.genes[id].kind]
        local spec = gene and gene.placements[slot]
        if spec and spec.earning == 'per_target' then
          forwarded.move_id = event.move_id .. '@' .. event.target
          break
        end
      end
    end
  end
  return self.Core.on_event(run, forwarded)
end

-- Room leave: drop pending transactions and release owned reversible state.
-- Keeps the settled provenance log for audit unless clear_evidence is set.
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
  for serial, contrib in pairs(self.contributions) do
    local release = self.world['release_' .. contrib.kind] or self.world.release
    if type(release) == 'function' then release(run, contrib) end
    self.contributions[serial] = nil
  end
  self.events = {}
  if clear_evidence then self.settled = {} end
  return n
end

function methods:on_room_leave() return self:clear(true) end

function methods:provenance()
  local out = {}
  for i, e in ipairs(self.settled) do out[i] = copy(e) end
  return out
end

return GA
