# arena-hooks

Code-only batch handoff. No build, game run, or commit was performed.

## Changes

- Added `gd.stage_set_origin(x, y, {frames=N})`: immediate by default; a nonnegative integer `frames` requests a transition lasting that many completed logic ticks. Uses the remaining-distance/remaining-ticks rule from `grKinokoRoute_80207C88`, landing exactly on the target.
- Added `gd.stage_set_camera_bounds(left, right, top, bottom)` and `gd.stage_set_blast_bounds(left, right, top, bottom)`. **Inputs are relative to the origin**, matching `Ground_801C39C0/801C3BB4`. They change the normal adaptive camera and KO limits, not terrain or fighter position.
- Added `gd.stage_restore_bounds()`: restores origin and both rectangles captured before the first bounds/origin write, cancels a pending transition, and releases bounds ownership. Repeated restore is harmless. The first write captures and owns the entire origin/camera/blast tuple; other scripts cannot edit it until released.
- Added `gd.stage_bounds()`: `{origin={x,y}, camera={left,right,top,bottom}, blast={left,right,top,bottom}, frames}`. Rectangles are **effective world coordinates**; `frames` is the remaining transition duration. Returns nil without an active match.
- Added `gd.stage_collision_groups()`: ascending list of `{id, enabled}`, using zero-based retail collision-joint IDs. Excludes the spare joints reserved for Lua-created geometry; returns an empty list outside a match.
- Added `gd.stage_collision_group(id, enabled)`: strict boolean, validates an existing retail group, then calls `mpJointListAdd` / `mpLib_80057BC0`, as `grKinokoRoute_8020836C` does. This is an immediate collision toggle, not a visual toggle or persistent override of a stage's own later terrain changes. Re-enabling uses the retail primitive's semantics of enabling the group's lines.
- All writes require an active offline match and a manifest-backed gameplay script, reject netplay even with `rollback_safe`, fork the LAB timeline, and log `script arena-hooks:` commands. Successful writes return true. Coordinates must be finite and within +/-100000; rectangles must have positive width/height.
- Owners, saved originals, transition state, and collision restoration bookkeeping live in `script_game.c`'s existing snapshotted game BSS. Bounds use one owner; each collision group has its own owner. Unload restores that script's bounds and touched groups (including original per-line enable flags). Scene teardown/load clears ownership. `stage_restore_bounds()` does not restore collision groups; toggle those explicitly or unload the script.

Implementation is isolated in `pc/gameworld/script_arena.inc` and `pc/platform/gw_script_arena.inc`. Shared-file edits are labeled blocks in `script_game.c` and `gw_script.c`, plus the `TARGET_PC` accessor override in `src/melee/gr/stage.c`. The latter keeps camera limits, camera origin/derived Y helpers, and fighter KO getters consistent even when a scrolling stage rewrites `stage_info` mid-frame. No new translation units or build-list changes are needed.

## Self-checking test

`pc/tests/arena-hooks.lua` logs `TEST arena-hooks section <n>: PASS|FAIL <detail>` for each check and ends with `gd.quit()`. A host-tick watchdog logs FAIL and quits if a Lua error or stalled setup prevents completion. It claims both pads as neutral, with no human input.

Checks: LAB mode, readback, malformed arguments, group enumeration/toggling/restoration, Battlefield centre lock, 60-tick origin progression and shared camera/blast offsets, saving/loading a partially completed transition and changed collision group, fighter alive inside the arena, KO at a point beyond the narrowed bottom zone but inside the original Battlefield zone, restoring both rectangles/origin, and idempotent restore. Save/load callbacks capture the exact boundary state; Lua coroutine variables are deliberately not expected to rewind.

Batch lane launch after its one combined build, from the workspace root in Git Bash (point `GW_MELEE` at the integrated checkout):

```bash
MELEE_SCRIPTS=0 \
MELEE_SCENE='mode=lab;p1=fox;p2=marth;stage=bf;time=0' \
MELEE_SCRIPT="$GW_MELEE/pc/tests/arena-hooks.lua" \
MELEE_TURBO=1 MELEE_FPS=u \
bash tools/port/run.sh arena-hooks --iso "$GW_ISO_VANILLA"
```

Use an absolute Windows-style path for `GW_MELEE` (for example, `pwd -W` in the checkout). The test supplies its own scripted input via `gd.input`; no separate pad file is needed. This is a scene launch, **not** `run.sh --test`, which runs the native headless unit suite instead. If the combined runner already supplies `MELEE_SCRIPT`, append this path with `;` rather than replacing its other scripts, and coordinate the final quit with that runner. Standalone this test always quits itself.

## Validation and untested work

- `luac -p pc/tests/arena-hooks.lua`: passed.
- `git diff --check`: passed (only local LF/CRLF conversion notices).
- Attempted game-side PowerPC syntax check for `script_game.c` and `stage.c` using the workspace compiler with `-fsyntax-only -w -DTARGET_PC --target=powerpc-unknown-eabi -nostdinc` and the repository include paths. The sandbox rejected execution of `../../_toolchains/llvm/bin/clang.exe` with **permission denied**, before compilation. C syntax is therefore **not verified**; native `gw_script.c` was not compiled either.
- No link, retarget/bridge audit, EXE-string inspection, game run, or observed runtime PASS lines. The supplied test has not been executed against the game. Runtime transition timing, savestate/rewind parity, KO behavior, collision behavior, unload/scene cleanup, multiplayer ownership, and online rejection still require the batch lane's verification. Scrolling Adventure stages and their native terrain updates need separate coverage beyond Battlefield.
- As with existing Lua gameplay APIs, Lua-side counters are not snapshotted. The engine-owned transition runs during LAB resimulation, but a script must arrange its own state reconstruction if it makes additional time-dependent writes after a load.
