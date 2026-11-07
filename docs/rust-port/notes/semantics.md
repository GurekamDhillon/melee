# Console arithmetic semantics: policing without a type rewrite (branch port/semantics)

Plain f32 `+ - * /` already are `fadds/fsubs/fmuls/fdivs` (Rust never contracts). What is left is keeping the risky
constructs out of game code and deciding the remaining fusion / rounding / order sites mechanically. Companion notes:
fp-audit.md (fusion rule, sign of zero), int-audit.md (div/rem/shift/fctiwz helpers), dolphin-oracle.md.

## 1. slw / srw / sraw: the low 6 bits of rB

Dolphin `Interpreter::slwx/srwx/srawx` (Interpreter_Integer.cpp): `rB & 0x20` selects the shifted-out result (0, or the
sign fill for sraw), `rB & 0x1f` is the shift. So 32..63 give 0 / sign fill, **64 shifts by 0**, 65 by 1, 96 gives 0, and a
negative amount is its two's-complement low 6 bits (-1 = 63 -> 0, -33 = 31, -64 = 0 -> unshifted).

Before: `melee_core::ppc::Ppc::pshl/pshr` gave 0 / sign fill for every amount >= 32 (and for every negative amount, via
`n as u64`); gwtool's `makeArithmeticSafe` selected on `amount >=u 32`, the same. Now both use the 6-bit rule for 32-bit
(and promoted 8/16-bit) operands. 64-bit operands are not slw/srw (MWCC calls its runtime shift helpers) and keep
"amount >= 64 gives 0 / sign fill". Tests: `ppc::tests::shift_six_bit_amount`. No golden changes: no recorded run shifts by
64 or more or by a negative amount (gate, coverage and slpcheck unchanged).

gwtool: rebuilt privately as `_build/agents/semantics/gwtool/gwtool.exe` from this branch's gwtool.cpp (it also carries the
int audit's NaN -> 0x80000000 and `INT_MIN % -1` changes). Checked on a two-function TU: `shl` / `ashr` now go through
`and 32` / `and 31` selects. The shared `_build/gwtool/gwtool.exe` was not replaced (other agents build with it); swap it in
when no C build is running, then rebuild every game TU (build_gwtool.bat's note).

## 2. The lint (tools/rust-port/semantics_lint.py, run by gate.sh)

Crates: melee_core, melee_types, melee_runtime, melee_world, melee_fighter, melee_chara, melee_items, melee_modes; every
file except raw/, c2r/, tests/, the helpers (melee_core ppc.rs, fp.rs) and the host files in
`tools/rust-port/semantics_allow.tsv`. A single line can carry `// semantics: allow(<reason>)` (or the line above it).

| rule | what | how found | fix |
|---|---|---|---|
| M1 | `f32/f64::mul_add`, host libm (sqrt, sin, cos, tan, sin_cos, asin, acos, atan, atan2, powf, powi, exp, exp2, ln, log*, hypot, cbrt, sinh/cosh/tanh), `round`, `rem_euclid`, float `min/max/clamp` | clippy `disallowed_methods` (rust/clippy.toml), nightly clippy, JSON output filtered by path | `fp::fmadds` (f32), `fp::Fmadd::fmadd` / `fp::fmadd` (f64), the MSL / lbtrigf ports, the C comparison |
| I1 | integer `/` `%` with a non-literal (or 0 / -1) divisor | MIR (`-Zunpretty=mir`), int_audit.mir_sites | `Ppc::pdiv/prem` |
| I2 | `<<` `>>` with a non-literal amount | MIR | `Ppc::pshl/pshr` |
| I3 | any float -> int `as` cast, u32/u64/usize included | MIR | `ppc::fctiwz[_f32]`, `fcvt_u32/u64/i64/usize` (same values as `as`; explicit) |

No clippy lint exists for I1-I3 with the literal/non-literal split (`integer_division`, `cast_possible_truncation` flag
every site), so they come from MIR, which has exact spans and folded literals. Const / static item bodies are skipped
(compile-time). `--list` prints everything, `--fix` rewrites I1-I3 with int_audit's fixer. A stamp (hash of the scoped
sources, clippy.toml, the allow-list and the tool) skips an unchanged tree; a full pass takes about 5 minutes. `SEMLINT=0`
skips it in the gate.

### Fixes made (each equal or console-correct)

- 123 `f64::mul_add` -> `.fmadd(` (the same `std::fma`; Dolphin's double `fmadd` is `std::fma(a, c, b)`): equal by
  definition.
- `Item` hitbox damage (item.rs, `it_8026B???` staling): `f32::mul_add(base, xC6C, 0.99)` -> `fp::fmadds`. mul_add rounds the
  exact value once to f32; Dolphin `fmaddsx` takes `fma` in f64 and then rounds to single (the C build's lowering is the same).
  They differ only at a double-rounding tie; fmadds is the console value.
- float -> u32/u64/i64/usize `as` casts -> `fcvt_*`: equal (the helper is the same `as`).
- `(f as u32) as u8/u16` where the C is a plain assignment of a float to a u8/u16 (fog.c colours / fog-adj width, tev.c
  ChanUpdateFunc, state.c HSD_SetMaterialColor): now `cabi::f2u8/f2u16` (fctiwz then truncate), which is what MWCC and
  gwtool emit; differs from the old code only for negative or >= 2^31 values (render state).
- Variable shifts / div / rem left in menu and PC-mod code (mnmain, mninfo*, geno_game, script_game...): helpers.

### Allowed (semantics_allow.tsv and inline)

Host: gcdecode, hash, gc, modhook, slippi_online (net), pose / pose_capture (render interpolation), melee_types gen.rs and
neutral_hash, melee_modes oracle.rs, tu/support.rs (u128 bitfields). Known console differences kept, both builds alike:
axdriver `powf` (MSL pow not ported; audio pitch only) and bytecode op 0x13 `logf` (MSL log not ported). Inline: mobj.rs
`kf` clamp (render-only TEV constant), granime const items, one geno_game literal shift.

## 3. Float trace matching (tools/rust-port/fptrace)

Both sides log every float operation of one call by value (f64 bit patterns; an f32 widened exactly) and
`match.py` compares them.

- Console side: a private Dolphin copy with `tools/rust-port/fptrace/dolphin_dump.patch` applied on top of the oracle patch (`NHV1Oracle.cpp` `Probe()`; not in the shared oracle).
  `MELEE_DOLPHIN_NHV1_DUMP=<prefix>` plus either `_DUMP_FRAME`/`_DUMP_COUNT` or `_DUMP_AT=pc:frame:call,...`, where
  call is the hit index of that pc within that frame. Probe one address per run: the JIT leaves `pc` stale at an HLE
  hook. A dump (`GDMP` v2) holds the registers, GQRs, MEM1 and ARAM. `gekko.py` (a Gekko interpreter from
  Dolphin's semantics: ForceSingle, Force25Bit, `std::fma` madd, estimate tables, quantized loads and stores)
  runs the call to its return and logs each float op with its pc and function. Hardware registers and the
  locked cache are scratch bytes. `ARQPostRequest` is done at once from the dumped ARAM, then the callback runs.
- Rust side: `#[cfg_attr(fptrace, melee_fptrace::trace)]` on a function. In a `RUSTFLAGS=--cfg fptrace` build, the
  proc macro logs its binary ops, compound assigns and float casts, and `fp` helpers log while a trace is on.
  Env vars: `MELEE_FPTRACE_FN=a,b`, `_FRAME`, `_NTH`, `_MINOPS`, and `_OUT` or `_DIR`+`_PER`
  (`<name>.f<frame>.c<call>.tsv`). The 71 fp-audit functions carry the attribute (in the normal build it does nothing).
- `match.py rust.tsv ppc.tsv`: classes are ok, neg, fusion+ (Rust fused, console did mul then add), fusion-, frsp,
  force25, prec and miss. `sweep.sh <slp> <dir>` does both sides for every annotated function the replay reaches.
- Dumps hold console RAM: they stay under `_build/agents/<lane>` and are deleted.

Result over 7 replays (every legal stage; Fox, Falco, Marth, Sheik, Peach, Jigglypuff, Falcon, Ganon, Mario, Luigi,
Samus; first 2-3 calls with at least 4 float ops):

| function | calls | result |
|---|---|---|
| ftColl_8007A06C, ftCo_SquatRv_CheckInput (+ its callees, ~3k-7k PPC ops), TObjSetupMtx, mpLib_80055E9C, ftCo_800D0CBC, it_8027781C, HSD_PadClampCheck3 | 50 | all equal |
| HSD_SisLib_803A84BC | 2 | equal up to sign (`neg`: fnmsubs vs fmsubs + negate) |
| ftCo_800C2600 | 2 | **fixed**: `d2 = z*z + (x*x + y*y)` is fmuls/fadds in the DOL (800C28A8); Rust fused it, C now uses `GW_NOFUSE_SQ` |
| psDispParticles | 12 | the input matrices already differ (render-time camera state), not an arithmetic difference; render only |

The other fp-audit functions are not reached by these replays: non-legal stages, menus, Ness/Yoshi specials, and
Bury/Song/Bind. The static "DOL fused vs predicted" counts in fp-audit.md were wrong for ftColl_8007A06C and
SquatRv: the traced values are equal.

## 4. Evaluation order (tools/rust-port/evalorder_audit.py)

`scan` reads the clang AST JSON of the game C (`-UTARGET_PC`) and lists expressions where two calls (or a call
and a read of what it writes) are unsequenced. A purity fixed point marks the sites that only call pure
functions (`-pure`). `dol` reads the `bl` order of each site from the DOL. MWCC often evaluates arguments right
to left, while clang and Rust go left to right.

Retail order differed and is fixed in Rust and in C (`TARGET_PC`):

- efasync case 0x42B: the two `va_arg` are drawn right to left (DOL 0x800659A0).
- itSeakNeedleThrown_SetupDrop: `HSD_Randi(2)` (the sign) is drawn before `HSD_Randi(8)` (0x802B0180/0x802B019C).
- `grZakoGenerator_801CAEB0(Ground_801C5840(), Ground_801C5940())`: both arguments draw RNG and 5940 runs first.
  Affected stages: gricemt, grpushon, grshrineroute, grzebesroute, grbigblueroute, grkinokoroute.

There are 43 impure call-call sites; 99 call-read-shared and 196 call-read sites were reviewed by callee
(getters, out-param helpers) and are benign. `int_audit.py`'s DOL scan now uses capstone skipdata (jump tables
in .text stopped it before).
