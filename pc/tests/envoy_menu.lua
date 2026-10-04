local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local path=T.root..'menu.lua'
assert(io.open(path),'menu state module missing')
local M=T.module('menu');local c={fighters={'fox','marth'}}
T.test('title profile hub and wrap disabled focus',function()
 local s=M.new();assert(s.screen=='title');s:input('accept',c);assert(s.screen=='profile');s:input('accept',c);assert(s.screen=='hub')
 s:input('up',c);assert(s.focus.hub==7);s:input('up',c);assert(s.focus.hub==4)
 s:input('accept',c);assert(s.screen=='records');s:input('back',c);assert(s.focus.hub==4)
 s.focus.hub=5;s:input('accept',c);assert(s.screen=='hub')
end)
T.test('fighter setup start effect and back',function()
 local s=M.new();s:show('hub');s.focus.hub=2;s:input('accept',c);assert(s.screen=='fighter');s:input('down',c)
 local e=s:input('accept',c);assert(e.type=='fighter' and e.fighter=='marth' and s.screen=='setup')
 e=s:input('accept',c);assert(e.type=='start' and e.fighter=='marth' and s.screen=='playing')
 e=s:input('start',c);assert(e.type=='pause' and s.screen=='pause')
 s:input('down',c);s:input('accept',c);assert(s.screen=='companion');s:input('back',c);assert(s.screen=='pause')
 s:input('down',c);s:input('accept',c);assert(s.screen=='confirm');s:input('accept',c);assert(s.screen=='pause')
 s.focus.pause=3;s:input('accept',c);s:input('down',c);e=s:input('accept',c);assert(e.type=='abandon' and s.screen=='hub')
end)
T.test('resume results back and empty fighter list',function()
 local s=M.new();s:show('playing');s:input('start',c);assert(s:input('back',c).type=='resume')
 s:results({boss='won'});assert(s.screen=='results' and s.result.boss=='won');s:input('accept',c);assert(s.screen=='hub')
 s:show('fighter');s:input('accept',{fighters={}});assert(s.screen=='fighter')
end)

T.test('results always returns to hub after pause companion',function()
 local s=M.new();s:show('pause');s:input('down');s:input('accept');s:input('back');s:results({});s:input('accept');assert(s.screen=='hub')
end)

T.test('profile refusal prevents continue',function()
 local s=M.new();s:show('profile');local c={error='Invalid profile'};assert(s:entries(c)[1].disabled);s:input('accept',c);assert(s.screen=='profile');s:input('back',c);assert(s.screen=='title')
end)
T.test('close from title and corrupt profile releases menu',function()
 local s=M.new();assert(s:input('back').type=='quit')
 s:show('title');s:input('down');assert(s:input('accept').type=='quit')
 s:show('profile');local c={error='corrupt'};s:input('down',c)
 assert(s:entries(c)[s.focus.profile].label=='Close Envoy');assert(s:input('accept',c).type=='quit')
end)
T.test('confirm screens always reset safe focus',function()
 local s=M.new();for _,screen in ipairs({'confirm','quit'}) do
  s:show(screen);s:input('down');assert(s.focus[screen]==2)
  s:input('back');s:show(screen);assert(s.focus[screen]==1)
 end
end)
T.test('all screens can return to gameplay or close through back',function()
 for _,screen in ipairs({'title','profile','hub','fighter','setup','pause','confirm','quit','companion','records','results'}) do
  local s=M.new();s:show(screen);local effect
  for _=1,6 do effect=s:input('back',{error='corrupt'});if effect then break end end
  assert(effect and (effect.type=='quit' or effect.type=='resume' or effect.type=='hub'),screen)
 end
end)
T.test('records lines scroll within available rows',function()
 local s=M.new();s:show('records');local c={records={}}
 for i=1,14 do c.records[i]='Run '..i end
 for _=1,20 do s:input('down',c) end;assert(s.scroll==4)
 s:input('up',c);assert(s.scroll==3)
end)
T.test('garden return and explicit interlude continue effects',function()
 local s=M.new();s:show('hub');assert(s:entries()[1].label=='Walk in garden');assert(s:input('accept').type=='hub')
 s:results({});assert(s:entries()[1].label=='Return to garden');assert(s:input('accept').type=='hub')
 s:results({});assert(s:input('back').type=='hub')
 s:interlude({theme='Maze'});assert(s.screen=='interlude');assert(s:input('back')==nil and s.screen=='interlude')
 assert(s:input('accept').type=='continue' and s.screen=='playing')
end)
T.done()


