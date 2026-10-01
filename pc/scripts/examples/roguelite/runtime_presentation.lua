-- Live presentation glue for main.lua. It owns nothing in the game world: it
-- composes the already reviewed, individually tested pure modules
-- (`hud_layout`, `onboarding`, `commands`, `feedback`) into the calls main
-- actually makes each tick.
--
-- Responsibilities, and nothing else:
--   * build one `Hud.layout` per presentation state and hand it to
--     `Feedback.draw`, so the compact rail, its opponent block and the
--     notification strips get real coordinates instead of hardcoded ones;
--   * own the native HUD visibility claim (`gd.hud_visible`) explicitly, and
--     give it back on leave / error / unload / match end;
--   * turn truthful observations from main's real gameplay paths into
--     `Onboarding` progress and route the resulting hint into the existing
--     toast queue (never a second UI channel, never a pause);
--   * rebuild the command tree from the run's real installed loadout after a
--     gene placement, a resume or a current-run commit.
--
-- Every public method is total: a layout, drawing, input or planning failure is
-- contained and reported, because UI trouble must never corrupt run or save
-- state.
--
-- Coordinate space: the port's documented fixed 640x480 virtual canvas
-- (`docs/scripting.md`). There is no viewport or pixel-DPI API, so the canvas is
-- not derived from the OS window: a larger window must not enlarge HUD
-- coverage. `dpi` is the mod's own declared UI scale and is clamped by Hud.
local Presentation = {version = 1}

local SLOTS = {'assault', 'traversal', 'guard'}

local function copy(t) if type(t) ~= 'table' then return t end local o = {} for k, v in pairs(t) do o[k] = copy(v) end return o end

function Presentation.new(gd, deps, opts)
  assert(type(gd) == 'table', 'engine table required')
  assert(type(deps) == 'table' and deps.Hud and deps.Onboarding and deps.Commands and deps.Feedback,
    'presentation dependencies missing')
  opts = opts or {}
  local self = setmetatable({
    gd = gd, Hud = deps.Hud, Onboarding = deps.Onboarding,
    Commands = deps.Commands, Feedback = deps.Feedback,
    -- The port documents one fixed virtual canvas; never a window-derived size.
    width = opts.width or 640, height = opts.height or 480, dpi = opts.dpi or 1,
    reduced = opts.reduced == true,
    log = opts.log or function() end,
    say = opts.say or function() end,
    -- The shared toast queue. Bound by main (or by bind) because Feedback is a
    -- single live state, not a module.
    feedback = opts.feedback,
    -- Owns the native HUD only while it is actually hidden: that is the same
    -- condition under which the compact rail is allowed to replace it.
    compact = false,
    hud_release_pending = false,
    layout = nil, layout_key = nil,
    charged = {}, told = {}, told_room = nil,
  }, {__index = Presentation})
  self.tutorial = deps.Onboarding.new(opts.tutorial)
  return self
end

-- ---------------------------------------------------------------------------
-- Layout
-- ---------------------------------------------------------------------------
-- One layout per presentation state. `command` only changes which side panel is
-- live, and `compact_menu` swaps the notification strip for the workbench strip;
-- neither moves the rail, so the rail's geometry is stable while a menu opens.
function Presentation:state(flags)
  flags = flags or {}
  local key = (flags.command == true and 'c' or '-')
    .. (flags.compact_menu == true and 'm' or '-')
    .. (self.compact and 'h' or '-')
  if self.layout and self.layout_key == key then return self.layout end
  local opts = {width = self.width, height = self.height, dpi = self.dpi,
    new_hud = self.compact, command = flags.command == true,
    compact_menu = flags.compact_menu == true}
  local ok, lay, why = pcall(self.Hud.layout, opts)
  if ok and lay then
    self.layout, self.layout_key = lay, key
    return lay
  end
  if not ok then why = lay end
  -- A broken custom layout cannot safely replace the native damage readout.
  -- A refused restore keeps its ownership token for the next lifecycle retry.
  self:release_hud()
  -- A refused layout must not blank the run: fall back to the unscaled canvas so
  -- the player keeps a readable life/damage readout, and report it honestly.
  if self.layout_key ~= 'fallback' then
    self.layout_key = 'fallback'
    self.log('HUD layout refused (' .. tostring(why) .. '); using the minimal safe layout')
  end
  local safe_ok, safe, safe_why = pcall(self.Hud.layout, {width = self.width, height = self.height})
  if not safe_ok then safe_why = safe; safe = nil end
  self.layout = safe or {width = self.width, height = self.height, safe = {x = 0, y = 0, w = self.width, h = self.height},
    replace_vanilla = false, scale = 1, content_scale = 1, note_scale = 1, compact_scale = 1}
  if not safe then self.log('HUD fallback layout refused (' .. tostring(safe_why) .. ')') end
  return self.layout
end

-- ---------------------------------------------------------------------------
-- Native HUD ownership
-- ---------------------------------------------------------------------------
-- Claim (`want=true`) hides the vanilla stock/percent cluster because the
-- compact rail replaces it; `want=false` gives the native HUD back. A
-- non-throwing call that returns `false` is the *new state* (hidden), not a
-- failed mutation, so only a throw or a missing API counts as a failure.
function Presentation:take_vanilla_hud(want)
  want = want == true
  if self.hud_release_pending then return self:release_hud() end
  if want == self.compact then return false, 'unchanged' end
  if type(self.gd.hud_visible) ~= 'function' then return false, 'native HUD control unavailable' end
  local ok, res = pcall(self.gd.hud_visible, not want)
  if not ok then return false, tostring(res) end
  self.compact = want
  self.layout, self.layout_key = nil, nil  -- replace_vanilla changed
  return true, res
end

-- Hand the native HUD back unconditionally; safe to call when not owned.
function Presentation:release_hud()
  if not self.compact then return false, 'not owned' end
  self.hud_release_pending = true
  if type(self.gd.hud_visible) ~= 'function' then return false, 'unavailable' end
  local ok, res = pcall(self.gd.hud_visible, true)
  if not ok then return false, tostring(res) end
  self.compact = false
  self.hud_release_pending = false
  self.layout, self.layout_key = nil, nil
  return true, res
end

function Presentation:hud_owned() return self.compact end

-- ---------------------------------------------------------------------------
-- Drawing
-- ---------------------------------------------------------------------------
-- ctx: {hud=, notifications=, compact_menu=, command=, player=, opponent=}
function Presentation:draw(feedback, ctx)
  ctx = ctx or {}
  local lay = self:state({command = ctx.command == true, compact_menu = ctx.compact_menu == true})
  local ok, err = pcall(self.Feedback.draw, feedback, {
    hud = ctx.hud ~= false, notifications = ctx.notifications ~= false,
    compact_menu = ctx.compact_menu == true, reduced = self.reduced,
    layout = lay, player = ctx.player, opponent = ctx.opponent})
  if not ok then self:release_hud(); self.log('HUD draw refused: ' .. tostring(err)) ; return false end
  return true
end

-- ---------------------------------------------------------------------------
-- Tutorial
-- ---------------------------------------------------------------------------
-- Announce the current step once per step (and once per room for the early
-- grammar steps). The hint always travels through the shared toast queue with a
-- step-keyed identity, so it coalesces instead of repeating and it sits below
-- enemy tells in priority.
function Presentation:announce(force)
  local view = self.Onboarding.view(self.tutorial)
  if view.skipped or not view.step then
    self.told[view.step or 'complete'] = true
    return nil
  end
  if not force and self.told[view.step] then return nil end
  self.told[view.step] = true
  local toast = self.Onboarding.toast(self.tutorial)
  if not toast then return nil end
  self:toast(toast)
  return toast
end

-- Bind the shared feedback/toast state. Returns the presentation so main can
-- bind at construction; a later rebind replaces the queue.
function Presentation:bind(feedback)
  if feedback == nil then return self end
  self.feedback = feedback
  return self
end

function Presentation:toast(toast)
  if type(toast) ~= 'table' then return false end
  if not self.feedback then return false, 'no toast queue' end
  local ok, err = pcall(self.Feedback.tutorial, self.feedback, toast)
  if not ok then self.log('tutorial notice refused: ' .. tostring(err));return false end
  return ok ~= false
end

-- observe(event) -> completed step id | nil
-- event: {kind=move|charge|command_navigate|command_back|cast|tell|door|reward|breed|fuse}
-- `available` is supplied by the caller and is the only thing that may teach
-- genealogy or fusion: an unimplemented feature is never announced.
function Presentation:observe(event)
  if type(event) ~= 'table' or type(event.kind) ~= 'string' then return nil end
  local ok, id = pcall(self.Onboarding.observe, self.tutorial, event)
  if not ok then self.log('tutorial observation refused: ' .. tostring(id));return nil end
  if id then self:announce(false) end
  return id
end

-- Map a real Commands.update event onto the tutorial's grammar steps. Only
-- navigation and a genuine back-out are taught; the root taunt is a native
-- action and is not a tutorial step.
local COMMAND_EVENTS = {navigate = 'command_navigate', back = 'command_back'}

function Presentation:observe_command(event)
  if type(event) ~= 'table' then return nil end
  local kind = COMMAND_EVENTS[event.kind]
  if not kind then return nil end
  return self:observe({kind = kind})
end

-- Route main's real gameplay observations. Keeps only per-slot/per-entity edge
-- state so a held state cannot spam the queue.
function Presentation:notice_charges(abilities)
  if type(abilities) ~= 'table' then return end
  for _, slot in ipairs(SLOTS) do
    local a = abilities[slot]
    local charged = type(a) == 'table' and type(a.charge) == 'number' and a.charge > 0
    if charged and not self.charged[slot] then self.charged[slot] = true;self:observe({kind = 'charge'}) end
    if not charged then self.charged[slot] = nil end
  end
end

function Presentation:notice_tells(states)
  if type(states) ~= 'table' then return end
  local live = {}
  for _, e in ipairs(states) do
    local key = tostring(e.handle or e.id)
    live[key] = true
    if (e.phase == 'teleport' or e.phase == 'telegraph' or e.phase == 'ready')
      and not self.told['tell:' .. key] then
      self.told['tell:' .. key] = true
      self:observe({kind = 'tell'})
    end
  end
  for key in pairs(self.told) do
    if key:sub(1, 5) == 'tell:' and not live[key:sub(6)] then self.told[key] = nil end
  end
end

-- The early grammar steps are only meaningful in the first rooms; a new room is
-- announced once so a route change is noticed without repeating old hints.
function Presentation:set_room(index)
  local room = self.Onboarding.set_room(self.tutorial, index)
  if self.told_room ~= room then self.told_room = room;self:announce(false) end
  return room
end

function Presentation:skip(on) self.Onboarding.skip(self.tutorial, on == true);return self:view() end
function Presentation:revisit() self.Onboarding.revisit(self.tutorial);self.told = {};self.told_room = nil;return self:view() end
function Presentation:reset() self.Onboarding.reset(self.tutorial);self.told = {};self.told_room = nil;self.charged = {} end
function Presentation:view() return self.Onboarding.view(self.tutorial) end

-- ---------------------------------------------------------------------------
-- Command loadout
-- ---------------------------------------------------------------------------
-- Declared capacities only: the consumable count comes from the run's own
-- progress mirror, never from a catalogue dump, so combat never browses the
-- whole collection and Restore is honestly disabled at zero supplies.
function Presentation:command_options(run)
  local supplies = run and run.progress and tonumber(run.progress.supplies) or nil
  return {capacities = {items = {restore = supplies}, supplies = supplies}}
end

-- Rebuild the tree from the run's installed loadout. `buttons` must be the
-- caller's current button word so a held Up keeps its latch across the rebuild
-- and cannot become a fresh root taunt. Contained: a planning failure leaves the
-- previously installed tree in place.
function Presentation:rebuild(state, run, buttons)
  if type(state) ~= 'table' then return false, 'no command state' end
  if type(run) ~= 'table' or type(run.genes) ~= 'table' then
    -- No run to plan from: keep the authored default tree rather than install a
    -- half-built plan.
    return false, 'no run'
  end
  local ok, err = pcall(self.Commands.loadout, state, run, self:command_options(run), buttons)
  if not ok then self.log('command loadout refused: ' .. tostring(err));return false, err end
  return true
end

function Presentation:view_commands(state, availability)
  local ok, view = pcall(self.Commands.view, state, availability)
  if not ok then self.log('command view refused: ' .. tostring(view));return nil end
  return view
end

-- ---------------------------------------------------------------------------
-- Diagnostics / menu context
-- ---------------------------------------------------------------------------
-- Declared settings only. The port exposes no reduced-motion switch, so the
-- value is whatever the mod's own data folder declared; it is reported to the
-- menu rather than invented here.
function Presentation:settings_view()
  return {items = {{id = 'reduced_motion', label = 'REDUCED MOTION',
    value = self.reduced and 'ON' or 'OFF'}}}
end

function Presentation:status()
  local v = self:view()
  return {version = Presentation.version, compact_rail = self.compact,
    width = self.width, height = self.height, dpi = self.dpi, reduced = self.reduced,
    layout = self.layout and self.layout_key or nil, tutorial_step = v.step,
    tutorial_done = v.completed, tutorial_total = v.total}
end

Presentation.copy = copy
return Presentation
