# Render: melee_gx, GX viewer, render proof

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 305-380 -->
## Platform track, step 2: melee_gx (2026-10-21)
- Crate `rust/melee_gx`: a port of the GX the C build gets from Aurora (`extern/aurora`), same
  architecture: `api::Gx` = the GX API encoders of `lib/dolphin/gx/*.cpp` + the `__GXData` shadow
  registers, writing BP/CP/XF/`GX_AURORA` commands into a big-endian FIFO (or a display list);
  `state::GxState::process` = `regs.cpp` + `command_processor.cpp` decode (BP mask, register dedupe,
  cull swap, tref channel remap, VAT/VCD, XF matrices/lights/viewport/projection); draws become
  `draw::DrawCmd` (raw vertex bytes, Aurora's index buffers, array snapshots, uniform bytes).
  `shader` = `shader.cpp`/`shader_info.cpp`/`attr_fmt.cpp`/`populate_pipeline_config`, generating
  Aurora's WGSL text byte for byte; `texture` = `texture_convert.cpp` (I4/I8/IA4/IA8/RGB565/RGB5A3/
  RGBA8/CMPR, C4/C8/C14X2 + IA8/RGB565/RGB5A3 TLUTs, arbitrary-mip check); `render::Renderer` =
  wgpu 27 offscreen target with Aurora's resource model (vbuf/abuf storage, 256-aligned dynamic
  uniforms, 64-byte immediates as push constants, reversed Z, CW front faces, Aurora's blend/depth
  mapping). Pointers in the FIFO are handles (`resources`); game data stays GC big-endian bytes.
- Verified (`cargo test -p melee_gx`), against Aurora's own code, not a re-implementation:
  `oracle/build.py` compiles `oracle/oracle.cpp` with the flags CMake used for aurora_gx and links
  the shipping Aurora build (`_build/agents/rustgold/ax86m`, MSVC x86), then writes tests/data:
  `tex_oracle.txt` (145 decode cases, FNV of the output), `shader_oracle.txt` (400 random configs,
  WGSL length + FNV + uniform size), `state_oracle.txt` (GXInit state + 300 random sequences of 36
  kinds of GX call; ShaderConfig, pipeline state, uniform-byte and WGSL hashes) and
  `render_aurora.png` (Aurora's screenshot of `oracle/scene.inc`: a 2-stage-TEV textured quad /
  strip / fan from an INDEX8 S16 position array, and a per-vertex-lit alpha-blended pair of
  triangles, both via GXCallDisplayList). All match exactly; tests/render.rs draws the same scene
  and gets 0 differing pixels on wgpu Vulkan and DX12 (RTX 5070) vs Dawn D3D11. Negative check: a
  one-character change in the generator fails 251/400 WGSL cases and the state test.
- WGSL for wgpu (`render::adapt_wgsl`, applied after generation, so the oracle compares Aurora's
  text): `var<immediate>` -> `var<push_constant>`; naga rejects `ptr<storage, ...>` function
  parameters, so they become a buffer selector (0 = vbuf, 1 = abuf); naga does not fold builtins
  over abstract floats (`mix(0.0, 0.0, 0.0)` in an all-constant alpha combiner), so literals on alpha
  combiner lines are typed `f32(...)` (exact values).
- Traps: the game calls GXInit (aurora_initialize does not); Aurora skips a draw while its pipeline
  compiles asynchronously (the oracle screenshots at frame 60); this Dawn build has no Vulkan and its
  D3D12 swapchain faults in `WaitForQueueSerial` in the oracle harness on this machine (oracle uses
  D3D11); `aurora_shutdown` faults after the device is destroyed (oracle `_Exit`s); Aurora writes its
  pipeline-coverage db to %LOCALAPPDATA%\GD Melee while the oracle runs.
- Not ported / not verified: the `pc/platform/shim_gx.c` layer above Aurora (indirect-stage guard,
  GXInitFogAdjTable, fog scope, GXProject/GXGet*, VI draw-done routing, draw suppression during
  resimulation), EFB copies (GXCopyTex/GXCopyDisp; logged), PC palette / surface programs / frame
  interpolation / motion capture, Aurora's draw merging and pipeline/uniform/texture caches (every
  draw recomputes; the renderer caches pipelines, samplers and decoded textures), texture
  replacements, swapchain/present (step 3 joins this to melee_platform's winit loop), MSAA.
  Lines/points, fog, indirect texturing and emboss are WGSL-verified but not rendered by a test.

## Platform track, night 3 step 1: shim_gx + swapchain/present + gx_viewer (2026-10-21)
- `melee_gx::shim::GxShim` is pc/platform/shim_gx.c in front of `melee_gx::api::Gx`. The C shim's
  big-endian conversions (matrices, float arrays, GXColorS10, render-mode images) are gone: the
  native port keeps game state native LE, so the shim takes native values. File data the GPU reads
  (vertex arrays, display lists, textures, TLUTs) stays the DAT's BE bytes, as on the GC. What the
  shim does is ported: the indirect-stage guard (`IndGuard`, lbRefract leak), `gx_project` (one f32
  rounding per op, it reaches the sim via Camera_LogicToScreen), `init_fog_adj_table`, the 3-arg
  `set_array`, camera-shake projection offset (`shake_offset`, gw_Fx_ShakeOffset's output), draw-done
  routing to HSD's callback, `suppress_draws` (resimulation), stats, the three render modes
  (`GX_NTSC_480_*`, `GxRenderModeObj` with GC layout), stubs. GXCopyDisp ends a frame: the FIFO is
  decoded and `take_frame()` hands the draws to the presenter. Aurora functions melee_gx lacked:
  GXGetTexBufferSize, GXInitLightSpot/DistAttn, GXGetProjectionv/Viewportv.
- `melee_gx::render::Renderer::with_instance(instance, Some(&surface), 640, 480)` picks an adapter
  that can present; `render_frame` draws into the EFB without readback, `blit_to(view, format)`
  scales the EFB onto a swapchain image (alpha forced to 1). `melee_platform::gfx::Presenter` owns
  the wgpu surface on the winit window (non-sRGB format preferred, AutoVsync, resize/lost
  handling) and takes EFB screenshots. `Game::present` now gets `&Arc<Window>`; `Game::resized`.
- Trap: `std::cos(float)` in Aurora on x86 MSVC (and clang i686-msvc) is `(float)cos((double)x)`,
  not a single-precision cosf; `init_light_spot` does the same (single cosf differed by an ulp in
  2 of 2000 cases, magnified by 1 - cr). And C's `(u32)f` on x86 leaves NaN/inf/>=2^32 with low bits
  0 where Rust's `as` saturates (fog table entries `000` vs `fff`): `x86_f32_to_u32`.
- Verified: `cargo test -p melee_gx` (shim_oracle: 9000 cases bit-identical vs the C cut verbatim
  from shim_gx.c/Aurora by oracle/shim_build.sh and built as shipping shims; a fused GXProject term
  fails 335; shim unit tests; the existing render test still 0-diff). `gx_viewer` on this machine:
  `cargo run -p melee_platform --bin gx_viewer -- --stage GrNLa.dat` (disc from GW_ISO_VANILLA)
  opens a window, presents FD (and `--stage GrNBa.dat` Battlefield) through GxShim -> FIFO ->
  decode -> WGSL -> wgpu -> swapchain, and writes the frame-60 EFB to rust/target/gx-viewer/
  (disc art, never committed). `--synthetic` draws a generated stage with no disc; its shot is
  docs/rust-port/shots/gx_viewer_synthetic.png. Checked by looking at the screenshots, no oracle.
- Not done: no game code calls GxShim yet (HSD's JObj/DObj/MObj/PObj disp is group 1's; the
  c2r `extern "C" GX*` declarations are not bound to it); EFB->texture copies; the PC matrix
  palette and the palette/envelope caches; frame pacing against VI; the viewer's material model is
  a stand-in (bind pose, first texture, one light, opaque alpha test at 128).


<!-- RUST_PORT.md lines 1550-1640 -->
## Platform track, render proof (plat-render, rounds 1-2, 2026-10-25)
- Method: C shots from `_build/agents/plat-render/pad_cam.lua` (`gd.screenshot` at match frames) vs `melee view-golden` shots.
  RUN THE C BUILD REALTIME, not turbo (`MELEE_UNATTENDED=1 MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000`, no `MELEE_TURBO`, run
  `melee-pc.exe` directly in a scratch dir with `MELEE_SCENE=mode=vs;at=match;p1=falcon/human;p2=fox/human;...;stage=fd`
  and `MELEE_PAD_SCRIPT=pad_cam.lua`; no window focus, no OS input). With realtime the screenshot for label n is exactly the frame of
  Rust tick n (`native_000n.png`; the camera is bit-identical). Round 1's turbo shots were captured some 13-22 frames late (and later
  frames drifted), which made its 900+ numbers meaningless. Compare: `tools/rust-port/shotcmp3.py RUSTDIR` (C shots in
  `cshots3/`, both box-downscaled to 640x480). Run the viewer as `MELEE_VIEW_SKIP=1 rv.sh DIR 1505 60,120,300,...`.
- Fixed this round (mean abs error 0..255, previous -> now): f60 10.6 -> 4.7, f120 12.6 -> 3.7, f300 22.2 -> 2.0, f600 19.6 -> 2.4,
  f900 14.3 -> 1.95, f1200 17.9 -> 1.4, f1500 19.5 -> 3.6 (average 16.7 -> 2.82; within16 %: 88.5 / 92.1 / 97.7 / 96.8 / 97.8 / 97.8 / 93.7):
  1. `melee_runtime::tu::mobj::setup_tev`: a TEV stage selects ONE konst colour; a TObj TEV of `(tev0 - konst) * tex` needs two constants
     in the same stage and the second silently collapsed into the first (result black). Extra distinct colour / alpha constants now go to
     TEV registers REG1 / REG2. This is the whole FD Milky Way dust band, the big nebula layers and every `0x85 0x80` gradient-map TObj.
  2. The FD backdrop animations (map gobjs 4-8) never move in the C build. `grAnime_801C752C(.., HSD_AObjSetRate, 1, rate)` calls the
     setter through `Callback1 (aobj, obj, flags, float)`; on the x64 build the float is the 4th argument but `HSD_AObjSetRate(aobj, f32)`
     reads its 2nd, so every AObj those calls touch gets framerate 0 (found by sweeping the viewer's animation phase against the C
     shots: best phase = -t at every frame). `stage_anim::grdo_anim` now sets rate 0 (`MELEE_VIEW_PPC_RATE` restores the PPC value).
  3. Fighters are lit by `stage_info.map_plit` (`ftCo_8009F4A4`), the stage by the light group (`Ground_801C466C`); they were one set.
     Stage light sets are animated (`HSD_LObjAnim`: VIS / LITC_* tracks) by `view::AnimLights` (FD's lights have no animation).
  4. Name tags (`ifnametag.c`, `ScInfPnm_scene_models[0]`): scale 10, `ReqAnimAll(5 * port)` (the frame picks label + colour), translated each
     frame to the head (`cur_pos + name_tag_height - 2.5`) projected by the match camera, own ortho camera (640x480, eye z 300), removed when
     the Go element is created (`un_802FD428`).
  5. Fighter bodies obey `invisible` / `x221E_b5` (`ftDrawCommon_80080A18`).
  6. Effect models (`efLib_Create`): the JObj trees of the effect banks (`EfCoData` & the per-character `Ef*Data`), with their joint and
     material animation, are drawn at the root the sim keeps (`effect_draws`: owner position or fixed spawn position, scale, Y/Z rotation
     overrides) by `view::FxModel` (a `StageAnim` per live effect; `EffectBank::model_dat` appends one synthetic `{joint, anims[], matanims[]}`
     record per effect id). This is the dash / skid dust rings, landing dust, the KO beam, shield and so on; `efSync_Spawn(0x42B)`'s
     `efLib_SetTevKonstColor` tint (PlCo pData[18]/[19]) is applied, and TObj tev colours now come from the live TObj
     (`TObjState.obj.tev`, initialised from the descriptor) so tev tracks animate too. Not drawn: effects whose animation has a PATH
     track (effect 36, the entry beam: needs the spline JObj of the AObj's `obj_id`), effects attached to a fighter bone (they follow the
     owner's root position only), `lb_8000C1C0` RObj constraints.
  7. Sim ordering bug found on the way: `ftCommon_8007DB24`'s `efLib_DestroyAll` is queued (`ef_ops`) to the s_link 9 flush, which freed the
     KO effect `dead_down` spawns right after it; `dead_down` now runs the queued DestroyAll first (the golden is still identical on
     all 1801 frames).
- AppSRT particles (`psDispSubAppSRT` quads and points) are ported in `ps_view.rs` but OFF by default (`MELEE_VIEW_APPSRT=1`): the KO / blast
  zone sprites they draw come out as oversized blocky quads next to the C build's thin beams (extent or texture still wrong), which scores
  worse than skipping them.
- Still different (largest first): (a) fighter colour overlays: the C paints the Entry glow (`ftCo_800BFFD0(0x75)` colour animation + the
  trophy-stand accessory, the white silhouettes of frames 25-85), the Rebirth ghost (Falcon is lavender and translucent at 300/900/1500: colour
  overlay, `ftMaterial_800BF2B8` adds TEV stages) and the respawn platform; none of `ftcolanim.c` / `ftmaterial.c` / `lb_013B` is ported (the sim
  only records them); (b) KO frames (215-250, 500-540, 790-830): the KO star / "SCORE -1" text / the exploding percent digits / the pillar's
  exact size and fade (errors 12-26 vs 2-4 in plain frames); (c) entry beam light pillars (PATH effect), C frames 1-45 light up blue panels on the
  stage underside that the viewer keeps dark; (d) floor texture aliasing (tiny amplitude). Shadows: FD draws none; fog: FD has none
  (`stage_info.unk8C.b3` unset); screen flashes: none occur in these frames.
- Round 3 (2026-10-25): fighter colour animation + overlay and particle texture fix; per-frame mean abs (0..255) f60 4.13, f120 3.51,
  f300 1.71, f600 2.13, f900 1.57, f1200 1.15, f1500 2.84 (avg 2.43, was 2.82); within16 88-99 %. Golden unchanged (1801 frames identical).
  1. `colanim.rs`: `lb_013B` colour-animation interpreter over PlCo tables, the x408/x488/x508 lists; `ftMaterial_800BF534/6BC`
     overlay (colour blend, light, alpha) as `FtOverlay` TEV stages (`HsdRender::ft_overlay`). Entry glow (0x75), Rebirth / intangible pulse (9).
     `Fighter_ChangeMotionState` with a row and `anim_start == 0` also calls `ftCo_800C0408` (an extra colour step; fixed the 1-frame phase of the pulse).
  2. `TexBank::image_off / pal_off` now add the texBank base (`psInitDataBankLocate` phase 3 relocates texTable by `texBank`); banks
     whose texg is not at offset 0 (EfCoData bank 0: entry sparkles, star flares) had garbage textures.
  3. Go! element is one anim frame in when drawn.
- Still different: entry trophy stand / light pillars / PATH-track beam effects (f60), name tag ~3 px high, KO frames, underside texture
  detail (blue haze in diff, tiny amplitude).
- Dev aids: MELEE_VIEW_UNHIDE, MELEE_VIEW_PSLOG, MELEE_VIEW_READYLOG, MELEE_VIEW_NOPS (round 1); MELEE_VIEW_BRANCHSCAN / PODUMP=g:j /
  TEXDUMP2 / TOBJDUMP / TAGDUMP / ONLY=g:j,j / FORCEVIS / PREROLL=n / FPLOG / LIGHTLOG / NOPLIT / NOLIGHTANIM / PPC_RATE / NOFX / FXLOG / APPSRT / APPLOG / APPFRAMES, and
  `MELEE_GX_DRAWLOG=<copy index>` (+ `MELEE_GX_DRAWLOG_TEV=<draw>`) in melee_gx's `copy_disp`.

- Round 4 (plat-render, 2026-10-25): other stages + KO + HUD timing. Method unchanged (realtime C shots, `MELEE_WINDOW_UNFOCUSED=1`, no OS input), but
  `MELEE_VIEW_SKIP=1` is NOT used for measuring any more: skipping draws hid a bug (below). Scenarios: `view-golden --scenario pad_falcon_fox_{fd,bf,ys,gr,fod,dl64}`
  (same pad cycles; the native sim only reaches frame 338 on BF, 196 on YS, 263 on Green Greens, 83 on FoD before an unported fighter state / blast zone,
  so only those prefixes are comparable). Compare with `tools/rust-port/shotcmp_dir.py CSHOTDIR RSDIR f1,f2,..`.
  Mean abs error (0..255) per stage, before -> now:
  | stage | frames | before | now |
  |---|---|---|---|
  | Battlefield | 60/120/300 | 26.4 | 2.65 |
  | Yoshi's Story | 10..190 step 10 | not drawn right (one rooted group set) | 3.2 |
  | Final Destination | 60..1500 | 2.43 | 2.33 |
  | FD KO window | 196..288 step 4 | 6.18 | 3.41 |
  | Green Greens | 60 | 6.8 | 5.8 (frame 120+ shows the grGreens star blocks the C animates; not ported) |
  1. `StageAnim` PATH tracks work: the AObj obj id is the spline JObj's DESC address (the C ID table key); the spline JObj is resolved inside the group's tree
     (or loaded next to it like `HSD_JObjLoadJoint`), `spline_point` reads the HSD_Spline from the DAT (`splArcLengthPoint`). Fixes the BF background panic.
  2. Stage gobj sets are per stage: Battlefield = grBattle_OnInit gobjs 0, 1 (pass 2, the active background), 6 (light group), the BG controller 3 is hidden;
     Green Greens = 0, 2 (pass 2), 6, 5, 4. Everything else the old "draw every group" path drew (the three other backgrounds at once, blocks).
  3. `grGroundParam->y` (Battlefield 0.8, Yoshi's Story etc.) is the scale of the wrapper JObj every map gobj hangs under: `StageAnim::world_scale` scales the
     world matrices. This was the whole BF error (stage 25% too large next to the fighters).
  4. `StageAnim::set_translate/set_scale/set_rotate_yz` now `HSD_JObjSetMtxDirty` (children follow). Before, a moved root left its children's matrices stale
     when a frame was drawn more than once; the name tag's arrow drifted up 8-15 px when every frame was drawn (SKIP mode drew once).
  5. Name tags stay visible after Go while the fighter is on its rebirth platform (`Player_80036058` = `ftLib_80086F4C`).
  6. KO camera quake (`Camera_RequestQuake(QuakeKind_Large, pos)` in `dead_down`, requests queued in `VsMatch::quake_requests`): the quake gobj
     (`grLib_801C9CEC`: `stage_info.quake_model_set` anim set 3, its root translation = `Camera_SetQuakeOffset`) is stepped by the view one frame after the request,
     before `CmDrive::step`. The camera now equals the C's `gd.camera_get` log exactly on every frame 205-259 (it was 2-3 units off from the first KO frame, which
     made every KO frame score 15). Quakes from other death states are not requested yet (those states are not ported).
  7. The Go element is not drawn for its first three frames (C 84-86 show nothing, 87 is fully settled); FD frames 84-86 went 63/40/23 -> 3.4.
- Not matched (largest first): (a) HUD KO pieces: exploding percent digits, "SCORE -1" score model, stock-loss animation (`ScInfStc`), the magnifier bubble for an
  off-screen fighter (`ifmagnify.c`, YS C frames 160/190); (b) FD frames 1-12: the C shows z-fight / interior geometry because the camera starts nearly edge-on to
  the top plane (depth precision, not logic); (c) Fountain of Dreams / Dream Land 64 / Pokemon Stadium: scenarios exist for FoD and DL64 (`GrIz.dat`, `GrOp.dat`,
  no stage logic: the platforms, the reflection camera and water are not driven; error 30 / 8 at 60), Pokemon Stadium has no scenario; (d) Randall (YS) and the
  Shy Guys are not ported; (e) AppSRT particles still off by default; (f) the C shots are taken while the build hitches, so consecutive-frame comparisons
  alternate between two logic frames; compare spaced frames.
