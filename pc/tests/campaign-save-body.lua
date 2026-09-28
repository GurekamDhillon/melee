-- Bundled by bundle_campaign_save.py; all failures terminate unattended runs.
local section, phase, ticks, boundary = 0, 'start', 0, false
local save, first_stage, first_epoch
local function check(ok, detail)
    section = section + 1
    gd.log(('TEST campaign-save section %d: %s %s'):format(section, ok and 'PASS' or 'FAIL', detail))
    assert(ok, detail)
end
local initial = {area='forest', entry='start', party={'fox','falco'},
    unlocks={gate=true}, collectibles={coin=3}, progress={quest=1}, cleared={}}
local function finish()
    local got, source = save:scene()
    check(source == 'primary' and got.area == 'cave' and got.entry == 'west', 'scene-boundary location')
    check(got.party[2] == 'falco' and got.unlocks.gate and got.collectibles.coin == 9
        and got.checkpoint.area == 'forest' and got.progress.quest == 2, 'all campaign fields survive scene')
    gd.data_write(save:filename(save.slot), 'corrupt campaign record')
    got, source = save:scene()
    check(source == 'fallback' and got.area == 'forest' and got.collectibles.coin == 3, 'corrupt newest falls back')
    local dying = save:get(); dying.collectibles.coin = 100; dying.area = 'cave'
    save:commit('death', dying)
    check(save:get().area == 'forest' and save:get().collectibles.coin == 3, 'death discards uncommitted gains')
    local quit = save:get(); quit.collectibles.coin = 7
    save:commit('quit', quit)
    check(save:scene().collectibles.coin == 7, 'explicit quit commits progress')
    gd.data_write(save:filename(1), 'broken')
    gd.data_write(save:filename(2), 'broken')
    got, source = save:scene()
    check(source == 'defaults' and got.area == 'forest', 'both corrupt use defaults')
    local legacy = CampaignSave.copy(initial); legacy.room = legacy.area; legacy.area = nil
    gd.campaign_storage(save:filename(1), CampaignSave.record(legacy, 1, 1, 'campaign-test', 1))
    got, source = save:scene()
    check(source == 'fallback' and got.schema == 2 and got.area == 'forest', 'schema 1 room migrates to area')
    local before = save:get()
    local invalid = save:get(); invalid.party = 'invalid'
    check(not pcall(function() save:commit('quit', invalid) end)
        and save:get().area == before.area, 'invalid save leaves committed state intact')
    gd.log('TEST campaign-save complete')
    phase = 'done'; gd.quit()
end
function on_scene()
    if phase == 'changing' then boundary = true end
end
function on_tick()
    if phase == 'done' then return end
    local ok, err = pcall(function()
        ticks = ticks + 1
        if ticks > 3600 then check(false, 'timeout waiting for LAB scene transition') end
        if gd.scene().name == 'GS_MEMCARD' then gd.input(1, {buttons='A'}, 2); return end
        if not gd.match().active then return end
        if phase == 'start' then
            check(gd.lab_mode(), 'running in LAB')
            gd.input(1, {}, 1) -- retain a neutral claim; no physical input is needed
            save = CampaignSave.new{id='campaign-test', version=1, slot='campaign-test', defaults=initial}
            -- Dedicated test slot, deliberately reset for repeatable batch runs.
            gd.data_write(save:filename(1), 'reset'); gd.data_write(save:filename(2), 'reset')
            save:scene()
            save:commit('checkpoint', initial)
            local nextroom = save:get()
            nextroom.area, nextroom.entry, nextroom.collectibles.coin, nextroom.progress.quest = 'cave','west',9,2
            save:commit('room_exit', nextroom)
            check(save:get().area == 'cave', 'save committed')
            first_stage, first_epoch = gd.match().stage, gd.scene().epoch
            phase = 'changing'
            gd.scene_launch{mode='lab', p1='fox', p2='falco/cpu', stage='battlefield'}
        elseif phase == 'changing' and boundary and gd.scene().epoch > first_epoch
            and gd.match().stage ~= first_stage then
            check(gd.lab_mode(), 'new LAB scene reached without input')
            finish()
        end
    end)
    if not ok then
        gd.log(('TEST campaign-save section %d: FAIL %s'):format(section + 1, tostring(err)))
        phase = 'done'; gd.quit()
    end
end
