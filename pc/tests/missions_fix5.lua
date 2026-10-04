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
  test('600 stationary clamped frames do not cut or write setters after initialization',function()
    local s,g,d,c=scene(16/9);local writes=0;local frame=0
    g.frame=function()return frame end
    for _,key in ipairs({'camera_params','stage_set_origin','stage_set_blast_bounds','stage_set_camera_bounds'})do
      local fn=g[key];g[key]=function(...)writes=writes+1;return fn(...)end
    end
    local project=g.project
    g.project=function(x,y,z)
      local a,b,v,depth=project(x,y,z)
      return a+(x-s.view.interest.x)*(frame%2==0 and 0.00001 or -0.00001),b,v,depth
    end
    d.camera.tick(g,c,{x=65,y=32.5,action=14,falls=0})
    local first=writes;local cuts=s.sets
    for i=1,600 do
      frame=i;s.normal=s.view
      d.camera.tick(g,c,{x=65,y=32.5,action=14,falls=0})
    end
    assert(writes==first,'stationary setters: '..(writes-first))
    assert(s.sets==cuts,'stationary cuts: '..(s.sets-cuts))
    near(s.view.interest.x-c.camera.w/2,-32.5)
  end)
  test('one cut for far respawn even when rebirth persists for 600 frames',function()
    local s,g,d,c=scene(16/9)
    c.doc.level.camera={left=0,right=3000,bottom=0,top=500}
    c.doc.level.camera_settings={mode='follow',min_dist=300}
    s.origin={x=2468,y=100};s.view={eye={x=2468,y=100,z=300},interest={x=2468,y=100,z=0},fov=25,roll=0};s.normal=s.view
    d.camera.tick(g,c,{x=2468,y=100,action=14,falls=0});local cuts=s.sets
    for i=1,600 do
      s.normal=s.view
      d.camera.tick(g,c,{x=26,y=32.5,action=13,falls=1})
    end
    assert(s.sets==cuts+1)
  end)
  test('CPU bench precedes world writes and P1 stays held through hiding; percent restored',function()
    local s,r,d=fixture();local g=r.g;local benched=false;s.p.percent=7
    local player=g.player;g.player=function(port)
      if port==2 then return {cpu=true,x=10,y=10,action=14}end
      if port==1 then return player(port)end
    end
    g.fighter_benched=function()return benched end
    g.fighter_bench=function()benched=true;return true end
    local load=d.world.load;d.world.load=function(...)assert(benched,'world before bench');return load(...)end
    local hide=g.stage_hide;g.stage_hide=function(v)
      if v then assert(s.holding,'hidden before P1 reserve');s.p.percent=33 end
      return hide(v)
    end
    g.set_damage=function(port,value)assert(port==1);s.p.percent=value end
    s.p.x=90;r:command('play test')
    assert(r.current,table.concat(s.logs,';'));assert(s.p.percent==7,'load damage retained')
  end)
  test('unsafe CPU defers load before world or hide writes then retries',function()
    local s,r,d=fixture();local g=r.g;local safe=false;local bench=false
    local player=g.player;g.player=function(port)
      if port==2 then return {cpu=true,x=10,y=10,action=14}end
      if port==1 then return player(port)end
    end
    g.fighter_benched=function()return bench end
    g.fighter_bench=function()bench=safe;return bench,'held item' end
    s.manual_stage=true;assert(r:command('play test'));assert(r.staging and not r.current)
    for _=1,20 do r:frame()end
    assert(s.loads==0 and not s.hidden and r.staging.phase=='prepare')
    safe=true;s.manual_stage=false;r:frame()
    assert(r.current and not r.pending)
  end)
  test('authored starting percent overrides snapshot and invalid percent refuses',function()
    local s,r=fixture();s.p.percent=7
    s.files['missions/test/level.lua']=s.files['missions/test/level.lua']:gsub('version=2','starting_percent=42,version=2')
    r.g.set_damage=function(_,v)s.p.percent=v end
    r:command('play test');assert(r.current and s.p.percent==42)
    for _,v in ipairs({'-1','1000','1.5','"bad"'})do
      local t,q=fixture();t.files['missions/test/level.lua']=t.files['missions/test/level.lua']:gsub('version=2','starting_percent='..v..',version=2')
      q:command('play test');assert(not q.current and t.loads==0)
    end
  end)

  test('failed staged load restores P1 damage and retains match safety reserve',function()
    local s,r=fixture();local g=r.g;local player=g.player;local bench=false;s.p.percent=7
    g.player=function(port)if port==1 then return player()elseif port==2 then return {cpu=true,x=90,y=10,action=14}end end
    g.fighter_benched=function()return bench end
    g.fighter_bench=function()bench=true;return true end
    g.fighter_call=function(_,x,y)assert(x==90 and y==10);bench=false;return true end
    g.set_damage=function(_,v)s.p.percent=v end
    local hide=g.stage_hide;g.stage_hide=function(v)if v then s.p.percent=33;error('hide refused')end;return hide(v)end
    r:command('play test');assert(not r.current and bench and s.p.percent==7)
  end)

end
