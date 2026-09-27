-- Type "fdcine" in the console. Simulation remains at 60 Hz; the renderer interpolates
-- camera matrices on extra presents when the port's frame-rate setting is above 60.
local active = false
local bars = 0

local function cinematic()
  gd.scene_launch{mode = "training", p1 = "fox", p2 = "falco/cpu", stage = "fd"}
  if not gd.wait_until(function() return gd.match().active end, 1200) then
    gd.log("fdcine: FD match did not start")
    active = false
    return
  end
  gd.wait(45) -- let the fighter entry finish
  local fighters = gd.players()
  if #fighters < 2 then
    gd.log("fdcine: two fighters are needed")
    active = false
    return
  end
  local cx = (fighters[1].x + fighters[2].x) / 2
  local cy = (fighters[1].y + fighters[2].y) / 2 + 24
  local interest = {x = cx, y = cy, z = 0}
  local keys = {}
  local orbit_frames, steps, radius = 180, 12, 190
  gd.camera_detach()
  for i = 0, steps do
    local a = (i / steps) * 2 * math.pi
    keys[#keys + 1] = {
      frame = i * orbit_frames / steps,
      eye = {x = cx + radius * math.sin(a), y = cy + 70, z = radius * math.cos(a)},
      interest = interest, fov = 38,
    }
  end
  gd.camera_path(keys)
  bars = 42
  gd.wait(orbit_frames + 1)
  local last = gd.camera_get()
  gd.camera_move{
    to = {eye = {x = cx, y = cy + 42, z = 95}, interest = interest, fov = 27},
    frames = 75, ease = "inout",
  }
  gd.wait(76)
  gd.camera_attach(45)
  gd.wait(46)
  bars = 0
  active = false
  gd.log(string.format("fdcine: complete (orbit started at %.1f, %.1f)", last.eye.x, last.eye.y))
end

gd.command("fdcine", function()
  if active then return end
  active = true
  gd.run(cinematic)
end, "Training on FD: slow orbit, push-in, then reattach the match camera")

function on_draw()
  if bars > 0 then
    -- Use the port kit palette when present for the two flat overlay bars.
    local black = gd.kit.colors.ink or 0x080B12FF
    gd.fill(0, 0, 640, bars, black)
    gd.fill(0, 480 - bars, 640, bars, black)
  end
end

function on_match_end()
  bars = 0
  active = false
end
