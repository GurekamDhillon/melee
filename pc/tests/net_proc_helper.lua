-- Helper for envoy_online.lua and envoy_online_build.lua: print what a FRESH Lua process computes, so the tests can compare processes
-- (Lua randomises string hashing per state: any dependence on `pairs` order would show up as a different line).
--   lua net_proc_helper.lua        a fixed build: record, pool digest, compiled values digest
--   lua net_proc_helper.lua set    a staged Versus set: both seats' records and the digest of their compiled ops
local T=dofile('melee/pc/tests/envoy_testlib.lua');local D=T.rules()
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_engine','keystones','mod_pool'}) do D[n]=T.module(n,D) end
local C=D.mod_codec
if arg and arg[1]=='set' then
 local picks={[2]={[1]=1,[2]=0},[3]={[1]=0,[2]=3},[4]={[1]=2,[2]=2}}
 for game=1,4 do
  local st=D.mod_progression.set_stage(D,31337,game,picks)
  for seat=1,2 do print('SET',game,seat,st[seat].record,C.digest64(C.encode(st[seat].ops),'ops:')) end
 end
 return
end
local eq={};local want={'kindling','pyre','heavy','featherweight','armoured','cinder','shatter','lingering'}
for _,id in ipairs(want) do eq[id]=1 end
eq.glass_core=1
print((C.build_record({seed=12345,game=2,loop=0,port=1},eq,{damage_dealt=1.1,status_duration=1.5})))
print(C.pool_digest(D.mod_pool))
local e=D.mod_engine.new(1,D.mod_pool);e:set_build(1,eq,{damage_dealt=1.1})
local v=e:values(1);local r,bits=e:native_rules(1)
print(C.digest64(C.encode({values=v,rules=r,bits=bits}),'proc:'))
