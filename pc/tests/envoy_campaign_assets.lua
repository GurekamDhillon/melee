local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local mod=T.root:gsub('scripts/$','')
local shared=T.root:gsub('envoy/scripts/$','missions/scripts/')
local D=T.missions()
local function fixture(kit)
  local g,logs,paths={},{},{}
  g.model_load=function(path) if kit then return 17 end;return nil,'missing mesh' end
  g.model_release=function(h) assert(h==17) end
  g.log=function(s) logs[#logs+1]=s end
  g.mod_list=function(path)
    assert(path:sub(1,9)=='missions/' and not path:find('..',1,true) and not path:find('\\',1,true) and not path:find(':',1,true),'engine path refused: '..path)
    paths[#paths+1]=path
    if path=='missions/models/' and kit then
      local rows={};for _,name in ipairs({'bf_floor_4m','bf_wall_solid_4m','bf_wall_doorway_4m','bf_door_leaf','bf_beam_4m'}) do rows[#rows+1]={name=name..'.gxmesh'} end;return rows
    end
    local marker=io.open(mod..path..'README.md')
    if marker then marker:close();return {} end
    return nil,'directory missing'
  end
  g.mod_read=function(path)
    local f=io.open(mod..path)
    if not f then return nil,'missing' end
    local text=f:read('a');f:close();return text
  end
  g.mod_stamp=function(path) local f=io.open(mod..path);if f then f:close();return 1 end end
  return g,logs,paths
end
T.test('fresh install uses four authored depths and logs fallback once',function()
  local C=T.module('campaign',D);local g,logs=fixture(false)
  for _=1,2 do
    local levels=C.levels(42,g)
    assert(#levels==4)
    for i,l in ipairs(levels) do
      assert(l.depth==i and l.command=='play '..l.name)
      local doc=D.loader.folder(g,l.name);assert(doc.mission and #doc.level.lines>0)
    end
    assert(levels[4].name=='boss')
    assert(D.loader.folder(g,'hub').mission,'garden must load without optional assets')
  end
  assert(#logs==1 and logs[1]:find('authored',1,true))
end)
T.test('listed but unreadable mesh uses authored fallback',function()
  local C=T.module('campaign',D);local g=fixture(true)
  g.model_load=function() return nil,'missing sidecar' end
  assert(C.levels(42,g)[1].command=='play path')
end)
T.test('present maze kit starts generated and authored rooms through real loader',function()
  local C=T.module('campaign',D);local g=fixture(true)
  local levels=C.levels(42,g)
  assert(levels[1].command=='maze 42 8' and levels[3].command=='maze 43 16')
  for _,l in ipairs(levels) do
    local doc
    if l.command:sub(1,5)=='maze ' then
      local r={g=g,install=function(_,name) doc=D.loader.folder(g,name) end}
      local words={};for w in l.command:gmatch('%S+') do words[#words+1]=w end
      D.maze_commands.dispatch(r,words)
    else doc=D.loader.folder(g,l.name) end
    assert(doc and doc.mission and (#doc.level.lines>0 or #doc.chunks>0))
  end
end)
T.test('catalogue refusal safely chooses authored rooms',function()
  local C=T.module('campaign',D);local g,logs=fixture(false)
  g.mod_list=function() error('unavailable') end
  local levels=C.levels(1,g)
  assert(levels[1].command=='play path' and #logs==1)
end)
T.test('partial kit cannot select a maze with unresolved chunk models',function()
  local C=T.module('campaign',D);local g=fixture(true);local list=g.mod_list
  g.mod_list=function(path) if path=='missions/models/' then return {{name=D.maze_set.part..'.gxmesh'}} end;return list(path) end
  assert(C.levels(42,g)[1].command=='play path')
end)
T.done()
