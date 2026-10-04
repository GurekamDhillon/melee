-- Small mission extension: keep generated geometry on the standard loader path.
return function(D)
  local C={}
  function C.settle(r)
    local t=r.maze_pending;if not t or r.staging then return end
    if r.last_install_error then
      D.loader.generated[t.next.folder]=t.previous;r.maze_pending=nil
    elseif r.current and r.current.doc.folder==t.next.folder then
      if t.old and t.old.folder~=t.next.folder then D.loader.generated[t.old.folder]=nil end
      r.maze=t.next;r.maze_pending=nil;r.g.log(D.maze.ascii(t.next.output))
    else D.loader.generated[t.next.folder]=t.previous;r.maze_pending=nil
    end
  end
  function C.dispatch(r,words)
    assert(D.maze and D.maze_check,'maze modules missing from bundle')
    if words[2]=='map' then
      assert(#words==2 and r.maze,'usage: mission maze map (after generation)')
      r.g.log(D.maze.ascii(r.maze.output));return
    end
    local seed,size
    if words[2]=='reroll' then
      assert(#words==2 and r.maze,'usage: mission maze reroll (after generation)')
      seed=(r.maze.seed+1)%2147483647;size=r.maze.size
    else
      assert(#words==2 or #words==3,'usage: mission maze <seed> [size]')
      seed=tonumber(words[2]);size=words[3] and assert(tonumber(words[3]),'invalid size') or 12
    end
    local templates=D.loader.data(r.g,'missions/maze-chunks/library.lua',true)
    local output=D.maze.generate(seed,{size=size,templates=templates})
    local name='maze_'..seed..'_'..size;local folder='missions/'..name..'/'
    local catalogue=D.loader.catalogue(r.g,'missions/')
    assert(catalogue[D.maze_set.part],'maze kit missing; run tools/maze/generate.py --prepare-mod --kit <room-kit>')
    local previous=D.loader.generated[folder]
    D.loader.generated[folder]={files=D.maze.files(output,name),catalogue=catalogue,
      stamp=(previous and previous.stamp or 0)+1}
    local ok,why=pcall(r.install,r,name,nil,false)
    if not ok then D.loader.generated[folder]=previous;error(why,0) end
    local next_maze={seed=seed,size=size,output=output,folder=folder}
    if r.staging then r.maze_pending={old=r.maze,next=next_maze,previous=previous}
    else
      if r.maze and r.maze.folder~=folder then D.loader.generated[r.maze.folder]=nil end
      r.maze=next_maze;r.g.log(D.maze.ascii(output))
    end
  end
  return C
end
