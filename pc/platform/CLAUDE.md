# pc/platform: the native side

Native x86 Windows code: the SDK shims the retargeted game calls, and the port's own systems.
`gw.h`'s header comment is the contract: read it before writing a shim. The top-of-file comment in
each `gw_*.c` is that file's design note and is kept current; start there, not in this file.

## Map

| file | owns |
|---|---|
| `shim_*.c` | one SDK library each (GX, OS, PAD, CARD, AX, DVD, AR, VI, libc). `gw_<Name>` = the game's `<Name>` |
| `gw_runtime.c` | boot, the scene launch (`MELEE_SCENE`), the frame loop's hooks |
| `gw_settings.c` | `settings.cfg` beside the exe (`key=value`; env vars win). Game code: `Settings_Int/Str/SetInt/SetStr` |
| `gw_script.c` / `gw_script_pad.c` | Lua 5.4 sandbox, the `gd` table, the console, scripted pads. `gw_script.h` says where it runs |
| `gw_snap.c` | savestates, the LAB's long rewind, SyncTest's compare |
| `gw_rollback.c`, `gw_netplay.c`, `gw_net.c` | rollback and the lobby |
| `gw_mex_*.c` | the m-ex compatibility layer: PPC interpreter, bridge, ftFunction / grFunction / item tables. `gw_mex_bridge.c` is **generated** by the build |
| `gw_mods.c`, `gw_uigen.c` | mod discovery and the m-ex CSS/SSS entries |
| `gw_kit.c`, `gw_overlay.cpp`, `gw_console.cpp` | the script-side kit drawing, the F9 panel, the console UI |
| `gc_adapter.c` | the raw GameCube adapter reader (bypasses SDL); `shim_pad.c` merges it with SDL pads |
| `gw_test*.c`, `gw_tests_core.c`, `gw_net_tests.c` | headless tests (`run.sh --test`) |

## Rules

- **Big-endian game memory.** `gw_r32`/`gw_w32` (and 16, 8, `rf32`) for every game-visible
  scalar. Structs passed by value from the game arrive big-endian too (`gw.h`).
- **Naming the boundary.** Define `gw_X`; game code calls `X`. Game-callable functions take and
  return scalars only; floats cross as bit patterns.
- **Gameplay writes fork the LAB timeline.** Anything that changes game state from a script goes
  through `gs_require_gameplay` / `gs_require_offline` and `gs_rw_branch()` (see `l_set_percent`).
  Writes during a netplay session are refused unless the script is `rollback_safe`, never from the
  console.
- **Rewind exactness.** Native state the simulation reads must be in the per-frame log
  (`gs_log`: pads, voice handles) or the snapshot; the exactness test is `gd.rewind_test`. A
  native helper called from the game must leave no state behind (`mpCheckFloor` clears its own
  bounding flags; that is why `gd.floor_below` is safe).
- **Every new `.c` is hand-added** to `_build/melee_link_objects.rsp` (workspace); a new import
  library to `melee_link_libs.rsp`.
- Log what happens (`gw_log`); rate-limit anything per-frame (`gw_pad_log` rolls its file).
- Windows-only headers: this directory does not compile off Windows. A stub `windows.h`
  (the types and a few functions) plus `extern/aurora/include` gets `clang -fsyntax-only` through
  `shim_pad.c`; it proves syntax, not a build.

## Environment variables

`MELEE_*` switches are listed in `pc/docs/PORT_DEV_QUICKREF.md`. Add new ones there in the same
table when you add them here.
