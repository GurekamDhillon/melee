# Geno through wasm: plan (port/geno3, 2026-11-09)

My user's decision (2026-11-08): the old gd.* Lua script host is not attached in Rust. The wasm mod host
(`melee_modhost`) replaces it, so the LAB and the menus are wasm mods. Pinned Geno (geno-v5.md, which excludes Lua)
is native and stays 1:1.

## What does not change
- **Native Geno**: the registry (`melee_script::geno`), the game half (`geno_game.rs`), defines, base "none", audio,
  and the stable ids / netplay identity. melee geno-tests 64/64.
- **EngineGame** (237 methods: the game half of gw_ScriptGame_*) and **snap_host**: the game side the wasm imports call.

## Work, in order
1. **Records (item 5)**: `geno_records.json` beside the save (`<card folder>/../`; the folder as the card host
   resolves it), `MELEE_GENO_RECORDS` still overriding (docs/geno.md 22.7). Native only.
2. **Controls schema (item 4, docs/geno.md 26)**:
   - parse and validate the define's `controls` block (limits, names, and fallback to no extras on any error);
   - fold it into the content identity as the C does;
   - remap the pads (layout, then extras, no cascades) at the pad read, per port.
   Tap jump is already in Rust (melee_fighter platform + ftCo_Jump*). The per-player choice comes from wasm, because
   the retail CSS has no kit panel:
   - pres import `gd_controls_get(port, out)` (the define's layouts / extras / current choice);
   - pres import `gd_controls_set(port, layout, extras_mask, tap)`, applied before the match (frozen per match, agreed
     in netplay setup like the C). It rides `gd_match_setup` when Envoy's plan lands; until then a pre-match host call.
   Saved choices per profile + content id go through `gd_data_*` when that exists. Until then a host file beside the
   save, as the C does.
3. **Art paths (item 6)**: a define's CSS portrait / results card / stock icon (gm_1601, gmresultplayer, ifstock)
   through `Geno_DefineArtOpen` (already the hook), plus installed skins (kind `skin` mods adding costumes to a base
   "none" define).
4. **`gd_geno_*` for wasm mods** (presentation, read-only; the LAB first). Names agreed against envoy-plan.md 3.9
   and geno-sprite-fighters.md (`gd_fighter_sprite_*`):
   - queries: `gd_geno_profile(port)`, `gd_geno_state_name(profile, s, out, cap)`, `gd_geno_state_count`,
     `gd_geno_define_name(ck, out, cap)`, `gd_geno_combo_info(port, what)`, `gd_geno_combo_route_name(profile, r, out, cap)`;
   - LAB writes become `LabCmd` opcodes (the existing channel): fly mode / teleport / fly attack, combo ki, hot reload
     (`gw_Geno_Reload` with its refusal text);
   - sim-side Geno writes for mods (Envoy's elites) wait for `gd_fighter_mod_set` and are not duplicated here.
   No ABI major bump: new imports are additive (a mod importing one fails instantiation on an older host, as today).
5. **Fighter Lua -> wasm (my user, 2026-11-09: "virtually 0% lua in this project")**:
   - Geno's fighter Lua (geno 10 `lua` blocks, per-state `lua` keys, combo `when.slot`) runs today on the vendored
     Lua 5.4.7 (`melee_script::geno::lua`, geno-tests 64/64). Only vanilla-charger and vanilla-riposte ship it.
   - Replace it with a wasm fighter block in Geno's format: the same callback points (state enter / frame, the
     combo state slots) on the mod host's deterministic, fuel-metered, snapshotted wasmi. It takes a new format
     number, coordinated with the sprite plan's proposed "geno": 12, and the id folds the module hash where it folded
     the Lua hash.
   - Rewrite the two mods' Lua as wasm and prove them against the existing C goldens (geno_vanilla_charger,
     geno_vanilla_riposte) hash for hash.
   - Then remove the Lua dependencies: melee_script's script host and geno::lua, and third_party/lua-src.
   - A `lua` key from a third-party mod becomes a conversion job: it is the one part that does not stay
     format-compatible.
6. **Re-verify items 2-3**: the coverage scenarios (`geno_vanilla_*`, `geno_courier_cpu`, `mex_goku_cpu`) and the v5
   conformance fixture (vanilla-hero jab, section 10).

## Dropped from this job
- The recorder (gw_script_rec.inc): it was a Lua host feature. The wasm LAB has its own record / play over LabCmd.
- Script identity (gs_identity): it was a Lua host feature.
- The parked Lua scene-loop attach (a local patch only).

## geno 12: the `wasm` fighter block (built 2026-11-09, e1c938332)

The geno 10 `lua` block, compiled. Rust-only: the C build reads up to geno 11; `melee_script::geno::GENO_FORMAT` = 12.

```json
"geno": 12,
"fighters": [{ "define": {...},
  "wasm": { "module": "wasm/goku.wasm", "state": { "charge": "int", "counter_damage": "float", "stored": "int" } },
  "states": [ { "name": "KameCharge", "wasm": { "enter": "charge_enter", "frame": "charge_frame" } }, ... ] }]
```

- **Same rules as `lua`**: a define only; at most 16 slots of int / float / bool; per-state enter / frame name
  exports; a frame function brings the "lua" anim callback. The module bytes and the slot layout fold into the id.
  An entry carries `lua` or `wasm`, never both.
- **Module ABI** (`rust/mods/geno_fighters/gd_geno.h`):
  - callbacks are `void f(void)` exports;
  - the ctx is imports from module `gd_geno`: `ctx(key) -> f64` (self.* / input.*), `slot(name, len)`,
    `get(slot)`, `set(slot, v)`, `go(name, len)`, `velocity(fwd, up)`, `hitbox_damage(mask, dmg)`, `loop()`, `turn()`,
    `ki(delta)`, `ledge(mode)`, each with the Lua ctx's checks and command encoding;
  - no start section; `memory` exported with a maximum of at most 16 pages.
- **Runtime** (`melee_engine::geno_wasm`):
  - wasmi with the mod host's deterministic features;
  - a fresh instance per call, so nothing survives a call (the state is the io block's slots, game memory, rolled back
    with it);
  - fuel 200,000 per call, running out = GENO_LUA_FAULT_INSNS.
- **Goku**: `rust/mods/geno_fighters/goku.c` ports goku-xv2's `lua/goku.lua` line for line; `build.sh` builds it with
  clang (wasm32, no libc). `tests/geno_wasm_fighters.rs` runs both over 4000 io blocks x 12 callbacks: identical io words
  and codes.
- **Converting a package**: `tools/rust-port/geno/lua_to_wasm_package.py <mod> <module.wasm> [--out <copy>]` rewrites
  geno.json (geno 12, `lua` -> `wasm`) and copies the module to `wasm/`.
- **Needs-PC**: mex_goku_cpu (and the charger / riposte scenarios once ported) against their C goldens with the
  converted package. Those packages are not on the laptop.
- **vanilla-charger / vanilla-riposte**: `charger.c` / `riposte.c` port their Lua the same way; the differential test
  covers both (4000 blocks each).
- **Still Lua**: goku_ki.lua (the combo / ki build). The Lua removal comes after every port matches its C golden.
