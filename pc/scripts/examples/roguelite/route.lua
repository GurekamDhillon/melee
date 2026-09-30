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
local Route = {version = 1}

function Route.new(deps)
  assert(type(deps) == 'table' and deps.topology and deps.adapter and deps.progress and deps.progression, 'route deps required')
  return setmetatable({topology = deps.topology, adapter = deps.adapter,
    progress = deps.progress, progression = deps.progression}, {__index = Route})
end

-- Create a fresh v2 route for a run. Returns {manifest, view, progress} or nil,reason.
function Route:create(run, opts)
  if type(run) ~= 'table' or type(run.id) ~= 'string' or type(run.world_seed) ~= 'number' then
    return nil, 'invalid run'
  end
  local manifest = self.topology:generate(run.world_seed, opts)
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
  local admissible, why = self.adapter:check(manifest)
  if not admissible then return false, why end
  if manifest.world_seed ~= run.world_seed then return false, 'route seed does not match the run' end
  local pok, pwhy = self.progress.validate(progress)
  if not pok then return false, pwhy end
  if progress.run_id ~= run.id then return false, 'progress belongs to another run' end
  if not manifest.rooms_by_id[progress.current_room] then return false, 'current room missing from route' end
  for room_id in pairs(progress.visited) do
    if not manifest.rooms_by_id[room_id] then return false, 'visited room missing from route: ' .. room_id end
  end
  for edge_id in pairs(progress.revealed) do
    if not manifest.edges_by_id[edge_id] then return false, 'revealed edge missing from route: ' .. edge_id end
  end
  local gok, gwhy = self.progression.validate(manifest)
  if not gok then return false, 'route not completable: ' .. tostring(gwhy) end
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
