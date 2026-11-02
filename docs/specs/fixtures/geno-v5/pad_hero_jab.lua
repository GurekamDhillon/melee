-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 GurekamDhillon
-- @name: Geno spec fixture pad (vanilla-hero walk + jab, per-frame state log)
-- @gameplay: true
-- @rollback_safe: false
-- Drives P1 (geno:vanilla-hero) and P2 (retail Mario, idle) for FRAMES logic frames with fixed,
-- frame-indexed input and logs one GENOFIX line per frame. No OS input: gd.input only.
-- Input is a function of the match frame count alone, so two runs are identical.
-- Columns (also the header of state_hero_jab.csv):
--   frame,p1_kind,p1_action,p1_action_frame,p1_anim_frame,p1_x,p1_y,p1_vx,p1_vy,p1_percent,p1_facing,
--   p1_airborne,p1_hitlag,p2_action,p2_percent,p2_x,p2_y,p2_hitlag
local FRAMES = 300
local n = 0
local function p1_input(f)
    if f <= 100 then return {} end                                  -- entry and landing
    if f <= 205 then return {x = 50} end                            -- walk (half stick) forward 105 frames
    if f == 215 or f == 245 or f == 275 then return {buttons = "A"} end  -- three jab presses
    return {}
end
local function fl(v) return string.format("%.6f", v or 0) end
function on_frame_pre()
    local m = gd.match()
    if not (m and m.active) then return end
    n = n + 1
    gd.input(1, p1_input(n), 2)
    gd.input(2, {}, 2)
    local a, b = gd.player(1), gd.player(2)
    if a and b then
        gd.log(string.format("GENOFIX %d,%d,%d,%d,%s,%s,%s,%s,%s,%s,%s,%d,%d,%d,%s,%s,%s,%d", n,
            a.kind or -1, a.action or -1, a.action_frame or -1, fl(a.anim_frame), fl(a.x), fl(a.y), fl(a.vx), fl(a.vy),
            fl(a.percent), fl(a.facing), a.airborne and 1 or 0, a.hitlag or 0,
            b.action or -1, fl(b.percent), fl(b.x), fl(b.y), b.hitlag or 0))
    end
    if n == FRAMES then
        gd.log("GENOFIX quit at frame " .. n)
        gd.quit()
    end
end
