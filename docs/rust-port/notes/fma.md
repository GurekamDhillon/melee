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

## Round 3: console contraction checked against .slp replays (2026-11-05, port/slp-fma)

`melee slpcheck` compares our fighter state with the console's own Post-Frame record, so it can tell which of two roundings
the console used. Method: for the first divergent field of each file, find the function that computes it, change the site in
Rust only (experiment), re-run the corpus, keep a change only if first divergences move later and none moves earlier; then make
the C build do the same (shared correctness fix, notes in the C source) and re-record the goldens. The C build stays the oracle:
`melee_runtime` oracle vectors, the four gate goldens and the coverage goldens were re-recorded from the changed C build.

Sites changed (C file and Rust file both):

| site | was (clang contraction / C reference) | now (what the console does) | evidence |
|---|---|---|---|
| `C_MTXLookAt` (pc/gameworld/mtx_pc.c; melee_runtime/src/mtx.rs `look_at`) | the three `-(z*vz + (x*vx + y*vy))` translation terms fused | not fused: the Dolphin SDK's C is built with MWCC `-fp_contract off` (configure.py `DolphinLib`). C: `#pragma STDC FP_CONTRACT OFF` around the function | +6 frame-exact files; the star-KO fighter (DeadUpFall, action 6) is placed by the inverse view matrix (`ftDrawCommon_80080E18`), 20 files had their first divergence there |
| `PSMTXInverse` (same two files) | C_MTXInverse formula, exact `1/det` | the retail paired-single dataflow: cofactors ps_mul + ps_msub, det ps_mul + 2 ps_madd, `1/det` = `fres` + one Newton step `-(det*(r*r) - 2r)`, translation ps_mul + ps_madd + ps_nmadd | first divergence later in 2 files alone, part of the LookAt gain |
| `PSVECMag`, `PSVECDotProduct`, `PSVECCrossProduct` (same files) | C reference grouping with clang fusion; `sqrtf` for Mag | retail paired-single: Mag = `sqsum * (frsqrte + one Newton step)` (as PSVECNormalize already did), Dot = `fma(x1,x2,y1*y2) + z1*z2`, Cross y/z negate an `ps_msub` whose rounded product is the other one | 8 files later, 1 file 1 frame earlier (later again with LookAt) |
| `ftCo_800925A4` shield decay (src/melee/ft/kinds/ftCommon/ftCo_Guard.c under TARGET_PC; melee_fighter/src/tu/ftCo_Guard.rs) | `shield_health -= x278 * inner` fused into one fnmsubs by clang | the product rounded first, then subtracted (the inner `ls*(x2F0-x2EC)+x2EC` stays fused) | 5 shield-field files: 2 became frame-exact, 3 diverge thousands of frames later |

Checked and left as they are (unfusing made things worse or changed nothing): every fused site of camera.rs (per function: 6
functions get worse unfused, the rest no change); every fmadds site of tu/fighter.rs, ftCo_Damage.rs, ftcoll.rs, mpcoll.rs, mplib.rs (per
function or per line); file-level unfusing of ~60 fighter/world/runtime files (every group that changed got worse). The f64 `mul_add`
sites in mplib/mpcoll are bit-identical either way (a product of two float-valued doubles is exact). Fusing the aerial-drift
`lsx * air_drift_stick_mul + accel_flat` across its two statements (an MWCC-style cross-statement fusion) made 43 files worse:
the console did not fuse it. `PSMTXRotAxisRad` in the retail dataflow changed nothing in the corpus (camera roll is 0), so it was not kept.
melee-unlocked's census: its `MU_NO_CONTRACT` on `C_MTXLookAt` agrees with the first row; its `MU_P(SQ(dx)) + MU_P(SQ(dy))`
distance sites in mplib and camera are not supported by the replays (unfusing them changed nothing or made things worse).

Cost: the changed routines are 1-15 ns per call slower or faster (A/B microbench, 200k random inputs: inverse +6 ns, Mag -2 ns,
Cross+Dot +1 ns, LookAt +6 to 15 ns); `golden --slp` on the benchmark replay runs at about 0.20 ms per tick (fast profile).
