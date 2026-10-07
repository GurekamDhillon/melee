-- Vanilla Charger: the neutral special, written as a Lua callback (Geno slice 5, docs/geno.md section 23).
-- Hold B to charge (up to 60 frames), release to strike: the dash is faster and the hit harder the longer it was held.
-- The only state is ctx.state (declared in geno.json "lua": {"state": ...}); the functions keep nothing between calls.
local MAX_CHARGE = 60
local BASE_SPEED, SPEED_PER_FRAME = 1.6, 0.03
local BASE_DAMAGE, DAMAGE_PER_FRAME = 6, 0.15

local M = {}

function M.charge_enter(ctx)
  ctx.state.charge = 0
  ctx.state.charged = false
end

function M.charge_frame(ctx)
  local s = ctx.state
  if ctx.input.special_held and s.charge < MAX_CHARGE then
    s.charge = s.charge + 1
    s.charged = s.charge >= MAX_CHARGE
    if ctx.self.anim_ended then ctx.loop() end -- the clip is shorter than the charge: play it again
  else
    ctx.go("Release")
  end
end

function M.release_enter(ctx)
  ctx.velocity(BASE_SPEED + SPEED_PER_FRAME * ctx.state.charge, 0)
end

function M.release_frame(ctx)
  ctx.hitbox_damage(3, BASE_DAMAGE + DAMAGE_PER_FRAME * ctx.state.charge)
end

return M
