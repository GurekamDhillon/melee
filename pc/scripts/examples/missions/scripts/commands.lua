-- Fly operations are fixtures, never traversal verdicts.
return function(D)
  local C={}
  C.TARGET_LIMIT=1500
  function C.cancel(r)
    local c,g=r.current,r.g
    local clear=c and c.clear
    if clear and clear.deadline and g.deadline_done then pcall(g.deadline_done,clear.deadline) end
    if c then c.clear=nil;c.pending_visit=nil;c.visit_frames=nil end
    if g.fly_clear then pcall(g.fly_clear,1) else pcall(g.fly_attack,1,false);pcall(g.fly,1,false) end
  end
  function C.target(g,p,control,frame)
    if control then
      local previous=control.target
      if g.fly(1) and previous and math.abs(previous.x-p.x)<1 and math.abs(previous.y-p.y)<1 then return true end
      if control.last_frame and frame-control.last_frame<6 then return false end
      control.last_frame=frame
    end
    if math.abs(p.x)>10000 or math.abs(p.y)>10000 then
      assert(math.abs(p.x)<50000 and math.abs(p.y)<50000,'target exceeds native coordinate limit')
      g.fly(1,true);g.teleport(1,p.x,p.y)
      g.log('mission: long range teleport fixture')
    else g.fly_target(1,p.x,p.y) end
    if control then control.target={x=p.x,y=p.y} end
    return true
  end
  function C.visit(r,p)
    local c,g=r.current,r.g
    local ok,why=D.chunks.preload(c.stream,p,g.player(1))
    assert(ok,why)
    local destination=D.chunks.containing(c.doc.chunks,p)
    if destination and (not c.stream.loaded[destination.id] or c.stream.pending)then c.pending_visit=p;return end
    c.pending_visit=p
    C.target(g,p)
    c.pending_visit=nil;c.visit_frames=nil
    g.log(('mission: fly %s %.1f %.1f'):format(p.name,p.x,p.y))
  end
  function C.fly(r,name)
    C.cancel(r);r.current.tour=nil
    local markers=D.glue.markers(r.current.doc)
    local index=r.current.cursor or 0
    if name=='next' then index=index%#markers+1
    elseif name=='prev' then index=(index-2)%#markers+1
    else
      index=nil
      for i,p in ipairs(markers) do if p.name==name then index=i end end
      assert(index,'unknown marker: '..tostring(name))
    end
    C.visit(r,markers[index]);r.current.cursor=index
  end
  function C.tour(r)
    local queue=D.glue.markers(r.current.doc)
    for _,c in ipairs(r.current.doc.chunks) do
      queue[#queue+1]={name='chunk:'..c.id,x=c.spawn.x,y=c.spawn.y}
    end
    r.current.tour={queue=queue,index=1,started=r.g.time(),sent=false,frames=0}
  end
  function C.tick(r)
    local c,g=r.current,r.g
    if c.pending_visit then
      c.visit_frames=(c.visit_frames or 0)+1
      local ok,why=true,nil
      if not c.stream.pending then ok,why=pcall(C.visit,r,c.pending_visit)end
      if c.visit_frames and c.visit_frames>=600 then ok,why=false,'queued fly timed out'end
      if not ok and not D.fighters.transient(why)then
        c.pending_visit=nil;c.visit_frames=nil
        if c.tour then c.tour.refusal=tostring(why)end
        g.log('mission: queued fly refused '..tostring(why))
      end
    end
    if c.clear then
      local clear=c.clear
      clear.elapsed=clear.elapsed+1
      local function finish(text)C.cancel(r);g.log('mission: '..text)end
      if clear.elapsed>=clear.limit then finish('clear whole-command timeout');return end
      while clear.handles[clear.index] and g.enemy_status(clear.handles[clear.index])~='alive' do
        clear.index=clear.index+1;clear.target_h=nil
      end
      local h=clear.handles[clear.index]
      if not h then finish('clear complete');return end
      if clear.target_h~=h then
        clear.target_h=h;clear.age=0;clear.pulse=0;clear.target=nil;clear.last_frame=nil
      end
      local e=g.enemy_state(h)
      if clear.age>=C.TARGET_LIMIT then
        finish(('clear: %s not defeated after %d frames (state %s, vulnerable=%s)'):format(
          e and e.kind or 'unknown',clear.age,tostring(e and e.state),tostring(e and e.vulnerable)))
        return
      end
      local player=c.membership and c.membership.point or g.player(1)
      local attacking=clear.pulse%40<28 -- leave 12 frames for hitlag/death logic
      local flight=g.fly_state and g.fly_state(1)
      local changed=clear.falls~=(player or {}).falls or not g.fly(1)
        or (flight and flight.attacking~=attacking) or clear.attacking~=attacking
      if not D.fighters.ready(player) then
        clear.wait=(clear.wait or 0)+1
      else
        local ok,why=true,true
        if changed then ok,why=pcall(g.fly_attack,1,attacking,30,30) end
        if ok and (not attacking or why) then
          clear.attacking=attacking;clear.falls=player.falls
          if e then
            -- Escalate by moving across the target rather than pinning one contact point.
            local target={x=e.x,y=e.y}
            if clear.age>=600 then target.x=target.x+(clear.age%80<40 and -8 or 8)end
            ok,why=pcall(C.target,g,target,clear,c.run.frames)
          end
        end
        if not ok then
          if not D.fighters.transient(why)then finish('clear refused '..tostring(why));return end
          clear.refusal=tostring(why);clear.wait=(clear.wait or 0)+1
        elseif why==false and (clear.refusal or (attacking and changed)) then clear.wait=(clear.wait or 0)+1
        else clear.refusal=nil;clear.wait=0;clear.pulse=clear.pulse+1 end
      end
      clear.age=clear.age+1
      if (clear.wait or 0)>=600 then finish('clear timed out waiting for controllable P1');return end
    end
    local t=c.tour
    if not t then return end
    local target=t.queue[t.index]
    if not target then c.tour=nil;C.cancel(r);g.log('mission: tour complete');return end
    t.frames=t.frames+1
    if not t.sent then
      t.started=g.time()
      local ok,why=pcall(C.visit,r,target)
      t.sent=ok
      if not ok and not D.fighters.transient(why) then t.refusal=tostring(why) end
    end
    local p=c.membership and c.membership.point or g.player(1)
    if t.refusal or (p and (p.x-target.x)^2+(p.y-target.y)^2<=1) or t.frames>=600 then
      local loaded={}
      for id in pairs(c.stream.loaded) do loaded[#loaded+1]=id end;table.sort(loaded)
      g.log(('mission: tour %s loaded=%s seconds=%.6f refusal=%s'):format(target.name,table.concat(loaded,','),
        g.time()-t.started,t.refusal or (t.frames>=600 and 'arrival timeout' or 'none')))
      t.index=t.index+1;t.sent=false;t.frames=0;t.refusal=nil
    end
  end
  function C.dispatch(r,arg)
    local words={};for w in arg:gmatch('%S+') do words[#words+1]=w end
    local op=words[1]
    if op=='world'then D.world_commands.dispatch(r,words);return end
    if op=='maze' then D.maze_commands.dispatch(r,words);return end
    if op=='list' then
      assert(#words==1,'usage: mission list')
      local entries,why=r.g.mod_list('missions/');assert(entries,why)
      table.sort(entries,function(a,b)return a.name<b.name end)
      for _,e in ipairs(entries) do if e.dir then r.g.log('mission: folder '..e.name) end end
      return
    elseif op=='stop' then assert(#words==1);r:stop('stopped');return
    elseif op=='play' then
      assert(#words==2 or (#words==4 and words[3]=='from'),'usage: mission play <name> [from <marker>]')
      r:install(words[2],words[4],false);return
    end
    assert(r.current,'no running mission; mission play <name>')
    if op=='reload' or op=='restart' then
      assert(#words==1);r:install(r.current.doc.name,op=='restart' and r.current.from or nil,op=='reload')
    elseif op=='zones' then assert(#words==1,'usage: mission zones');D.zones.describe(r.g,r.current)
    elseif op=='fly' then assert(#words==2,'usage: mission fly next|prev|<marker>');C.fly(r,words[2])
    elseif op=='drop' then
      assert(#words==1);C.cancel(r);r.current.tour=nil;r.current.stream.destination=nil
      pcall(r.g.fly_attack,1,false);r.g.fly(1,'place')
      r.g.log('mission: drop normal control')
    elseif op=='clear' then
      assert(#words==1);C.cancel(r);r.current.tour=nil
      local handles={};for h in pairs(r.current.run.state.tracked) do handles[#handles+1]=h end;table.sort(handles)
      local clear={handles=handles,index=1,elapsed=0,limit=601+(C.TARGET_LIMIT+1)*#handles}
      r.current.clear=clear
      if r.g.deadline and r.g.deadline_done then
        clear.deadline='mission.clear.'..r.generation
        local ok,why=pcall(r.g.deadline,clear.deadline,clear.limit+1)
        if not ok or not why then C.cancel(r);error('clear deadline refused: '..tostring(why))end
      end
      C.tick(r);r.g.log('mission: clear begin (target limit 1500 frames; pulse 28/12)')
    elseif op=='tour' then assert(#words==1);C.cancel(r);C.tour(r)
    else error('usage: mission play|stop|restart|reload|list|fly|clear|drop|tour|maze') end
  end
  return C
end
