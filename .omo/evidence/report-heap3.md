# Heap 3 expansion for six full Geno fighters

Source changes only, 2026-10-10. **Not built or run; no commit created.** The packet assigns building and testing to Claude. All edits are in this game worktree. No disc-derived assets were created or copied.

## Change and sizing

| Region | Before | After |
| --- | ---: | ---: |
| PC heap 3 (Stay) | 5,351,056 B (`0x51A690`) | 13,739,664 B (`0xD1A690`) |
| PC MEM1 | 40 MiB (41,943,040 B) | 48 MiB (50,331,648 B) |
| MEM1 address range, exclusive end | `[0x80000000, 0x82800000)` | `[0x80000000, 0x83000000)` |
| PC heap 4 (AllM) | 23,376,896 B (`0x164B400`) | unchanged |
| PC heap 5 (AllA, in ARAM) | 26,658,816 B (`0x196C800`) | unchanged |
| PC ARAM | 32 MiB | unchanged |
| Main heap 0 | remainder of the PC arena | same capacity for the same reservations |

Heap 3 and MEM1 both gain exactly 8 MiB. This funds the stage heap without consuming the existing main heap or reducing heaps 4/5. Heap 3 grows upward; heap 4, anchored to the arena end, moves upward by the same amount. The main heap's endpoints both move by 8 MiB, preserving its capacity. The small heaps 2 and 6 retain their sizes. Retail descriptors remain untouched under `#else`, including retail heap 3 at `0x4F8800`.

The packet's worst observed ACE ext 121 load left 9,520 B. With identical allocations, the extra 8,388,608 B projects **8,398,128 B free**; Fountain's 139,856 B projects 8,528,464 B. These are arithmetic projections, **not new measurements**. Adding only 4 MiB would barely clear the requested floor on ACE; 8 MiB gives another roughly 4 MiB of future growth above it. The regression uses a conservative binary floor of **4 MiB = 4,194,304 B**. No stage sweep gates the expansion.

## Files changed

- `src/melee/lb/lbheap.c`: PC heap 3 descriptor, allocation rationale, and a startup capacity log in `lbHeap_80015F3C`.
- `pc/platform/gw_runtime.c`: MEM1 40 -> 48 MiB, with the paired heap-growth and simulated-memory caveat documented.
- `pc/platform/gw_snap.c`: replace the remaining fixed 24 MiB pointer-diagnostic bound with the runtime MEM1 end; check the complete twelve-word read using 64-bit arithmetic.
- `pc/platform/gw_net.h`: protocol 5 -> 6, because the heap addresses and snapshot layout change.
- `pc/platform/gw_net_tests.c`: `net_refuse_old_heap_layout` simulates both an old guest HELLO and an old host ACCEPT, with equal content hashes, requiring refusal before either peer starts.
- `pc/platform/gw_script.c`: additive read-only `gd.match().stage_external` from `gw_Stage_80225194()`, so the regression checks the loaded external stage rather than the requested scene or internal GrKind.
- `docs/scripting.md`: documents `stage_external` and its `-1` value outside a match.
- `pc/tests/geno_six_full_heap3.lua`: real six-CPU load/play driver, including workbench resume and active ext-121 verification.
- `pc/tests/geno_six_full_heap3.py`: log-based heap-headroom assertion after the real load.
- `.omo/evidence/report-heap3.md`: this handoff.

No new native translation unit or link-list entry is needed. The generated `gw_mex_bridge.c` was not hand-edited; the build below must regenerate it.

## Heap path and dependent audit

This is a source audit, not executable or runtime verification.

| Area / files checked | Finding / action |
| --- | --- |
| `src/melee/lb/lbheap.c`, `lbheap.h`, `lbheap.static.h` | `lbHeap_803BA380` supplies sizes; `lbHeap_80015F3C` lays out the persistent heaps from `HSD_GetNextArena`; `lbHeap_80015900` gives the remaining interval to HSD. `lbHeap_Capacity` and `lbHeap_Free` read live allocator state. Only the PC heap-3 descriptor changes. |
| `src/melee/lb/lbmemory.c` | Allocation/handle bookkeeping uses the heap start/end and aligned request sizes; no separate 5,351,056-byte ceiling. ARAM remains separate. |
| `src/melee/lb/lbdvd.c` | `lbDvd_SixBudget` uses live capacities for admission. Existing heap-4/5 1 MiB reserves and the 4 MiB main-heap floor stay unchanged. `lbDvd_80017CC4` logs `six-slot after-load heap=... free=... capacity=... fighter_files=...` only when the planned scene's preload queue has no pending entries and fighter files are loaded. This is the regression's measured source; planned-budget lines cannot satisfy it. |
| `src/sysdolphin/baselib/initialize.c` | `HSD_OSInit`, `HSD_GetNextArena`, `HSD_CreateMainHeap` follow OS arena bounds; no old MEM1 capacity baked into allocation. |
| `pc/platform/gw_runtime.c`, `shim_os.c` | Both VirtualAlloc paths use `GW_MEM1_SIZE`. Physical-memory size and low-memory size words follow it. OS arena high, m-ex persist (0x60000), optional Geno persist (0x100000), costume mirror, and scratch placement derive from `gw_mem1_size`; their capacities stay unchanged and their addresses move with the top. |
| `src/melee/gm/gmmain.c`, `pc/platform/shim_os.c` | `OSGetConsoleSimulatedMemSize()` intentionally stays 24 MiB. Reporting 48 here would trigger the retail dev-kit branch that withholds 24 MiB. Do not change it along with physical MEM1. |
| `src/sysdolphin/baselib/leak.c` | Pre-existing limitation: `HSD_Leak_80387DF8` scans to simulated-memory end (24 MiB), so its reference scan omits expanded PC memory. It is a diagnostic, not an allocation/snapshot boundary. Left unchanged and not used as evidence for expanded-memory coverage. |
| `pc/platform/gw_ppc.c` | Interpreter effective-address validation (`gw_ppc_ea_ok`) and stack checks use runtime MEM1 size; the wide range arithmetic admits the new tail without overflow. |
| `pc/platform/gw_mex_ftfunction.c`, `gw_mex_ftfunction_runtime.c`, `gw_mex_grfunction.c`, `gw_mex_items_query.inc` | Blob/code installation, guest-pointer checks, scratch/stack ranges, costume/runtime mirrors and item queries use runtime bounds or the OS reservation helpers. Fixed original guest function addresses are symbol identities, not heap upper bounds. |
| `pc/geno/geno_registry.c`, `geno_define_registry.inc` | Script-memory and native-callback guest ranges use runtime bounds; the full-definition registry fixture is independent of the arena capacity. |
| `pc/platform/shim_gx.c`, `gw_script.c`, `gw_runtime.c` | GX array remaining-byte calculation, LAB memory inspection/test-tail accesses and address diagnostics use `gw_mem1_size`. New stage query crosses the native/game boundary as an integer; it does not read native-endian game memory. `Stage_802251E8` selects the external id before ground initialization, and `Stage_80225194` returns that selected id. |
| `pc/platform/gw_snap.c`: storage / dirty tracking | `sn_open` derives page count from size; page maps, dirty bitmaps, GetWriteWatch address buffers, hash-page arrays, comparison/pre-save buffers, full views and copy loops follow it. 4 KiB page count grows 10,240 -> 12,288. No fixed 40 MiB snapshot allocation remains. |
| `gw_snap.c`: hashes / masking / rewind | MEM1 masks, raw snapshot hash traversal, cross-platform hash traversal, rewind keyframes/deltas and full comparison buffers use runtime size/page count. The REGION-DIFF pointer-detail diagnostic alone still used 0x81800000; this change widens it and checks all 0x30 bytes before reading. |
| `gw_snap.c`: saved files | The file header stores MEM1 size and page count. Load rejects a size/page mismatch before restoration, so old 40 MiB states are incompatible and rejected. No file-format field changes are required. Create fresh states with this build. |
| `pc/platform/gw_rollback.c`, `src/melee/ft/fighter.c` | Rollback storage delegates to the dynamic snapshot system. `ftRb_GameHash` / `RB_FighterHash` hash curated simulation fields (including six ports/followers and Geno data), not a fixed old MEM1 span. |
| `pc/platform/gw_net.c`, `gw_net.h`, `gw_netplay.c` | HELLO/ACCEPT serialize and validate the protocol macro. Version 6 explicitly rejects old-layout peers in either direction. The generated source build id provides a separate compatibility check; the new test does not rely on different content hashes. |
| `pc/platform/gw_test.c`, `gw_writewatch_linux.c` | Test-isolation MEM1 copies are runtime-sized. The Linux write-watch implementation takes the supplied region size and derives page arrays from it (Windows remains the requested validation target). |
| `pc/platform/gw_mex_bridge.c/.h`, `gw_mex_sigs_gen.inc`, workspace `tools/port/build.sh` | The bridge maps original guest symbols to current native addresses, rather than limiting MEM1 to its old end. Function/data addresses may move after the edits. Use build.sh's link/regenerate/relink fixpoint and final ABI audit; do not use --no-bridge or a raw link. |
| `pc/tests/mex_items_query_test.c`, fixed-address test fixtures | Its 24 MiB value is a standalone mock, not the game's MEM1 allocation. Fixed fixture scratch addresses inspected remain inside both old/new arenas. No production resizing needed. |

The source search covered old heap-3 values, 40 MiB literals/expressions, old MEM1 ends (0x81800000 and 0x82800000), and users of `gw_mem1_size` in the game/PC sources. Historical comments describing older expansions remain historical.

Memory cost: MEM1 commitment grows by 8 MiB; each full MEM1 buffer also grows by 8 MiB. Each snapshot slot adds 8,192 B of page indices and 256 B of dirty bits; shared hash tables grow too. Unmodified snapshot pages can be shared, but there is no claim of zero RAM or timing impact. Measure rollback/rewind memory and timing if those costs matter for release acceptance.

## Regression design and review

The existing `geno_define_six_full` tests registry capacity (six definitions, 300 overlays and 4,200 pool words each); it does not boot HSD or load a stage. `main.c --test` deliberately has no game boot/frame driver, so an assertion added only there could not prove post-load heap headroom.

The new Lua/Python pair runs the existing GPU-free workbench mode, loads the existing stress6 six-Falcon fixtures, resumes workbench's automatic first-frame pause, waits for frame 60, and verifies all six CPU players, distinct kinds and expected fixture names. It verifies the actual external stage is 121, then allows another 600 CPU frames and checks the match/players remain present. The Python check requires all six full-definition budget records (>=300 overlays and >=4,200 words), successful play, actual-stage evidence, and real after-load heap-3 free bytes >=4 MiB. Current local fixture metadata reports 313 overlays / 4,806 words per definition.

The checker fails on missing load evidence, wrong stage, missing players/budgets, low free bytes, or logged fatal/allocation/player-test failures. It must run after the game command: Lua catches failures to print a verdict and quit, so a process exit alone is not its verdict. `--require-headless` also demands the null-backend startup log. This mode creates an offscreen Windows window but uses no GPU device. ACE disc identity is supplied by the explicit `--iso "$GW_ISO_ACE"` command.

A separate read-only code review checked heap topology, dynamic snapshots, protocol bytes and fixture/API agreement. It found the missing stage assertion and the workbench startup pause; both are addressed. No compiler, tests, script execution, game launch or performance check ran during this task.

## Exact commands for Claude

Run these blocks in order from **Windows Git Bash**. They use the existing stress6 worktree/build root and six fixtures; no asset generation or shared-mod installation is needed. Use the configured ACE image from the workspace .env.

### Build and prove the new code is in the EXE

```bash
cd "E:/Projects/Melee Workspace"
set -euo pipefail
set -a
source ./.env
set +a
export GW_ROOT="E:/Projects/Melee Workspace"
export GW_MELEE="$GW_ROOT/worktrees/stress6"
export GW_BUILD_ROOT="$GW_ROOT/_build/agents/stress6"
export MELEE_RUN_OWNER="stress6-heap3"
: "${GW_ISO_ACE:?GW_ISO_ACE must name the user's ACE disc image}"

bash tools/port/build.sh \
  --tu src/melee/lb/lbheap.c \
  --shim gw_runtime.c --shim gw_snap.c --shim gw_script.c \
  --shim gw_net.c --shim gw_net_tests.c --shim gw_netplay.c

grep -a -oF 'lbHeap: PC stage heap 3 expanded capacity=' \
  "$GW_BUILD_ROOT/melee-pc.exe"
```

Require successful bridge fixpoint and ABI audit. The build's dependency scan also recompiles other consumers of the changed protocol header. A source string without a matching rebuilt executable is insufficient.

### Native headless suite, including the new compatibility regression

```bash
HEAP3_STAMP=$(date +%Y%m%d-%H%M%S)
HEAP3_SUITE="heap3-suite-$HEAP3_STAMP"
unset MELEE_TEST_FILTER MELEE_SCENE MELEE_PAD_SCRIPT MELEE_SCRIPT
unset MELEE_LOBBY_AUTOPLAY MELEE_CPU_IDLE MELEE_WB_HEADLESS
export MELEE_MODS_DIR="$GW_ROOT/_build/mods"
export MELEE_SCRIPTS=0 MELEE_CARD=0 MELEE_UNATTENDED=1 MELEE_VOLUME=0
MELEE_TURBO=1 bash tools/port/run.sh --test --max-seconds 600 \
  "$HEAP3_SUITE" --iso "$GW_ISO_ACE"
python tools/port/runs.py --root "$GW_BUILD_ROOT/runs" wait "$HEAP3_SUITE"
grep -aE 'geno_define_six_full|net_refuse_old_heap_layout|tests complete' \
  "$GW_BUILD_ROOT/runs/$HEAP3_SUITE/melee-pc.log"
```

Require zero test failures and both named tests actually reported. Do not substitute an old baseline suite result.

### GPU-free actual six-fighter load on ACE ext 121

```bash
HEAP3_HEADLESS="heap3-load-$HEAP3_STAMP"
export MELEE_MODS_DIR="$GW_ROOT/_build/stress6/mods"
export MELEE_PAD_SCRIPT="$GW_MELEE/pc/tests/geno_six_full_heap3.lua"
export MELEE_PAD_IGNORE_ADAPTER=1 MELEE_SCRIPTS=0 MELEE_CARD=0
export MELEE_UNATTENDED=1 MELEE_VOLUME=0
export MELEE_WINDOW_W=960 MELEE_WINDOW_H=540 MELEE_RENDER_SCALE=1
export MELEE_WINDOW_X=30000 MELEE_WINDOW_Y=30000
HEAP3_PLAYERS="p1=geno-falcon-1/cpu9/c0;p2=geno-falcon-2/cpu9/c1;p3=geno-falcon-3/cpu9/c2;p4=geno-falcon-4/cpu9/c3;p5=geno-falcon-5/cpu9/c4;p6=geno-falcon-6/cpu9/c5"
HEAP3_RULES="stage=ext:121;items=off;time=0;stocks=99;at=match"
MELEE_WB_HEADLESS=1 MELEE_TURBO=1 MELEE_TURBO_RENDER=0 \
  MELEE_SCENE="mode=workbench;$HEAP3_PLAYERS;$HEAP3_RULES" \
  bash tools/port/run.sh --max-seconds 300 "$HEAP3_HEADLESS" --iso "$GW_ISO_ACE"
python tools/port/runs.py --root "$GW_BUILD_ROOT/runs" wait "$HEAP3_HEADLESS"
python "$GW_MELEE/pc/tests/geno_six_full_heap3.py" --require-headless \
  "$GW_BUILD_ROOT/runs/$HEAP3_HEADLESS/melee-pc.log"
```

### Rendered six-CPU VS smoke on ACE ext 121

This uses the same probe and fixtures in ordinary VS, with rendering enabled in an offscreen window. It records numeric/log evidence and takes no screenshots.

```bash
HEAP3_CPU="heap3-cpu-$HEAP3_STAMP"
MELEE_WB_HEADLESS=0 MELEE_TURBO=1 MELEE_TURBO_RENDER=1 \
  MELEE_TURBO_DRAWS=1 MELEE_FPS=u \
  MELEE_SCENE="mode=vs;$HEAP3_PLAYERS;$HEAP3_RULES" \
  bash tools/port/run.sh --max-seconds 300 "$HEAP3_CPU" --iso "$GW_ISO_ACE"
python tools/port/runs.py --root "$GW_BUILD_ROOT/runs" wait "$HEAP3_CPU"
python "$GW_MELEE/pc/tests/geno_six_full_heap3.py" \
  "$GW_BUILD_ROOT/runs/$HEAP3_CPU/melee-pc.log"
grep -aE 'MEM1|lbHeap: PC stage heap 3|six-slot after-load|HEAP3_SIX_FULL|TEST geno_six_full_heap3' \
  "$GW_BUILD_ROOT/runs/$HEAP3_CPU/melee-pc.log"
python tools/port/runs.py --root "$GW_BUILD_ROOT/runs" status --owner "$MELEE_RUN_OWNER"
```

Require owned-run verdicts `OK`, both Python checks `PASS`, startup heap-3 capacity **13,739,664**, and actual heap-3 free >=4,194,304. Compare the heap-0/4/5 lines to the packet baseline and retain the EXE/map, run metadata, logs and measured free bytes. The final owner status must show no remaining run. Build failure, missing test evidence, runtime failure or timeout is a failed/unverified handoff, not success. If a command fails, inspect its owned run before continuing. Do not commit.
