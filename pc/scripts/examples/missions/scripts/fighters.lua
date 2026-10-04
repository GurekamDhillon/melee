-- CPU ports are level guests; humans retain their controller and placement.
return function(D)
  local F={}
  function F.ready(p)
    if not p then return false end
    local a=p.action
    -- Public common motion IDs; mirror Geno fly's dead/held/throw ranges.
    return a==nil or (a>13 and a~=42 and a~=43 and not (a>=223 and a<=232) and not (a>=239 and a<=243) and not (a>=266 and a<=339))
  end
  function F.stage_ready(g)
    if not F.ready(g.player(1))then return false end
    if g.fighter_benched then
      local held,entities=g.fighter_benched(1)
      if held then return false end
      for _,e in ipairs(entities or {})do
        if e.present and (e.refusal_code or 0)~=0 then return false end
      end
    end
    return true
  end
  function F.transient(why)
    local s=tostring(why)
    return s:find('state cannot fly',1,true) or s:find('respawning',1,true) or s:find('dead fighter',1,true)
  end
  function F.tick(g,c)
    c.fighters=c.fighters or {}
    local at=c.stream.current and c.stream.current.spawn or c.run.respawn or c.doc.mission.start
    for port=2,6 do
      local p=g.player(port);local old=c.fighters[port]
      if p and p.cpu then
        local mode=c.doc.level.fighters[port] or 'stand'
        if not old or old.kind~=p.kind or old.char~=p.char then
          old={kind=p.kind,char=p.char,original={x=p.x,y=p.y}};c.fighters[port]=old
        end
        if old.mode~=mode or old.falls~=p.falls then
          old.mode=mode;old.falls=p.falls;old.ticks=0;old.configured=nil;old.at=nil
        end
        old.ticks=(old.ticks or 0)+1
        local benched=g.fighter_benched and g.fighter_benched(port)
        if old.owned and not benched then old.owned=nil;old.configured=nil end
        if mode=='fight' then
          if old.owned then
            local ok,called=pcall(g.fighter_call,port,at.x,at.y,{intangible_frames=0})
            if ok and called then old.owned=nil;old.configured=nil;benched=false end
          end
          if not old.owned and not benched and not old.configured and g.cpu_mode then
            local ok,accepted=pcall(g.cpu_mode,port,'fight')
            if ok and accepted then old.configured=true end
          end
        elseif not old.owned and not benched then
          if g.fighter_bench and (old.ticks-1)%30==0 then
            local ok,accepted,why=pcall(g.fighter_bench,port)
            if ok and accepted then old.owned=true;old.refusal=nil;g.log('mission: CPU '..port..' benched') end
          end
          if not old.owned and F.ready(p) and (old.ticks-1)%6==0 then
            local ok=pcall(function()
              if not old.configured and g.cpu_mode then
                assert(g.cpu_mode(port,'stand'),'CPU stand refused');old.configured=true;old.fallback=true
              end
              if old.at~=at then g.teleport(port,at.x,at.y);old.at=at end
            end)
            if not ok then old.at=nil end
          end
        end
      else c.fighters[port]=nil end
    end
  end
  function F.guard(g,state)
    state.ticks=(state.ticks or 0)+1;state.fighters=state.fighters or {};local safe=true
    for port=2,6 do
      local p=g.player(port)
      if p and p.cpu and not (g.fighter_benched and g.fighter_benched(port)) then
        local old=state.fighters[port] or {kind=p.kind,char=p.char,original={x=p.x,y=p.y}}
        state.fighters[port]=old
        if g.cpu_mode and not old.stood then
          local ok,accepted=pcall(g.cpu_mode,port,'stand');old.stood=ok and accepted;old.fallback=old.stood
        end
        if g.fighter_bench and (state.ticks-1)%30==0 then
          local ok,accepted=pcall(g.fighter_bench,port)
          if ok and accepted then old.owned=true;g.log('mission: CPU '..port..' benched') end
        end
        if not old.owned then safe=false end
      end
    end
    return safe
  end
  function F.prepare(g,c)
    -- Reserve every CPU before loading collision; unsafe reserves defer the whole load.
    c.fighters=c.fighters or {}
    for port=2,6 do
      local p=g.player(port)
      if p and p.cpu and not (g.fighter_benched and g.fighter_benched(port)) then
        assert(g.fighter_bench,'mission load requires fighter_bench for CPU safety')
        local ok,accepted=pcall(g.fighter_bench,port)
        if not ok or not accepted then error('respawning or unsafe CPU '..port..': waiting for bench before mission load') end
        local old=c.fighters[port] or {kind=p.kind,char=p.char,original={x=p.x,y=p.y}}
        old.owned=true;c.fighters[port]=old;g.log('mission: CPU '..port..' benched')
      end
    end
  end
  function F.stop(g,c)
    for port,old in pairs(c and c.fighters or {}) do
      if old.owned and g.fighter_call then
        local at=old.original
        local ok,called=pcall(g.fighter_call,port,at.x,at.y,{intangible_frames=0})
        if not ok or not called then g.log('mission: CPU '..port..' reserve release refused') end
      end
      if old.fallback and g.cpu_mode then pcall(g.cpu_mode,port,'fight') end
    end
  end
  return F
end
