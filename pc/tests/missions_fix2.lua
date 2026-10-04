-- Follow-up contracts share the offline engine fixture, never run the game.
return function(test,fixture,chunks,level,mission,count)
  local function camera(g,s)
    g.stage_set_origin=function(x,y,o) s.origin={x=x,y=y,frames=o and o.frames or 0};return true end
    g.camera_params=function(v) local old=s.params or {};if v==nil then s.params=nil elseif next(v) then s.params=v end;return old end
  end
  test('clear waits for respawn and re-arms after another death',function()
    local s,r=fixture();assert(r:command('play test'))
    local attack=r.g.fly_attack;local refused=true
    r.g.fly_attack=function(...)if refused then error('fly_attack: state cannot fly')end;return attack(...)end
    assert(r:command('clear'));assert(r.current.clear)
    for _=1,20 do r:frame() end
    refused=false;r:frame();assert(s.attacking)
    s.attacking=false;s.flying=false;s.p.falls=1;refused=true;r:frame()
    refused=false;for _=1,10 do r:frame() end;assert(s.attacking)
  end)
  test('play and reload wait before mutating world and stop cancels wait',function()
    local s,r=fixture();s.p.action=12;s.reject_teleport=true
    assert(r:command('play test'));assert(not r.current and s.loads==0)
    for _=1,20 do r:frame()end;assert(s.loads==0)
    s.p.action=14;s.reject_teleport=false;for _=1,10 do r:frame()end;assert(r.current)
    local old=r.current;s.p.action=12;assert(r:command('reload'))
    for _=1,10 do r:frame()end;assert(r.current==old)
    s.p.action=14;for _=1,10 do r:frame()end;assert(r.current~=old)
    s.p.action=12;assert(r:command('play test'));r:stop('cancel');s.p.action=14;r:frame();assert(not r.current)
  end)
  test('unready commands expire once with clear timeout',function()
    local s,r=fixture();s.p.action=12;assert(r:command('play test'))
    for _=1,620 do r:frame()end
    assert(not r.pending and not r.current)
    assert(table.concat(s.logs,'\n'):find('timed out waiting for controllable P1',1,true))
  end)
  test('every horizontal and vertical border holds both inclusive zone ends',function()
    local s,r=fixture();chunks(s);assert(r:command('play test'))
    for _,axis in ipairs({'x','y'})do
      for border=100,200,100 do
        local margin=axis=='x' and 8 or 12
        s.p.x=150;s.p.y=150;s.p[axis]=border-margin-2;r:frame()
        s.p[axis]=border+margin+0.4;r:frame();local current=r.current.stream.current
        for _,v in ipairs({border-margin,border+margin-0.2,border-margin+0.1,border+margin})do
          s.p[axis]=v;r:frame();assert(r.current.stream.current==current)
        end
        s.p[axis]=border-margin-0.1;r:frame();assert(r.current.stream.current~=current)
        current=r.current.stream.current;s.p[axis]=border+margin;r:frame();assert(r.current.stream.current==current)
      end
    end
  end)
  test('CPU fallback stand parks once and re-applies after respawn while humans stay untouched',function()
    local s,r=fixture();assert(r:command('play test'));local player=r.g.player;local cpu={cpu=true,x=0,y=-100,falls=0,action=14};local calls={}
    r.g.player=function(port)if port==1 then return player()elseif port==2 then return cpu end end
    r.g.cpu_mode=function(port,mode)calls[#calls+1]={port,mode};return port==2 end
    local tp=r.g.teleport;r.g.teleport=function(port,x,y)if port==2 then cpu.x=x;cpu.y=y else tp(port,x,y)end end
    r:frame();assert(cpu.y==10 and calls[1][2]=='stand')
    local n=#calls;for _=1,30 do r:frame()end;assert(#calls==n)
    cpu.falls=1;cpu.action=12;r:frame();cpu.action=14;for _=1,6 do r:frame()end;assert(#calls>n and cpu.y==10)
  end)
  test('fight opt-in is validated and CPU is called after staging',function()
    local s,r=fixture();local player=r.g.player;local cpu={cpu=true,x=99,y=99,falls=0,action=14};local mode
    r.g.player=function(port)if port==1 then return player()elseif port==2 then return cpu end end
    r.g.cpu_mode=function(_,m)mode=m;return true end
    local benched=false
    r.g.fighter_bench=function()benched=true;return true end
    r.g.fighter_benched=function()return benched end
    r.g.fighter_call=function(_,x,y)assert(s.hidden);benched=false;cpu.x=x;cpu.y=y;return true end
    s.files['missions/test/level.lua']=level:gsub('units=6.5','units=6.5,fighters={[2]="fight"}')
    assert(r:command('play test'));r:frame();assert(mode=='fight' and cpu.x==0 and not benched)
    s.files['missions/test/level.lua']=level:gsub('units=6.5','units=6.5,fighters={[1]="stand"}')
    assert(not r:command('reload'))
  end)
  test('follow camera leash clamp zero gains restores without touching input',function()
    local s,r=fixture();camera(r.g,s)
    s.files['missions/test/level.lua']=level:gsub('camera={.-},blast=', 'camera={mode="follow",window={w=250,h=180},ends={left=0,right=1000,bottom=0,top=500}},blast=')
    assert(r:command('play test'));s.p.x=400;s.p.y=200;r:frame()
    assert(s.origin.x==400 and s.origin.y==200 and s.params.yaw_gain==0 and s.params.pitch_gain==0)
    s.p.x=2000;r:frame();assert(s.origin.x==875)
    r:stop('stop');assert(s.params==nil)
  end)
  test('chunk camera uses child override measured room sizing and Lua transition',function()
    local s,r=fixture();camera(r.g,s);chunks(s)
    s.files['missions/test/level.lua']=s.files['missions/test/level.lua']:gsub('camera={.-},blast=', 'camera={left=0,right=500,bottom=0,top=300},blast=')
    s.files['missions/test/chunks/c1_1/level.lua']=s.files['missions/test/chunks/c1_1/level.lua']:gsub('units=6.5','units=6.5,camera={mode="chunk",fov=25}')
    assert(r:command('play test'));s.p.x=150;s.p.y=150;r:frame()
    assert(s.origin.frames==0 and math.abs(s.p.x-s.origin.x)<60 and math.abs(s.p.y-s.origin.y)<60)
    assert(s.params.fov==25 and math.abs(2*s.params.min_dist*math.tan(math.rad(25)/2)-108)<0.00001)
  end)
  test('shaft fixes x while y follows and invalid camera is atomic',function()
    local s,r=fixture();camera(r.g,s)
    s.files['missions/test/level.lua']=level:gsub('camera={.-},blast=', 'camera={mode="shaft",x=25},blast=')
    assert(r:command('play test'));s.p.x=500;s.p.y=200;r:frame()
    assert(s.origin.x==25 and s.origin.y==200 and math.abs(2*s.params.min_dist*math.tan(math.rad(20)/2)-260)<0.00001)
    local old=r.current
    for _,value in ipairs({'{mode="bad"}','{mode="follow",fov=90}','{mode="follow",window={w=0,h=1}}','{mode="follow",min_dist=300,max_depth=100}'})do
      s.files['missions/test/level.lua']=level:gsub('camera={.-},blast=', 'camera='..value..',blast=')
      assert(not r:command('reload'));assert(r.current==old)
    end
  end)
  test('label spelling preserves per-instance label with type fallback',function()
    local s,r=fixture();local labels={};r.g.model_label=function(h,n)labels[h]=n end
    s.files['missions/test/level.lua']=level:gsub('part="custom"','part="custom",label="Wall1"')
    assert(r:command('play test'));assert(labels[next(s.models)]=='Wall1')
  end)
  test('held states defer reload and permanent cursor ownership fails immediately',function()
    local s,r=fixture();assert(r:command('play test'));local old=r.current
    for _,action in ipairs({223,232,239,243,266,339})do
      s.p.action=action;assert(r:command('reload'));r:frame();assert(r.current==old)
    end
    s.p.action=14;for _=1,6 do r:frame()end;assert(r.current~=old)
    r.g.fly_attack=function()error('cursor belongs to another script')end
    assert(r:command('clear'));assert(not r.current.clear)
  end)
  test('clear target and arming refusals time out in 600 logic frames',function()
    for _,operation in ipairs({'fly_target','fly_attack'})do
      local s,r=fixture();assert(r:command('play test'))
      r.g[operation]=function()error('state cannot fly (dead, held, respawning)')end
      assert(r:command('clear'));for _=1,610 do r:frame()end
      assert(not r.current.clear and not s.attacking)
      assert(table.concat(s.logs,'\n'):find('clear timed out waiting for controllable P1',1,true))
    end
  end)
  test('tour retries a respawn refusal rather than skipping its marker',function()
    local s,r=fixture();assert(r:command('play test'));s.p.x=33;local target=r.g.fly_target;local tries=0
    r.g.fly_target=function(...)tries=tries+1;if tries<5 then error('state cannot fly')end;return target(...)end
    assert(r:command('tour'));for _=1,8 do r:frame()end
    assert(s.target and s.target.x==0 and r.current.tour.index==1)
  end)
  test('camera API refusal rolls back world origin and zoom before retiring old world',function()
    local s,r=fixture();camera(r.g,s);assert(r:command('play test'));local old=r.current
    local origin=s.origin.x;local set=r.g.camera_params
    r.g.camera_params=function(v)if v and v.fov==40 then error('camera owned by another script')end;return set(v)end
    s.files['missions/test/level.lua']=level:gsub('camera={.-},blast=', 'camera={mode="follow",fov=40},blast=')
    assert(not r:command('reload'));assert(r.current==old and count(s.models)==1 and s.origin.x==origin)
  end)

  test('malformed camera options and conflicting distant child refuse before staging',function()
    local s,r=fixture();assert(r:command('play test'));local old=r.current;local loads=s.loads
    for _,value in ipairs({'{window=false}','{ends=false}'})do
      s.files['missions/test/level.lua']=level:gsub('camera={.-},blast=', 'camera='..value..',blast=')
      assert(not r:command('reload'));assert(r.current==old and s.loads==loads)
    end
    chunks(s)
    s.files['missions/test/level.lua']=s.files['missions/test/level.lua']:gsub('camera={.-},blast=', 'camera={min_dist=300,max_depth=300},blast=')
    s.files['missions/test/chunks/c4_2/level.lua']=s.files['missions/test/chunks/c4_2/level.lua']:gsub('units=6.5','units=6.5,camera={min_dist=1000}')
    assert(not r:command('reload'));assert(r.current==old and s.loads==loads)
  end)

end
