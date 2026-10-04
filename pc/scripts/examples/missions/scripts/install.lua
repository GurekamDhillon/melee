-- Resumable install: each engine on_frame callback performs one bounded phase.
return function(D)
  -- Native writes are synchronous; reserve readiness is queried without setter retries.
  local function restoration(D)
  local S={EPS=0.01}
  local function near(a,b)return a~=nil and b~=nil and math.abs(a-b)<=S.EPS end
  local function same(a,b)
    if not a or not b then return false end
    for k,v in pairs(b)do
      if type(v)=='number' then if not near(a[k],v)then return false end
      elseif type(v)=='table' then if not same(a[k],v)then return false end
      elseif a[k]~=v then return false end
    end
    return true
  end
  S.same=same
  function S.origin(g,p)
    if not same(g.stage_bounds().origin or {x=0,y=0},p)then assert(g.stage_set_origin(p.x,p.y,{frames=0}))end
  end
  function S.bounds(g,kind,b)
    local live=g.stage_bounds()
    if not same(live[kind],b)then
      local p=live.origin or {x=0,y=0}
      assert(g['stage_set_'..kind..'_bounds'](b.left-p.x,b.right-p.x,b.top-p.y,b.bottom-p.y))
    end
  end
  function S.params(g,p)
    if p and not same(g.camera_params({}),p)then g.camera_params(p)end
  end
  function S.spawn(g,slot,p)
    local x,y=g.stage_spawn(slot)
    if not near(x,p.x)or not near(y,p.y)then assert(g.stage_set_spawn(slot,p.x,p.y))end
  end
  function S.hide(g,t,value)
    local live=t.hidden
    if g.stage_isolate then live=g.stage_isolate()end
    if live~=value then local ok,why=g.stage_hide(value);assert(ok,why);t.hidden=value end
  end
  function S.damage(g,n)
    if n==nil or not g.set_damage then return end
    n=math.max(0,math.min(999,math.floor(n)))
    local p=g.player(1)
    if not p or not near(p.percent,n)then g.set_damage(1,n)end -- returns ZERO Lua values
    p=g.player(1);assert(p and near(p.percent,n),'P1 percent readback differs from synchronous set_damage')
  end
  function S.ready(g,port)
    local _,entities=g.fighter_benched(port)
    for _,p in ipairs(entities or {})do
      if p.present and p.refusal_code and p.refusal_code~=0 then
        return false,'reserve '..port..' entity '..(p.entity_index or 0)..' refusal_code='..p.refusal_code
      end
    end
    return true
  end
  function S.new(r,t)
    local g=r.g;local ops={}
    local function add(name,fn)ops[#ops+1]={name='rollback.'..name,run=fn}end
    if t.changed then
      add('origin',function()S.origin(g,t.bounds.origin or {x=0,y=0})end)
      for _,kind in ipairs({'camera','blast'})do local b=t.bounds[kind];if b then add(kind,function()S.bounds(g,kind,b)end)end end
      add('params',function()S.params(g,t.params)end)
      for slot,p in pairs(t.spawn)do add('spawn'..slot,function()S.spawn(g,slot,p)end)end
      add('hide',function()S.hide(g,t,t.old~=nil)end)
    end
    add('P1',function()
      if t.held then
        local ready,why=S.ready(g,1);if not ready then return false,why end
        local ok,reason=g.fighter_call(1,t.player.x,t.player.y,{facing=t.player.facing or 1,intangible_frames=60})
        if not ok then return false,reason end;t.held=nil
      elseif t.placed then
        local p=g.player(1)
        if not p or not near(p.x,t.player.x)or not near(p.y,t.player.y)then g.teleport(1,t.player.x,t.player.y)end
      end
    end)
    add('damage',function()S.damage(g,t.player.percent)end)
    add('pose',function()
      if t.camera_held and t.pose and not same(g.camera_get(),{eye=t.pose.eye,interest=t.pose.interest,fov=t.pose.fov,roll=t.pose.roll})then D.camera.cut(g,t.pose)end
    end)
    for port,record in pairs(t.c.fighters or {})do
      add('CPU'..port,function()
        local previous=t.old and t.old.fighters or r.guests.fighters
        if t.old then
          local ready,why=S.ready(g,port);if not ready then return false,why end
          if previous[port]and previous[port].owned then
            if not g.fighter_benched(port)then local ok,reason=g.fighter_bench(port);if not ok then return false,reason end end
          else
            local p=t.cpus[port]
            if p then
              if not g.fighter_benched(port)then local ok,reason=g.fighter_bench(port);if not ok then return false,reason end end
              local ok,reason=g.fighter_call(port,p.x,p.y,{intangible_frames=60});if not ok then return false,reason end
              if previous[port]and previous[port].mode and g.cpu_mode then g.cpu_mode(port,previous[port].mode)end
            end
          end
        elseif not previous[port]then D.fighters.stop(g,{fighters={[port]=record}})end
      end)
    end
    return {ops=ops,index=1}
  end
  function S.tick(t)
    local op=t.ops[t.index];if not op then return true end
    local accepted,why=op.run()
    if accepted==false then return false,why end
    t.index=t.index+1;return true
  end
  return S
end

  local T={WAIT_LIMIT=120};local S=restoration(D);D.install_restore=S
  local function areas(c)
    local out={};if c and c.root then out[#out+1]=c.root end
    for _,a in pairs(c and c.stream.loaded or {}) do out[#out+1]=a end
    return out
  end
  local function copy(records)
    local out={}
    for port,r in pairs(records or {}) do local q={};for k,v in pairs(r) do q[k]=v end;out[port]=q end
    return out
  end
  function T.begin(r,name,marker,reload)
    assert(not r.staging,'mission installation already in progress')
    local g=r.g;local match=g.match()
    assert(match and match.active and not match.netplay,'active offline match required')
    local p=assert(g.player(1),'P1 is required')
    assert(D.fighters.stage_ready(g),'respawning or entry/landing: waiting for reserve-ready P1')
    local cpus={};for port=2,6 do local q=g.player(port);if q and q.cpu then cpus[port]=q end end
    if r.current then D.commands.cancel(r)end
    r.generation=r.generation+1;r.last_install_error=nil
    r.staging={name=name,marker=marker,reload=reload,old=r.current,phase='prepare',frames=0,
      player=p,cpus=cpus,bounds=g.stage_bounds(),params=g.camera_params and g.camera_params({}),
      pose=g.camera_get and g.camera_get(),prefix='mission'..r.generation..'_',assets={},asset_paths={},
      built={},spawn={},c={fighters=copy(r.current and r.current.fighters or r.guests.fighters)}}
    local t=r.staging;t.hidden=g.stage_isolate and g.stage_isolate() or t.old~=nil
    if not r.host_bounds then
      r.host_bounds=t.bounds;r.host_spawns={}
      for _,range in ipairs({{0,7},{127,146}})do for slot=range[1],range[2]do
        local x,y=g.stage_spawn(slot);r.host_spawns[slot]={x=x,y=y}
      end end
    end
    g.log('mission: staging '..name)
  end
  local function prepare(r,t)
    local g=r.g
    if not t.old then r.guests.fighters=t.c.fighters end -- retain match safety reserve on refusal
    t.c.doc={level={fighters={}}}
    local safe,reason=pcall(D.fighters.prepare,g,t.c)
    if not safe then
      if D.fighters.transient(reason)then t.wait_condition=tostring(reason);return end
      error(reason,0)
    end
    if not D.fighters.stage_ready(g)then t.wait_condition='waiting for reserve-ready P1';return end
    assert(g.fighter_bench and g.fighter_call,'mission staging requires fighter reserve APIs')
    assert(not (g.fighter_benched and g.fighter_benched(1)),'P1 already reserved')
    local ok,why=g.fighter_bench(1);assert(ok,'P1 bench refused: '..tostring(why));t.held=true
    if t.pose then D.camera.cut(g,t.pose);t.camera_held=true end
    t.phase='validate'
  end
  local function validate(r,t)
    local g=r.g;local doc=D.loader.folder(g,t.name);t.doc=doc
    if t.reload and t.old and t.old.run.state.cp and doc.mission.checkpoints[t.old.run.state.cp] then
      t.marker='checkpoint'..t.old.run.state.cp
    end
    t.target=t.reload and {x=t.player.x,y=t.player.y} or (t.marker and D.glue.marker(doc,t.marker) or doc.mission.start)
    local initial=D.zones.initial(doc,t.target,t.reload and t.old and t.old.stream.current)
    t.queue,t.centre=D.chunks.plan(doc,t.target,initial,true)
    local c=t.c;c.doc=doc;c.from=t.marker;c.run=D.glue.new(doc,t.marker)
    c.stream=D.chunks.new(g,doc,t.prefix);c.stream.current=t.centre;c.run.falls=t.player.falls or 0
    c.maze_rewards=t.reload and t.old and t.old.maze_rewards or {}
    local slots={[D.mission.RESPAWN_SLOT]=true}
    for slot in pairs(doc.level.spawn) do slots[slot]=true end
    if t.old then for slot in pairs(t.old.doc.level.spawn) do slots[slot]=true end end
    for slot in pairs(slots) do local x,y=g.stage_spawn(slot);if x then t.spawn[slot]={x=x,y=y} end end
    local lo,hi=math.huge,-math.huge;local seen={}
    local function scan(level,collect)
      for _,p in ipairs(level.parts) do
        lo=math.min(lo,p.y);hi=math.max(hi,p.y)
        if collect and not seen[p.path] then seen[p.path]=true;t.asset_paths[#t.asset_paths+1]=p.path end
      end
      for _,l in ipairs(level.lines) do lo=math.min(lo,l.y1,l.y2);hi=math.max(hi,l.y1,l.y2) end
    end
    scan(doc.level,true);for _,chunk in ipairs(doc.chunks) do scan(chunk.level,true) end
    for _,a in ipairs(areas(t.old)) do scan(a.level,false) end
    if lo==math.huge then lo,hi=0,0 end
    local b=t.bounds.blast
    local above=b.top+1024-lo;local below=b.bottom-1024-hi
    if hi+above<48000 then t.offset=above
    elseif lo+below>-48000 then t.offset=below
    else error('no safe offscreen staging space inside native coordinates') end
    t.index=1;t.phase='assets'
  end
  local function step_name(t)
    if t.phase=='assets' then return 'asset:'..(t.asset_paths[t.index]or 'complete')end
    if t.phase=='chunks' then return 'chunk:'..(t.queue[t.index]and t.queue[t.index].id or 'complete')end
    if t.phase=='rollback' then
      if t.glue_cleanup then return 'rollback.enemies'end
      if t.restore_areas[t.index]then return 'rollback.geometry:'..t.restore_areas[t.index].name end
      if #t.built>0 then return 'rollback.unload:'..t.built[#t.built].name end
      local path=next(t.assets);if path then return 'rollback.asset:'..path end
      if not t.restore then return 'rollback.prepare'end
      return t.restore.ops[t.restore.index]and t.restore.ops[t.restore.index].name or 'rollback.finish'
    end
    return t.phase
  end
  function T.fail(r,why,name,frames)
    local t=r.staging;if not t or t.phase=='rollback'then return end
    if D.mission_warm then D.mission_warm.cleanup(r.g,t)end
    t.failure={step=name or step_name(t),condition=tostring(why),frames=frames or t.step_age or 1}
    t.phase='rollback';t.index=1;t.step_key=nil
    t.restore_areas=t.changed and areas(t.old)or {};t.glue_cleanup=t.c.run~=nil
  end
  local function refusal(r,t,failure)
    local why=('staging step %s: %s not met after %d frames'):format(failure.step,failure.condition,failure.frames)
    r.last_install_error=why;r.g.log('mission: refused '..why)
  end
  local function reset(g,r)
    local live=g.stage_bounds();local changed=not S.same(live,r.host_bounds)
    for slot,p in pairs(r.host_spawns)do local x,y=g.stage_spawn(slot)
      if not S.same({x=x,y=y},p)then changed=true end
    end
    if changed then assert(g.stage_restore_bounds())end
  end
  local function finish(r,t)
    local g=r.g;local c=t.c
    -- All disk reads/builders finished. One callback commits cached native state.
    t.changed=true
    for _,a in ipairs(areas(t.old)) do D.world.position(g,a,t.offset) end
    for _,a in ipairs(t.built) do D.world.position(g,a,0) end
    reset(g,r);D.world.bounds(g,t.commit_bounds)
    -- Hold P1 until host isolation, collision validation and setup have completed.
    S.hide(g,t,t.commit_hide)
    if c.run.respawn then assert(g.stage_set_spawn(D.mission.RESPAWN_SLOT,c.run.respawn.x,c.run.respawn.y)) end
    D.glue.step(g,c,{x=t.target.x,y=t.target.y,falls=(g.player(1) or t.player).falls or 0})
    assert(not c.run.state.fail,c.run.state.fail and c.run.state.fail.detail)
    D.chunks.respawn(c.stream)
    local percent=t.commit_player.percent
    S.damage(g,percent)
    if t.camera_held then D.camera.release(g,true) end
    D.zones.sample(g,c,{x=t.target.x,y=t.target.y},c.run.frames)
    D.camera.tick(g,c,{x=t.target.x,y=t.target.y,falls=t.player.falls or 0,action=14})
    D.fighters.tick(g,c)
    pcall(g.fly_attack,1,false);if not t.reload then g.fly(1,false) end
    if t.doc.mission.objective.lives then pcall(g.set_stocks,1,math.min(99,t.doc.mission.objective.lives+1)) end
    assert(g.floor_below,'mission staging requires floor_below')
    local floor=g.floor_below(t.target.x,t.target.y+0.5,200)
    assert(floor and floor<=t.target.y+0.5,'no live floor below mission placement')
    g.log(('mission: placement floor verified x=%.2f y=%.2f floor=%.2f'):format(t.target.x,t.target.y,floor))
    assert(g.fighter_call(1,t.target.x,t.target.y,{facing=t.player.facing or 1,intangible_frames=60}))
    t.held=nil;t.placed=true
    -- The mission starts at handover, not at the pre-staging player snapshot.
    local live=assert(g.player(1),'P1 missing after handover')
    c.run.falls=live.falls or 0;c.run.state.falls=c.run.falls
    if t.old then D.glue.cleanup(g,t.old.run) end
    c.asset_pool=t.assets;r.current=c;r.poll=0;r.staging=nil
    local retired={areas=areas(t.old),assets=t.old and t.old.asset_pool or {},index=1}
    if r.retiring then r.retire_queue=r.retire_queue or {};r.retire_queue[#r.retire_queue+1]=retired
    else r.retiring=retired end
    r.guests.fighters=c.fighters
    t.loaded_message=('mission: %s %s%s'):format(t.reload and 'reloaded' or 'loaded',t.name,t.marker and ' from '..t.marker or '')
  end
  local function run(r,t)
    local g=r.g
    if t.phase=='rollback' then
      if t.glue_cleanup then D.glue.cleanup(g,t.c.run);t.glue_cleanup=nil;return end
      local a=t.restore_areas[t.index]
      if a then D.world.position(g,a,0);t.index=t.index+1;return end
      a=t.built[#t.built]
      if a then D.world.unload(g,a);table.remove(t.built);return end
      local path,asset=next(t.assets)
      if path then g.model_release(asset);t.assets[path]=nil;return end
      if not t.restore then t.restore=S.new(r,t);return end
      if t.restore.ops[t.restore.index]then
        local accepted,why=S.tick(t.restore);if not accepted then t.wait_condition=why end;return
      end
      r.staging=nil;t.terminal_failure=t.failure
      if t.stop_reason then r:stop(t.stop_reason)end
      return
    end
      if t.phase=='prepare' then prepare(r,t)
      elseif t.phase=='validate' then validate(r,t)
      elseif t.phase=='assets' then
        local path=t.asset_paths[t.index]
        if path then t.assets[path]=assert(g.model_load(path));t.index=t.index+1
        else t.phase='root' end
      elseif t.phase=='root' then
        t.c.root=D.world.load(g,t.prefix..'root',t.doc.level,{assets=t.assets,offset=t.offset})
        t.built[#t.built+1]=t.c.root;t.index=1;t.phase='chunks'
      elseif t.phase=='chunks' then
        local chunk=t.queue[t.index]
        if chunk then
          local started=g.time()
          local area=D.world.load(g,t.prefix..chunk.serial,chunk.level,{assets=t.assets,offset=t.offset})
          t.built[#t.built+1]=area;t.c.stream.loaded[chunk.id]=area;t.index=t.index+1
          g.log(('mission: staged chunk %s seconds=%.6f'):format(chunk.id,g.time()-started))
        else t.phase=D.mission_warm and 'warm' or 'bounds' end
      elseif t.phase=='warm' then
        if D.mission_warm.tick(g,t)then t.phase='bounds'end
      elseif t.phase=='bounds' then
        -- Prepare plans now; applying them across frames would move the live camera.
        t.commit_bounds=t.doc.level;t.phase='place'
      elseif t.phase=='place' then
        t.commit_player={x=t.target.x,y=t.target.y,facing=t.player.facing or 1,
          percent=t.doc.level.starting_percent or t.player.percent};t.phase='hide'
      elseif t.phase=='hide' then
        assert(type(g.stage_hide)=='function','mission requires stage_hide');t.commit_hide=true;t.phase='start'
      elseif t.phase=='start' then finish(r,t)
      else error('unknown staging phase '..tostring(t.phase))end
  end
  function T.tick(r)
    local t=r.staging;if not t then return end
    t.frames=t.frames+1;local name=step_name(t)
    if t.step_key~=name then t.step_key=name;t.step_age=0;t.wait_condition=nil end
    t.step_age=t.step_age+1
    local ok,why=pcall(run,r,t)
    if not ok then
      if t.phase=='rollback' then t.wait_condition=tostring(why)
      else T.fail(r,why,name,t.step_age);return end
    end
    if r.staging~=t or step_name(t)~=name then
      r.g.log(('mission: staging step %s completed after %d frames (total %d)'):format(name,t.step_age,t.frames))
      t.step_key=nil
      if t.loaded_message then r.g.log(t.loaded_message)end
      if t.terminal_failure then refusal(r,t,t.terminal_failure)end
    elseif t.step_age>=(t.phase=='warm'and 3600 or t.phase=='prepare'and 600 or T.WAIT_LIMIT) then
      local failed={step=name,condition=t.wait_condition or 'step made no progress',frames=t.step_age}
      if t.phase=='rollback' then r.recovery=t;r.staging=nil;refusal(r,t,failed)
      else T.fail(r,failed.condition,name,failed.frames)end
    end
  end
  function T.retire(r)
    local t=r.retiring;if not t then return end
    local a=t.areas[t.index]
    if a then D.world.unload(r.g,a);t.index=t.index+1;return end
    local path,asset=next(t.assets)
    if path then r.g.model_release(asset);t.assets[path]=nil;return end
    r.retiring=table.remove(r.retire_queue or {},1)
  end
  function T.abandon(r)
    local t=r.staging;if not t then return end
    if D.mission_warm then D.mission_warm.cleanup(r.g,t)end
    if t.c.run then D.glue.cleanup(r.g,t.c.run) end
    for _,a in ipairs(t.built) do D.world.unload(r.g,a) end
    for _,asset in pairs(t.assets) do pcall(r.g.model_release,asset) end
    if t.held then pcall(r.g.fighter_call,1,t.player.x,t.player.y,{intangible_frames=60}) end
    r.staging=nil
  end
  function T.dispose_retired(r)
    while r.retiring do
      for i=r.retiring.index,#r.retiring.areas do D.world.unload(r.g,r.retiring.areas[i]) end
      for _,asset in pairs(r.retiring.assets) do pcall(r.g.model_release,asset) end
      r.retiring=table.remove(r.retire_queue or {},1)
    end
  end
  return T
end
