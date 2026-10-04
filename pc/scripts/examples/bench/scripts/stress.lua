-- Original benchmark visual workload. Assets are generated without game/disc data.
local M={posts={},instances={},assets={}}
local chain={
  {'bloom.wgsl',{order=0,half=true,params={threshold=.7,intensity=.5,radius=1}}},
  {'bloom-compose.wgsl',{order=1}},
  {'grade.wgsl',{order=2,params={lift={0,0,0,0},gamma={1,1,1,1},gain={1,1,1,1},saturation=1.1}}},
  {'vignette.wgsl',{order=3,params={strength=.4,radius=.3,softness=.4}}},
  {'outline.wgsl',{order=4,params={width=2,threshold=.01,strength=.6,tint={0,0,0,1}}}},
}
local function call(gd,feature,name,api,...)
  if type(gd[api])~='function' then feature(name,'unsupported',api..' unavailable');return nil end
  local ok,value,err=pcall(gd[api],...)
  local success=ok and value~=false and not(value==nil and err~=nil)
  feature(name,success and 'applied' or 'refused',success and '' or tostring(ok and err or value))
  return success and value or nil
end
function M.post(gd,feature,index)
  local spec=chain[index]
  local h=call(gd,feature,'post_'..spec[1],'post_add','shaders/'..spec[1],spec[2])
  if h then M.posts[#M.posts+1]=h end
  return h
end
function M.surface(gd,cfg,feature,index,variant)
  local variants={'cel-outline.wgsl','rim-light.wgsl','dissolve.wgsl'}
  local path=variants[((variant or index)-1)%3+1]
  local params=path=='dissolve.wgsl' and {.15,.05,20,0,1,.6,.1,.5}
    or path=='rim-light.wgsl' and {.2,.5,1,.4,2} or {4,.22,.8}
  return call(gd,feature,'surface_slot_'..index,'fighter_shader',index,'shaders/'..path,{params=params})
end
function M.models(gd,cfg,feature)
  M.assets={call(gd,feature,'custom_lit_asset','model_load','bench_lit'),call(gd,feature,'custom_glass_asset','model_load','bench_glass')}
  if not M.assets[1] or not M.assets[2] then return end
  local before=type(gd.model_instances)=='function' and #gd.model_instances() or 0
  local requested=math.max(0,256-before)
  for i=1,requested do
    local ok,h,err=pcall(gd.model_spawn,M.assets[(i-1)%2+1],{
      x=((i-1)%32-15.5)*9,y=math.floor((i-1)/32)*9-20,z=(i%3-1)*8,
      collision=false,scale=1,visible=true})
    if not ok or not h then feature('instance_limit_custom_level_materials_glass','partial',tostring(err or h));break end
    M.instances[#M.instances+1]=h
  end
  feature('instance_limit_custom_level_materials_glass',#M.instances==requested and 'applied' or 'partial',
          tostring(before+#M.instances)..'/256 live instances; original synthetic grid, lit and glass')
end
function M.setup(gd,cfg,feature)
  if cfg.scenario~='steady_worst' then return end
  for i=1,#chain do M.post(gd,feature,i) end
  feature('full_post_chain',#M.posts==#chain and 'applied' or 'partial',tostring(#M.posts)..' enabled')
  for i=1,#cfg.roster do M.surface(gd,cfg,feature,i) end
  M.models(gd,cfg,feature)
  feature('all_fighter_surface_shaders','requested','see per-slot results')
  feature('repeated_clank_pass','armed','actual on_clank events create transient passes; no fabricated clash events')
end
function M.tick(gd,cfg,feature,frame)
  if cfg.scenario~='hitch_worst' then return end
  -- Distinct first-use events, spaced far enough for hitch neighbour capture.
  for i=1,#chain do if frame==i*40 then M.post(gd,feature,i) end end
  if frame==260 then M.surface(gd,cfg,feature,1) end
  if frame==280 then M.surface(gd,cfg,feature,2) end
  if frame==300 then M.surface(gd,cfg,feature,1,3) end
  if frame==320 then call(gd,feature,'effect_shader_compile','shader_load','shaders/effect.wgsl',{kind='effect'}) end
  if frame==340 then M.models(gd,cfg,feature) end
  if frame==380 then call(gd,feature,'clank_pass_first_use','post_add','shaders/clank.wgsl',{order=100,duration_frames=12,clock=true}) end
end
function M.clank(gd,cfg,feature,event)
  if cfg.scenario~='steady_worst' then return end
  M.clanks=(M.clanks or 0)+1
  call(gd,feature,'clank_pass_event','post_add','shaders/clank.wgsl',{order=100,duration_frames=12,clock=true})
end
M.hitch_events={{frame=40,feature='shader_bloom'},{frame=80,feature='shader_bloom_compose'},
 {frame=120,feature='shader_grade'},{frame=160,feature='shader_vignette'},{frame=200,feature='shader_outline'},
 {frame=260,feature='surface_cel'},{frame=280,feature='surface_rim'},{frame=300,feature='surface_dissolve'},{frame=320,feature='effect_shader_compile'},{frame=340,feature='custom_model_upload'},
 {frame=380,feature='clank_pass_first_use'}}
return M
