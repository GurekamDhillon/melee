-- Presentation demo. Offline only because warm-up is an offline diagnostic API.
local handle,warm,status=nil,nil,'Waiting for P1'
local copies,follow,intensity=3,false,0.45
local styles,style,width,length,hits={'solid','glow','fire','electric','frost','dark'},2,6,31,false
local intensity=1
local looks={solid={tint={1,1,0.6,1},tail={1,0.5,0,0.2}},glow={tint={0.1,1,1,1},tail={0.2,0.2,1,0.1}},fire={tint={1,0.55,0.05,1},tail={1,0,0,0.1}},electric={tint={1,1,0.2,1},tail={0.3,0.6,1,0.1}},frost={tint={0.7,0.95,1,1},tail={0.2,0.5,1,0.1}},dark={tint={0.7,0.1,1,1},tail={0.1,0,0.2,0.2}}}
local function start()
  if handle or not gd.match().active or not gd.player(1) then return end
  handle=assert(gd.tracer_add{port=1,anchor='right_hand',width=6,taper=0.4,length=31,smoothing=8,shader='glow',params={1,2,0.5,2},tint=looks.glow.tint,tail=looks.glow.tail,edge={1,1,1,0.8},intensity=0})
  warm=gd.warm{tracers=true}
  status='Preparing motion pipelines'
end
function on_frame() start() end
function on_match_start() start() end
function on_scene() handle,warm=nil,nil;status='Waiting for P1' end
function on_tick()
  if not gd.match().active or not handle then return end
  if warm then
    local done,why=gd.warm_done(warm)
    if why then status='Warm refused: '..tostring(why);gd.log(status);gd.warm_release(warm);warm=nil;return end
    if not done then return end
    gd.warm_release(warm);warm=nil;assert(gd.tracer_set(handle,{intensity=intensity}));status='Ready'
  end
  if status~='Ready' then return end
  if gd.key_pressed('S') then style=style%6+1;assert(gd.tracer_set(handle,{shader=styles[style],tint=looks[styles[style]].tint,tail=looks[styles[style]].tail})) end
  if gd.key_pressed('W') then width=width>=24 and 0.5 or math.min(24,width+6);assert(gd.tracer_set(handle,{width=width})) end
  if gd.key_pressed('L') then length=length>=60 and 4 or math.min(60,length+20);assert(gd.tracer_set(handle,{length=length})) end
  if gd.key_pressed('A') then hits=not hits;assert(gd.tracer_set(handle,{anchor=hits and 'active_hitboxes' or 'right_hand'})) end
end
function on_draw()
  local a=gd.safe_area()
  gd.fill(a.x+12,a.y+100,math.min(a.w-24,760),92,0x101827dd)
  gd.text(a.x+24,a.y+110,'Tracers: '..status,0xffd369ff,1.2)
  gd.text(a.x+24,a.y+136,'S: shader colour  W: width to 24  L: length to 60  A: hand / attacking hitboxes')
  gd.text(a.x+24,a.y+158,('shader=%s width=%.1f length=%d hitboxes=%s'):format(styles[style],width,length,tostring(hits)))
end
function on_unload() if warm then gd.warm_release(warm) end end
gd.command('demo_state',function(token) gd.log('tour state '..tostring(token)..' '..status) end,'demo_state <token>')
