# Rust vs C benchmarks (2026-11-01)

One session on a quiet machine, headless, no OS input. Numbers are medians of 3 runs unless a row says otherwise.
Hash CSVs and PNGs were written to scratch only; nothing disc-derived is committed.

## Machine and toolchains
- Intel Core Ultra 7 265F (20 cores / 20 threads), 32 GB RAM, NVIDIA GeForce RTX 5070, Windows 11 Pro 10.0.26200.
  Sources, build roots and target dirs all on E:.
- Rust: rustc/cargo 1.96.0 (2026-05-25), `cargo build -p melee --release` (profile: overflow-checks off, `debug = 1`),
  `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`, private `CARGO_TARGET_DIR`.
  Tree: rust-port at 838a01542 (the binary was built there; later commits were not benchmarked).
- C: `tools/port/build.sh` with `GW_MELEE` = this worktree and `GW_BUILD_ROOT` = a private root `_build/agents/bench`
  (993 game TUs through gwtool, 86 shims, 2 link passes for the bridge fixpoint; 12 parallel jobs, the script default).
- Quiet check: no cargo, rustc, cl, clang or melee processes from other agents at the start. One other agent's
  `melee play` ran for a few minutes around 09:22 (between the Rust render runs and the C build); the medians are
  unaffected, but a reader re-running the render rows should keep the machine idle.

## Method
- Clean build, Rust: delete `release/` of the private target dir (all dependencies rebuilt), time `cargo build -p melee --release`.
- Clean build, C: delete the build root's `masstest/`, generated files, exe/map/link list (Aurora, the 3rd-party GPU stack, left
  in place so the number is the game itself); time `build.sh`. The very first build of a fresh root also compiles Aurora
  and Dawn: ONE sample, 1050 s (17.5 min) in total.
- No-op: the same command again with nothing changed. Rust is cargo's fingerprint check; C is `build.sh`
  (TU/shim scan, Aurora check, provenance, exe verification). The first no-op after a fresh build took 21 s, later ones 11 s.
- One-file change: Rust `touch rust/melee_items/src/hash.rs` (mtime only; cargo rebuilds melee_items and everything
  downstream: modes, the bin, ...). C: a content change is needed because TU staleness is by content hash (a plain
  `touch` rebuilds nothing, 11 s); a comment line was appended to `src/melee/it/item.c` (2433 lines) per run and
  the file restored afterwards (1 TU + 1 shim + 1 link).
- Sim speed, Rust: `melee golden --engine tu --iso $GW_ISO_VANILLA --scenario S [--idle-pads]` with the golden's length
  (1801 / 1801 / 3601 / 3601 ticks), no window, no render. The boot cost is measured per run with `--frames 2` and
  subtracted ("sim only"); "end-to-end" is rows / total wall time.
- Sim speed, C: `tools/rust-port/golden/record.sh`'s recipe (`run.sh`, `MELEE_TURBO=1`, `MELEE_PAD_SCRIPT`, `MELEE_SCENE`,
  `MELEE_NEUTRAL_HASHLOG`, window parked at x=30000, unfocused, self-exits). The C run has no `--frames 2` equivalent, so
  only end-to-end (rows / wall incl. `run.sh` sandbox copy, process start, disc load, menus to the match) is comparable.
  The game's own perf summary gives the steady "frame work mean" per sim frame.
- Rollback: `melee synctest --scenario pad_falcon_fox_fd --depth K --frames 600`, K = 1 and 7, boot (2 frames)
  subtracted. Each tick = run frame + save + 3 hashes + load + K resimulated frames + hashes; the K=7 minus K=1 difference
  over 6 frames is the marginal cost of one resimulated frame.
- Render: `melee golden --engine tu --frames 600 --shots ...` (offscreen melee_gx on wgpu, no window) against the same run without
  `--shots`: 0 shots (render off), 1 shot (render on) and 11 shots (render on).

## Results

### Build times (seconds)
| measurement | Rust (release) | C (build.sh) |
|---|---|---|
| clean build | 184.3 (185.6 / 183.9 / 184.3) | 65.5 (67.6 / 65.5 / 65.4), Aurora cached |
| clean build with its GPU stack | same (wgpu is part of the 184 s) | 1050 (one sample, Aurora + Dawn + game) |
| no-op rebuild | 0.28 | 11.4 (21.4 / 11.4 / 11.1) |
| one-file rebuild | 150.9 (melee_items; rebuilds downstream) | 13.3 (1 TU, 1 shim, relink) |

The Rust one-file figure depends on the crate: it is the cost of a late crate in the chain (melee_items -> melee_modes -> melee ->
...), an early crate (melee_runtime) costs about the clean build. The `touch` case without content change is free for C.

### Headless sim (4 goldens, 60 Hz game frames per second of wall time)
| scenario | ticks | Rust sim only | Rust end-to-end | C end-to-end | C steady frame work |
|---|---|---|---|---|---|
| pad_falcon_fox_fd | 1801 | 143.5 fps (6.97 ms/tick) | 103.8 fps (17.35 s) | 290 fps (6.21 s) | 1.67 ms |
| pad_marth_sheik_bf | 1801 | 178.3 fps (5.61 ms/tick) | 137.7 fps (13.08 s) | 276 fps (6.52 s) | 1.75 ms |
| cpu_fox_falco_dl | 3601 | 95.3 fps (10.49 ms/tick) | 83.5 fps (43.11 s) | 328 fps (10.99 s) | 2.10 ms |
| cpu_peach_samus_ys_items | 3601 | 158.3 fps (6.32 ms/tick) | 138.7 fps (25.96 s) | 326 fps (11.05 s) | 2.17 ms |

The C turbo run also executes the title/menu frames before the match (about 30-240 extra sim frames) and the startup, so the
end-to-end C rate is a lower bound of its sim rate; the game's own steady per-frame work (1.7-2.2 ms) is the sim-only
figure, at roughly 450-600 fps. The Rust engine is at about 3-4x the C cost per tick end-to-end, on the same hash-equal output.
Every Rust run is hash-equal to its golden (see STATUS.md); runs here were not re-compared.

### Rollback (Rust, pad_falcon_fox_fd, 600 ticks)
| measurement | value |
|---|---|
| synctest depth 1, per tick | 23.4 ms (14.04 s over 600 ticks) |
| synctest depth 7, per tick | 53.0 ms (31.78 s over 600 ticks) |
| marginal cost of one resimulated frame | 4.9 ms |
| **7-frame resim** | **34.5 ms** (7 x 4.9) |
| per-tick overhead outside the resim frames (save, load, hashes, the live frame) | about 18.5 ms |
| plain sim tick for comparison | 7.0 ms |

A 7-frame rollback costs about two 60 Hz frame budgets (16.7 ms) on this machine, before the live tick, so a deep rollback in a real match is
over budget on a single core; resim ticks are cheaper than live ticks (4.9 vs 7.0 ms) because they skip the audio/presentation side. All runs: 0 mismatches.

### Render (Rust, offscreen, pad_falcon_fox_fd, 600 ticks)
| measurement | value |
|---|---|
| wall, render off | 8.39 s |
| wall, render on, 1 shot (tick 599) | 9.58 s |
| wall, render on, 11 shots | 11.79 s |
| cost per screenshot (draw + GPU + readback + PNG) | 221 ms |
| render-on cost per tick without a shot (display-list recording, draw callbacks; renderer init amortised) | about 1.6 ms |

Frame GPU time is not separated from readback and PNG encode here; the engine has no windowed present in this mode. The C build has
no headless render path (it needs the window), so there is no C figure.

## Notes
- No-op for Rust is a fingerprint scan only (0.28 s); C pays 11 s each time for the stale scan, bridge check and provenance.
- A fresh C root costs ~17 min once, almost all of it Aurora/Dawn; after that a clean game rebuild is about 1 min.
- Cleanup after the run: the private C root `_build/agents/bench` was deleted and the Rust target dir used here removed.
