-- Canonical bounded data encoding. No load(), functions, metatables or wall clock.
return function()
 local C={}
 function C.encode(value)
  local nodes,active=0,{}
  local function pack(v,depth)
   nodes=nodes+1;assert(nodes<=6000 and depth<=20,'modifier state too complex')
   local kind=type(v)
   if kind=='boolean' then return v and 't' or 'f' end
   if kind=='number' then assert(v==v and math.abs(v)<1e15,'non-finite state');local s=string.format('%.17g',v);return 'd'..#s..':'..s end
   if kind=='string' then return 's'..#v..':'..v end
   assert(kind=='table' and not getmetatable(v) and not active[v],'plain acyclic state required');active[v]=true
   local keys={};for k in pairs(v) do assert(type(k)=='string' or type(k)=='number' and k%1==0,'invalid state key');keys[#keys+1]=k end
   assert(#keys<=512,'checkpoint table too large')
   table.sort(keys,function(a,b) if type(a)~=type(b) then return type(a)=='number' end;return a<b end)
   local out={'{'..#keys..':'};for _,k in ipairs(keys) do out[#out+1]=pack(k,depth+1);out[#out+1]=pack(v[k],depth+1) end
   active[v]=nil;return table.concat(out)
  end
  local out=pack(value,0);assert(#out<=16384,'modifier checkpoint exceeds 16 KiB');return out
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
