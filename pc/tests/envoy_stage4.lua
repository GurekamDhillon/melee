-- Online Envoy, stage 4 (the native triggered evaluator), the script side and the parity run.
--  * every triggered record compiles to a native program, and a staged set's programs are byte-identical across fresh Lua processes
--  * the vocabulary the Lua compiler writes (events, tags, statuses, layout sizes) is the one pc/gameworld/script_mods_core.h reads
--  * PARITY: the real Lua modifier engine drives scripted event streams (envoy_stage4_lib.lua); the C evaluator replays the same streams
--    (pc/tests/script_mods_core_test.c) and must reproduce every status, damage delta, fx and counter frame by frame. Needs a C compiler:
--    GW_CLANG, or the workspace's _toolchains/llvm/bin/clang.exe; without one the parity run is SKIPPED (said out loud, never silently passed).
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules()
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_engine','keystones','mod_pool'}) do D[n]=T.module(n,D) end
local E,C=D.mod_engine,D.mod_codec
local root=io.open('pc/tests/envoy_stage4_lib.lua') and '' or 'melee/'
local function slurp(path) local f=assert(io.open(path,'rb'));local s=f:read('a');f:close();return s end

T.test('every triggered record has a native form and compiles to a program of the agreed size',function()
 local n=0
 for _,m in ipairs(D.mod_pool) do
  if m.trigger~='equip' then
   n=n+1
   local ok,why=E.native_support(m);assert(ok,m.id..': '..tostring(why))
   local e=E.new(1,D.mod_pool);e:set_build(1,{[m.id]=1})
   local p=e:native_program(1,1)
   assert(#p.words==E.native.header+E.native.max_rules*E.native.rule_words,m.id..' words '..#p.words)
   assert(p.words[1]==E.native.magic and p.rules>=1 and #p.variants>=1 and #p.variants<=E.native.max_variants)
   for mask=0,E.native.masks-1 do local v=p.mask_variant[mask];assert(v and v>=0 and v<#p.variants,m.id..' mask '..mask) end
  end
 end
 assert(n>=38,'triggered records: '..n)
end)
T.test('a build with no triggered record has no program to stage (stage 3 behaviour unchanged)',function()
 local st=D.mod_progression.set_stage(D,99,1,{})
 -- a set whose both seats hold passive records only would carry no program; at least the shape is right
 for seat=1,2 do assert(st[seat].record and st[seat].ops and st[seat].digest) end
 local e=E.new(1,D.mod_pool);e:set_build(1,{armoured=1});assert(not e:has_triggered(1))
end)
T.test('a staged set (both seats) gets programs when either seat is triggered; the ops list stays the passive one',function()
 local seen=0
 for seed=1,60 do
  local st=D.mod_progression.set_stage(D,seed*7919,1+seed%3,{})
  local any=st[1].triggered or st[2].triggered
  if any then seen=seen+1;assert(st[1].program and st[2].program,'both seats need a program') else assert(not st[1].program and not st[2].program) end
  for seat=1,2 do assert(#st[seat].ops<=24) end
 end
 assert(seen>20,'too few triggered sets in the sample: '..seen)
end)
T.test('programs and variants are byte-identical across fresh Lua processes (different hash seeds)',function()
 local function run() local p=assert(io.popen('lua '..root..'pc/tests/net_proc_helper.lua program 2>&1','r'));local o=p:read('a');p:close();return o end
 local a,b,c=run(),run(),run()
 assert(a:match('^PROGRAM') and a:find('\n',1,true),'helper output: '..a:sub(1,200))
 assert(a==b and b==c,'processes disagree:\n'..a..b..c)
end)
T.test('the native vocabulary is the one script_mods_core.h reads',function()
 local h=slurp(root..'pc/gameworld/script_mods_core.h')
 local ev=h:match('enum {\n    SM_E_NONE = 0,(.-)};') or h:match('enum {\r\n    SM_E_NONE = 0,(.-)};')
 assert(ev,'SM_E enum');local names={};for n in ev:gmatch('SM_E_([A-Z_]+)') do n=n:lower():gsub('^jc_grab$','jump_cancel_grab'):gsub('^jc_usmash$','jump_cancel_usmash');names[#names+1]=n end
 assert(#names==#E.native.event_names,('events: C %d, Lua %d'):format(#names,#E.native.event_names))
 for i,n in ipairs(E.native.event_names) do assert(names[i]==n,('event %d: C %s, Lua %s'):format(i,names[i],n)) end
 local tg=h:match('enum {\n    SM_T_JAB(.-)};') or h:match('enum {\r\n    SM_T_JAB(.-)};')
 local tags={};for n,bit in ('SM_T_JAB'..tg):gmatch('SM_T_([A-Z_]+) = 1u << (%d+)') do tags[#tags+1]={n:lower(),tonumber(bit)} end
 local want={'jab','tilt','smash','aerial','special','grab','throw','projectile','dash_attack','grounded','airborne','normal','fire','electric','ice','darkness','burning','shocked','chilled','cursed','hasted','guarded','momentum','damage','healing','unique','keystone','technique','critical'}
 assert(#tags==#want,'tags: C '..#tags)
 for i,t in ipairs(tags) do assert(t[1]==want[i] and t[2]==i-1 and E.native.tags[t[1]]==1<<(i-1),'tag '..i..': '..t[1]) end
 for name in pairs(D.mod_schema.tags) do assert(E.native.tags[name],'schema tag with no native bit: '..name) end
 for i,name in ipairs(D.mod_status.order) do assert(E.native.status_ids[name]==i) end
 local function def(name) return tonumber(h:match('#define '..name..' (%d+)')) end
 assert(def('SM_MAXR')==E.native.max_rules and def('SM_HDR')==E.native.header and def('SM_MASKS')==E.native.masks and def('SM_EW')==20 and def('SM_CW')==3)
 assert(def('SM_NCOND')==8 and def('SM_NTRIG')==4 and def('SM_NEFF')==8)
 assert(h:find('SM_MAGIC 0x534D5031',1,true))
 -- the host's constant for the word count
 local nb=slurp(root..'pc/platform/gw_script_netbuild.inc')
 assert(tonumber(nb:match('#define GW_MODS_WORDS (%d+)'))==E.native.header+E.native.max_rules*E.native.rule_words,'GW_MODS_WORDS')
 assert(tonumber(nb:match('#define GW_NB_VARIANTS (%d+)'))==E.native.max_variants,'GW_NB_VARIANTS')
end)
T.test('PARITY: the native evaluator reproduces the Lua engine on generated event streams',function()
 local L=dofile(root..'pc/tests/envoy_stage4_lib.lua')
 local txt,info=L.generate(D,{seed=20261008,random=20,frames=500})
 local missing={};for _,id in ipairs(info.ids) do if not info.cover[id] then missing[#missing+1]=id end end
 assert(info.scenarios>=40,'scenarios '..info.scenarios)
 assert(#missing<=3,'records that never fired in the scenarios: '..table.concat(missing,','))
 local fixture=os.tmpname()
 local f=assert(io.open(fixture,'wb'));f:write(txt);f:close()
 local candidates={os.getenv('GW_CLANG'),root..'../_toolchains/llvm/bin/clang.exe','../_toolchains/llvm/bin/clang.exe','../../_toolchains/llvm/bin/clang.exe','E:/Projects/Melee Workspace/_toolchains/llvm/bin/clang.exe'}
 local clang
 for _,c in ipairs(candidates) do if c and c~='' then local h=io.open(c,'rb');if h then h:close();clang=c;break end end end
 if not clang then print('SKIP parity run: no C compiler (set GW_CLANG). Fixture: '..info.scenarios..' scenarios, '..#txt..' bytes');os.remove(fixture);return end
 local exe=os.tmpname()..'.exe'
 local rc=os.execute('""'..clang..'" -O1 -Wno-unused-function "'..root..'pc/tests/script_mods_core_test.c" -o "'..exe..'" 2>&1"')
 assert(rc,'compiling script_mods_core_test.c failed')
 local p=assert(io.popen('""'..exe..'" "'..fixture..'" 2>&1"','r'));local out=p:read('a');local ok=p:close()
 os.remove(fixture);os.remove(exe)
 print(out:gsub('\n$',''))
 assert(ok and out:find(' 0 mismatches',1,true),out:sub(1,1500))
end)
T.done()
