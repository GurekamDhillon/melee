# Function identity in the C-oracle tests

The TU oracles hash function pointers as ids: a state-table row (`ItemStateTable.animated / physics_updated /
collided`), a callback the code under test installed (`take_dmg_cb`, `accessory4_cb`, ...), an `UNK_T` slot holding a
function. The C driver numbers its functions in registration order; the Rust harness registers the same list
(`freg(m::f as usize)`) and names a pointer by its index. Three things break the plain address match in an optimized
Rust build. Two are handled, one is open.

## 1. Cross-crate copies (handled: `fnsym`)

rustc inlines small leaf functions across crates and gives the test crate its own copy, so `m::f as usize` taken in
the test and the pointer the library stored are different addresses of the same function (the symbol table shows `f`
and `f.llvm.<n>`). `rust/melee_core/test_support/fnsym.rs` (included with `#[path]`, `backtrace` dev-dependency) names
an address by its symbol: demangled without the hash and the `.llvm.<n>` suffix. `backtrace::resolve` takes a return
address (it looks up `addr - 1`), so fnsym asks for `addr + 1`; with debuginfo the frames inlined at the entry come
first, so the function is the last symbol. `fnsym::find` (the harness `fi` / `pi` lookups) matches by address, else by
symbol; `fnsym::same` replaces the direct `p as usize == m::f as usize` callback compares.

## 2. Function merging (handled: `oracle_tests.sh`)

LLVM's MergeFunctions (on in optimized rustc builds) folds body-identical functions into one: distinct C callbacks
with one body (the empty `UnkMotion*_Phys` procs, one-call wrappers) end up at one address, and the folded copy keeps no
symbol of its own (`kinds_itbat`: `it_803F57B0[0].physics_updated` resolves to `itLikelike_UnkMotion9_Phys`). No
lookup can tell them apart, so the oracle tests run with merging off:

    bash tools/rust-port/oracle_tests.sh -p melee_items --no-fail-fast

(`RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zmerge-functions=disabled`, `RUST_TEST_THREADS=1` since the harnesses share `static
 mut` state across a binary's tests, own target dir `build/cargo-oracle`, overridable with
`ORACLE_TARGET_DIR`). `fnsym::require_unmerged` runs on the first lookup and panics with that command when two
body-identical probe functions share an address, so a plain `cargo test` cannot silently compare merged ids.
Measured on Linux (melee_items, 198 test binaries): 44 pass merged, 67 with merging off.

The shipping build keeps merging (the release / perf profiles are unchanged). Game code that compares function
pointers is therefore audited separately (`fn-pointer audit` below).

## 3. Adapters in the state tables (handled: `pred_adapter`)

A state-table predicate slot holds an `int`-returning shim of the C `bool` callback (`pred_b!`, melee_items
tu/kinds_inl.rs). The shim is the generic `pred_adapter::<T>`, `T` a block-local type named after the callback, so its
symbol is `pred_adapter::<..::itBat_UnkMotion0_Anim>`; the TUs' own `pred_i32!` shims are named `<callback>_p`.
`fnsym::find` matches either to the registered callback by its last path segment when no name matches exactly.
Measured on Linux (melee_items through oracle_tests.sh): 67 test binaries pass before, 127 after; kinds_* 0 -> 60, and
none of the 36 still failing stops first at a state-table op.

## Fn-pointer audit (game code)

The C compares a stored callback with a named function in two places: `ftseakspecialn.c:59-60`
(`death2_cb / take_dmg_cb == ftSk_Init_80110198`) and `ftkirbyspecials.c:69` (`death2_cb != ftKb_Init_800EE74C`).
- Kirby: every store and the compare use `chars::cb::ftkirby__ftKb_Init_800EE74C`. Same pointer.
- Sheik: the port had two wrappers of `ftSk_Init_80110198`: `tu::ftseak::cb_ftSk_Init_80110198` (stored by the
  ftSk_SpecialN_* paths and ftseakspecials.rs set_chain_damage_callbacks) and `chars::cb::ftseak__ftSk_Init_80110198`
  (stored by ftSk_SpecialS_80111440 / 801114E4 / 80111D54), and the compare (ftSk_SpecialN_80111FBC) names the first. Their addresses differ in the
  shipping binary (separate codegen units: MergeFunctions does not fold them), so after the SpecialS stores the C's
  test was true and the port's false. Fixed: the chars::cb name re-exports the tu fn.
- mex_bridge.rs (`gobj_fn`, the callbacks m-ex guests store) gave every name a local shim, so a guest's
  `ftKb_Init_800EE74C` / `ftSk_Init_80110198` / `cb_ftSk_Init_80110198` never equalled the game's. gen_mex_bridge.py
  now hands out the pointer the game stores: melee_chara's CALLBACKS wrapper (128 names), else a TU's `cb_X` entry
  (98), else the port itself when it is an `unsafe extern "C" fn(*mut HSD_GObj)` (1574); 3404 keep a shim (plain
  Rust fns with no single stored pointer; none is compared by game code).
- Port-only compares (not in the C): geno_lab_mode.rs:353 (`phys_cb == fly_phys`), geno_tests.rs:4376 / 4403.
