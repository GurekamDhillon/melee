-- Offline LAB: Mario P1, standing CPU P2 on FD. Jump/neutral-air past P2.
-- Three pictures share spacing4; only copy2 replays aerials at age8.
local emitter,handles,started,warm=nil,{},false,nil
local hits,status,neutral=0,'Waiting for offline match',false
local function cleanup()
 if warm and gd.warm_release then pcall(gd.warm_release,warm)end;warm=nil
 for _,h in ipairs(handles)do pcall(gd.echo_remove,h)end
 if emitter then pcall(gd.afterimage_remove,emitter)end
 emitter,handles,started=nil,{},false
end
local function start()
 local m=gd.match()
 if started or not m.active or m.netplay or not gd.player(1)then return end
 started=true
 local ok,why=pcall(function()
  assert(gd.echo_afterimage,'echo engine unavailable; rebuild required')
  local rules=neutral and {{copy=1,match={move='nair'},damage=.4,knockback=1,once_per_move=true},{copy=2,match={move='nair'},damage=.4,knockback=1,once_per_move=true}} or {{copy=2,match={move='aerial'},damage=.4,knockback=1,once_per_move=true}}
  local h,r=gd.echo_afterimage(1,{copies=3,spacing=4,echoes=rules,
   visual={surface='silhouette',blend='additive',trigger='always',intensity=gd.warm and gd.warm_done and 0 or 1,tint={.2,.7,1,.65}}})
  assert(type(h)=='number' and h>0 and type(r)=='table',r or 'echo picture unavailable');emitter,handles=h,r
  if gd.player(2) and gd.player(2).cpu then assert(gd.cpu_mode(2,'stand'))end
  if gd.warm and gd.warm_done then warm=assert(gd.warm{fighters={1}})end
 end)
 if not ok then cleanup();started=true;status='refused: '..tostring(why);gd.log(status);return end
 status=warm and 'Preparing pictures' or neutral and 'Ready: copies1/2 repeat only neutral-air at ages4/8' or 'Ready: copy2 repeats aerials at age8; jump/nair past standing P2'
 gd.log('echo demo: '..status)
end
gd.command('demo_state',function(token)
 gd.log('tour state '..tostring(token)..' active='..tostring(emitter~=nil)..' echoes='..#(gd.echoes and gd.echoes(1)or {})..' hits='..hits..' acceptance=operator-required')
end,'demo_state <token>: echo ownership and actual hit count')
gd.command('demo_key',function(key)if key=='E'or key=='N'then neutral=key=='N';cleanup();start()end end,'demo_key E: aerial copy2; N: neutral-air copies1/2')
function on_frame()start()end
function on_match_start()start()end
function on_tick()
 if warm then
  local done,why=gd.warm_done(warm)
  if why then cleanup();started=true;status='refused: '..tostring(why);gd.log(status)
  elseif done then gd.warm_release(warm);warm=nil;assert(gd.afterimage_set(emitter,{intensity=1}));status='Ready: '..(neutral and 'nair copies1/2' or 'aerial copy2')end
 end
 if gd.key_pressed('E')then neutral=false;cleanup();start()
 elseif gd.key_pressed('N')then neutral=true;cleanup();start()end
end
function on_hit(attacker,victim,e)
 if attacker==1 then hits=hits+1;status='P1 connected: '..tostring(e and e.move_tag or 'unknown')..'; hits='..hits end
end
function on_draw()
 gd.text(20,20,'Echoes: three afterimages, delayed hitboxes at40% damage')
 gd.text(20,42,status)
 gd.text(20,64,'E: aerial copy2. N: nair copies1/2. Jump past P2; delayed hit stays at its past position.')
end
function on_scene()if warm and gd.warm_release then pcall(gd.warm_release,warm)end;emitter,handles,started,warm=nil,{},false,nil;status='Waiting for offline match'end
function on_match_end()cleanup()end
function on_unload()cleanup()end
