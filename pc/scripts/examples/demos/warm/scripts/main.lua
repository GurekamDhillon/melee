-- One feature: prepare enemy material pipelines before their first visible spawn.
local enemies={'goomba','koopa','redead','like_like','octorok','polar_bear','topi'}
local warm,enemy,started,status=nil,nil,false,'Waiting for offline match'
local function begin()
  if started or not gd.match().active then return end
  started=true
  warm=gd.warm{enemies=enemies}
  status='Warming seven enemy descriptors in the background'
  gd.log('warm demo declared seven enemy kinds')
end
function on_match_start() begin() end
function on_scene() warm,enemy,started=nil,nil,false end
local function progress()
  begin()
  if not warm then return end
  local done,why=gd.warm_done(warm)
  if why then
    status='Warm failed: '..tostring(why);gd.log(status)
    gd.warm_release(warm);warm=nil;return
  end
  if not done then
    local p=gd.warm_status(warm)
    status=('Prepared %d/%d; %d pipeline/material jobs pending'):format(p.prepared,p.objects,p.pending)
    return
  end
  local why_spawn
  enemy,why_spawn=gd.spawn_enemy('goomba',25,12)
  status=enemy and 'Warm complete: Goomba appears only after readiness' or 'Spawn refused: '..tostring(why_spawn)
  gd.log(status);gd.warm_release(warm);warm=nil
end
function on_tick()
  if not gd.match().active then return end
  for port=1,6 do local p=gd.player(port);if p and p.cpu then gd.cpu_mode(port,'stand') end end
  progress()
end
-- Catch up when loaded into an already-running match, as the other demos do.
function on_frame() if not started then begin() end end
function on_draw()
  local a=gd.safe_area()
  gd.fill(a.x+12,a.y+12,math.min(a.w-24,650),80,0x101827dd)
  gd.text(a.x+24,a.y+24,'Warm: declare before draw',0xffd369ff,1.2)
  gd.text(a.x+24,a.y+50,status,0xffffffff)
  gd.text(a.x+24,a.y+70,'Seven descriptors, then one Goomba. Readiness is polled each tick.',0x88e0aaff)
end
function on_unload()
  if warm then gd.warm_release(warm);warm=nil end
  if enemy and gd.enemy_alive(enemy) then gd.enemy_remove(enemy) end
end

-- Owner-local completion proof for the catalogue tour; no foreign handles exposed.
gd.command('demo_state',function(token)
  local ready=enemy~=nil and warm==nil
  gd.log('tour state '..token..' '..(ready and 'ready' or 'pending')..' | '..status)
end,'demo_state <token>: report this warm demo readiness')
