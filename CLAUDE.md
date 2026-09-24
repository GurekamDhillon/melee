# melee (pc-port): notes for agents

A fork of `doldecomp/melee` (the matching decompilation of Melee NTSC 1.02). Branch **`pc-port`**
adds a native Windows port under `pc/`. The build system, tools, research and releases live in the
**workspace repo** (`GurekamDhillon/gd-melee-workspace`), which expects this checkout at
`<workspace>/melee`; its `CLAUDE.md` and `docs/NEXT-SESSION.md` are the project-level notes.

## Layout

| dir | what |
|---|---|
| `src/melee/` | the game, real editable C. Port changes are under `#if defined(TARGET_PC)` |
| `src/melee/gm/gmfrontend*.c/.inc` | the port's own menus (the "kit"): menu tree, settings pages, online lobby, select screens |
| `pc/platform/` | native shims (GX→Aurora, OS, PAD, CARD, AX, DVD...) and the port's own systems: netplay, rollback, scripting, m-ex interpreter, mods, settings |
| `pc/gameworld/` | game-side code compiled through the PPC retarget (`script_game.c` is the LAB's game half) |
| `pc/geno/` | the Geno fighter-extension engine and the LAB mod (`mods/geno-lab/`) |
| `pc/tools/gwtool` | the PPC→x86 retargeter |
| `pc/docs/PORT_DEV_QUICKREF.md` | **read first for building**: the pipeline, env vars, traps |
| `docs/geno.md` | Geno and the LAB: the API reference and each stage's design |
| `docs/getting_started.md`, `docs/*.md` | upstream decomp docs |

Per-directory `CLAUDE.md` files under `pc/platform`, `pc/geno`, `pc/geno/mods/geno-lab` and
`src/melee/gm` carry the local rules.

## The shim boundary (the rule that bites most)

Game TUs are compiled for PowerPC and retargeted: every memory access is byte-swapped (game memory
stays **big-endian**) and every symbol gets a `gw_` prefix. So:

- A shim defines `gw_X`; **game code calls the unprefixed `X`** (declare `extern int X(void);`
  under `TARGET_PC`). `gw_X` from game code double-prefixes and fails to link.
- Native code reading or writing game memory uses `gw_r32` / `gw_w32` (`gw_r16`, `gw_rf32`...).
  A native little-endian store the game reads back is wrong.
- Scalars cross the boundary; floats go as integer bit patterns (`gs_fbits`, the `union { int i;
  float f; }` idiom in `pc/gameworld/script_game.c`). Never hand the game a native pointer.
- The game cannot read the exe's data: the interpreter reads MEM1 only. Mirror what it needs
  (the costume table in `gw_mex_ftfunction_runtime.c` is the precedent).
- A game-side blocking spin that calls no shim deadlocks: pump `wait_idle()` inside it.

## Building and checking

- **Windows only**, with the user's disc images. Use the workspace's `tools/port/build.sh`
  (it regenerates the bridge: `pc/docs/PORT_DEV_QUICKREF.md`, "THE BRIDGE FIXPOINT") and `run.sh`.
- Headless tests: `run.sh --test <name>`; the suite is in `pc/platform/gw_test*.c`,
  `gw_tests_core.c`, `pc/geno/geno_tests.c`, `pc/tests/`.
- Off Windows: game-side C syntax-checks as PowerPC
  (`clang -fsyntax-only -w -DTARGET_PC --target=powerpc-unknown-eabi -nostdinc -Isrc -Isrc/melee
  -Iinclude -Ilibs/dolphin/include -Ipc -Ipc/gameworld -Isrc/sysdolphin -Isrc/MSL <file>`);
  `pc/platform/*.c` needs a stub `windows.h`. Neither proves a link or a run: say so.
- A fix is verified when a string it adds is in the exe (`grep -a`), and then on screen or in the
  log, not when the source has it.

## Conventions

- Match the surrounding code: comments explain *why* and cite the decomp function or the m-ex asm
  they mirror (`ftCo_LandingAir_EnterWithLag`, `ftCo_800986B0`...). Numbers from the game come
  from its tables (PlCo, attributes), never typed in.
- Anything observable gets a log line (`gw_log`, `OSReport`, `SceneReport_*`); the log is how a
  headless run is checked.
- Upstream decomp files keep upstream's style; port additions in them stay under `TARGET_PC`.
- Never commit disc-derived data. `pc/platform/gw_mex_bridge.c` regenerates on every build; its
  churn in a diff is normal.
- Commit messages: what changed and the cause, with the function names; a "not built or run"
  line when that is the case.
