return function(test,fixture,chunks)
  local function scene()
    local s,r,d=fixture();s.manual_stage=true;chunks(s)
    s.files['missions/test/mission.lua']='return {start={x=150,y=150},goal={x=290,y=290,w=10,h=10},objective={type="reach_goal"}}'
    for path,text in pairs(s.files)do
      if path:find('/chunks/',1,true) then
        s.files[path]=text:gsub('parts={{(.-)}}','parts={{%1},{%1},{%1},{%1},{%1}}')
      end
    end
    s.p.percent=7
    return s,r,d
  end
  test('nine chunks stage nearest first with fresh bounded work each frame',function()
    local s,r,d=scene();r.g.teleport=function()error('teleport refuses frozen reserve')end;local instructions,ms=0,0;local peaks={};local loads={}
    local load=d.world.load;d.world.load=function(g,name,level,...)
      loads[#loads+1]=name;instructions=instructions+60000+#level.parts*20000;ms=ms+4
      return load(g,name,level,...)
    end
    for _,key in ipairs({'model_load','model_spawn','model_set','area_load','stage_set_camera_bounds','stage_set_blast_bounds','stage_set_origin','stage_hide','fighter_bench','fighter_call'})do
      local fn=r.g[key];r.g[key]=function(...)ms=ms+(key=='model_load' and 8 or 0.05);return fn(...)end
    end
    assert(r:command('play test'));assert(not r.current and s.loads==0,'command did synchronous install')
    for frame=1,100 do
      instructions,ms=0,0;local before=s.loads
      debug.sethook(function()instructions=instructions+100 end,'',100)
      local ok,why=pcall(r.frame,r);debug.sethook();assert(ok,why)
      assert(s.loads-before<=1,'multiple areas in one callback')
      assert(instructions<500000 and ms<12.5,'step exceeded quarter budget')
      peaks[#peaks+1]={instructions,ms}
      if r.staging then assert(s.holding and not r.current,'unsafe staging');assert(not s.params,'camera moved before commit')end
      if r.current then break end
    end
    assert(r.current and not r.staging and not s.holding)
    assert(#loads==10 and loads[2]:match('5$'),'centre chunk must load before neighbours')
    assert(s.p.percent==7)
    local max_i,max_ms=0,0;for _,p in ipairs(peaks)do max_i=math.max(max_i,p[1]);max_ms=math.max(max_ms,p[2])end
    print(('fix6 staged budget: %d callbacks, peak %d instructions, %.2f simulated ms'):format(#peaks,max_i,max_ms))
  end)
  test('mid-staging refusal keeps old world and restores held player and camera',function()
    local s,r,d=scene();r:command('play test');for i=1,100 do if not r.staging then break end;r:frame()end
    local old=r.current;assert(old);local x,y=s.p.x,s.p.y;local origin=s.origin
    local load=d.world.load;local n=0
    d.world.load=function(...)n=n+1;if n==4 then error('staged area refused')end;return load(...)end
    r:command('reload');assert(r.current==old)
    for i=1,100 do if not r.staging then break end;r:frame();assert(r.current==old)end
    assert(not r.staging and not s.holding and s.p.x==x and s.p.y==y and s.p.percent==7)
    assert(s.origin.x==origin.x and s.origin.y==origin.y and s.hidden)
    for _,a in pairs(s.models)do assert(a.opts.visible~=false,'candidate model leaked')end
    assert(table.concat(s.logs,';'):find('staged area refused',1,true))
  end)
  test('first match frame benches CPU without a play command',function()
    local s,r=fixture();s.manual_stage=true;local player=r.g.player;local bench=false
    r.g.player=function(port)if port==1 then return player()elseif port==2 then return {cpu=true,x=10,y=10,action=14}end end
    r.g.fighter_bench=function(port)assert(port==2);bench=true;return true end
    r.g.fighter_benched=function(port)return port==2 and bench end
    r:frame();assert(bench and not r.current and s.loads==0)
  end)
  test('commit damage refusal restores parked old geometry, bounds, spawns and damage',function()
    local s,r=scene();r:command('play test');for i=1,100 do if not r.staging and not r.retiring then break end;r:frame()end
    local old=r.current;local before=r.g.stage_bounds();local spawn=s.spawns[4];local x,y=s.p.x,s.p.y
    local damage=r.g.set_damage;local refuse=true
    s.files['missions/test/level.lua']=s.files['missions/test/level.lua']:gsub('version=2','version=2,starting_percent=42')
    r.g.set_damage=function(port,n)if refuse then refuse=false;error('commit damage refused')end;damage(port,n)end
    r:command('reload')
    for i=1,100 do if not r.staging then break end;r:frame();assert(r.current==old)end
    assert(not r.staging and not s.holding and s.hidden)
    assert(s.p.x==x and s.p.y==y and s.p.percent==7)
    local after=r.g.stage_bounds()
    for _,k in ipairs({'camera','blast','origin'})do for field,value in pairs(before[k])do assert(after[k][field]==value)end end
    assert(s.spawns[4].x==spawn.x and s.spawns[4].y==spawn.y)
    for _,model in pairs(s.models)do assert(model.opts.visible and model.opts.y<1000,'parked geometry leaked')end
  end)
  test('stop during staging safely cancels, and overlapping play is refused',function()
    local s,r=scene();r:command('play test');for i=1,5 do r:frame()end
    assert(r.staging and s.holding);local transaction=r.staging
    assert(not r:command('play test') and r.staging==transaction)
    r:command('stop');for i=1,100 do if not r.staging then break end;r:frame()end
    assert(not r.staging and not r.current and not s.holding and not s.hidden and not next(s.models))
  end)
  test('scene teardown abandons partial staging without leaving a transaction',function()
    local s,r=scene();r:command('play test');for i=1,7 do r:frame()end
    s.ended=true;r:frame();assert(not r.staging and not r.current and not next(s.models))
  end)

  test('failed first install releases held camera on the next completed frame',function()
    local s,r,d=scene();local pose={eye={x=0,y=100,z=300},interest={x=0,y=100,z=0},fov=25,roll=0}
    local attached=0;r.g.camera_get=function()return pose end
    r.g.camera_set=function(v)assert(v.interest.x==0);pose=v end
    r.g.camera_attach=function()attached=attached+1 end
    s.files['missions/test/level.lua']='return {}'
    r:command('play test');for i=1,100 do if not r.staging then break end;r:frame()end
    assert(not r.current and not s.holding and d.camera.cut_owned)
    r:frame();assert(attached==1 and not d.camera.cut_owned)
  end)

end
