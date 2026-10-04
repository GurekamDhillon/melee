local enabled = false
local function add(path, options)
    local handle, error = gd.post_add("shaders/" .. path, options)
    if not handle then gd.log("shader-demo: " .. tostring(error)) end
end
function on_tick()
    if not gd.key_pressed("F6") or not gd.match().active then return end
    enabled = not enabled
    gd.post_clear()
    if not enabled then return end
    add("bloom.wgsl", {order=0, half=true, params={threshold=0.7, intensity=0.5, radius=1.0}})
    add("bloom-compose.wgsl", {order=1})
    add("grade.wgsl", {order=2, params={lift={0,0,0,0}, gamma={1,1,1,1}, gain={1,1,1,1}, saturation=1.1}})
    add("vignette.wgsl", {order=3, params={strength=0.4, radius=0.3, softness=0.4}})
end
function on_match_end() enabled = false end
function on_unload() gd.post_clear() end
