-- Single-feature catalogue demo: select once; update uniforms with no shader load.
-- Use this as the mod entry instead of main.lua to run it independently.
local frame = 0
function on_match_start()
    frame = 0
    local ok, err = gd.fighter_shader(1, "shaders/rim-light.wgsl", {params={0.1,0.6,1,0.15,2}})
    if not ok then gd.log("surface parameter demo: " .. tostring(err)) end
end
function on_frame()
    frame = frame + 1
    gd.fighter_shader_set(1, {params={0.1,0.6,1,0.1 + 0.05 * math.sin(frame / 60),2}})
end
function on_match_end() gd.fighter_shader(1,nil) end
