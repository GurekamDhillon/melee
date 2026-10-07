-- Presentation demo. Offline only because warm-up is an offline diagnostic API.
-- No always-on afterimages: they are the visible sign of a status you EARNED. Land an aerial on the opponent and
-- L-cancel the landing: a speed status starts and blue afterimages run for exactly its length. Miss the L-cancel
-- and nothing shows. Press D for a continuous debug view (clearly labelled; it needs debug=true).
local handle,warm,status=nil,nil,'Waiting for P1'
local debug,earned,misses=false,0,0
function on_lcancel_hit(e) if e.port==1 then earned=earned+1;gd.fighter_timed_status(e.entity,1,1,120) end end
function on_lcancel_miss(e) if e.port==1 then misses=misses+1 end end
local copies,follow,intensity=6,false,1
local lives,life,spacings,space={8,16,31,60},3,{2,4,8},2
local styles,style,width,length,hits={'solid','glow','fire','electric','frost','dark'},2,1.2,12,false
-- Per-copy colour presets (P palette, H hue shift per copy, E echo scale). Copy 1 is the newest.
local palettes={
  {name='off',list={}},
  {name='rainbow',list={{1,0.2,0.2,0.9},{1,0.7,0.1,0.9},{0.3,1,0.3,0.9},{0.2,0.8,1,0.9},{0.4,0.3,1,0.9},{1,0.3,0.9,0.9}}},
  {name='gold/blue',list={{1,0.85,0.2,0.9},{0.2,0.5,1,0.9}}},
  {name='ember',list={{1,0.95,0.5,1},{1,0.5,0.1,0.9},{0.8,0.15,0.05,0.8}}},
}
local pal,hues,hue,falls,fall=1,{0,25,60,-40,120},1,{0,0.6,-0.5},1
local function start()
  if handle or not gd.match().active or not gd.player(1) then return end
  handle=assert(gd.afterimage_add(1,{copies=6,spacing=4,lifetime=31,fade=0.5,blend='additive',trigger='flag',surface='silhouette',tint={0.1,1,1,1},tail={1,0.1,0.9,0.8},intensity=0}))
  warm=gd.warm{fighters={1}}
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
    gd.warm_release(warm);warm=nil;assert(gd.afterimage_set(handle,{intensity=intensity}));assert(gd.afterimage_bind(handle,{status=1,tint={0.15,0.45,1,0.8},tail={0.05,0.1,0.6,0}}));status='Ready (earn it: aerial hit + L-cancel)'
  end
  if status:sub(1,5)~='Ready' then return end
  if gd.key_pressed('D') then debug=not debug;if debug then assert(gd.afterimage_unbind(handle));assert(gd.afterimage_set(handle,{trigger='always',debug=true})) else assert(gd.afterimage_set(handle,{trigger='flag',flag=false}));assert(gd.afterimage_bind(handle,{status=1,tint={0.15,0.45,1,0.8},tail={0.05,0.1,0.6,0}})) end end
  if gd.key_pressed('C') then copies=copies%12+1;assert(gd.afterimage_set(handle,{copies=copies})) end
  if gd.key_pressed('F') then follow=not follow;assert(gd.afterimage_set(handle,{follow=follow})) end
  if gd.key_pressed('I') then intensity=intensity==1 and 0.45 or intensity==0.45 and 0 or 1;assert(gd.afterimage_set(handle,{intensity=intensity})) end
  if gd.key_pressed('T') then life=life%#lives+1;assert(gd.afterimage_set(handle,{lifetime=lives[life]})) end
  if gd.key_pressed('P') then pal=pal%#palettes+1;assert(gd.afterimage_set(handle,{palette=palettes[pal].list})) end
  if gd.key_pressed('H') then hue=hue%#hues+1;assert(gd.afterimage_set(handle,{hue_shift=hues[hue]})) end
  if gd.key_pressed('E') then fall=fall%#falls+1;assert(gd.afterimage_set(handle,{scale_falloff=falls[fall]})) end
  if gd.key_pressed('G') then space=space%3+1;assert(gd.afterimage_set(handle,{spacing=spacings[space]})) end
end
function on_draw()
  local a=gd.safe_area()
  gd.fill(a.x+12,a.y+12,math.min(a.w-24,760),114,0x101827dd)
  gd.text(a.x+24,a.y+22,'Afterimages: '..status,0xffd369ff,1.2)
  gd.text(a.x+24,a.y+48,'C: copies  T: lifetime  G: spacing  F: follow  I: intensity  D: DEBUG continuous view ('..(debug and 'ON' or 'off')..')   earned '..earned..' / missed '..misses)
  gd.text(a.x+24,a.y+92,('P: palette=%s  H: hue_shift=%d deg/copy  E: scale_falloff=%.1f  (palette replaces the status tint)'):format(palettes[pal].name,hues[hue],falls[fall]))
  gd.text(a.x+24,a.y+70,('copies=%d lifetime=%d spacing=%d follow=%s intensity=%.2f'):format(copies,lives[life],spacings[space],tostring(follow),intensity))
end
function on_unload() if warm then gd.warm_release(warm) end end
gd.command('demo_state',function(token) gd.log('tour state '..tostring(token)..' '..status) end,'demo_state <token>')
gd.command('afterimage_probe',function()
  local s=gd.motion_stats()
  gd.log(('motion: demo P1 poses=%d copy_draws=%d skipped=%d copy_budget_skips=%d warm_variants=%d fail_pipeline_missing=%d fail_palette=%d fail_empty_pose=%d'):format(s.poses,s.copy_draws,s.skipped,s.copy_budget_skips,s.warm_variants,s.fail_pipeline_missing,s.fail_palette,s.fail_empty_pose))
end,'afterimage_probe: report retained and replayed poses')
