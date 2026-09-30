-- Pure, deterministic roguelite rules. Bundled before main.lua; no engine access.
local C = {version = 1, capacity = 3}
local slots = {assault=true, traversal=true, guard=true}
local limits = {potency={1,30},capacity={1,12},gain={0.1,4},reach={1,30},cooldown={1,600}}
C.definitions = {
 cinder={version=1,name='Cinder Drive',family='fire',recipe='SolarEruption',
  variants={assault={name='Cinder Eruption',trigger='direct_hit',action='eruption',category='Magic'},
   traversal={name='Cinder Step',trigger='move',action='step',category='Special'},
   guard={name='Cinder Counter',trigger='defend',action='counter',category='Special'}}},
 rime={version=1,name='Rime Guard',family='frost',recipe='GlacialShatter',
  variants={assault={name='Rime Strike',trigger='direct_hit',action='mark',category='Magic'},
   traversal={name='Rime Glide',trigger='move',action='glide',category='Special'},
   guard={name='Rime Response',trigger='defend',action='mark',category='Magic'}}}
}
local function finite(x) return type(x)=='number' and x==x and math.abs(x)<math.huge end
local function integer(x,lo,hi) return finite(x) and x%1==0 and x>=lo and x<=hi end
local function valid_name(x) return type(x)=='string' and #x>0 and #x<=96 and not x:find('[%z\1-\31]') end
local function copy(t)
 if type(t)~='table' then return t end
 local out={} for k,v in pairs(t) do out[k]=copy(v) end return out
end
local function sorted(t) local a={} for k in pairs(t) do a[#a+1]=k end table.sort(a) return a end
local function seed(x) return integer(x,1,2147483646) end
local function next_seed(x) return (x*48271)%2147483647 end
local function gene(id,kind,s)
 return {id=id,kind=kind,version=1,seed=s,base={potency=8,capacity=3,gain=1,reach=9,cooldown=90},upgrades={},parents={},locks={}}
end
function C.new_profile(s)
 assert(seed(s),'invalid seed')
 local genes={g1=gene('g1','cinder',s),g2=gene('g2','rime',next_seed(s)),g3=gene('g3','cinder',next_seed(next_seed(s)))}
 genes.g1.base.potency=10;genes.g1.base.reach=7
 genes.g3.base.potency=6;genes.g3.base.reach=13;genes.g3.base.cooldown=60
  -- world_seed advances per run so successive runs do not repeat the dungeon;
  -- it is separate from the gene stream seed. Legacy profiles may omit it.
  return {type='profile',id='p'..s,version=1,seed=s,world_seed=next_seed(s),next_id=4,next_run=1,genes=genes,finished={}}
end
function C.new_run(p,opts)
 opts=opts or {}; assert(p.type=='profile' and p.version==1,'invalid profile')
 assert(#sorted(p.finished)<512,'finish ledger full; cannot start another run')
 assert(p.next_run<1000000000,'run counter exhausted')
 -- Advance the profile's world stream deliberately; an explicit opts.seed still
 -- reproduces that exact world without depending on the counter.
 local world=next_seed(p.world_seed or p.seed); p.world_seed=world
 local s=opts.seed or world; assert(seed(s),'invalid seed')
 local stocks=opts.stocks or 3; assert(integer(stocks,1,99),'invalid stocks')
 local id='run'..p.next_run; p.next_run=p.next_run+1
 local r={type='run',version=1,id=id,owner=p.id,seed=s,world_seed=s,next_id=1,frame=0,stocks=stocks,status='active',genes={},hosts={},marks={},runtime={},progress={room='entry',cleared={},claimed={},supplies=2}}
 for _,k in ipairs(sorted(p.genes)) do
  local g=copy(p.genes[k]); g.id='r'..r.next_id; g.origin=k; r.next_id=r.next_id+1; r.genes[g.id]=g
 end
 local starter={};for id,g in pairs(r.genes) do starter[g.origin]=id end
 C.equip(r,'player','assault',starter.g1); C.equip(r,'player','guard',starter.g2)
 return r
end
local function active(r) return type(r)=='table' and r.type=='run' and r.status=='active' end
function C.equip(r,host,slot,id)
 if not active(r) or not valid_name(host) or #host>64 or not slots[slot] then return nil,'invalid host or slot' end
 if id~=nil and (not r.genes[id] or not C.definitions[r.genes[id].kind].variants[slot]) then return nil,'unsupported gene placement' end
 if id then for h,v in pairs(r.hosts) do for s,x in pairs(v.slots) do
  if x==id and (h~=host or s~=slot) then return nil,'gene already equipped; unequip first' end
 end end end
 local h=r.hosts[host]; if not h then if #sorted(r.hosts)>=32 then return nil,'host capacity' end;h={slots={},state={},modifiers={}};r.hosts[host]=h end
 h.slots[slot]=id
 if id then r.runtime[id]=r.runtime[id] or {charge=0,ready_at=0,seen={}};h.state[slot]=r.runtime[id]
 else h.state[slot]={charge=0,ready_at=0,seen={}} end
 h.modifiers[slot]={}
 return true
end
function C.resolve(r,host,slot)
 local h=r.hosts[host]; local id=h and h.slots[slot]; local g=id and r.genes[id]
 if not g then return nil,'empty slot' end
 local out=copy(g.base)
 for stat,delta in pairs(g.upgrades) do out[stat]=out[stat]+delta end
 for _,m in pairs(h.modifiers[slot] or {}) do if not m.expires or m.expires>r.frame then out[m.stat]=out[m.stat]+m.add end end
 for k,b in pairs(limits) do out[k]=math.max(b[1],math.min(b[2],out[k])) end
 out.cooldown=math.floor(out.cooldown)
 return out
end
function C.apply_modifier(r,host,slot,m)
 if not active(r) or not C.resolve(r,host,slot) or type(m)~='table' or not valid_name(m.id) or #m.id>64 or not limits[m.stat] or not finite(m.add) or math.abs(m.add)>600 or (m.expires~=nil and not integer(m.expires,r.frame+1,1000000000)) then return nil,'invalid modifier' end
 for k in pairs(m) do if k~='id' and k~='stat' and k~='add' and k~='expires' then return nil,'unknown modifier field' end end
 if not r.hosts[host].modifiers[slot][m.id] and #sorted(r.hosts[host].modifiers[slot])>=128 then return nil,'modifier capacity' end
 r.hosts[host].modifiers[slot][m.id]=copy(m); return true
end
function C.remove_modifier(r,host,slot,id)
 local h=r.hosts[host]; if not active(r) or not h or not h.modifiers[slot] then return nil,'invalid slot' end
 h.modifiers[slot][id]=nil; return true
end
function C.tick(r,n)
 if not active(r) or not integer(n,0,36000) or r.frame+n>1000000000 then return nil,'invalid tick' end
 r.frame=r.frame+n
 for host,h in pairs(r.hosts) do for slot,st in pairs(h.state) do
  for id,m in pairs(h.modifiers[slot]) do if m.expires and m.expires<=r.frame then h.modifiers[slot][id]=nil end end
  local p=C.resolve(r,host,slot); if p then st.charge=math.min(st.charge,p.capacity) end
  -- Move IDs expire after 600 frames; callers must supply genuine unique move instances.
  for id,f in pairs(st.seen) do if r.frame-f>600 then st.seen[id]=nil end end
 end end
 for id,m in pairs(r.marks) do if m.expires<=r.frame then r.marks[id]=nil end end
 return true
end
function C.ability(r,host,slot)
 local p,err=C.resolve(r,host,slot); if not p then return nil,err end
 local h=r.hosts[host]; local g=r.genes[h.slots[slot]]; local d=C.definitions[g.kind]; local v=d.variants[slot]; local st=h.state[slot]
 return {name=v.name,family=d.family,slot=slot,category=v.category,action=v.action,recipe=d.recipe,
  cost=p.capacity,cooldown=p.cooldown,reach=p.reach,damage=p.potency,knockback={angle=45,kbg=70,bkb=20},
  charge=math.min(st.charge,p.capacity),ready=st.charge>=p.capacity and r.frame>=st.ready_at,remaining=math.max(0,st.ready_at-r.frame),trigger=v.trigger}
end
function C.on_event(r,e)
 if not active(r) or type(e)~='table' or not valid_name(e.host) or not valid_name(e.move_id) then return nil,'invalid event' end
 if e.reaction or (e.lineage and e.lineage~='direct') then return 0 end
 local h=r.hosts[e.host]; if not h then return 0 end
 for slot in pairs(h.slots) do
  local a=C.ability(r,e.host,slot);local st=h.state[slot]
  if a.trigger==e.kind and not st.seen[e.move_id] and #sorted(st.seen)>=512 then return nil,'event history capacity' end
 end
 local gained=0
 for _,slot in ipairs(sorted(h.slots)) do
  local a=C.ability(r,e.host,slot); local st=h.state[slot]
  if a.trigger==e.kind and not st.seen[e.move_id] then
   if #sorted(st.seen)>=512 then return nil,'event history capacity' end
   st.seen[e.move_id]=r.frame
   if r.frame>=st.ready_at then local p=C.resolve(r,e.host,slot);local before=math.min(st.charge,p.capacity);st.charge=math.min(p.capacity,before+p.gain);gained=gained+st.charge-before end
  end
 end
 return gained
end
function C.activate(r,host,slot,opts)
 if not active(r) then return nil,'run ended' end
 local a,err=C.ability(r,host,slot); if not a then return nil,err end
 if not a.ready then return nil,'not ready' end
 opts=opts or {}; if opts.target~=nil and (not valid_name(opts.target) or #opts.target>64) then return nil,'invalid target' end
 if a.action=='mark' and not opts.target then return nil,'target required' end
 local st=r.hosts[host].state[slot];st.charge=0;st.ready_at=r.frame+a.cooldown
 a.source=host;a.target=opts.target;a.lineage='reaction';a.charge=nil;a.ready=nil
 if a.action=='mark' then r.marks[opts.target]={source=host,expires=r.frame+180}
 elseif a.family=='fire' and opts.target and r.marks[opts.target] then
  r.marks[opts.target]=nil;a.reaction='thermal_shock';a.damage=math.min(30,a.damage+4)
 end
 return a
end
function C.reward(r,id,stat,delta)
 local g=r.genes[id]
 if not active(r) or not g or not limits[stat] or not finite(delta) or math.abs(delta)>600 then return nil,'invalid reward' end
 local b=limits[stat];local before=g.base[stat]+(g.upgrades[stat] or 0)
 g.upgrades[stat]=math.max(b[1],math.min(b[2],before+delta))-g.base[stat];return true
end
function C.acquire(r,kind)
 if not active(r) or not C.definitions[kind] or #sorted(r.genes)>=128 then return nil,'invalid acquisition' end
 local id='r'..r.next_id;local s=next_seed(r.seed);r.genes[id]=gene(id,kind,s);r.seed=s;r.next_id=r.next_id+1;return id
end
local function offspring(a,b,id,s)
 local g=gene(id,a.kind,s);g.parents={a.id,b.id}
 for _,k in ipairs(sorted(limits)) do
  if a.locks[k] and b.locks[k] and a.base[k]~=b.base[k] then return nil,'conflicting locked traits' end
  s=next_seed(s);g.base[k]=(a.locks[k] and a.base[k]) or (b.locks[k] and b.base[k]) or (s%2==0 and a.base[k] or b.base[k]);if a.locks[k] or b.locks[k] then g.locks[k]=true end
 end
 return g
end
function C.lock(container,id,stat,enabled)
 local g=container.genes[id];if not g or not limits[stat] or type(enabled)~='boolean' or (container.type=='run' and not active(container)) then return nil,'invalid lock' end
 g.locks[stat]=enabled or nil;return true
end
function C.breed(p,a,b)
 if p.type~='profile' or a==b or not p.genes[a] or not p.genes[b] or p.genes[a].kind~=p.genes[b].kind then return nil,'incompatible parents' end
 if #sorted(p.genes)>=128 then return nil,'collection full' end
 local id='g'..p.next_id;local s=next_seed(p.seed);local g,err=offspring(p.genes[a],p.genes[b],id,s);if not g then return nil,err end
 p.seed=s;p.next_id=p.next_id+1;p.genes[id]=g;return id
end
function C.fuse(r,a,b)
 if not active(r) or a==b or not r.genes[a] or not r.genes[b] or r.genes[a].kind~=r.genes[b].kind then return nil,'incompatible fusion' end
 for _,h in pairs(r.hosts) do for _,id in pairs(h.slots) do if id==a or id==b then return nil,'unequip fusion parents first' end end end
 local id='r'..r.next_id;local s=next_seed(r.seed);local g,err=offspring(r.genes[a],r.genes[b],id,s);if not g then return nil,err end
 -- Run improvements remain run improvements, bounded independently of inherited base.
 for stat,bounds in pairs(limits) do local delta=((r.genes[a].upgrades[stat] or 0)+(r.genes[b].upgrades[stat] or 0))/2;g.upgrades[stat]=math.max(bounds[1],math.min(bounds[2],g.base[stat]+delta))-g.base[stat] end
 r.seed=s;r.next_id=r.next_id+1;r.genes[a]=nil;r.genes[b]=nil;r.runtime[a]=nil;r.runtime[b]=nil;r.genes[id]=g;return id
end
function C.finish(p,r,outcome,id)
 local serial=r.id and r.id:match('^run(%d+)$')
 if r.owner~=p.id or not serial or tonumber(serial)>=p.next_run then return nil,'run belongs to another profile' end
 if p.finished[r.id] then return copy(p.finished[r.id]) end
 if not active(r) or (outcome~='success' and outcome~='failure') then return nil,'invalid finish' end
 if #sorted(p.finished)>=512 then return nil,'finish ledger full' end
 if outcome=='success' and id~=nil and (not r.genes[id] or #sorted(p.genes)>=128) then return nil,'invalid export or collection full; choose no export' end
 if outcome=='success' and id~=nil then
  for host,h in pairs(r.hosts) do if host~='player' then for _,owned in pairs(h.slots) do
   if owned==id then return nil,'enemy-owned genes cannot be exported' end
  end end end
 end
 local result={outcome=outcome}
 if outcome=='success' and id~=nil then local g=copy(r.genes[id]);local nid='g'..p.next_id;g.id=nid;g.upgrades={};g.origin=nil;p.genes[nid]=g;p.next_id=p.next_id+1;result.export=nid end
 r.status=outcome;p.finished[r.id]=copy(result);return result
end

-- Bounded JSON data codec. Objects only: no executable Lua, functions or metatables.
local function quote(s)
 return '"'..s:gsub('[%z\1-\31\\"]',function(c) return string.format('\\u%04x',c:byte()) end)..'"'
end
local function encode(v,depth)
 assert(depth<=16,'too deep')
 if type(v)=='string' then assert(#v<=256,'string too long');return quote(v)
 elseif type(v)=='number' then assert(finite(v),'nonfinite');return string.format('%.17g',v)
 elseif type(v)=='boolean' then return tostring(v)
 elseif type(v)=='table' then
  assert(not getmetatable(v),'metatable');local keys={};for k in pairs(v) do assert(type(k)=='string' or (integer(k,1,16)),'bad key');keys[#keys+1]=tostring(k) end;table.sort(keys)
  assert(#keys<=512,'too many fields');local a={};for _,k in ipairs(keys) do local value=v[k];if value==nil then value=v[tonumber(k)] end;a[#a+1]=quote(k)..':'..encode(value,depth+1) end
  return '{'..table.concat(a,',')..'}'
 end
 error('unsupported value')
end
local function decode(text)
 assert(type(text)=='string' and #text<=262144,'invalid size');local pos,nodes=1,0
 local function ws() local _,b=text:find('^%s*',pos);pos=(b or pos-1)+1 end
 local function str()
  assert(text:sub(pos,pos)=='"','expected string');pos=pos+1;local a={}
  while pos<=#text do local ch=text:sub(pos,pos);pos=pos+1
   if ch=='"' then local s=table.concat(a);assert(#s<=256,'string too long');return s end
   if ch=='\\' then local esc=text:sub(pos,pos);pos=pos+1
    if esc=='u' then local hex=text:sub(pos,pos+3);assert(hex:match('^%x%x%x%x$'),'bad escape');local x=tonumber(hex,16);assert(x<=127,'unsupported unicode');a[#a+1]=string.char(x);pos=pos+4
    else local map={['"']='"',['\\']='\\',['/']='/',b='\b',f='\f',n='\n',r='\r',t='\t'};assert(map[esc],'bad escape');a[#a+1]=map[esc] end
   else assert(ch:byte()>=32,'control character');a[#a+1]=ch end
  end;error('unterminated string')
 end
 local parse
 parse=function(depth)
  nodes=nodes+1;assert(nodes<=20000 and depth<=16,'too complex');ws();local ch=text:sub(pos,pos)
  if ch=='{' then pos=pos+1;ws();local out={};local count=0;if text:sub(pos,pos)=='}' then pos=pos+1;return out end
   while true do ws();local k=str();assert(out[k]==nil,'duplicate key');ws();assert(text:sub(pos,pos)==':','expected colon');pos=pos+1;out[k]=parse(depth+1);count=count+1;assert(count<=512,'too many fields');ws();ch=text:sub(pos,pos);pos=pos+1;if ch=='}' then return out end;assert(ch==',','expected comma') end
  elseif ch=='"' then return str()
  elseif text:sub(pos,pos+3)=='true' then pos=pos+4;return true
  elseif text:sub(pos,pos+4)=='false' then pos=pos+5;return false
  else local token=text:match('^[%-0-9%.eE+]+',pos);assert(token,'expected value');local mantissa,exponent=token:match('^(.-)[eE]([+-]?%d+)$');mantissa=mantissa or token
   assert(not mantissa:find('[eE+]') and (mantissa:match('^-?%d+$') or mantissa:match('^-?%d+%.%d+$')),'bad number')
   local unsigned=mantissa:gsub('^-','');assert(not unsigned:match('^0%d'),'leading zero');local x=tonumber(token);assert(finite(x),'bad number');pos=pos+#token;return x end
 end
 local value=parse(0);ws();assert(pos>#text,'trailing input');return value
end
local function fields(t,allowed)
 assert(type(t)=='table','expected object');for k in pairs(t) do assert(allowed[k],'unknown field '..tostring(k)) end
end
local name=valid_name
local function validate_gene(g,id)
 fields(g,{id=true,kind=true,version=true,seed=true,base=true,upgrades=true,parents=true,origin=true,locks=true})
 assert(g.id==id and name(id) and C.definitions[g.kind] and g.version==1 and seed(g.seed),'invalid gene')
 fields(g.base,limits);fields(g.upgrades,limits)
 fields(g.locks,limits);for _,x in pairs(g.locks) do assert(x==true,'invalid lock') end
 for stat,b in pairs(limits) do assert(finite(g.base[stat]) and g.base[stat]>=b[1] and g.base[stat]<=b[2],'invalid base');local x=g.upgrades[stat];assert(x==nil or (finite(x) and math.abs(x)<=600 and g.base[stat]+x>=b[1] and g.base[stat]+x<=b[2]),'invalid upgrade') end
 assert(type(g.parents)=='table','invalid ancestry');local n=0;local parents={};for k,v in pairs(g.parents) do n=n+1;assert((k=='1' or k=='2' or k==1 or k==2) and name(v),'invalid parent');parents[tonumber(k)]=v end;assert(n==0 or (n==2 and parents[1] and parents[2]),'invalid ancestry');g.parents=parents
 assert(g.origin==nil or name(g.origin),'invalid origin')
end
local function validate(v)
 assert(type(v)=='table' and v.version==1 and seed(v.seed),'unsupported save')
 assert(integer(v.next_id,1,1000000000) and type(v.genes)=='table','invalid counters')
 local prefix=v.type=='profile' and 'g' or 'r';local count=0
 for id,g in pairs(v.genes) do count=count+1;validate_gene(g,id);local suffix=id:match('^'..prefix..'(%d+)$');assert(suffix and tonumber(suffix)<v.next_id,'invalid gene id') end;assert(count<=128,'too many genes')
 if v.type=='profile' then
  fields(v,{type=true,id=true,version=true,seed=true,world_seed=true,next_id=true,next_run=true,genes=true,finished=true});assert(name(v.id),'invalid profile identity')
  assert(v.world_seed==nil or seed(v.world_seed),'invalid profile world seed')
  assert(integer(v.next_run,1,1000000000) and type(v.finished)=='table','invalid profile')
  local n=0;for id,result in pairs(v.finished) do n=n+1;local serial=id:match('^run(%d+)$');assert(serial and tonumber(serial)<v.next_run,'invalid result id');fields(result,{outcome=true,export=true});assert(result.outcome=='success' or result.outcome=='failure','bad outcome');assert((result.outcome=='success' and (result.export==nil or v.genes[result.export])) or (result.outcome=='failure' and result.export==nil),'bad export') end;assert(n<=512,'ledger full')
 elseif v.type=='run' then
  fields(v,{type=true,version=true,seed=true,world_seed=true,id=true,owner=true,next_id=true,frame=true,stocks=true,status=true,genes=true,hosts=true,marks=true,runtime=true,progress=true});assert(name(v.owner) and seed(v.world_seed),'invalid run owner/world seed')
  fields(v.progress,{room=true,cleared=true,claimed=true,supplies=true})
  assert(name(v.progress.room) and #v.progress.room<=64 and integer(v.progress.supplies,0,9),'invalid progress')
  for _,dict in ipairs({v.progress.cleared,v.progress.claimed}) do assert(type(dict)=='table','invalid progress flags');local n=0;for k,x in pairs(dict) do n=n+1;assert(name(k) and #k<=64 and type(x)=='boolean','invalid progress flag') end;assert(n<=32,'too many progress flags') end
  assert(name(v.id) and integer(v.frame,0,1000000000) and integer(v.stocks,0,99) and (v.status=='active' or v.status=='success' or v.status=='failure'),'invalid run')
  assert(type(v.hosts)=='table' and type(v.marks)=='table' and type(v.runtime)=='table','invalid hosts');local equipped={};local hosts=0
  local function state(st)
   fields(st,{charge=true,ready_at=true,seen=true});assert(finite(st.charge) and st.charge>=0 and st.charge<=12 and integer(st.ready_at,0,1000000600) and type(st.seen)=='table','invalid state')
   local n=0;for id,f in pairs(st.seen) do n=n+1;assert(name(id) and integer(f,0,v.frame),'invalid event') end;assert(n<=512,'invalid event history')
  end
  for id,st in pairs(v.runtime) do assert(v.genes[id],'orphan runtime');state(st) end
  for host,h in pairs(v.hosts) do hosts=hosts+1;assert(name(host) and #host<=64,'invalid host');fields(h,{slots=true,state=true,modifiers=true});fields(h.slots,slots);fields(h.state,slots);fields(h.modifiers,slots)
   for _,st in pairs(h.state) do state(st) end
   for slot,id in pairs(h.slots) do assert(v.genes[id] and not equipped[id] and h.state[slot] and h.modifiers[slot] and v.runtime[id],'invalid equipped gene');assert(encode(h.state[slot],0)==encode(v.runtime[id],0),'inconsistent state');h.state[slot]=v.runtime[id];equipped[id]=true end
   for slot,mods in pairs(h.modifiers) do assert(type(mods)=='table','invalid modifiers');for id,m in pairs(mods) do fields(m,{id=true,stat=true,add=true,expires=true});assert(name(id) and m.id==id and limits[m.stat] and finite(m.add) and math.abs(m.add)<=600 and (m.expires==nil or integer(m.expires,0,1000000000)),'invalid modifier') end end
  end;assert(hosts<=32,'too many hosts')
  for target,m in pairs(v.marks) do assert(name(target),'invalid target');fields(m,{source=true,expires=true});assert(v.hosts[m.source] and integer(m.expires,0,1000000180),'invalid mark') end
 else error('unsupported save type') end
 return v
end
function C.snapshot(v)
 local ok,out=pcall(function() validate(copy(v));local text=encode(v,0);assert(#text<=262144,'save too large');return text end)
 if ok then return out end;return nil,tostring(out)
end
function C.restore(text)
 local ok,out=pcall(function() return validate(decode(text)) end)
 if ok then return out end;return nil,tostring(out)
end
return C
