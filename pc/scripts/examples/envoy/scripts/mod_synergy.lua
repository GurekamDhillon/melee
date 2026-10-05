-- The synergy table of the pool: DERIVED from the interaction graph (mod_graph), never typed and never a shared-tag overlap.
-- A pair is listed when one record's effect gives what the other's trigger, conditions or effect reads (who feeds whom). `reasons` name the key and say
-- whether the reader is triggered by it ("event:") or its effect needs it ("effect:"). `degree` counts a record's pairs.
return function(D)
 local M={}
 function M.generate(pool)
  local G=D.mod_graph or assert(loadfile and nil,'mod_graph required')
  local g=G.graph(pool);local pairs,by,degree={},{},{}
  for _,m in ipairs(pool) do degree[m.id]=0 end
  local function add(e)
   local a,b=e.from,e.to;local flip=a>b;local x,y=flip and b or a,flip and a or b
   local key=x..'|'..y;local p=by[key]
   if not p then p={a=x,b=y,reasons={}};by[key]=p;pairs[#pairs+1]=p;degree[x]=degree[x]+1;degree[y]=degree[y]+1 end
   local prof=g.prof[e.to];local triggered=false
   for _,s in ipairs(prof.sources) do for _,k in ipairs(s) do if k==e.why then triggered=true end end end
   p.reasons[#p.reasons+1]=(triggered and 'event:' or 'effect:')..e.from..'>'..e.to..':'..e.why
  end
  for _,e in ipairs(g.list) do add(e) end
  table.sort(pairs,function(p,q) if p.a~=q.a then return p.a<q.a end;return p.b<q.b end)
  for _,p in ipairs(pairs) do table.sort(p.reasons) end
  return pairs,degree
 end
 return M
end
