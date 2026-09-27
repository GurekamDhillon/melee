-- Copy this folder to scripts/fd_stage_content beside melee-pc.exe and enable it as a script mod.
-- Gr_Kind_Last is Final Destination (gr/forward.h, 0x25). Coordinates are world units.
local FINAL_DESTINATION = 0x25

function on_match_start()
  local match = gd.match()
  if match.netplay or match.stage ~= FINAL_DESTINATION then return end

  -- Battlefield map_head group 6 has separate platform branches 13/14/15.
  -- Their source origins and floor lines are in _research/stage-model-parts.md.
  local pieces = {
    {joint = 13, x = -48.5, y = 34},
    {joint = 14, x = 48.5, y = 34},
    {joint = 15, x = 0, y = 68},
  }
  for _, p in ipairs(pieces) do
    local floor = assert(gd.stage_add_platform(p.x, p.y, 47, {passthrough = true}))
    local model = assert(gd.stage_add_model{
      file = 'GrNBa.dat', symbol = 'map_head', group = 6, joint = p.joint,
      x = p.x, y = p.y, z = 0, platform = floor,
    })
    gd.log(string.format("FD Battlefield platform: model %d, floor %d", model, floor))
  end

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
