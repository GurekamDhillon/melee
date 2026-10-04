return function(test,fixture)
  local function scene()
    local s,r,d=fixture();s.manual_stage=true;s.p.percent=0
    r.g.set_damage=function(port,value)assert(port==1);s.p.percent=value;s.damage_calls=(s.damage_calls or 0)+1 end -- native C returns zero Lua values
    local hide=r.g.stage_hide;r.g.stage_hide=function(v)if v then s.p.percent=33 end;return hide(v)end
    return s,r,d
  end
  local function step(r,n)for _=1,n do r:frame()end end
  test('native void damage setter commits instead of entering rollback',function()
    local s,r=scene();r:command('play test');step(r,80)
    assert(r.current and not r.staging and not s.holding and s.damage_calls==1,table.concat(s.logs,';'))
  end)
  test('native void damage setter finishes rollback and stops repeated restoration',function()
    local s,r=scene();local hide=r.g.stage_hide
    r.g.stage_hide=function(v)if v then s.p.percent=33;error('one-shot hide refused')end;return hide(v)end
    r:command('play test');step(r,80)
    assert(not r.staging and not s.holding and not r.current and s.p.percent==0)
    local writes=0
    for _,key in ipairs({'stage_hide','stage_set_camera_bounds','stage_set_blast_bounds','stage_set_origin','camera_params','set_damage','stage_set_spawn','model_set'})do
      local fn=r.g[key];r.g[key]=function(...)if key~='camera_params' or select(1,...)==nil or next(select(1,...)) then writes=writes+1 end;return fn(...)end
    end
    step(r,600);assert(writes==0,'rollback setters keep repeating: '..writes)
    assert(table.concat(s.logs,';'):find('mission: refused staging step start:',1,true))
  end)
  test('unsafe rollback polls native reserve readiness without rewriting restored state',function()
    local s,r,d=scene();local hide=r.g.stage_hide;local refused=false
    r.g.stage_hide=function(v)
      if v and not refused then refused=true;s.p.percent=33;s.reserve_refusal_code=4;error('hide refused')end
      return hide(v)
    end
    r:command('play test')
    for i=1,100 do r:frame();if r.staging and r.staging.restore and r.staging.restore.ops[r.staging.restore.index].name=='rollback.P1' then break end end
    assert(r.staging and s.holding)
    local writes=0
    for _,key in ipairs({'stage_hide','stage_set_camera_bounds','stage_set_blast_bounds','stage_set_origin','camera_params','set_damage','stage_set_spawn','model_set'})do
      local fn=r.g[key];r.g[key]=function(...)if key~='camera_params' or select(1,...)==nil or next(select(1,...)) then writes=writes+1 end;return fn(...)end
    end
    local calls,benches=s.reserve_calls,s.reserve_benches
    step(r,30);assert(writes==0 and r.staging and s.reserve_calls==calls and s.reserve_benches==benches,'unsafe readiness caused writes')
    s.reserve_refusal_code=0;step(r,20)
    assert(not r.staging and not r.recovery and not s.holding and s.p.percent==0)
    assert(table.concat(s.logs,';'):find('rollback.P1 completed after 31 frames',1,true))
  end)
  test('permanent rollback readiness ends once in a named bounded refusal',function()
    local s,r,d=scene();local hide=r.g.stage_hide;local refused=false
    r.g.stage_hide=function(v)if v and not refused then refused=true;s.reserve_refusal_code=4;error('hide refused')end;return hide(v)end
    r:command('play test');step(r,200)
    assert(not r.staging and r.recovery and s.holding)
    local lines=0;for _,line in ipairs(s.logs)do if line:find('mission: refused staging step rollback.P1:',1,true)then
      lines=lines+1;assert(line:find('refusal_code=4 not met after 120 frames',1,true))
    end end
    assert(lines==1);local n=#s.logs;step(r,600);assert(#s.logs==n)
    s.reserve_refusal_code=0;r:command('stop');step(r,30)
    assert(not r.recovery and not r.staging and not s.holding)
  end)

  test('completed staging steps log once with their frame counts',function()
    local s,r=scene();r:command('play test');step(r,80);assert(r.current)
    local names={}
    for _,line in ipairs(s.logs)do
      local name,n=line:match('mission: staging step (.-) completed after (%d+) frames')
      if name then assert(not names[name],'duplicate completed step '..name);names[name]=true;assert(tonumber(n)>=1)end
    end
    for _,name in ipairs({'prepare','validate','root','bounds','place','hide','start'})do assert(names[name],'missing step '..name)end
  end)
  test('delta restoration skips unchanged native values within float tolerance',function()
    local s,r,d=scene();local g=r.g;local restore=d.install_restore;local writes=0
    g.camera_params({min_dist=83,max_depth=1000,fov=25})
    for _,key in ipairs({'stage_hide','stage_set_camera_bounds','stage_set_blast_bounds','stage_set_origin','camera_params','set_damage','stage_set_spawn'})do
      local fn=g[key];g[key]=function(...)if key~='camera_params' or select(1,...)==nil or next(select(1,...)) then writes=writes+1 end;return fn(...)end
    end
    for i=1,600 do
      restore.origin(g,{x=0.005,y=-0.005})
      restore.bounds(g,'blast',{left=-399.995,right=400,top=400,bottom=-400})
      restore.params(g,{min_dist=83.005,max_depth=1000,fov=25})
      restore.spawn(g,4,{x=0.005,y=30});restore.hide(g,{hidden=false},false);restore.damage(g,0)
    end
    assert(writes==0,'unchanged setters: '..writes)
  end)

  test('rollback skips an already applied model write after a setter throws',function()
    local s,r=scene();r:command('play test');step(r,50);local old=r.current;assert(old)
    s.files['missions/test/level.lua']=s.files['missions/test/level.lua']:gsub('version=2','version=2,starting_percent=42')
    local damage=r.g.set_damage;local refused=false
    r.g.set_damage=function(port,n)if not refused then refused=true;error('commit damage refused')end;damage(port,n)end
    local model=r.g.model_set;local writes=0;local threw=false;local handle=old.root.parts[1].handle
    r.g.model_set=function(h,options)
      local result=model(h,options)
      if r.staging and r.staging.phase=='rollback'and h==handle then
        writes=writes+1;if not threw then threw=true;error('setter failed after applying')end
      end
      return result
    end
    r:command('reload');step(r,80)
    assert(not r.staging and not r.recovery and r.current==old and writes==1,'already applied model setter repeated')
  end)

end
