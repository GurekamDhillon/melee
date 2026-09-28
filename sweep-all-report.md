# sweep-all (code-only lane)

## Changes

All implementation is under `pc/tests/sweep/`; no shared C, workspace sweep,
build files or test registries changed. No build, game launch or commit was made.

- `sweep_all.py`: serial process-per-case runner with a 75-second wall timeout,
  default 180 logic frames, turbo, and draw work retained (`MELEE_TURBO_DRAWS=1`).
  Each run has a fresh directory, copied executable/map and DLLs, isolated empty
  mods directory, disabled boot scripts, deterministic Lua pads and retained logs.
- Full mode first probes the **supplied ACE disc**, requires successful Lua
  checks and both native catalog log lines, then enumerates every reported m-ex
  fighter kind by `fk:` (no assumption that CKind equals FighterKind).
- VS stage coverage: 29 retail selectable stages and every appended m-ex
  external stage ID in `[288, external_count)`, following the existing workspace
  crash sweep's extension boundary. Retail unfinished Akaneia (21) and the
  Icetop alias/unfinished variant (26) are excluded. These are loadable VS
  stages, not just tournament-legal stages. Stage runs use Fox against a level-0
  Fox; each m-ex fighter gets its own Battlefield run.
- Real 1P mode coverage uses `mode=classic;step=0..10` and
  `mode=adventure;step=0..11;difficulty=2`, with Fox. These are the current
  `gw_SceneLaunch_OnePStep` bounds. The old workspace sweep's 18 Adventure
  requests would repeat the clamped last step. This is a per-step launch smoke
  sweep, not a complete playthrough of each chapter's subsequent fights.
- `sweep_all.lua`: shared pad/self-check script. Checks entry, mode/fighter/stage
  identity where an expected stage is known, and continuous match frame progress.
  Prints `TEST sweep-all section <n>: PASS|FAIL <detail>` per check, then calls
  `gd.quit()`. Intro and memory-card input is scripted. Lua has an additional
  wall deadline; the process timeout catches native hangs too.
- `known-issues.json`: initially empty; no unverified historical defect is
  suppressed. A reviewed issue must contain exact `tag`, `result`, `rva`,
  `detail`, and explanatory `reason`. Copy the signature from `results.json`.
  Only the fault's absolute instruction address is ignored for ASLR matching;
  changed RVAs, exception details, cases and failure types remain new failures.
- `summary.md` collapses all passes and known failures into a count line and
  lists only new failures, each with scene, log tail, crash artifact tail when
  present, and fault RVA. A stack return address is never substituted for a
  missing fault RVA: it says unavailable. `results.json` keeps all cases,
  duration/exit status, executable SHA-256 and discovery metadata; `plan.json`
  records the selected cases. Reports update after each run. Fresh timestamped
  directories preserve prior evidence. Exit 0 means no new failures, 1 means
  new failures, 2 means configuration/discovery failure. Known failures still run.

## Integration launch

Copy `pc/tests/sweep/run-sweep-all.ps1` to the workspace lane's tools directory
if desired. Point `-Harness` at the **integrated** game checkout's
`pc/tests/sweep`; the workspace `tools/sweep` files remain read-only here.
Use the lane's already-built EXE, its ACE ISO path, and `_build` for fallback
DLL/cache files. No runner command builds anything.

```powershell
& .\tools\sweep\run-sweep-all.ps1 `
  -Harness .\melee\pc\tests\sweep `
  -Exe .\_build\agents\integration\melee-pc.exe `
  -Runtime .\_build -Iso $env:GW_ISO_ACE `
  -Out .\_build\sweep-all -Subset
```

Remove `-Subset` for the full sweep. The subset is **Fountain, Battlefield,
Final Destination x Fox, Falco**, six separate processes, three checks each.
The runner exports `MELEE_SCENE` for each requested case, `MELEE_PAD_SCRIPT`
to the Lua fixture, `MELEE_TURBO=1`, `MELEE_TURBO_RENDER=8`,
`MELEE_TURBO_DRAWS=1`, `MELEE_SCRIPTS=0`, `MELEE_INPUT=none`, and the
`MELEE_LAB_SWEEP_*` case parameters. All checks run in stated **VS mode**;
full-suite 1P cases run in their real Classic/Adventure modes.

The same Lua fixture also runs all six subset cases in one process when no
`MELEE_LAB_SWEEP_*` variables are set (useful in the lane's combined test):

```powershell
$env:MELEE_SCENE = 'mode=vs;p1=fk:1;p2=fox/cpu0;stage=ext:2;time=0;items=off'
$env:MELEE_PAD_SCRIPT = (Resolve-Path .\melee\pc\tests\sweep\sweep_all.lua).Path
$env:MELEE_SCRIPTS = '0'
$env:MELEE_TURBO = '1'
$env:MELEE_TURBO_RENDER = '8'
$env:MELEE_TURBO_DRAWS = '1'
# From the lane's prepared run directory (DLLs/map already copied):
& .\melee-pc.exe --iso $env:GW_ISO_ACE
```

Require 18 ordered PASS sections and zero FAIL sections, native fatal/panic or
crash artifacts. Apply an external 450-second timeout to this combined form;
use the runner form for per-case isolation and automatic timeout/reporting.

Planning without launching:

```powershell
python -B .\melee\pc\tests\sweep\sweep_all.py --plan --subset
python -B .\melee\pc\tests\sweep\sweep_all.py --plan --catalog-log .\saved-ace-boot.log
```

Full planning deliberately requires saved discovery evidence. Actual full runs
always probe their current disc rather than trusting a possibly stale log.

## Verification and limits

Host-only verification performed:

```text
python -B pc/tests/sweep/test_runner.py
lua pc/tests/sweep/test_lua.lua pc/tests/sweep/sweep_all.lua
luac -p pc/tests/sweep/sweep_all.lua
python -B pc/tests/sweep/sweep_all.py --plan --subset
```

All **9 Python tests passed**. The Lua host checks and both Lua syntax checks
passed; the PowerShell runner also passed parser validation. `git diff --check`
reported no whitespace errors (Git emitted an unrelated warning that the
user-level ignore file was not readable in the sandbox).

The Python tests cover missing/duplicate success markers, crashes overriding
success, timeout/exit classification, discovery completeness, enumerated steps
and fighters, exact issue matching with ASLR, and collapsed failure reports.
The Lua stub executes the actual six-case coroutine and verifies wrong-stage
and stalled-frame rejection plus `gd.quit()`. These are harness tests, not
evidence of in-game correctness. No C changed, so PowerPC C syntax checking
is not applicable. Lua syntax was checked with the installed `luac`.

**Untested:** actual ACE discovery, scene transitions, 1P intro handling,
process timeout/termination against Melee, every game case, rendered results,
and integration with other batch lanes. No claim of a passing game sweep.
The extension catalog follows the port's contiguous appended stage layout;
if a disc changes that layout, review the catalog before calling coverage
complete. Appended stage runs currently check that the loaded internal stage
stays stable, not an independently supplied ext-to-int mapping. The small runs
do not cover every move, costume, random matchup, later 1P subfight, or all
stage/fighter combinations. Mods are intentionally disabled for the ACE sweep.
Crashes with no native fault record cannot have a trustworthy RVA recovered
by this harness alone; their retained logs/exit codes need debugger follow-up.
