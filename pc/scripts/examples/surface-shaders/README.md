# Surface shader examples

Requires the Aurora GD surface carried patch and a fresh port build. Enable this
script mod and start a match: player 1 receives cel quantisation and a dark surface
edge. The mod is visual-only. These shaders were written for this project.

In this mod's script, swap the selection for:

```lua
gd.fighter_shader(1, "shaders/rim-light.wgsl", {
    params = {0.2, 0.6, 1.0, 0.8, 3}
})
gd.fighter_shader(2, "shaders/dissolve.wgsl", {
    params = {0.4, 0.08, 48, 0, 1, 0.3, 0.05, 1}
})
gd.stage_shader("shaders/cel-outline.wgsl", {params = {5, 0.12, 0.5}})
gd.fighter_shader(1, nil)
gd.stage_shader(nil)
```

Paths resolve inside the **calling mod**, including junction containment checks;
these path calls therefore belong in a mod script, not the root console.
Calling again after editing reloads the content. No automatic file polling.
At most 256 distinct shader sources may be registered in one process. A bad load
returns `nil, error` and leaves the previous selection intact.

`cel-outline` darkens the inward silhouette surface; it cannot draw an expanded
external outline. `dissolve` uses raw vertex UV0 (zero if absent); meshes without
UV0 dissolve uniformly. None changes collision, hits, shadows' game state or AI.
See the packet H section in the workspace `docs/shaders.md` for the contract and
the remaining screen-space G-buffer integration limits.
