-- The Atlas pause screen (kind = "pause" and gd.ui.pause_screen): Resume and a Log row. The retail pause (START in a VS-style match) pushes it only when the
-- takeover is on (MELEE_ATLAS_PAUSE=1 or the setting atlas_pause); with it off, nothing changes: retail's pause looks as it always did.
-- Resume asks the engine to unpause (gd.ui.unpause: a one-shot request, offline only), and the game runs retail's own unpause routine for the pauser (audio, camera,
-- the ten-frame unpause timer). The pausing port drives the screen. A pause screen is never opened online.
local ui = gd.ui
local ID = 'demo_atlas_pause.pause'

local function describe()
  return { id = ID, kind = 'pause', trail = { 'PAUSED' , title = 'ATLAS PAUSE DEMO' }, chapter = 1,
    primary = { kind = 'list', items = {
      { id = 'resume', label = 'Resume' },
      { id = 'log', label = 'Log a line' } } },
    explainer = { width = 'normal', provide = function(cid)
      if cid == 'resume' then return { kicker = 'PAUSED', title = 'RESUME', what = 'Asks the game to unpause: the retail routine runs for the port that paused.' } end
      return { kicker = 'PAUSED', title = 'LOG', what = 'Writes a line to the log, so you can see the screen is the demo\'s.' } end },
    keys = { { 'A', 'Select' }, { 'B', 'Resume' } },
    on = {
      accept = function(cid)
        if cid == 'resume' then gd.log('demo_atlas_pause: resume -> ' .. tostring(ui.unpause()))
        else gd.log('demo_atlas_pause: log row pressed') end
      end,
      back = function() gd.log('demo_atlas_pause: back -> ' .. tostring(ui.unpause())) end } }
end

-- There is no on_load hook (the engine calls on_tick, on_scene, on_unload ...): the demo's old on_load never ran, so the screen was never named
-- (Atlas proof D6). The Atlas roles also load a frame after the scripts, so ui.available() is false at first: register on the first tick it is true.
local named = false
local function register()
  if named then return true end
  if not ui.available() then return false end
  ui.screen(describe())
  ui.pause_screen(ID)
  named = true
  gd.log('demo_atlas_pause: named ' .. ID .. ' as the pause takeover screen (it shows only with MELEE_ATLAS_PAUSE=1)')
  return true
end

function on_tick() register() end   -- until it succeeds; then a no-op

function on_unload()
  if named and ui.available() then pcall(ui.pause_screen, nil) end
  named = false
end
