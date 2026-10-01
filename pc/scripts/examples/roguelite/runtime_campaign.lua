-- Live v2 campaign controller. It owns no pure-save format and no native handle
-- of its own beyond what it delegates; every engine effect goes through injected
-- collaborators:
--
--   gd        engine table (stage/teleport/player/spawn_enemy/...)
--   deps.Core, deps.Progress, deps.GeneCatalogue, deps.RouteMap
--   deps.Rooms                          the room module
--   deps.RuntimeRooms/Encounters/Rewards reviewed helper classes
--   deps.EnemyGenes, EnemyCatalogue, TechAI
--   deps.routes                         the Route service (travel/collect/validate)
--
-- Persistence is injected as callbacks so this module never writes disk:
--   opts.route / opts.run     the exact route/run this instance owns for life;
--                             every read, mutation and orchestrator callback is
--                             bound to these, never a module-level global.
--   opts.save_route(route,progress,run) -> true|false,reason
--       validates the pair and atomically writes a TBD3 generation WITHOUT
--       mutating the canonical route; only a success updates the route/run.
--   opts.finish(outcome)                main's run-finish transaction
--   opts.say(text,kind,key)             compact feedback
--   opts.stock_baseline                 native stock display kept for the CPU port
--
-- Correct ordering is the point of this module:
--   * a request never allocates. `request_travel`/`enter_current` only stage the
--     pure Route transaction and move to `requested`; the engine is paused by
--     main BEFORE tick() calls RuntimeRooms:begin/build/place. The pause lasts
--     through persist, commit, settling, flushing, recovery and effect rollback.
--   * a failed build/save/placement is not abandoned: the RuntimeRooms
--     transaction and its ownership stay in `self.tx` while a bounded recovery
--     retries rollback so the destination is destroyed and the fighters are
--     restored before the source room resumes.
--   * the source encounter is kept owned until a durable commit, then cleared
--     before the destination's own encounter starts, so a refused travel does
--     not lose the fight and a new combat room can begin.
--   * tick() performs exactly one bounded phase per call and loading yields one
--     asset per call, so a single hook never serializes a floor build, a durable
--     save, a destructive commit and an enemy spawn inside one 50ms window.
--   * the CPU stock display stays at the native baseline; only an observed
--     decrease is treated as exactly one KO. `_start_encounter` never writes a
--     live `remaining` into the engine.
--   * a failed progress save keeps the newest in-memory record, mirrors it, and
--     blocks on a bounded paused retry. A later successful change replaces the
--     pending record, so an old generation can never be written after a newer one.
--   * reward native effects are applied with pcall and readback verification;
--     a failed save rolls them back (retrying) without re-snapshotting the live
--     run, which would orphan live enemy host/gene references.
local RuntimeCampaign = {version = 2}

local function copy(t)
  if type(t) ~= 'table' then return t end
  local out = {}
  for k, v in pairs(t) do out[k] = copy(v) end
  return out
end
local function count(t) local n = 0 for _ in pairs(t) do n = n + 1 end return n end

local COMBAT = {combat = true, boss = true}
local SAFE_REWARD = {reward = true, rest = true}

function RuntimeCampaign.new(gd, deps, opts)
  assert(type(gd) == 'table', 'engine table required')
  opts = opts or {}
  for _, k in ipairs({'Core', 'Progress', 'RouteMap', 'Rooms', 'RuntimeRooms',
    'RuntimeEncounters', 'RuntimeRewards', 'EnemyGenes', 'EnemyCatalogue', 'TechAI', 'routes', 'Encounters'}) do
    assert(deps and deps[k], 'campaign dependency missing: ' .. k)
  end
  assert(type(opts.route) == 'table' and type(opts.run) == 'table'
    and type(opts.save_route) == 'function', 'campaign route/run/persistence required')
  local self = setmetatable({
    gd = gd, Core = deps.Core, Progress = deps.Progress, RouteMap = deps.RouteMap, Rooms = deps.Rooms,
    routes = deps.routes, encounters_table = deps.Encounters,
    -- Identity binding: this campaign and its orchestrator only ever read and
    -- write the exact route/run they were created with, so a later run
    -- promotion can never redirect an old owner's teardown at the new run.
    route = opts.route, run = opts.run, save_route = opts.save_route,
    finish_run = opts.finish, say = opts.say or function() end,
    on_event = opts.on_event, fighter_family = opts.fighter_family or 'cinder',
    fighter_slot = opts.fighter_slot or 'assault', spawn_point = opts.spawn_point,
    stock_baseline = opts.stock_baseline or 99,
    max_rollback_attempts = opts.max_rollback_attempts or 8,
    max_save_attempts = opts.max_save_attempts or 8,
    max_effect_attempts = opts.max_effect_attempts or 8,
    phase = 'idle', error_msg = nil, notice = nil,
    pending = nil, tx = nil, staged = nil, dest_node = nil,
    pending_save = nil, effect_pending = nil, rollback_attempts = 0,
    locked = false, lock_pos = nil, encounter_node = nil, encounter_pending = false,
    encounter_active = false, reward_room = nil,
  }, {__index = RuntimeCampaign})
  self.rooms = deps.RuntimeRooms.new(gd, deps.Rooms, opts)
  self.encounters = deps.RuntimeEncounters.new(deps.Core, gd, {
    EnemyGenes = deps.EnemyGenes, EnemyCatalogue = deps.EnemyCatalogue,
    TechAI = deps.TechAI, Progress = deps.Progress,
  }, {
    get_run = function() return self.run end, on_event = opts.on_event,
    fighter_family = self.fighter_family,
    fighter_slot = self.fighter_slot, buff_fighters = opts.buff_fighters,
    champion_potency = opts.champion_potency, spawn_point = opts.spawn_point,
    bounds = opts.bounds, max_actors = opts.max_actors,
  })
  self.rewards = deps.RuntimeRewards.new(deps.Core, {
    Progress = deps.Progress, EncounterCatalogue = deps.Encounters,
    GeneCatalogue = deps.GeneCatalogue,
  })
  return self
end

local function route_of(self) return self.route end
local function node_of(self, id)
  local route = route_of(self)
  return route and route.view and route.view.nodes[id]
end
function RuntimeCampaign:manifest() return self.route and self.route.manifest end
function RuntimeCampaign:progress() return self.route and self.route.progress end
function RuntimeCampaign:view() return self.route and self.route.view end
function RuntimeCampaign:current_node() local p = self:progress() return p and node_of(self, p.current_room) end
function RuntimeCampaign:active_room() return self.rooms:active_room() end

-- ---------------------------------------------------------------------------
-- Phase predicates
-- ---------------------------------------------------------------------------
-- `blocked` is the single gate main uses to keep the engine paused and gameplay
-- frozen: any paused transition, a bounded pending save or a pending effect
-- rollback. `running` is the only state in which main drives Core/physics.
function RuntimeCampaign:transitioning()
  return self.phase == 'requested' or self.phase == 'loading' or self.phase == 'placing'
    or self.phase == 'persisting' or self.phase == 'committing' or self.phase == 'settling'
    or self.phase == 'recovering'
end
function RuntimeCampaign:blocked()
  -- A terminal error surfaces through `failed` instead, so main can show the
  -- recovery menu rather than pausing silently forever.
  if self.phase == 'error' then return false end
  return self:transitioning() or self.pending_save ~= nil or self.effect_pending ~= nil
end
function RuntimeCampaign:failed() return self.phase == 'error' end
function RuntimeCampaign:running()
  return self.phase == 'active' and not self:blocked()
end
function RuntimeCampaign:take_notice()
  local n = self.notice
  self.notice = nil
  return n
end

-- Which deferred native effects this engine can actually apply and undo. Charge
-- has no documented native API, so it is honestly unavailable; specs needing it
-- are refused by the resolver rather than advertised.
function RuntimeCampaign:effect_support()
  local heal = type(self.gd.set_percent) == 'function' and type(self.gd.player) == 'function'
  return { version = 1, heal = heal, charge = false }
end

-- ---------------------------------------------------------------------------
-- Requests (never allocate; main pauses before tick builds)
-- ---------------------------------------------------------------------------
function RuntimeCampaign:_stage_exit(exit)
  local route = route_of(self)
  if not route then return nil, 'no active route' end
  local staged, why = self.routes:travel({ manifest = route.manifest, progress = route.progress }, exit)
  if not staged then return nil, why end
  local holder = { manifest = route.manifest, progress = staged }
  local ok, cwhy = self.routes:collect(holder)
  if not ok then return nil, cwhy end
  return holder.progress
end

function RuntimeCampaign:_stage_enter()
  local staged = copy(self:progress())
  local holder = { manifest = self:manifest(), progress = staged }
  local ok, why = self.routes:collect(holder)
  if not ok then return nil, why end
  return holder.progress
end

-- Enter the saved current room. Only stages the request; tick performs the build.
function RuntimeCampaign:enter_current()
  if self.phase ~= 'idle' and self.phase ~= 'error' then return nil, 'campaign already entering' end
  local p = self:progress()
  if not (p and node_of(self, p.current_room)) then return nil, 'no current room' end
  if self.error_msg then return nil, self.error_msg end
  self.pending = { kind = 'enter' }
  self.phase = 'requested'
  return true
end

-- Deliberate exit request. Pure staging only: a one-way/unsatisfied gate is
-- refused here; the physical transaction waits for a paused tick.
function RuntimeCampaign:request_travel(exit)
  if self.phase ~= 'active' then return nil, 'transition already in progress' end
  local node = self:current_node()
  local p = self:progress()
  if not node then return nil, 'no current room' end
  -- A combat objective must be legitimately fulfilled; a direct request cannot
  -- bypass the encounter.
  if COMBAT[node.role] and p.objectives[node.id] ~= 'done' then return nil, 'encounter not complete' end
  local dest = node_of(self, exit and exit.to)
  if not dest then return nil, 'unknown destination' end
  local staged, why = self:_stage_exit(exit)
  if not staged then return nil, why end
  self.pending = { kind = 'travel', exit = exit, staged = staged }
  self.phase = 'requested'
  return true
end

-- ---------------------------------------------------------------------------
-- Paused transition machine
-- ---------------------------------------------------------------------------
function RuntimeCampaign:_begin_pending()
  local req = self.pending
  self.pending = nil
  if not req then self.phase = 'active'; return end
  if req.kind == 'enter' then
    local node = self:current_node()
    if not node then return self:_fail_request('no current room') end
    local staged, why = self:_stage_enter()
    if not staged then return self:_fail_request(why) end
    local tx, twhy = self.rooms:begin(node)
    if not tx then return self:_fail_request(twhy) end
    self.tx, self.staged, self.dest_node, self.phase = tx, staged, node, 'loading'
  else
    local dest = node_of(self, req.exit.to)
    local tx, twhy = self.rooms:begin(dest, { from = self.rooms:active_room(), exit = req.exit })
    if not tx then return self:_fail_request(twhy) end
    self.tx, self.staged, self.dest_node, self.phase = tx, req.staged, dest, 'loading'
  end
end

-- A request or begin refusal left no transaction; stay in the usable source.
function RuntimeCampaign:_fail_request(why)
  self.pending, self.tx, self.staged, self.dest_node = nil, nil, nil, nil
  self.notice = tostring(why)
  if self.rooms:active_room() then
    self.phase = 'active'
  else
    self.phase = 'error'; self.error_msg = tostring(why)
  end
end

-- A failure once a transaction exists enters bounded recovery: fighters are
-- restored and the destination destroyed before the source resumes. Ownership is
-- retained (self.tx) until RuntimeRooms:rollback actually succeeds.
function RuntimeCampaign:_fail_transition(why)
  self.error_msg = tostring(why)
  self.phase = 'recovering'
  self.rollback_attempts = 0
  self:attempt_recovery()
end

function RuntimeCampaign:attempt_recovery()
  if not self.tx then
    self.phase = self.rooms:active_room() and 'active' or 'error'
    return
  end
  local cleaned, cwhy = self.rooms:rollback(self.tx)
  if cleaned then
    self.tx, self.staged, self.dest_node = nil, nil, nil
    self.notice = self.error_msg
    self.error_msg = nil
    self.rollback_attempts = 0
    self.phase = self.rooms:active_room() and 'active' or 'error'
    return
  end
  self.rollback_attempts = self.rollback_attempts + 1
  if self.rollback_attempts >= self.max_rollback_attempts then
    self.error_msg = 'transition cleanup refused after retries: ' .. tostring(cwhy)
    self.phase = 'error'
  end
end

function RuntimeCampaign:_persist_staged()
  local ok, why = self:_save(self.staged, nil)
  if not ok then
    self:_fail_transition('transition save refused: ' .. tostring(why))
    return false
  end
  return true
end

-- Every successful durable write makes any queued pending record obsolete, so a
-- later write can never resurrect an older generation. Persisted against this
-- instance's own route; a run_override (e.g. a menu edit or reward) is committed
-- into the instance only on a durable write so identity and disk stay coherent.
function RuntimeCampaign:_save(progress, run_override)
  local ok, why = self.save_route(self.route, progress, run_override or self.run)
  if ok then
    self.route.progress = progress
    if run_override then self.run = run_override end
    self.pending_save = nil
    return true
  end
  return false, why
end

-- Rebind this (current) campaign to a run table produced by a real Core
-- transaction (menu place/unequip/fuse/reward or a refusal restore). Only ever
-- called on the live current campaign; retired owners keep their own run.
function RuntimeCampaign:replace_run(new_run)
  if type(new_run) ~= 'table' or new_run.type ~= 'run' then return nil, 'invalid run' end
  self.run = new_run
  if self.route and self.route.progress then self:_mirror(self.route.progress) end
  return true
end

-- One bounded phase per callback, so a single engine hook never serializes a
-- whole floor build, a durable save, a destructive commit and an enemy spawn in
-- one 50ms budget window. Loading still yields one asset at a time.
function RuntimeCampaign:tick()
  if self.effect_pending then self:retry_effects(); return end
  if self.pending_save then self:_flush_pending_save(); return end
  if self.phase == 'recovering' then self:attempt_recovery(); return end
  if self.phase == 'error' then return end
  if self.phase == 'requested' then self:_begin_pending(); return end
  if self.phase == 'loading' then
    local st, why = self.rooms:step(self.tx)
    if st == nil then return self:_fail_transition(why) end
    if st == 'ready' then self.phase = 'placing' end
    return
  end
  if self.phase == 'placing' then
    local ok, why = self:_place()
    if not ok then return self:_fail_transition('fighter placement refused: ' .. tostring(why)) end
    if not self.staged then self.staged = copy(self:progress()) end
    self.phase = 'persisting'
    return
  end
  if self.phase == 'persisting' then
    if not self:_persist_staged() then return end
    self.phase = 'committing'
    return
  end
  if self.phase == 'committing' then
    local outcome, reason = self.rooms:commit(self.tx)
    if not outcome then return self:_fail_transition(tostring(reason or 'room commit refused')) end
    self.tx, self.staged, self.dest_node = nil, nil, nil
    self.locked = true
    self.lock_pos = { x = outcome.arrival and outcome.arrival.x, y = outcome.arrival and outcome.arrival.y }
    self.phase = 'settling'
    return
  end
  if self.phase == 'settling' then self:_settle(); return end
end

-- The destination arrival: the exit's destination-socket arrival on a travel,
-- else the room's authored arrival/spawn. The destination node drives this, so a
-- travel never resolves the source room's spawn (p2 placement bug).
function RuntimeCampaign:_arrival()
  local tx = self.tx
  if tx and tx.info then
    local a = self.rooms:arrival(tx)
    if a then return a end
  end
  local node = self.dest_node or self:current_node()
  local p = self:progress()
  if not node then return { x = 0, y = 2, facing = 1 } end
  local socket = p and p.current_socket
  local a = socket and self.Rooms.arrival(node, socket) or nil
  return a or self.Rooms.arrival(node)
end

function RuntimeCampaign:_place()
  local a = self:_arrival()
  local placements = { { port = 1, x = a.x, y = a.y } }
  local node = self.dest_node or self:current_node()
  local encounter = node and node.encounter_spec
  if encounter then
    local spawn = (self.spawn_point and self.spawn_point(node)) or { x = 28, y = 2 }
    placements[#placements + 1] = { port = 2, x = spawn.x, y = spawn.y }
  else
    placements[#placements + 1] = { port = 2, x = 58, y = 2 }
  end
  return self.rooms:place(self.tx, placements)
end

-- ---------------------------------------------------------------------------
-- Settling: source cleanup, encounter handover, objective/reward
-- ---------------------------------------------------------------------------
function RuntimeCampaign:_configure_encounter_engine()
  local e = self.encounters:active_entity()
  if e then
    -- Keep the native stock display at the baseline; logical remaining lives in
    -- the orchestrator and is decremented only by observed KO evidence.
    if self.gd.set_stocks then self.gd.set_stocks(2, self.stock_baseline) end
    if self.gd.cpu_mode then self.gd.cpu_mode(2, 'fight') end
  elseif self.gd.cpu_mode then
    self.gd.cpu_mode(2, 'stand')
  end
end

function RuntimeCampaign:_settle()
  if self.rooms:pending() then
    local ok, why = self.rooms:flush()
    if not ok then self.error_msg = 'source cleanup pending: ' .. tostring(why); return end
  end
  -- Retire whatever the encounter orchestrator still owns before beginning
  -- another. A cleared encounter keeps the orchestrator active with no live
  -- wave, so this is required, not just when campaign bookkeeping says active.
  -- A refused clear stays owned and blocks progress until it succeeds.
  local cok, cwhy = self.encounters:clear()
  if not cok then
    self.encounter_pending = true
    self.error_msg = 'encounter cleanup pending: ' .. tostring(cwhy)
    return
  end
  self.encounter_pending, self.encounter_active, self.encounter_node = false, false, nil
  local node = self:current_node()
  if not node then self.phase = 'error'; self.error_msg = 'no current room'; return end
  if not COMBAT[node.role] then self:mark_objective(node) end
  if self.pending_save then return end
  if node.encounter_spec and self:progress().objectives[node.id] ~= 'done' then
    local ok, events = self.encounters:begin(node, self:progress())
    if not ok then
      -- Fail closed: the objective stays incomplete; bounded cleanup then a
      -- paused, explicit error rather than a silent softlock.
      self.encounters:clear()
      self.phase = 'error'
      self.error_msg = 'encounter unavailable: ' .. tostring(events)
      return
    end
    self.encounter_active, self.encounter_node = true, node.id
    self:_configure_encounter_engine()
  end
  if node.role == 'finish' and self:progress().objectives[node.id] == 'done' then
    if self.finish_run then self.finish_run('success') end
  end
  if node.reward_spec and not self:progress().claimed['reward:' .. node.id] then
    if SAFE_REWARD[node.role] or (COMBAT[node.role] and self:progress().objectives[node.id] == 'done') then
      self.reward_room = node.id
    end
  end
  self.phase = 'active'
  self.error_msg = nil
  self.notice = self.notice or (node.title or node.id)
  if self.on_event then self.on_event({ kind = 'entered', room = node.id }) end
end

-- ---------------------------------------------------------------------------
-- Objective / progress / pending saves
-- ---------------------------------------------------------------------------
function RuntimeCampaign:mark_objective(node)
  local p = self:progress()
  if not p or p.objectives[node.id] == 'done' then return end
  if COMBAT[node.role] then return end
  self:_complete_objective(node.id)
end

function RuntimeCampaign:_complete_objective(room_id)
  local p = self:progress()
  if p.objectives[room_id] == 'done' then return true end
  local staged = copy(p)
  local ok, why = self.Progress.complete_objective(staged, room_id, 'done')
  if not ok then return nil, why end
  return self:_persist_progress(staged)
end

-- Install the newest in-memory progress (mirrored into this instance's own run)
-- and queue a bounded paused retry when the durable write fails. A later change
-- stages from this installed record and replaces the pending one, so a newer
-- generation is never followed by an older write.
function RuntimeCampaign:_mirror(progress)
  local r = self.run
  if not r then return end
  r.progress.room = progress.current_room
  r.progress.supplies = progress.supplies
  r.stocks = progress.lives
  local cleared, claimed = {}, {}
  for id, state in pairs(progress.objectives) do if state == 'done' then cleared[id] = true end end
  for id in pairs(progress.claimed) do local room = id:match('^reward:(.+)$'); if room then claimed[room] = true end end
  r.progress.cleared, r.progress.claimed = cleared, claimed
end

function RuntimeCampaign:_persist_progress(staged)
  local ok, why = self:_save(staged)
  if ok then return true end
  self.route.progress = staged
  self:_mirror(staged)
  self.pending_save = { manifest = self.route.manifest, progress = staged, attempts = 0,
    resume_phase = self.phase }
  self.notice = 'Progress save pending: ' .. tostring(why)
  return false, why
end

function RuntimeCampaign:_flush_pending_save()
  local pending = self.pending_save
  if not pending then return true end
  pending.attempts = (pending.attempts or 0) + 1
  local ok, why = self:_save(pending.progress)
  if ok then
    -- Exhaustion changes the phase to error, but does not complete the operation
    -- that queued this write. In particular, settling still owes room entry,
    -- finish and reward dispatch after its objective becomes durable.
    self.phase = pending.resume_phase or 'active'
    self.error_msg = nil
    return true
  end
  if pending.attempts >= self.max_save_attempts then
    self.error_msg = 'Progress save refused after retries: ' .. tostring(why)
    self.phase = 'error'
  end
  return false, why
end

-- ---------------------------------------------------------------------------
-- Encounter wiring
-- ---------------------------------------------------------------------------
function RuntimeCampaign:frame(player, previous)
  if self.phase ~= 'active' then return end
  self:_update_lock(player)
  if self.encounter_active then
    local staged = copy(self:progress())
    local ok, events = self.encounters:update(staged)
    if ok then
      if self:_stage_events(staged, events) then self:_persist_progress(staged) end
    elseif events ~= nil then
      self.error_msg = tostring(events)
    end
  end
  self:_detect_drop(player, previous)
end

function RuntimeCampaign:_update_lock(player)
  if not self.locked or not player then return end
  local p0 = self.lock_pos
  if not p0 then self.locked = false; return end
  if math.abs(player.x - p0.x) > 18 or math.abs(player.y - p0.y) > 12 then self.locked = false end
end

function RuntimeCampaign:_detect_drop(player, previous)
  local node = self:current_node()
  if not node or not player then return end
  local p = self:progress()
  if COMBAT[node.role] and p.objectives[node.id] ~= 'done' then return end
  for _, exit in ipairs(node.exits) do
    local anchor = self.Rooms.anchor(node, exit)
    if anchor and anchor.drop then
      local info = self.rooms:resolve_exit(node, node_of(self, exit.to), exit)
      if info and self.rooms:triggered(info, player, node.room and node.room.floor, previous) then
        self:request_travel(exit)
        return
      end
    end
  end
end

function RuntimeCampaign:_stage_events(staged, events)
  if type(events) ~= 'table' then return false end
  local dirty = false
  for _, e in ipairs(events) do
    if type(e) == 'table' then
      if e.kind == 'defeat' or e.kind == 'stock' or e.kind == 'respawn' then
        dirty = true
      elseif e.kind == 'clear' then
        dirty = true
        self.encounter_active, self.encounter_node = false, nil
        if self.gd.cpu_mode then self.gd.cpu_mode(2, 'stand') end
        local node = self:current_node()
        local rid = node and node.id
        if rid and staged.objectives[rid] ~= 'done' then
          self.Progress.complete_objective(staged, rid, 'done')
        end
        if node and node.reward_spec and not staged.claimed['reward:' .. node.id] then
          self.reward_room = node.id
        end
      elseif e.kind == 'error' then
        self.error_msg = tostring(e.reason or 'encounter error')
      end
    end
  end
  if self.on_event then self.on_event({ kind = 'encounter_events', events = events }) end
  return dirty
end

function RuntimeCampaign:enemy_defeated(handle)
  if not self.encounter_active then return false end
  local staged = copy(self:progress())
  local ok, events = self.encounters:defeat(handle, staged)
  if not ok then return false, events end
  self:_stage_events(staged, events)
  return self:_persist_progress(staged)
end

-- One observed native KO. The caller passes (1,0): never a fabricated multi-KO
-- jump from a baseline mismatch.
function RuntimeCampaign:stock(port, before, after)
  if not self.encounter_active then return false end
  local staged = copy(self:progress())
  local ok, events = self.encounters:stock(port, before, after, staged)
  if not ok then return false, events end
  self:_stage_events(staged, events)
  return self:_persist_progress(staged)
end

function RuntimeCampaign:hit(event)
  if not self.encounter_active then return nil end
  return self.encounters:hit(event)
end

function RuntimeCampaign:active_entity()
  if not self.encounter_active then return nil end
  return self.encounters:active_entity()
end

function RuntimeCampaign:encounter_composition()
  if not self.encounter_active then return {} end
  return self.encounters:composition()
end

-- ---------------------------------------------------------------------------
-- Doors
-- ---------------------------------------------------------------------------
function RuntimeCampaign:door_exit(player)
  if self.phase ~= 'active' or not player then return nil end
  local node = self:current_node()
  local p = self:progress()
  if not node then return nil end
  if COMBAT[node.role] and p.objectives[node.id] ~= 'done' then return nil, 'Defeat the encounter first' end
  if self.locked then return nil, 'Clear the doorway first' end
  for _, exit in ipairs(node.exits) do
    local anchor = self.Rooms.anchor(node, exit)
    if anchor and not anchor.drop then
      if math.abs(player.x - anchor.x) <= self.rooms.door_reach
        and math.abs(player.y - anchor.y) <= self.rooms.vertical_reach then
        return exit
      end
    end
  end
  return nil
end

function RuntimeCampaign:request_door(player)
  local exit, why = self:door_exit(player)
  if exit then return self:request_travel(exit) end
  if why then return nil, why end
  return nil, 'No exit here'
end

-- ---------------------------------------------------------------------------
-- Rewards and native effects
-- ---------------------------------------------------------------------------
function RuntimeCampaign:reward_options(node)
  node = node or self:current_node()
  if not node or not node.reward_spec then return {} end
  local preview, why = self.rewards:preview({ run = self.run, progress = self:progress(),
    node = node, effects = self:effect_support() })
  if not preview then return {}, why end
  if not preview.requires_target then return { { id = nil } }, nil, preview end
  return preview.targets or {}, nil, preview
end

-- Apply reversible native effects with pcall + readback. Restoration
-- responsibility is recorded BEFORE the write is issued, because a refusing or
-- throwing callback may still have mutated the engine; on any failure every
-- issued entry is restored and anything that cannot be verified as restored is
-- retained in self.effect_pending for a bounded paused retry.
function RuntimeCampaign:_restore_effect(effect)
  local ok, res = pcall(self.gd.set_percent, effect.port, effect.before)
  local read = self.gd.player and self.gd.player(effect.port)
  return ok and res ~= false and type(read) == 'table' and read.percent == effect.before
end

function RuntimeCampaign:_rollback_effects(applied)
  local kept = {}
  for i = #applied, 1, -1 do
    local effect = applied[i]
    if effect.kind == 'heal' then
      if not self:_restore_effect(effect) then kept[#kept + 1] = effect end
    end
  end
  if #kept > 0 then return false, kept end
  return true
end

function RuntimeCampaign:_apply_effects(effects)
  local issued = {}
  local function fail(why)
    local rolled, kept = self:_rollback_effects(issued)
    if not rolled then
      self.effect_pending = { applied = kept, attempts = 0, reason = why }
      self.phase = 'effects'
      return nil, why .. '; effect rollback pending'
    end
    return nil, why
  end
  for _, effect in ipairs(effects or {}) do
    if effect.kind == 'heal' then
      local before_p = self.gd.player(1)
      if type(before_p) ~= 'table' or type(before_p.percent) ~= 'number' then
        return fail('heal unavailable: no player percent')
      end
      local before = before_p.percent
      local target = math.max(0, before - effect.amount)
      -- Record restoration responsibility before the write: the callback may
      -- mutate then refuse or throw.
      issued[#issued + 1] = { kind = 'heal', port = 1, before = before, after = target }
      local ok, res = pcall(self.gd.set_percent, 1, target)
      if not ok then return fail('heal threw: ' .. tostring(res)) end
      if res == false then return fail('heal was refused') end
      local after_p = self.gd.player(1)
      if type(after_p) ~= 'table' or after_p.percent ~= target then
        return fail('heal readback mismatch')
      end
    else
      return fail('unsupported native effect ' .. tostring(effect.kind))
    end
  end
  return issued
end

-- Retry the pending native effect rollback. Returns true when nothing is pending.
function RuntimeCampaign:retry_effects()
  if not self.effect_pending then return true end
  self.effect_pending.attempts = (self.effect_pending.attempts or 0) + 1
  local rolled, kept = self:_rollback_effects(self.effect_pending.applied)
  if rolled then
    self.effect_pending = nil
    self.phase = self.rooms:active_room() and 'active' or 'idle'
    self.error_msg = nil
    return true
  end
  self.effect_pending.applied = kept
  if self.effect_pending.attempts >= self.max_effect_attempts then
    self.error_msg = 'effect rollback refused after retries: ' .. tostring(self.effect_pending.reason)
    self.phase = 'error'
  end
  return false, self.error_msg
end

function RuntimeCampaign:reward_preview(node, target)
  node = node or self:current_node()
  if not node or not node.reward_spec then return nil, 'no reward' end
  return self.rewards:preview({ run = self.run, progress = self:progress(), node = node,
    target = target, effects = self:effect_support() })
end

function RuntimeCampaign:reward_commit(node, target)
  node = node or self:current_node()
  if not node or not node.reward_spec then return nil, 'no reward' end
  if self:blocked() then return nil, 'a transition is in progress' end
  local result, why = self.rewards:commit({ run = self.run, progress = self:progress(),
    node = node, target = target, effects = self:effect_support() })
  if not result then return nil, why end
  local applied, ewhy = self:_apply_effects(result.effects)
  if not applied then return nil, ewhy end
  local ok, swhy = self:_save(result.progress, result.run)
  if not ok then
    -- Roll the native effects back; the live run is untouched by the failed save
    -- (save only commits run_override on success), so no snapshot/rebind is done.
    local rolled, kept = self:_rollback_effects(applied)
    if not rolled then
      self.effect_pending = { applied = kept, attempts = 0, reason = swhy }
      self.phase = 'effects'
      return nil, 'reward save refused; effect rollback pending: ' .. tostring(swhy)
    end
    return nil, 'reward save refused: ' .. tostring(swhy)
  end
  -- _save committed result.run into this instance on the durable write.
  self.reward_room = nil
  self.say(result.message or 'Reward claimed', 'upgrade', 'reward:' .. node.id)
  if self.on_event then self.on_event({ kind = 'reward', room = node.id, claim = result.claim_id }) end
  return result
end

-- Spend one life on an observed player stock loss. Returns 'failure' at zero.
function RuntimeCampaign:lose_life()
  local staged = copy(self:progress())
  local ok, why = self.Progress.lose_life(staged)
  if not ok then self.error_msg = tostring(why); return 'failure' end
  self:_persist_progress(staged)
  if staged.lives <= 0 then return 'failure' end
  return 'survived'
end

-- Spend one supply for a documented heal, atomically and verifiably. A refused
-- native write is not spent; a failed save restores the percent.
function RuntimeCampaign:use_supply()
  local p = self:progress()
  local player = self.gd.player and self.gd.player(1)
  if not (p and type(player) == 'table') then return false, 'no player' end
  if p.supplies <= 0 or player.percent <= 0 then return false, 'Nothing to restore' end
  local staged = copy(p)
  local ok, why = self.Progress.spend_supply(staged)
  if not ok then return false, why end
  local applied, ewhy = self:_apply_effects({ { kind = 'heal', amount = 30 } })
  if not applied then return false, ewhy end
  local saved, swhy = self:_save(staged)
  if not saved then
    local rolled, kept = self:_rollback_effects(applied)
    if not rolled then self.effect_pending = { applied = kept, attempts = 0, reason = swhy }; self.phase = 'effects' end
    return false, swhy
  end
  self.say('Supply used: percent -30', 'info', 'supply')
  return true
end

-- ---------------------------------------------------------------------------
-- Map, diagnostics, teardown
-- ---------------------------------------------------------------------------
function RuntimeCampaign:route_map()
  local route = route_of(self)
  if not route then return nil, 'no route' end
  return self.RouteMap.build(route.manifest, route.progress)
end

-- Release everything this owner holds. `opts.drop_pending` applies ONLY to a
-- superseded logical save (never to native effect ownership): an unpersisted
-- advance of the OLD run must not be written into the NEW run's checkpoint. A
-- refused native effect rollback keeps the owner dirty so retirement/promotion
-- stays blocked and a later paused retry can restore it.
function RuntimeCampaign:teardown(opts)
  opts = opts or {}
  local failures = {}
  if self.effect_pending then
    local restored = self:retry_effects()
    if not restored then failures[#failures + 1] = 'native effect rollback pending' end
  end
  if self.pending_save then
    if opts.drop_pending then
      self.pending_save = nil
    else
      local ok, why = self:_flush_pending_save()
      if not ok then failures[#failures + 1] = 'pending save not durable: ' .. tostring(why) end
    end
  end
  local ok, why = self.encounters:clear()
  if not ok then failures[#failures + 1] = tostring(why) end
  local clean, cwhy = self.rooms:cleanup()
  if not clean then failures[#failures + 1] = tostring(cwhy) end
  if #failures > 0 then return false, table.concat(failures, '; ') end
  return true
end

-- Explicit user/hook recovery attempt for a terminal error: retries the pending
-- native rollback, the pending durable save, or a refused transition rollback
-- from a fresh bound. Returns true only when play can resume or a retired owner
-- is clean and idle; recovering/settling still owe work and remain blocked.
function RuntimeCampaign:retry_recovery()
  if self.effect_pending then
    local ok = self:retry_effects()
    if not ok then return false, self.error_msg end
  end
  if self.pending_save then
    self.pending_save.attempts = 0
    local ok, why = self:_flush_pending_save()
    if not ok then return false, why end
    self.error_msg = nil
    return self:running()
  end
  if self.phase == 'error' and self.tx then
    self.rollback_attempts = 0
    self.phase = 'recovering'
    self:attempt_recovery()
    if self:running() then self.error_msg = nil; return true end
    return false, self.error_msg
  end
  return self:running() or (self.phase == 'idle' and not self:blocked() and self.tx == nil)
end

-- Destructive retirement of the source happens only after a durable save. A
-- genuine native scene teardown (on_match_end) may call reset(true); a script
-- unload (gs_unload drops only the script env) is NOT scene teardown and must
-- use reset(false) after teardown, or retain the handles for a native seam.
function RuntimeCampaign:reset(confirmed)
  -- Never discard unresolved logical/native recovery without a confirmed scene
  -- teardown: the engine still holds the mutated state we must undo.
  if confirmed ~= true and (self.effect_pending or self.pending_save) then
    return false, 'unresolved effect/save recovery; confirmed teardown required'
  end
  local ok, why = self.rooms:reset(confirmed)
  if not ok then return false, why end
  self.phase = 'idle'
  self.tx, self.staged, self.dest_node, self.pending = nil, nil, nil, nil
  self.pending_save, self.effect_pending, self.error_msg = nil, nil, nil
  self.encounter_active, self.encounter_node, self.encounter_pending = false, nil, false
  self.reward_room, self.locked, self.rollback_attempts = nil, false, 0
  return true
end

function RuntimeCampaign:status()
  local node = self:current_node()
  local composition = self.encounter_active and self.encounters:composition() or {}
  return { version = 2, phase = self.phase, room = node and node.id, role = node and node.role,
    depth = node and node.depth, transitioning = self:transitioning(), blocked = self:blocked(),
    failed = self:failed(), error = self.error_msg, notice = self.notice,
    locked = self.locked, encounter = self.encounter_active and self.encounter_node or nil,
    entities = #composition, reward_room = self.reward_room,
    pending_save = self.pending_save ~= nil, effect_pending = self.effect_pending ~= nil,
    pending_source = self.rooms:pending() ~= nil }
end

return RuntimeCampaign
