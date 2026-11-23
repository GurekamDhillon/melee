# Gamemode enemy lifecycle fix

2026-09-27, `codex/gamemode-fix`, based on `beta/gamemode-wip`.
Code only: **no build, game run, or commit**. Runtime success is not claimed.

## Cause

Read `../../_build/tmp/fail-gamemode.md`. The missing ReDead event matches
`ScriptGame_EnemyDestroyed`: it cleared the tracked item without notifying Lua.
`ScriptGame_EnemyRemove` also retired the handle before destruction. The director
only removed ownership on a defeat hook, so an absent item could block its wave forever.
The exact physical cause of the reported ReDead disappearance remains unverified.

## Changes

- `pc/gameworld/script_game.c`: one clearly marked terminal-lifecycle block after
  `ScriptGame_SpawnEnemy`. Stock `it_8027CE44` defeats, explicit removal, and the
  existing generic item-destruction hooks now share terminal bookkeeping. The
  snapshotted enemy record is marked before notification, suppressing duplicate
  callbacks and a second event when a defeated corpse is later destroyed.
- `pc/platform/gw_script.c`: `on_enemy_removed({kind, handle, reason})`, a `reason`
  field on `on_enemy_defeated`, and `gd.enemy_status(handle)` for owned-handle queries.
  Reasons are `defeated`, `explicit_remove`, and `item_destroyed`. Generic fallout,
  blast-zone removal, timeout and despawn use `item_destroyed`: the shared destructor
  does not expose the finer cause. All shim arguments are scalars; game callers use
  unprefixed symbols. New observable paths log the handle and reason.
- Event dispatch retains its current batch until delivery finishes, so a hook that
  removes another enemy cannot overwrite pending events. Events added by callbacks
  are dispatched on the next frame. The existing bounded queue can still overflow;
  the watchdog covers missed delivery rather than promising an unbounded event log.
- `pc/scripts/lib/gamemode.lua`: removals clear owned handles by default. Each
  `defeat_all` goal can set `count_removed=false`, which produces an explicit error
  on removal, with a handle/reason, rather than granting victory or hanging. Retry
  reconstructs the checkpoint and clears the previous error message.
- Before advancing waves, the watchdog polls each owned handle. Missing handles
  are reconciled as removed (`watchdog_missing`); retained defeated status counts
  as defeat. Live enemies, future waves, targets, survival timers and other goals
  still block completion as appropriate. Unknown/reused slots report removed;
  status is not permanent terminal history. No new host simulation state is added.
- The demo forwards removal events. Its generated `scripts/main.lua` is refreshed.
  Beta's `pending_tp` entry-teleport retry remains intact.

## Merge guidance for codex/enemies2

The game-side change is confined to the marked block from
`script_enemy_terminal` through `ScriptGame_EnemyDestroyed`. No edits to enemy
kind tables, preload, spawn adapters, pool size/struct layout, or item source files.
The existing integer `defeated` now means 0 live, 1 defeated, 2 removed; initialization
stays zero. Keep that terminal bookkeeping if enemies2 changes lifecycle functions.
The new native removal hook derives its kind bound from `gs_enemy_names`; native
API registration, event dispatch and payload-test additions are separate small hunks.
The existing defeat hook's kind bound remains enemies2's responsibility when adding kinds.

## Tests and source verification

- `lua pc/scripts/tests/gamemode_test.lua`: PASS. Added coverage for live enemies
  blocking, unrelated/duplicate removal events, default removal clearance, missing
  events in earlier/final waves, retained defeats under strict policy, strict
  removal errors, and checkpoint retry. Existing two-area, target/timer, save/load,
  capacity failure and boss tests still pass. Before implementation the new test
  failed at the missing `enemy_removed` method.
- `luac -p` on library, host test, demo source, generated demo and integration test:
  PASS. Also concatenated the real library/demo/test via `bundle.py --test` into a
  temporary directory, checked bundle freshness, and syntax-checked that combined
  entry. It was **not executed**.
- Normal `bundle.py --check`: PASS. Bundling only concatenates source here; no
  assets were exported or copied and no game build was invoked.
- `git diff --check`: PASS (Git reports the repository's LF-to-CRLF warnings).
- Extended native `script_stage_events` payload tests for defeated/removal reasons:
  **not run**. No C compilation/link, EXE string check, native suite or visual check.

## Two-area integration test to run later

`pc/tests/gamemode_demo.lua` did not exist in this checkout. Added it using beta's
playback sequence as the basis, with assertions on the actual director and terminal
events. `bundle.py --test` appends it after the real demo in the same Lua chunk:

```powershell
python pc/scripts/examples/gamemode_demo/bundle.py --test pc/tests/gamemode_demo.lua --output scratch/gamemode_demo_test --kit-models "PATH/TO/KIT/models"
```

Install that test mod instead of the normal demo. Use offline VS, Fox P1, idle human
P2, Final Destination, time=0. This is a mod integration fixture, not a standalone
pad script. It attacks the hall wave, checks one terminal event/reason per enemy,
verifies the unlocked door enters the gallery, breaks both targets, waits for the
survival objective and takes the final exit. It requires both areas marked cleared
and phase `complete`; item disappearance alone cannot pass. It logs `GMD PASS` or
`GMD FAIL` and calls `gd.quit()` on completion, handled failure or the tick deadline.
The host tests separately exercise deliberately lost events and strict policy.

Still required in an authorized runtime lane: run the fixture and native tests,
confirm natural ReDead fallout and explicit removal each deliver their terminal
event once, and inspect collision/visuals if the original disappearance needs a
separate terrain fix. This change addresses lifecycle/goal completion, not terrain.
