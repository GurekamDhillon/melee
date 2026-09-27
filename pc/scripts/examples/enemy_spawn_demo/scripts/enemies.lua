-- Copy this mod folder to mods/ and start an offline Battlefield match.
-- Positions are examples for Battlefield's main platform; adjust them for your arena.
local spawned = {}

local function add(kind, x, y, facing)
    local handle, err = gd.spawn_enemy(kind, x, y, { facing = facing })
    if handle then
        spawned[handle] = kind
        gd.log(string.format("enemy demo: spawned %s handle=%d", kind, handle))
    else
        gd.log(string.format("enemy demo: %s failed: %s", kind, err))
    end
end

function on_match_start()
    spawned = {}
    add("goomba", -45, 20, 1)
    add("koopa", 0, 20, -1)
    add("redead", 45, 20, -1)
end

function on_enemy_defeated(event)
    if spawned[event.handle] then
        gd.log(string.format("enemy demo: defeated %s handle=%d", event.kind, event.handle))
        spawned[event.handle] = nil
    end
end

-- The native handles are saved with the match. Lua tables are not, so discard the example's
-- bookkeeping after a same-scene savestate/rewind load; defeat events still carry both fields.
function on_loadstate()
    spawned = {}
end
