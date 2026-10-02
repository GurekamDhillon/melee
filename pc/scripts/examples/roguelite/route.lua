-- Route service: one integration seam for a v2 campaign. It composes the pure
-- generator, the certification-gated adapter and the progress record, and
-- validates a saved route before it is accepted. Deps are injected:
--   topology: Topology instance   adapter: Adapter instance
--   progress: Progress module     progression: Progression module
--   catalogue: RoomCatalogue      recipes: RoomRecipes
--
-- create() refuses an uncertified manifest; resume() re-validates the graph,
-- seed/ownership and progress consistency. A syntactically valid table is not a
-- valid run.
local Route = {version = 2}
local function copy(t)
  if type(t) ~= 'table' then return t end
  local out={} for k,v in pairs(t) do out[k]=copy(v) end return out
end
local function equal(a,b)
  if type(a) ~= type(b) then return false end
  if type(a) ~= 'table' then return a==b end
  for k,v in pairs(a) do if not equal(v,b[k]) then return false end end
  for k in pairs(b) do if a[k]==nil then return false end end
  return true
end

function Route.new(deps)
  assert(type(deps) == 'table' and deps.topology and deps.adapter and deps.progress and deps.progression, 'route deps required')
  return setmetatable({topology = deps.topology, adapter = deps.adapter,
    progress = deps.progress, progression = deps.progression, encounters = deps.encounters}, {__index = Route})
end

-- Create a fresh v2 route for a run. Returns {manifest, view, progress} or nil,reason.
function Route:create(run, opts)
  if type(run) ~= 'table' or type(run.id) ~= 'string' or type(run.world_seed) ~= 'number' then
    return nil, 'invalid run'
  end
  local generated,manifest = pcall(self.topology.generate,self.topology,run.world_seed,opts)
  if not generated then return nil, 'generation refused: '..tostring(manifest) end
  for _,id in ipairs(manifest.order) do
    local room=manifest.rooms_by_id[id]
    local template=self.adapter.catalogue.rooms[room.template_id]
    local geometry,recipe=self.adapter.recipes.resolve(template)
    if not geometry then return nil,recipe end
    room.geometry,room.recipe_version,room.recipe_modules=copy(geometry),recipe.version,copy(recipe.modules or {})
    if self.encounters then
      room.encounter_spec=copy(room.encounter and self.encounters.encounters[room.encounter])
      room.reward_spec=copy(room.reward and self.encounters.rewards[room.reward])
    end
  end
  local view, why = self.adapter:manifest(manifest)
  if not view then return nil, why end
  local progress = self.progress.new(run.id, manifest.start_room,
    {supplies = (run.progress and run.progress.supplies) or 2, lives = run.stocks or 3})
  return {manifest = manifest, view = view, progress = progress}
end

-- Validate a saved route for a run, then return the view. Never regenerates.
function Route:resume(run, manifest, progress)
  local ok, why = self:validate_save(run, manifest, progress)
  if not ok then return nil, why end
  local view = assert(self.adapter:manifest(manifest))
  return {manifest = manifest, view = view, progress = progress}
end

function Route:validate_save(run, manifest, progress)
  if type(run) ~= 'table' or type(run.id) ~= 'string' then return false, 'invalid run' end
  if type(manifest) ~= 'table' or type(progress) ~= 'table' then return false, 'missing route/progress' end
  if manifest.generator_version~=self.topology.version then return false,'unsupported generator version; preserve checkpoint' end
  local checked,valid,why=pcall(self.topology.validate,self.topology,manifest)
  if not checked or not valid then return false,'invalid saved graph: '..tostring(why or valid) end
  local admissible, why = self.adapter:check(manifest)
  if not admissible then return false, why end
  if manifest.world_seed ~= run.world_seed then return false, 'route seed does not match the run' end
  local pok, pwhy = self.progress.validate(progress)
  if not pok then return false, pwhy end
  if progress.run_id ~= run.id then return false, 'progress belongs to another run' end
  if progress.start_room ~= manifest.start_room then return false,'progress start mismatch' end
  local current=manifest.rooms_by_id[progress.current_room]
  if not current then return false,'current room missing from route' end
  if progress.current_socket and not current.sockets_by_id[progress.current_socket] then return false,'unknown arrival socket' end
  local rooms,edges=manifest.rooms_by_id,manifest.edges_by_id
  local order={} for _,id in ipairs(manifest.order) do
    if order[id] then return false,'duplicate ordered room' end order[id]=true
  end
  local rewards,pickups,entities,key_sources={}, {}, {}, {}
  for id,room in pairs(rooms) do
    if not order[id] then return false,'room missing from order' end
    if type(room.geometry)~='table' or type(room.recipe_modules)~='table' or type(room.recipe_version)~='number' then return false,'missing resolved geometry' end
    local template=self.adapter.catalogue.rooms[room.template_id]
    local authored={} for _,socket in ipairs(template.sockets) do authored[socket.id]=socket.side end
    for sid,socket in pairs(room.sockets_by_id) do
      if authored[sid]~=socket.side then return false,'socket differs from authored template' end
      authored[sid]=nil
    end
    if next(authored) then return false,'missing authored socket' end
    rewards['reward:'..id]=room.reward_spec~=nil
    if room.grants_key then key_sources[room.grants_key]=id end
    if room.grants_consumable then pickups[room.pickup_id or ('pickup:'..id)]=id end
    for _,pickup in ipairs(room.pickups or {}) do pickups[pickup.id]=id end
    local encounter=room.encounter_spec
    if self.encounters then
      if not equal(encounter,room.encounter and self.encounters.encounters[room.encounter]) then return false,'unsupported encounter specification' end
      if not equal(room.reward_spec,room.reward and self.encounters.rewards[room.reward]) then return false,'unsupported reward specification' end
    end
    local serial=0
    for _,row in ipairs(encounter and encounter.enemies or {}) do
      for _=1,row.count do serial=serial+1;entities['enemy_'..id..'_'..serial]=id end
    end
  end
  local indexed={}
  for id,list in pairs(manifest.edges_by_room) do
    if not rooms[id] or type(list)~='table' then return false,'invalid edge index' end
    local seen={}
    for _,eid in ipairs(list) do
      local e=edges[eid]
      if seen[eid] or not e or e.from_room~=id and e.to_room~=id then return false,'invalid edge adjacency' end
      seen[eid]=true;indexed[eid]=(indexed[eid] or 0)+1
    end
  end
  for id,e in pairs(edges) do
    if indexed[id]~=2 or (e.direction~='both' and e.direction~='forward' and e.direction~='backward') then return false,'invalid edge direction/index' end
    if e.discovery_rule~='always' and e.discovery_rule~='hidden' then return false,'invalid discovery rule' end
  end
  for _,field in ipairs({'visited','discovered','objectives','encounter_kos'}) do
    for id in pairs(progress[field]) do
      if not rooms[id] then return false,'unknown '..field..' room' end
      if (field=='objectives' or field=='encounter_kos') and not progress.visited[id] then return false,'unvisited encounter progress' end
    end
  end
  for id in pairs(progress.visited) do if not progress.discovered[id] then return false,'visited room undiscovered' end end
  for eid in pairs(progress.revealed) do
    local e=edges[eid]
    if not e or not progress.discovered[e.from_room] or not progress.discovered[e.to_room] then return false,'invalid revealed edge' end
  end
  for id in pairs(progress.claimed) do
    local room=id:match('^reward:(.+)$')
    if not rewards[id] or not progress.visited[room] or progress.objectives[room]~='done' then return false,'invalid reward claim' end
  end
  for id in pairs(progress.pickups) do if not pickups[id] or not progress.visited[pickups[id]] then return false,'invalid pickup claim' end end
  for id in pairs(progress.defeated) do if not entities[id] or not progress.visited[entities[id]] then return false,'invalid defeated entity' end end
  for key in pairs(progress.keys) do if not key_sources[key] or not progress.visited[key_sources[key]] then return false,'key without visited source' end end
  for id in pairs(progress.opened) do
    local lock=manifest.locks[id]
    if not lock or lock.kind~='consumable_key' or lock['repeat'] then return false,'invalid opened lock' end
  end
  if run.progress then
    if run.progress.room~=progress.current_room or run.progress.supplies~=progress.supplies or run.stocks~=progress.lives then return false,'run/progress counters disagree' end
    for id,value in pairs(run.progress.cleared) do if value and progress.objectives[id]~='done' then return false,'clear records disagree' end end
    for id,state in pairs(progress.objectives) do if (state=='done')~=(run.progress.cleared[id]==true) then return false,'objective records disagree' end end
    for id,value in pairs(run.progress.claimed) do if value and not progress.claimed['reward:'..id] then return false,'claim records disagree' end end
    for id in pairs(progress.claimed) do if not run.progress.claimed[id:sub(8)] then return false,'claim records disagree' end end
  end
  if progress.outcome and progress.outcome~=run.status then return false,'outcome disagreement' end
  return true
end

-- Apply traversal to a copy. The caller commits it only after room placement and
-- checkpoint persistence succeed; interrupted transitions retain the old route.
function Route:travel(route, exit)
  local p=copy(route.progress)
  local room=route.manifest.rooms_by_id[p.current_room]
  local edge=exit and route.manifest.edges_by_id[exit.edge_id]
  if not room or not edge then return nil,'unknown route exit' end
  local forward=edge.from_room==p.current_room
  if not forward and edge.to_room~=p.current_room then return nil,'exit belongs to another room' end
  if edge.direction~='both' and not (forward and edge.direction=='forward' or not forward and edge.direction=='backward') then return nil,'one-way exit' end
  local target=forward and edge.to_room or edge.from_room
  local socket=forward and edge.to_socket or edge.from_socket
  if exit.to~=target or exit.arrival_socket~=socket then return nil,'exit destination mismatch' end
  if edge.gate_rule then
    local lock=route.manifest.locks[edge.gate_rule]
    if not lock then return nil,'unknown lock' end
    if lock.kind=='persistent_key' then
      if not p.keys[lock.key] then return nil,'Requires '..lock.key end
    elseif lock.kind=='consumable_key' then
      if lock['repeat'] or not p.opened[edge.gate_rule] then
        local ok,why=self.progress.spend_consumable(p,lock.key)
        if not ok then return nil,why end
        if not lock['repeat'] then p.opened[edge.gate_rule]=true end
      end
    else return nil,'unsupported lock kind' end
  end
  local ok,why=self.progress.enter(p,target,socket)
  if not ok then return nil,why end
  local revealed,why=self.progress.reveal(p,room.id,edge.id)
  if not revealed then return nil,why end
  revealed,why=self.progress.reveal(p,target,edge.id)
  if not revealed then return nil,why end
  return p
end

function Route:collect(route)
  local p=copy(route.progress)
  local room=route.manifest.rooms_by_id[p.current_room]
  if room.grants_key then
    local ok,why=self.progress.unlock(p,room.grants_key)
    if not ok then return nil,why end
  end
  local pickups=copy(room.pickups or {})
  if room.grants_consumable then
    pickups[#pickups+1]={id=room.pickup_id or ('pickup:'..room.id),key=room.grants_consumable,amount=room.consumable_amount or 1}
  end
  for _,pickup in ipairs(pickups) do
    if not p.pickups[pickup.id] then
      local ok,why=self.progress.grant_consumable(p,pickup.key,pickup.amount or 1)
      if not ok then return nil,why end
      p.pickups[pickup.id]=true
    end
  end
  route.progress=p
  return true
end

function Route:diagnostics(route)
  if not route then return {diag_version = 1, route = false} end
  local d = self.adapter:diagnostics(route.manifest)
  d.current_room = route.progress and route.progress.current_room
  d.visited = route.progress and (function() local n = 0 for _ in pairs(route.progress.visited) do n = n + 1 end return n end)() or 0
  return d
end

return Route
