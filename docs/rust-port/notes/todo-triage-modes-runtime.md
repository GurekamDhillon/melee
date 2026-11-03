# todo!() triage: melee_modes and melee_runtime (task #38, modes/runtime half)

Tool: `python tools/rust-port/link/todo_triage.py .` (counts on stderr, non-hooked sites on stdout).

Counts (rust-port 5db4e68d6 -> this branch): melee_modes 89 -> 89, melee_runtime 326 -> 324.

## melee_modes (89)

- 87 are `ext_fns!` defaults whose hook is installed at boot (melee_link `gen_table.rs`, `melee_engine/src/{engine,seams_gen,scenario}.rs`).
  They never run in the engine. Most were turned into todo! on purpose by `tools/rust-port/dedupe/r4_todo.py`
  (the canonical function lives in another crate). Several are installed as no-ops/zeros (`db_Setup`, `db_RunEveryFrame`,
  `Ui_RetailPause`, `Replay_*`, `Netplay_*`, `Mex_*`, `Geno_*`, `Script_OnePScripted`): these are PC-port extensions or
  debug code, and the no-op is what the C does in a vanilla retail run (no mods, no netplay, `DbLevel == Master`).
- `gm_1601::ext::hsd_80392528` (not hooked): reached only from `gm_80165388`, the develop-mode stress test, which
  `gm_801A4970` (gmscene.c) calls under `DbLevel >= DbLKind_DebugRom`. Unreachable in retail. Left as is.
- `mex_tests.rs` `todo_ext!` (1 macro site, 18 functions): the pc/tests m-ex test harness; nothing outside tests calls it.

## melee_runtime (324)

None is reachable from a retail run:

- `db*.rs` (dballoc, dbanim, dbbonus, dbcamera, dbcpu, dbeffect, dberror, dbinit, dbitem, dbscreenshot, dbsound; 249):
  the `ext::Ext` stub impls of each TU's Host trait. The engine never instantiates them: only the tests
  (`tests/tu_db*.rs`) use these modules. The game's item-side debug queries go through `melee_items::tu::dbitem`, and
  `dbsound` is only read for its `db_804D4AF8` data. In the C, every db entry point is gated on `DbLevel >= DebugRom`
  (or `== Develop`), and `gmMain_8015FDA4` sets `DbLevel = Master` unless the disc has `/develop.ini` (retail has none).
- `hsd_3915/3924/392C/3933/393C/397E.rs` (60): Host-trait stub impls (GX debug drawing, MCC/FIO host I/O, the
  profiler). Production uses `hsd_3915::NativeHost` (`native_80391A04`) and the free glyph functions, never `Ext`. The
  others have no non-test callers.
- `hsd_3982.rs` (5): `ext` stubs of `hsd_80398310`'s debug-perf callees; only the develop-mode stress test above reaches them.
- `jobj.rs` `ext_native` (5): the JObjExt/RObjHost adapters for `Vec<*mut HSD_JObj>`; the only production use is
  `native_ik` -> `resolve_ik_joint1/2`, which never calls the host. `splArcLengthPoint` is now linked to `spline_world`.
- `wobj.rs` `native_spline_point`: now calls `spline_world::splArcLengthPoint` (was todo!; the hook was already installed).
- `leak.rs` (4): `HSD_Leak_80387DF8` is called only from the gmscene debug dump (DbLevel-gated).
- `card.rs` (1 macro site): all 13 slots are set by melee_link/seams_gen.
