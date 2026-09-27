-- Install this folder as scripts/bf_platform beside melee-pc.exe.
-- Export the model into models/ first; see docs/scripting.md.
local FINAL_DESTINATION = 0x25

function on_match_start()
  local match = gd.match()
  if match.netplay or match.stage ~= FINAL_DESTINATION then return end

  local opts = {passthrough = true, ledges = true, model = "bf_platform"}
  local left = assert(gd.stage_add_platform(-60, 26, 28, opts))
  local centre = assert(gd.stage_add_platform(0, 50, 48, opts))
  local right = assert(gd.stage_add_platform(58, 30, 36, opts))
  gd.log(string.format("BF platform models: %d, %d, %d", left, centre, right))
end
