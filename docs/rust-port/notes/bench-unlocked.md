# Rust port vs melee-unlocked (Static Recomp playback), 2026-11-01

Benchmark only. melee-unlocked is GPL-3.0; nothing of it is copied here besides command-line flag names. No OS input,
all windows hidden or absent and self-exiting. The replay and the disc stay in scratch/untracked; nothing disc-derived is committed.

## What was compared
| | Rust port | melee-unlocked |
|---|---|---|
| version | rust-port HEAD 7154861ba (built from `git archive HEAD` in scratch, so the dirty worktree is not in it) | 0.9.86, upstream main 271402ea8 (2026-11-01 02:35 -0700), fresh clone in scratch |
| engine measured | `melee golden --slp` (native sim, no window, no render) | `melee_port_playback.exe` (Static Recomp: the retail PowerPC code translated to C++, plus Slippi Playback) |
| build | `cargo build --release -p melee --bin melee` (rustc 1.96.0, opt, `debug = 1`, no fat LTO, 3m44s) | cmake VS 2022 Release, MSVC 19.44, `/O2 /Ob2`, `/arch:AVX2`, no Streamline/DLSS (SDK not present), D3D12 |
| machine | Core Ultra 7 265F, 32 GB, RTX 5070, Windows 11 26200; about 30% unrelated background CPU load all session (no cargo/rustc/cl/melee processes) | same |

The Source Port engine of melee-unlocked (decomp compiled to `melee_game.dll`, the closest analogue of ours) was NOT built: it needs
MinGW-w64 GCC 14+, which is not installed (no system-wide installs allowed). So the C++ side is the translated-PPC engine, which is
likely slower than their Source Port.

## Method
- Build route (README manual): `tools/extract_dol.py` then `port/recomp/recomp.py`, cmake `-DMELEE_BUILD_EXPERIMENTAL_PORT=ON`, targets
  `melee_port` and `melee_port_playback`. The playback target only exists after a second recomp with `--sys-dir port/slippi_sys_playback`
  and, because a playback build has one replay code list baked in, `--extra-gct <the replay's gecko_list.bin> --extra-gct-base 0x80BE0A00`
  (the list is written by a first playback run). With the stock list the first run diverged from frame 0 (state 76 vs 24); with the
  replay's own list it passes.
- Workload: one tournament replay from `scratch/replays-ext`, `20476_06200225...slp` (a 2-human match with characters from Falcon/Fox/Marth/Sheik
  match on Final Destination, stage ext 32; one of the 20 files `melee golden --slp` accepts), 10071 frames -123..9947.
- melee-unlocked: `--replay F --hidden|--headless --fast --volume 0 --no-music`, per-frame times from its `MELEE_SIM_TIMES` csv
  (`cpu_ms` = thread cycles actually run in the frame, `sim_ms` = frame work wall time), match frames only. `--headless` has no window or
  renderer; `--hidden` runs the renderer in a hidden window. Boot and disc load are outside the per-frame figures.
- Rust: `melee golden --iso $GW_ISO_VANILLA --slp F --frames N --out x.csv` (no render). Per tick = (wall at N - wall at `--frames 2`) / (N-2),
  which subtracts the ~4.5 s boot but includes the golden harness (row hashes, CSV append). N=8700 and N=3175.
- 3 runs each, medians of the per-run means.

## Results
| measurement | Rust (render off) | melee-unlocked `--headless --fast` | melee-unlocked `--hidden --fast` (renderer on) |
|---|---|---|---|
| per-frame cost, first 3175 frames | 0.50 ms (0.57 / 0.50 / 0.39) | 3.58 ms cpu, 3.65 ms sim (cpu 3.87 / 3.58 / 3.43) | 5.5 ms cpu (about 6.9 ms sim_ms) |
| per-frame cost, long run | 0.30 ms over 8700 ticks (0.30 / 0.28 / 0.41) | 3.42 ms cpu, 3.49 ms sim over 10064 frames (cpu 3.77 / 3.42 / 3.33) | 5.47 ms cpu, 6.93 ms sim |
| frames per second | about 3300 (long) / 2000 (3175) | about 290 | about 145 (wall 72 s for 10064 frames) |
| p99 frame | not measured | 4.2 - 5.3 ms | 7.9 - 8.9 ms |
| whole replay wall | crashes at row 8792 (see below); 3175 ticks in 6 - 7 s with boot | 35 - 41 s | 72 - 74 s |
| frame-exact? | not verified for this file | yes: 20142 player-frames over 10071 frames, 0 mismatches (their replay_compare gate, final build) | same engine; not re-gated |

Rust is about 7x (3175-frame window) to 11x (long run) cheaper per frame than the translated-PPC headless playback, and about 15x+
against the rendering-on hidden mode, which is not like for like (their number includes recording the draw commands, ours skips the
GPU path entirely).

## Caveats
- Not frame-exact on our side. `melee golden --slp` on tournament replays does not match the C build (notes/replays-ext.md: frame-0
  differences, ports/costumes/rules not read from the file, UCF in every Slippi file). Our hash CSV has no x/y/state fields, so it cannot be gated against the
  replay's post-frame data. Control: this binary reproduces both committed C-recorded replays hash-for-hash (1801 and 1802 frames,
  `check_slp.sh`), so the engine is sound, but the timed match may not be the replayed match. Unequal work is therefore possible.
- The Rust engine panicked at row 8792 on this file (non-unwinding panic, abort), so the long-run figure is over 8700 ticks and
  the whole replay is not covered.
- The committed C-recorded replays (`golden/slp/*.slp`, the only inputs known exact for us) did not play in melee-unlocked: the match never
  started (last frame -123, no stored code list, unclosed raw block). So the two engines were not timed on a replay both verify.
- Different work per frame: theirs also executes the Slippi playback and recording event path and the Gecko caves; ours plays vanilla
  rules. Their `sim_ms` includes gx command decode/record even headless; `cpu_ms` is the safer figure.
- Their hidden-mode `sim_ms` was 6.9 ms in all three runs to within 0.02 ms (suspiciously constant: possibly pacing or render-thread
  hand-off inside the frame); use `cpu_ms` there.
- Build flags are not equal: ours is plain release (perf.md: `--profile perf`, fat LTO, is about 25% faster), theirs MSVC /O2 without
  LTCG. Boot noise on the Rust side is about +-0.6 s on a 1.5 - 4 s signal, hence the long N.
- Rollback: not compared (melee-unlocked exposes savestate code but no headless rollback-cost mode); our synctest figures are in perf.md.
- melee-unlocked build used a code list baked from this one replay; other replays (other Slippi versions) may diverge without a rebuild.
