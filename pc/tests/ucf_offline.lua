-- @name: ucf-offline
-- @gameplay: true
-- Offline UCF 0.84 vs vanilla 1.02 (Controller Fix setting / MELEE_UCF=off|0.84).
-- Run twice, once per mode, with --idle-cpus and
--   MELEE_SCENE="mode=lab;p1=fox;p2=marth;stage=bf;time=0"
-- and read the "UCFTEST" lines:
--   MELEE_UCF=0.84 -> "UCFTEST mode=on  dashback=1 shield_drop=1 PASS"
--   MELEE_UCF=off  -> "UCFTEST mode=off dashback=0 shield_drop=0 PASS"
-- dashback: Wait, flick left through one half-way sample (0, -40, -80). UCF 0.84 turns that into a
--   dash at once; vanilla waits (Turn lasts several frames longer).
-- shield drop: on a Battlefield platform, shield with the stick at (50,-62) after holding x: UCF drops
--   through the platform (Pass); vanilla spot-dodges (EscapeN).
local ticks, done = 0, false
function on_tick()
    ticks = ticks + 1
    if not done and ticks > 9000 then gd.log('UCFTEST FAIL watchdog') done = true gd.quit() end
end
local function name(a) return gd.motion_name(a, 1) or tostring(a) end
-- play segments {x,y,cx,cy,l,buttons,frames}; returns the list of {action name, frame}
local function play(segs)
    local out, last, f = {}, nil, 0
    for _, s in ipairs(segs) do
        for _ = 1, s[7] do
            gd.input(1, {buttons = s[6] or 0, x = s[1], y = s[2], cx = s[3] or 0, cy = s[4] or 0, l = s[5] or 0}, 1)
            gd.wait(1) f = f + 1
            local p = gd.player(1)
            local k = p.action .. ':' .. p.facing
            if k ~= last then out[#out + 1] = {name(p.action), f, p.facing} last = k end
        end
    end
    return out
end
local function first(seq, nm) for _, e in ipairs(seq) do if e[1] == nm then return e[2] end end end
local function show(seq) local t = {} for _, e in ipairs(seq) do t[#t + 1] = e[1] .. '@' .. e[2] end return table.concat(t, ' ') end
local function reset_right()
    for _ = 1, 60 do gd.input(1, {x = 0, y = 0}, 1) gd.wait(1) end
    for _ = 1, 4 do gd.input(1, {x = 80, y = 0}, 1) gd.wait(1) end
    for _ = 1, 40 do gd.input(1, {x = 0, y = 0}, 1) gd.wait(1) end
end
gd.run(function()
    gd.input(1, 0, 1) gd.input(2, 0, 1)
    gd.wait_until(function() return gd.match().active and gd.match().frame > 150 end, 3000)
    reset_right()
    local d = play({{0,0,0,0,0,0,5},{-40,0,0,0,0,0,1},{-80,0,0,0,0,0,25}})
    local turn, dash = first(d, 'Turn'), first(d, 'Dash')
    local dashback = (turn and dash and dash - turn <= 2) and 1 or 0
    gd.log('UCFTEST dashback seq: ' .. show(d))
    gd.teleport(1, -57.6, 32) gd.wait(40)
    local s = play({{0,0,0,0,0,0,5},{0,0,0,0,200,gd.buttons.L,6},{50,0,0,0,200,gd.buttons.L,8},{50,-62,0,0,200,gd.buttons.L,12}})
    gd.log('UCFTEST shield seq: ' .. show(s))
    local drop = first(s, 'Pass') and not first(s, 'EscapeN')
    local dodge = first(s, 'EscapeN') and not first(s, 'Pass')
    local sd = drop and 1 or 0
    local ok = (dashback == 1 and drop) or (dashback == 0 and dodge)
    gd.log(('UCFTEST mode=%s dashback=%d shield_drop=%d %s'):format(dashback == 1 and 'on' or 'off', dashback, sd, ok and 'PASS' or 'FAIL (mixed)'))
    done = true
    gd.quit()
end)
