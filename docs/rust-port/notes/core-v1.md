# Core v1: the Rust engine inside Hero's Source Port host

Status 2026-11-07, branch `port/corev1` (from rust-port fa7140ca0). Hero88go's melee-unlocked adds a frozen game/host
boundary, `port/runtime/abi/mu_core_v1.h` (local clone, branch `gd/core-v1` 4359db5 on his main 53bbcc9; nothing has been
sent to him). This note covers our side, which was written from that header alone. None of his code is copied here, and
none of ours went to him.

## What the boundary is

There are two append-only tables. The host gives `log`, `poll`, `ticks`, `stop`, `gx_fifo`, `vi_wait_retrace`,
`mem1_base`/`mem1_size` (40 MB at 0x80000000), `pad_read`, `disc_entrynum`/`disc_file`/`disc_read`, the memory card
calls, and `native_alloc`/`native_free`. The game gives `run`, `retrace`, `fire_alarms` and an optional `ai_dma_done`.
There is no audio, and there are no state hooks (no savestates, rollback or practice bridge). The library exports
`mu_game_entry_core_v1` and must load at 0x82800000.

## Our build: `rust/melee_corev1` (opt-in, normal builds unchanged)

- The package `melee_corev1` is a cdylib named `melee_game.dll` whose lib root is `../melee/src/main.rs`. These are the
  same sources as `melee.exe`, built with feature `corev1`. The `melee` package declares the feature but never enables
  it. Building them in separate cargo invocations keeps features from unifying, so `cargo build -p melee` compiles
  nothing new: `melee.exe` holds no core v1 string, and the gate passes.
- `melee/build.rs` adds `/BASE:0x82800000 /DYNAMICBASE:NO`, and the version resource goes to the cdylib, only when
  `CARGO_FEATURE_COREV1` is set. Image size is 0x23DC000 (about 36 MB). His unsandboxed loader only checks the base;
  his sandboxed mapper would refuse an image past 0x84000000, which is untested.
- `melee/src/corev1.rs` holds the `#[repr(C)]` tables, with the header's pinned offsets as compile-time asserts.
  - Entry refuses an ABI other than 1 or a host table shorter than ours.
  - `run()` is `melee play` (the retail flow), with extra arguments from `MELEE_COREV1_ARGS` (default
    `--skip-intro`).
  - `retail_flow::run` hands the loop to `corev1::drive`: one tick, then `vi_wait_retrace()`. The host paces and
    presents.
  - `MELEE_WINDOW_TITLE` is applied to the host's window from our side (Win32 `SetWindowTextW` on this process's
    visible windows).
- Disc: `melee_engine::gcm::HOST_READ` routes every raw read through `disc_read`, polling until it completes. The
  `--iso` path is a placeholder; the host's disc is the one used.
- Pads: `melee_platform::pad_read::HOST_PADS` makes `pad_read` ports 1-4 when no `MELEE_PAD_SCRIPT` is set. A pad
  script drives on top, which is how the tests ran.
- GX: `melee_gx::corev1::Transcoder` sits under feature `corev1`, with a hook in `Gx::drain` and one in
  `GxShim::copy_disp`. Our FIFO is Aurora's, while his decoder is a console command processor that reads memory by
  physical address. The transcoder rewrites each drained buffer:
  - Every handle's bytes are copied once into a MEM1 bump arena from 1 MB up, cached by the shared buffer, which is
    kept alive while mapped. The registers then carry console addresses: CP 0xA0+n for arrays, BP TEX_IMAGE3 for
    textures, BP LOADTLUT0 before LOADTLUT1.
  - When the arena fills mid-frame, the transcoder sends a draw-done, flushes, restarts the arena and re-sends what is
    bound. This happened 6 times in a 3400-tick run.
  - `DRAW_SIZED` becomes a counted primitive and `DRAW_INDEXED` a triangle list. Neither occurred in the VS run.
  - GXCopyTex becomes the console's EFB copy registers, into a 4 MB pool at the top of MEM1, kept per destination. A
    texture whose `guest` image is a copy destination samples that block.
  - GXCopyDisp becomes the XFB copy that ends his frame.
  - PC-only subcommands are dropped and counted: render viewport/scissor/projection, polygon offset, pose tags, and
    the Geno matrix palette.
- Four encoder differences from the console that his decoder exposed, all fixed in the transcoder (our renderer is
  untouched):
  1. BP 0x59, the scissor box offset, is never written. The transcoder writes GXInit's 171/171 once. Without it,
     everything was clipped to the bottom-right quadrant.
  2. The viewport origin is biased by 340, not the console's 342. The transcoder adds 2 to XF 0x101D/0x101E.
  3. LOADTLUT1's size field is written as entries-1, where the console counts 16-entry blocks. The transcoder
     rewrites it, so palette loads no longer clobber each other.
  4. TEX_TLUT carries no palette format. The transcoder adds the format of the loaded slot, so CI textures are no
     longer decoded as IA8 (grey portraits and stage trim).

## How to run

```
cd rust && CARGO_TARGET_DIR=<shared fast target> bash ../tools/rust-port/cargo_fast.sh build -p melee_corev1 --profile fast
HOST_EXE=<his build>/port/Release/melee_source.exe HOST_DIR=<his checkout root> \
  bash tools/rust-port/corev1/run.sh host <out> 3400 [--capture <out>/cap/f.ppm --capture-sim-frame N]
bash tools/rust-port/corev1/run.sh ref <out> 3400 590-615,1690-1715     # our renderer, same script and frames
python tools/rust-port/corev1/compare_shots.py <dir with h<N>/> <ref out>
```

Notes on running:

- His host finds the library through `MELEE_GAME_DLL`.
- It must run with his checkout root as the working directory, for the Sys tree; elsewhere it stops with
  "Sys generation failed".
- His host was built in his normal configuration: `cmake -DMELEE_BUILD_EXPERIMENTAL_PORT=ON`, target `melee_source`,
  after `recomp.py` has filled `port/generated`.
- His `MELEE_NO_SLIPPI` configuration does not compile on his main. The errors are in `pc_settings.cpp`,
  `hle_pad.cpp` and `host.cpp` (Slippi-only symbols), are unrelated to core v1, and were left alone.
- Building his Windows full host with core v1 needed no change, so `gd/core-v1` has no new commits.
- `--capture-every` counts presented frames, not sim frames, and drifts. `--capture-sim-frame N` counts his XFB
  copies, which run 2 to 5 below our tick because scene loads skip a copy.
- `--window` and `--aspect 4:3` zoom his back-buffer capture, so compare his default presentation instead.

## Results (retail_full.txt: title, VS, four CPUs, Battlefield, 30 s, one match; fast profile; one machine)

- Boots the retail menus to a VS match and plays it to the end in his host, at 59.9 fps (retrace-paced, 3400 ticks
  in 56.8 s).
- Sim: the `MELEE_HASH_LOG` per-match-tick NHV1 hashes are identical to our own windowed renderer's run, 2174 of 2174
  ticks, in two separate runs.
- Cost per tick, averaged over the whole run (menus included):
  - engine tick 2.60 ms, of which his `gx_fifo` decode is 1.47 ms;
  - transcode 0.125 ms;
  - about 1 ms for our sim.
- MEM1 traffic: 1110 MB of FIFO out, 238 MB staged, 8702 texture copies.
- Screenshots, his capture registered to ours (his presenter pillarboxes the EFB to 73:60, crop x 56..1224 of 1280),
  640x480:

  | his XFB seq | our tick | mean abs diff | PSNR | px off by >24 |
  |---|---|---|---|---|
  | 600 (CSS) | 602 | 2.41/255 | 31.6 dB | 1.8% |
  | 1700 (match, 0:54) | 1705 | 9.16 | 20.2 dB | 14.1% |
  | 2300 (match) | 2304 | 4.46 | 24.7 dB | 6.8% |
  | 2900 (match) | 2904 | 2.12 | 28.7 dB | 3.2% |
  | 3300 (match, 0:27) | 3305 | 11.63 | 17.8 dB | 16.1% |

- What differs in the renders:
  - The same state is on screen: timer, damage, poses and particles. No geometry or texture is missing.
  - The 3D view is horizontally wider in his frame than a rescale of ours, so after HUD-anchored registration the
    scene edges are off. This is his 73:60 presentation, not the stream.
  - Filtering and edge differences come from rescaling.
  - Small HUD glyph placement differs.
  - Brightness and colour match on the menus.

## Core v1.1 (branch `port/corev11`, his `gd/core-v1` f7d55cf; local, not pushed, not sent)

Both tables only grow (ABI 1, same entry point). A side tells the other's revision from `struct_size`: host 240 / 248,
game 40 / 88 bytes (`MU_CORE_V1_*_SIZE` in the header).

- `MuHostCoreV1.audio_out(lr, frames, 32000)`: our mix, captured per tick (`audio::HOST_MIX`), goes to his
  `audio_push_native`. Our sound reads the disc through the host (`engine_audio::HostFs`). A v1 host gets `--no-audio`.
- `MuGameCoreV1.state_*`: a VS match runs under `retail_flow::Hosted` (a `ModesGame` with keyed snapshots and the pads
  each frame last ran with). The host calls the state functions between ticks, inside `vi_wait_retrace`.
  `MELEE_PLAY_HOSTED=1` runs the same session in our own exe.
- His card behind our CARD hooks (`melee_engine::card_host`) when `MELEE_CARD` is not 0.
  `MELEE_COREV1_CARD_TEST=1` exercises his 14 calls directly.
- `stop` ends the run; `ticks` measures the host clock.
- Geno's matrix palette is converted to console matrices: triangle batches of at most 10 slots, loaded into XF
  matrices 0, 3 .. 27, then the game's own matrices are put back (`melee_gx` unit test
  `palette_strip_becomes_console_matrices`). Not yet seen in a run: no Geno fighter is on the default disc.
- Rollback safety in the transcoder. It lives on the system heap, outside the statics a load rewinds. Frames being
  re-simulated (`set_suppress_draws`) send no draws and no copies. In runs that load states it holds no game buffer
  (`ROLLBACK`): its cache is keyed by address, length and a sampled fingerprint. Before this, his window under a
  synctest showed black and flickering frames at 39 fps, with 12 GB staged, and then crashed.
- `static_sections` uses the module that contains our code, not the process's main module (the host's statics).

Results, fast profile, one host at a time, on one machine:

| run | result |
|---|---|
| Battlefield retail_full, v1.1 host + v1.1 game | 59.9 fps; hashes 2174/2174 same as our own engine; his WAV dump byte-identical to our mix (1,813,280 frames) |
| his state test K=7 (`MELEE_COREV1_STATE_TEST=7`) | 50 round trips and 50 rollbacks with no difference; first-pass hashes same as the reference |
| synctest depth 7 inside his host | 56.2 fps (expected cost of re-simulating); 3831 passes, 26,817 checks, 0 neutral mismatches; pacing p99 8 to 9 ms |
| FD, Yoshi's Story, Pokemon Stadium, Fountain, Dream Land N64 (CPU v CPU, `MATCH=`) | hashes 2600/2600 and mix byte-identical on each; 59.7 to 59.9 fps |
| v1.1 host + v1 game (`MELEE_COREV1_FORCE_V1=1`) | savestates off, no audio_out; hashes same |
| v1 host + v1.1 game (`MELEE_COREV1_HOST_V1=1`) | savestates on, sound off; hashes same |
| card test, scratch `--card-dir` | his 14 calls all as expected; the game created and loaded its save through his card |
| his controllers (`--script`, from `to_host_script.py retail_full.txt`), no `MELEE_PAD_SCRIPT` | same scenes at the same ticks; hashes 2174/2174 same |

His sandboxed loader (`map_game_image`, inside an AppContainer) would refuse our image: 39.8 MB against his 24 MB
range (`pe_check.py`). It also sets up no implicit TLS, and our image has 360 bytes of it.

## Gaps

- Pokemon Stadium with savestates on: engine tick 12.6 ms (2.9 ms with v1 or headless). It still runs at 59.8 fps, but
  with little margin. The cost is in the rollback-mode engine path, not in the transcoder. A detach copy that cost
  more is already removed.
- His rollback calls none of `state_*`: his Slippi rollback drives the game's own EXI savestate commands. Only his
  test mode uses them.
- No settings overlay, UI or wasm presentation mods: they draw with our renderer.
- Geno palette conversion is checked by a unit test only.
- Depth (Z) and other non-colour EFB copies are encoded but unverified. Sized and indexed draws did not occur.
- Items, 1P modes and the results screen were not compared shot for shot.
- The sandbox limits above.

## Image size on Linux (parked: no longer needed)

Parked 2026-11-08: the project goes independent of Hero's host, so his Linux loader's 64 MiB check (a game ELF must end below
0x84000000, 24 MB from the 0x82800000 base) no longer applies. Numbers measured before stopping, release profile, Linux:

| artifact | loadable (sum of PT_LOAD memsz) | .text | .bss | .eh_frame | .gcc_except_table | .rodata | .data.rel.ro |
|---|---|---|---|---|---|---|---|
| `libmelee_game.so` (`-p melee_corev1`, rust-port db9d048e4) | 42.4 MB | 21.2 MB | 7.6 MB | 2.7 MB | 1.3 MB | 2.4 MB | 2.4 MB |
| `melee` (the exe, for comparison) | 41.1 MB | 23.4 MB | 9.1 MB | 2.9 MB | 1.3 MB | 2.6 MB | 2.7 MB |

No lever was built. One finding for anyone who picks this up: `panic=abort` (and with it dropping the unwind tables,
.eh_frame + .gcc_except_table, about 4 MB) is not behaviour-neutral as the code stands: two game paths use unwinding as C's
setjmp/longjmp (`melee_world/src/tu/granime.rs` resume_unwind(LongJmp) caught in the same TU, and
`melee_runtime/src/tu/hsd_3B34.rs` jpeg_fail's JPEG_EXHAUSTED).

