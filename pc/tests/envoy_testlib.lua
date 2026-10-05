local T={count=0}
T.root='pc/scripts/examples/envoy/scripts/'
if not io.open(T.root..'genetics.lua') then T.root='melee/'..T.root end
function T.module(name,D) if D and name=='mod_pool' and not D.mod_techniques then D.mod_techniques=assert(loadfile(T.root..'mod_techniques.lua'))()(D) end;if D and not D.mod_status and name~='mod_status' then D.mod_status=assert(loadfile(T.root..'mod_status.lua'))()(D) end;if D and not D.mod_skill and name~='mod_skill' then D.mod_skill=assert(loadfile(T.root..'mod_skill.lua'))()(D) end;if D and not D.mod_registry and name~='mod_registry' then D.mod_registry=assert(loadfile(T.root..'mod_registry.lua'))()(D) end;if D and not D.mod_echo and name~='mod_echo' then D.mod_echo=assert(loadfile(T.root..'mod_echo.lua'))()(D) end;if D and not D.mod_progression and name~='mod_progression' then D.mod_progression=assert(loadfile(T.root..'mod_progression.lua'))()(D) end;if D and D.mod_schema and not D.mod_budget and name~='mod_budget' then D.mod_budget=assert(loadfile(T.root..'mod_budget.lua'))()(D) end;return assert(loadfile(T.root..name..'.lua'))()(D) end
function T.test(name,f)
  local ok,err=pcall(f)
  assert(ok,name..': '..tostring(err));T.count=T.count+1
end
function T.done() print('PASS '..T.count..' tests') end
function T.rules()
  local D={};D.mod_echo=T.module('mod_echo',D);D.pickup_juice=assert(loadfile(T.root:gsub('examples/envoy/scripts/$','lib/')..'pickup_juice.lua'))();D.genetics=T.module('genetics');D.companion=T.module('companion',D)
  return D
end
function T.missions()
  local path=T.root:gsub('envoy/scripts/$','missions/scripts/')
  local f=io.open('tools/port/missions_bundle.py') or assert(io.open('../tools/port/missions_bundle.py'))
  local source=f:read('a');f:close();local D={}
  for name in assert(source:match('MODULES = %((.-)%)')):gmatch("'([%w_]+)'") do
    local factory=assert(loadfile(path..name..'.lua'))()
    D[name]=name=='mission' and factory or factory(D)
  end
  return D
end
function T.refuses(f) assert(not pcall(f),'expected refusal') end
return T
