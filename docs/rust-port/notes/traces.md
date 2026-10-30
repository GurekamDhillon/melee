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
