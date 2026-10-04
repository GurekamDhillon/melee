-- Offline acceptance companion; enable the separate vanilla-hero fighter mod.
-- Host controls only. No fighter Lua simulation is introduced by slice 1.
local saved
function on_tick()
    if not gd.match().active then return end
    if gd.key_pressed('S') then gd.savestate(1) end
    if gd.key_pressed('L') then gd.loadstate(1) end
    if gd.key_pressed('P') then gd.pause() end
    if gd.key_pressed('N') then gd.step(1) end
    if gd.key_pressed('R') then gd.resume() end
end
function on_savestate(slot)
    if slot == 1 and gd.player(1) then
        saved = gd.player(1).percent
        gd.set_percent(1, saved + 25)
        gd.log('Hero snapshot saved; damage changed +25. Load must restore original.')
    end
end
function on_loadstate(slot)
    if slot == 1 and saved and gd.player(1) then
        gd.log('Hero snapshot damage restore: '..tostring(gd.player(1).percent == saved))
    end
end
