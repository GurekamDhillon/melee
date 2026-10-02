-- Original authored environment assembly. Core/Dungeon physics remain root-owned.
-- BF sidecars are suppressed with collision=false; explicit route colliders own physics.
local R={version=3,max_instances=28,max_assets=10,unit=6.5}
local valid={entry='entry',trail='traversal',arena_a='arena',arena_b='arena',
 rest='rest',approach='traversal',boss='boss',exit='exit'}
local function finite(v) return type(v)=='number' and v==v and math.abs(v)<math.huge end
local function copy(t)
 if type(t)~='table' then return t end
 local out={} for k,v in pairs(t) do out[k]=copy(v) end return out
end
local function keys(t) local a={} for k in pairs(t) do a[#a+1]=k end table.sort(a);return a end
local function handle(h) return finite(h) and h%1==0 and h>0 end
local preload={'bf_floor_4m','bf_floor_end_trim','bf_wall_solid_4m','bf_beam_4m','bf_rear_post_4m','bf_wall_doorway_4m'}
function R.preload_step(s,node)
 local models=preload
 if node and node.recipe then
  local plan,why=R.plan(node);if not plan then return nil,why end
  local seen={};models={}
  for _,part in ipairs(plan.parts) do if not seen[part.model] then models[#models+1]=part.model;seen[part.model]=true end end
 end
 if s.preload_error then return nil,s.preload_error end
 for _,name in ipairs(models) do if not s.assets[name] then
  if #keys(s.assets)>=R.max_assets then s.preload_error='room asset budget';return nil,s.preload_error end
  local ok,h,why=false,nil,nil
  if gd and gd.model_load then ok,h,why=pcall(gd.model_load,name) end
  if not ok or not handle(h) then s.preload_error=tostring(ok and why or h or 'model load unavailable');return nil,s.preload_error end
  s.assets[name]=h;return false -- Yield even after the final disk load.
 end end
 return true
end
function R.new() return {assets={},handles={},room_id=nil,theme=nil,error=nil} end
-- Only after engine scene teardown; while live, clear/release must retain refused handles.
function R.reset(s) s.assets={};s.handles={};s.room_id=nil;s.theme=nil;s.error=nil;s.preload_error=nil end
-- Pure helpers keep physics, authored sockets and destinations aligned.
function R.collision(node)
 local g=type(node)=='table' and node.room
 if type(g)~='table' or type(g.floor)~='table' or type(g.platforms)~='table' then return nil,'missing geometry' end
 local f=g.floor
 if not finite(f.left) or not finite(f.right) or not finite(f.y) or f.left>=f.right then return nil,'invalid floor' end
 local holes={}
 for _,o in ipairs(f.openings or {}) do
  if type(o)~='table' or not finite(o.x) or not finite(o.width) or o.width<=0 then return nil,'invalid floor opening' end
  holes[#holes+1]={left=o.x-o.width/2,right=o.x+o.width/2}
 end
 table.sort(holes,function(a,b) return a.left<b.left end)
 for _,p in ipairs(g.platforms) do
  if type(p)~='table' or not finite(p.x) or not finite(p.y) or not finite(p.width) or p.width<=0
   or p.x-p.width/2<f.left or p.x+p.width/2>f.right or p.y<0 or p.y>60
   or type(p.passthrough)~='boolean' or type(p.ledges)~='boolean' then return nil,'invalid collision platform' end
 end
 for _,line in ipairs(g.lines or {}) do
  if type(line)~='table' or line.kind~='floor' or not finite(line.x0) or not finite(line.y0)
   or not finite(line.x1) or not finite(line.y1) or line.x0>=line.x1
   or line.x0<f.left or line.x1>f.right or line.y0<0 or line.y1>60
   or type(line.passthrough)~='boolean' or type(line.ledges)~='boolean' then return nil,'invalid collision line' end
 end
 local spans,cursor={},f.left
 for _,o in ipairs(holes) do
  if o.left<cursor or o.right>f.right then return nil,'overlapping or out-of-bounds floor opening' end
  if o.left>cursor then spans[#spans+1]={left=cursor,right=o.left,y=f.y} end
  cursor=o.right
 end
 if cursor<f.right then spans[#spans+1]={left=cursor,right=f.right,y=f.y} end
 return {floor_segments=spans,platforms=copy(g.platforms),lines=copy(g.lines or {})}
end
function R.anchor(node,exit)
 if type(exit)~='table' then return nil end
 local anchors=node and node.room and node.room.exit_anchors or {}
 return copy(exit.anchor or anchors[exit.socket] or anchors[exit.side])
end
function R.arrival(node,socket)
 local arrivals=node and node.room and node.room.arrivals or {}
 return copy(arrivals[socket] or (not socket and node and node.room and node.room.spawn))
end
local function recipe_plan(node)
 if type(node.id)~='string' or type(node.template_id)~='string' or type(node.recipe)~='string'
  or type(node.exits)~='table' then return nil,'unsupported recipe room' end
 -- Geometry is always read back from the resolved snapshot (`node.room`); the
 -- planner never consults the live recipe table, so a saved route keeps the
 -- recipe version it was certified with even after the catalogue changes.
 if node.recipe_version~=nil and (not finite(node.recipe_version) or node.recipe_version%1~=0 or node.recipe_version<1) then
  return nil,'invalid recipe version'
 end
 local collision,why=R.collision(node);if not collision then return nil,why end
 local g=node.room;local f,k=g.floor,g.kit
 if f.left~=-65 or f.right~=65 or f.y~=0 or type(k)~='table' or k.unit~=6.5 or k.bay~=26 or k.grid~=13 or k.height~=26 then
  return nil,'incompatible kit structure'
 end
 local plan={room_id=node.id,recipe_version=node.recipe_version,theme=node.theme or 'cobalt',parts={},platforms=collision.platforms,
  floor_segments=collision.floor_segments,lines=collision.lines,anchors=copy(g.exit_anchors),arrivals=copy(g.arrivals)}
 local function add(kind,model,x,y,sx,sy)
  if #plan.parts>=R.max_instances or type(model)~='string' or not model:match('^bf_')
   or not finite(x) or not finite(y) or not finite(sx or 1) or not finite(sy or 1)
   or math.abs(sx or 1)<.001 or math.abs(sx or 1)>100 or (sy or 1)<.001 or (sy or 1)>100 then error('invalid recipe model transform/budget') end
  plan.parts[#plan.parts+1]={kind=kind,model=model,x=x,y=y,z=0,scale_x=sx or 1,scale_y=sy or 1,scale_z=1,collision=false}
 end
 local ok,detail=pcall(function()
  local holes=f.openings or {}
  if #holes>1 or (#holes==1 and (holes[1].x~=0 or holes[1].width~=13)) then error('floor opening does not match reviewed kit') end
  for i=1,5 do
   local x=-65+(i-.5)*26
   add('mainfloor',(#holes==1 and x==0) and 'bf_floor_opening_4m' or 'bf_floor_4m',x,0)
  end
  add('trim','bf_floor_end_trim',-65,0,-1);add('trim','bf_floor_end_trim',65,0)
  local doors={}
  for _,exit in ipairs(node.exits) do
   local a=R.anchor(node,exit)
   if not a or not finite(a.x) or not finite(a.y) or doors[exit.side] then error('invalid recipe socket') end
   doors[exit.side]=true
   if exit.side=='left' and (a.x~=-52 or a.y~=0) then error('left door off grid') end
   if exit.side=='right' and (a.x~=52 or a.y~=0) then error('right door off grid') end
   if exit.side=='top' and (a.x~=39 or a.y~=26) then error('upper door off grid') end
   if exit.side=='bottom' and (not a.drop or #holes~=1 or a.x~=0 or a.y~=-6) then error('drop lacks real opening') end
  end
  for i=1,5 do
   local x=-65+(i-.5)*26;local door=(i==1 and doors.left) or (i==5 and doors.right)
   add(door and 'portal' or 'wall',door and 'bf_wall_doorway_4m' or 'bf_wall_solid_4m',x,0)
   add('beam','bf_beam_4m',x,26)
  end
  for i=1,4 do add('post','bf_rear_post_4m',-65+i*26,0) end
  local modules=node.recipe_modules or {}
  if #modules>0 then
   for _,m in ipairs(modules) do add(m.part=='bf_wall_doorway_4m' and 'portal' or 'ascent',m.part,m.x,m.y,m.scale_x,m.scale_y) end
  else
   if doors.top or #collision.lines>0 then error('ascent modules missing') end
   for _,p in ipairs(g.platforms) do add('platform','bf_floor_4m',p.x,p.y,p.width/26) end
  end
  if #collision.floor_segments+#collision.platforms+#collision.lines>16 then error('collision budget') end
 end)
 if not ok then return nil,tostring(detail) end
 return plan
end
function R.plan(node)
 if type(node)=='table' and node.recipe then return recipe_plan(node) end
 if type(node)~='table' or valid[node.id]~=node.kind or not valid[node.id]
  or type(node.room)~='table' or type(node.room.platforms)~='table'
  or type(node.exits)~='table' then return nil,'unsupported room' end
 local theme='cobalt'
 if node.encounter=='guard' then theme='frost'
 elseif node.encounter=='pressure' or node.id=='boss' or node.id=='approach' then theme='fire'
 elseif node.kind=='arena' then return nil,'missing arena encounter' end
 local plan={room_id=node.id,theme=theme,parts={},platforms={}}
 local function add(kind,model,x,y,z,sx,sy)
  assert(#plan.parts<R.max_instances,'room instance budget')
  sx=sx or 1;sy=sy or 1
  assert(finite(x) and finite(y) and finite(z) and finite(sx) and math.abs(sx)>=.001
   and math.abs(sx)<=100 and finite(sy) and sy>=.001 and sy<=100,'invalid room transform')
  plan.parts[#plan.parts+1]={kind=kind,model=model,x=x,y=y,z=z,
   scale_x=sx,scale_y=sy,scale_z=1,collision=false}
 end
 local ok,why=pcall(function()
  if #node.room.platforms>3 then error('platform budget') end
  local f,k=node.room.floor,node.room.kit
  assert(type(f)=='table' and f.left==-65 and f.right==65 and f.y==0 and type(k)=='table'
   and k.unit==6.5 and k.grid==13 and k.bay==26 and k.height==26 and k.depth==0,'incompatible kit structure')
  local bay=k.bay
  for i=1,5 do add('mainfloor','bf_floor_4m',f.left+(i-.5)*bay,f.y,0) end
  add('trim','bf_floor_end_trim',f.left,f.y,0,-1)
  add('trim','bf_floor_end_trim',f.right,f.y,0,1)
  for _,p in ipairs(node.room.platforms) do
   assert(type(p)=='table' and finite(p.x) and finite(p.y) and p.y>=0 and finite(p.width)
    and p.width>0 and p.width<=130 and p.x-p.width/2>=-65 and p.x+p.width/2<=65,'invalid platform')
   add('platform','bf_floor_4m',p.x,p.y,0,p.width/26)
   plan.platforms[#plan.platforms+1]={x=p.x,y=p.y,width=p.width,visual_top=p.y}
  end
  local doors={}
  for _,e in ipairs(node.exits) do
   assert(type(e)=='table' and (e.side=='left' or e.side=='right') and not doors[e.side],'invalid exit marker')
   local anchor=node.room.exit_anchors and node.room.exit_anchors[e.side]
   local expected=e.side=='left' and f.left+bay/2 or f.right-bay/2
   assert(type(anchor)=='table' and anchor.x==expected and anchor.y==f.y,'incompatible door grid')
   doors[e.side]=true
  end
  -- Replace the outer wall bay with its door. Authored local depth is already
  -- behind fighters; every structural part uses the same unshifted kit origin.
  for i=1,5 do
   local x=f.left+(i-.5)*bay
   local door=(i==1 and doors.left) or (i==5 and doors.right)
   add(door and 'portal' or 'wall',door and 'bf_wall_doorway_4m' or 'bf_wall_solid_4m',x,f.y,k.depth)
   add('beam','bf_beam_4m',x,k.height,k.depth)
  end
  for i=1,4 do add('post','bf_rear_post_4m',f.left+i*bay,f.y,k.depth) end

 end)
 if not ok then return nil,tostring(why) end
 return plan
end
local function absent(h)
 if not gd or not gd.model_get then return false end
 local ok,result=pcall(gd.model_get,h);return ok and result==nil
end
function R.clear(s)
 local remaining,why={},nil
 for _,h in ipairs(s.handles) do
  local ok,result,detail=false,nil,nil
  if gd and gd.model_despawn then ok,result,detail=pcall(gd.model_despawn,h) end
  if not (ok and result==true) and not absent(h) then
   remaining[#remaining+1]=h;why=tostring(ok and detail or result or 'model despawn unavailable')
  end
 end
 s.handles=remaining;s.room_id=nil;s.theme=nil;s.error=why
 return #remaining==0,why
end
function R.enter(s,node)
 local cleared,why=R.clear(s);if not cleared then return false,why end
 local plan;plan,why=R.plan(node)
 if not plan then s.error=why;return false,why end
 if not gd or not gd.model_load or not gd.model_spawn or not gd.model_despawn or not gd.model_release then
  s.error='room model API unavailable';return false,s.error
 end
 for _,part in ipairs(plan.parts) do
  local asset=s.assets[part.model]
  if not asset then
   if #keys(s.assets)>=R.max_assets then why='room asset budget'
   else
    local ok,result,detail=pcall(gd.model_load,part.model)
    if ok and handle(result) then asset=result;s.assets[part.model]=asset
    else why=tostring(ok and detail or result or 'model load refused') end
   end
  end
  if not why then
   local opts={x=part.x,y=part.y,z=part.z,scale_x=part.scale_x,scale_y=part.scale_y,
    scale_z=1,collision=false}
   local ok,result,detail=pcall(gd.model_spawn,asset,opts)
   if ok and handle(result) then s.handles[#s.handles+1]=result
   else why=tostring(ok and detail or result or 'model spawn refused') end
  end
  if why then
   local removed,cleanup_why=R.clear(s)
   s.error=why..(not removed and ('; cleanup refused: '..tostring(cleanup_why)) or '')
   return false,s.error
  end
 end
 s.room_id=plan.room_id;s.theme=plan.theme;s.error=nil
 return true,#s.handles
end
function R.release(s)
 local cleared,why=R.clear(s)
 if not cleared then return false,why end
 for _,name in ipairs(keys(s.assets)) do
  local ok,result,detail=false,nil,nil
  if gd and gd.model_release then ok,result,detail=pcall(gd.model_release,s.assets[name]) end
  -- Native gd.model_release returns zero values; a successful nil is expected.
  if ok and result~=false then s.assets[name]=nil
  else s.error=tostring(ok and detail or result or 'model release unavailable');return false,s.error end
 end
 s.error=nil;return true
end
function R.view(s)
 return {room_id=s.room_id,theme=s.theme,instances=#s.handles,assets=#keys(s.assets),error=s.error,
  handles=copy(s.handles)}
end
return R
