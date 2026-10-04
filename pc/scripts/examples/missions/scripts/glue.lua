-- Engine observations/actions around the unchanged pure mission machine.
return function(D)
  local M,G=D.mission,{}
  function G.markers(doc)
    local m,out=doc.mission,{}
    local function add(name,p,kind,index) out[#out+1]={name=name,x=p.x,y=p.y,kind=kind,index=index} end
    add('start',m.start,'start')
    for i,e in ipairs(m.enemies) do add('enemy'..i,e,'enemy',i) end
    for i,p in ipairs(m.checkpoints) do add('checkpoint'..i,p,'checkpoint',i) end
    for i,p in ipairs(m.triggers) do add('trigger'..i,p,'trigger',i) end
    if m.goal then add('goal',m.goal,'goal') end
    for _,p in ipairs(doc.level.markers) do
      local replaced=false
      for i,old in ipairs(out) do if old.name==p.name then out[i]=p;replaced=true;break end end
      if not replaced then out[#out+1]=p end
    end
    local direction=m.goal and m.goal.x<m.start.x and -1 or 1
    table.sort(out,function(a,b)
      if a.kind=='start' then return b.kind~='start' end
      if b.kind=='start' then return false end
      if a.x~=b.x then return a.x*direction<b.x*direction end
      if a.y~=b.y then return a.y<b.y end
      return a.name<b.name
    end)
    return out
  end
  function G.marker(doc,name)
    for _,p in ipairs(G.markers(doc)) do if p.name==name then return p end end
    error('unknown marker: '..tostring(name),0)
  end
  function G.new(doc,marker)
    local normalized=M.validate(doc.mission)
    local st=M.new(normalized)
    local run={state=st,frames=0,last={},falls=0,message=nil,handles={},damaged={}}
    if marker and marker~='start' then
      local target=G.marker(doc,marker)
      local visited,waves={},{}
      for _,p in ipairs(G.markers(doc)) do
        if p.name==target.name then break end
        visited[p.name]=true
      end
      for i,e in ipairs(doc.mission.enemies) do
        waves[e.wave]=waves[e.wave]~=false and visited['enemy'..i]==true
      end
      if target.authored then
        waves={};for _,w in ipairs(target.cleared_waves) do waves[w]=true end
        run.frames=target.frames
      end
      for w,done in pairs(waves) do if done then st.begun[w]=true;st.nbegun=st.nbegun+1 end end
      for _,e in ipairs(doc.mission.enemies) do if st.begun[e.wave] then st.defeated=st.defeated+1 end end
      for _,p in ipairs(G.markers(doc)) do
        if p.kind=='checkpoint' and (visited[p.name] or p.name==target.name) then st.cp=p.index end
        if p.kind=='trigger' and visited[p.name] then st.fired[p.index]=true end
      end
      if target.authored then st.cp=target.checkpoint end
      st.started=true;st.cleared=st.nbegun==#st.waves
      run.respawn=st.cp and doc.mission.checkpoints[st.cp] or target
    end
    return run
  end
  function G.cleanup(g,run)
    for h in pairs(run.handles) do pcall(g.enemy_remove,h) end
    run.handles={};run.damaged={};run.spawn_queue={};run.collision_queue={}
    run.state.tracked={}
  end
  function G.step(g,current,player)
    local run= current.run
    local st=run.state
    local p=player or (current.membership and current.membership.point) or g.player(1)
    run.observation={player=p,zones=current.membership} -- shared trigger/contact trace reference
    local alive,defeated={},{}
    for h in pairs(st.tracked) do
      local status=g.enemy_status(h)
      if status=='alive' then
        alive[h]=true;local e=g.enemy_state(h);run.last[h]=e
        if e and (e.received or 0)>0 and e.last_attacker==1 then run.damaged[h]=true end
      elseif status=='defeated' then defeated[h]=true
      else
        local at,b=run.last[h],current.doc.level.blast
        local near=at and b and (at.x<=b.left+30 or at.x>=b.right-30 or at.y<=b.bottom+30 or at.y>=b.top-30)
        if not near or run.damaged[h] then defeated[h]=true end
      end
    end
    run.falls=p and p.falls or run.falls
    if D.maze_encounters then D.maze_encounters.observe(current)end
    local actions,events=M.step(st,{frames=run.frames,player=p,falls=run.falls or 0,alive=alive,defeated=defeated})
    if st.result and not run.terminal_logged then
      local text=st.result.status
      if text=='failed'then text='failed reason='..tostring(st.result.reason)..' '..tostring(st.result.detail or '')end
      g.log('mission: '..text);run.terminal_logged=true
    end
    if D.maze_encounters and current.doc.maze then
      for _,a in ipairs(run.spawn_queue or{})do actions[#actions+1]=a end
      run.spawn_queue={};actions=D.maze_encounters.actions(current,actions)
    end
    run.spawn_queue=run.spawn_queue or {}
    local immediate={}
    for _,a in ipairs(actions)do if a.type=='spawn'then run.spawn_queue[#run.spawn_queue+1]=a else immediate[#immediate+1]=a end end
    actions=immediate
    if st.result then run.spawn_queue={}end
    local frame=current.membership and current.membership.frame or run.frames
    local committed=current.membership and current.membership.committed
    local safe_room=not committed or current.stream.loaded[committed.id]
    local live=0;for _ in pairs(st.tracked)do live=live+1 end
    if live<M.LIMITS.per_wave and safe_room and not current.stream.pending and current.stream.load_frame~=frame and #run.spawn_queue>0 then
      actions[#actions+1]=table.remove(run.spawn_queue,1);run.execute_spawn=actions[#actions]
    else run.execute_spawn=nil end
    for _,a in ipairs(actions) do
      if a.type=='spawn' and a==run.execute_spawn then
        current.stream.wave_frame=frame
        local ok,h,why=pcall(g.spawn_enemy,a.kind,a.x,a.y,{facing=a.facing})
        if ok and h then run.handles[h]=true;M.spawned(st,a.index,h)
        else M.spawn_failed(st,a.index,ok and why or h) end
      elseif a.type=='respawn_point' then
        if current.stream.current then D.chunks.respawn(current.stream)
        else assert(g.stage_set_spawn(M.RESPAWN_SLOT,a.x,a.y)) end
      elseif a.type=='collision' then
        local areas={current.root}
        for _,area in pairs(current.stream.loaded) do areas[#areas+1]=area end
        run.collision_queue=run.collision_queue or{}
        run.collision_queue[#run.collision_queue+1]={areas=areas,action=a}
      elseif a.type=='cleanup' then G.cleanup(g,run) end
    end
    if not st.result and not current.stream.pending and current.stream.load_frame~=frame and current.stream.wave_frame~=frame then
      local job=run.collision_queue and run.collision_queue[1]
      if job then
        local done,changed=D.world.collision_tick(g,current,job)
        if changed then current.stream.load_frame=frame end
        if done then table.remove(run.collision_queue,1)end
      end
    end
    for _,e in ipairs(events) do
      local text=e.type
      if e.type=='wave' then text=('wave %d of %d'):format(e.n,e.of)
      elseif e.type=='checkpoint' then text='checkpoint '..e.index
      elseif e.type=='message' then run.message={text=e.text,left=M.MESSAGE_FRAMES};text='message '..e.text
      elseif e.type=='defeat' then text='defeated '..e.kind
      elseif e.type=='vanish' then text='lost '..e.kind..' (not a defeat)'
      elseif e.type=='failed' then text='failed reason='..tostring(e.reason)..' '..tostring(e.detail or '') end
      if e.type~='complete' and e.type~='failed' then g.log('mission: '..text)end
    end
    if run.message then run.message.left=run.message.left-1;if run.message.left<=0 then run.message=nil end end
  end
  return G
end
