-- Frozen v5 physical campaign. Analytical admission is separate from native
-- and controller certification; never infer certification from this recipe.
local C={version=1,schema_version=1,generator_version=5}
local function copy(t) if type(t)~='table' then return t end local n={} for k,v in pairs(t)do n[k]=copy(v)end return n end
local function equal(a,b)
 if type(a)~=type(b)then return false end
 if type(a)~='table'then return a==b end
 for k,v in pairs(a)do if not equal(v,b[k])then return false end end
 for k in pairs(b)do if a[k]==nil then return false end end return true
end
local function integer(x)return type(x)=='number' and x==math.floor(x) and x>=1 and x<=2147483646 end
local function finite(x)return type(x)=='number' and x==x and math.abs(x)<math.huge end
function C.geometry(index,variant)
 local patterns={'broken_causeway','overhead_switchback','sentinel_ring','split_underpass','summit_spiral','tier_crossing','champion_fortress'}
 local g={pattern=patterns[index],floor={left=-416,right=416,y=0,openings={}},
  kit={unit=13,grid=26,bay=52,height=52,depth=0},spawn={x=-390,y=0},
  enemy_spawns={{x=260,y=0}},exit_anchors={},
  camera={left=-416,right=416,bottom=-36,top=234},
  physical_bounds={left=-416,right=416,bottom=0,top=156},platforms={},blockers={},lines={}}
 local function pad(x,y,w,pass)g.platforms[#g.platforms+1]={x=x,y=y,width=w,passthrough=pass,ledges=false}end
 local function holes(xs)for _,x in ipairs(xs)do g.floor.openings[#g.floor.openings+1]={x=x,width=26}end end
 local function stairs(x,dx,top,w)
  local i=0;for y=18,top-1,18 do i=i+1;pad(x+dx*i,y,w or 52,true)end
  i=i+1;pad(x+dx*i,top,w or 52,true)
 end
 local function body(l,r,b,t)
  g.blockers[#g.blockers+1]={left=l,right=r,bottom=b,top=t}
  g.lines[#g.lines+1]={x0=l,y0=b,x1=l,y1=t,kind='left_wall',passthrough=false,ledges=false}
  g.lines[#g.lines+1]={x0=r,y0=t,x1=r,y1=b,kind='right_wall',passthrough=false,ledges=false}
  g.lines[#g.lines+1]={x0=r,y0=b,x1=l,y1=b,kind='ceiling',passthrough=false,ledges=false}
 end
 local function bridge(y)
  pad(-156,y,104,true);pad(0,y,208,true);pad(156,y,104,true);pad(260,y,104,true);pad(364,y,104,true);pad(390,y,52,true)
 end
 local cap=156
 if index==1 then
  -- Broken ground and a full-height obstruction require a real upper crossing.
  -- The obstruction is two bays wide: with both staircases a four-bay one
  -- exceeds Rooms.max_instances and the whole room is refused its art.
  holes({-234,26,234});body(-52,52,0,156);stairs(-364,18,156);stairs(364,-18,156);bridge(156)
 elseif index==2 then
  -- Staggered overhead slabs enclose a lower encounter well. Drop into the well,
  -- then climb its return staircase through the lower cap and back to the bridge.
  holes({-234,234});body(-208,-104,52,104);body(104,208,104,156)
  stairs(-364,18,156)
  pad(-156,156,104,true);pad(0,156,208,true);pad(182,156,156,false);pad(338,156,156,true);pad(390,156,52,true)
  stairs(-78,26,108,104);pad(-156,104,104,false)
  -- The well's staircase ends 48 under the bridge; two more steps return it.
  pad(52,126,104,true);pad(0,144,104,true)
 elseif index==3 then
  -- Two independent edge ascents surround the sentinel's underpass and low
  -- fighting balconies. The central solid cap separates the two battle tiers.
  cap=130;holes({26});body(-52,52,78,130);stairs(-364,18,130);stairs(364,-18,130)
  pad(0,130,104,false)
  for _,side in ipairs({-1,1})do pad(side*130,130,156,true);pad(side*260,130,104,true);pad(side*338,130,104,true);pad(side*390,130,52,true);pad(side*234,72,156,true)end
 elseif index==4 then
  -- The lower route passes beneath two separate masses; either outside ascent
  -- returns along their caps. Both loops meet at the high exit, not ground level.
  cap=130;body(-208,-104,78,130);body(104,208,78,130);stairs(-364,18,130);stairs(364,-18,130)
  pad(-156,130,104,false);pad(156,130,104,false);pad(0,130,208,true)
  for _,side in ipairs({-1,1})do pad(side*260,130,104,true);pad(side*338,130,104,true);pad(side*390,130,52,true)end
 elseif index==5 then
  -- A sheltered central zigzag, rather than another mirrored staircase room.
  cap=130;body(104,208,78,130)
  for i=1,7 do pad(-260+(i%2==0 and -26 or 26),18*i,104,true)end
  pad(-260,130,104,true);pad(-156,130,104,true);pad(0,130,208,true);pad(156,130,104,false);pad(260,130,104,true);pad(364,130,104,true);pad(390,130,52,true)
 elseif index==6 then
  -- A grounded left tower and a raised right tower create a three-tier crossing
  -- and a lower well with a return staircase. Enemies occupy different tiers, making the detour compulsory.
  holes({-234,234});body(-208,-104,0,104);body(104,208,78,156);stairs(-390,26,104)
  pad(-156,104,104,false);pad(-78,122,104,true);pad(0,140,104,true);pad(78,156,104,true);pad(156,156,104,false)
  pad(260,156,104,true);pad(338,156,104,true);pad(390,156,52,true);stairs(-78,26,108,104)
  -- Well staircase ends 48 below the crossing; one step returns it. The raised
  -- right tower leaves an underpass so the right-hand ground is not a pit.
  pad(52,126,104,true)
 else
  -- The champion's grounded fortress divides the arena; balcony crossings and
  -- two outside return paths connect its battlefields and raised finish doorway.
  cap=130;holes({26});body(-104,104,0,52);stairs(-364,18,130);stairs(364,-18,130)
  pad(0,130,208,true)
  for _,side in ipairs({-1,1})do pad(side*156,130,156,true);pad(side*260,130,104,true);pad(side*338,130,104,true);pad(side*390,130,52,true);pad(side*260,36,104,true)end
 end
 g.exit_anchors={right={x=390,y=cap}}
 -- Only the fork has a westward upper doorway; use the first upper landing for it.
 if index==2 then g.exit_anchors.top={x=-156,y=156}end
 -- Seed changes the broken ground while keeping each authored route structure.
 for _,h in ipairs(g.floor.openings)do h.x=h.x+(variant%2==0 and 0 or -52)end
 g.collision_count=1+#g.floor.openings+#g.platforms+#g.lines
 return g
end
function C.generate(seed)
 assert(integer(seed),'campaign seed must be an integer in 1..2147483646')
 local variant=(seed*48271)%2147483647
 local order={'entry','switchback','arena','detour','rest','approach','boss','exit'}
 local spec={entry={kind='entry',title='The Broken Causeway',to='switchback'},
  switchback={kind='traversal',title='Split Ascent',to='arena',monsters={{kind='goomba',x=-26,y=54},{kind='goomba',x=52,y=90}}},
  arena={kind='arena',title='Sentinel Trial',to='rest',encounter='guard'},
  detour={kind='traversal',title='The Haunted Detour',to='rest',monsters={{kind='redead',x=156,y=0},{kind='goomba',x=312,y=0}}},
  rest={kind='rest',title='Quiet Summit',to='approach'},
  approach={kind='traversal',title='Champion Approach',to='boss',monsters={{kind='redead',x=-156,y=104},{kind='goomba',x=-26,y=54},{kind='redead',x=156,y=156}}},
  boss={kind='boss',title='The Summit Champion',to='exit',encounter='striker'}}
 local nodes={exit={id='exit',kind='exit',terminal=true,exits={}}}
 for i,id in ipairs(order)do local s=spec[id];if s then
  local g=C.geometry(i,variant)
  nodes[id]={id=id,kind=s.kind,title=s.title,depth=i,physical=true,campaign=true,room=g,
   encounter=s.encounter,monsters=copy(s.monsters),exits={{to=s.to,side='right',socket='right',label=id=='boss' and 'Finish campaign' or 'Climb onward'}}}
 end end
 nodes.switchback.exits[1].label='Sentinel trial / earn an upgrade'
 nodes.switchback.exits[2]={to='detour',side='top',socket='top',label='Haunted detour / avoid the fighter'}
 return {version=1,schema_version=1,generator_version=5,seed=seed,start='entry',certified=false,order=order,nodes=nodes}
end
function C.parts(node)
 local g=assert(node and node.room);local out={}
 local function part(model,x,y,sx,sy)
  out[#out+1]={model=model,x=x,y=y,z=0,scale_x=sx or 2,scale_y=sy or 2,scale_z=2,background=true,collision=false}
 end
 local holes={};for _,h in ipairs(g.floor.openings)do holes[h.x]=true end
 local i=0;while i<16 do local x=-390+52*i
  if holes[x]then part('bf_floor_opening_4m',x,0);i=i+1
  elseif i<15 and not holes[x+52]then part('bf_floor_4m',x+26,0,4);i=i+2
  else part('bf_floor_4m',x,0);i=i+1 end
 end
 for _,p in ipairs(g.platforms)do part('bf_floor_4m',p.x,p.y,p.width/26)end
 for _,b in ipairs(g.blockers)do
  for x=b.left+26,b.right-1,52 do
   for y=b.bottom,b.top-1,52 do part('bf_wall_solid_4m',x,y,2,math.min(52,b.top-y)/26)end
  end
 end
 for _,e in ipairs(node.exits)do local a=g.exit_anchors[e.side];part('bf_wall_doorway_4m',a.x,a.y)end
 part('bf_wall_doorway_4m',g.spawn.x,g.spawn.y)
 part('bf_floor_end_trim',-416,0,-2);part('bf_floor_end_trim',416,0,2)
 return out
end
function C.screen(g)
 if type(g)~='table' or type(g.platforms)~='table' or type(g.blockers)~='table' or not g.blockers[1]then return false,'missing campaign geometry'end
 for _,b in ipairs(g.blockers)do
  if not finite(b.bottom)or not finite(b.top)or b.top<=b.bottom or(b.bottom>0 and b.bottom<52)then return false,'invalid obstruction'end
 end
 local surfaces={};local cursor=g.floor.left;local cuts={}
 for _,h in ipairs(g.floor.openings)do cuts[#cuts+1]={left=h.x-h.width/2,right=h.x+h.width/2}end
 -- A floor behind a grounded solid is not one freely traversable graph node.
 for _,b in ipairs(g.blockers)do if b.bottom==0 then cuts[#cuts+1]={left=b.left,right=b.right}end end
 table.sort(cuts,function(a,b)return a.left<b.left end)
 for _,h in ipairs(cuts)do local l,r=h.left,h.right
  if l<g.floor.left or r>g.floor.right then return false,'invalid ground opening'end
  if l>cursor then surfaces[#surfaces+1]={x=(cursor+l)/2,y=0,width=l-cursor}end
  cursor=math.max(cursor,r)
 end
 surfaces[#surfaces+1]={x=(cursor+g.floor.right)/2,y=0,width=g.floor.right-cursor}
 local seen={[1]=true};local queue={1}
 for _,p in ipairs(g.platforms)do
  if not finite(p.x)or not finite(p.y)or not finite(p.width)or p.width<=0 or p.ledges~=false
   or p.x-p.width/2<g.floor.left or p.x+p.width/2>g.floor.right or p.y<0 or p.y>g.physical_bounds.top then return false,'invalid platform'end
  surfaces[#surfaces+1]=p
 end
 local head=1;local maxrise=0
 while queue[head]do local a=surfaces[queue[head]];head=head+1
  for j,p in ipairs(surfaces)do local rise=p.y-a.y;local gap=math.max(0,math.abs(p.x-a.x)-(a.width+p.width)/2)
   local clear=true;local height=math.max(a.y,p.y)
   local ax=a.y==0 and math.max(a.x-a.width/2,math.min(p.x,a.x+a.width/2))or a.x
   local px=p.y==0 and math.max(p.x-p.width/2,math.min(a.x,p.x+p.width/2))or p.x
   for _,b in ipairs(g.blockers)do
    if height<b.top and height+72>b.bottom and math.max(ax,px)>b.left and math.min(ax,px)<b.right then clear=false end
   end
   if not seen[j]and rise<=18 and gap<=30 and clear then seen[j]=true;queue[#queue+1]=j;maxrise=math.max(maxrise,rise)end
  end
 end
 if #queue~=#surfaces then
  for j,p in ipairs(surfaces)do if not seen[j]then return false,'unreachable surface '..p.x..','..p.y end end
 end
 for _,a in pairs(g.exit_anchors)do local supported=false
  if a.y<130 then return false,'ground exit bypass'end
  for j,p in ipairs(surfaces)do if seen[j]and p.y==a.y and math.abs(p.x-a.x)<=p.width/2 then supported=true end end
  if not supported then return false,'unsupported raised exit'end
 end
 if g.collision_count>32 then return false,'collision budget'end
 return true,{reachable_surfaces=#queue,max_jump_rise=maxrise}
end
function C.validate(m)
 if type(m)~='table'or not integer(m.seed)then return false,'invalid campaign seed'end
 if not equal(m,C.generate(m.seed))then return false,'campaign differs from frozen generator 5 recipe'end
 for _,id in ipairs(m.order)do local n=m.nodes[id];if not n.terminal then local ok,why=C.screen(n.room);if not ok then return false,id..': '..why end end end
 return true,'analytical campaign admitted; native/controller certification pending'
end
return C
