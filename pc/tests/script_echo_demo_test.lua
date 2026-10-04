-- Real demo entry with a deterministic API stub; never launches the game.
local root='melee/pc/scripts/examples/demos/echoes/scripts/main.lua'
local calls,logs,active,fail={}, {},true,false
local gd={}
gd.match=function()return {active=active,netplay=false}end
gd.player=function(p)return {cpu=p==2}end
gd.echo_afterimage=function(p,o)
 assert(p==1 and o.copies==3 and o.spacing==4)
 if #o.echoes==1 then assert(o.echoes[1].copy==2 and o.echoes[1].match.move=='aerial')
 else assert(#o.echoes==2 and o.echoes[1].copy==1 and o.echoes[2].copy==2 and o.echoes[1].match.move=='nair' and o.echoes[2].match.move=='nair')end
 assert(o.echoes[1].damage==.4)
 if fail=='return'then return nil,'motion unavailable'end
 if fail then error('unsupported')end
 calls[#calls+1]='install';return 7,{11}
end
gd.echo_remove=function(h)assert(h==11);calls[#calls+1]='echo_remove';return true end
gd.afterimage_remove=function(h)assert(h==7);calls[#calls+1]='visual_remove';return true end
gd.cpu_mode=function(p,m)assert(p==2 and m=='stand');calls[#calls+1]='stand';return true end
gd.echoes=function()return calls[1] and {{handle=11}} or {}end
gd.fighter_history_depth=function()return 61 end
gd.command=function(n,f)gd[n]=f end
gd.log=function(s)logs[#logs+1]=s end
gd.key_pressed=function()return false end
gd.text=function()end
local env=setmetatable({gd=gd},{__index=_G})
assert(loadfile(root,'t',env))()
env.on_frame();env.on_frame();assert(#calls==2 and calls[1]=='install' and calls[2]=='stand')
gd.demo_state('probe');assert(logs[#logs]:find('active=true',1,true))
env.on_unload();assert(calls[3]=='echo_remove' and calls[4]=='visual_remove')
env.on_scene();active=false;env.on_frame();assert(#calls==4)
active=true;fail=true;env.on_frame();assert(logs[#logs]:find('refused',1,true))
fail=false;gd.demo_key('N');assert(calls[#calls]=='stand' and calls[#calls-1]=='install','neutral-air owner example not installed')
env.on_scene();fail='return';local n=#calls;env.on_frame();assert(#calls==n and logs[#logs]:find('refused',1,true),'nil emitter accepted as ready')
env.on_scene();fail=false;local done=false
gd.warm=function(o)assert(o.fighters[1]==1);calls[#calls+1]='warm';return 17 end
gd.warm_done=function(h)assert(h==17);return done end
gd.warm_release=function(h)assert(h==17);calls[#calls+1]='warm_release' end
gd.afterimage_set=function(h,o)assert(h==7 and o.intensity==1);calls[#calls+1]='show';return true end
env.on_frame();assert(calls[#calls]=='warm','new emitter pipelines not prepared')
env.on_tick();assert(calls[#calls]=='warm');done=true;env.on_tick();assert(calls[#calls]=='show' and calls[#calls-1]=='warm_release')
print('PASS echo demo setup, standing CPU, cleanup and capability refusal')
