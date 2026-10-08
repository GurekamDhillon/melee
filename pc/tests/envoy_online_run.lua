-- Online Envoy, stage 5 (_research/envoy-netplay-scoping-2026-10-05.md section 5; plan 2026-10-08-envoy-online-stage5.md): the script side of the run record.
-- The record itself is native (pc/platform/gw_netrun.h, tested by the netplay_run_* native tests); this checks that its digest is the codec's, that a saved
-- Versus run turns into an offline carry that validates and rebuilds the online build, that nothing of the new code reads the wall clock or a default RNG,
-- and that the bundle and the bindings carry the new calls.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules()
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_engine','keystones','mod_pool','drive_loot'}) do D[n]=T.module(n,D) end
local C=D.mod_codec
local function read(name) local f=assert(io.open(T.root..name..'.lua','rb'));local s=f:read('a');f:close();return s end

T.test('the native run record digest is mod_codec.digest64(body, "netrun:") (the vector the native test checks)',function()
 local body='RN2|123456789|2|0|1|1|0|0|45560001|3|1|1302|abcdef123456|deadbeef|-|0|1|0'
 assert(C.digest64(body,'netrun:')=='235fd23981123889',C.digest64(body,'netrun:'))
 assert(C.digest64(body,'build:')~=C.digest64(body,'netrun:'),'the salts must separate a build record from a run record')
end)

-- a saved run, as gd.netplay_run() presents it: seed, the last resolved reward, both players' picks, the seat
local function run(seed,round,me)
 return {seed=seed,round=round,game=round,me=me,mode='versus',state='interrupted',digest='0000000000000000',picks={[2]={1,3},[3]={0,2},[4]={2,1}}}
end

T.test('set_carry rebuilds the online build: the same records at the same tiers, as drives that validate',function()
 local loot=D.drive_loot.new(D.mod_pool)
 for _,seed in ipairs({7,123456,2147483000}) do
  for seat=1,2 do
   local r=run(seed,4,seat-1)
   local eq=D.mod_progression.set_build(D,seed,4,seat,r.picks)
   local carry=D.mod_progression.set_carry(D,r)
   assert(carry.seat==seat and carry.game==4)
   local seen={}
   for _,d in ipairs(carry.drives) do
    assert(loot:validate(d),'a carried drive does not validate');local a=d.affixes[1];seen[a.id]=a.tier
    assert(d.colour=='white' and d.rarity=='common' and #d.affixes==1)
   end
   local n=0;for id,t in pairs(eq) do
    local kind;for _,m in ipairs(D.mod_pool) do if m.id==id then kind=m.kind end end
    if kind=='keystone' then local found;for _,k in ipairs(carry.keystones) do if k==id then found=true end end;assert(found,'keystone '..id..' lost')
    else
     local was_raised=false;for _,r in ipairs(carry.raised) do if r==id then was_raised=true end end
     assert(seen[id]==t or (was_raised and seen[id]>t),('%s: tier %s carried as %s'):format(id,tostring(t),tostring(seen[id])));n=n+1
     if was_raised then local m;for _,x in ipairs(D.mod_pool) do if x.id==id then m=x end end;assert(m.min_depth and m.min_depth>=5,id..' was raised without a depth floor') end
    end
   end
   local c=0;for _ in pairs(seen) do c=c+1 end;assert(c==n and #carry.drives==n,'the carry has other records than the build')
   assert(carry.depth==math.max(r.game-1,5*math.max(0,n-4)) and #carry.keystones>=1)
  end
 end
end)

T.test('set_carry is a pure function of the record; the host and the guest seat differ; a missing pick is the default',function()
 local a,b=D.mod_progression.set_carry(D,run(99,3,0)),D.mod_progression.set_carry(D,run(99,3,0))
 assert(C.encode(a)==C.encode(b),'same record, different carry')
 local g=D.mod_progression.set_carry(D,run(99,3,1));assert(C.encode(a)~=C.encode(g),'both seats carried the same build')
 local nopicks=D.mod_progression.set_carry(D,{seed=99,round=3,me=0});assert(#nopicks.drives>=1,'no picks: the defaults still make a build')
 local r1=D.mod_progression.set_carry(D,{seed=99,round=0,me=0});assert(r1.game==1 and #r1.drives==1 and #r1.keystones==1,'round 0 is the starter and the keystone')
 T.refuses(function() D.mod_progression.set_carry(D,nil) end)
 T.refuses(function() D.mod_progression.set_carry(D,{seed=0}) end)
end)

T.test('the new calls have no wall clock, no default RNG and no os access',function()
 local forbidden={'math%.random','os%.time','os%.clock','os%.date','%f[%w_]g%.time','%.time%('}
 for _,name in ipairs({'mod_progression'}) do
  for raw in read(name):gmatch('[^\r\n]+') do
   local line=raw:gsub('^%s*%-%-.*$',''):gsub(' %-%-.*$','')
   for _,pat in ipairs(forbidden) do assert(not line:find(pat),name..': '..pat..': '..line:sub(1,120)) end
  end
 end
end)

T.test('the rule host carries a build in, and the console command and the bundle exist',function()
 local host=read('run_host');assert(host:find('function H:carry_in',1,true) and host:find('dev.carry',1,true))
 local app=read('retail_app');assert(app:find("arg=='continue'",1,true) and app:find("netplay_act('rstate','continued')",1,true) and app:find('set_carry',1,true))
 local lab=read('mod_lab');assert(lab:find("word=='run'",1,true))
 local main=read('main');assert(main:find('function P.set_carry',1,true) and main:find('function H:carry_in',1,true),'main.lua is stale: run tools/port/envoy_bundle.py')
end)

T.test('offline carry preserves depth and loop from the saved boundary',function()
 local r=run(99,3,0);r.game=12;r.loop=2;r.stocks=6;r.continues=0;r.lost=false
 local carry=D.mod_progression.set_carry(D,r)
 assert(carry.depth>=11,'saved depth was lost')
 assert(carry.loop==2,'saved loop was lost')
 assert(carry.stocks==6 and carry.continues==0 and carry.lost==false,'run resources were lost')
end)

T.done()
