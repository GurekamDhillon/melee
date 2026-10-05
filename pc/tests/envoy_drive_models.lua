local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local V=T.module('drive_models')
local function rig()
  local s={logs={},missing_names={},loaded={},released={},instances={},sets={},removed={},next=0}
  local g={log=function(message) s.logs[#s.logs+1]=message end}
  g.model_load=function(path)
    s.loaded[#s.loaded+1]=path
    if s.missing or s.missing_names[path] or (#s.loaded==s.loadfail) then return nil,'absent' end
    return path
  end
  g.model_release=function(h) if s.releasefail then error('scene unavailable') end;s.released[#s.released+1]=h end
  g.model_spawn=function(h,o)
    s.next=s.next+1
    if s.next==s.spawnfail then return nil,'pool full' end
    assert(o.collision==false);local copy={};for k,v in pairs(o) do copy[k]=v end
    s.instances[s.next]={asset=h,options=copy};return s.next
  end
  g.model_set=function(h,o)
    if s.setfail then return false end
    assert(s.instances[h]);s.sets[#s.sets+1]=o;return true
  end
  g.model_despawn=function(h)
    if s.dropfail then error('scene unavailable') end
    s.instances[h]=nil;s.removed[#s.removed+1]=h;return true
  end
  g.model_get=function(h) return s.instances[h] end
  return s,V.new(g),g
end
T.test('absent APIs and assets retain glyph and retry without log spam',function()
  assert(not V.new({}):sync({}))
  local s,v=rig();s.missing=true;local p={colour='red',x=1,y=2,left=120}
  assert(not v:sync({p}) and not v:visible(p) and not v:active())
  local n=#s.loaded;for _=1,10 do v:sync({p}) end;assert(#s.loaded==n)
  s.missing=false;for _=1,120 do v:sync({p}) end;assert(v:visible(p))
end)
T.test('partial load retains usable assets and unload permits fresh discovery',function()
  local s,v=rig();s.loadfail=3;assert(v:load());assert(#s.released==0)
  v:unload();assert(#s.released==5);s.loadfail=nil;assert(v:load());v:unload();v:unload();assert(#s.released==11)
end)
T.test('every colour gets two collision-free models and animated bob blink',function()
  local s,v=rig();local p={colour='blue',x=10,y=20,left=100}
  assert(v:tick({p}) and v:visible(p) and v:active());assert(s.next==2)
  assert(s.instances[1].asset:find('envoy_drives/models/drive_blue',1,true))
  local y=s.sets[1].y;v:tick({p});assert(s.sets[3].y~=y)
  p.left=30;local on,off=false,false
  for _=1,8 do v:tick({p});local o=s.sets[#s.sets];on=on or o.visible;off=off or not o.visible end
  assert(on and off);v:clear();v:clear();assert(#s.removed==2)
end)
T.test('glass spawn refusal rolls back solid and glyph remains for that pickup',function()
  local s,v=rig();s.spawnfail=2;local p={colour='red',x=0,y=0,left=99}
  assert(not v:sync({p}));assert(not v:visible(p) and not v:active());assert(#s.removed==1)
  v:sync({p});assert(v:visible(p))
end)
T.test('set refusal removes pair rather than suppressing fallback',function()
  local s,v=rig();s.setfail=true;local p={colour='white',x=0,y=0,left=99}
  assert(not v:sync({p}));assert(not v:visible(p) and #s.removed==2)
end)
T.test('collection pulses fourteen ticks but expiry immediately cleans up',function()
  local s,v=rig();local p={colour='green',x=0,y=0,left=120}
  v:sync({p});v:sync({});assert(#s.removed==0 and #v.dying==1)
  for _=1,13 do v:sync({}) end;assert(#s.removed==2 and #v.dying==0)
  p.left=1;v:sync({p});v:sync({});assert(#s.removed==4 and #v.dying==0)
end)
T.test('refused cleanup retains handles for later retry',function()
  local s,v=rig();local p={colour='yellow',x=0,y=0,left=120}
  v:sync({p});s.dropfail=true;v:clear();assert(next(s.instances))
  s.dropfail=false;v:clear();assert(not next(s.instances));v:unload();assert(#s.released==6)
end)
T.test('all colour mesh and glass tints are explicit and collision stays disabled',function()
  for _,colour in ipairs({'red','green','yellow','blue','white'}) do
    local s,v=rig();local p={colour=colour,x=0,y=0,left=120}
    assert(v:sync({p}));assert(s.instances[1].asset==V.MOD..'/models/'..V.MESH[colour])
    assert(s.instances[1].options.tint==V.TINT[colour] and s.instances[2].options.tint==V.GLASS[colour])
    v:unload();assert(not next(s.instances) and #s.released==6)
  end
end)
T.test('explicit collection starts pulse and clear never invents one',function()
  local s,v=rig();local p={colour='red',x=0,y=0,left=100}
  v:sync({p});assert(v:collected(p));assert(not v:collected(p));assert(#v.dying==1)
  v:clear();assert(#s.removed==2 and #v.dying==0)
end)
T.test('reset stale handles are discarded without touching another owner',function()
  local s,v,g=rig();local p={colour='red',x=0,y=0,left=100}
  v:sync({p});s.instances={};s.dropfail=true
  v:clear();assert(#v.pending==0);v:unload();assert(#s.released==6)
end)
T.test('refused asset releases survive unload and are retried on clear',function()
  local s,v=rig();assert(v:load());s.releasefail=true;v:unload()
  assert(#v.releases==6 and #s.released==0)
  s.releasefail=false;v:clear();v:clear();assert(#v.releases==0 and #s.released==6)
end)
T.test('delivered assets support the first red and green drops without neutral',function()
  local s,v=rig();s.missing_names[V.MOD..'/models/drive_neutral']=true
  local red={colour='red',x=0,y=0,left=120};local green={colour='green',x=5,y=0,left=120}
  assert(v:sync({red,green}) and v:visible(red) and v:visible(green))
  for _,path in ipairs(s.loaded) do assert(not path:find('drive_neutral',1,true)) end
end)
T.test('one missing colour preserves other models and logs the exact filename once',function()
  local s,v=rig();s.missing_names[V.MOD..'/models/drive_blue']=true
  local red={colour='red',x=0,y=0,left=120};local blue={colour='blue',x=5,y=0,left=120}
  assert(not v:sync({red,blue}) and v:visible(red) and not v:visible(blue))
  for _=1,250 do v:sync({red,blue}) end
  local n=0;for _,line in ipairs(s.logs) do if line:find('drive_blue.gxmesh',1,true) then n=n+1 end end
  assert(n==1)
  v:unload();assert(not next(s.instances))
end)
T.test('missing glass releases solids and retries without repeating the filename warning',function()
  local s,v=rig();s.missing_names[V.MOD..'/models/drive_glass']=true
  local red={colour='red',x=0,y=0,left=120}
  assert(not v:sync({red}) and not v:visible(red));assert(#s.released==5)
  for _=1,120 do v:sync({red}) end
  local n=0;for _,line in ipairs(s.logs) do if line:find('drive_glass.gxmesh',1,true) then n=n+1 end end
  assert(n==1 and #s.released==10)
  s.missing_names={};for _=1,120 do v:sync({red}) end
  assert(v:visible(red));v:unload();assert(not next(s.instances))
end)
T.done()
