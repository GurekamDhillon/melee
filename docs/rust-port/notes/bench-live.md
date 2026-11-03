# Live play: Rust port vs melee-unlocked Source Port, 2026-11-02

This is a benchmark only; none of melee-unlocked's code is copied here. Both games were driven entirely by their own in-engine pad scripts: no OS input, and neither window was ever focused. Runs went one at a time. Nothing disc-derived is committed. The harness scripts are in `bench-live/`.

## Builds
| | Rust port | melee-unlocked Source Port |
|---|---|---|
| version | 306c457f7 (the port/render merge), worktree `bench2`, branch `port/bench-live` | v0.9.86 271402e, scratch clone `scratch/ext/melee-unlocked` |
| build | `cargo build --release -p melee --bin melee` (rustc release, `debug = 1`, no fat LTO) | `melee_game.dll`: WinLibs GCC 15.3.0 UCRT (portable zip; his tools name `winlibs-gcc-15.3`), `-O2 -g`, AVX2 baseline, per `docs/build-source-port.md`. `melee_source.exe`: MSVC Release (VS 18 BuildTools), D3D12 |
| renderer | wgpu (D3D12), our GX port | his D3D12 GX FIFO renderer, `--threaded-renderer` |
| machine | Core Ultra 7 265F, 32 GB, RTX 5070, Windows 11 26200, 144 Hz display | same |

Toolchain note: GCC's linker cannot handle the space in `E:\Projects\Melee Workspace`, and resolving through a junction doesn't avoid it. So the toolchain lives at `E:\Projects\mu_gcc15` (portable, on E:, nothing system-wide). It was put on PATH only for the build shell.

Code change on our side: `MELEE_FRAME_TIMES=<csv>` in `melee_platform/src/main_loop.rs`. It is off by default and writes one row per tick and one per present, used here for the percentiles.

## Scenario (both builds)
- Hyrule Temple, 4 CPUs at level 9: Fox, Marth, Falco, Peach. Second scenario: Final Destination, Fox vs Marth, both CPU level 9.
- Items off, stock 4, window 1280x960, internal resolution 2x (1280x960), vsync off, uncapped, sub-frame interpolation on, 3600 match frames, 3 runs each, medians reported.
- **Rust**:
  - command: `melee play --match fox/cpu9,marth/cpu9,falco/cpu9,peach/cpu9,14 --items OFF --stocks 4 --time-secs 480 --max-frames 3603` (stage 32 for FD; the match starts at tick 3)
  - env: `MELEE_PAD_SCRIPT=<idle script> MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_VSYNC=off MELEE_INTERP=1 MELEE_RENDER_SCALE=2 MELEE_FRAME_TIMES=...`
  - The window is visible, never active, at the bottom of the z-order. With `MELEE_WINDOW_UNFOCUSED=hidden`, winit never delivers RedrawRequested, so 0 frames would be drawn.
- **Source Port**:
  - command: `melee_source.exe --hidden --threaded-renderer --fps unlocked --frame-mode interpolate --time-base 1 --window 1280x960 --scale 2 --dlss off --ssaa 1 --sharpness 0 --volume 0 --no-music --script s_hyrule.txt --frames 6241 --frame-times ... --settings-path <file: "vsync 0", "fps 0"> --load-settings` (FD: `s_fd.txt`, `--frames 6228`)
  - env: `MELEE_PRESENT_UNCAPPED=1` (otherwise "uncapped" is held at 2x refresh) and `MELEE_SIM_TIMES=...`
  - Its window code calls `ShowWindow(SW_SHOW)`, which takes focus, so it must run `--hidden` (the Static Recomp host is the same).
  - How the match was started: `--match` only forces the outcome after the menus have run, so the menus were driven by its own `--script` pad scripting. The script dismisses the card prompts, backs out of Online, goes to VS, then Custom Rules, sets Item Switch to None, and enters the CSS. Four scripted ports each pick a character, switch to CPU and drag the level slider to 9. The SSS cursor then picks the stage. `gen_css.py` generates the CSS part.
  - Every step was checked with its own `--capture` screenshots, and all four slots read level 9. The same script drives the Static Recomp `melee_port.exe` identically.
- Measurement:
  - frame times: from each build's own per-present log (`MELEE_FRAME_TIMES` / `--frame-times`), match frames only.
  - CPU, working set and private bytes: psutil every 100 ms.
  - GPU: nvidia-smi `utilization.gpu` every 100 ms. PresentMon is not installed, so there is no real GPU time.

## Results (medians of 3)
| | Rust Temple 4P | Source Port Temple 4P | Rust FD 2P | Source Port FD 2P |
|---|---|---|---|---|
| render fps, average | 334 | 425 (hidden window, see caveats) | 952 | 1103 (hidden window) |
| render fps, 1% low | 54 | 152 | 127 | 143 |
| frame time p50 / p95 / p99 (ms) | 1.10 / 9.67 / 14.8 | 2.00 / 4.45 / 5.40 | 0.64 / 4.47 / 6.04 | 0.72 / 1.85 / 2.92 |
| sim per tick | 7.5 ms p50, 7.6 mean, 11.4 p99 (wall of `Game::tick`) | 4.37 ms p50, 4.45 mean (`cpu_ms`, game-thread CPU) | 4.06 ms p50, 4.3 mean, 6.9 p99 | 2.24 ms p50, 2.27 mean |
| process CPU (cores busy, average) | 0.94 | 1.29 | 0.93 | 1.04 |
| process CPU per rendered frame | 2.8 ms | 3.0 ms | 0.99 ms | 0.95 ms |
| GPU utilisation (nvidia-smi, proxy) | 25% | 34% | 29% | 40% |
| working set, peak / last sample | 3310 / 882 MB | 962 / 317 MB | 4564 / 683 MB | 880 / 250 MB |
| private bytes, peak / last sample | 3560 / 3162 MB | 1150 / 403 MB | 4999 / 4604 MB | 1083 / 367 MB |
| launch to first match frame | 3.5 s | 45.1 s (about 43 s of it is the scripted menu walk) | 5.0 s | 45.7 s |
| presents in 3600 match frames | 20015 | 25506 | 57026 | 66180 |

Per-run spread:
- Rust Temple run 1 was an outlier: 85 fps, sim p50 9.3 ms, p99 frame 90 ms, the first run after the build (cold caches); runs 2 and 3 gave 334 / 415 fps.
- Source Port Temple: 407 / 493 / 425 fps. FD: Rust 849 / 1057 / 952, Source Port 1045 / 1103 / 1104.
- The "last sample" memory is the last 100 ms sample before exit, which may already be during teardown. The working-set drop to under 1 GB there is that, not a steady state.

Reading:
- On average fps we are 0.79x of the Source Port on Temple and 0.86x on FD.
- On tail latency the gap is larger: our p99 frame is 2.7x (Temple) and 2.1x (FD) theirs, and our 1% low on Temple is about a third of theirs.
- The sim tick is about 1.7x (Temple) to 1.8x (FD) their game-thread CPU per frame. Temple at 7.5 ms/tick is a known regression another agent is fixing. Headless sim was 0.3 to 0.5 ms/tick in bench-unlocked.md, so most of the live tick is not the simulation itself.
- Memory: our peak private bytes are 3.1x (Temple) and 4.6x (FD) theirs.
- Startup: ours is faster, but not comparably: `--match` skips the menus, while theirs walks them by script.

## Caveats (why this is not like for like)
- **Window**: theirs is a hidden window, ours a visible unfocused one, because their visible window takes focus and our hidden window never redraws. Their rendering is verified: about 1027 GX draws (Temple) / 471 (FD) are recorded and submitted on every present, with 1.0 to 1.3 ms submit per present and GPU utilisation up during the match. Their mid-run captures in `--hidden` mode show correct frames. Whether a hidden window's Present actually reaches DWM composition is **unverified**, and it may make their Present cheaper than ours.
- **Work per frame differs**: theirs runs the Slippi layer (Slippi menus, UCF 0.84 shown on the CSS), a time-plus-stock match with the 8-minute timer on, its sub-frame pose interpolation (`interpolate`) and its own audio mixer at volume 0. Ours: no Slippi codes, a stock match (no visible timer), our interpolation (`MELEE_INTERP=1`), and audio to the device with many underruns logged in live play.
- **Sim figures are different quantities**: ours is the wall time of the whole `Game::tick` (sim plus audio plus draw-list building, inside a winit loop). Theirs is `cpu_ms`, the game thread's own CPU per frame. Their `sim_ms` (wall) reads a constant 13.4 ms, which is pacing, not cost.
- **Renderers differ**: wgpu and our GX port, against his D3D12 FIFO decoder. Draw counts were not compared one to one.
- **Builds**: our plain release profile (`--profile perf`, fat LTO, is about 25% faster per perf.md) against GCC `-O2` (his default) plus MSVC Release.
- **Hyrule Temple stage numbers**: same stage, same characters, but CPU behaviour diverges from frame 1 (different RNG and engines), so the on-screen load (KOs, effects, camera) differs between runs and builds.
- **GPU**: utilisation % is a coarse proxy, not GPU time.
- Background load on the PC was not controlled. Sim pacing also fell behind on our side in the outlier run: 69 s of wall time for 60 s of game.

## Side note: Static Recomp (`melee_port.exe`)
This was timed first, then stopped in favour of the Source Port (the user's call), so there is no full 3-run set. The same script and flags reached the same Temple 4-CPU match. In short probe runs its log reported 700 to 2200 fps uncapped (1400 to 1500 typical in the match). Its game-thread `cpu_ms` was about 7.8 ms per match frame on Temple, about 1.8x the Source Port's 4.4 ms.

## Re-run 2026-11-03: after memfix, GX parity rounds 1-2, sound parity and demofix
Rust is rust-port f0c3a13aa merged into this branch (release profile, built in the `bench2` worktree only). The Source Port is the same v0.9.86 build as above, re-run now. Same scenarios and method; 3 runs each, one game at a time, medians. Rust runs used `MELEE_VOLUME=3`, `MELEE_WINDOW_TITLE="bench: rust-release <scenario>"`, `MELEE_PAD_IGNORE_ADAPTER=1`, `MELEE_WINDOW_UNFOCUSED=1`; the Source Port ran `--volume 0 --no-music --hidden`. Draw counts come from a separate extra run per scenario with `MELEE_GX_STATS` (so the stats cost is not in the timings). Scripts: `bench-live/all4.sh` (loop), `run_r.sh`, `analyze.py`.

Background load: another agent's cargo/rustc was running before most runs (CPU 17 to 24% machine-wide before each run, see `all4.log`); no other game was running during timed runs. Pre-parity runs of this session (before the invisible-fighter fix, and an aborted rerun) are discarded.

### Results (medians of 3)
| | Rust Temple 4P | Source Port Temple 4P | Rust FD 2P | Source Port FD 2P |
|---|---|---|---|---|
| render fps, average | 1295 (was 334) | 555 (was 425) | 2451 (was 952) | 1326 (was 1103) |
| render fps, 1% low | 125 (was 54) | 184 (was 152) | 238 (was 127) | 398 (was 143) |
| frame time p50 / p95 / p99 (ms) | 0.17 / 4.88 / 6.89 (was 1.10 / 9.67 / 14.8) | 1.51 / 3.75 / 4.82 (was 2.00 / 4.45 / 5.40) | 0.19 / 1.52 / 3.61 (was 0.64 / 4.47 / 6.04) | 0.66 / 1.59 / 2.10 (was 0.72 / 1.85 / 2.92) |
| sim per tick | 5.38 ms p50, 5.59 mean, 7.55 p99 (wall of `Game::tick`; was 7.5) | 4.03 ms p50, 4.17 mean (game-thread `cpu_ms`; was 4.37) | 2.96 ms p50, 3.06 mean, 4.22 p99 (was 4.06) | 2.16 ms p50, 2.20 mean (was 2.24) |
| process CPU, cores busy | 0.95 | 1.30 | 0.91 | 1.10 |
| process CPU per rendered frame | 0.74 ms | 2.36 ms | 0.37 ms | 0.84 ms |
| GPU utilisation (nvidia-smi proxy) | 14% | 25% | 19% | 34% |
| working set, peak / last sample | 504 / 418 MB (was 3310 / 882) | 929 / 415 MB | 446 / 368 MB (was 4564 / 683) | 865 / 299 MB |
| private bytes, peak / last sample | 657 / 552 MB (was 3560 / 3162) | 1109 / 541 MB (was 1150 peak) | 585 / 491 MB (was 4999 / 4604) | 1053 / 389 MB |
| presents in 3600 match frames | 77605 | 33276 | 146863 | 79535 |
| draws per presented frame | 2519 raw, 587 after merging state-identical runs | 1027 (863 paired) | 1280 raw, 292 merged | 471 (372 paired) |
| distinct textures / pipeline configs per frame | 205 / 58 | not logged | 107 / 47 | not logged |
| copies (EFB) per frame | 4 | not logged | 2 | not logged |

Per-run fps: Rust Temple 1335 / 1295 / 1173, FD 2363 / 2879 / 2451; Source Port Temple 639 / 555 / 548, FD 1341 / 1326 / 1218. The Source Port is faster than in the first set (425 / 1103 before): same binary, so this is machine state (the earlier set ran with other load).

### Reading
- We are now ahead on average fps (2.3x Temple, 1.85x FD) and on CPU per frame, and our memory is about half theirs. Our 1% low is still lower than theirs (125 vs 184 on Temple, 238 vs 398 on FD) and our p99 frame is 1.4x (Temple) and 1.7x (FD) theirs.
- Sim tick: ours is 5.4 ms (Temple) / 3.0 ms (FD) wall against their 4.0 / 2.2 ms game-thread CPU, i.e. 1.3x; still different quantities (see caveats).
- **Our present rate is not like for like.** Our presents are far more numerous (77605 vs 33276) with a 0.17 ms median: only about one frame per sim tick is actually prepared and drawn (3575 prepare rows in 3600 ticks; `present prepare` p50 1.9 ms, `draw` p50 1.06 ms in the perf line), the rest re-present with interpolation alpha. So the fps figure above is largely cheap re-presents; the cost per prepared frame is the better number to compare, and the Source Port's `interval_ms` rows are also sub-frame presents.
- **Draws per frame**: ours counts every GX draw (2519 / 1280), his `draws` column counts after his per-display-list primitive merge (1027 / 471). Our comparable number is the run-merged count `batches`: 587 / 292, which is 0.57x and 0.62x of his. We therefore still issue fewer draws than he does, consistent with missing content.

### Caveat: our GX parity is not complete
Still missing on our side at this point: Stadium lamps, the Corneria shadow, depth copies, and the shield / magnifier effects (a gxparity3 agent is working on them). Fewer draws and less GPU work mean our numbers are flattering until parity lands. Everything in the Caveats section above also still applies (hidden window on his side, Slippi/UCF layer, different sim quantities, different renderers, GCC -O2).
