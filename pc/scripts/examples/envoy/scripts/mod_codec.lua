-- Canonical bounded data encoding. No load(), functions, metatables or wall clock.
return function(D)
 local C={}
 -- One shared output buffer (a table's pieces are appended in place, one concat at the end) rather than a
 -- string per node: the checkpoint is encoded every logic frame, so allocation, not parsing, is its cost.
 -- Sequences (keys exactly 1..n) skip the sort, which for them is the identity. Output bytes are unchanged.
 local format,concat,sort=string.format,table.concat,table.sort
 local function order(a,b) if type(a)~=type(b) then return type(a)=='number' end;return a<b end
 function C.encode(value)
  local nodes,active=0,{}
  local buf,n={},0
  local pack
  function pack(v,depth)
   nodes=nodes+1;assert(nodes<=6000 and depth<=20,'modifier state too complex')
   local kind=type(v)
   if kind=='boolean' then n=n+1;buf[n]=v and 't' or 'f';return end
   if kind=='number' then assert(v==v and math.abs(v)<1e15,'non-finite state');local s=format('%.17g',v);n=n+1;buf[n]='d'..#s..':'..s;return end
   if kind=='string' then n=n+1;buf[n]='s'..#v..':'..v;return end
   assert(kind=='table' and not getmetatable(v) and not active[v],'plain acyclic state required');active[v]=true
   local count=0;for k in pairs(v) do assert(type(k)=='string' or type(k)=='number' and k%1==0,'invalid state key');count=count+1 end
   assert(count<=512,'checkpoint table too large')
   n=n+1;buf[n]='{'..count..':'
   local seq=#v==count;if seq then for i=1,count do if v[i]==nil then seq=false;break end end end -- a border can equal the count with holes
   if seq then for i=1,count do pack(i,depth+1);pack(v[i],depth+1) end
   else
    local keys,i={},0;for k in pairs(v) do i=i+1;keys[i]=k end
    sort(keys,order)
    for j=1,count do local k=keys[j];pack(k,depth+1);pack(v[k],depth+1) end
   end
   active[v]=nil
  end
  pack(value,0);local out=concat(buf,'',1,n);assert(#out<=16384,'modifier checkpoint exceeds 16 KiB');return out
 end
 function C.decode(text)
  assert(type(text)=='string' and #text<=16384,'invalid modifier checkpoint');local at,nodes=1,0
  local function count()
   local endat=text:find(':',at,true);assert(endat and endat-at<7,'invalid checkpoint length')
   local s=text:sub(at,endat-1);assert(s:match('^%d+$'),'invalid checkpoint count');at=endat+1;return tonumber(s)
  end
  local function read(depth)
   nodes=nodes+1;assert(nodes<=6000 and depth<=20,'checkpoint too complex')
   local kind=text:sub(at,at);at=at+1
   if kind=='t' then return true elseif kind=='f' then return false end
   if kind=='s' or kind=='d' then
    local n=count();assert(at+n-1<=#text,'truncated checkpoint');local s=text:sub(at,at+n-1);at=at+n
    if kind=='s' then return s end;local v=tonumber(s);assert(v and v==v and math.abs(v)<1e15,'invalid state number');return v
   end
   assert(kind=='{','invalid checkpoint type');local n=count();assert(n<=512,'checkpoint table too large');local out={}
   for _=1,n do local k=read(depth+1);assert(type(k)=='string' or type(k)=='number' and k%1==0,'invalid checkpoint key');assert(out[k]==nil,'duplicate checkpoint key');out[k]=read(depth+1) end
   return out
  end
  local out=read(0);assert(at==#text+1,'trailing checkpoint bytes');return out
 end
 -- ---- online play (Envoy netplay plan, stages 0 and 1) ------------------------------------------------------------------------
 -- Everything two peers must AGREE on is built here from pure functions: seeds, a canonical build record, and digests of both.
 -- No wall clock, no math.random except in the one named per-peer draw (fresh_seed), and every set is written in sorted order, so
 -- two processes (Lua randomises string hashing per state, so `pairs` order differs between them) write the same bytes.
 C.record_version=1
 local function fnv32(text,h)
  h=h or 2166136261
  for i=1,#text do h=((h~text:byte(i))*16777619)&0xFFFFFFFF end
  return h
 end
 C.fnv32=fnv32
 -- 64-bit digest as 16 hex characters: two FNV-1a words, the second salted. Not cryptographic: it detects difference, not tampering.
 function C.digest64(text,salt) return ('%08x%08x'):format(fnv32(text),fnv32(text,fnv32(salt or 'envoy:'))) end
 -- The one formula every roll derives from (run_host.lua, coop.lua and drive_loot.lua carry the same; the tests pin that they agree).
 function C.seed_for(seed,stage,loop,k) return (seed+stage*104729+loop*15485863+k*32452843)%2147483646+1 end
 -- A deterministic [0,1) stream (Park-Miller, warmed so adjacent seeds do not share a first draw): what a hosted run uses instead of math.random.
 function C.rng(seed) local state=seed%2147483646+1;local function nxt(n) state=state*16807%2147483647;return (state-1)/2147483646*(n or 1) end;for _=1,6 do nxt() end;return nxt end
 -- The ONLY per-peer random draw of a run: where the run seed comes from when nobody supplied one. Offline it is math.random; online
 -- it is refused, because the lobby host chooses the seed and both peers receive it.
 function C.fresh_seed(g,random)
  local m=g and g.match and g.match()
  assert(not (m and m.netplay),'online run seeds come from the lobby host, never a local draw')
  return (random or math.random)(0,2147483646)
 end
 -- Tier of one record in a build: a level 1..9, or a stack table. Canonical text: "2", "2x3" (level 2, three copies) or "t1.2.2" (an explicit list).
 local function tier_text(t)
  if type(t)=='number' then assert(t%1==0 and t>=1 and t<=9,'tier out of range');return tostring(t) end
  assert(type(t)=='table' and not getmetatable(t),'tier must be a level or a stack')
  if t.tiers then local o={};for i,x in ipairs(t.tiers) do assert(type(x)=='number' and x%1==0 and x>=1 and x<=9,'tier out of range');o[i]=tostring(x) end;assert(#o>=1 and #o<=6,'stack size');return 't'..concat(o,'.') end
  assert(type(t.tier)=='number' and t.tier%1==0 and t.tier>=1 and t.tier<=9,'tier out of range')
  local copies=t.copies or 1;assert(copies%1==0 and copies>=1 and copies<=6,'copies out of range')
  return copies==1 and tostring(t.tier) or (t.tier..'x'..copies)
 end
 local function tier_parse(s)
  if s:match('^[1-9]$') then return tonumber(s) end
  local lv,n=s:match('^([1-9])x([1-6])$');if lv then return {tier=tonumber(lv),copies=tonumber(n)} end
  local list=s:match('^t([1-9%.]+)$');if list then local out={};for d in list:gmatch('[1-9]') do out[#out+1]=tonumber(d) end;assert(#out>=1 and #out<=6 and concat(out,'.')==list,'bad tier list');return {tiers=out} end
  error('bad tier text '..s)
 end
 C.tier_text,C.tier_parse=tier_text,tier_parse
 local function sorted_keys(t) local k={};for key in pairs(t or {}) do k[#k+1]=key end;sort(k);return k end
 -- A digest of the rules themselves (ids, kinds, tiers, triggers, conditions, effects: not the wording or the look), so a peer with a
 -- different pool is refused rather than silently computing different numbers.
 function C.pool_digest(list)
  list=list or (D and D.mod_pool);assert(type(list)=='table','no pool to digest')
  local rows={}
  for _,m in ipairs(list) do rows[#rows+1]={id=m.id,kind=m.kind,tiers=m.tiers,trigger=m.trigger,conditions=m.conditions,effects=m.effects,interval=m.interval or 0,cost=m.cost or ''} end
  sort(rows,function(a,b) return a.id<b.id end)
  local parts={};for i,r in ipairs(rows) do parts[i]=C.encode(r) end
  return C.digest64(concat(parts,'\0'),'pool:')
 end
 -- The build record of ONE player: the modifiers (sorted by id, with tier), the colour implicits (sorted by key) and the run context.
 -- meta = {seed=,game=,loop=,port=} (integers). Text form (one line, ASCII, no separators inside a field):
 --   EB1|<seed>|<game>|<loop>|<port>|<pool digest>|<id>=<tier>,<id>=<tier>...|<key>=<number>,...|<keystone ids>|<digest of everything before it>
 function C.build_record(meta,equipped,implicits,pool)
  meta=meta or {}
  for _,k in ipairs({'seed','game','loop','port'}) do assert(type(meta[k])=='number' and meta[k]%1==0 and meta[k]>=0 and meta[k]<=2147483647,'record '..k..' must be an integer') end
  local mods,ks={},{}
  local list=pool or (D and D.mod_pool);local kind={};for _,m in ipairs(list or {}) do kind[m.id]=m.kind end
  for _,id in ipairs(sorted_keys(equipped)) do
   assert(type(id)=='string' and id:match('^[%l%d_]+$'),'bad modifier id');assert(not list or kind[id],'unknown modifier '..id)
   mods[#mods+1]=id..'='..tier_text(equipped[id]);if kind[id]=='keystone' then ks[#ks+1]=id end
  end
  local imps={}
  for _,key in ipairs(sorted_keys(implicits)) do
   local v=implicits[key];assert(type(key)=='string' and key:match('^[%l%d_]+$') and type(v)=='number' and v==v and math.abs(v)<1000,'bad implicit')
   imps[#imps+1]=key..'='..format('%.17g',v)
  end
  local body=concat({'EB'..C.record_version,format('%d',meta.seed),format('%d',meta.game),format('%d',meta.loop),format('%d',meta.port),C.pool_digest(list),concat(mods,','),concat(imps,','),concat(ks,',')},'|')
  local digest=C.digest64(body,'build:')
  return body..'|'..digest,digest
 end
 -- Strict parse. Returns {meta=,equipped=,implicits=,keystones=,pool=,digest=} or nil,reason. Refuses a wrong version, a bad digest, a keystone list that
 -- is not exactly the keystones among the modifiers (when a pool is known), unsorted or duplicate fields.
 function C.parse_build_record(text,pool)
  if type(text)~='string' or #text>400 or text:find('[^%g]') then return nil,'record is not compact ASCII' end
  local f={};for field in (text..'|'):gmatch('([^|]*)|') do f[#f+1]=field end
  if #f~=10 then return nil,'record has '..#f..' fields, expected 10' end
  if f[1]~='EB'..C.record_version then return nil,'record version '..f[1]..' is not supported (this build reads EB'..C.record_version..')' end
  local body=concat(f,'|',1,9);if C.digest64(body,'build:')~=f[10] then return nil,'record digest does not match its contents' end
  local meta={};for i,k in ipairs({'seed','game','loop','port'}) do local v=math.tointeger(tonumber(f[i+1]));if not v or v<0 or v>2147483647 or tostring(v)~=f[i+1] then return nil,'bad '..k end;meta[k]=v end
  if not f[6]:match('^%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x%x$') then return nil,'bad pool digest' end
  local eq,last={},nil
  for item in f[7]:gmatch('[^,]+') do
   local id,t=item:match('^([%l%d_]+)=(.+)$');if not id then return nil,'bad modifier entry' end
   if last and id<=last then return nil,'modifiers not strictly sorted' end;last=id
   local ok,tier=pcall(tier_parse,t);if not ok then return nil,'bad tier for '..id end;eq[id]=tier
  end
  local imp;last=nil
  for item in f[8]:gmatch('[^,]+') do
   local key,v=item:match('^([%l%d_]+)=(.+)$');v=v and tonumber(v);if not key or not v then return nil,'bad implicit entry' end
   if last and key<=last then return nil,'implicits not strictly sorted' end;last=key;imp=imp or {};imp[key]=v
  end
  local keys={};last=nil
  for id in f[9]:gmatch('[^,]+') do if last and id<=last then return nil,'keystones not sorted' end;last=id;if not eq[id] then return nil,'keystone '..id..' is not in the build' end;keys[#keys+1]=id end
  local list=pool or (D and D.mod_pool)
  if list then
   local kind={};for _,m in ipairs(list) do kind[m.id]=m.kind end
   for id in pairs(eq) do if not kind[id] then return nil,'unknown modifier '..id end end
   local want={};for id in pairs(eq) do if kind[id]=='keystone' then want[#want+1]=id end end;sort(want)
   if concat(want,',')~=concat(keys,',') then return nil,'keystone list does not match the modifiers' end
   if C.pool_digest(list)~=f[6] then return nil,'the record was made with a different modifier pool' end
  end
  return {meta=meta,equipped=eq,implicits=imp or {},keystones=keys,pool=f[6],digest=f[10]}
 end

 return C
end
