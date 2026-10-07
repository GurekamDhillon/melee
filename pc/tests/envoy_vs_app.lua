-- Envoy VS builds, part 3: the `envoy vs ...` console route, and that nothing else in the `envoy` command moved.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();for _,k in ipairs({'save','drives','drive_models','fighter','campaign','hub','run','classic','hud','menu','menu_draw','menu_input','recolour','visual'}) do D[k]=T.module(k,D) end
D.retail_app=T.module('retail_app',D);D.app=T.module('app',D)
local function fixture()
 local s={text=D.save.encode(D.save.new_profile()),commands={},logs={},net=false}
 local g={command=function(n,f) s.commands[n]=f end,log=function(l) s.logs[#s.logs+1]=l end,data_read=function() return s.text end,
  data_write_atomic=function(_,v) s.text=v;return true end,match=function() return {active=true,netplay=s.net} end,player=function() return {} end,
  pad=function() return {} end,input_mask=function() end,paused=function() return false end,pause=function() end,resume=function() end,
  fx_world=function() return 1 end,fx_move=function() end,fx_control=function() end,fx_end=function() end}
 local mission={frame=function() end,draw=function() end,stop=function(m) m.current=nil end}
 s.a=D.app.new(g,mission);return s,s.a
end
T.test('envoy vs is routed to the rule host with its arguments, and only that',function()
 local s,a=fixture();local seen={}
 a.mods={vs_command=function(_,arg) seen[#seen+1]=arg;return true end}
 assert(s.commands.envoy('vs on'));assert(s.commands.envoy('vs max 5 12 3'));assert(s.commands.envoy('  vs   status'))
 assert(seen[1]=='on' and seen[2]=='max 5 12 3' and seen[3]=='status',table.concat(seen,'|'))
 local ok,why=a:command('vs');assert(ok and seen[4]=='','a bare `vs` reaches the host, which says its usage')
end)
T.test('envoy vs without a rule host is refused in the log; other words never match it',function()
 local s,a=fixture();local ok,why=a:command('vs on');assert(not ok and tostring(why):find('host',1,true))
 assert(not a:command('vsx'),'only the word vs is routed');assert(not a:command('hubs'))
 a.mods={vs_command=function() error('must not be reached') end}
 assert(a:command('status'),'envoy status is the old command');assert(a:command('dump'))
end)
T.test('online: envoy vs refuses through the host and the app leaves netplay alone',function()
 local s,a=fixture();s.net=true
 local calls=0;a.mods={vs_command=function() calls=calls+1;return false,'offline only: no VS builds in netplay' end}
 local ok,why=a:command('vs on');assert(not ok and tostring(why):find('netplay',1,true));assert(calls==1)
 ok=a:command('menu');assert(not ok,'the old offline-only rule still holds for the other commands')
end)
T.done()
