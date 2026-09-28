-- Run ../make_models.py once before installing. Replace its original placeholder
-- art with exported kit parts using the same documented GXMS/sidecar contract.
local parts = {
  {model = "deck", x = -45, y = 30, scale = 1},
  {model = "ramp", x = 0, y = 45, scale = 1},
  {model = "deck", x = 45, y = 30, scale = 0.8, moving = true},
}
local moving

function on_match_start()
  moving = nil
  if gd.match().netplay then return end
  local assets = {}
  for _, part in ipairs(parts) do
    -- For another mounted mod: gd.model_load("stage_kit/models/deck").
    assets[part.model] = assets[part.model] or gd.model_load(part.model)
    local instance, why = gd.model_spawn(assets[part.model], part)
    if not instance then gd.log("runtime models: " .. why) end
    if instance and part.moving then moving = instance end
  end
  -- The instances retain their own references; scene pins also preserve old snapshots.
  for _, asset in pairs(assets) do gd.model_release(asset) end
end

function on_frame()
  if not moving then return end
  -- Read the snapshotted position instead of keeping a second position in Lua.
  local state = gd.model_get(moving)
  if state then
    local x = state.x + 0.15
    if x > 65 then x = 25 end
    gd.model_move(moving, x, state.y, state.z)
  end
end

function on_match_end()
  moving = nil -- engine owns scene teardown, including every model and sidecar
end
