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

(`RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zmerge-functions=disabled`, own target dir `build/cargo-oracle`, overridable with
`ORACLE_TARGET_DIR`). `fnsym::require_unmerged` runs on the first lookup and panics with that command when two
body-identical probe functions share an address, so a plain `cargo test` cannot silently compare merged ids.
Measured on Linux (melee_items, 198 test binaries): 44 pass merged, 67 with merging off.

The shipping build keeps merging (the release / perf profiles are unchanged). Game code that compares function
pointers is therefore audited separately (`fn-pointer audit` below).

## 3. Adapters in the state tables (open)

`pred_b!` (melee_items tu/kinds_inl.rs) stores a fresh adapter `w` (the C bool predicate as an `i32` one) in a table
row, not the C function: the adapter's symbol is `<table>::w` for every row, so neither the address nor the symbol
names the C function the row holds. 58 of the 96 `kinds_*` item tests still fail first at a state-table op for this
reason (with merging off). Naming the adapter needs either the adapter to carry its target (a library change) or the
harness to read the call target out of its code.

## Fn-pointer audit (game code)

The C compares a stored callback with a named function in two places: `ftseakspecialn.c:59-60`
(`death2_cb / take_dmg_cb == ftSk_Init_80110198`) and `ftkirbyspecials.c:69` (`death2_cb != ftKb_Init_800EE74C`).
- Kirby: every store and the compare use `chars::cb::ftkirby__ftKb_Init_800EE74C`. Same pointer.
- Sheik: the port had two wrappers of `ftSk_Init_80110198`: `tu::ftseak::cb_ftSk_Init_80110198` (stored by the
  ftSk_SpecialN_* paths and ftseakspecials.rs set_chain_damage_callbacks) and `chars::cb::ftseak__ftSk_Init_80110198`
  (stored by ftSk_SpecialS_80111440 / 801114E4 / 80111D54), and the compare (ftSk_SpecialN_80111FBC) names the first. Their addresses differ in the
  shipping binary (separate codegen units: MergeFunctions does not fold them), so after the SpecialS stores the C's
  test was true and the port's false. Fixed: the chars::cb name re-exports the tu fn.
- Open: melee_engine/src/mex_bridge.rs:487 hands mods a third wrapper (a local `fn f`) for "cb_ftSk_Init_80110198";
  a mod storing it as death2_cb would miss the same compare.
- Port-only compares (not in the C): geno_lab_mode.rs:353 (`phys_cb == fly_phys`), geno_tests.rs:4376 / 4403.
