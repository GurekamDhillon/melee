-- Compatibility validator for resolved unversioned widened prototype saves.
-- Never regenerate a resumed manifest from its seed.
-- Pure seeded route and authored geometry. Analytical screening only: real Melee
-- movement, seams, camera and controller play must still be tested in-engine.
local D = {version=1}
local function finite(v) return type(v)=='number' and v==v and math.abs(v)<math.huge end
local function point(p) return type(p)=='table' and finite(p.x) and finite(p.y) and math.abs(p.x)<=130 and p.y>=0 and p.y<=60 end
local function rng(seed)
 return function(n) seed=(seed*48271)%2147483647; return seed%n+1 end
end
local function room(kind,rand)
 local shift=rand(9)-5
 local platforms={}
 if kind=='traversal' or kind=='arena' or kind=='boss' then
  platforms={{x=-44+shift,y=18,width=22,passthrough=true,ledges=false},
             {x=44+shift,y=18,width=22,passthrough=true,ledges=false},
             {x=shift,y=36,width=24,passthrough=true,ledges=false}}
 end
 return {platforms=platforms,lines={},spawn={x=-84,y=0},
  floor={left=-130,right=130,y=0},kit={unit=6.5,grid=13,bay=26,height=26,depth=0},
  enemy_spawns=(kind=='arena' or kind=='boss') and {{x=56,y=0,kind=kind=='boss' and 'champion' or (rand(2)==1 and 'pressure' or 'guard')}} or {},
  exit_anchors={left={x=-117,y=0},right={x=117,y=0}},
  camera={left=-130,right=130,bottom=-18,top=54}}
end
function D.generate(seed,opts)
 assert(finite(seed) and seed%1==0 and seed>=1 and seed<=2147483646,'seed must be integer 1..2147483646')
 local rand=rng(seed)
 local m={version=D.version,seed=seed,start='entry',nodes={},order={}}
 local specs={{'entry','entry','The Threshold',0},{'trail','traversal','Inner Passage',1},
  {'arena_a','arena','Ember Court',2},{'arena_b','arena','Rime Gallery',2},
  {'rest','rest','Quiet Landing',3},{'approach','traversal','Champion Approach',4},
  {'boss','boss','Gene Champion',5},{'exit','exit','Run Complete',6}}
 local labels=rand(2)==1 and {'Pressure route','Guard route'} or {'Guard route','Pressure route'}
 for _,s in ipairs(specs) do
  m.order[#m.order+1]=s[1]
  m.nodes[s[1]]={id=s[1],kind=s[2],title=s[3],depth=s[4],exits={},room=room(s[2],rand)}
 end
 local function edge(a,b,label,side) m.nodes[a].exits[#m.nodes[a].exits+1]={to=b,label=label,side=side or 'right'} end
 edge('entry','trail','Explore'); edge('trail','arena_a',labels[1],'left'); edge('trail','arena_b',labels[2])
 edge('arena_a','rest','Collect reward'); edge('arena_b','rest','Collect reward')
 edge('rest','approach','Continue'); edge('approach','boss','Challenge champion'); edge('boss','exit','Finish run')
 -- Branch content also varies, not just its label. No gene is needed to finish.
 m.nodes.arena_a.encounter=labels[1]=='Pressure route' and 'pressure' or 'guard'
 m.nodes.arena_b.encounter=labels[2]=='Pressure route' and 'pressure' or 'guard'
 assert(D.validate(m))
 return m
end
local function reachable(nodes,start,allow_gates)
 local seen,queue={[start]=true},{start}
 local i=1
 while queue[i] do
  for _,e in ipairs(nodes[queue[i]].exits) do
   if (allow_gates or e.requires==nil or e.requires==false) and not seen[e.to] then
    seen[e.to]=true; queue[#queue+1]=e.to
   end
  end
  i=i+1
 end
 return seen
end
local function screen_room(r,mobility)
 if type(r)=='table' then
  local f,k=r.floor,r.kit
  if type(f)~='table' or f.left~=-130 or f.right~=130 or f.y~=0 or type(k)~='table' or
   k.unit~=6.5 or k.grid~=13 or k.bay~=26 or k.height~=26 or k.depth~=0 then return false,'incompatible kit structure' end
 end
 if type(r)~='table' or type(r.platforms)~='table' or type(r.lines or {})~='table' then return false,'missing room geometry' end
 if #r.platforms+#(r.lines or {})>16 then return false,'room collision budget exceeds 16' end
 if not point(r.spawn) then return false,'invalid spawn' end
 local height=mobility.jump_height or 18; local lateral=mobility.horizontal_gap or 30
 local body=mobility.body_clearance or 8
 if not finite(height) or height<=0 or not finite(lateral) or lateral<=0 or not finite(body) or body<=0 then return false,'invalid mobility profile' end
 -- This is the explicit owned floor, independent of original stage collision.
 -- Edges use launch/landing intervals, conservative rise/gap bounds, and permit
 -- drops onto lower floors. This is not an exact dynamics or recovery simulator.
 local surfaces={{lo=r.floor.left,hi=r.floor.right,y=r.floor.y}}
 for _,p in ipairs(r.platforms) do
  if not point(p) or not finite(p.width) or p.width<=0 or p.x-p.width/2 < -130 or p.x+p.width/2 >130 then return false,'invalid platform bounds' end
  if type(p.passthrough)~='boolean' or type(p.ledges)~='boolean' then return false,'missing platform flags' end
  if not p.passthrough then return false,'solid overhead platform needs authored clearance validation' end
  surfaces[#surfaces+1]={lo=p.x-p.width/2+2,hi=p.x+p.width/2-2,y=p.y}
  if p.width<4 then return false,'landing surface too narrow' end
 end
 -- Arbitrary walls/ceilings cannot be declared safe from graph connectivity.
 -- Future templates must add obstacle-aware swept body checks before accepting.
 for _,line in ipairs(r.lines or {}) do
  if type(line)~='table' or not point({x=line.x1,y=line.y1}) or not point({x=line.x2,y=line.y2}) then return false,'invalid line' end
  return false,'additional collision requires obstacle-aware validation'
 end
 local start
 for i,s in ipairs(surfaces) do if r.spawn.x>=s.lo and r.spawn.x<=s.hi and math.abs(r.spawn.y-s.y)<0.01 then start=i end end
 if not start then return false,'spawn lacks standing surface' end
 local seen={[start]=true}; local changed=true
 while changed do
  changed=false
  for i,a in ipairs(surfaces) do if seen[i] then
   for j,b in ipairs(surfaces) do
    local gap=math.max(0,b.lo-a.hi,a.lo-b.hi)
    local rise=b.y-a.y
    if not seen[j] and rise<=height and gap<=lateral then seen[j]=true; changed=true end
   end
  end end
 end
 for i in ipairs(surfaces) do if not seen[i] then return false,'unreachable platform' end end
 local anchors=r.exit_anchors
 if type(anchors)~='table' then return false,'missing exit anchors' end
 for _,side in ipairs({'left','right'}) do
  local p=anchors[side]
  if not point(p) or p.x~=(side=='left' and r.floor.left+r.kit.bay/2 or r.floor.right-r.kit.bay/2) or p.y~=r.floor.y then return false,'invalid exit anchor' end
  local found=false
  for i,s in ipairs(surfaces) do if seen[i] and p.x>=s.lo and p.x<=s.hi and math.abs(p.y-s.y)<0.01 then found=true end end
  if not found then return false,'unreachable exit anchor' end
 end
 for _,p in ipairs(r.enemy_spawns or {}) do if not point(p) then return false,'invalid enemy spawn' end end
 return true
end
function D.validate(m,mobility)
 if type(m)~='table' or m.version~=D.version or type(m.nodes)~='table' or not m.nodes[m.start] then return false,'invalid manifest/start' end
 if type(m.order)~='table' then return false,'missing order' end
 local count,ordered=0,{}
 for _,id in ipairs(m.order) do if ordered[id] or not m.nodes[id] then return false,'invalid order' end; ordered[id]=true end
 local exit_count=0
 local kinds={entry=true,traversal=true,arena=true,rest=true,boss=true,exit=true}
 for id,n in pairs(m.nodes) do
  count=count+1
  if not ordered[id] or type(n)~='table' or n.id~=id or not kinds[n.kind] or type(n.exits)~='table' then return false,'invalid node' end
  if n.kind=='exit' then exit_count=exit_count+1 end
  local ok,why=screen_room(n.room,mobility or {})
  if not ok then return false,id..': '..why end
  for _,e in ipairs(n.exits) do
   if type(e)~='table' or not m.nodes[e.to] or (e.side~='left' and e.side~='right') then return false,'invalid graph edge' end
  end
 end
 if count<6 or count>9 or count~=#m.order or exit_count~=1 then return false,'invalid room/exit count' end
 local all=reachable(m.nodes,m.start,true)
 local free=reachable(m.nodes,m.start,false)
 for id,n in pairs(m.nodes) do
  if not all[id] then return false,'unreachable graph node '..id end
  if n.kind=='exit' and not free[id] then return false,'exit requires a gene/key' end
 end
 -- Every selected branch must have an unconditional route to the finish; this
 -- prevents trapping a player after a valid choice behind a reward's own gate.
 for id in pairs(m.nodes) do
  local downstream=reachable(m.nodes,id,false); local finish=false
  for to in pairs(downstream) do if m.nodes[to].kind=='exit' then finish=true end end
  if not finish then return false,'branch has no unconditional exit '..id end
 end
 return true
end
return D
