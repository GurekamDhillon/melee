-- Installer prepends lexical Core, Dungeon, DungeonV1 and Commands modules. No
-- require. Initial offline vertical slice: bounded FD rooms and ordinary native
-- CPU AI.
local profile,run,manifest,node
-- v2 route state kept explicit and separate from the v1 manifest: a validated
-- adapter view plus its own schema-2 progress record. Non-nil only when a saved
-- v2 route has been loaded and re-validated; never derived from a v1 seed.
local route=nil
-- Live v2 campaign controller (RuntimeCampaign). Non-nil once a validated v2
-- route is being played; nil for v1 runs.
local campaign=nil
local orphan_campaigns={}
-- Central authority for the current run: a real Core/menu transaction installs a
-- fresh run table, so the live campaign is rebound to that exact table (never a
-- retired owner) to keep saves and actor/host references coherent.
local function rebind_current_run(r)
 run=r
 if campaign and r then campaign:replace_run(r) end
end
-- New-run replacement gate: while `retiring` owns live resources, promotion is
-- paused and blocked until its cleanup actually succeeds.
local retiring=nil
local retire_attempts=0
local pending_promote=nil
local v2_paused=false
local reward_ui=nil
local v2_previous={}
-- Sustained grounded-movement counter used as truthful tutorial evidence.
local v2_move_frames=0
local ready,launching,active=false,false,false
local launch_seen=false
local pending_entry=false
local tick_serial,entry_after_tick=0,0
local pending_room=nil
local pending_ticks=0
local transition_error,pending_finish=nil,nil
local campaign_recovery_error=false
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
-- v2 route services (pure). Live generation still uses Dungeon v1 until a
-- template set is certified; these are wired for creation/diagnostics.
local topology=Topology.new(RoomCatalogue,EncounterCatalogue,Progression,Rng)
local adapter=Adapter.new(RoomCatalogue,RoomRecipes)
local routes=Route.new({topology=topology,adapter=adapter,progress=Progress,progression=Progression,encounters=EncounterCatalogue})
local previous_stocks={};local move_serial={0,0};local move_ids={}
local movement={0,0}
local enemy_host,enemy_gene,enemy_kos=nil,nil,0
local BASE_STOCKS=99
local generation,save_error=0,false
-- Slots holding an unsupported future checkpoint; never overwritten by a save.
local protected={}
local starter='g1'
local config=gd.data_read('config.txt') or ''
local demo=config:find('demo=true',1,true)~=nil
local slots={'assault','traversal','guard'}
-- Compact presentation (hud_layout + onboarding + loadout-derived commands)
-- composed into the calls main already makes. It owns no run or save state; see
-- runtime_presentation.lua. The canvas is the port's documented fixed 640x480
-- virtual screen, never a window-derived size.
local presentation=Presentation.new(gd,{Hud=Hud,Onboarding=Onboarding,Commands=Commands,Feedback=Feedback},
 {width=640,height=480,reduced=config:find('reduced_motion=true',1,true)~=nil,feedback=feedback,
  log=function(m) if gd.log then gd.log('roguelite: '..m) end end})
local function say(s,kind,key)
 toast=s;gd.log('roguelite: '..s)
 Feedback.notify(feedback,{key=key or s,kind=kind or 'info',title=s})
end
-- Rebuild the recursive command tree from the run's real installed loadout. The
-- current button word is passed through so a held Up keeps its latch across the
-- rebuild and can never become a fresh root taunt.
--
-- Both route versions install the same loadout-derived grammar.
local function sync_loadout() presentation:rebuild(command_state,run,buttons) end
-- The most recent real command-tree edge, for read-only diagnostics. main never
-- fabricates one: it is written only where Commands.update actually returns.
local last_command_event=nil
local function record_command_event(event)
 if event then last_command_event={kind=event.kind,action=event.action,slot=event.slot,reason=event.reason,node=command_state.node} end
end
-- Route depth drives the tutorial's first-rooms window; it is read from the
-- resolved adapter node, never fabricated.
local function tutorial_room(depth) presentation:set_room((tonumber(depth) or 0)+1) end
local function feedback_sync(frames,paused)
 local abilities={}
 if run and run.hosts.player then for _,slot in ipairs(slots) do abilities[slot]=Core.ability(run,'player',slot) end end
 Feedback.update(feedback,abilities,frames or 0,paused,run and run.hosts.player and run.hosts.player.slots)
 presentation:notice_charges(abilities)
end
local function count_keys(t) local n=0 if type(t)=='table' then for _ in pairs(t) do n=n+1 end end return n end
local function keys(t) local a={} for k in pairs(t) do a[#a+1]=k end table.sort(a) return a end
-- Keep Core's run mirrors synchronized with the schema-2 progress record so the
-- saved run and route cannot disagree. Route.validate_save checks these exactly;
-- called before every v2 write and after every v2 load.
local function route_mirror(r,record)
 r.progress.room=record.current_room
 r.progress.supplies=record.supplies
 r.stocks=record.lives
 local cleared,claimed={},{}
 for id,state in pairs(record.objectives) do if state=='done' then cleared[id]=true end end
 for id in pairs(record.claimed) do local room=id:match('^reward:(.+)$');if room then claimed[room]=true end end
 r.progress.cleared,r.progress.claimed=cleared,claimed
end
-- Schema-1 progress records never tracked opened locks, finite pickup claims or
-- encounter KO counts. Upgrading them with empty maps is safe only when the
-- route cannot carry that history; otherwise preserve and refuse rather than
-- silently grant or duplicate finite pickups and locks.
local function progress_migration_safe(m,record)
 if m.locks~=nil and type(m.locks)~='table' then return false,'route lock table is malformed' end
 if type(m.rooms_by_id)~='table' then return false,'route rooms are malformed' end
 local pickups,gates=false,false
 for _,room in pairs(m.rooms_by_id) do
  if type(room)=='table' then
   if room.pickups and next(room.pickups) then pickups=true end
   if room.grants_consumable~=nil then pickups=true end
  end
 end
 for _,lock in pairs(m.locks or {}) do
  if type(lock)=='table' and lock.kind=='consumable_key' then gates=true end
 end
 if pickups and record.pickups==nil then return false,'route has finite pickups with no saved claim history' end
 if gates and record.opened==nil then return false,'route has consumable gates with no saved open history' end
 return true
end
local function checkpoint(raw)
 if type(raw)~='string' or #raw>1048700 then return nil end
 -- TBD3: resolved manifest (+ optional progress) with an integrity checksum.
 -- Expected inner versions are declared so a future manifest/progress schema is
 -- refused and preserved, not returned as if interchangeable. A v1 dungeon
 -- manifest carries no schema_version and is treated as schema 1. The decoder
 -- reports an explicit preserve flag for newer-than-supported versions so this
 -- loader never has to guess from reason text.
 local opts={manifest_schemas={[1]=true,[2]=true},progress_versions={[1]=true,[2]=true}}
 local decoded,dwhy,dpreserve=Checkpoint.decode(raw,Core,Codec,opts)
 if not decoded and dpreserve then return nil,dwhy,true end
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
  local m=decoded.manifest
  -- A run must carry its resolved manifest; do not let resume regenerate one.
  if r and not m then return nil,'run is missing its resolved manifest' end
  if m and m.schema_version==2 then
   -- v2 route: manifest and a separate schema-2 progress record are both
   -- mandatory. Missing or unsupported sections are refused, never treated as a
   -- legacy or new run. An unknown generator version is a preserved future
   -- record, detected here rather than through a free-text validator reason.
   if m.generator_version~=Topology.version then
    return nil,'unsupported generator version '..tostring(m.generator_version)..'; preserved',true
   end
   if not r then return nil,'v2 route is missing its run' end
   local record=decoded.progress
   if not record then return nil,'v2 route is missing its progress record' end
   if record.version==1 then
    -- Explicit, supported migration only. Schema 1 never tracked opened locks,
    -- finite pickup claims or encounter KOs: migrate with empty maps only when
    -- the route has no such history to lose, otherwise preserve and refuse.
    local safe,mwhy=progress_migration_safe(m,record)
    if not safe then return nil,'progress migration refused: '..tostring(mwhy)..'; preserved',true end
    local migrated,why=Legacy.migrate_progress(record,Progress)
    if not migrated then return nil,'progress migration refused: '..tostring(why) end
    record=migrated
   end
   local ok,res,why=pcall(routes.resume,routes,r,m,record)
   if not ok then return nil,'route resume error: '..tostring(res) end
   if not res then return nil,why end
   return {generation=decoded.generation,profile=p,run=r,selected=selected,fighter=played,route=res}
  end
  -- A progress record without a v2 manifest is inconsistent.
  if decoded.progress then return nil,'progress record without a v2 route' end
  return {generation=decoded.generation,profile=p,run=r,selected=selected,fighter=played,manifest=m}
 end
 -- Legacy TBD2/TBD1 envelope. No separate progress was ever stored.
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
 -- Reconstruct the historical route with the frozen v1 generator so an old run
 -- keeps its original room ids rather than a newer generator's layout.
 local man
 if r then
  local generated,result=pcall(DungeonV1.generate,r.world_seed)
  if not generated or type(result)~='table' then return nil end
  local validated,valid,why=pcall(DungeonV1.validate,result)
  if not validated or not valid or not result.nodes[r.progress.room] then return nil end
  man=result
 end
 return {generation=g,profile=p,run=r,selected=selected,fighter=played,manifest=man}
end
local function save(override_route, override_run)
 if save_error then say('Save disabled: repair the preserved invalid checkpoint first','error','save');return false end
 -- Stage Core mirrors on a deep copy; live in-memory state is committed only
 -- after a durable, validated write so a refused save cannot leave the run
 -- altered. A v2 write carries the exact validated resolved manifest and its
 -- own progress record. save() accepts explicit overrides so a v2 transaction
 -- can persist a staged route/run without first mutating the live references.
 local saving_run = override_run or run
 local saving_route = override_route or route
 local staged=nil
 if saving_run then
  local rs,rwhy=Core.snapshot(saving_run)
  if not rs then say('Run save refused: '..tostring(rwhy),'error','save');return false end
  local restored=Core.restore(rs)
  if not restored then say('Run save refused: working copy failed','error','save');return false end
  staged=restored
  if saving_route then route_mirror(staged,saving_route.progress) end
 end
 local p,why=Core.snapshot(profile);if not p then say('Save refused: '..tostring(why),'error','save');return false end
 local r=''
 if staged then r,why=Core.snapshot(staged);if not r then say('Run save refused: '..tostring(why),'error','save');return false end end
 -- Persist resolved data so a later catalogue/generator change cannot relocate
 -- a saved doorway or recompute saved progress. v2 carries the route's own
 -- manifest and schema-2 progress; v1 keeps the existing resolved manifest only.
 local mtext,ptext
 if staged then
  if saving_route then
   local ewhy
   mtext,ewhy=Codec.encode(saving_route.manifest)
   if not mtext then say('Route manifest encode refused: '..tostring(ewhy),'error','save');return false end
   ptext,ewhy=Codec.encode(saving_route.progress)
   if not ptext then say('Route progress encode refused: '..tostring(ewhy),'error','save');return false end
  else
   local man=(manifest and manifest.seed==staged.world_seed) and manifest or Dungeon.generate(staged.world_seed)
   local encoded,ewhy=Codec.encode(man)
   if not encoded then say('Manifest encode refused: '..tostring(ewhy),'error','save');return false end
   mtext=encoded
  end
 end
 local next_generation=generation+1;local file=next_generation%2==0 and 'checkpoint-a.txt' or 'checkpoint-b.txt'
 -- Never overwrite a preserved unsupported future checkpoint.
 if protected[file] then
  local other=file=='checkpoint-a.txt' and 'checkpoint-b.txt' or 'checkpoint-a.txt'
  if protected[other] then say('Save disabled: a preserved checkpoint must be removed first','error','save');return false end
  gd.log('roguelite: saving to '..other..' to preserve a protected checkpoint')
  file=other
 end
 local metadata=assert(Roster.encode(selected_fighter))..assert(Roster.encode(run_fighter))
 local text
 do
  local ok,res=pcall(Checkpoint.encode,{generation=next_generation,profile=p,run=r~='' and r or nil,manifest=mtext,roster=metadata,progress=ptext})
  if not ok then say('Checkpoint encode refused: '..tostring(res),'error','save');return false end
  text=res
 end
 -- Validate the staged generation BEFORE any disk write: a semantic refusal
 -- must never clobber the last usable slot, especially when the other slot is
 -- protected and this save was redirected into it.
 local vok,vres,vwhy=pcall(checkpoint,text)
 if not (vok and vres) then
  say('Save refused: '..tostring(vwhy or 'checkpoint failed validation')..'; previous checkpoint retained','error','save');return false
 end
 -- A/B is insufficient when the sibling slot is preserved: a raw partial
 -- write could destroy the only supported checkpoint. Require the native
 -- atomic helper for every checkpoint mutation, including ordinary A/B writes.
 local write=gd.data_write_atomic
 if type(write)~='function' then
  say('Save refused: native atomic checkpoint writer unavailable; previous checkpoint retained','error','save');return false
 end
 local ok,result=pcall(write,file,text)
 local read_ok,readback=pcall(gd.data_read,file)
 if not (ok and result~=false and read_ok and readback==text) then
  say('Save write/readback failed; previous checkpoint retained','error','save');return false
 end
 -- Commit the live references only now that the bytes are validated and durable.
 if override_run then run=override_run end
 if override_route then route=override_route end
 if run and route then route_mirror(run,route.progress) end
 generation=next_generation
 return true
end
-- Validate a staged v2 route/run pair with the pure service, then persist it
-- through save(). The route table belongs to the campaign that owns it; only on
-- a durable write is its progress advanced, so a refused save leaves the
-- previous room, key spend and visited state intact. `route_table` is never
-- replaced, so campaign identity is preserved across writes.
local function save_route(route_table,progress2,run_override)
 local base=run_override
 local staged_run=base and Core.restore(assert(Core.snapshot(base))) or nil
 if staged_run then route_mirror(staged_run,progress2) end
 local ok,valid,why=pcall(routes.validate_save,routes,staged_run or base,route_table.manifest,progress2)
 if not ok then return false,'route validation error: '..tostring(valid) end
 if not valid then return false,why end
 local saved_progress=route_table.progress
 route_table.progress=progress2
 local wrote=save(route_table,run_override)
 if not wrote then route_table.progress=saved_progress;return false,'checkpoint save refused' end
 route_table.progress=progress2
 return true
end
local function classify(name,raw)
 if type(raw)~='string' or raw=='' then return nil end
 local ok,data,why,preserve=pcall(checkpoint,raw)
 if not ok then gd.log('roguelite: checkpoint '..name..' threw: '..tostring(data));return nil end
 if data then return data end
 if preserve then protected[name]=why end
 return nil
end
local function load_data()
 protected={}
 route=nil
 local a,b=gd.data_read('checkpoint-a.txt'),gd.data_read('checkpoint-b.txt')
 local ca,cb=classify('checkpoint-a.txt',a),classify('checkpoint-b.txt',b)
 local best=ca;if cb and (not best or cb.generation>best.generation) then best=cb end
 if best then profile,run,generation=best.profile,best.run,best.generation;selected_fighter,run_fighter=best.selected,best.fighter
   manifest=best.manifest
   -- A validated v2 route is retained intact for later orchestration; a v1
   -- manifest continues to drive the live fixed-room runtime.
   route=best.route
   if run and (profile.finished[run.id] or run.status~='active') then run=nil;route=nil end
   if route and run then route_mirror(run,route.progress) end
 elseif (a and a~='') or (b and b~='') then
   save_error=true;profile=Core.new_profile(17029);menu='error'
   if protected['checkpoint-a.txt'] or protected['checkpoint-b.txt'] then say('A preserved checkpoint could not be used; files left intact. Remove it to continue')
   else say('Both checkpoints invalid; files preserved') end
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
 if campaign then
  local ok,why=campaign:teardown()
  if not ok then gd.log('roguelite: v2 teardown pending: '..tostring(why))
  else campaign:reset(false);campaign=nil end
 end
 -- Retry any orphaned previous owner; keep whatever the engine still refuses.
 local pending_orphans={}
 for _,c in ipairs(orphan_campaigns) do
  local clean,why=c:teardown({drop_pending=true})
  if clean then c:reset(false) else pending_orphans[#pending_orphans+1]=c;gd.log('roguelite: orphan v2 cleanup pending: '..tostring(why)) end
 end
 orphan_campaigns=pending_orphans
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
local function pause_menu(which) menu=which;Menus.reset(menu_state,which);gd.pause();gd.input_mask(1,15);Commands.reset(command_state,buttons);if which=='error' then presentation:release_hud() end end
local function unpause() menu=nil;gd.resume();Commands.reset(command_state,buttons) end
local function host(port)
 if port==1 then return 'player' end
 if port==2 then
  if campaign then local e=campaign:active_entity();return e and e.host or nil end
  return enemy_host
 end
 return nil
end
local function finish(outcome)
 campaign_recovery_error=false
 local h=run.hosts.player;local id=h.slots.assault or h.slots.traversal or h.slots.guard
 -- A full collection must never silently elect "no export". main defers instead:
 -- Core keeps the earned gene on the finish record, so it survives the run, a
 -- relaunch and a refused save, and the player claims or declines it from the
 -- collection. main never chooses on the player's behalf.
 local finished_run_id=run.id
 local old_profile,old_run=assert(Core.snapshot(profile)),assert(Core.snapshot(run))
 local result,why=Core.finish(profile,run,outcome,id,{defer_export=true})
 if not result then say('Finish refused: '..tostring(why));return end
 Feedback.reset(feedback)
 if not save() then
  profile=assert(Core.restore(old_profile));rebind_current_run(assert(Core.restore(old_run)));sync_loadout()
  pending_finish=outcome;active=false;transition_error='Run result could not be saved. Retry after fixing storage.';pause_menu('error');return
 end
 pending_finish=nil;active=false;presentation:release_hud();pause_menu('collection');Feedback.finish(feedback,result)
 toast=outcome=='success' and 'Run complete' or 'Run ended';gd.log('roguelite: '..toast)
 if result.deferred then
  -- Honest: the gene was NOT lost, it is waiting on the finish record. The plain
  -- completion notice outranks an info notice and would read as "everything was
  -- kept", so retire it in favour of the one thing the player must act on.
  gd.log('roguelite: export deferred for '..tostring(result.deferred.id)..' on '..tostring(finished_run_id))
  Feedback.dismiss(feedback,'finish')
  Feedback.notify(feedback,{key='export:pending:'..tostring(finished_run_id),kind='blocked',
   title='Run complete / export pending',
   detail=tostring(result.deferred.id)..' could not join the collection (128/128). Claim or decline it from the collection.',ttl=900})
 end
end
-- Live v2 campaign controller, created the first time a validated v2 route is
-- loaded or created. All persistence goes back through save_route, so this
-- module never writes disk on its own.
-- The physical lifecycle needs the stage/room engine surface. An older build
-- without it is refused gracefully (collection review stays available) rather
-- than asserting inside the room constructor.
local function v2_engine_ready()
 return type(gd.stage_isolate)=='function' and type(gd.stage_add_platform)=='function'
  and type(gd.stage_add_line)=='function' and type(gd.stage_remove)=='function'
  and type(gd.teleport)=='function' and type(gd.player)=='function'
end
-- v2 builds only in the isolated empty stage; main owns the symmetry.
local function v2_isolate()
 if type(gd.stage_isolate)~='function' then return false end
 local ok,value=pcall(gd.stage_isolate,true)
 return ok and value==true
end
-- Pause ownership for v2 transitions/recovery. main keeps the engine paused for
-- the whole request/build/persist/commit/recovery sequence and only resumes when
-- the campaign is running again, restoring the ordinary command tree.
local function v2_set_paused(want)
 if want and not v2_paused then
  if type(gd.pause)=='function' then pcall(gd.pause) end
  v2_paused=true
 elseif not want and v2_paused then
  v2_paused=false
  unpause()
 end
end
-- Create a campaign bound to the exact route/run tables it will own for life.
-- It is never cached: a replacement run builds a fresh instance so no old owner
-- callback can reach the new run.
local function ensure_campaign(route_arg,run_arg)
 local owned_route=route_arg or route
 local owned_run=run_arg or run
 return RuntimeCampaign.new(gd,{Core=Core,Progress=Progress,RouteMap=RouteMap,Rooms=Rooms,
  RuntimeRooms=RuntimeRooms,RuntimeEncounters=RuntimeEncounters,RuntimeRewards=RuntimeRewards,
  EnemyGenes=EnemyGenes,EnemyCatalogue=EnemyCatalogue,TechAI=TechAI,GeneCatalogue=GeneCatalogue,
  routes=routes,Encounters=EncounterCatalogue},
  {route=owned_route,run=owned_run,save_route=save_route,
   stock_baseline=BASE_STOCKS,
   finish=function(outcome)finish(outcome) end,say=say,
   on_event=function(e)if gd.log then gd.log('roguelite_v2 '..tostring(e.kind))end end,
   fighter_family='cinder',fighter_slot='assault',
   spawn_point=function()return {x=28,y=2}end})
end
local function v2_ready() return campaign~=nil and campaign.phase~='idle' end
-- A campaign error that has actually recovered is retired with the error: the run
-- is playable again, and a page left with no usable control is not a recovery.
local function v2_recovered()
 if not campaign_recovery_error or pending_finish or not campaign then return false end
 local retired=campaign.phase=='idle' and not campaign:blocked() and campaign.tx==nil
 if not campaign:running() and not retired then return false end
 campaign_recovery_error=false
 transition_error=nil
 if menu=='error' then
  if retired then active=false;pause_menu('collection') else unpause() end
 end
 return true
end
-- Release a previous owner deliberately. A refused native cleanup keeps the
-- campaign retained so its handles are retried at the next paused promotion
-- tick or confirmed teardown instead of being forgotten.
local function retire_campaign(c)
 if not c then return true end
 local clean,why=c:teardown({drop_pending=true})
 if clean then c:reset(false);return true end
 gd.log('roguelite: previous v2 room cleanup pending: '..tostring(why))
 for _,x in ipairs(orphan_campaigns) do if x==c then return false end end
 orphan_campaigns[#orphan_campaigns+1]=c
 return false
end
-- Complete the promotion of the already-durably-saved new run. Called only once
-- any previous owner has actually released its resources.
local function finish_promotion(result)
 if not v2_isolate() then transition_error='Empty playfield unavailable; install the updated native build.';pause_menu('error');return end
 if launched_scene~=Roster.scene(run_fighter) then pending_begin=true;launch(run_fighter);return end
 pending_begin=false;gd.set_percent(1,0);active=false
 previous_stocks[1]=BASE_STOCKS;previous_stocks[2]=BASE_STOCKS
 v2_set_paused(true)
 local ok,why=campaign:enter_current()
 if not ok then v2_set_paused(false);transition_error=why;pause_menu('error');return end
 sync_loadout()
 if result and result.view and result.manifest then
  say(result.view.nodes[result.manifest.start_room].title or 'New route','info','v2-start')
 end
end
-- Compact v2 reward overlay. It resolves from the campaign's RuntimeRewards
-- preview (supported targets only) and commits through the transactional
-- reward_commit; a refused save leaves the run and any native effect unchanged.
local function open_v2_reward(node)
 if not campaign or not node then return end
 local preview,why=campaign:reward_preview(node)
 if not preview then say(tostring(why or 'Reward unavailable'),'error','reward');campaign.reward_room=nil;return end
 if not preview.requires_target then
   local result,why=campaign:reward_commit(node,nil)
   campaign.reward_room=nil
   if not result then say('Reward refused: '..tostring(why),'blocked','reward')
   else presentation:observe({kind='reward'});sync_loadout() end
   return
  end
 local options={}
 for _,target in ipairs(preview.targets or {}) do if target.supported then options[#options+1]=target end end
 if #options==0 then say('No eligible reward target','blocked','reward');campaign.reward_room=nil;return end
 reward_ui={room=node.id,options=options,focus=1}
 gd.pause();gd.input_mask(1,15)
end
local function handle_v2_reward(b)
 gd.input_mask(1,15)
 local ui=reward_ui
 if edge(b,gd.buttons.B) then reward_ui=nil;campaign.reward_room=nil;gd.resume();return end
 if edge(b,gd.buttons.DOWN) then ui.focus=ui.focus%#ui.options+1 end
 if edge(b,gd.buttons.UP) then ui.focus=(ui.focus-2)%#ui.options+1 end
 if edge(b,gd.buttons.A) then
   local result,why=campaign:reward_commit(campaign:current_node(),ui.options[ui.focus].id)
   if result then reward_ui=nil;campaign.reward_room=nil;gd.resume();presentation:observe({kind='reward'});sync_loadout()
   else say('Reward refused: '..tostring(why),'error','reward') end
 end
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
 tutorial_room(node and node.depth)
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
 campaign_recovery_error=false
 if save_error then say('Invalid checkpoints preserved; cannot start until repaired');return end
 Feedback.reset(feedback)
 if resume and route then
  -- A validated v2 route is live. Enter its current room through the campaign
  -- controller, which owns the bounded physical lifecycle. An engine without the
  -- room/stage surface refuses gracefully instead of asserting.
  if not v2_engine_ready() then
   say('Saved v2 route loaded; traversal needs the updated native build. Collection review remains available.','blocked','v2-route')
   return
  end
  if not v2_isolate() then transition_error='Empty playfield unavailable; install the updated native build.';pause_menu('error');return end
  campaign=ensure_campaign(route,run)
  -- Resuming installs this run's loadout as the live command tree.
  sync_loadout()
  if launched_scene~=Roster.scene(run_fighter) then pending_begin=true;launch(run_fighter);return end
  pending_begin=false;gd.set_percent(1,0);active=false
  previous_stocks[1]=BASE_STOCKS;previous_stocks[2]=BASE_STOCKS
  v2_set_paused(true)
  local ok,why=campaign:enter_current()
  if not ok then v2_set_paused(false);transition_error=why;pause_menu('error');return end
  return
 end
 if not resume then
  -- New-run dispatch probes the production v2 certification gate on a staged
  -- profile/run and only promotes it after the pair is durably saved. Every live
  -- reference (and the previous room owner) is snapshotted first, so a refused
  -- initial save restores them byte-equivalently instead of advancing
  -- profile.next_run or replacing the run/fighter/route.
  local old_profile,old_run,old_route,old_campaign,old_fighter=profile,run,route,campaign,run_fighter
  local staged_profile=Core.restore(assert(Core.snapshot(profile)))
  local staged_run=Core.new_run(staged_profile,{stocks=3})
  local created,result=pcall(routes.create,routes,staged_run)
  if created and result then
   profile,run=staged_profile,staged_run
   run_fighter=assert(Roster.validate(selected_fighter))
   for id,g in pairs(run.genes) do if g.origin==starter then Core.equip(run,'player','assault',nil);Core.equip(run,'player','assault',id);break end end
   route={manifest=result.manifest,view=result.view,progress=result.progress}
   campaign=ensure_campaign(route,run)
   sync_loadout()
   if not (v2_isolate() and save()) then
    -- Restore the exact previous references; the previous owner was never
    -- retired, so its room ownership survives the refusal.
    profile,run,route,campaign,run_fighter=old_profile,old_run,old_route,old_campaign,old_fighter
    sync_loadout()
    v2_set_paused(false)
    transition_error='New v2 route could not be saved; the previous run is unchanged.';pause_menu('error');return
   end
   -- Durable: the previous owner may now be released, but promotion stays
   -- paused and blocked until its resources are actually gone.
   if old_campaign and old_campaign~=campaign then
    retiring=old_campaign;retire_attempts=0;pending_promote={result=result}
    v2_set_paused(true)
    return
   end
   finish_promotion(result);return
  end
  gd.log('roguelite: v2 route unavailable: '..tostring(created and result))
  -- Falling back to the legacy slice is a deliberate replacement of any live v2
  -- owner; a refused cleanup aborts rather than forgetting its handles.
  if old_campaign and not retire_campaign(old_campaign) then
   transition_error='Previous room cleanup pending; cannot start a new route.';pause_menu('error');return
  end
  campaign=nil;route=nil
  run=Core.new_run(profile,{stocks=3});run_fighter=assert(Roster.validate(selected_fighter))
  for id,g in pairs(run.genes) do if g.origin==starter then Core.equip(run,'player','assault',nil);Core.equip(run,'player','assault',id);break end end
  sync_loadout()
 end
 -- Resume uses the saved resolved manifest; a new run generates one. A loaded
 -- manifest whose seed does not match the run is stale and regenerated.
 if not (resume and manifest and manifest.seed==run.world_seed) then
  manifest=Dungeon.generate(run.world_seed)
 end
 if not Dungeon.validate(manifest) or not manifest.nodes[run.progress.room] then say('Saved route unavailable');pause_menu('error');return end
 if launched_scene~=Roster.scene(run_fighter) then pending_begin=true;launch(run_fighter);return end
 sync_loadout()
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
 if campaign then
  if active and gd.enemy_state then
   for _,e in ipairs(campaign:encounter_composition()) do
    if e.spawned and e.handle then candidate({handle=e.handle,host=e.host},gd.enemy_state(e.handle)) end
   end
  end
  if active and source==1 then local fe=campaign:active_entity();if fe then candidate({port=2,host=fe.host},gd.player(2)) end end
  if active and source==2 then candidate({port=1,host='player'},gd.player(1)) end
 else
  if enemy_host then local port=source==1 and 2 or 1;candidate({port=port,host=host(port)},gd.player(port)) end
  if source==1 and gd.enemy_state then for handle,e in pairs(enemies) do candidate({handle=handle,host=e.host},gd.enemy_state(handle)) end end
 end
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
 if leaf.action=='route' then if campaign then return active==true,'No active run' end return false,'Approach a world exit + D-pad Down' end
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
  if not gd.impulse(source,{x=fighter.facing*2.4,y=action.action=='glide' and fighter.airborne and 1.4 or 0}) then rebind_current_run(assert(Core.restore(before)));say('Movement unavailable; charge retained');return false end
 else
  -- gd.hit directly processes a hit, but does not enqueue the collision-loop
  -- on_hit callback. No pending-count filter: it would swallow a later real hit.
  local spec={damage=math.floor(action.damage),angle=action.knockback.angle,kbg=action.knockback.kbg,bkb=action.knockback.bkb,from=source,reach=action.reach}
  local ok
  if target.handle then ok=gd.enemy_hurt and gd.enemy_hurt(target.handle,spec) else ok=gd.hit(target.port,spec) end
  if not ok then rebind_current_run(assert(Core.restore(before)));say('Activation refused; charge retained');return false end
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
 return {menu=menu,profile=profile,run=run,starter=starter,node=node,fighters=Roster.list,fighter={name=Roster.name(selected_fighter)},notice=Feedback.view(feedback).notification,retry=pending_finish~=nil,error=transition_error or (save_error and 'Both checkpoints are invalid. Files preserved; repair before continuing.'),
  -- Real presentation state, so the reviewed tutorial/settings screens never
  -- render against fabricated input.
  onboarding=presentation:view(),settings=presentation:settings_view(),
  -- Declared collection facts the menus render but never invented themselves.
  capacity={count=count_keys(profile and profile.genes),max=128,full=count_keys(profile and profile.genes)>=128},
  pending_exports=Core.pending_exports(profile)}
end
local function choose_menu(action)
 if not action then return end
 if menu=='error' then if action.kind=='retry_finish' and pending_finish then finish(pending_finish) end;return end
 if action.kind=='choose_fighter' then roster_state=Roster.new(selected_fighter);pause_menu('fighter');return end
 if action.kind=='fighter_back' then pause_menu('collection');return end
 if action.kind=='fighter_selected' then local previous=selected_fighter;selected_fighter=assert(Roster.validate(action.fighter));if not save() then selected_fighter=previous;return end;pause_menu('collection');say('Selected '..Roster.name(selected_fighter));return end
 if action.kind=='blocked' then say(action.message,'blocked','menu');return end
 if action.kind=='start' or action.kind=='resume' then begin(action.kind=='resume');return end
 if action.kind=='claim_export' or action.kind=='decline_export' then
  local before_profile=assert(Core.snapshot(profile))
  local claimed,why
  if action.kind=='claim_export' then claimed,why=Core.claim_deferred(profile,action.run)
  else claimed,why=Core.decline_deferred(profile,action.run) end
  if not claimed then say((action.kind=='claim_export' and 'Export claim refused: ' or 'Export decline refused: ')..tostring(why),'blocked','export');return end
  -- Exactly-once and failure-safe: if the durable write is refused, the pending
  -- gene stays exactly where it was, so the claim can be retried and the gene is
  -- neither duplicated nor lost.
  if not save() then
   profile=assert(Core.restore(before_profile))
   say('Export change could not be saved; nothing changed. Retry after fixing storage.','error','export')
   return
  end
  say(action.kind=='claim_export' and ('Exported '..tostring(claimed)) or ('Declined export for run '..tostring(action.run)),
   action.kind=='claim_export' and 'upgrade' or 'info','export')
  feedback_sync(0,menu~=nil)
  return
 end
 if action.kind=='discard' then
  local before_profile=assert(Core.snapshot(profile))
  local ok,why=Core.discard(profile,action.id)
  if not ok then say('Discard refused: '..tostring(why),'blocked','discard');return end
  if not save() then profile=assert(Core.restore(before_profile));say('Discard could not be saved; nothing changed.','error','discard');return end
  say('Discarded '..tostring(action.id),'info','discard')
  menu_state.selected=nil
  feedback_sync(0,menu~=nil)
  return
 end
 if action.kind=='continue' then run.progress.cleared[node.id]=true;if save() then unpause() end;return end
 if action.kind=='leave' then if save() then active=false;cleanup();presentation:release_hud();pause_menu('collection') end;return end
 local before,slot
 if action.kind=='reward' and action.id then
  for k,id in pairs(run.hosts.player.slots) do if id==action.id then slot=k;before=Core.resolve(run,'player',k) end end
 end
 local old_profile,old_run,old_starter=assert(Core.snapshot(profile)),run and assert(Core.snapshot(run)),starter
 local result=Menus.apply(menu_context(),action)
 if not result.ok then say(result.message or 'Action unavailable','blocked','menu');return end
 if result.run then rebind_current_run(result.run) end
 if result.starter then starter=result.starter end
 if not save() then
  profile=assert(Core.restore(old_profile));rebind_current_run(old_run and assert(Core.restore(old_run)) or nil);starter=old_starter
  sync_loadout();feedback_sync(0,true);return
 end
 if action.kind=='breed' or action.kind=='fuse' then
  Menus.reset(menu_state,menu);menu_state.selected=result.id
 end
 if result.reward_claimed then
   Feedback.dismiss(feedback,'clear:'..node.id)
   if before and slot then Feedback.reward(feedback,{key='reward:'..node.id,name=result.message,before=before,after=Core.resolve(run,'player',slot)})
   else say(result.message,'upgrade','reward:'..node.id) end
   presentation:observe({kind='reward'})
   unpause()
  else say(result.message) end
 -- A real placement/unequip/fuse/reward installs a new run table: rebuild the
 -- command tree from the committed loadout so names, readiness and the declared
 -- supply count match what is actually equipped.
 if result.run then sync_loadout() end
 -- Genealogy and fusion are only taught when this build actually offers them.
 if action.kind=='breed' then presentation:observe({kind='breed',available=result.ok==true}) end
 if action.kind=='fuse' then presentation:observe({kind='fuse',available=result.ok==true}) end
 if run and active then Visuals.update(run,{[1]='player',[2]=enemy_host},true) end
 feedback_sync(0,menu~=nil)
end
load_data()
launch=function(choice)
 choice=choice or selected_fighter
 if ready then active=false;cleanup() end
 -- A fresh TBD launch clears a transient transition error so the new scene
 -- returns to the collection menu instead of re-opening the stale error page.
 if menu=='error' and not save_error then menu=nil;transition_error=nil;Commands.reset(command_state,buttons) end
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
 -- The compact rail is what replaces the vanilla stock/percent cluster, so
  -- claiming the native HUD and enabling the rail are one decision, taken only
  -- while a room is actually live. Ownership is returned on leave, on the error
  -- page, at match end and on unload. gd.hud_visible(false) legitimately returns
  -- false (the new state); only a throw or a missing API is a failure.
  if not match.netplay then
   if active or pending_entry then presentation:take_vanilla_hud(true)
   else presentation:release_hud() end
  end
 local pad=gd.pad(1,true);local b=pad and pad.buttons or 0
 if retiring then
  -- New-run replacement: the engine stays paused and the new room is not built
  -- until the previous owner's colliders/models are actually released. A refused
  -- native cleanup is retried a bounded number of times; it is never treated as
  -- permission to run two rooms at once.
  v2_set_paused(true)
  gd.input_mask(1,15)
  local clean,why=retiring:teardown({drop_pending=true})
  if clean then
   retiring:reset(false);retiring=nil;retire_attempts=0
   local pp=pending_promote;pending_promote=nil
   if pp then finish_promotion(pp.result) end
  else
   retire_attempts=retire_attempts+1
   if retire_attempts>=16 then
    transition_error='Previous room cleanup refused; the new run was not started ('..tostring(why)..')'
    pause_menu('error')
    -- Retain the old owner for later cleanup; never forget live handles. Drop
    -- the un-entered new campaign so no idle campaign can be treated as running.
    local dup=false
    for _,x in ipairs(orphan_campaigns) do if x==retiring then dup=true end end
    if not dup then orphan_campaigns[#orphan_campaigns+1]=retiring end
    retiring=nil;pending_promote=nil;campaign=nil
   end
  end
  buttons=b;return
 end
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
 if campaign then
  -- v2 lifecycle. A request/build/persist/commit/recovery keeps the engine
  -- paused; on_frame is gated on campaign:running(). The command tree resumes
  -- real-time only once the campaign is running again.
  node=campaign:current_node()
  tutorial_room(node and node.depth)
  if campaign:blocked() then
   v2_set_paused(true)
   gd.input_mask(1,15);campaign:tick()
   local notice=campaign:take_notice()
   if notice then say(notice,'info','v2') end
   buttons=b;feedback_sync(0,true);return
  end
  if campaign:failed() then
   v2_set_paused(true)
   active=false
   -- Real recovery hook: a terminal pending save or refused rollback may recover
   -- once storage/engine recovers. Retried on a slow cadence (and by the error
   -- menu confirm), never rewriting a stale checkpoint because pending always
   -- holds the latest authoritative record.
   campaign_recovery_error=true
   if tick_serial%30==0 or (menu=='error' and not pending_finish and edge(b,gd.buttons.A)) then campaign:retry_recovery() end
   -- A retry can leave error for recovering or settling. Those phases still
   -- own native/save work and must stay gated in this very callback.
   if not campaign:running() then
    if campaign.phase=='idle' then v2_recovered() end
    transition_error=campaign.error_msg
    if campaign:failed() and menu~='error' then pause_menu('error') end
    buttons=b;return
   end
  end
  local notice=campaign:take_notice()
  local iso_ok,iso=false,false
  if gd.stage_isolate then iso_ok,iso=pcall(gd.stage_isolate) end
  if not iso_ok or not iso then
   active=false;cleanup();transition_error='Empty playfield isolation was lost. Restart to resume the preserved checkpoint.';pause_menu('error');buttons=b;return
  end
  -- Isolation and the campaign are both healthy again, so a recovery page whose
  -- error no longer applies must not strand the player behind inert controls.
  if menu=='error' then v2_recovered() end
  active=campaign:running() and menu~='error'
  if v2_paused and campaign:running() and menu~='error' then v2_set_paused(false) end
  if notice then say(notice,'info','v2') end
  feedback_sync(0,menu~=nil)
  if menu then
   gd.input_mask(1,15)
   -- Explicit recovery control: confirm on the error page retries the pending
   -- native/save recovery; a successful retry resumes the run.
   if menu=='error' and campaign_recovery_error and not pending_finish and edge(b,gd.buttons.A) then
    campaign:retry_recovery()
    if campaign:running() then v2_recovered();v2_set_paused(false);buttons=b;return end
    v2_set_paused(true);active=false;buttons=b;return
   end
   local mx,my,mb=gd.mouse()
   local input={up=edge(b,gd.buttons.UP),down=edge(b,gd.buttons.DOWN),left=edge(b,gd.buttons.LEFT),right=edge(b,gd.buttons.RIGHT),confirm=edge(b,gd.buttons.A),back=edge(b,gd.buttons.B),mx=mx,my=my,click=mb%2==1 and mouse_buttons%2==0}
   local action=menu=='fighter' and Roster.update(roster_state,{coverage=Bindings.coverage},input) or nil
   if menu~='fighter' then action=Menus.update(menu_state,menu_context(),input) end
   mouse_buttons=mb;choose_menu(action);buttons=b;return
  end
  if reward_ui then handle_v2_reward(b);buttons=b;return end
  if campaign.reward_room then open_v2_reward(campaign:current_node());if reward_ui then buttons=b;return end end
  local p=gd.player(1)
  -- Cleared-door proximity routing comes first: one Down press at the root either
  -- travels or forks the item branch, never both, so a consumable and a travel
  -- can never be spent by the same edge.
  if p and command_state.node=='root' and edge(b,gd.buttons.DOWN) then
   local ok,why=campaign:request_door(p)
   if ok then
    v2_set_paused(true);Commands.reset(command_state,b);presentation:observe({kind='door'});buttons=b;return
   end
   if why and why~='No exit here' and why~='Clear the doorway first' then say(why,'blocked','door') end
  end
  local event,mask=Commands.update(command_state,b,availability);gd.input_mask(1,mask)
  record_command_event(event)
  if event then presentation:observe_command(event) end
  if event and event.kind=='execute' then
   if event.action=='gene' then
    if apply_gene(1,event.slot) then presentation:observe({kind='cast'}) end
   elseif event.action=='restore' then
    -- A refused spend rolls its heal back and spends nothing; the reason is
    -- reported rather than swallowed, so a save refusal is never a silent no-op.
    local spent,why=campaign:use_supply()
    if spent then sync_loadout() else say('Restore refused: '..tostring(why or 'unavailable'),'blocked','supply') end
   elseif event.action=='route' then local map=campaign:route_map();if map then say('Route: '..map.counts.discovered_rooms..' rooms, '..map.counts.known_exits..' exits','info','map') end
   elseif event.action=='loadout' then pause_menu('rest')
   elseif event.action=='collection' then save();active=false;cleanup();presentation:release_hud();pause_menu('collection') end
  elseif event and event.kind=='blocked' then say(event.reason or 'Command unavailable','blocked','command') end
  buttons=b
  return
 end
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
  -- Same precedence as the live-v2 path: the cleared-door proximity routing owns
   -- the root Down edge, so one press can never both travel and fork a branch.
   if p and command_state.node=='root' and edge(b,gd.buttons.DOWN) and (run.progress.cleared[node.id] or (node.kind~='arena' and node.kind~='boss' and not next(enemies))) then
    for _,e in ipairs(node.exits) do local anchor=node.room.exit_anchors[e.side]
     if math.abs(p.x-anchor.x)<13 and math.abs(p.y-anchor.y)<10 then
      if enter(e.to) then presentation:observe({kind='door'});Commands.reset(command_state,b);travelled=true end
      break
     end end
   end
   if not travelled then
    local event,mask=Commands.update(command_state,b,availability);gd.input_mask(1,mask)
    record_command_event(event)
    if event then presentation:observe_command(event) end
    if event and event.kind=='execute' then
     if event.action=='gene' then
      if apply_gene(1,event.slot) then presentation:observe({kind='cast'}) end
     elseif event.action=='restore' and run.progress.supplies>0 then run.progress.supplies=run.progress.supplies-1;gd.set_percent(1,math.max(0,gd.player(1).percent-30));save();sync_loadout();say('Supply used: percent -30')
     elseif event.action=='restore' then say('Restore refused: no supplies in this run','blocked','supply') end
     if event.action=='loadout' then pause_menu('rest')
     elseif event.action=='collection' then save();active=false;cleanup();presentation:release_hud();pause_menu('collection') end
    elseif event and event.kind=='blocked' then say(event.reason or 'Command unavailable','blocked','command') end
   end
 end
 buttons=b
 feedback_sync(0,menu~=nil)
end
function on_action_change(port,old,new,sub)
 if not sub then Visuals.dirty(port) end
 if campaign then
  if active and port<=2 and not sub then
   move_serial[port]=(move_serial[port] or 0)+1;move_ids[port]='p'..port..':'..run.id..':'..run.frame..':'..move_serial[port]
   local ehost=host(port)
   if new==181 and not menu and not reward_ui and ehost then Core.on_event(run,{host=ehost,kind='defend',move_id=move_ids[port],lineage='direct'}) end
  end
  return
 end
 if active and port<=2 and not sub then
  move_serial[port]=(move_serial[port] or 0)+1;move_ids[port]='p'..port..':'..run.id..':'..run.frame..':'..move_serial[port]
  if new==181 and not menu and (enemy_host or next(enemies)) and not run.progress.cleared[node.id] and host(port) then Core.on_event(run,{host=host(port),kind='defend',move_id=move_ids[port],lineage='direct'}) end -- confirmed GuardSetOff/shieldstun in an active encounter
 end
end
function on_hit(attacker,victim,info)
 if campaign then
  if not active or menu or reward_ui or not info or info.item then return end
  if attacker and attacker==1 and campaign:active_entity() then
   Core.on_event(run,{host='player',kind='direct_hit',move_id=move_ids[1] or 'p1:initial',lineage='direct'})
   feedback_sync(0,false)
  end
  return
 end
 if not active or menu or not enemy_host or run.progress.cleared[node.id] then return end
 if attacker and host(attacker) and host(victim) and attacker~=victim and not info.item then Core.on_event(run,{host=host(attacker),kind='direct_hit',move_id=move_ids[attacker] or ('p'..attacker..':initial'),lineage='direct'}) end
end
function on_enemy_hit(event)
 if campaign then
  if not active or menu or reward_ui or not event or event.from~=1 then return end
  local verdict=campaign:hit({handle=event.handle,from=event.from,damage=event.damage,id=event.id})
  if verdict and verdict.handled then
   Core.on_event(run,{host='player',kind='direct_hit',move_id=move_ids[1] or 'p1:initial',lineage='direct'})
   feedback_sync(0,false)
  end
  return
 end
 if active and not menu and enemies[event.handle] and event.from==1 and not run.progress.cleared[node.id] then
  Core.on_event(run,{host='player',kind='direct_hit',move_id=move_ids[1] or ('p1:initial'),lineage='direct'})
  feedback_sync(0,false)
 end
end
function on_enemy_defeated(event)
 if campaign then if active and event then campaign:enemy_defeated(event.handle) end return end
 if active and enemies[event.handle] then enemy_controller:detach(event.handle);enemies[event.handle]=nil;if not next(enemies) then room_clear() end end
end
function on_frame()
 if not active or not ready or menu or reward_ui then return end
 -- Gameplay (Core/fighters/stocks/move earning) is frozen for the whole paused
 -- v2 transition/recovery; only a running campaign drives physics.
 if campaign and not campaign:running() then return end
 local isolation_ok,isolation_value=false,false
 if gd.stage_isolate then isolation_ok,isolation_value=pcall(gd.stage_isolate) end
 if not isolation_ok or not isolation_value then
  active=false;pending_entry=false;cleanup()
  transition_error='Empty playfield isolation was lost. Restart to resume the preserved checkpoint.'
  pause_menu('error');return
 end
 local match=gd.match();if match.netplay then active=false;ready=false;cleanup();say('Offline mode only');return end
 if match.frame==last_frame then return end;last_frame=match.frame
 if campaign then
  Core.tick(run,1)
  local p=gd.player(1)
  campaign:frame(p,v2_previous[1])
  if p then v2_previous[1]={x=p.x,y=p.y} end
  -- The CPU stock display stays at the native baseline: only an observed
  -- decrease is exactly one KO. Never pass the raw drop as a KO count.
  for port=1,2 do
   local pl=gd.player(port)
   if pl and type(pl.stocks)=='number' and pl.stocks<(previous_stocks[port] or BASE_STOCKS) then
    if port==1 then
     previous_stocks[1]=BASE_STOCKS;gd.set_stocks(1,BASE_STOCKS)
     if campaign:lose_life()=='failure' then finish('failure');return end
    else
     previous_stocks[2]=BASE_STOCKS;gd.set_stocks(2,BASE_STOCKS)
     if campaign.encounter_active then campaign:stock(2,1,0) end
    end
   end
   if pl then previous_stocks[port]=pl.stocks<BASE_STOCKS and BASE_STOCKS or pl.stocks end
  end
  local ae=campaign:active_entity()
  if ae then apply_gene(2,'assault') end
  -- Tutorial evidence from real gameplay only: sustained ground movement and an
  -- actually telegraphing enemy, never a constructed completion.
  if p and not p.airborne and math.abs(p.vx or 0)>0.6 then
   v2_move_frames=v2_move_frames+1
   if v2_move_frames%60==0 then presentation:observe({kind='move'}) end
  end
  if campaign.encounter_active then presentation:notice_tells(campaign.encounters:states()) end
  Visuals.update(run,{[1]='player',[2]=ae and ae.host or nil})
  feedback_sync(1,false)
  return
 end
 Core.tick(run,1);enemy_controller:tick();feedback_sync(1,false)
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
   if not p.airborne and math.abs(p.vx or 0)>0.6 then movement[port]=movement[port]+1;if movement[port]%60==0 then Core.on_event(run,{host=who,kind='move',move_id=who..':move:'..run.frame,lineage='direct'});if port==1 then presentation:observe({kind='move'}) end end end
  end
 end
 presentation:notice_tells(enemy_controller:states())
 if enemy_host and not run.progress.cleared[node.id] then
  if run.frame%30==0 and gd.cpu_technical and not TechAI.status(2).enabled then configure_ai() end
  apply_gene(2,'assault')
 end
 Visuals.update(run,{[1]='player',[2]=enemy_host})
 -- Fighter coloration hooks remain optional; geometry and gameplay do not depend on FX.
end
local function text(x,y,s,role,color) gd.kit.text(x,y,s,role or 'caption',color or 'bone') end
local function draw_scene()
 if not ready then return end
 if reward_ui then
  gd.fill(40,60,480,220,0x101a2ce0)
  gd.kit.text(52,80,'ENCOUNTER REWARD','caption','gold','left',{max_w=440,shear=0})
  for i,option in ipairs(reward_ui.options) do
   local label=(option.kind or 'gene')..' '..tostring(option.id or '')
   gd.kit.text(60,110+(i-1)*26,(i==reward_ui.focus and '> ' or '  ')..label,'caption',i==reward_ui.focus and 'gold' or 'bone','left',{max_w=420,shear=0})
  end
  return
 end
 if pending_room then gd.kit.text(24,36,'Preparing room','caption','bone');return end
 if menu then
  if menu=='fighter' then Roster.draw(roster_state,{coverage=Bindings.coverage}) else Menus.draw(menu_state,menu_context()) end
 else
  gd.fill(12,12,188,24,0x101a2c90);gd.kit.text(20,29,node.title,'caption','gold','left',{max_w=172,shear=0})
  local tree=presentation:view_commands(command_state,availability)
  if tree then Visuals.tree(tree) end
  local enemy_states=campaign and campaign.encounters:states() or enemy_controller:states()
  for _,enemy in ipairs(enemy_states) do
   if enemy.phase=='telegraph' or enemy.phase=='ready' then
    local x,y=gd.project(enemy.x,enemy.y+13,0)
    if x and y then gd.kit.text(math.max(4,math.min(550,x-38)),math.max(16,math.min(390,y)),enemy.phase=='telegraph' and 'CASTING' or 'READY','caption',enemy.family=='rime' and 'bone' or 'gold','left',{max_w=82,shear=0}) end
   end
  end
  for _,e in ipairs(node.exits) do local a=node.room.exit_anchors[e.side];local x,y=gd.project(a.x,a.y+8,0);if x and y then text(x,y,e.label,'caption','gold') end end
 end
 local player=gd.player(1)
 local ae=campaign and campaign:active_entity() or nil
 local opponent
 if campaign then opponent=ae and not run.progress.cleared[node.id] and gd.player(2)
 else opponent=enemy_host and not run.progress.cleared[node.id] and gd.player(2) end
 local opp_lives=campaign and ((ae and ae.remaining) or 1) or ((node and node.kind=='boss' and 2 or 1)-enemy_kos)
 -- The real reviewed layout (the port's fixed 640x480 canvas) drives every
 -- box: the compact life/percent/ability rail, its opponent block and the
 -- notification strips. A window resize cannot enlarge HUD coverage.
 presentation:draw(feedback,{hud=not menu,notifications=true,compact_menu=menu~=nil,
  command=command_state.node~='root',
  player={percent=player and player.percent or 0,lives=run and run.stocks or 0,supplies=run and run.progress.supplies or 0},
  opponent=opponent and {label=node.kind=='boss' and 'CHAMPION' or 'OPPONENT',percent=opponent.percent,lives=opp_lives}})
end
-- A renderer failure is contained and returns the native HUD when possible.
-- Drawing never advances the run or writes a save; a refused HUD restoration
-- stays owned by Presentation for the next lifecycle retry.
function on_draw()
 local ok,err=pcall(draw_scene)
 if not ok then
  presentation:release_hud()
  if gd.log then gd.log('roguelite: draw refused: '..tostring(err)) end
 end
end
function on_match_end()
 pending_room=nil;ready=false;active=false;pending_entry=false
 -- The scene is gone with the stage, so the vanilla HUD goes back with it.
 presentation:release_hud()
 -- Confirmed native scene teardown (a new scene has begun): owned handles are
 -- gone with the old stage, so bookkeeping may be reset.
 if campaign then campaign:teardown();campaign:reset(true);campaign=nil end
 for _,c in ipairs(orphan_campaigns) do c:teardown({drop_pending=true});c:reset(true) end
 orphan_campaigns={}
 enemy_controller:clear();platforms={};enemies={};fx_handles={};enemy_host=nil;Feedback.reset(feedback);Rooms.reset(room_visuals)
end
function on_unload()
 pending_room=nil
 -- gs_unload only drops the script environment, commands and tasks; it does NOT
 -- tear down the scene or remove stage colliders. So this is not a confirmed
 -- scene teardown and reset(true) must never be fabricated here. Release what
 -- the engine accepts; a refused release needs a native owner-cleanup seam.
 if campaign then
  local clean,why=campaign:teardown()
  if clean then campaign:reset(false)
  else gd.log('roguelite: v2 owned handles survive script unload; native cleanup seam required: '..tostring(why)) end
  campaign=nil
 end
 for _,c in ipairs(orphan_campaigns) do
  local clean,why=c:teardown({drop_pending=true})
  if not clean then gd.log('roguelite: orphan v2 cleanup still pending at unload: '..tostring(why)) end
 end
 orphan_campaigns={}
 if gd.stage_isolate then pcall(gd.stage_isolate,false) end
 if ready then cleanup();if gd.cpu_mode then gd.cpu_mode(2,'fight') end end
 Rooms.release(room_visuals);presentation:release_hud();gd.release_pad(4);gd.input_mask(1,0);gd.resume()
end
-- Read-only inspection for integration tests and script-console diagnostics.
function roguelite_state() return {loading=pending_room~=nil,ready=ready,active=active,menu=menu,
 command_event=last_command_event,presentation=presentation:status(),hud_rail=presentation:hud_owned(),profile=profile,run=run,node=node and node.id,toast=toast,command=command_state.node,fighter=selected_fighter,run_fighter=run_fighter,menu_view=menu and (menu=='fighter' and Roster.view(roster_state,{coverage=Bindings.coverage}) or Menus.view(menu_state,menu_context())),feedback=Feedback.view(feedback),save_error=save_error,retiring=retiring~=nil,orphans=#orphan_campaigns,v2=route and {schema=route.manifest.schema_version,generator=route.manifest.generator_version,rooms=#route.manifest.order,current=route.progress.current_room,progress=route.progress.version,campaign=campaign and campaign:status() or nil} or nil,reward_ui=reward_ui~=nil} end
-- Public handle for focused integration tests (read/drive the live campaign).
function roguelite_v2() return campaign end
gd.command('rogue_state',function()
 local a=run and Core.ability(run,'player','assault')
 -- Versioned diagnostics: runtime_ready is engine/scene readiness, ability_ready
 -- is the assault gene's charge state. The legacy `ready` fields stay for
 -- existing readers until live_acceptance has fully migrated.
 gd.log('rogue_state ready='..tostring(ready)..' active='..tostring(active)..' menu='..tostring(menu)..' room='..tostring(node and node.id)..' stocks='..tostring(run and run.stocks)..' charge='..tostring(a and a.charge)..' runframe='..tostring(run and run.frame)..' saveerror='..tostring(save_error)..' command='..command_state.node..' cleared='..tostring(run and node and run.progress.cleared[node.id]==true)..' claimed='..tostring(run and node and run.progress.claimed[node.id]==true)..' collection='..tostring(profile and #keys(profile.genes))..' finished='..tostring(profile and #keys(profile.finished))..' outcome='..tostring(run and run.status)..' cost='..tostring(a and a.cost)..' ready='..tostring(a and a.ready)..' remaining='..tostring(a and a.remaining)..' reach='..tostring(a and a.reach)..' diag_version=1 runtime_ready='..tostring(ready)..' ability_ready='..tostring(a and a.ready))
end)
-- Read-only v2 route diagnostics: generate a topology for a seed and report the
-- adapter's admission decision. Never mutates the live run or starts a route.
gd.command('rogue_route',function(arg)
 local seed=tonumber(arg) or (run and run.world_seed) or 12345
 local ok,manifest=pcall(topology.generate,topology,seed)
 if not ok then gd.log('rogue_route seed='..seed..' generation_error='..tostring(manifest));return end
 local d=adapter:diagnostics(manifest)
 gd.log('rogue_route diag_version='..tostring(d.diag_version)..' schema='..tostring(d.schema_version)..' seed='..seed..' rooms='..tostring(d.rooms)..' spine='..tostring(d.spine)..' fallback='..tostring(d.fallback_used)..' admissible='..tostring(d.manifest_admissible)..' signature='..tostring(d.topology_signature)..' refusal='..tostring(d.refusal))
end)
gd.command('rogue_start',function() if not ready then demo=true end end)
gd.command('rogue_map',function()
 local map,why=campaign and campaign:route_map() or nil,nil
 if not map then gd.log('rogue_map unavailable: '..tostring(why or 'no active campaign'));return end
 gd.log('rogue_map current='..tostring(map.current)..' rooms='..tostring(map.counts.discovered_rooms)..' exits='..tostring(map.counts.known_exits)..' hidden_rewards='..tostring(map.counts.undiscovered_reward_count))
end)
gd.command('rogue_enemies',function()
 local states=campaign and campaign.encounters:states() or enemy_controller:states()
 for _,e in ipairs(states) do
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
