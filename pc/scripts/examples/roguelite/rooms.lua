-- Original authored environment assembly. Core/Dungeon physics remain root-owned.
-- BF sidecars are suppressed with collision=false; explicit route colliders own physics.
local R={version=2,max_instances=28,max_assets=6,unit=6.5}
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
function R.preload_step(s)
 if s.preload_error then return nil,s.preload_error end
 for _,name in ipairs(preload) do if not s.assets[name] then
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
function R.plan(node)
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
