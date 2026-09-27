-- Requires gd.spawn_target from the stage-Lua lane. Without it, leave the vanilla finish alone.
local active, started, left = false, 0, 0
local targets = {}

local function finish(why)
  if not active then return end
  active = false
  targets = {}
  gd.log("boss_bonus: " .. why)
  gd.boss_release()
end

function on_boss_defeated(e)
  if e.kind ~= "master_hand" or active or not gd.spawn_target then return end
  -- The script releases at 20 s; the native 21 s limit is a backstop if this script stops.
  if not gd.boss_hold(21) then return end

  -- World coordinates on Final Destination; these sit above the main platform.
  local positions = {{-42, 23}, {0, 32}, {42, 23}}
  targets = {}
  left = 0
  for _, p in ipairs(positions) do
    local handle, err = gd.spawn_target(p[1], p[2])
    if not handle then
      gd.log("boss_bonus: target spawn failed: " .. tostring(err))
      gd.boss_release()
      targets = {}
      return
    end
    targets[handle] = true
    left = left + 1
  end
  active = true
  started = gd.match().frame
  gd.log(string.format("boss_bonus: Master Hand P%d defeated; %d targets spawned", e.port, left))
end

function on_target_broken(handle)
  if active and targets[handle] then
    targets[handle] = nil
    left = left - 1
    gd.log(string.format("boss_bonus: target %d broken; %d left", handle, left))
    if left == 0 then finish("all targets broken") end
  end
end

function on_frame()
  if active and gd.match().frame - started >= 20 * 60 then finish("20 second timeout") end
end

function on_draw()
  if active and gd.kit.available() then
    gd.kit.text(320, 42, "Bonus! Break the targets", "label", "gold", "center")
  end
end

function on_match_end()
  active = false
  targets = {}
end
