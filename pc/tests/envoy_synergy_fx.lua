-- The look of synergies (synergy_fx.lua): emblems, the surface lane, chain flashes and the counter, the grid overlay, the console. A recording gd stands in for the engine.
local T=dofile('melee/pc/tests/envoy_testlib.lua');local D=T.rules()
D.mod_tuning=T.module('mod_tuning',D)
for _,n in ipairs{'mod_codec','mod_schema','mod_graph','mod_budget','mod_engine','keystones','mod_pool','synergy_fx'} do D[n]=T.module(n,D) end
local P,G,F=D.mod_progression,D.mod_graph,D.synergy_fx
local function fake()
 local g={calls={},cmds={},logs={},frame_n=0}
 for _,n in ipairs{'fill','box','line','text'} do g[n]=function(...) g.calls[#g.calls+1]={n,...} end end
 g.kit={text=function(x,y,s,role,col) g.calls[#g.calls+1]={'ktext',x,y,s,role,col};return #s*6 end,measure=function(s) return #s*6 end,panel=function() end}
 g.safe_area=function() return {x=0,y=0,w=640,h=360} end
 g.frame=function() return g.frame_n end
 g.player=function(p) if p<=2 then return {x=p*100,y=0,cpu=p==2} end end
 g.project=function(x,y) return x+300,200-y,true end
 g.command=function(name,fn) g.cmds[name]=fn end
 g.log=function(s) g.logs[#g.logs+1]=s end
 return g
end
local function engine(mods) local e=D.mod_engine.new(1,D.mod_pool,{context=P.context(60,3)});e:set_build(1,mods,{});return e end
local function fx_with(host) local g=fake();local f=F.new(g,host or {mods={},hud={}});return f,g end
T.test('emblems: nine silhouettes, all different, none empty, readable in greyscale',function()
 local seen={}
 for _,a in ipairs(G.archetypes) do
  local rows=assert(F.emblems[a.motif],a.id);assert(#rows==9)
  local key=table.concat(rows,'/');assert(not seen[key],a.id..' duplicates '..tostring(seen[key]));seen[key]=a.id
  local on=0;for _,r in ipairs(rows) do assert(#r==9,a.id..' row width');for c in r:gmatch('#') do on=on+1 end end;assert(on>=17 and on<=70,a.id..' '..on)
 end
 -- colours are distinct from each other (the hue is the second cue after the shape)
 local cols={};for _,a in ipairs(G.archetypes) do assert(not cols[a.colour]);cols[a.colour]=true end
 local f,g=fx_with();F.emblem(g,G.by_id.burn,10,10,16,255);assert(#g.calls>10)
end)
T.test('surface lane: archetype index plus level, only while assembled; a firing chain raises it; preview overrides',function()
 local f,g=fx_with();local e=engine({kindling=1,burning=1,cinder=1,glass_core=1});e.frame=100
 local lane=f:lane(e,1);assert(lane>=1 and lane<2,lane);assert(math.abs(lane-(1+.625*.99))<.06,'base level')
 assert(f:lane(e,2)==0,'nothing assembled on an empty port')
 local half=engine({kindling=1,burning=1});assert(f:lane(half,1)==0,'partial: no treatment')
 f.active[1]={arch=G.by_id.burn,frame=100,size=1};assert(f:lane(e,1)-1>.9,'firing: strong')
 e.frame=100+95;assert(f:lane(e,1)-1<.7,'it settles back to the base level')
 D.mod_tuning.fx.surface=false;assert(f:lane(e,1)==0);D.mod_tuning.fx.surface=true
 f.pv={arch=G.by_id.chill,port=2,level=.5,until_f=e.frame+10};assert(math.floor(f:lane(e,2))==2 and f:lane(e,1)>=1)
 f.pv=nil
 -- a second archetype completed at the same time: the active chain's one shows
 local both=engine({kindling=1,burning=1,cinder=1,icebound=1,frosted=1,shatter=1});both.frame=1
 f.active[1]={arch=G.by_id.chill,frame=1,size=.5};assert(math.floor(f:lane(both,1))==G.by_id.chill.index)
end)
T.test('chains: the trace names the records, a flash is made at most every 18 frames, the counter climbs and fades',function()
 local host={mods={},hud={}};local f,g=fx_with(host);local e=engine({ledge=1,still_heart=1,renewal=1});host.mods.engine=e
 e:begin_frame({[1]={percent=0},[2]={percent=0}});e:emit{kind='ledge_grab',port=1,tags={}};e:drain()
 assert(e.trace_port==1)
 local ids=F.trace_ids(e);assert(#ids>=2,table.concat(ids,','))
 f:scan(e);assert(f.counter[1] and f.counter[1].n==1 and #f.flashes==1 and f.flashes[1].to==2)
 local arch=f.flashes[1].arch
 for i=1,5 do e.frame=e.frame+3;f:chain_fired(e,1,arch,.5) end   -- rapid: 15 frames
 assert(#f.flashes==1,'no strobe: one flash inside 18 frames')
 assert(f.counter[1].n<=2,'the counter is rate limited too ('..f.counter[1].n..')')
 e.frame=e.frame+25;f:chain_fired(e,1,arch,.5);assert(#f.flashes==2 and f.flashes[2].soft,'inside 45 frames the flash is softened')
 e.frame=e.frame+200;f:chain_fired(e,1,arch,.5);assert(f.counter[1].n==1,'a chain that stopped starts the counter again')
 -- the look of the pulse follows the archetype
 local look=F.pulse_look(e);assert(look and #look.tint==4 and look.shape>=0)
 -- scaling: a bigger chain makes a wider, stronger link
 g.calls={};f.flashes={{from=1,to=2,arch=arch,start=e.frame,size=.2}};e.frame=e.frame+8;f:draw_flashes(e);local small=#g.calls
 g.calls={};f.flashes={{from=1,to=2,arch=arch,start=e.frame-8,size=1}};f:draw_flashes(e);assert(#g.calls>=small)
 D.mod_tuning.fx.chain=false;g.calls={};f.flashes={{from=1,to=2,arch=arch,start=e.frame-8,size=1}};f:draw_flashes(e);assert(#g.calls==0,'layer off');D.mod_tuning.fx.chain=true
end)
T.test('the HUD pill shows assembled emblems and a counter; the announcement is said once per archetype',function()
 local said={};local host={mods={},hud={strip_w=200,announce=function(_,lines,arch) said[#said+1]=arch.id end},log=function() end}
 local f,g=fx_with(host);local e=engine({kindling=1,burning=1,cinder=1});host.mods.engine=e
 e.frame=40;f:frame(e,1);f:frame(e,1);assert(#said==1 and said[1]=='burn')
 e.frame=60;f:frame(e,1);assert(#said==1,'only once')
 f.counter[1]={arch=G.by_id.burn,n=3,last=60,stamp=60,size=.6};g.calls={};f:draw_hud(e,1)
 local x=false;for _,c in ipairs(g.calls) do if c[1]=='ktext' and c[4]=='x3' then x=true end end;assert(x,'the counter reads x3')
 e.frame=60+200;g.calls={};f:draw_hud(e,1);for _,c in ipairs(g.calls) do assert(not (c[1]=='ktext' and c[4]:match('^x%d')),'the counter has faded out') end
 D.mod_tuning.fx.announce=false;local f2=fx_with({mods={engine=e},hud={announce=function() error('no') end},log=function() end});e.frame=80;f2:frame(e,1);D.mod_tuning.fx.announce=true
 g.calls={};D.mod_tuning.fx.hud=false;f:draw_hud(e,1);assert(#g.calls==0);D.mod_tuning.fx.hud=true
end)
T.test('the grid: links, banner, offer marks and the detail line',function()
 local tier=1
 local equipped={{affixes={{id='kindling'}}},{affixes={{id='burning'}}}}
 local host={bag=function() return {slots=function() return 3 end,equipped=equipped,items={},context=P.context(5,0)} end,keystone_ids=function() return {'smash_doctrine'} end,mods={},hud={}}
 local f,g=fx_with(host)
 local cinder={affixes={{id='cinder'}}};local icy={affixes={{id='icebound'}}}
 local blocks={
  {id='offer',cols=2,rows=1,cells={{ref={kind='offer',record=cinder},name='Cinder'},{ref={kind='offer',record=icy},name='Icebound'}}},
  {id='eq',cols=3,rows=1,cells={{ref={kind='eq',where='equipped',index=1,record=equipped[1]},name='K'},{ref={kind='eq',where='equipped',index=2,record=equipped[2]},name='B'},{empty=true,ref={kind='eq',index=3},name='Empty'}}},
  {id='key',cols=6,rows=1,cells={{ref={kind='key',id='smash_doctrine'},name='S'}}},
 }
 local view={lay={head={x=20,y=10,w=600,h=30}},focused=function() return blocks[1].cells[1],'offer',1 end,
  cell_rect=function(_,b,i) local bi=({offer=0,eq=1,key=2})[b];return 40+i*60,40+bi*70,56,56,41+i*60,41+bi*70,54,54 end}
 local screen={view=view,blocks=blocks,layout='main',key='k1'}
 local m=f:grid_model(screen)
 assert(m.offers['offer:1'] and m.offers['offer:1'].kind=='advance' or m.offers['offer:1'].kind=='complete',m.offers['offer:1'] and m.offers['offer:1'].kind)
 assert(not m.offers['offer:2'],'Icebound connects to nothing held and builds no chain')
 assert(#m.links>=2,'Cinder links to Kindling; Burning links to Kindling')
 assert(m.marks['eq:1'] and m.marks['offer:1'] and not m.marks['key:1'],'a stat stick has no mark')
 assert(m.banner[1].archetype.id=='burn' and m.banner[1].filled==1 and m.banner[1].missing=='a payoff'~=nil)
 g.calls={};f:draw_grid(screen);assert(#g.calls>20)
 local banner=false;for _,c in ipairs(g.calls) do if c[1]=='ktext' and c[4]:match('Burn stacking 1/2: needs a payoff') then banner=true end end;assert(banner,'partial progress reads in the banner')
 local lines=f:detail_lines(screen,blocks[1].cells[1]);assert(#lines>=1 and lines[1]:match('Cinder'),lines[1])
 local done=false;for _,l in ipairs(lines) do if l:match('completes Burn stacking') then done=true end end;assert(done,'the offer completes the chain')
 for _,k in ipairs{'grid_links','grid_banner','offer_marks'} do D.mod_tuning.fx[k]=false end
 g.calls={};f:draw_grid(screen);assert(#g.calls==0,'every grid layer can be switched off');for _,k in ipairs{'grid_links','grid_banner','offer_marks'} do D.mod_tuning.fx[k]=true end
 D.mod_tuning.fx.intensity=0;g.calls={};f:draw_grid(screen);assert(#g.calls==0);D.mod_tuning.fx.intensity=1
 -- the model is cached until the screen changes
 assert(f:grid_model(screen)==m);screen.key='k2';assert(f:grid_model(screen)~=m)
end)
T.test('the console: preview any archetype, layers on and off, intensity',function()
 local host={mods={},hud={announce=function() end},log=function() end};local f,g=fx_with(host);local e=engine({});e.frame=10;host.mods.engine=e
 assert(g.cmds.synergy('fx preview shock 1') and f.pv.arch.id=='shock' and f.pv.level==1)
 assert(g.cmds.synergy('fx preview Burn') and f.pv.arch.id=='burn' and math.abs(f.pv.level-.8)<1e-9)
 assert(f:lane(e,1)>=1,'the preview shows on port 1 without assembling anything')
 for i=1,3 do e.frame=e.frame+31;f:frame(e,1) end;assert(f.counter[1] and f.counter[1].n>=2 and #f.flashes>=1,'the preview runs the chain moment')
 assert(not g.cmds.synergy('fx preview nothing'))
 assert(g.cmds.synergy('fx chain off') and D.mod_tuning.fx.chain==false);assert(g.cmds.synergy('fx chain on') and D.mod_tuning.fx.chain==true)
 assert(g.cmds.synergy('fx intensity 0.5') and D.mod_tuning.fx.intensity==.5);g.cmds.synergy('fx intensity 1')
 assert(g.cmds.synergy('archetypes') and g.cmds.synergy('fx') and g.cmds.synergy('fx dump'))
end)
T.done()
