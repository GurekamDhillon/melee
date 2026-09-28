-- Standalone Lua contract tests; no game, rendering or engine binary involved.
local commands, files, models, serial, loads = {}, {}, {}, 0, 0
local active, online, flying, fail_spawn = true, false, false, false
local pad, keys, overlay, reject_rotation = {}, {}, false, false
local safe_w = 640
local p = {x=13, y=26, z=0}
local mouse_state = {x=-1000, y=-1000, buttons=0}
local camera = {eye={x=0,y=0,z=100}, interest={x=0,y=0,z=0}, fov=30, roll=0, mode='standard'}
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
  project=function(x,y) return x,y,true end, line=function() end, box=function() end, fill=function() end,
  safe_area=function() return {x=0,y=0,w=safe_w,h=480} end,
  mouse=function() return mouse_state.x, mouse_state.y, mouse_state.buttons, 0 end,
  camera_get=function() return camera end,
  kit={available=function() return true end,panel=function() end,text=function() end,
       paragraph=function() return 1, 16 end,
       button=function() return 20 end,
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
-- New surface: tools, the mouse->plane mapping, scale/mirror transforms and the help overlay.
command('tool place')
mouse_state={x=320, y=240, buttons=1} on_frame_pre()
mouse_state={x=320, y=240, buttons=0} on_frame_pre()
command('save mouse.lua')
local placed=assert(load(files['mouse.lua'],'','t',{}))().parts
assert(#placed==3, 'LMB with the place tool adds one part')
assert(math.abs(placed[3].x-320)<7 and math.abs(placed[3].y-240)<7, 'mouse place maps the pointer to the depth plane')
assert(placed[3].scale==nil and placed[3].scale_x==nil, 'unit scale is omitted from the file')
command('tool scale')
command('scale 2') command('mirror x')
command('save mirrored.lua')
local mirrored=assert(load(files['mirrored.lua'],'','t',{}))().parts
assert(#mirrored==3, 'mirror keeps the part count')
assert(mirrored[3].scale==2 and mirrored[3].scale_x==-1, 'scale and mirror reach the document')
command('unscale')
command('save unscaled.lua')
local un=assert(load(files['unscaled.lua'],'','t',{}))().parts[3]
assert((un.scale or 1)==1 and (un.scale_x or 1)==1, 'reset scale restores the axes')
command('tool move') command('snap off') command('snap on') command('tool place')
keys={F1=true} on_tick() keys={} on_draw() -- the help overlay draws through the kit
keys={F1=true} on_tick() keys={}
for _=1,4 do command('undo') end
command('save back.lua')
assert(#assert(load(files['back.lua'],'','t',{}))().parts==2, 'undo unwinds the tool edits')
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
-- P0 hybrid controls (bible §4.3): modal transforms, tap-to-switch, axis lock, frame, snap, errors.
active=false on_match_end()
active=true models={} on_match_start()
command('on') command('clear')
local function saved(name) command('save '..name) return assert(load(files[name],'','t',{}))().parts end
command('tool place')
mouse_state={x=320, y=240, buttons=1} on_frame_pre()
mouse_state={x=320, y=240, buttons=0} on_frame_pre()
assert(count()==1, 'P0 baseline placement')
command('tool select') command('select')
local before=saved('p0a.lua')[1]
keys={G=true} on_tick() keys={} on_tick()
assert(count()==1, 'tap G must switch tool, not transform')
keys={G=true} on_tick()
mouse_state={x=380, y=240, buttons=0} on_frame_pre() on_tick()
keys={} on_tick()
local moved=saved('p0b.lua')[1]
assert(moved.x~=before.x, 'hold G must move the selection')
command('undo')
assert(saved('p0c.lua')[1].x==before.x, 'a modal move must be one undo step')
command('redo')
local keep=saved('p0d.lua')[1]
keys={G=true} on_tick()
mouse_state={x=420, y=260, buttons=0} on_frame_pre() on_tick()
keys={G=true, ESCAPE=true} on_tick() keys={} on_tick()
assert(saved('p0e.lua')[1].x==keep.x, 'ESC must cancel the modal')
command('undo')
assert(saved('p0f.lua')[1].x==before.x, 'a cancelled modal must leave history alone')
command('redo')
local lock_base=saved('p0g.lua')[1]
keys={SHIFT=true, C=true} on_tick() keys={} on_tick()
keys={G=true} on_tick()
mouse_state={x=450, y=320, buttons=0} on_frame_pre() on_tick()
keys={} on_tick()
local locked=saved('p0h.lua')[1]
assert(locked.y==lock_base.y and locked.x~=lock_base.x, 'axis lock must constrain the modal to X')
keys={SHIFT=true, C=true} on_tick() keys={} on_tick()
keys={SHIFT=true, C=true} on_tick() keys={} on_tick()
keys={F=true} on_tick() keys={}
assert(math.abs(p.x-locked.x)<0.01 and math.abs(p.y-locked.y)<0.01, 'F must move the fly cursor to the selection')
command('tool place')
keys={Z=true} on_tick() keys={} on_tick()
mouse_state={x=333, y=240, buttons=1} on_frame_pre() mouse_state={x=333, y=240, buttons=0} on_frame_pre()
local free=saved('p0i.lua')
assert(math.abs(free[#free].x-333)<0.01, 'snap off must keep the raw pointer')
keys={Z=true} on_tick() keys={} on_tick()
command('tool place')
keys={SHIFT=true} mouse_state={x=333, y=240, buttons=1} on_frame_pre()
mouse_state={x=333, y=240, buttons=0} on_frame_pre() keys={}
local fine=saved('p0j.lua')
assert(math.abs(fine[#fine].x-333.125)<0.01, 'Shift must quarter the snap step')
reject_rotation=true command('rotate 90') on_draw()
assert(#saved('p0k.lua')==#fine, 'a rejected rotate must not change the document')
command('off')
-- P1 palette: filter, recents, typed filter mode (bible §5.3)
active=false on_match_end()
active=true models={} on_match_start()
command('on') command('clear') command('tool place')
command('filter wall')
command('place')
local p1a=saved('p1a.lua')
assert(p1a[#p1a].part:find('^bf_wall'), 'filter must restrict placement to wall parts')
command('filter')
command('place')
local p1b=saved('p1b.lua')
assert(p1b[#p1b].part==p1a[#p1a].part, 'recents pin the last-used part at the top of the palette')
keys={F4=true} on_tick() keys={} on_tick()
for _,c in ipairs({'W','A','L','L'}) do keys={[c]=true} on_tick() keys={} on_tick() end
command('place') on_draw()
local p1c=saved('p1c.lua')
assert(p1c[#p1c].part:find('^bf_wall'), 'typed filter narrows the palette')
keys={ESCAPE=true} on_tick() keys={} on_tick()
command('off')
-- P1 inspector: selection rows, click toggles collision/flags (bible §5.7)
active=false on_match_end()
active=true models={} on_match_start()
command('on') command('clear') command('tool place') command('place')
local q1=saved('p2a.lua')[1]
assert(q1.collision~=false, 'default collision is on')
mouse_state={x=450, y=200, buttons=1} on_frame_pre() mouse_state={x=450, y=200, buttons=0} on_frame_pre()
local q2=saved('p2b.lua')[1]
assert(q2.collision==false, 'inspector click toggles collision')
mouse_state={x=450, y=222, buttons=1} on_frame_pre() mouse_state={x=450, y=222, buttons=0} on_frame_pre()
local q3=saved('p2c.lua')[1]
assert(q3.floor_flags==0, 'inspector click cycles floor flags')
on_draw()
command('off')
-- Widescreen: the layout follows gd.safe_area (bible §4.6)
active=false on_match_end()
active=true models={} on_match_start()
command('on') command('clear') command('tool place') command('place')
mouse_state={x=750, y=200, buttons=1} on_frame_pre() mouse_state={x=750, y=200, buttons=0} on_frame_pre()
assert(saved('p4a.lua')[1].collision==true, 'beyond 640 nothing is clickable at 4:3')
safe_w=800
mouse_state={x=750, y=200, buttons=1} on_frame_pre() mouse_state={x=750, y=200, buttons=0} on_frame_pre()
assert(saved('p4b.lua')[1].collision==false, 'the inspector sits at the right edge of an 800-wide canvas')
safe_w=640
on_draw()
command('off')
-- P1 gizmos + typed entry: handles transform, inspector typing (bible §5.2, §5.7)
active=false on_match_end()
active=true models={} on_match_start()
command('on') command('clear') command('tool place') command('place')
command('set x 500') command('set y 300')
mouse_state={x=450, y=100, buttons=1} on_frame_pre()
mouse_state={x=450, y=100, buttons=0} on_frame_pre()
for _,c in ipairs({'1','2','PERIOD','5'}) do keys={[c]=true} on_tick() keys={} on_tick() end
keys={ENTER=true} on_tick() keys={} on_tick()
assert(math.abs(saved('p6b.lua')[1].x-12.5)<0.01, 'typed entry applies the value')
command('set x 500') command('set y 300')
on_draw()
mouse_state={x=510, y=300, buttons=1} on_frame_pre()
mouse_state={x=600, y=300, buttons=1} on_frame_pre()
mouse_state={x=600, y=300, buttons=0} on_frame_pre()
local g1=saved('p6c.lua')[1]
assert(g1.x~=500 and g1.y==300, 'the red X handle moves along X only')
command('undo')
assert(saved('p6d.lua')[1].x==500, 'a handle drag is one undo step')
command('redo')
command('set x 500') command('set y 300') command('set rot 0')
on_draw()
mouse_state={x=514, y=286, buttons=1} on_frame_pre()
mouse_state={x=530, y=270, buttons=1} on_frame_pre()
mouse_state={x=530, y=270, buttons=0} on_frame_pre()
assert(math.abs(saved('p6e.lua')[1].rot+45)<0.01, 'the gold ring rotates, snapped')
command('set x 500') command('set y 300') command('set rot 0')
on_draw()
mouse_state={x=500, y=266, buttons=1} on_frame_pre()
mouse_state={x=540, y=266, buttons=1} on_frame_pre()
mouse_state={x=540, y=266, buttons=0} on_frame_pre()
local g2=saved('p6f.lua')[1]
assert(math.abs((g2.scale or 1)-1.54)<0.06, 'the cyan handle scales')
command('off')
print('map_editor_test: PASS')
