local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();local V=T.module('drives',D)
local function fixture()
 local s={serial=0,items={},fx={},sounds={},ended=0,controls={},moves={}}
 local function id() s.serial=s.serial+1;return s.serial end
 local g={log=function() end,player=function() return {x=20,y=10} end,
 item_spawn=function(_,x,y,o) local h=id();s.items[h]={handle=h,x=x,y=y,z=0,visual_y=y+6,rotation=90,visible=true,age=0,payload=o.payload};return h end,
 item_despawn=function(h) s.items[h]=nil;return true end,
 items=function() local rows={};for _,v in pairs(s.items) do rows[#rows+1]=v end;return rows end,
 fx_world=function(name,x,y,z) local h=id();s.fx[h]={name=name,x=x,y=y,z=z};return h end,
 fx_move=function(h,x,y,z) assert(s.fx[h]);s.moves[h]={x=x,y=y,z=z};return true end,
 fx_control=function(h,_,o) s.controls[h]=o;return true end,
 fx_end=function(h) assert(s.fx[h]);s.fx[h]=nil;s.ended=s.ended+1;return true end,
 play_sound=function(i,o) s.sounds[#s.sounds+1]={id=i,pitch=o.pitch} end}
 return s,V.new(function() return .1 end,g.log,g),D.companion.new(),g
end
local function drop(v,c,h,kind)
 v:track(h,{kind=kind,x=5,y=0});assert(v:defeat({handle=h,kind=kind},c));return next(v.native)
end
local function collect(s,v,c,h)
 local p=s.items[h].payload;assert(v:collect({name='drive',port=1,item=h,payload=p},c));s.items[h]=nil
end
T.test('all shipped polish features enabled and resting effects end on collect',function()
 local s,v,c=fixture();local h=drop(v,c,1,'goomba');local d=v.juice.drops[h]
 for _,kind in ipairs({'glow','pool','sparkles','highlight','pop_trail'}) do assert(d.fx[kind],kind..' disabled') end
 v:tick(nil,c);assert(s.moves[d.fx.pool].y==.15 and s.moves[d.fx.glow].y==11.6875)
 assert(math.abs(s.moves[d.fx.highlight].z-3)<1e-9)
 collect(s,v,c,h);assert(not v.juice.drops[h] and v.juice.flash.red==24 and #v.juice.transients==1)
 assert(s.ended==5);for _=1,24 do v:tick(nil,c) end
 assert(not next(s.fx) and not next(v.juice.flash) and #v.juice.transients==0)
end)
T.test('per colour and white sound identity with cross-colour pitch streak',function()
 local s,v,c=fixture();local h=drop(v,c,1,'goomba');collect(s,v,c,h);assert(s.sounds[#s.sounds].pitch==-100)
 h=drop(v,c,2,'koopa');collect(s,v,c,h);assert(s.sounds[#s.sounds].pitch==70)
 h=drop(v,c,3,'topi');collect(s,v,c,h);assert(s.sounds[#s.sounds].pitch==340)
 h=drop(v,c,4,'octorok');collect(s,v,c,h);assert(s.sounds[#s.sounds].pitch==310)
 v.rng=function() return 0 end;h=drop(v,c,5,'goomba');collect(s,v,c,h)
 assert(s.sounds[#s.sounds].id==250 and s.sounds[#s.sounds].pitch==580)
 for _=1,91 do v:tick(nil,c) end
 v.rng=function() return .1 end;h=drop(v,c,6,'goomba');collect(s,v,c,h);assert(s.sounds[#s.sounds].pitch==-100)
 v:clear();assert(not next(s.fx))
end)
T.test('expire clear missing native row and absent engines never leak',function()
 local s,v,c=fixture();local h=drop(v,c,1,'goomba');v:expire{name='drive',item=h};s.items[h]=nil
 assert(not next(s.fx) and not next(v.native))
 h=drop(v,c,2,'koopa');v:tick(nil,c);s.items[h]=nil;v:tick(nil,c);assert(not next(s.fx))
 drop(v,c,3,'topi');v:clear();assert(not next(s.fx) and not next(s.items))
 local bare=V.new(function() return .1 end,nil,{item_spawn=function() return 42 end})
 drop(bare,c,1,'goomba');assert(bare:collect({name='drive',port=1,item=42,payload={colour='red',amount=20}},c));bare:clear()
end)
T.test('every presentation switch can disable independently',function()
 for _,key in ipairs({'glow','pool','sparkles','highlight','pop_trail','collect_burst','sound','hud_flash','blink'}) do
  local s,v,c,g=fixture();v.juice=D.pickup_juice.new(g,{[key]=false})
  local h=drop(v,c,1,'goomba');local d=v.juice.drops[h]
  if d.fx[key]~=nil then error(key..' remains enabled') end
  s.items[h].visible=false;v:tick(nil,c)
  if key=='blink' then assert(s.controls[d.fx.glow].opacity==1) end
  collect(s,v,c,h)
  if key=='collect_burst' then assert(#v.juice.transients==0) end
  if key=='sound' then assert(#s.sounds==0) end
  if key=='hud_flash' then assert(not next(v.juice.flash)) end
  v:clear();assert(not next(s.fx))
 end
end)
T.test('native collection radius stays fixed and never calls the withdrawn adapter',function()
 local s,v,c,g=fixture();g.item_set_radius=function() error('withdrawn API must not be called') end
 local h=drop(v,c,1,'goomba');D.companion.feed(c,'yellow',2000)
 v:tick(nil,c);assert(D.companion.effects(c).pickup_radius==17.5)
 assert(v.radius==nil and c.stats.power.points==0);collect(s,v,c,h);v:clear()
 assert(not next(s.fx))
end)
T.test('partial FX engines cannot create effects without teardown support',function()
 for _,name in ipairs({'fx_world','fx_move','fx_control','fx_end'}) do
  local s,_,c,g=fixture();g[name]=nil
  local v=V.new(function() return .1 end,g.log,g);drop(v,c,1,'goomba')
  assert(not next(s.fx),'missing '..name..' must disable unsafe FX')
  local h=next(v.native);collect(s,v,c,h);assert(c.stats.power.points==20 and #s.sounds==2)
  v:clear();assert(not next(s.fx))
 end
end)
T.done()
