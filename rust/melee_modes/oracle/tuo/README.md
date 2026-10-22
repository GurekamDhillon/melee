# TU state oracle

Verifies a ported game TU (`src/melee/**/<tu>.c` -> `melee_modes/src/tu/<tu>.rs`) against the C
compiled natively, for functions that call other TUs and mutate game state (the kind the
pure-function oracles in `../` cannot cover).

    bash rust/melee_modes/oracle/tuo/build.sh <tu stem>       # e.g. gmpause; from the worktree root

Needs LLVM at `C:\Program Files\LLVM`, Windows Python with `libclang==18.1.1`, the usual
`../../melee/build/GALE01/include` font includes. It writes `tests/data/tuo_<stem>.{cfg,txt}` and
`tests/tuo_gen/<stem>_gen.rs`; `tests/tuo_<stem>.rs` replays the cfg against the Rust port.

How it works

* `gen_tuo.py <spec> <outdir>` reads `specs/<stem>.py` (C file, cases, value domains, pointer
  links) and libclang-parses the TU. `unit.c` `#include`s the TU verbatim, compiled with the
  shipping flags (`-ffp-contract=on`, gwtool's IR rewrites via `melee_runtime/oracle/lower.py`).
  Guest-memory reads of OS globals (`*(u32*)0x800000F8`) are redirected to plain variables
  (`guest_patch.py`). Every function the TU calls but does not define is a logging stub
  (`stubs.c`) returning generated values.
* The synthetic game state is built from a seeded generator (splitmix64) by visitors that walk
  the C types leaf by leaf (`tuo_rt.c`); the Rust twin (`tests/tuo_common/mod.rs` +
  the generated visitors) walks the same leaves in the same order through the `melee_types`
  accessors. Bitfields therefore compare by value although melee_types stores them MSB-first.
* Per case: fill globals and arguments, call the function, then dump return value, call log
  (callee name + arguments, strings by content, pointers by object label + offset), globals and
  every argument/pool object. The case line is `<fn> <idx> <fnv64 of that whole log>`; the Rust
  side must produce the identical hash. `TUO_FULL=<fn>[:<idx>]`, `TUO_ONLY=<fn>:<idx>` and
  `TUO_TRACE=1` (Rust test) and `oracle.exe tuo.cfg --full <fn> <idx>` (C) print the logs to diff.
* The port's `mod ext` (callee hooks `H_<name>`, extern globals) is generated too
  (`<outdir>/ext.rs.txt`); an unset hook panics `TODO: <name> is not ported`, or forwards to an
  existing port (`PORTED` in the spec).

Spec keys: `TU`, `RUST_MOD`, `CASES {fn: {n, args}}`, `DOM {path-suffix: [values]}`,
`LINKS {path-suffix: null|&global|new:T[:n]|maybe:T|buf:N|cell:bits|ptrs:T:n}`, `LINK_GATE`,
`PRUNE {type: [allowed field trails]}`, `ARG_EXPRS`, `OUTS`/`OUT_INTS` (out-parameters),
`VARIADIC`, `RET_FIELD`, `PTR_BYTES`, `GUEST_VARS`, `RUST_TYPES`, `RUST_GLOBAL_TYPES`, `PORTED`,
`SKIP_LEAF_TYPES`, `SIZE_OK`.

Known gaps (by design): records `melee_types` does not generate (e.g. `SoundTestMenuData`) are
compared as bytes / passed as `c_void`; `HSD_IDKey` is `u32` in melee_types but `uintptr_t`
natively (group-0 mismatch), so it is skipped and `HSD_JObj`'s size differs (`SIZE_OK`).
