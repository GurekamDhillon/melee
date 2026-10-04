# Shader examples

Copy this mod into your mods directory and enable `shader-demo`. In a match, F6
toggles whole-scene bloom, colour grade and vignette before the retail HUD. The
sample is visual-only and makes no gameplay writes. The runtime clears its handles
at scene changes; F6 creates a fresh chain next match. Check the log for errors.

For isolated passes from the console, use mounted paths:

```lua
gd.post_add("shader-demo/shaders/vignette.wgsl", {
  stage="world", params={strength=0.6, radius=0.25, softness=0.35}
})
gd.post_add("shader-demo/shaders/outline.wgsl", {
  order=10, params={width=1, threshold=0.02, strength=0.8, tint={0,0,0,1}}
})
```

`gd.post_clear()` only clears the calling script/console's passes. Run each
comparison from the same owner. `world` is after the completed main world
(effects and near glass included), before HUD cameras. `final` is after retail
HUD, before the host overlay/console. There is no separate pre-effects stage.

Each `.wgsl` is a fragment **body**, with `in`, `params`, `scene_color`,
`previous_color`, `scene_depth`, `linear_depth`, `time_seconds`, `resolution`.
Named float parameters occupy a vec4 slot: write `params.strength.x`. Named
vec4 parameters use `.xyz` or `.xyzw`. Declarations/defaults are the `params`
table at load/add; updates preserve types and reject undeclared names.

The colour grade uses lift/gamma/gain with saturation, without a LUT. Vignette
uses one colour sample. Outline uses one colour and five depth loads; depth
snapshots exclude translucent depth writers and may be unavailable on a device.
Bloom extracts and averages 25 samples at half resolution, then a separate
full-resolution composition adds them to the original scene. Put the bloom pair
first: its composition reads this chain's original scene, so placing it after
other passes would discard their colour edits. Bloom thresholds below 1 work on
Aurora's normalized scene target; this is not an HDR framebuffer upgrade.

`models/glass.material.json` is an installation template, not a mesh. Copy it
next to a GXMS kit mesh as `<mesh-name>.material.json`; use that mesh's existing
atlas or add an `albedo` path relative to the sidecar. Its `.coll.json` must
retain `"alpha":1` so it participates in the existing far/near translucent split.
Glass defaults to authored tint/opacity with a power-shaped bright rim, without
needing repeated draws. Existing room-kit glass is not automatically switched:
the art packaging/export step must install these sidecars alongside glass parts.

Do not put `@fragment`, resource bindings or an entry point in a body. WGSL
`textureSample` needs uniform control flow. Prefer `textureSampleLevel(...,0.0)`
for conditional/per-particle sampling; vertex displacement must always use an
explicit LOD. See the workspace author's guide `docs/shaders.md` for the full
contract and costs. No shader compile or draw proves visual acceptance.
