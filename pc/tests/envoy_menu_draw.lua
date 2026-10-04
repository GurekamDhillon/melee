local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
assert(io.open(T.root..'menu_draw.lua'),'menu drawing missing')
local D=T.rules();local V=T.module('menu_draw',D);local M=T.module('menu')
T.test('kit geometry stays in safe area at three widths with companion and results',function()
 for _,w in ipairs({640,853,1140}) do
  local texts={};local function rect(x,y,rw,rh) assert(x>=0 and y>=0 and x+rw<=w and y+rh<=480) end
  local g={safe_area=function() return {x=0,y=0,w=w,h=480} end,kit={panel=rect,list=function(x,y,rw,items,selected,o) assert(o.h==30,'native h must be row height');local count=math.min(o.visible,#items-(o.first or 1)+1);rect(x,y,rw,math.max(0,count-1)*o.pitch+o.h);assert(selected>=0) end,text=function(x,y,t,role,col,align,o) assert(x>=0 and x<=w and y<=480);texts[#texts+1]=t end}}
  local s=M.new();local c={companion={age=2,type='young',stats={power={grade='S',level=2,points=210}},dna={colour={'red','blue'}}},palette={{label='Body',rgba={r=255,g=0,b=0,a=255}}}}
  s:show('companion');V.draw(g,s,c);assert(table.concat(texts,' '):find('red / blue',1,true))
  s:results({drives={power=3},levels={power=1},boss='won'});V.draw(g,s,c);assert(table.concat(texts,' '):find('won',1,true))
  for _,screen in ipairs({'title','profile','hub','fighter','setup','playing','pause','confirm','records','quit'}) do s:show(screen);V.draw(g,s,c) end
 end
end)
T.test('profile refusal gives repair guidance',function()
 local out={};local g={safe_area=function() return {x=0,y=0,w=640,h=480} end,kit={panel=function() end,list=function() end,text=function(x,y,t) out[#out+1]=t end}}
 local s=M.new();s:show('profile');V.draw(g,s,{error='Invalid profile'});local all=table.concat(out,' ');assert(all:find('Invalid profile',1,true));assert(all:find('repair',1,true))
end)
T.test('records renders supplied history and scrolls to oldest rows',function()
 local out={};local g={safe_area=function() return {x=0,y=0,w=640,h=480} end,kit={panel=function() end,list=function() end,text=function(x,y,t) out[#out+1]=t end}}
 local s=M.new();s:show('records');local c={records={}}
 for i=1,14 do c.records[i]='Run '..i..' won' end
 V.draw(g,s,c);local all=table.concat(out,' ');assert(all:find('Run 1 won',1,true));assert(not all:find('Run 14 won',1,true))
 for _=1,4 do s:input('down',c) end;out={};V.draw(g,s,c);all=table.concat(out,' ')
 assert(all:find('Run 14 won',1,true));assert(not all:find('Run 1 won',1,true));assert(all:find('Up / Down scroll',1,true))
end)
T.test('growth snapshots evolution age passive and interlude are legible',function()
 local out={};local g={safe_area=function() return {x=11,y=7,w=853,h=480} end,kit={panel=function() end,list=function() end,text=function(x,y,t) out[#out+1]=t end}}
 local localD={companion={tuning={lifespan=6},progress=function() return .25,10,40 end,passive=function() return {name='Fleet Feet'} end}}
 local v=T.module('menu_draw',localD);local s=M.new();local c={companion={age=2,type='speed',stats={power={points=10,level=0}}},notice='Reincarnated as an egg',passive={name='Fleet Feet'}}
 s:show('companion');v.draw(g,s,c);local all=table.concat(out,' ');assert(all:find('4 runs remaining',1,true));assert(all:find('Fleet Feet',1,true));assert(all:find('10/40',1,true))
 out={};s:results({before={stats={power={grade='C',level=1,points=20}}},after={stats={power={grade='B',level=3,points=90}}},evolution={type='power',passive='Heavy Hands'}});v.draw(g,s,c);all=table.concat(out,' ')
 assert(all:find('C -> B',1,true));assert(all:find('L1 -> L3',1,true));assert(all:find('20 -> 90',1,true));assert(all:find('Heavy Hands',1,true))
 out={};s:interlude({theme='Copper Maze',drives={power=4},levels={power=2}});v.draw(g,s,c);all=table.concat(out,' ');assert(all:find('Copper Maze',1,true));assert(all:find('drives +4',1,true))
 out={};s:show('hub');v.draw(g,s,c);assert(table.concat(out,' '):find('Reincarnated as an egg',1,true))
end)
T.test('setup renders start refusal in the actual menu',function()
 local out={};local g={safe_area=function() return {x=0,y=0,w=640,h=480} end,kit={panel=function() end,list=function() end,text=function(x,y,t) out[#out+1]=t end}}
 local s=M.new();s:show('setup');V.draw(g,s,{notice='Run could not start: first room unavailable'})
 assert(table.concat(out,' '):find('first room unavailable',1,true))
end)
T.test('Jump labels and evolution follow the tuning table throughout menus',function()
 local out={};local g={safe_area=function() return {x=0,y=0,w=640,h=480} end,kit={panel=function() end,list=function() end,text=function(x,y,t) out[#out+1]=t end}}
 local localD={companion={tuning={lifespan=6,stat_names={jump='Leap'}},progress=function() return .1,2,20 end,passive=function() return {name='Skybound'} end}}
 local v=T.module('menu_draw',localD);local s=M.new();s:show('companion');v.draw(g,s,{companion={type='jump',stats={jump={level=1,points=20,grade='C'}},dna={jump={'B','C'}}}})
 local all=table.concat(out,' ');assert(all:find('Type Leap',1,true) and all:find('Leap  C  Lv 1',1,true) and all:find('Leap: B / C',1,true));assert(not all:find('Reach',1,true))
 out={};s:results({drives={jump=2},levels={jump=1},evolution={type='jump'}});v.draw(g,s,{});all=table.concat(out,' ');assert(all:find('Leap: drives +2',1,true) and all:find('EVOLVED: Leap',1,true) and all:find('Skybound',1,true))
end)
T.done()




