-- Physical traversal prototype. Analytical admission is not native certification.
local T={version=1,schema_version=1,generator_version=4}
local function finite(x) return type(x)=='number' and x==x and math.abs(x)<math.huge end
local function integer(x) return type(x)=='number' and x==math.floor(x) and x>=1 and x<=2147483646 end
local function equal(a,b)
 if type(a)~=type(b) then return false end
 if type(a)~='table' then return a==b end
 for k,v in pairs(a) do if not equal(v,b[k]) then return false end end
 for k in pairs(b) do if a[k]==nil then return false end end
 return true
end
function T.geometry(shift,side)
 assert(shift==-26 or shift==0 or shift==26,'invalid physical shift')
 assert(side==-1 or side==1,'invalid spur side')
 local g={shift=shift,spur_side=side,floor={left=-416,right=416,y=0},
 kit={unit=13,grid=26,bay=52,height=52,depth=0},spawn={x=-390,y=0},enemy_spawns={},
 exit_anchors={right={x=390,y=0}},camera={left=-416,right=416,bottom=-36,top=208},
 physical_bounds={left=-416,right=416,bottom=0,top=156},platforms={},lines={},
 blockers={{left=-104+shift,right=104+shift,bottom=78,top=130}},collision_count=24,model_count=48}
 local function pad(x,y,w,pass) g.platforms[#g.platforms+1]={x=x,y=y,width=w,passthrough=pass,ledges=false} end
 local heights={18,36,54,72,90,108,126,130}
 for i,y in ipairs(heights) do pad(-338+26*i+shift,y,i==8 and 52 or 44,true) end
 for i,y in ipairs(heights) do pad(338-26*i+shift,y,i==8 and 52 or 44,true) end
 pad(shift,130,208,false)
 pad(side*156+shift,148,52,true);pad(side*182+shift,156,52,true)
 pad(side*260+shift,156,104,true)
 local b=g.blockers[1]
 g.lines={{x0=b.left,y0=78,x1=b.left,y1=130,kind='left_wall',passthrough=false,ledges=false},
 {x0=b.right,y0=130,x1=b.right,y1=78,kind='right_wall',passthrough=false,ledges=false},
 {x0=b.right,y0=78,x1=b.left,y1=78,kind='ceiling',passthrough=false,ledges=false}}
 return g
end
function T.generate(seed)
 assert(integer(seed),'physical traversal seed must be an integer in 1..2147483646')
 -- Park-Miller step; no global RNG state or admission-dependent consumption.
 local state=(seed*48271)%2147483647
 local shift=({-26,0,26})[state%3+1]
 state=(state*48271)%2147483647
 local side=state%2==0 and -1 or 1
 return {version=1,schema_version=1,generator_version=4,seed=seed,start='entry',certified=false,
 order={'entry','exit'},nodes={entry={id='entry',kind='entry',physical=true,room=T.geometry(shift,side),
 exits={{to='exit',side='right',socket='right',label='Finish level'}}},
 exit={id='exit',kind='exit',terminal=true,exits={}}}}
end
function T.parts(node)
 assert(node and node.physical and node.room,'physical room required')
 local g=node.room;local out={}
 local function part(model,x,y,sx)
 out[#out+1]={model=model,x=x,y=y,z=0,scale_x=sx or 2,scale_y=2,scale_z=2,background=true,collision=false}
 end
 for i=0,15 do part('bf_floor_4m',-390+52*i,0) end
 for i=1,16 do local p=g.platforms[i];part('bf_floor_4m',p.x,p.y,p.width/26) end
 for _,x in ipairs({-78,-26,26,78}) do part('bf_floor_4m',g.shift+x,130) end
 for _,x in ipairs({-78,-26,26,78}) do part('bf_wall_solid_4m',g.shift+x,78) end
 for i=18,19 do local p=g.platforms[i];part('bf_floor_4m',p.x,p.y) end
 local p=g.platforms[20];part('bf_floor_4m',p.x-26,p.y);part('bf_floor_4m',p.x+26,p.y)
 part('bf_wall_doorway_4m',-390,0);part('bf_wall_doorway_4m',390,0)
 part('bf_floor_end_trim',-416,0,-2);part('bf_floor_end_trim',416,0,2)
 return out
end
function T.screen(g)
 if type(g)~='table' or type(g.blockers)~='table' or not g.blockers[1] or g.blockers[1].bottom<72 then return false,'underpass requires 72 units headroom' end
 local body=g.blockers[1]
 for _,k in ipairs({'left','right','bottom','top'}) do if not finite(body[k]) then return false,'non-finite blocker' end end
 if body.left>=body.right or body.bottom>=body.top then return false,'invalid solid slab' end
 if not g.lines or #g.lines~=3 or not equal(g.lines,{
 {x0=body.left,y0=body.bottom,x1=body.left,y1=body.top,kind='left_wall',passthrough=false,ledges=false},
 {x0=body.right,y0=body.top,x1=body.right,y1=body.bottom,kind='right_wall',passthrough=false,ledges=false},
 {x0=body.right,y0=body.bottom,x1=body.left,y1=body.bottom,kind='ceiling',passthrough=false,ledges=false}}) then return false,'solid slab requires oriented sides and ceiling' end
 if not g.platforms or #g.platforms~=20 then return false,'expected 20 physical surfaces' end
 local surfaces={{x=0,y=0,width=832}}
 for _,p in ipairs(g.platforms) do
  if not finite(p.x) or not finite(p.y) or not finite(p.width) or p.width<=0 or p.x-p.width/2 < -416 or p.x+p.width/2 >416 or p.y<0 or p.y>156 then return false,'invalid surface' end
  surfaces[#surfaces+1]=p
 end
 local seen={[1]=true};local queue={1};local maxrise=0
 local function clear(a,b)
  local body=g.blockers[1]
  -- Stairs remain outside the slab; routes above its cap are clear.
  if a.y==0 and b.y<=18 then return true end
  if a.y>=body.top and b.y>=body.top then return true end
  local left=math.min(a.x,b.x);local right=math.max(a.x,b.x)
  return right<=body.left or left>=body.right
 end
 local head=1
 while queue[head] do
  local a=surfaces[queue[head]];head=head+1
  for j,b in ipairs(surfaces) do
   local rise=b.y-a.y
   local gap=math.max(0,math.abs(b.x-a.x)-(a.width+b.width)/2)
   if not seen[j] and rise>=0 and rise<=18 and gap<=30 and clear(a,b) then
    seen[j]=true;queue[#queue+1]=j;maxrise=math.max(maxrise,rise)
   end
  end
 end
 if #queue~=#surfaces then return false,'unreachable physical branch; maximum supported rise is 18' end
 return true,{max_jump_rise=maxrise,reachable_surfaces=#queue,lower_distance=g.exit_anchors.right.x-g.spawn.x,
 upper_reachable=seen[18]==true,spur_reachable=seen[21]==true}
end
function T.validate(m)
 if type(m)~='table' or not integer(m.seed) then return false,'invalid physical manifest seed' end
 local expected=T.generate(m.seed)
 if not equal(m,expected) then return false,'physical manifest differs from its deterministic generator version 4 recipe' end
 return true,'analytical prototype admitted; native playability certification pending'
end
return T
