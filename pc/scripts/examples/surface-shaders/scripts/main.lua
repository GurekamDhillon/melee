function on_match_start()
    local ok, err = gd.fighter_shader(1, "shaders/cel-outline.wgsl", {
        params = {4, 0.22, 0.8}
    })
    if not ok then gd.log("surface example: " .. tostring(err)) end
end

function on_match_end()
    gd.fighter_shader(1, nil)
end
