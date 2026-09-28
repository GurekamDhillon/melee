-- Host-only Lua checks; no game, renderer, native DLL, or disk save access.
dofile('pc/scripts/lib/campaign_save.lua')
local files, fail_write, online, logs, quit, stage, epoch = {}, false, false, {}, false, 1, 1
gd = {
    campaign_storage=function(name, bytes)
        assert(not online, 'offline only')
        if not name then return true end
        if bytes then assert(not fail_write, 'injected write failure'); files[name]=bytes; return true end
        return files[name]
    end,
    data_write=function(name, bytes) files[name]=bytes end,
    log=function(s) logs[#logs+1]=s end,
    quit=function() quit=true end,
    scene=function() return {name='GS_TRAINING', epoch=epoch} end,
    match=function() return {active=true, netplay=online, stage=stage} end,
    lab_mode=function() return true end,
    input=function() end,
    scene_launch=function() stage=2; epoch=2 end,
}
-- Run the exact LAB body with only engine scene/storage surfaces substituted.
dofile('pc/tests/campaign-save-body.lua')
on_tick(); on_scene(); on_tick()
assert(quit, 'body did not quit')
for _,s in ipairs(logs) do assert(not s:find(': FAIL',1,true), s) end
local C = CampaignSave
local opts = {id='unit', version=1, slot='unit', defaults={area='a', party={'fox'}}}
local save = C.new(opts)
save:scene(); save:commit('checkpoint', save:get())
local old = files[save:filename(save.slot)]
local nextstate = save:get(); nextstate.area='b'
fail_write=true
assert(not pcall(save.commit,save,'room_exit',nextstate))
assert(save:get().area == 'a' and files[save:filename(save.slot)] == old)
fail_write=false
save:commit('room_exit',nextstate)
files[save:filename(save.slot)] = files[save:filename(save.slot)]:sub(1,-4)
assert(save:scene().area == 'a', 'truncation fallback')
files[save:filename(2)] = C.record(save:get(), 99, 50, 'unit', 1)
save:scene()
assert(save.blocked and not pcall(save.commit,save,'quit',save:get()), 'future schema must not be overwritten')
online=true
assert(not pcall(save.scene,save), 'online load refused, not defaulted')
online=false
local cyclic={}; cyclic.self=cyclic
assert(not pcall(C.copy,cyclic), 'cycle rejected')
files={}; save=C.new(opts); save:scene()
save:commit('checkpoint',save:get())
-- Optional read-only integration against the actual sibling Gamemode v1 source.
if arg[1] then
    dofile(arg[1])
    local blob, teleports = '', 0
    gd.mode_blob=function(bytes) if bytes then blob=bytes end; return blob end
    gd.teleport=function() teleports=teleports+1; return true end
    gd.camera_attach=function() end
    local mode=Gamemode.new{id='unit',version=1,start='a',areas={
        a={entries={start={x=0,y=0}},checkpoint=true},
        b={entries={start={x=20,y=0},west={x=10,y=0}}}}}
    local data=save:get(); data.area='b'; data.entry='west'; data.collectibles.coin=8
    save:commit('room_exit',data)
    local adapter=C.adapter(mode,save)
    adapter:scene()
    local ok,err=adapter:start(); assert(ok,err)
    assert(mode.state.area=='b' and mode.state.entry=='west' and teleports==1)
    assert(mode.state.progress.collectibles.coin==8 and mode.state.checkpoint.area=='a')
    assert(next(mode.state.enemies)==nil and mode.state.clock==0)
    adapter:commit('quit',{'fox','falco'})
    assert(save:scene().party[2]=='falco')
end
print('campaign-save host checks: PASS (mock storage/scene; native IO and game untested)')
