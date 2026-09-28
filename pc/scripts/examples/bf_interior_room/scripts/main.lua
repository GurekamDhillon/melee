-- BF interior kit: an open-front room assembled at runtime from separate part models.
-- Each part is one GXMS mesh (models/<model>.gxmesh) with a collision sidecar
-- (models/<model>.coll.json) naming the shared atlas (bf_kit.gxtex / .glow.gxtex, loaded once).
-- gd.model_spawn places a part anywhere and adds its sidecar's lines; walls, beams, posts,
-- corners and trims have no lines and are visual only.
-- Rebuild models/: pc/assets_src/bf_interior/export_kit.py (see its docstring).

local U = 6.5  -- game units per kit metre (export_kit.py KIT_SCALE 1.3; generated)

-- The layout, in kit metres at each part's origin: {part, x, y, depth}. Depth is the Blender -Y
-- offset (towards the camera); the parts already sit behind or on the fighter plane.
-- solid = a floor fighters cannot drop through; ledges = override the sidecar's ledge flag.
local ROOM = {
  -- the gameplay route: solid ground -> stairs -> balcony -> ramp -> landing with a drop-through gap
  {"Floor_2m", -9, 0, solid = true, ledges = true},
  {"Floor_4m", -6, 0, solid = true, ledges = false},
  {"Floor_4m", -2, 0, solid = true, ledges = false},
  {"Floor_4m",  2, 0, solid = true, ledges = false},
  {"Floor_4m",  6, 0, solid = true, ledges = true},
  {"Stairs_4m_Rise2m", -6, 0},
  {"Balcony_4m", -2, 2, ledges = false},  -- both ends meet slopes: an edge there pops a runner airborne
  {"Ramp_4m_Rise2m", 2, 2},
  {"Floor_Opening_4m", 6, 4},
  -- back walls, two storeys
  {"Wall_Doorway_4m", -6, 0}, {"Wall_Solid_4m", -2, 0}, {"Wall_Window_4m", 2, 0}, {"Wall_Solid_4m", 6, 0},
  {"Wall_Solid_4m", -6, 4}, {"Wall_Solid_4m", -2, 4}, {"Wall_Solid_4m", 2, 4}, {"Wall_Doorway_4m", 6, 4},
  {"Door_Leaf", -6, 0},
  -- ends: side returns, inside corners at the back, outside corners on the exposed front edge
  {"Wall_Side_Return", -8, 0}, {"Wall_Side_Return", -8, 4},
  {"Wall_Side_Return",  8, 0, mirror = true}, {"Wall_Side_Return",  8, 4, mirror = true},
  {"Corner_Inside_4m", -8, 0}, {"Corner_Inside_4m", -8, 4},
  {"Corner_Inside_4m", 8, 0, mirror = true}, {"Corner_Inside_4m", 8, 4, mirror = true},
  {"Corner_Outside_4m", -8, 0, 1.2}, {"Corner_Outside_4m", -8, 4, 1.2},
  {"Corner_Outside_4m",  8, 0, 1.2, mirror = true}, {"Corner_Outside_4m",  8, 4, 1.2, mirror = true},
  -- structure and trim
  {"Beam_4m", -6, 8}, {"Beam_4m", -2, 8}, {"Beam_4m", 2, 8}, {"Beam_4m", 6, 8},
  {"Rear_Post_4m", 4, 0}, {"Rear_Post_4m", 8, 0},
  {"Rear_Glass_Rail_4m", 6, 4},
  {"Rear_Glass_Rail_4m_glass", 6, 4}, {"Window_Glass_Insert_glass", 2, 0},
  {"Balcony_4m_glass", -2, 2}, {"Ramp_4m_Rise2m_glass", 2, 2},
  {"Floor_End_Trim", -10, 0}, {"Floor_End_Trim", 8, 0}, {"Floor_End_Trim", 4, 4},
  {"Floor_End_Trim", -4, 4},
}

room = nil  -- global, so a test or the console can despawn it: {instances = {...}, assets = {...}}

local function model_name(part) return "bf_" .. part:lower() end

function room_build(ox, oy)
  local assets, instances, lines = {}, {}, 0
  for _, p in ipairs(ROOM) do
    local name = model_name(p[1])
    assets[name] = assets[name] or assert(gd.model_load(name))
    local opts = {x = ox + p[2] * U, y = oy + p[3] * U, z = (p[4] or 0) * U}
    opts.scale_x = p.mirror and -1 or 1
    if p.solid ~= nil or p.ledges ~= nil then
      opts.floor_flags = (p.solid and 0 or 1) + ((p.ledges == false) and 0 or 2)
    end
    local inst, why = gd.model_spawn(assets[name], opts)
    if inst then instances[#instances + 1] = inst
    else gd.log("bf_interior_room: " .. p[1] .. " not spawned: " .. tostring(why)) end
  end
  -- Instances hold their own references; release the load references.
  local n = 0
  for _, a in pairs(assets) do gd.model_release(a) n = n + 1 end
  room = {instances = instances, origin = {ox, oy}}
  gd.log(string.format("bf_interior_room: %d of %d parts (%d models) at (%g, %g)",
                       #instances, #ROOM, n, ox, oy))
  return #instances
end

function room_despawn()
  if not room then return 0 end
  local n = 0
  for _, inst in ipairs(room.instances) do
    if gd.model_despawn(inst) then n = n + 1 end
  end
  room = nil
  gd.log("bf_interior_room: despawned " .. n .. " parts")
  return n
end

function on_match_start()
  room = nil
  local match = gd.match()
  if match.netplay then return end
  local bounds = gd.stage_bounds()
  if not bounds or not bounds.main_floor then
    gd.log("bf_interior_room: no solid main floor available")
    return
  end
  local floor = bounds.main_floor
  -- Stage geometry supplies the placement: above the highest base-stage platform,
  -- centred on the widest connected solid floor (FD and Battlefield need no IDs).
  room_build((floor.left + floor.right) / 2, bounds.surface_top + 20)
end

function on_match_end()
  room = nil  -- the engine frees every model and its lines at scene end
end
