-- First-run onboarding. Pure progression over the player's own recorded events;
-- it never calls the engine and never pauses combat. The tutorial teaches the
-- recursive D-pad grammar in the first rooms and then gets out of the way.
--
-- Design rules:
--   * Combat is never force-paused. view().pause is always false.
--   * The grammar steps (branch, back) are only offered inside the first
--     configured rooms; later they lapse instead of nagging.
--   * Genealogy/fusion are explained only once those features exist for the
--     player (declared by the caller, never invented here).
--   * Experienced players can skip, and any step can be revisited.
local Onboarding = {version = 1}

-- Ordered core steps. `event` is the observation kind that completes the step,
-- `early` gates it to the first rooms, `text`/`hint` are player-facing.
local CORE = {
  {id = 'move', event = 'move', text = 'Move and attack normally',
   hint = 'Move with the stick; attack with A. This plays like Melee.'},
  {id = 'charge', event = 'charge', text = 'Earn one ability charge',
   hint = 'Land hits to charge your placed ability. Watch the HUD segment.'},
  {id = 'branch', event = 'command_navigate', early = true, text = 'Open a command branch',
   hint = 'D-pad Left: abilities. Right: supplies. Down: run options. Combat keeps running.'},
  {id = 'back', event = 'command_back', early = true, text = 'Back out with Up',
   hint = 'Up returns one level. At the root, a fresh Up is your normal taunt.'},
  {id = 'cast', event = 'cast', text = 'Cast one ability',
   hint = 'D-pad Left opens abilities. Left / Right / Down on the final branch casts. Release between presses.'},
  {id = 'tell', event = 'tell', text = 'Recognize an enemy tell',
   hint = 'When an enemy casts, move or shield before the hit lands.'},
  {id = 'door', event = 'door', text = 'Use a door',
   hint = 'Clear enemies, stand at an exit, then press D-pad Down to enter the next room.'},
  {id = 'reward', event = 'reward', text = 'Accept a reward',
   hint = 'Read the reward, choose its target with the D-pad, then press A to apply.'},
}

local function copy(t) if type(t) ~= 'table' then return t end local o = {} for k, v in pairs(t) do o[k] = copy(v) end return o end
local function index_of(id) for i, step in ipairs(CORE) do if step.id == id then return i end end end

function Onboarding.new(opts)
  opts = opts or {}
  local s = {steps = {}, done = {}, order = {}, first_rooms = opts.first_rooms or 3,
    room_index = opts.room_index or 1, skipped = opts.skip == true, revisit = false}
  local enabled = {}
  if type(opts.steps) == 'table' then
    for _, id in ipairs(opts.steps) do if index_of(id) then enabled[id] = true end end
  else
    for _, step in ipairs(CORE) do enabled[step.id] = true end
  end
  for _, step in ipairs(CORE) do if enabled[step.id] then s.order[#s.order + 1] = step.id end end
  return s
end

function Onboarding.reset(s)
  s.done = {}; s.revisit = false
end

function Onboarding.skip(s, on) s.skipped = on == true end

-- Experienced players can replay the sequence from the first incomplete step.
function Onboarding.revisit(s)
  s.skipped = false; s.revisit = true; s.done = {}; s.room_index = 1
end

function Onboarding.set_room(s, index)
  if type(index) == 'number' and index == index and index >= 1 then s.room_index = math.floor(index) end
  return s.room_index
end

local function step_by_id(id) for _, step in ipairs(CORE) do if step.id == id then return step end end end

-- An early grammar step whose first-rooms window has passed is effectively
-- done; it must not nag the player later. view() stays read-only by computing
-- this rather than mutating s.done.
local function effective_done(s, step)
  if s.done[step.id] then return true end
  return step.early == true and s.room_index > s.first_rooms
end

local function current_id(s)
  for _, id in ipairs(s.order) do local step = step_by_id(id); if step and not effective_done(s, step) then return id end end
  return nil
end

-- observe(s, event) -> completed id | nil, reason
-- event.kind: move|charge|command_navigate|command_back|cast|tell|door|reward|
--             breed|fuse. `early` steps only complete within the first rooms.
function Onboarding.observe(s, event)
  if type(event) ~= 'table' or type(event.kind) ~= 'string' then return nil, 'invalid event' end
  if s.skipped then return nil, 'skipped' end
  -- Optional late explanations, only when the caller declares the feature is
  -- actually available to this player (never fabricated here).
  if event.kind == 'breed' and event.available == true and not s.done.genealogy then
    s.done.genealogy = true; return 'genealogy', 'Bred traits are inherited; originals are kept.'
  end
  if event.kind == 'fuse' and event.available == true and not s.done.fusion then
    s.done.fusion = true; return 'fusion', 'Fusion consumes both run parents and averages their upgrades.'
  end
  local id = current_id(s)
  if not id then return nil, 'complete' end
  local step = step_by_id(id)
  if step.event ~= event.kind then return nil, 'not the current step' end
  if step.early and s.room_index > s.first_rooms then
    s.done[id] = true; return nil, 'window passed'
  end
  s.done[id] = true
  return id
end

-- view(s) is read-only and always reports pause=false; onboarding must never
-- stop combat. It returns the current objective plus the remaining count.
function Onboarding.view(s)
  local id = current_id(s)
  local step = id and step_by_id(id)
  local done = 0
  for _, sid in ipairs(s.order) do local st = step_by_id(sid); if st and effective_done(s, st) then done = done + 1 end end
  return {
    version = Onboarding.version, skipped = s.skipped, pause = false,
    step = id, text = step and step.text or nil,
    hint = (not s.skipped) and step and step.hint or nil,
    index = id and index_of(id) or nil, total = #s.order, completed = done,
    remaining = #s.order - done, room_index = s.room_index,
    early = step and step.early == true or false,
  }
end

-- The text main should route through the toast queue (coalesced by step id).
function Onboarding.toast(s)
  local v = Onboarding.view(s)
  if v.skipped or not v.hint then return nil end
  return {kind = 'info', key = 'tutorial:' .. v.step, title = v.text, detail = v.hint, ttl = 480}
end

function Onboarding.core() return copy(CORE) end
return Onboarding
