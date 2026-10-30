# Trace verification (pre-melee_* branches)

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 1260-1304 -->
## melee_chara trace verification (port/pre-melee_chara, 2026-10-24)
`docs/rust-port/trace/pad_falcon_fox_fd.csv` listed no stub or missing `melee_chara` function; the 10 `ported-unverified`
rows now have C oracles (all equal, rows flipped to `ported+verified`): `ftPr_Init_8013C2F8`, `mpColl_8004730C`,
`ftCa_SpecialHi_800E3EAC`, `ftCa_SpecialLw_Coll/Anim`, `ftCa_SpecialLwEndAir_Phys`, `ftFx_SpecialAirNStart_Anim/IASA`,
`ftFx_SpecialN_CreateBlasterShot`, `ftFx_SpecialAirNLoop_Anim`.
- Build: `python rust/melee_chara/gen/turef/tu_pre.py [stem]` (same machinery as `tu_shard89.py`: the game `.c` is `#include`d
  verbatim, shipping contraction, recording stubs for callees, 4000-6000 seeded cases to `tests/data/tu_<stem>.txt`);
  tests are `tests/tu_{ftcaptainspeciallw,ftfoxspecialn,ftpurin,mpcoll}_c.rs`.
- `tu_mpcoll_ref.c` includes `mplib.c` + `mpisland.c` + `mpcoll.c` together and loads a synthetic stage with `mpLibLoad`; it is
  the first oracle of the generated `melee_world::tu::{mplib,mpisland,mpcoll}` (it also exercises mpCollPrev,
  mpColl_80042C58, the whole 80043754 loop, floor / wall / ceiling passes and the joint callbacks). The image is linked
  below 4 GB because those files route pointers through `int`. Trap: a group named `LeftWall` has normal -x (hit while
  moving right), i.e. it is the stage's right-hand wall.
- Write C driver edits with the Edit tool, not a Python string: `\n` inside a Python literal puts a real newline in the C.

## melee_world trace verification (port/pre-melee_world, 2026-10-24)
All 93 `melee_world` rows of `docs/rust-port/trace/pad_falcon_fox_fd.csv` that were stub, missing or ported-unverified are now
`ported+verified`: camera.c (`tu::cmcamera`, `tests/tu_cm.rs`), eflib/efasync/efdata (`tu::efn`, `tests/tu_ef.rs`) and
mplib/mpcoll/mpisland (`tu::{mplib,mpcoll,mpisland}`, `tests/tu_mp.rs` plus melee_chara's `tu_mpcoll_c`) each have a state oracle against the
shipping-contraction C. `Camera_SetQuakeOffset` / `Camera_SetBackgroundColor` of grlib, grlast, ground and grpura now default to
`tu::cmcamera` (they wrote the hand-port `cm::camera::world()` state, a different `game_camera` from the one tu::cmcamera renders).
Rows that are static helpers (`update_bounds`, `cm_script_update`, `fn_801C8EF8_inline`, ...) are verified through their callers' ops.

## Trace status regenerated after the pre-melee_* merges (2026-10-25)
`rust_status`/`rust_crate` of `docs/rust-port/trace/pad_falcon_fox_fd.csv` were re-derived against the current tree with the
`calltrace.py` rules (`index_rust` + `tested_names` + `missing.md`; the function list and counters are unchanged, 2625 rows).
Status is name-level: "verified" = a real-bodied Rust fn plus a `rust/*/tests` file naming it, so it also counts
delegating wrappers; the per-agent notes above are the finer-grained claim.

| crate | ported+verified | ported-unverified | stub | missing | total |
|---|---:|---:|---:|---:|---:|
| melee | 13 | 2 | 0 | 0 | 15 |
| melee_audio | 1 | 0 | 0 | 0 | 1 |
| melee_chara | 88 | 0 | 0 | 0 | 88 |
| melee_fighter | 524 | 137 | 2 | 6 | 669 |
| melee_items | 160 | 1 | 0 | 0 | 161 |
| melee_modes | 447 | 0 | 0 | 0 | 447 |
| melee_runtime | 723 | 41 | 1 | 0 | 765 |
| melee_types | 1 | 0 | 0 | 0 | 1 |
| melee_world | 475 | 3 | 0 | 0 | 478 |
| **total** | **2432** | **184** | **3** | **6** | **2625** |

Before: 1989 verified, 525 unverified, 55 stub, 56 missing (melee_runtime 378/314/39/37, melee_fighter 444/190/16/19). The
remaining 9 stub/missing rows are melee_fighter (8) and melee_runtime (1). Note the earlier melee_runtime report counted 41 unported audio/devcom rows; the CSV
now reads them as ported-unverified because same-name Rust fns with real bodies exist (name-level, not a test claim).

## melee_fighter trace verification (port/pre-melee_fighter, 2026-10-25)
`docs/rust-port/trace/pad_falcon_fox_fd.csv` listed 225 `melee_fighter` rows as stub / missing / ported-unverified. 176 were already
covered by the committed ref/state oracles (fighter, ftanim, ftaction, ftcoll, ftdynamics, ft_081b/0c31/0c88/0c8c/0d31/0d4d, ftparts,
ftCo_09f4, ftCo_Turn, player/stale, ftcommon) and were only never flipped; this pass flipped them and closed most of the rest:
- `tests/tu_ftcpuattack.rs` (ftCo_800B9704), `tu_player_vs_c.rs` (ops 144-146 on `gen/cref/player2_ref.c`; `player_ref.c` / `player.txt`
  stay at 144 ops for the crate-level `player_ref.rs`), `ftleaf_ref.rs` op 50 (mpClearFloorSkip), `Camera_ScriptState` seam of fighter.rs now
  defaults to melee_world's oracle-checked `tu::cmcamera` port.
- Unity oracles (`MTU_UNITY=1`, `gen/cref/tu/*_unity.c`): `tu_plbonus_vs_c.rs` (player.c + plattack.c + plbonus.c + plbonuslib.c + ft_0892.c vs
  `tu::plbonus` / `tu::plbonuslib`), `tu_ftdrawcommon_vs_c.rs` (ftdrawcommon.c + mtx_pc.c; `script_motion_draw_begin` is now a real port),
  `tu_crowdsfx_vs_c.rs`. `gen/mtu/stubs.py` takes several port modules (`a.rs,b.rs,c.rs` -> `m`, `m2`, `m3`), the `=>` ext form and
  `[f32; 4]` return types; `gen_tu.py` treats every #included game `.c` as the TU under `MTU_UNITY`.
- `tu_ftdata_vs_c.rs` (spec from `gen/mtu/mk_ftdata_spec.py`): ft_800852B0, ftData_800855C8, ftData_80085B10, ftData_80085CD8 and the
  pc/geno `GenoDefine_InitKinds / CaptureBase / RestPoseTree / BindFighter` / `ResetDescriptors`, which are now real ports in `tu::ftdata`
  (the platform hooks default to them; the registry queries `Geno_Define*` are `ext` hooks that default to "no define registered").
  Every per-kind table of the TU is filled and hashed (pointer-sized words by pool id). Not covered: the `Geno_DefineThrownAs` stand-in
  legs of ftData_80085B10 (`as != kind` calls ftData_ThrownStandInPrepare -> ftData_8008572C; the oracle returns -1 or the kind).
- `tu_ftparts_class.rs` (JObjInfoInit, IntpJObjInfoInit, PObjInfoInit, ftParts_80073758, IntpJObjLoad, JObjMakePositionMtx against the real
  class.c / mtx.c / mtx_pc.c; parent classes synthetic), `tu_ftparts_pobj.rs` (ftParts_PObjSetupMtx z-scale paths).
- Traps: a Python string with `\n` written through the tool's shell lands as a real newline in the C (use the Edit tool); the oracle C is
  native, so `size_t` is the 4-byte `gc_size_t` while Rust `usize` is 8 (hash `(u32)`); native x86 bitfields are LSB-first, so raw-byte
  readers (`FT_DRAW_FLAG` reads `x21FC_flag.byte`) need the byte set on both sides after the field-wise fill; fused sites of
  `fn_8003F654` / `fn_8003F294` / `script_motion_draw_begin` mutation checks: only the last is discriminated by the generated inputs.

### melee_fighter trace rows, last six (port/pre-melee_fighter)
`ftParts_PObjSetupMtx` (`tests/tu_ftparts_pobj.rs`, 4000 cases; the C reference evaluates the mark arguments in a fixed order: the
MS-ABI argument order is unspecified, so a call with two `rnd()` arguments diverged), `ftParts_8007482C` (op 6 of `tu_ftparts_class`;
`src/tu/ftparts_render.rs` `ext` hooks for HSD_JObjSetDefaultClass / HSD_JObjLoadJoint, stand-ins record the default-class bracket) and
`ftMaterial_800BF260 / 2B8 / 534 / 6BC` (mtu state oracle `tests/tu_ftmaterial_vs_c.rs`, spec `gen/mtu/specs/ftmaterial.json`, 4 ops x 600
cases over the real ftmaterial.c; the TU's callees are recording stubs and `src/tu/ftmaterial.rs` gained an `ext` seam; `ftMaterial_800BF2B8`
is `extern "C-unwind"` so the C `__assert` longjmp has a Rust panic counterpart; `x61A_controller_index` is held to 0..6 because the
C reads `p_ftCommonData->x6D8[idx]` unchecked). Not covered: the lbRefract / LObj / TExp / GX callees themselves (verified in their own TUs).

## Trace status after all six pre-melee_* merges (2026-10-30)
`rust_status`/`rust_crate` of `docs/rust-port/trace/pad_falcon_fox_fd.csv` re-derived with the `calltrace.py` rules (2625 rows). Rows already
`ported+verified` are never downgraded: the name-level rules would drop 12 rows that agents verified with C oracles (static inlines such as
`ftCo_GuardOn_Anim_inline`, `ftDrawCommon_80080E18_surface_impl`, and wrappers without a test naming them). Only the crate of 9 rows changed.
Open: melee_runtime 41 ported-unverified (axdriver, lbaudio_ax, devcom, lobj, hsd_3933, hsd_3A76, particle `ref_INC`), melee_world 3,
melee 2, melee_items 1; stub 1 (`HSD_AudioSFXSetPitchFid`, macro-defined body, actually ported); missing 0.

| crate | ported+verified | ported-unverified | stub | missing | total |
|---|---:|---:|---:|---:|---:|
| melee | 12 | 2 | 0 | 0 | 14 |
| melee_audio | 1 | 0 | 0 | 0 | 1 |
| melee_chara | 86 | 0 | 0 | 0 | 86 |
| melee_fighter | 677 | 0 | 0 | 0 | 677 |
| melee_items | 159 | 1 | 0 | 0 | 160 |
| melee_modes | 447 | 0 | 0 | 0 | 447 |
| melee_runtime | 720 | 41 | 1 | 0 | 762 |
| melee_types | 1 | 0 | 0 | 0 | 1 |
| melee_world | 474 | 3 | 0 | 0 | 477 |
| **total** | **2577** | **47** | **1** | **0** | **2625** |
