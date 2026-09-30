-- Installer prepends lexical Core, Dungeon and Commands modules. No require.
-- Initial offline vertical slice: bounded FD rooms and ordinary native CPU AI.
local profile,run,manifest,node
local ready,launching,active=false,false,false
local launch_seen=false
local pending_entry=false
local tick_serial,entry_after_tick=0,0
local pending_room=nil
local pending_ticks=0
local transition_error,pending_finish=nil,nil
local menu='collection'
local menu_state=Menus.new()
local selected_fighter,run_fighter={id='falco',costume=0},{id='falco',costume=0}
local roster_state=Roster.new(selected_fighter)
local launched_scene,pending_begin,launch=nil,false,nil
local platforms,enemies,fx_handles={},{},{}
local buttons,mouse_buttons,last_frame=0,0,-1
local toast=''
local feedback=Feedback.new()
local room_visuals=Rooms.new()
local command_state=Commands.new()
local previous_stocks={};local move_serial={0,0};local move_ids={}
local movement={0,0}
local enemy_host,enemy_gene,enemy_kos=nil,nil,0
local BASE_STOCKS=99
local generation,save_error=0,false
local starter='g1'
local config=gd.data_read('config.txt') or ''
local demo=config:find('demo=true',1,true)~=nil
local slots={'assault','traversal','guard'}
local function say(s,kind,key)
 toast=s;gd.log('roguelite: '..s)
 Feedback.notify(feedback,{key=key or s,kind=kind or 'info',title=s})
end
local function feedback_sync(frames,paused)
 local abilities={}
 if run and run.hosts.player then for _,slot in ipairs(slots) do abilities[slot]=Core.ability(run,'player',slot) end end
 Feedback.update(feedback,abilities,frames or 0,paused,run and run.hosts.player and run.hosts.player.slots)
end
local function keys(t) local a={} for k in pairs(t) do a[#a+1]=k end table.sort(a) return a end
local function checkpoint(raw)
 if type(raw)~='string' or #raw>1048700 then return nil end
 -- TBD3: resolved manifest (+ optional progress) with an integrity checksum. A
 -- section of the wrong schema version is refused and preserved, not returned.
 local decoded=Checkpoint.decode(raw,Core,Codec)
 if decoded then
  local p=decoded.profile
  if not p or p.type~='profile' then return nil end
  local r=decoded.run
  if r then local serial=tonumber(r.id:match('^run(%d+)$'));if not serial or serial>=p.next_run then return nil end end
  local selected,played={id='falco',costume=0},{id='falco',costume=0}
  if decoded.roster then
   local a,b=decoded.roster:match('^(ROSTER1 [^\n]+\n)(ROSTER1 [^\n]+\n)$')
   selected,played=Roster.decode(a),Roster.decode(b)
   if not selected or not played then return nil end
  end
  return {generation=decoded.generation,profile=p,run=r,selected=selected,fighter=played,manifest=decoded.manifest,progress=decoded.progress}
 end
 -- Legacy TBD2/TBD1 envelope.
 if #raw>530000 then return nil end
 local g,plen,rlen,mlen,body=raw:match('^TBD2 (%d+) (%d+) (%d+) (%d+)\n(.*)$')
 local legacy=not g
 if legacy then g,plen,rlen,body=raw:match('^TBD1 (%d+) (%d+) (%d+)\n(.*)$');mlen=0 end
 mlen=tonumber(mlen)
 if not mlen or mlen<0 or mlen>192 then return nil end
 g,plen,rlen=tonumber(g),tonumber(plen),tonumber(rlen)
 if not g or g~=g or g<0 or g>1000000000 or g%1~=0 or not plen or not rlen or plen<1 or plen>262144 or rlen>262144 or #body~=plen+rlen+mlen then return nil end
 local p=Core.restore(body:sub(1,plen));local r=rlen>0 and Core.restore(body:sub(plen+1,plen+rlen)) or nil
 if not p or p.type~='profile' or (rlen>0 and (not r or r.type~='run' or r.owner~=p.id)) then return nil end
 if r then local serial=tonumber(r.id:match('^run(%d+)$'));if not serial or serial>=p.next_run then return nil end end
 local selected,played={id='falco',costume=0},{id='falco',costume=0}
 if not legacy then
  local meta=body:sub(plen+rlen+1);local a,b=meta:match('^(ROSTER1 [^\n]+\n)(ROSTER1 [^\n]+\n)$')
  selected,played=Roster.decode(a),Roster.decode(b)
  if not selected or not played then return nil end
 end
 return {generation=g,profile=p,run=r,selected=selected,fighter=played}
end
local function save()
 if save_error then say('Save disabled: repair the preserved invalid checkpoint first','error','save');return false end
 local p,why=Core.snapshot(profile);if not p then say('Save refused: '..tostring(why),'error','save');return false end
 local r=''
 if run then r,why=Core.snapshot(run);if not r then say('Run save refused: '..tostring(why),'error','save');return false end end
 -- Persist the resolved manifest so a later catalogue/generator change cannot
 -- relocate a saved doorway. v1 runs carry no separate progress record yet.
 local mtext=nil
 if run then
  local man=(manifest and manifest.seed==run.world_seed) and manifest or Dungeon.generate(run.world_seed)
  local encoded,ewhy=Codec.encode(man)
  if not encoded then say('Manifest encode refused: '..tostring(ewhy),'error','save');return false end
  mtext=encoded
 end
 local next_generation=generation+1;local file=next_generation%2==0 and 'checkpoint-a.txt' or 'checkpoint-b.txt'
 local metadata=assert(Roster.encode(selected_fighter))..assert(Roster.encode(run_fighter))
 local text
 do
  local ok,res=pcall(Checkpoint.encode,{generation=next_generation,profile=p,run=r~='' and r or nil,manifest=mtext,roster=metadata})
  if not ok then say('Checkpoint encode refused: '..tostring(res),'error','save');return false end
  text=res
 end
 -- Atomic when the native helper is present; otherwise the raw write with the
 -- same readback proof. Previous bytes survive a refused or failed write.
 local write=gd.data_write_atomic or gd.data_write
 local ok,result=pcall(write,file,text)
 local read_ok,readback=pcall(gd.data_read,file)
 if not ok or result==false or not read_ok or readback~=text or not checkpoint(text) then say('Save write/readback failed; previous checkpoint retained','error','save');return false end
 generation=next_generation
 return true
end
local function load_data()
 local a,b=gd.data_read('checkpoint-a.txt'),gd.data_read('checkpoint-b.txt')
 local ca,cb=checkpoint(a),checkpoint(b)
 local best=ca;if cb and (not best or cb.generation>best.generation) then best=cb end
 if best then profile,run,generation=best.profile,best.run,best.generation;selected_fighter,run_fighter=best.selected,best.fighter
   manifest=best.manifest
   if run and (profile.finished[run.id] or run.status~='active') then run=nil end
 elseif (a and a~='') or (b and b~='') then save_error=true;profile=Core.new_profile(17029);menu='error';say('Both checkpoints invalid; files preserved')
 else profile=Core.new_profile(17029) end
end
local enemy_tells={}
local enemy_controller=EnemyGenes.new(Core,gd,{get_run=function()return run end,
 on_tell=function(state,event)
  if event=='detached' then enemy_tells[state.handle]=nil else enemy_tells[state.handle]=state end
  if event=='release' and gd.fx_play then
   local h=gd.fx_play(state.family=='cinder' and 'RogueCinderRelease' or 'RogueRimeRelease',1,0,0,8,0,.7,run.seed)
   if h and h>0 then if #fx_handles>=8 then gd.fx_end(table.remove(fx_handles,1),0) end;fx_handles[#fx_handles+1]=h end
  end
 end})
local function cleanup()
 TechAI.clear(2)
 local cleared,why=Rooms.clear(room_visuals)
 enemy_controller:clear();enemy_tells={}
 for _,h in ipairs(platforms) do gd.stage_remove(h) end;platforms={}
 for h in pairs(enemies) do gd.enemy_remove(h) end;enemies={}
 for _,h in ipairs(fx_handles) do gd.fx_end(h,0) end;fx_handles={}
 if gd.parts_clear then gd.parts_clear() end
 Visuals.clear()
 return cleared,why
end
local function held(b,bit) return (b//bit)%2==1 end
local function edge(b,bit) return held(b,bit) and not held(buttons,bit) end
local function pause_menu(which) menu=which;Menus.reset(menu_state,which);gd.pause();gd.input_mask(1,15);Commands.reset(command_state,buttons) end
local function unpause() menu=nil;gd.resume();Commands.reset(command_state,buttons) end
local function host(port) return port==1 and 'player' or port==2 and enemy_host or nil end
local function finish(outcome)
 local h=run.hosts.player;local id=h.slots.assault or h.slots.traversal or h.slots.guard
 if outcome=='success' and #keys(profile.genes)>=128 then id=nil end
 local old_profile,old_run=assert(Core.snapshot(profile)),assert(Core.snapshot(run))
 local result,why=Core.finish(profile,run,outcome,id)
 if not result then say('Finish refused: '..tostring(why));return end
 Feedback.reset(feedback)
 if not save() then
  profile,run=assert(Core.restore(old_profile)),assert(Core.restore(old_run))
  pending_finish=outcome;active=false;transition_error='Run result could not be saved. Retry after fixing storage.';pause_menu('error');return
 end
 pending_finish=nil;active=false;pause_menu('collection');Feedback.finish(feedback,result)
 toast=outcome=='success' and 'Run complete' or 'Run ended';gd.log('roguelite: '..toast)
end
local function room_clear()
 if run.progress.cleared[node.id] then return end
 run.progress.cleared[node.id]=true;save()
 TechAI.clear(2)
 if gd.cpu_mode then gd.cpu_mode(2,'stand') end
 local reward=node.kind=='arena' or node.kind=='boss'
 if reward then pause_menu('reward') end
 Feedback.clear(feedback,{key='clear:'..node.id,title=node.title,reward=reward})
end
local function configure_ai()
 if enemy_host and not run.progress.cleared[node.id] then
  local ok,why=TechAI.configure(2,node.kind=='boss' and 3 or 2,(run.world_seed+node.depth*31)%2147483646+1)
  if not ok then gd.log('roguelite: '..why) end
 else TechAI.clear(2) end
end
local function setup_enemy()
 enemy_host=nil;enemy_gene=nil;enemy_kos=0
 if node.kind=='arena' or node.kind=='boss' then
  enemy_host='enemy_'..node.id
  if not run.hosts[enemy_host] then
   enemy_gene=assert(Core.acquire(run,node.encounter=='guard' and 'rime' or 'cinder'))
   assert(Core.equip(run,enemy_host,'assault',enemy_gene))
   assert(Core.apply_modifier(run,enemy_host,'assault',{id='encounter',stat='potency',add=node.kind=='boss' and 4 or 1}))
  else enemy_gene=run.hosts[enemy_host].slots.assault end
  gd.set_percent(2,0);gd.set_stocks(2,BASE_STOCKS)
  if gd.cpu_mode then gd.cpu_mode(2,run.progress.cleared[node.id] and 'stand' or 'fight') end
  configure_ai()
 else if gd.cpu_mode then gd.cpu_mode(2,'stand') end end
 previous_stocks[2]=BASE_STOCKS
 if node.kind=='traversal' and not run.progress.cleared[node.id] then
  local kind=node.id=='approach' and 'redead' or 'goomba'
  local h,why=gd.spawn_enemy(kind,12,2,{facing=-1})
  if h then
   local identity='enemy_'..node.id..'_1';enemies[h]={host=identity,kind=kind}
   local attached,reason=enemy_controller:attach(h,{host=identity,family=kind=='redead' and 'rime' or 'cinder',slot='assault'})
   if not attached then say('Enemy gene unavailable: '..tostring(reason),'error','enemy-setup') end
  else say('Adventure enemy unavailable: '..tostring(why)..'; passage remains open') end
 end
end
local function complete_entry()
 -- Death/respawn cannot be teleported. Keep simulation running and retry actor
 -- placement before enabling the encounter or freezing a room menu.
 local combat=node.kind=='arena' or node.kind=='boss'
 local placed=pcall(function()
  gd.teleport(1,node.room.spawn.x,node.room.spawn.y+2)
  gd.teleport(2,combat and 28 or 58,2)
 end)
 if not placed then
  pending_ticks=pending_ticks+1
  if pending_ticks>=600 then
   pending_entry=false;cleanup();transition_error='Fighter placement timed out; prior checkpoint retained. Restart to resume.'
   say(transition_error);pause_menu('error')
  end
  return false
 end
 pending_entry=false
 local ok,why=pcall(function()
  gd.set_stocks(1,BASE_STOCKS);previous_stocks[1]=BASE_STOCKS;setup_enemy()
 end)
 if not ok then
  cleanup();transition_error='Encounter setup failed; prior checkpoint retained. Restart to resume.'
  say(transition_error..' '..tostring(why));pause_menu('error');return false
 end
 active=true;save()
 Visuals.update(run,{[1]='player',[2]=enemy_host},true)
 if node.kind=='exit' then finish('success');return true end
 if combat and run.progress.cleared[node.id] and not run.progress.claimed[node.id] then pause_menu('reward')
 elseif node.kind=='rest' then pause_menu('rest') else unpause() end
 say(node.title..' — '..(combat and 'Defeat the fighter by ring-out.' or 'Explore both elevations; exits are at the lane ends.'))
 return true
end
local function enter(id)
 active=false;local cleared,clear_why=cleanup()
 if not cleared then transition_error='Room cleanup refused: '..tostring(clear_why);pause_menu('error');return false end
 move_ids={};movement={0,0};node=manifest.nodes[id];run.progress.room=id
 local art,art_why=Rooms.enter(room_visuals,node)
 if not art then gd.log('roguelite: room visuals unavailable: '..tostring(art_why)) end
 local f=node.room.floor
 local floor,floor_why=gd.stage_add_platform((f.left+f.right)/2,f.y,f.right-f.left,{passthrough=false,ledges=true,draw=not art})
 if not floor then cleanup();transition_error='Room floor could not be constructed: '..tostring(floor_why);pause_menu('error');return false end
 platforms[#platforms+1]=floor
 for _,p in ipairs(node.room.platforms) do
  local h,why=gd.stage_add_platform(p.x,p.y,p.width,{passthrough=p.passthrough,ledges=p.ledges,draw=not art})
  if not h then cleanup();say('Room construction failed: '..tostring(why));pause_menu('error');return false end
  platforms[#platforms+1]=h
 end
 local isolated,isolation_result=false,false
 if gd.stage_isolate then isolated,isolation_result=pcall(gd.stage_isolate,true) end
 if not isolated or not isolation_result then
  cleanup();transition_error='Empty playfield unavailable; install the updated native build.';pause_menu('error');return false
 end
 pending_entry=true;entry_after_tick=tick_serial;pending_ticks=0;transition_error=nil;enemy_host=nil;enemy_gene=nil
 if gd.cpu_mode then gd.cpu_mode(2,'stand') end
 unpause();say('Entering room; waiting for fighters to finish respawning.')
 return true
end
local function begin(resume)
 if save_error then say('Invalid checkpoints preserved; cannot start until repaired');return end
 Feedback.reset(feedback)
 if not resume then
  run=Core.new_run(profile,{stocks=3});run_fighter=assert(Roster.validate(selected_fighter))
  for id,g in pairs(run.genes) do if g.origin==starter then Core.equip(run,'player','assault',nil);Core.equip(run,'player','assault',id);break end end
 end
 -- Resume uses the saved resolved manifest; a new run generates one. A loaded
 -- manifest whose seed does not match the run is stale and regenerated.
 if not (resume and manifest and manifest.seed==run.world_seed) then
  manifest=Dungeon.generate(run.world_seed)
 end
 if not Dungeon.validate(manifest) or not manifest.nodes[run.progress.room] then say('Saved route unavailable');pause_menu('error');return end
 if launched_scene~=Roster.scene(run_fighter) then pending_begin=true;launch(run_fighter);return end
 pending_begin=false;gd.set_percent(1,0);active=false;pending_room=run.progress.room;unpause()
end
local function nearest_target(source,reach)
 local a=gd.player(source);if not a or not node or run.progress.cleared[node.id] then return nil end
 local best,point,distance
 local function candidate(target,q)
  if not q or (target.handle and q.vulnerable==false) then return end
  if target.port and (q.hitlag and q.hitlag>0 or q.action and (q.action<14 or q.action>=178 and q.action<=182)) then return end
  local dx,dy=q.x-a.x,q.y-a.y
  if math.abs(dx)<=reach and math.abs(dy)<=12 and dx*a.facing>=0 then
   local d=dx*dx+dy*dy
   if not distance or d<distance then best,point,distance=target,q,d end
  end
 end
 if enemy_host then local port=source==1 and 2 or 1;candidate({port=port,host=host(port)},gd.player(port)) end
 if source==1 and gd.enemy_state then for handle,e in pairs(enemies) do candidate({handle=handle,host=e.host},gd.enemy_state(handle)) end end
 return best,a,point
end
local function free_to_cast(p)
 return p and p.hitlag==0 and p.action>=14 and p.action<=34 and p.action~=24
end
local function availability(leaf)
 if leaf.action=='restore' then
  local p=gd.player(1)
  return run and run.progress.supplies>0 and free_to_cast(p) and p.percent>0,'Needs damage, a supply and free movement'
 end
 if leaf.action=='thermal_shock' then return false,'Use fire on a marked target' end
 if leaf.action=='loadout' then return node and node.kind=='rest','Placement changes at rest' end
 if leaf.action=='collection' then return node and node.kind=='rest','Save and leave at rest' end
 if leaf.action=='route' then return false,'Approach a world exit + D-pad Down' end
 if leaf.action~='gene' then return false,'Available between rooms' end
 if not run or not active then return false,'No active run' end
 if not free_to_cast(gd.player(1)) then return false,'Finish your current action first' end
 local id=run.hosts.player.slots[leaf.slot]
 if not id or run.genes[id].kind~=leaf.family then return false,'Place this family in '..leaf.slot end
 local a=Core.ability(run,'player',leaf.slot)
 if not a.ready then return false,'Charge '..a.charge..' / '..a.cost end
 if a.action=='step' or a.action=='glide' then
  local p=gd.player(1)
  return gd.impulse~=nil and p and p.hitlag==0 and p.action>=14 and p.action<=34 and p.action~=24 and (a.action=='glide' or not p.airborne),'Needs free movement; Step is grounded'
 end
 local target,p,q=nearest_target(1,a.reach)
 if not target or math.abs(p.x-q.x)>a.reach or math.abs(p.y-q.y)>12 or (q.x-p.x)*p.facing<0 then return false,'Face a nearby opponent' end
 return true
end
local function apply_gene(source,slot)
 if not free_to_cast(gd.player(source)) then return false end
 local who=host(source);local a=Core.ability(run,who,slot);if not a or not a.ready then return false end
 local target,p,q=nearest_target(source,a.reach)
 if a.action~='step' and a.action~='glide' then
  if not target or math.abs(p.x-q.x)>a.reach or math.abs(p.y-q.y)>12 or (q.x-p.x)*p.facing<0 then return false end
 elseif not gd.impulse then return false end
 local before=assert(Core.snapshot(run))
 local action,why=Core.activate(run,who,slot,{target=target and target.host})
 if not action then say(why);return false end
 if action.action=='step' or action.action=='glide' then
  local fighter=gd.player(source)
  if not gd.impulse(source,{x=fighter.facing*2.4,y=action.action=='glide' and fighter.airborne and 1.4 or 0}) then run=assert(Core.restore(before));say('Movement unavailable; charge retained');return false end
 else
  -- gd.hit directly processes a hit, but does not enqueue the collision-loop
  -- on_hit callback. No pending-count filter: it would swallow a later real hit.
  local spec={damage=math.floor(action.damage),angle=action.knockback.angle,kbg=action.knockback.kbg,bkb=action.knockback.bkb,from=source,reach=action.reach}
  local ok
  if target.handle then ok=gd.enemy_hurt and gd.enemy_hurt(target.handle,spec) else ok=gd.hit(target.port,spec) end
  if not ok then run=assert(Core.restore(before));say('Activation refused; charge retained');return false end
 end
 if gd.fx_play then local h=gd.fx_play(action.reaction and 'ThermalShock' or (action.family=='fire' and 'RogueCinderRelease' or 'RogueRimeRelease'),source,0,0,8,0,1,run.seed)
  if h and h>0 then
   if #fx_handles>=8 then gd.fx_end(table.remove(fx_handles,1),0) end
   fx_handles[#fx_handles+1]=h
  end end
 if source==1 then say(action.name..(action.reaction and ' / Thermal Shock' or ''),'release','release:'..slot) end
 feedback_sync(0,false)
 return true
end
local function menu_context()
 return {menu=menu,profile=profile,run=run,starter=starter,node=node,fighters=Roster.list,fighter={name=Roster.name(selected_fighter)},notice=Feedback.view(feedback).notification,retry=pending_finish~=nil,error=transition_error or (save_error and 'Both checkpoints are invalid. Files preserved; repair before continuing.')}
end
local function choose_menu(action)
 if not action then return end
 if menu=='error' then if action.kind=='retry_finish' and pending_finish then finish(pending_finish) end;return end
 if action.kind=='choose_fighter' then roster_state=Roster.new(selected_fighter);pause_menu('fighter');return end
 if action.kind=='fighter_back' then pause_menu('collection');return end
 if action.kind=='fighter_selected' then local previous=selected_fighter;selected_fighter=assert(Roster.validate(action.fighter));if not save() then selected_fighter=previous;return end;pause_menu('collection');say('Selected '..Roster.name(selected_fighter));return end
 if action.kind=='blocked' then say(action.message,'blocked','menu');return end
 if action.kind=='start' or action.kind=='resume' then begin(action.kind=='resume');return end
 if action.kind=='continue' then run.progress.cleared[node.id]=true;if save() then unpause() end;return end
 if action.kind=='leave' then if save() then active=false;cleanup();pause_menu('collection') end;return end
 local before,slot
 if action.kind=='reward' and action.id then
  for k,id in pairs(run.hosts.player.slots) do if id==action.id then slot=k;before=Core.resolve(run,'player',k) end end
 end
 local old_profile,old_run,old_starter=assert(Core.snapshot(profile)),run and assert(Core.snapshot(run)),starter
 local result=Menus.apply(menu_context(),action)
 if not result.ok then say(result.message or 'Action unavailable','blocked','menu');return end
 if result.run then run=result.run end
 if result.starter then starter=result.starter end
 if not save() then
  profile=assert(Core.restore(old_profile));run=old_run and assert(Core.restore(old_run)) or nil;starter=old_starter
  feedback_sync(0,true);return
 end
 if action.kind=='breed' or action.kind=='fuse' then
  Menus.reset(menu_state,menu);menu_state.selected=result.id
 end
 if result.reward_claimed then
  Feedback.dismiss(feedback,'clear:'..node.id)
  if before and slot then Feedback.reward(feedback,{key='reward:'..node.id,name=result.message,before=before,after=Core.resolve(run,'player',slot)})
  else say(result.message,'upgrade','reward:'..node.id) end
  unpause()
 else say(result.message) end
 if run and active then Visuals.update(run,{[1]='player',[2]=enemy_host},true) end
 feedback_sync(0,menu~=nil)
end
load_data()
launch=function(choice)
 choice=choice or selected_fighter
 if ready then active=false;cleanup() end
 local scene=assert(Roster.scene(choice))
 local ok,why=pcall(gd.scene_launch,{mode='vs',p1=scene,p2='fox/c0/cpu9',stage='fd',stocks=99,items='off',time=0})
 if not ok then launching=false;ready=false;say('TBD launch refused: '..tostring(why));return false end
 launching=true;ready=false;launch_seen=false;launched_scene=scene;return true
end
function on_tick()
 tick_serial=tick_serial+1
 gd.input(4,{},1)
 if demo and not ready and not launching then demo=false;launch();return end
 if gd.tbd_request(true) then launch();return end
 local match=gd.match()
 if launching and (not match.active or match.frame<=90) then launch_seen=true end
 if launching and launch_seen and match.active and match.frame>90 and gd.player(1) and gd.player(2) then launching=false;ready=true;gd.set_stocks(1,BASE_STOCKS);gd.set_stocks(2,BASE_STOCKS);if pending_begin then begin(true) else pause_menu(menu=='error' and 'error' or 'collection') end end
 if not ready then return end
 if gd.hud_visible and not match.netplay then gd.hud_visible(false) end
 local pad=gd.pad(1,true);local b=pad and pad.buttons or 0
 if pending_room then
  gd.input_mask(1,15)
  local preloaded,why=Rooms.preload_step(room_visuals)
  if preloaded==nil then
   pending_room=nil;active=false;cleanup();Rooms.release(room_visuals)
   transition_error='Room assets unavailable: '..tostring(why);pause_menu('error')
  elseif preloaded then local id=pending_room;pending_room=nil;enter(id) end
  buttons=b;return
 end
 if active or pending_entry then
  local ok,value=false,false
  if gd.stage_isolate then ok,value=pcall(gd.stage_isolate) end
  if not ok or not value then
   active=false;pending_entry=false;cleanup()
   transition_error='Empty playfield isolation was lost. Restart to resume the preserved checkpoint.'
   pause_menu('error');buttons=b;return
  end
 end
 if pending_entry then gd.input_mask(1,15);if tick_serial>entry_after_tick then complete_entry() end;buttons=b;return end
 feedback_sync(0,menu~=nil)
 if menu then
  gd.input_mask(1,15)
  local mx,my,mb=gd.mouse()
  local input={up=edge(b,gd.buttons.UP),down=edge(b,gd.buttons.DOWN),left=edge(b,gd.buttons.LEFT),right=edge(b,gd.buttons.RIGHT),confirm=edge(b,gd.buttons.A),back=edge(b,gd.buttons.B),mx=mx,my=my,click=mb%2==1 and mouse_buttons%2==0}
  local action=menu=='fighter' and Roster.update(roster_state,{coverage=Bindings.coverage},input) or nil
  if menu~='fighter' then action=Menus.update(menu_state,menu_context(),input) end
  mouse_buttons=mb;choose_menu(action)
 else
  local p=gd.player(1);local travelled=false
  if p and command_state.node=='root' and edge(b,gd.buttons.DOWN) and (run.progress.cleared[node.id] or (node.kind~='arena' and node.kind~='boss' and not next(enemies))) then
   for _,e in ipairs(node.exits) do local anchor=node.room.exit_anchors[e.side]
    if math.abs(p.x-anchor.x)<13 and math.abs(p.y-anchor.y)<10 then enter(e.to);Commands.reset(command_state,b);travelled=true;break end end
  end
  if not travelled then
   local event,mask=Commands.update(command_state,b,availability);gd.input_mask(1,mask)
   if event and event.kind=='execute' then
    if event.action=='gene' then apply_gene(1,event.slot)
    elseif event.action=='restore' and run.progress.supplies>0 then run.progress.supplies=run.progress.supplies-1;gd.set_percent(1,math.max(0,gd.player(1).percent-30));save();say('Supply used: percent -30') end
    if event.action=='loadout' then pause_menu('rest')
    elseif event.action=='collection' then save();active=false;cleanup();pause_menu('collection') end
   elseif event and event.kind=='blocked' then say(event.reason or 'Command unavailable','blocked','command') end
  end
 end
 buttons=b
 feedback_sync(0,menu~=nil)
end
function on_action_change(port,old,new,sub)
 if not sub then Visuals.dirty(port) end
 if active and port<=2 and not sub then
  move_serial[port]=(move_serial[port] or 0)+1;move_ids[port]='p'..port..':'..run.id..':'..run.frame..':'..move_serial[port]
  if new==181 and not menu and (enemy_host or next(enemies)) and not run.progress.cleared[node.id] and host(port) then Core.on_event(run,{host=host(port),kind='defend',move_id=move_ids[port],lineage='direct'}) end -- confirmed GuardSetOff/shieldstun in an active encounter
 end
end
function on_hit(attacker,victim,info)
 if not active or menu or not enemy_host or run.progress.cleared[node.id] then return end
 if attacker and host(attacker) and host(victim) and attacker~=victim and not info.item then Core.on_event(run,{host=host(attacker),kind='direct_hit',move_id=move_ids[attacker] or ('p'..attacker..':initial'),lineage='direct'}) end
end
function on_enemy_hit(event)
 if active and not menu and enemies[event.handle] and event.from==1 and not run.progress.cleared[node.id] then
  Core.on_event(run,{host='player',kind='direct_hit',move_id=move_ids[1] or ('p1:initial'),lineage='direct'})
  feedback_sync(0,false)
 end
end
function on_enemy_defeated(event)
 if active and enemies[event.handle] then enemy_controller:detach(event.handle);enemies[event.handle]=nil;if not next(enemies) then room_clear() end end
end
function on_frame()
 if not active or not ready or menu then return end
 local isolation_ok,isolation_value=false,false
 if gd.stage_isolate then isolation_ok,isolation_value=pcall(gd.stage_isolate) end
 if not isolation_ok or not isolation_value then
  active=false;pending_entry=false;cleanup()
  transition_error='Empty playfield isolation was lost. Restart to resume the preserved checkpoint.'
  pause_menu('error');return
 end
 local match=gd.match();if match.netplay then active=false;ready=false;cleanup();say('Offline mode only');return end
 if match.frame==last_frame then return end;last_frame=match.frame;Core.tick(run,1);enemy_controller:tick();feedback_sync(1,false)
 -- Native monsters can leave the blast zone without taking the stock-defeat
 -- path. Release their bookkeeping without fabricating a defeat event.
 local vanished=false
 if gd.enemy_alive then for handle in pairs(enemies) do
  if not gd.enemy_alive(handle) then enemy_controller:detach(handle);gd.enemy_remove(handle);enemies[handle]=nil;vanished=true end
 end end
 if vanished and not next(enemies) and node.kind=='traversal' then room_clear() end
 for port=1,2 do local p=gd.player(port)
  if p and p.stocks< (previous_stocks[port] or BASE_STOCKS) then
   if port==1 then run.stocks=run.stocks-1;if run.stocks<=0 then finish('failure');return end;say('Stock lost — '..run.stocks..' remaining');gd.set_stocks(1,BASE_STOCKS);save()
   elseif enemy_host and not run.progress.cleared[node.id] then enemy_kos=enemy_kos+1
    if enemy_kos >= (node.kind=='boss' and 2 or 1) then room_clear() else gd.set_stocks(2,BASE_STOCKS);gd.set_percent(2,0);say('Champion stock cleared; one remains') end
   end
  end
  if p then previous_stocks[port]=p.stocks<BASE_STOCKS and BASE_STOCKS or p.stocks end
 end
 for port=1,2 do local who=host(port);local p=gd.player(port)
  if who and p then
   if not p.airborne and math.abs(p.vx or 0)>0.6 then movement[port]=movement[port]+1;if movement[port]%60==0 then Core.on_event(run,{host=who,kind='move',move_id=who..':move:'..run.frame,lineage='direct'}) end end
  end
 end
 if enemy_host and not run.progress.cleared[node.id] then
  if run.frame%30==0 and gd.cpu_technical and not TechAI.status(2).enabled then configure_ai() end
  apply_gene(2,'assault')
 end
 Visuals.update(run,{[1]='player',[2]=enemy_host})
 -- Fighter coloration hooks remain optional; geometry and gameplay do not depend on FX.
end
local function text(x,y,s,role,color) gd.kit.text(x,y,s,role or 'caption',color or 'bone') end
function on_draw()
 if not ready then return end
 if pending_room then gd.kit.text(24,36,'Preparing room','caption','bone');return end
 if menu then
  if menu=='fighter' then Roster.draw(roster_state,{coverage=Bindings.coverage}) else Menus.draw(menu_state,menu_context()) end
 else
  gd.fill(12,12,188,24,0x101a2c90);gd.kit.text(20,29,node.title,'caption','gold','left',{max_w=172,shear=0})
  Visuals.tree(Commands.view(command_state,availability))
  for _,enemy in ipairs(enemy_controller:states()) do
   if enemy.phase=='telegraph' or enemy.phase=='ready' then
    local x,y=gd.project(enemy.x,enemy.y+13,0)
    if x and y then gd.kit.text(math.max(4,math.min(550,x-38)),math.max(16,math.min(390,y)),enemy.phase=='telegraph' and 'CASTING' or 'READY','caption',enemy.family=='rime' and 'bone' or 'gold','left',{max_w=82,shear=0}) end
   end
  end
  for _,e in ipairs(node.exits) do local a=node.room.exit_anchors[e.side];local x,y=gd.project(a.x,a.y+8,0);if x and y then text(x,y,e.label,'caption','gold') end end
 end
 local player=gd.player(1);local opponent=enemy_host and not run.progress.cleared[node.id] and gd.player(2)
 Feedback.draw(feedback,{hud=not menu,notifications=true,compact_menu=menu~=nil,player={percent=player and player.percent or 0,lives=run and run.stocks or 0,supplies=run and run.progress.supplies or 0},opponent=opponent and {label=node.kind=='boss' and 'CHAMPION' or 'OPPONENT',percent=opponent.percent,lives=(node.kind=='boss' and 2 or 1)-enemy_kos}})
end
function on_match_end() pending_room=nil;ready=false;active=false;pending_entry=false;enemy_controller:clear();platforms={};enemies={};fx_handles={};enemy_host=nil;Feedback.reset(feedback);Rooms.reset(room_visuals) end
function on_unload() pending_room=nil;if gd.stage_isolate then pcall(gd.stage_isolate,false) end;if ready then cleanup();if gd.cpu_mode then gd.cpu_mode(2,'fight') end end;Rooms.release(room_visuals);if gd.hud_visible then pcall(gd.hud_visible,true) end;gd.release_pad(4);gd.input_mask(1,0);gd.resume() end
-- Read-only inspection for integration tests and script-console diagnostics.
function roguelite_state() return {loading=pending_room~=nil,ready=ready,active=active,menu=menu,profile=profile,run=run,node=node and node.id,toast=toast,command=command_state.node,fighter=selected_fighter,run_fighter=run_fighter,menu_view=menu and (menu=='fighter' and Roster.view(roster_state,{coverage=Bindings.coverage}) or Menus.view(menu_state,menu_context())),feedback=Feedback.view(feedback)} end
gd.command('rogue_state',function()
 local a=run and Core.ability(run,'player','assault')
 -- Versioned diagnostics: runtime_ready is engine/scene readiness, ability_ready
 -- is the assault gene's charge state. The legacy `ready` fields stay for
 -- existing readers until live_acceptance has fully migrated.
 gd.log('rogue_state ready='..tostring(ready)..' active='..tostring(active)..' menu='..tostring(menu)..' room='..tostring(node and node.id)..' stocks='..tostring(run and run.stocks)..' charge='..tostring(a and a.charge)..' runframe='..tostring(run and run.frame)..' saveerror='..tostring(save_error)..' command='..command_state.node..' cleared='..tostring(run and node and run.progress.cleared[node.id]==true)..' claimed='..tostring(run and node and run.progress.claimed[node.id]==true)..' collection='..tostring(profile and #keys(profile.genes))..' finished='..tostring(profile and #keys(profile.finished))..' outcome='..tostring(run and run.status)..' cost='..tostring(a and a.cost)..' ready='..tostring(a and a.ready)..' remaining='..tostring(a and a.remaining)..' reach='..tostring(a and a.reach)..' diag_version=1 runtime_ready='..tostring(ready)..' ability_ready='..tostring(a and a.ready))
end)
gd.command('rogue_start',function() if not ready then demo=true end end)
gd.command('rogue_enemies',function()
 for _,e in ipairs(enemy_controller:states()) do
  local actor=enemies[e.handle]
  local native=gd.enemy_state and gd.enemy_state(e.handle)
  gd.log('rogue_enemy handle='..e.handle..' kind='..tostring(actor and actor.kind)..' host='..e.host..' x='..tostring(e.x)..' y='..tostring(e.y)..' phase='..e.phase..' charge='..e.charge..' cost='..e.cost..' damage='..tostring(native and native.damage)..' hits='..tostring(native and native.hits)..' received='..tostring(native and native.received)..' vulnerable='..tostring(native and native.vulnerable))
 end
end)
gd.command('rogue_bindings',function() for port,why in pairs(Visuals.status()) do gd.log('rogue_binding port='..port..' '..tostring(why)) end end)
gd.command('rogue_ai',function()
 local s=TechAI.status(2)
 gd.log('rogue_ai enabled='..tostring(s.enabled)..' skill='..tostring(s.skill)..' opportunities='..tostring(s.opportunities)..' lcancels='..tostring(s.lcancel_inputs)..' techs='..tostring(s.tech_inputs)..' misses='..tostring(s.missed_decisions)..' policy='..tostring(s.policy))
end)
gd.command('rogue_build',function()
 if not run then return end
 for _,id in ipairs(keys(run.genes)) do
  local owner,slot='unplaced','none'
  for who,h in pairs(run.hosts) do for where,used in pairs(h.slots) do if used==id then owner,slot=who,where end end end
  gd.log('rogue_gene id='..id..' family='..run.genes[id].kind..' host='..owner..' slot='..slot)
 end
end)
gd.command('rogue_menu',function()
 if not menu then gd.log('rogue_menu closed');return end
 local view=menu=='fighter' and Roster.view(roster_state,{coverage=Bindings.coverage}) or Menus.view(menu_state,menu_context())
 local focus=menu=='fighter' and roster_state.focus or menu_state.focus
 gd.log('rogue_menu focus='..tostring(focus or (view.controls[1] and view.controls[1].id))..' section='..tostring(view.section)..' wrap='..tostring(menu~='fighter'))
 for _,c in ipairs(view.controls) do gd.log('rogue_control id='..c.id..' x='..c.x..' y='..c.y..' w='..c.w..' h='..c.h..' enabled='..tostring(c.enabled~=false)) end
end)
