return function(test,fixture,grid)
  local function scene()
    local s,r,d=fixture();grid(s);s.files['missions/test/level.lua']=s.files['missions/test/level.lua']:gsub('camera={','camera={mode="chunk",',1)
    assert(r:command('play test'));local c=r.current;c.doc.level.camera={left=-100,right=500,bottom=-100,top=300};local frame=0
    local function step(x,y,extra)
      frame=frame+1;s.p.x=x;s.p.y=y or 50
      for k,v in pairs(extra or {})do s.p[k]=v end
      local result=d.zones.sample(r.g,c,s.p,frame);assert(result==c.membership)
      assert(d.chunks.update(c.stream,s.p,result));d.camera.tick(r.g,c,s.p)
      assert(c.stream.current==result.committed,'consumers disagree')
      local cam=c.camera;assert(math.abs(s.p.x-cam.x)<=cam.w/2+0.02 and math.abs(s.p.y-cam.y)<=cam.h/2+0.02,'offscreen')
      return result
    end
    step(50,50);return s,r,d,c,step
  end
  test('doorway stop and pre-commit turn are quiet for 300 frames',function()
    local s,r,d,c,step=scene();local room=c.stream.current
    step(99,50,{vx=9});assert(c.stream.current==room)
    local writes=0;local origin=r.g.stage_set_origin;r.g.stage_set_origin=function(...)writes=writes+1;return origin(...)end
    for _=1,300 do step(100,50,{vx=0})end
    assert(c.stream.current==room and c.membership.transitions==0 and writes<=1)
    step(96,50,{vx=-9});step(80,50);assert(c.membership.transitions==0)
  end)
  test('ten genuine crossings never reverse inside an ease and keep player visible',function()
    local s,r,d,c,step=scene();local last=-100;local changes=0
    for _=1,10 do
      for _,x in ipairs({110,90})do
        for i=1,26 do local z=step(x,50);if z.changed then assert(z.frame-last>=24);last=z.frame;changes=changes+1 end end
      end
    end
    assert(changes==20 and c.membership.transitions==20)
  end)
  test('knockback across line and back inside doorway never commits',function()
    local _,_,_,c,step=scene();step(105,50,{vx=20});step(96,50,{vx=-20});step(70,50,{vx=0})
    assert(c.membership.transitions==0)
  end)
  test('genuine knockback return waits for the existing ease to finish',function()
    local _,_,_,c,step=scene();step(110,50,{vx=40});assert(c.membership.transitions==1)
    for _=1,23 do step(90,50,{vx=-40});assert(c.membership.transitions==1)end
    step(90,50,{vx=0});assert(c.membership.transitions==2)
  end)
  test('vertical airborne jump peaks beyond border then returns without commit',function()
    local _,_,_,c,step=scene();step(50,98,{airborne=true});step(50,115);step(50,98);step(50,50,{airborne=false})
    assert(c.membership.transitions==0)
    step(50,115,{airborne=false});assert(c.stream.current.id=='c0_1')
  end)
  test('outside zones retains committed respawn and re-enters another room without snap',function()
    local s,r,d,c,step=scene();local room=c.stream.current
    local w,h=c.camera.w,c.camera.h;local x=c.camera.x
    step(-30,50);assert(c.membership.outside and c.stream.current==room)
    assert(c.camera.w==w and c.camera.h==h and math.abs(c.camera.x-x)<=12)
    for _=1,12 do step(-30,50)end
    step(150,50);assert(not c.membership.outside and c.stream.current.id=='c1_0')
    local log=table.concat(s.logs,';');assert(log:find('zones outside',1,true) and log:find('zones re-enter',1,true))
    assert(r:command('zones'))
  end)
  test('invalid zone tuning refuses before any gameplay write',function()
    local _,_,d=fixture()
    for _,camera in ipairs({{commit_distance=0},{vertical_commit_distance=41},{ease_frames=1.5},
      {curve='bounce'},{doors={door_A_B={mode='follow'}}}})do
      assert(not pcall(d.validator.camera,camera))
    end
    assert(not pcall(d.validator.layout,{version=2,units=6.5,parts={},zones={
      {name='bad',kind='transition',rooms={'a'},rect={left=0,right=10,bottom=0,top=10}}}},{}))
  end)
  test('overlapping authored rooms refuse with both object names',function()
    local _,_,d=fixture()
    local doc={level={zones={{name='one',kind='room',room='a',rect={left=0,right=20,bottom=0,top=20}},
      {name='two',kind='room',room='b',rect={left=10,right=30,bottom=0,top=20}}}},chunks={{id='a'},{id='b'}}}
    local ok,why=pcall(d.zones.prepare,doc);assert(not ok and tostring(why):find('one') and tostring(why):find('two'))
  end)
  test('reload bootstrap preserves the committed room for every published consumer',function()
    local s,r,d,c,step=scene();step(110,50);local room=c.stream.current
    c.zone_state=nil;c.membership=nil
    local m=d.zones.sample(r.g,c,{x=80,y=50},0)
    assert(m.committed==room and c.stream.current==room and not m.changed)
  end)
  test('KO snapshot never chooses a room from the fighter reset position',function()
    local s,r,d,c,step=scene();step(110,50);local room=c.stream.current
    local m=d.zones.sample(r.g,c,{x=0,y=0,action=0},200)
    assert(m.outside and m.committed==room and not m.changed)
  end)
  test('membership source is sampled once and all stationary setters stay quiet',function()
    local s,r,d,c,step=scene();local calls=0;local source=d.zones.source
    d.zones.source=function(...)calls=calls+1;return source(...)end
    local m=step(50,50);local before=calls;assert(d.zones.sample(r.g,c,s.p,m.frame)==m and calls==before)
    local writes=0
    for _,key in ipairs({'stage_set_origin','stage_set_blast_bounds','stage_set_spawn','stage_set_camera_bounds'})do
      local fn=r.g[key];r.g[key]=function(...)writes=writes+1;return fn(...)end
    end
    for _=1,300 do step(50,50)end;assert(writes==0)
  end)
  test('refused streaming preserves one committed room across every consumer',function()
    local s,r,d,c,step=scene();local room=c.stream.current
    -- A remote room would require new window assets; reject the load before publication.
    local load=d.world.load;d.world.load=function()error('window unavailable')end
    s.p.x=350;s.p.y=50;local m=d.zones.sample(r.g,c,s.p,99)
    assert(m.changed);local ok=d.chunks.update(c.stream,s.p,m)
    assert(not ok and m.committed==room and c.zone_state.committed==room and c.stream.current==room)
    d.world.load=load
  end)
  test('per-door settings override level defaults without velocity anticipation',function()
    local s,r,d,c,step=scene();c.doc.zone_data=nil;c.zone_state=nil
    c.doc.level.camera_settings.doors={door_c0_0_c1_0={commit_distance=16,ease_frames=36,curve='linear'}}
    step(50,50);step(110,50,{vx=100});assert(c.stream.current.id=='c0_0')
    step(117,50);assert(c.stream.current.id=='c1_0' and c.membership.tuning.ease_frames==36)
    for _=1,35 do step(80,50);assert(c.stream.current.id=='c1_0');if c.camera.ease then assert(c.camera.ease.curve=='linear' and c.camera.ease.frames==36)end end
    step(80,50);assert(c.stream.current.id=='c0_0')
  end)
  test('authored T junction keeps source room until leaving into one branch',function()
    local s,r,d,c,step=scene()
    c.doc.level.zones={{name='junction',kind='transition',rooms={'c0_0','c1_0','c0_1'},rect={left=85,right=115,bottom=85,top=115}}}
    c.doc.zone_data=nil;c.zone_state=nil;step(50,50)
    step(100,100);assert(c.membership.transition.name=='junction' and c.stream.current.id=='c0_0')
    step(120,95);assert(c.stream.current.id=='c1_0')
  end)
end
