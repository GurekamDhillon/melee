-- Caches added by the Envoy performance pass: each must give the same answer as the uncached path and
-- must invalidate when its inputs change (codec byte identity, pool validation, per-port memo).
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_budget','mod_engine','mod_pool'}) do D[n]=T.module(n,D) end
local C=D.mod_codec
-- reference encoder: the original per-node-string implementation
local function reference(value)
 local nodes,active=0,{}
 local function pack(v,depth)
  nodes=nodes+1;assert(nodes<=6000 and depth<=20)
  local kind=type(v)
  if kind=='boolean' then return v and 't' or 'f' end
  if kind=='number' then local s=string.format('%.17g',v);return 'd'..#s..':'..s end
  if kind=='string' then return 's'..#v..':'..v end
  active[v]=true
  local keys={};for k in pairs(v) do keys[#keys+1]=k end
  table.sort(keys,function(a,b) if type(a)~=type(b) then return type(a)=='number' end;return a<b end)
  local out={'{'..#keys..':'};for _,k in ipairs(keys) do out[#out+1]=pack(k,depth+1);out[#out+1]=pack(v[k],depth+1) end
  active[v]=nil;return table.concat(out)
 end
 return pack(value,0)
end
T.test('encoder output is byte-identical to the reference on random values',function()
 math.randomseed(7)
 local function rnd(d)
  local r=math.random(1,7)
  if d>3 or r<=3 then local k=math.random(1,5);if k==1 then return math.random()*100 elseif k==2 then return math.random(-5,50) elseif k==3 then return 'x'..math.random(1,99) elseif k==4 then return math.random()<.5 else return 0.0 end end
  local t={}
  if r==4 then for i=1,math.random(0,6) do t[i]=rnd(d+1) end
  elseif r==5 then for _=1,math.random(0,6) do t['k'..math.random(1,9)]=rnd(d+1) end
  else for _=1,math.random(0,5) do t[math.random(1,9)]=rnd(d+1) end;for _=1,2 do t['s'..math.random(1,4)]=rnd(d+1) end end
  return t
 end
 for i=1,2000 do local v=rnd(0);local a,b=reference(v),C.encode(v);assert(a==b,'mismatch '..i);assert(C.encode(C.decode(b))==b) end
end)
T.test('an engine rebuilt by import() does not revalidate the pool',function()
 local calls=0;local real=D.mod_schema.validate
 local e=D.mod_engine.new(104729,D.mod_pool) -- first engine validates the pool once
 D.mod_schema.validate=function(m) calls=calls+1;return real(m) end
 local probe=D.mod_engine.new(104729,D.mod_pool);probe:import(e:export());probe:import(probe:export())
 D.mod_schema.validate=real
 assert(calls==0,'import revalidated the pool '..calls)
end)
T.test('derived values memoise by content and invalidate on every kind of change',function()
 local e=D.mod_engine.new(104729,D.mod_pool)
 local ids={};for _,m in ipairs(e.list) do if m.kind=='normal' and #ids<2 then for _,x in ipairs(m.effects) do if x.op=='value' then ids[#ids+1]=m.id;break end end end end
 assert(#ids==2)
 local empty=e:values(1);assert(next(empty)==nil)
 e:equip(1,ids[1]);local one=e:values(1);assert(next(one))
 assert(e:values(1)~=one,'callers get their own copy');one.speed_marker=1;assert(e:values(1).speed_marker==nil)
 local budget=e:family_budget(1);assert(e:family_budget(1)==budget,'unchanged build reuses the budget')
 local rules=e:native_rules(1);assert(e:native_rules(1)==rules)
 e:equip(1,ids[2]);local two=e:values(1);local same=true;for k,v in pairs(one) do if two[k]~=v then same=false end end
 assert(e:family_budget(1)~=budget,'equip invalidates');assert(not same or next(two)~=nil)
 -- statuses: stacks/amount are part of the key, expiry is not
 local before=e:family_budget(1)
 e.statuses[1]={haste={stacks=1,max=1,amount=1,expires=100,next_tick=60,origin={}}}
 local hasted=e:values(1);assert(hasted.run_speed and hasted.run_speed>(e:values(2).run_speed or 1),'status changes the values')
 assert(e:family_budget(1)~=before)
 local cached=e:family_budget(1);e.statuses[1].haste.expires=999;assert(e:family_budget(1)==cached,'expiry does not enter the key')
 e.statuses[1]=nil;local back=e:values(1);assert(back.run_speed==nil or back.run_speed==two.run_speed)
 -- a content-equal clone engine derives identical values
 local p=D.mod_engine.new(104729,D.mod_pool);p:import(e:export())
 local a,b=p:values(1),e:values(1);for k,v in pairs(a) do assert(b[k]==v) end
end)
T.test('recolour ignores pose-weight jitter, re-measures on a slow clock, reapplies on identity or class change',function()
 local R=T.module('recolour');local c={colour='red',two_tone=true,type='speed'}
 local frame,refreshes=100,0;local p={kind=2,costume=0,stocks=4,falls=0,action=14}
 local parts={geometry_signature='one',{index=0,path=1,joint=2,source=0,status=0,regions={torso=1}},{index=1,path=2,joint=3,source=0,status=0,regions={left_arm=1}}}
 local clears,tints=0,0
 local g={player=function() return p end,frame=function() return frame end,
  parts=function(_,refresh) if refresh then refreshes=refreshes+1 end;return parts end,rgb=function(r,g,b) return {r,g,b} end,
  parts_clear=function() clears=clears+1 end,dobj_tint=function() tints=tints+1;return true end}
 local v=R.new(g);assert(v:apply(c));assert(tints==2 and refreshes==1)
 for i=1,40 do frame=frame+1;parts[1].regions={torso=.9-i*.001};parts[2].regions={left_arm=.7+i*.001};v:tick(c) end
 assert(clears==0 and tints==2,'weight jitter must not re-tint');assert(refreshes==1,'no re-measure inside the interval')
 frame=frame+R.REMEASURE;v:tick(c);assert(refreshes==2 and clears==0,'slow re-measure, same classes, no reapply')
 parts[1].regions={left_hand=1};frame=frame+R.REMEASURE;v:tick(c);assert(clears==1 and tints==4,'class change reapplies')
 p.stocks=3;frame=frame+1;v:tick(c);assert(clears==2,'identity change reapplies at once, inside the interval')
end)
T.test('bag screen derives its model once per input change, not per drawn frame',function()
 local M=T.module('drive_menu',{menu_input={new=function() return {} end}})
 local views,texts=0,0
 local owner={rev=1,pending={},bag={items={},equipped={},context={depth=0,loop=0}},lab={engine={frame=5}},loot={name=function() return 'x' end,tooltip=function() return {} end},
  view=function() views=views+1;return {slots=function() return 1 end,equipped={},items={},keystones={}} end,
  budget_lines=function() return {'Strength'} end,delta=function() return {} end}
 owner.lab.engine.list={}
 local kit={panel=function() end,text=function() texts=texts+1 end,list=function() end}
 local g={kit=kit,safe_area=function() return {x=0,y=0,w=800,h=600} end}
 local m=M.new(g,owner);m.active=true
 m:draw();local first=views;assert(first>0)
 for _=1,5 do m:draw() end;assert(views==first,'unchanged state must not re-derive')
 owner.rev=2;m:draw();assert(views>first,'a writer bumping rev invalidates')
 local second=views;m.focus=2;m:draw();assert(views>second,'focus change invalidates')
 assert(texts>0)
end)
T.done()
