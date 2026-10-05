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
T.test('with gd.cpu_assist the skill maps straight to the assist, seeded, cleared on leave, and no switch ever happens',function()
 local f=fake();local calls={}
 f.g.cpu_assist=function(p,c) calls[#calls+1]={p=p,c=c};return true end
 local d=X.new(f.g);d.stats={}
 assert(d:set(2,{clean_landing=1,wavedash=nil,tech_guard=1},{depth=12,loop=0},777))
 local c=calls[1].c;local sk=X.skill({depth=12,loop=0})
 assert(calls[1].p==2 and c.lcancel==sk and c.tech==sk and c.fast_fall==sk and c.wavedash==nil and c.perfect_shield==nil and c.seed==777 and c.tech_dir=='random')
 f.players[2]={port=2,cpu=true,x=0,y=10,vy=-1,airborne=true,action=67,action_frame=6,facing=1,hitlag=0};f.players[1]={port=1,x=40,y=0,action=14}
 for fr=1,30 do f.players[2].y=math.max(0,f.players[2].y-1.2);d:frame(fr) end
 assert(#f.calls==0,'the assist driver switched modes: '..table.concat(f.calls,','))
 f.players[2]=nil;d:frame(31);assert(d.foes[2]==nil and calls[#calls].c==nil and calls[#calls].p==2,'cleared when the opponent left')
 assert(not d:set(3,{kindling=1},{depth=3,loop=0},1),'a plain build installs no assist')
end)
T.test('the switch driver stays the fallback when the exe has no assist, and when forced',function()
 local f=fake();local d=X.new(f.g);d:set(2,{clean_landing=1},{depth=12,loop=3},1);assert(not d.foes[2].assist)
 local f2=fake();f2.g.cpu_assist=function() return true end;X.force_switch=true;local d2=X.new(f2.g);d2:set(2,{clean_landing=1},{depth=12,loop=3},1);X.force_switch=nil
 assert(not d2.foes[2].assist)
end)
T.test('an assist refused (netplay, not a CPU) leaves the foe undriven but reported',function()
 local f=fake();f.g.cpu_assist=function() return false end;local d=X.new(f.g);d:set(2,{wavedasher=1},{depth=3,loop=0},5)
 assert(d.foes[2].assist and not d.foes[2].accepted and d.stats[2].refused)
 assert(d:report()[1]:match('accepted=false'))
end)
T.test('assist_config: every wanted technique gets the skill; fast fall rides along; tech direction random',function()
 local c=X.assist_config({lcancel=true,ps=true,tech=true,wavedash=true},.5,3e9)
 assert(c.lcancel==.5 and c.perfect_shield==.5 and c.tech==.5 and c.wavedash==.5 and c.fast_fall==.5 and c.seed==2147483647)
 assert(select(2,X.assist_config({},.5,1))==false)
end)
T.done()
