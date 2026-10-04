return function(test,fixture)
  test('second scene waits through entry and landing before reserving and staging',function()
    local s,r,d=fixture();s.p.action=323;s.manual_stage=true
    assert(r:command('play test'));for _=1,12 do r:frame()end
    assert(r.pending and not r.staging and not s.holding and s.loads==0)
    s.p.action=42;for _=1,6 do r:frame()end
    assert(r.pending and not r.staging,'landing is not safe reserve locomotion')
    s.p.action=14;for _=1,50 do r:frame()end
    assert(r.current and not r.staging and not s.holding)
  end)
  test('P1 remains benched until isolated collision passes floor probe',function()
    local s,r=fixture();s.manual_stage=true;s.p.x=90;local probes=0
    r.g.floor_below=function(x,y,depth)
      if r.staging and r.staging.phase=='start' then
        assert(s.holding and s.hidden and not s.reserve_calls,'release before isolated floor check')
        probes=probes+1;return y-0.25
      end
      return y-0.25
    end
    assert(r:command('play test'));for _=1,50 do r:frame()end
    assert(r.current and probes==1 and s.reserve_calls==1)
  end)
  test('missing mission floor refuses commit and restores held P1 safely',function()
    local s,r=fixture();r.g.floor_below=function()return nil end
    assert(not r:command('play test'));assert(not r.current and not s.holding and not s.hidden)
    assert(table.concat(s.logs,';'):find('no live floor below mission placement',1,true))
  end)
  test('settlement makes old grounded fighter airborne before removing floor 17',function()
    local s,r=fixture();assert(r:command('play test'));s.grounded=true
    local fly=r.g.fly;r.g.fly=function(port,v)
      if v==false and s.flying then s.grounded=false end
      return fly(port,v)
    end
    local unload=r.g.area_unload;r.g.area_unload=function(n)
      assert(not s.grounded or s.holding,'grounded P1 over removed mission floor 17')
      return unload(n)
    end
    r:stop('envoy win');assert(not r.current and not s.hidden and not s.grounded)
    -- The old actor can process input before the queued Falco scene actually replaces it.
    s.p.action=17;assert(not s.grounded)
  end)
  test('unsafe grounded cleanup retains collision and retries when P1 is ready',function()
    local s,r=fixture();assert(r:command('play test'));local fly=r.g.fly;local blocked=true
    r.g.fly=function(...)if blocked then error('state cannot fly')end;return fly(...)end
    r:stop('envoy win');assert(r.stopping and r.current and s.hidden and next(s.areas))
    for _=1,12 do r:frame()end;assert(r.current and r.stopping)
    blocked=false;for _=1,6 do r:frame()end
    assert(not r.current and not r.stopping and not s.hidden)
  end)
end
