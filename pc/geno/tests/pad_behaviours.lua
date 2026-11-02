-- @name: Geno v5.1 behaviours acceptance pad
-- @gameplay: true
-- @rollback_safe: false
-- SPDX-License-Identifier: MIT
-- Copyright (c) 2026 GurekamDhillon
--
-- Drives port 1 (Kirby with the behaviour-test mod: tests/mods/behaviour-test) through the tornado, the
-- drill, the cape and a jump-hold glide with fixed, frame-indexed input; logs one GBLIVE line per
-- frame. tests/check_behaviours.py reads the log. No OS input: gd.input only.
local n = 0
local phase = 0
local pn = 0          -- frames in the phase
local idle = 0        -- consecutive frames out of every Geno state
local GENO0 = 1024    -- GENO_MOTION_BASE
local QUIT_AT = 2600

local function press(btn, x, y, frames) gd.input(1, {buttons = btn, x = x or 0, y = y or 0}, frames or 3) end

function on_frame_pre()
    local m = gd.match()
    if not (m and m.active) then return end
    n = n + 1
    local p = gd.player(1)
    if not p then return end
    local geno = p.action >= GENO0 and p.action < GENO0 + 32
    if geno then idle = 0 else idle = idle + 1 end
    pn = pn + 1
    -- the log has a line budget: every frame inside a Geno state or an air action, every 10th otherwise
    if geno or phase >= 4 or n % 10 == 0 then
        gd.log(string.format("GBLIVE %d ph %d act %d x %.3f y %.3f vx %.3f vy %.3f f %d",
            n, phase, p.action, p.x, p.y, p.vx, p.vy, p.facing))
    end
    -- phase 0: settle on the stage, then each move after a quiet spell
    if phase == 0 and n == 150 then phase = 1; pn = 0; press('B', 0, 0, 3)
    elseif phase == 1 and pn > 20 and idle >= 30 then phase = 2; pn = 0; press('B', 100, 0, 3)
    elseif phase == 2 and pn > 20 and idle >= 30 then phase = 3; pn = 0; press('B', 0, -100, 3)
    elseif phase == 3 and pn > 20 and idle >= 30 then phase = 4; pn = 0; press('X', 0, 0, 3)
    elseif phase == 4 and pn == 20 then phase = 5; pn = 0; press('X', 0, 0, 45)   -- midair jump, X held
    elseif phase == 5 and pn == 34 then phase = 6; pn = 0; press('A', 0, 0, 3)    -- glide attack
    elseif phase == 6 and pn > 200 then
        gd.log("GBLIVE quit at frame " .. n)
        gd.quit()
    end
    if n >= QUIT_AT then gd.log("GBLIVE timeout at frame " .. n); gd.quit() end
end
