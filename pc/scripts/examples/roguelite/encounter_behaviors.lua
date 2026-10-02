-- Gate 6 enemy controller layer: a pure observation -> decision -> legal
-- controller-input / ability-request boundary for custom Adventure actors and
-- fighter CPUs.
--
-- This module contains no engine calls. Callers inject:
--   * Core                     the real rules module (read-only Core.ability use)
--   * deps.get_run()           the active run, or nil
--   * tick(frame, observations) frames are monotonic; observations is a stable
--                              id -> observable-state table (see OBSERVABLE).
--   * deps.visible             optional occlusion/visibility predicate
--   * deps.actions             optional concurrent gene_actions transaction API
--   * deps.on_event            optional event sink
--
-- It returns requests for the host to execute. It never teleports, never reads
-- hidden player input, never grants charge, invulnerability or skip-lag, and
-- never fabricates a defeat. Observation is rate-limited by a real reaction
-- delay: decisions aim at the target pose the actor could see `reaction` frames
-- ago, not at the current frame.
--
-- The six archetypes are mechanically distinct data rows; the twelve
-- compositions are authored actor groups. Legal ability decisions use the real
-- Core ability state (readiness, reach, trigger, action), so this layer cannot
-- invent an engine API.
local Behaviors = {version = 1}

Behaviors.max_actors = 24
Behaviors.history = 32
Behaviors.memory = 120
Behaviors.pending_capacity = 64
Behaviors.pending_ttl = 600
Behaviors.anchor_margin = 20

-- Explicit host adapter contract. The observation is only as honest as the host
-- that fills it; these names say what each value must actually mean.
Behaviors.adapter_contract = {
  version = 1,
  self_kos = 'logical encounter losses/KOs attributed to this actor, tracked by the host; gd.player has no kos field',
  self_hits_taken = 'measured hits this actor received; a custom actor derives it from gd.enemy_state.received',
  self_guard_hit = 'observed successful guard/block event affecting this actor',
  custom_received = 'gd.enemy_state(handle).received is damage this actor received',
  custom_hits = 'gd.enemy_state(handle).hits is contacts this actor delivered, not received',
  target_attacking = 'target action/hitbox is a real attack; observable, never inferred from input',
  vertical = 'room y increases upward; a placement height of N is above the floor',
}

-- Only these fields cross the observation boundary. Anything else (input,
-- buttons, teleport, handles used as keys, ...) is dropped, not consulted.
Behaviors.OBSERVABLE = {
  frame = true, self = true, targets = true, bounds = true, terrain = true, events = true,
}
local SELF_FIELDS = {
  id = true, x = true, y = true, vx = true, vy = true, facing = true, grounded = true,
  airborne = true, on_stage = true, hitlag = true, action = true, percent = true,
  kos = true, hits_taken = true, guard_hit = true, alive = true, vulnerable = true,
}
local TARGET_FIELDS = {
  id = true, port = true, kind = true, x = true, y = true, vx = true, vy = true, facing = true,
  grounded = true, airborne = true, offstage = true, hitlag = true, action = true,
  attacking = true, vulnerable = true, percent = true,
}

local function finite(x) return type(x) == 'number' and x == x and math.abs(x) < math.huge end
local function integer(x, lo, hi) return finite(x) and x % 1 == 0 and x >= lo and x <= hi end
local function valid_id(x) return type(x) == 'string' and #x > 0 and #x <= 64 and not x:find('[%z\1-\31]') end
local function count(t) local n = 0 for _ in pairs(t) do n = n + 1 end return n end
local function copy(t) if type(t) ~= 'table' then return t end local o = {} for k, v in pairs(t) do o[k] = copy(v) end return o end
local function sorted(t) local a = {} for k in pairs(t) do a[#a + 1] = k end table.sort(a) return a end
local function sign(x) if x > 0 then return 1 elseif x < 0 then return -1 else return 0 end end

-- Deterministic stream. Same constants as Core's next_seed, so behaviour choice
-- is reproducible across platforms with doubles only.
function Behaviors.next_seed(x) return (x * 48271) % 2147483647 end
function Behaviors.rng(seed)
  local s = math.floor(seed or 1)
  if s < 1 then s = s + 2147483646 end
  s = s % 2147483647
  if s < 1 then s = 1 end
  return function()
    s = Behaviors.next_seed(s)
    return s / 2147483647
  end
end
function Behaviors.pick(seed, salt, n)
  if n <= 1 then return 1 end
  return (Behaviors.next_seed(((math.floor(seed or 1) + salt * 2654435761) % 2147483646) + 1) % n) + 1
end

-- ---------------------------------------------------------------- archetypes
-- Each row changes at least one of: movement policy, vision, reaction,
-- tell/recovery timing, ability slot choice, ledge policy and reaction trigger.
Behaviors.archetypes = {
  pressure = {
    id = 'pressure', host = 'custom', label = 'Pressure', motion = 'advance',
    band = {min = 1, max = 999}, close = 999, front_gate = false,
    vision_x = 160, vision_y = 48, reaction = 5, decision_interval = 5,
    tell_frames = 12, recovery_frames = 18, vertical = 14,
    attack_slots = {'assault'}, reaction_trigger = 'in_range',
    ledge = 'turn', fall = 'recover', run = true, gap_leap = true,
    distinct = 'relentless direct advance with a bounded gap leap',
  },
  guard = {
    id = 'guard', host = 'custom', label = 'Guard', motion = 'hold',
    band = {min = 0, max = 48}, close = 30, front_gate = false,
    vision_x = 120, vision_y = 40, reaction = 3, decision_interval = 6,
    tell_frames = 14, recovery_frames = 22, vertical = 12,
    attack_slots = {'assault'}, reaction_trigger = 'target_attack',
    ledge = 'climb', fall = 'despawn', run = false, gap_leap = false,
    distinct = 'holds a post and counter-attacks a committed target or a guarded hit',
  },
  zone = {
    id = 'zone', host = 'fighter', label = 'Zone', motion = 'spacing',
    band = {min = 24, max = 64}, close = 999, front_gate = true,
    vision_x = 200, vision_y = 56, reaction = 6, decision_interval = 6,
    tell_frames = 18, recovery_frames = 14, vertical = 14,
    attack_slots = {'assault'}, reaction_trigger = 'in_range',
    ledge = 'avoid', fall = 'recover', run = true, gap_leap = false,
    distinct = 'distance-band spacing that punishes approach and retreats when crowded',
  },
  aerial = {
    id = 'aerial', host = 'fighter', label = 'Aerial', motion = 'aerial',
    band = {min = 0, max = 160}, close = 999, front_gate = false,
    vision_x = 200, vision_y = 96, reaction = 7, decision_interval = 5,
    tell_frames = 16, recovery_frames = 20, vertical = 40,
    attack_slots = {'assault'}, reaction_trigger = 'target_airborne',
    ledge = 'recover', fall = 'recover', run = true, gap_leap = false,
    distinct = 'waits for an airborne or offstage target, then commits to a dive',
  },
  elite = {
    id = 'elite', host = 'custom', label = 'Elite', motion = 'alternate',
    band = {min = 1, max = 72}, close = 999, front_gate = false,
    vision_x = 170, vision_y = 52, reaction = 5, decision_interval = 7,
    tell_frames = 20, recovery_frames = 16, vertical = 14,
    attack_slots = {'assault', 'guard'}, reaction_trigger = 'alternate',
    ledge = 'turn', fall = 'recover', run = true, gap_leap = false, period = 60,
    distinct = 'alternates advance and hold stances and uses two ability slots',
  },
  boss = {
    id = 'boss', host = 'fighter', label = 'Champion', motion = 'phased',
    band = {min = 0, max = 200}, close = 999, front_gate = false,
    vision_x = 220, vision_y = 120, reaction = 6, decision_interval = 4,
    tell_frames = 24, recovery_frames = 20, vertical = 40,
    attack_slots = {'assault'}, reaction_trigger = 'phased',
    ledge = 'recover', fall = 'recover', run = true, gap_leap = false,
    distinct = 'phase machine delegated to a boss_behaviors controller',
  },
}

-- Twelve authored compositions. Lanes run -3..3 (left to right), height 0..2.
-- Each group is a stable id suffix so the host can derive `enemy_<room>_<serial>`
-- style stable ids; native handles never appear here.
Behaviors.compositions = {
  scout_pair = {id = 'scout_pair', archetype = 'pressure', difficulty = 1, actors = {
    {kind = 'goomba', role = 'custom', archetype = 'pressure', family = 'cinder', slot = 'assault', level = 3, count = 2, lane = -1, height = 0},
  }},
  pincer_pair = {id = 'pincer_pair', archetype = 'pressure', difficulty = 2, actors = {
    {kind = 'goomba', role = 'custom', archetype = 'pressure', family = 'cinder', slot = 'assault', level = 5, count = 2, lane = 0, height = 0},
    {kind = 'redead', role = 'custom', archetype = 'guard', family = 'rime', slot = 'assault', level = 4, count = 1, lane = 2, height = 1},
  }},
  guard_post = {id = 'guard_post', archetype = 'guard', difficulty = 1, actors = {
    {kind = 'redead', role = 'custom', archetype = 'guard', family = 'rime', slot = 'assault', level = 4, count = 1, lane = 0, height = 0},
  }},
  shield_wall = {id = 'shield_wall', archetype = 'guard', difficulty = 2, actors = {
    {kind = 'redead', role = 'custom', archetype = 'guard', family = 'rime', slot = 'assault', level = 5, count = 2, lane = -2, height = 0},
  }},
  zoner_wall = {id = 'zoner_wall', archetype = 'zone', difficulty = 2, actors = {
    {kind = 'goomba', role = 'custom', archetype = 'zone', family = 'cinder', slot = 'assault', level = 5, count = 3, lane = 1, height = 0},
  }},
  skyline_denial = {id = 'skyline_denial', archetype = 'zone', difficulty = 3, actors = {
    {kind = 'fighter', role = 'fighter', archetype = 'zone', family = 'cinder', slot = 'assault', level = 6, count = 1, lane = 1, height = 0},
    {kind = 'goomba', role = 'custom', archetype = 'pressure', family = 'cinder', slot = 'assault', level = 6, count = 2, lane = -1, height = 1},
  }},
  aerial_duel = {id = 'aerial_duel', archetype = 'aerial', difficulty = 2, actors = {
    {kind = 'fighter', role = 'fighter', archetype = 'aerial', family = 'rime', slot = 'assault', level = 6, count = 1, lane = 0, height = 0},
  }},
  recovery_hunt = {id = 'recovery_hunt', archetype = 'aerial', difficulty = 3, actors = {
    {kind = 'fighter', role = 'fighter', archetype = 'aerial', family = 'cinder', slot = 'assault', level = 7, count = 1, lane = 0, height = 0},
    {kind = 'goomba', role = 'custom', archetype = 'pressure', family = 'cinder', slot = 'assault', level = 5, count = 1, lane = -2, height = 0},
  }},
  elite_mix = {id = 'elite_mix', archetype = 'elite', difficulty = 3, actors = {
    {kind = 'redead', role = 'custom', archetype = 'elite', family = 'rime', slot = 'guard', level = 7, count = 1, lane = 1, height = 0},
    {kind = 'goomba', role = 'custom', archetype = 'pressure', family = 'cinder', slot = 'assault', level = 5, count = 1, lane = -1, height = 0},
  }},
  elite_twin = {id = 'elite_twin', archetype = 'elite', difficulty = 4, actors = {
    {kind = 'fighter', role = 'fighter', archetype = 'zone', family = 'cinder', slot = 'assault', level = 8, count = 1, lane = 1, height = 0},
    {kind = 'redead', role = 'custom', archetype = 'elite', family = 'rime', slot = 'guard', level = 7, count = 1, lane = -1, height = 1},
  }},
  swarm_rush = {id = 'swarm_rush', archetype = 'pressure', difficulty = 3, actors = {
    {kind = 'goomba', role = 'custom', archetype = 'pressure', family = 'cinder', slot = 'assault', level = 5, count = 4, lane = -2, height = 0},
  }},
  bastion_hold = {id = 'bastion_hold', archetype = 'guard', difficulty = 3, actors = {
    {kind = 'redead', role = 'custom', archetype = 'guard', family = 'rime', slot = 'assault', level = 6, count = 2, lane = -1, height = 0},
    {kind = 'goomba', role = 'custom', archetype = 'pressure', family = 'cinder', slot = 'assault', level = 4, count = 1, lane = 2, height = 0},
  }},
}

-- Map a lane/height hint inside a room bound to a concrete spawn point. Arena
-- actors are placed by the host; this is a deterministic suggestion only.
function Behaviors.placement(bounds, lane, height)
  if type(bounds) ~= 'table' then return nil end
  local center = finite(bounds.center_x) and bounds.center_x or ((bounds.min_x + bounds.max_x) / 2)
  local floor_y = finite(bounds.floor_y) and bounds.floor_y or bounds.max_y
  lane = integer(lane, -3, 3) and lane or 0
  height = integer(height, 0, 2) and height or 0
  return center + lane * 32, floor_y + height * 24
end

-- --------------------------------------------------------------- observation
local function snapshot_target(t, frame)
  local o = {frame = frame}
  for k in pairs(TARGET_FIELDS) do o[k] = t[k] end
  return o
end

local function sanitize_side(src, fields)
  if type(src) ~= 'table' then return nil, 1 end
  local out, dropped = {}, 0
  for k, v in pairs(src) do
    if fields[k] then out[k] = v else dropped = dropped + 1 end
  end
  return out, dropped
end

-- Returns a clean observation or nil,reason, plus the number of hidden fields
-- that were ignored. Only observable target/self state survives. Malformed
-- input is refused or dropped safely: it never raises and never admits a target
-- without a stable id and a finite position.
function Behaviors.sanitize_obs(raw)
  if type(raw) ~= 'table' then return nil, 'invalid observation', 0 end
  local out, dropped = {}, 0
  for k, v in pairs(raw) do
    if Behaviors.OBSERVABLE[k] then out[k] = v else dropped = dropped + 1 end
  end
  local self_out, self_dropped = sanitize_side(raw.self, SELF_FIELDS)
  dropped = dropped + (self_dropped or 0)
  out.self = self_out
  local targets = {}
  if type(raw.targets) == 'table' then
    for _, t in ipairs(raw.targets) do
      local clean, more = sanitize_side(t, TARGET_FIELDS)
      dropped = dropped + (more or 0)
      if clean and valid_id(clean.id) and finite(clean.x) and finite(clean.y) then
        targets[#targets + 1] = clean
      end
    end
  elseif raw.targets ~= nil then
    dropped = dropped + 1
  end
  out.targets = targets
  if not out.self or not finite(out.self.x) or not finite(out.self.y) then return nil, 'invalid self state', dropped end
  return out, nil, dropped
end

-- Surface a failed/denied callback instead of swallowing it. The decision layer
-- fails closed: a thrown visibility/permission/action callback never becomes an
-- implicit yes.
function Behaviors:surface(kind, agent, detail)
  if agent then agent.counters.surfaced = agent.counters.surfaced + 1 end
  self.agg.surfaced = self.agg.surfaced + 1
  local ev = {kind = kind, actor = agent and agent.id or nil, detail = detail}
  self:emit(ev)
  return ev
end

-- True only when the target is inside the vision box, in front where required
-- and not blocked by the injected visibility predicate. A configured predicate
-- must return exactly `true` to grant visibility; `false`, `nil`, a thrown
-- error or any other value fails closed and is never an implicit yes. With no
-- predicate configured, visibility follows the explicit observed-target
-- semantics above.
function Behaviors:visibility(agent, target, selfpose, frame)
  if not finite(target.x) or not finite(target.y) then return false end
  local dx, dy = target.x - selfpose.x, target.y - selfpose.y
  if math.abs(dx) > agent.vision_x or math.abs(dy) > agent.vision_y then return false end
  local front = agent.front_gate
  if front == nil then front = agent.spec and agent.spec.front_gate end
  if front and dx * (selfpose.facing or 1) < -2 then return false end
  if self.visible then
    local ok, result = pcall(self.visible, agent, target, selfpose, frame)
    if not ok then self:surface('visible_error', agent, tostring(result)); return false end
    if result ~= true then return false end
  end
  return true
end

-- Store only poses the actor can actually observe this frame and mark the
-- currently observable set. An occluded target loses its history immediately,
-- so a long-gone target can never be attacked from stale memory; a target that
-- leaves the observation entirely is forgotten too.
function Behaviors:record_history(agent, obs, frame)
  local selfpose = obs.self
  agent.visible_now = {}
  local present = {}
  for _, t in ipairs(obs.targets or {}) do
    present[t.id] = true
    if self:visibility(agent, t, selfpose, frame) then
      agent.visible_now[t.id] = true
      agent.last_seen = agent.last_seen or {}
      agent.last_seen[t.id] = frame
      local h = agent.hist[t.id]
      if not h then h = {}; agent.hist[t.id] = h end
      if #h == 0 or h[#h].frame ~= frame then
        h[#h + 1] = snapshot_target(t, frame)
        while #h > Behaviors.history do table.remove(h, 1) end
      end
    elseif agent.hist[t.id] then
      -- LINE-OF-SIGHT FAILURE INVALIDATES THE CANDIDATE even if history exists.
      agent.hist[t.id] = nil
      if agent.last_seen then agent.last_seen[t.id] = nil end
    end
  end
  for id in pairs(agent.hist) do
    if not present[id] then agent.hist[id] = nil; if agent.last_seen then agent.last_seen[id] = nil end end
  end
end

-- The pose the actor can actually have seen `reaction` frames ago, within the
-- bounded memory window. nil until the history spans the delay, which prevents
-- instantaneous tracking, and nil once the memory is stale.
function Behaviors.delayed_pose(agent, tid, frame, reaction)
  local h = agent.hist[tid]
  if not h then return nil end
  for i = #h, 1, -1 do
    if h[i].frame <= frame - reaction then
      if frame - h[i].frame > Behaviors.memory then return nil end
      return h[i]
    end
  end
  return nil
end

-- ------------------------------------------------------------------- arbiter
-- Prevents simultaneous overlapping tells with no response window: at most
-- max_active tells globally, and (when one at a time) a mandatory gap between
-- the end of a tell and the next one.
local Arbiter = {}
Arbiter.__index = Arbiter
function Arbiter:expire(frame)
  for id, until_ in pairs(self.active) do
    if until_ <= frame then self.active[id] = nil; self.count = self.count - 1; self.last_end = math.max(self.last_end, until_) end
  end
end
function Arbiter:can(frame)
  if self.count >= self.max_active then return false end
  if self.max_active == 1 and frame < self.last_end + self.gap then return false end
  return true
end
function Arbiter:acquire(id, frame, until_)
  if self.active[id] then return true end
  if not self:can(frame) then return false end
  self.active[id] = until_
  self.count = self.count + 1
  return true
end
function Arbiter:release(id, frame)
  if self.active[id] then
    self.active[id] = nil; self.count = self.count - 1
    if frame then self.last_end = math.max(self.last_end, frame) end
  end
end
function Behaviors.new_arbiter(max_active, gap)
  return setmetatable({max_active = max_active or 1, gap = gap or 8, active = {}, count = 0, last_end = -math.huge}, Arbiter)
end

-- -------------------------------------------------------------------- manager
function Behaviors.new(Core, deps, opts)
  assert(type(Core) == 'table' and type(Core.ability) == 'function', 'Core module with Core.ability required')
  deps, opts = deps or {}, opts or {}
  assert(deps.get_run == nil or type(deps.get_run) == 'function', 'invalid get_run')
  if deps.actions ~= nil then
    assert(type(deps.actions) == 'table', 'actions callback must be a table')
    assert(deps.actions.available == nil or type(deps.actions.available) == 'function', 'invalid actions.available')
    assert(deps.actions.submit == nil or type(deps.actions.submit) == 'function', 'invalid actions.submit')
  end
  assert(deps.permission == nil or type(deps.permission) == 'function', 'invalid permission callback')
  local self = setmetatable({
    Core = Core, deps = deps, get_run = deps.get_run, on_event = deps.on_event,
    visible = deps.visible, actions = deps.actions, permission = deps.permission,
    max_actors = opts.max_actors or Behaviors.max_actors,
    frame = 0, order = {}, by_id = {}, pending = {}, hidden_ignored = 0, next_generation = 1,
    agg = {opportunities = 0, requests = 0, successes = 0, tells = 0, deferred = 0,
      refused = 0, failed = 0, denied = 0, surfaced = 0, ineligible = 0},
  }, {__index = Behaviors})
  self.arbiter = Behaviors.new_arbiter(opts.max_simultaneous_tells, opts.tell_gap)
  return self
end

function Behaviors:count()
  return #self.order
end

function Behaviors:add(actor)
  if type(actor) ~= 'table' then return nil, 'invalid actor' end
  if not valid_id(actor.id) then return nil, 'actor needs a stable id' end
  if self.by_id[actor.id] then return nil, 'duplicate actor id' end
  if #self.order >= self.max_actors then return nil, 'actor budget exhausted' end
  local brain = actor.brain
  local archetype = actor.archetype
  local spec = archetype and Behaviors.archetypes[archetype]
  if not brain then
    if not spec then return nil, 'unknown archetype ' .. tostring(archetype) end
  end
  if actor.host ~= nil and not valid_id(actor.host) then return nil, 'invalid host' end
  local seed = actor.seed
  if seed ~= nil and not integer(seed, 1, 2147483646) then return nil, 'invalid seed' end
  local reaction = actor.reaction or (spec and spec.reaction) or 6
  if not integer(reaction, 0, 30) then return nil, 'invalid reaction delay' end
  local interval = actor.decision_interval or (spec and spec.decision_interval) or 6
  if not integer(interval, 1, 30) then return nil, 'invalid decision interval' end
  local agent = {
    id = actor.id, handle = actor.handle, host = actor.host, slot = actor.slot,
    family = actor.family, archetype = archetype, spec = spec, brain = brain,
    level = actor.level, seed = seed or 1, reaction = reaction, decision_interval = interval,
    vision_x = actor.vision_x or (spec and spec.vision_x) or 160,
    vision_y = actor.vision_y or (spec and spec.vision_y) or 64,
    capability = actor.capability or ((spec and spec.motion == 'aerial') and 'aerial' or 'grounded'),
    spawn = actor.spawn, lane = actor.lane, height = actor.height,
    generation = self.next_generation,
    stance = 'idle', tell_until = nil, tell_ready = false, recovery_until = nil,
    next_decision = 0, last_frame = 0, serial = 0, cond = {}, hist = {},
    phase = actor.phase, phase_start = nil, phase_choices = nil, attack_choice = nil,
    motion = nil, band = nil, vertical = nil, front_gate = nil, anchor_x = nil,
    tell_frames = actor.tell_frames or (spec and spec.tell_frames) or 16,
    recovery_frames = actor.recovery_frames or (spec and spec.recovery_frames) or 18,
    vulnerable_until = nil, vulnerable_reason = nil,
    defeated = false, escaped = false, active = true,
    counters = {
      ability = {opportunities = 0, requests = 0, successes = 0},
      recovery = {opportunities = 0, requests = 0, successes = 0},
      locomotion = {opportunities = 0, requests = 0, successes = 0},
      tells = 0, deferred = 0, refused = 0, failed = 0, denied = 0,
      ineligible = 0, surfaced = 0, hidden_ignored = 0,
    },
  }
  self.next_generation = self.next_generation + 1
  self.by_id[agent.id] = agent
  self.order[#self.order + 1] = agent.id
  return self:view(agent)
end

function Behaviors:remove(id, reason)
  local agent = self.by_id[id]
  if not agent then return true, {kind = 'ignored', actor = id, reason = 'unknown actor'} end
  reason = reason or 'cleanup'
  self.arbiter:release(id)
  -- Purge this incarnation's confirmable requests so a re-added actor with the
  -- same stable id cannot inherit an old success.
  for move_id, p in pairs(self.pending) do
    if p.actor == id and p.gen == agent.generation then self.pending[move_id] = nil end
  end
  agent.active = false
  if reason == 'defeated' then agent.defeated = true
  elseif reason == 'escaped' then agent.escaped = true end
  self.by_id[id] = nil
  for i = #self.order, 1, -1 do if self.order[i] == id then table.remove(self.order, i) end end
  local ev = {kind = reason == 'defeated' and 'defeat' or (reason == 'escaped' and 'escaped' or 'release'),
    actor = id, generation = agent.generation, escaped = agent.escaped, defeated = agent.defeated}
  self:emit(ev)
  return true, ev
end

function Behaviors:has_live()
  for _, id in ipairs(self.order) do
    local a = self.by_id[id]
    if a and a.active and not a.defeated and not a.escaped then return true, id end
  end
  return false
end

-- Cleanup every owned decision agent. It never reports a defeat.
function Behaviors:release()
  local removed = {}
  for _, id in ipairs(copy(self.order)) do
    removed[#removed + 1] = id
    self:remove(id, 'cleanup')
  end
  return removed
end

function Behaviors:view(agent)
  local spec = agent.spec
  return {
    id = agent.id, handle = agent.handle, host = agent.host, slot = agent.slot,
    family = agent.family, archetype = agent.archetype, level = agent.level,
    motion = agent.motion or (spec and spec.motion), stance = agent.stance,
    phase = agent.phase, tell_until = agent.tell_until, tell_ready = agent.tell_ready,
    recovery_until = agent.recovery_until,
    vulnerable = agent.vulnerable_until ~= nil and self.frame < agent.vulnerable_until,
    vulnerable_reason = agent.vulnerable_reason,
    defeated = agent.defeated, escaped = agent.escaped, active = agent.active,
  }
end

function Behaviors:state(id)
  local a = self.by_id[id]
  if not a then return nil end
  return self:view(a)
end

function Behaviors:states()
  local out = {}
  for _, id in ipairs(self.order) do out[#out + 1] = self:view(self.by_id[id]) end
  return out
end

function Behaviors:emit(ev)
  if self.on_event then self.on_event(ev) end
end

-- ------------------------------------------------------------- legality reads
function Behaviors:ability(agent, slot)
  local run = self.get_run and self.get_run()
  if not run or not agent.host then return nil end
  local ok, a = pcall(self.Core.ability, run, agent.host, slot)
  if ok and type(a) == 'table' then return a end
  return nil
end

-- The exact gene objects currently equipped in a host/slot, plus the exact owned
-- run table. A tell binds these table identities (never textual ids, which Core
-- reuses per run) and refuses to release against a different instance.
-- Returns gene_id (string), gene (table), state (table), run (table).
function Behaviors:gene_instance(agent, slot)
  local run = self.get_run and self.get_run()
  if not run then return nil, nil, nil, nil end
  local host = run.hosts and run.hosts[agent.host]
  local gene_id = host and host.slots and host.slots[slot] or nil
  local gene = gene_id and run.genes and run.genes[gene_id] or nil
  local state = host and host.state and host.state[slot] or nil
  return gene_id, gene, state, run
end

-- A delayed pose came from a frame where the target was visible, so only the
-- vision box and front gate need re-checking against the actor's current pose.
function Behaviors:visible_target(agent, target, pose, selfpose)
  if not pose then return false end
  local dx, dy = pose.x - selfpose.x, pose.y - selfpose.y
  if math.abs(dx) > agent.vision_x or math.abs(dy) > agent.vision_y then return false end
  local spec = agent.spec
  local front = agent.front_gate
  if front == nil then front = spec and spec.front_gate end
  if front and dx * (selfpose.facing or 1) < -2 then return false end
  return true
end

function Behaviors:select_target(agent, obs, frame)
  local selfpose = obs.self
  local best, best_pose, best_d
  for _, t in ipairs(obs.targets or {}) do
    -- The target must be observable THIS frame; a lost line of sight invalidates
    -- it even if a delayed pose is still in memory.
    if agent.visible_now and agent.visible_now[t.id] then
      local pose = Behaviors.delayed_pose(agent, t.id, frame, agent.reaction)
      if pose and self:visible_target(agent, t, pose, selfpose) then
        local d = (pose.x - selfpose.x) ^ 2 + (pose.y - selfpose.y) ^ 2
        if not best_d or d < best_d then best, best_pose, best_d = t, pose, d end
      end
    end
  end
  return best, best_pose
end

function Behaviors:in_reach(agent, selfpose, pose, a)
  if not a or not pose then return false end
  local spec = agent.spec
  local vertical = agent.vertical or (spec and spec.vertical) or 14
  return math.abs(pose.x - selfpose.x) <= a.reach and math.abs(pose.y - selfpose.y) <= vertical
end

-- Reaction-delayed condition edge for reactive triggers. Returns true only after
-- the condition has been continuously observed for `reaction` frames.
function Behaviors:edge(agent, tid, tag, cond, frame)
  local key = tid .. ':' .. tag
  if not cond then agent.cond[key] = nil; return false end
  if agent.cond[key] == nil then agent.cond[key] = frame end
  return frame - agent.cond[key] >= agent.reaction
end

function Behaviors:condition_delayed(agent, selfpose, pose, frame)
  local trigger = (agent.spec and agent.spec.reaction_trigger) or 'in_range'
  if trigger == 'in_range' or trigger == 'alternate' or trigger == 'phased' then return true end
  if trigger == 'target_attack' then
    return self:edge(agent, pose.id, 'attack', pose.attacking == true, frame)
  end
  if trigger == 'target_airborne' then
    return self:edge(agent, pose.id, 'airborne', pose.airborne == true or pose.offstage == true, frame)
  end
  if trigger == 'guard_hit' then
    return self:edge(agent, pose.id, 'guard', (selfpose.guard_hit or 0) > 0, frame)
  end
  return true
end

function Behaviors:choose_slot(agent, obs, target)
  local choices = agent.phase_choices or (agent.spec and agent.spec.attack_slots) or {agent.slot}
  if #choices == 0 then return agent.slot end
  -- A slot the host has not equipped has no Core ability and can never be a
  -- legal request, so it is not a candidate.
  local equipped = {}
  for _, s in ipairs(choices) do if self:ability(agent, s) then equipped[#equipped + 1] = s end end
  if #equipped == 0 then return choices[1] end
  choices = equipped
  local policy = agent.attack_choice or 'single'
  if policy == 'single' then return choices[1] end
  if policy == 'alternate' then
    local stance = self:stance(agent, obs.frame or self.frame)
    if stance == 'advance' then return choices[1] end
    return choices[2] or choices[1]
  end
  local ready = {}
  for _, s in ipairs(choices) do
    local a = self:ability(agent, s)
    if a and a.ready then ready[#ready + 1] = s end
  end
  if #ready == 0 then return choices[1] end
  local idx = Behaviors.pick(agent.seed + agent.serial * 7 + (agent.phase or 1) * 101, 17, #ready)
  return ready[idx]
end

function Behaviors:stance(agent, frame)
  local spec = agent.spec
  local period = (spec and spec.period) or 60
  local start = agent.phase_start or 0
  if math.floor((frame - start) / period) % 2 == 0 then return 'advance' end
  return 'hold'
end

function Behaviors:count_opportunity(agent, slot, target, frame)
  local key = slot .. ':' .. target.id
  if agent.opportunity_key ~= key then
    agent.opportunity_key = key
    agent.counters.ability.opportunities = agent.counters.ability.opportunities + 1
    self.agg.opportunities = self.agg.opportunities + 1
  end
end

-- --------------------------------------------------------------- dispersal
-- Permission and the optional action transaction run before anything becomes
-- confirmable. The protocol is explicit: a configured callback must return
-- exactly `true` to permit/accept. `false`, `nil` or a thrown error is a
-- denial/failure, is surfaced and leaves no pending entry, so a failed action
-- can never later "succeed".
function Behaviors:allowed(agent, request)
  if self.permission then
    local ok, result = pcall(self.permission, agent, request)
    if not ok then self:surface('permission_error', agent, tostring(result)); return false, 'permission error' end
    if result ~= true then return false, result == false and 'denied' or 'denied nil' end
  end
  if self.actions and self.actions.available then
    local ok, result = pcall(self.actions.available, request)
    if not ok then self:surface('available_error', agent, tostring(result)); return false, 'available error' end
    if result ~= true then return false, result == false and 'refused' or 'refused nil' end
  end
  return true
end

function Behaviors:dispatch(agent, request)
  local ok, why = self:allowed(agent, request)
  if not ok then
    request.refused = true
    request.reason = why
    local field = (why == 'denied' or why == 'denied nil' or why == 'permission error') and 'denied' or 'refused'
    agent.counters[field] = agent.counters[field] + 1
    self.agg[field] = self.agg[field] + 1
    self:surface(field, agent, why)
    return false
  end
  if self.actions and self.actions.submit then
    local submitted, result = pcall(self.actions.submit, request)
    if not submitted or result ~= true then
      request.failed = true
      request.reason = submitted and (result == false and 'submit refused' or 'submit nil') or tostring(result)
      agent.counters.failed = agent.counters.failed + 1
      self.agg.failed = self.agg.failed + 1
      self:surface('failed', agent, request.reason)
      return false
    end
  end
  if count(self.pending) >= Behaviors.pending_capacity then
    local oldest_id, oldest
    for k, p in pairs(self.pending) do if not oldest or p.frame < oldest then oldest_id, oldest = k, p.frame end end
    if oldest_id then self.pending[oldest_id] = nil end
  end
  self.pending[request.move_id] = {actor = agent.id, gen = agent.generation, frame = self.frame, slot = request.slot}
  self:emit({kind = 'request', actor = agent.id, move_id = request.move_id, slot = request.slot})
  return true
end

-- Host reports the outcome of a previously emitted request. Success is only
-- counted here; the decision layer never claims a hit it did not observe. A
-- stale move_id from an earlier actor incarnation has no pending entry.
function Behaviors:confirm(move_id, success)
  local p = self.pending and self.pending[move_id]
  if not p then return false end
  local agent = self.by_id[p.actor]
  if not agent or agent.generation ~= p.gen then
    self.pending[move_id] = nil
    return false
  end
  if success then
    agent.counters.ability.successes = agent.counters.ability.successes + 1
    self.agg.successes = self.agg.successes + 1
  end
  if not success and agent.vulnerable_on_whiff then
    agent.vulnerable_until = self.frame + (agent.vulnerable_on_whiff)
    agent.vulnerable_reason = 'whiff'
    self:emit({kind = 'vulnerable', actor = agent.id, reason = 'whiff', until_frame = agent.vulnerable_until})
  end
  p.success = success and true or false
  self.pending[move_id] = nil
  return true
end

function Behaviors:counters(id)
  if id then
    local a = self.by_id[id]
    if not a then return nil end
    return copy(a.counters)
  end
  local total = {opportunities = 0, requests = 0, successes = 0, tells = 0, deferred = 0,
    refused = 0, failed = 0, denied = 0, surfaced = 0, ineligible = 0, hidden_ignored = self.hidden_ignored}
  for _, id in ipairs(self.order) do
    local c = self.by_id[id].counters
    total.opportunities = total.opportunities + c.ability.opportunities + c.recovery.opportunities
    total.requests = total.requests + c.ability.requests + c.recovery.requests
    total.successes = total.successes + c.ability.successes + c.recovery.successes
    total.tells = total.tells + c.tells
    total.deferred = total.deferred + c.deferred
    total.refused = total.refused + c.refused
    total.failed = total.failed + c.failed
    total.denied = total.denied + c.denied
    total.ineligible = total.ineligible + c.ineligible
    total.surfaced = total.surfaced + c.surfaced
  end
  return total
end

-- ------------------------------------------------------------- locomotion
function Behaviors:ledge_blocked(agent, obs, selfpose, dir)
  if dir == 0 then return false end
  local spec = agent.spec
  local policy = (spec and spec.ledge) or 'turn'
  if policy == 'climb' or policy == 'recover' then return false end
  local b = obs.bounds
  if b then
    local step = 8
    if selfpose.x + dir * step < b.min_x + 8 or selfpose.x + dir * step > b.max_x - 8 then return true end
  end
  for _, ledge in ipairs((obs.terrain and obs.terrain.ledges) or {}) do
    if finite(ledge.x) then
      local ahead = (ledge.x - selfpose.x) * dir
      if ahead > 0 and ahead <= 8 then return true end
      if ledge.side == 'right' and dir > 0 and ahead >= 0 and ahead <= 8 then return true end
      if ledge.side == 'left' and dir < 0 and ahead >= 0 and ahead <= 8 then return true end
    end
  end
  return false
end

function Behaviors:recover_dir(agent, obs, selfpose)
  local b = obs.bounds
  local center = b and (finite(b.center_x) and b.center_x or ((b.min_x + b.max_x) / 2))
  if not center then return 0 end
  return sign(center - selfpose.x)
end

function Behaviors:locomotion(agent, obs, selfpose, target, pose, slot, recovering)
  local spec = agent.spec
  local motion = agent.motion or (spec and spec.motion) or 'advance'
  local band = agent.band or (spec and spec.band) or {min = 1, max = 999}
  local dir, intent = 0, 'wait'
  local function toward(x) return sign(x - selfpose.x) end
  if not target or not pose then
    local home = agent.spawn and agent.spawn.x
    local cx = obs.bounds and (finite(obs.bounds.center_x) and obs.bounds.center_x or ((obs.bounds.min_x + obs.bounds.max_x) / 2))
    local anchor = home or cx or selfpose.x
    if math.abs(anchor - selfpose.x) > 12 then dir = toward(anchor); intent = 'patrol' else dir = 0; intent = 'wait' end
  else
    local dist = math.abs(pose.x - selfpose.x)
    if motion == 'advance' then
      dir = toward(pose.x); intent = 'approach'
    elseif motion == 'hold' then
      if dist < band.max then dir = 0; intent = 'guard' else dir = toward(pose.x); intent = 'reposition' end
    elseif motion == 'spacing' then
      if dist < band.min then dir = -toward(pose.x); intent = 'retreat'
      elseif dist > band.max then dir = toward(pose.x); intent = 'approach'
      else dir = 0; intent = 'space' end
    elseif motion == 'aerial' then
      if pose.airborne or pose.offstage then dir = 0; intent = 'hover'
      else dir = toward(pose.x); intent = 'track' end
    elseif motion == 'alternate' then
      if self:stance(agent, obs.frame or self.frame) == 'advance' then dir = toward(pose.x); intent = 'approach'
      elseif dist < band.max then dir = 0; intent = 'guard'
      else dir = toward(pose.x); intent = 'reposition' end
    else
      dir = toward(pose.x); intent = 'track'
    end
    -- A held post does not walk itself off the stage chasing a target it cannot
    -- respect; the ledge check applies to every grounded policy below.
    -- A phase positioning demand is a real requested movement, not only idle
    -- patrol: away from its anchor and not point-blank, the actor moves back
    -- toward the demanded position. This is a horizontal request only; the host
    -- decides whether the capability supports it.
    if agent.anchor_x and not recovering then
      local adx = agent.anchor_x - selfpose.x
      local target_dist = pose and math.abs(pose.x - selfpose.x) or nil
      if math.abs(adx) > Behaviors.anchor_margin and (not target_dist or target_dist > band.min) then
        dir = sign(adx); intent = 'position'
      end
    end
  end
  dir = self:avoid(agent, obs, selfpose, dir)
  agent.counters.locomotion.requests = agent.counters.locomotion.requests + 1
  return {kind = 'move', actor = agent.id, dir = dir, run = dir ~= 0 and (spec == nil or spec.run ~= false),
    jump = false, intent = intent, slot = slot, capability = agent.capability, recovering = recovering and true or false}
end

function Behaviors:avoid(agent, obs, selfpose, dir)
  if dir == 0 or not selfpose.grounded then return dir end
  if self:ledge_blocked(agent, obs, selfpose, dir) then return 0 end
  return dir
end

function Behaviors:make_ability_request(agent, a, target, pose, slot)
  return {
    kind = 'ability', actor = agent.id, host = agent.host, slot = slot or agent.slot, family = agent.family,
    generation = agent.generation,
    target = target and target.id, target_kind = target and target.kind, target_port = target and target.port,
    move_id = agent.id .. '#' .. agent.generation .. ':' .. agent.slot .. ':' .. tostring(agent.serial),
    reach = a.reach, damage = a.damage, action = a.action, trigger = a.trigger,
    reason = 'legal_opportunity', tell_frames = agent.tell_frames,
  }
end

-- ------------------------------------------------------------------ decision
local function find_target(obs, id)
  for _, t in ipairs(obs.targets or {}) do if t.id == id then return t end end
  return nil
end

-- Drop every field of an advertised tell, including the bound object identity.
local function clear_tell(agent)
  agent.tell_until, agent.tell_ready = nil, false
  agent.tell_target, agent.tell_slot, agent.tell_move = nil, nil, nil
  agent.tell_gene, agent.tell_gene_id, agent.tell_state, agent.tell_run = nil, nil, nil, nil
  agent.tell_action, agent.tell_family, agent.tell_trigger = nil, nil, nil
  agent.attack_phase = 'idle'
end

function Behaviors:archetype_decide(agent, obs, frame)
  local events = {}
  local Core, run = self.Core, self.get_run and self.get_run()
  if not obs or not obs.self then return nil, events end
  local selfpose = obs.self
  -- Dead or hit-stunned actors are not eligible to act. Any advertised tell is
  -- cancelled rather than released.
  if selfpose.alive == false or (selfpose.hitlag or 0) > 0 then
    agent.counters.ineligible = agent.counters.ineligible + 1
    self.agg.ineligible = self.agg.ineligible + 1
    if agent.tell_until then
      self.arbiter:release(agent.id, frame)
      events[#events + 1] = {kind = 'tell_abort', actor = agent.id, reason = 'ineligible'}
      clear_tell(agent)
    end
    return nil, events
  end
  -- A tell release keeps the exact target/slot/instance/gene/state/action it
  -- advertised, by TABLE IDENTITY. Core reuses textual ids ('run1', 'r4') across
  -- runs, so an id comparison is not enough; the exact owned run table, gene
  -- table, host-slot state table and original action must all still match. On any
  -- drift the tell cancels and may retell, without spending anything.
  if agent.tell_until and frame >= agent.tell_until then
    self.arbiter:release(agent.id, frame)
    local tid, slot = agent.tell_target, agent.tell_slot
    local target = tid and find_target(obs, tid)
    local pose = tid and Behaviors.delayed_pose(agent, tid, frame, agent.reaction)
    local a = slot and self:ability(agent, slot)
    local current_gene_id, current_gene, current_state, current_run = self:gene_instance(agent, slot)
    local same_run = current_run ~= nil and current_run == agent.tell_run
    local same_gene = current_gene ~= nil and current_gene == agent.tell_gene
    local same_state = current_state ~= nil and current_state == agent.tell_state
    local same_action = a ~= nil and a.slot == slot and a.action == agent.tell_action
      and a.family == agent.tell_family and a.trigger == agent.tell_trigger
    local why
    if not same_run then why = 'run_changed'
    elseif not same_gene then why = 'gene_changed'
    elseif not same_state then why = 'state_changed'
    elseif not same_action then why = 'action_changed'
    elseif not (agent.tell_ready and target and pose and a and a.ready and self:in_reach(agent, selfpose, pose, a)) then why = 'invalid'
    end
    if not why then
      agent.recovery_until = frame + agent.recovery_frames
      agent.attack_phase = 'recovery'
      local req = self:make_ability_request(agent, a, target, pose, slot)
      req.move_id = agent.tell_move
      req.gene = current_gene
      req.gene_id = current_gene_id
      agent.counters.ability.requests = agent.counters.ability.requests + 1
      self.agg.requests = self.agg.requests + 1
      if agent.vulnerable_after then
        agent.vulnerable_until = frame + agent.vulnerable_after
        agent.vulnerable_reason = 'recovery'
        events[#events + 1] = {kind = 'vulnerable', actor = agent.id, reason = 'recovery', until_frame = agent.vulnerable_until}
      end
      self:dispatch(agent, req)
      clear_tell(agent)
      return req, events
    end
    events[#events + 1] = {kind = 'tell_abort', actor = agent.id, target = tid, slot = slot, reason = why}
    clear_tell(agent)
  end
  if agent.tell_until and frame < agent.tell_until then return nil, events end

  local recovering = agent.recovery_until ~= nil and frame < agent.recovery_until
  if recovering then agent.attack_phase = 'recovery' elseif agent.attack_phase ~= 'startup' then agent.attack_phase = 'idle' end
  -- off-stage recovery is a real legal-input request and takes priority
  local b = obs.bounds
  local offstage = selfpose.on_stage == false or (b and selfpose.y < b.min_y)
  if offstage then
    agent.counters.recovery.opportunities = agent.counters.recovery.opportunities + 1
    self.agg.opportunities = self.agg.opportunities + 1
    local dir = self:recover_dir(agent, obs, selfpose)
    if dir ~= 0 then
      agent.counters.recovery.requests = agent.counters.recovery.requests + 1
      self.agg.requests = self.agg.requests + 1
      return {kind = 'recover', actor = agent.id, dir = dir, jump = true,
        capability = agent.capability, reason = 'offstage'}, events
    end
    return nil, events
  end

  local target, pose = self:select_target(agent, obs, frame)
  local slot = self:choose_slot(agent, obs, target)
  local a = self:ability(agent, slot)
  local opportunity = target and pose and a and a.ready and self:in_reach(agent, selfpose, pose, a)
    and self:condition_delayed(agent, selfpose, pose, frame)
  if opportunity then self:count_opportunity(agent, slot, target, frame) else agent.opportunity_key = nil end

  if recovering then
    return self:locomotion(agent, obs, selfpose, target, pose, slot, true), events
  end
  if frame < (agent.next_decision or 0) then
    return self:locomotion(agent, obs, selfpose, target, pose, slot, false), events
  end
  agent.next_decision = frame + agent.decision_interval
  if opportunity and self.arbiter:can(frame) then
    local until_ = frame + agent.tell_frames
    if self.arbiter:acquire(agent.id, frame, until_) then
      agent.serial = agent.serial + 1
      agent.tell_until, agent.tell_ready = until_, true
      agent.tell_target, agent.tell_slot = target.id, slot
      agent.tell_move = agent.id .. '#' .. agent.generation .. ':' .. slot .. ':' .. agent.serial
      local gene_id, gene, state, tell_run = self:gene_instance(agent, slot)
      agent.tell_gene, agent.tell_gene_id = gene, gene_id
      agent.tell_state, agent.tell_run = state, tell_run
      agent.tell_action = a and a.action
      agent.tell_family = a and a.family
      agent.tell_trigger = a and a.trigger
      agent.stance = 'tell'
      agent.attack_phase = 'startup'
      agent.counters.tells = agent.counters.tells + 1
      self.agg.tells = self.agg.tells + 1
      events[#events + 1] = {kind = 'tell', actor = agent.id, slot = slot, target = target.id,
        gene = agent.tell_gene_id, gene_id = agent.tell_gene_id, action = agent.tell_action,
        generation = agent.generation, move_id = agent.tell_move,
        tell_frames = agent.tell_frames, until_frame = until_}
      return nil, events
    end
  elseif opportunity then
    agent.counters.deferred = agent.counters.deferred + 1
    self.agg.deferred = self.agg.deferred + 1
  end
  return self:locomotion(agent, obs, selfpose, target, pose, slot, false), events
end

function Behaviors:decide(agent, obs, frame)
  if agent.brain then return agent.brain(agent, obs, frame, self) end
  return self:archetype_decide(agent, obs, frame)
end

function Behaviors:tick(frame, observations)
  if not integer(frame, 0, 1000000000) then return nil, 'invalid decision frame' end
  if frame < self.frame then return nil, 'non-monotonic decision time' end
  self.frame = frame
  self.arbiter:expire(frame)
  local requests, events = {}, {}
  for _, id in ipairs(self.order) do
    local agent = self.by_id[id]
    local raw = observations and observations[id]
    local obs, why, dropped = nil, nil, 0
    if raw then obs, why, dropped = Behaviors.sanitize_obs(raw) end
    if raw and not obs then
      -- A malformed observation is refused: no currently observable target, and
      -- no invisible history is retained.
      events[#events + 1] = {kind = 'observation_refused', actor = id, reason = why}
      agent.visible_now = {}
      agent.hist = {}
      agent.last_seen = {}
    elseif obs then
      if dropped > 0 then
        agent.counters.hidden_ignored = agent.counters.hidden_ignored + dropped
        self.hidden_ignored = self.hidden_ignored + dropped
      end
      self:record_history(agent, obs, frame)
    else
      -- No observation means no currently observable target; stale memory is
      -- never a candidate.
      agent.visible_now = {}
    end
    local req, evs = self:decide(agent, obs, frame)
    if req then requests[#requests + 1] = req end
    for _, e in ipairs(evs or {}) do events[#events + 1] = e end
  end
  for move_id, p in pairs(self.pending) do
    if frame - p.frame > Behaviors.pending_ttl then self.pending[move_id] = nil end
  end
  return true, {requests = requests, events = events}
end

-- --------------------------------------------------------------- validation
-- Structural validation of the authored data against the real catalogues.
function Behaviors.validate(Core, EnemyCatalogue, EncounterCatalogue)
  if type(Core) ~= 'table' or type(Core.definitions) ~= 'table' then return false, 'Core definitions required' end
  local seen, n = {}, 0
  for id, spec in pairs(Behaviors.archetypes) do
    n = n + 1
    if id ~= spec.id or (spec.host ~= 'custom' and spec.host ~= 'fighter') then return false, 'invalid archetype ' .. tostring(id) end
    if type(spec.motion) ~= 'string' or type(spec.attack_slots) ~= 'table' or #spec.attack_slots < 1 then return false, 'archetype lacks mechanics' end
    if not integer(spec.tell_frames, 4, 60) or not integer(spec.recovery_frames, 1, 60) then return false, 'invalid tell/recovery' end
    if not integer(spec.reaction, 0, 30) or not integer(spec.vision_x, 1, 400) then return false, 'invalid vision/reaction' end
    for _, slot in ipairs(spec.attack_slots) do
      if not ({assault = true, traversal = true, guard = true})[slot] then return false, 'invalid slot ' .. tostring(slot) end
    end
    seen[id] = true
  end
  if n ~= 6 then return false, 'exactly six archetypes required' end
  local comps = 0
  for id, comp in pairs(Behaviors.compositions) do
    comps = comps + 1
    if comp.id ~= id or not seen[comp.archetype] then return false, 'invalid composition ' .. tostring(id) end
    if type(comp.actors) ~= 'table' or #comp.actors == 0 then return false, 'empty composition ' .. id end
    for _, row in ipairs(comp.actors) do
      if not seen[row.archetype] then return false, 'unknown row archetype' end
      if not integer(row.count, 1, 8) or not integer(row.level, 1, 9) then return false, 'invalid row count/level' end
      if not Core.definitions[row.family] or not Core.definitions[row.family].variants[row.slot] then
        return false, 'unsupported Core family/slot ' .. tostring(row.family) .. '/' .. tostring(row.slot)
      end
      if row.role == 'custom' then
        if not ({goomba = true, redead = true})[row.kind] then return false, 'unsupported custom kind ' .. tostring(row.kind) end
      elseif row.role == 'fighter' then
        if row.kind ~= 'fighter' and row.kind ~= 'champion' then return false, 'unsupported fighter kind' end
      else return false, 'invalid actor role' end
    end
    if EncounterCatalogue and EncounterCatalogue.encounters and EncounterCatalogue.encounters[id] then
      local live = EncounterCatalogue.encounters[id]
      local total = 0
      for _, row in ipairs(comp.actors) do total = total + row.count end
      local live_total = 0
      for _, row in ipairs(live.enemies) do live_total = live_total + row.count end
      if total ~= live_total then return false, 'composition actor count disagrees with catalogue for ' .. id end
    end
  end
  if comps ~= 12 then return false, 'exactly twelve compositions required' end
  return true
end

return Behaviors
