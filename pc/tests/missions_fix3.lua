-- Measured camera contracts: native rendering remains integrator acceptance.
return function(test,fixture)
  local function setup(mode)
    local s,r,d=fixture()
    local c={doc={level={camera={left=0,right=3000,bottom=0,top=1000},
      camera_settings={mode=mode},blast={left=-100,right=3100,bottom=-200,top=1200},fighters={}},
      chunks={},mission={start={x=10,y=10}}},stream={},run={respawn={x=10,y=10}}}
    return s,r.g,d,c
  end
  local function near(a,b)assert(math.abs(a-b)<0.00001,('%s ~= %s'):format(a,b))end
  test('rectangle plus settings retains exporter camera mode and measured distance',function()
    local _,_,d=fixture()
    local l=d.validator.layout({version=2,units=6.5,parts={},camera={left=0,right=650,bottom=0,top=312,
      mode='chunk',window={w=130,h=104},min_dist=252,fov=25}}, {})
    assert(l.camera.right==650 and l.camera_settings.mode=='chunk' and l.camera_settings.min_dist==252)
  end)
  test('every origin step preserves authored blast edges without native tween',function()
    local s,g,d,c=setup('follow');local writes=0;local blast=g.stage_set_blast_bounds
    g.stage_set_blast_bounds=function(...)writes=writes+1;return blast(...)end
    for x=200,2800 do
      d.camera.tick(g,c,{x=x,y=300,action=14,vx=1.4,facing=1})
      local b=g.stage_bounds().blast
      near(b.left,-100);near(b.right,3100);near(b.bottom,-200);near(b.top,1200)
      assert(s.origin.frames==0)
    end
    assert(writes>2500,'sub-30-unit movements must be compensated')
  end)
  test('respawn and long teleport snap instead of freezing or sweeping',function()
    local s,g,d,c=setup('follow')
    d.camera.tick(g,c,{x=2468,y=300,action=14})
    d.camera.tick(g,c,{x=10,y=10,action=13})
    assert(s.origin.x<200 and s.origin.frames==0)
    d.camera.tick(g,c,{x=2000,y=300,action=14})
    assert(math.abs(s.origin.x-2000)<50 and s.origin.frames==0)
    d.camera.tick(g,c,nil);assert(s.origin.x<200)
  end)
  test('follow lookahead leads motion and facing with measured footprint',function()
    local s,g,d,c=setup('follow')
    for i=1,100 do d.camera.tick(g,c,{x=500+i*1.4,y=300,action=14,vx=1.4,facing=1})end
    local p=640
    assert(s.origin.x-p>=8 and s.origin.x-p<=40)
    near(s.params.track_smooth,1.8)
    local h=2*s.params.min_dist*math.tan(math.rad(s.params.fov)/2)
    near(h*(16/9),250);near(h,140.625)
    c.doc.level.camera_settings.leash=0;c.doc.level.camera_settings.look_ahead=12
    c.doc.level.camera_settings.velocity_lead=0
    d.camera.tick(g,c,{x=640,y=300,action=14,vx=0,facing=-1});near(s.origin.x,628)
  end)
  local function grid(c)
    c.doc.level.camera={left=0,right=650,bottom=0,top=312}
    for i=0,4 do c.doc.chunks[i+1]={rect={left=i*130,right=(i+1)*130,bottom=104,top=208},
      level={},spawn={x=i*130+10,y=156}}end
    c.stream.current=c.doc.chunks[1]
  end
  test('130 by 104 chunk fits height and clamps actual width at outer ends',function()
    local s,g,d,c=setup('chunk');grid(c)
    d.camera.tick(g,c,{x=5,y=156,action=14})
    local h=2*s.params.min_dist*math.tan(math.rad(25)/2)
    near(h,112);near(h*16/9,199.11111111111)
    assert(s.origin.x-h*16/9/2>=-0.00001)
    c.stream.current=c.doc.chunks[5]
    d.camera.tick(g,c,{x=645,y=156,action=14})
    assert(s.origin.x+h*16/9/2<=650.00001)
  end)
  test('room target stays committed before border and safety bias retains fast player',function()
    local s,g,d,c=setup('chunk');grid(c)
    d.camera.tick(g,c,{x=65,y=156,action=14,vx=0})
    d.camera.tick(g,c,{x=110,y=156,action=14,vx=2,facing=1})
    assert(c.camera.tx==65,'room target must not anticipate border 130')
    for x=116,620,6 do
      c.stream.current=c.doc.chunks[math.min(5,math.floor(x/130)+1)]
      d.camera.tick(g,c,{x=x,y=156,action=14,vx=6})
      local half=2*s.params.min_dist*math.tan(math.rad(25)/2)*16/9/2
      assert(math.abs(x-s.origin.x)<=half+0.00001,'player outside during transition')
      assert(s.origin.frames==0)
    end
  end)
  test('shaft fast ascent descent uses velocity lead and taller measured view',function()
    local s,g,d,c=setup('shaft');c.doc.level.camera_settings.x=1500
    for _,vy in ipairs({8,-8})do
      for i=1,60 do
        local y=vy>0 and 200+i*8 or 800-i*8
        d.camera.tick(g,c,{x=1500,y=y,action=14,vy=vy})
        near(s.origin.x,1500);assert(math.abs(y-s.origin.y)<130)
      end
    end
    near(2*s.params.min_dist*math.tan(math.rad(20)/2),260)
  end)
  test('bench excludes unused CPU until explicit fighter participation and restores on stop',function()
    local s,g,d,c=setup('follow');local cpu={cpu=true,x=20,y=30,action=14,falls=0};local benched=false
    local calls,bench=0,0
    g.player=function(port)if port==2 then return cpu end end
    g.fighter_bench=function(port)assert(port==2);bench=bench+1;benched=true;return true end
    g.fighter_benched=function()return benched end
    g.fighter_call=function(port,x,y)assert(port==2);calls=calls+1;benched=false;cpu.x=x;cpu.y=y;return true end
    g.cpu_mode=function()return true end
    d.fighters.tick(g,c);assert(benched and bench==1 and calls==0)
    for _=1,60 do d.fighters.tick(g,c)end;assert(bench==1 and calls==0)
    c.doc.level.fighters[2]='fight';d.fighters.tick(g,c);assert(not benched and calls==1)
    c.doc.level.fighters[2]='stand';d.fighters.tick(g,c);assert(benched)
    d.fighters.stop(g,c);assert(not benched and calls==2)
  end)
  test('bench refusal falls back to stand and later retries bench without spawning humans',function()
    local s,g,d,c=setup('follow');local cpu={cpu=true,x=20,y=30,action=14};local tries,stand=0,0
    g.player=function(port)if port==2 then return cpu elseif port==3 then return {cpu=false}end end
    g.fighter_bench=function(port)assert(port==2);tries=tries+1;return tries>1,'temporary refusal' end
    g.fighter_benched=function()return tries>1 end
    g.cpu_mode=function(port,mode)assert(port==2 and mode=='stand');stand=stand+1;return true end
    local tp=g.teleport;g.teleport=function(port,x,y)assert(port==2);cpu.x=x;cpu.y=y end
    d.fighters.tick(g,c);assert(stand==1 and cpu.x==10)
    for _=1,35 do d.fighters.tick(g,c)end;assert(tries>=2)
  end)
  test('partial exporter rectangle refuses and camera tuning ranges validate',function()
    local _,_,d=fixture()
    for _,camera in ipairs({{right=100,mode='chunk'},{leash=-1},{aspect=0},{door_margin=101},
      {look_ahead=0/0},{vertical_lead=31},{safe_margin=false}})do
      assert(not pcall(d.validator.layout,{version=2,units=6.5,parts={},camera=camera},{}))
    end
  end)
  test('default chunk dimensions derive 130 by 104 measured frame',function()
    local _,_,d=fixture();local params,w,h=d.camera.geometry({mode='chunk'},nil,nil)
    near(w,199.11111111111);near(h,112);near(params.min_dist,112/(2*math.tan(math.rad(25)/2)))
  end)
  test('narrow outer level centres the unchanged view',function()
    local s,g,d,c=setup('shaft');c.doc.level.camera={left=0,right=130,bottom=0,top=1000}
    c.doc.level.camera_settings.x=65
    d.camera.tick(g,c,{x=65,y=500,action=14,vy=-8})
    local h=2*s.params.min_dist*math.tan(math.rad(20)/2)
    near(h,260);near(s.origin.x,65)
    assert(math.abs(s.origin.y-500)<=h/2,'speed lead must fit shortened view')
  end)
  test('compensation refusal restores old origin and world blast edges',function()
    local s,g,d,c=setup('follow');d.camera.tick(g,c,{x=500,y=300,action=14})
    local old=s.origin.x;local blast=g.stage_set_blast_bounds;local refused=false
    g.stage_set_blast_bounds=function(...)if not refused then refused=true;return false end;return blast(...)end
    assert(not pcall(d.camera.tick,g,c,{x=550,y=300,action=14}))
    near(s.origin.x,old);near(g.stage_bounds().blast.left,-100)
  end)

  test('reload retains owned reserve without calling unused port into the level',function()
    local s,r=fixture();local player=r.g.player;local cpu={cpu=true,x=30,y=40,action=14};local reserve=false
    local calls,benches=0,0
    r.g.player=function(port)if port==1 then return player()elseif port==2 then return cpu end end
    r.g.fighter_bench=function()reserve=true;benches=benches+1;return true end
    r.g.fighter_benched=function()return reserve end
    r.g.fighter_call=function()reserve=false;calls=calls+1;return true end
    assert(r:command('play test'));assert(reserve)
    assert(r:command('reload'));assert(reserve and calls==0 and benches==1)
    r:stop('stop');assert(not reserve and calls==1)
  end)
  test('fighter policy cannot modify another script reserve controller',function()
    local _,g,d,c=setup('follow');c.doc.level.fighters[2]='fight'
    g.player=function(port)if port==2 then return {cpu=true,x=10,y=10,action=14}end end
    g.fighter_benched=function()return true end
    local writes=0;g.cpu_mode=function()writes=writes+1;return true end
    g.fighter_call=function()error('foreign reserve called')end
    d.fighters.tick(g,c);assert(writes==0 and not c.fighters[2].configured)
  end)

end
