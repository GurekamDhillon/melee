local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();local H=T.module('hud',D)
T.test('safe area panel at three aspects shows levels and progress',function()
  for _,w in ipairs({640,853,1140}) do
    local rects,texts={},{};local g={safe_area=function() return {x=20,y=5,w=w,h=480,right=20+w,bottom=485} end,
      fill=function(x,y,r,h,c) assert(x>=20 and x+r<=20+w and y>=5 and y+h<=485);rects[#rects+1]={x,y,r,h,c} end,
      text=function(x,y,t) texts[#texts+1]=t end}
    local c=D.companion.new();D.companion.feed(c,'red',150);H.draw(g,c,'running')
    assert(#rects>=9 and #texts>=5);assert(table.concat(texts,' '):find('Power L'..c.stats.power.level,1,true))
    assert(rects[1][1]>w-320)
  end
end)
T.test('kit text fits inside safe-area panel',function()
  local n=0;local g={safe_area=function() return {x=0,y=0,w=640,h=480,right=640,bottom=480} end,
    fill=function() end,text=function() error('kit should draw labels') end,
    kit={available=function() return true end,text=function(x,y,t,role,colour,align,opts)
      assert(opts.max_w==260 and align=='left');n=n+1
    end}}
  H.draw(g,D.companion.new(),'Collect drives, reach exit');assert(n==5)
end)
T.test('pickup glyph uses projection and skips invisible',function()
  local n=0;local g={project=function(x,y) return x,y,x>0 end,text=function() n=n+1 end}
  H.pickups(g,{{x=5,y=0,colour='red'},{x=-5,y=0,colour='blue'}});assert(n==1)
end)
T.test('nonlinear progress and level-up moment use companion rule',function()
 local localD={companion={stats={'power'},tuning={},progress=function(c,k) return .25,10,40 end}}
 local h=T.module('hud',localD);local rects,texts={},{}
 local g={safe_area=function() return {x=0,y=0,w=640,h=480,right=640} end,fill=function(x,y,w,h) rects[#rects+1]=w end,text=function(x,y,t) texts[#texts+1]=t end}
 h.draw(g,{stats={power={level=2,grade='B',points=70}}},'running',{power=true})
 assert(rects[3]==rects[2]*.25);local all=table.concat(texts,' ');assert(all:find('10/40',1,true) and all:find('LEVEL UP',1,true))
 h.draw(g,{stats={}},nil)
end)
T.test('pickup flash targets matching stat bar and rare white label',function()
 local c=D.companion.new();D.companion.feed(c,'white',1);local fills,texts={},{}
 local g={safe_area=function() return {x=0,y=0,w=853,h=480,right=853} end,
 fill=function(x,y,w,h,rgba) fills[#fills+1]={h=h,rgba=rgba} end,
 text=function(x,y,t,rgba) texts[#texts+1]={text=t,rgba=rgba} end}
 H.draw(g,c,'running',nil,{blue=24,white=24})
 local n=0;for _,f in ipairs(fills) do if f.h==7 and f.rgba==0xFFFFFF80 then n=n+1 end end;assert(n==1)
 assert(texts[#texts].rgba==0xEBD175FF)
 H.draw(g,c,'running',nil,{})
 assert(texts[#texts].rgba==0xFFFFFFFF)
end)
T.test('yellow bar uses the tuning-table Jump label',function()
 local c={stats={jump={level=1,grade='C',points=20}}};local out={}
 local localD={companion={stats={'jump'},tuning={stat_names={jump='Leap'}},progress=function() return .1,2,20 end}}
 local h=T.module('hud',localD);local g={safe_area=function() return {x=0,y=0,w=640,h=480,right=640} end,fill=function() end,text=function(x,y,t) out[#out+1]=t end}
 h.draw(g,c,'running');assert(table.concat(out,' '):find('Leap L1',1,true))
end)
T.done()
