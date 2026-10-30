# Float contraction (FMA)

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 86-98 -->
## Float contraction (group 1, 2026-10-21; corrects inventory.md section 3)
- The C build compiles with clang's default `-ffp-contract=on`: each `a*b+c` / `a*b-c` / `c-a*b`
  inside one expression is `llvm.fmuladd`, which gwtool lowers to `fmadds` (f32: f64 multiply,
  f64 add, then f32) or a true `fma` (f64). A port must fuse exactly those sites: LHS multiply
  first, compound `+=` included (`docs/rust-port/group-1 runtime, math, HSD (core + render).md`).
  c2rust ran with `-ffp-contract=off`, so raw output has lost every fusion; check the C IR.
- gwtool also defines float->int casts (saturate to i32, then truncate for narrow types), div/rem
  by zero and oversize shifts; uninitialised locals are 0. Helpers: `melee_runtime::cabi`.
- C oracle: `rust/melee_runtime/oracle/build.sh` (native IR + gwtool's float rewrites redone in
  `lower.py`) -> `tests/data/oracle_vectors.txt`, checked bit for bit by `tests/oracle.rs`.
- Ported and oracle-equal: gekko_fp.c, math_ppc.h sqrtf, mtx_pc.c, mtx44.c, baselib/random.c,
  lb/lbtrigf.c, lb/lbvector.c (not WorldToScreen). The rest of group 1 (HSD objects) is not ported.


<!-- RUST_PORT.md lines 225-250 -->
## Step 2: FMA contraction, groups 2/4/5 re-checked (done)
- Shipping build, verified from `_build/masstest/pipe_win.sh` (what `build_objects.py` runs per game
  TU): `_toolchains/llvm` clang 23.1.1, `--target=ppc32-none-eabi -O2 -Xclang -disable-llvm-passes`,
  no `-ffp-contract` flag, so clang's default `on`: every `a*b+c` within one expression is
  `llvm.fmuladd`. gwtool runs with `GW_GWTOOL_FLAGS` empty, so `--no-exact-fmuladd` is off and
  `lowerFMulAdd` turns every f32 site into fmadds (f64 mul+add, one f32 rounding), f64 into `fma`.
- Tool: `python tools/rust-port/fma_sites.py [-v] [--func RE] <tu.c>...` runs that exact front end
  and prints per-function fmuladd sites. Use it before porting any float code.
- Oracles rebuilt with contraction on + `melee_runtime/oracle/lower.py`:
  `melee_world/cref/gen_mplib_lines_ref.sh` (15 f32 sites lowered), `melee_chara/gen/run_ref.sh`
  (6 f64 fma), `melee_items/tests/c_oracle/gen_expected.sh` (0 sites). All three outputs came out
  bit-identical to the uncontracted ones: the existing inputs do not discriminate.
- Ports fixed to fuse where the shipping IR fuses:
  `melee_world::mp::lines::check_link` (`SQ(dx)+SQ(dy) < 4.0` in mpLineGetNext/Prev and the
  CheckInline variants: `fmadds(dx, dx, dy*dy)`); `melee_chara::pure::my_sqrtf` (all 7 copies:
  `(-(g*g)).mul_add(x, 3.0)`, true f64 fma); `melee_fighter::ftcommon::calc_hitlag`
  (`ftCommon_CalcHitlag`: `fmadds(dmg, x198, x19C)`; group 3, found by running fma_sites.py over
  its ported functions). Group 5's 9 ported item functions have 0 sites (checked).
- Not proven: since the oracle rows did not change, no test currently fails on the unfused form
  of those three sites; they are right by the IR, not by a discriminating vector.
  A 20M-input Rust experiment found no f32 result difference for my_sqrtf fused vs unfused.
- Still uncontracted: group 0's `gen/nh_oracle.py` (-ffp-contract=off). (Group 3's `gen/cref/run.py`
  was switched to contraction on + lower.py in group 3 port round 1; nh/stale/ftcommon unchanged.)
  NHV1 has no float arithmetic; group 3's ftcommon has 12 fused sites in ftcommon.c functions
  (`fma_sites.py src/melee/ft/ftcommon.c`), only CalcHitlag among the ported ones.
