-- Copy this folder to scripts/fd_stage_content beside melee-pc.exe and enable it as a script mod.
-- Gr_Kind_Last is Final Destination (gr/forward.h, 0x25). Coordinates are world units.
local FINAL_DESTINATION = 0x25

function on_match_start()
  local match = gd.match()
  if match.netplay or match.stage ~= FINAL_DESTINATION then return end

  local a = assert(gd.stage_add_platform(-55, 22, 34, {passthrough = true, ledges = true}))
  local b = assert(gd.stage_add_platform(0, 42, 42, {passthrough = true}))
  local c = assert(gd.stage_add_platform(57, 22, 34, {passthrough = false, ledges = true}))
  gd.log(string.format("FD Lua platforms: %d, %d, %d", a, b, c))

  local positions = {
    {-82, 48}, {-55, 58}, {-28, 66}, {0, 74}, {28, 66},
    {55, 58}, {82, 48}, {-45, 12}, {0, 17}, {45, 12}
  }
  for _, p in ipairs(positions) do
    assert(gd.spawn_target(p[1], p[2]))
  end
  gd.log("FD Lua targets: 10 placed")
end

function on_target_broken(handle, remaining)
  gd.log(string.format("FD Lua target %d broken; %d remain", handle, remaining))
end

function on_all_targets_broken()
  gd.log("FD Lua: all targets broken")
end
