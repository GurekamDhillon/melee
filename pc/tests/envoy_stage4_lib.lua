-- Online Envoy stage 4: the PARITY FIXTURE generator. Runs the REAL Lua modifier engine (mod_engine.lua, the reference) on scripted/random event
-- streams for two seats and writes everything the native evaluator (pc/gameworld/script_mods_core.h) must reproduce, frame by frame: the staged
-- program words, the sampled players, the events, then the expected statuses, damage, fx, recents and drop counter.
-- Used by envoy_stage4.lua (which also runs pc/tests/script_mods_core_test.c on the result when a C compiler is at hand).
local L={}
local function rng(seed) local s=seed%2147483647;if s<=0 then s=1 end;return function(n) s=s*48271%2147483647;return s%n end end
local function f9(x) return string.format('%.9g',x) end
function L.generate(D,opts)
 opts=opts or {}
 local E,S,N=D.mod_engine,D.mod_schema,D.mod_engine.native
 local out={'VER 1'}
 local cover,scenarios={},0
 local ids={};for _,m in ipairs(D.mod_pool) do if m.trigger~='equip' then ids[#ids+1]=m.id end end
 local passive={};for _,m in ipairs(D.mod_pool) do if m.trigger=='equip' and select(1,E.online_safe(m)) then passive[#passive+1]=m.id end end
 local skill_kinds={}
 for _,n in ipairs(N.event_names) do if D.mod_skill.is_skill(n) and n~='perfect_shield' then skill_kinds[#skill_kinds+1]=n end end
 local action_kinds={'landing','jump','air_jump','ledge_grab','grab','throw','taunt','shield_hit','perfect_shield'}
 local moves={'jab','dash_attack','tilt','smash','aerial','grab','throw','special','projectile'}
 local elements={'normal','fire','electric','ice','darkness'}
 local armor_limit={super=30,damage_threshold=300,knockback_threshold=300,hit_count=600,damage_pool=600}
 local native_armor={knockback=1,damage_threshold=2,knockback_threshold=3,super=4,hit_count=5,damage_pool=6}
 local function tagmask(t) local m=0;for k,v in pairs(t or {}) do if v then m=m|assert(N.tags[k],'tag '..k) end end;return m end
 local function ctxline(c)
  if not c then return '0 0 0 0 0 0 0 0 0 0 0' end
  return table.concat({c.percent~=nil and 1 or 0,c.grounded~=nil and 1 or 0,c.stocks~=nil and 1 or 0,c.air_frames~=nil and 1 or 0,c.aerial_hit~=nil and 1 or 0,
   f9(c.percent or 0),c.grounded and 1 or 0,c.stocks or 0,c.air_frames or 0,c.aerial_hit and 1 or 0,0},' ')
 end
 local function evline(e)
  local names=N.events
  local aer={nair=0,fair=1,bair=2,uair=3,dair=4};local dir={in_place=0,toward=1,away=2,wall=3,ceiling=4}
  return table.concat({'E',names[e.kind],e.port-1,e.target and e.target-1 or -1,tagmask(e.tags),e.status and N.status_ids[e.status] or 0,e.hit and 1 or 0,
   e.aerial and aer[e.aerial] or -1,e.direction and dir[e.direction] or -1,e.count or -1,e.damage and 1 or 0,f9(e.damage or 0),e.strength and 1 or 0,f9(e.strength or 0),
   e.absorbed and 1 or 0,e.broke and 1 or 0,e.damage_a and 1 or 0,f9(e.damage_a or 0),f9(e.damage_b or 0),
   e.self_context and 1 or 0,ctxline(e.self_context),e.target_context and 1 or 0,ctxline(e.target_context)},' ')
 end
 local function run_scenario(seed,builds,frames,focus)
  local r=rng(seed)
  local engine=E.new(seed,D.mod_pool)
  local programs={}
  for port=1,2 do
   engine:set_build(port,builds[port])
  end
  local any=false
  for port=1,2 do if engine:has_triggered(port) then any=true end end
  if not any then return false end
  for port=1,2 do local ok,p=pcall(engine.native_program,engine,port,port);if not ok then return false,p end;programs[port]=p end
  scenarios=scenarios+1
  local apply=engine.apply
  engine.apply=function(self,m,e,tier) cover[m.id]=(cover[m.id] or 0)+1;return apply(self,m,e,tier) end
  out[#out+1]='SCENARIO '..scenarios..' '..seed
  for port=1,2 do
   out[#out+1]='PROGRAM '..(port-1)..' '..#programs[port].words
   out[#out+1]=table.concat(programs[port].words,' ')
  end
  local pl={{percent=0,grounded=true,stocks=1+r(3),x=-20,y=0},{percent=0,grounded=true,stocks=1+r(3),x=20,y=0}}
  out[#out+1]='FRAMES '..frames
  local focus_events={}
  for _,id in ipairs(focus or {}) do local m=engine.rules[id];focus_events[#focus_events+1]=m.trigger;for _,a in ipairs(m.also or {}) do focus_events[#focus_events+1]=a.trigger end end
  for f=1,frames do
   out[#out+1]='F '..f
   local evs={}
   local function ctx(p) return {percent=pl[p].percent,grounded=pl[p].grounded,stocks=pl[p].stocks} end
   local function add(e) evs[#evs+1]=e end
   local function make(kind)
    local a=1+r(2);local v=3-a
    if kind=='hit_dealt' or kind=='hit_taken' then
     local tags={[moves[1+r(#moves)]]=true,[elements[1+r(#elements)]]=true}
     if r(2)==0 then tags[pl[a].grounded and 'grounded' or 'airborne']=true end
     add{kind='hit_dealt',port=a,target=v,tags=tags,self_context=ctx(a),target_context=ctx(v)}
     local taken={};for k,x in pairs(tags) do if k~='grounded' and k~='airborne' then taken[k]=x end end
     add{kind='hit_taken',port=v,target=a,tags=taken,self_context=ctx(v),target_context=ctx(a)}
    elseif kind=='ko_dealt' or kind=='stock_lost' then
     add{kind='ko_dealt',port=a,target=v,tags={}};add{kind='stock_lost',port=v,tags={}};if pl[v].stocks>1 then pl[v].stocks=pl[v].stocks-1 end;pl[v].percent=0
    elseif kind=='clank' then
     local da,db=r(20)*0.5,r(20)*0.5
     add{kind='clank',port=a,target=v,tags={},damage_a=da,damage_b=db};add{kind='clank',port=v,target=a,tags={},damage_a=da,damage_b=db}
    elseif kind=='crit' then add{kind='crit',port=a,target=v,tags={critical=true},strength=r(9)/8}
    elseif kind=='armor' then local ab=r(2)==0;add{kind='armor',port=a,tags={},absorbed=ab,broke=not ab}
    elseif D.mod_skill.is_skill(kind) and kind~='perfect_shield' then
     local own={stocks=pl[a].stocks,air_frames=r(40),aerial_hit=r(2)==0}
     local e={kind=kind,port=a,tags={},self_context=own,hit=r(2)==0}
     if kind=='lcancel' or kind=='lcancel_hit' or kind=='lcancel_miss' or kind=='auto_cancel' then e.aerial=({'nair','fair','bair','uair','dair'})[1+r(5)] end
     if kind=='tech' then e.direction=({'in_place','toward','away','wall','ceiling'})[1+r(5)] end
     if kind=='combo' or kind=='combo_end' then e.port=a;e.target=v;e.count=2+r(6);e.damage=r(60)*0.5 end
     add(e)
    else add{kind=kind,port=a,tags={}} end
   end
   if r(100)<8 then make('hit_dealt') end
   if r(100)<2 then make('ko_dealt') end
   if r(100)<2 then make('clank') end
   if r(100)<3 then make('crit') end
   if r(100)<3 then make('armor') end
   if r(100)<8 then make(action_kinds[1+r(#action_kinds)]) end
   if r(100)<8 then make(skill_kinds[1+r(#skill_kinds)]) end
   if #focus_events>0 and r(100)<20 then local k=focus_events[1+r(#focus_events)];if k~='interval' and k~='status_applied' and k~='status_removed' and k~='stacks_changed' then make(k) end end
   for p=1,2 do pl[p].percent=math.min(300,math.max(0,pl[p].percent+(r(5)==0 and r(12)*0.5 or 0)));if r(40)==0 then pl[p].grounded=not pl[p].grounded end;pl[p].x=pl[p].x+r(5)-2 end
   local players={};for p=1,2 do players[p]={percent=pl[p].percent,grounded=pl[p].grounded,stocks=pl[p].stocks,x=pl[p].x,y=pl[p].y} end
   for p=1,2 do out[#out+1]=table.concat({'P',p-1,1,f9(pl[p].percent),pl[p].grounded and 1 or 0,pl[p].stocks,f9(pl[p].x),f9(pl[p].y)},' ') end
   local ok,err=true,nil
   for _,e in ipairs(evs) do
    -- only what the engine accepts is sent to both sides (a full queue / too deep drops identically)
    out[#out+1]=evline(e);engine:emit(e)
   end
   engine:begin_frame(players);engine:drain()
   for p=1,2 do
    for i,name in ipairs(D.mod_status.order) do local v=(engine.statuses[p] or {})[name]
     if v then out[#out+1]=table.concat({'S',p-1,i,v.stacks,v.max,v.expires,v.next_tick,f9(v.amount),v.cause and D.mod_skill.cause[v.cause].index or 0},' ') end
    end
    out[#out+1]='D '..(p-1)..' '..f9(engine.damage[p] or 0)
    for ek,fr in pairs(engine.recent[p] or {}) do out[#out+1]=table.concat({'R',p-1,N.events[ek],fr},' ') end
   end
   for _,x in ipairs(engine.fx) do
    local line
    if x.op=='armor' then
     local ty=x.type;local frames_=math.max(1,math.min(armor_limit[ty] or 30,math.floor(x.frames or 1)))
     local v=ty=='super' and 1 or math.max(1,math.min(ty=='hit_count' and 3 or 40,x.value or 1));if ty=='hit_count' then v=math.floor(v) end
     line={'FX',1,x.port-1,native_armor[ty],frames_,(x.direction=='front' and 1 or x.direction=='back' and -1 or 0),0,0,0,f9(v)}
    elseif x.op=='intangible' then line={'FX',2,x.port-1,0,math.max(1,math.min(24,x.frames)),0,0,0,0,'0'}
    elseif x.op=='interrupt' then
     local mask=0x7FFF;if x.exits then mask=0;for i,n in ipairs({'jab','tilt','smash','aerial','special','grab','jump','dash','crouch','turn','walk','escape','shield','air_dodge','air_jump'}) do for _,y in ipairs(x.exits) do if y==n then mask=mask|(1<<(i-1)) end end end end
     line={'FX',3,x.port-1,0,math.max(1,math.min(20,x.frames)),0,0,mask,(x.guard and 1 or 0)|(x.restore_jumps and 2 or 0),'0'}
    elseif x.op=='crit_next' then line={'FX',4,x.port-1,0,0,0,math.max(1,math.min(3,x.count)),0,0,'0'}
    end
    out[#out+1]=table.concat(line,' ')
   end
   out[#out+1]='X '..engine.dropped
  end
  out[#out+1]='ENDSCENARIO'
  return true
 end
 -- a random build from the pool
 local function random_build(r,focus)
  local eq={}
  for _,id in ipairs(focus or {}) do eq[id]=1+r(3) end
  -- a focus build also holds the feeders of the statuses other records wait for (Momentum, Haste, Chill, Burn, Guarded)
  if focus then for _,id in ipairs({'updraft','rush','icebound','kindling','reprisal'}) do if r(2)==0 then eq[id]=1 end end end
  local n=2+r(4)
  for i=1,n do
   local id=r(4)==0 and passive[1+r(#passive)] or ids[1+r(#ids)]
   if id then eq[id]=1+r(3) end
  end
  return eq
 end
 local base=opts.seed or 7
 local tried,built=0,0
 -- one scenario per triggered record (it is equipped with random company), then random builds
 local plan={}
 for i,id in ipairs(ids) do plan[#plan+1]={focus={id}} end
 for i=1,(opts.random or 24) do plan[#plan+1]={} end
 for i,p in ipairs(plan) do
  local r=rng(base+i*7919)
  local done=false
  for attempt=1,6 do
   tried=tried+1
   local b1,b2=random_build(r,p.focus),random_build(r)
   local ok1=pcall(function() local e=E.new(1,D.mod_pool);e:set_build(1,b1);e:set_build(2,b2) end)
   if ok1 then
    local ok,why=run_scenario(base+i*131+attempt,{b1,b2},opts.frames or 700,p.focus)
    if ok then done=true;built=built+1;break end
   end
  end
 end
 return table.concat(out,'\n')..'\n',{scenarios=scenarios,cover=cover,ids=ids,tried=tried}
end
return L
