-- Online Envoy, stages 0 and 1 (_research/envoy-netplay-scoping-2026-10-05.md): nothing a hosted run decides may come from the wall clock,
-- the default RNG or table order, and a player's build has ONE canonical record with a digest both peers can compare.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules()
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_engine','keystones','mod_pool'}) do D[n]=T.module(n,D) end
local C=D.mod_codec
local function read(name) local f=assert(io.open(T.root..name..'.lua','rb'));local s=f:read('a');f:close();return s end

-- ---- stage 0: hygiene ---------------------------------------------------------------------------------------------------
T.test('online-path modules contain no wall clock, no default RNG and no os calls',function()
 local mods={'mod_engine','mod_budget','mod_registry','mod_schema','mod_status','mod_skill','mod_progression','mod_pool','mod_techniques','mod_graph','mod_tuning','mod_synergy',
  'keystones','drive_loot','drive_bag','drive_merge','drive_economy','foe_roll','mod_echo','mod_codec','fighters'}
 local forbidden={'math%.random','os%.time','os%.clock','os%.date','%f[%w_]g%.time','self%.g%.time','%.time%('}
 for _,name in ipairs(mods) do
  local ok,src=pcall(read,name);assert(ok,'cannot read '..name)
  local n=0
  for raw in src:gmatch('[^\r\n]+') do n=n+1;local line=raw:gsub('^%s*%-%-.*$',''):gsub(' %-%-.*$','') -- code only: comments may talk about the clock
   for _,pat in ipairs(forbidden) do
    -- the one named per-peer draw (mod_codec.fresh_seed) is the only allowed mention
    if line:find(pat) and not line:find('(random or math.random)',1,true) then error(name..'.lua:'..n..' uses '..pat..': '..line:sub(1,120)) end
   end
  end
 end
end)
T.test('the run-seed formula is the same in every module that rolls',function()
 local f='(seed+stage*104729+loop*15485863+%s*32452843)%%2147483646+1'
 assert(read('run_host'):find(f:format('port'),1,true),'run_host seed_for changed')
 assert(read('coop'):find(f:format('k'),1,true),'coop seed_for changed')
 assert(read('mod_codec'):find(f:format('k'),1,true),'codec seed_for changed')
 for _,a in ipairs({{1,0,0,1},{99,3,1,2},{2147483000,7,2,4}}) do
  assert(C.seed_for(a[1],a[2],a[3],a[4])==(a[1]+a[2]*104729+a[3]*15485863+a[4]*32452843)%2147483646+1)
 end
end)
T.test('a hosted run seed is never drawn locally; offline it is unchanged',function()
 local online={match=function() return {netplay=true} end}
 local ok,err=pcall(C.fresh_seed,online,function() return 5 end);assert(not ok and tostring(err):find('lobby host'),'online draw must refuse')
 local offline={match=function() return {netplay=false} end}
 assert(C.fresh_seed(offline,function(a,b) assert(a==0 and b==2147483646);return 77 end)==77)
 assert(C.fresh_seed(nil,function() return 78 end)==78)
 local a,b=C.rng(5),C.rng(5);for _=1,20 do assert(a()==b()) end
 local c,d=C.rng(5),C.rng(6);assert(c()~=d(),'adjacent seeds must not share a first draw')
end)
T.test('the reward countdown of an online run counts ticks, never the wall clock',function()
 local src=read('run_screen')
 assert(src:find('not self.deterministic and self.g.time',1,true),'seconds_left must test the deterministic flag')
 assert(src:find('self.host.online',1,true),'the flag comes from the host')
 -- behaviour: load the module against a stub (its factory only needs D at call time)
 D.grid=dofile(T.root..'../../demos/grid-inventory/scripts/grid.lua')
 local ok,S=pcall(function() return assert(loadfile(T.root..'run_screen.lua'))()(D) end)
 if ok and S then
  local self={mode='reward',deterministic=true,g={time=function() error('wall clock used') end},ticks=600,opened=0}
  local left=S.seconds_left(self);assert(math.abs(left-(S.tuning.safe_seconds-10))<1e-9,'600 ticks = 10 s, got '..tostring(left))
  self.deterministic=false;self.g.time=function() return 4 end;left=S.seconds_left(self);assert(math.abs(left-(S.tuning.safe_seconds-4))<1e-9,'offline keeps the wall clock')
 else print('note: run_screen not loadable in this fixture, source check only: '..tostring(S)) end
end)

-- ---- stage 1: the record ------------------------------------------------------------------------------------------------
local function build()
 return {kindling=1,glass_core=1,lingering={tier=2,copies=1},pyre={tiers={1,2}}},{damage_dealt=1.1,status_duration=1.5}
end
T.test('a build record is canonical: sorted, compact, round-trips, same bytes whatever the insertion order',function()
 local eq,imp=build();local rec,dig=C.build_record({seed=12345,game=2,loop=0,port=1},eq,imp)
 assert(rec:match('^EB1|12345|2|0|1|%x+|'),rec);assert(#rec<400 and #dig==16 and rec:sub(-16)==dig)
 local p=C.parse_build_record(rec);assert(p and p.digest==dig and p.meta.seed==12345 and p.meta.game==2 and p.meta.port==1)
 assert(p.equipped.kindling==1 and p.equipped.lingering==2 and p.equipped.pyre.tiers[2]==2 and p.implicits.damage_dealt==1.1)
 assert(#p.keystones==0,'glass_core is a unique, not a keystone')
 local eq2,imp2={},{};local ids={};for id in pairs(eq) do ids[#ids+1]=id end;table.sort(ids,function(a,b) return a>b end)
 for _,id in ipairs(ids) do eq2[id]=eq[id] end;imp2.status_duration=1.5;imp2.damage_dealt=1.1
 local rec2=C.build_record({seed=12345,game=2,loop=0,port=1},eq2,imp2);assert(rec2==rec,'insertion order changed the record')
 local again=C.build_record(p.meta,p.equipped,p.implicits);assert(again==rec,'parse then build is not the identity')
end)
T.test('keystones are listed and the record changes with every field',function()
 local ks;for _,m in ipairs(D.mod_pool) do if m.kind=='keystone' then ks=m.id break end end
 local eq,imp=build();eq[ks]=1;local rec=C.build_record({seed=1,game=0,loop=0,port=1},eq,imp);local p=assert(C.parse_build_record(rec))
 assert(#p.keystones==1 and p.keystones[1]==ks)
 local seen={[rec:sub(-16)]=true}
 local function differs(r) local d=r:sub(-16);assert(not seen[d],'a field did not change the digest');seen[d]=true end
 differs((C.build_record({seed=2,game=0,loop=0,port=1},eq,imp)))
 differs((C.build_record({seed=1,game=1,loop=0,port=1},eq,imp)))
 differs((C.build_record({seed=1,game=0,loop=1,port=1},eq,imp)))
 differs((C.build_record({seed=1,game=0,loop=0,port=2},eq,imp)))
 local e2,i2=build();e2[ks]=1;e2.kindling=2;differs((C.build_record({seed=1,game=0,loop=0,port=1},e2,i2)))
 local e3,i3=build();e3[ks]=1;i3.damage_dealt=1.2;differs((C.build_record({seed=1,game=0,loop=0,port=1},e3,i3)))
 local e4,i4=build();e4[ks]=1;e4.kindling=nil;differs((C.build_record({seed=1,game=0,loop=0,port=1},e4,i4)))
end)
T.test('the parser refuses a wrong version, a bad digest, unsorted fields, a lying keystone list, unknown ids and another pool',function()
 local eq,imp=build();local rec=C.build_record({seed=7,game=1,loop=0,port=2},eq,imp)
 local function bad(text,why) local p,e=C.parse_build_record(text);assert(p==nil and e and e:find(why,1,true),why..' expected, got '..tostring(e)) end
 local function resign(body) return body..'|'..C.digest64(body,'build:') end
 local body=rec:sub(1,-18)
 bad(rec:sub(1,-2)..'0','digest');bad('EB2'..rec:sub(4),'version');bad((rec:gsub('|','/')),'fields');bad(rec..'x','digest')
 local f={};for x in (body..'|'):gmatch('([^|]*)|') do f[#f+1]=x end
 local function with(i,v) local g={table.unpack(f)};g[i]=v;return resign(table.concat(g,'|')) end
 bad(with(7,'pyre=1,kindling=1'),'sorted');bad(with(7,'kindling=1,kindling=1'),'sorted');bad(with(7,'nonsense_id=1'),'unknown');bad(with(9,'lingering'),'keystone')
 bad(with(6,'0123456789abcdef'),'different modifier pool');bad(with(7,'kindling=0'),'tier');bad(with(2,'-3'),'seed');bad(with(2,'1.5'),'seed')
 bad(string.rep('x',500),'compact');bad('a b','compact')
 local ks;for _,m in ipairs(D.mod_pool) do if m.kind=='keystone' then ks=m.id break end end
 local e2,i2=build();e2[ks]=1;local r2=C.build_record({seed=7,game=1,loop=0,port=2},e2,i2);assert(C.parse_build_record(r2))
 local g2={};for x in (r2:sub(1,-18)..'|'):gmatch('([^|]*)|') do g2[#g2+1]=x end;g2[9]='';bad(resign(table.concat(g2,'|')),'keystone list')
end)
T.test('the pool digest ignores wording and looks but sees a rule change',function()
 local d=C.pool_digest(D.mod_pool);assert(d:match('^%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x$'))
 local copy={};for i,m in ipairs(D.mod_pool) do copy[i]=m end
 local m1={};for k,v in pairs(copy[1]) do m1[k]=v end;m1.text='other words';m1.label='Other';copy[1]=m1
 assert(C.pool_digest(copy)==d,'wording must not matter')
 local m2={};for k,v in pairs(D.mod_pool[2]) do m2[k]=v end;m2.cost=(m2.cost or 0)+.01;copy[1]=D.mod_pool[1];copy[2]=m2
 assert(C.pool_digest(copy)~=d,'a cost change must change the digest')
end)
T.test('three fresh Lua processes (different hash seeds) write byte-identical records, pool digest and compiled values',function()
 local function run() local p=assert(io.popen('lua melee/pc/tests/net_proc_helper.lua 2>&1','r'));local o=p:read('a');p:close();return o end
 local a,b,c=run(),run(),run()
 assert(a:match('^EB1|') and a:find('\n',1,true),'helper output: '..a:sub(1,200))
 assert(a==b and b==c,'processes disagree:\n'..a..b..c)
end)
T.done()
