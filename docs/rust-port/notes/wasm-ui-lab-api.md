# Wasm UI and Lab API (design, not implemented)

Decision (user): GD's LAB and GD's custom menu system are **rebuilt** as wasm mods on the Rust mod host
(`rust/melee_modhost`, see `modhost.md`). They are not ported through the C/Lua script host (`pc/gameworld/script_*.c`,
`pc/geno/mods/geno-lab/scripts/lab.lua`, `src/melee/gm/gmfrontend*.inc`); those are a feature list only. With no UI
mod loaded, retail menus run vanilla and nothing below exists at runtime (hashes unchanged, as with sim mods today).

This note defines two ABI extensions to mod-c-v1, **gd-ui-v1** (A) and **gd-lab-v1** (B), plus the shared
interface that lets both halves be built in parallel.

## 0. Ground rules

| rule | consequence |
|---|---|
| UI never writes sim state unless explicitly allowed | UI code runs in a **presentation store**, a second wasmi `Store` per mod that is separate from the sim store. It has no gd sim accessors that write. Its only paths into the sim are the input path (section A4) and the match setup (A5) |
| statics rule (`net.md`) | the sim store lives in `GameScope` and rolls back. The presentation store is **harness-owned** (a field of the play flow, like `play_net::Detacher`), never in the arena and never snapshotted. A savestate load or rollback must not rewind the menu or the Lab itself |
| determinism | sim stores: fuel, NaN canonicalisation and ordering, as today. Presentation stores may use wall time and are never hashed |
| sandbox | as in modhost.md (wasmi 2.0, no start section, fixed memory of at most 1 MiB). The import modules `gd_ui` / `gd_lab` are allowed only when the matching feature bit is declared; any other import still fails instantiation |
| fuel | `UI_FUEL` = 4,000,000 per presentation call. A trap disables that mod's presentation instance, releases its captures and pops its scenes back to the retail flow (logged) |

### Versioning and capability flags

The manifest stays the same as mod-c-v1 (124 bytes). New exported i32 globals: `gdmod_ui_abi` / `gdmod_lab_abi`
(major<<16 | minor). They are required when a UI or LAB bit is set. The rule matches the ABI check: major must
equal the host's, and minor may be lower. New `features` bits (required or optional, as today):

| bit | name | grants |
|---|---|---|
| 4 | `UI_DRAW` | `gd_ui_draw_*`, textures, fonts |
| 5 | `UI_MENU` | node tree, focus, events |
| 6 | `UI_INPUT` | input capture (A4) |
| 7 | `UI_SCENE` | scene register, replace and goto, match setup (A5) |
| 8 | `UI_SIM_INPUT` | capture may **inject** pad input into the sim (offline only; A4) |
| 9 | `UI_HTML` | reserved for Blitz (section D); refused by v1 |
| 15 | `UI_VECTOR` | `VECTOR` records and `gd_ui_path`, through Vello (the optional renderer) |
| 10 | `LAB` | the `gd_lab` module; implies the gate in B6 |
| 11-14 | `LAB_STATE`, `LAB_INPUT`, `LAB_CPU`, `LAB_OVERLAY` | sub-areas B1-B4 (each needs `LAB`) |

A mod may be presentation-only (no hooks) or have both kinds. Sim hooks keep their current rules. Only mods with
hooks enter `play_net::mods_identity`. A presentation-only mod is sim-inert, so it does not split netplay peers.

### Host-owned renderer (performance first)

Performance is the top criterion (Hero's priority). The host owns one UI renderer and mods drive it only through
`gd_ui`. Stack (`unlocked-survey.md` section 3), adjusted for cost:

- `melee_kit`: serde structs for the kit JSON in `menu/out_kit` (the layouts, `font_manifest`, `kit_motion`).
  These are parsed once at boot into flat arrays, so no JSON or serde values are touched per frame.
- **The kit draws through our own batched quad + atlas renderer, inside the existing `melee_gx` wgpu pass.** Kit
  content is flat quads, textured quads, sheared quads (one affine per vertex) and manifest-metric glyphs, and a
  single instanced pipeline covers all of it.
  - Vello is a chain of compute dispatches (flatten, binning, coarse, fine) into a storage texture. That needs
    compute support plus an extra target and a composite. On the low-end and integrated GPUs and the
    DX11/GL-class downlevel adapters Hero targets, it has a fixed cost (estimated 0.5-2 ms, not measured: the
    benchmark below measures it) and VRAM even for a few quads. On downlevel adapters without compute it is
    unavailable.
  - So **Vello is optional, for vector content only** (curves, gradients, a mod's SVG). It is behind the
    `UI_VECTOR` flag and a cargo feature, runs only on frames that contain vector commands, and its result is
    cached to a texture while they do not change.
- **Parley** is for non-kit text (shaping and fallback). Shaped runs are cached by (string hash, role, max_w), and
  glyphs are rasterised once into the shared atlas. Kit text uses the manifest metrics, so widths match the C.
- **Taffy** handles layout of non-kit node trees only. Layout is cached per tree and recomputed only when a node is
  marked dirty (a label, value, flags or show change). Kit screens are absolute 640x480 layouts and never use Taffy.

All UI goes into the same command encoder and render pass as the GX frame (after GX, like `practice.rs`'s
`DrawCmd`s). There is no second surface, no readback and no extra submit. No GPU handle is stored in game memory
(the `net.md` #42 audit holds). Mods never see wgpu.

### Budgets (acceptance criteria for both halves)

| budget | target |
|---|---|
| UI CPU per present, full kit menu (CSS-sized: about 60 nodes and 40 text runs) | **<= 0.15 ms**, hard cap 0.3 ms on the bench machine; Lab overlay plus panels <= 0.25 ms |
| wasm boundary | **one** host call per mod per present to submit draws (`gd_ui_submit`, A1), plus at most about 10 query calls (each crossing is an estimated 50-100 ns in wasmi, to be measured) |
| draw calls | the UI adds **<= 2** draw calls per frame (one instanced quad batch per atlas page, typically 1); a frame with vector content adds Vello's dispatches and 1 composite |
| allocations | **zero** in steady state: the command buffer, quad instance buffer, sort keys and layout caches are preallocated and reused. They grow only at a new high-water mark (logged in debug) |
| GPU uploads | the instance buffer uploads only the used range (`queue.write_buffer`); the atlas uploads only new glyphs and textures |
| mods off | zero cost: no store, no buffer, no change to the pass |

## A. UI layer (gd-ui-v1)

Common types: canvas units are f32 on the 640x480 4:3 band. Colours are u32 RGBA8. Strings are `(ptr, len)` UTF-8
in the mod's memory. Handles are u32, with 0 meaning invalid. Results are i32, where 0 means ok and a negative
value is a `GD_E*` code (`EINVAL -1`, `ECAP -2`, `EOFFLINE -3`, `ESTATE -4`, `EFULL -5`).

### A1. Draw: one command buffer per frame (`UI_DRAW`)

The mod encodes its draws into a buffer in its own linear memory and hands it over **once** per present with
`gd_ui_submit(ptr: u32, len: u32) -> i32`. The host validates the buffer in one linear scan (bounds and opcodes; the
first error refuses the whole buffer) and converts it straight into quad instances. Per-widget host calls are not
offered. The buffer is little-endian and 4-byte aligned. Each record is `u16 op, u16 words`, then its payload:

| op | payload |
|---|---|
| `LAYER` | `i32 layer, i32 z`: sets the current layer and z for the records that follow |
| `RECT` | `f32 x,y,w,h, u32 rgba, f32 shear` |
| `TEX` | `u32 tex, f32 x,y,w,h, f32 u0,v0,u1,v1, u32 rgba` |
| `TEXT` | `f32 x,y, i32 role, u32 rgba, i32 align, f32 max_w, u32 len, bytes` (padded to 4) |
| `KIT` | `u32 kit_id, f32 x,y, f32 scale, u32 rgba`: a named kit piece or icon |
| `CLIP` / `UNCLIP` | `f32 x,y,w,h` / no payload; stack depth 8 |
| `VECTOR` | `u32 path_id, f32 affine[6], u32 rgba` (`UI_VECTOR` only) |

Setup calls are made once or rarely, not every frame:

| import | does |
|---|---|
| `gd_ui_measure(s, len, role) -> f32` | text width, cached on the host |
| `gd_ui_kit_id(name, len) -> u32` | resolves a kit piece name to its id once |
| `gd_ui_tex_load(path, len) -> u32` | loads a `.gxtex` or `.png` from the mod's package dir only (no `..`) into the atlas |
| `gd_ui_path(ptr, len) -> u32` | uploads a vector path once (`UI_VECTOR` only) |

A Rust/C helper crate, `gd_ui_enc`, wraps the encoding, so a mod writes `ui.rect(..)` without crossing into the host.

Layers are, in order: `0 SCENE` (full-screen custom scene, drawn instead of GX for a replaced screen), `1 HUD_UNDER`
(above the world, below the retail HUD), `2 HUD`, `3 OVERLAY` (Lab panels), `4 MENU`, `5 MODAL`, `6 DEBUG`. The
sort is stable on (layer, z, mod order, submission index), so the order is total and does not depend on load order.

### A2. Menu nodes, focus and navigation (`UI_MENU`)

The tree is retained and host-owned, so the kit look, focus, wrap and key repeat are the same in every mod.

| import | signature |
|---|---|
| `gd_ui_node(parent: u32, kind: i32, id: u32) -> u32` | kind: `PANEL LIST TABS BUTTON TOGGLE SLIDER CHOICE LABEL` |
| `gd_ui_node_label(n, s, len)` / `gd_ui_node_value(n, v: i32)` / `gd_ui_node_range(n, lo, hi, step)` | |
| `gd_ui_node_choices(n, ptr, len)` | NUL-separated list |
| `gd_ui_node_flags(n, flags)` | `HIDDEN DISABLED NO_WRAP` |
| `gd_ui_node_free(n)` | frees the subtree |
| `gd_ui_focus(n)` / `gd_ui_focused() -> u32` | |
| `gd_ui_show(root: u32, layer: i32, z: i32)` / `gd_ui_hide(root)` | the host lays out (kit list layout, or Taffy for non-kit panels) and draws |

The host draws node trees into the same instance batch and caches their layout until a node is dirty. It navigates from captured input: d-pad and stick with the kit's repeat; A activates; B goes back; L/R
change tabs. It reports through the export `gd_ui_event(node: u32, ev: i32, value: i32)`, where `ev` is
`ACTIVATE CHANGE BACK FOCUS`.

### A3. Exports (presentation instance)

`gd_ui_init() -> i32`, `gd_ui_frame(present_no: u32)`, `gd_ui_event(...)`, `gd_ui_scene_enter(id: u32)`,
`gd_ui_scene_leave(id: u32)`. All of them are optional, except that `gd_ui_frame` is required when `UI_DRAW` is set.
The call order on each present is input arbitration, then events, then `gd_ui_frame`, then draw. This runs after the
logic frame, so reads see the frame that just finished.

Read-only sim views are available here: the existing gdmod.h getters (fighter pos/vel/percent/facing, players,
items, stage), plus `gd_ui_session() -> i32` (`OFFLINE NETPLAY SYNCTEST REPLAY`) and `gd_ui_scene() -> i32`. They go
through the same handle resolution as the sim host but use read-only linkage. Every write accessor is absent from
the presentation linker.

### A4. Input capture and release (`UI_INPUT`)

| import | signature |
|---|---|
| `gd_ui_capture(port_mask: u32, flags: u32) -> u32 token` | flags: `EXCLUSIVE` (zero the captured ports for the game), `SHARED` (only observe), `KEYBOARD` (host keyboard, offline only), `INJECT` (needs `UI_SIM_INPUT`) |
| `gd_ui_release(token)` | |
| `gd_ui_pad(port, out_ptr) -> i32` | the captured `PADStatus` of this present (after the game's clamp) and its edge mask |
| `gd_ui_inject(port, pad_ptr) -> i32` | replaces that port's input for the next logic frame |

Contract:

1. Capture applies at the **input boundary**, before `set_pads` and before rollback submit. That is the same place
   as `lcancel_helper`, so a `--record` replay carries the result and a netplay peer receives the masked input. The
   game never sees a half-applied capture.
2. Exclusive captures stack per port and the last one wins. Release takes effect at the next frame boundary.
3. Leaving a scene, a trap or unloading the mod releases all of its tokens.
4. In netplay and synctest, `EXCLUSIVE` is allowed on **local ports only** and only in non-match scenes. `INJECT`
   and `KEYBOARD` return `EOFFLINE` there.

### A5. Scene flow (`UI_SCENE`)

| import | signature |
|---|---|
| `gd_ui_scene_register(name, len) -> u32` | a custom scene; drawn on layer `SCENE`; retail GX is off while it is on top |
| `gd_ui_scene_replace(major: i32, minor: i32, scene: u32) -> i32` | from now on, entering that retail scene (for example `mnMain`, CSS or SSS) enters the custom one instead. One replacer per retail scene; a second one is a load refusal |
| `gd_ui_scene_goto(scene_or_retail: u32) -> i32` | at the next frame boundary |
| `gd_ui_scene_back() -> i32` | |
| `gd_ui_match_start(setup_ptr, len) -> i32` | a versioned `gd_match_setup` struct (rules, players, characters, costumes, stage, items): the only way UI builds a match |

Retail scenes that are not replaced run unchanged. In netplay, `gd_match_setup` is not applied locally. It goes
into the existing handshake or room flow, where both peers agree on it, so the sim starts identical on both.

## B. Lab layer (gd-lab-v1)

The Lab is a presentation mod with `LAB`. Its own state never lives in the arena, so a savestate load does not
rewind the Lab. Snapshots and recordings are host objects that the mod refers to by handle. The mod's memory never
holds them.

### B1. Savestates and rewind (`LAB_STATE`)

| import | signature |
|---|---|
| `gd_lab_save(slot: u32) -> i32` | `melee_engine::snapshot` `save()`; slots 0..15, each holds an `SnapRef` |
| `gd_lab_load(slot: u32) -> i32` | `load()` at the next frame boundary; also drops the forward history (it forks the timeline) |
| `gd_lab_slot_frame(slot) -> i32` | match frame of the slot, or -1 |
| `gd_lab_ring(frames: u32) -> i32` | enables the rewind ring (0 = off, max 600) |
| `gd_lab_rewind(n: u32) -> i32` | back n frames: loads the nearest ring snapshot and re-sims to the target with the logged inputs |
| `gd_lab_pause(on)` / `gd_lab_step(n: i32)` | negative n = rewind(-n) |
| `gd_lab_go_live()` | stop replaying logged inputs after a rewind |

The ring reuses the rollback machinery: the same page-range `Snapshot` and the same "load, then re-sim forward with
stored `RbInput`s" loop as `melee_net::rollback`. The difference is a sparse ring (one snapshot every 4 frames,
re-sim the rest), so 600 frames fit in memory. Loads happen only at frame boundaries, so the sim store's wasm state
(sim mods) rolls back with the game, as it already does in netplay.

### B2. Input record and playback, per port (`LAB_INPUT`)

`gd_lab_rec_start(port_mask) -> u32 rec`, `gd_lab_rec_stop(rec)`, `gd_lab_play(rec, port_mask, flags) -> i32`
(flags `LOOP MIRROR FROM_SLOT`), `gd_lab_play_stop()`, `gd_lab_rec_len(rec) -> i32`, `gd_lab_rec_free(rec)`.
Recordings are per-frame `PADStatus` per port and are applied at the A4 input boundary. That makes them replayable
and `--record` safe. `MIRROR` flips the stick X axis. `FROM_SLOT` loads the savestate taken at record start.

### B3. CPU behaviour (`LAB_CPU`)

`gd_lab_cpu(port, behaviour: i32, param: i32) -> i32` takes a behaviour of `NONE STAND CROUCH JUMP SHIELD
LEVEL(param 1..9) RECORDING(param rec)`. `gd_lab_reaction(port, kind, option)` takes:

- kind: `DI SDI TECH GETUP LEDGE SHIELD_DROP OOS`;
- option: `NONE IN OUT AWAY TOWARD RANDOM`, `TECH_IN_PLACE`, `ROLL_L/R` and so on.

These are host-implemented **input synthesisers**, working at the input boundary like B2. RANDOM draws from the
game RNG, so the result stays deterministic and replayable. `LEVEL` sets the retail CPU level on the player.
`gd_lab_set_percent(port, f32)` and `gd_lab_set_pos(port, x, y)` are direct sim writes. They are allowed only offline
and are logged as forking the timeline.

### B4. Overlays (`LAB_OVERLAY`)

`gd_lab_overlay(port_mask, bits: u32)` takes the bits `HITBOX HURTBOX ECB LEDGE_GRAB COLLISION BLASTZONE CAMERA
LABELS SWEPT SHIELD GRAB IASA INVINC`. The host draws these as a GX/wgpu pass from fighter data. It does **not**
set the game's debug-draw fields, so overlays are presentation only and never change a hash. `gd_lab_overlay_style`
takes colours per box kind.

### B5. Read-only queries

All queries copy into the mod's memory and none of them write. They are:

- `gd_lab_fighter(port, out, len)`: the versioned `gd_lab_fighter_v1` struct with action state, anim frame, motion
  id, hitlag, hitstun, IASA, invincibility, shield HP, jumps, ground/air, pos/vel, percent, ECB and grab state;
- `gd_lab_hitboxes(port, out, max) -> n`;
- `gd_lab_hurtboxes(port, out, max) -> n`;
- `gd_lab_framedata(char, action, out, len)`: the subaction timeline;
- `gd_lab_attr(char, idx) -> f32`;
- `gd_lab_frame_adv(port_a, port_b) -> i32`;
- `gd_lab_events(out, max) -> n`: hits, shields, techs and L-cancels of the last frame, with the L-cancel rule from
  `practice.rs`.

### B6. Gating: offline only

- A mod declaring `LAB` must not declare sim hooks (load refusal otherwise). So it is sim-inert and outside the
  handshake digest.
- The Lab instance is created only when the session is `OFFLINE`. Under `NETPLAY`, `SYNCTEST` or `REPLAY`, every
  `gd_lab_*` import returns `EOFFLINE` and B1-B4 state is cleared on session entry.
- The Lab UI may still draw a read-only "offline only" notice.
- Entering netplay with a Lab session active ends that session first. `lcancel_helper`'s `forbid_helper` path is
  the model for this.

## C. Two halves, built in parallel

| half | owns |
|---|---|
| **A (UI)** | presentation store and linker, `gd_ui` imports, input boundary hook (capture, inject, keyboard), scene replace, `melee_kit`, the batched quad/atlas renderer in the GX pass (Parley/Taffy caches, optional Vello), budgets and bench, read-only linker for the gdmod.h getters, example `ui_hello` mod |
| **B (Lab)** | `gd_lab` imports, slots and sparse ring over `melee_engine::snapshot`, recorder and synthesisers, overlay pass, query structs, gate, example `lab_min` mod |

Shared interface: a `melee_modhost::pres` module. A lands it first as stubs, one small merge, and then both halves
build against it.

```rust
pub enum Session { Offline, Netplay, Synctest, Replay }
pub struct PresFrame<'a> { pub session: Session, pub scene: (i32, i32), pub match_frame: i32 }
pub trait InputBoundary {            // A implements; B calls (record, play, synthesise)
    fn read(&self, port: usize) -> PADStatus;            // this frame's local pad after captures
    fn set(&mut self, port: usize, pad: PADStatus, src: u32);  // B2/B3 writes; offline-checked
}
pub trait PresExtension {             // B implements; A's host calls it
    fn feature_bits(&self) -> u32;    // LAB..LAB_OVERLAY
    fn link(&self, l: &mut wasmi::Linker<PresData>) -> Result<(), String>;
    fn on_session(&mut self, s: Session);
    fn before_logic(&mut self, f: &PresFrame, io: &mut dyn InputBoundary);  // inject inputs, pending load
    fn after_logic(&mut self, f: &PresFrame);                               // ring capture
    fn overlay(&mut self, f: &PresFrame, out: &mut Vec<DrawCmd>);           // B4 into the GX pass
}
pub struct PresData { /* per-mod: features, captures, draw list; ext: Box<dyn Any> for B */ }
```

The constants (feature bits, error codes, layer ids, `gd_match_setup`, `gd_lab_fighter_v1`) live in one header,
`docs/specs/gd-ui-lab-v1.h`, with a Rust mirror in `pres`. Both halves are tested offline with scripted pads
(`MELEE_PAD_SCRIPT`). B can stub A with a `Vec<DrawCmd>` sink and a fake `InputBoundary`.

Acceptance checks, per half:

- mods off: goldens unchanged;
- a UI mod loaded with no capture: sim hashes unchanged;
- synctest with a UI mod: 0 mismatches;
- for B: save, then 300 frames, then load, then re-sim gives the same hash as straight play;
- for B: a Lab call under synctest returns `EOFFLINE`.

## D. Blitz (HTML/CSS screens): no

Blitz is DioxusLabs' HTML/CSS engine on Stylo, Taffy, Vello and Parley. We judge it mainly on performance and binary size.

| criterion | assessment |
|---|---|
| CPU per frame | each DOM change means Stylo style resolution, Taffy layout and a full Vello scene rebuild. A menu with animated focus changes the DOM every frame, so it is restyled and laid out continuously. On low-end CPUs that is likely in the millisecond range (not measured), against our 0.15 ms target |
| GPU | it renders only through Vello, so it has Vello's compute cost even for flat kit quads |
| binary size | Stylo, html5ever, selectors, cssparser and the font stack would add an estimated 5-10+ MB to the exe (not measured), plus a large build-time cost, for one optional feature |
| wasm boundary | Blitz has no JS, so mods would drive a DOM through a new set of op imports. That is a second UI API to version, and it crosses the boundary more often than the A1 command buffer |
| determinism and netplay | layout floats and font fallback vary by platform and version. That is harmless for presentation, which is never hashed, but it breaks UI golden tests, and the kit's exact C metrics cannot be met through CSS. It has no netplay impact while it stays presentation-only |

Verdict: not worth it. The `UI_HTML` bit stays reserved and nothing is built. Revisit only if a mod author needs
free-form web screens and accepts an opt-in, size-heavy build.

## Benchmark plan (against Hero's menus)

- **Scenes:** main menu idle; CSS with 4 ports moving cursors; SSS; a settings list scrolling; on our side only,
  the Lab with every overlay on.
- **Builds:** Hero's melee-unlocked menus (his launcher build, used as a reference only) and ours (`melee play` with
  the menu mod loaded). Both run at 1080p, once uncapped (`MELEE_FPS=u`) and once with vsync at 60.
- **Machines:** the dev box, a low-end iGPU (Intel UHD or Vega 8 class), and a forced DX11/GL downlevel adapter.
- **Metrics:**
  - UI CPU time per present, from host timers around submit, layout and encode plus the mod's fuel: p50 and p99
    over 3600 frames;
  - frame time p50 and p99, and GPU time (wgpu timestamp queries, PresentMon on both builds);
  - draw calls per frame (wgpu counters, a RenderDoc capture);
  - allocations per frame, from a counting global allocator in a bench build; this must read 0 after warm-up;
  - exe size and VRAM deltas;
  - menu input-to-photon latency, from the scripted pad edge to the frame where focus changes.
- **Rules:** input comes from `MELEE_PAD_SCRIPT` only, with no OS input. Windows are titled as the test rules say,
  and volume is 3.
- **Pass bar:** every budget in section 0 is met, and our frame time p99 is at least level with Hero's build in
  every scene. Results go in `bench.md`.

## E. Features the mods implement afterwards

**LAB** (from `pc/geno/mods/geno-lab/scripts/lab.lua`, `geno_lab_mode.c`, ui icon set):

- global: pause and resume; step and step back (1 or 10 frames, hold for slow play); 1-9 savestates; rewind and
  go-live; slow-mo; hide UI; help overlay; perf readout; LAB pause menu; focus the next fighter; settings remembered
  per mode;
- modes: CLEAN, HITBOXES (boxes, labels, ECB, hitbox data, swept boxes, hurtbox states, shield, grab), FRAMES
  (timeline with IASA, invincibility, gfx and sfx markers, scrub, lock-step, mirror pad, rollback strip), STAGE
  (collision, ledges, terrain, points, blast zones), INSPECT (model, skeleton, joint numbers, info, attributes,
  event log), MOVES (move library, filter, speed, loop, from neutral), LAUNCH (pick hitbox, DI, percent +-10, live
  percent, victim, knockback arc, KO check), A/B (record A, re-sim B on reloaded or the same data, two fighters),
  TRAINING (frame advantage, move card, input display, tech feedback, record and play back the dummy), COMBO (DI fan);
- tools: frame-data export and diff; fighter size control (offline; forks the timeline).

**Menus** (from `src/melee/gm/gmfrontend*.inc`, as a replacement for the retail `mn*` only when loaded):

- main menu tree with modes: VS rules (stock, time, handicap, damage ratio, friendly fire, items), special melee
  (giant, invisible, lightning, cruel and others), 1P (classic, adventure, all-star, event, home-run, endless),
  data and gallery, bonus records;
- character select (pick, costume, name tags, CPU level, random) and stage select (all stages, counterpicks, coin
  flip and lottery);
- settings pages: audio, display (fullscreen, frame rate, camera mode), controls and controller panel (GameCube
  adapter, recalibrate), language, erase data;
- online lobby: host or join room, room code copy, finding-opponent and locked-in states, input delay, help and
  how-to-play;
- shared: kit styling and motion, mouse support, help and hotkeys page.
