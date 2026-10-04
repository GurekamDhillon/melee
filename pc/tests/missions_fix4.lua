return function(test,fixture)
  local function scene(aspect)
    local s,r,d=fixture();local g=r.g
    local c={doc={level={camera={left=-32.5,right=552.5,bottom=-26,top=234},
      camera_settings={mode='chunk'},blast={left=-130,right=650,bottom=-156,top=364}},
      chunks={},mission={start={x=26,y=19.5}}},stream={},run={respawn={x=26,y=19.5}}}
    for i=0,3 do c.doc.chunks[i+1]={rect={left=i*130,right=(i+1)*130,bottom=0,top=104},level={},spawn={x=i*130+26,y=19.5}}end
    c.stream.current=c.doc.chunks[1]
    s.view={eye={x=65,y=52,z=112/(2*math.tan(math.rad(25)/2))},interest={x=65,y=52,z=0},fov=25,roll=0}
    s.normal=s.view;s.sets=0;s.attaches=0
    g.safe_area=function()return {w=480*16/9,h=480}end -- different from real projection on purpose
    g.camera_get=function()return s.view end
    g.project=function(x,y,z)
      local v=s.view;local depth=v.eye.z-(z or 0);local h=2*depth*math.tan(math.rad(v.fov)/2)
      return (0.5+(x-v.interest.x)/(h*aspect))*480*16/9,
        (0.5-(y-v.interest.y)/h)*480,true,depth
    end
    g.camera_set=function(v)s.view=v;s.sets=s.sets+1 end
    g.camera_attach=function(frames)assert(frames==0);s.view=s.normal;s.attaches=s.attaches+1 end
    return s,g,d,c
  end
  local function near(a,b)assert(math.abs(a-b)<0.00001,('%s ~= %s'):format(a,b))end
  test('actual projected aspect overrides window and authored constants at all ratios',function()
    for _,aspect in ipairs({4/3,16/9,21/9})do
      local s,g,d,c=scene(aspect);c.doc.level.camera_settings.aspect=1
      d.camera.tick(g,c,{x=26,y=19.5,action=14})
      near(c.camera.w,112*aspect)
      assert(s.camera[2]-s.camera[1]<=1.00001,'native clamp must not permit retail-width centre excursion')
    end
  end)
  test('projected outer edges clamp on both sides including native aspect mismatch',function()
    for _,aspect in ipairs({4/3,16/9,21/9})do
      local s,g,d,c=scene(aspect)
      d.camera.tick(g,c,{x=26,y=19.5,action=14})
      local h=2*s.view.eye.z*math.tan(math.rad(s.view.fov)/2)
      near(s.view.interest.x-h*aspect/2,math.max(-32.5,65-h*aspect/2))
      s.normal=s.view;c.stream.current=c.doc.chunks[4]
      s.view={eye={x=490,y=52,z=s.view.eye.z},interest={x=490,y=52,z=0},fov=25,roll=0}
      d.camera.tick(g,c,{x=500,y=19.5,action=14})
      near(s.view.interest.x+h*aspect/2,552.5)
    end
  end)
  test('level narrower than visible footprint centres without changing authored zoom',function()
    local s,g,d,c=scene(21/9);c.doc.level.camera={left=0,right=130,bottom=0,top=104}
    d.camera.tick(g,c,{x=26,y=19.5,action=14})
    near(c.camera.x,65);near(c.camera.w,112*21/9);near(c.camera.h,112)
    near(s.view.interest.x,65);near(s.view.interest.y,52)
  end)
  test('far respawn cuts eye and interest immediately then attaches after native update',function()
    local s,g,d,c=scene(16/9);c.doc.level.camera={left=0,right=3000,bottom=0,top=500}
    c.doc.level.camera_settings={mode='follow',min_dist=300}
    s.origin={x=2468,y=100};s.view={eye={x=2468,y=100,z=300},interest={x=2468,y=100,z=0},fov=25,roll=0};s.normal=s.view
    d.camera.tick(g,c,{x=2468,y=100,action=14});local sets=s.sets
    d.camera.tick(g,c,{x=26,y=32.5,action=13})
    assert(s.view.eye.x<200 and s.view.interest.x<200 and s.sets==sets+1 and s.attaches==0)
    s.normal=s.view -- next completed native update starts from the committed cut
    d.camera.tick(g,c,{x=26,y=32.5,action=13})
    assert(s.attaches==1 and s.view.eye.x<200 and s.sets==sets+1)
    d.camera.stop(g);assert(not d.camera.cut_owned)
  end)
  test('transient bench refusals are quiet and successful reserve logs once',function()
    local s,g,d,c=scene(16/9);c.doc.level.fighters={};local tries=0;local benched=false
    g.player=function(port)if port==2 then return {cpu=true,x=10,y=10,action=14}end end
    g.fighter_bench=function()tries=tries+1;benched=tries>=3;return benched,'unsafe action or held item' end
    g.fighter_benched=function()return benched end
    g.cpu_mode=function()return true end
    g.teleport=function()end
    for _=1,60 do d.fighters.tick(g,c)end;assert(#s.logs==0)
    for _=1,60 do d.fighters.tick(g,c)end
    assert(#s.logs==1 and s.logs[1]:find('CPU 2 benched',1,true))
  end)
  test('cut remains owned through repeated ticks in same completed logic frame',function()
    local s,g,d,c=scene(16/9);local frame=10;g.frame=function()return frame end
    c.doc.level.camera={left=0,right=3000,bottom=0,top=500};c.doc.level.camera_settings={mode='follow',min_dist=300}
    s.origin={x=2468,y=100};s.view={eye={x=2468,y=100,z=300},interest={x=2468,y=100,z=0},fov=25,roll=0};s.normal=s.view
    d.camera.tick(g,c,{x=2468,y=100,action=14})
    d.camera.tick(g,c,{x=26,y=32.5,action=13});d.camera.tick(g,c,{x=26,y=32.5,action=13})
    assert(s.attaches==0 and s.view.eye.x<200)
    s.normal=s.view;frame=11;d.camera.tick(g,c,{x=26,y=32.5,action=13});assert(s.attaches==1)
  end)
  test('projected footprint inversion accounts for perspective depth on plane',function()
    local _,_,d=fixture();local g={}
    g.project=function(x,y)
      local depth=100+0.1*x
      return (40000+200*x)/depth,(24000-200*y)/depth,true,depth
    end
    local b=d.camera.footprint(g,{interest={x=0,y=0}},{w=800,h=480})
    near(b.left,-200);near(b.right,1000/3)
    near(b.bottom,-200);near(b.top,120)
  end)

end
