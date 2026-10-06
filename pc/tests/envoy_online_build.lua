-- Online Envoy, stages 2 and 3 on the script side: which records are online-safe, that the passive ops compiled for the native table are the ones the
-- offline rule host commits, and that the Versus set (starters, offers, picks, builds) is a pure function of (seed, game, seat, picks).
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
-- reuse the rule-host fixture of envoy_mod_lab.lua: everything before its first test
local f=assert(io.open(T.root..'../../../../tests/envoy_mod_lab.lua','rb'));local src=f:read('a');f:close()
local cut3=assert(src:find("T.test('LAB modifiers dormant",1,true),'fixture anchor moved')
local chunk=src:sub(1,cut3-1)..'\nreturn T,D,fixture'
chunk=assert(chunk:gsub("{'mod_schema','mod_codec','mod_engine','mod_pool','mod_echo_lab'}","{'mod_progression','mod_schema','mod_codec','mod_engine','keystones','mod_pool','mod_echo_lab'}",1))
local ok,T2,D,fixture=pcall(function() return assert(load(chunk,'@lab_fixture'))() end)
assert(ok,'cannot load the rule-host fixture: '..tostring(T2))
local C=D.mod_codec

local function census()
 local safe,unsafe,equip={}, {}, 0
 for _,m in ipairs(D.mod_pool) do
  if m.trigger=='equip' then equip=equip+1;local s,why=D.mod_engine.online_safe(m);if s then safe[#safe+1]=m.id else unsafe[#unsafe+1]=m.id..' ('..why..')' end end
 end
 return safe,unsafe,equip
end
T.test('the pool census: 38 passive records, 33 online-safe, the 5 echo records are not',function()
 local safe,unsafe,equip=census()
 assert(equip==38,'passive (equip) records: '..equip)
 assert(#safe==33 and #unsafe==5,('safe %d unsafe %d'):format(#safe,#unsafe))
 for _,u in ipairs(unsafe) do assert(u:find('echo'),'an unsafe record that is not an echo record: '..u) end
 local triggered=0;for _,m in ipairs(D.mod_pool) do if m.trigger~='equip' then local s=D.mod_engine.online_safe(m);assert(not s);triggered=triggered+1 end end
 assert(triggered==#D.mod_pool-38)
 print(('pool %d records: %d passive (%d online-safe, %d echo), %d triggered'):format(#D.mod_pool,equip,#safe,#unsafe,triggered))
end)
T.test('passive_ops refuses a triggered or echo record and a unknown id',function()
 local e=D.mod_engine.new(1,D.mod_pool);e:set_build(1,{kindling=1},{})
 local ok,err=pcall(e.passive_ops,e,1);assert(not ok and tostring(err):find('not online%-safe'),tostring(err))
 e:set_build(1,{trailing=1},{});ok,err=pcall(e.passive_ops,e,1);assert(not ok and tostring(err):find('echo'),tostring(err))
end)
T.test('every online-safe record compiles to ops the registry accepts, and they equal what the offline rule host commits',function()
 local safe=census()
 for _,id in ipairs(safe) do
  local e=D.mod_engine.new(1,D.mod_pool);e:set_build(1,{[id]=1},{})
  local ops=e:passive_ops(1,1);assert(#ops<=24,id..' ops '..#ops) -- lingering only lengthens statuses: no native op
  -- the offline host: the same build, one frame
  local s,a=fixture();assert(a:command('add '..id..' 1'));a:frame();a:frame()
  local mod,rules;for _,o in ipairs(ops) do if o.op=='fighter_mod' then mod=o.values elseif o.op=='hit_rules' then rules={rules=o.rules,bits=o.status_bits} end end
  assert(C.encode(s.mods[1] or false)==C.encode(mod or false),id..': fighter values differ from the rule host')
  assert(C.encode(s.hit_rules[1] or false)==C.encode(rules or false),id..': hit rules differ from the rule host')
 end
end)
T.test('the set: starters differ by seat, offers are three distinct online-safe drives, picks apply, keep and default behave',function()
 local P=D.mod_progression;local seed=424242
 local a1,k1=P.set_starter(D,seed,1);local a2,k2=P.set_starter(D,seed,2)
 assert(a1 and k1 and a2 and k2);assert(D.mod_pool and true)
 local kind={};for _,m in ipairs(D.mod_pool) do kind[m.id]=m.kind end
 assert(kind[a1]=='normal' and kind[k1]=='keystone')
 for s=1,40 do local x,y=P.set_starter(D,s*977,1);assert(D.mod_engine.online_safe((function() for _,m in ipairs(D.mod_pool) do if m.id==x then return m end end end)()));assert(kind[y]=='keystone') end
 local eq={[a1]=1,[k1]=1};local offers=P.set_offers(D,seed,2,1,eq)
 assert(#offers==3 and offers[1]~=offers[2] and offers[2]~=offers[3] and offers[1]~=offers[3])
 for _,id in ipairs(offers) do assert(kind[id]=='normal') end
 assert(table.concat(P.set_offers(D,seed,2,1,eq),',')==table.concat(offers,','),'offers are not a pure function')
 assert(table.concat(P.set_offers(D,seed,3,1,eq),',')~=table.concat(offers,','),'the game must change the offers')
 assert(table.concat(P.set_offers(D,seed,2,2,eq),',')~=table.concat(offers,','),'the seat must change the offers')
 local after=P.set_apply(eq,offers,1);assert(after[offers[2]]==1 and after[a1]==1 and eq[offers[2]]==nil,'a pick adds the record at tier 1 and leaves the old build alone')
 local twice=P.set_apply(after,{offers[2],offers[1],offers[3]},0);assert(twice[offers[2]]==2,'a held record goes up a tier')
 local capped=after;for _=1,5 do capped=P.set_apply(capped,{offers[2],'x','y'},0) end;assert(capped[offers[2]]==3,'tiers stop at 3')
 local kept=P.set_apply(eq,offers,3);assert(C.encode(kept)==C.encode(eq),'keep changes nothing')
 -- builds over a whole set, from the picks
 local picks={[2]={[1]=0,[2]=2},[3]={[1]=3,[2]=1}}
 local b1=P.set_build(D,seed,3,1,picks);local b2=P.set_build(D,seed,3,2,picks)
 assert(C.encode(b1)~=C.encode(b2),'the two players must end up with different builds')
 assert(C.encode((P.set_build(D,seed,3,1,picks)))==C.encode(b1))
 local dflt=P.set_build(D,seed,3,1,{});assert(C.encode(dflt)==C.encode((P.set_build(D,seed,3,1,{[2]={[1]=0},[3]={[1]=0}}))),'a missing pick is offer 1')
end)
T.test('staging a game: records parse, ops pass the registry, the keystone is in the record, both seats differ',function()
 local P=D.mod_progression;local st=P.set_stage(D,99,3,{[2]={[1]=1,[2]=0},[3]={[1]=0,[2]=3}})
 for seat=1,2 do
  local p=assert(C.parse_build_record(st[seat].record));assert(p.meta.port==seat and p.meta.game==3 and p.meta.seed==99 and #p.keystones==1)
  assert(#st[seat].ops>=1 and #st[seat].ops<=24)
  for _,o in ipairs(st[seat].ops) do D.mod_registry.operation(o) end
 end
 assert(st[1].digest~=st[2].digest)
end)
T.test('three fresh Lua processes stage byte-identical sets (records and compiled ops)',function()
 local function run() local p=assert(io.popen('lua melee/pc/tests/net_proc_helper.lua set 2>&1','r'));local o=p:read('a');p:close();return o end
 local a,b,c=run(),run(),run()
 assert(a:find('SET',1,true) and #a>60,'helper output: '..a:sub(1,200));assert(a==b and b==c,'processes disagree:\n'..a..b..c)
end)
T.done()
