-- Vanilla Riposte: the down special as a counter with a follow-up, written as Lua callbacks (Geno slice 5, docs/geno.md section 23).
--   Parry    a stance. A hit taken in action frames 4-24 is countered by the ENGINE (the window is declared in geno.json: the hit's
--            damage and knockback are dropped) and the fighter is sent to Riposte. Letting go of B after frame 8 cancels the stance;
--            running out the window is a whiff; both break the streak.
--   Riposte  the answer: damage 1.5 x the countered hit (clamped 9..30) plus one per hit of the streak (up to +3); turns around if the
--            hit came from behind. Press A in action frames 6-16 to chain into
--   Follow   a second strike for 0.6 x the riposte's damage. One follow-up per riposte.
-- The only state is ctx.state (power, streak, follow; declared in geno.json); the functions keep nothing between calls.
local WINDOW_END = 24           -- the last frame of the counter window in geno.json
local CANCEL_AFTER = 8          -- B may be let go from this frame on
local STANCE_END = 30           -- the stance's total length
local MIN_POWER, MAX_POWER, SCALE = 9, 30, 1.5
local MAX_STREAK = 4
local FOLLOW_FROM, FOLLOW_TO = 6, 16
local FOLLOW_SCALE = 0.6

local M = {}

function M.parry_enter(ctx)
  -- nothing to set: the streak survives from the last riposte until a whiff breaks it
end

function M.parry_frame(ctx)
  local s, f = ctx.state, ctx.self.action_frame
  if f > WINDOW_END and s.streak ~= 0 then s.streak = 0 end   -- the window has passed with no counter: a whiff
  if f >= STANCE_END then
    ctx.go("auto")
  elseif f >= CANCEL_AFTER and f <= WINDOW_END and not ctx.input.special_held then
    s.streak = 0
    ctx.go("auto")
  elseif ctx.self.anim_ended then
    ctx.loop()                                                 -- the clip is shorter than the stance: play it again
  end
end

function M.riposte_enter(ctx)
  local s = ctx.state
  s.streak = math.min(s.streak + 1, MAX_STREAK)
  s.follow = false
  s.power = math.min(MAX_POWER, math.max(MIN_POWER, SCALE * ctx.self.hit_damage)) + (s.streak - 1)
  if ctx.self.hit_from < 0 then ctx.turn() end                 -- the attacker is behind: face them
  ctx.velocity(0.9, 0)
end

function M.riposte_frame(ctx)
  local s, f = ctx.state, ctx.self.action_frame
  ctx.hitbox_damage(3, s.power)
  if ctx.input.attack_pressed and not s.follow and f >= FOLLOW_FROM and f <= FOLLOW_TO then
    s.follow = true
    ctx.go("Follow")
  end
end

function M.follow_enter(ctx)
  ctx.velocity(1.6, 0)
end

function M.follow_frame(ctx)
  ctx.hitbox_damage(3, FOLLOW_SCALE * ctx.state.power)
end

return M
