-- New authoring doubles cells/models; stored layouts retain explicit transforms.
local files,models,serial,commands={}, {},0,{}
local p={x=8,y=21}
gd={command=function(n,f)commands[n]=f end,log=function()end,
 match=function()return {active=true,netplay=false}end,player=function()return p end,
 key=function()return false end,fly=function(_,v)return v~=nil and v or false end,
 model_load=function(n)return n end,model_release=function()end,
 model_spawn=function(a,o)serial=serial+1;models[serial]=o;return serial end,
 model_set=function(h,o)models[h]=o;return true end,
 model_despawn=function(h)models[h]=nil;return true end,
 stage_view=function()return true,false end,stage_restore_bounds=function()return true end,
 data_read=function(n)return files[n]end,data_write=function(n,t)files[n]=t end}
assert(loadfile('pc/scripts/examples/map_editor/scripts/main.lua'))()
local function cmd(t)commands.map(t)end
local function saved(n)cmd('save '..n);return assert(load(files[n],'layout','t',{}))()end
cmd('on');cmd('ghost off');cmd('place')
local new=saved('new.lua')
assert(new.units==6.5,'exported asset unit must stay fixed')
assert(new.parts[1].x==13 and new.parts[1].y==26,'new cell is 13 world units')
assert(new.parts[1].scale==2 and not new.parts[1].scale_x,'uniform XYZ scale stored once')
cmd('unscale')
assert(saved('reset.lua').parts[1].scale==2,'reset restores doubled authoring scale')
files['old.lua']='return {version=1,units=6.5,parts={{part="bf_floor_4m",x=6.5,y=19.5,z=3.25,rot=15,collision=true,floor_flags=2},{part="bf_wall_solid_4m",x=-9,y=7,z=2,rot=0,collision=true,floor_flags=2,scale=.75,scale_x=-1,scale_y=1.25,scale_z=.5}}}'
cmd('load old.lua')
local old=saved('old-back.lua')
assert(old.parts[1].x==6.5 and old.parts[1].y==19.5 and old.parts[1].z==3.25 and old.parts[1].rot==15 and not old.parts[1].scale)
assert(old.parts[2].x==-9 and old.parts[2].scale==.75 and old.parts[2].scale_x==-1 and old.parts[2].scale_y==1.25 and old.parts[2].scale_z==.5)
cmd('duplicate')
local dupe=saved('duplicate.lua');assert(dupe.parts[3].scale==.75 and dupe.parts[3].scale_x==-1,'duplicate retains imported scale')
cmd('place')
assert(saved('added.lua').parts[4].scale==2,'new placement after import uses authoring scale')
for _,expect in ipairs({6.5,9.75,13}) do
 cmd('run Grid 2 / 1 / 0.5');p.x=9;p.y=9;cmd('place')
 local after=saved('cycle.lua');assert(after.parts[#after.parts].x==expect,'grid cycle mismatch')
end
print('map editor doubled authoring preserves imported transforms passed')
