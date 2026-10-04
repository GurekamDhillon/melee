-- Explicit schema-1 generator-3 prototype. Randomized DFS plus loops; no certification claim.
-- Geometry is original collision, not a v2 recipe override. Analytical standard
-- mobility/clearance screening requires subsequent native controller evidence.
local M={version=1,schema_version=1,generator_version=3,width=4,height=3}
local opposite={left='right',right='left',top='bottom',bottom='top'}
local directions={{side='left',dx=-1,dy=0},{side='right',dx=1,dy=0},{side='top',dx=0,dy=1},{side='bottom',dx=0,dy=-1}}
local function finite(x)return type(x)=='number' and x==x and math.abs(x)<math.huge end
local function equal(a,b)
 if type(a)~=type(b)then return false end
 if type(a)~='table'then return a==b end
 for k,v in pairs(a)do if not equal(v,b[k])then return false end end
 for k in pairs(b)do if a[k]==nil then return false end end
 return true
end
local function rng(seed)return function(n)seed=seed*48271%2147483647;return seed%n+1 end end
local function id(x,y)if x==1 and y==1 then return 'entry' end;return 'maze_'..x..'_'..y end
local function geometry(shift)
 local g={floor={left=-130,right=130,y=0},kit={unit=13,grid=26,bay=52,height=52,depth=0},
  platforms={},lines={},blockers={},spawn={x=-117,y=0},enemy_spawns={{x=104,y=0}},
  exit_anchors={left={x=-104,y=0},right={x=104,y=0},top={x=-52+shift,y=54},bottom={x=52+shift,y=54}},
  camera={left=-130,right=130,bottom=-36,top=80},collision_count=14,model_count=32,shift=shift}
 for _,x in ipairs({-104,104})do g.platforms[#g.platforms+1]={x=x+shift,y=18,width=44,passthrough=true,ledges=false}end
 for _,x in ipairs({-82,82})do g.platforms[#g.platforms+1]={x=x+shift,y=36,width=44,passthrough=true,ledges=false}end
 for _,x in ipairs({-52,0,52})do g.platforms[#g.platforms+1]={x=x+shift,y=54,width=48,passthrough=true,ledges=false}end
 for _,x in ipairs({-52,52})do
  x=x+shift
  g.blockers[#g.blockers+1]={left=x-4,right=x+4,bottom=0,top=48}
  g.lines[#g.lines+1]={x0=x-4,y0=0,x1=x-4,y1=48,kind='left_wall',ledges=false,passthrough=false}
  g.lines[#g.lines+1]={x0=x+4,y0=48,x1=x+4,y1=0,kind='right_wall',ledges=false,passthrough=false}
  g.lines[#g.lines+1]={x0=x-4,y0=48,x1=x+4,y1=48,kind='floor',ledges=false,passthrough=false}
 end
 return g
end
M.geometry=geometry
-- Conservative centre-body flight corridor between launch/landing intervals.
-- Ground spans are split at actual blockers. Merely sharing the continuous
-- underlying floor never creates a fictitious walk-through-wall connection.
local function screen(g)
 local s=g.shift
 local surfaces={{lo=-128,hi=-58+s,y=0},{lo=-46+s,hi=46+s,y=0},{lo=58+s,hi=128,y=0}}
 for _,p in ipairs(g.platforms)do surfaces[#surfaces+1]={lo=p.x-p.width/2+2,hi=p.x+p.width/2-2,y=p.y}end
 for _,b in ipairs(g.blockers)do surfaces[#surfaces+1]={lo=b.left,hi=b.right,y=b.top}end
 local function edge(a,b)
  local gap=math.max(0,b.lo-a.hi,a.lo-b.hi)
  if b.y-a.y>18 or gap>(b.y<a.y and 45 or 30)then return false end
  for _,ax in ipairs({a.lo,(a.lo+a.hi)/2,a.hi})do
   for _,bx in ipairs({b.lo,(b.lo+b.hi)/2,b.hi})do
    local clear=true
    for _,wall in ipairs(g.blockers)do
     local low,high=math.min(ax,bx),math.max(ax,bx)
     if low<wall.right+2 and high>wall.left-2 then
      if math.abs(ax-bx)<.001 then
       if math.min(a.y,b.y)<wall.top then clear=false end
      else
       for _,wx in ipairs({wall.left-2,wall.right+2})do
        local t=(wx-ax)/(bx-ax)
        if t>0 and t<1 and a.y+(b.y-a.y)*t<wall.top+2 then clear=false end
       end
      end
     end
    end
    if clear then return true end
   end
  end
  return false
 end
 local seen={[1]=true};local changed=true
 while changed do changed=false;for i,a in ipairs(surfaces)do if seen[i]then
  for j,b in ipairs(surfaces)do if not seen[j] and edge(a,b)then seen[j]=true;changed=true end end
 end end end
 -- Narrow solid wall caps are collision closure, not required safe landings.
 for i=1,10 do if not seen[i]then return false,'unreachable corridor surface' end end
 for _,a in pairs(g.exit_anchors)do
  local supported=false
  for i,p in ipairs(surfaces)do if seen[i] and a.x>=p.lo and a.x<=p.hi and a.y==p.y then supported=true end end
  if not supported then return false,'door lacks reachable landing' end
 end
 return true
end
M.screen=screen
-- Only nine immutable authored translations exist. Keep the canonical shapes
-- private: callers still receive fresh geometry, and every validation compares
-- the complete live/decoded table to its reference before reusing a proof.
-- Cache the analytical proof for the reference, never for caller-owned tables.
local canonical,screened={},{}
local function reviewed_geometry(shift)
 local reference=canonical[shift]
 if not reference then reference=geometry(shift);canonical[shift]=reference end
 if screened[shift]==nil then
  local ok,why=screen(reference)
  if not ok then return nil,why end
  screened[shift]=true
 end
 return reference
end
function M.generate(seed)
 assert(finite(seed) and seed%1==0 and seed>=1 and seed<=2147483646,'seed must be integer 1..2147483646')
 local rand=rng(seed)
 local m={version=1,schema_version=1,generator_version=3,seed=seed,width=4,height=3,
  start=id(1,1),nodes={},order={},certified=false,validation='analytical-standard-clearance; native verification pending',loop_count=2}
 for y=1,3 do for x=1,4 do local key=id(x,y);m.order[#m.order+1]=key
  m.nodes[key]={id=key,maze=true,kind='traversal',title='Maze Corridor '..x..','..y,depth=0,grid={x=x,y=y},exits={},room=geometry(rand(9)-5)}
 end end
 local function neighbors(n)
  local out={};for _,d in ipairs(directions)do local nextid=id(n.grid.x+d.dx,n.grid.y+d.dy)
   if m.nodes[nextid]then out[#out+1]={to=nextid,side=d.side}end
  end;return out
 end
 local function link(a,e)
  m.nodes[a].exits[#m.nodes[a].exits+1]={to=e.to,side=e.side,socket=e.side}
  m.nodes[e.to].exits[#m.nodes[e.to].exits+1]={to=a,side=opposite[e.side],socket=opposite[e.side]}
 end
 -- Recursive backtracking implemented with an explicit stack: bounded 12 cells.
 local visited={[m.start]=true};local stack={m.start};local depth={[m.start]=0}
 while #stack>0 do
  local current=stack[#stack];local choices={}
  for _,e in ipairs(neighbors(m.nodes[current]))do if not visited[e.to]then choices[#choices+1]=e end end
  if #choices==0 then table.remove(stack)else
   local e=choices[rand(#choices)];link(current,e);visited[e.to]=true
   depth[e.to]=depth[current]+1;stack[#stack+1]=e.to
  end
 end
 local finish
 for _,key in ipairs(m.order)do if key~=m.start and #m.nodes[key].exits==1 and (not finish or depth[key]>depth[finish])then finish=key end end
 local boss=m.nodes[finish].exits[1].to
 local possible={}
 for _,key in ipairs(m.order)do for _,e in ipairs(neighbors(m.nodes[key]))do
  if key<e.to and key~=finish and e.to~=finish then
   local linked=false;for _,existing in ipairs(m.nodes[key].exits)do if existing.to==e.to then linked=true end end
   if not linked then possible[#possible+1]={from=key,to=e.to,side=e.side}end
  end
 end end
 for _=1,2 do local e=table.remove(possible,rand(#possible));link(e.from,e)end
 local rest
 for _,key in ipairs(m.order)do if key~=m.start and key~=finish and key~=boss
  and (not rest or math.abs(depth[key]-depth[finish]/2)<math.abs(depth[rest]-depth[finish]/2))then rest=key end end
 for _,key in ipairs(m.order)do local n=m.nodes[key];n.depth=depth[key]
  if key==m.start then n.kind='entry';n.title='Maze Entrance'
  elseif key==finish then n.kind='exit';n.title='Maze Exit'
  elseif key==boss then n.kind='boss';n.title='Maze Champion'
  elseif key==rest then n.kind='rest';n.title='Maze Sanctuary'
  elseif rand(4)==1 then n.kind='arena';n.encounter=rand(2)==1 and 'guard' or 'pressure';n.title='Maze Encounter' end
  table.sort(n.exits,function(a,b)return a.side<b.side end)
 end
 assert(M.validate(m));return m
end
local function validate(m)
 if type(m)~='table' or m.schema_version~=1 or m.generator_version~=3 or m.version~=1 or m.certified~=false
  or type(m.nodes)~='table' or type(m.order)~='table' or #m.order~=12 or not m.nodes[m.start]then return false,'unsupported maze manifest' end
 local ordered,counts,edges={},{entry=0,rest=0,boss=0,exit=0},0
 local kinds={entry=true,traversal=true,arena=true,rest=true,boss=true,exit=true}
 for _,key in ipairs(m.order)do
  local n=m.nodes[key]
  if ordered[key]or type(n)~='table' or n.id~=key or not kinds[n.kind]or type(n.grid)~='table' or key~=id(n.grid.x,n.grid.y)
   or type(n.exits)~='table' or type(n.room)~='table' or not finite(n.room.shift)or n.room.shift%1~=0 or math.abs(n.room.shift)>4 then return false,'invalid maze cell' end
  ordered[key]=true;if counts[n.kind]then counts[n.kind]=counts[n.kind]+1 end
  local reference,why=reviewed_geometry(n.room.shift)
  if not reference then return false,why end
  if not equal(n.room,reference)then return false,'unrecognized prototype geometry' end
  local used={}
  for _,e in ipairs(n.exits)do
   local dest=m.nodes[e.to]
   if not opposite[e.side]or used[e.side]or e.socket~=e.side or not dest then return false,'invalid maze door' end
   used[e.side]=true;edges=edges+1
   local adjacent=false;for _,d in ipairs(directions)do if d.side==e.side and dest.grid.x==n.grid.x+d.dx and dest.grid.y==n.grid.y+d.dy then adjacent=true end end
   local reciprocal=false;for _,back in ipairs(dest.exits)do if back.to==key and back.side==opposite[e.side]then reciprocal=true end end
   if not adjacent or not reciprocal then return false,'nonreciprocal grid corridor' end
  end
 end
 local nodecount=0;for key in pairs(m.nodes)do nodecount=nodecount+1;if not ordered[key]then return false,'unordered maze cell' end end
 if nodecount~=12 or edges~=26 or counts.entry~=1 or counts.rest~=1 or counts.boss~=1 or counts.exit~=1 or m.nodes[m.start].kind~='entry'then return false,'invalid maze roles/loop count' end
 local seen={[m.start]=true};local queue={m.start}
 for i=1,12 do if queue[i]then for _,e in ipairs(m.nodes[queue[i]].exits)do if not seen[e.to]then seen[e.to]=true;queue[#queue+1]=e.to end end end end
 if #queue~=12 then return false,'disconnected maze' end
 for _,n in pairs(m.nodes)do if n.kind=='exit' and (#n.exits~=1 or m.nodes[n.exits[1].to].kind~='boss')then return false,'exit must follow champion' end end
 return true
end
function M.validate(m)
 local ok,valid,why=pcall(validate,m)
 if not ok then return false,'malformed maze manifest' end
 return valid,why
end
return M
