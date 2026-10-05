-- The opponent technique driver: pure decisions over a fake engine (the CPU controller calls are recorded).
local T=dofile('melee/pc/tests/envoy_testlib.lua')
local D={}
for _,n in ipairs({'mod_progression','mod_echo','mod_registry','mod_status','mod_skill','mod_schema','mod_codec','mod_budget','mod_engine','keystones','mod_techniques','mod_pool','drive_loot','drive_merge','drive_economy','drive_bag','foe_roll','foe_driver'}) do D[n]=T.module(n,D) end
local X=D.foe_driver
local function fake()
 local f={calls={},players={},frame=0,modes={}}
 f.g={players=function() local l={};for _,p in pairs(f.players) do l[#l+1]=p end;return l end,player=function(p) return f.players[p] end,
  floor_below=function() return 0 end,cpu_attrs=function() return {gravity=.095,terminal_velocity=1.7} end,
  cpu_mode=function(p,m) f.calls[#f.calls+1]='mode:'..m;f.modes[p]=m;return true end,
  cpu_pad=function(p,t,n) f.calls[#f.calls+1]='pad:'..tostring(t.buttons)..':'..tostring(n);return true end,
  cpu_macro=function(p,name,o) f.calls[#f.calls+1]='macro:'..name;return true end,cpu_script_done=function() return true end}
 return f
end
local function has(f,s) for _,c in ipairs(f.calls) do if c==s then return true end end return false end
T.test('skill grows with effective depth and loop, capped; the decision is a pure hash of seed port frame',function()
 local last=-1;for e=0,40 do local s=X.skill({depth=e%13,loop=e//13});assert(s>=last-1e-9 or e%13==0);if e%13~=0 then last=s end end
 assert(X.skill({depth=0,loop=0})<.1 and X.skill({depth=12,loop=0})>.6 and X.skill({depth=0,loop=3})==X.tuning.cap)
 assert(X.roll(5,2,100,1)==X.roll(5,2,100,1) and X.roll(5,2,100,1)~=X.roll(5,2,101,1))
 local n=0;for fr=1,2000 do if X.roll(9,2,fr,2)<.3 then n=n+1 end end;assert(n>480 and n<720,'roughly 30% of 2000: '..n)
end)
T.test('only the techniques a build rewards are driven; a plain build is not driven at all',function()
 local f=fake();local d=X.new(f.g)
 assert(not d:set(2,{kindling=1},{depth=3,loop=0},1))
 assert(d:set(2,{clean_landing=1,shield_stance=1},{depth=3,loop=0},1) and d.foes[2].want.lcancel and d.foes[2].want.ps and not d.foes[2].want.wavedash)
 assert(d:set(3,{wavedasher=1},{depth=3,loop=0},1) and d.foes[3].want.wavedash)
 assert(d:set(4,{tech_guard=1},{depth=3,loop=0},1) and d.foes[4].want.tech)
end)
T.test('an aerial about to land: L is pressed by a switch to script and control is handed back',function()
 local f=fake();local d=X.new(f.g);d:set(2,{clean_landing=1},{depth=12,loop=3},1);d.foes[2].skill=1
 f.players[2]={port=2,cpu=true,x=0,y=10,vy=-1.0,airborne=true,action=67,action_frame=6,facing=1,hitlag=0}
 f.players[1]={port=1,x=40,y=0,action=14,airborne=false}
 local began
 for fr=1,12 do f.players[2].y=math.max(0,f.players[2].y-1.2);d:frame(fr);if d.foes[2].active and not began then began=fr end end
 assert(began and has(f,'mode:script') and has(f,'pad:L:3'),table.concat(f.calls,','))
 f.players[2].airborne=false;f.players[2].action=14;f.players[2].action_frame=9;for fr=13,30 do d:frame(fr) end
 assert(not d.foes[2].active and f.modes[2]=='fight' and d.stats[2].attempts.lcancel==1)
end)
T.test('skill 0 never switches, and a driven foe in hitstun on the ground is handed back at once',function()
 local f=fake();local d=X.new(f.g);d:set(2,{clean_landing=1},{depth=0,loop=0},1);d.foes[2].skill=0
 f.players[2]={port=2,cpu=true,x=0,y=10,vy=-1.0,airborne=true,action=65,action_frame=6,facing=1,hitlag=0}
 for fr=1,10 do f.players[2].y=math.max(0,f.players[2].y-1.5);d:frame(fr) end
 assert(#f.calls==0,'skill 0 switched anyway');assert(d.stats[2].opportunities.lcancel==1,'the opportunity is counted')
 d.foes[2].skill=1;d.foes[2].active={kind='wavedash',start=1,release=999};d.foes[2].cool=0
 f.players[2].airborne=false;f.players[2].in_hitstun=true;f.players[2].action=38;d:frame(20);assert(not d.foes[2].active and f.modes[2]=='fight')
end)
T.test('a perfect shield attempt needs the attacker in range inside its startup frame; dead and mid-air foes are ignored',function()
 local f=fake();local d=X.new(f.g);d:set(2,{shield_stance=1},{depth=12,loop=3},1);d.foes[2].skill=1
 f.players[2]={port=2,cpu=true,x=0,y=0,airborne=false,action=14,facing=1,hitlag=0,action_frame=0}
 f.players[1]={port=1,x=20,y=0,action=45,action_frame=0,airborne=false}
 d:frame(1);assert(#f.calls==0)
 f.players[1].action_frame=1;d:frame(2);assert(has(f,'macro:perfect_shield'),table.concat(f.calls,','))
 local f2=fake();local d2=X.new(f2.g);d2:set(2,{shield_stance=1},{depth=12,loop=3},1);d2.foes[2].skill=1
 f2.players[2]={port=2,cpu=true,x=0,y=0,airborne=false,action=14,facing=1,hitlag=0};f2.players[1]={port=1,x=200,y=0,action=45,action_frame=1}
 d2:frame(1);assert(#f2.calls==0,'out of range')
end)
T.test('foe_roll keeps full weight for driven techniques and weights dead ones down',function()
 local r=D.foe_roll.new(D.mod_pool);local w=r:weights(4)
 assert(w.clean_landing==nil and w.wave_edge==nil and w.shield_stance==nil and w.tech_guard==nil,'driven records are not down-weighted')
end)
T.done()
