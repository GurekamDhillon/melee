local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local factory=loadfile(T.root..'campaign.lua')
T.test('four rooms preserve deterministic seeds and depth budgets',function()
  assert(factory,'campaign module missing')
  local C=factory()()
  local levels=C.levels(42)
  assert(#levels==4 and levels[1].command=='maze 42 8')
  assert(levels[2].command=='play path' and levels[3].command=='maze 43 16')
  assert(levels[4].command=='play boss')
  assert(levels[1].name=='maze_42_8' and levels[3].name=='maze_43_16')
  for i,l in ipairs(levels) do assert(l.depth==i and type(l.theme)=='string') end
end)
T.test('shared generator emits more real enemies at later maze depth',function()
  local root=T.root:gsub('envoy/scripts/$','missions/scripts/')
  local D=T.missions()
  for seed=0,99 do
    local function enemies(n,size)
      local output=D.maze.generate(n,{size=size})
      local files=D.maze.files(output,'test')
      local mission=assert(load(files['missions/test/mission.lua']))()
      return #mission.enemies
    end
    assert(enemies((seed+1)%2147483647,16)>enemies(seed,8))
  end
end)
T.done()

