-- Standalone Lua contract tests; no game, rendering or engine binary involved.
local commands, files, models, serial, loads = {}, {}, {}, 0, 0
local active, online, flying, fail_spawn = true, false, false, false
local pad, keys, overlay, reject_rotation = {}, {}, false, false
local p = {x=13, y=26, z=0}
local function copy(t) local o={} for k,v in pairs(t) do o[k]=v end return o end
gd = {
  command=function(n,f) commands[n]=f end, log=function() end,
  match=function() return {active=active, netplay=online} end,
  player=function() return p end,
  fly=function(_, v) if v~=nil then flying=v end return flying end,
  model_load=function(n) loads=loads+1 return n end, model_release=function() end,
  model_spawn=function(a,o)
    if fail_spawn then fail_spawn=false return nil,'capacity' end
    serial=serial+1 models[serial]=copy(o) return serial
  end,
  model_set=function(h,o)
    assert(models[h], 'stale handle')
    if reject_rotation then reject_rotation=false error('reversed floor') end
    models[h]=copy(o) return true
  end,
  model_get=function(h) return models[h] end,
  model_despawn=function(h) models[h]=nil return true end,
  stage_view=function(_,v) if v~=nil then overlay=v end return true,overlay end,
  data_write=function(n,s) files[n]=s end, data_read=function(n) return files[n] end,
  key=function(k) return keys[k] end, key_pressed=function(k) return keys[k] end,
  pad=function() return pad end, time=function() return 0 end,
  fly_speed=function() return 2 end, teleport=function(_,x,y) p.x,p.y=x,y end,
  project=function(x,y) return x,y,true end, line=function() end, box=function() end,
  kit={available=function() return true end,panel=function() end,text=function() end,
       list=function(...) assert(select('#',...)==6, 'kit.list opts must be argument 6') end},
}
local chunk = loadfile('pc/scripts/examples/map_editor/scripts/main.lua')
assert(chunk, 'map editor implementation is missing')
chunk()
local function command(s) commands.map(s) end
local function count() local n=0 for _ in pairs(models) do n=n+1 end return n end
command('on')
assert(flying, 'editor must enable flight')
command('place') assert(count()==1)
command('save test.lua')
local first=assert(files['test.lua'])
local layout=assert(load(first, 'layout', 't', {}))()
assert(layout.version==1 and #layout.parts==1)
assert(layout.parts[1].x==13 and layout.parts[1].y==26, 'snap must use exported units')
command('duplicate') assert(count()==2)
command('undo') assert(count()==1)
command('redo') assert(count()==2)
fail_spawn=true command('duplicate') assert(count()==2, 'failed spawn must roll back')
command('save after.lua')
assert(#assert(load(files['after.lua'],'','t',{}))().parts==2)
files['bad.lua']='return {version=1,units=6.5,parts={{part="not_a_part",x=0,y=0,z=0,rot=0}}}'
command('load bad.lua') assert(count()==2, 'invalid load must preserve map')
command('load test.lua') assert(count()==1)
command('undo') assert(count()==2, 'load must be undoable')
command('delete') assert(count()==1)
command('undo') assert(count()==2)
reject_rotation=true command('rotate 180')
command('save rotation.lua')
for _,part in ipairs(assert(load(files['rotation.lua'],'','t',{}))().parts) do
  assert(part.rot==0, 'rejected transform must not modify document')
end
-- A failed multi-part load restores deleted originals, and does not advance history.
fail_spawn=true command('load test.lua') assert(count()==2)
command('save restore.lua') assert(files['restore.lua']==files['rotation.lua'])
files['malicious.lua']='gd.quit(); return {}'
command('load malicious.lua') assert(count()==2)
files['scale.lua']=first:gsub('units=6.5','units=7')
command('load scale.lua') assert(count()==2)
files['sparse.lua']='return {version=1,units=6.5,parts={[2]={}}}'
command('load sparse.lua') assert(count()==2)
command('save ../escape.lua') assert(not files['../escape.lua'])
command('save test.lua') assert(files['test.lua.bak']==first)
files['test.lua']=first
on_draw()
command('off') assert(not flying)
assert(not overlay, 'editor must restore overlay')
-- Controller edges: holding A places once; action menu reaches undo without keyboard.
pad={Z=true,UP=true} on_tick() assert(flying)
pad={} on_tick() pad={A=true} on_tick() on_tick() assert(count()==3)
pad={} on_tick() pad={Z=true} on_tick() pad={} on_tick()
for _=1,7 do pad={DOWN=true} on_tick() pad={} on_tick() end
pad={A=true} on_tick() assert(count()==2)
pad={} on_tick() pad={B=true} on_tick() pad={} on_tick()
keys={W=true} on_frame_pre() keys={} assert(p.y==28)
command('off')
online=true command('on') command('place') assert(count()==2 and not flying)
online=false command('play test.lua') assert(count()==1 and not flying)
command('on') on_savestate(1)
local native_snapshot={} for h,m in pairs(models) do native_snapshot[h]=copy(m) end
local snapshot_fly=flying
command('duplicate') assert(count()==2)
command('off')
models=native_snapshot flying=snapshot_fly
on_loadstate(1) assert(count()==1 and not flying, 'restored native fly must use snapshot editor restoration state')
command('save snapshot.lua') assert(#assert(load(files['snapshot.lua'],'','t',{}))().parts==1)
local retained_loads=loads
command('on') command('duplicate')
assert(loads==retained_loads, 'tracked savestate must preserve retained asset references')
active=false models={} on_match_end() assert(not overlay)
active=true on_match_start() assert(count()==2, 'resuming edits must cancel play autoload and retain unsaved changes')
on_loadstate(0) command('on') command('place') assert(count()==2, 'untracked state must block editing')
on_unload() assert(count()==0, 'unload must remove owned instances')
print('map_editor_test: PASS')
