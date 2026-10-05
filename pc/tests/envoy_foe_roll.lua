local T=dofile('melee/pc/tests/envoy_testlib.lua')
local D={};for _,n in ipairs({'mod_schema','mod_codec','mod_engine','mod_pool','drive_loot','drive_bag'}) do D[n]=T.module(n,D) end
T.test('seeded opponent module exists',function() assert(io.open(T.root..'foe_roll.lua'),'foe roll missing') end)
D.foe_roll=T.module('foe_roll',D)
T.test('bounded seeded rolls validate and average around representative strengths',function()
 local r=D.foe_roll.new(D.mod_pool);local unique,key=0,0
 for _,target in ipairs({1,1.4,1.8}) do local sum=0
  for seed=1,120 do local a=r:roll(target,seed,3,2,{depth=12});r:validate(a);assert(D.mod_codec.encode(a)==D.mod_codec.encode(r:roll(target,seed,3,2,{depth=12})));sum=sum+a.strength
   for _,v in pairs(a.build.equipped) do if v.unique then unique=unique+1 end end;if a.build.keystone then key=key+1 end
  end
  local mean=sum/120;print(('foe target %.2f mean %.6f tolerance %.2f'):format(target,mean,r.tolerance));local want=target*D.mod_progression.factor({depth=12});assert(math.abs(mean-want)<=r.tolerance*want/target,'mean '..mean..' vs target*factor '..want)
 end
 assert(unique>0 and key>0);print('unique '..unique..' keystone '..key)
 T.refuses(function()r:roll(math.huge,1,3,2)end)
end)
T.done()
