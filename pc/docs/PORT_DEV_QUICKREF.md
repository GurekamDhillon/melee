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
This workspace revision uses timestamp scans and fixed `xargs -P 8`; it has no `GW_JOBS` option.

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
| `MELEE_DIRECTINPUT=1` | let SDL enumerate DirectInput joysticks (off by default: it could stall the first frame for seconds) |
| `MELEE_SLIPPI_MODE=loopback\|direct` | opt-in experimental two-client replay driver; absent means the existing replay/netplay paths |
| `MELEE_SLIPPI_REPLAY_ROLE=1\|2` | fixture player owned locally; Direct adopts the server's assigned port |
| `MELEE_SLIPPI_DELAY=1..7` | applied-input delay, default 2; initial pads must be neutral |
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
| `MELEE_PAD_DIAG=1` | adapter enumeration + raw report dumps |
| `MELEE_NO_ONBOARD=1` | skip the first boot's visit to SETTINGS > CONTROLS (also skipped for any `MELEE_SCENE` / `MELEE_PAD_SCRIPT` run; settings.cfg `onboarded=1` records it) |
| `MELEE_PROFILE=1` | per-frame timing split, percentiles, histogram (see section 20) |
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

## Run without the window appearing on a monitor

Some work needs the game running while the user is doing something else on screen. Place the window
off every monitor at creation time and hide the console:

```powershell
$env:MELEE_WINDOW_X = "30000"; $env:MELEE_WINDOW_Y = "30000"
Start-Process -FilePath C:\gdm\_build\melee-pc.exe `
  -ArgumentList '--iso','"C:\iso\Super Smash Bros. Melee (USA) (En,Ja) (v1.02).iso"' `
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
- Not built: button remapping (Aurora's `PADSetButtonMapping` is there for SDL pads; it needs a
  "press the button for Z" capture screen).

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
