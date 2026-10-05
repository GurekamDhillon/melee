-- Canonical bounded data encoding. No load(), functions, metatables or wall clock.
return function()
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
 return C
end
