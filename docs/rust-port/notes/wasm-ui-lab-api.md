# Wasm UI and Lab API (A implemented; B design)

Decision (user): GD's LAB and GD's custom menu system are **rebuilt** as wasm mods on the Rust mod host
(`rust/melee_modhost`, see `modhost.md`). They are not ported through the C/Lua script host (`pc/gameworld/script_*.c`,
`pc/geno/mods/geno-lab/scripts/lab.lua`, `src/melee/gm/gmfrontend*.inc`); those are a feature list only. With no UI
mod loaded, retail menus run vanilla and nothing below exists at runtime (hashes unchanged, as with sim mods today).

This note defines two ABI extensions to mod-c-v1, **gd-ui-v1** (A) and **gd-lab-v1** (B), plus the shared
interface that lets both halves be built in parallel.

## Status: half A (gd-ui-v1) implemented

| piece | where |
|---|---|
| presentation host (one wasmi `Store` per mod, harness-owned, `UI_FUEL`, trap = disable + release + pop) | `rust/melee_modhost/src/ui/mod.rs` (`PresHost`); the sim host skips any module importing `gd_ui` |
| command buffer (`gd_ui_submit`: LAYER RECT TEX TEXT KIT CLIP UNCLIP SLICE SPACE; VECTOR refused without Vello) and the sort | `ui/cmd.rs` |
| batched quad + atlas renderer: one 96-byte instance per quad, one 4096x4096 mipmapped, premultiplied atlas page (glyphs, the hatch, mod PNGs; rows allocated as they fill), analytic shapes for every rounded box, circle, ring and glow (slices and round kit pieces are one shape quad each; see `ui-quality.md`), per-instance clip, drawn in the EFB blit pass | `rust/melee_gx/src/ui.rs`, `Renderer::blit_to` |
| node tree (focus, wrap, repeat 15/4, A/B/L/R, layout cached until dirty) | `ui/nodes.rs` |
| theme tokens A C E G | `ui/theme.rs` |
| input boundary (capture, inject, exclusive masks before `set_pads`) and keyboard (GAP-14) | `PresHost::input`, `melee_platform` `PadSystem::ui_keys`, `retail_flow.rs` |
| window space (GAP-0): `SPACE` record + `gd_ui_band_rect` | `ui/cmd.rs`, shader `to_px` |
| scenes: register, replace (on entering the retail scene: custom scene on top, GX hidden, ports held), goto, back | `PresHost::retail_scene`, `scene_switch` |
| text (gd-ui-v1.1): Inter + JetBrains Mono (OFL, `rust/melee_gx/assets/fonts`, subset by `tools/rust-port/fonts/subset_fonts.py`), regular/bold, rasterized once by `ab_glyph` into single-channel SDF tiles (48 texels/em, 6 texel spread; minified through the atlas mips) on the UI atlas when the atlas is built; quads flagged `Q_SDF` are decoded in the same shader with a linear sampler (still one draw call). Style bits in the role word: `GD_UI_TEXT_REGULAR/BOLD/MONO/SIZE(px)`; v1.0 roles unchanged | `rust/melee_gx/src/ui_font.rs`, `ui/cmd.rs` `style` |
| encoder crate, example mod | `rust/mods/gd_ui_enc`, `rust/mods/ui_demo` (Frame data HUD preset, theme A, static data; D-pad left opens a node menu) |
| bench | `cargo run -p melee_modhost --example ui_bench --profile fast -- ../build/mods/ui_demo.wasm` |

Measured (dev box, fast profile, 3480 frames after warm-up): HUD p50 16.5 us / p99 19.7 us, 1152 quads; HUD + menu
(42 nodes, ~110 text runs) p50 21.3 us / p99 26.1 us, 1715 quads; **0 allocations** per present after warm-up; 1 draw
call per frame (one atlas page). In the game (`melee play`, retail flow, 1280x720, `ui_demo`): p50 43 us, p99 111 us
per present including the wasm call; 1646 UI draw calls over 1646 presents (1 per present). Mods off: `PresHost` is
None, `Renderer::ui` stays None, the blit pass is unchanged.

Not done in A yet: `gd_ui_match_start` (returns `GD_ESTATE` until `gd_match_setup` is defined), driving the retail
scene machine from `gd_ui_scene_goto` (only "back to the retail scene underneath"), the read-only gdmod.h getters in the
presentation linker, `melee_kit`/Parley/Taffy (complex shaping: the host lays out one line with per-face advances + ASCII pair kerning), `.gxtex` textures,
the stand-in flow (`MELEE_STANDIN_MENUS`, play.rs). Netplay: captures mask the pads handed to `step`, which feeds the peer.

## Status: half B (gd-lab-v1) implemented

| piece | where |
|---|---|
| `gd_lab` imports on A's `Linker<PresData>` (`LabExt`, a `PresExtension`; installed when a mod imports `gd_lab`), gate (LAB bits need `gdmod_lab_abi`; Offline / LabNetplay only) | `rust/melee_modhost/src/lab/mod.rs` |
| lab commands (GAP-13): every sim-changing call is a 16-byte `LabCmd` queued with its apply frame, which the call returns | same, `LabShared::push` |
| sim side: slots 0..15, sparse rewind ring (a snapshot every 4 frames, re-sim from the logged pads, logged future plays back until go-live), record / play (loop, mirror, from-slot, ordered / random / weighted sets), dummy synthesiser, events, frame-meter cells, observed frame data, B4 overlay pass | `rust/melee/src/lab.rs` (`LabRun`), wired in `retail_flow.rs` |
| ring memory: `Base::save_after` shares the previous snapshot's unchanged pages | `rust/melee_engine/src/snapshot.rs` |
| online lab: `--np-lab` (implies `--np-css`); the match stays in lockstep; commands ride the lockstep stream (`MLS2`: per frame pad + commands, plus a per-frame hash for the desync check); host's commands first on both peers | `rust/melee/src/np_lockstep.rs`, `play_net.rs`, `retail_flow.rs` |
| handshake: presentation-only mods are no longer hashed; Lab mods and `lab cmd v1` are, under `--np-lab` | `play_net::wasm_mods_identity` |
| the LAB mod | `rust/mods/lab` (lab-ui-spec.md status: presets Clean / Frame data / Full, freeze, step, scrubber, slots); `gd_lab_applied` reports the last frame's commands with their issuer |
| example mod | `rust/mods/lab_demo` (status chip, frame meter from `gd_lab_meter`, toasts from `gd_lab_events`, overlays; D-pad controls and an autopilot) |
| checks | `melee lab-check` (engine, headless), `cargo test -p melee np_lockstep lab_identity`, `cargo test -p melee_modhost lab`, `tools/rust-port/netplay/lab_pair.sh` |

GAPs closed: GAP-1 (`gd_lab_events_range`), GAP-5 (reaction param: delay, DI/SDI strength, random-block chance,
"play recording N"; weighted option tables), GAP-7 (lab RNG: a xorshift in the sim side, seeded from the agreed seed
or `gd_lab_rng_seed`; `gd_lab_rng` on the presentation side is a pure function of the synced seed and frame), GAP-8
(`gd_lab_fighter_v1.cell`, `gd_lab_meter`), GAP-11 (`DI_LINE`), GAP-12 (POWERSHIELD, WAVEDASH, LEDGEDASH, SDI, ASDI,
AIRDODGE events), GAP-13 (command format and transport above).

Measured (dev box, release, `melee lab-check`, pad_falcon_fox_fd, headless): step 216 us/frame (baseline without the
ring 194 us); ring capture 1.4-1.5 ms per snapshot every 4 frames = **~250-280 us per frame amortised**, ~630 KB newly
copied per capture, a full 600-frame ring ~87-110 MB distinct page data (it was ~890 MB before page sharing); slot save
1.8 ms; load 1.4 ms; rewind ~2.2 ms (load + up to 3 re-simmed frames). Online pair (`lab_pair.sh`, loopback, 0 ms /
40 ms + 5 % loss): 4733-4737 hash checks per peer, **0 desyncs**, both peers applied the same 9 lab actions (save,
record, stop, rewind, live, load, play mirrored on P2, guest save, guest load) on the same frames.

Not done in B: GAP-2 (speed), GAP-3 (thumbnails), GAP-4 (facing / action reset), GAP-6 (cheats), GAP-9 (action names),
GAP-10 (tint), `KB_ARC`, `BLASTZONE`, `CAMERA`, `SHIELD`, `GRAB` overlay bits (accepted, not drawn); `gd_lab_framedata`
is the observed timeline of the last run of an action, not parsed subaction scripts; `CPU_LEVEL` sets the retail level
(the port must already be a CPU for the AI to drive it); powershield is the game's own `GuardReflect` state, wavedash
/ ledgedash / ASDI are derived from state transitions and positions (documented in `lab.rs`); the dummy's tech timing
is predicted from the last ground height (deterministic, not frame-perfect on platforms). Online, one player's
freeze / rewind / load applies to both (spec open question 4: no consent prompt yet).

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
| `SHAPE` (v1.4) | `u32 kind, f32 x,y,w,h, f32 radius, f32 border, f32 spread, u32 fill, u32 line`: one analytic shape (`GD_UI_SHAPE_BOX` rounded box / pill / circle, a ring with a clear fill, a dashed border with `spread` > 0; `DIAMOND`; `GLOW`). Evaluated per device pixel: exact at every size, resolution, rotation and tilt, always one quad. Prefer it to textures for any round or rounded piece |
| `OUTLINE` (v1.4) | `f32 width, u32 rgba`: the `TEXT` that follows is outlined from its glyphs' distance field (one pass under the fill); width 0 ends it |

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
| `gd_ui_pad_present(port) -> i32` | 1 when a controller is plugged into the port (its read is not PAD_ERR_NO_CONTROLLER), 0 when none; read-only, no capture |
| `gd_ui_inject(port, pad_ptr) -> i32` | replaces that port's input for the next logic frame |

Contract:

1. Capture applies at the **input boundary**, before `set_pads` and before rollback submit. That is the same place
   as `lcancel_helper`, so a `--record` replay carries the result and a netplay peer receives the masked input. The
   game never sees a half-applied capture.
2. Exclusive captures stack per port and the last one wins. Release takes effect at the next frame boundary.
3. Leaving a scene, a trap or unloading the mod releases all of its tokens.
4. In netplay and synctest, `EXCLUSIVE` is allowed on **local ports only** and only in non-match scenes. `INJECT`
   and `KEYBOARD` return `EOFFLINE` there.

### A6. Host catalogs and requests (`UI_HOST`, gd-ui-v1.2): the in-game browser

Feature bit 16 `UI_HOST`, `gdmod_ui_abi` 1.2. Three imports: `gd_ui_list_info(kind) -> gen << 16 | count`,
`gd_ui_list_get(kind, i, out, cap) -> bytes`, `gd_ui_request(op, arg, len)`. Catalogs (MODES, MODS, REPLAYS, SKINS,
STATUS, ONLINE) and request ops (LAUNCH, MOD_SET, REPLAY_PLAY, SKIN_SET, NET_HOST, NET_JOIN, NET_SPECTATE, REFRESH)
and their record fields are in `gd-ui-lab-v1.h`. Host: `ui::PresHost::{wants_host, set_list, take_requests}`; the
play flow side is `rust/melee/src/browser.rs` (catalogs built only at the first present and after a changing request).

What a request does: a retail mode (Training, Event Match, 1P, Stadium; Special Melee is listed but refused, as the
menu's `menuguard` refuses it) starts in place from the main menu with the menu's own two writes
(`MenuExitData.pending_mode`, `gm_801A4B60`), offline only. Like any menu choice that is not an input, a `--record`
replay of a session that used it does not reproduce it. Boot-time modes relaunch `melee play` with this run's options
plus the new ones and quit this run: the LAB / workbench (`MELEE_SCENE=mode=lab|workbench..`), a `.mrpl` replay
(`--replay`, with replay_ctl's controls), online host / join (`--netplay host|join CODE --np-css`), spectate
(`--spectate`, a bare code through `relay:<MELEE_NP_SERVER>/CODE`). `MELEE_BROWSER_DRY_RUN=1` logs the relaunch
instead (headless checks); the log replaces the disc path with `<iso>`. MOD_SET writes enabled.txt (next launch).
SKIN_SET writes `skin_<slot>=` and mounts live offline through `cosmetics::mount` (next match), as the settings
overlay does. Envoy is listed as unavailable (not in this build).

The mod: `rust/mods/browser` (theme A, the LAB's parts). Z on the main menu opens it (a SHARED capture watches the
pads while closed; open: EXCLUSIVE + KEYBOARD on all ports); closed, it submits nothing. Open: one `gd_ui_submit`, at
most 9 host calls per present (4 pads, keys, scene, 2 list infos, submit), no allocation (no_std).

#### A6.1 SSBM Nucleus (gd-ui-v1.3): the Mods section's mod browser

`gdmod_ui_abi` 1.3 (minor bump; a 1.2 mod still loads). Three catalogs, four requests and one import, all under
`UI_HOST` (records and arguments in `gd-ui-lab-v1.h`):

| | |
|---|---|
| `GD_UI_LIST_NC` (6) | the search results, at most 60 (then one "... N more" record, id 0): id, title, author, kind, queue state, percent, picture key, installable, why not, downloads, description |
| `GD_UI_LIST_NC_QUEUE` (7) | the download queue: id, state, percent, title, reason |
| `GD_UI_LIST_NC_STATE` (8) | one record: sync line, catalog size, processing, pending, last install message, restart needed, online, error, site |
| `GD_UI_REQ_NC_SEARCH` (9) | text, tab, sort. The first one starts the host engine (its threads and a sync) |
| `GD_UI_REQ_NC_QUEUE` (10) | post id: queue it, or take it out while it waits |
| `GD_UI_REQ_NC_PROCESS` (11) | "1" starts installing the queue, "0" pauses it (saved: a restart resumes) |
| `GD_UI_REQ_NC_SYNC` (12) | Refresh: skips the 10-minute rule, never the one-minute floor or the back-off |
| `gd_ui_host_pic(key) -> tex` | the texture handle of a record's picture (96 x 72, fitted), 0 while it is not on hand. Ask every frame the picture is drawn: the ask renews it and an unknown key is fetched off the game thread. At most 2 uploads per present into 16 fixed atlas slots recycled least recently used |

Host: `rust/melee/src/nucleus/` (port of `pc/platform/gw_nucleus_*`): `http.rs` (WinHTTP, or curl elsewhere; only
`ssbmnucleus.net` over https, the final address after redirects re-checked; User-Agent
`GDMelee/<version> (GD's Melee Rust port; +https://github.com/GurekamDhillon/melee)`; 10 / 10 / 20 / 30 s timeouts),
`catalog.rs` (the catalog, its JSON-lines cache in the C client's record shape, search, the sync: one full keyset pass,
then `updated_since = W - 15 min` and `/mods/removed`, at most one poll per 10 minutes, a press never twice a minute,
700 ms between pages, 429 / 5xx / no answer back off 5-15-45-135 s or Retry-After, capped at 120 s), `queue.rs` (the
queue, `queue.json` written atomically after every change; on load RUNNING becomes QUEUED, DONE is dropped),
`install.rs` (where each kind goes) and `mod.rs` (the threads). The play flow (`browser.rs`) only `try_lock`s the
engine: a frame where a worker holds the lock shows the previous records. No engine state is a static or lives in
the arena (the statics rule): the engine is a field of the play flow and its threads allocate on the system heap.
Nothing starts while a netplay, replay or spectate session is on; a running download stops and stays queued.

Where installs go: a costume into the skin catalog (`cosmetics::dir()/skins/<slot>/`, typed by the archive's
`Ply<Fighter>5K<Colour>_Share_joint`, with the post's portrait and stock icon as its pictures; the Skins section
mounts it); a stage skin, effect, menu or audio file into `mods/nucleus-<post>-<group>/files/<disc path>` with the C
build's `mod.json` (checked by content against the disc's own file, conflicts turned off, enabled for the next
launch); a GD mod zip's `mods/<id>/` folders (never the exe). Custom stages and MexManager fighter zips are refused
with a reason (they need the m-ex runtime, not ported). Downloads are cached by post version under
`.nucleus/dl` (a retry does not count another download), pictures under `.nucleus/thumbs` (PNG only; a JPEG
screenshot keeps the empty box).

The mod: in Mods, X switches between Nucleus (the default) and the installed mods. Row 0 is the search (A types, the
d-pad cycles a character or the keyboard types; left / right the tab; Y the sort); A on a result queues it, Start
installs the queue, Y asks for a sync. `MELEE_NUCLEUS_PERF=1` logs the game-thread pump cost and the frame interval
every 600 presents.

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

### B5a. Geno (gd-lab-v1.1, `gd_geno_*`)

The LAB's Geno panel over the mod host (geno-wasm-plan.md step 4; declarations in `docs/specs/gd-ui-lab-v1.h`).
`LAB_ABI_VERSION` 1.1 (additive: a mod importing a `gd_geno_*` call fails instantiation on a 1.0 host).

| import | reaches | how |
|---|---|---|
| `gd_geno_profile(port)`, `gd_geno_combo_info(port, what)`, `gd_geno_fly_get(port)`, `gd_geno_fly_state(port, field)` | `gw_Geno_ProfileForKind(fp->kind)`, `GenoGame_ComboInfo`, `GenoFly_Get`, `GenoFly_State` | published per port after every frame (`LabShared::geno`, `melee/src/lab/geno.rs::publish_port`) |
| `gd_geno_state_count(p)`, `gd_geno_state_name(p, s, out, cap)`, `gd_geno_combo_route_name(p, r, out, cap)`, `gd_geno_define_name(ck, out, cap)` | `gw_Geno_StateCount/StateName`, `gw_Geno_ComboName`, `gw_Geno_DefineName` | registry tables, published when the lab run starts and after a reload (`tables`); the presentation side never calls the registry |
| `gd_geno_result(out, cap)` | the last Geno command's code and text | published when a Geno command applies |
| `gd_geno_fly` / `_teleport` / `_fly_attack` / `_fly_clear` / `_combo_ki` (LAB_CPU), `gd_geno_reload` (LAB_STATE) | `GenoFly_Set`, `GenoFly_Teleport`, `GenoFly_AttackConfig` (flight first when on, as gs_fly_attack_cfg), `GenoFly_Clear`, `GenoGame_ComboSetKi`, `gw_Geno_Reload` + `GenoGame_LabReload` / `GenoLab_Leave(3)` | `LabCmd` ops 24-29 (20-23 are the drills'), applied by `LabRun::apply` through `EngineGame` |

Offline only, as the C LAB (`gs_require_offline`): the imports return `EOFFLINE` in `LabNetplay` and the play flow
ignores a Geno op there, so none rides the lockstep stream and `lab cmd v1` stays. A write drops a running lookahead
(the C's `gs_rw_branch`). The reload applies on its command frame (the C rewinds to the request frame and replays; not
done here) and restarts the rewind history there; with the layout changed it restarts a LAB-mode match, else freezes
the lab. Tests: `cargo test -p melee_modhost lab::tests::geno`, `cargo test -p melee lab::geno`.

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

## Amendment: 2-player lab

The Lab supports two players, locally and online. This supersedes "offline only" in B6:

- Sessions: `pres::Session` has `LabNetplay`, an online Lab session, alongside `Offline`. `gd_lab_*` calls are
  allowed in `Offline` and `LabNetplay`. `EOFFLINE` is kept only for normal or ranked `Netplay`, `Replay` and
  `Synctest`.
- Lab actions (savestate, load, rewind, step, record and playback, CPU changes) are not applied locally when
  online. They become synced commands sent through the netplay handshake and lockstep input stream, and both peers
  apply them on the same frame, so the sims stay identical.
- Lab mods take part in the netplay mod check (`play_net::mods_identity`) when online, so both peers must load the
  same Lab mods.
