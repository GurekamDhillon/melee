# Port dev quick-reference (agents: READ THIS FIRST)

**Reviewed 2026-09-27.** Workspace `docs/NEXT-SESSION.md` owns dated current state and
supersedes older operating instructions here. Commands below match workspace `fc23753`;
game API/timing behaviour matches `4c676892a`. No build or runtime check accompanied this review.

The workspace and game are separate repos. Run workspace tools from the workspace root;
`GW_ROOT` defaults to that tools checkout, `GW_MELEE` to its `melee/` (or the caller's game
checkout), and `GW_BUILD_ROOT` to `_build`. Override them for another layout. A machine's
`C:/gdm` junction is not a portable checkout path.

## Toolchain and build

Use the workspace scripts; game TUs require the PPC frontend/gwtool transform and cannot be
compiled with a raw native clang command. Native shims use Windows clang and the workspace
link environment. Full command reference: workspace `tools/port/README.md`.

```bash
bash tools/port/build.sh --tu src/melee/ft/ftdata.c --shim shim_dvd.c
bash tools/port/run.sh play --iso "C:/path/game.iso"
bash tools/port/run.sh --test tests --iso "C:/path/game.iso"
bash tools/port/run.sh --test --realtime tests-rt --iso "C:/path/game.iso"
```

`build.sh` scans stale game TUs and native shims, links, regenerates `gw_mex_bridge` from the
map, recompiles/relinks when required, and checks until stable (four regeneration checks max).
The final EXE gets a bridge ABI audit. An unchanged trusted bridge avoids the extra link.
`GW_JOBS` sets compile jobs; objects rebuild by content hash (workspace tools/port/README.md).

`run.sh` copies the EXE/map into `GW_BUILD_ROOT/runs/<name>/`. Each name gets its own logs,
card and runtime files; a running copy cannot block linking the baseline. Do not reuse a name
for concurrent runs. `--test` and `--realtime` go before the name. Test mode exports
`MELEE_TURBO=1` unless `--realtime` clears it; the headless test runner itself has no paced
frame loop and does not enter turbo simulation.

## Two agents at once

```
bash tools/port/agent_new.sh stages     # worktree + build root, objects hardlinked
export GW_MELEE="$(pwd -W)/worktrees/stages"
export GW_BUILD_ROOT="$(pwd -W)/_build/agents/stages"
```

Build roots separate link outputs, but `agent_new.sh` hardlinks baseline objects. Verify that
local `_build/masstest/pipe_win.sh` replaces its output rather than truncating a shared link.
Aurora/Dawn/SDL3 libraries remain shared; coordinate their rebuilds. Use separate run directories,
and keep audio/controller checks serial. Read workspace `docs/HANDOFF.md` section 6.

Clean up with `agent_rm.sh <name>`; it refuses if the worktree has uncommitted work and never
deletes the branch.

## Build inputs and run evidence

New native shims need entries in the curated object response file, not `files.txt`. New game TUs
need both the pipeline list and link list. `build.sh` is still required after either changes:
an isolated compile or raw link does not establish the bridge fixpoint.

Launch checks visibly for GD when visual/controller judgement is needed; use logs and numeric
probes for automation. Stop only a PID you started after checking its executable path, not every
`melee-pc.exe` on the machine. A failed link or an older sandbox copy can leave an old EXE running.
Record actual EXE/map, renderer, disc, mods, commands and run directory with each result.
Never redirect stdout into the game's own `melee-pc.log` (two writers).

## Resolve an rva/crash address to a symbol (needs Git Bash gawk; WSL mawk fails)
```
"/mnt/c/Program Files/Git/bin/bash.exe" -lc 'bash /c/gdm/_build/masstest/mapsym.sh 0x10355E93'
```

## Environment variables

| Variable | Effect |
|---|---|
| `MELEE_ISO=<path>` | disc image, if not passed as `--iso` |
| `MELEE_RUN_OWNER=<tag>` | launcher ownership; agents must set it and finish with `runs.py wait NAME`, then `runs.py status --owner TAG` empty |
| `MELEE_UNATTENDED=1` | wrapper defaults: 300-second overall limit and watchdog action `exit`; `run.sh --test` sets it |
| `MELEE_MAX_SECONDS=N` | wrapper overall limit; `run.sh --max-seconds N` overrides; 0 disables, interactive default 0 |
| `MELEE_WATCHDOG_SECS=N` | missing-main-tick deadline, default 10 seconds; 0 disables; debugger/system-modal dialogs suspend enforcement |
| `MELEE_WATCHDOG_ACTION=log\|dump\|exit` | interactive default log, unattended default exit; dump/exit attempt `hang.dmp`, exit uses code 86 |
| `MELEE_SCRIPT_WATCHDOG_SECS=N` | recurring failing/refusing Lua callback summary interval, default 10 seconds |
| `MELEE_WATCHDOG_TEST=1` | explicit opt-in for console `watchdog-stall 1..120` (diagnostic tests only) |
| `MELEE_DIRECTINPUT=1` | let SDL enumerate DirectInput joysticks (off by default: it could stall the first frame for seconds) |
| `MELEE_SLIPPI_MODE=loopback\|direct` | opt-in experimental two-client replay driver; absent means the existing replay/netplay paths |
| `MELEE_SLIPPI_REPLAY_ROLE=1\|2` | fixture player owned locally; Direct adopts the server's assigned port |
| `MELEE_SLIPPI_DELAY=1..7` | applied-input delay, default 2; initial pads must be neutral |
| `MELEE_SLIPPI_TIMEOUT_MS` | `5000` | ENet and gameplay-stall timeout in ms (`1000..60000`); default preserves 3 s outage grace. Terminal loss shows a toast for 2 s before exit 2. |
| `MELEE_SLIPPI_LOCAL_PORT`, `MELEE_SLIPPI_REMOTE_PORT` | distinct UDP ports required by loopback; Direct uses its ticket assignment |
| `MELEE_SLIPPI_USER_JSON=<path or ->`, `MELEE_SLIPPI_CODE` | Direct profile and opponent code; `-` reads bounded profile JSON from stdin, never log it |
| `MELEE_SLIPPI_EVIDENCE=<path>` | required diagnostic JSON output; pair with `MELEE_STATE_TRACE`, `MELEE_RB_HASHLOG`, `MELEE_SLP_RECORD` |
| `MELEE_SLIPPI_RUN_SALT=<32 hex digits>` | Direct run's shared salt for reciprocal account tags; supplied by the workspace runner |
| `MELEE_SLIPPI_MATCH_ID=<id>` | shared loopback run ID, including when using a UDP impairment relay |
| `MELEE_CARD=0` | disable the memory card (on by default; GCI folder at `_build/card`) |
| `MELEE_SKIP_INTRO=1` | skip the opening movie and boot straight to the title |
| `MELEE_TARGET_TEST=<char>` | boot straight into Target Test with that character (name or ckind; dev/testing) |
| `MELEE_PAD_SCRIPT=<file>` | text scripts consume PADReads; `.lua` files run gameplay scripts whose `gd.input` holds count completed logic frames, including paused single steps |
| `MELEE_PAD_IGNORE_ADAPTER=1` | ignore a physical adapter (use with scripted input) |
| `MELEE_NETPLAY_TURBO=on` / `off` / `<hex>` | scripted netplay: the host's Turbo match rule (gw_matchrules.h); on a guest it is what the guest insists on, and a host with another word refuses it at the handshake ("different match rules"). Scene token `turbo=` for offline runs; settings `turbo_online` (hosting), `turbo_versus` (local Versus), `turbo_colanim` (the indicator's colour-animation id) |
| `MELEE_CPU_IDLE=1` | every CPU-controlled fighter stands still for the whole process (neutral input at the AI's write point; it still takes hits, falls, respawns). Same as scene `cpus=idle`; per slot `p2=fox/idle`. A script's `gd.cpu_mode(port,"fight")` overrides one slot, `"default"` returns it. Ignored in netplay. `run.sh --idle-cpus` sets it; agent test runs should. Log: `cpu: P2 idle (global)` at match start; `gd.cpu_modes()` |
| `MELEE_PAD_DIAG=1` | adapter enumeration + raw report dumps |
| `MELEE_NO_ONBOARD=1` | skip the first boot's visit to SETTINGS > CONTROLS (also skipped for any `MELEE_SCENE` / `MELEE_PAD_SCRIPT` run; settings.cfg `onboarded=1` records it) |
| `MELEE_PROFILER=1` | bounded native zones/counters; see workspace `docs/profiling.md` |
| `MELEE_PROF_REPORT=<path>` | JSON run report on clean shutdown or `prof report` |
| `MELEE_PROF_TRACE=<path>` | Chrome/Perfetto JSON trace; also enables shutdown trace |
| `MELEE_PROF_HITCH_MS=<ms>` | work-time hitch threshold, default 16.6667 ms; excludes pacing |
| `MELEE_PROF_GPU=1` | request optional timestamp-query device feature at startup |
| `MELEE_MSAA=1 or 4` | native multisampling request; benchmark uses 4, default 1 |
| `MELEE_SYNCTEST_BENCH=1` | test-only offline scene-counter SyncTest; combine with `MELEE_SYNCTEST=12` |
| `GW_PROF_TRACY=1` | build-time optional Tracy client; forbidden with `GW_RELEASE_BUILD=1` |
| `MELEE_PROFILE=1` | per-frame timing split, percentiles, histogram (see section 20) |
| `MELEE_SYNCTEST_BENCH=1` | test-only offline SyncTest frame source for ordinary matches; pair with `MELEE_SYNCTEST=12` to force the deployed maximum rollback depth after warmup. Existing render-pool safety may defer resimulation; report achieved `rollback.frames`.  Refuses takeover while rollback/netplay owns snapshots. **The full byte compare is not a gameplay proof:** with `turbo=off`, a plain LAB match mismatches from about frame 41 on (2371 of 3600 checks in a 100 s run) in HSD particle state only (`hsd_804D0908` = `particle_list` heads, `hsd_804D0F90` = the generator pool, and the particle objects they own, which draw `HSD_Randf`); use `MELEE_SYNCTEST_CURATED=1` (gameplay-only hash) as the proof: same scene, `MELEE_SYNCTEST=12`, 70800 checks, 0 mismatching (2026-10-04). |
| `MELEE_PAD_BOT="<slot>,<channel>,<seed>[;...]"`, `MELEE_PAD_BOT_EDGE=<half width>` | TEST-ONLY reactive pad program (gw_script_pad.c) for the Turbo soak: drives pad `channel` as the fighter in `slot` (0 = P1), reads the fighters like the Lua API does, and writes ordinary pad values, so online it is sent and rolled back like a human's input. It walks into range, throws a move and presses a DIFFERENT move on the first free frame after the Turbo window opens (jab/tilt chains, smash chains, special-jab, jab-dash, jab-crouch-jab loops, aerials with the air-jump restore). Logs `pad bot slot N: ...` counters every 1800 reads. Edge 62 = Battlefield (default), 70 = Final Destination. Not for play. |
| `MELEE_SYNCTEST_BENCH=1` + `MELEE_SYNCTEST_CURATED=1` with changing input | A live scripted pad is re-read for a resimulated frame, so a match with changing input used to mismatch for that reason alone (only idle fighters passed). The bench now records each fighter's processed input per frame on the first pass and gives it back to the resimulation (`gw_Snap_InputHook`, fighter.c). The curated record includes the Turbo window word. A first mismatch lists the differing field (`snap:   record N ... word K`; words are byte-swapped, word 3 motion, 17 hitlag, 24 the Turbo word). Sustained heavy combat still starts mismatching after 2-4 thousand frames with Turbo OFF as well (hitlag/animation frame off by one in the resimulation): a separate, unexplained SyncTest issue, not a Turbo one. |
| `MELEE_SYNCTEST_CURATED_DIFF=1` (with CURATED) | report-only byte compare, at every resimulated frame, of each fighter's whole struct, its GObj and its joint tree against the first pass (`gw_snap.c sn_region_diff`, regions registered by `fighter.c`). Logs the first frame each word differs (`snap: REGION-DIFF new word: frame F tag T +0xOFF`), pointer differences with both objects' headers, and a periodic `region diff:` line. Tags: 1-12 fighter struct, 100+ GObj, 1000+player*256+n a joint. See `_research/rollback-synctest-status.md` (2026-10-05). |
| `MELEE_SYNCTEST_CURATED_SETUP=1` | experiment: run each fighter's `HSD_JObjSetupMatrix` over its joint tree at the top of every curated iteration, so a logic-only resimulation has the world matrices a rendered first pass has (removes the ulp-level hit/hurt position differences; does not by itself make the bench mismatch-free). |
| `MELEE_MEX=test_no_capturecut_guard`, `test_log_capturecut` | test flags for the grab-cut fault (`ftCo_800DCE34` NULL partner): the first compiles the TARGET_PC NULL guard out to reproduce the retail-latent read of low memory, the second logs `CAPTURECUT-TRACE` at each `ftCo_8008EC90` call site with the grabber's `victim_gobj`. |
| `MELEE_TURBO=1` / `--turbo` | accelerate simulation with a virtual clock and muted audio; requires `MELEE_PAD_SCRIPT` or `MELEE_LAB_BATCH`, refuses netplay/Slippi/fake rollback and ordinary player windows |
| `MELEE_TURBO_RENDER=N` | present every Nth game frame in turbo (default 8, 0 never; 0-10000); hidden/minimised windows never present |
| `MELEE_TURBO_DRAWS=1` | retain display lists and skinning on unpresented turbo match frames; normally suppressed while render callbacks still run |
| `MELEE_FPS=u` / `MELEE_FPS=120` | uncapped / capped interpolated presentation; realtime logic remains 60 Hz |
| `MELEE_MODS_DIR=<path>` | parent of mod folders; use a Windows path (`pwd -W` in Git Bash), not `/c/...` |
| `MELEE_TURBO_HASHLOG=<path>` | optional per-match-frame full snapshot hash CSV for realtime/turbo parity checks |
| `MELEE_TEST_SEED=<integer>` | fix the boot RNG seed for scripted parity checks; otherwise use OSGetTick |
| `MELEE_WINDOW_X/Y` | window position; may be negative. Applied at creation, so no flash |
| `MELEE_WINDOW_W/H` | window size (Aurora clamps to at least 640x480) |
| `MELEE_AUDIO_LATENCY_MS` | output ring cushion, default 60 (section 19.2) |
| `MELEE_AUDIO_DUMP=<path>` | write the final mix to a 32 kHz stereo WAV |
| `MELEE_AUDIO_NOFX=1` | bypass the aux effect processors |
| `MELEE_BACKEND=d3d12\|auto\|vulkan` | override the default D3D11 backend |
| `MELEE_AURORA_VERBOSE=1` | log Aurora INFO (present mode, adapter) |
| `MELEE_LOG=gobj` | log each GObj render callback address, class, owner and link before invocation (very verbose; Classic IntroEasy diagnosis) |

`MELEE_WINDOW_HIDE=1` remains unsuitable for realtime play: a hidden window makes the D3D11
present block. Turbo runs GX/EFB work offscreen and skips swapchain presentation while hidden.

### Agent lifetime ownership

Always launch through the workspace's `tools/port/run.sh` with `MELEE_RUN_OWNER`
and a distinct sandbox. Native Windows Python creates the suspended child inside
a Windows 10+ Job Object atomically, with kill-on-close and no inherited job
handle. Wrapper death, Ctrl+C, or supervisor death ends only that owned game.
The supervisor watches bash's native parent handle, including when MSYS `timeout`
ends only the wrapper. Finish with `python tools/port/runs.py wait NAME`, then
`python tools/port/runs.py status --owner TAG`; confirm empty before claiming no
games remain. `--max-seconds N` belongs before the sandbox name; unattended
default 300 seconds, interactive default no overall limit.

`runs.py status` lists all live games, ownership, window/monitor position, memory
and heartbeat state; `reap --owner TAG` / `--sandbox NAME` / `--hung` /
`--older-than SECONDS` combine with AND and only terminate verified tracked
identities. It never reaps untracked games. Default root `_build` includes agent
lanes; supply global `--root PATH` before the command to narrow it. `wait` refuses
ambiguous names. The one-line verdict and `verdict.json` preserve outcome; code
86 is a hang, 124 an overall timeout, 125 wrapper/reap/launch failure, 130 interrupt.

`heartbeat.json` is atomically refreshed about once a second independently of
the main thread. `hang.txt` gives stuck main-thread PC/map symbol or image offset,
scene, last instrumented shim and Lua callback/instruction count. Intentional
pause, hidden/minimized/DWM cloak, no presentation and no logic progress are
distinct states. Only the foreground interactive window may hold the adapter;
ignore/unattended runs cannot open it. General covering by
other windows is not measurable (`occlusion_known=false`). Native and supervisor
watchdog enforcement are disabled by `MELEE_WATCHDOG_SECS=0`; the overall limit
is separate. Debugger/system-modal exemption has a fresh grace interval on release.

Creation-time identity checks allow 1 ms for CIM rounding; no-match `reap` exits 1.
Disc paths appear as `<disc>` in diagnostic output and run metadata. Unattended
volume defaults to 0, interactive to 3; explicit `MELEE_VOLUME` overrides either.
Scene launch has a bounded 30-second `scene-transition` state and never exempts
a stalled main loop. Pause logging has a 30-second entry cap and summaries.

Progress deadlines: `gd.deadline("staging maze",600)` before staging and
`gd.deadline_done("staging maze")` after it. Expiry logs once and invokes the
owner's `on_deadline(name,frames_waited)`; it counts completed live logic frames
and pauses with logic. A repeated open does not extend a deadline; cancel before
rearming. See the workspace `tools/port/README.md` for evidence files and tests.

## Run without the window appearing on a monitor

Some work needs the game running while the user is doing something else on screen. Place the window
off every monitor at creation time and hide the console:

```powershell
$env:MELEE_WINDOW_X = "30000"; $env:MELEE_WINDOW_Y = "30000"
Start-Process -FilePath C:\gdm\_build\melee-pc.exe `
  -ArgumentList '--iso','"<GW_ISO_VANILLA>"' `
  -WorkingDirectory C:\gdm\_build -WindowStyle Hidden -PassThru
```

Frames can then be captured with `PrintWindow(hwnd, hdc, PW_RENDERFULLCONTENT)`, which works on an
off-screen window. A separate Windows desktop (`CreateDesktop` + `STARTUPINFO.lpDesktop`) does
**not** work - the process dies with `STATUS_DLL_INIT_FAILED` (0xC0000142).

**Audio tests must drive input.** With no pad script the port sits on the opening movie and then
the title screen; the game requests no AX voices there, so the mixer is legitimately silent and it
looks exactly like broken audio. Use `MELEE_PAD_SCRIPT` and confirm `gw: AX: first audible frame`
in the log before concluding anything.

## Stage `.dat` tooling (HSD): read, edit, re-emit — see DEVLOG §30–§33

Outside-game tooling for stage `.dat` files, built on [HSDLib](https://github.com/Ploaj/HSDLib) (MIT).
Blender is the mesh authoring tool; the `.dat` stays the source of truth.

```
# build the library (one-off). Windows .NET SDK 10. 0 errors; a missing GCILib reference is a harmless warning.
cp -r <HSDLib clone> C:\gdm\_build\HSDLib
cmd.exe /c "cd /d C:\gdm\_build\HSDLib && dotnet build HSDLib.sln -c Release"

# tools (both net8.0-windows7.0 console apps that ProjectReference HSDRawViewer.csproj)
C:\gdm\_build\hsd_export\bin\Release\net8.0-windows7.0\hsd_export.exe <dat> <outdir>   # DAT -> per-group glTF
C:\gdm\_build\stagec\bin\Release\net8.0-windows7.0\stagec.exe  <verb> ...            # the compiler
```

`stagec` verbs: `rt <in> <out>` (round-trip + compare), `coll <dat>` (dump collision),
`load <file>` (headless scene-load probe), `objim <in.obj> <out.dat>` (OBJ -> HSD, tests the GX
encoder), `build <base.dat> <out.dat> <mesh.obj> <groupIndex> [clear] [material] [floor]`.
**`floor` panics the game — do not use it (DEVLOG §32.2).**

**Three HSDLlib patches are required** (fork-diff, DEVLOG §30.6): `NewModel` private->public,
`Work()`'s `MessageBox.Show` -> stderr, and `HSD_POBJ`'s `DisplayListSize`/`DisplayListBuffer`
setters internal->public.

### Gotchas learned the hard way
- The `IONET.dll` vendored in `HSDRawViewer/lib/` has **no glTF importer** (only Fbx/Obj/SMD), so
  `IOManager.LoadScene()` returns null for any `.glb`. **Mesh interchange is OBJ.** glTF *export*
  works (it uses SharpGLTF directly).
- `ImportModelFromScene` opens GUI dialogs – drive `ModelImporter` directly (ctor + `Work(bw)`), and
  pass `new BackgroundWorker { WorkerReportsProgress = true }`. With the default `false`,
  `ReportProgress` throws into a catch that pops a **modal MessageBox = infinite headless hang**.
- Root symbols are typed **by name** (`HSDRawFile.cs:896`); an emitted root must match an existing
  rule (e.g. `*_joint` -> `HSD_JOBJ`), or it reloads as nothing.
- `HSD_DOBJ`/`HSD_POBJ` are `Next`-linked; "keep one" means setting `Next = null`.
- **Mesh space != world space** — the JOBJ tree carries the scale (collision says the stage is
  ~281x306 units; raw mesh verts span ~1335x690). Apply or reset joint transforms deliberately.

### Testing a modified stage in-game without touching the port
The port reads the ISO by path and trusts the FST, so patch a **copy** of the disc image: write the
rebuilt `.dat` at the original offset and update that FST entry's length. Rebuilt stages are usually
*smaller*, so they fit in place.

| | |
|---|---|
| disc image copy | `_build/melee_mod.iso` |
| `GrTFx.dat` FST entry | `435` |
| FST start | `0x456e00` (node = `FST + entry*12`; length field at `+8`) |
| `GrTFx.dat` disc offset | `1296203776` |

Restore vanilla before handing the machine back (same write, with the original 633,100-byte file).

### Windows/Blender helpers in `_build/`
- `launch_stage.ps1 -Iso <iso> -TT <char>` — launch on the **primary monitor** with
  `MELEE_TARGET_TEST` set. `capture_stage.ps1` / `capture_now.ps1` — PrintWindow captures;
  `press_start.ps1` — focus the window and send Enter; `self_test_keys.ps1`.
- Blender (Steam, v5.2.2): `D:\SteamLibrary\steamapps\common\Blender\blender.exe`. Headless:
  `blender.exe --background --python <script.py>`; script and arguments must be **Windows paths**.
  `bpy.ops.wm.obj_export` in 5.x takes **no** `export_format` argument.
- **Two PowerShell/`cmd` traps that cost 15-minute timeouts each:** `Start-Process ... -PassThru
  -RedirectStandardOutput <f>` makes PowerShell **block until the child exits** (never use it for a
  long-running game); and `cmd.exe /c "tasklist /FI \"IMAGENAME eq x.exe\""` from WSL mangles its
  quoting and hangs. Use a plain `cmd.exe /c tasklist | grep -i melee` instead.

### First boot and SETTINGS > CONTROLS
- **The save.** The memory card's "There is no save data. Create one?" answers Yes by itself on the
  port (`gmscmemcard.c` case 2, the prompt's own Yes branch; the log says `memcard-autocreate`).
  settings.cfg `auto_create_save=0` puts the question back.
- **The controller page.** The first time the menus are ready, they open SETTINGS > CONTROLS once
  (`fm_first_boot`, `gw_Onboard_Pending` in `shim_pad.c`; settings.cfg `onboarded=1` afterwards).
  The page shows each port live (the device, stick, C-stick, triggers and held buttons, from
  `gw_Pad_Source` / `gw_Pad_Value` / `gw_Pad_Name`: what the game saw on the last read), so it is
  also the controller test. It has GameCube Adapter recalibrate, a Stick Dead Zone row for SDL
  controllers (settings.cfg `stick_deadzone`, percent; applied through Aurora's `PADGetDeadZones`
  as each SDL pad appears; not shown without one), and How to Play Online (six read-only steps).
- Controller remapping is implemented in source at CONTROLS > Remap Controller
  (`gmfrontend_controls.inc`, `shim_pad.c` / `gw_controls_runtime.inc`). Choose a
  Game Input, Bind Input, then press/release a physical input. Swap/Also resolves
  conflicts, with four named profiles, three data presets, live testing, per-stick
  deadzones, swap sticks, analog-off, digital light/full shield and rumble.
  The editor uses the original layout for navigation. START+B cancels capture;
  hold the original START+B for two real seconds in menus to reset the active
  profile. GC identities are adapter ports; SDL uses GUID/name, falling back to
  VID/PID/name. Identical SDL devices share profiles. Assign to Port copies the
  current profile to a connected pad of the same source kind; it does not move
  the hardware. Persistence uses settings.cfg `ctlNN_id`, `ctlNN_active`,
  `ctlNN_pP_map/name` (16 device identities, four profiles each).
  Untouched profiles retain Aurora/adapter calibration. Custom SDL samples use
  the public SDL handle exposed by Aurora; no Aurora source edits are needed.
  Script pads are overlaid afterwards and are never remapped.
- Tap-jump-off remains outstanding: clamping up would break tilts/aim and a
  local-only game-side preference would desync. No such approximation is shipped.
  Compile/link, executable tests and real controller/aspect-ratio acceptance are
  pending. See workspace `_build/tmp/codex-controls-remap-report.md`.

### The keyboard does not play
The keyboard is hotkeys only (F9/F10, the console's backquote, `gd.key` for scripts); it never
drives a pad. For unattended runs use `MELEE_PAD_SCRIPT` (or `gd.input` / the console `input`
command). `MELEE_INPUT=none` (old value: `keyboard`) opens no devices and leaves port 1 as a
controller at rest.

## Conventions that have bitten workers
- **Big-endian game memory.** Native shims must read/write game-visible scalars with `gw_r32`/`gw_w32` (and `gw_r16`/`gw_w16`, `gw_rf32`/`gw_wf32`). A native little-endian store the game then byte-swaps reads back wrong (e.g. `AXVPB.index`).
- **Shim boundary.** A shim defines `gw_X`; the pipe/gwtool prefixes *every* symbol in a game TU with `gw_`, so game code must call the **unprefixed** `X`. Declare `extern void wait_idle(void);` and call `wait_idle()` under `TARGET_PC` - NOT `gw_wait_idle`, which double-prefixes to `gw_gw_wait_idle` and fails to link (this exact mistake cost a link cycle).
- **Deferred completions.** ARQ and DVD completions are queued via `gw_defer` and pumped only in `gw_wait_idle`/`gw_frame_tick`. Any game-side blocking spin that calls no shim deadlocks; fix it by pumping `gw_wait_idle()` inside the spin (TARGET_PC-guarded). Precedents: the pad gate (`shim_dvd.c` `gw_DVDGetDriveStatus` -> `gw_wait_idle`), `lbarq.c` ARQ wait, and `synth.c` deflag sync (see `shim_ar.c:12-14`).
- **Game-source changes** must be `#if defined(TARGET_PC)`-guarded with the original code kept.
- **Audio backend exists** (2026-09-12/13, reworked 2026-09-14 - see DEVLOG section 19). Aurora has no `ax`/`ai`/`dsp`, so `shim_ax.c` implements AX: DSP-ADPCM decode, a 64-voice mixer over `gw_aram` with linear resampling, both aux buses with native AXFX reverb-std and delay, ITD and de-pop, over an SDL3 32 kHz stereo output paced by the ring fill level. `HSD_SynthCallback` is pumped per sub-frame from `gw_frame_tick`.
- **Cutscenes work** (2026-09-14 - DEVLOG section 21). `extern/dolphin/src/dolphin/thp/THPDec.c` is built as a game TU with its Gekko paired-single IDCT and assembly Huffman decoders reimplemented in C behind `TARGET_PC`; the opening movie plays by default.
- **Commits:** work in the authorised branch. Do not commit, merge or push when the task forbids it; otherwise stage named paths and use a short subject. Never `git add -A`.
- **Evidence:** each task writes `.omo/evidence/task-<name>.log` with the exact commands and their outputs.
- **Aurora bootstrap cache.** `build_aurora_x86.bat` builds the vendored `melee/extern/aurora`. If `_build/ax86` was previously configured from another Aurora checkout (`dusklight/extern/aurora`), CMake refuses the stale cache ("does not match the source ... used to generate cache") - delete `_build/ax86/CMakeCache.txt` and `_build/ax86/CMakeFiles/` and re-run. Keep `_build/ax86/_deps`: `ax86m` reuses its `dawn-src` and SDL3 package.

## Endianness: one system exists (`gwtool` + `gw.h`) - do not build a second

**Mechanism.** Game TUs go clang `--target=ppc32-none-eabi ... -DLINT -DTARGET_PC` -> LLVM IR ->
`_build/gwtool/gwtool.exe` -> x86 COFF. gwtool byte-swaps *every* memory access, so game memory is
big-endian exactly as on GameCube and values are native only in registers; it also prefixes every
symbol with `gw_`. The full contract is the header comment of `pc/platform/gw.h` - read it before
touching a shim.

**Who swaps.** Game `src/` contains no swap code by construction (gwtool does it). Shims are native
x86 and swap by hand: every multi-byte field behind a game pointer must use `gw_r16/w16`,
`gw_r32/w32`, `gw_r64/w64`, `gw_rf32/wf32`, `gw_rptr/wptr`. A native LE store the game then
byte-swaps reads back wrong.

**ABI at the boundary** (`gw.h`): scalars (int/float/pointer) arrive native - declare normally; a
by-value struct arrives as a byte copy, so its fields are big-endian; a <=8-byte struct return comes
packed big-endian; a larger struct return uses a BE sret pointer.

**Swap-site inventory (2026-09-13; `shim_ax` has grown a lot since - recount before relying on it).** Shims: `shim_card` 40, `shim_ax` 23, `shim_gx` 23,
`shim_dvd` 7, `gw_runtime` 6, `shim_pad` 5, `shim_ar` 4, `shim_libc` 2, `shim_os` 2, `gc_adapter` 1.
Game `src/`: 0 (by construction).

**Formerly-known boundary violation (FIXED, 2586eccbc).** `shim_ax.c` stored the `AXVPB`
`priority`, `callback` and `userContext` fields with *native* stores while the game read
them byte-swapped, which made `HSD_SynthSFXUnloadBank_inline` unlink the wrong sfx ids and
crash in `HSD_SynthSFXPlayWithGroup`. Kept here as the canonical worked example of the
class: a wrong native store surfaces later as a garbage pointer, not as an obvious
endianness bug.

**Rule for new shims:** any field a shim writes that the game can read must go through a `gw_*`
accessor. When in doubt, use the accessor - a wrong native store surfaces later as a garbage
pointer, not as an obvious endianness bug.

### Pipeline warming and covered launches (2026-10-03)

`MELEE_PIPELINE_SKIP` defaults to `1`: a missing GX pipeline on the native
worker backend skips its draw while compilation proceeds. `0` restores blocking
waits for diagnostics. This applies to GX draws, including undeclared fighter
permutations; explicitly warm loaded fighter ports when staging them.
`AURORA_PIPELINE_WORKERS` selects 1..8 workers (default hardware threads minus
two, clamped). WebGPU's existing synchronous backend is outside this guarantee.

`MELEE_PIPELINE_COVERAGE_DIR` overrides the learned snapshot directory; Windows
defaults to `%LOCALAPPDATA%/GD Melee/pipeline-coverage`. Each sandbox retains its
own mutable SQLite database. Clean shutdown publishes a uniquely named immutable
snapshot; boot imports the latest eight snapshots. Crashes do not publish a new
snapshot. The immutable database carries content keys and origin labels, not
portable driver binaries.

`MELEE_SCENE=mission=<folder>` or `maze=<seed>,<size>` requests staging under
the loader. An active `mod.json` may declare `"autostart":"mission=<folder>"`
or `"autostart":"maze=7,12"`; an explicit scene wins. The owning gameplay
script implements `on_launch(request)` and calls `gd.launch_ready()` after its
stage and `gd.warm` handles are ready. Missing/failing owners keep the host
covered; `gd.launch_cancel(reason)` is the explicit escape. See the workspace's
`docs/scripting.md` no-hitch APIs and `_build/tmp/codex-no-hitch-engine-report.md`
for restrictions and native acceptance targets. A matching Aurora rebuild is
required; syntax checks do not establish behavior in an existing EXE.
