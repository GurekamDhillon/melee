-- @name: Render determinism regression
-- @gameplay: true
-- @rollback_safe: false
-- Run with MELEE_RENDER_DET_TEST=1, MELEE_SCRIPTS=0 and this file as
-- MELEE_PAD_SCRIPT in a vanilla offline match:
--   mode=vs;at=match;p1=falcon/human;p2=fox/human;p3=none;p4=none;
--   stage=fd;match=time;minutes=0;items=off
-- Use run.sh --realtime --max-seconds 180 (without the native --test flag).
-- gd.render_test(1) presents with one logic iteration per outer tick. Mode 0
-- skips presentation and batches two logic iterations, including canonical
-- draw callbacks for both. Both modes retain the real, unmodified sim seed.
--
-- Save/load acknowledgements run before on_frame_pre (gs_apply_pending in
-- gw_Script_FramePre), so all trials inject input 1 in that same logic frame.
-- Only on_frame advances n: repeated pre hooks cannot shift the input script.
-- on_frame observes the completed simulation before its canonical draw; the
-- following sample therefore also detects any draw-side simulation mutation.
local FRAMES, SLOT, MAX_TICKS = 480, 1, 20000
local schedules = {1, 0, 1}
local phase, trial, n, ticks = 'wait_match', 0, 0, 0
local armed = false
local baseline_hash, baseline_seed, baseline_pad = {}, {}, {}
local initial_hash, initial_seed, initial_frame
local last_render_serial
local distinct, distinct_count, kick_frames, moved, origin_x, origin_y
local neutral = {buttons = 0, x = 0, y = 0}

-- Three Down+B presses with release windows, turns, movement, jump and attack.
-- Stick Y, rather than the D-pad Down button, selects Falcon Kick.
local sequence = {
    {20, 0,   0,    0},
    { 6, 'B', 0, -127},
    {80, 0,   0,    0},
    { 6, 0, -80,    0},
    {20, 0,   0,    0},
    { 6, 'B', 0, -127},
    {80, 0,   0,    0},
    { 4, 'X', 0,    0},
    { 6, 'A',40,    0},
    {60, 0,   0,    0},
    {12, 0,  60,    0},
    {20, 0,   0,    0},
    { 6, 'B', 0, -127},
    {80, 0,   0,    0},
    {12, 0, -60,    0},
    {62, 0,   0,    0},
}

local function finish(ok, why)
    if phase == 'done' then return end
    phase, armed = 'done', false
    -- Cleanup should not hide the original failure (including a missing API).
    if type(gd.render_test) == 'function' then pcall(gd.render_test, -1) end
    pcall(gd.release, 1)
    pcall(gd.release, 2)
    gd.log('TEST render_determinism ' .. (ok and 'PASS ' or 'FAIL ') .. why)
    gd.quit()
end

local function guarded(fn, ...)
    if phase == 'done' then return end
    local ok, why = pcall(fn, ...)
    if not ok then finish(false, tostring(why)) end
end

local function sample(presents)
    assert(type(gd.render_test) == 'function', 'gd.render_test is unavailable')
    local hash, seed, applied_pad, render_serial = gd.render_test(presents)
    assert(type(hash) == 'string' and hash:match('^%x%x%x%x%x%x%x%x$'),
           'render_test did not return an eight-digit simulation hash')
    assert(type(seed) == 'number' and seed >= 0 and seed <= 0xFFFFFFFF
           and seed == math.floor(seed), 'render_test did not return the raw seed')
    assert(type(applied_pad) == 'number' and applied_pad >= 0 and applied_pad <= 0xFFFFFFFF
           and applied_pad == math.floor(applied_pad),
           'render_test did not return the applied PADStatus hash')
    assert(type(render_serial) == 'number' and render_serial >= 0
           and render_serial <= 0xFFFFFFFF and render_serial == math.floor(render_serial),
           'render_test did not return the canonical-render completion count')
    return hash, seed, applied_pad, render_serial
end

local function players()
    local p, q = gd.player(1), gd.player(2)
    assert(p and p.char_name == 'Captain Falcon', 'P1 must be Captain Falcon')
    assert(q, 'P2 must be present')
    assert(not p.cpu and not q.cpu, 'both players must use human ports')
    return p, q
end

local function pad_for_frame(frame)
    for _, part in ipairs(sequence) do
        if frame <= part[1] then
            return {buttons = part[2], x = part[3], y = part[4]}
        end
        frame = frame - part[1]
    end
    error('input sequence exhausted')
end

local function begin_trial(next_trial)
    local m = gd.match()
    assert(m and m.active and not m.netplay, 'an offline match is required')
    local p = players()
    -- The native applied-pad trace is deliberately not snapshotted; compare it
    -- only after this trial has completed a logic frame with its own input.
    local hash, seed = sample(schedules[next_trial])
    if next_trial == 1 then
        initial_hash, initial_seed, initial_frame = hash, seed, m.frame
    else
        assert(m.frame == initial_frame,
               string.format('load frame differs: expected=%d actual=%d', initial_frame, m.frame))
        assert(hash == initial_hash and seed == initial_seed,
               string.format('load differs: hash=%s/%s seed=%08X/%08X',
                             initial_hash, hash, initial_seed, seed))
    end
    trial, n, armed = next_trial, 0, false
    distinct, distinct_count, kick_frames, moved = {}, 0, 0, false
    origin_x, origin_y = p.x, p.y
    phase = 'running'
    gd.log(string.format('RENDER_DET begin trial=%d presents=%d match_frame=%d hash=%s seed=%08X',
                         trial, schedules[trial], m.frame, hash, seed))
end

function on_savestate(slot)
    guarded(function()
        if phase ~= 'saving' or slot ~= SLOT then return end
        begin_trial(1)
    end)
end

function on_loadstate(slot)
    guarded(function()
        if phase ~= 'loading' or slot ~= SLOT then return end
        begin_trial(trial + 1)
    end)
end

function on_frame_pre()
    guarded(function()
        if phase == 'running' then
            -- A one-frame hold is consumed by FramePost, once per logic frame;
            -- catch-up frames each receive their own override before procs.
            gd.input(1, pad_for_frame(n + 1), 1)
            gd.input(2, neutral, 1)
            armed = true
        elseif phase == 'wait_match' then
            local m = gd.match()
            if m and m.active and not m.netplay then
                gd.input(1, neutral, 1)
                gd.input(2, neutral, 1)
            end
        end
    end)
end

function on_frame()
    guarded(function()
        local m = gd.match()
        if phase == 'wait_match' then
            if not m or not m.active or m.frame <= 120 then return end
            assert(not m.netplay, 'netplay is not a valid determinism fixture')
            players()
            local total = 0
            for _, part in ipairs(sequence) do total = total + part[1] end
            assert(total == FRAMES, 'input sequence length differs from FRAMES')
            sample() -- Fail early if the test-only API/environment is absent.
            phase = 'saving'
            gd.savestate(SLOT)
            return -- Never sample the frame which merely requested a save.
        end
        if phase ~= 'running' then return end
        assert(armed, 'on_frame ran without this trial injecting an input')
        assert(m and m.active and not m.netplay, 'offline match ended during trial')
        local frame = n + 1
        assert(m.frame == initial_frame + frame,
               string.format('logic frame differs: trial=%d frame=%d match_frame=%d',
                             trial, frame, m.frame))
        local hash, seed, applied_pad, render_serial = sample()
        -- Hashes can miss an omitted draw in a scene whose effects happen to be
        -- insensitive. Independently require the prior frame's canonical walk.
        assert(frame == 1 or render_serial ~= last_render_serial,
               string.format('canonical draw missing: trial=%d frame=%d', trial, frame))
        last_render_serial = render_serial
        local p = players()
        local motion = p.motion_name or gd.motion_name(p.action, 1) or ''
        if motion:match('^SpecialLw') or motion:match('^SpecialAirLw') then
            kick_frames = kick_frames + 1
        end
        if p.x ~= origin_x or p.y ~= origin_y then moved = true end
        if not distinct[hash] then
            distinct[hash], distinct_count = true, distinct_count + 1
        end
        gd.log(string.format('RENDER_DET trial=%d presents=%d frame=%d hash=%s seed=%08X pad=%08X motion=%s renders=%u',
                             trial, schedules[trial], frame, hash, seed, applied_pad, motion, render_serial))
        if trial == 1 then
            baseline_hash[frame], baseline_seed[frame], baseline_pad[frame] = hash, seed, applied_pad
        else
            assert(applied_pad == baseline_pad[frame],
                   string.format('input diverged trial=%d presents=%d frame=%d pad=%08X/%08X',
                                 trial, schedules[trial], frame, baseline_pad[frame], applied_pad))
            assert(hash == baseline_hash[frame] and seed == baseline_seed[frame],
                   string.format('diverged trial=%d presents=%d frame=%d hash=%s/%s seed=%08X/%08X',
                                 trial, schedules[trial], frame, baseline_hash[frame], hash,
                                 baseline_seed[frame], seed))
        end
        n, armed = frame, false
        if n < FRAMES then return end
        assert(distinct_count > 1, 'simulation hash never changed')
        assert(moved, 'fighter never moved')
        -- gd.fx() exposes Geno emitters, not the retail HSD particle list.
        -- Require the actual native move that requests Falcon's flame effect
        -- in ftCa_SpecialHi_800E3EAC, rather than count intended button presses.
        assert(kick_frames > 0, 'no Falcon Kick motion was observed')
        gd.log(string.format('RENDER_DET end trial=%d frames=%d distinct=%d kick_frames=%d',
                             trial, n, distinct_count, kick_frames))
        if trial == #schedules then
            finish(true, 'frames=' .. FRAMES .. ' schedules=1,0,1')
        else
            phase = 'loading'
            gd.loadstate(SLOT)
            return -- Acknowledge the load before sending or sampling another input.
        end
    end)
end

function on_tick()
    guarded(function()
        ticks = ticks + 1
        assert(ticks <= MAX_TICKS,
               string.format('timeout phase=%s trial=%d frame=%d', phase, trial, n))
    end)
end

function on_match_end()
    if phase ~= 'wait_match' and phase ~= 'done' then
        finish(false, 'match ended before all trials completed')
    end
end
