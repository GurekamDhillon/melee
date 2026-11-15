# Profiling the Rust port: Tracy, the contention flag, perf-ab, WPR/ETW

The zero-lag rule: no game-caused frame over 16 ms, in any scene. Three tools explain a frame that breaks it. Use
them in this order:

1. **The frame profiler** (`MELEE_PROFILE=1`, always built in). One CSV row per presented frame with the timed
   parts, plus a contention flag (below). It answers "which part was long?" and "was the machine busy?".
2. **Tracy** (`--features profile-tracy`). A timeline of named zones on every thread, with context switches and
   samples. It answers "what exactly ran inside that part?".
3. **WPR/ETW**. The OS view: driver, DWM, DXGI present queue and GPU work. It covers the time Tracy cannot see
   (inside the driver and in other processes).

`tools/rust-port/perf-ab.sh` then tells whether a change made frames faster or slower, and whether the difference
is larger than the run-to-run noise.

## 1. The frame profiler and the contention flag

`MELEE_PROFILE=1` writes `profiles/profile-*.{txt,csv,jsonl}` beside the run (melee_core/src/frameprof.rs). The CSV
has one row per present. Since the profiler branch it also has these columns:

| column | meaning |
|---|---|
| `thread_cpu_ms` | CPU time the game (main) thread got during the frame (QueryThreadCycleTime, exact to the cycle; Linux: CLOCK_THREAD_CPUTIME_ID) |
| `proc_cpu_ms` | CPU time of the whole process; only with `MELEE_PROFILE_PROC_CPU=1` (QueryProcessCycleTime walks every thread, tens of µs) |
| `sys_cpu_pct` | the whole machine's CPU load over the frame, at least the last 100 ms (GetSystemTimes; Linux: the 1 s machine sample) |
| `slow` | 1 when the frame's game work (interval minus pacing sleep and display wait) or its longest tick is over 16.7 ms |
| `contended` | 1 when a slow frame was **starved**: the machine was at least 80% busy and the game thread was on a CPU for under 70% of the work time |

A contended frame is one where the game was ready to run but other processes held the cores, such as a cargo build
or another agent's game. A **clean** slow frame is the game's own cost. It is also clean when the thread was off the
CPU on an idle machine, which means it was waiting on the disc, the driver or a lock. That wait is still the game's
problem.

The report's FRAME TIMES block ends with `frames over 16.7 ms of game work: N clean, M contended`, and the jsonl
summary carries `slow_clean` and `slow_contended`. `tools/rust-port/perf/zero_hitch_check.py` lists contended offenders
under `CONTENDED (...)` and fails only on clean ones. Use `--strict-contended` to fail on both. Every perf row that
calls it gets the split: the geno VM check's perf row, quickswap, the LAB and pause checks, and zero_hitch_gate.sh.
A run with contended frames is not a verdict on those frames: re-run on a quiet machine.

Recording only reads clocks. Game memory, hashes and NHV1 rows do not change.

## 2. Tracy

### Build

```
cd rust
CARGO_TARGET_DIR=<your target> cargo build --release -p melee --features profile-tracy
```

Without the feature, `melee_core::tracy` functions are empty `#[inline(always)]` bodies, `Zone` is zero-sized, and
the `tracy-client` crate is not built. The default build is unchanged, and perf-ab measures that (see section 5).

The client is `tracy-client` 0.19 / `tracy-client-sys` 0.30.1, which is **Tracy protocol 0.14.1**. The GUI must be
the same release: download `Tracy-0.14.1` (Windows zip) from github.com/wolfpld/tracy/releases. The client listens
on localhost only (`only-localhost`). It has no crash handler, because the game has its own.

### Run

1. Start `tracy-profiler.exe` (the GUI) and leave it on the connect screen. For a headless capture, use
   `tracy-capture.exe -o run.tracy` instead; it waits for the client and writes a file the GUI opens later.
2. Run the game as usual (hidden window, volume 3, pad script). Optionally add `MELEE_PROFILE=1`; both work together.
   `TRACY_NO_EXIT=1` keeps the process alive at exit until the GUI or `tracy-capture` has all the data. Use it for
   short runs, and never in unattended scripts without a capture attached.
3. Click the client in the GUI, or let `tracy-capture` connect. Tracy buffers from process start, so a capture
   that connects late still has boot and the first scenes.
4. For context switches and CPU samples (the "starved" picture in Tracy itself), run the game from an
   **elevated** shell. Without admin rights Tracy still records the zones but not the system tracing.

### Zones

| zone | where |
|---|---|
| `tick` (+ the "tick" frame set) | main_loop: one 60 Hz tick (`Game::tick`) |
| `sim (engine step)` | retail_flow: `RetailPlay::step`, mods' sim hooks included |
| `capture / decode` | retail_flow: frame take / detach / EFB copies handed to the presenter |
| `ui (gd_ui frame)` | retail_flow: UI mods' frame handed to the renderer |
| `present phase (...)` (+ frame mark, `present ms` plot) | main_loop: `Game::present` |
| `prepare`, `draw` | gfx.rs: fresh-frame prepare, draw + interpolation |
| `acquire (get_current_texture)` | gfx.rs: waiting for a swapchain image (display / GPU queue) |
| `present (blit + overlay + submit + present)` | gfx.rs: the rest of `swap_present`, including `SurfaceTexture::present` |
| `pipeline create` | render.rs: a new GPU pipeline (shader) |
| `texture decode + upload` | render.rs: value = bytes, text = `WxH mips N KB` |
| `buffer upload (vertex/attr/index)`, `uniform upload` | render.rs: value = bytes |
| `disc read`, `archive parse` | dvd.rs, archive.rs: value = bytes |
| `note: ...`, `fighter: ...`, `stage: ...` | every `frameprof::Span` (asset decode / apply, scene preload / on_enter / on_exit, match setup steps, stage switch, fighter create, wave spawns) |
| `mod hook: <id>`, `ui mod: <id>` | melee_modhost: each wasm hook call |
| `boss host step (petey / Geno VM)` | melee_modhost: `petey::tick` (the VM boss frame runs inside it) |
| `rollback (load + resimulate)` | melee_net `RollbackSession::resolve_rollback`, value = frames resimulated |
| `netplay snapshot load (rollback)`, `netplay frame`, `netplay resim frame` | play_net.rs |

GPU zones are not instrumented. wgpu needs `Features::TIMESTAMP_QUERY` (and `TIMESTAMP_QUERY_INSIDE_ENCODERS`) at
device creation, plus a resolve and readback per frame. Use the WPR GPU capture (section 3) for GPU time until that
is worth adding.

### Adding a zone

```rust
let _z = melee_core::tracy::zone("my phase");            // static name, caller's file/line
_z.value(bytes as u64);                                  // optional number
let _z = melee_core::tracy::zone_dyn(|| format!("x {n}")); // run-time name; the closure runs only with the feature
```

Zones close when dropped, and they must nest on one thread: `drop(z)` explicitly before opening a sibling. Never
read or write game state just to name a zone.

## 3. WPR/ETW: driver, DWM and present time

Tracy stops at `SurfaceTexture::present()`. Time inside the D3D12/Vulkan driver, the DXGI present queue, DWM
composition and GPU execution needs ETW. Windows Performance Recorder ships with Windows (`wpr.exe`). Windows
Performance Analyzer (WPA) comes from the Windows ADK or the Microsoft Store. Both need an **elevated** shell.

```
wpr -cancel                                     (clears a stale session; an error here is fine)
wpr -start CPU -start GPU -start DesktopComposition -filemode
  ... run the scenario: the hidden-window run, MELEE_PROFILE=1 for the frame numbers ...
wpr -stop present.etl "melee SSE match start"
wpa present.etl
```

- `CPU`: sampled stacks and context switches (CPU Usage (Sampled) / (Precise): the game thread's Ready time is its
  starvation, exactly).
- `GPU`: DxgKrnl, the GPU queues and the driver's packets ("GPU Utilization", "GPU Hardware Queue").
- `DesktopComposition`: DWM and the DXGI present events (`Microsoft-Windows-DXGI` Present start/stop,
  `DxgKrnl` Present/Flip/VSync). In WPA, the **Window In Focus / Present** analysis or a generic events table
  filtered to `Microsoft-Windows-DXGI` shows each present's call time and when the frame reached the screen.

Keep captures short (30-60 s) because the files grow by hundreds of MB. Line up WPA with the frame profiler by the
process start time plus the CSV's `t_ms`, or set a marker: the profile's `.txt` names the hitch times.

`PresentMon` (github.com/GameTechDev/PresentMon) is a lighter alternative for the present path alone:
`PresentMon64.exe -process_name melee.exe -output_file pm.csv`. Its `MsInPresentAPI`, `MsUntilRenderComplete` and
`MsUntilDisplayed` columns separate the CPU time in the present call from GPU and display latency.

## 4. Reading the results

Start from the frame profiler's offender line, for example
`frame 22.4 tick 3.1 | present 4.6, draw 3.0, unaccounted 9.8`.

- Read the `contended` column first. A contended frame says nothing about the game: re-run on a quiet machine.
- **tick / sim (engine step) long**: open Tracy at that frame. The nested `note:` / `fighter:` / `disc read` /
  `archive parse` zones name the load or spawn. A long `mod hook:` zone names the mod.
- **prepare long**: look for `pipeline create` and `texture decode + upload` zones under it. The first sight of new
  content makes pipelines and textures; the fix is warming (warmlist) or moving the upload off the frame.
- **acquire long**: the display or GPU queue was full. The GPU is behind, which is a GPU-side cost, so use WPR GPU.
- **present long, or "unaccounted" long with present after it**: time in the driver or the DXGI queue. Tracy shows
  only one long `present (...)` zone; WPR `DesktopComposition` + `GPU` shows whether the driver blocked (a flip
  queue wait, a compile in the driver: the D3D12 driver's PSO compile appears as driver CPU samples under
  `Present`) or DWM was late.
- **unaccounted, nothing else**: the event loop or the OS. Tracy's context-switch lane (admin) shows the game thread
  descheduled, and WPA's CPU (Precise) Ready time gives the same figure exactly.

perf-ab output (section 5): a `*` difference means every pair moved the same way, so the difference is outside the
run-to-run noise. Compare a difference against the "base spread" column, the base's own run-to-run range. A p99
difference smaller than the spread is noise however it looks.

## 5. perf-ab

```
bash tools/rust-port/perf-ab.sh <base-rev> <head-rev> <fd|sse|boss> [N]
```

- Builds both revisions with `cargo build --release -p melee` in private worktrees and target dirs
  (`_build/perf-ab/wt-<sha>/build/cargo-ab`; `PERF_AB_SHARED=1`: the lanes' shared `_build/cargo-shared`, incremental, the exe copied out right after each build). It goes through the buildlock (lane `perf-ab`, `-j 3`, waits for 8 GB
  free) and stops below 15 GB free on C: or 20 GB on E:.
- Checks each exe's banner id (`melee [<where> <hash>]`) against its sha.
- Runs one warm-up per side (not counted), then N runs (default 5) alternating base/head (ABBA order) with
  `MELEE_PROFILE=1`. Windows are hidden, volume is 3, the title is `perf-ab <side> <scenario> <label> <i>`, and the
  pad scripts come from the head revision's tree. It waits while the owner-play lock is held.
- Prints `perf/perf_ab.py`'s table and writes it to `<out>/summary.txt`. Worktrees and target dirs are deleted at
  the end unless `KEEP=1`; the exes and CSVs stay in `_build/perf-ab/out-<base>-<head>-<scenario>/`.
- `FEATURES=profile-tracy` builds both sides with Tracy (the overhead of Tracy on).

Scenarios:

| scenario | what | revisions |
|---|---|---|
| `fd` | real-menu walk to a 2-CPU FD match (`walk_vs_fd_2cpu.txt`), 2400 frames | any |
| `sse` | scratch/sse4/quiet.sh's walks: Lake with its doors, Jungle, then FD | with the SSE stages (port/sse-stage2) |
| `boss` | the geno VM check's perf row: Falcon vs the CPU Fox carrier on FD, `MELEE_VS_BOSS=${BOSS:-rayquaza}`, jab schedule, 3000 frames | with MELEE_VS_BOSS (the geno VM branches) |

### Measured: the overhead with Tracy off

`perf-ab.sh 020ccdd4b 20c949a6f fd` (the merge base vs the profiler branch with the Tracy zones and the
contention flag, default build, so Tracy is off; 5 alternating pairs, 2392 frames each, MELEE_PROFILE=1, desktop,
Nov 15):

| phase (ms) | base p50 | head p50 | base p99 | head p99 | p50 diff [range] | p99 diff [range] |
|---|---|---|---|---|---|---|
| frame interval | 16.665 | 16.664 | 18.264 | 18.355 | -0.004 [-0.012, +0.010] | +0.077 [-0.330, +0.173] |
| game work | 1.886 | 1.817 | 4.068 | 4.098 | -0.068 [-0.166, +0.157] | +0.037 [-0.274, +0.214] |
| tick | 0.726 | 0.663 | 2.168 | 2.101 | -0.059 [-0.187, +0.052] | -0.014 [-0.359, +0.339] |
| prepare | 0.256 | 0.246 | 1.574 | 1.513 | -0.001 [-0.039, +0.063] | -0.022 [-0.288, +0.297] |
| draw | 0.478 | 0.468 | 1.153 | 1.166 | -0.008 [-0.061, +0.110] | -0.058 [-0.140, +0.061] |
| present | 0.131 | 0.126 | 0.361 | 0.353 | -0.007 [-0.014, +0.018] | +0.005 [-0.053, +0.086] |
| unaccounted | 0.025 | 0.026 | 0.401 | 0.422 | +0.001 [+0.000, +0.007] | +0.021 [-0.015, +0.094] |

Every range contains 0, so the Tracy-off build is within noise of the base. Frames over 16.7 ms: 1 clean per run on
both sides (the first match frame), 0 contended.

The first version of the contention flag was not within noise: it cost **+0.34 ms of unaccounted time per frame,
on every pair**. GetSystemTimes takes about 50 µs a call and QueryProcessCycleTime walks every thread of the
process. The game thread now reads only QueryThreadCycleTime (0.13 µs). A sampler thread refreshes the system times
every 10 ms, and process CPU is opt-in (`MELEE_PROFILE_PROC_CPU=1`). This is the kind of difference perf-ab exists
to catch.
