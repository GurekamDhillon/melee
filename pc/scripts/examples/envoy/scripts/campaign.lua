-- Four real shared-runtime rooms; generated names follow maze_commands.lua.
return function(D)
  local C={}
  local warned=false
  local function maze_available(g,seed)
    if not g then return true end -- Pure seed callers do not resolve installed assets.
    if not D or not D.loader or not D.maze_set then return false end
    local ok,catalogue=pcall(D.loader.catalogue,g,'missions/')
    if not ok or not catalogue[D.maze_set.part] or not D.maze then return false end
    if type(g.model_load)~='function' or type(g.model_release)~='function' then return false end
    local generated,parts=pcall(function()
      local templates=D.loader.data(g,'maze-chunks/library.lua',true)
      local needed={[D.maze_set.part]=true}
      for i,size in ipairs({8,16}) do
        local maze=D.maze.generate((seed+i-1)%2147483647,{size=size,templates=templates})
        for _,cell in ipairs(maze.cells) do for _,part in ipairs(cell.level.parts or {}) do needed[part.part]=true end end
      end
      return needed
    end)
    if not generated then return false end
    for part in pairs(parts) do
      if not catalogue[part] then return false end
      local loaded,h=pcall(g.model_load,catalogue[part])
      if not loaded or not h then return false end
      local released,why=pcall(g.model_release,h)
      if not released then
        if type(g.log)=='function' then g.log('envoy: maze probe release refused '..tostring(why)) end
        return false
      end
    end
    return true
  end
  function C.levels(seed,g)
    assert(type(seed)=='number' and seed%1==0 and seed>=0 and seed<2147483647,'invalid campaign seed')
    local second=(seed+1)%2147483647
    if not maze_available(g,seed) then
      if not warned then
        warned=true
        if g and type(g.log)=='function' then
          g.log('envoy: optional maze kit unavailable; using authored rooms for this campaign')
        end
      end
      return {
        {command='play path',name='path',theme='First crossing',depth=1},
        {command='play path',name='path',theme='The crossing',depth=2},
        {command='play path',name='path',theme='Deep crossing',depth=3},
        {command='play boss',name='boss',theme='Boss room',depth=4},
      }
    end
    return {
      {command='maze '..seed..' 8',name='maze_'..seed..'_8',theme='First maze',depth=1},
      {command='play path',name='path',theme='The crossing',depth=2},
      {command='maze '..second..' 16',name='maze_'..second..'_16',theme='Deep maze',depth=3},
      {command='play boss',name='boss',theme='Boss room',depth=4},
    }
  end
  return C
end
