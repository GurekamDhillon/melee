-- Generate/validate incrementally, then hand one continuous world to staged install.
return function(D)
  local C={}
  function C.cancel(r)
    local t=r.world_generation
    if t then D.loader.generated[t.folder]=t.previous;r.world_generation=nil end
  end
  function C.settle(r)
    local t=r.world_pending;if not t or r.staging then return end
    if not r.last_install_error and r.current and r.current.doc.folder==t.next.folder then
      if t.old and t.old.folder~=t.next.folder then D.loader.generated[t.old.folder]=nil end
      if r.maze then D.loader.generated[r.maze.folder]=nil;r.maze=nil end
      r.world=t.next;r.world_pending=nil;r.g.log(D.maze_world.ascii(t.next.output))
    else D.loader.generated[t.next.folder]=t.previous;r.world_pending=nil end
  end
  function C.dispatch(r,words)
    if words[2]=='map'then assert(#words==2 and r.world,'mission world map requires a world');r.g.log(D.maze_world.ascii(r.world.output));return end
    local seed,regions,size
    if words[2]=='reroll'then assert(#words==2 and r.world,'mission world reroll requires a world');seed=(r.world.seed+1)%2147483647;regions=r.world.regions;size=r.world.size
    else assert(#words>=2 and #words<=4,'usage: mission world <seed> [regions] [size]');seed=assert(tonumber(words[2]),'invalid world seed');regions=words[3]and assert(tonumber(words[3]))or 4;size=words[4]and assert(tonumber(words[4]))or 12 end
    assert(seed%1==0 and math.abs(seed)<=2147483646 and regions%1==0 and regions>=2 and regions<=6 and size%1==0 and size>=8 and size<=20,'invalid world seed/regions/size')
    assert(r.g.match().active and not r.g.match().netplay,'active offline match required')
    assert(not r.world_generation,'world generation already running')
    local templates=D.loader.data(r.g,'missions/maze-chunks/library.lua',true)
    local catalogue=D.loader.catalogue(r.g,'missions/')
    local name='world_'..seed..'_'..regions..'_'..size;local folder='missions/'..name..'/'
    local t={folder=folder,previous=D.loader.generated[folder],frames=0,phase='regions'}
    local counts={}
    local function work(label)
      counts[label]=(counts[label]or 0)+1
      local batch=label=='encode'and 64 or label=='route'and 2 or label=='partition'and 2 or
        (label=='graph proof'or label=='normalize zone'or label=='normalize marker'or label=='zone membership'or label=='geometry'or label=='region geometry'or label=='metadata'or label=='connector chunk'or label=='clearance'or label=='check child'or label=='validate child')and 8 or 1
      if counts[label]%batch==0 then coroutine.yield(label)end
    end
    t.co=coroutine.create(function()
      local output=D.maze_world.generate(seed,{regions=regions,size=size,templates=templates,work=work})
      local files=D.maze.files(output,name,work)
      D.loader.generated[folder]={files=files,catalogue=catalogue,stamp=(t.previous and t.previous.stamp or 0)+1}
      local doc=D.loader.folder(r.g,name,work);D.loader.generated[folder].doc=doc
      t.next={seed=seed,regions=regions,size=size,output=output,folder=folder};t.name=name
    end)
    if r.current then D.commands.cancel(r);r.current.stream.destination=nil end
    r.world_generation=t;r.g.log('mission: world generating '..name)
  end
  function C.tick(r)
    local t=r.world_generation;if not t then return end;t.frames=t.frames+1
    if coroutine.status(t.co)~='dead'then
      local ok,phase=coroutine.resume(t.co)
      if not ok then C.cancel(r);r.g.log('mission: world refused '..tostring(phase));return end
      t.phase=phase or 'install'
      if t.frames%60==0 then r.g.log('mission: world generating '..t.phase)end
      return
    end
    if not D.fighters.stage_ready(r.g)then
      t.wait=(t.wait or 0)+1;if t.wait>=600 then C.cancel(r);r.g.log('mission: world timed out waiting for reserve-ready P1')end;return
    end
    local ok,why=pcall(r.install,r,t.name,nil,false)
    if not ok then C.cancel(r);r.g.log('mission: world install refused '..tostring(why));return end
    r.world_generation=nil;r.world_pending={old=r.world,next=t.next,previous=t.previous}
    C.settle(r)
  end
  return C
end
