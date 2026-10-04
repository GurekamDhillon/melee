local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local root=T.root:gsub('scripts/$','');local path=root..'missions/path/'
local function load() return assert(loadfile(path..'level.lua'))(),assert(loadfile(path..'mission.lua'))() end
local function floor_at(lines,x,y)
  for _,l in ipairs(lines) do if l.kind=='floor' and l.y1==l.y2 and l.y1<y and y-l.y1<=14 and x>=l.x1+20 and x<=l.x2-20 then return l end end
end
T.test('all seven enemies stand away from edges on one enclosed continuous floor',function()
  local level,m=load();local kinds={};local floor
  for _,e in ipairs(m.enemies) do
    kinds[e.kind]=true;local l=assert(floor_at(level.lines,e.x,e.y),'edge unsafe '..e.kind)
    if floor then assert(l==floor,'walker on isolated platform') else floor=l end
  end
  local n=0;for _ in pairs(kinds) do n=n+1 end;assert(n==7)
  local left,right=false,false
  for _,l in ipairs(level.lines) do
    if l.kind=='left_wall' and l.x1==floor.x1 and l.y1<=floor.y1 and l.y2>=floor.y1+60 then left=true end
    if l.kind=='right_wall' and l.x1==floor.x2 and l.y2<=floor.y1 and l.y1>=floor.y1+60 then right=true end
  end
  assert(left and right,'ground walkers can leave floor')
end)
T.test('start pocket is elevated and separated from the first wave by a wall',function()
  local level,m=load();local shelf=assert(floor_at(level.lines,m.start.x,m.start.y))
  assert(shelf.y1>=40 and level.starting_percent==0)
  local barrier=false
  for _,l in ipairs(level.lines) do if l.kind=='left_wall' and l.x1>shelf.x2 and l.x1<shelf.x2+40 and l.y1==0 and l.y2>=30 then barrier=true end end
  assert(barrier)
  for _,e in ipairs(m.enemies) do if e.wave==1 then assert(e.x-m.start.x>=90 and e.y<shelf.y1) end end
end)
T.test('authored walls and markers validate against the actual shared schema',function()
  local base=T.root:gsub('envoy/scripts/$','missions/scripts/')
  local d={mission=assert(loadfile(base..'mission.lua'))()}
  local v=assert(loadfile(base..'validator.lua'))()(d)
  local level,m=load();assert(v.layout(level,{}));assert(d.mission.validate(m))
end)
T.done()
