-- Actual single-feature scripts, narrow offline stub; no native execution.
local root='melee/pc/scripts/examples/demos/'
local function load_demo(name,g)
 local e={gd=g,ipairs=ipairs,pairs=pairs,tostring=tostring,math=math,string=string,table=table}
 assert(loadfile(root..name..'/scripts/main.lua','t',e))();return e
end
local logs,draws={},0
local g={log=function(s) logs[#logs+1]=s end,kit={text=function() draws=draws+1 end,panel=function() draws=draws+1 end}}
local e={mode='classic',stage_index=0,stage_kind='battle',loop=0,opponents={{port=2}},player_port=1}
local m=load_demo('1p-awareness',g)
m.on_1p_stage_start(e);m.on_1p_stage_clear(e);e.boss_kind='master_hand';m.on_1p_boss_defeated(e);m.on_1p_complete(e);m.on_1p_game_over(e);m.on_draw()
assert(#logs==5 and draws==1)
local hold,release,button=0,0,false
g.hold_1p=function(n) assert(n==180);hold=hold+1;return true end
g.release_1p=function() release=release+1;return true end
g.pad=function()return {A=button}end
m=load_demo('1p-hold',g);m.on_1p_stage_clear(e);m.on_draw();m.on_tick();button=true;m.on_tick();assert(release==1)
button=false;m.on_1p_stage_clear(e);for i=1,180 do m.on_tick()end;assert(release==2 and hold==2)
local active,calls={},0
g.spawn_1p=function(port,opt)calls=calls+1;active[port]=opt;return true end
m=load_demo('1p-spawn',g);m.on_1p_stage_start(e);assert(active[2].run_speed==1.15 and active[2].tint==0xff8800ff)
m.on_1p_spawn{port=2,entity=8};assert(calls==1) -- Notification never reapplies itself.
m.on_1p_stage_clear(e);assert(active[2]==nil);m.on_unload()
local start,ended,loops=0,0,0
g.start_1p=function(o)assert(o.mode=='classic' and o.difficulty==2 and o.stocks==3 and o.loop);start=start+1;return true end
g.end_1p=function()ended=ended+1;return true end
g.loop_1p=function(v)assert(v==false);loops=loops+1 end
m=load_demo('1p-loop',g);m.on_key_down('C');m.on_1p_complete(e);m.on_key_down('B');m.on_unload()
assert(start==1 and ended==2 and loops==1)
print('1P demos PASS: lifecycle logging, hold A/timeout, per-spawn template/cleanup, launch/loop/end')
