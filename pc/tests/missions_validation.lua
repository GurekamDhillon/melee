-- Structural refusal coverage against the real validator and unchanged pure machine.
local base='pc/scripts/examples/missions/scripts/'
local f=io.open(base..'mission.lua');if f then f:close() else base='melee/'..base end
local d={mission=assert(loadfile(base..'mission.lua'))()}
local v=assert(loadfile(base..'validator.lua'))()(d)
local n=0
local function check(name,fn)
  assert(not pcall(fn),'accepted invalid '..name);n=n+1;print('PASS refuse '..name)
end
local function layout()
  return {version=2,units=6.5,parts={{part='p',x=0,y=0,z=0,rot=0,collision=true,floor_flags=0}}}
end
local function bad(name,change)
  check(name,function()local x=layout();change(x);v.layout(x,{p='missions/a/models/p.gxmesh'})end)
end
check('non-table layout',function()v.layout(1,{})end)
bad('parts non-table',function(x)x.parts=false end)
bad('parts keyed list',function(x)x.parts.bad={}end)
bad('too many parts',function(x)for i=2,129 do x.parts[i]=x.parts[1]end end)
bad('non-table part',function(x)x.parts[1]=3 end)
bad('unsafe part path',function(x)x.parts[1].part='../p'end)
for _,key in ipairs({'x','y','z','rot'}) do bad('nonfinite '..key,function(x)x.parts[1][key]=1/0 end)end
for _,key in ipairs({'scale','scale_x','scale_y','scale_z'}) do
  for _,value in ipairs({0,0.0001,101,false}) do bad('scale '..key..' '..tostring(value),function(x)x.parts[1][key]=value end)end
end
bad('noninteger floor flags',function(x)x.parts[1].floor_flags=0.5 end)
bad('v1 embedded mission',function(x)x.version=1;x.mission={}end)
bad('nontable bounds',function(x)x.camera=false end)
bad('incomplete bounds',function(x)x.blast={left=0,right=1,top=2}end)
bad('reversed vertical bounds',function(x)x.camera={left=0,right=1,top=0,bottom=1}end)
bad('nontable spawn',function(x)x.spawn=false end)
bad('nonnumeric spawn slot',function(x)x.spawn={a={x=0,y=0}}end)
bad('nonfinite spawn point',function(x)x.spawn={[0]={x=0,y=1/0}}end)
bad('nontable lines',function(x)x.lines=3 end)
bad('line missing endpoint',function(x)x.lines={{x1=0,y1=0,x2=1,kind='floor'}}end)
bad('line kind',function(x)x.lines={{x1=0,y1=0,x2=1,y2=1,kind='other'}}end)
bad('line direction',function(x)x.lines={{x1=1,y1=0,x2=0,y2=0,kind='floor'}}end)
bad('line flag type',function(x)x.lines={{x1=0,y1=0,x2=1,y2=0,kind='floor',draw=1}}end)
bad('duplicate marker',function(x)x.markers={{name='a',x=0,y=0},{name='a',x=1,y=1}}end)
bad('marker wave',function(x)x.markers={{name='a',x=0,y=0,cleared_waves={9}}}end)
bad('marker checkpoint',function(x)x.markers={{name='a',x=0,y=0,checkpoint=0}}end)
bad('marker frame',function(x)x.markers={{name='a',x=0,y=0,frames=-1}}end)
local mission_bad={
  3,{unexpected=true},{start={x=0}},{start={x=0,y=0,z=0}},{goal={x=0,y=0,w=0,h=1}},
  {enemies={bad={}}},{enemies={{kind='goomba',x=0,y=0,wave=0}}},
  {checkpoints={{x=0,y=0,w=2001,h=1}}},
  {waves={{wave=1,time=0,x=0}}},{waves={{wave=1,time=-1}}},{waves={{wave=1,x=0,dir=0}}},
  {waves={{wave=1,time=0},{wave=1,time=1}}},
  {triggers={{x=0,y=0,w=1,h=1,action='other'}}},
  {triggers={{x=0,y=0,w=1,h=1,action='message',text='bad\n'}}},
  {triggers={{x=0,y=0,w=1,h=1,action='wave',wave=9}}},
  {triggers={{x=0,y=0,w=1,h=1,action='collision',at={x=0,y=0},open=true,r=0}}},
  {triggers={{x=0,y=0,w=1,h=1,action='message',text='x',wave=1}}},
  {objective={type='other'}},{objective={type='reach_goal',lives=0}},
  {objective={type='reach_goal',time=0}},
}
for i,x in ipairs(mission_bad) do check('mission structure '..i,function()d.mission.validate(x)end)end
local valid=v.layout(layout(),{p='missions/a/models/p.gxmesh'})
assert(valid.parts[1].scale==1)
local mirrored=layout();mirrored.parts[1].scale_x=-1;v.layout(mirrored,{p='missions/a/models/p.gxmesh'})
print(('missions_validation: %d refusals passed, 0 failed'):format(n))
